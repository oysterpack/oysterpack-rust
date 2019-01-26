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

//! Provides metrics support for prometheus
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
use prometheus::{core::Collector, Encoder};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use std::{collections::HashMap, fmt, io::Write, str::FromStr, sync::Mutex};

lazy_static! {
    /// Global metrics registry
    pub static ref METRIC_REGISTRY: Mutex<MetricRegistry> = Mutex::new(MetricRegistry::default());
}

const SMALLVEC_SIZE: usize = 8;

/// Metric Registry
/// - process metrics collector is automatically added
pub struct MetricRegistry {
    registry: prometheus::Registry,
    counters: Mutex<fnv::FnvHashMap<MetricId<prometheus::Counter>, prometheus::Counter>>,
    counter_vecs: Mutex<fnv::FnvHashMap<MetricId<prometheus::CounterVec>, prometheus::CounterVec>>,
    int_counters: Mutex<fnv::FnvHashMap<MetricId<prometheus::IntCounter>, prometheus::IntCounter>>,
    int_counter_vecs:
        Mutex<fnv::FnvHashMap<MetricId<prometheus::IntCounterVec>, prometheus::IntCounterVec>>,
    gauges: Mutex<fnv::FnvHashMap<MetricId<prometheus::Gauge>, prometheus::Gauge>>,
    gauge_vecs: Mutex<fnv::FnvHashMap<MetricId<prometheus::GaugeVec>, prometheus::GaugeVec>>,
    int_gauges: Mutex<fnv::FnvHashMap<MetricId<prometheus::IntGauge>, prometheus::IntGauge>>,
    int_gauge_vecs:
        Mutex<fnv::FnvHashMap<MetricId<prometheus::IntGaugeVec>, prometheus::IntGaugeVec>>,
    histograms:
        Mutex<fnv::FnvHashMap<MetricId<prometheus::Histogram>, (prometheus::Histogram, Buckets)>>,
    histogram_vecs: Mutex<
        fnv::FnvHashMap<MetricId<prometheus::HistogramVec>, (prometheus::HistogramVec, Buckets)>,
    >,
}

impl MetricRegistry {
    /// Tries to register an int gauge metric
    pub fn register_int_gauge(
        &self,
        metric_id: MetricId<prometheus::IntGauge>,
        help: String,
        const_labels: Option<HashMap<LabelId, String>>,
    ) -> prometheus::Result<()> {
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
        metrics.insert(metric_id, metric);
        Ok(())
    }

    /// Tries to register an int gauge metric
    pub fn register_gauge(
        &self,
        metric_id: MetricId<prometheus::Gauge>,
        help: String,
        const_labels: Option<HashMap<LabelId, String>>,
    ) -> prometheus::Result<()> {
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
        metrics.insert(metric_id, metric);
        Ok(())
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
        metric_id: MetricId<prometheus::GaugeVec>,
        help: String,
        label_ids: &[LabelId],
        const_labels: Option<HashMap<LabelId, String>>,
    ) -> prometheus::Result<()> {
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
        metrics.insert(metric_id, metric);
        Ok(())
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
        metric_id: MetricId<prometheus::IntGaugeVec>,
        help: String,
        label_ids: &[LabelId],
        const_labels: Option<HashMap<LabelId, String>>,
    ) -> prometheus::Result<()> {
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
        metrics.insert(metric_id, metric);
        Ok(())
    }

    /// Tries to register an int counter metric
    pub fn register_int_counter(
        &self,
        metric_id: MetricId<prometheus::IntCounter>,
        help: String,
        const_labels: Option<HashMap<LabelId, String>>,
    ) -> prometheus::Result<()> {
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
        metrics.insert(metric_id, metric);
        Ok(())
    }

