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

//! Provides metrics support for prometheus.
//!
//! - [METRIC_REGISTRY](struct.METRIC_REGISTRY.html) provides a global [MetricRegistry](struct.MetricRegistry.html) that can be used
//!   throughout the application
//!   - it's a threadsafe singleton - protected by a Mutex
//! - Instead of using arbitrary strings, numeric based identifiers are used (see below for rationale)
//!   - [MetricId](struct.MetricId.html)
//!   - [LabelId](struct.LabelId.html)
//! - Metric descriptors can be retrieved for the registered metrics
//!   - [MetricDesc](struct.MetricDesc.html)
//!   - [MetricVecDesc](struct.MetricVecDesc.html)
//!   - [HistogramDesc](struct.HistogramDesc.html)
//!   - [HistogramVecDesc](struct.HistogramVecDesc.html)
//!
//! ### Why use a number as a metric name and label names ?
//! - because names change over time, which can break components that depend on metric names
//! - to avoid name collision
//!
//! Assigning unique numerical identifiers is much more stable. Human friendly metric labels and any
//! additional information can be mapped externally to the MetricId.
//!
//! ### Notes
//! - the prometheus metric `help` attribute can be used to provide a human friendly label and
//!   short description

use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use oysterpack_uid::{macros::ulid, ulid_u128_into_string, ULID};
use prometheus::{core::Collector, Encoder};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use std::{
    collections::{HashMap, HashSet},
    fmt,
    io::Write,
    iter::Iterator,
    str::FromStr,
    sync::{Arc, RwLock},
};

lazy_static! {
    /// Global metrics registry
    pub static ref METRIC_REGISTRY: MetricRegistry = MetricRegistry::default();
}

// used to minimize memory allocations on the heap
const METRIC_DESC_SMALLVEC_SIZE: usize = 8;
const BUCKETS_SMALLVEC_SIZE: usize = 8;

/// Metric Registry
/// - process metrics collector is automatically added
pub struct MetricRegistry {
    registry: prometheus::Registry,

    // TODO: remove f64 based counters - int based counters provide better performance and are more practical as counters
    counters: RwLock<fnv::FnvHashMap<MetricId, prometheus::Counter>>,
    counter_vecs: RwLock<fnv::FnvHashMap<MetricId, prometheus::CounterVec>>,
    int_counters: RwLock<fnv::FnvHashMap<MetricId, prometheus::IntCounter>>,
    int_counter_vecs: RwLock<fnv::FnvHashMap<MetricId, prometheus::IntCounterVec>>,

    gauges: RwLock<fnv::FnvHashMap<MetricId, prometheus::Gauge>>,
    gauge_vecs: RwLock<fnv::FnvHashMap<MetricId, prometheus::GaugeVec>>,
    int_gauges: RwLock<fnv::FnvHashMap<MetricId, prometheus::IntGauge>>,
    int_gauge_vecs: RwLock<fnv::FnvHashMap<MetricId, prometheus::IntGaugeVec>>,

    histograms: RwLock<fnv::FnvHashMap<MetricId, (prometheus::Histogram, Buckets)>>,
    histogram_vecs: RwLock<fnv::FnvHashMap<MetricId, (prometheus::HistogramVec, Buckets)>>,

    process_collector: ProcessCollector,
}

impl MetricRegistry {
    /// Tries to register an int gauge metric
    pub fn register_int_gauge(
        &self,
        metric_id: MetricId,
        help: String,
        const_labels: Option<HashMap<LabelId, String>>,
    ) -> prometheus::Result<prometheus::IntGauge> {
        let help = Self::check_help(help)?;
        let const_labels = Self::check_const_labels(const_labels)?;

        let mut metrics = self.int_gauges.write().unwrap();
        if metrics.contains_key(&metric_id) {
            return Err(prometheus::Error::AlreadyReg);
        }

        let mut opts = prometheus::Opts::new(metric_id.name(), help);
        if let Some(const_labels) = const_labels {
            opts = opts.const_labels(const_labels);
        }

        let metric = prometheus::IntGauge::with_opts(opts)?;
        self.registry.register(Box::new(metric.clone()))?;
        metrics.insert(metric_id, metric.clone());
        Ok(metric)
    }

    /// Tries to register an int gauge metric
    pub fn register_gauge(
        &self,
        metric_id: MetricId,
        help: String,
        const_labels: Option<HashMap<LabelId, String>>,
    ) -> prometheus::Result<prometheus::Gauge> {
        let help = Self::check_help(help)?;
        let const_labels = Self::check_const_labels(const_labels)?;

        let mut metrics = self.gauges.write().unwrap();
        if metrics.contains_key(&metric_id) {
            return Err(prometheus::Error::AlreadyReg);
        }

        let mut opts = prometheus::Opts::new(metric_id.name(), help);
        if let Some(const_labels) = const_labels {
            opts = opts.const_labels(const_labels);
        }

        let metric = prometheus::Gauge::with_opts(opts)?;
        self.registry.register(Box::new(metric.clone()))?;
        metrics.insert(metric_id, metric.clone());
        Ok(metric)
    }

    /// Tries to register a CounterVec metric
    ///
    /// ## Params
    /// - **metric_id** ULID is prefixed with 'M' to construct the [metric fully qualified name](https://prometheus.io/docs/concepts/data_model/#metric-names-and-labels)
    ///   - e.g. if the MetricId ULID is *01D1ZMQVMQ5C6Z09JBF32T41ZK*, then the metric name will be **M***01D1ZMQVMQ5C6Z09JBF32T41ZK*
    /// - **help** is mandatory - use it to provide a human friendly name for the metric and provide a short description
    /// - label_names - the labels used to define the metric's dimensions
    ///   - labels will be trimmed and must not be blank
    /// - **buckets** define the buckets into which observations are counted.
    ///   - Each element in the slice is the upper inclusive bound of a bucket.
    ///   - The values will be deduped and sorted in strictly increasing order.
    ///   - There is no need to add a highest bucket with +Inf bound, it will be added implicitly.
    ///
    /// ## Errors
    /// - if no labels are provided
    /// - if labels are blank
    /// - if any of the constant label names or values are blank
    /// - if there are no buckets defined
    ///
    /// ## Notes
    ///
    pub fn register_gauge_vec(
        &self,
        metric_id: MetricId,
        help: String,
        label_ids: &[LabelId],
        const_labels: Option<HashMap<LabelId, String>>,
    ) -> prometheus::Result<prometheus::GaugeVec> {
        let label_names = Self::check_labels(label_ids)?;
        let help = Self::check_help(help)?;
        let const_labels = Self::check_const_labels(const_labels)?;

        let mut metrics = self.gauge_vecs.write().unwrap();
        if metrics.contains_key(&metric_id) {
            return Err(prometheus::Error::AlreadyReg);
        }

        let mut opts = prometheus::Opts::new(metric_id.name(), help);
        if let Some(const_labels) = const_labels {
            opts = opts.const_labels(const_labels);
        }

        let label_names: Vec<&str> = label_names.iter().map(|label| label.as_str()).collect();
        let metric = prometheus::GaugeVec::new(opts, &label_names)?;
        self.registry.register(Box::new(metric.clone()))?;
        metrics.insert(metric_id, metric.clone());
        Ok(metric)
    }

    /// Tries to register a CounterVec metric
    ///
    /// ## Params
    /// - **metric_id** ULID is prefixed with 'M' to construct the [metric fully qualified name](https://prometheus.io/docs/concepts/data_model/#metric-names-and-labels)
    ///   - e.g. if the MetricId ULID is *01D1ZMQVMQ5C6Z09JBF32T41ZK*, then the metric name will be **M***01D1ZMQVMQ5C6Z09JBF32T41ZK*
    /// - **help** is mandatory - use it to provide a human friendly name for the metric and provide a short description
    /// - label_names - the labels used to define the metric's dimensions
    ///   - labels will be trimmed and must not be blank
    /// - **buckets** define the buckets into which observations are counted.
    ///   - Each element in the slice is the upper inclusive bound of a bucket.
    ///   - The values will be deduped and sorted in strictly increasing order.
    ///   - There is no need to add a highest bucket with +Inf bound, it will be added implicitly.
    ///
    /// ## Errors
    /// - if no labels are provided
    /// - if labels are blank
    /// - if any of the constant label names or values are blank
    /// - if there are no buckets defined
    ///
    /// ## Notes
    ///
    pub fn register_int_gauge_vec(
        &self,
        metric_id: MetricId,
        help: String,
        label_ids: &[LabelId],
        const_labels: Option<HashMap<LabelId, String>>,
    ) -> prometheus::Result<prometheus::IntGaugeVec> {
        let label_names = Self::check_labels(label_ids)?;
        let help = Self::check_help(help)?;
        let const_labels = Self::check_const_labels(const_labels)?;

        let mut metrics = self.int_gauge_vecs.write().unwrap();
        if metrics.contains_key(&metric_id) {
            return Err(prometheus::Error::AlreadyReg);
        }

        let mut opts = prometheus::Opts::new(metric_id.name(), help);
        if let Some(const_labels) = const_labels {
            opts = opts.const_labels(const_labels);
        }

        let label_names: Vec<&str> = label_names.iter().map(|label| label.as_str()).collect();
        let metric = prometheus::IntGaugeVec::new(opts, &label_names)?;
        self.registry.register(Box::new(metric.clone()))?;
        metrics.insert(metric_id, metric.clone());
        Ok(metric)
    }

    /// Tries to register an int counter metric
    pub fn register_int_counter(
        &self,
        metric_id: MetricId,
        help: String,
        const_labels: Option<HashMap<LabelId, String>>,
    ) -> prometheus::Result<prometheus::IntCounter> {
        let help = Self::check_help(help)?;
        let const_labels = Self::check_const_labels(const_labels)?;

        let mut metrics = self.int_counters.write().unwrap();
        if metrics.contains_key(&metric_id) {
            return Err(prometheus::Error::AlreadyReg);
        }

        let mut opts = prometheus::Opts::new(metric_id.name(), help);
        if let Some(const_labels) = const_labels {
            opts = opts.const_labels(const_labels);
        }

        let metric = prometheus::IntCounter::with_opts(opts)?;
        self.registry.register(Box::new(metric.clone()))?;
        metrics.insert(metric_id, metric.clone());
        Ok(metric)
    }

