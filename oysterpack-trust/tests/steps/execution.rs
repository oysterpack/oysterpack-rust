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

use futures::{prelude::*, task::SpawnExt};
use maplit::*;
use oysterpack_trust::{
    concurrent::execution::{self, *},
    metrics,
};
use std::{collections::HashSet, num::NonZeroUsize, panic, thread, time::Duration};

pub mod executor;
pub mod registry;

steps!(TestContext => {

    given regex "01D3Y1CYCKZHY675FKEPPX4JE4-1" |world, _matches, _step| {
        world.init_with_new_executor(1, false);
    };

    when regex "01D3Y1CYCKZHY675FKEPPX4JE4-2" |world, _matches, _step| {
        spawn_tasks(world, 10, 0);
        await_tasks_completed(world);
    };

    then regex "01D3Y1CYCKZHY675FKEPPX4JE4-3" |world, _matches, _step| {
        check_spawned_task_count(world, 10);
    };

    then regex "01D3Y1CYCKZHY675FKEPPX4JE4-4" |world, _matches, _step| {
        check_completed_task_count(world, 10);
    };

    then regex "01D3Y1CYCKZHY675FKEPPX4JE4-5" |world, _matches, _step| {
        check_active_task_count(world, 0);
    };

    given regex "01D3Y1D8SJZ8JWPGJKFK4BYHP0-1" |world, _matches, _step| {
        world.init_with_new_executor(1, true);
    };

    when regex "01D3Y1D8SJZ8JWPGJKFK4BYHP0-2" |world, _matches, _step| {
        spawn_tasks(world, 5, 5);
        await_tasks_completed_while_gt(world, 0);
    };

    then regex "01D3Y1D8SJZ8JWPGJKFK4BYHP0-3" |world, _matches, _step| {
        check_spawned_task_count(world, 10);
    };

    then regex "01D3Y1D8SJZ8JWPGJKFK4BYHP0-4" |world, _matches, _step| {
        check_completed_task_count(world, 10);
    };

    then regex "01D3Y1D8SJZ8JWPGJKFK4BYHP0-5" |world, _matches, _step| {
        check_active_task_count(world, 0);
    };

    then regex "01D3Y1D8SJZ8JWPGJKFK4BYHP0-6" |world, _matches, _step| {
        check_panicked_task_count(world, 5);
    };

    given regex "01D3YW91CYQRB0XVAKF580WX04-1" |world, _matches, _step| {
        world.init();
        spawn_tasks(world, 0, num_cpus::get() * 2);
        await_tasks_completed_while_gt(world, 0);
    };

    when regex "01D3YW91CYQRB0XVAKF580WX04-2" |world, _matches, _step| {
        spawn_tasks(world, 10, 0);
    };

    then regex "01D3YW91CYQRB0XVAKF580WX04-3" |world, _matches, _step| {
        await_tasks_completed_while_gt(world, 0);
        check_completed_task_count(world, (10 + (num_cpus::get() * 2)) as u64);
    };

    then regex "01D3YW91CYQRB0XVAKF580WX04-4" |world, _matches, _step| {
        check_panicked_task_count(world, (num_cpus::get() * 2) as u64);
    };

    when regex "01D3W0MDTMRJ6GNFCQCPTS55HG-1" |world, _matches, _step| {
        world.init_with_executor_builder(ExecutorBuilder::new(ExecutorId::generate()));
    };

    then regex "01D3W0MDTMRJ6GNFCQCPTS55HG-2" |world, _matches, _step| {
        check_exeutor_thread_pool_size(world, num_cpus::get());
    };


    when regex "01D40G5CFDP2RS7V75WJQCSME4-1" |world, _matches, _step| {
        world.init_with_executor_builder(ExecutorBuilder::new(ExecutorId::generate())
            .set_pool_size(NonZeroUsize::new(20).unwrap())
        );
    };

    then regex "01D40G5CFDP2RS7V75WJQCSME4-2" |world, _matches, _step| {
        check_exeutor_thread_pool_size(world, 20);
    };

    when regex "01D40G6X1ABZK6532CVE00EWHW-1" |world, _matches, _step| {
        world.init_with_executor_builder(ExecutorBuilder::new(ExecutorId::generate())
            .set_stack_size(NonZeroUsize::new(1024*64).unwrap())
        );
    };

    then regex "01D40G6X1ABZK6532CVE00EWHW-2" |world, _matches, _step| {
        assert_eq!(world.executor.stack_size().unwrap(), 1024*64);
    };

    when regex "01D40G78JNHX519WEP1A1E5FVT-1" |world, _matches, _step| {
        world.init_with_executor_builder(ExecutorBuilder::new(ExecutorId::generate()));
    };

    then regex "01D40G78JNHX519WEP1A1E5FVT-2" |world, _matches, _step| {
    };

    when regex "01D40G7FQDMWEVGSGFH96KQMZ0-1" |world, _matches, _step| {
        world.init();
    };

    then regex "01D40G7FQDMWEVGSGFH96KQMZ0-2" |world, _matches, _step| {
        match ExecutorBuilder::new(world.executor.id()).register() {
            Err(ExecutorRegistryError::ExecutorAlreadyRegistered(_)) => println!("failed to register Executor because ExecutorAlreadyRegistered"),
            other => panic!("unexpected result: {:?}", other)
        }
    };

    when regex "01D40WTESDPHA8BZVM2VS7VRK2-1" |world, _matches, _step| {
        world.init_with_executor_builder(ExecutorBuilder::new(ExecutorId::generate()));
    };

    then regex "01D40WTESDPHA8BZVM2VS7VRK2-2" |world, _matches, _step| {
        match ExecutorBuilder::new(world.executor.id()).register() {
            Err(ExecutorRegistryError::ExecutorAlreadyRegistered(_)) => println!("failed to register Executor because ExecutorAlreadyRegistered"),
            other => panic!("unexpected result: {:?}", other)
        }
    };

    when regex "01D3W1NYG4YT4MM5HDR4YWT7ZD-1" |world, _matches, _step| {
        world.executor_ids.extend(vec![ExecutorId::generate(), ExecutorId::generate()]);
        world.executor_ids.iter().for_each(|id| {
            ExecutorBuilder::new(*id).register().unwrap();
        });
    };

    then regex "01D3W1NYG4YT4MM5HDR4YWT7ZD-2" |world, _matches, _step| {
        let executor_ids = execution::executor_ids();
        assert!(world.executor_ids.iter().all(|id1| executor_ids.iter().any(|id2| *id1 == *id2)));
    };

    given regex "01D3W2RF94W85YGQ49JFDXB3XB-1" |world, _matches, _step| {
        world.init();
    };

    when regex "01D3W2RF94W85YGQ49JFDXB3XB-2" |world, _matches, _step| {
        let handles: Vec<_> = (0..2).map(|_| {
            thread::spawn(|| {
                (0..10).for_each(|_| {
                    execution::global_executor().spawn(async{}).unwrap();
                });
            })
        }).collect();

        handles.into_iter().for_each(|handle| {
            handle.join().unwrap();
            thread::sleep(Duration::from_millis(10));
        });
    };

    then regex "01D3W2RF94W85YGQ49JFDXB3XB-3" |world, _matches, _step| {
        await_tasks_completed(world);
        check_completed_task_count(world, 20);
    };

    given regex "01D41EX3HY16EH06RVHAHE2Q0F-1" |world, _matches, _step| {
        world.init_with_executor_builder(ExecutorBuilder::new(ExecutorId::generate()));
    };

    when regex "01D41EX3HY16EH06RVHAHE2Q0F-2" |world, _matches, _step| {
        spawn_tasks(world, 5, 5);
    };

    then regex "01D41EX3HY16EH06RVHAHE2Q0F-3" |world, _matches, _step| {
        await_tasks_completed(world);
        check_completed_task_count(world, 10);
        check_metrics_against_executor(world);
    };

});

