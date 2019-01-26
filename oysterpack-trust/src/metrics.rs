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
//! - [METRIC_REGISTRY](struct.METRIC_REGISTRY.html) provides a global registry that can be used
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
//! Because names change over time, which can break components that depend on metric names ...
//! Assigning unique numerical identifiers is much more stable. Human friendly metric labels and any
//! additional information can be mapped externally to the MetricId.
//!
//! ### Notes
//! - for prometheus metrics use the metric `help` attribute to provide a human friendly label and
//!   short description

use lazy_static::lazy_static;
use oysterpack_uid::{macros::ulid, ulid_u128_into_string, ULID};
use prometheus::{
    core::{Atomic, Collector},
    Encoder,
};
use serde::{Deserialize, Serialize};

use chrono::{DateTime, Utc};
use smallvec::SmallVec;
use std::{collections::HashMap, fmt, io::Write, str::FromStr, sync::Mutex};

lazy_static! {
    /// Global metrics registry
    pub static ref METRIC_REGISTRY: Mutex<MetricRegistry> = Mutex::new(MetricRegistry::default());
}

// used to minimize memory allocations on the heap
const METRIC_DESC_SMALLVEC_SIZE: usize = 8;
const BUCKETS_SMALLVEC_SIZE: usize = 16;

/// Metric Registry
/// - process metrics collector is automatically added
pub struct MetricRegistry {
    registry: prometheus::Registry,
    counters: Mutex<fnv::FnvHashMap<MetricId, prometheus::Counter>>,
    counter_vecs: Mutex<fnv::FnvHashMap<MetricId, prometheus::CounterVec>>,
    int_counters: Mutex<fnv::FnvHashMap<MetricId, prometheus::IntCounter>>,
    int_counter_vecs: Mutex<fnv::FnvHashMap<MetricId, prometheus::IntCounterVec>>,
    gauges: Mutex<fnv::FnvHashMap<MetricId, prometheus::Gauge>>,
    gauge_vecs: Mutex<fnv::FnvHashMap<MetricId, prometheus::GaugeVec>>,
    int_gauges: Mutex<fnv::FnvHashMap<MetricId, prometheus::IntGauge>>,
    int_gauge_vecs: Mutex<fnv::FnvHashMap<MetricId, prometheus::IntGaugeVec>>,
    histograms: Mutex<fnv::FnvHashMap<MetricId, (prometheus::Histogram, Buckets)>>,
    histogram_vecs: Mutex<fnv::FnvHashMap<MetricId, (prometheus::HistogramVec, Buckets)>>,
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

        let mut metrics = self.int_gauges.lock().unwrap();
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

        let mut metrics = self.gauges.lock().unwrap();
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

        let mut metrics = self.gauge_vecs.lock().unwrap();
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

        let mut metrics = self.int_gauge_vecs.lock().unwrap();
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

        let mut metrics = self.int_counters.lock().unwrap();
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

        let mut metrics = self.counters.lock().unwrap();
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

        let mut metrics = self.counter_vecs.lock().unwrap();
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

        let mut metrics = self.int_counter_vecs.lock().unwrap();
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

        let mut metrics = self.histograms.lock().unwrap();
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

