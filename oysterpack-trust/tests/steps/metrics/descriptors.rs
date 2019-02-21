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
use std::collections::HashSet;

steps!(World => {
    // Feature: [01D3SF7KGJZZM50TXXW5HX4N99] All metric descriptors can be retrieved from the metric registry.

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

    given regex "01D3PQBDWM4BAJQKXF9R0MQED7" | world, _matches, _step | {
        metrics::registry().register(world.clone()).unwrap();
    };

    when regex "01D3PSPCNHH6CSW08RTFKZZ8SP" | world, _matches, _step| {
        let desc_names: HashSet<_> = world.desc().iter().map(|desc| desc.fq_name.as_str()).collect();
        world.descs = metrics::registry().find_descs(|desc| desc_names.contains(desc.fq_name.as_str()));
    };

    then regex "01D3PSPCNHH6CSW08RTFKZZ8SP" | world, _matches, _step| {
        let desc_names: HashSet<_> = world.desc().iter().map(|desc| desc.fq_name.as_str()).collect();
        assert!(world.descs.iter().all(|desc| desc_names.contains(desc.fq_name.as_str())));
    };
});

#[derive(Clone)]
pub struct World {
    counter: IntCounter,
    gauge: IntGauge,
    descs: Vec<prometheus::core::Desc>,
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
