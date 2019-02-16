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

//! Provides metrics support for [prometheus](https://prometheus.io/).
//! - provides a global [MetricRegistry](struct.MetricRegistry.html) via [registry()](fn.registry.html)
//!
//! ## Recommendations
//!
//! ### Use [MetricId](struct.MetricId.html) and [LabelId](struct.LabelId.html) for metric and label names
//! - because naming is hard ...
//! - because names should be unique and immutable over time
//!   - the prometheus metric `help` attribute can be used to provide a human friendly label and short description
//!
//! ### Use the Int version of the metrics where possible
//! - because they are more efficient
//! - IntCounter, IntCounterVec, IntGauge, IntGaugeVec

use lazy_static::lazy_static;
use oysterpack_uid::{macros::ulid, ulid_u128_into_string, ULID};
use prometheus::{core::Collector, Encoder};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt,
    hash::BuildHasher,
    io::Write,
    iter::Iterator,
    str::FromStr,
    sync::{Arc, RwLock},
    time::Duration,
};

lazy_static! {
    /// Global metrics registry
    static ref METRIC_REGISTRY: MetricRegistry = MetricRegistry::default();
}

/// Returns the global metric registry
pub fn registry() -> &'static MetricRegistry {
    &METRIC_REGISTRY
}

/// IntCounter constructor using MetricId and LabelId
pub fn new_int_counter<S: BuildHasher>(
    metric_id: MetricId,
    help: &str,
    const_labels: Option<HashMap<LabelId, String, S>>,
) -> prometheus::Result<prometheus::IntCounter> {
    let help = MetricRegistry::check_help(help)?;
    let const_labels = MetricRegistry::check_const_labels(const_labels)?;

    let mut opts = prometheus::Opts::new(metric_id.name(), help);
    if let Some(const_labels) = const_labels {
        opts = opts.const_labels(const_labels);
    }

    prometheus::IntCounter::with_opts(opts)
}

/// IntGauge constructor using MetricId and LabelId
pub fn new_int_gauge<S: BuildHasher>(
    metric_id: MetricId,
    help: &str,
    const_labels: Option<HashMap<LabelId, String, S>>,
) -> prometheus::Result<prometheus::IntGauge> {
    let help = MetricRegistry::check_help(help)?;
    let const_labels = MetricRegistry::check_const_labels(const_labels)?;

    let mut opts = prometheus::Opts::new(metric_id.name(), help);
    if let Some(const_labels) = const_labels {
        opts = opts.const_labels(const_labels);
    }

    prometheus::IntGauge::with_opts(opts)
}

/// Gauge constructor using MetricId and LabelId
pub fn new_gauge<S: BuildHasher>(
    metric_id: MetricId,
    help: &str,
    const_labels: Option<HashMap<LabelId, String, S>>,
) -> prometheus::Result<prometheus::Gauge> {
    let help = MetricRegistry::check_help(help)?;
    let const_labels = MetricRegistry::check_const_labels(const_labels)?;

    let mut opts = prometheus::Opts::new(metric_id.name(), help);
    if let Some(const_labels) = const_labels {
        opts = opts.const_labels(const_labels);
    }

    prometheus::Gauge::with_opts(opts)
}

/// GaugeVec constructor using MetricId and LabelId
pub fn new_gauge_vec<S: BuildHasher>(
    metric_id: MetricId,
    help: &str,
    label_ids: &[LabelId],
    const_labels: Option<HashMap<LabelId, String, S>>,
) -> prometheus::Result<prometheus::GaugeVec> {
    let label_names = MetricRegistry::check_labels(label_ids)?;
    let help = MetricRegistry::check_help(help)?;
    let const_labels = MetricRegistry::check_const_labels(const_labels)?;

    let mut opts = prometheus::Opts::new(metric_id.name(), help);
    if let Some(const_labels) = const_labels {
        opts = opts.const_labels(const_labels);
    }

    let label_names: Vec<&str> = label_names.iter().map(|label| label.as_str()).collect();
    prometheus::GaugeVec::new(opts, &label_names)
}

/// IntGaugeVec constructor using MetricId and LabelId
pub fn new_int_gauge_vec<S: BuildHasher>(
    metric_id: MetricId,
    help: &str,
    label_ids: &[LabelId],
    const_labels: Option<HashMap<LabelId, String, S>>,
) -> prometheus::Result<prometheus::IntGaugeVec> {
    let label_names = MetricRegistry::check_labels(label_ids)?;
    let help = MetricRegistry::check_help(help)?;
    let const_labels = MetricRegistry::check_const_labels(const_labels)?;

    let mut opts = prometheus::Opts::new(metric_id.name(), help);
    if let Some(const_labels) = const_labels {
        opts = opts.const_labels(const_labels);
    }

    let label_names: Vec<&str> = label_names.iter().map(|label| label.as_str()).collect();
    prometheus::IntGaugeVec::new(opts, &label_names)
}

