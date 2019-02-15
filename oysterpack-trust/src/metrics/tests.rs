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

//! metrics tests

use super::*;
use crate::configure_logging;
use maplit::*;
use oysterpack_log::*;
use std::{collections::HashSet, thread, time::Duration};

const METRIC_ID_1: MetricId = MetricId(1871943882688894749067493983019708136);

#[test]
fn metric_registry_int_gauge() {
    configure_logging();

    use oysterpack_uid::ULID;

    let metric_id = MetricId::generate();
    let registry = MetricRegistry::default();
    let mut gauge = registry
        .register_int_gauge(metric_id, "Active Sessions", None)
        .unwrap();

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

    use oysterpack_uid::ULID;

    let metric_id = MetricId::generate();
    let registry = MetricRegistry::default();
    let mut gauge = registry
        .register_gauge(metric_id, "Active Sessions", None)
        .unwrap();

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

    use oysterpack_uid::ULID;

    let metric_id = MetricId::generate();
    let registry = MetricRegistry::default();
    let label = LabelId::generate();
    let labels = vec![label];
    let mut gauge_vec = registry
        .register_gauge_vec(metric_id, "A Gauge Vector", &labels, None)
        .unwrap();

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

    use oysterpack_uid::ULID;

    let metric_id = MetricId::generate();
    let registry = MetricRegistry::default();
    let label = LabelId::generate();
    let labels = vec![label];
    let mut gauge_vec = registry
        .register_int_gauge_vec(metric_id, "A Gauge Vector", &labels, None)
        .unwrap();

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

    use oysterpack_uid::ULID;

    let metric_id = MetricId::generate();
    let registry = MetricRegistry::default();
    let counter = registry
        .register_int_counter(metric_id, "ReqRep timer", None)
        .unwrap();

    let mut counter = counter.local();
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

    use oysterpack_uid::ULID;

    let metric_id = MetricId::generate();
    let registry = MetricRegistry::default();
    let label = LabelId::generate();
    let labels = vec![label];
    let mut counter_vec = registry
        .register_int_counter_vec(metric_id, "ReqRep timer", &labels, None)
        .unwrap();

    info!("{:#?}", registry);

    let mut counter = counter_vec.with_label_values(&["ABC"]).local();
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

    use oysterpack_uid::ULID;

    const METRIC_ID: MetricId = MetricId(1872045779718506837202123142606941790);
    let registry = MetricRegistry::default();
    let mut reqrep_timer_local = registry
        .register_histogram_vec(
            METRIC_ID,
            "ReqRep timer",
            &[LabelId::generate()],
            vec![0.01, 0.025, 0.05, 0.005, 0.0050, 0.005], // will be sorted and deduped automatically
            None,
        )
        .unwrap();

    info!("{:#?}", registry);

    let reqrep_timer =
        reqrep_timer_local.with_label_values(&[ULID::generate().to_string().as_str()]);
    let clock = quanta::Clock::new();
    for _ in 0..10 {
        let ulid_u128: u128 = ULID::generate().into();
        let sleep_ms = (ulid_u128 % 100) as u64;
        info!("sleeping for {}", sleep_ms);
        let delta = time(&clock, || {
            std::thread::sleep(std::time::Duration::from_millis(sleep_ms))
        });
        reqrep_timer.observe(as_float_secs(delta));
    }
}

#[test]
fn metric_registry_histogram() {
    configure_logging();

    use oysterpack_uid::ULID;

    let metric_id = MetricId::generate();
    let registry = MetricRegistry::default();
    let mut reqrep_timer = registry
        .register_histogram(
            metric_id,
            "ReqRep timer",
            vec![0.01, 0.025, 0.05, 0.005, 0.0050, 0.005], // will be sorted and deduped automatically
            None,
        )
        .unwrap()
        .local();

    info!("{:#?}", registry);

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
    let mut reqrep_timer = registry
        .register_histogram(
            metric_id,
            "ReqRep timer",
            vec![0.01, 0.025, 0.05, 0.005, 0.0050, 0.005], // will be sorted and deduped automatically
            None,
        )
        .unwrap();

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
    let mut reqrep_timer_local = registry
        .register_histogram_vec(
            metric_id,
            "ReqRep timer",
            &[LabelId::generate()],
            vec![0.01, 0.025, 0.05, 0.005, 0.0050, 0.005], // will be sorted and deduped automatically
            Some(const_labels),
        )
        .unwrap()
        .local();

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
            metric_id,
            "ReqRep timer",
            &[LabelId::generate()],
            vec![0.01, 0.025, 0.05, 0.005, 0.0050, 0.005], // will be sorted and deduped automatically
            Some(const_labels),
        );
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
        metric_id,
        " ",
        &[LabelId::generate()],
        vec![0.01, 0.025, 0.05, 0.005, 0.0050, 0.005], // will be sorted and deduped automatically
        None,
    );
    assert!(result.is_err());
}

