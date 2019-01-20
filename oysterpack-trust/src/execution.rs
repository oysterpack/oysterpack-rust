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

//! Exposes lower level primitives for dealing with asynchronous execution

use failure::Fail;
use futures::{
    executor::{ThreadPool, ThreadPoolBuilder},
    future::{Future, FutureObj},
    task::{Spawn, SpawnError},
};
use lazy_static::lazy_static;
use oysterpack_log::*;
use oysterpack_uid::macros::ulid;
use serde::{Deserialize, Serialize};
use std::{
    io,
    iter::ExactSizeIterator,
    sync::{Arc, Mutex},
};
use std::panic::{catch_unwind, AssertUnwindSafe};

lazy_static! {
    /// Global Executor registry
    pub static ref EXECUTORS: Executors = Executors::default();
}

/// Provides a ThreadPool
#[derive(Debug)]
pub struct Executors {
    global_executor: Executor,
    thread_pools: fnv::FnvHashMap<ExecutorId, Executor>,
}

impl Executors {
    /// An executor can only be registered once, and once it is registered, it stays registered for
    /// life of the app.
    /// - returns false is an excutor with the same ID is already registered
    pub fn register(
        &mut self,
        id: ExecutorId,
        builder: &mut ThreadPoolBuilder,
    ) -> Result<(), Error> {
        if self.thread_pools.contains_key(&id) {
            return Err(Error::ExecutorAlreadyRegistered(id));
        }
        self.thread_pools.insert(id, Executor::new(id, builder)?);
        Ok(())
    }

    /// Returns the registered executor IDs
    pub fn executor_ids(&self) -> smallvec::SmallVec<[ExecutorId; 16]> {
        let keys = self.thread_pools.keys();
        let mut ids = smallvec::SmallVec::with_capacity(keys.len());
        for key in keys {
            ids.push(*key);
        }
        ids
    }

    /// returns the Executor for the specified ID
    pub fn executor(&self, id: ExecutorId) -> Option<Executor> {
        match self.thread_pools.get(&id) {
            Some(executor) => Some(executor.clone()),
            None => {
                if id == Executor::DEFAULT_EXECUTOR_ID {
                    return Some(self.global_executor.clone());
                }
                None
            }
        }
    }

    /// Returns the global executor, which is provided by default.
    pub fn global_executor(&self) -> Executor {
        self.global_executor.clone()
    }
}

impl Default for Executors {
    fn default() -> Self {
        fn default_executor() -> Executor {
            let mut builder = ThreadPool::builder();
            builder
                .name_prefix(format!("{}-", Executor::DEFAULT_EXECUTOR_ID))
                .after_start(|thread_index| {
                    debug!(
                        "Executer thread has started: {}-{}",
                        Executor::DEFAULT_EXECUTOR_ID,
                        thread_index
                    )
                })
                .before_stop(|thread_index| {
                    debug!(
                        "Executer thread is stopping: {}-{}",
                        Executor::DEFAULT_EXECUTOR_ID,
                        thread_index
                    )
                });
            Executor::new(Executor::DEFAULT_EXECUTOR_ID, &mut builder).unwrap()
        }

        Self {
            global_executor: default_executor(),
            thread_pools: fnv::FnvHashMap::default(),
        }
    }
}

/// Provides a threadsafe ThreadPool based executor
#[derive(Debug, Clone)]
pub struct Executor {
    id: ExecutorId,
    thread_pool: Arc<Mutex<ThreadPool>>,
}

impl Executor {
    /// Default ExecutorId
    pub const DEFAULT_EXECUTOR_ID: ExecutorId = ExecutorId(1871427164235073850597045237139528853);

    /// constructor
    pub fn new(id: ExecutorId, builder: &mut ThreadPoolBuilder) -> Result<Self, Error> {
        Ok(Self {
            id,
            thread_pool: Arc::new(Mutex::new(
                builder
                    .create()
                    .map_err(|err| Error::ThreadPoolCreateFailed(err))?,
            )),
        })
    }

