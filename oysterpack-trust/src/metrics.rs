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
//! ### Why use a number as a metric name ?
//! Because names change over time, which can break components that depend on metric names ...
//! Assigning unique numerical identifiers is much more stable. Human friendly metric labels and any
//! additional information can be mapped externally to the MetricId.
//!
//! ### Notes
//! - for prometheus metrics use the metric `help` attribute to provide a human friendly label and
//!   short description

use lazy_static::lazy_static;
use oysterpack_uid::macros::ulid;
use prometheus::{core::Collector, Encoder};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt, io::Write, sync::Mutex};

lazy_static! {
    /// Global metrics registry
    pub static ref METRIC_REGISTRY: Mutex<MetricRegistry> = Mutex::new(MetricRegistry::default());
}

/// Metric Registry
/// - process metrics collector is automatically added
pub struct MetricRegistry {
    registry: prometheus::Registry,
    counters: Mutex<fnv::FnvHashMap<CounterId, prometheus::Counter>>,
    counter_vecs: Mutex<fnv::FnvHashMap<CounterVecId, prometheus::CounterVec>>,
    int_counters: Mutex<fnv::FnvHashMap<IntCounterId, prometheus::IntCounter>>,
    int_counter_vecs: Mutex<fnv::FnvHashMap<IntCounterVecId, prometheus::IntCounterVec>>,
    gauges: Mutex<fnv::FnvHashMap<GaugeId, prometheus::Gauge>>,
    gauge_vecs: Mutex<fnv::FnvHashMap<GaugeVecId, prometheus::GaugeVec>>,
    int_gauges: Mutex<fnv::FnvHashMap<IntGaugeId, prometheus::IntGauge>>,
    int_gauge_vecs: Mutex<fnv::FnvHashMap<IntGaugeVecId, prometheus::IntGaugeVec>>,
    histograms: Mutex<fnv::FnvHashMap<HistogramId, (prometheus::Histogram, Buckets)>>,
    histogram_vecs: Mutex<fnv::FnvHashMap<HistogramVecId, (prometheus::HistogramVec, Buckets)>>,
}

