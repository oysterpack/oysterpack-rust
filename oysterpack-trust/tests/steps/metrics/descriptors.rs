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
use prometheus::{
    core::{Collector, Desc},
    proto::MetricFamily,
    IntCounter, IntGauge,
};
use std::collections::{HashMap, HashSet};

steps!(World => {
    // Feature: [01D3SF7KGJZZM50TXXW5HX4N99] All metric descriptors can be retrieved from the metric registry.

    // Scenario: [01D3SF3R0DTBTVRKC9PFHQEEM9] All metric descriptors are returned
    given regex "01D3SF3R0DTBTVRKC9PFHQEEM9" | world, _matches, _step | {
        metrics::registry().register(world.clone()).unwrap();
    };

    when regex "01D3SF3R0DTBTVRKC9PFHQEEM9" | world, _matches, _step| {
        world.descs = metrics::registry().descs();
    };

    then regex "01D3SF3R0DTBTVRKC9PFHQEEM9" | world, _matches, _step| {
        let desc_ids = world.descs.iter().map(|desc| desc.id.clone()).collect::<HashSet<_>>();
        assert!(metrics::registry().collectors().iter().all(|collector| collector.desc().iter().all(|desc|desc_ids.contains(&desc.id))));
    };

    // Feature: [01D3SF7KGJZZM50TXXW5HX4N99] Find descriptors matching filters

    // Background:
    //    Given [01D3PQBDWM4BAJQKXF9R0MQED7] metrics are registered
    given regex "01D3PQBDWM4BAJQKXF9R0MQED7" | world, _matches, _step | {
        metrics::registry().register(world.clone()).unwrap();
    };

    // Scenario: [01D3PSPCNHH6CSW08RTFKZZ8SP] Find metric descriptors matching a filter
    when regex "01D3PSPCNHH6CSW08RTFKZZ8SP" | world, _matches, _step| {
        let desc_names: HashSet<_> = world.desc().iter().map(|desc| desc.fq_name.as_str()).collect();
        world.descs = metrics::registry().find_descs(|desc| desc_names.contains(desc.fq_name.as_str()));
    };

    then regex "01D3PSPCNHH6CSW08RTFKZZ8SP" | world, _matches, _step| {
        let desc_names: HashSet<_> = world.desc().iter().map(|desc| desc.fq_name.as_str()).collect();
        assert!(world.descs.iter().all(|desc| desc_names.contains(desc.fq_name.as_str())));
    };

    // Scenario: [01D3PSP4TQK6ESKSB6AEFWAAYF] Find descriptors for MetricId(s)
    when regex "01D3PSP4TQK6ESKSB6AEFWAAYF" | world, _matches, _step| {
        let metric_ids: Vec<_> = world.desc().iter().map(|desc| metrics::parse_desc_metric_id(desc).unwrap()).collect();
        world.descs = metrics::registry().descs_for_metric_ids(metric_ids.as_slice());
    };

    then regex "01D3PSP4TQK6ESKSB6AEFWAAYF" | world, _matches, _step| {
        let desc_names: HashSet<_> = world.desc().iter().map(|desc| desc.id.clone()).collect();
        assert!(world.descs.iter().all(|desc| desc_names.contains(&desc.id)));
    };

    // Scenario: [01D48G17F9EYM0XEZBZ794SGMW] Find descriptors for MetricId(s) containing some non-registered MetricId(s)
    when regex "01D48G17F9EYM0XEZBZ794SGMW" | world, _matches, _step| {
        let mut metric_ids: Vec<_> = world.desc().iter().map(|desc| metrics::parse_desc_metric_id(desc).unwrap()).collect();
        metric_ids.push(metrics::MetricId::generate());
        world.descs = metrics::registry().descs_for_metric_ids(metric_ids.as_slice());
    };

    then regex "01D48G17F9EYM0XEZBZ794SGMW" | world, _matches, _step| {
        let desc_names: HashSet<_> = world.desc().iter().map(|desc| desc.id.clone()).collect();
        assert!(world.descs.iter().all(|desc| desc_names.contains(&desc.id)));
    };

    // Scenario: [01D48TCVH8R57XSNQN4E89PYXC] Find descriptors where all MetricId(s) are not registered
    when regex "01D48TCVH8R57XSNQN4E89PYXC" | world, _matches, _step| {
        let metric_ids = vec![metrics::MetricId::generate(), metrics::MetricId::generate()];
        world.descs = metrics::registry().descs_for_metric_ids(metric_ids.as_slice());
    };

    then regex "01D48TCVH8R57XSNQN4E89PYXC" | world, _matches, _step| {        
        assert!(world.descs.is_empty());
    };

    // Scenario: [01D48TCN32NHVFEYSJCHCQE451] Find descriptors for an empty &[MetricId]
    when regex "01D48TCN32NHVFEYSJCHCQE451" | world, _matches, _step| {
        world.descs = metrics::registry().descs_for_metric_ids(&[]);
    };

    then regex "01D48TCN32NHVFEYSJCHCQE451" | world, _matches, _step| {        
        assert!(world.descs.is_empty());
    };

    // Scenario: [01D48FX6T8SAJZWHDTZBQYWFAG] Find descriptors that match const labels
    when regex "01D48FX6T8SAJZWHDTZBQYWFAG" | world, _matches, _step| {
        let labels = world.all_desc_const_labels();
        world.descs = metrics::registry().descs_for_labels(&labels);
    };

    then regex "01D48FX6T8SAJZWHDTZBQYWFAG" | world, _matches, _step| {
        let labels = world.all_desc_const_labels();
        assert_eq!(world.descs.len(), labels.len());
    };

    // Scenario: [01D48TM50MN9ZPGYD1TD2QBSKA] Find descriptors against labels containing unknown label names and values
    when regex "01D48TM50MN9ZPGYD1TD2QBSKA" | world, _matches, _step| {
        let mut labels = world.all_desc_const_labels();
        labels.insert(metrics::LabelId::generate().name(),"A".to_string());
        world.descs = metrics::registry().descs_for_labels(&labels);
    };

    then regex "01D48TM50MN9ZPGYD1TD2QBSKA" | world, _matches, _step| {
        let labels = world.all_desc_const_labels();
        assert_eq!(world.descs.len(), labels.len());
    };

    // Scenario: [01D48TKY8GJJ56Z14NAX000DPZ] Find descriptors against labels that are all unknown
    when regex "01D48TKY8GJJ56Z14NAX000DPZ" | world, _matches, _step| {
        let mut labels = HashMap::new();
        labels.insert(metrics::LabelId::generate().name(),"A".to_string());
        world.descs = metrics::registry().descs_for_labels(&labels);
    };

    then regex "01D48TKY8GJJ56Z14NAX000DPZ" | world, _matches, _step| {
        assert!(world.descs.is_empty());
    };

    // Scenario: [01D48TK6AMZCQ5CNYMJC0NVR37] Find descriptors against an empty labels HashMap
    when regex "01D48TK6AMZCQ5CNYMJC0NVR37" | world, _matches, _step| {
        let mut labels = HashMap::new();
        world.descs = metrics::registry().descs_for_labels(&labels);
    };

    then regex "01D48TK6AMZCQ5CNYMJC0NVR37" | world, _matches, _step| {
        assert!(world.descs.is_empty());
    };
});

