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
use oysterpack_log::*;
use std::{thread, time::Duration};

const METRIC_ID_1: MetricId = MetricId(1871943882688894749067493983019708136);

#[test]
fn metric_registry_int_gauge() {
    configure_logging();

    use oysterpack_uid::ULID;

    let metric_id = MetricId::generate();
    let registry = MetricRegistry::default();
    registry
        .register_int_gauge(metric_id, "Active Sessions".to_string(), None)
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

    use oysterpack_uid::ULID;

    let metric_id = MetricId::generate();
    let registry = MetricRegistry::default();
    registry
        .register_gauge(metric_id, "Active Sessions".to_string(), None)
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

    use oysterpack_uid::ULID;

    let metric_id = MetricId::generate();
    let registry = MetricRegistry::default();
    let label = LabelId::generate();
    let labels = vec![label];
    registry
        .register_gauge_vec(metric_id, "A Gauge Vector".to_string(), &labels, None)
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

    use oysterpack_uid::ULID;

    let metric_id = MetricId::generate();
    let registry = MetricRegistry::default();
    let label = LabelId::generate();
    let labels = vec![label];
    registry
        .register_int_gauge_vec(metric_id, "A Gauge Vector".to_string(), &labels, None)
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

    use oysterpack_uid::ULID;

    let metric_id = MetricId::generate();
    let registry = MetricRegistry::default();
    registry
        .register_int_counter(metric_id, "ReqRep timer".to_string(), None)
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
fn metric_registry_int_counter_vec() {
    configure_logging();

    use oysterpack_uid::ULID;

    let metric_id = MetricId::generate();
    let registry = MetricRegistry::default();
    let label = LabelId::generate();
    let labels = vec![label];
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
}

#[test]
fn metric_registry_histogram_vec() {
    configure_logging();

    use oysterpack_uid::ULID;

    const METRIC_ID: MetricId = MetricId(1872045779718506837202123142606941790);
    let registry = MetricRegistry::default();
    registry
        .register_histogram_vec(
            METRIC_ID,
            "ReqRep timer".to_string(),
            &[LabelId::generate()],
            vec![0.01, 0.025, 0.05, 0.005, 0.0050, 0.005], // will be sorted and deduped automatically
            None,
        )
        .unwrap();

    info!("{:#?}", registry);

    let mut reqrep_timer_local = registry.histogram_vec(&METRIC_ID).unwrap().local();
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
            metric_id,
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
            metric_id,
            "ReqRep timer".to_string(),
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
        " ".to_string(),
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
    let int_counter_vec_with_const_labels_metric_id = MetricId::generate();
    let int_counter_vec_metric_id = MetricId::generate();

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
                .register_int_counter(int_counter_metric_id, "IntCounter".to_string(), None)
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
                    "IntCounterVec with const labels".to_string(),
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
                    "IntCounterVec with no const labels".to_string(),
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
                .register_gauge(gauge_metric_id, "Gauge".to_string(), None)
                .unwrap();

            for _ in 0..5 {
                metric.inc();
            }
        }
        {
            let metric = registry
                .register_int_gauge(int_gauge_metric_id, "IntGauge".to_string(), None)
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
                    "GaugeVec with const labels".to_string(),
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
                    "GaugeVec with no const labels".to_string(),
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
                    "IntGaugeVec with const labels".to_string(),
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
                    "IntgaugeVec with no const labels".to_string(),
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
                    "Histogram with no const labels".to_string(),
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
                    "HistogramVec with const labels".to_string(),
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
                    "HistogramVec with no const labels".to_string(),
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
                    "HistogramVec with const labels".to_string(),
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

    let metrics = registry.gather();
    info!("{:#?}", metrics);

    let metrics = registry.gather_metrics(&metric_ids);
    info!("{}", serde_json::to_string_pretty(&metrics).unwrap());
    assert_eq!(metrics.metrics().len(), metric_ids.len());

    for metric_id in metric_ids {
        assert!(metrics.metric(metric_id).is_some());
    }

    let all_metrics = registry.gather_all_metrics();
    assert_eq!(metrics.metrics().len(), all_metrics.metrics.len());

    // verify that metrics are reporting as expected
    match metrics.metric(int_counter_metric_id).unwrap() {
        Metric::IntCounter { desc, value } => {
            assert_eq!(*value, 5);
            assert_eq!(desc.help, "IntCounter");
        }
        metric => panic!("Wrong metric type has been returned: {:?}", metric),
    }
    match metrics
        .metric(int_counter_vec_with_const_labels_metric_id)
        .unwrap()
    {
        Metric::IntCounterVec { desc, values } => {
            assert_eq!(values.len(), 1);
            assert_eq!(values[0].value, 5);
            assert_eq!(values[0].labels[0].1, "FOO".to_string());
            assert_eq!(desc.help(), "IntCounterVec with const labels");
        }
        metric => panic!("Wrong metric type has been returned: {:?}", metric),
    }

    //    let int_counter_vec_with_const_labels_metric_id = MetricId::generate();
    //    let int_counter_vec_metric_id = MetricId::generate();
    //
    //    let gauge_metric_id = MetricId::generate();
    //    let int_gauge_metric_id = MetricId::generate();
    //    let gauge_vec_with_const_labels_metric_id = MetricId::generate();
    //    let gauge_vec_metric_id = MetricId::generate();
    //    let int_gauge_vec_with_const_labels_metric_id = MetricId::generate();
    //    let int_gauge_vec_metric_id = MetricId::generate();
    //
    //    let histogram_metric_id = MetricId::generate();
    //    let histogram_with_const_labels_metric_id = MetricId::generate();
    //    let histogram_vec_metric_id = MetricId::generate();
    //    let histogram_vec_with_const_labels_metric_id = MetricId::generate();
}

#[test]
fn gather_process_metrics() {
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
fn metric_descs() {
    configure_logging();

    let metric_registry = MetricRegistry::default();
    let descs = metric_registry.metric_descs();
    info!("empty MetricRegistry: {:#?}", descs);
    info!("{}", serde_json::to_string_pretty(&descs).unwrap());
    assert!(descs.int_counters().is_none());
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

    let metric_id = MetricId::generate();
    let metric = metric_registry
        .register_int_counter(
            metric_id,
            "int_counter".to_string(),
            Some(const_labels.clone()),
        )
        .unwrap();
    let metric_desc = MetricDesc::from(&metric);
    assert_eq!(metric_desc.id(), metric_id);
    assert_eq!(metric_desc.help(), "int_counter");
    assert_eq!(
        metric_desc.const_labels().unwrap().iter().fold(
            HashMap::new(),
            |mut map, (label_id, value)| {
                map.insert(*label_id, value.clone());
                map
            }
        ),
        const_labels
    );

    let metric_id = MetricId::generate();
    let metric = metric_registry
        .register_int_counter_vec(
            metric_id,
            "int_counter_vec".to_string(),
            &labels,
            Some(const_labels.clone()),
        )
        .unwrap();
    let metric_desc = MetricVecDesc::from(&metric);
    assert_eq!(metric_desc.id(), metric_id);
    assert_eq!(metric_desc.help(), "int_counter_vec");
    assert_eq!(
        metric_desc.const_labels().unwrap().iter().fold(
            HashMap::new(),
            |mut map, (label_id, value)| {
                map.insert(*label_id, value.clone());
                map
            }
        ),
        const_labels
    );
    assert_eq!(
        metric_desc
            .labels()
            .iter()
            .cloned()
            .collect::<HashSet<LabelId>>(),
        labels.iter().cloned().collect::<HashSet<LabelId>>()
    );

    let metric_id = MetricId::generate();
    let metric = metric_registry
        .register_gauge(metric_id, "gauge".to_string(), Some(const_labels.clone()))
        .unwrap();
    let metric_desc = MetricDesc::from(&metric);
    assert_eq!(metric_desc.id(), metric_id);
    assert_eq!(metric_desc.help(), "gauge");
    assert_eq!(
        metric_desc.const_labels().unwrap().iter().fold(
            HashMap::new(),
            |mut map, (label_id, value)| {
                map.insert(*label_id, value.clone());
                map
            }
        ),
        const_labels
    );

    let metric_id = MetricId::generate();
    let metric = metric_registry
        .register_int_gauge(
            metric_id,
            "int_gauge".to_string(),
            Some(const_labels.clone()),
        )
        .unwrap();
    let metric_desc = MetricDesc::from(&metric);
    assert_eq!(metric_desc.id(), metric_id);
    assert_eq!(metric_desc.help(), "int_gauge");
    assert_eq!(
        metric_desc.const_labels().unwrap().iter().fold(
            HashMap::new(),
            |mut map, (label_id, value)| {
                map.insert(*label_id, value.clone());
                map
            }
        ),
        const_labels
    );

    let metric_id = MetricId::generate();
    let metric = metric_registry
        .register_gauge_vec(
            metric_id,
            "gauge_vec".to_string(),
            &labels,
            Some(const_labels.clone()),
        )
        .unwrap();
    let metric_desc = MetricVecDesc::from(&metric);
    assert_eq!(metric_desc.id(), metric_id);
    assert_eq!(metric_desc.help(), "gauge_vec");
    assert_eq!(
        metric_desc.const_labels().unwrap().iter().fold(
            HashMap::new(),
            |mut map, (label_id, value)| {
                map.insert(*label_id, value.clone());
                map
            }
        ),
        const_labels
    );
    assert_eq!(
        metric_desc
            .labels()
            .iter()
            .cloned()
            .collect::<HashSet<LabelId>>(),
        labels.iter().cloned().collect::<HashSet<LabelId>>()
    );

    let metric_id = MetricId::generate();
    let metric = metric_registry
        .register_int_gauge_vec(
            metric_id,
            "int_gauge_vec".to_string(),
            &labels,
            Some(const_labels.clone()),
        )
        .unwrap();
    let metric_desc = MetricVecDesc::from(&metric);
    assert_eq!(metric_desc.id(), metric_id);
    assert_eq!(metric_desc.help(), "int_gauge_vec");
    assert_eq!(
        metric_desc.const_labels().unwrap().iter().fold(
            HashMap::new(),
            |mut map, (label_id, value)| {
                map.insert(*label_id, value.clone());
                map
            }
        ),
        const_labels
    );
    assert_eq!(
        metric_desc
            .labels()
            .iter()
            .cloned()
            .collect::<HashSet<LabelId>>(),
        labels.iter().cloned().collect::<HashSet<LabelId>>()
    );

    let buckets = vec![0.0, 1.0, 5.0, 10.0];
    let metric_id = MetricId::generate();
    let metric = metric_registry
        .register_histogram(
            metric_id,
            "histogram".to_string(),
            buckets.clone(),
            Some(const_labels.clone()),
        )
        .unwrap();
    let metric_desc = HistogramDesc::from((&metric, Buckets::from(buckets.as_slice())));
    assert_eq!(metric_desc.id(), metric_id);
    assert_eq!(metric_desc.help(), "histogram");
    assert_eq!(
        metric_desc.const_labels().unwrap().iter().fold(
            HashMap::new(),
            |mut map, (label_id, value)| {
                map.insert(*label_id, value.clone());
                map
            }
        ),
        const_labels
    );
    assert_eq!(
        metric_desc
            .buckets()
            .0
            .iter()
            .map(|value| *value as u64)
            .collect::<HashSet<u64>>(),
        buckets
            .as_slice()
            .iter()
            .map(|value| *value as u64)
            .collect::<HashSet<u64>>()
    );

    let metric_id = MetricId::generate();
    let metric = metric_registry
        .register_histogram_vec(
            metric_id,
            "histogram_vec".to_string(),
            &labels,
            buckets.clone(),
            Some(const_labels.clone()),
        )
        .unwrap();
    let metric_desc = HistogramVecDesc::from((&metric, Buckets::from(buckets.as_slice())));
    assert_eq!(metric_desc.id(), metric_id);
    assert_eq!(metric_desc.help(), "histogram_vec");
    assert_eq!(
        metric_desc.const_labels().unwrap().iter().fold(
            HashMap::new(),
            |mut map, (label_id, value)| {
                map.insert(*label_id, value.clone());
                map
            }
        ),
        const_labels
    );
    assert_eq!(
        metric_desc
            .labels()
            .iter()
            .cloned()
            .collect::<HashSet<LabelId>>(),
        labels.iter().cloned().collect::<HashSet<LabelId>>()
    );

    let descs = metric_registry.metric_descs();
    info!("{:#?}", descs);

    let descs_json = serde_json::to_string_pretty(&descs).unwrap();
    // verify that MetricDescs are serde compatible
    info!("{}", descs_json);
    assert!(descs.int_counters().is_some());
    assert!(descs.int_counter_vecs().is_some());
    assert!(descs.gauges().is_some());
    assert!(descs.int_gauges().is_some());
    assert!(descs.gauge_vecs().is_some());
    assert!(descs.int_gauge_vecs().is_some());
    assert!(descs.histograms().is_some());
    assert!(descs.histogram_vecs().is_some());

    let descs1: MetricDescs = serde_json::from_str(descs_json.as_str()).unwrap();

    assert!(descs1.int_counters().is_some());
    assert!(descs1.int_counter_vecs().is_some());
    assert!(descs1.gauges().is_some());
    assert!(descs1.int_gauges().is_some());
    assert!(descs1.gauge_vecs().is_some());
    assert!(descs1.int_gauge_vecs().is_some());
    assert!(descs1.histograms().is_some());
    assert!(descs1.histogram_vecs().is_some());

    let bytes = bincode::serialize(&descs).unwrap();
    let descs2: MetricDescs = bincode::deserialize(&bytes).unwrap();

    assert!(descs2.int_counters().is_some());
    assert!(descs2.int_counter_vecs().is_some());
    assert!(descs2.gauges().is_some());
    assert!(descs2.int_gauges().is_some());
    assert!(descs2.gauge_vecs().is_some());
    assert!(descs2.int_gauge_vecs().is_some());
    assert!(descs2.histograms().is_some());
    assert!(descs2.histogram_vecs().is_some());
}