    /// Tries to register a counter metric
    pub fn register_counter(
        &self,
        metric_id: MetricId<prometheus::Counter>,
        help: String,
        const_labels: Option<HashMap<LabelId, String>>,
    ) -> prometheus::Result<()> {
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
        metrics.insert(metric_id, metric);
        Ok(())
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
        metric_id: MetricId<prometheus::CounterVec>,
        help: String,
        label_ids: &[LabelId],
        const_labels: Option<HashMap<LabelId, String>>,
    ) -> prometheus::Result<()> {
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
        metrics.insert(metric_id, metric);
        Ok(())
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
        metric_id: MetricId<prometheus::IntCounterVec>,
        help: String,
        label_ids: &[LabelId],
        const_labels: Option<HashMap<LabelId, String>>,
    ) -> prometheus::Result<()> {
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
        metrics.insert(metric_id, metric);
        Ok(())
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
        metric_id: MetricId<prometheus::Histogram>,
        help: String,
        buckets: Vec<f64>,
        const_labels: Option<HashMap<LabelId, String>>,
    ) -> prometheus::Result<()> {
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
        metrics.insert(metric_id, (metric, Buckets(SmallVec::from(buckets))));
        Ok(())
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
        metric_id: MetricId<prometheus::HistogramVec>,
        help: String,
        label_ids: &[LabelId],
        buckets: Vec<f64>,
        const_labels: Option<HashMap<LabelId, String>>,
    ) -> prometheus::Result<()> {
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
        metrics.insert(metric_id, (metric, Buckets(SmallVec::from(buckets))));
        Ok(())
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
    pub fn histogram_vec(
        &self,
        metric_id: &MetricId<prometheus::HistogramVec>,
    ) -> Option<prometheus::HistogramVec> {
        let histogram_vecs = self.histogram_vecs.lock().unwrap();
        histogram_vecs
            .get(&metric_id)
            .map(|(metric, _opts)| metric.clone())
    }

    /// Returns a Histogram for the specified metric ID - if it is registered
    pub fn histogram(
        &self,
        metric_id: &MetricId<prometheus::Histogram>,
    ) -> Option<prometheus::Histogram> {
        let histograms = self.histograms.lock().unwrap();
        histograms
            .get(&metric_id)
            .map(|(metric, _opts)| metric.clone())
    }

    /// Returns a Counter for the specified metric ID - if it is registered
    pub fn counter(
        &self,
        metric_id: &MetricId<prometheus::Counter>,
    ) -> Option<prometheus::Counter> {
        let counters = self.counters.lock().unwrap();
        counters.get(&metric_id).cloned()
    }

    /// Returns an IntCounter for the specified metric ID - if it is registered
    pub fn int_counter(
        &self,
        metric_id: &MetricId<prometheus::IntCounter>,
    ) -> Option<prometheus::IntCounter> {
        let counters = self.int_counters.lock().unwrap();
        counters.get(&metric_id).cloned()
    }

    /// Returns a CounterVec for the specified metric ID - if it is registered
    pub fn counter_vec(
        &self,
        metric_id: &MetricId<prometheus::CounterVec>,
    ) -> Option<prometheus::CounterVec> {
        let counter_vecs = self.counter_vecs.lock().unwrap();
        counter_vecs.get(&metric_id).cloned()
    }

    /// Returns a CounterVec for the specified metric ID - if it is registered
    pub fn int_counter_vec(
        &self,
        metric_id: &MetricId<prometheus::IntCounterVec>,
    ) -> Option<prometheus::IntCounterVec> {
        let int_counter_vecs = self.int_counter_vecs.lock().unwrap();
        int_counter_vecs.get(&metric_id).cloned()
    }

    /// Returns a Counter for the specified metric ID - if it is registered
    pub fn gauge(&self, metric_id: &MetricId<prometheus::Gauge>) -> Option<prometheus::Gauge> {
        let gauges = self.gauges.lock().unwrap();
        gauges.get(&metric_id).cloned()
    }

    /// Returns an IntCounter for the specified metric ID - if it is registered
    pub fn int_gauge(
        &self,
        metric_id: &MetricId<prometheus::IntGauge>,
    ) -> Option<prometheus::IntGauge> {
        let int_gauges = self.int_gauges.lock().unwrap();
        int_gauges.get(&metric_id).cloned()
    }

    /// Returns a CounterVec for the specified metric ID - if it is registered
    pub fn gauge_vec(
        &self,
        metric_id: &MetricId<prometheus::GaugeVec>,
    ) -> Option<prometheus::GaugeVec> {
        let gauges_vecs = self.gauge_vecs.lock().unwrap();
        gauges_vecs.get(&metric_id).cloned()
    }

    /// Returns a CounterVec for the specified metric ID - if it is registered
    pub fn int_gauge_vec(
        &self,
        metric_id: &MetricId<prometheus::IntGaugeVec>,
    ) -> Option<prometheus::IntGaugeVec> {
        let int_gauge_vecs = self.int_gauge_vecs.lock().unwrap();
        int_gauge_vecs.get(&metric_id).cloned()
    }

    /// gather calls the Collect method of the registered Collectors and then gathers the collected
    /// metrics into a lexicographically sorted slice of MetricFamily protobufs.
    pub fn gather(&self) -> Vec<prometheus::proto::MetricFamily> {
        self.registry.gather()
    }

    /// returns the descriptors for registered metrics
    /// - this exludes the process collector metrics
    pub fn metric_descs(&self) -> MetricDescs {
        //    counters: Option<Vec<MetricDesc<prometheus::Counter>>>,
        //    int_counters: Option<Vec<MetricDesc<prometheus::IntCounter>>>,
        //    counter_vecs: Option<Vec<MetricDesc<prometheus::CounterVec>>>,
        //    int_counter_vecs: Option<Vec<MetricDesc<prometheus::IntCounterVec>>>,
        //
        //    gauges: Option<Vec<MetricDesc<prometheus::Gauge>>>,
        //    int_gauges: Option<Vec<MetricDesc<prometheus::IntGauge>>>,
        //    gauge_vecs: Option<Vec<MetricDesc<prometheus::GaugeVec>>>,
        //    int_gauge_vecs: Option<Vec<MetricDesc<prometheus::IntGaugeVec>>>,
        //
        //    histograms: Option<Vec<HistogramDesc>>,
        //    histogram_vecs: Option<Vec<HistogramVecDesc>>,

        let counters: Option<Vec<MetricDesc<prometheus::Counter>>> = {
            let metrics = self.counters.lock().unwrap();
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
                            id: id.clone(),
                            help: desc.help.clone(),
                            const_labels,
                        }
                    })
                    .collect();
                Some(descs)
            }
        };

        let int_counters: Option<Vec<MetricDesc<prometheus::IntCounter>>> = {
            let metrics = self.int_counters.lock().unwrap();
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
                            id: id.clone(),
                            help: desc.help.clone(),
                            const_labels,
                        }
                    })
                    .collect();
                Some(descs)
            }
        };

        let counter_vecs: Option<Vec<MetricVecDesc<prometheus::CounterVec>>> = {
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
                            id: id.clone(),
                            help: desc.help.clone(),
                            labels,
                            const_labels,
                        }
                    })
                    .collect();
                Some(descs)
            }
        };

        let int_counter_vecs: Option<Vec<MetricVecDesc<prometheus::IntCounterVec>>> = {
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
                            id: id.clone(),
                            help: desc.help.clone(),
                            labels,
                            const_labels,
                        }
                    })
                    .collect();
                Some(descs)
            }
        };

        let gauges: Option<Vec<MetricDesc<prometheus::Gauge>>> = {
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
                            id: id.clone(),
                            help: desc.help.clone(),
                            const_labels,
                        }
                    })
                    .collect();
                Some(descs)
            }
        };

        let int_gauges: Option<Vec<MetricDesc<prometheus::IntGauge>>> = {
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
                            id: id.clone(),
                            help: desc.help.clone(),
                            const_labels,
                        }
                    })
                    .collect();
                Some(descs)
            }
        };

        let gauge_vecs: Option<Vec<MetricVecDesc<prometheus::GaugeVec>>> = {
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
                            id: id.clone(),
                            help: desc.help.clone(),
                            labels,
                            const_labels,
                        }
                    })
                    .collect();
                Some(descs)
            }
        };

        let int_gauge_vecs: Option<Vec<MetricVecDesc<prometheus::IntGaugeVec>>> = {
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
                            id: id.clone(),
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
                            id: id.clone(),
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
                            id: id.clone(),
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
#[derive(Clone)]
pub struct MetricDesc<T> {
    id: MetricId<T>,
    help: String,
    const_labels: Option<SmallVec<[(LabelId, String); SMALLVEC_SIZE]>>,
}

impl<T> MetricDesc<T> {
    /// returns the MetricId
    pub fn id(&self) -> &MetricId<T> {
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
}

impl<T> fmt::Debug for MetricDesc<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.const_labels.as_ref() {
            Some(const_labels) => write!(
                f,
                "id = {}, help = {}, const_labels = {:?}",
                self.id, self.help, const_labels
            ),
            None => write!(f, "id = {}, help = {}", self.id, self.help),
        }
    }
}