fn check_metrics_against_executor(world: &mut TestContext) {
    let mfs = execution::gather_metrics();
    let executor_id = world.executor.id().to_string();
    let executor_id = executor_id.as_str();
    let mfs: Vec<_> = mfs
        .iter()
        .filter(|mf| {
            mf.get_metric().iter().any(|metric| {
                metric
                    .get_label()
                    .iter()
                    .any(|label_pair| label_pair.get_value() == executor_id)
            })
        })
        .collect();
    assert_eq!(mfs.len(), 4);
    let metric_names: HashSet<_> = mfs.iter().map(|mf| mf.get_name().to_string()).collect();
    let descs = metrics::registry().find_descs(|desc| metric_names.contains(&desc.fq_name));
    assert_eq!(descs.len(), 4);
    let labels = hashmap! {
        execution::EXECUTOR_ID_LABEL_ID.name() => executor_id.to_string()
    };
    let metric_families = metrics::registry().gather_for_labels(&labels);

    println!("ExecutorId: {}", executor_id);
    println!("count = {}\n{:#?}", metric_families.len(), metric_families);
    assert!(
        metric_families.iter().all(|mf| mf.get_metric().len() == 1),
        "Each MetricFamily should return the metric for the specified Executor"
    );
}

fn run_tasks(
    world: &mut TestContext,
    success_count: usize,
    panic_count: usize,
    catch_unwind: bool,
) {
    for _ in 0..success_count {
        world.executor.run(async {});
    }

    for i in 0..panic_count {
        let mut executor = world.executor.clone();
        let future = async move { panic!("BOOM #{} !!!", i) };
        if catch_unwind {
            if executor.run(future.catch_unwind()).is_err() {
                eprintln!("run_tasks(): task panicked");
            }
        } else {
            if panic::catch_unwind(move || executor.run(future)).is_err() {
                eprintln!("run_tasks(): task panicked");
            }
        }
    }
}