    /// Tries to register a counter metric
    pub fn register_counter(
        &self,
        metric_id: MetricId,
        help: String,
        const_labels: Option<HashMap<LabelId, String>>,
    ) -> prometheus::Result<prometheus::Counter> {
        let help = Self::check_help(help)?;
        let const_labels = Self::check_const_labels(const_labels)?;

        let mut metrics = self.counters.write().unwrap();
        if metrics.contains_key(&metric_id) {
            return Err(prometheus::Error::AlreadyReg);
        }

        let mut opts = prometheus::Opts::new(metric_id.name(), help);
        if let Some(const_labels) = const_labels {
            opts = opts.const_labels(const_labels);
        }

        let metric = prometheus::Counter::with_opts(opts)?;
        self.registry.register(Box::new(metric.clone()))?;
        metrics.insert(metric_id, metric.clone());
        Ok(metric)
    }

    /// Tries to register a CounterVec metric
    ///
    /// ## Params
    /// - **metric_id** ULID is prefixed with 'M' to construct the [metric fully qualified name](https://prometheus.io/docs/concepts/data_model/#metric-names-and-labels)
    ///   - e.g. if the MetricId ULID is *01D1ZMQVMQ5C6Z09JBF32T41ZK*, then the metric name will be **M***01D1ZMQVMQ5C6Z09JBF32T41ZK*
    /// - **help** is mandatory - use it to provide a human friendly name for the metric and provide a short description
    /// - label_names - the labels used to define the metric's dimensions
    ///   - labels will be trimmed and must not be blank
    /// - **buckets** define the buckets into which observations are counted.
    ///   - Each element in the slice is the upper inclusive bound of a bucket.
    ///   - The values will be deduped and sorted in strictly increasing order.
    ///   - There is no need to add a highest bucket with +Inf bound, it will be added implicitly.
    ///
    /// ## Errors
    /// - if no labels are provided
    /// - if labels are blank
    /// - if any of the constant label names or values are blank
    /// - if there are no buckets defined
    ///
    /// ## Notes
    ///
    pub fn register_counter_vec(
        &self,
        metric_id: MetricId,
        help: String,
        label_ids: &[LabelId],
        const_labels: Option<HashMap<LabelId, String>>,
    ) -> prometheus::Result<prometheus::CounterVec> {
        let label_names = Self::check_labels(label_ids)?;
        let help = Self::check_help(help)?;
        let const_labels = Self::check_const_labels(const_labels)?;

        let mut metrics = self.counter_vecs.write().unwrap();
        if metrics.contains_key(&metric_id) {
            return Err(prometheus::Error::AlreadyReg);
        }

        let mut opts = prometheus::Opts::new(metric_id.name(), help);
        if let Some(const_labels) = const_labels {
            opts = opts.const_labels(const_labels);
        }

        let label_names: Vec<&str> = label_names.iter().map(|label| label.as_str()).collect();
        let metric = prometheus::CounterVec::new(opts, &label_names)?;
        self.registry.register(Box::new(metric.clone()))?;
        metrics.insert(metric_id, metric.clone());
        Ok(metric)
    }

    /// Tries to register a IntCounterVec metric
    ///
    /// ## Params
    /// - **metric_id** ULID is prefixed with 'M' to construct the [metric fully qualified name](https://prometheus.io/docs/concepts/data_model/#metric-names-and-labels)
    ///   - e.g. if the MetricId ULID is *01D1ZMQVMQ5C6Z09JBF32T41ZK*, then the metric name will be **M***01D1ZMQVMQ5C6Z09JBF32T41ZK*
    /// - **help** is mandatory - use it to provide a human friendly name for the metric and provide a short description
    /// - label_names - the labels used to define the metric's dimensions
    ///   - labels will be trimmed and must not be blank
    /// - **buckets** define the buckets into which observations are counted.
    ///   - Each element in the slice is the upper inclusive bound of a bucket.
    ///   - The values will be deduped and sorted in strictly increasing order.
    ///   - There is no need to add a highest bucket with +Inf bound, it will be added implicitly.
    ///
    /// ## Errors
    /// - if no labels are provided
    /// - if labels are blank
    /// - if any of the constant label names or values are blank
    /// - if there are no buckets defined
    ///
    /// ## Notes
    ///
    pub fn register_int_counter_vec(
        &self,
        metric_id: MetricId,
        help: String,
        label_ids: &[LabelId],
        const_labels: Option<HashMap<LabelId, String>>,
    ) -> prometheus::Result<prometheus::IntCounterVec> {
        let label_names = Self::check_labels(label_ids)?;
        let help = Self::check_help(help)?;
        let const_labels = Self::check_const_labels(const_labels)?;

        let mut metrics = self.int_counter_vecs.write().unwrap();
        if metrics.contains_key(&metric_id) {
            return Err(prometheus::Error::AlreadyReg);
        }

        let mut opts = prometheus::Opts::new(metric_id.name(), help);
        if let Some(const_labels) = const_labels {
            opts = opts.const_labels(const_labels);
        }

        let label_names: Vec<&str> = label_names.iter().map(|label| label.as_str()).collect();
        let metric = prometheus::IntCounterVec::new(opts, &label_names)?;
        self.registry.register(Box::new(metric.clone()))?;
        metrics.insert(metric_id, metric.clone());
        Ok(metric)
    }

    /// Tries to register a Histogram metric
    ///
    /// ## Params
    /// - **metric_id** ULID is prefixed with 'M' to construct the [metric fully qualified name](https://prometheus.io/docs/concepts/data_model/#metric-names-and-labels)
    ///   - e.g. if the MetricId ULID is *01D1ZMQVMQ5C6Z09JBF32T41ZK*, then the metric name will be **M***01D1ZMQVMQ5C6Z09JBF32T41ZK*
    /// - **help** is mandatory - use it to provide a human friendly name for the metric and provide a short description
    /// - **buckets** define the buckets into which observations are counted.
    ///   - Each element in the slice is the upper inclusive bound of a bucket.
    ///   - The values will be deduped and sorted in strictly increasing order.
    ///   - There is no need to add a highest bucket with +Inf bound, it will be added implicitly.
    ///
    /// ## Errors
    /// - if no labels are provided
    /// - if any of the constant label names or values are blank
    /// - if there are no buckets defined
    ///
    /// ## Notes
    ///
    pub fn register_histogram(
        &self,
        metric_id: MetricId,
        help: String,
        buckets: Vec<f64>,
        const_labels: Option<HashMap<LabelId, String>>,
    ) -> prometheus::Result<prometheus::Histogram> {
        let help = Self::check_help(help)?;
        let buckets = Self::check_buckets(buckets)?;
        let const_labels = Self::check_const_labels(const_labels)?;

        let mut metrics = self.histograms.write().unwrap();
        if metrics.contains_key(&metric_id) {
            return Err(prometheus::Error::AlreadyReg);
        }

        let mut opts =
            prometheus::HistogramOpts::new(metric_id.name(), help).buckets(buckets.clone());
        if let Some(const_labels) = const_labels {
            opts = opts.const_labels(const_labels);
        }

        let metric = prometheus::Histogram::with_opts(opts)?;
        self.registry.register(Box::new(metric.clone()))?;
        metrics.insert(
            metric_id,
            (metric.clone(), Buckets(SmallVec::from(buckets))),
        );
        Ok(metric)
    }

    /// Tries to register a HistogramVec metric
    ///
    /// ## Params
    /// - **metric_id** ULID is prefixed with 'M' to construct the [metric fully qualified name](https://prometheus.io/docs/concepts/data_model/#metric-names-and-labels)
    ///   - e.g. if the MetricId ULID is *01D1ZMQVMQ5C6Z09JBF32T41ZK*, then the metric name will be **M***01D1ZMQVMQ5C6Z09JBF32T41ZK*
    /// - **help** is mandatory - use it to provide a human friendly name for the metric and provide a short description
    /// - label_names - the labels used to define the metric's dimensions
    ///   - labels will be trimmed and must not be blank
    /// - **buckets** define the buckets into which observations are counted.
    ///   - Each element in the slice is the upper inclusive bound of a bucket.
    ///   - The values will be deduped and sorted in strictly increasing order.
    ///   - There is no need to add a highest bucket with +Inf bound, it will be added implicitly.
    ///
    /// ## Errors
    /// - if no labels are provided
    /// - if labels are blank
    /// - if any of the constant label names or values are blank
    /// - if there are no buckets defined
    ///
    /// ## Notes
    ///
    pub fn register_histogram_vec(
        &self,
        metric_id: MetricId,
        help: String,
        label_ids: &[LabelId],
        buckets: Vec<f64>,
        const_labels: Option<HashMap<LabelId, String>>,
    ) -> prometheus::Result<prometheus::HistogramVec> {
        let label_names = Self::check_labels(label_ids)?;
        let help = Self::check_help(help)?;
        let buckets = Self::check_buckets(buckets)?;
        let const_labels = Self::check_const_labels(const_labels)?;

        let mut metrics = self.histogram_vecs.write().unwrap();
        if metrics.contains_key(&metric_id) {
            return Err(prometheus::Error::AlreadyReg);
        }

        let mut opts =
            prometheus::HistogramOpts::new(metric_id.name(), help).buckets(buckets.clone());
        if let Some(const_labels) = const_labels {
            opts = opts.const_labels(const_labels);
        }

        let label_names: Vec<&str> = label_names.iter().map(|label| label.as_str()).collect();
        let metric = prometheus::HistogramVec::new(opts, &label_names)?;
        self.registry.register(Box::new(metric.clone()))?;
        metrics.insert(
            metric_id,
            (metric.clone(), Buckets(SmallVec::from(buckets))),
        );
        Ok(metric)
    }