/// Counter constructor using MetricId and LabelId
pub fn new_counter<S: BuildHasher>(
    metric_id: MetricId,
    help: &str,
    const_labels: Option<HashMap<LabelId, String, S>>,
) -> prometheus::Result<prometheus::Counter> {
    let help = MetricRegistry::check_help(help)?;
    let const_labels = MetricRegistry::check_const_labels(const_labels)?;

    let mut opts = prometheus::Opts::new(metric_id.name(), help);
    if let Some(const_labels) = const_labels {
        opts = opts.const_labels(const_labels);
    }

    prometheus::Counter::with_opts(opts)
}

/// IntCounterVec constructor using MetricId and LabelId
pub fn new_int_counter_vec<S: BuildHasher>(
    metric_id: MetricId,
    help: &str,
    label_ids: &[LabelId],
    const_labels: Option<HashMap<LabelId, String, S>>,
) -> prometheus::Result<prometheus::IntCounterVec> {
    let label_names = MetricRegistry::check_labels(label_ids)?;
    let help = MetricRegistry::check_help(help)?;
    let const_labels = MetricRegistry::check_const_labels(const_labels)?;

    let mut opts = prometheus::Opts::new(metric_id.name(), help);
    if let Some(const_labels) = const_labels {
        opts = opts.const_labels(const_labels);
    }

    let label_names: Vec<&str> = label_names.iter().map(|label| label.as_str()).collect();
    prometheus::IntCounterVec::new(opts, &label_names)
}

/// CounterVec constructor using MetricId and LabelId
pub fn new_counter_vec<S: BuildHasher>(
    metric_id: MetricId,
    help: &str,
    label_ids: &[LabelId],
    const_labels: Option<HashMap<LabelId, String, S>>,
) -> prometheus::Result<prometheus::CounterVec> {
    let label_names = MetricRegistry::check_labels(label_ids)?;
    let help = MetricRegistry::check_help(help)?;
    let const_labels = MetricRegistry::check_const_labels(const_labels)?;

    let mut opts = prometheus::Opts::new(metric_id.name(), help);
    if let Some(const_labels) = const_labels {
        opts = opts.const_labels(const_labels);
    }

    let label_names: Vec<&str> = label_names.iter().map(|label| label.as_str()).collect();
    prometheus::CounterVec::new(opts, &label_names)
}

/// Histogram constructor using MetricId and LabelId
pub fn new_histogram<S: BuildHasher>(
    metric_id: MetricId,
    help: &str,
    buckets: Vec<f64>,
    const_labels: Option<HashMap<LabelId, String, S>>,
) -> prometheus::Result<prometheus::Histogram> {
    let help = MetricRegistry::check_help(help)?;
    let buckets = MetricRegistry::check_buckets(buckets)?;
    let const_labels = MetricRegistry::check_const_labels(const_labels)?;

    let mut opts = prometheus::HistogramOpts::new(metric_id.name(), help).buckets(buckets.clone());
    if let Some(const_labels) = const_labels {
        opts = opts.const_labels(const_labels);
    }

    prometheus::Histogram::with_opts(opts)
}

/// HistogramVec constructor using MetricId and LabelId
pub fn new_histogram_vec<S: BuildHasher>(
    metric_id: MetricId,
    help: &str,
    label_ids: &[LabelId],
    buckets: Vec<f64>,
    const_labels: Option<HashMap<LabelId, String, S>>,
) -> prometheus::Result<prometheus::HistogramVec> {
    let label_names = MetricRegistry::check_labels(label_ids)?;
    let help = MetricRegistry::check_help(help)?;
    let buckets = MetricRegistry::check_buckets(buckets)?;
    let const_labels = MetricRegistry::check_const_labels(const_labels)?;

    let mut opts = prometheus::HistogramOpts::new(metric_id.name(), help).buckets(buckets.clone());
    if let Some(const_labels) = const_labels {
        opts = opts.const_labels(const_labels);
    }

    let label_names: Vec<&str> = label_names.iter().map(|label| label.as_str()).collect();
    prometheus::HistogramVec::new(opts, &label_names)
}

