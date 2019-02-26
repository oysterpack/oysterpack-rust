/*
 * Copyright 2019 OysterPack Inc.
 *
 *    Licensed under the Apache License, Version 2.0 (the "License");
 *    you may not use this file except in compliance with the License.
 *    You may obtain a copy of the License at
 *
 *        http://www.apache.org/licenses/LICENSE-2.0
 *
 *    Unless required by applicable law or agreed to in writing, software
 *    distributed under the License is distributed on an "AS IS" BASIS,
 *    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *    See the License for the specific language governing permissions and
 *    limitations under the License.
 */

use cucumber_rust::*;

use futures::{channel::oneshot, prelude::*, task::SpawnExt};
use maplit::*;
use oysterpack_trust::{
    concurrent::execution::{self, *},
    metrics,
};
use std::{collections::HashSet, num::NonZeroUsize, panic, thread, time::Duration};

steps!(World => {
    // Feature: [01D3YVY445KA4YF5KYMHHQK2TP] Executors are configured to catch unwinding panics for spawned futures

    // Scenario: [01D3YW91CYQRB0XVAKF580WX04] Spawning tasks after spawning tasks that panic on the global executor
    given regex "01D3YW91CYQRB0XVAKF580WX04" | _world, _matches, _step | {
        let mut executor = global_executor();
        let task_count = executor.thread_pool_size()*2;
        for _ in 0..task_count {
            executor.spawn(async { panic!("Boom!!!"); });
        }
        // wait for tasks to complete
        let panic_count = executor.task_panic_count() + task_count as u64;
        while executor.task_panic_count() < panic_count {
            thread::yield_now();
        }
    };

    when regex "01D3YW91CYQRB0XVAKF580WX04" | world, _matches, _step | {
        let (mut tx, rx) = oneshot::channel();
        let mut executor = global_executor();
        executor.spawn( async move {
            tx.send(()).unwrap();
        });
        world.rx = Some(rx);
    };

    then regex "01D3YW91CYQRB0XVAKF580WX04" | world, _matches, _step | {
        let mut executor = global_executor();
        let rx = world.rx.take().unwrap();
        executor.run( async {
            let result = await!(rx);
            println!("received message: {:?}", result);
            result
        }).unwrap();
    };

    // Feature: [01D3W3G8A7H32MVG3WYBER6J13] Spawned tasks are tracked via metrics

    // Scenario: [01D3Y1D8SJZ8JWPGJKFK4BYHP0] Spawning tasks
    when regex "01D3Y1D8SJZ8JWPGJKFK4BYHP0" | world, _matches, _step | {
        let mut executor = ExecutorBuilder::new(ExecutorId::generate()).register().unwrap();
        for _ in 0..5 {
            executor.spawn(async {});
        }
        for _ in 0..3 {
            executor.spawn(async { panic!("Boom!!!"); });
        }
        // wait for tasks to complete
        while executor.task_active_count() > 0 {
            thread::yield_now();
        }
        world.executor = Some(executor);
    };

    then regex "01D3Y1D8SJZ8JWPGJKFK4BYHP0-1" | world, _matches, _step | {
        for executor in world.executor.as_ref() {
            assert_eq!(executor.task_spawned_count(), 8);
        }
    };

    then regex "01D3Y1D8SJZ8JWPGJKFK4BYHP0-2" | world, _matches, _step | {
        for executor in world.executor.as_ref() {
            assert_eq!(executor.task_completed_count(), 8);
        }
    };
    then regex "01D3Y1D8SJZ8JWPGJKFK4BYHP0-3" | world, _matches, _step | {
        for executor in world.executor.as_ref() {
            assert_eq!(executor.task_active_count(), 0);
        }
    };

    then regex "01D3Y1D8SJZ8JWPGJKFK4BYHP0-4" | world, _matches, _step | {
        for executor in world.executor.as_ref() {
            assert_eq!(executor.task_panic_count(), 3);
        }
    };

    then regex "01D3Y1D8SJZ8JWPGJKFK4BYHP0-5" | world, _matches, _step | {
        for executor in world.executor.as_ref() {
           let mfs = executor.collect_metrics();
           println!("{:#?}", mfs);
           assert!(mfs.iter().any(|mf| {
                if mf.get_name() == TASK_COMPLETED_COUNTER_METRIC_ID.name().as_str() {
                    let count = mf.get_metric().iter().next().unwrap().get_counter().get_value() as u64;
                    count == executor.task_completed_count()
                } else {
                    false
                }
           }));
           assert!(mfs.iter().any(|mf| {
                if mf.get_name() == TASK_SPAWNED_COUNTER_METRIC_ID.name().as_str() {
                    let count = mf.get_metric().iter().next().unwrap().get_counter().get_value() as u64;
                    count == executor.task_spawned_count()
                } else {
                    false
                }
           }));
           assert!(mfs.iter().any(|mf| {
                if mf.get_name() == TASK_PANIC_COUNTER_METRIC_ID.name().as_str() {
                    let count = mf.get_metric().iter().next().unwrap().get_counter().get_value() as u64;
                    count == executor.task_panic_count()
                } else {
                    false
                }
           }));
           assert!(mfs.iter().any(|mf| {
                if mf.get_name() == THREADS_POOL_SIZE_GAUGE_METRIC_ID.name().as_str() {
                    let count = mf.get_metric().iter().next().unwrap().get_gauge().get_value() as u64;
                    count == executor.thread_pool_size() as u64
                } else {
                    false
                }
           }));
        }
    };

    // Feature: [01D3W2RTE80P64E1W1TD61KGBN] A global Executor will be automatically provided by the Executor registry

    // Scenario: [01D3W2RF94W85YGQ49JFDXB3XB] Use the global Executor from 10 different threads
    then regex "01D3W2RF94W85YGQ49JFDXB3XB" | world, _matches, _step | {
        let executor = global_executor();
        let completed_count = executor.task_completed_count() + 100;

        for _ in 0..10 {
            thread::spawn(|| {
                for _ in 0..10 {
                    let mut executor = global_executor();
                    executor.spawn(async {});
                }
            });
        }

        while executor.task_completed_count() < completed_count {
            thread::sleep(Duration::from_millis(1));
            println!("waiting for tasks to complete ...");
        }
        assert_eq!(completed_count, executor.task_completed_count());
    };

    // Scenario: [01D4P2Z3JWR05CND2N96TMBKT2] Use the global Executor from 10 different threads
    then regex "01D4P2Z3JWR05CND2N96TMBKT2" | world, _matches, _step | {
        let id = ExecutorId::generate();
        let executor = ExecutorBuilder::new(id).register().unwrap();
        let completed_count = executor.task_completed_count() + 100;

        for _ in 0..10 {
            thread::spawn(move || {
                for _ in 0..10 {
                    let mut executor = execution::executor(id).unwrap();
                    executor.spawn(async {});
                }
            });
        }

        while executor.task_completed_count() < completed_count {
            thread::sleep(Duration::from_millis(1));
            println!("waiting for tasks to complete ...");
        }
        assert_eq!(completed_count, executor.task_completed_count());
    };
});

#[derive(Default)]
pub struct World {
    rx: Option<oneshot::Receiver<()>>,
    executor: Option<Executor>,
}
