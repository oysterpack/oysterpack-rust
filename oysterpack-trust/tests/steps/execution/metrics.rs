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

use futures::{channel::oneshot, task::SpawnExt};
use oysterpack_trust::{
    concurrent::execution::{metrics::*, *},
    metrics,
};
use std::thread;

steps!(World => {

    // Feature: [01D3W3G8A7H32MVG3WYBER6J13] Spawned tasks are tracked via metrics

    // Scenario: [01D3Y1D8SJZ8JWPGJKFK4BYHP0] Spawning tasks
    when regex "01D3Y1D8SJZ8JWPGJKFK4BYHP0" | world, _matches, _step | {
        let mut executor = ExecutorBuilder::new(ExecutorId::generate()).register().unwrap();
        for _ in 0..5 {
            executor.spawn(async {}).unwrap();
        }
        for _ in 0..3 {
            executor.spawn(async { panic!("Boom!!!"); }).unwrap();
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
           let mfs = executor.gather_metrics();
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

    // Feature: [01D4P0Q8M3ZAWCDH22VXHGN4ZX] Executor metrics can be collected

    // Scenario: [01D4P0QFZ2YK0HYC74T9S74WXQ] Collect metrics for an individual Executor
    then regex "01D4P0QFZ2YK0HYC74T9S74WXQ" | _world, _matches, _step | {
        let executor = global_executor();
        let mfs = executor.gather_metrics();
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
    };

    // Scenario: [01D4P0TGP2D9H4GAXZC1PKMQH3] Collect metrics for all registered Executor(s)
    then regex "01D4P0TGP2D9H4GAXZC1PKMQH3" | _world, _matches, _step | {
        let mfs = gather_metrics();
        println!("{:#?}", mfs);
        assert_eq!(mfs.len(), 4);
        // add 1 for the global executor
        let executor_count = executor_ids().len() + 1;
        assert!(mfs.iter().all(|mf| mf.get_metric().len() == executor_count));
    };

    // Feature: [01D418RZF94XJCRQ5D2V4DRMJ6] Executor thread pool size is recorded as a metric

    // Scenario: [01D41GJ0WRB49AX2NX4T09BKA8] Verify total Executor threads match against the metric registry
    given regex "01D41GJ0WRB49AX2NX4T09BKA8" | _world, _matches, _step | {
        if executor_ids().is_empty() {
            ExecutorBuilder::new(ExecutorId::generate()).register().unwrap();
        }
    };

    // Scenario: [01D41GJ0WRB49AX2NX4T09BKA8] Verify total Executor threads match against the metric registry
    then regex "01D41GJ0WRB49AX2NX4T09BKA8" | _world, _matches, _step | {
        let thread_count = total_thread_count();
        let mfs = metrics::registry().gather_for_metric_ids(&[THREADS_POOL_SIZE_GAUGE_METRIC_ID]);
        let count: u64 = mfs.first()
            .unwrap()
            .get_metric().iter().map(|metric| metric.get_gauge().get_value() as u64)
            .sum();
        assert_eq!(thread_count as u64, count);
    };
});

#[derive(Default)]
pub struct World {
    rx: Option<oneshot::Receiver<()>>,
    executor: Option<Executor>,
}