fn spawn_tasks(world: &mut TestContext, success_count: usize, panic_count: usize) {
    for _ in 0..success_count {
        world.executor.spawn(async {}).unwrap();
    }

    for i in 0..panic_count {
        let future = async move { panic!("BOOM #{} !!!", i) };
        world.executor.spawn(future).unwrap();
    }
}

fn check_completed_task_count(world: &mut TestContext, expected_inc: u64) {
    assert_eq!(
        world.executor.completed_task_count(),
        world.executor_completed_task_count + expected_inc,
        "check_completed_task_count failed"
    );
}

fn check_spawned_task_count(world: &mut TestContext, expected_inc: u64) {
    assert_eq!(
        world.executor.spawned_task_count(),
        world.executor_spawned_task_count + expected_inc,
        "check_spawned_task_count failed"
    );
}

fn check_active_task_count(world: &mut TestContext, expected: u64) {
    assert_eq!(
        world.executor.active_task_count(),
        expected,
        "check_active_task_count failed"
    );
}

fn check_panicked_task_count(world: &mut TestContext, expected_inc: u64) {}

fn await_tasks_completed(world: &mut TestContext) {
    while world.executor.active_task_count() > 0 {
        println!(
            "await_tasks_completed(): {}",
            world.executor.active_task_count()
        );
        thread::sleep(Duration::from_millis(1));
    }
}

fn await_tasks_completed_while_gt(world: &mut TestContext, count: u64) {
    while world.executor.active_task_count() > count {
        println!(
            "await_tasks_completed_while_gt(): {}",
            world.executor.active_task_count()
        );
        thread::sleep(Duration::from_millis(1));
    }
}

fn check_threads_started_inc(world: &mut TestContext, expected_inc: usize) {
    println!("total_threads_started = {}", execution::total_threads());
    assert_eq!(
        execution::total_threads(),
        world.total_threads + expected_inc
    );
}

fn check_exeutor_thread_pool_size(world: &mut TestContext, expected_count: usize) {
    for _ in 0..10 {
        if world.executor.thread_pool_size() < expected_count {
            println!(
                "waiting for thread pool to initialize - size = {}",
                world.executor.thread_pool_size()
            );
            thread::sleep(Duration::from_millis(1));
        }
    }
    assert_eq!(world.executor.thread_pool_size(), expected_count);
}

fn check_total_threads_count_inc(world: &mut TestContext, expected_inc: usize) {
    let expected_count = world.total_threads + expected_inc;
    for _ in 0..10 {
        if total_threads() < expected_count {
            println!(
                "waiting for thread pool to initialize - size = {}",
                world.executor.thread_pool_size()
            );
            thread::sleep(Duration::from_millis(1));
        }
    }
    assert_eq!(total_threads(), expected_count);
}

fn check_thread_pool_size_unchanged(world: &mut TestContext) {
    assert_eq!(
        world.executor.thread_pool_size(),
        world.executor_thread_pool_size
    );
}

pub struct TestContext {
    pub executor: Executor,
    pub executor_spawned_task_count: u64,
    pub executor_completed_task_count: u64,
    pub executor_thread_pool_size: usize,
    pub executor_panicked_task_count: u64,
    pub total_threads: usize,
    pub executor_ids: Vec<ExecutorId>,
}

impl TestContext {
    pub fn init(&mut self) {
        self.executor_ids.clear();
        self.executor = execution::global_executor();
        self.gather_metrics();
        await_tasks_completed(self);
    }

    pub fn init_with_new_executor(&mut self, thread_pool_size: usize, catch_unwind: bool) {
        self.init_with_executor_builder(
            ExecutorBuilder::new(ExecutorId::generate())
                .set_pool_size(NonZeroUsize::new(thread_pool_size).unwrap()),
        );
    }

    pub fn init_with_executor_builder(&mut self, builder: ExecutorBuilder) {
        self.executor = builder.register().unwrap();
        self.gather_metrics();
    }

    pub fn gather_metrics(&mut self) {
        self.executor_spawned_task_count = self.executor.spawned_task_count();
        self.executor_completed_task_count = self.executor.completed_task_count();
        self.executor_thread_pool_size = self.executor.thread_pool_size();
        self.executor_panicked_task_count = self.executor.panicked_task_count();
        self.total_threads = total_threads();
    }
}

impl Default for TestContext {
    fn default() -> Self {
        Self {
            executor: execution::global_executor(),
            executor_spawned_task_count: 0,
            executor_completed_task_count: 0,
            executor_thread_pool_size: 0,
            total_threads: 0,
            executor_panicked_task_count: 0,
            executor_ids: Vec::new(),
        }
    }
}