    fn check_help(help: String) -> Result<String, prometheus::Error> {
        let help = help.trim();
        if help.is_empty() {
            Err(prometheus::Error::Msg(
                "help is required and cannot be blank".to_string(),
            ))
        } else {
            Ok(help.to_string())
        }
    }

    fn check_const_labels(
        const_labels: Option<HashMap<LabelId, String>>,
    ) -> Result<Option<HashMap<String, String>>, prometheus::Error> {
        match const_labels {
            Some(const_labels) => {
                let mut trimmed_const_labels = HashMap::with_capacity(const_labels.len());
                for (key, value) in const_labels {
                    let key = key.name().to_string();

                    let value = value.trim().to_string();
                    if value.is_empty() {
                        return Err(prometheus::Error::Msg(
                            "Const label value cannot be blank".to_string(),
                        ));
                    }
                    trimmed_const_labels.insert(key, value);
                }
                Ok(Some(trimmed_const_labels))
            }
            None => Ok(None),
        }
    }

    fn check_labels(label_names: &[LabelId]) -> Result<Vec<String>, prometheus::Error> {
        if label_names.is_empty() {
            return Err(prometheus::Error::Msg(
                "At least one label name must be provided".to_string(),
            ));
        }
        Ok(label_names.iter().map(|label| label.name()).collect())
    }

    fn check_buckets(buckets: Vec<f64>) -> Result<Vec<f64>, prometheus::Error> {
        fn sort_dedupe(buckets: Vec<f64>) -> Vec<f64> {
            fn dedupe(buckets: Vec<f64>) -> Vec<f64> {
                let mut buckets = buckets;
                if buckets.len() > 1 {
                    let mut i = 1;
                    let mut found_dups = false;
                    while i < buckets.len() {
                        use std::cmp::Ordering;
                        match buckets[i - 1].partial_cmp(&buckets[i]) {
                            Some(Ordering::Less) => (),
                            _ => {
                                buckets.remove(i);
                                found_dups = true;
                            }
                        }
                        i += 1;
                    }
                    if found_dups {
                        return dedupe(buckets);
                    }
                }
                buckets
            }

            fn sort(buckets: Vec<f64>) -> Vec<f64> {
                let mut buckets = buckets;
                buckets.sort_unstable_by(|a, b| {
                    use std::cmp::Ordering;
                    if a < b {
                        return Ordering::Less;
                    }

                    if a > b {
                        return Ordering::Greater;
                    }

                    Ordering::Equal
                });

                buckets
            }

            dedupe(sort(buckets))
        }

        if buckets.is_empty() {
            return Err(prometheus::Error::Msg(
                "At least 1 bucket must be defined".to_string(),
            ));
        }
        Ok(sort_dedupe(buckets))
    }

    /// Text encodes a snapshot of the current metrics
    pub fn text_encode_metrics<W: Write>(&self, writer: &mut W) -> prometheus::Result<()> {
        let metric_families = self.registry.gather();
        let encoder = prometheus::TextEncoder::new();
        encoder.encode(&metric_families, writer)
    }

    /// Returns a HistogramVec for the specified metric ID - if it is registered
    pub fn histogram_vec(&self, metric_id: &MetricId) -> Option<prometheus::HistogramVec> {
        let histogram_vecs = self.histogram_vecs.read().unwrap();
        histogram_vecs
            .get(&metric_id)
            .map(|(metric, _opts)| metric.clone())
    }

    /// Returns a Histogram for the specified metric ID - if it is registered
    pub fn histogram(&self, metric_id: &MetricId) -> Option<prometheus::Histogram> {
        let histograms = self.histograms.read().unwrap();
        histograms
            .get(&metric_id)
            .map(|(metric, _opts)| metric.clone())
    }

    /// Returns a Counter for the specified metric ID - if it is registered
    pub fn counter(&self, metric_id: &MetricId) -> Option<prometheus::Counter> {
        let counters = self.counters.read().unwrap();
        counters.get(&metric_id).cloned()
    }

    /// Returns an IntCounter for the specified metric ID - if it is registered
    pub fn int_counter(&self, metric_id: &MetricId) -> Option<prometheus::IntCounter> {
        let counters = self.int_counters.read().unwrap();
        counters.get(&metric_id).cloned()
    }

    /// Returns a CounterVec for the specified metric ID - if it is registered
    pub fn counter_vec(&self, metric_id: &MetricId) -> Option<prometheus::CounterVec> {
        let counter_vecs = self.counter_vecs.read().unwrap();
        counter_vecs.get(&metric_id).cloned()
    }

    /// Returns a CounterVec for the specified metric ID - if it is registered
    pub fn int_counter_vec(&self, metric_id: &MetricId) -> Option<prometheus::IntCounterVec> {
        let int_counter_vecs = self.int_counter_vecs.read().unwrap();
        int_counter_vecs.get(&metric_id).cloned()
    }

    /// Returns a Counter for the specified metric ID - if it is registered
    pub fn gauge(&self, metric_id: &MetricId) -> Option<prometheus::Gauge> {
        let gauges = self.gauges.read().unwrap();
        gauges.get(&metric_id).cloned()
    }

    /// Returns an IntCounter for the specified metric ID - if it is registered
    pub fn int_gauge(&self, metric_id: &MetricId) -> Option<prometheus::IntGauge> {
        let int_gauges = self.int_gauges.read().unwrap();
        int_gauges.get(&metric_id).cloned()
    }

    /// Returns a CounterVec for the specified metric ID - if it is registered
    pub fn gauge_vec(&self, metric_id: &MetricId) -> Option<prometheus::GaugeVec> {
        let gauges_vecs = self.gauge_vecs.read().unwrap();
        gauges_vecs.get(&metric_id).cloned()
    }

    /// Returns a CounterVec for the specified metric ID - if it is registered
    pub fn int_gauge_vec(&self, metric_id: &MetricId) -> Option<prometheus::IntGaugeVec> {
        let int_gauge_vecs = self.int_gauge_vecs.read().unwrap();
        int_gauge_vecs.get(&metric_id).cloned()
    }

    /// gather calls the Collect method of the registered Collectors and then gathers the collected
    /// metrics into a lexicographically sorted slice of MetricFamily protobufs.
    pub fn gather(&self) -> Vec<prometheus::proto::MetricFamily> {
        self.registry.gather()
    }
    /// Gathers the specified metrics
    pub fn gather_metrics(&self, metric_ids: &[MetricId]) -> Metrics {
        fn gather_histogram_metrics(
            registry: &MetricRegistry,
            metric_ids: &[MetricId],
            metrics: &mut Metrics,
        ) {
            let histograms = registry.histograms.read().unwrap();
            for metric_id in metric_ids {
                if let Some((metric, buckets)) = histograms.get(metric_id) {
                    let histogram = Metric::histogram(*metric_id, metric, buckets.clone());
                    metrics.metrics.push(histogram);
                }
            }
        }

        fn gather_histogram_vec_metrics(
            registry: &MetricRegistry,
            metric_ids: &[MetricId],
            metrics: &mut Metrics,
        ) {
            let histogram_vecs = registry.histogram_vecs.read().unwrap();
            for metric_id in metric_ids {
                if let Some((metric, buckets)) = histogram_vecs.get(metric_id) {
                    let histogram_vec = Metric::histogram_vec(*metric_id, metric, buckets.clone());
                    metrics.metrics.push(histogram_vec);
                }
            }
        }

        fn gather_gauge_metrics(
            registry: &MetricRegistry,
            metric_ids: &[MetricId],
            metrics: &mut Metrics,
        ) {
            let gauges = registry.gauges.read().unwrap();
            for metric_id in metric_ids {
                if let Some(metric) = gauges.get(metric_id) {
                    let desc = MetricDesc::gauge_metric_desc(*metric_id, metric);
                    let value = metric.get();
                    metrics.metrics.push(Metric::Gauge { desc, value });
                }
            }
        }

        fn gather_int_gauge_metrics(
            registry: &MetricRegistry,
            metric_ids: &[MetricId],
            metrics: &mut Metrics,
        ) {
            let gauges = registry.int_gauges.read().unwrap();
            for metric_id in metric_ids {
                if let Some(metric) = gauges.get(metric_id) {
                    let desc = MetricDesc::int_gauge_metric_desc(*metric_id, metric);
                    let value: u64 = metric.get() as u64;
                    metrics.metrics.push(Metric::IntGauge { desc, value });
                }
            }
        }

        fn gather_gauge_vec_metrics(
            registry: &MetricRegistry,
            metric_ids: &[MetricId],
            metrics: &mut Metrics,
        ) {
            let gauge_vecs = registry.gauge_vecs.read().unwrap();
            for metric_id in metric_ids {
                if let Some(metric) = gauge_vecs.get(metric_id) {
                    metrics.metrics.push(Metric::gauge_vec(*metric_id, metric));
                }
            }
        }

        fn gather_int_gauge_vec_metrics(
            registry: &MetricRegistry,
            metric_ids: &[MetricId],
            metrics: &mut Metrics,
        ) {
            let int_gauge_vecs = registry.int_gauge_vecs.read().unwrap();
            for metric_id in metric_ids {
                if let Some(metric) = int_gauge_vecs.get(metric_id) {
                    metrics
                        .metrics
                        .push(Metric::int_gauge_vec(*metric_id, metric));
                }
            }
        }

        fn gather_counter_metrics(
            registry: &MetricRegistry,
            metric_ids: &[MetricId],
            metrics: &mut Metrics,
        ) {
            let counters = registry.counters.read().unwrap();
            for metric_id in metric_ids {
                if let Some(metric) = counters.get(metric_id) {
                    let desc = MetricDesc::counter_metric_desc(*metric_id, metric);
                    let value = metric.get();
                    metrics.metrics.push(Metric::Counter { desc, value });
                }
            }
        }

        fn gather_int_counter_metrics(
            registry: &MetricRegistry,
            metric_ids: &[MetricId],
            metrics: &mut Metrics,
        ) {
            let counters = registry.int_counters.read().unwrap();
            for metric_id in metric_ids {
                if let Some(metric) = counters.get(metric_id) {
                    let desc = MetricDesc::int_counter_metric_desc(*metric_id, metric);
                    let value: u64 = metric.get() as u64;
                    metrics.metrics.push(Metric::IntCounter { desc, value });
                }
            }
        }

        fn gather_counter_vec_metrics(
            registry: &MetricRegistry,
            metric_ids: &[MetricId],
            metrics: &mut Metrics,
        ) {
            let counter_vecs = registry.counter_vecs.read().unwrap();
            for metric_id in metric_ids {
                if let Some(metric) = counter_vecs.get(metric_id) {
                    metrics
                        .metrics
                        .push(Metric::counter_vec(*metric_id, metric));
                }
            }
        }

        fn gather_int_counter_vec_metrics(
            registry: &MetricRegistry,
            metric_ids: &[MetricId],
            metrics: &mut Metrics,
        ) {
            let int_counter_vecs = registry.int_counter_vecs.read().unwrap();
            for metric_id in metric_ids {
                if let Some(metric) = int_counter_vecs.get(metric_id) {
                    metrics
                        .metrics
                        .push(Metric::int_counter_vec(*metric_id, metric));
                }
            }
        }

        let mut metrics = Metrics::new(metric_ids.len());

        gather_counter_metrics(self, metric_ids, &mut metrics);
        gather_int_counter_metrics(self, metric_ids, &mut metrics);
        gather_counter_vec_metrics(self, metric_ids, &mut metrics);
        gather_int_counter_vec_metrics(self, metric_ids, &mut metrics);

        gather_gauge_metrics(self, metric_ids, &mut metrics);
        gather_int_gauge_metrics(self, metric_ids, &mut metrics);
        gather_gauge_vec_metrics(self, metric_ids, &mut metrics);
        gather_int_gauge_vec_metrics(self, metric_ids, &mut metrics);

        gather_histogram_metrics(self, metric_ids, &mut metrics);
        gather_histogram_vec_metrics(self, metric_ids, &mut metrics);
        metrics
    }