/// Metric Desc
#[derive(Clone)]
pub struct MetricVecDesc<T> {
    id: MetricId<T>,
    help: String,
    labels: SmallVec<[LabelId; SMALLVEC_SIZE]>,
    const_labels: Option<SmallVec<[(LabelId, String); SMALLVEC_SIZE]>>,
}

impl<T> MetricVecDesc<T> {
    /// returns the MetricId
    pub fn id(&self) -> &MetricId<T> {
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
}

impl<T> fmt::Debug for MetricVecDesc<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.const_labels.as_ref() {
            Some(const_labels) => write!(
                f,
                "id = {}, help = {}, labels = {:?}, const_labels = {:?}",
                self.id, self.help, self.labels, const_labels
            ),
            None => write!(f, "id = {}, help = {}", self.id, self.help),
        }
    }
}

/// Histogram Desc
#[derive(Clone)]
pub struct HistogramDesc {
    id: MetricId<prometheus::Histogram>,
    help: String,
    buckets: Buckets,
    const_labels: Option<SmallVec<[(LabelId, String); SMALLVEC_SIZE]>>,
}

impl HistogramDesc {
    /// returns the MetricId
    pub fn id(&self) -> &MetricId<prometheus::Histogram> {
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
#[derive(Clone)]
pub struct HistogramVecDesc {
    id: MetricId<prometheus::HistogramVec>,
    help: String,
    labels: SmallVec<[LabelId; SMALLVEC_SIZE]>,
    buckets: Buckets,
    const_labels: Option<SmallVec<[(LabelId, String); SMALLVEC_SIZE]>>,
}

impl HistogramVecDesc {
    /// returns the MetricId
    pub fn id(&self) -> &MetricId<prometheus::HistogramVec> {
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
#[derive(Debug, Clone)]
pub struct MetricDescs {
    counters: Option<Vec<MetricDesc<prometheus::Counter>>>,
    int_counters: Option<Vec<MetricDesc<prometheus::IntCounter>>>,
    counter_vecs: Option<Vec<MetricVecDesc<prometheus::CounterVec>>>,
    int_counter_vecs: Option<Vec<MetricVecDesc<prometheus::IntCounterVec>>>,

    gauges: Option<Vec<MetricDesc<prometheus::Gauge>>>,
    int_gauges: Option<Vec<MetricDesc<prometheus::IntGauge>>>,
    gauge_vecs: Option<Vec<MetricVecDesc<prometheus::GaugeVec>>>,
    int_gauge_vecs: Option<Vec<MetricVecDesc<prometheus::IntGaugeVec>>>,

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
    pub fn gauges(&self) -> Option<&[MetricDesc<prometheus::Gauge>]> {
        self.gauges.as_ref().map(|gauges| gauges.as_slice())
    }

    /// returns descriptors for registered metrics
    pub fn int_gauges(&self) -> Option<&[MetricDesc<prometheus::IntGauge>]> {
        self.int_gauges
            .as_ref()
            .map(|int_gauges| int_gauges.as_slice())
    }

    /// returns descriptors for registered metrics
    pub fn gauge_vecs(&self) -> Option<&[MetricVecDesc<prometheus::GaugeVec>]> {
        self.gauge_vecs
            .as_ref()
            .map(|gauge_vecs| gauge_vecs.as_slice())
    }

    /// returns descriptors for registered metrics
    pub fn int_gauge_vecs(&self) -> Option<&[MetricVecDesc<prometheus::IntGaugeVec>]> {
        self.int_gauge_vecs
            .as_ref()
            .map(|int_gauge_vecs| int_gauge_vecs.as_slice())
    }

    /// returns descriptors for registered metrics
    pub fn counters(&self) -> Option<&[MetricDesc<prometheus::Counter>]> {
        self.counters.as_ref().map(|counters| counters.as_slice())
    }

    /// returns descriptors for registered metrics
    pub fn int_counters(&self) -> Option<&[MetricDesc<prometheus::IntCounter>]> {
        self.int_counters
            .as_ref()
            .map(|counters| counters.as_slice())
    }

    /// returns descriptors for registered metrics
    pub fn counter_vecs(&self) -> Option<&[MetricVecDesc<prometheus::CounterVec>]> {
        self.counter_vecs
            .as_ref()
            .map(|counters| counters.as_slice())
    }

    /// returns descriptors for registered metrics
    pub fn int_counter_vecs(&self) -> Option<&[MetricVecDesc<prometheus::IntCounterVec>]> {
        self.int_counter_vecs
            .as_ref()
            .map(|counters| counters.as_slice())
    }
}

/// Histogram buckets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Buckets(pub SmallVec<[f64; SMALLVEC_SIZE]>);

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

/// Generic Metric Id
/// - where `T` should correspond to the prometheus metric type, e.g. `MetricId<prometheus::IntCounter>`
#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct MetricId<T> {
    id: u128,
    _phantom_type: std::marker::PhantomData<T>,
}

impl<T> PartialEq<MetricId<T>> for MetricId<T> {
    fn eq(&self, other: &MetricId<T>) -> bool {
        self.id == other.id
    }
}

impl<T> Eq for MetricId<T> {}

impl<T> std::hash::Hash for MetricId<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl<T> MetricId<T> {
    /// generate a new unique MetricId
    pub fn generate() -> MetricId<T> {
        Self::new(oysterpack_uid::ulid_u128())
    }

    /// constructor
    pub const fn new(id: u128) -> MetricId<T> {
        MetricId {
            id,
            _phantom_type: std::marker::PhantomData,
        }
    }

    /// ID getter
    pub fn id(&self) -> u128 {
        self.id
    }

    /// return the ID as a ULID
    pub fn ulid(&self) -> ULID {
        ULID::from(self.id)
    }

    /// Metric name
    pub fn name(&self) -> String {
        self.to_string()
    }
}

impl<T> fmt::Display for MetricId<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "M{}", ulid_u128_into_string(self.id))
    }
}