#[test]
fn global_metric_registry() {
    configure_logging();

    let metrics = METRIC_REGISTRY.gather();
    info!("{:#?}", metrics);
}

#[test]
fn metric_registry_gather() {
    configure_logging();

    let registry = &METRIC_REGISTRY;

    let int_counter_metric_id = MetricId::generate();
    let counter_metric_id = MetricId::generate();
    let int_counter_vec_with_const_labels_metric_id = MetricId::generate();
    let int_counter_vec_metric_id = MetricId::generate();
    let counter_vec_metric_id = MetricId::generate();

    let gauge_metric_id = MetricId::generate();
    let int_gauge_metric_id = MetricId::generate();
    let gauge_vec_with_const_labels_metric_id = MetricId::generate();
    let gauge_vec_metric_id = MetricId::generate();
    let int_gauge_vec_with_const_labels_metric_id = MetricId::generate();
    let int_gauge_vec_metric_id = MetricId::generate();

    let histogram_metric_id = MetricId::generate();
    let histogram_with_const_labels_metric_id = MetricId::generate();
    let histogram_vec_metric_id = MetricId::generate();
    let histogram_vec_with_const_labels_metric_id = MetricId::generate();

    let metric_ids = vec![
        counter_metric_id,
        counter_vec_metric_id,
        int_counter_metric_id,
        int_counter_vec_with_const_labels_metric_id,
        int_counter_vec_metric_id,
        gauge_metric_id,
        int_gauge_metric_id,
        gauge_vec_with_const_labels_metric_id,
        gauge_vec_metric_id,
        int_gauge_vec_with_const_labels_metric_id,
        int_gauge_vec_metric_id,
        histogram_metric_id,
        histogram_with_const_labels_metric_id,
        histogram_vec_metric_id,
        histogram_vec_with_const_labels_metric_id,
    ];

    let registry = MetricRegistry::default();

    let mut register_counters = || {
        {
            let metric = registry
                .register_int_counter(int_counter_metric_id, "IntCounter", None)
                .unwrap();

            for _ in 0..5 {
                metric.inc();
            }
        }
        {
            let metric = registry
                .register_counter(counter_metric_id, "Counter", None)
                .unwrap();

            for _ in 0..5 {
                metric.inc();
            }
        }
        {
            let mut const_labels = HashMap::new();
            const_labels.insert(LabelId::generate(), "BAR".to_string());
            let mut metric = registry
                .register_int_counter_vec(
                    int_counter_vec_with_const_labels_metric_id,
                    "IntCounterVec with const labels",
                    &[LabelId::generate()],
                    Some(const_labels),
                )
                .unwrap()
                .local();

            let counter = metric.with_label_values(&["FOO"]);

            for _ in 0..5 {
                counter.inc();
                counter.flush();
            }
        }
        {
            let mut metric = registry
                .register_int_counter_vec(
                    int_counter_vec_metric_id,
                    "IntCounterVec with no const labels",
                    &[LabelId::generate()],
                    None,
                )
                .unwrap()
                .local();

            let counter = metric.with_label_values(&["FOO"]);

            for _ in 0..5 {
                counter.inc();
                counter.flush();
            }
        }
        {
            let mut metric = registry
                .register_int_counter_vec(
                    counter_vec_metric_id,
                    "CounterVec with no const labels",
                    &[LabelId::generate()],
                    None,
                )
                .unwrap()
                .local();

            let counter = metric.with_label_values(&["FOO"]);

            for _ in 0..5 {
                counter.inc();
                counter.flush();
            }
        }
    };
    register_counters();

    let mut register_gauges = || {
        {
            let metric = registry
                .register_gauge(gauge_metric_id, "Gauge", None)
                .unwrap();

            for _ in 0..5 {
                metric.inc();
            }
        }
        {
            let metric = registry
                .register_int_gauge(int_gauge_metric_id, "IntGauge", None)
                .unwrap();

            for _ in 0..5 {
                metric.inc();
            }
        }
        {
            let mut const_labels = HashMap::new();
            const_labels.insert(LabelId::generate(), "BAR".to_string());
            let mut metric = registry
                .register_gauge_vec(
                    gauge_vec_with_const_labels_metric_id,
                    "GaugeVec with const labels",
                    &[LabelId::generate()],
                    Some(const_labels),
                )
                .unwrap();

            let counter = metric.with_label_values(&["FOO"]);

            for _ in 0..5 {
                counter.inc();
            }
        }
        {
            let mut metric = registry
                .register_gauge_vec(
                    gauge_vec_metric_id,
                    "GaugeVec with no const labels",
                    &[LabelId::generate()],
                    None,
                )
                .unwrap();

            let counter = metric.with_label_values(&["FOO"]);

            for _ in 0..5 {
                counter.inc();
            }
        }
        {
            let mut const_labels = HashMap::new();
            const_labels.insert(LabelId::generate(), "BAR".to_string());
            let mut metric = registry
                .register_int_gauge_vec(
                    int_gauge_vec_with_const_labels_metric_id,
                    "IntGaugeVec with const labels",
                    &[LabelId::generate()],
                    Some(const_labels),
                )
                .unwrap();

            let counter = metric.with_label_values(&["FOO"]);

            for _ in 0..5 {
                counter.inc();
            }
        }
        {
            let mut metric = registry
                .register_int_gauge_vec(
                    int_gauge_vec_metric_id,
                    "IntgaugeVec with no const labels",
                    &[LabelId::generate()],
                    None,
                )
                .unwrap();

            let counter = metric.with_label_values(&["FOO"]);

            for _ in 0..5 {
                counter.inc();
            }
        }
    };
    register_gauges();

    let mut register_histograms = || {
        {
            let metric = registry
                .register_histogram(
                    histogram_metric_id,
                    "Histogram with no const labels",
                    vec![0.001, 0.0025, 0.005], // will be sorted and deduped automatically
                    None,
                )
                .unwrap();

            const METRIC_COUNT: u64 = 5;
            for _ in 0..METRIC_COUNT {
                let ulid_u128: u128 = ULID::generate().into();
                let sleep_ms = (ulid_u128 % 5) as u32;
                info!("sleeping for {}", sleep_ms);
                {
                    let timer = metric.start_timer();
                    thread::sleep_ms(sleep_ms)
                }
            }
        }
        {
            let mut const_labels = HashMap::new();
            let label = LabelId::generate();
            const_labels.insert(label, "BAR - CONST LABEL".to_string());
            let metric = registry
                .register_histogram_vec(
                    histogram_with_const_labels_metric_id,
                    "HistogramVec with const labels",
                    &[LabelId::generate()],
                    vec![0.01, 0.025, 0.05, 0.005, 0.0050, 0.005], // will be sorted and deduped automatically
                    Some(const_labels),
                )
                .unwrap();

            let mut metric = metric.local();
            let metric = metric.with_label_values(&["FOO - VAR LABEL"]);
            let clock = quanta::Clock::new();
            const METRIC_COUNT: usize = 5;
            for _ in 0..METRIC_COUNT {
                let ulid_u128: u128 = ULID::generate().into();
                let sleep_ms = (ulid_u128 % 5) as u32;
                info!("sleeping for {}", sleep_ms);
                let delta = time(&clock, || thread::sleep_ms(sleep_ms));
                metric.observe(as_float_secs(delta));
                metric.flush();
            }
        }
        {
            let labels = vec![LabelId::generate()];
            let metric = registry
                .register_histogram_vec(
                    histogram_vec_metric_id,
                    "HistogramVec with no const labels",
                    &labels,
                    vec![0.001, 0.0025, 0.005], // will be sorted and deduped automatically
                    None,
                )
                .unwrap();

            let mut metric = metric.local();
            let metric = metric.with_label_values(&["BITCOIN"]);
            const METRIC_COUNT: u64 = 5;
            for _ in 0..METRIC_COUNT {
                let ulid_u128: u128 = ULID::generate().into();
                let sleep_ms = (ulid_u128 % 5) as u32;
                info!("sleeping for {}", sleep_ms);
                {
                    let timer = metric.start_timer();
                    thread::sleep_ms(sleep_ms)
                }
            }
            metric.flush()
        }
        {
            let mut const_labels = HashMap::new();
            let label = LabelId::generate();
            const_labels.insert(label, "BAR - CONST LABEL".to_string());
            let labels = vec![LabelId::generate()];
            let metric = registry
                .register_histogram_vec(
                    histogram_vec_with_const_labels_metric_id,
                    "HistogramVec with const labels",
                    &labels,
                    vec![0.001, 0.0025, 0.005], // will be sorted and deduped automatically
                    Some(const_labels),
                )
                .unwrap();

            let mut metric = metric.local();
            let metric = metric.with_label_values(&["BITCOIN"]);
            const METRIC_COUNT: u64 = 5;
            for _ in 0..METRIC_COUNT {
                let ulid_u128: u128 = ULID::generate().into();
                let sleep_ms = (ulid_u128 % 5) as u32;
                info!("sleeping for {}", sleep_ms);
                {
                    let timer = metric.start_timer();
                    thread::sleep_ms(sleep_ms)
                }
            }
            metric.flush()
        }
    };
    register_histograms();

    // adding 1 because the ProcessCollector is automatically registered
    assert_eq!(metric_ids.len() + 1, registry.collector_count());

    let metrics = registry.gather();
    info!("{:#?}", metrics);
    assert_eq!(metrics.len(), registry.metric_family_count());
    let descs = registry.descs();
    assert_eq!(descs.len(), metrics.len());
}

