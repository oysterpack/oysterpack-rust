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

use maplit::*;
use oysterpack_trust::metrics;
use oysterpack_uid::ULID;
use prometheus::{
    core::{Collector, Desc},
    proto::MetricFamily,
    Counter, CounterVec, Gauge, GaugeVec, Histogram, HistogramVec, IntCounter, IntCounterVec,
    IntGauge, IntGaugeVec,
};
use std::{
    collections::{hash_map::DefaultHasher, HashMap, HashSet},
    hash::BuildHasherDefault,
    sync::Arc,
    time::Duration,
    thread
};

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
        let builder = metrics::HistogramBuilder::new_timer(metrics::MetricId::generate(), "help", metrics::TimerBuckets::from(vec![Duration::from_millis(50)]));
        let metric = builder.build().unwrap();
        println!("{:#?}", metric.desc());
        metrics::registry().register(metric).unwrap();
    };

    // Scenario: [01D4G04E4XCY5SFC0XAYSMH9G6] Construct a new histogram timer vec and register it
    then regex "01D4G04E4XCY5SFC0XAYSMH9G6" | _world, _matches, _step| {
        let buckets = metrics::TimerBuckets::from(vec![Duration::from_millis(50)]);
        let builder = metrics::HistogramVecBuilder::new_timer(metrics::MetricId::generate(), "help",buckets,vec![metrics::LabelId::generate()]);
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
        metrics::registry().text_encode_metrics(&mut world.text_encoded_metrics);
    };

    then regex "01D3M9ZJQSTWFFMKBR3Z2DXJ9N" | world, _matches, _step| {
        let metrics_text = String::from_utf8_lossy(&world.text_encoded_metrics);
        println!("{}",metrics_text);
        //TODO: verify that the output format can be ingested via prometheus - use regex
    };

    // Feature: [01D3XX3ZBB7VW0GGRA60PMFC1M] Functions are provided to help collecting timer based metrics

    // Scenario: [01D3XX46RZ63QYR0AAWVBCHWGP] Timing a function that sleeps for 1 ms
    then regex "01D3XX46RZ63QYR0AAWVBCHWGP" | _world, _matches, _step| {
        let clock = quanta::Clock::new();
        let time_nanos = metrics::time(&clock, || thread::sleep(Duration::from_millis(1)));
        let nano_in_millis = 1_000_000;
        assert!(time_nanos > nano_in_millis && time_nanos < (time_nanos as f64 * 1.1) as u64);
        let time_secs = metrics::as_float_secs(time_nanos);
        println!("{} s",time_secs);
        assert!(time_secs > 0.001 && time_secs < 0.001 * 1.1);
    };

    // Scenario: [01D3XZ6GCY1ECSKMBC6870ZBS0] Timing a function that sleeps for 1 ms and returns a result
    then regex "01D3XZ6GCY1ECSKMBC6870ZBS0" | _world, _matches, _step| {
        let clock = quanta::Clock::new();
        let (time_nanos, result) = metrics::time_with_result(&clock, || {
            thread::sleep(Duration::from_millis(1));
            true
        });
        assert!(result);
        let nano_in_millis = 1_000_000;
        assert!(time_nanos > nano_in_millis && time_nanos < (time_nanos as f64 * 1.1) as u64);
        let time_secs = metrics::as_float_secs(time_nanos);
        println!("{} s",time_secs);
        assert!(time_secs > 0.001 && time_secs < 0.001 * 1.1);
    };
});

#[derive(Clone, Default)]
pub struct World {
    text_encoded_metrics: Vec<u8>
}