impl<T> fmt::Debug for MetricId<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.to_string().as_str())
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
pub fn as_float_secs(nanos: u64) -> f64 {
    (nanos as f64) / f64::from(NANOS_PER_SEC)
}

#[allow(warnings)]
#[cfg(test)]
mod tests {

    use super::*;
    use crate::configure_logging;
    use oysterpack_log::*;
    use std::{thread, time::Duration};

    const METRIC_ID_1: MetricId<prometheus::Counter> =
        MetricId::new(1871943882688894749067493983019708136);

    #[test]
    fn metric_registry_int_gauge() {
        configure_logging();

        use crate::concurrent::messaging::reqrep::ReqRepId;
        use oysterpack_uid::ULID;

        let metric_id = MetricId::generate();
        let registry = MetricRegistry::default();
        registry
            .register_int_gauge(metric_id.clone(), "Active Sessions".to_string(), None)
            .unwrap();

        let mut gauge = registry.int_gauge(&metric_id).unwrap();
        const COUNT: u64 = 10;
        for _ in 0..COUNT {
            gauge.inc();
        }

        // check that the metrics were recorded
        let metrics_family = registry.gather();
        let metric_family = metrics_family
            .iter()
            .filter(|metric_family| metric_family.get_name() == metric_id.name().as_str())
            .next()
            .unwrap();
        let metric = &metric_family.get_metric()[0];
        assert_eq!(metric.get_gauge().get_value(), COUNT as f64);
    }