#[test]
fn registry_gather_process_metrics() {
    configure_logging();

    let metric_registry = MetricRegistry::default();
    let process_metrics = metric_registry.gather_process_metrics();
    info!("{:#?}", process_metrics);
    assert!(process_metrics.max_fds() > 0.0);
    assert!(process_metrics.open_fds() > 0.0);
    assert!(process_metrics.virtual_memory_bytes() > 0.0);
    assert!(process_metrics.resident_memory_bytes() > 0.0);
    assert!(process_metrics.start_time_seconds() > 0.0);
}

#[test]
fn registry_gather_metrics() {
    configure_logging();
    let metric_registry = MetricRegistry::default();

    // Given a metric is registered with a random MetricId
    let timer = metric_registry
        .register_histogram_vec(
            MetricId::generate(),
            "ReqRep processor timer",
            &[LabelId::generate()],
            vec![0.0, 1.0, 5.0],
            Some({
                let mut labels = HashMap::new();
                labels.insert(LabelId::generate(), "A".to_string());
                labels
            }),
        )
        .unwrap();
    timer.with_label_values(&["1"]).observe(0.2);

    // And 2 metrics are registered with the same MetricId and LabelId, but different const label values
    let metric_id = MetricId::generate();
    let label_id = LabelId::generate();
    let var_label_id = LabelId::generate();
    for i in 0..2 {
        let timer = metric_registry
            .register_histogram_vec(
                metric_id,
                "ReqRep processor timer",
                &[var_label_id],
                vec![0.0, 1.0, 5.0],
                Some(hashmap! {
                    label_id => format!("{}",i)
                }),
            )
            .unwrap();
        // And metrics are observed across 2 dimensions
        timer.with_label_values(&["1"]).observe(1.2);
        timer.with_label_values(&["2"]).observe(2.2);
    }

    let descs = metric_registry.descs();
    info!("descs: {:#?}", descs);
    // When metrics are gathered
    let mfs = metric_registry.gather();
    info!("{:#?}", mfs);
    // Then when gathering metrics for each Desc, only 1 MetricFamily will be returned because a Metric
    // can only be part of 1 MetricFamily
    for ref desc in descs.iter() {
        let mfs = metric_registry.gather_metrics(&[desc.id]);
        info!("{}: {:#?}", desc.fq_name, mfs);
        assert_eq!(mfs.len(), 1);
    }

    // And each MetricFamily for the above 2 MetricVec(s) will have 2 metrics
    let desc = descs
        .iter()
        .find(|desc| desc.fq_name == metric_id.name())
        .unwrap();
    assert_eq!(
        mfs.iter()
            .find(|mf| mf.get_name() == desc.fq_name.as_str())
            .unwrap()
            .get_metric()
            .len(),
        4
    );
    // When metrics are gathered for each desc
    let mfs = metric_registry.gather_metrics(&[desc.id]);
    for mf in mfs {
        // Then the returned MetricFamily will only contain the metric for that Desc
        assert_eq!(mf.get_metric().len(), 1);
    }
}

