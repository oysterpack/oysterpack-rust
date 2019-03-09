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
use std::collections::{
    HashSet, HashMap
};

steps!(World => {
    // Feature: [01D3JAHR4Z02XTJGTNE4D63VRT] Any `prometheus::core::Collector` can be registered

    given regex "01D3JAKE384RJA4FM9NJJNDPV6" |_world, _matches, _step| {
        // World implements Collector
    };

    when regex "01D3JAKE384RJA4FM9NJJNDPV6" |world, _matches, _step| {
        metrics::registry().register(world.clone()).unwrap();
        metrics::registry().register(World::default()).unwrap();
    };

    then regex "01D3JAKE384RJA4FM9NJJNDPV6-1" |world, _matches, _step| {
        check_collector_is_registered(world);
    };

    then regex "01D3JAKE384RJA4FM9NJJNDPV6-2" |world, _matches, _step| {
        check_collector_descs_from_registry(world);
    };

    then regex "01D3JAKE384RJA4FM9NJJNDPV6-3" |world, _matches, _step| {
        check_collector_descs_from_registry(world);
    };

    // Feature: [01D3SF69J8P9T9PSKEXKQJV1ME] All registered collectors can be retrieved from the registry

    given regex "01D46BWEKHMHZGSAZF4QQCZ0RV" |world, _matches, _step| {
        metrics::registry().register(world.clone()).unwrap();
    };

    when regex "01D3PSPRYX7XHSGX0JFC8TT59H" |world, _matches, _step| {
        world.collectors = metrics::registry().collectors()
    };

    then regex "01D3PSPRYX7XHSGX0JFC8TT59H-1" |world, _matches, _step| {
        check_all_descs_match(world);
    };

    then regex "01D3PSPRYX7XHSGX0JFC8TT59H-2" |world, _matches, _step| {
        assert_eq!(world.collectors.len(), metrics::registry().collector_count());
    };

    // Feature: [01D3SF69J8P9T9PSKEXKQJV1ME] Collectors can be retrieved selectively by applying filters

    // Scenario: [01D3PX3BGCMV2PS6FDXHH0ZEB1] Find collectors using a filter
    when regex "01D3PX3BGCMV2PS6FDXHH0ZEB1" |world, _matches, _step| {
        world.collectors = metrics::registry().find_collectors(|descs| descs.len() == world.desc().len());
    };

    then regex "01D3PX3BGCMV2PS6FDXHH0ZEB1" |world, _matches, _step| {
        assert!(world.collectors.iter().all(|collector| collector.desc().len() == world.desc().len()));
    };

    // Scenario: [01D3PX3NRADQPMS95EB5C7ECD7] Find collectors for MetricId(s)
    when regex "01D3PX3NRADQPMS95EB5C7ECD7" |world, _matches, _step| {
        world.collectors_for_metric_ids = metrics::registry().collectors_for_metric_ids(world.metric_ids().as_slice());
    };

    then regex "01D3PX3NRADQPMS95EB5C7ECD7" |world, _matches, _step| {
        assert_eq!(world.collectors_for_metric_ids.len(), world.metric_ids().len());
        check_collectors_for_metric_ids_match(world);
    };

    // Scenario: [01D44BHGQVNTCMK7YXM2F0W65K] Find collectors for MetricId(s) containing some non-matching MetricId(s)
    when regex "01D44BHGQVNTCMK7YXM2F0W65K" |world, _matches, _step| {
        let mut metric_ids = world.metric_ids();
        metric_ids.push(metrics::MetricId::generate());
        metric_ids.push(metrics::MetricId::generate());
        world.collectors_for_metric_ids = metrics::registry().collectors_for_metric_ids(metric_ids.as_slice());
    };

    then regex "01D44BHGQVNTCMK7YXM2F0W65K" |world, _matches, _step| {
        assert_eq!(world.collectors_for_metric_ids.len(), world.metric_ids().len());
        check_collectors_for_metric_ids_match(world);
    };

    // Scenario: [01D44BHVAVQXV9WHA6CYGVB7V6] Find collectors for multiple MetricId(s) containing no matching MetricId(s)
    when regex "01D44BHVAVQXV9WHA6CYGVB7V6" |world, _matches, _step| {
        let metric_ids = vec![metrics::MetricId::generate(), metrics::MetricId::generate()];
        world.collectors_for_metric_ids = metrics::registry().collectors_for_metric_ids(metric_ids.as_slice());
    };

    then regex "01D44BHVAVQXV9WHA6CYGVB7V6" |world, _matches, _step| {
        assert!(world.collectors_for_metric_ids.is_empty());
    };

    // Scenario: [01D44BJ3MGK6GMNJV8KAFSNFH9] Find collectors passing in an empty MetricId slice
    when regex "01D44BJ3MGK6GMNJV8KAFSNFH9" |world, _matches, _step| {
        world.collectors_for_metric_ids = metrics::registry().collectors_for_metric_ids(&[]);
    };

    then regex "01D44BJ3MGK6GMNJV8KAFSNFH9" |world, _matches, _step| {
        assert!(world.collectors_for_metric_ids.is_empty());
    };

    // Scenario: [01D44BJV9VR2RWBARMBS1A0MXC] Find collectors for a MetricId
    when regex "01D44BJV9VR2RWBARMBS1A0MXC" |world, _matches, _step| {
        world.collectors = metrics::registry().collectors_for_metric_id(world.metric_ids()[0])
    };

    then regex "01D44BJV9VR2RWBARMBS1A0MXC" |world, _matches, _step| {
        assert_eq!(world.collectors.len(), 1);
        assert_eq!(world.collectors[0].desc()[0].fq_name, world.metric_ids()[0].name());
    };

    // Scenario: [01D44BK3DYBM5JJJMBVXK36S49] Find collectors for a MetricId that is not registered
    when regex "01D44BK3DYBM5JJJMBVXK36S49" |world, _matches, _step| {
        world.collectors = metrics::registry().collectors_for_metric_id(metrics::MetricId::generate());
    };

    then regex "01D44BK3DYBM5JJJMBVXK36S49" |world, _matches, _step| {
        assert!(world.collectors.is_empty());
    };

    // Scenario: [01D45SST98R0VJY58JM2X1WN7E] Find collectors for DescId(s)
    when regex "01D45SST98R0VJY58JM2X1WN7E" |world, _matches, _step| {
        world.collectors_for_desc_ids = metrics::registry().collectors_for_desc_ids(world.desc_ids().iter().cloned().collect::<Vec<_>>().as_slice());
    };

    then regex "01D45SST98R0VJY58JM2X1WN7E" |world, _matches, _step| {
        let desc_ids = world.desc_ids();
        assert!(world.collectors_for_desc_ids.keys().all(|id| desc_ids.contains(id)));
    };

    // Scenario: [01D44BKW1E97TGFJGE23FK654K] Find collectors for DescId(s) containing some that are not registered
    when regex "01D44BKW1E97TGFJGE23FK654K" |world, _matches, _step| {
        let mut desc_ids = world.desc_ids();
        desc_ids.insert(0);
        world.collectors_for_desc_ids = metrics::registry().collectors_for_desc_ids(desc_ids.iter().cloned().collect::<Vec<_>>().as_slice());
    };

    then regex "01D44BKW1E97TGFJGE23FK654K" |world, _matches, _step| {
        let desc_ids = world.desc_ids();
        assert!(world.collectors_for_desc_ids.keys().all(|id| desc_ids.contains(id)));
    };

    // Scenario: [01D44BM35C61QE76Q2JGKGBKV7] Find collectors for DescId(s) that are not registered
    when regex "01D44BM35C61QE76Q2JGKGBKV7" |world, _matches, _step| {
        let desc_ids = vec![0,1,2];
        world.collectors_for_desc_ids = metrics::registry().collectors_for_desc_ids(desc_ids.as_slice());
    };

    then regex "01D44BM35C61QE76Q2JGKGBKV7" |world, _matches, _step| {
        assert!(world.collectors_for_desc_ids.is_empty());
    };

    // Scenario: [01D44BMDK667A9QNFMQ9H89T95] Find collectors with empty DescId slice
    when regex "01D44BMDK667A9QNFMQ9H89T95" |world, _matches, _step| {
        let desc_ids = vec![];
        world.collectors_for_desc_ids = metrics::registry().collectors_for_desc_ids(desc_ids.as_slice());
    };

    then regex "01D44BMDK667A9QNFMQ9H89T95" |world, _matches, _step| {
        assert!(world.collectors_for_desc_ids.is_empty());
    };

    // Scenario: [01D44BMWHBX0BY1JVRZHGA78HM] Find collector by metric DescId
    when regex "01D44BMWHBX0BY1JVRZHGA78HM" |_world, _matches, _step| {
        // find collector by DescId
    };

    then regex "01D44BMWHBX0BY1JVRZHGA78HM" |world, _matches, _step| {
        let desc_id = world.desc_ids().iter().next().unwrap().clone();
        match metrics::registry().collectors_for_desc_id(desc_id) {
            Some(collector) => assert!(collector.desc().iter().any(|desc| desc.id == desc_id)),
            None => panic!("collector not found")
        }
    };

    // Scenario: [01D44BN406V10VRCBGWM4PBDTX] Find collector by metric DescId that is not registered
    when regex "01D44BN406V10VRCBGWM4PBDTX" |_world, _matches, _step| {
        // Find collector by metric DescId that is not registered
    };

    then regex "01D44BN406V10VRCBGWM4PBDTX" |_world, _matches, _step| {
        assert!(metrics::registry().collectors_for_desc_id(0).is_none());
    };

});

