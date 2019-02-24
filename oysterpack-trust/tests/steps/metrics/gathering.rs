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
use std::collections::HashMap;
use std::{collections::HashSet, sync::Arc};

steps!(World => {
    // Feature: [01D43V3KAZ276MQZY1TZG793EQ] Gathering all metrics

    // Scenario: [01D3PPPT1ZNXPKKWM29R14V5ZT] Gathering all metrics
    given regex "01D3PPPT1ZNXPKKWM29R14V5ZT" | world, _matches, _step | {
       world.register_metrics()
    };

    when regex "01D3PPPT1ZNXPKKWM29R14V5ZT" | world, _matches, _step| {
        world.metric_families = metrics::registry().gather();
    };

    then regex "01D3PPPT1ZNXPKKWM29R14V5ZT" | world, _matches, _step| {
        world.check_all_metrics_returned();
    };

    // Feature: [01D43V3KAZ276MQZY1TZG793EQ] Gathering a subset of the metrics

    // Background:
    //    Given [01D3J441N6BM05NKCBQEVYTZY8] metrics are registered
    given regex "01D3J441N6BM05NKCBQEVYTZY8" | world, _matches, _step | {
       world.register_metrics();
    };

    // Scenario: [01D3PPY3E710BYY8DQDKVQ31KY] Gather metrics for DescId(s)
    when regex "01D3PPY3E710BYY8DQDKVQ31KY" | world, _matches, _step| {
        world.desc_ids = vec![
            world.counter.desc()[0].id,
            world.int_counter.desc()[0].id,
            world.counter_vec.desc()[0].id,
            world.int_counter_vec.desc()[0].id,
        ];
        metrics::registry().gather();
        world.metric_families = metrics::registry().gather_for_desc_ids(&world.desc_ids);
    };

    then regex "01D3PPY3E710BYY8DQDKVQ31KY" | world, _matches, _step| {
        let mut desc_ids: HashSet<_> = world.desc_ids.iter().cloned().collect();
        assert_eq!(world.metric_families.len(), desc_ids.len());
    };

    // Scenario: [01D4BXN2ZMYRHNGRRCSTKVN0AP] Gather metrics for DescId(s) containing dups
    when regex "01D4BXN2ZMYRHNGRRCSTKVN0AP" | world, _matches, _step| {
        world.desc_ids = vec![
            world.counter.desc()[0].id,
            world.int_counter.desc()[0].id,
            world.counter_vec.desc()[0].id,
            world.int_counter_vec.desc()[0].id,
        ];
        let mut desc_ids = world.desc_ids.clone();
        desc_ids.extend(world.desc_ids.clone());
        metrics::registry().gather();
        world.metric_families = metrics::registry().gather_for_desc_ids(&desc_ids);
    };

    then regex "01D4BXN2ZMYRHNGRRCSTKVN0AP" | world, _matches, _step| {
        let mut desc_ids: HashSet<_> = world.desc_ids.iter().cloned().collect();
        assert_eq!(world.metric_families.len(), desc_ids.len());
    };

    // Scenario: [01D4D0GEXXQ3WKK78DYC0RJHKD] Gather metrics for DescId(s) containing some that do not match
    when regex "01D4D0GEXXQ3WKK78DYC0RJHKD" | world, _matches, _step| {
        let mut desc_ids = vec![
            world.counter.desc()[0].id,
            world.int_counter.desc()[0].id,
            world.counter_vec.desc()[0].id,
            world.int_counter_vec.desc()[0].id,
        ];
        world.desc_ids = desc_ids.clone();
        let non_existent_desc_id = find_next_non_existent_desc_id(0);
        desc_ids.push(non_existent_desc_id);
        desc_ids.push(find_next_non_existent_desc_id(non_existent_desc_id + 1));
        metrics::registry().gather();
        world.metric_families = metrics::registry().gather_for_desc_ids(&desc_ids);
    };

    then regex "01D4D0GEXXQ3WKK78DYC0RJHKD" | world, _matches, _step| {
        let mut desc_ids: HashSet<_> = world.desc_ids.iter().cloned().collect();
        assert_eq!(world.metric_families.len(), desc_ids.len());
    };

    // [01D4D1774JHQNB8X0QRBYEAEBW] Gather metrics for DescId(s) containing none that not match
    when regex "01D4D1774JHQNB8X0QRBYEAEBW" | world, _matches, _step| {
        let mut desc_ids = Vec::with_capacity(2);
        let non_existent_desc_id = find_next_non_existent_desc_id(0);
        desc_ids.push(non_existent_desc_id);
        desc_ids.push(find_next_non_existent_desc_id(non_existent_desc_id + 1));
        metrics::registry().gather();
        world.metric_families = metrics::registry().gather_for_desc_ids(&desc_ids);
    };

    then regex "01D4D1774JHQNB8X0QRBYEAEBW" | world, _matches, _step| {
        assert!(world.metric_families.is_empty());
    };

    // [01D4D1WEKJBFQSR0Z1Q10ZHD2R] Gather metrics for DescId(s) with an empty &[DescId]
    when regex "01D4D1WEKJBFQSR0Z1Q10ZHD2R" | world, _matches, _step| {
        let mut desc_ids = vec![];
        world.metric_families = metrics::registry().gather_for_desc_ids(&desc_ids);
    };

    then regex "01D4D1WEKJBFQSR0Z1Q10ZHD2R" | world, _matches, _step| {
        assert!(world.metric_families.is_empty());
    };

    // Scenario: [01D3PQ2KMBY07K48Q281SMPED6] Gather metrics for descriptor names
    when regex "01D3PQ2KMBY07K48Q281SMPED6" | world, _matches, _step| {
        world.desc_names = vec![
            world.counter.desc()[0].fq_name.clone(),
            world.int_counter.desc()[0].fq_name.clone(),
            world.counter_vec.desc()[0].fq_name.clone(),
            world.int_counter_vec.desc()[0].fq_name.clone(),
        ];
        metrics::registry().gather();
        world.metric_families = metrics::registry().gather_for_desc_names(&world.desc_names);
    };

    then regex "01D3PQ2KMBY07K48Q281SMPED6" | world, _matches, _step| {
        assert_eq!(world.metric_families.len(), world.desc_names.len());
    };

    // Scenario: [01D4BXX8A1SY3CYA8V9330F7QM] Gather metrics for descriptor names with dup names
    when regex "01D4BXX8A1SY3CYA8V9330F7QM" | world, _matches, _step| {
        world.desc_names = vec![
            world.counter.desc()[0].fq_name.clone(),
            world.int_counter.desc()[0].fq_name.clone(),
            world.counter_vec.desc()[0].fq_name.clone(),
            world.int_counter_vec.desc()[0].fq_name.clone(),
        ];
        let mut desc_names = world.desc_names.clone();
        desc_names.extend(world.desc_names.clone());
        metrics::registry().gather();
        world.metric_families = metrics::registry().gather_for_desc_names(&desc_names);
    };

    then regex "01D4BXX8A1SY3CYA8V9330F7QM" | world, _matches, _step| {
        assert_eq!(world.metric_families.len(), world.desc_names.len());
    };

    // Scenario: [01D4D2YZXEES3GHA30J5ZZFPGF] Gather metrics for descriptor names containing some that do not match
    when regex "01D4D2YZXEES3GHA30J5ZZFPGF" | world, _matches, _step| {
        world.desc_names = vec![
            world.counter.desc()[0].fq_name.clone(),
            world.int_counter.desc()[0].fq_name.clone(),
            world.counter_vec.desc()[0].fq_name.clone(),
            world.int_counter_vec.desc()[0].fq_name.clone(),
        ];
        let mut desc_names = world.desc_names.clone();
        desc_names.push(ULID::generate().to_string());
        metrics::registry().gather();
        world.metric_families = metrics::registry().gather_for_desc_names(&desc_names);
    };

    then regex "01D4D2YZXEES3GHA30J5ZZFPGF" | world, _matches, _step| {
        assert_eq!(world.metric_families.len(), world.desc_names.len());
    };

    // Scenario: [01D4D302NGKYAVCHDF4A1Z6SB3] Gather metrics for descriptor names containing none that match
    when regex "01D4D302NGKYAVCHDF4A1Z6SB3" | world, _matches, _step| {
        let mut desc_names = vec![ULID::generate().to_string(), ULID::generate().to_string()];
        metrics::registry().gather();
        world.metric_families = metrics::registry().gather_for_desc_names(&desc_names);
    };

    then regex "01D4D302NGKYAVCHDF4A1Z6SB3" | world, _matches, _step| {
        assert!(world.metric_families.is_empty());
    };

    // Scenario: [01D4D30ABTZ72781C5NDP42217] Gather metrics for descriptor names using an empty &[Name]
    when regex "01D4D30ABTZ72781C5NDP42217" | world, _matches, _step| {
        let mut desc_names = Vec::<String>::new();
        metrics::registry().gather();
        world.metric_families = metrics::registry().gather_for_desc_names(&desc_names);
    };

    then regex "01D4D30ABTZ72781C5NDP42217" | world, _matches, _step| {
        assert!(world.metric_families.is_empty());
    };

    // Scenario: [01D3VC85Q8MVBJ543SHZ4RE9T2] Gather metrics for MetricId(s)
    when regex "01D3VC85Q8MVBJ543SHZ4RE9T2" | world, _matches, _step| {
        world.desc_names = vec![
            world.counter.desc()[0].fq_name.clone(),
            world.int_counter.desc()[0].fq_name.clone(),
            world.counter_vec.desc()[0].fq_name.clone(),
            world.int_counter_vec.desc()[0].fq_name.clone(),
        ];
        let metric_ids: Vec<metrics::MetricId> = world.desc_names.iter().map(|name| name.as_str().parse().unwrap()).collect();
        metrics::registry().gather();
        world.metric_families = metrics::registry().gather_for_metric_ids(&metric_ids);
    };

    then regex "01D3VC85Q8MVBJ543SHZ4RE9T2" | world, _matches, _step| {
        let metric_families = metrics::registry().gather_for_desc_names(&world.desc_names);
        assert_eq!(world.metric_families.len(), metric_families.len());
    };

    // Scenario: [01D4D3C0EBPZX8NWCYRD8YJ0Y3] Gather metrics for MetricId(s) containing dups
    when regex "01D4D3C0EBPZX8NWCYRD8YJ0Y3" | world, _matches, _step| {
        world.desc_names = vec![
            world.counter.desc()[0].fq_name.clone(),
            world.int_counter.desc()[0].fq_name.clone(),
            world.counter_vec.desc()[0].fq_name.clone(),
            world.int_counter_vec.desc()[0].fq_name.clone(),
            // dups
            world.int_counter.desc()[0].fq_name.clone(),
            world.counter_vec.desc()[0].fq_name.clone(),
        ];
        let metric_ids: Vec<metrics::MetricId> = world.desc_names.iter().map(|name| name.as_str().parse().unwrap()).collect();
        metrics::registry().gather();
        world.metric_families = metrics::registry().gather_for_metric_ids(&metric_ids);
    };

    then regex "01D4D3C0EBPZX8NWCYRD8YJ0Y3" | world, _matches, _step| {
        let metric_families = metrics::registry().gather_for_desc_names(&world.desc_names);
        assert_eq!(world.metric_families.len(), metric_families.len());
    };

    // Scenario: [01D4D3EX9TP87RQ2S11PFNXG2T] Gather metrics for MetricId(s) containing some that do not match
    when regex "01D4D3EX9TP87RQ2S11PFNXG2T" | world, _matches, _step| {
        world.desc_names = vec![
            world.counter.desc()[0].fq_name.clone(),
            world.int_counter.desc()[0].fq_name.clone(),
            world.counter_vec.desc()[0].fq_name.clone(),
            world.int_counter_vec.desc()[0].fq_name.clone(),
        ];
        let mut metric_ids: Vec<metrics::MetricId> = world.desc_names.iter().map(|name| name.as_str().parse().unwrap()).collect();
        metric_ids.push(metrics::MetricId::generate());
        metrics::registry().gather();
        world.metric_families = metrics::registry().gather_for_metric_ids(&metric_ids);
    };

    then regex "01D4D3EX9TP87RQ2S11PFNXG2T" | world, _matches, _step| {
        let metric_families = metrics::registry().gather_for_desc_names(&world.desc_names);
        assert_eq!(world.metric_families.len(), metric_families.len());
    };

    // Scenario: [01D4D3EKJME2MCH81DXTAMGMJS] Gather metrics for MetricId(s) containing none that match
    when regex "01D4D3EKJME2MCH81DXTAMGMJS" | world, _matches, _step| {
        let metric_ids: Vec<metrics::MetricId> = vec![metrics::MetricId::generate()];
        metrics::registry().gather();
        world.metric_families = metrics::registry().gather_for_metric_ids(&metric_ids);
    };

    then regex "01D4D3EKJME2MCH81DXTAMGMJS" | world, _matches, _step| {
        assert!(world.metric_families.is_empty());
    };

    // Scenario: [01D4D3EBMA7XR2FWA1Q6E5F560] Gather metrics for MetricId(s) using an empty &[MetricId]
    when regex "01D4D3EBMA7XR2FWA1Q6E5F560" | world, _matches, _step| {
        let metric_ids: Vec<metrics::MetricId> = vec![];
        metrics::registry().gather();
        world.metric_families = metrics::registry().gather_for_metric_ids(&metric_ids);
    };

    then regex "01D4D3EBMA7XR2FWA1Q6E5F560" | world, _matches, _step| {
        assert!(world.metric_families.is_empty());
    };

    // Scenario: [01D43MQQ1H59ZGJ9G2AMEJB5RF] Gather metrics for labels
    when regex "01D43MQQ1H59ZGJ9G2AMEJB5RF" | world, _matches, _step| {
        world.desc_names = vec![
            world.counter.desc()[0].fq_name.clone(),
            world.int_counter.desc()[0].fq_name.clone(),
            world.counter_vec.desc()[0].fq_name.clone(),
            world.int_counter_vec.desc()[0].fq_name.clone(),
        ];
        for label_pair in &world.counter.desc()[0].const_label_pairs {
            world.labels.insert(label_pair.get_name().to_string(), label_pair.get_value().to_string());
        }
        for label_pair in &world.int_counter.desc()[0].const_label_pairs {
            world.labels.insert(label_pair.get_name().to_string(), label_pair.get_value().to_string());
        }
        for label_pair in &world.counter_vec.desc()[0].const_label_pairs {
            world.labels.insert(label_pair.get_name().to_string(), label_pair.get_value().to_string());
        }
        for label_pair in &world.int_counter_vec.desc()[0].const_label_pairs {
            world.labels.insert(label_pair.get_name().to_string(), label_pair.get_value().to_string());
        }
        metrics::registry().gather();
        world.metric_families = metrics::registry().gather_for_labels(&world.labels);
    };

    then regex "01D43MQQ1H59ZGJ9G2AMEJB5RF" | world, _matches, _step| {
        let metric_families = metrics::registry().gather_for_desc_names(&world.desc_names);
        assert_eq!(world.metric_families.len(), metric_families.len());
        assert!(world.metric_families.iter().all(|mf| metric_families.iter().any(|mf2| mf2.get_name() == mf.get_name())));
        assert!(world.metric_families.iter().all(|mf| {
            mf.get_metric().iter().any(|m| m.get_label().iter().any(|label|{
                match world.labels.get(label.get_name()) {
                    Some(value) => value.as_str() == label.get_value(),
                    None => false
                }
            }))
        }));
    };

    // Scenario: [01D4D40A3652FWV58EQMY6907F] Gather metrics for labels with some non-matching labels
    when regex "01D4D40A3652FWV58EQMY6907F" | world, _matches, _step| {
        world.desc_names = vec![
            world.counter.desc()[0].fq_name.clone(),
            world.int_counter.desc()[0].fq_name.clone(),
            world.counter_vec.desc()[0].fq_name.clone(),
        ];
        for label_pair in &world.counter.desc()[0].const_label_pairs {
            world.labels.insert(label_pair.get_name().to_string(), label_pair.get_value().to_string());
        }
        for label_pair in &world.int_counter.desc()[0].const_label_pairs {
            world.labels.insert(label_pair.get_name().to_string(), label_pair.get_value().to_string());
        }
        for label_pair in &world.counter_vec.desc()[0].const_label_pairs {
            world.labels.insert(label_pair.get_name().to_string(), label_pair.get_value().to_string());
        }
        for label_pair in &world.int_counter_vec.desc()[0].const_label_pairs {
            // non matching label value
            world.labels.insert(label_pair.get_name().to_string(), ULID::generate().to_string());
        }
        // non matching label
        world.labels.insert(ULID::generate().to_string(), ULID::generate().to_string());

        metrics::registry().gather();
        world.metric_families = metrics::registry().gather_for_labels(&world.labels);
    };

    then regex "01D4D40A3652FWV58EQMY6907F" | world, _matches, _step| {
        let metric_families = metrics::registry().gather_for_desc_names(&world.desc_names);
        assert_eq!(world.metric_families.len(), metric_families.len());
        assert!(world.metric_families.iter().all(|mf| metric_families.iter().any(|mf2| mf2.get_name() == mf.get_name())));
        assert!(world.metric_families.iter().all(|mf| {
            mf.get_metric().iter().any(|m| m.get_label().iter().any(|label|{
                match world.labels.get(label.get_name()) {
                    Some(value) => value.as_str() == label.get_value(),
                    None => false
                }
            }))
        }));
    };

    // Scenario: [01D4D417QGFCY2XSSARWWH49P5] Gather metrics for labels with no matching labels
    when regex "01D4D417QGFCY2XSSARWWH49P5" | world, _matches, _step| {
        let labels = hashmap! {
            ULID::generate().to_string() => ULID::generate().to_string()
        };
        metrics::registry().gather();
        world.metric_families = metrics::registry().gather_for_labels(&labels);
    };

    then regex "01D4D417QGFCY2XSSARWWH49P5" | world, _matches, _step| {
        assert!(world.metric_families.is_empty());
    };

    // Scenario: [01D4D3WKY9607QG71S76DE65W8] Gather metrics for labels using an empty HashMap
    when regex "01D4D3WKY9607QG71S76DE65W8" | world, _matches, _step| {
        metrics::registry().gather();
        world.metric_families = metrics::registry().gather_for_labels(&HashMap::new());
    };

    then regex "01D4D3WKY9607QG71S76DE65W8" | world, _matches, _step| {
        assert!(world.metric_families.is_empty());
    };


});

