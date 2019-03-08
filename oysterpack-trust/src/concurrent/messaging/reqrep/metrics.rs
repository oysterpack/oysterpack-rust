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

//! request / reply related metrics

use super::{ReqRepId, ReqRepServiceMetrics};
use oysterpack_uid::ULID;
use std::sync::RwLock;

lazy_static::lazy_static! {
    pub(super) static ref REQ_REP_METRICS: RwLock<fnv::FnvHashMap<ReqRepId, ReqRepServiceMetrics>> = RwLock::new(fnv::FnvHashMap::default());

    pub(crate) static ref REQ_REP_SERVICE_INSTANCE_COUNT: prometheus::IntGaugeVec = crate::metrics::registry().register_int_gauge_vec(
        SERVICE_INSTANCE_COUNT_METRIC_ID,
        "ReqRep service instance count",
        &[REQREPID_LABEL_ID],
        None,
    ).unwrap();

    pub(crate) static ref REQREP_SEND_COUNTER: prometheus::IntCounterVec = crate::metrics::registry().register_int_counter_vec(
        REQREP_SEND_COUNTER_METRIC_ID,
        "ReqRep request send count",
        &[REQREPID_LABEL_ID],
        None,
    ).unwrap();

    pub(crate) static ref PROCESSOR_PANIC_COUNTER: prometheus::IntCounterVec = crate::metrics::registry().register_int_counter_vec(
        PROCESSOR_PANIC_COUNTER_METRIC_ID,
        "Processor FutureReply panic count",
        &[REQREPID_LABEL_ID],
        None,
    ).unwrap();
}

/// ReqRep service instance count MetricId: `M01D2Q7VG1HFFXG6JT6HD11ZCJ3`
/// - metric type is IntGaugeVec
pub const SERVICE_INSTANCE_COUNT_METRIC_ID: crate::metrics::MetricId =
    crate::metrics::MetricId(1872765971344832352273831154704953923);

/// ReqRep backend message processing timer MetricId: `M01D4ZMEFGBPCK2HSNAPGBARR14`
/// - metric type is Histogram
pub const REQREP_PROCESS_TIMER_METRIC_ID: crate::metrics::MetricId =
    crate::metrics::MetricId(1875702602137856142367281339226152996);

/// The ReqRepId ULID will be used as the label value: `L01D2Q81HQJJVPQZSQE7BHH67JK`
pub const REQREPID_LABEL_ID: crate::metrics::LabelId =
    crate::metrics::LabelId(1872766211119679891800112881745469011);

/// ReqRep request send counter MetricId: `M01D52BD1MYW4GJ2VN44S14R28Z`
/// - metric type is IntCounterVec
pub const REQREP_SEND_COUNTER_METRIC_ID: crate::metrics::MetricId =
    crate::metrics::MetricId(1875812830972763422767373669165173023);

/// ReqRep request send counter MetricId: `M01D52BD1MYW4GJ2VN44S14R28Z`
/// - metric type is IntCounterVec
pub const PROCESSOR_PANIC_COUNTER_METRIC_ID: crate::metrics::MetricId =
    crate::metrics::MetricId(1876035517884156224063178768953919720);

/// Gathers metrics related to ReqRep
pub fn gather() -> Vec<prometheus::proto::MetricFamily> {
    crate::metrics::registry().gather_for_metric_ids(metric_ids().as_slice())
}

/// ReqRep related metric descriptors
pub fn descs() -> Vec<prometheus::core::Desc> {
    crate::metrics::registry().descs_for_metric_ids(metric_ids().as_slice())
}

/// ReqRep related MetricId(s)
pub fn metric_ids() -> Vec<crate::metrics::MetricId> {
    vec![
        SERVICE_INSTANCE_COUNT_METRIC_ID,
        REQREP_PROCESS_TIMER_METRIC_ID,
        REQREP_SEND_COUNTER_METRIC_ID,
        PROCESSOR_PANIC_COUNTER_METRIC_ID,
    ]
}

/// return the ReqRep backend service count
pub fn service_instance_count(reqrep_id: ReqRepId) -> u64 {
    let label_name = REQREPID_LABEL_ID.name();
    let label_value = reqrep_id.to_string();
    crate::metrics::registry()
        .gather_for_desc_names(&[SERVICE_INSTANCE_COUNT_METRIC_ID.name().as_str()])
        .iter()
        .filter_map(|mf| {
            mf.get_metric()
                .iter()
                .find(|metric| {
                    metric.get_label().iter().any(|label_pair| {
                        label_pair.get_name() == label_name && label_pair.get_value() == label_value
                    })
                })
                .map(|mf| mf.get_gauge().get_value() as u64)
        })
        .next()
        .unwrap_or(0)
}