    /// Returns the ExecutorId
    pub const fn id(&self) -> ExecutorId {
        self.id
    }

    /// Runs the given future with this thread pool as the default spawner for spawning tasks.
    ///
    /// ## Notes
    /// - This function will block the calling thread until the given future is complete.
    /// - The function will return when the provided future completes, even if some of the
    ///   tasks it spawned are still running.
    ///
    /// ## Panics
    /// If the ThreadPool Mutex cannot be locked.
    pub fn run<F: Future>(&mut self, f: F) -> F::Output {
        let result = {
            let mut thread_pool = self.thread_pool.lock().unwrap();
            catch_unwind(AssertUnwindSafe(|| thread_pool.run(f)))
        };
        match result {
            Ok(res) => res,
            Err(err) => panic!(err)
        }
    }
}

impl Spawn for Executor {
    fn spawn_obj(&mut self, future: FutureObj<'static, ()>) -> Result<(), SpawnError> {
        let mut thread_pool = self.thread_pool.lock().unwrap();
        thread_pool.spawn_obj(future)
    }

    fn status(&self) -> Result<(), SpawnError> {
        let thread_pool = self.thread_pool.lock().unwrap();
        thread_pool.status()
    }
}

#[ulid]
/// Unique Executor ID
pub struct ExecutorId(pub u128);

/// module related errors
#[derive(Fail, Debug)]
pub enum Error {
    /// When a ThreadPool creation failure occurs.
    #[fail(display = "Failed to create ThreadPool: {}", _0)]
    ThreadPoolCreateFailed(io::Error),
    /// When trying to register an Executor using an ID that is already registered.
    #[fail(display = "Executor is already registered: {}", _0)]
    ExecutorAlreadyRegistered(ExecutorId),
}

#[allow(warnings)]
#[cfg(test)]
mod tests {
    use super::*;
    use futures::task::SpawnExt;
    use std::thread;

    fn log_config() -> oysterpack_log::LogConfig {
        oysterpack_log::config::LogConfigBuilder::new(oysterpack_log::Level::Info)
            .target_level(
                oysterpack_log::Target::from(env!("CARGO_PKG_NAME")),
                Level::Error,
            )
            .build()
    }

    #[test]
    fn global_executor() {
        oysterpack_log::init(log_config(), oysterpack_log::StderrLogger);

        let mut executor = EXECUTORS.global_executor();
        executor.spawn(async { info!("task #1") });
        let task_handle = executor
            .spawn_with_handle(async { info!("spawned task says hello") })
            .unwrap();
        executor.run(task_handle);
    }

    // the panic is bubbled up to the current thread when awaiting on a task that panics
    #[test]
    #[should_panic]
    fn run_spawned_panic_task() {
        oysterpack_log::init(log_config(), oysterpack_log::StderrLogger);

        let mut executor = EXECUTORS.global_executor();
        let task_handle = executor
            .spawn_with_handle(async { panic!("BOOM!!") })
            .unwrap();
        executor.run(task_handle);
    }

    // the panic is bubbled up to the current thread when awaiting on a task that panics
    #[test]
    #[should_panic]
    fn await_spawned_panic_task() {
        oysterpack_log::init(log_config(), oysterpack_log::StderrLogger);

        let mut executor = EXECUTORS.global_executor();
        let task_handle = executor
            .spawn_with_handle(async { panic!("BOOM!!") })
            .unwrap();
        executor.run(async {
            await!(task_handle);
        });
    }

    // threads that panic in the pool do get replaced
    #[test]
    fn spawned_task_panics_all_threads() {
        oysterpack_log::init(log_config(), oysterpack_log::StderrLogger);

        let mut executor = EXECUTORS.global_executor();
        let panic_task_count = num_cpus::get() * 2;
        let mut handles = vec![];
        for i in 0..panic_task_count {
            let handle = executor.spawn_with_handle(async move {
                panic!("BOOM({})!", i);
            });
            handles.push(handle.unwrap());
        }

        thread::sleep_ms(10);

        executor.run(
            async move {
                info!("this should hang ...");
            },
        );
    }
}
