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
    given regex "01D3JAKE384RJA4FM9NJJNDPV6" |_world, _matches, step| {
        // World implements Collector
    };

    when regex "01D3JAKE384RJA4FM9NJJNDPV6" |world, _matches, step| {
        metrics::registry().register(world.clone()).unwrap();
        metrics::registry().register(World::default()).unwrap();
    };

    then regex "01D3JAKE384RJA4FM9NJJNDPV6-1" |world, _matches, step| {
        check_collector_is_registered(world);
    };

    then regex "01D3JAKE384RJA4FM9NJJNDPV6-2" |world, _matches, step| {
        check_collector_descs_from_registry(world);
    };

    then regex "01D3JAKE384RJA4FM9NJJNDPV6-3" |world, _matches, step| {
        check_collector_descs_from_registry(world);
    };
});

fn check_collector_is_registered(world: &mut World) {
    let desc_ids: HashSet<_> = world.desc().iter().map(|desc| desc.id).collect();
    let collectors = metrics::registry().filter_collectors(|collector| {
        let descs = collector.desc();
        descs.len() == desc_ids.len() && descs.iter().all(|desc| desc_ids.contains(&desc.id))
    });
    assert_eq!(collectors.len(), 1);
}

fn check_collector_descs_from_registry(world: &mut World) {
    let desc_ids: HashSet<_> = world.desc().iter().map(|desc| desc.id).collect();
    let descs = metrics::registry().filter_descs(|desc| desc_ids.contains(&desc.id));
    assert_eq!(descs.len(), desc_ids.len());
}

fn check_collector_metrics_gathered(world: &mut World) {
    let desc_ids: Vec<_> = world.desc().iter().map(|desc| desc.id).collect();
    let descs = metrics::registry().gather_for_desc_ids(desc_ids.as_slice());
    assert_eq!(descs.len(), desc_ids.len());
}

#[derive(Clone)]
pub struct World {
    counter: IntCounter,
    gauge: IntGauge,
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
        }
    }
}
