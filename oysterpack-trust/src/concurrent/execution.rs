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

//! Exposes lower level primitives for dealing with asynchronous execution:
//! - async executors
//!

use crate::metrics;
use failure::Fail;
use futures::{
    channel,
    executor::{ThreadPool, ThreadPoolBuilder},
    future::{Future, FutureObj},
    task::{Spawn, SpawnError, SpawnExt},
};
use lazy_static::lazy_static;
use oysterpack_log::*;
use oysterpack_uid::macros::ulid;
use serde::{Deserialize, Serialize};
use std::{
    fmt, io,
    iter::ExactSizeIterator,
    num::NonZeroUsize,
    panic::{catch_unwind, AssertUnwindSafe},
    sync::RwLock,
};

lazy_static! {
    /// Global Executor registry
    static ref EXECUTORS: RwLock<ExecutorRegistry> = RwLock::new(ExecutorRegistry::default());

    /// Metric: Number of tasks that the Executor has spawned
    static ref SPAWNED_TASK_COUNTER: prometheus::IntCounterVec = metrics::registry().register_int_counter_vec(
        ExecutorRegistry::SPAWNED_TASK_COUNTER_METRIC_ID,
        "Number of tasks that the Executor has spawned".to_string(),
        &[ExecutorRegistry::EXECUTOR_ID_LABEL_ID],
        None
    ).unwrap();
}

/// An executor can only be registered once, and once it is registered, it stays registered for the
/// life of the app.
/// - returns false is an executor with the same ID is already registered
pub fn register(
    id: ExecutorId,
    builder: &mut ThreadPoolBuilder,
) -> Result<Executor, ExecutorRegistryError> {
    let mut executors = EXECUTORS.write().unwrap();
    if executors.thread_pools.contains_key(&id) {
        return Err(ExecutorRegistryError::ExecutorAlreadyRegistered(id));
    }
    let executor = Executor::new(id, builder)?;
    executors.thread_pools.insert(id, executor.clone());
    Ok(executor)
}

/// Returns the registered executor IDs
pub fn executor_ids() -> smallvec::SmallVec<[ExecutorId; 16]> {
    let executors = EXECUTORS.read().unwrap();
    executors.thread_pools.keys().cloned().collect()
}

/// returns the Executor for the specified ID
pub fn executor(id: ExecutorId) -> Option<Executor> {
    let executors = EXECUTORS.read().unwrap();
    match executors.thread_pools.get(&id) {
        Some(executor) => Some(executor.clone()),
        None => {
            if id == Executor::GLOBAL_EXECUTOR_ID {
                return Some(executors.global_executor.clone());
            }
            None
        }
    }
}

/// Returns the global executor, which is provided by default.
pub fn global_executor() -> Executor {
    let executors = EXECUTORS.read().unwrap();
    executors.global_executor.clone()
}

/// Executor registry
pub struct ExecutorRegistry {
    global_executor: Executor,
    thread_pools: fnv::FnvHashMap<ExecutorId, Executor>,
}

impl ExecutorRegistry {
    /// MetricId for spawned task counter: `M01D2DMYKJSPRG6H419R7ZFXVRH`
    pub const SPAWNED_TASK_COUNTER_METRIC_ID: metrics::MetricId =
        metrics::MetricId(1872376925834227814610238473431346961);
    /// The ExecutorId will be used as the label value: `L01D2DN1VBMW6XC7EQ971PBGW68`
    pub const EXECUTOR_ID_LABEL_ID: metrics::LabelId =
        metrics::LabelId(1872377054303353796724661249788899528);