impl MetricRegistry {
    /// Tries to register a counter metric
    pub fn register_int_counter(
        &self,
        metric_id: IntCounterId,
        help: String,
        const_labels: Option<HashMap<String, String>>,
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
        metric_id: CounterId,
        help: String,
        const_labels: Option<HashMap<String, String>>,
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
        metric_id: CounterVecId,
        help: String,
        label_names: &[&str],
        const_labels: Option<HashMap<String, String>>,
    ) -> prometheus::Result<()> {
        let check_labels = || {
            if label_names.is_empty() {
                return Err(prometheus::Error::Msg(
                    "At least one label name must be provided".to_string(),
                ));
            }
            let mut trimmed_label_names: Vec<&str> = Vec::with_capacity(label_names.len());
            for label in label_names.iter() {
                let label = label.trim();
                if label.is_empty() {
                    return Err(prometheus::Error::Msg("Labels cannot be blank".to_string()));
                }
                trimmed_label_names.push(label);
            }
            Ok(trimmed_label_names)
        };

        let label_names = check_labels()?;
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
        metric_id: IntCounterVecId,
        help: String,
        label_names: &[&str],
        const_labels: Option<HashMap<String, String>>,
    ) -> prometheus::Result<()> {
        let check_labels = || {
            if label_names.is_empty() {
                return Err(prometheus::Error::Msg(
                    "At least one label name must be provided".to_string(),
                ));
            }
            let mut trimmed_label_names: Vec<&str> = Vec::with_capacity(label_names.len());
            for label in label_names.iter() {
                let label = label.trim();
                if label.is_empty() {
                    return Err(prometheus::Error::Msg("Labels cannot be blank".to_string()));
                }
                trimmed_label_names.push(label);
            }
            Ok(trimmed_label_names)
        };

        let label_names = check_labels()?;
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
        metric_id: HistogramId,
        help: String,
        buckets: Vec<f64>,
        const_labels: Option<HashMap<String, String>>,
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
        metrics.insert(metric_id, (metric, Buckets(buckets)));
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
        metric_id: HistogramVecId,
        help: String,
        label_names: &[&str],
        buckets: Vec<f64>,
        const_labels: Option<HashMap<String, String>>,
    ) -> prometheus::Result<()> {
        let check_labels = || {
            if label_names.is_empty() {
                return Err(prometheus::Error::Msg(
                    "At least one label name must be provided".to_string(),
                ));
            }
            let mut trimmed_label_names: Vec<&str> = Vec::with_capacity(label_names.len());
            for label in label_names.iter() {
                let label = label.trim();
                if label.is_empty() {
                    return Err(prometheus::Error::Msg("Labels cannot be blank".to_string()));
                }
                trimmed_label_names.push(label);
            }
            Ok(trimmed_label_names)
        };

        let label_names = check_labels()?;
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

        let metric = prometheus::HistogramVec::new(opts, &label_names)?;
        self.registry.register(Box::new(metric.clone()))?;
        metrics.insert(metric_id, (metric, Buckets(buckets)));
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
        const_labels: Option<HashMap<String, String>>,
    ) -> Result<Option<HashMap<String, String>>, prometheus::Error> {
        match const_labels {
            Some(const_labels) => {
                let mut trimmed_const_labels = HashMap::with_capacity(const_labels.len());
                for (key, value) in const_labels {
                    let key = key.trim().to_string();
                    if key.is_empty() {
                        return Err(prometheus::Error::Msg(
                            "Const label key cannot be blank".to_string(),
                        ));
                    }

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
    pub fn histogram_vec(&self, metric_id: &HistogramVecId) -> Option<prometheus::HistogramVec> {
        let histogram_vecs = self.histogram_vecs.lock().unwrap();
        histogram_vecs
            .get(&metric_id)
            .map(|(metric, _opts)| metric.clone())
    }

    /// Returns a Histogram for the specified metric ID - if it is registered
    pub fn histogram(&self, metric_id: &HistogramId) -> Option<prometheus::Histogram> {
        let histograms = self.histograms.lock().unwrap();
        histograms
            .get(&metric_id)
            .map(|(metric, _opts)| metric.clone())
    }

    /// Returns a Counter for the specified metric ID - if it is registered
    pub fn counter(&self, metric_id: &CounterId) -> Option<prometheus::Counter> {
        let counters = self.counters.lock().unwrap();
        counters.get(&metric_id).cloned()
    }

    /// Returns an IntCounter for the specified metric ID - if it is registered
    pub fn int_counter(&self, metric_id: &IntCounterId) -> Option<prometheus::IntCounter> {
        let counters = self.int_counters.lock().unwrap();
        counters.get(&metric_id).cloned()
    }

    /// Returns a CounterVec for the specified metric ID - if it is registered
    pub fn counter_vec(&self, metric_id: &CounterVecId) -> Option<prometheus::CounterVec> {
        let counter_vecs = self.counter_vecs.lock().unwrap();
        counter_vecs.get(&metric_id).cloned()
    }

    /// Returns a CounterVec for the specified metric ID - if it is registered
    pub fn int_counter_vec(
        &self,
        metric_id: &IntCounterVecId,
    ) -> Option<prometheus::IntCounterVec> {
        let int_counter_vecs = self.int_counter_vecs.lock().unwrap();
        int_counter_vecs.get(&metric_id).cloned()
    }

    /// gather calls the Collect method of the registered Collectors and then gathers the collected
    /// metrics into a lexicographically sorted slice of MetricFamily protobufs.
    pub fn gather(&self) -> Vec<prometheus::proto::MetricFamily> {
        self.registry.gather()
    }
}

impl fmt::Debug for MetricRegistry {
    /// TODO: the output is clunky - make it cleaner - perhaps a JSON view
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("MetricRegistry\n")?;
        f.write_str("==============\n")?;

        f.write_str("**********\n")?;
        f.write_str("Histograms\n")?;
        f.write_str("**********\n")?;
        {
            let histograms = self.histograms.lock().unwrap();
            for (_key, (histogram, buckets)) in histograms.iter() {
                writeln!(f, "----------")?;
                writeln!(f, "{:#?}", histogram.desc())?;
                writeln!(f, "{:#?}", buckets)?;
            }
        }

        f.write_str("*************\n")?;
        f.write_str("HistogramVecs\n")?;
        f.write_str("*************\n")?;
        {
            let histogram_vecs = self.histogram_vecs.lock().unwrap();
            for (_key, (histogram, buckets)) in histogram_vecs.iter() {
                writeln!(f, "-------------")?;
                writeln!(f, "{:#?}", histogram.desc())?;
                writeln!(f, "{:#?}", buckets)?;
            }
        }

        // TODO - rest of metrics

        Ok(())
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

/// Histogram buckets
#[derive(Debug, Clone)]
pub struct Buckets(pub Vec<f64>);

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

/// Metric Id
#[ulid]
pub struct GaugeId(pub u128);

impl GaugeId {
    /// returns the metric name
    /// - the MetricId ULID is prefixedwith 'M' to ensure it does not start with a number because
    ///   prometheus metric names must match the following pattern `[a-zA-Z_:][a-zA-Z0-9_:]*`
    pub fn name(&self) -> String {
        format!("M{}", self)
    }
}

/// Metric Id
#[ulid]
pub struct GaugeVecId(pub u128);

impl GaugeVecId {
    /// returns the metric name
    /// - the MetricId ULID is prefixedwith 'M' to ensure it does not start with a number because
    ///   prometheus metric names must match the following pattern `[a-zA-Z_:][a-zA-Z0-9_:]*`
    pub fn name(&self) -> String {
        format!("M{}", self)
    }
}

/// Metric Id
#[ulid]
pub struct IntGaugeId(pub u128);

impl IntGaugeId {
    /// returns the metric name
    /// - the MetricId ULID is prefixedwith 'M' to ensure it does not start with a number because
    ///   prometheus metric names must match the following pattern `[a-zA-Z_:][a-zA-Z0-9_:]*`
    pub fn name(&self) -> String {
        format!("M{}", self)
    }
}

/// Metric Id
#[ulid]
pub struct IntGaugeVecId(pub u128);

impl IntGaugeVecId {
    /// returns the metric name
    /// - the MetricId ULID is prefixedwith 'M' to ensure it does not start with a number because
    ///   prometheus metric names must match the following pattern `[a-zA-Z_:][a-zA-Z0-9_:]*`
    pub fn name(&self) -> String {
        format!("M{}", self)
    }
}

/// Metric Id
#[ulid]
pub struct CounterId(pub u128);

impl CounterId {
    /// returns the metric name
    /// - the MetricId ULID is prefixedwith 'M' to ensure it does not start with a number because
    ///   prometheus metric names must match the following pattern `[a-zA-Z_:][a-zA-Z0-9_:]*`
    pub fn name(&self) -> String {
        format!("M{}", self)
    }
}

/// Metric Id
#[ulid]
pub struct CounterVecId(pub u128);

impl CounterVecId {
    /// returns the metric name
    /// - the MetricId ULID is prefixedwith 'M' to ensure it does not start with a number because
    ///   prometheus metric names must match the following pattern `[a-zA-Z_:][a-zA-Z0-9_:]*`
    pub fn name(&self) -> String {
        format!("M{}", self)
    }
}

/// Metric Id
#[ulid]
pub struct IntCounterVecId(pub u128);

impl IntCounterVecId {
    /// returns the metric name
    /// - the MetricId ULID is prefixedwith 'M' to ensure it does not start with a number because
    ///   prometheus metric names must match the following pattern `[a-zA-Z_:][a-zA-Z0-9_:]*`
    pub fn name(&self) -> String {
        format!("M{}", self)
    }
}

/// Metric Id
#[ulid]
pub struct IntCounterId(pub u128);

impl IntCounterId {
    /// returns the metric name
    /// - the MetricId ULID is prefixedwith 'M' to ensure it does not start with a number because
    ///   prometheus metric names must match the following pattern `[a-zA-Z_:][a-zA-Z0-9_:]*`
    pub fn name(&self) -> String {
        format!("M{}", self)
    }
}

/// Metric Id
#[ulid]
pub struct HistogramId(pub u128);

impl HistogramId {
    /// returns the metric name
    /// - the MetricId ULID is prefixedwith 'M' to ensure it does not start with a number because
    ///   prometheus metric names must match the following pattern `[a-zA-Z_:][a-zA-Z0-9_:]*`
    pub fn name(&self) -> String {
        format!("M{}", self)
    }
}

/// Metric Id
#[ulid]
pub struct HistogramVecId(pub u128);

impl HistogramVecId {
    /// returns the metric name
    /// - the MetricId ULID is prefixedwith 'M' to ensure it does not start with a number because
    ///   prometheus metric names must match the following pattern `[a-zA-Z_:][a-zA-Z0-9_:]*`
    pub fn name(&self) -> String {
        format!("M{}", self)
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

    #[test]
    fn metric_registry_int_counter() {
        configure_logging();

        use crate::concurrent::messaging::reqrep::ReqRepId;
        use oysterpack_uid::ULID;

        let metric_id = IntCounterId::generate();
        let registry = MetricRegistry::default();
        registry
            .register_int_counter(metric_id, "ReqRep timer".to_string(), None)
            .unwrap();

        info!("{:#?}", registry);

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

        let metrics_family = registry.gather();
        info!("{:#?}", metrics_family);
        registry.text_encode_metrics(&mut std::io::stderr());
    }

    #[test]
    fn metric_registry_counter() {
        configure_logging();

        use crate::concurrent::messaging::reqrep::ReqRepId;
        use oysterpack_uid::ULID;

        let metric_id = CounterId::generate();
        let registry = MetricRegistry::default();
        registry
            .register_counter(metric_id, "ReqRep timer".to_string(), None)
            .unwrap();

        info!("{:#?}", registry);

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

        let metrics_family = registry.gather();
        info!("{:#?}", metrics_family);
        registry.text_encode_metrics(&mut std::io::stderr());
    }

    #[test]
    fn metric_registry_counter_vec() {
        configure_logging();

        use crate::concurrent::messaging::reqrep::ReqRepId;
        use oysterpack_uid::ULID;

        let metric_id = CounterVecId::generate();
        let registry = MetricRegistry::default();
        let label = LabelId::generate().name();
        let labels = vec![label.as_str()];
        registry
            .register_counter_vec(metric_id, "ReqRep timer".to_string(), &labels, None)
            .unwrap();

        info!("{:#?}", registry);

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

        let metrics_family = registry.gather();
        info!("{:#?}", metrics_family);
        registry.text_encode_metrics(&mut std::io::stderr());
    }

    #[test]
    fn metric_registry_int_counter_vec() {
        configure_logging();

        use crate::concurrent::messaging::reqrep::ReqRepId;
        use oysterpack_uid::ULID;

        let metric_id = IntCounterVecId::generate();
        let registry = MetricRegistry::default();
        let label = LabelId::generate().name();
        let labels = vec![label.as_str()];
        registry
            .register_int_counter_vec(metric_id, "ReqRep timer".to_string(), &labels, None)
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

        let metrics_family = registry.gather();
        info!("{:#?}", metrics_family);
        registry.text_encode_metrics(&mut std::io::stderr());
    }

    #[test]
    fn metric_registry_histogram_vec() {
        configure_logging();

        use crate::concurrent::messaging::reqrep::ReqRepId;
        use oysterpack_uid::ULID;

        let metric_id = HistogramVecId::generate();
        let registry = MetricRegistry::default();
        registry
            .register_histogram_vec(
                metric_id,
                "ReqRep timer".to_string(),
                &["REQREPID_1"],
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

        let metrics_family = registry.gather();
        info!("{:#?}", metrics_family);
        registry.text_encode_metrics(&mut std::io::stderr());
    }

    #[test]
    fn metric_registry_histogram() {
        configure_logging();

        use oysterpack_uid::ULID;

        let metric_id = HistogramId::generate();
        let registry = MetricRegistry::default();
        registry
            .register_histogram(
                metric_id,
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

        let metric_id = HistogramId::generate();
        let registry = MetricRegistry::default();
        registry
            .register_histogram(
                metric_id,
                "ReqRep timer".to_string(),
                vec![0.01, 0.025, 0.05, 0.005, 0.0050, 0.005], // will be sorted and deduped automatically
                None,
            )
            .unwrap();

        info!("{:#?}", registry);

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

        let metric_id = HistogramVecId::generate();
        let registry = MetricRegistry::default();
        let mut const_labels = HashMap::new();
        const_labels.insert("FOO  ".to_string(), "  BAR".to_string());
        registry
            .register_histogram_vec(
                metric_id,
                "ReqRep timer".to_string(),
                &["REQREPID_1"],
                vec![0.01, 0.025, 0.05, 0.005, 0.0050, 0.005], // will be sorted and deduped automatically
                Some(const_labels),
            )
            .unwrap();

        info!("{:#?}", registry);

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
        let label_pair = &metric.get_label()[0];
        assert_eq!(label_pair.get_name(), "FOO");
        assert_eq!(label_pair.get_value(), "BAR")
    }

    #[test]
    fn metric_registry_histogram_vec_with_blank_const_label() {
        configure_logging();

        use oysterpack_uid::ULID;

        let metric_id = HistogramVecId::generate();
        let registry = MetricRegistry::default();

        {
            let mut const_labels = HashMap::new();
            const_labels.insert("FOO  ".to_string(), "  ".to_string());
            let result = registry.register_histogram_vec(
                metric_id,
                "ReqRep timer".to_string(),
                &["REQREPID_1"],
                vec![0.01, 0.025, 0.05, 0.005, 0.0050, 0.005], // will be sorted and deduped automatically
                Some(const_labels),
            );
            info!("const label value is blank: {:?}", result);
            assert!(result.is_err());
            assert!(result.err().unwrap().to_string().contains("value"));
        }

        {
            let mut const_labels = HashMap::new();
            const_labels.insert("  ".to_string(), "BAR".to_string());
            let result = registry.register_histogram_vec(
                metric_id,
                "ReqRep timer".to_string(),
                &["REQREPID_1"],
                vec![0.01, 0.025, 0.05, 0.005, 0.0050, 0.005], // will be sorted and deduped automatically
                Some(const_labels),
            );
            info!("const label key is blank: {:?}", result);
            assert!(result.is_err());
            assert!(result.err().unwrap().to_string().contains("key"));
        }
    }

    #[test]
    fn metric_registry_histogram_vec_with_blank_help() {
        configure_logging();

        use oysterpack_uid::ULID;

        let metric_id = HistogramVecId::generate();
        let registry = MetricRegistry::default();

        let result = registry.register_histogram_vec(
            metric_id,
            " ".to_string(),
            &["REQREPID_1"],
            vec![0.01, 0.025, 0.05, 0.005, 0.0050, 0.005], // will be sorted and deduped automatically
            None,
        );
        info!("help is blank: {:?}", result);
        assert!(result.is_err());
    }

}
