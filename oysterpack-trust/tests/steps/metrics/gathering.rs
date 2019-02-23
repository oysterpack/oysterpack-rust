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
use std::{collections::HashSet, sync::Arc};

steps!(World => {
    // Feature: [01D43V3KAZ276MQZY1TZG793EQ] Gathering all metrics

    // Scenario: [01D3PPPT1ZNXPKKWM29R14V5ZT] Gathering all metrics
    given regex "01D3PPPT1ZNXPKKWM29R14V5ZT" | world, _matches, step | {
       world.register_metrics()
    };

    when regex "01D3PPPT1ZNXPKKWM29R14V5ZT" | world, _matches, step| {
        world.metric_families = metrics::registry().gather();
    };

    then regex "01D3PPPT1ZNXPKKWM29R14V5ZT" | world, _matches, step| {
        world.check_all_metrics_returned();
    };

    // Feature: [01D43V3KAZ276MQZY1TZG793EQ] Gathering a subset of the metrics

    // Background:
    //    Given [01D3J441N6BM05NKCBQEVYTZY8] metrics are registered
    given regex "01D3J441N6BM05NKCBQEVYTZY8" | world, _matches, step | {
       world.register_metrics();
    };

    // Scenario: [01D3PPY3E710BYY8DQDKVQ31KY] Gather metrics for DescId(s)
    when regex "01D3PPY3E710BYY8DQDKVQ31KY" | world, _matches, step| {
        world.desc_ids = vec![
            world.counter.desc()[0].id,
            world.int_counter.desc()[0].id,
            world.counter_vec.desc()[0].id,
            world.int_counter_vec.desc()[0].id,
        ];
        metrics::registry().gather();
        world.metric_families = metrics::registry().gather_for_desc_ids(&world.desc_ids);
    };

    then regex "01D3PPY3E710BYY8DQDKVQ31KY" | world, _matches, step| {
        let mut desc_ids: HashSet<_> = world.desc_ids.iter().cloned().collect();
        assert_eq!(world.metric_families.len(), desc_ids.len());
    };

});

#[derive(Clone)]
pub struct World {
    counter: Counter,
    gauge: Gauge,
    int_counter: IntCounter,
    int_gauge: IntGauge,

    counter_vec: CounterVec,
    gauge_vec: GaugeVec,
    int_counter_vec: IntCounterVec,
    int_gauge_vec: IntGaugeVec,

    histogram: Histogram,
    histogram_vec: HistogramVec,

    world2: Option<Arc<World>>,
    metric_families: Vec<prometheus::proto::MetricFamily>,
    desc_ids: Vec<metrics::DescId>,
}

impl World {
    fn metric_ids(&self) -> Vec<metrics::MetricId> {
        vec![
            self.int_counter.desc()[0].fq_name.as_str().parse().unwrap(),
            self.int_gauge.desc()[0].fq_name.as_str().parse().unwrap(),
        ]
    }

    fn desc_ids(&self) -> HashSet<metrics::DescId> {
        vec![self.int_counter.desc(), self.int_gauge.desc()]
            .iter()
            .flat_map(|descs| descs.iter().map(|desc| desc.id))
            .collect()
    }

    fn register_metrics(&mut self) {
        metrics::registry().register(self.clone()).unwrap();
        self.world2 = Some(Arc::new(World::default()));

        for world2 in self.world2.as_ref() {
            metrics::registry()
                .register(world2.counter.clone())
                .unwrap();
            metrics::registry()
                .register(world2.int_counter.clone())
                .unwrap();
            metrics::registry().register(world2.gauge.clone()).unwrap();
            metrics::registry()
                .register(world2.int_gauge.clone())
                .unwrap();
            metrics::registry()
                .register(world2.histogram.clone())
                .unwrap();

            // records metrics as a side effect for the vectors, so that they can be gathered
            world2.collect();
            metrics::registry()
                .register(world2.gauge_vec.clone())
                .unwrap();
            metrics::registry()
                .register(world2.int_gauge_vec.clone())
                .unwrap();
            metrics::registry()
                .register(world2.counter_vec.clone())
                .unwrap();
            metrics::registry()
                .register(world2.int_counter_vec.clone())
                .unwrap();
            metrics::registry()
                .register(world2.histogram_vec.clone())
                .unwrap();
        }
    }

    fn check_all_metrics_returned(&self) {
        let expected_metric_family_names: HashSet<String> = metrics::registry()
            .descs()
            .iter()
            .map(|desc| desc.fq_name.clone())
            .collect();
        assert_eq!(
            expected_metric_family_names.len(),
            self.metric_families.len()
        );
    }
}