    #[test]
    fn metric_registry_gauge() {
        configure_logging();

        use crate::concurrent::messaging::reqrep::ReqRepId;
        use oysterpack_uid::ULID;

        let metric_id = MetricId::generate();
        let registry = MetricRegistry::default();
        registry
            .register_gauge(metric_id.clone(), "Active Sessions".to_string(), None)
            .unwrap();

        let mut gauge = registry.gauge(&metric_id).unwrap();
        const COUNT: u64 = 10;
        for _ in 0..COUNT {
            gauge.inc();
        }

        // check that the metrics were recorded
        let metrics_family = registry.gather();
        let metric_family = metrics_family
            .iter()
            .filter(|metric_family| metric_family.get_name() == metric_id.name().as_str())
            .next()
            .unwrap();
        let metric = &metric_family.get_metric()[0];
        assert_eq!(metric.get_gauge().get_value(), COUNT as f64);
    }

    #[test]
    fn metric_registry_gauge_vec() {
        configure_logging();

        use crate::concurrent::messaging::reqrep::ReqRepId;
        use oysterpack_uid::ULID;

        let metric_id = MetricId::generate();
        let registry = MetricRegistry::default();
        let label = LabelId::generate();
        let labels = vec![label];
        registry
            .register_gauge_vec(
                metric_id.clone(),
                "A Gauge Vector".to_string(),
                &labels,
                None,
            )
            .unwrap();

        let mut gauge_vec = registry.gauge_vec(&metric_id).unwrap();
        let mut counter = gauge_vec.with_label_values(&["ABC"]);
        const COUNT: u64 = 10;
        for _ in 0..COUNT {
            counter.inc();
        }

        // check that the metrics were recorded
        let metrics_family = registry.gather();
        let metric_family = metrics_family
            .iter()
            .filter(|metric_family| metric_family.get_name() == metric_id.name().as_str())
            .next()
            .unwrap();
        let metric = &metric_family.get_metric()[0];
        assert_eq!(metric.get_gauge().get_value(), COUNT as f64);
    }

    #[test]
    fn metric_registry_int_gauge_vec() {
        configure_logging();

        use crate::concurrent::messaging::reqrep::ReqRepId;
        use oysterpack_uid::ULID;

        let metric_id = MetricId::generate();
        let registry = MetricRegistry::default();
        let label = LabelId::generate();
        let labels = vec![label];
        registry
            .register_int_gauge_vec(
                metric_id.clone(),
                "A Gauge Vector".to_string(),
                &labels,
                None,
            )
            .unwrap();

        let mut gauge_vec = registry.int_gauge_vec(&metric_id).unwrap();
        let mut counter = gauge_vec.with_label_values(&["ABC"]);
        const COUNT: u64 = 10;
        for _ in 0..COUNT {
            counter.inc();
        }

        // check that the metrics were recorded
        let metrics_family = registry.gather();
        let metric_family = metrics_family
            .iter()
            .filter(|metric_family| metric_family.get_name() == metric_id.name().as_str())
            .next()
            .unwrap();
        let metric = &metric_family.get_metric()[0];
        assert_eq!(metric.get_gauge().get_value(), COUNT as f64);
    }

    #[test]
    fn metric_registry_int_counter() {
        configure_logging();

        use crate::concurrent::messaging::reqrep::ReqRepId;
        use oysterpack_uid::ULID;

        let metric_id = MetricId::generate();
        let registry = MetricRegistry::default();
        registry
            .register_int_counter(metric_id.clone(), "ReqRep timer".to_string(), None)
            .unwrap();

        let mut counter = registry.int_counter(&metric_id).unwrap().local();
        const COUNT: u64 = 10;
        for _ in 0..COUNT {
            counter.inc();
        }

        // check that the metrics were NOT recorded because they were not flushed yet
        let metrics_family = registry.gather();
        let metric_family = metrics_family
            .iter()
            .filter(|metric_family| metric_family.get_name() == metric_id.name().as_str())
            .next()
            .unwrap();
        let metric = &metric_family.get_metric()[0];
        assert_eq!(metric.get_counter().get_value(), 0.0);

        // flush the metrics
        counter.flush();

        // check that the metrics were recorded
        let metrics_family = registry.gather();
        let metric_family = metrics_family
            .iter()
            .filter(|metric_family| metric_family.get_name() == metric_id.name().as_str())
            .next()
            .unwrap();
        let metric = &metric_family.get_metric()[0];
        assert_eq!(metric.get_counter().get_value(), COUNT as f64);
    }