#[derive(Clone)]
pub struct World {
    counter: IntCounter,
    gauge: IntGauge,
    descs: Vec<prometheus::core::Desc>,
}

impl World {
    fn all_desc_const_labels(&self) -> HashMap<String, String> {
        self.desc().iter().fold(HashMap::new(), |mut map, desc| {
            for label_pair in &desc.const_label_pairs {
                map.insert(
                    label_pair.get_name().to_string(),
                    label_pair.get_value().to_string(),
                );
            }
            map
        })
    }
}

/// Each World instance contains unique metrics, i.e., unique metric descriptors because of unique MetricId
impl Collector for World {
    fn desc(&self) -> Vec<&Desc> {
        self.counter
            .desc()
            .iter()
            .cloned()
            .chain(self.gauge.desc().iter().cloned())
            .collect()
    }

    fn collect(&self) -> Vec<MetricFamily> {
        self.counter
            .collect()
            .iter()
            .cloned()
            .chain(self.gauge.collect().iter().cloned())
            .collect()
    }
}

impl Default for World {
    fn default() -> World {
        Self {
            counter: metrics::new_int_counter(
                metrics::MetricId::generate(),
                "counter",
                Some(hashmap! {
                    metrics::LabelId::generate() => "A".to_string()
                }),
            )
            .unwrap(),
            gauge: metrics::new_int_gauge(
                metrics::MetricId::generate(),
                "gauge",
                Some(hashmap! {
                    metrics::LabelId::generate() => "B".to_string()
                }),
            )
            .unwrap(),
            descs: Vec::new(),
        }
    }
}
