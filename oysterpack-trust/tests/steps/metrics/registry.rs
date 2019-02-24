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
    time::Duration,
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
    then regex "01D4B036AWCJD6GCDNVGA5YVBB" | world, _matches, _step | {
        world.metric_id = metrics::MetricId::generate();
        match metrics::registry().register_counter(world.metric_id, "  ", None) {
            Err(err) => println!("{}", err),
            Ok(_) => panic!("should have failed to register")
        }
        match metrics::registry().register_int_counter(world.metric_id, "  ", None) {
            Err(err) => println!("{}", err),
            Ok(_) => panic!("should have failed to register")
        }
        match metrics::registry().register_gauge(world.metric_id, "  ", None) {
            Err(err) => println!("{}", err),
            Ok(_) => panic!("should have failed to register")
        }
        match metrics::registry().register_int_gauge(world.metric_id, "  ", None) {
            Err(err) => println!("{}", err),
            Ok(_) => panic!("should have failed to register")
        }

        let labels = hashmap! {
            metrics::LabelId::generate() => "A".to_string()
        };

        let help = " ".to_string();

        let label_ids = vec![metrics::LabelId::generate()];
        let buckets = vec![0.1,0.2];

        assert!(metrics::registry().register_gauge_vec(world.metric_id, help.clone(), &label_ids, Some(labels.clone())).is_err());
        assert!(metrics::registry().register_int_gauge_vec(world.metric_id, help.clone(), &label_ids, Some(labels.clone())).is_err());

        assert!(metrics::registry().register_histogram(world.metric_id, help.clone(), buckets.clone(), Some(labels.clone())).is_err());
        assert!(metrics::registry().register_histogram_timer(world.metric_id, help.clone(), vec![Duration::from_millis(1)].into(), Some(labels.clone())).is_err());
        assert!(metrics::registry().register_histogram_vec(world.metric_id, help.clone(), &label_ids, buckets.clone(), Some(labels.clone())).is_err());
    };

    // Scenario: [01D4B08N90FM8EZTT3X5Y72D3M] Register a collector containing multiple descriptors where 1 descriptor has a blank help message
    then regex "01D4B08N90FM8EZTT3X5Y72D3M" | world, _matches, _step | {
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
            good_desc: prometheus::core::Desc::new(world.metric_id.name(), "help".to_string(), vec![], HashMap::new()).unwrap(),
            bad_desc:  prometheus::core::Desc::new(metrics::MetricId::generate().name(), " ".to_string(), vec![], HashMap::new()).unwrap()
        };

        match metrics::registry().register(foo) {
            Err(err) => println!("{}", err),
            Ok(_) => panic!("should have failed to register")
        }
    };

    // Rule: descriptor `help` max length is 250

    // Scenario: [01D4B0S8QW63C6YFCB83CQZXA7] Register metrics with a help message with the max allowed length
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

    // Scenario: [01D4B0RS3V7NHCPDSPQTJDNB6C] Register metrics with a help message length 1 char bigger then the max allowed length
    then regex "01D4B0RS3V7NHCPDSPQTJDNB6C" | world, _matches, _step | {
        world.metric_id = metrics::MetricId::generate();
        let mut help = world.metric_id.to_string();
        while help.len() <  metrics::MetricRegistry::DESC_HELP_MAX_LEN + 1 {
            help.extend(world.metric_id.to_string().chars());
        }
        let help = &help[..metrics::MetricRegistry::DESC_HELP_MAX_LEN+1];
        assert!(metrics::registry().register_counter(world.metric_id, help, None).is_err());
    };

    // Scenario: [01D4B0S1J3XV06GEZJGA9Q5F8V] Register a collector containing multiple descriptors where 1 descriptor has a help message length 1 char bigger then the max allowed length
    then regex "01D4B0S1J3XV06GEZJGA9Q5F8V" | world, _matches, _step | {
        world.metric_id = metrics::MetricId::generate();
        let mut help = world.metric_id.to_string();
        while help.len() <  metrics::MetricRegistry::DESC_HELP_MAX_LEN + 1 {
            help.extend(world.metric_id.to_string().chars());
        }
        let help = &help[..metrics::MetricRegistry::DESC_HELP_MAX_LEN+1];

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
            good_desc:  prometheus::core::Desc::new(metrics::MetricId::generate().name(), help.to_string(), vec![], HashMap::new()).unwrap()
        };

        match metrics::registry().register(foo) {
            Err(err) => println!("{}", err),
            Ok(_) => panic!("should have failed to register")
        }
    };

    // Rule: descriptor constant label name or value must not be blank

    // Scenario: [01D4B0K42BC2TB0TAA2QP6BRWZ] Register metrics containing a descriptor with a blank label value
    then regex "01D4B0K42BC2TB0TAA2QP6BRWZ" | world, _matches, _step | {
        world.metric_id = metrics::MetricId::generate();
        let labels = hashmap! {
            metrics::LabelId::generate() => " ".to_string()
        };

        let help = "help".to_string();

        let label_ids = vec![metrics::LabelId::generate()];
        let buckets = vec![0.1,0.2];

        assert!(metrics::registry().register_counter(world.metric_id, help.clone(), Some(labels.clone())).is_err());
        assert!(metrics::registry().register_int_counter(world.metric_id, help.clone(), Some(labels.clone())).is_err());

        assert!(metrics::registry().register_counter_vec(world.metric_id, help.clone(), &label_ids, Some(labels.clone())).is_err());
        assert!(metrics::registry().register_int_counter_vec(world.metric_id, help.clone(), &label_ids, Some(labels.clone())).is_err());

        assert!(metrics::registry().register_gauge(world.metric_id, help.clone(), Some(labels.clone())).is_err());
        assert!(metrics::registry().register_int_gauge(world.metric_id, help.clone(), Some(labels.clone())).is_err());

        assert!(metrics::registry().register_gauge_vec(world.metric_id, help.clone(), &label_ids, Some(labels.clone())).is_err());
        assert!(metrics::registry().register_int_gauge_vec(world.metric_id, help.clone(), &label_ids, Some(labels.clone())).is_err());

        assert!(metrics::registry().register_histogram(world.metric_id, help.clone(), buckets.clone(), Some(labels.clone())).is_err());
        assert!(metrics::registry().register_histogram_timer(world.metric_id, help.clone(), vec![Duration::from_millis(1)].into(), Some(labels.clone())).is_err());
        assert!(metrics::registry().register_histogram_vec(world.metric_id, help.clone(), &label_ids, buckets.clone(), Some(labels.clone())).is_err());
    };

    // Scenario: [01D4B0KBWVFHEAVJSRD41TBJ6Z] Create a new Desc with a blank const label name
    then regex "01D4B0KBWVFHEAVJSRD41TBJ6Z" | world, _matches, _step| {
        let labels = hashmap! {
            " ".to_string() => "A".to_string()
        };

        assert!(prometheus::core::Desc::new(world.metric_id.name(), "help".to_string(), vec![], labels).is_err());
    };

    // Scenario: [01D4B0JCKY2ZQNXD0A0CQA89WK] Register a collector containing a descriptor with a blank label value
    then regex "01D4B0JCKY2ZQNXD0A0CQA89WK" | world, _matches, _step| {
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

        let labels = hashmap! {
            "A".to_string() => " ".to_string()
        };

        let foo = Foo{
            world: world.clone(),
            bad_desc: prometheus::core::Desc::new(world.metric_id.name(), "help".to_string(), vec![], labels).unwrap(),
            good_desc:  prometheus::core::Desc::new(metrics::MetricId::generate().name(), "help".to_string(), vec![], HashMap::new()).unwrap()
        };

        match metrics::registry().register(foo) {
            Err(err) => println!("{}", err),
            Ok(_) => panic!("should have failed to register")
        }
    };

    // Rule: descriptor label name max length is 30 and label value max length is 150

    // Scenario: [01D4ED3RW0MP6SRH0T169YSP0J] Register collector using the max allowed length for the const label name
    then regex "01D4ED3RW0MP6SRH0T169YSP0J" | world, _matches, _step | {
        struct Foo {
            world: World,
            desc: prometheus::core::Desc,
        };

        impl Collector for Foo {
            fn desc(&self) -> Vec<&Desc> {
                vec![&self.desc]
            }

            fn collect(&self) -> Vec<MetricFamily> {
                vec![]
            }
        }

        let label_id = metrics::LabelId::generate();
        let mut label_name = label_id.name();
        while label_name.len() <  metrics::MetricRegistry::DESC_LABEL_NAME_LEN {
            label_name.extend(label_id.to_string().chars());
        }
        let label_name = &label_name[..metrics::MetricRegistry::DESC_LABEL_NAME_LEN];
        assert_eq!(label_name.len(), metrics::MetricRegistry::DESC_LABEL_NAME_LEN);

        let labels = hashmap! {
            label_name.to_string() => "A".to_string()
        };

        let foo = Foo{
            world: world.clone(),
            desc: prometheus::core::Desc::new(metrics::MetricId::generate().name(), "help".to_string(), vec![], labels).unwrap(),
        };

        metrics::registry().register(foo).unwrap();
    };

    // Scenario: [01D4B0W77XVHM7BP2PJ5M33HK7] Register collector using the max allowed length for the const label value
    then regex "01D4B0W77XVHM7BP2PJ5M33HK7" | world, _matches, _step | {
        struct Foo {
            world: World,
            desc: prometheus::core::Desc,
        };

        impl Collector for Foo {
            fn desc(&self) -> Vec<&Desc> {
                vec![&self.desc]
            }

            fn collect(&self) -> Vec<MetricFamily> {
                vec![]
            }
        }

        let ulid = ULID::generate();
        let mut label_value = ulid.to_string();
        while label_value.len() <  metrics::MetricRegistry::DESC_LABEL_VALUE_LEN {
            label_value.extend(ulid.to_string().chars());
        }
        let label_value = &label_value[..metrics::MetricRegistry::DESC_LABEL_VALUE_LEN];
        assert_eq!(label_value.len(), metrics::MetricRegistry::DESC_LABEL_VALUE_LEN);

        let labels = hashmap! {
            "A".to_string() => label_value.to_string()
        };

        let foo = Foo{
            world: world.clone(),
            desc: prometheus::core::Desc::new(metrics::MetricId::generate().name(), "help".to_string(), vec![], labels).unwrap(),
        };

        metrics::registry().register(foo).unwrap();
    };

    // Scenario: [01D4ECRFSTXAW3RHQ0C2D6J2GZ] Register collector using the max allowed length for the variable label name
    then regex "01D4ECRFSTXAW3RHQ0C2D6J2GZ" | world, _matches, _step | {
        struct Foo {
            world: World,
            desc: prometheus::core::Desc,
        };

        impl Collector for Foo {
            fn desc(&self) -> Vec<&Desc> {
                vec![&self.desc]
            }

            fn collect(&self) -> Vec<MetricFamily> {
                vec![]
            }
        }

        let label_id = metrics::LabelId::generate();
        let mut label_name = label_id.name();
        while label_name.len() <  metrics::MetricRegistry::DESC_LABEL_NAME_LEN {
            label_name.extend(label_id.to_string().chars());
        }
        let label_name = &label_name[..metrics::MetricRegistry::DESC_LABEL_NAME_LEN];
        assert_eq!(label_name.len(), metrics::MetricRegistry::DESC_LABEL_NAME_LEN);

        let foo = Foo{
            world: world.clone(),
            desc:  prometheus::core::Desc::new(metrics::MetricId::generate().name(), "help".to_string(), vec![label_name.to_string()], HashMap::new()).unwrap()
        };

        metrics::registry().register(foo).unwrap();
    };

    // Scenario: [01D4B0XMQ2ZR2FHZHYM5KSBH90] Register metrics with a const label value whose length is 1 greater than the max length
    then regex "01D4B0XMQ2ZR2FHZHYM5KSBH90" | world, _matches, _step | {
        world.metric_id = metrics::MetricId::generate();

        let mut label_value = world.metric_id.to_string();
        while label_value.len() <  metrics::MetricRegistry::DESC_LABEL_VALUE_LEN + 1 {
            label_value.extend(world.metric_id.to_string().chars());
        }
        let label_value = &label_value[..metrics::MetricRegistry::DESC_LABEL_VALUE_LEN + 1];
        assert_eq!(label_value.len(), metrics::MetricRegistry::DESC_LABEL_VALUE_LEN + 1);

        let labels = hashmap! {
            metrics::LabelId::generate() => label_value.to_string()
        };

        let help = "help".to_string();

        let label_ids = vec![metrics::LabelId::generate()];
        let buckets = vec![0.1,0.2];

        assert!(metrics::registry().register_counter(world.metric_id, help.clone(), Some(labels.clone())).is_err());
        assert!(metrics::registry().register_int_counter(world.metric_id, help.clone(), Some(labels.clone())).is_err());

        assert!(metrics::registry().register_counter_vec(world.metric_id, help.clone(), &label_ids, Some(labels.clone())).is_err());
        assert!(metrics::registry().register_int_counter_vec(world.metric_id, help.clone(), &label_ids, Some(labels.clone())).is_err());

        assert!(metrics::registry().register_gauge(world.metric_id, help.clone(), Some(labels.clone())).is_err());
        assert!(metrics::registry().register_int_gauge(world.metric_id, help.clone(), Some(labels.clone())).is_err());

        assert!(metrics::registry().register_gauge_vec(world.metric_id, help.clone(), &label_ids, Some(labels.clone())).is_err());
        assert!(metrics::registry().register_int_gauge_vec(world.metric_id, help.clone(), &label_ids, Some(labels.clone())).is_err());

        assert!(metrics::registry().register_histogram(world.metric_id, help.clone(), buckets.clone(), Some(labels.clone())).is_err());
        assert!(metrics::registry().register_histogram_timer(world.metric_id, help.clone(), vec![Duration::from_millis(1)].into(), Some(labels.clone())).is_err());
        assert!(metrics::registry().register_histogram_vec(world.metric_id, help.clone(), &label_ids, buckets.clone(), Some(labels.clone())).is_err());
    };

    // Scenario: [01D4B1XP3V78X2HG3Z8NA1H0KH] Register collector with a variable name whose length is 1 greater than the max length
    then regex "01D4B1XP3V78X2HG3Z8NA1H0KH" | world, _matches, _step | {
        struct Foo {
            world: World,
            desc: prometheus::core::Desc,
        };

        impl Collector for Foo {
            fn desc(&self) -> Vec<&Desc> {
                vec![&self.desc]
            }

            fn collect(&self) -> Vec<MetricFamily> {
                vec![]
            }
        }

        let label_id = metrics::LabelId::generate();
        let mut label_name = label_id.name();
        while label_name.len() <  metrics::MetricRegistry::DESC_LABEL_NAME_LEN + 1 {
            label_name.extend(label_id.to_string().chars());
        }
        let label_name = &label_name[..metrics::MetricRegistry::DESC_LABEL_NAME_LEN + 1];
        assert_eq!(label_name.len(), metrics::MetricRegistry::DESC_LABEL_NAME_LEN + 1);

        let foo = Foo{
            world: world.clone(),
            desc:  prometheus::core::Desc::new(metrics::MetricId::generate().name(), "help".to_string(), vec![label_name.to_string()], HashMap::new()).unwrap()
        };

        match metrics::registry().register(foo) {
            Ok(_) => panic!("should have failed to register"),
            Err(err) => println!("{}", err)
        }
    };

    // Scenario: [01D4B0YGEN4XF275ZE660W1PRC] Register a collector containing a const label name whose length is 1 greater than the max length
    then regex "01D4B0YGEN4XF275ZE660W1PRC" | world, _matches, _step | {
        struct Foo {
            world: World,
            desc: prometheus::core::Desc,
        };

        impl Collector for Foo {
            fn desc(&self) -> Vec<&Desc> {
                vec![&self.desc]
            }

            fn collect(&self) -> Vec<MetricFamily> {
                vec![]
            }
        }

        let label_id = metrics::LabelId::generate();
        let mut label_name = label_id.name();
        while label_name.len() <  metrics::MetricRegistry::DESC_LABEL_NAME_LEN + 1{
            label_name.extend(label_id.to_string().chars());
        }
        let label_name = &label_name[..metrics::MetricRegistry::DESC_LABEL_NAME_LEN + 1];
        assert_eq!(label_name.len(), metrics::MetricRegistry::DESC_LABEL_NAME_LEN + 1);

        let labels = hashmap! {
            label_name.to_string() => "A".to_string()
        };

        let foo = Foo{
            world: world.clone(),
            desc:  prometheus::core::Desc::new(metrics::MetricId::generate().name(), "help".to_string(), vec![], labels).unwrap()
        };

        match metrics::registry().register(foo) {
            Ok(_) => panic!("should have failed to register"),
            Err(err) => println!("{}", err)
        }
    };

    // Scenario: [01D4B0Y6Y494DYFVE3YVQYXPPR] Register a collector containing a const label value whose length is 1 greater than the max length
    then regex "01D4B0Y6Y494DYFVE3YVQYXPPR" | world, _matches, _step | {
        struct Foo {
            world: World,
            desc: prometheus::core::Desc,
        };

        impl Collector for Foo {
            fn desc(&self) -> Vec<&Desc> {
                vec![&self.desc]
            }

            fn collect(&self) -> Vec<MetricFamily> {
                vec![]
            }
        }

        let label_id = metrics::LabelId::generate();
        let mut label_value = label_id.name();
        while label_value.len() <  metrics::MetricRegistry::DESC_LABEL_VALUE_LEN + 1{
            label_value.extend(label_id.to_string().chars());
        }
        let label_value = &label_value[..metrics::MetricRegistry::DESC_LABEL_VALUE_LEN + 1];
        assert_eq!(label_value.len(), metrics::MetricRegistry::DESC_LABEL_VALUE_LEN + 1);

        let labels = hashmap! {
            "A".to_string() => label_value.to_string()
        };

        let foo = Foo{
            world: world.clone(),
            desc:  prometheus::core::Desc::new(metrics::MetricId::generate().name(), "help".to_string(), vec![], labels).unwrap()
        };

        match metrics::registry().register(foo) {
            Ok(_) => panic!("should have failed to register"),
            Err(err) => println!("{}", err)
        }
    };

    // Rule: for metric vectors, at least 1 variable label must be defined on the descriptor

    // Scenario: [01D4B1F6AXH4DHBXC42756CVNZ] Register a metric vectors with no variable labels
    then regex "01D4B1F6AXH4DHBXC42756CVNZ" | _world, _matches, _step | {
        let metric_id = metrics::MetricId::generate();
        match metrics::registry().register_counter_vec(metric_id,"help", &vec![], None) {
            Ok(_) => panic!("should have failed to register"),
            Err(err) => println!("{}", err)
        }
        match metrics::registry().register_int_counter_vec(metric_id,"help", &vec![], None) {
            Ok(_) => panic!("should have failed to register"),
            Err(err) => println!("{}", err)
        }
        match metrics::registry().register_gauge_vec(metric_id,"help", &vec![], None) {
            Ok(_) => panic!("should have failed to register"),
            Err(err) => println!("{}", err)
        }
        match metrics::registry().register_int_gauge_vec(metric_id,"help", &vec![], None) {
            Ok(_) => panic!("should have failed to register"),
            Err(err) => println!("{}", err)
        }
        match metrics::registry().register_histogram_vec(metric_id,"help", &vec![],vec![0.1], None) {
            Ok(_) => panic!("should have failed to register"),
            Err(err) => println!("{}", err)
        }
    };

    // Rule: for metric vectors, variable labels must not be blank

    // Scenario: [01D4B1KQZ9F4FMKF51FHF84D72] Construct a Desc with blank variable labels
    then regex "01D4B1KQZ9F4FMKF51FHF84D72" | _world, _matches, _step | {
        // prometheus enforces this rule at the Desc level
        match prometheus::core::Desc::new("name".to_string(), "help".to_string(), vec!["".to_string()],HashMap::new()) {
            Ok(_) => panic!("Desc constructor should have failed"),
            Err(err) => println!("{}", err)
        }
        match prometheus::core::Desc::new("name".to_string(), "help".to_string(), vec!["  ".to_string()],HashMap::new()) {
            Ok(_) => panic!("Desc constructor should have failed"),
            Err(err) => println!("{}", err)
        }
    };

    // Rule: for metric vectors, variable labels must be unique

    // Scenario: [01D4B1ZKJ821A86MX88PPS05RY] Register a metric vectors with duplicate labels
    then regex "01D4B1ZKJ821A86MX88PPS05RY" | _world, _matches, _step | {
        // prometheus enforces this rule at the Desc level
        match prometheus::core::Desc::new("name".to_string(), "help".to_string(), vec!["A".to_string(), "A".to_string()],HashMap::new()) {
            Ok(_) => panic!("Desc constructor should have failed"),
            Err(err) => println!("{}", err)
        }
    };

    // Feature: [01D3JB8ZGW3KJ3VT44VBCZM3HA] A process metrics Collector is automatically registered with the global metrics registry

    // Scenario: [01D3JB9B4NP8T1PQ2Q85HY25FQ] gathering all metrics
    then regex "01D3JB9B4NP8T1PQ2Q85HY25FQ" | _world, _matches, _step | {
        let metric_families = metrics::registry().gather();
        assert!(metrics::ProcessMetrics::METRIC_NAMES.iter().all(|name| metric_families.iter().any(|mf| mf.get_name() == *name)));
    };

    // Scenario: [01D4FTH1WN3WWZZZH2HN66Y1YK] All metrics descriptors are retrieved
    then regex "01D4FTH1WN3WWZZZH2HN66Y1YK" | _world, _matches, _step | {
        let descs = metrics::registry().find_descs(|desc| metrics::ProcessMetrics::METRIC_NAMES.iter().any(|name| desc.fq_name.as_str() == *name));
        println!("{:#?}", descs);
        assert_eq!(descs.len(), metrics::ProcessMetrics::METRIC_NAMES.len());
        assert!(metrics::ProcessMetrics::METRIC_NAMES.iter().all(|name| descs.iter().any(|desc| desc.fq_name.as_str() == *name)));
    };

    // Scenario: [01D3JBCE21WYX6VMWCM4GW2ZTE] gathering process metrics
    then regex "01D3JBCE21WYX6VMWCM4GW2ZTE" | _world, _matches, _step | {
        let process_metrics = metrics::registry().gather_process_metrics();
        let metric_families = metrics::registry().gather_for_desc_names(&metrics::ProcessMetrics::METRIC_NAMES);
        println!("{}", serde_json::to_string_pretty(&process_metrics).unwrap());
        println!("{:#?}", metric_families);

        // based on timing, the numbers might not exactly match - the comparisons may need to be adjusted to be approximate
        let process_cpu_seconds_total = metric_families.iter().find(|mf| mf.get_name() == metrics::ProcessMetrics::PROCESS_CPU_SECONDS_TOTAL)
            .unwrap()
            .get_metric().iter().next().unwrap()
            .get_counter()
            .get_value();
        assert_eq!(process_metrics.cpu_seconds_total() as u64, process_cpu_seconds_total as u64);
        let process_open_fds = metric_families.iter().find(|mf| mf.get_name() == metrics::ProcessMetrics::PROCESS_OPEN_FDS)
            .unwrap()
            .get_metric().iter().next().unwrap()
            .get_gauge()
            .get_value();
        assert_eq!(process_metrics.open_fds() as u64, process_open_fds as u64);
        let process_max_fds = metric_families.iter().find(|mf| mf.get_name() == metrics::ProcessMetrics::PROCESS_MAX_FDS)
            .unwrap()
            .get_metric().iter().next().unwrap()
            .get_gauge()
            .get_value();
        assert_eq!(process_metrics.max_fds() as u64, process_max_fds as u64);
        let process_virtual_memory_bytes = metric_families.iter().find(|mf| mf.get_name() == metrics::ProcessMetrics::PROCESS_VIRTUAL_MEMORY_BYTES)
            .unwrap()
            .get_metric().iter().next().unwrap()
            .get_gauge()
            .get_value();
        assert_eq!(process_metrics.virtual_memory_bytes() as u64, process_virtual_memory_bytes as u64);
        let process_resident_memory_bytes = metric_families.iter().find(|mf| mf.get_name() == metrics::ProcessMetrics::PROCESS_RESIDENT_MEMORY_BYTES)
            .unwrap()
            .get_metric().iter().next().unwrap()
            .get_gauge()
            .get_value();
        assert_eq!(process_metrics.resident_memory_bytes() as u64, process_resident_memory_bytes as u64);
        let process_start_time_seconds = metric_families.iter().find(|mf| mf.get_name() == metrics::ProcessMetrics::PROCESS_START_TIME_SECONDS)
            .unwrap()
            .get_metric().iter().next().unwrap()
            .get_gauge()
            .get_value();
        assert_eq!(process_metrics.start_time_seconds() as u64, process_start_time_seconds as u64);
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
