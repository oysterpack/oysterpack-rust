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

});

#[derive(Clone, Default)]
pub struct World;
