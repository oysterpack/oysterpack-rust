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
    collections::{HashMap, HashSet},
    sync::Arc,
    thread,
};

steps!(World => {
    // Feature: [01D43V1W2BHDR5MK08D1HFSFZX] A global prometheus metrics registry is provided.

    // Scenario: [01D3J3D7PA4NR9JABZWT635S6B] Using the global registry from multiple threads
    given regex "01D3J3D7PA4NR9JABZWT635S6B" | _world, _matches, _step | {
        // there are 2 threads using the global registry
    };

    when regex "01D3J3D7PA4NR9JABZWT635S6B" | world, _matches, _step| {
        // register metrics on the main thread
        world.register_metrics();
    };

    then regex "01D3J3D7PA4NR9JABZWT635S6B" | _world, _matches, _step| {
        // gather metrics on another thread
        let handle = thread::spawn(|| {
            metrics::registry().gather()
        });

        assert_eq!(metrics::registry().gather().len(), handle.join().unwrap().len());
    };

    // Rule: Descriptors registered with the same registry have to fulfill certain consistency and uniqueness criteria if they share the same fully-qualified name.

    // Scenario: [01D3J3DRS0CJ2YN99KAWQ19103] Register 2 metrics using the same MetricId and no labels
    given regex "01D3J3DRS0CJ2YN99KAWQ19103" | world, _matches, _step | {
        world.metric_id = metrics::MetricId::generate();
        metrics::registry().register_counter(world.metric_id, "01D3J3D7PA4NR9JABZWT635S6B", None).unwrap();;
    };

    when regex "01D3J3DRS0CJ2YN99KAWQ19103" | world, _matches, _step| {
        assert!(metrics::registry().register_gauge(world.metric_id, "01D3J3D7PA4NR9JABZWT635S6B", None).is_err());
    };

    then regex "01D3J3DRS0CJ2YN99KAWQ19103" | world, _matches, _step| {
        assert_eq!(metrics::registry().descs_for_metric_id(world.metric_id).len(), 1);
    };

    // Scenario: [01D3MT4JY1NZH2WW0347B9ZAS7] Register 2 metrics using the same MetricId and same const labels
    given regex "01D3MT4JY1NZH2WW0347B9ZAS7" | world, _matches, _step | {
        world.metric_id = metrics::MetricId::generate();
        let labels = world.counter.desc()[0].const_label_pairs.iter().fold(HashMap::new(),|mut map, label_pair| {
            map.insert(label_pair.get_name().parse().unwrap(), label_pair.get_value().to_string());
            map
        });
        metrics::registry().register_counter(world.metric_id, "01D3MT4JY1NZH2WW0347B9ZAS7", Some(labels)).unwrap();;
    };

    when regex "01D3MT4JY1NZH2WW0347B9ZAS7" | world, _matches, _step| {
        let labels = world.counter.desc()[0].const_label_pairs.iter().fold(HashMap::new(),|mut map, label_pair| {
            map.insert(label_pair.get_name().parse().unwrap(), label_pair.get_value().to_string());
            map
        });
        assert!(metrics::registry().register_gauge(world.metric_id, "01D3MT4JY1NZH2WW0347B9ZAS7", Some(labels)).is_err());
    };

    then regex "01D3MT4JY1NZH2WW0347B9ZAS7" | world, _matches, _step| {
        assert_eq!(metrics::registry().descs_for_metric_id(world.metric_id).len(), 1);
    };

    // Scenario: [01D3MT8KDP434DKZ6Y54C80BB0] Register 2 metrics using the same MetricId and same const label names but different label values
    given regex "01D3MT8KDP434DKZ6Y54C80BB0" | world, _matches, _step | {
        world.metric_id = metrics::MetricId::generate();
        let label_id: metrics::LabelId = format!("L{}", world.metric_id.ulid()).as_str().parse().unwrap();
        let labels = hashmap!{
            label_id => "A".to_string()
        };
        metrics::registry().register_counter(world.metric_id, "01D3MT4JY1NZH2WW0347B9ZAS7", Some(labels)).unwrap();;
    };

    when regex "01D3MT8KDP434DKZ6Y54C80BB0" | world, _matches, _step| {
        let label_id: metrics::LabelId = format!("L{}", world.metric_id.ulid()).as_str().parse().unwrap();
        let labels = hashmap!{
            label_id => "B".to_string()
        };
        metrics::registry().register_gauge(world.metric_id, "01D3MT4JY1NZH2WW0347B9ZAS7", Some(labels)).unwrap();
    };

    then regex "01D3MT8KDP434DKZ6Y54C80BB0" | world, _matches, _step| {
        assert_eq!(metrics::registry().descs_for_metric_id(world.metric_id).len(), 2);
    };

    // Scenario: [01D4D8W99GP21E6MZHAQXEHTE3] Register 2 metrics using the same MetricId and same const label values but different label names
    given regex "01D4D8W99GP21E6MZHAQXEHTE3" | world, _matches, _step | {
        world.metric_id = metrics::MetricId::generate();
        let labels = hashmap!{
            metrics::LabelId::generate() => "A".to_string()
        };
        metrics::registry().register_counter(world.metric_id, "01D3MT4JY1NZH2WW0347B9ZAS7", Some(labels)).unwrap();;
    };

    when regex "01D4D8W99GP21E6MZHAQXEHTE3" | world, _matches, _step| {
        let labels = hashmap!{
            metrics::LabelId::generate() => "A".to_string()
        };
        assert!(metrics::registry().register_gauge(world.metric_id, "01D3MT4JY1NZH2WW0347B9ZAS7", Some(labels)).is_err());
    };

    then regex "01D4D8W99GP21E6MZHAQXEHTE3" | world, _matches, _step| {
        assert_eq!(metrics::registry().descs_for_metric_id(world.metric_id).len(), 1);
    };

    // Scenario: [01D4D68AW6FYYESQZQCZH8JGCG] Register 2 metrics using the same MetricId and same const label names but different label values and different help
    given regex "01D4D68AW6FYYESQZQCZH8JGCG" | world, _matches, _step | {
        world.metric_id = metrics::MetricId::generate();
        let label_id: metrics::LabelId = format!("L{}", world.metric_id.ulid()).as_str().parse().unwrap();
        let labels = hashmap!{
            label_id => "B".to_string()
        };
        metrics::registry().register_counter(world.metric_id, ULID::generate().to_string(), Some(labels)).unwrap();;
    };

    when regex "01D4D68AW6FYYESQZQCZH8JGCG" | world, _matches, _step| {
        let label_id: metrics::LabelId = format!("L{}", world.metric_id.ulid()).as_str().parse().unwrap();
        let labels = hashmap!{
            label_id => "A".to_string()
        };
        assert!(metrics::registry().register_gauge(world.metric_id, ULID::generate().to_string(), Some(labels)).is_err());
    };

    then regex "01D4D68AW6FYYESQZQCZH8JGCG" | world, _matches, _step| {
        assert_eq!(metrics::registry().descs_for_metric_id(world.metric_id).len(), 1);
    };

    // Rule: descriptor `help` must not be blank

    // Scenario: [01D4B036AWCJD6GCDNVGA5YVBB] Register metrics with a blank help message on the descriptor
    when regex "01D4B036AWCJD6GCDNVGA5YVBB" | world, _matches, _step | {
        world.metric_id = metrics::MetricId::generate();
        match metrics::registry().register_counter(world.metric_id, "  ", None) {
            Err(err) => println!("{}", err),
            Ok(_) => panic!("should have failed to register")
        }
    };

    then regex "01D4B036AWCJD6GCDNVGA5YVBB" | world, _matches, _step| {
        assert!(metrics::registry().descs_for_metric_id(world.metric_id).is_empty());
    };

    // Scenario: [01D4B08N90FM8EZTT3X5Y72D3M] Register a collector containing multiple descriptors where 1 descriptor has a blank help message
    when regex "01D4B08N90FM8EZTT3X5Y72D3M" | world, _matches, _step | {
        world.metric_id = metrics::MetricId::generate();

        struct Foo {
            world: World,
            bad_desc: prometheus::core::Desc,
            good_desc: prometheus::core::Desc
        };

        impl Collector for Foo {
            fn desc(&self) -> Vec<&Desc> {
                vec![&self.bad_desc,&self.good_desc]
            }

            fn collect(&self) -> Vec<MetricFamily> {
                vec![]
            }
        }

        let foo = Foo{
            world: world.clone(),
            bad_desc: prometheus::core::Desc::new(world.metric_id.name(), "help".to_string(), vec![], HashMap::new()).unwrap(),
            good_desc:  prometheus::core::Desc::new(metrics::MetricId::generate().name(), " ".to_string(), vec![], HashMap::new()).unwrap()
        };

        match metrics::registry().register(foo) {
            Err(err) => println!("{}", err),
            Ok(_) => panic!("should have failed to register")
        }
    };

    then regex "01D4B08N90FM8EZTT3X5Y72D3M" | world, _matches, _step| {
        assert!(metrics::registry().descs_for_metric_id(world.metric_id).is_empty());
    };

    // Rule: descriptor `help` max length is 250

    // Scenario: [01D4B0S8QW63C6YFCB83CQZXA7] Register metrics with a help message length of 250
    when regex "01D4B0S8QW63C6YFCB83CQZXA7" | world, _matches, _step | {
        world.metric_id = metrics::MetricId::generate();
        let mut help = world.metric_id.to_string();
        while help.len() <  metrics::MetricRegistry::DESC_HELP_MAX_LEN {
            help.extend(world.metric_id.to_string().chars());
        }
        let help = &help[..metrics::MetricRegistry::DESC_HELP_MAX_LEN];
        metrics::registry().register_counter(world.metric_id, help, None).unwrap();
    };

    then regex "01D4B0S8QW63C6YFCB83CQZXA7" | world, _matches, _step| {
        assert_eq!(metrics::registry().descs_for_metric_id(world.metric_id).len(), 1);
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
    desc_names: Vec<String>,
    labels: HashMap<String, String>,

    metric_id: metrics::MetricId,
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
            desc_names: Vec::new(),
            labels: HashMap::new(),
            metric_id: metrics::MetricId::generate(),
        }
    }
}
