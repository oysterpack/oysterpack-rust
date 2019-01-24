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

//! Provides metrics support

use oysterpack_uid::macros::ulid;
use prometheus::Encoder;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt, io::Write, sync::Mutex};

/// Metric Registry
pub struct MetricRegistry {
    registry: prometheus::Registry,
    histogram_vecs:
        Mutex<fnv::FnvHashMap<MetricId, (prometheus::HistogramVec, prometheus::HistogramOpts)>>,
}

impl MetricRegistry {
    /// Tries to register a HistogramVec metric
    pub fn register_histogram_vec(
        &self,
        metric_id: MetricId,
        help: String,
        label_names: &[&str],
        buckets: Vec<f64>,
        const_labels: Option<HashMap<String, String>>,
    ) -> prometheus::Result<()> {
        if label_names.len() == 0 {
            return Err(prometheus::Error::Msg(
                "At least one label name must be provided".to_string(),
            ));
        }
        let mut histogram_vecs = self.histogram_vecs.lock().unwrap();
        if histogram_vecs.contains_key(&metric_id) {
            return Err(prometheus::Error::AlreadyReg);
        }

        let mut opts =
            prometheus::HistogramOpts::new(format!("M{}", metric_id), help).buckets(buckets);
        if let Some(const_labels) = const_labels {
            opts = opts.const_labels(const_labels);
        }

        let metric = prometheus::HistogramVec::new(opts.clone(), label_names)?;
        self.registry.register(Box::new(metric.clone()))?;
        histogram_vecs.insert(metric_id, (metric, opts));
        Ok(())
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

    /// gather calls the Collect method of the registered Collectors and then gathers the collected
    /// metrics into a lexicographically sorted slice of MetricFamily protobufs.
    pub fn gather(&self) -> Vec<prometheus::proto::MetricFamily> {
        self.registry.gather()
    }
}

impl fmt::Debug for MetricRegistry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let histogram_vecs = self.histogram_vecs.lock().unwrap();
        // TODO: write HistogramOpts
        write!(f, "histogram_vecs: {:#?}", histogram_vecs.keys())
    }
}

impl Default for MetricRegistry {
    fn default() -> Self {
        Self {
            registry: prometheus::Registry::new(),
            histogram_vecs: Mutex::new(fnv::FnvHashMap::default()),
        }
    }
}

/// Metric Id
///
/// ### Why use a number as a metric name ?
/// Because names change over time, which can break components that depend on metric names ...
/// Assigning unique numerical identifiers is much more stable. Human friendly metric labels and any
/// additional information can be mapped externally to the MetricId.
#[ulid]
pub struct MetricId(pub u128);

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
    fn metrics_prometheus_histogram_vec_as_timer() {
        configure_logging();

        use crate::concurrent::messaging::reqrep::ReqRepId;
        use oysterpack_uid::ULID;

        let REQREP_TIMER = format!("M{}", ULID::generate());
        let REQREP_SERVICE_ID_LABEL = format!("L{}", ULID::generate());

        let registry = prometheus::Registry::new();
        let opts = prometheus::HistogramOpts::new(REQREP_TIMER, "reqrep timer".to_string());

        let REQREP_TIMER =
            prometheus::HistogramVec::new(opts, &[REQREP_SERVICE_ID_LABEL.as_str()]).unwrap();
        registry.register(Box::new(REQREP_TIMER.clone())).unwrap();

        let mut reqrep_timer_local = REQREP_TIMER.local();
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
                metric_id,
                "ReqRep timer".to_string(),
                &["REQREPID_1"],
                vec![
                    0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
                ],
                None,
            )
            .unwrap();

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
    }

}