    /// Gathers all metrics that are registered using MetricId(s)
    /// - this means process metrics are excluded
    pub fn gather_all_metrics(&self) -> Metrics {
        fn gather_histogram_metrics(registry: &MetricRegistry, metrics: &mut Metrics) {
            let histograms = registry.histograms.read().unwrap();
            for (metric_id, (metric, buckets)) in histograms.iter() {
                let histogram = Metric::histogram(*metric_id, metric, buckets.clone());
                metrics.metrics.push(histogram);
            }
        }

        fn gather_histogram_vec_metrics(registry: &MetricRegistry, metrics: &mut Metrics) {
            let histogram_vecs = registry.histogram_vecs.read().unwrap();
            for (metric_id, (metric, buckets)) in histogram_vecs.iter() {
                let histogram_vec = Metric::histogram_vec(*metric_id, metric, buckets.clone());
                metrics.metrics.push(histogram_vec);
            }
        }

        fn gather_gauge_metrics(registry: &MetricRegistry, metrics: &mut Metrics) {
            let gauges = registry.gauges.read().unwrap();
            for (metric_id, metric) in gauges.iter() {
                let desc = MetricDesc::gauge_metric_desc(*metric_id, metric);
                let value = metric.get();
                metrics.metrics.push(Metric::Gauge { desc, value });
            }
        }

        fn gather_int_gauge_metrics(registry: &MetricRegistry, metrics: &mut Metrics) {
            let gauges = registry.int_gauges.read().unwrap();
            for (metric_id, metric) in gauges.iter() {
                let desc = MetricDesc::int_gauge_metric_desc(*metric_id, metric);
                let value: u64 = metric.get() as u64;
                metrics.metrics.push(Metric::IntGauge { desc, value });
            }
        }

        fn gather_gauge_vec_metrics(registry: &MetricRegistry, metrics: &mut Metrics) {
            let gauge_vecs = registry.gauge_vecs.read().unwrap();
            for (metric_id, metric) in gauge_vecs.iter() {
                metrics.metrics.push(Metric::gauge_vec(*metric_id, metric));
            }
        }

        fn gather_int_gauge_vec_metrics(registry: &MetricRegistry, metrics: &mut Metrics) {
            let int_gauge_vecs = registry.int_gauge_vecs.read().unwrap();
            for (metric_id, metric) in int_gauge_vecs.iter() {
                metrics
                    .metrics
                    .push(Metric::int_gauge_vec(*metric_id, metric));
            }
        }

        fn gather_counter_metrics(registry: &MetricRegistry, metrics: &mut Metrics) {
            let counters = registry.counters.read().unwrap();
            for (metric_id, metric) in counters.iter() {
                let desc = MetricDesc::counter_metric_desc(*metric_id, metric);
                let value = metric.get();
                metrics.metrics.push(Metric::Counter { desc, value });
            }
        }

        fn gather_int_counter_metrics(registry: &MetricRegistry, metrics: &mut Metrics) {
            let counters = registry.int_counters.read().unwrap();
            for (metric_id, metric) in counters.iter() {
                let desc = MetricDesc::int_counter_metric_desc(*metric_id, metric);
                let value: u64 = metric.get() as u64;
                metrics.metrics.push(Metric::IntCounter { desc, value });
            }
        }

        fn gather_counter_vec_metrics(registry: &MetricRegistry, metrics: &mut Metrics) {
            let counter_vecs = registry.counter_vecs.read().unwrap();
            for (metric_id, metric) in counter_vecs.iter() {
                metrics
                    .metrics
                    .push(Metric::counter_vec(*metric_id, metric));
            }
        }

        fn gather_int_counter_vec_metrics(registry: &MetricRegistry, metrics: &mut Metrics) {
            let int_counter_vecs = registry.int_counter_vecs.read().unwrap();
            for (metric_id, metric) in int_counter_vecs.iter() {
                metrics
                    .metrics
                    .push(Metric::int_counter_vec(*metric_id, metric));
            }
        }

        let mut metrics = Metrics::new(16);

        gather_counter_metrics(self, &mut metrics);
        gather_int_counter_metrics(self, &mut metrics);
        gather_counter_vec_metrics(self, &mut metrics);
        gather_int_counter_vec_metrics(self, &mut metrics);

        gather_gauge_metrics(self, &mut metrics);
        gather_int_gauge_metrics(self, &mut metrics);
        gather_gauge_vec_metrics(self, &mut metrics);
        gather_int_gauge_vec_metrics(self, &mut metrics);

        gather_histogram_metrics(self, &mut metrics);
        gather_histogram_vec_metrics(self, &mut metrics);
        metrics
    }

    /// Gathers process related metrics
    pub fn gather_process_metrics(&self) -> ProcessMetrics {
        ProcessMetrics::collect(&self.process_collector)
    }

    /// returns the descriptors for registered metrics
    /// - this exludes the process collector metrics
    pub fn metric_descs(&self) -> MetricDescs {
        let counters: Option<Vec<MetricDesc>> = {
            let metrics = self.counters.read().unwrap();
            if metrics.is_empty() {
                None
            } else {
                let descs = metrics
                    .iter()
                    .map(|(id, metric)| MetricDesc::counter_metric_desc(*id, metric))
                    .collect();
                Some(descs)
            }
        };

        let int_counters: Option<Vec<MetricDesc>> = {
            let metrics = self.int_counters.read().unwrap();
            if metrics.is_empty() {
                None
            } else {
                let descs = metrics
                    .iter()
                    .map(|(id, metric)| MetricDesc::int_counter_metric_desc(*id, metric))
                    .collect();
                Some(descs)
            }
        };

        let counter_vecs: Option<Vec<MetricVecDesc>> = {
            let metrics = self.counter_vecs.read().unwrap();
            if metrics.is_empty() {
                None
            } else {
                let descs = metrics
                    .iter()
                    .map(|(id, metric)| MetricVecDesc::counter_vec_metric_desc(*id, metric))
                    .collect();
                Some(descs)
            }
        };

        let int_counter_vecs: Option<Vec<MetricVecDesc>> = {
            let metrics = self.int_counter_vecs.read().unwrap();
            if metrics.is_empty() {
                None
            } else {
                let descs = metrics
                    .iter()
                    .map(|(id, metric)| MetricVecDesc::int_counter_vec_metric_desc(*id, metric))
                    .collect();
                Some(descs)
            }
        };

        let gauges: Option<Vec<MetricDesc>> = {
            let metrics = self.gauges.read().unwrap();
            if metrics.is_empty() {
                None
            } else {
                let descs = metrics
                    .iter()
                    .map(|(id, metric)| MetricDesc::gauge_metric_desc(*id, metric))
                    .collect();
                Some(descs)
            }
        };

        let int_gauges: Option<Vec<MetricDesc>> = {
            let metrics = self.int_gauges.read().unwrap();
            if metrics.is_empty() {
                None
            } else {
                let descs = metrics
                    .iter()
                    .map(|(id, metric)| MetricDesc::int_gauge_metric_desc(*id, metric))
                    .collect();
                Some(descs)
            }
        };

        let gauge_vecs: Option<Vec<MetricVecDesc>> = {
            let metrics = self.gauge_vecs.read().unwrap();
            if metrics.is_empty() {
                None
            } else {
                let descs = metrics
                    .iter()
                    .map(|(id, metric)| MetricVecDesc::gauge_vec_metric_desc(*id, metric))
                    .collect();
                Some(descs)
            }
        };

        let int_gauge_vecs: Option<Vec<MetricVecDesc>> = {
            let metrics = self.int_gauge_vecs.read().unwrap();
            if metrics.is_empty() {
                None
            } else {
                let descs = metrics
                    .iter()
                    .map(|(id, metric)| MetricVecDesc::int_gauge_vec_metric_desc(*id, metric))
                    .collect();
                Some(descs)
            }
        };

        let histograms: Option<Vec<HistogramDesc>> = {
            let metrics = self.histograms.read().unwrap();
            if metrics.is_empty() {
                None
            } else {
                let descs = metrics
                    .iter()
                    .map(|(id, &(ref metric, ref buckets))| {
                        HistogramDesc::new(*id, metric, buckets.clone())
                    })
                    .collect();
                Some(descs)
            }
        };

        let histogram_vecs: Option<Vec<HistogramVecDesc>> = {
            let metrics = self.histogram_vecs.read().unwrap();
            if metrics.is_empty() {
                None
            } else {
                let descs = metrics
                    .iter()
                    .map(|(id, &(ref metric, ref buckets))| {
                        HistogramVecDesc::new(*id, metric, buckets.clone())
                    })
                    .collect();
                Some(descs)
            }
        };

        MetricDescs {
            counters,
            int_counters,
            counter_vecs,
            int_counter_vecs,

            gauges,
            int_gauges,
            gauge_vecs,
            int_gauge_vecs,
            histograms,
            histogram_vecs,
        }
    }
}

