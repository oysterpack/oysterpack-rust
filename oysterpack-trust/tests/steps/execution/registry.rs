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

steps!(World => {
    // Feature: [01D3W0H2B7KNTBJTGDYP3CRB7K] A global Executor registry is provided.

    // Scenario: [01D3W0MDTMRJ6GNFCQCPTS55HG] Registering an Executor with default settings
    then regex "01D3W0MDTMRJ6GNFCQCPTS55HG-1" | world, _matches, _step | {
        let executor = ExecutorBuilder::new(ExecutorId::generate()).register().unwrap();
        wait_for_thread_pool_to_initialize(&executor, num_cpus::get());
        assert_eq!(executor.thread_pool_size(), num_cpus::get());
        assert!(executor.stack_size().is_none()); // which means the Rust default stack size is used
        world.executor = Some(executor);
    };

    then regex "01D3W0MDTMRJ6GNFCQCPTS55HG-2" | world, _matches, _step | {
        let executor = world.executor.take().unwrap();
        assert_eq!(executor.active_task_count(), 0);
        assert_eq!(executor.spawned_task_count(), 0);
        assert_eq!(executor.completed_task_count(), 0);
        assert_eq!(executor.panicked_task_count(), 0);
    };

    // Scenario: [01D40G5CFDP2RS7V75WJQCSME4] Registering an Executor configured with thread pool size = 20
    then regex "01D40G5CFDP2RS7V75WJQCSME4" | _world, _matches, _step | {
        let executor = ExecutorBuilder::new(ExecutorId::generate())
            .set_pool_size(NonZeroUsize::new(20).unwrap())
            .register().unwrap();
        wait_for_thread_pool_to_initialize(&executor, 20);
        assert_eq!(executor.thread_pool_size(), 20);
    };

    // Scenario: [01D40G6X1ABZK6532CVE00EWHW] Registering an Executor configured with a custom thread stack size
    then regex "01D40G6X1ABZK6532CVE00EWHW" | _world, _matches, _step | {
        const STACK_SIZE: usize = 1024*64;
        let executor = ExecutorBuilder::new(ExecutorId::generate())
            .set_stack_size(NonZeroUsize::new(STACK_SIZE).unwrap())
            .register().unwrap();
        assert_eq!(executor.stack_size(), Some(STACK_SIZE));
    };

    // Scenario: [01D40G7FQDMWEVGSGFH96KQMZ0] Registering an Executor using the global ExecutorId that is already in use
    then regex "01D40G7FQDMWEVGSGFH96KQMZ0" | _world, _matches, _step | {
        match ExecutorBuilder::new(global_executor().id()).register() {
            Err(err) => println!("{}", err),
            Ok(_) => panic!("should have failed to register")
        }
    };

    // Scenario: [01D40WTESDPHA8BZVM2VS7VRK2] Registering an Executor using an ExecutorId that is already in use
    then regex "01D40WTESDPHA8BZVM2VS7VRK2" | _world, _matches, _step | {
        let executor = ExecutorBuilder::new(ExecutorId::generate()).register().unwrap();
        match ExecutorBuilder::new(executor.id()).register() {
            Err(err) => println!("{}", err),
            Ok(_) => panic!("should have failed to register")
        }
    };

    // Feature: [01D3W1C9YZDYMDPT98JCFS8F4P] The list of registered ExecutorId(s) can be retrieved from the Executor registry

    // Scenario: [01D3W1NYG4YT4MM5HDR4YWT7ZD] Registering Executor(s)
    then regex "01D3W1NYG4YT4MM5HDR4YWT7ZD" | _world, _matches, _step | {
        let mut executor_ids = Vec::with_capacity(3);
        for _ in 0..3 {
            executor_ids.push(ExecutorBuilder::new(ExecutorId::generate()).register().unwrap().id());
        }
        let registry_executor_ids = execution::executor_ids();
        assert!(executor_ids.iter().all(|id| registry_executor_ids.iter().any(|id2| id2 == id)));
    };
});

fn wait_for_thread_pool_to_initialize(executor: &Executor, pool_size: usize) {
    loop {
        if executor.thread_pool_size() == pool_size {
            break;
        }
        thread::yield_now();
        println!(
            "executor.thread_pool_size() = {}",
            executor.thread_pool_size()
        );
    }
}

#[derive(Clone, Default)]
pub struct World {
    executor: Option<Executor>,
}