fn check_collectors_for_metric_ids_match(world: &mut World) {
    let metric_ids = world.metric_ids();
    assert!(world
        .collectors_for_metric_ids
        .keys()
        .all(|metric_id| metric_ids.iter().any(|id| id == metric_id)));
}

fn check_all_descs_match(world: &mut World) {
    let descs = metrics::registry().descs();
    let collector_descs: Vec<_> = world.collectors.iter().flat_map(|c| c.desc()).collect();
    assert_eq!(descs.len(), collector_descs.len());
    let desc_ids: HashSet<_> = descs.iter().map(|desc| desc.id).collect();
    assert!(collector_descs
        .iter()
        .all(|desc| desc_ids.contains(&desc.id)));
}

fn check_collector_is_registered(world: &mut World) {
    let desc_ids: HashSet<_> = world.desc().iter().map(|desc| desc.id).collect();
    let collectors = metrics::registry().find_collectors(|descs| {
        descs.len() == desc_ids.len() && descs.iter().all(|desc| desc_ids.contains(&desc.id))
    });
    assert_eq!(collectors.len(), 1);
}

fn check_collector_descs_from_registry(world: &mut World) {
    let desc_ids: HashSet<_> = world.desc().iter().map(|desc| desc.id).collect();
    let descs = metrics::registry().find_descs(|desc| desc_ids.contains(&desc.id));
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
    collectors: Vec<metrics::ArcCollector>,
    collectors_for_metric_ids: HashMap<metrics::MetricId, Vec<metrics::ArcCollector>>,
    collectors_for_desc_ids: HashMap<metrics::DescId, metrics::ArcCollector>,
}

impl World {
    fn metric_ids(&self) -> Vec<metrics::MetricId> {
        vec![
            self.counter.desc()[0].fq_name.as_str().parse().unwrap(),
            self.gauge.desc()[0].fq_name.as_str().parse().unwrap(),
        ]
    }

    fn desc_ids(&self) -> HashSet<metrics::DescId> {
        vec![self.counter.desc(), self.gauge.desc()]
            .iter()
            .flat_map(|descs| descs.iter().map(|desc| desc.id))
            .collect()
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
            counter: metrics::IntCounterBuilder::new(metrics::MetricId::generate(), "counter")
                .with_label(metrics::LabelId::generate(), "A")
                .build()
                .unwrap(),
            gauge: metrics::IntGaugeBuilder::new(metrics::MetricId::generate(), "gauge")
                .with_label(metrics::LabelId::generate(), "A")
                .build()
                .unwrap(),
            collectors: Vec::new(),
            collectors_for_metric_ids: HashMap::default(),
            collectors_for_desc_ids: HashMap::default(),
        }
    }
}