/// Each World instance contains unique metrics, i.e., unique metric descriptors because of unique MetricId
impl Collector for World {
    fn desc(&self) -> Vec<&Desc> {
        self.int_counter
            .desc()
            .iter()
            .cloned()
            .chain(self.int_gauge.desc().iter().cloned())
            .chain(self.gauge.desc().iter().cloned())
            .chain(self.counter.desc().iter().cloned())
            .chain(self.int_gauge_vec.desc().iter().cloned())
            .chain(self.gauge_vec.desc().iter().cloned())
            .chain(self.counter_vec.desc().iter().cloned())
            .chain(self.int_counter_vec.desc().iter().cloned())
            .chain(self.histogram.desc().iter().cloned())
            .chain(self.histogram_vec.desc().iter().cloned())
            .collect()
    }

    fn collect(&self) -> Vec<MetricFamily> {
        // recording metrics for vectors in order for them to be gathered
        let value = ULID::generate().to_string();
        let label_values = vec![value.as_str()];
        self.counter.inc();
        self.int_counter.inc();
        self.gauge.inc();
        self.int_gauge.inc();
        self.histogram.observe(0.25);
        self.counter_vec.with_label_values(&label_values).inc();
        self.int_counter_vec.with_label_values(&label_values).inc();
        self.gauge_vec.with_label_values(&label_values).inc();
        self.int_gauge_vec.with_label_values(&label_values).inc();
        self.histogram_vec
            .with_label_values(&label_values)
            .observe(0.15);

        self.int_counter
            .collect()
            .iter()
            .cloned()
            .chain(self.int_gauge.collect().iter().cloned())
            .chain(self.gauge.collect().iter().cloned())
            .chain(self.counter.collect().iter().cloned())
            .chain(self.int_gauge_vec.collect().iter().cloned())
            .chain(self.gauge_vec.collect().iter().cloned())
            .chain(self.counter_vec.collect().iter().cloned())
            .chain(self.int_counter_vec.collect().iter().cloned())
            .chain(self.histogram.collect().iter().cloned())
            .chain(self.histogram_vec.collect().iter().cloned())
            .collect()
    }
}

impl Default for World {
    fn default() -> World {
        Self {
            counter: metrics::new_counter(
                metrics::MetricId::generate(),
                "counter",
                Some(hashmap! {
                    metrics::LabelId::generate() => "A".to_string()
                }),
            )
            .unwrap(),
            int_counter: metrics::new_int_counter(
                metrics::MetricId::generate(),
                "int counter",
                Some(hashmap! {
                    metrics::LabelId::generate() => "A".to_string()
                }),
            )
            .unwrap(),
            gauge: metrics::new_gauge(
                metrics::MetricId::generate(),
                "int gauge",
                Some(hashmap! {
                    metrics::LabelId::generate() => "B".to_string()
                }),
            )
            .unwrap(),
            int_gauge: metrics::new_int_gauge(
                metrics::MetricId::generate(),
                "gauge",
                Some(hashmap! {
                    metrics::LabelId::generate() => "B".to_string()
                }),
            )
            .unwrap(),

            counter_vec: metrics::new_counter_vec(
                metrics::MetricId::generate(),
                "counter vec",
                &[metrics::LabelId::generate()],
                Some(hashmap! {
                    metrics::LabelId::generate() => "A".to_string()
                }),
            )
            .unwrap(),
            int_counter_vec: metrics::new_int_counter_vec(
                metrics::MetricId::generate(),
                "int counter vec",
                &[metrics::LabelId::generate()],
                Some(hashmap! {
                    metrics::LabelId::generate() => "A".to_string()
                }),
            )
            .unwrap(),
            gauge_vec: metrics::new_gauge_vec(
                metrics::MetricId::generate(),
                "int gauge vec",
                &[metrics::LabelId::generate()],
                Some(hashmap! {
                    metrics::LabelId::generate() => "B".to_string()
                }),
            )
            .unwrap(),
            int_gauge_vec: metrics::new_int_gauge_vec(
                metrics::MetricId::generate(),
                "gauge vec",
                &[metrics::LabelId::generate()],
                Some(hashmap! {
                    metrics::LabelId::generate() => "B".to_string()
                }),
            )
            .unwrap(),

            histogram: metrics::new_histogram(
                metrics::MetricId::generate(),
                "histogram",
                vec![0.1, 0.2],
                Some(hashmap! {
                    metrics::LabelId::generate() => "B".to_string()
                }),
            )
            .unwrap(),
            histogram_vec: metrics::new_histogram_vec(
                metrics::MetricId::generate(),
                "histogram vec",
                &[metrics::LabelId::generate()],
                vec![0.1, 0.2],
                Some(hashmap! {
                    metrics::LabelId::generate() => "B".to_string()
                }),
            )
            .unwrap(),

            world2: None,
            metric_families: Vec::new(),
            desc_ids: Vec::new(),
        }
    }
}