impl fmt::Debug for MetricRegistry {
    /// TODO: the output is clunky - make it cleaner - perhaps a JSON view
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.metric_descs())
    }
}

impl Default for MetricRegistry {
    fn default() -> Self {
        let registry = prometheus::Registry::new();
        let process_collector = ProcessCollector::default();
        registry
            .register(Box::new(process_collector.clone()))
            .unwrap();
        Self {
            registry,
            int_counters: RwLock::new(fnv::FnvHashMap::default()),
            int_counter_vecs: RwLock::new(fnv::FnvHashMap::default()),
            counters: RwLock::new(fnv::FnvHashMap::default()),
            counter_vecs: RwLock::new(fnv::FnvHashMap::default()),

            gauges: RwLock::new(fnv::FnvHashMap::default()),
            gauge_vecs: RwLock::new(fnv::FnvHashMap::default()),
            int_gauges: RwLock::new(fnv::FnvHashMap::default()),
            int_gauge_vecs: RwLock::new(fnv::FnvHashMap::default()),

            histogram_vecs: RwLock::new(fnv::FnvHashMap::default()),
            histograms: RwLock::new(fnv::FnvHashMap::default()),

            process_collector,
        }
    }
}

/// Metric Desc
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricDesc {
    id: MetricId,
    help: String,
    const_labels: Option<Vec<(LabelId, String)>>,
}

impl MetricDesc {
    /// returns the MetricId
    pub fn id(&self) -> MetricId {
        self.id
    }

    /// returns the metric help
    pub fn help(&self) -> &str {
        &self.help
    }

    /// returns the metric's constant labels
    pub fn const_labels(&self) -> Option<&[(LabelId, String)]> {
        self.const_labels.as_ref().map(|labels| labels.as_slice())
    }

    // TODO: the constructors are good candidates for macros

    /// MetricDesc constructor for a Gauge
    fn gauge_metric_desc(metric_id: MetricId, metric: &prometheus::Gauge) -> MetricDesc {
        let desc = metric.desc()[0];
        let const_labels = if desc.const_label_pairs.is_empty() {
            None
        } else {
            Some(
                desc.const_label_pairs
                    .iter()
                    .map(|label_pair| {
                        let label_id: LabelId = label_pair.get_name().parse().unwrap();
                        let label_value = label_pair.get_value().to_string();
                        (label_id, label_value)
                    })
                    .collect(),
            )
        };
        MetricDesc {
            id: metric_id,
            help: desc.help.clone(),
            const_labels,
        }
    }

    /// MetricDesc constructor for an IntGauge
    fn int_gauge_metric_desc(metric_id: MetricId, metric: &prometheus::IntGauge) -> MetricDesc {
        let desc = metric.desc()[0];
        let const_labels = if desc.const_label_pairs.is_empty() {
            None
        } else {
            Some(
                desc.const_label_pairs
                    .iter()
                    .map(|label_pair| {
                        let label_id: LabelId = label_pair.get_name().parse().unwrap();
                        let label_value = label_pair.get_value().to_string();
                        (label_id, label_value)
                    })
                    .collect(),
            )
        };
        MetricDesc {
            id: metric_id,
            help: desc.help.clone(),
            const_labels,
        }
    }

    /// MetricDesc constructor for a Counter
    fn counter_metric_desc(metric_id: MetricId, metric: &prometheus::Counter) -> MetricDesc {
        let desc = metric.desc()[0];
        let const_labels = if desc.const_label_pairs.is_empty() {
            None
        } else {
            Some(
                desc.const_label_pairs
                    .iter()
                    .map(|label_pair| {
                        let label_id: LabelId = label_pair.get_name().parse().unwrap();
                        let label_value = label_pair.get_value().to_string();
                        (label_id, label_value)
                    })
                    .collect(),
            )
        };
        MetricDesc {
            id: metric_id,
            help: desc.help.clone(),
            const_labels,
        }
    }

    /// MetricDesc constructor for an IntCounter
    fn int_counter_metric_desc(metric_id: MetricId, metric: &prometheus::IntCounter) -> MetricDesc {
        let desc = metric.desc()[0];
        let const_labels = if desc.const_label_pairs.is_empty() {
            None
        } else {
            Some(
                desc.const_label_pairs
                    .iter()
                    .map(|label_pair| {
                        let label_id: LabelId = label_pair.get_name().parse().unwrap();
                        let label_value = label_pair.get_value().to_string();
                        (label_id, label_value)
                    })
                    .collect(),
            )
        };
        MetricDesc {
            id: metric_id,
            help: desc.help.clone(),
            const_labels,
        }
    }
}

impl From<&prometheus::Counter> for MetricDesc {
    /// ## Panics
    /// If the metric name fails to parse as a MetricId
    fn from(counter: &prometheus::Counter) -> Self {
        let desc = counter.desc()[0];
        let metric_id = desc.fq_name.as_str().parse::<MetricId>().unwrap();
        MetricDesc::counter_metric_desc(metric_id, counter)
    }
}

impl From<&prometheus::IntCounter> for MetricDesc {
    /// ## Panics
    /// If the metric name fails to parse as a MetricId
    fn from(counter: &prometheus::IntCounter) -> Self {
        let desc = counter.desc()[0];
        let metric_id = desc.fq_name.as_str().parse::<MetricId>().unwrap();
        MetricDesc::int_counter_metric_desc(metric_id, counter)
    }
}

impl From<&prometheus::Gauge> for MetricDesc {
    /// ## Panics
    /// If the metric name fails to parse as a MetricId
    fn from(gauge: &prometheus::Gauge) -> Self {
        let desc = gauge.desc()[0];
        let metric_id = desc.fq_name.as_str().parse::<MetricId>().unwrap();
        MetricDesc::gauge_metric_desc(metric_id, gauge)
    }
}

impl From<&prometheus::IntGauge> for MetricDesc {
    /// ## Panics
    /// If the metric name fails to parse as a MetricId
    fn from(gauge: &prometheus::IntGauge) -> Self {
        let desc = gauge.desc()[0];
        let metric_id = desc.fq_name.as_str().parse::<MetricId>().unwrap();
        MetricDesc::int_gauge_metric_desc(metric_id, gauge)
    }
}

/// Metric Desc
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricVecDesc {
    id: MetricId,
    help: String,
    labels: SmallVec<[LabelId; METRIC_DESC_SMALLVEC_SIZE]>,
    const_labels: Option<Vec<(LabelId, String)>>,
}

impl MetricVecDesc {
    /// returns the MetricId
    pub fn id(&self) -> MetricId {
        self.id
    }

    /// returns the metric help
    pub fn help(&self) -> &str {
        &self.help
    }

    /// returns the metric's constant labels
    pub fn const_labels(&self) -> Option<&[(LabelId, String)]> {
        self.const_labels.as_ref().map(|labels| labels.as_slice())
    }

    /// returns the metric's dimension labels
    pub fn labels(&self) -> &[LabelId] {
        self.labels.as_slice()
    }

    /// MetricDesc constructor for a GaugeVec
    fn gauge_vec_metric_desc(metric_id: MetricId, metric: &prometheus::GaugeVec) -> MetricVecDesc {
        let desc = metric.desc()[0];
        let const_labels = if desc.const_label_pairs.is_empty() {
            None
        } else {
            Some(
                desc.const_label_pairs
                    .iter()
                    .map(|label_pair| {
                        let label_id: LabelId = label_pair.get_name().parse().unwrap();
                        let label_value = label_pair.get_value().to_string();
                        (label_id, label_value)
                    })
                    .collect(),
            )
        };
        let labels = desc
            .variable_labels
            .iter()
            .map(|label| {
                let label_id: LabelId = label.parse().unwrap();
                label_id
            })
            .collect();
        MetricVecDesc {
            id: metric_id,
            help: desc.help.clone(),
            labels,
            const_labels,
        }
    }

    /// MetricDesc constructor for an IntGaugeVec
    fn int_gauge_vec_metric_desc(
        metric_id: MetricId,
        metric: &prometheus::IntGaugeVec,
    ) -> MetricVecDesc {
        let desc = metric.desc()[0];
        let const_labels = if desc.const_label_pairs.is_empty() {
            None
        } else {
            Some(
                desc.const_label_pairs
                    .iter()
                    .map(|label_pair| {
                        let label_id: LabelId = label_pair.get_name().parse().unwrap();
                        let label_value = label_pair.get_value().to_string();
                        (label_id, label_value)
                    })
                    .collect(),
            )
        };
        let labels = desc
            .variable_labels
            .iter()
            .map(|label| {
                let label_id: LabelId = label.parse().unwrap();
                label_id
            })
            .collect();
        MetricVecDesc {
            id: metric_id,
            help: desc.help.clone(),
            labels,
            const_labels,
        }
    }

