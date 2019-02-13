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
//! - async [executors](struct.Executor.html)
//! - global [executor registry](struct.ExecutorRegistry.html)
//!   - executors can be globally registered via [register()](fn.register.html)
//! - [global executor](fn.global_executor.html)
//!
//! ## Config
//! - [ExecutorBuilder](struct.ExecutorBuilder.html) - is also used to register a new [Executor](struct.Executor.html)
//! ``` rust
//! # use oysterpack_trust::concurrent::execution::*;
//! # use std::num::*;
//! const EXECUTOR_ID: ExecutorId = ExecutorId(1872692872983539779132843447162269015);
//! let mut executor = ExecutorBuilder::new(EXECUTOR_ID)
//!     .set_pool_size(NonZeroUsize::new(16).unwrap())
//!     .set_stack_size(NonZeroUsize::new(1024*64).unwrap())
//!     .register()
//!     .unwrap();
//! ```
//! - each Executor is uniquely identified by its [ExecutorId](struct.ExecutorId.html)
//!
//! ## Metrics
//! - number of spawned tasks per Executor
//! - number of completed tasks per Executor
//! - number of threads started
//! - Executor thread pool sizes

use crate::metrics;
use failure::Fail;
use futures::{
    executor::{ThreadPool, ThreadPoolBuilder},
    future::{Future, FutureExt, FutureObj},
    task::{Spawn, SpawnError, SpawnExt},
};
use lazy_static::lazy_static;
use oysterpack_log::*;
use oysterpack_uid::macros::ulid;
use serde::{Deserialize, Serialize};
use std::{fmt, io, iter::ExactSizeIterator, num::NonZeroUsize, sync::RwLock};

lazy_static! {
    /// Global Executor registry
    static ref EXECUTOR_REGISTRY: RwLock<ExecutorRegistry> = RwLock::new(ExecutorRegistry::default());

    /// Metric: Number of tasks that the Executor has spawned
    static ref SPAWNED_TASK_COUNTER: prometheus::IntCounterVec = metrics::registry().register_int_counter_vec(
        SPAWNED_TASK_COUNTER_METRIC_ID,
        "Number of tasks that the Executor has spawned",
        &[EXECUTOR_ID_LABEL_ID],
        None
    ).unwrap();

    /// Metric: Number of tasks that the Executor has completed
    static ref COMPLETED_TASK_COUNTER: prometheus::IntCounterVec = metrics::registry().register_int_counter_vec(
        COMPLETED_TASK_COUNTER_METRIC_ID,
        "Number of tasks that the Executor has completed",
        &[EXECUTOR_ID_LABEL_ID],
        None
    ).unwrap();

    /// Metric: Executor thread pool sizes
    static ref THREAD_POOL_SIZE_GAUGE: prometheus::IntGaugeVec = metrics::registry().register_int_gauge_vec(
        THREADS_POOL_SIZE_GAUGE_METRIC_ID,
        "Thread pool size",
        &[EXECUTOR_ID_LABEL_ID],
        None
    ).unwrap();

    /// Metric: Total number of Executor threads that have been started
    static ref THREADS_STARTED_TOTAL_COUNTER: prometheus::IntCounter = metrics::registry().register_int_counter(
        THREADS_STARTED_TOTAL_COUNTER_METRIC_ID,
        "Total number of threads that have been started across all Executors",
        None
    ).unwrap();

}

/// MetricId for spawned task counter: `M01D2DMYKJSPRG6H419R7ZFXVRH`
pub const SPAWNED_TASK_COUNTER_METRIC_ID: metrics::MetricId =
    metrics::MetricId(1872376925834227814610238473431346961);
/// MetricId for spawned task counter: `01D39C05YGY6NY3RD18TJ6975H`
pub const COMPLETED_TASK_COUNTER_METRIC_ID: metrics::MetricId =
    metrics::MetricId(1873501394267260593175681052637961393);
/// MetricId for total number of Executor threads that have been started: `01D3950A0931ESKR66XG7KMD7Z`
pub const THREADS_STARTED_TOTAL_COUNTER_METRIC_ID: metrics::MetricId =
    metrics::MetricId(1873492525732701726868218222598567167);
/// MetricId for total number of Executor threads that have been started: `01D395423XG3514YP762RYTDJ1`
pub const THREADS_POOL_SIZE_GAUGE_METRIC_ID: metrics::MetricId =
    metrics::MetricId(1873492674426234963241985324245399105);