    #[test]
    fn metric_registry_counter() {
        configure_logging();

        use crate::concurrent::messaging::reqrep::ReqRepId;
        use oysterpack_uid::ULID;

        let metric_id = MetricId::generate();
        let registry = MetricRegistry::default();
        registry
            .register_counter(metric_id.clone(), "ReqRep timer".to_string(), None)
            .unwrap();

        let mut counter = registry.counter(&metric_id).unwrap().local();
        const COUNT: u64 = 10;
        for _ in 0..COUNT {
            counter.inc();
        }

        // check that the metrics were NOT recorded because they were not flushed yet
        let metrics_family = registry.gather();
        let metric_family = metrics_family
            .iter()
            .filter(|metric_family| metric_family.get_name() == metric_id.name().as_str())
            .next()
            .unwrap();
        let metric = &metric_family.get_metric()[0];
        assert_eq!(metric.get_counter().get_value(), 0.0);

        // flush the metrics
        counter.flush();

        // check that the metrics were recorded
        let metrics_family = registry.gather();
        let metric_family = metrics_family
            .iter()
            .filter(|metric_family| metric_family.get_name() == metric_id.name().as_str())
            .next()
            .unwrap();
        let metric = &metric_family.get_metric()[0];
        assert_eq!(metric.get_counter().get_value(), COUNT as f64);
    }

    #[test]
    fn metric_registry_counter_vec() {
        configure_logging();

        use crate::concurrent::messaging::reqrep::ReqRepId;
        use oysterpack_uid::ULID;

        let metric_id = MetricId::generate();
        let registry = MetricRegistry::default();
        let label = LabelId::generate();
        let labels = vec![label];
        registry
            .register_counter_vec(metric_id.clone(), "ReqRep timer".to_string(), &labels, None)
            .unwrap();

        let mut counter_vec = registry.counter_vec(&metric_id).unwrap().local();
        let mut counter = counter_vec.with_label_values(&["ABC"]);
        const COUNT: u64 = 10;
        for _ in 0..COUNT {
            counter.inc();
        }

        // check that the metrics were NOT recorded because they were not flushed yet
        let metrics_family = registry.gather();
        let metric_family = metrics_family
            .iter()
            .filter(|metric_family| metric_family.get_name() == metric_id.name().as_str())
            .next()
            .unwrap();
        let metric = &metric_family.get_metric()[0];
        assert_eq!(metric.get_counter().get_value(), 0.0);

        // flush the metrics
        counter.flush();

        // check that the metrics were recorded
        let metrics_family = registry.gather();
        let metric_family = metrics_family
            .iter()
            .filter(|metric_family| metric_family.get_name() == metric_id.name().as_str())
            .next()
            .unwrap();
        let metric = &metric_family.get_metric()[0];
        assert_eq!(metric.get_counter().get_value(), COUNT as f64);
    }

    #[test]
    fn metric_registry_int_counter_vec() {
        configure_logging();

        use crate::concurrent::messaging::reqrep::ReqRepId;
        use oysterpack_uid::ULID;

        let metric_id = MetricId::generate();
        let registry = MetricRegistry::default();
        let label = LabelId::generate();
        let labels = vec![label];
        registry
            .register_int_counter_vec(metric_id.clone(), "ReqRep timer".to_string(), &labels, None)
            .unwrap();

        info!("{:#?}", registry);

        let mut counter_vec = registry.int_counter_vec(&metric_id).unwrap().local();
        let mut counter = counter_vec.with_label_values(&["ABC"]);
        const COUNT: u64 = 10;
        for _ in 0..COUNT {
            counter.inc();
        }

        // check that the metrics were NOT recorded because they were not flushed yet
        let metrics_family = registry.gather();
        let metric_family = metrics_family
            .iter()
            .filter(|metric_family| metric_family.get_name() == metric_id.name().as_str())
            .next()
            .unwrap();
        let metric = &metric_family.get_metric()[0];
        assert_eq!(metric.get_counter().get_value(), 0.0);

        // flush the metrics
        counter.flush();

        // check that the metrics were recorded
        let metrics_family = registry.gather();
        let metric_family = metrics_family
            .iter()
            .filter(|metric_family| metric_family.get_name() == metric_id.name().as_str())
            .next()
            .unwrap();
        let metric = &metric_family.get_metric()[0];
        assert_eq!(metric.get_counter().get_value(), COUNT as f64);
    }

    #[test]
    fn metric_registry_histogram_vec() {
        configure_logging();

        use crate::concurrent::messaging::reqrep::ReqRepId;
        use oysterpack_uid::ULID;

        let metric_id = MetricId::generate();
        let registry = MetricRegistry::default();
        registry
            .register_histogram_vec(
                metric_id.clone(),
                "ReqRep timer".to_string(),
                &[LabelId::generate()],
                vec![0.01, 0.025, 0.05, 0.005, 0.0050, 0.005], // will be sorted and deduped automatically
                None,
            )
            .unwrap();

        info!("{:#?}", registry);

        let mut reqrep_timer_local = registry.histogram_vec(&metric_id).unwrap().local();
        let reqrep_timer =
            reqrep_timer_local.with_label_values(&[ULID::generate().to_string().as_str()]);
        let clock = quanta::Clock::new();
        for _ in 0..10 {
            let ulid_u128: u128 = ULID::generate().into();
            let sleep_ms = (ulid_u128 % 100) as u32;
            info!("sleeping for {}", sleep_ms);
            let delta = time(&clock, || thread::sleep_ms(sleep_ms));
            reqrep_timer.observe(as_float_secs(delta));
            reqrep_timer.flush();
        }
    }