/// Metric Registry
/// - process metrics collector is automatically added
pub struct MetricRegistry {
    registry: prometheus::Registry,
    metric_collectors: RwLock<Vec<ArcCollector>>,
}

impl MetricRegistry {
    /// Registers a new metrics Collector.
    /// It returns an error if the descriptors provided by the Collector are invalid or if they —
    /// in combination with descriptors of already registered Collectors — do not fulfill the consistency
    /// and uniqueness criteria described in the documentation of Desc.
    ///
    /// If the provided Collector is equal to a Collector already registered (which includes the
    /// case of re-registering the same Collector), the AlreadyReg error returns.
    pub fn register(
        &self,
        collector: impl prometheus::core::Collector + 'static,
    ) -> prometheus::Result<ArcCollector> {
        let collector = ArcCollector::new(collector);
        self.registry.register(Box::new(collector.clone()))?;
        {
            let mut metric_collectors = self.metric_collectors.write().unwrap();
            metric_collectors.push(collector.clone());
        }
        Ok(collector)
    }

    /// Collects descriptors for registered metrics
    pub fn descs(&self) -> Vec<prometheus::core::Desc> {
        let metric_collectors = self.metric_collectors.read().unwrap();
        metric_collectors
            .iter()
            .flat_map(|collector| collector.desc())
            .cloned()
            .collect()
    }

    /// Collects descriptors for registered metrics that match the specified filter
    pub fn filter_descs<F>(&self, mut filter: F) -> Vec<prometheus::core::Desc>
    where
        F: FnMut(&prometheus::core::Desc) -> bool,
    {
        let metric_collectors = self.metric_collectors.read().unwrap();
        metric_collectors
            .iter()
            .flat_map(|collector| collector.desc())
            .filter(|desc| filter(desc))
            .cloned()
            .collect()
    }

    /// Returns descriptors for the specified MetricId(s)
    pub fn descs_for_metric_ids(&self, metric_ids: &[MetricId]) -> Vec<prometheus::core::Desc> {
        let metric_names = metric_ids
            .iter()
            .map(|id| id.name())
            .collect::<fnv::FnvHashSet<_>>();
        self.filter_descs(|desc| metric_names.contains(&desc.fq_name))
    }

    /// Returns descriptors for the specified MetricId
    pub fn descs_for_metric_id(&self, metric_id: MetricId) -> Vec<prometheus::core::Desc> {
        let metric_name = metric_id.name();
        self.filter_descs(|desc| desc.fq_name == metric_name)
    }

    /// Returns collectors that match against the specified filter
    pub fn filter_collectors<F>(&self, mut filter: F) -> Vec<ArcCollector>
    where
        F: FnMut(&ArcCollector) -> bool,
    {
        let metric_collectors = self.metric_collectors.read().unwrap();
        metric_collectors
            .iter()
            .filter(|collector| filter(collector))
            .cloned()
            .collect()
    }

    /// Returns collectors that contain metric descriptors for the specified MetricId(s)
    pub fn collectors_for_metric_ids(&self, metric_ids: &[MetricId]) -> Vec<ArcCollector> {
        let metric_names = metric_ids
            .iter()
            .map(|id| id.name())
            .collect::<fnv::FnvHashSet<_>>();
        self.filter_collectors(|c| {
            c.desc()
                .iter()
                .any(|desc| metric_names.contains(&desc.fq_name))
        })
    }

    /// Returns collectors that contain metric descriptors for the specified MetricId
    pub fn collectors_for_metric_id(&self, metric_id: MetricId) -> Vec<ArcCollector> {
        let metric_name = metric_id.name();
        self.filter_collectors(|c| c.desc().iter().any(|desc| desc.fq_name == metric_name))
    }

    /// Returns the registered collectors
    pub fn collectors(&self) -> Vec<ArcCollector> {
        let metric_collectors = self.metric_collectors.read().unwrap();
        metric_collectors.iter().cloned().collect()
    }

    /// Returns the number of metric families that would be gathered without gathering metrics.
    /// The number of metric families equates to the total number of unique registered metric descriptor
    /// fully qualified names.
    ///
    /// ## Notes
    /// Each metric family may map to more than 1 metric Desc depending on label values
    pub fn metric_family_count(&self) -> usize {
        let metric_collectors = self.metric_collectors.read().unwrap();
        let mut desc_names = metric_collectors
            .iter()
            .flat_map(|collector| collector.desc())
            .collect::<Vec<_>>();
        desc_names.dedup_by(|desc1, desc2| desc1.fq_name == desc2.fq_name);
        desc_names.len()
    }