        let mut metrics = self.histogram_vecs.lock().unwrap();
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
        let histogram_vecs = self.histogram_vecs.lock().unwrap();
        histogram_vecs
            .get(&metric_id)
            .map(|(metric, _opts)| metric.clone())
    }

    /// Returns a Histogram for the specified metric ID - if it is registered
    pub fn histogram(&self, metric_id: &MetricId) -> Option<prometheus::Histogram> {
        let histograms = self.histograms.lock().unwrap();
        histograms
            .get(&metric_id)
            .map(|(metric, _opts)| metric.clone())
    }

    /// Returns a Counter for the specified metric ID - if it is registered
    pub fn counter(&self, metric_id: &MetricId) -> Option<prometheus::Counter> {
        let counters = self.counters.lock().unwrap();
        counters.get(&metric_id).cloned()
    }

    /// Returns an IntCounter for the specified metric ID - if it is registered
    pub fn int_counter(&self, metric_id: &MetricId) -> Option<prometheus::IntCounter> {
        let counters = self.int_counters.lock().unwrap();
        counters.get(&metric_id).cloned()
    }

    /// Returns a CounterVec for the specified metric ID - if it is registered
    pub fn counter_vec(&self, metric_id: &MetricId) -> Option<prometheus::CounterVec> {
        let counter_vecs = self.counter_vecs.lock().unwrap();
        counter_vecs.get(&metric_id).cloned()
    }

    /// Returns a CounterVec for the specified metric ID - if it is registered
    pub fn int_counter_vec(&self, metric_id: &MetricId) -> Option<prometheus::IntCounterVec> {
        let int_counter_vecs = self.int_counter_vecs.lock().unwrap();
        int_counter_vecs.get(&metric_id).cloned()
    }

    /// Returns a Counter for the specified metric ID - if it is registered
    pub fn gauge(&self, metric_id: &MetricId) -> Option<prometheus::Gauge> {
        let gauges = self.gauges.lock().unwrap();
        gauges.get(&metric_id).cloned()
    }

    /// Returns an IntCounter for the specified metric ID - if it is registered
    pub fn int_gauge(&self, metric_id: &MetricId) -> Option<prometheus::IntGauge> {
        let int_gauges = self.int_gauges.lock().unwrap();
        int_gauges.get(&metric_id).cloned()
    }

    /// Returns a CounterVec for the specified metric ID - if it is registered
    pub fn gauge_vec(&self, metric_id: &MetricId) -> Option<prometheus::GaugeVec> {
        let gauges_vecs = self.gauge_vecs.lock().unwrap();
        gauges_vecs.get(&metric_id).cloned()
    }

    /// Returns a CounterVec for the specified metric ID - if it is registered
    pub fn int_gauge_vec(&self, metric_id: &MetricId) -> Option<prometheus::IntGaugeVec> {
        let int_gauge_vecs = self.int_gauge_vecs.lock().unwrap();
        int_gauge_vecs.get(&metric_id).cloned()
    }

    /// gather calls the Collect method of the registered Collectors and then gathers the collected
    /// metrics into a lexicographically sorted slice of MetricFamily protobufs.
    pub fn gather(&self) -> Vec<prometheus::proto::MetricFamily> {
        self.registry.gather()
    }

    /// Gathers the specified metrics
    pub fn gather_metrics(&self, metric_ids: &[MetricId]) -> Metrics {
        let mut metrics = Metrics::new(metric_ids.len());
        self.gather_counter_metrics(metric_ids, &mut metrics);
        self.gather_int_counter_metrics(metric_ids, &mut metrics);
        metrics
    }

    fn gather_counter_metrics(&self, metric_ids: &[MetricId], metrics: &mut Metrics) {
        let counters = self.counters.lock().unwrap();
        for metric_id in metric_ids {
            if let Some(metric) = counters.get(metric_id) {
                let desc = Self::counter_metric_desc(*metric_id, metric);
                let value = metric.get();
                metrics.metrics.push(Metric::Counter { desc, value });
            }
        }
    }

    fn gather_int_counter_metrics(&self, metric_ids: &[MetricId], metrics: &mut Metrics) {
        let counters = self.int_counters.lock().unwrap();
        for metric_id in metric_ids {
            if let Some(metric) = counters.get(metric_id) {
                let desc = Self::int_counter_metric_desc(*metric_id, metric);
                let value: u64 = metric.get() as u64;
                metrics.metrics.push(Metric::IntCounter { desc, value });
            }
        }
    }

    /// gathers
    pub fn gather_all_metrics(&self, metric_ids: &[MetricId]) -> Metrics {
        unimplemented!()
    }

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

    /// returns the descriptors for registered metrics
    /// - this exludes the process collector metrics
    pub fn metric_descs(&self) -> MetricDescs {
        let counters: Option<Vec<MetricDesc>> = {
            let metrics = self.counters.lock().unwrap();
            if metrics.is_empty() {
                None
            } else {
                let descs = metrics
                    .iter()
                    .map(|(id, metric)| Self::counter_metric_desc(*id, metric))
                    .collect();
                Some(descs)
            }
        };

        let int_counters: Option<Vec<MetricDesc>> = {
            let metrics = self.int_counters.lock().unwrap();
            if metrics.is_empty() {
                None
            } else {
                let descs = metrics
                    .iter()
                    .map(|(id, metric)| Self::int_counter_metric_desc(*id, metric))
                    .collect();
                Some(descs)
            }
        };

        let counter_vecs: Option<Vec<MetricVecDesc>> = {
            let metrics = self.counter_vecs.lock().unwrap();
            if metrics.is_empty() {
                None
            } else {
                let descs = metrics
                    .iter()
                    .map(|(id, metric)| {
                        let desc = metric.desc()[0];
                        let const_labels = if desc.const_label_pairs.is_empty() {
                            None
                        } else {
                            Some(
                                desc.const_label_pairs
                                    .iter()
                                    .map(|label_pair| {
                                        let label_id: LabelId =
                                            label_pair.get_name().parse().unwrap();
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
                            id: *id,
                            help: desc.help.clone(),
                            labels,
                            const_labels,
                        }
                    })
                    .collect();
                Some(descs)
            }
        };

        let int_counter_vecs: Option<Vec<MetricVecDesc>> = {
            let metrics = self.int_counter_vecs.lock().unwrap();
            if metrics.is_empty() {
                None
            } else {
                let descs = metrics
                    .iter()
                    .map(|(id, metric)| {
                        let desc = metric.desc()[0];
                        let const_labels = if desc.const_label_pairs.is_empty() {
                            None
                        } else {
                            Some(
                                desc.const_label_pairs
                                    .iter()
                                    .map(|label_pair| {
                                        let label_id: LabelId =
                                            label_pair.get_name().parse().unwrap();
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
                            id: *id,
                            help: desc.help.clone(),
                            labels,
                            const_labels,
                        }
                    })
                    .collect();
                Some(descs)
            }
        };

        let gauges: Option<Vec<MetricDesc>> = {
            let metrics = self.gauges.lock().unwrap();
            if metrics.is_empty() {
                None
            } else {
                let descs = metrics
                    .iter()
                    .map(|(id, metric)| {
                        let desc = metric.desc()[0];
                        let const_labels = if desc.const_label_pairs.is_empty() {
                            None
                        } else {
                            Some(
                                desc.const_label_pairs
                                    .iter()
                                    .map(|label_pair| {
                                        let label_id: LabelId =
                                            label_pair.get_name().parse().unwrap();
                                        let label_value = label_pair.get_value().to_string();
                                        (label_id, label_value)
                                    })
                                    .collect(),
                            )
                        };
                        MetricDesc {
                            id: *id,
                            help: desc.help.clone(),
                            const_labels,
                        }
                    })
                    .collect();
                Some(descs)
            }
        };

        let int_gauges: Option<Vec<MetricDesc>> = {
            let metrics = self.int_gauges.lock().unwrap();
            if metrics.is_empty() {
                None
            } else {
                let descs = metrics
                    .iter()
                    .map(|(id, metric)| {
                        let desc = metric.desc()[0];
                        let const_labels = if desc.const_label_pairs.is_empty() {
                            None
                        } else {
                            Some(
                                desc.const_label_pairs
                                    .iter()
                                    .map(|label_pair| {
                                        let label_id: LabelId =
                                            label_pair.get_name().parse().unwrap();
                                        let label_value = label_pair.get_value().to_string();
                                        (label_id, label_value)
                                    })
                                    .collect(),
                            )
                        };
                        MetricDesc {
                            id: *id,
                            help: desc.help.clone(),
                            const_labels,
                        }
                    })
                    .collect();
                Some(descs)
            }
        };

        let gauge_vecs: Option<Vec<MetricVecDesc>> = {
            let metrics = self.gauge_vecs.lock().unwrap();
            if metrics.is_empty() {
                None
            } else {
                let descs = metrics
                    .iter()
                    .map(|(id, metric)| {
                        let desc = metric.desc()[0];
                        let const_labels = if desc.const_label_pairs.is_empty() {
                            None
                        } else {
                            Some(
                                desc.const_label_pairs
                                    .iter()
                                    .map(|label_pair| {
                                        let label_id: LabelId =
                                            label_pair.get_name().parse().unwrap();
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
                            id: *id,
                            help: desc.help.clone(),
                            labels,
                            const_labels,
                        }
                    })
                    .collect();
                Some(descs)
            }
        };

        let int_gauge_vecs: Option<Vec<MetricVecDesc>> = {
            let metrics = self.int_gauge_vecs.lock().unwrap();
            if metrics.is_empty() {
                None
            } else {
                let descs = metrics
                    .iter()
                    .map(|(id, metric)| {
                        let desc = metric.desc()[0];
                        let const_labels = if desc.const_label_pairs.is_empty() {
                            None
                        } else {
                            Some(
                                desc.const_label_pairs
                                    .iter()
                                    .map(|label_pair| {
                                        let label_id: LabelId =
                                            label_pair.get_name().parse().unwrap();
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
                            id: *id,
                            help: desc.help.clone(),
                            labels,
                            const_labels,
                        }
                    })
                    .collect();
                Some(descs)
            }
        };

        let histograms: Option<Vec<HistogramDesc>> = {
            let metrics = self.histograms.lock().unwrap();
            if metrics.is_empty() {
                None
            } else {
                let descs = metrics
                    .iter()
                    .map(|(id, &(ref metric, ref buckets))| {
                        let desc = metric.desc()[0];
                        let const_labels = if desc.const_label_pairs.is_empty() {
                            None
                        } else {
                            Some(
                                desc.const_label_pairs
                                    .iter()
                                    .map(|label_pair| {
                                        let label_id: LabelId =
                                            label_pair.get_name().parse().unwrap();
                                        let label_value = label_pair.get_value().to_string();
                                        (label_id, label_value)
                                    })
                                    .collect(),
                            )
                        };
                        HistogramDesc {
                            id: *id,
                            help: desc.help.clone(),
                            const_labels,
                            buckets: buckets.clone(),
                        }
                    })
                    .collect();
                Some(descs)
            }
        };

        let histogram_vecs: Option<Vec<HistogramVecDesc>> = {
            let metrics = self.histogram_vecs.lock().unwrap();
            if metrics.is_empty() {
                None
            } else {
                let descs = metrics
                    .iter()
                    .map(|(id, &(ref metric, ref buckets))| {
                        let desc = metric.desc()[0];
                        let const_labels = if desc.const_label_pairs.is_empty() {
                            None
                        } else {
                            Some(
                                desc.const_label_pairs
                                    .iter()
                                    .map(|label_pair| {
                                        let label_id: LabelId =
                                            label_pair.get_name().parse().unwrap();
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
                            id: *id,
                            help: desc.help.clone(),
                            const_labels,
                            buckets: buckets.clone(),
                            labels,
                        }
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
        registry
            .register(Box::new(
                prometheus::process_collector::ProcessCollector::for_self(),
            ))
            .unwrap();
        Self {
            registry,
            int_counters: Mutex::new(fnv::FnvHashMap::default()),
            int_counter_vecs: Mutex::new(fnv::FnvHashMap::default()),
            counters: Mutex::new(fnv::FnvHashMap::default()),
            counter_vecs: Mutex::new(fnv::FnvHashMap::default()),

            gauges: Mutex::new(fnv::FnvHashMap::default()),
            gauge_vecs: Mutex::new(fnv::FnvHashMap::default()),
            int_gauges: Mutex::new(fnv::FnvHashMap::default()),
            int_gauge_vecs: Mutex::new(fnv::FnvHashMap::default()),

            histogram_vecs: Mutex::new(fnv::FnvHashMap::default()),
            histograms: Mutex::new(fnv::FnvHashMap::default()),
        }
    }
}

/// Metric Desc
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricDesc {
    id: MetricId,
    help: String,
    const_labels: Option<SmallVec<[(LabelId, String); METRIC_DESC_SMALLVEC_SIZE]>>,
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
}

/// Metric Desc
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricVecDesc {
    id: MetricId,
    help: String,
    labels: SmallVec<[LabelId; METRIC_DESC_SMALLVEC_SIZE]>,
    const_labels: Option<SmallVec<[(LabelId, String); METRIC_DESC_SMALLVEC_SIZE]>>,
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
}

/// Histogram Desc
#[derive(Clone, Serialize, Deserialize)]
pub struct HistogramDesc {
    id: MetricId,
    help: String,
    buckets: Buckets,
    const_labels: Option<SmallVec<[(LabelId, String); METRIC_DESC_SMALLVEC_SIZE]>>,
}

impl HistogramDesc {
    /// returns the MetricId
    pub fn id(&self) -> &MetricId {
        &self.id
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
    labels: SmallVec<[LabelId; METRIC_DESC_SMALLVEC_SIZE]>,
    buckets: Buckets,
    const_labels: Option<SmallVec<[(LabelId, String); METRIC_DESC_SMALLVEC_SIZE]>>,
}

impl HistogramVecDesc {
    /// returns the MetricId
    pub fn id(&self) -> &MetricId {
        &self.id
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
        desc: MetricDesc,
        /// values
        values: Vec<MetricValue<f64>>,
    },
    /// IntCounterVec
    IntCounterVec {
        /// desc
        desc: MetricDesc,
        /// values
        values: Vec<MetricValue<u64>>,
    },
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

    /// When the metrics started to be gathered
    pub fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    /// Metrics that were gathered
    pub fn metrics(&self) -> &[Metric] {
        self.metrics.as_slice()
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

#[allow(warnings)]
#[cfg(test)]
mod tests;