#[test]
fn registry_gather_metrics_by_name() {
    configure_logging();
    let metric_registry = MetricRegistry::default();

    let timer = metric_registry
        .register_histogram_vec(
            MetricId::generate(),
            "ReqRep processor timer",
            &[LabelId::generate()],
            vec![0.0, 1.0, 5.0],
            Some({
                let mut labels = HashMap::new();
                labels.insert(LabelId::generate(), "A".to_string());
                labels
            }),
        )
        .unwrap();
    timer.with_label_values(&["1"]).observe(0.2);

    // register 2 HistogramVec metrics with the same MetricId but different constant label values
    let histogram_vec_metric_id = {
        let metric_id = MetricId::generate();
        let label_id = LabelId::generate();
        let var_label_id = LabelId::generate();
        let timer = metric_registry
            .register_histogram_vec(
                metric_id,
                "ReqRep processor timer",
                &[var_label_id],
                vec![0.0, 1.0, 5.0],
                Some({
                    let mut labels = HashMap::new();
                    labels.insert(label_id, "B".to_string());
                    labels
                }),
            )
            .unwrap();
        timer.with_label_values(&["1"]).observe(1.2);
        timer.with_label_values(&["2"]).observe(2.2);

        let timer = metric_registry
            .register_histogram_vec(
                metric_id,
                "ReqRep processor timer",
                &[var_label_id],
                vec![0.0, 1.0, 5.0],
                Some({
                    let mut labels = HashMap::new();
                    labels.insert(label_id, "C".to_string());
                    labels
                }),
            )
            .unwrap();
        timer.with_label_values(&["3"]).observe(1.2);
        timer.with_label_values(&["4"]).observe(2.2);

        metric_id
    };

    // register 2 Histogram metrics with the same MetricId but different constant label values
    let histogram_metric_id = {
        let metric_id = MetricId::generate();
        let label_id = LabelId::generate();
        let timer = metric_registry
            .register_histogram(
                metric_id,
                "ReqRep processor timer",
                vec![0.0, 1.0, 5.0],
                Some({
                    let mut labels = HashMap::new();
                    labels.insert(label_id, "D".to_string());
                    labels
                }),
            )
            .unwrap();
        timer.observe(11.21);
        timer.observe(12.21);

        let timer = metric_registry
            .register_histogram(
                metric_id,
                "ReqRep processor timer",
                vec![0.0, 1.0, 5.0],
                Some({
                    let mut labels = HashMap::new();
                    labels.insert(label_id, "E".to_string());
                    labels
                }),
            )
            .unwrap();
        timer.observe(11.2);
        timer.observe(12.2);
        metric_id
    };

    let descs = metric_registry.descs();
    info!("descs: {:#?}", descs);
    let mfs = metric_registry.gather();
    info!("{:#?}", mfs);
    for ref desc in descs.iter() {
        let mfs = metric_registry.gather_metrics_by_name(&[desc.fq_name.as_str()]);
        info!("{}: {:#?}", desc.fq_name, mfs);
    }

    let desc = descs
        .iter()
        .find(|desc| desc.fq_name == histogram_vec_metric_id.name())
        .unwrap();
    // there should be 2 metric families, each with 2 metrics
    assert_eq!(
        mfs.iter()
            .filter(|mf| mf.get_name() == desc.fq_name.as_str())
            .flat_map(|mfs| mfs.get_metric())
            .count(),
        4
    );
    let mfs = metric_registry.gather_metrics_by_name(&[histogram_vec_metric_id.name().as_str()]);
    assert_eq!(mfs.len(), 2);
    for mf in mfs {
        assert_eq!(mf.get_metric().len(), 2);
    }

    let collectors = metric_registry.filter_collectors(|collector| {
        collector
            .desc()
            .iter()
            .any(move |desc| desc.fq_name == histogram_vec_metric_id.name())
    });
    assert_eq!(collectors.len(), 2);

    let collectors = metric_registry.filter_collectors(|collector| {
        collector
            .desc()
            .iter()
            .any(move |desc| desc.fq_name == histogram_metric_id.name())
    });
    assert_eq!(collectors.len(), 2);
}