    #[test]
    fn metric_registry_histogram() {
        configure_logging();

        use oysterpack_uid::ULID;

        let metric_id = MetricId::generate();
        let registry = MetricRegistry::default();
        registry
            .register_histogram(
                metric_id.clone(),
                "ReqRep timer".to_string(),
                vec![0.01, 0.025, 0.05, 0.005, 0.0050, 0.005], // will be sorted and deduped automatically
                None,
            )
            .unwrap();

        info!("{:#?}", registry);

        let mut reqrep_timer = registry.histogram(&metric_id).unwrap().local();
        let clock = quanta::Clock::new();
        const METRIC_COUNT: u64 = 5;
        for _ in 0..5 {
            let ulid_u128: u128 = ULID::generate().into();
            let sleep_ms = (ulid_u128 % 10) as u32;
            info!("sleeping for {}", sleep_ms);
            let delta = time(&clock, || thread::sleep_ms(sleep_ms));
            reqrep_timer.observe(as_float_secs(delta));
            reqrep_timer.flush();
        }

        let metrics_family = registry.gather();
        info!("{:#?}", metrics_family);
        registry.text_encode_metrics(&mut std::io::stderr());

        // check that the metrics were recorded
        let metric_family = metrics_family
            .iter()
            .filter(|metric_family| metric_family.get_name() == metric_id.name().as_str())
            .next()
            .unwrap();
        let metric = &metric_family.get_metric()[0];
        assert_eq!(metric.get_histogram().get_sample_count(), METRIC_COUNT);
    }

    #[test]
    fn metric_registry_histogram_using_timer() {
        configure_logging();

        use oysterpack_uid::ULID;

        let metric_id = MetricId::generate();
        let registry = MetricRegistry::default();
        registry
            .register_histogram(
                metric_id.clone(),
                "ReqRep timer".to_string(),
                vec![0.01, 0.025, 0.05, 0.005, 0.0050, 0.005], // will be sorted and deduped automatically
                None,
            )
            .unwrap();

        let mut reqrep_timer = registry.histogram(&metric_id).unwrap();
        const METRIC_COUNT: u64 = 5;
        for _ in 0..METRIC_COUNT {
            let ulid_u128: u128 = ULID::generate().into();
            let sleep_ms = (ulid_u128 % 5) as u32;
            info!("sleeping for {}", sleep_ms);
            {
                let timer = reqrep_timer.start_timer();
                thread::sleep_ms(sleep_ms)
            }
        }

        let metrics_family = registry.gather();
        info!("{:#?}", metrics_family);
        registry.text_encode_metrics(&mut std::io::stderr());

        // check that the metrics were recorded
        let metric_family = metrics_family
            .iter()
            .filter(|metric_family| metric_family.get_name() == metric_id.name().as_str())
            .next()
            .unwrap();
        let metric = &metric_family.get_metric()[0];
        assert_eq!(metric.get_histogram().get_sample_count(), METRIC_COUNT);
    }

    #[test]
    fn metric_registry_histogram_vec_with_const_labels() {
        configure_logging();

        use oysterpack_uid::ULID;

        let metric_id = MetricId::generate();
        let registry = MetricRegistry::default();
        let mut const_labels = HashMap::new();
        let label = LabelId::generate();
        const_labels.insert(label, "  BAR".to_string());
        registry
            .register_histogram_vec(
                metric_id.clone(),
                "ReqRep timer".to_string(),
                &[LabelId::generate()],
                vec![0.01, 0.025, 0.05, 0.005, 0.0050, 0.005], // will be sorted and deduped automatically
                Some(const_labels),
            )
            .unwrap();

        let mut reqrep_timer_local = registry.histogram_vec(&metric_id).unwrap().local();
        let reqrep_timer =
            reqrep_timer_local.with_label_values(&[ULID::generate().to_string().as_str()]);
        let clock = quanta::Clock::new();
        const METRIC_COUNT: usize = 5;
        for _ in 0..METRIC_COUNT {
            let ulid_u128: u128 = ULID::generate().into();
            let sleep_ms = (ulid_u128 % 100) as u32;
            info!("sleeping for {}", sleep_ms);
            let delta = time(&clock, || thread::sleep_ms(sleep_ms));
            reqrep_timer.observe(as_float_secs(delta));
            reqrep_timer.flush();
        }

        let metrics_family = registry.gather();
        info!("{:#?}", metrics_family);

        // check that the const label was trimmed FOO=BAR
        let metric_family = metrics_family
            .iter()
            .filter(|metric_family| metric_family.get_name() == metric_id.name().as_str())
            .next()
            .unwrap();
        let metric = &metric_family.get_metric()[0];
        let label_pair = metric
            .get_label()
            .iter()
            .filter(|label_pair| label_pair.get_name() == label.name().as_str())
            .next()
            .unwrap();
        assert_eq!(label_pair.get_name(), label.name());
        assert_eq!(label_pair.get_value(), "BAR")
    }

