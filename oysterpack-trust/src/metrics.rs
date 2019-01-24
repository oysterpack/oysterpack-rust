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

use lazy_static::lazy_static;
use oysterpack_uid::macros::ulid;
use prometheus::Encoder;
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
    histogram_vecs:
        Mutex<fnv::FnvHashMap<MetricId, (prometheus::HistogramVec, prometheus::HistogramOpts)>>,
    histograms:
        Mutex<fnv::FnvHashMap<MetricId, (prometheus::Histogram, prometheus::HistogramOpts)>>,
}

impl MetricRegistry {
    /// Tries to register a Histogram metric
    pub fn register_histogram(
        &self,
        metric_id: MetricId,
        help: String,
        buckets: Vec<f64>,
        const_labels: Option<HashMap<String, String>>,
    ) -> prometheus::Result<()> {
        let help = Self::check_help(help)?;
        let buckets = Self::check_buckets(buckets)?;
        let const_labels = Self::check_const_labels(const_labels)?;

        let mut histograms = self.histograms.lock().unwrap();
        if histograms.contains_key(&metric_id) {
            return Err(prometheus::Error::AlreadyReg);
        }

        let mut opts = prometheus::HistogramOpts::new(metric_id.name(), help)
            .buckets(Self::sort_dedupe(buckets));
        if let Some(const_labels) = const_labels {
            opts = opts.const_labels(const_labels);
        }

        let metric = prometheus::Histogram::with_opts(opts.clone())?;
        self.registry.register(Box::new(metric.clone()))?;
        histograms.insert(metric_id, (metric, opts));
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
        metric_id: MetricId,
        help: String,
        label_names: &[&str],
        buckets: Vec<f64>,
        const_labels: Option<HashMap<String, String>>,
    ) -> prometheus::Result<()> {
        let check_labels = || {
            if label_names.len() == 0 {
                return Err(prometheus::Error::Msg(
                    "At least one label name must be provided".to_string(),
                ));
            }
            let mut trimmed_label_names: Vec<&str> = Vec::with_capacity(label_names.len());
            for label in label_names.iter() {
                let label = label.trim();
                if label.len() == 0 {
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

        let mut histogram_vecs = self.histogram_vecs.lock().unwrap();
        if histogram_vecs.contains_key(&metric_id) {
            return Err(prometheus::Error::AlreadyReg);
        }

        let mut opts = prometheus::HistogramOpts::new(metric_id.name(), help)
            .buckets(Self::sort_dedupe(buckets));
        if let Some(const_labels) = const_labels {
            opts = opts.const_labels(const_labels);
        }

        let metric = prometheus::HistogramVec::new(opts.clone(), &label_names)?;
        self.registry.register(Box::new(metric.clone()))?;
        histogram_vecs.insert(metric_id, (metric, opts));
        Ok(())
    }

    fn check_help(help: String) -> Result<String, prometheus::Error> {
        let help = help.trim();
        if help.len() == 0 {
            Err(prometheus::Error::Msg("help is required and cannot be blank".to_string()))
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
                    if key.len() == 0 {
                        return Err(prometheus::Error::Msg(
                            "Const label key cannot be blank".to_string(),
                        ));
                    }

                    let value = value.trim().to_string();
                    if value.len() == 0 {
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
        if buckets.is_empty() {
            return Err(prometheus::Error::Msg(
                "At least 1 bucket must be defined".to_string(),
            ));
        }
        Ok(buckets)
    }

    fn sort_dedupe(buckets: Vec<f64>) -> Vec<f64> {
        fn dedupe(buckets: Vec<f64>) -> Vec<f64> {
            let mut buckets = buckets;
            if buckets.len() > 1 {
                let mut i = 1;
                let mut found_dups = false;
                while i < buckets.len() {
                    if !(buckets[i - 1] < buckets[i]) {
                        buckets.remove(i);
                        found_dups = true;
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

    /// Text encodes a snapshot of the current metrics
    pub fn text_encode_metrics<W: Write>(&self, writer: &mut W) -> prometheus::Result<()> {
        let metric_families = self.registry.gather();
        let encoder = prometheus::TextEncoder::new();
        encoder.encode(&metric_families, writer)
    }

    /// Returns a LocalHistogramVec for the specified MetricId - if it is registered
    pub fn histogram_vec(
        &self,
        metric_id: &MetricId,
    ) -> Option<prometheus::local::LocalHistogramVec> {
        let histogram_vecs = self.histogram_vecs.lock().unwrap();
        histogram_vecs
            .get(&metric_id)
            .map(|(metric, _opts)| metric.local())
    }

    /// Returns a LocalHistogram for the specified MetricId - if it is registered
    pub fn histogram(
        &self,
        metric_id: &MetricId,
    ) -> Option<prometheus::local::LocalHistogram> {
        let histograms = self.histograms.lock().unwrap();
        histograms
            .get(&metric_id)
            .map(|(metric, _opts)| metric.local())
    }

    /// gather calls the Collect method of the registered Collectors and then gathers the collected
    /// metrics into a lexicographically sorted slice of MetricFamily protobufs.
    pub fn gather(&self) -> Vec<prometheus::proto::MetricFamily> {
        self.registry.gather()
    }
}

impl fmt::Debug for MetricRegistry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        #[derive(Debug)]
        struct HistogramOpts {
            pub opts: prometheus::Opts,
            pub buckets: Vec<f64>,
        }

        let mut metrics = fnv::FnvHashMap::<MetricId, HistogramOpts>::default();
        {
            let histogram_vecs = self.histogram_vecs.lock().unwrap();

            for (key, value) in histogram_vecs.iter() {
                metrics.insert(
                    key.clone(),
                    HistogramOpts {
                        opts: value.1.common_opts.clone(),
                        buckets: value.1.buckets.clone(),
                    },
                );
            }
        }

        write!(
            f,
            r#"MetricRegistry
==============
histogram_vecs: {:#?}"#,
            metrics
        )
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
            registry: registry,
            histogram_vecs: Mutex::new(fnv::FnvHashMap::default()),
            histograms: Mutex::new(fnv::FnvHashMap::default()),
        }
    }
}

/// Metric Id
///
/// ### Why use a number as a metric name ?
/// Because names change over time, which can break components that depend on metric names ...
/// Assigning unique numerical identifiers is much more stable. Human friendly metric labels and any
/// additional information can be mapped externally to the MetricId.
///
/// ### Notes
/// - for prometheus metrics use the metric `help` attribute to provide a human friendly label and
///   short description
#[ulid]
pub struct MetricId(pub u128);

impl MetricId {
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
    (nanos as f64) / (NANOS_PER_SEC as f64)
}

#[allow(warnings)]
#[cfg(test)]
mod tests {

    use super::*;
    use crate::configure_logging;
    use oysterpack_log::*;
    use std::{thread, time::Duration};

    #[test]
    fn metric_registry_histogram_vec() {
        configure_logging();

        use crate::concurrent::messaging::reqrep::ReqRepId;
        use oysterpack_uid::ULID;

        let metric_id = MetricId::generate();
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

        let mut reqrep_timer_local = registry.histogram_vec(&metric_id).unwrap();
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

        let metric_id = MetricId::generate();
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
    fn metric_registry_histogram_using_timer() {
        configure_logging();

        use oysterpack_uid::ULID;

        let metric_id = MetricId::generate();
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
        for _ in 0..10 {
            let ulid_u128: u128 = ULID::generate().into();
            let sleep_ms = (ulid_u128 % 100) as u32;
            info!("sleeping for {}", sleep_ms);
            {
                let timer = reqrep_timer.start_timer();
                thread::sleep_ms(sleep_ms)
            }
        }

        let metrics_family = registry.gather();
        info!("{:#?}", metrics_family);
        registry.text_encode_metrics(&mut std::io::stderr());
    }

    #[test]
    fn metric_registry_histogram_vec_with_const_labels() {
        configure_logging();

        use oysterpack_uid::ULID;

        let metric_id = MetricId::generate();
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

        let mut reqrep_timer_local = registry.histogram_vec(&metric_id).unwrap();
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

        let metric_id = MetricId::generate();
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

        let metric_id = MetricId::generate();
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