    /// MetricDesc constructor for a CounterVec
    fn counter_vec_metric_desc(
        metric_id: MetricId,
        metric: &prometheus::CounterVec,
    ) -> MetricVecDesc {
        let desc = metric.desc()[0];
        let const_labels = if desc.const_label_pairs.is_empty() {
            None
        } else {
            Some(
                desc.const_label_pairs
                    .iter()
                    .map(|label_pair| {
                        let label_id: LabelId = label_pair.get_name().parse().unwrap();
                        let label_value = label_pair.get_value().to_string();
                        (label_id, label_value)
                    })
                    .collect(),
            )
        };
        let labels = desc
            .variable_labels
            .iter()
            .map(|label| {
                let label_id: LabelId = label.parse().unwrap();
                label_id
            })
            .collect();
        MetricVecDesc {
            id: metric_id,
            help: desc.help.clone(),
            labels,
            const_labels,
        }
    }

    /// MetricDesc constructor for an IntCounterVec
    fn int_counter_vec_metric_desc(
        metric_id: MetricId,
        metric: &prometheus::IntCounterVec,
    ) -> MetricVecDesc {
        let desc = metric.desc()[0];
        let const_labels = if desc.const_label_pairs.is_empty() {
            None
        } else {
            Some(
                desc.const_label_pairs
                    .iter()
                    .map(|label_pair| {
                        let label_id: LabelId = label_pair.get_name().parse().unwrap();
                        let label_value = label_pair.get_value().to_string();
                        (label_id, label_value)
                    })
                    .collect(),
            )
        };
        let labels = desc
            .variable_labels
            .iter()
            .map(|label| {
                let label_id: LabelId = label.parse().unwrap();
                label_id
            })
            .collect();
        MetricVecDesc {
            id: metric_id,
            help: desc.help.clone(),
            labels,
            const_labels,
        }
    }
}

impl From<&prometheus::IntCounterVec> for MetricVecDesc {
    /// ## Panics
    /// If the metric name fails to parse as a MetricId
    fn from(counter: &prometheus::IntCounterVec) -> Self {
        let desc = counter.desc()[0];
        let metric_id = desc.fq_name.as_str().parse::<MetricId>().unwrap();
        MetricVecDesc::int_counter_vec_metric_desc(metric_id, counter)
    }
}

impl From<&prometheus::CounterVec> for MetricVecDesc {
    /// ## Panics
    /// If the metric name fails to parse as a MetricId
    fn from(counter: &prometheus::CounterVec) -> Self {
        let desc = counter.desc()[0];
        let metric_id = desc.fq_name.as_str().parse::<MetricId>().unwrap();
        MetricVecDesc::counter_vec_metric_desc(metric_id, counter)
    }
}

impl From<&prometheus::IntGaugeVec> for MetricVecDesc {
    /// ## Panics
    /// If the metric name fails to parse as a MetricId
    fn from(gauge: &prometheus::IntGaugeVec) -> Self {
        let desc = gauge.desc()[0];
        let metric_id = desc.fq_name.as_str().parse::<MetricId>().unwrap();
        MetricVecDesc::int_gauge_vec_metric_desc(metric_id, gauge)
    }
}

impl From<&prometheus::GaugeVec> for MetricVecDesc {
    /// ## Panics
    /// If the metric name fails to parse as a MetricId
    fn from(gauge: &prometheus::GaugeVec) -> Self {
        let desc = gauge.desc()[0];
        let metric_id = desc.fq_name.as_str().parse::<MetricId>().unwrap();
        MetricVecDesc::gauge_vec_metric_desc(metric_id, gauge)
    }
}

/// Histogram Desc
#[derive(Clone, Serialize, Deserialize)]
pub struct HistogramDesc {
    id: MetricId,
    help: String,
    buckets: Buckets,
    const_labels: Option<Vec<(LabelId, String)>>,
}
impl HistogramDesc {
    /// returns the MetricId
    pub fn id(&self) -> MetricId {
        self.id
    }

    /// returns the metric help
    pub fn help(&self) -> &str {
        &self.help
    }

    /// returns the metric's constant labels
    pub fn const_labels(&self) -> Option<&[(LabelId, String)]> {
        self.const_labels.as_ref().map(|labels| labels.as_slice())
    }

    /// returns the histogram's buckets
    pub fn buckets(&self) -> &Buckets {
        &self.buckets
    }

    /// constructor
    fn new(metric_id: MetricId, metric: &prometheus::Histogram, buckets: Buckets) -> HistogramDesc {
        let desc = metric.desc()[0];
        //        let metric_id_ : MetricId = desc.fq_name.as_str().parse().unwrap();
        let const_labels = if desc.const_label_pairs.is_empty() {
            None
        } else {
            Some(
                desc.const_label_pairs
                    .iter()
                    .map(|label_pair| {
                        let label_id: LabelId = label_pair.get_name().parse().unwrap();
                        let label_value = label_pair.get_value().to_string();
                        (label_id, label_value)
                    })
                    .collect(),
            )
        };
        HistogramDesc {
            id: metric_id,
            help: desc.help.clone(),
            const_labels,
            buckets,
        }
    }
}

impl From<(&prometheus::Histogram, Buckets)> for HistogramDesc {
    /// ## Panics
    /// If the metric name fails to parse as a MetricId
    fn from((histogram, buckets): (&prometheus::Histogram, Buckets)) -> Self {
        let desc = histogram.desc()[0];
        let metric_id = desc.fq_name.as_str().parse::<MetricId>().unwrap();
        HistogramDesc::new(metric_id, histogram, buckets)
    }
}

impl fmt::Debug for HistogramDesc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.const_labels.as_ref() {
            Some(const_labels) => write!(
                f,
                "id = {}, help = {}, buckets = {:?}, const_labels = {:?}",
                self.id, self.help, self.buckets, const_labels
            ),
            None => write!(
                f,
                "id = {}, help = {}, buckets = {:?}",
                self.id, self.help, self.buckets
            ),
        }
    }
}

/// HistogramVec Desc
#[derive(Clone, Serialize, Deserialize)]
pub struct HistogramVecDesc {
    id: MetricId,
    help: String,
    labels: Vec<LabelId>,
    buckets: Buckets,
    const_labels: Option<Vec<(LabelId, String)>>,
}

impl HistogramVecDesc {
    /// returns the MetricId
    pub fn id(&self) -> MetricId {
        self.id
    }

    /// returns the metric help
    pub fn help(&self) -> &str {
        &self.help
    }

    /// returns the metric's constant labels
    pub fn const_labels(&self) -> Option<&[(LabelId, String)]> {
        self.const_labels.as_ref().map(|labels| labels.as_slice())
    }

    /// returns the metric's dimension labels
    pub fn labels(&self) -> &[LabelId] {
        self.labels.as_slice()
    }

    /// returns the histogram's buckets
    pub fn buckets(&self) -> &Buckets {
        &self.buckets
    }

    /// constructor
    fn new(metric_id: MetricId, metric: &prometheus::HistogramVec, buckets: Buckets) -> Self {
        let desc = metric.desc()[0];
        let const_labels = if desc.const_label_pairs.is_empty() {
            None
        } else {
            Some(
                desc.const_label_pairs
                    .iter()
                    .map(|label_pair| {
                        let label_id: LabelId = label_pair.get_name().parse().unwrap();
                        let label_value = label_pair.get_value().to_string();
                        (label_id, label_value)
                    })
                    .collect(),
            )
        };
        let labels = desc
            .variable_labels
            .iter()
            .map(|label| {
                let label_id: LabelId = label.parse().unwrap();
                label_id
            })
            .collect();
        HistogramVecDesc {
            id: metric_id,
            help: desc.help.clone(),
            const_labels,
            buckets,
            labels,
        }
    }
}

impl From<(&prometheus::HistogramVec, Buckets)> for HistogramVecDesc {
    /// ## Panics
    /// If the metric name fails to parse as a MetricId
    fn from((histogram, buckets): (&prometheus::HistogramVec, Buckets)) -> Self {
        let desc = histogram.desc()[0];
        let metric_id = desc.fq_name.as_str().parse::<MetricId>().unwrap();
        HistogramVecDesc::new(metric_id, histogram, buckets)
    }
}

impl fmt::Debug for HistogramVecDesc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.const_labels.as_ref() {
            Some(const_labels) => write!(
                f,
                "id = {}, help = {}, buckets = {:?}, labels = {:?}, const_labels = {:?}",
                self.id, self.help, self.buckets, self.labels, const_labels
            ),
            None => write!(
                f,
                "id = {}, help = {}, buckets = {:?}, labels = {:?}",
                self.id, self.help, self.buckets, self.labels
            ),
        }
    }
}

/// Metric descriptors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricDescs {
    counters: Option<Vec<MetricDesc>>,
    int_counters: Option<Vec<MetricDesc>>,
    counter_vecs: Option<Vec<MetricVecDesc>>,
    int_counter_vecs: Option<Vec<MetricVecDesc>>,

    gauges: Option<Vec<MetricDesc>>,
    int_gauges: Option<Vec<MetricDesc>>,
    gauge_vecs: Option<Vec<MetricVecDesc>>,
    int_gauge_vecs: Option<Vec<MetricVecDesc>>,

    histograms: Option<Vec<HistogramDesc>>,
    histogram_vecs: Option<Vec<HistogramVecDesc>>,
}

impl MetricDescs {
    /// returns descriptors for registered metrics
    pub fn histograms(&self) -> Option<&[HistogramDesc]> {
        self.histograms
            .as_ref()
            .map(|histograms| histograms.as_slice())
    }