fn find_next_non_existent_desc_id(mut start: metrics::DescId) -> metrics::DescId {
    let desc_ids: HashSet<_> = metrics::registry()
        .descs()
        .iter()
        .map(|desc| desc.id)
        .collect();
    loop {
        if !desc_ids.contains(&start) {
            return start;
        }
        start += 1;
    }
}

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
    desc_names: Vec<String>,
    labels: HashMap<String, String>,
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
            counter: metrics::CounterBuilder::new(metrics::MetricId::generate(), "counter")
                .with_label(metrics::LabelId::generate(), "A")
                .build()
                .unwrap(),
            int_counter: metrics::IntCounterBuilder::new(
                metrics::MetricId::generate(),
                "int counter",
            )
            .with_label(metrics::LabelId::generate(), "A")
            .build()
            .unwrap(),
            gauge: metrics::GaugeBuilder::new(metrics::MetricId::generate(), "gauge")
                .with_label(metrics::LabelId::generate(), "A")
                .build()
                .unwrap(),
            int_gauge: metrics::IntGaugeBuilder::new(metrics::MetricId::generate(), "int gauge")
                .with_label(metrics::LabelId::generate(), "A")
                .build()
                .unwrap(),
            counter_vec: metrics::CounterVecBuilder::new(
                metrics::MetricId::generate(),
                "counter vec",
                vec![metrics::LabelId::generate()],
            )
            .with_label(metrics::LabelId::generate(), "A")
            .build()
            .unwrap(),
            int_counter_vec: metrics::IntCounterVecBuilder::new(
                metrics::MetricId::generate(),
                "int counter vec",
                vec![metrics::LabelId::generate()],
            )
            .with_label(metrics::LabelId::generate(), "A")
            .build()
            .unwrap(),
            gauge_vec: metrics::GaugeVecBuilder::new(
                metrics::MetricId::generate(),
                "gauge vec",
                vec![metrics::LabelId::generate()],
            )
            .with_label(metrics::LabelId::generate(), "A")
            .build()
            .unwrap(),
            int_gauge_vec: metrics::IntGaugeVecBuilder::new(
                metrics::MetricId::generate(),
                "int gauge vec",
                vec![metrics::LabelId::generate()],
            )
            .with_label(metrics::LabelId::generate(), "A")
            .build()
            .unwrap(),
            histogram: metrics::HistogramBuilder::new(
                metrics::MetricId::generate(),
                "histogram",
                vec![0.1],
            )
            .with_label(metrics::LabelId::generate(), "A")
            .build()
            .unwrap(),
            histogram_vec: metrics::HistogramVecBuilder::new(
                metrics::MetricId::generate(),
                "histogram vec",
                vec![0.1],
                vec![metrics::LabelId::generate()],
            )
            .with_label(metrics::LabelId::generate(), "A")
            .build()
            .unwrap(),

            world2: None,
            metric_families: Vec::new(),
            desc_ids: Vec::new(),
            desc_names: Vec::new(),
            labels: HashMap::new(),
        }
    }
}
