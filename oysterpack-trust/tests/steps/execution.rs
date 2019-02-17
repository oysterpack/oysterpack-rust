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

use futures::task::SpawnExt;
use oysterpack_trust::concurrent::execution::{self, *};
use std::{num::NonZeroUsize, thread};

steps!(TestContext => {

    given regex "01D3W3GDYVS4P2SR0SECVT0JJT-1" |world, _matches, _step| {
        world.init();
    };

    when regex "01D3W3GDYVS4P2SR0SECVT0JJT-2" |world, _matches, _step| {
        spawn_await_tasks(world, 0, 1);
    };

    then regex "01D3W3GDYVS4P2SR0SECVT0JJT-3" |world, _matches, _step| {
        check_thread_pool_size_unchanged(world);
    };

    then regex "01D3W3GDYVS4P2SR0SECVT0JJT-4" |world, _matches, _step| {
        check_threads_started_inc(world);
    };

    then regex "01D3W3GDYVS4P2SR0SECVT0JJT-5" |world, _matches, _step| {
        check_spawned_task_count(world, 1);
        check_completed_task_count(world, 0);
    };

    given regex "01D3Y1CYCKZHY675FKEPPX4JE4-1" |world, _matches, _step| {
        world.init_with_new_executor(1);
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
        world.init_with_new_executor(1);
    };

    when regex "01D3Y1D8SJZ8JWPGJKFK4BYHP0-2" |world, _matches, _step| {
        spawn_tasks(world, 5, 5);
        await_tasks_completed_while_gt(world, 5);
    };

    then regex "01D3Y1D8SJZ8JWPGJKFK4BYHP0-3" |world, _matches, _step| {
        check_spawned_task_count(world, 10);
    };

    then regex "01D3Y1D8SJZ8JWPGJKFK4BYHP0-4" |world, _matches, _step| {
        check_completed_task_count(world, 5);
    };

    then regex "01D3Y1D8SJZ8JWPGJKFK4BYHP0-5" |world, _matches, _step| {
        check_active_task_count(world, 5);
    };

});

fn spawn_tasks(world: &mut TestContext, success_count: u64, panic_count: u64) {
    for _ in 0..success_count {
        world.executor.spawn(async {}).unwrap();
    }

    for i in 0..panic_count {
        world
            .executor
            .spawn(async move { panic!("BOOM #{} !!!", i) })
            .unwrap();
    }
}

fn spawn_await_tasks(world: &mut TestContext, success_count: u64, panic_count: u64) {
    for i in 0..success_count {
        println!("spawn_await_tasks() - SPAWNED #{}", i);
        world.executor.spawn_await(async {}).unwrap();
        println!("spawn_await_tasks() - DONE #{}", i);
    }

    for i in 0..panic_count {
        println!("spawn_await_tasks() - SPAWNED PANIC #{}", i);
        let _ = world
            .executor
            .spawn_await(async move { panic!("BOOM #{} !!!", i) });
        println!("spawn_await_tasks() - DONE PANIC #{}", i);
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

fn await_tasks_completed(world: &mut TestContext) {
    while world.executor.active_task_count() > 0 {
        println!(
            "await_tasks_completed(): {}",
            world.executor.active_task_count()
        );
        thread::yield_now();
    }
}

fn await_tasks_completed_while_gt(world: &mut TestContext, count: u64) {
    while world.executor.active_task_count() > count {
        println!(
            "await_tasks_completed_while_gt(): {}",
            world.executor.active_task_count()
        );
        thread::yield_now();
    }
}

fn check_threads_started_inc(world: &mut TestContext) {
    println!(
        "total_threads_started = {}",
        execution::total_threads_started()
    );
    assert_eq!(
        execution::total_threads_started(),
        world.total_threads_started
    );
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
    pub executor_thread_pool_size: u64,
    pub total_threads_started: u64,
}

impl TestContext {
    pub fn init(&mut self) {
        self.executor = execution::global_executor();
        self.gather_metrics();
    }

    pub fn init_with_new_executor(&mut self, thread_pool_size: usize) {
        self.executor = ExecutorBuilder::new(ExecutorId::generate())
            .set_pool_size(NonZeroUsize::new(thread_pool_size).unwrap())
            .register()
            .unwrap();
        self.gather_metrics();
    }

    pub fn gather_metrics(&mut self) {
        self.executor_spawned_task_count = self.executor.spawned_task_count();
        self.executor_completed_task_count = self.executor.completed_task_count();
        self.executor_thread_pool_size = self.executor.thread_pool_size();
        self.total_threads_started = total_threads_started();
    }
}

impl Default for TestContext {
    fn default() -> Self {
        Self {
            executor: execution::global_executor(),
            executor_spawned_task_count: 0,
            executor_completed_task_count: 0,
            executor_thread_pool_size: 0,
            total_threads_started: 0,
        }
    }
}