    #[test]
    fn metric_registry_histogram_vec_with_blank_const_label() {
        configure_logging();

        use oysterpack_uid::ULID;

        let metric_id = MetricId::generate();
        let registry = MetricRegistry::default();

        {
            let mut const_labels = HashMap::new();
            const_labels.insert(LabelId::generate(), "  ".to_string());
            let result = registry.register_histogram_vec(
                metric_id.clone(),
                "ReqRep timer".to_string(),
                &[LabelId::generate()],
                vec![0.01, 0.025, 0.05, 0.005, 0.0050, 0.005], // will be sorted and deduped automatically
                Some(const_labels),
            );
            info!("const label value is blank: {:?}", result);
            assert!(result.is_err());
            assert!(result.err().unwrap().to_string().contains("value"));
        }
    }

    #[test]
    fn metric_registry_histogram_vec_with_blank_help() {
        configure_logging();

        use oysterpack_uid::ULID;

        let metric_id = MetricId::generate();
        let registry = MetricRegistry::default();

        let result = registry.register_histogram_vec(
            metric_id.clone(),
            " ".to_string(),
            &[LabelId::generate()],
            vec![0.01, 0.025, 0.05, 0.005, 0.0050, 0.005], // will be sorted and deduped automatically
            None,
        );
        info!("help is blank: {:?}", result);
        assert!(result.is_err());
    }

    #[test]
    fn global_metric_registry() {
        configure_logging();

        let registry = METRIC_REGISTRY.lock().unwrap();
        let metrics = registry.gather();
        info!("{:#?}", metrics);
    }

    #[test]
    fn metric_descs() {
        configure_logging();

        let metric_registry = MetricRegistry::default();
        let descs = metric_registry.metric_descs();
        info!("empty MetricRegistry: {:#?}", descs);
        assert!(descs.counters().is_none());
        assert!(descs.int_counters().is_none());
        assert!(descs.counter_vecs().is_none());
        assert!(descs.int_counter_vecs().is_none());
        assert!(descs.gauges().is_none());
        assert!(descs.int_gauges().is_none());
        assert!(descs.gauge_vecs().is_none());
        assert!(descs.int_gauge_vecs().is_none());
        assert!(descs.histograms().is_none());
        assert!(descs.histogram_vecs().is_none());

        let mut const_labels = HashMap::new();
        const_labels.insert(LabelId::generate(), "FOO".to_string());
        let labels = vec![LabelId::generate(), LabelId::generate()];
        metric_registry
            .register_counter(
                MetricId::generate(),
                "counter".to_string(),
                Some(const_labels.clone()),
            )
            .unwrap();
        metric_registry
            .register_int_counter(
                MetricId::generate(),
                "int_counter".to_string(),
                Some(const_labels.clone()),
            )
            .unwrap();
        metric_registry
            .register_counter_vec(
                MetricId::generate(),
                "counter_vec".to_string(),
                &labels,
                Some(const_labels.clone()),
            )
            .unwrap();
        metric_registry
            .register_int_counter_vec(
                MetricId::generate(),
                "int_counter_vec".to_string(),
                &labels,
                Some(const_labels.clone()),
            )
            .unwrap();

        metric_registry
            .register_gauge(
                MetricId::generate(),
                "gauge".to_string(),
                Some(const_labels.clone()),
            )
            .unwrap();
        metric_registry
            .register_int_gauge(
                MetricId::generate(),
                "int_gauge".to_string(),
                Some(const_labels.clone()),
            )
            .unwrap();
        metric_registry
            .register_gauge_vec(
                MetricId::generate(),
                "gauge_vec".to_string(),
                &labels,
                Some(const_labels.clone()),
            )
            .unwrap();
        metric_registry
            .register_int_gauge_vec(
                MetricId::generate(),
                "int_gauge_vec".to_string(),
                &labels,
                Some(const_labels.clone()),
            )
            .unwrap();

        let buckets = vec![0.0, 1.0, 5.0, 10.0];
        metric_registry
            .register_histogram(
                MetricId::generate(),
                "histogram".to_string(),
                buckets.clone(),
                Some(const_labels.clone()),
            )
            .unwrap();
        metric_registry
            .register_histogram_vec(
                MetricId::generate(),
                "histogram_vec".to_string(),
                &labels,
                buckets.clone(),
                Some(const_labels.clone()),
            )
            .unwrap();
        let descs = metric_registry.metric_descs();
        info!("{:#?}", descs);
        assert!(descs.counters().is_some());
        assert!(descs.int_counters().is_some());
        assert!(descs.counter_vecs().is_some());
        assert!(descs.int_counter_vecs().is_some());
        assert!(descs.gauges().is_some());
        assert!(descs.int_gauges().is_some());
        assert!(descs.gauge_vecs().is_some());
        assert!(descs.int_gauge_vecs().is_some());
        assert!(descs.histograms().is_some());
        assert!(descs.histogram_vecs().is_some());
    }

}