/// return the ReqRep backend service count
pub fn service_instance_counts() -> fnv::FnvHashMap<ReqRepId, u64> {
    crate::metrics::registry()
        .gather_for_desc_names(&[SERVICE_INSTANCE_COUNT_METRIC_ID.name().as_str()])
        .first()
        .map(|mf| {
            let label_name = REQREPID_LABEL_ID.name();
            let label_name = label_name.as_str();
            let metrics = mf.get_metric();
            let counts = fnv::FnvHashMap::with_capacity_and_hasher(
                metrics.len(),
                fnv::FnvBuildHasher::default(),
            );
            metrics.iter().fold(counts, |mut counts, metric| {
                let reqrep_id = metric
                    .get_label()
                    .iter()
                    .find_map(|label_pair| {
                        if label_pair.get_name() == label_name {
                            Some(ReqRepId::from(
                                label_pair.get_value().parse::<ULID>().unwrap(),
                            ))
                        } else {
                            None
                        }
                    })
                    .unwrap();
                let gauge = metric.get_gauge();
                counts.insert(reqrep_id, gauge.get_value() as u64);
                counts
            })
        })
        .unwrap_or_else(fnv::FnvHashMap::default)
}

/// return the ReqRep request send count
pub fn request_send_count(reqrep_id: ReqRepId) -> u64 {
    count(reqrep_id, REQREP_SEND_COUNTER_METRIC_ID)
}

/// return the ReqRep backend service count
pub fn request_send_counts() -> fnv::FnvHashMap<ReqRepId, u64> {
    counts(REQREP_SEND_COUNTER_METRIC_ID)
}

/// return the ReqRep service Processor panic count
pub fn processor_panic_count(reqrep_id: ReqRepId) -> u64 {
    count(reqrep_id, PROCESSOR_PANIC_COUNTER_METRIC_ID)
}

/// return the ReqRep backend service count
pub fn processor_panic_counts() -> fnv::FnvHashMap<ReqRepId, u64> {
    counts(PROCESSOR_PANIC_COUNTER_METRIC_ID)
}

fn count(reqrep_id: ReqRepId, metric_id: crate::metrics::MetricId) -> u64 {
    let label_name = REQREPID_LABEL_ID.name();
    let label_value = reqrep_id.to_string();
    crate::metrics::registry()
        .gather_for_desc_names(&[metric_id.name().as_str()])
        .iter()
        .filter_map(|mf| {
            mf.get_metric()
                .iter()
                .find(|metric| {
                    metric.get_label().iter().any(|label_pair| {
                        label_pair.get_name() == label_name && label_pair.get_value() == label_value
                    })
                })
                .map(|mf| mf.get_counter().get_value() as u64)
        })
        .next()
        .unwrap_or(0)
}

fn counts(metric_id: crate::metrics::MetricId) -> fnv::FnvHashMap<ReqRepId, u64> {
    crate::metrics::registry()
        .gather_for_desc_names(&[metric_id.name().as_str()])
        .first()
        .map(|mf| {
            let label_name = REQREPID_LABEL_ID.name();
            let label_name = label_name.as_str();
            let metrics = mf.get_metric();
            let counts = fnv::FnvHashMap::with_capacity_and_hasher(
                metrics.len(),
                fnv::FnvBuildHasher::default(),
            );
            metrics.iter().fold(counts, |mut counts, metric| {
                let reqrep_id = metric
                    .get_label()
                    .iter()
                    .find_map(|label_pair| {
                        if label_pair.get_name() == label_name {
                            Some(ReqRepId::from(
                                label_pair.get_value().parse::<ULID>().unwrap(),
                            ))
                        } else {
                            None
                        }
                    })
                    .unwrap();
                let counter = metric.get_counter();
                counts.insert(reqrep_id, counter.get_value() as u64);
                counts
            })
        })
        .unwrap_or_else(fnv::FnvHashMap::default)
}

/// returns the histogram timer metric corresponding to the ReqRepId
pub fn histogram_timer_metric(reqrep_id: ReqRepId) -> Option<prometheus::proto::Histogram> {
    let reqrep_id = reqrep_id.to_string();
    let reqrep_id = reqrep_id.as_str();
    let histogram: Vec<_> = crate::metrics::registry()
        .gather_for_metric_ids(&[REQREP_PROCESS_TIMER_METRIC_ID])
        .iter()
        .filter_map(|mf| {
            let metric = &mf.get_metric().iter().next().unwrap();
            if metric
                .get_label()
                .iter()
                .any(|label_pair| label_pair.get_value() == reqrep_id)
            {
                Some(metric.get_histogram().clone())
            } else {
                None
            }
        })
        .collect();
    histogram.first().cloned()
}