    /// An executor can only be registered once, and once it is registered, it stays registered for the
    /// life of the app.
    /// - returns false is an executor with the same ID is already registered
    pub fn register(
        &mut self,
        id: ExecutorId,
        builder: &mut ThreadPoolBuilder,
    ) -> Result<Executor, ExecutorRegistryError> {
        if self.thread_pools.contains_key(&id) {
            return Err(ExecutorRegistryError::ExecutorAlreadyRegistered(id));
        }
        let executor = Executor::new(id, builder)?;
        self.thread_pools.insert(id, executor.clone());
        Ok(executor)
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
                if id == Executor::GLOBAL_EXECUTOR_ID {
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

impl Default for ExecutorRegistry {
    fn default() -> Self {
        fn default_executor() -> Executor {
            let mut builder = ExecutorBuilder::new(Executor::GLOBAL_EXECUTOR_ID).builder();
            Executor::new(Executor::GLOBAL_EXECUTOR_ID, &mut builder).unwrap()
        }

        Self {
            global_executor: default_executor(),
            thread_pools: fnv::FnvHashMap::default(),
        }
    }
}

impl fmt::Debug for ExecutorRegistry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Executors(thread pool count = {})",
            self.thread_pools.len()
        )
    }
}

/// A general-purpose thread pool based executor for scheduling tasks that poll futures to completion.
/// - The thread pool multiplexes any number of tasks onto a fixed number of worker threads.
/// - This type is a clonable handle to the threadpool itself. Cloning it will only create a new reference, not a new threadpool.
/// - is a thin wrapper around futures [ThreadPool](https://rust-lang-nursery.github.io/futures-api-docs/0.3.0-alpha.12/futures/executor/struct.ThreadPool.html)
///   executor
///
/// ## Metrics
///
#[derive(Clone)]
pub struct Executor {
    id: ExecutorId,
    thread_pool: ThreadPool,
    spawned_task_counter: prometheus::IntCounter,
}

impl fmt::Debug for Executor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Executor({})", self.id)
    }
}

impl Executor {
    /// Global ExecutorId, i.e., for the global Executor
    pub const GLOBAL_EXECUTOR_ID: ExecutorId = ExecutorId(1871427164235073850597045237139528853);

    /// constructor
    fn new(id: ExecutorId, builder: &mut ThreadPoolBuilder) -> Result<Self, ExecutorRegistryError> {
        Ok(Self {
            id,
            thread_pool: builder
                .create()
                .map_err(ExecutorRegistryError::ThreadPoolCreateFailed)?,
            spawned_task_counter: SPAWNED_TASK_COUNTER
                .with_label_values(&[id.to_string().as_str()]),
        })
    }

    /// Returns the ExecutorId
    pub const fn id(&self) -> ExecutorId {
        self.id
    }

    /// Runs the given future with this Executor.
    ///
    /// ## Notes
    /// - This function will block the calling thread until the given future is complete. This also
    ///   means that all other threads waiting on this Executor will also be blocked.
    /// - The function will return when the provided future completes, even if some of the
    ///   tasks it spawned are still running.
    ///
    /// ## Panics
    /// If the task panics.
    pub fn run<F: Future>(&mut self, f: F) -> F::Output {
        let result = { catch_unwind(AssertUnwindSafe(|| self.thread_pool.run(f))) };
        match result {
            Ok(res) => res,
            Err(err) => panic!(err),
        }
    }

    /// Spawns the future and awaits until it is done, returning it's result. This enables a future
    /// to be awaited on outside of an async context.
    ///
    /// ## Notes
    /// - This function will block the calling thread until the given future is complete. However,
    ///   the executor is released as soon as the future is spawned, making it available for other
    ///   threads to use the executor.
    pub fn spawn_await<F>(&mut self, f: F) -> Result<F::Output, ExecutorError>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        let (sender, receiver) = crossbeam::channel::bounded(0);
        {
            self.thread_pool
                .spawn(
                    async move {
                        let result = await!(f);
                        sender.send(result).unwrap();
                    },
                )
                .map_err(|err| ExecutorError::SpawnError {
                    is_executor_shutdown: err.is_shutdown(),
                })?;
        }
        receiver
            .recv()
            .map_err(|_| ExecutorError::SpawnedFuturePanic)
    }

    /// Spawns the future and returns a channel Receiver that can be used to retrieve the future result
    /// from both inside and outside of an async context.
    ///
    /// ## Notes
    /// If the future panics, then the Receiver will become disconnected.
    pub fn spawn_channel<F>(
        &mut self,
        f: F,
    ) -> Result<channel::oneshot::Receiver<F::Output>, ExecutorError>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        let (sender, receiver) = channel::oneshot::channel::<F::Output>();
        {
            self.thread_pool
                .spawn(
                    async move {
                        let result = await!(f);
                        let _ = sender.send(result);
                    },
                )
                .map_err(|err| ExecutorError::SpawnError {
                    is_executor_shutdown: err.is_shutdown(),
                })?;
        }
        Ok(receiver)
    }

    /// returns the number of tasks that have been spawned by this Executor
    pub fn spawned_task_count(&self) -> u64 {
        self.spawned_task_counter.get() as u64
    }
}

