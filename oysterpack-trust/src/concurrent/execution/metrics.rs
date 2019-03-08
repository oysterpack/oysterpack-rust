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

//! execution related metrics

use crate::metrics;
use lazy_static::lazy_static;
use prometheus::core::Collector;

lazy_static! {

    /// Metric: Number of tasks that the Executor has spawned and run
    pub (super) static ref TASK_SPAWNED_COUNTER: prometheus::IntCounterVec = metrics::registry().register_int_counter_vec(
        TASK_SPAWNED_COUNTER_METRIC_ID,
        "Task spawned count",
        &[EXECUTOR_ID_LABEL_ID],
        None
    ).unwrap();

    /// Metric: Number of tasks that the Executor has completed
    pub (super) static ref TASK_COMPLETED_COUNTER: prometheus::IntCounterVec = metrics::registry().register_int_counter_vec(
        TASK_COMPLETED_COUNTER_METRIC_ID,
        "Task completed count",
        &[EXECUTOR_ID_LABEL_ID],
        None
    ).unwrap();

    /// Metric: Executor thread pool sizes
    pub (super) static ref THREAD_POOL_SIZE_GAUGE: prometheus::IntGaugeVec = metrics::registry().register_int_gauge_vec(
        THREADS_POOL_SIZE_GAUGE_METRIC_ID,
        "Thread pool size",
        &[EXECUTOR_ID_LABEL_ID],
        None
    ).unwrap();

    /// Metric: Number of spawned tasks that panicked
    /// - this is only tracked for Executors that are configured to catch unwinding panics
    pub (super) static ref TASK_PANIC_COUNTER: prometheus::IntCounterVec = metrics::registry().register_int_counter_vec(
        TASK_PANIC_COUNTER_METRIC_ID,
        "Task panic count",
        &[EXECUTOR_ID_LABEL_ID],
        None
    ).unwrap();
}

/// MetricId for spawned task counter: `M01D2DMYKJSPRG6H419R7ZFXVRH`
pub const TASK_SPAWNED_COUNTER_METRIC_ID: metrics::MetricId =
    metrics::MetricId(1872376925834227814610238473431346961);
/// MetricId for spawned task counter: `01D39C05YGY6NY3RD18TJ6975H`
pub const TASK_COMPLETED_COUNTER_METRIC_ID: metrics::MetricId =
    metrics::MetricId(1873501394267260593175681052637961393);
/// MetricId for total number of Executor threads that have been started: `01D3950A0931ESKR66XG7KMD7Z`
pub const TASK_PANIC_COUNTER_METRIC_ID: metrics::MetricId =
    metrics::MetricId(1873492525732701726868218222598567167);
/// MetricId for total number of Executor threads that have been started: `01D395423XG3514YP762RYTDJ1`
pub const THREADS_POOL_SIZE_GAUGE_METRIC_ID: metrics::MetricId =
    metrics::MetricId(1873492674426234963241985324245399105);
/// The ExecutorId will be used as the label value: `L01D2DN1VBMW6XC7EQ971PBGW68`
pub const EXECUTOR_ID_LABEL_ID: metrics::LabelId =
    metrics::LabelId(1872377054303353796724661249788899528);

/// Gathers Executor related metrics
pub fn gather_metrics() -> Vec<prometheus::proto::MetricFamily> {
    let mut mfs = Vec::with_capacity(7);
    mfs.extend(TASK_SPAWNED_COUNTER.collect());
    mfs.extend(TASK_COMPLETED_COUNTER.collect());
    mfs.extend(TASK_PANIC_COUNTER.collect());
    mfs.extend(THREAD_POOL_SIZE_GAUGE.collect());
    mfs
}

/// Returns Executor related metric descriptors
pub fn metric_descs() -> Vec<&'static prometheus::core::Desc> {
    let mut descs = Vec::with_capacity(7);
    descs.extend(TASK_SPAWNED_COUNTER.desc());
    descs.extend(TASK_COMPLETED_COUNTER.desc());
    descs.extend(TASK_PANIC_COUNTER.desc());
    descs.extend(THREAD_POOL_SIZE_GAUGE.desc());
    descs
}