/// The ExecutorId will be used as the label value: `L01D2DN1VBMW6XC7EQ971PBGW68`
pub const EXECUTOR_ID_LABEL_ID: metrics::LabelId =
    metrics::LabelId(1872377054303353796724661249788899528);

/// Gathers Executor related metrics
pub fn gather_metrics() -> Vec<prometheus::proto::MetricFamily> {
    metrics::registry().gather_metrics_by_name(&[
        SPAWNED_TASK_COUNTER_METRIC_ID.name().as_str(),
        COMPLETED_TASK_COUNTER_METRIC_ID.name().as_str(),
        THREADS_STARTED_TOTAL_COUNTER_METRIC_ID.name().as_str(),
        THREADS_POOL_SIZE_GAUGE_METRIC_ID.name().as_str(),
    ])
}

/// An executor can only be registered once, and once it is registered, it stays registered for the
/// life of the app.
/// - returns false is an executor with the same ID is already registered
pub fn register(
    id: ExecutorId,
    builder: &mut ThreadPoolBuilder,
) -> Result<Executor, ExecutorRegistryError> {
    let mut executors = EXECUTOR_REGISTRY.write().unwrap();
    if executors.thread_pools.contains_key(&id) {
        return Err(ExecutorRegistryError::ExecutorAlreadyRegistered(id));
    }
    let executor = Executor::new(id, builder)?;
    executors.thread_pools.insert(id, executor.clone());
    Ok(executor)
}

/// Returns the registered executor IDs
pub fn executor_ids() -> smallvec::SmallVec<[ExecutorId; 16]> {
    let executors = EXECUTOR_REGISTRY.read().unwrap();
    executors.thread_pools.keys().cloned().collect()
}

/// returns the Executor for the specified ID
pub fn executor(id: ExecutorId) -> Option<Executor> {
    let executors = EXECUTOR_REGISTRY.read().unwrap();
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
    let executors = EXECUTOR_REGISTRY.read().unwrap();
    executors.global_executor.clone()
}

/// Returns the total number of tasks spawned across all registered Executor(s)
pub fn spawned_task_count() -> u64 {
    let executors = EXECUTOR_REGISTRY.read().unwrap();
    executors.spawned_task_count()
}

/// Returns the total number of threads that have been started across all Executors
/// - this is not the active count
pub fn total_threads_started() -> u64 {
    THREADS_STARTED_TOTAL_COUNTER.get() as u64
}

/// Returns the current thread pool sizes for the currently registered Executors
pub fn executor_thread_pool_sizes() -> Vec<(ExecutorId, u64)> {
    let executors = EXECUTOR_REGISTRY.read().unwrap();
    executors.executor_thread_pool_sizes()
}

/// Executor registry
pub struct ExecutorRegistry {
    global_executor: Executor,
    thread_pools: fnv::FnvHashMap<ExecutorId, Executor>,
}

impl ExecutorRegistry {
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

    /// Returns the total number of tasks spawned across all registered Executor(s)
    pub fn spawned_task_count(&self) -> u64 {
        self.thread_pools.values().fold(
            self.global_executor.spawned_task_count(),
            |sum, executor| sum + executor.spawned_task_count(),
        )
    }