impl Spawn for Executor {
    fn spawn_obj(&mut self, future: FutureObj<'static, ()>) -> Result<(), SpawnError> {
        self.thread_pool.spawn_obj(future)?;
        self.spawned_task_counter.inc();
        Ok(())
    }

    fn status(&self) -> Result<(), SpawnError> {
        self.thread_pool.status()
    }
}

#[ulid]
/// Unique Executor ID
pub struct ExecutorId(pub u128);

/// Executor registry related errors
#[derive(Fail, Debug)]
pub enum ExecutorRegistryError {
    /// When a ThreadPool creation failure occurs.
    #[fail(display = "Failed to create ThreadPool: {}", _0)]
    ThreadPoolCreateFailed(io::Error),
    /// When trying to register an Executor using an ID that is already registered.
    #[fail(display = "Executor is already registered: {}", _0)]
    ExecutorAlreadyRegistered(ExecutorId),
}

/// Executor related errors
#[derive(Fail, Debug)]
pub enum ExecutorError {
    /// An error that occurred during spawning.
    #[fail(
        display = "Spawning Future failed: executor shutdown = {}",
        is_executor_shutdown
    )]
    SpawnError {
        /// whether spawning failed to the executor being shut down
        is_executor_shutdown: bool,
    },
    /// The spawned Future panicked while running.
    #[fail(display = "The spawned Future panicked while running.")]
    SpawnedFuturePanic,
}

/// ThreadPool config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutorBuilder {
    id: ExecutorId,
    stack_size: Option<NonZeroUsize>,
    pool_size: Option<NonZeroUsize>,
}

impl ExecutorBuilder {
    /// constructor
    pub fn new(id: ExecutorId) -> Self {
        Self {
            id,
            stack_size: None,
            pool_size: None,
        }
    }

    /// Sets the thread stack size
    pub fn set_stack_size(self, size: NonZeroUsize) -> Self {
        let mut this = self;
        this.stack_size = Some(size);
        this
    }

    /// Sets the thread pool size
    pub fn set_pool_size(self, size: NonZeroUsize) -> Self {
        let mut this = self;
        this.pool_size = Some(size);
        this
    }

    /// Returns the ExecutorId
    pub fn executor_id(&self) -> ExecutorId {
        self.id
    }

    /// Returns the thread stack size
    pub fn stack_size(&self) -> Option<usize> {
        self.stack_size.map(|size| size.get())
    }

    /// Returns the thread stack size
    pub fn pool_size(&self) -> Option<usize> {
        self.pool_size.map(|size| size.get())
    }

    fn builder(&self) -> ThreadPoolBuilder {
        let mut builder = ThreadPool::builder();
        builder
            .name_prefix(format!("{}-", self.id))
            .after_start(|thread_index| {
                debug!(
                    "Executer thread has started: {}-{}",
                    Executor::GLOBAL_EXECUTOR_ID,
                    thread_index
                )
            })
            .before_stop(|thread_index| {
                debug!(
                    "Executer thread is stopping: {}-{}",
                    Executor::GLOBAL_EXECUTOR_ID,
                    thread_index
                )
            });
        if let Some(ref size) = self.stack_size {
            builder.stack_size(size.get());
        }
        if let Some(ref size) = self.pool_size {
            builder.pool_size(size.get());
        }
        builder
    }

    /// Tries to build and register the Executor with the global ExecutorRegistry
    pub fn register(&self) -> Result<Executor, ExecutorRegistryError> {
        let mut executors = EXECUTORS.write().unwrap();
        let mut threadpool_builder = self.builder();
        executors.register(self.id, &mut threadpool_builder)
    }
}