    /// returns descriptors for registered metrics
    pub fn histogram_vecs(&self) -> Option<&[HistogramVecDesc]> {
        self.histogram_vecs
            .as_ref()
            .map(|histogram_vecs| histogram_vecs.as_slice())
    }

    /// returns descriptors for registered metrics
    pub fn gauges(&self) -> Option<&[MetricDesc]> {
        self.gauges.as_ref().map(|gauges| gauges.as_slice())
    }

    /// returns descriptors for registered metrics
    pub fn int_gauges(&self) -> Option<&[MetricDesc]> {
        self.int_gauges
            .as_ref()
            .map(|int_gauges| int_gauges.as_slice())
    }

    /// returns descriptors for registered metrics
    pub fn gauge_vecs(&self) -> Option<&[MetricVecDesc]> {
        self.gauge_vecs
            .as_ref()
            .map(|gauge_vecs| gauge_vecs.as_slice())
    }

    /// returns descriptors for registered metrics
    pub fn int_gauge_vecs(&self) -> Option<&[MetricVecDesc]> {
        self.int_gauge_vecs
            .as_ref()
            .map(|int_gauge_vecs| int_gauge_vecs.as_slice())
    }

    /// returns descriptors for registered metrics
    pub fn counters(&self) -> Option<&[MetricDesc]> {
        self.counters.as_ref().map(|counters| counters.as_slice())
    }

    /// returns descriptors for registered metrics
    pub fn int_counters(&self) -> Option<&[MetricDesc]> {
        self.int_counters
            .as_ref()
            .map(|counters| counters.as_slice())
    }

    /// returns descriptors for registered metrics
    pub fn counter_vecs(&self) -> Option<&[MetricVecDesc]> {
        self.counter_vecs
            .as_ref()
            .map(|counters| counters.as_slice())
    }

    /// returns descriptors for registered metrics
    pub fn int_counter_vecs(&self) -> Option<&[MetricVecDesc]> {
        self.int_counter_vecs
            .as_ref()
            .map(|counters| counters.as_slice())
    }
}

/// Histogram buckets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Buckets(pub SmallVec<[f64; BUCKETS_SMALLVEC_SIZE]>);

impl From<&[f64]> for Buckets {
    fn from(buckets: &[f64]) -> Self {
        Buckets(SmallVec::from(buckets))
    }
}

/// Label Id
#[ulid]
pub struct LabelId(pub u128);

impl LabelId {
    /// returns the metric name
    /// - the MetricId ULID is prefixedwith 'M' to ensure it does not start with a number because
    ///   prometheus metric names must match the following pattern `[a-zA-Z_:][a-zA-Z0-9_:]*`
    pub fn name(&self) -> String {
        format!("L{}", self)
    }
}

impl FromStr for LabelId {
    type Err = oysterpack_uid::DecodingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id: ULID = s[1..].parse()?;
        Ok(Self(id.into()))
    }
}

/// Metric Id
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct MetricId(pub u128);

impl MetricId {
    /// generate a new unique MetricId
    pub fn generate() -> MetricId {
        Self(oysterpack_uid::ulid_u128())
    }

    /// ID getter
    pub fn id(&self) -> u128 {
        self.0
    }

    /// return the ID as a ULID
    pub fn ulid(&self) -> ULID {
        ULID::from(self.0)
    }

    /// The fully qualified metric name that is registered with prometheus
    /// - name pattern is `M{ULID}`
    pub fn name(&self) -> String {
        self.to_string()
    }
}

impl fmt::Display for MetricId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "M{}", ulid_u128_into_string(self.0))
    }
}

impl FromStr for MetricId {
    type Err = oysterpack_uid::DecodingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id: ULID = s[1..].parse()?;
        Ok(Self(id.into()))
    }
}

/// Runs the function and returns how long it took in nanos.
pub fn time<F>(clock: &quanta::Clock, f: F) -> u64
where
    F: FnOnce(),
{
    let start = clock.start();
    f();
    let end = clock.end();
    clock.delta(start, end)
}

const NANOS_PER_SEC: u32 = 1_000_000_000;

/// converts nanos into secs as f64
/// - this comes in handy when reporting timings to prometheus, which uses `f64` as the number type
pub fn as_float_secs(nanos: u64) -> f64 {
    (nanos as f64) / f64::from(NANOS_PER_SEC)
}

/// Metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Metric {
    /// Counter
    Counter {
        /// desc
        desc: MetricDesc,
        /// value
        value: f64,
    },
    /// IntCounter
    IntCounter {
        /// desc
        desc: MetricDesc,
        /// value
        value: u64,
    },
    /// CounterVec
    CounterVec {
        /// desc
        desc: MetricVecDesc,
        /// values
        values: Vec<MetricValue<f64>>,
    },
    /// IntCounterVec
    IntCounterVec {
        /// desc
        desc: MetricVecDesc,
        /// values
        values: Vec<MetricValue<u64>>,
    },
    /// Gauge
    Gauge {
        /// desc
        desc: MetricDesc,
        /// value
        value: f64,
    },
    /// IntGauge
    IntGauge {
        /// desc
        desc: MetricDesc,
        /// value
        value: u64,
    },
    /// GaugeVec
    GaugeVec {
        /// desc
        desc: MetricVecDesc,
        /// values
        values: Vec<MetricValue<f64>>,
    },
    /// IntGaugeVec
    IntGaugeVec {
        /// desc
        desc: MetricVecDesc,
        /// values
        values: Vec<MetricValue<u64>>,
    },
    /// Histogram
    Histogram {
        /// desc
        desc: HistogramDesc,
        /// total number of data points that have been collected
        sample_count: SampleCount,
        /// values
        values: Vec<BucketValue>,
    },
    /// HistogramVec
    HistogramVec {
        /// desc
        desc: HistogramVecDesc,
        /// values
        values: Vec<HistogramValue>,
    },
}

impl Metric {
    /// Returns the MetricId
    pub fn metric_id(&self) -> MetricId {
        match self {
            Metric::Counter { desc, .. } => desc.id,
            Metric::IntCounter { desc, .. } => desc.id,
            Metric::CounterVec { desc, .. } => desc.id,
            Metric::IntCounterVec { desc, .. } => desc.id,

            Metric::Gauge { desc, .. } => desc.id,
            Metric::IntGauge { desc, .. } => desc.id,
            Metric::GaugeVec { desc, .. } => desc.id,
            Metric::IntGaugeVec { desc, .. } => desc.id,

            Metric::Histogram { desc, .. } => desc.id,
            Metric::HistogramVec { desc, .. } => desc.id,
        }
    }

    fn histogram(metric_id: MetricId, metric: &prometheus::Histogram, buckets: Buckets) -> Self {
        let desc = HistogramDesc::new(metric_id, metric, buckets);
        let metric_families = metric.collect();
        let metric_family = &metric_families[0];
        let metrics_ = metric_family.get_metric();
        let metric = &metrics_[0];
        let histogram = metric.get_histogram();
        let sample_count = histogram.get_sample_count();
        let values: Vec<BucketValue> = histogram
            .get_bucket()
            .iter()
            .map(|bucket| BucketValue {
                cumulative_count: bucket.get_cumulative_count(),
                upper_bound: bucket.get_upper_bound(),
            })
            .collect();
        Metric::Histogram {
            desc,
            values,
            sample_count,
        }
    }

    fn histogram_vec(
        metric_id: MetricId,
        metric: &prometheus::HistogramVec,
        buckets: Buckets,
    ) -> Self {
        let desc = HistogramVecDesc::new(metric_id, metric, buckets);

        // used to filter out the const labels when building the MetricValue
        // - the const labels are listed separately in the MetricVecDesc
        // - in the MetricValue we only want to show the variable labels to minimize the
        //   duplicated info
        let const_label_ids =
            Self::const_label_ids(desc.const_labels.as_ref().map(|labels| labels.as_slice()));

        let values = metric
            .collect()
            .iter()
            .map(|metric_family| {
                let metrics_ = metric_family.get_metric();
                let metric = &metrics_[0];
                let histogram = metric.get_histogram();
                let sample_count = histogram.get_sample_count();
                let values: Vec<BucketValue> = histogram
                    .get_bucket()
                    .iter()
                    .map(|bucket| BucketValue {
                        cumulative_count: bucket.get_cumulative_count(),
                        upper_bound: bucket.get_upper_bound(),
                    })
                    .collect();

                let labels = Self::variable_labels(metric.get_label(), &const_label_ids);

                HistogramValue {
                    labels,
                    sample_count,
                    values,
                }
            })
            .collect();
        Metric::HistogramVec { desc, values }
    }

    fn gauge_vec(metric_id: MetricId, metric: &prometheus::GaugeVec) -> Self {
        let desc = MetricVecDesc::gauge_vec_metric_desc(metric_id, metric);

        // used to filter out the const labels when building the MetricValue
        // - the const labels are listed separately in the MetricVecDesc
        // - in the MetricValue we only want to show the variable labels to minimize the
        //   duplicated info
        let const_label_ids =
            Self::const_label_ids(desc.const_labels.as_ref().map(|labels| labels.as_slice()));

        let mut values = Vec::<MetricValue<f64>>::new();
        for metric_family in metric.collect() {
            for metric in metric_family.get_metric() {
                // variable labels, i.e., const labels are filtered out
                let labels = Self::variable_labels(metric.get_label(), &const_label_ids);
                let value = metric.get_gauge().get_value();
                values.push(MetricValue { labels, value })
            }
        }

        Metric::GaugeVec { desc, values }
    }

    fn int_gauge_vec(metric_id: MetricId, metric: &prometheus::IntGaugeVec) -> Self {
        let desc = MetricVecDesc::int_gauge_vec_metric_desc(metric_id, metric);

        // used to filter out the const labels when building the MetricValue
        // - the const labels are listed separately in the MetricVecDesc
        // - in the MetricValue we only want to show the variable labels to minimize the
        //   duplicated info
        let const_label_ids =
            Self::const_label_ids(desc.const_labels.as_ref().map(|labels| labels.as_slice()));

        let mut values = Vec::<MetricValue<u64>>::new();
        for metric_family in metric.collect() {
            for metric in metric_family.get_metric() {
                // variable labels, i.e., const labels are filtered out
                let labels = Self::variable_labels(metric.get_label(), &const_label_ids);
                let value: u64 = metric.get_gauge().get_value() as u64;
                values.push(MetricValue { labels, value })
            }
        }

        Metric::IntGaugeVec { desc, values }
    }