    /// Returns the current thread pool sizes for the currently registered Executors
    pub fn executor_thread_pool_sizes(&self) -> Vec<(ExecutorId, u64)> {
        self.thread_pools
            .iter()
            .map(|(executor_id, executor)| (*executor_id, executor.thread_pool_size()))
            .collect()
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
    completed_task_counter: prometheus::IntCounter,
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
        let label_name = id.to_string();
        let labels = [label_name.as_str()];
        Ok(Self {
            id,
            thread_pool: builder
                .create()
                .map_err(ExecutorRegistryError::ThreadPoolCreateFailed)?,
            spawned_task_counter: SPAWNED_TASK_COUNTER.with_label_values(&labels),
            completed_task_counter: COMPLETED_TASK_COUNTER.with_label_values(&labels),
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
        self.thread_pool.run(f)
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

    /// Spawns the future and returns a crossbeam channel Receiver that can be used to retrieve the
    /// future result.
    ///
    /// ## Notes
    /// If the future panics, then the Receiver will become disconnected.
    pub fn spawn_channel<F>(
        &mut self,
        f: F,
    ) -> Result<crossbeam::channel::Receiver<F::Output>, ExecutorError>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        let (sender, receiver) = crossbeam::channel::bounded(1);
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

    /// returns the number of tasks that the Executor has spawned
    pub fn spawned_task_count(&self) -> u64 {
        self.spawned_task_counter.get() as u64
    }

    /// retuns the number of tasks that the Executor has completed
    pub fn completed_task_count(&self) -> u64 {
        self.completed_task_counter.get() as u64
    }

    /// returns the number of active tasks, i.e., the difference between spawned and completed
    pub fn active_task_count(&self) -> u64 {
        self.spawned_task_count() - self.completed_task_count()
    }

    /// returns the current thread pool size
    pub fn thread_pool_size(&self) -> u64 {
        let executor_thread_gauge =
            THREAD_POOL_SIZE_GAUGE.with_label_values(&[self.id.to_string().as_str()]);
        executor_thread_gauge.get() as u64
    }
}

impl Spawn for Executor {
    fn spawn_obj(&mut self, future: FutureObj<'static, ()>) -> Result<(), SpawnError> {
        let completed_task_counter = self.completed_task_counter.clone();
        //        let future = future.map( move |result| {
        //            completed_task_counter.inc();
        //            result
        //        }).boxed();
        let future = async move {
            await!(future);
            completed_task_counter.inc();
        }
            .boxed();
        self.thread_pool.spawn_obj(FutureObj::new(future))?;
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
        /// whether spawning failed because the executor is shut down
        is_executor_shutdown: bool,
    },
    /// The spawned Future panicked while running.
    #[fail(display = "The spawned Future panicked while running.")]
    SpawnedFuturePanic,
}

/// Executor builder, which is used to register the Executor with the global Executor registry
///
/// ## Example
/// ``` rust
/// # use oysterpack_trust::concurrent::execution::*;
/// const EXECUTOR_ID: ExecutorId = ExecutorId(1872692872983539779132843447162269015);
/// let mut executor = ExecutorBuilder::new(EXECUTOR_ID).register().unwrap();
/// ```
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
    pub fn set_stack_size(mut self, size: NonZeroUsize) -> Self {
        self.stack_size = Some(size);
        self
    }

    /// Sets the thread pool size
    pub fn set_pool_size(mut self, size: NonZeroUsize) -> Self {
        self.pool_size = Some(size);
        self
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
        let threads_started_counter = THREADS_STARTED_TOTAL_COUNTER.clone();
        let executor_thread_gauge_after_start =
            THREAD_POOL_SIZE_GAUGE.with_label_values(&[self.id.to_string().as_str()]);
        let executor_thread_gauge_before_stop = executor_thread_gauge_after_start.clone();
        let mut builder = ThreadPool::builder();
        builder
            .name_prefix(format!("{}-", self.id))
            .after_start(move |thread_index| {
                threads_started_counter.inc();
                executor_thread_gauge_after_start.inc();
                debug!(
                    "Executer thread has started: {}-{}",
                    Executor::GLOBAL_EXECUTOR_ID,
                    thread_index
                )
            })
            .before_stop(move |thread_index| {
                executor_thread_gauge_before_stop.dec();
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
    pub fn register(self) -> Result<Executor, ExecutorRegistryError> {
        let mut executors = EXECUTOR_REGISTRY.write().unwrap();
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
    use std::{iter::Iterator, panic::*, thread};

    #[test]
    fn global_executor() {
        configure_logging();

        // GIVEN: Tasks are spawned
        let EXECUTOR_ID = ExecutorId::generate();
        let mut executor = ExecutorBuilder::new(EXECUTOR_ID).register().unwrap();
        // GIVEN: sub tasks are spawned on the parent task executor
        let mut task_executor = executor.clone();
        // GIVEN: TASK #1 is spawned
        executor.spawn(
            async move {
                info!("global_executor(): task #1");
                let mut task_executor_2 = task_executor.clone();
                // GIVEN: SUB TASK #1.1 is spawned
                task_executor.spawn(
                    async move {
                        info!("global_executor(): task #1.1");
                        // GIVEN: SUB TASK #1.1.1 is spawned
                        await!(task_executor_2
                            .spawn_with_handle(
                                async move { info!("global_executor(): task #1.1.1") }
                            )
                            .unwrap());
                    },
                );
            },
        );

        // GIVEN: TASK #2 is spawned on the same executor as TASK #1
        let mut task_executor = executor.clone();
        let task_handle = executor
            .spawn_with_handle(
                async move {
                    info!("global_executor(): task #2");
                    // GIVEN: TASK #2.1 is spawned on the same executor as TASK #1
                    await!(task_executor
                        .spawn_with_handle(async { info!("global_executor(): task #2.1") })
                        .unwrap());
                },
            )
            .unwrap();

        // THEN: wait for TASK #2 to complete
        executor.run(task_handle);
        // THEN: wait for TASK #3 to complete
        executor.run(async { info!("global_executor(): task #3") });
        thread::yield_now();

        // returns the spawned task count for the above Executor
        let get_counter = || {
            let gathered_metrics = metrics::registry().gather();
            info!("gathered_metrics: {:#?}", gathered_metrics);
            let metric_family = gathered_metrics
                .iter()
                .find(|metric_family| {
                    metric_family.get_name() == SPAWNED_TASK_COUNTER_METRIC_ID.name().as_str()
                })
                .unwrap();
            let metrics = metric_family.get_metric();
            let metric = metric_family
                .get_metric()
                .iter()
                .find(|metric| {
                    metric
                        .get_label()
                        .iter()
                        .find(|label_pair| {
                            label_pair.get_value() == EXECUTOR_ID.to_string().as_str()
                        })
                        .is_some()
                })
                .unwrap();
            metric.get_counter().clone()
        };

        // THEN: wait until the tasks have completed
        for i in 0..5 {
            if get_counter().get_value() < 5.0 {
                info!(
                    "yielding to give the spawned tasks a chance to run: ({})",
                    i
                );
                thread::yield_now();
            } else {
                break;
            }
        }

        // THEN: the spawned task count should match
        assert_eq!(get_counter().get_value() as u64, 5);
        assert_eq!(
            executor.spawned_task_count(),
            get_counter().get_value() as u64
        );

        // WHEN: the total spawned task count is gathered directly from metrics
        let mfs = gather_metrics();
        info!("Executor metrics: {:#?}", mfs);
        let total_spawned_task_count: u64 = mfs
            .iter()
            .filter(|mf| mf.get_name() == SPAWNED_TASK_COUNTER_METRIC_ID.name().as_str())
            .flat_map(|mf| mf.get_metric())
            .map(|metric| metric.get_counter().get_value() as u64)
            .collect::<Vec<_>>()
            .iter()
            .sum();
        // THEN: it should match the count from `spawned_task_count()`
        // there may be a race condition when tests are run in parallel that are spawning tasks
        assert!(total_spawned_task_count <= spawned_task_count());

        // THEN: all tasks should be completed
        while executor.active_task_count() != 0 {
            info!(
                "waiting for tasks to complete: executor.active_task_count() = {}",
                executor.active_task_count()
            );
            thread::yield_now();
        }
    }

    #[test]
    fn executor_spawn_await() {
        configure_logging();

        let executors = EXECUTOR_REGISTRY.read().unwrap();
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

        let mut executor = super::global_executor();
        // GIVEN: a task is spawned
        let result_rx = executor
            .spawn_channel(
                async {
                    info!("spawned task says hello");
                    true
                },
            )
            .unwrap();

        // THEN: it is successfully received
        let result = result_rx.recv().unwrap();
        info!("result: {:?}", result);
        assert!(result);

        // GIVEN: a task that panics
        let result_rx = executor
            .spawn_channel(
                async {
                    panic!("spawned task says hello");
                    true
                },
            )
            .unwrap();

        // THEN: the channel will become disconnected
        match result_rx.recv() {
            Err(crossbeam::channel::RecvError) => {
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
                    let executors = EXECUTOR_REGISTRY.read().unwrap();
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
        let executor_id = threadpool_config.executor_id();
        assert!(threadpool_config.register().is_ok());
        let threadpool_config = ExecutorBuilder::new(executor_id);
        match threadpool_config
            .register()
            .expect_err("expected ExecutorAlreadyRegistered")
        {
            ExecutorRegistryError::ExecutorAlreadyRegistered(id) => assert_eq!(id, executor_id),
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
            let executors = EXECUTOR_REGISTRY.read().unwrap();
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
            let executors = EXECUTOR_REGISTRY.read().unwrap();
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

        let executors = EXECUTOR_REGISTRY.read().unwrap();
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
