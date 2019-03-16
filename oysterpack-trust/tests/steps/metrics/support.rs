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

use cucumber_rust::*;

use oysterpack_trust::metrics::{self, timer_buckets};
use prometheus::core::Collector;
use std::{num::NonZeroUsize, thread, time::Duration};

steps!(World => {
    // Feature: [01D3VG4CEEPF8NNBM348PKRDH3] Metric builders are provided for each of the supported metrics.

    //  Scenario: [01D3VGSGCP9ZN9BX3BTB349FRJ] Construct a new counter and register it
    then regex "01D3VGSGCP9ZN9BX3BTB349FRJ" | _world, _matches, _step | {
        let builder = metrics::CounterBuilder::new(metrics::MetricId::generate(), "help");
        let metric = builder.build().unwrap();
        println!("{:#?}", metric.desc());
        metrics::registry().register(metric).unwrap();

        let label_id = metrics::LabelId::generate();
        let builder = metrics::CounterBuilder::new(metrics::MetricId::generate(), "help")
            .with_label(label_id, "A");
        let metric = builder.build().unwrap();
        println!("{:#?}", metric.desc());
        let label_pair = &metric.desc()[0].const_label_pairs[0];
        assert_eq!(label_id.name().as_str(), label_pair.get_name());
        assert_eq!("A", label_pair.get_value());
        metrics::registry().register(metric).unwrap();
    };

    // Scenario: [01D4G02JDYSR3PBY1MTFZQNJ46] Construct a new int counter and register it
    then regex "01D4G02JDYSR3PBY1MTFZQNJ46" | _world, _matches, _step| {
        let builder = metrics::IntCounterBuilder::new(metrics::MetricId::generate(), "help");
        let metric = builder.build().unwrap();
        println!("{:#?}", metric.desc());
        metrics::registry().register(metric).unwrap();

        let label_id = metrics::LabelId::generate();
        let builder = metrics::IntCounterBuilder::new(metrics::MetricId::generate(), "help")
            .with_label(label_id, "A");
        let metric = builder.build().unwrap();
        println!("{:#?}", metric.desc());
        let label_pair = &metric.desc()[0].const_label_pairs[0];
        assert_eq!(label_id.name().as_str(), label_pair.get_name());
        assert_eq!("A", label_pair.get_value());
        metrics::registry().register(metric).unwrap();
    };

    // Scenario: [01D4G02N1YCHM8N9DYED2P8SRV] Construct a new counter vec and register it
    then regex "01D4G02N1YCHM8N9DYED2P8SRV" | _world, _matches, _step| {
        let builder = metrics::CounterVecBuilder::new(metrics::MetricId::generate(), "help", vec![metrics::LabelId::generate()]);
        let metric = builder.build().unwrap();
        println!("{:#?}", metric.desc());
        metrics::registry().register(metric).unwrap();
    };

    // Scenario: [01D4G02NNQBW5NC2B5R6QPC38Z] Construct a new int counter vec and register it
    then regex "01D4G02NNQBW5NC2B5R6QPC38Z" | _world, _matches, _step| {
        let builder = metrics::IntCounterVecBuilder::new(metrics::MetricId::generate(), "help", vec![metrics::LabelId::generate()]);
        let metric = builder.build().unwrap();
        println!("{:#?}", metric.desc());
        metrics::registry().register(metric).unwrap();
    };

    // Scenario: [01D4G02P8P3ZPSDHJ058479441] Construct a new gauge and register it
    then regex "01D4G02P8P3ZPSDHJ058479441" | _world, _matches, _step | {
        let builder = metrics::GaugeBuilder::new(metrics::MetricId::generate(), "help");
        let metric = builder.build().unwrap();
        println!("{:#?}", metric.desc());
        metrics::registry().register(metric).unwrap();

        let label_id = metrics::LabelId::generate();
        let builder = metrics::GaugeBuilder::new(metrics::MetricId::generate(), "help")
            .with_label(label_id, "A");
        let metric = builder.build().unwrap();
        println!("{:#?}", metric.desc());
        let label_pair = &metric.desc()[0].const_label_pairs[0];
        assert_eq!(label_id.name().as_str(), label_pair.get_name());
        assert_eq!("A", label_pair.get_value());
        metrics::registry().register(metric).unwrap();
    };

    // Scenario: [01D4G02PV54CR9MDHYNYP7G69M] Construct a new int gauge and register it
    then regex "01D4G02PV54CR9MDHYNYP7G69M" | _world, _matches, _step | {
        let builder = metrics::IntGaugeBuilder::new(metrics::MetricId::generate(), "help");
        let metric = builder.build().unwrap();
        println!("{:#?}", metric.desc());
        metrics::registry().register(metric).unwrap();

        let label_id = metrics::LabelId::generate();
        let builder = metrics::IntGaugeBuilder::new(metrics::MetricId::generate(), "help")
            .with_label(label_id, "A");
        let metric = builder.build().unwrap();
        println!("{:#?}", metric.desc());
        let label_pair = &metric.desc()[0].const_label_pairs[0];
        assert_eq!(label_id.name().as_str(), label_pair.get_name());
        assert_eq!("A", label_pair.get_value());
        metrics::registry().register(metric).unwrap();
    };

    // Scenario: [01D4G02QC5A2J0CF6TG0863N1J] Construct a new gauge vec and register it
    then regex "01D4G02QC5A2J0CF6TG0863N1J" | _world, _matches, _step| {
        let builder = metrics::GaugeVecBuilder::new(metrics::MetricId::generate(), "help", vec![metrics::LabelId::generate()]);
        let metric = builder.build().unwrap();
        println!("{:#?}", metric.desc());
        metrics::registry().register(metric).unwrap();
    };

    // Scenario: [01D4G02QYHTQ6N5EP4XADW67ZG] Construct a new int gauge vec and register it
    then regex "01D4G02QYHTQ6N5EP4XADW67ZG" | _world, _matches, _step| {
        let builder = metrics::IntGaugeVecBuilder::new(metrics::MetricId::generate(), "help", vec![metrics::LabelId::generate()]);
        let metric = builder.build().unwrap();
        println!("{:#?}", metric.desc());
        metrics::registry().register(metric).unwrap();
    };

    // Scenario: [01D4G02RFAQCQMM3C7WH7VZECG] Construct a new histogram and register it
    then regex "01D4G02RFAQCQMM3C7WH7VZECG" | _world, _matches, _step| {
        let builder = metrics::HistogramBuilder::new(metrics::MetricId::generate(), "help", vec![0.1,0.2]);
        let metric = builder.build().unwrap();
        println!("{:#?}", metric.desc());
        metrics::registry().register(metric).unwrap();
    };

    // Scenario: [01D4G04MZ2VXN226H8R2CRASE5] Construct a new histogram vec and register it
    then regex "01D4G04MZ2VXN226H8R2CRASE5" | _world, _matches, _step| {
        let builder = metrics::HistogramVecBuilder::new(metrics::MetricId::generate(), "help",vec![0.1,0.2],vec![metrics::LabelId::generate()]);
        let metric = builder.build().unwrap();
        println!("{:#?}", metric.desc());
        metrics::registry().register(metric).unwrap();
    };

    // Scenario: [01D4G04V16ZSWAKBMADJ5M2ZS9] Construct a new histogram timer and register it
    then regex "01D4G04V16ZSWAKBMADJ5M2ZS9" | _world, _matches, _step| {
        let builder = metrics::HistogramBuilder::new(metrics::MetricId::generate(), "help", timer_buckets(vec![Duration::from_millis(50)]).unwrap());
        let metric = builder.build().unwrap();
        println!("{:#?}", metric.desc());
        metrics::registry().register(metric).unwrap();
    };

    // Scenario: [01D4G04E4XCY5SFC0XAYSMH9G6] Construct a new histogram timer vec and register it
    then regex "01D4G04E4XCY5SFC0XAYSMH9G6" | _world, _matches, _step| {
        let buckets = timer_buckets(vec![Duration::from_millis(50)]).unwrap();
        let builder = metrics::HistogramVecBuilder::new(metrics::MetricId::generate(), "help",buckets,vec![metrics::LabelId::generate()]);
        let metric = builder.build().unwrap();
        println!("{:#?}", metric.desc());
        metrics::registry().register(metric).unwrap();
    };

    // Feature: [01D43V2S6HBV642EKK5YGJNH32] MetricId can be used as the metric name.

    // Scenario: [01D3PB6MDJ85MWP3SQ1H94S6R7] Define MetricId as a constant
    then regex "01D3PB6MDJ85MWP3SQ1H94S6R7" | _world, _matches, _step| {
        const METRIC_ID: metrics::MetricId = metrics::MetricId(1875087812780658887130798484160706713);
        assert_eq!(METRIC_ID.to_string(), "M01D4GFF0KYKX79T919MTG4NY4S".to_string());
        assert_eq!(METRIC_ID.to_string(), METRIC_ID.name());
        assert_eq!(METRIC_ID.to_string().as_str().parse::<metrics::MetricId>().unwrap(), METRIC_ID);
    };

    // Scenario: [01D4GEXWKASWC6MHRZVSEHJG5G] Register a metric using a MetricId
    then regex "01D4GEXWKASWC6MHRZVSEHJG5G" | _world, _matches, _step| {
        const METRIC_ID: metrics::MetricId = metrics::MetricId(1875087812780658887130798484160706713);
        let metric = metrics::IntCounterBuilder::new(METRIC_ID,"help").build().unwrap();
        metrics::registry().register(metric).unwrap();
        let descs = metrics::registry().descs_for_metric_id(METRIC_ID);
        assert_eq!(descs.len(), 1);
        assert_eq!(descs.first().unwrap().fq_name.as_str().parse::<metrics::MetricId>().unwrap(),METRIC_ID);
    };

    // Feature: [01D43V2S6HBV642EKK5YGJNH32] LabelId can be used for constant and variable label names.

    // Scenario: [01D4GFESB7GQY04JGR0CQ5S6TW] Define LabelId as a constant
    then regex "01D4GFESB7GQY04JGR0CQ5S6TW" | _world, _matches, _step| {
        const LABEL_ID: metrics::LabelId = metrics::LabelId(1875087812780658887130798484160706713);
        assert_eq!(LABEL_ID.to_string(), "L01D4GFF0KYKX79T919MTG4NY4S".to_string());
        assert_eq!(LABEL_ID.to_string(), LABEL_ID.name());
        assert_eq!(LABEL_ID.to_string().as_str().parse::<metrics::LabelId>().unwrap(), LABEL_ID);
    };

    // Scenario: [01D4GFF0KYKX79T919MTG4NY4S] Register a metric with constant label pairs using LabelId as the label name
    then regex "01D4GFF0KYKX79T919MTG4NY4S" | _world, _matches, _step| {
        const METRIC_ID: metrics::MetricId = metrics::MetricId(1875094762743445343460054874084371924);
        const CONST_LABEL_ID: metrics::LabelId = metrics::LabelId(1875089559860207622555416072996156405,);
        const VAR_LABEL_ID: metrics::LabelId = metrics::LabelId(1875089568089933988084098457787782225,);
        let metric = metrics::IntCounterVecBuilder::new(METRIC_ID,"help", vec![VAR_LABEL_ID])
            .with_label(CONST_LABEL_ID, "A")
            .build()
            .unwrap();
        metrics::registry().register(metric).unwrap();
        let descs = metrics::registry().descs_for_metric_id(METRIC_ID);
        assert_eq!(descs.len(), 1);
        let desc = descs.first().unwrap();
        assert_eq!(desc.fq_name.as_str().parse::<metrics::MetricId>().unwrap(),METRIC_ID);
        assert!(desc.const_label_pairs.iter().any(|label_pair| label_pair.get_name() == CONST_LABEL_ID.name().as_str()));
        assert!(desc.variable_labels.iter().any(|label| label == &VAR_LABEL_ID.name()));
    };

    // Feature: [01D3M9X86BSYWW3132JQHWA3AT] Gathered metrics can be encoded in prometheus compatible text format

    // Scenario: [01D3M9ZJQSTWFFMKBR3Z2DXJ9N] gathering metrics
    given regex "01D3M9ZJQSTWFFMKBR3Z2DXJ9N" | _world, _matches, _step| {
        let registry = metrics::registry();
        use metrics::*;

        let counter = CounterBuilder::new(MetricId::generate(),"help").with_label(LabelId::generate(), "A").build().unwrap();
        counter.inc();
        registry.register(counter).unwrap();

        let counter = IntCounterBuilder::new(MetricId::generate(),"help").with_label(LabelId::generate(), "A").build().unwrap();
        counter.inc();
        registry.register(counter).unwrap();

        let counter_vec = IntCounterVecBuilder::new(MetricId::generate(),"help", vec![LabelId::generate()]).with_label(LabelId::generate(), "A").build().unwrap();
        let counter = counter_vec.with_label_values(&["1"]);
        counter.inc();
        registry.register(counter_vec).unwrap();

        let gauge = GaugeBuilder::new(MetricId::generate(),"help").with_label(LabelId::generate(), "A").build().unwrap();
        gauge.inc();
        registry.register(gauge).unwrap();

        let gauge = IntGaugeBuilder::new(MetricId::generate(),"help").with_label(LabelId::generate(), "A").build().unwrap();
        gauge.inc();
        registry.register(gauge).unwrap();

        let gauge_vec = IntGaugeVecBuilder::new(MetricId::generate(),"help", vec![LabelId::generate()]).with_label(LabelId::generate(), "A").build().unwrap();
        let gauge = gauge_vec.with_label_values(&["1"]);
        gauge.inc();
        registry.register(gauge_vec).unwrap();
    };

    when regex "01D3M9ZJQSTWFFMKBR3Z2DXJ9N" | world, _matches, _step| {
        metrics::registry().text_encode_metrics(&mut world.text_encoded_metrics).unwrap();
    };

    then regex "01D3M9ZJQSTWFFMKBR3Z2DXJ9N" | world, _matches, _step| {
        let metrics_text = String::from_utf8_lossy(&world.text_encoded_metrics);
        println!("{}",metrics_text);
    };

    // Feature: [01D3XX3ZBB7VW0GGRA60PMFC1M] Time conversion functions to report timings in seconds as f64

    // Scenario: [01D3XX46RZ63QYR0AAWVBCHWGP] Convert 1_000_000 ns into a sec
    then regex "01D3XX46RZ63QYR0AAWVBCHWGP" | _world, _matches, _step| {
         let secs = metrics::nanos_as_secs_f64(1_000_000);
         println!("0.001 sec = {}", secs);
         assert!(secs >= 0.001 && secs < 0.0011);
    };

    // Scenario: [01D3XZ6GCY1ECSKMBC6870ZBS0] Convert a Duration into secs
    then regex "01D3XZ6GCY1ECSKMBC6870ZBS0" | _world, _matches, _step| {
        let secs = metrics::duration_as_secs_f64(Duration::from_millis(1));
        assert!(secs >= 0.001 && secs < 0.0011);
    };

    // Feature: [01D63TZ7T07W3K8K6QTR1CN9HH] Creating histogram timer buckets based on time durations

    // Scenario: [01D63V3G7Q3S9F1JV4A3TJYJQH] metrics::timer_buckets()
    then regex "01D63V3G7Q3S9F1JV4A3TJYJQH" | _world, _matches, _step| {
        let buckets = metrics::timer_buckets(vec![Duration::from_millis(1)]).unwrap();
        let buckets = metrics::timer_buckets(vec![Duration::from_millis(1), Duration::from_millis(10)]).unwrap();
        let buckets = metrics::timer_buckets(vec![
            Duration::from_millis(1),
            Duration::from_millis(10),
            Duration::from_millis(10),
        ])
        .unwrap();
        let buckets = metrics::timer_buckets(vec![Duration::from_millis(10), Duration::from_millis(1)]).unwrap();
    };

    // Scenario: [01D63V8E55T03C161QTGHP0THK] metrics::exponential_timer_buckets()
    then regex "01D63V8E55T03C161QTGHP0THK" | _world, _matches, _step| {
        let buckets = metrics::exponential_timer_buckets(Duration::from_millis(1), 2.0, NonZeroUsize::new(10).unwrap()).unwrap();
        println!("buckets = {:?}", buckets);
        assert_eq!(buckets.len(), 10);
        let expected_buckets = vec![
            0.001, 0.002, 0.004, 0.008, 0.016, 0.032, 0.064, 0.128, 0.256, 0.512,
        ];
        use float_cmp::ApproxEq;
        buckets
            .iter()
            .zip(expected_buckets)
            .for_each(|(left, right)| assert!(right.approx_eq(left, std::f64::EPSILON, 2)));
    };

    // Scenario: [01D63V9Z9J1HC5NBGM64JJMXXZ] metrics::linear_timer_buckets()
    then regex "01D63V9Z9J1HC5NBGM64JJMXXZ" | _world, _matches, _step| {
        let buckets =
            metrics::linear_timer_buckets(Duration::from_millis(10), Duration::from_millis(50), NonZeroUsize::new(10).unwrap())
                .unwrap();
        println!("buckets = {:?}", buckets);
        assert_eq!(buckets.len(), 10);
        let expected_buckets = vec![0.01, 0.06, 0.11, 0.16, 0.21, 0.26, 0.31, 0.36, 0.41, 0.46];
        use float_cmp::ApproxEq;
        buckets
            .iter()
            .zip(expected_buckets)
            .for_each(|(left, right)| assert!(right.approx_eq(left, std::f64::EPSILON, 2)));
    };

    // Rule: the starting bucket upper bound must be greater than 0 ns

    // Scenario: [01D63VEPJZMH40CKH872E1CB8X] metrics::timer_buckets(): start = Duration::from_millis(0)
    then regex "01D63VEPJZMH40CKH872E1CB8X" | _world, _matches, _step| {
        let result =
            metrics::timer_buckets(vec![Duration::from_millis(0), Duration::from_millis(1)]);
        println!(
            "timer_buckets(vec![Duration::from_millis(0), Duration::from_millis(1)])-> {:?}",
            result
        );
        assert!(result.is_err());
    };

    // Scenario: [01D63VEX6RSDMCQ8P83WEX0ND6] metrics::exponential_timer_buckets(): start = Duration::from_millis(0)
    then regex "01D63VEX6RSDMCQ8P83WEX0ND6" | _world, _matches, _step| {
        let result = metrics::exponential_timer_buckets(Duration::from_millis(0), 2.0, NonZeroUsize::new(10).unwrap());
        println!(
            "exponential_timer_buckets(Duration::from_millis(0), 2.0, 10) -> {:?}",
            result
        );
        assert!(result.is_err());
    };

    // Scenario: [01D63VF3AZHA0SH3KYEDZC1W4P] metrics::linear_timer_buckets(): start = Duration::from_millis(0)
    then regex "01D63VF3AZHA0SH3KYEDZC1W4P" | _world, _matches, _step| {
        let result =
            metrics::linear_timer_buckets(Duration::from_millis(0), Duration::from_millis(50), NonZeroUsize::new(10).unwrap());
        println!(
            "linear_timer_buckets(Duration::from_millis(0), Duration::from_millis(50), 10) -> {:?}",
            result
        );
        assert!(result.is_err());
    };

    // Rule: at least 1 bucket must be specified

    // Scenario: [01D63W11BGS89YFSZRK7A4JHP7] metrics::timer_buckets() with empty durations
    then regex "01D63W11BGS89YFSZRK7A4JHP7" | _world, _matches, _step| {
        let result = metrics::timer_buckets(vec![]);
        println!("timer_bucketsvec![])-> {:?}", result);
        assert!(result.is_err());
    };

    // Rule: tmetrics::exponential_timer_buckets(): factor must be > 1

    // Scenario: [01D63W0QCYEB6P6Z4YP4ZZ75C9] metrics::exponential_timer_buckets(): factor = 1.0
    then regex "01D63W0QCYEB6P6Z4YP4ZZ75C9" | _world, _matches, _step| {
        let result = metrics::exponential_timer_buckets(Duration::from_millis(1), 1.0, NonZeroUsize::new(10).unwrap());
        println!(
            "exponential_timer_buckets(Duration::from_millis(1), 1.0, 10) -> {:?}",
            result
        );
        assert!(result.is_err());
    };

    // Rule: metrics::linear_timer_buckets(): width must be > 0 ns

    // Scenario: [01D63W389MA1H1HQJ45Y7GPXM5] metrics::linear_timer_buckets(): width = Duration::from_millis(0)
    then regex "01D63W389MA1H1HQJ45Y7GPXM5" | _world, _matches, _step| {
        let result =
            metrics::linear_timer_buckets(Duration::from_millis(10), Duration::from_millis(0), NonZeroUsize::new(10).unwrap());
        println!(
            "linear_timer_buckets(Duration::from_millis(10), Duration::from_millis(0), 10) -> {:?}",
            result
        );
        assert!(result.is_err());
    };

});

#[derive(Clone, Default)]
pub struct World {
    text_encoded_metrics: Vec<u8>,
}