#[allow(warnings)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::configure_logging;
    use crate::metrics;
    use futures::{future::FutureExt, task::SpawnExt};
    use std::{iter::Iterator, thread};

    #[test]
    fn global_executor() {
        configure_logging();

        let EXECUTOR_ID = ExecutorId::generate();
        let mut executor = ExecutorBuilder::new(EXECUTOR_ID).register().unwrap();
        let mut task_executor = executor.clone();
        executor.spawn(
            async move {
                info!("global_executor(): task #1");
                let mut task_executor_2 = task_executor.clone();
                task_executor.spawn(
                    async move {
                        info!("global_executor(): task #1.1");
                        await!(task_executor_2
                            .spawn_with_handle(
                                async move { info!("global_executor(): task #1.1.1") }
                            )
                            .unwrap());
                    },
                );
            },
        );
        let mut task_executor = executor.clone();
        let task_handle = executor
            .spawn_with_handle(
                async move {
                    info!("global_executor(): task #2");
                    await!(task_executor
                        .spawn_with_handle(async { info!("global_executor(): task #2.1") })
                        .unwrap());
                },
            )
            .unwrap();
        executor.run(task_handle);
        executor.run(async { info!("global_executor(): task #3") });

        let gathered_metrics =
            metrics::registry().gather_metrics(&[ExecutorRegistry::SPAWNED_TASK_COUNTER_METRIC_ID]);
        info!("gathered_metrics: {:#?}", gathered_metrics);
        if let metrics::Metric::IntCounterVec { desc, values } = gathered_metrics
            .metric(ExecutorRegistry::SPAWNED_TASK_COUNTER_METRIC_ID)
            .unwrap()
        {
            assert_eq!(desc.id(), ExecutorRegistry::SPAWNED_TASK_COUNTER_METRIC_ID);
            let metric_value = values
                .iter()
                .find(|metric_value| {
                    metric_value
                        .labels
                        .iter()
                        .find(|(label_id, value)| {
                            *label_id == ExecutorRegistry::EXECUTOR_ID_LABEL_ID
                                && *value == EXECUTOR_ID.to_string()
                        })
                        .is_some()
                })
                .unwrap();

            for i in 0..5 {
                if metric_value.value != 5 {
                    info!(
                        "yielding to give the spawned tasks a chance to run: ({})",
                        i
                    );
                    thread::yield_now();
                } else {
                    break;
                }
            }
            assert_eq!(metric_value.value, 5);
            assert_eq!(executor.spawned_task_count(), metric_value.value);
        } else {
            panic!("Metric was not found for Executors::SPAWNED_TASK_COUNTER_METRIC_ID");
        }
    }

    #[test]
    fn executor_spawn_await() {
        configure_logging();

        let executors = EXECUTORS.read().unwrap();
        let mut executor = executors.global_executor();
        let result = executor.spawn_await(
            async {
                info!("spawned task says hello");
                true
            },
        );
        info!("result: {:?}", result);
        assert!(result.unwrap());
        let result = executor.spawn_await(
            async {
                panic!("spawned task says hello");
                true
            },
        );
        info!("result: {:?}", result);
        match result {
            Ok(_) => panic!("should have returned an ExecutorError::SpawnedFuturePanic"),
            Err(ExecutorError::SpawnedFuturePanic) => info!("failed as expected"),
            Err(err) => panic!("failed with unexpected error: {}", err),
        }
    }

    #[test]
    fn executor_spawn_channel() {
        configure_logging();

        let executors = EXECUTORS.read().unwrap();
        let mut executor = executors.global_executor();
        let result_rx = executor.spawn_channel(
            async {
                info!("spawned task says hello");
                true
            },
        );
        let result = executor.run(result_rx.unwrap());
        info!("result: {:?}", result);
        assert!(result.unwrap());
        let result_rx = executor.spawn_channel(
            async {
                panic!("spawned task says hello");
                true
            },
        );
        let result = executor.spawn_await(result_rx.unwrap());
        info!("result: {:?}", result);
        match result {
            Ok(Err(channel::oneshot::Canceled)) => {
                info!("The future panicked, which caused the Receiver to be cancelled")
            }
            other => panic!("Unexpected result: {:?}", other),
        }
    }

    #[test]
    fn global_executor_shared_across_threads() {
        configure_logging();

        let thread_handles: Vec<thread::JoinHandle<_>> = (1..=2)
            .map(|i| {
                let handle = thread::spawn(move || {
                    let executors = EXECUTORS.read().unwrap();
                    let mut executor = executors.global_executor();
                    executor
                        .spawn(async move { info!("{:?} : task #{}", thread::current().id(), i) });
                    let task_handle = executor
                        .spawn_with_handle(
                            async move {
                                // simulate doing work
                                if i == 1 {
                                    info!(
                                        "{:?} : task #{} sleeping ...",
                                        thread::current().id(),
                                        i
                                    );
                                    thread::sleep_ms(1000);
                                    info!("{:?} : task #{} awoke ...", thread::current().id(), i);
                                }

                                info!(
                                    "{:?} : spawned task #{} says hello",
                                    thread::current().id(),
                                    i
                                )
                            },
                        )
                        .unwrap();
                    // because we are spawning the task async, the executor becomes available for other
                    // threads to use, i.e., no blocking occurs. If `executor.run(task_handle)` had
                    // been used, then all other threads waiting on this executor will be blocked
                    // until the task completes
                    executor.spawn(task_handle);
                });
                thread::yield_now();
                handle
            })
            .collect();

        for handle in thread_handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn registered_executors() {
        configure_logging();

        for _ in 0..32 {
            assert!(ExecutorBuilder::new(ExecutorId::generate())
                .register()
                .is_ok());
        }

        let threadpool_config = ExecutorBuilder::new(ExecutorId::generate());
        assert!(threadpool_config.register().is_ok());
        match threadpool_config
            .register()
            .expect_err("expected ExecutorAlreadyRegistered")
        {
            ExecutorRegistryError::ExecutorAlreadyRegistered(id) => {
                assert_eq!(id, threadpool_config.executor_id())
            }
            err => panic!(
                "expected ExecutorAlreadyRegistered, but error was : {:?}",
                err
            ),
        }
    }

    #[test]
    fn threadpool_config() {
        let id = ExecutorId::generate();
        let config = ExecutorBuilder::new(id);
        assert_eq!(config.executor_id(), id);
        assert!(config.stack_size().is_none());
        assert!(config.pool_size().is_none());
        let config = config.set_stack_size(NonZeroUsize::new(1024).unwrap());
        assert_eq!(config.stack_size().unwrap(), 1024);
        assert!(config.pool_size().is_none());
        let config = config.set_pool_size(NonZeroUsize::new(64).unwrap());
        assert_eq!(config.stack_size().unwrap(), 1024);
        assert_eq!(config.pool_size().unwrap(), 64);
    }

    // the panic is bubbled up to the current thread when awaiting on a task that panics
    #[test]
    #[should_panic]
    fn run_spawned_panic_task() {
        configure_logging();

        let result = {
            let executors = EXECUTORS.read().unwrap();
            let mut executor = executors.global_executor();
            let task_handle = executor
                .spawn_with_handle(async { panic!("BOOM!!") })
                .unwrap();
            catch_unwind(AssertUnwindSafe(|| executor.run(task_handle)))
        };
        if let Err(err) = result {
            panic!(err);
        }
    }

    // the panic is bubbled up to the current thread when awaiting on a task that panics
    #[test]
    #[should_panic]
    fn await_spawned_panic_task() {
        configure_logging();

        let result = {
            let executors = EXECUTORS.read().unwrap();
            let mut executor = executors.global_executor();
            let task_handle = executor
                .spawn_with_handle(async { panic!("BOOM!!") })
                .unwrap();
            catch_unwind(AssertUnwindSafe(|| {
                executor.run(
                    async {
                        await!(task_handle);
                    },
                )
            }))
        };
        if let Err(err) = result {
            panic!(err);
        }
    }

    // threads that panic in the pool do get replaced
    #[test]
    fn spawned_task_panics_all_threads() {
        configure_logging();

        let executors = EXECUTORS.read().unwrap();
        let mut executor = executors.global_executor();
        let panic_task_count = num_cpus::get() * 2;
        let mut handles = vec![];
        for i in 0..panic_task_count {
            let handle = executor.spawn_with_handle(
                async move {
                    panic!("BOOM({})!", i);
                },
            );
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