    /// Returns the number of registered collectors
    pub fn collector_count(&self) -> usize {
        let metric_collectors = self.metric_collectors.read().unwrap();
        metric_collectors.len()
    }

    /// Tries to register an IntGauge metric
    pub fn register_int_gauge(
        &self,
        metric_id: MetricId,
        help: &str,
        const_labels: Option<HashMap<LabelId, String>>,
    ) -> prometheus::Result<prometheus::IntGauge> {
        let metric = new_int_gauge(metric_id, help, const_labels)?;
        self.register(metric.clone())?;
        Ok(metric)
    }

    /// Tries to register an Gauge metric
    pub fn register_gauge(
        &self,
        metric_id: MetricId,
        help: &str,
        const_labels: Option<HashMap<LabelId, String>>,
    ) -> prometheus::Result<prometheus::Gauge> {
        let metric = new_gauge(metric_id, help, const_labels)?;
        self.register(metric.clone())?;
        Ok(metric)
    }

    /// Tries to register a GaugeVec metric
    pub fn register_gauge_vec(
        &self,
        metric_id: MetricId,
        help: &str,
        label_ids: &[LabelId],
        const_labels: Option<HashMap<LabelId, String>>,
    ) -> prometheus::Result<prometheus::GaugeVec> {
        let metric = new_gauge_vec(metric_id, help, label_ids, const_labels)?;
        self.register(metric.clone())?;
        Ok(metric)
    }

    /// Tries to register a IntGaugeVec metric
    pub fn register_int_gauge_vec(
        &self,
        metric_id: MetricId,
        help: &str,
        label_ids: &[LabelId],
        const_labels: Option<HashMap<LabelId, String>>,
    ) -> prometheus::Result<prometheus::IntGaugeVec> {
        let metric = new_int_gauge_vec(metric_id, help, label_ids, const_labels)?;
        self.register(metric.clone())?;
        Ok(metric)
    }

    /// Tries to register an IntCounter metric
    pub fn register_int_counter(
        &self,
        metric_id: MetricId,
        help: &str,
        const_labels: Option<HashMap<LabelId, String>>,
    ) -> prometheus::Result<prometheus::IntCounter> {
        let metric = new_int_counter(metric_id, help, const_labels)?;
        self.register(metric.clone())?;
        Ok(metric)
    }

    /// Tries to register a Counter metric
    pub fn register_counter(
        &self,
        metric_id: MetricId,
        help: &str,
        const_labels: Option<HashMap<LabelId, String>>,
    ) -> prometheus::Result<prometheus::Counter> {
        let metric = new_counter(metric_id, help, const_labels)?;
        self.register(metric.clone())?;
        Ok(metric)
    }

    /// Tries to register an IntCounterVec metric
    pub fn register_int_counter_vec(
        &self,
        metric_id: MetricId,
        help: &str,
        label_ids: &[LabelId],
        const_labels: Option<HashMap<LabelId, String>>,
    ) -> prometheus::Result<prometheus::IntCounterVec> {
        let metric = new_int_counter_vec(metric_id, help, label_ids, const_labels)?;
        self.register(metric.clone())?;
        Ok(metric)
    }

    /// Tries to register a CounterVec metric
    pub fn register_counter_vec(
        &self,
        metric_id: MetricId,
        help: &str,
        label_ids: &[LabelId],
        const_labels: Option<HashMap<LabelId, String>>,
    ) -> prometheus::Result<prometheus::CounterVec> {
        let metric = new_counter_vec(metric_id, help, label_ids, const_labels)?;
        self.register(metric.clone())?;
        Ok(metric)
    }

    /// Tries to register a Histogram metric
    pub fn register_histogram(
        &self,
        metric_id: MetricId,
        help: &str,
        buckets: Vec<f64>,
        const_labels: Option<HashMap<LabelId, String>>,
    ) -> prometheus::Result<prometheus::Histogram> {
        let metric = new_histogram(metric_id, help, buckets, const_labels)?;
        self.register(metric.clone())?;
        Ok(metric)
    }

    /// Tries to register a Histogram metric that is meant to be used as timer metric
    pub fn register_histogram_timer(
        &self,
        metric_id: MetricId,
        help: &str,
        buckets: TimerBuckets,
        const_labels: Option<HashMap<LabelId, String>>,
    ) -> prometheus::Result<prometheus::Histogram> {
        self.register_histogram(metric_id, help, buckets.into(), const_labels)
    }