    fn counter_vec(metric_id: MetricId, metric: &prometheus::CounterVec) -> Self {
        let desc = MetricVecDesc::counter_vec_metric_desc(metric_id, metric);

        // used to filter out the const labels when building the MetricValue
        // - the const labels are listed separately in the MetricVecDesc
        // - in the MetricValue we only want to show the variable labels to minimize the
        //   duplicated info
        let const_label_ids =
            Self::const_label_ids(desc.const_labels.as_ref().map(|labels| labels.as_slice()));

        let mut values = Vec::<MetricValue<f64>>::new();
        for metric_family in metric.collect() {
            for metric in metric_family.get_metric() {
                // variable labels, i.e., const labels are filtered out
                let labels = Self::variable_labels(metric.get_label(), &const_label_ids);
                let value = metric.get_counter().get_value();
                values.push(MetricValue { labels, value })
            }
        }

        Metric::CounterVec { desc, values }
    }

    fn int_counter_vec(metric_id: MetricId, metric: &prometheus::IntCounterVec) -> Self {
        let desc = MetricVecDesc::int_counter_vec_metric_desc(metric_id, metric);

        // used to filter out the const labels when building the MetricValue
        // - the const labels are listed separately in the MetricVecDesc
        // - in the MetricValue we only want to show the variable labels to minimize the
        //   duplicated info
        let const_label_ids: HashSet<LabelId> = match desc.const_labels.as_ref() {
            Some(const_labels) => const_labels
                .iter()
                .map(|(label_id, _value)| label_id)
                .cloned()
                .collect(),
            None => HashSet::new(),
        };

        let mut values = Vec::<MetricValue<u64>>::new();
        for metric_family in metric.collect() {
            for metric in metric_family.get_metric() {
                // variable labels, i.e., const labels are filtered out
                let labels: SmallVec<[(LabelId, String); METRIC_DESC_SMALLVEC_SIZE]> = metric
                    .get_label()
                    .iter()
                    .filter_map(|label_pair| {
                        let label_id = LabelId::from_str(label_pair.get_name()).unwrap();
                        if const_label_ids.contains(&label_id) {
                            None
                        } else {
                            Some((label_id, label_pair.get_value().to_string()))
                        }
                    })
                    .collect();
                let value: u64 = metric.get_counter().get_value() as u64;
                values.push(MetricValue { labels, value })
            }
        }

        Metric::IntCounterVec { desc, values }
    }

    /// filters out the const labels, and returns just the variable labels
    fn variable_labels(
        labels: &[prometheus::proto::LabelPair],
        const_label_ids: &HashSet<LabelId>,
    ) -> SmallVec<[(LabelId, String); METRIC_DESC_SMALLVEC_SIZE]> {
        labels
            .iter()
            .filter_map(|label_pair| {
                let label_id = label_pair.get_name().parse::<LabelId>().unwrap();
                if const_label_ids.contains(&label_id) {
                    None
                } else {
                    Some((label_id, label_pair.get_value().to_string()))
                }
            })
            .collect()
    }

    /// extracts the LabelId(s) into a HashSet
    fn const_label_ids(const_labels: Option<&[(LabelId, String)]>) -> HashSet<LabelId> {
        match const_labels {
            Some(const_labels) => const_labels
                .iter()
                .map(|(label_id, _value)| label_id)
                .cloned()
                .collect(),
            None => HashSet::new(),
        }
    }
}

/// Type alias for a sample count
pub type SampleCount = u64;

/// Histogram bucket value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BucketValue {
    /// cumulative count for the number of data points that are `<=` the upper bound
    pub cumulative_count: u64,
    /// upper bound
    pub upper_bound: f64,
}

/// Histogram value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistogramValue {
    /// metric variable label pairs
    pub labels: SmallVec<[(LabelId, String); METRIC_DESC_SMALLVEC_SIZE]>,
    /// total number of data points that have been collected
    pub sample_count: SampleCount,
    /// values
    pub values: Vec<BucketValue>,
}

/// Metric value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricValue<T> {
    /// metric variable label pairs
    pub labels: SmallVec<[(LabelId, String); METRIC_DESC_SMALLVEC_SIZE]>,
    /// metric value
    pub value: T,
}

/// Metric snapshot at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metrics {
    timestamp: DateTime<Utc>,
    metrics: Vec<Metric>,
}

impl Metrics {
    /// constructor
    pub fn new(capacity: usize) -> Self {
        Self {
            timestamp: Utc::now(),
            metrics: Vec::with_capacity(capacity),
        }
    }

    /// When the metrics were gathered
    pub fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    /// Metrics that were gathered
    pub fn metrics(&self) -> &[Metric] {
        self.metrics.as_slice()
    }

    /// Returns the Metric for the specified MetricId
    pub fn metric(&self, id: MetricId) -> Option<&Metric> {
        self.metrics.iter().find(|metric| metric.metric_id() == id)
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self {
            timestamp: Utc::now(),
            metrics: Vec::new(),
        }
    }
}

/// Process metrics collector
#[derive(Clone)]
pub struct ProcessCollector(Arc<prometheus::process_collector::ProcessCollector>);

impl Default for ProcessCollector {
    fn default() -> Self {
        ProcessCollector(Arc::new(
            prometheus::process_collector::ProcessCollector::for_self(),
        ))
    }
}

impl prometheus::core::Collector for ProcessCollector {
    /// Return descriptors for metrics.
    fn desc(&self) -> Vec<&prometheus::core::Desc> {
        self.0.desc()
    }

    /// Collect metrics.
    fn collect(&self) -> Vec<prometheus::proto::MetricFamily> {
        self.0.collect()
    }
}

impl fmt::Debug for ProcessCollector {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("ProcessCollector")
    }
}

/// Process related metrics
///
/// ## Notes
/// - the process metrics are collected by prometheus' provided process collector
///   - the metrics are registered directly with the registry and use the prometheus provided names,
///     i.e., they are not assigned MetricId(s)
#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ProcessMetrics {
    cpu_seconds_total: f64,
    open_fds: f64,
    max_fds: f64,
    virtual_memory_bytes: f64,
    resident_memory_bytes: f64,
    start_time_seconds: f64,
}

impl ProcessMetrics {
    /// metric name for: Total user and system CPU time spent in seconds.
    pub const PROCESS_CPU_SECONDS_TOTAL: &'static str = "process_cpu_seconds_total";
    /// metric name for: Number of open file descriptors.
    pub const PROCESS_OPEN_FDS: &'static str = "process_open_fds";
    /// metric name for: Maximum number of open file descriptors.
    pub const PROCESS_MAX_FDS: &'static str = "process_max_fds";
    /// metric name for: Virtual memory size in bytes.
    pub const PROCESS_VIRTUAL_MEMORY_BYTES: &'static str = "process_virtual_memory_bytes";
    /// metric name for: Resident memory size in bytes.
    pub const PROCESS_RESIDENT_MEMORY_BYTES: &'static str = "process_resident_memory_bytes";
    /// metric name for: Start time of the process since unix epoch in seconds.
    pub const PROCESS_START_TIME_SECONDS: &'static str = "process_start_time_seconds";

    fn collect(process_collector: &ProcessCollector) -> Self {
        let mut process_metrics = ProcessMetrics::default();
        for metric_family in process_collector.collect() {
            match metric_family.get_name() {
                ProcessMetrics::PROCESS_CPU_SECONDS_TOTAL => {
                    process_metrics.cpu_seconds_total =
                        metric_family.get_metric()[0].get_counter().get_value();
                }
                ProcessMetrics::PROCESS_OPEN_FDS => {
                    process_metrics.open_fds =
                        metric_family.get_metric()[0].get_gauge().get_value();
                }
                ProcessMetrics::PROCESS_MAX_FDS => {
                    process_metrics.max_fds = metric_family.get_metric()[0].get_gauge().get_value();
                }
                ProcessMetrics::PROCESS_VIRTUAL_MEMORY_BYTES => {
                    process_metrics.virtual_memory_bytes =
                        metric_family.get_metric()[0].get_gauge().get_value();
                }
                ProcessMetrics::PROCESS_RESIDENT_MEMORY_BYTES => {
                    process_metrics.resident_memory_bytes =
                        metric_family.get_metric()[0].get_gauge().get_value();
                }
                ProcessMetrics::PROCESS_START_TIME_SECONDS => {
                    process_metrics.start_time_seconds =
                        metric_family.get_metric()[0].get_gauge().get_value();
                }
                unknown => debug_assert!(false, "unknown process metric: {}", unknown),
            }
        }
        process_metrics
    }

    /// Total user and system CPU time spent in seconds.
    pub fn cpu_seconds_total(&self) -> f64 {
        self.cpu_seconds_total
    }

    /// Number of open file descriptors.
    pub fn open_fds(&self) -> f64 {
        self.open_fds
    }

    /// Maximum number of open file descriptors.
    pub fn max_fds(&self) -> f64 {
        self.max_fds
    }

    /// Virtual memory size in bytes.
    pub fn virtual_memory_bytes(&self) -> f64 {
        self.virtual_memory_bytes
    }

    /// Resident memory size in bytes.
    pub fn resident_memory_bytes(&self) -> f64 {
        self.resident_memory_bytes
    }

    /// Start time of the process since unix epoch in seconds.
    pub fn start_time_seconds(&self) -> f64 {
        self.start_time_seconds
    }
}

#[allow(warnings)]
#[cfg(test)]
mod tests;