    /// Tries to register a HistogramVec metric
    pub fn register_histogram_vec(
        &self,
        metric_id: MetricId,
        help: &str,
        label_ids: &[LabelId],
        buckets: Vec<f64>,
        const_labels: Option<HashMap<LabelId, String>>,
    ) -> prometheus::Result<prometheus::HistogramVec> {
        let metric = new_histogram_vec(metric_id, help, label_ids, buckets, const_labels)?;
        self.register(metric.clone())?;
        Ok(metric)
    }

    /// Tries to register a HistogramVec metric that is meant to be used as timer metric
    pub fn register_histogram_vec_timer(
        &self,
        metric_id: MetricId,
        help: &str,
        label_ids: &[LabelId],
        buckets: TimerBuckets,
        const_labels: Option<HashMap<LabelId, String>>,
    ) -> prometheus::Result<prometheus::HistogramVec> {
        self.register_histogram_vec(metric_id, help, label_ids, buckets.into(), const_labels)
    }

    fn check_help(help: &str) -> Result<String, prometheus::Error> {
        let help = help.trim();
        if help.is_empty() {
            Err(prometheus::Error::Msg(
                "help is required and cannot be blank".to_string(),
            ))
        } else {
            Ok(help.to_string())
        }
    }

    fn check_const_labels<S: BuildHasher>(
        const_labels: Option<HashMap<LabelId, String, S>>,
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
            fn dedupe(mut buckets: Vec<f64>) -> Vec<f64> {
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

            fn sort(mut buckets: Vec<f64>) -> Vec<f64> {
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

    /// gathers metrics from all registered metric collectors
    pub fn gather(&self) -> Vec<prometheus::proto::MetricFamily> {
        self.registry.gather()
    }

    /// gather metrics for collectors for the specified desc ids
    /// - Desc.id maps to a compound key composed of: `(Desc.fq_name, [Desc.const_label_values])`,
    ///   i.e., it enables you to gather metrics with specific constant label values
    ///   - if metrics do not have constant labels, then the id maps to `Desc.fq_name`
    /// - the returned MetricFamily will contain only the requested metrics
    pub fn gather_metrics(&self, desc_ids: &[u64]) -> Vec<prometheus::proto::MetricFamily> {
        let collectors = self.metric_collectors.read().unwrap();
        //BUG: a collector may have more than 1 matching desc

        let descs = self.filter_descs(|desc| desc_ids.iter().any(|id| *id == desc.id));

        collectors
            .iter()
            .filter(|collector| {
                // do any of the collector's desc match on id
                collector
                    .desc()
                    .iter()
                    .any(|desc| desc_ids.iter().any(|desc_id| *desc_id == desc.id))
            })
            .flat_map(|collector| collector.collect())
            .filter_map(|mut mf| {
                // filter out MetricFamily that does not match any Desc
                // a collector may return multiple MetricFamily
                match descs
                    .iter()
                    .find(|desc| desc.fq_name.as_str() == mf.get_name())
                {
                    None => None,
                    Some(desc) => {
                        // A MetricFamily may consist of more than 1 metric
                        let metrics = mf.get_metric();
                        if metrics.len() > 1 && !desc.const_label_pairs.is_empty() {
                            // filter out metrics that do not match const label values
                            let metrics = mf.mut_metric();
                            let mut i = 0;
                            while i < metrics.len() {
                                let metric = &metrics[i];
                                for label_pair in metric.get_label() {
                                    let value = label_pair.get_value();
                                    if desc
                                        .const_label_pairs
                                        .iter()
                                        .find(|label_pair| value == label_pair.get_value())
                                        .is_none()
                                    {
                                        metrics.remove(i);
                                        break;
                                    }
                                }
                                i += 1
                            }
                        }
                        Some(mf)
                    }
                }
            })
            .collect()
    }

    /// gather metrics for collectors for the specified metric fully qualified names
    pub fn gather_metrics_by_name(
        &self,
        desc_names: &[&str],
    ) -> Vec<prometheus::proto::MetricFamily> {
        let collectors = self.metric_collectors.read().unwrap();
        collectors
            .iter()
            .filter(|collector| {
                collector
                    .desc()
                    .iter()
                    .any(|desc| desc_names.iter().any(|name| *name == desc.fq_name.as_str()))
            })
            .flat_map(|collector| {
                collector
                    .collect()
                    .into_iter()
                    .filter(|mf| desc_names.iter().any(|name| *name == mf.get_name()))
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    /// Gathers process related metrics
    pub fn gather_process_metrics(&self) -> ProcessMetrics {
        let collectors = self.metric_collectors.read().unwrap();
        // the ProcessCollector will always be the first registered collector
        ProcessMetrics::collect(&collectors[0])
    }
}

impl fmt::Debug for MetricRegistry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MetricRegistry")
    }
}

impl Default for MetricRegistry {
    fn default() -> Self {
        let registry = Self {
            registry: prometheus::Registry::new(),
            metric_collectors: RwLock::new(Vec::new()),
        };

        registry
            .register(prometheus::process_collector::ProcessCollector::for_self())
            .unwrap();

        registry
    }
}

/// Label Id
#[ulid]
pub struct LabelId(pub u128);

impl LabelId {
    /// returns the metric name
    /// - the LabelId ULID is prefixedwith 'L' to ensure it does not start with a number because
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

impl From<ULID> for MetricId {
    fn from(ulid: ULID) -> Self {
        Self(ulid.into())
    }
}

/// Times how long it takes to run the function in nanos.
///
/// ## Use Case
/// Used to record timings which can then be reported on a Histogram metric
///
/// ### Example
/// ```rust
/// # use oysterpack_trust::metrics::*;
///
/// const METRIC_ID: MetricId = MetricId(1872045779718506837202123142606941790);
///    let registry = MetricRegistry::default();
///    let mut reqrep_timer_local = registry
///        .register_histogram_vec(
///            METRIC_ID,
///            "ReqRep timer",
///            &[LabelId::generate()],
///            vec![0.01, 0.025, 0.05, 0.1],
///            None,
///        )
///        .unwrap();
///
/// let reqrep_timer =
///        reqrep_timer_local.with_label_values(&["A"]);
///    let clock = quanta::Clock::new();
///    for _ in 0..10 {
///        // time the work
///        let delta = time(&clock, || std::thread::sleep(std::time::Duration::from_millis(1)));
///        // report the time in seconds
///        reqrep_timer.observe(as_float_secs(delta));
///    }
/// ```
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

/// Used to specify histogram buckets that will be used as timer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerBuckets(smallvec::SmallVec<[Duration; 10]>);

impl TimerBuckets {
    /// adds a new bucket
    pub fn add_bucket(mut self, upper_boundary: Duration) -> TimerBuckets {
        self.0.push(upper_boundary);
        self
    }

    /// returns the buckets
    pub fn buckets(&self) -> &[Duration] {
        self.0.as_slice()
    }
}

impl From<&[Duration]> for TimerBuckets {
    fn from(buckets: &[Duration]) -> Self {
        Self(smallvec::SmallVec::from_slice(buckets))
    }
}

impl From<Vec<Duration>> for TimerBuckets {
    fn from(buckets: Vec<Duration>) -> Self {
        Self(smallvec::SmallVec::from_slice(buckets.as_slice()))
    }
}

impl Into<Vec<f64>> for TimerBuckets {
    fn into(self) -> Vec<f64> {
        self.0
            .into_iter()
            .map(|duration| duration.as_float_secs())
            .collect()
    }
}

/// Arc wrapped metrics collector
/// - metric collectors that are registered are stored within the MetricRegistry within an ArcCollector
/// - this enables the collectors to be shared and used across threads
#[derive(Clone)]
pub struct ArcCollector(Arc<dyn prometheus::core::Collector + 'static>);

impl ArcCollector {
    fn new(collector: impl prometheus::core::Collector + 'static) -> Self {
        ArcCollector(Arc::new(collector))
    }
}

impl prometheus::core::Collector for ArcCollector {
    /// Return descriptors for metrics.
    fn desc(&self) -> Vec<&prometheus::core::Desc> {
        self.0.desc()
    }

    /// Collect metrics.
    fn collect(&self) -> Vec<prometheus::proto::MetricFamily> {
        self.0.collect()
    }
}

impl fmt::Debug for ArcCollector {
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

    /// Process metric names
    pub const METRIC_NAMES: [&'static str; 6] = [
        Self::PROCESS_CPU_SECONDS_TOTAL,
        Self::PROCESS_OPEN_FDS,
        Self::PROCESS_MAX_FDS,
        Self::PROCESS_VIRTUAL_MEMORY_BYTES,
        Self::PROCESS_RESIDENT_MEMORY_BYTES,
        Self::PROCESS_START_TIME_SECONDS,
    ];

    fn collect(process_collector: &ArcCollector) -> Self {
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
