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
use oysterpack_trust::metrics;
use oysterpack_uid::ULID;
use prometheus::core::Collector;
use std::{collections::HashMap, sync::Arc, time::Duration};
use maplit::*;

steps!(crate::TestContext => {

    given regex "01D3J3D7PA4NR9JABZWT635S6B-1" |world, _matches, _step| {
        world.init();
        world.spawn_command_handlers();
    };

    when regex "01D3J3D7PA4NR9JABZWT635S6B-2" |world, _matches, _step| {
        send_register_metric_command(world);
    };

    then regex "01D3J3D7PA4NR9JABZWT635S6B-3" |world, _matches, _step| {
        send_check_metric_command(world);
        world.stop_command_handlers();
    };

    given regex "01D3J3DRS0CJ2YN99KAWQ19103-1" |world, _matches, _step| {
        world.init();
        register_metric(world);
    };

    when regex "01D3J3DRS0CJ2YN99KAWQ19103-2" |world, _matches, _step| {
        register_duplicate_metric(world);
    };

    then regex "01D3J3DRS0CJ2YN99KAWQ19103-3" |world, _matches, _step| {
        check_metric_id_desc_count(world, 1);
    };

    given regex "01D3MT4JY1NZH2WW0347B9ZAS7-1" |world, _matches, _step| {
        world.init();
        register_counter_with_const_labels(world)
    };

    when regex "01D3MT4JY1NZH2WW0347B9ZAS7-2" |world, _matches, _step| {
        register_gauge_with_dup_desc(world);
    };

    then regex "01D3MT4JY1NZH2WW0347B9ZAS7-3" |world, _matches, _step| {
        check_metric_id_desc_count(world, 1);
    };

    given regex "01D3MT8KDP434DKZ6Y54C80BB0-1" |world, _matches, _step| {
        world.init();
        register_counter_with_const_labels(world)
    };

    when regex "01D3MT8KDP434DKZ6Y54C80BB0-2" |world, _matches, _step| {
        register_gauge_with_different_const_label_values(world);
    };

    then regex "01D3MT8KDP434DKZ6Y54C80BB0-3" |world, _matches, _step| {
        check_metric_id_desc_count(world, 2);
    };

    given regex "01D3PB6MDJ85MWP3SQ1H94S6R7-1" |world, _matches, step| {
        register_metrics(world, step);
    };

    then regex "01D3PB6MDJ85MWP3SQ1H94S6R7-2" |world, _matches, _step| {
        check_metric_names_are_metric_ids(world);
    };

    then regex "01D3PB6MDJ85MWP3SQ1H94S6R7-3" |world, _matches, _step| {
        check_label_names_are_label_ids(world);
    };

    given regex "01D3J441N6BM05NKCBQEVYTZY8" |world, _matches, step| {
        register_metrics(world, step);
    };

});

fn send_register_metric_command(world: &mut crate::TestContext) {
    for sender in world.command_sender.iter() {
        let (tx, rx) = crossbeam::channel::unbounded();
        sender.send(crate::Command::RegisterMetrics(tx));
        let metric_id = rx.recv().unwrap();
        world.metric_id = Some(metric_id);
    }
}

fn send_check_metric_command(world: &mut crate::TestContext) {
    for metric_id in world.metric_id.iter().cloned() {
        for sender in world.command_sender.iter() {
            let (tx, rx) = crossbeam::channel::unbounded();
            sender.send(crate::Command::CheckMetric(metric_id, tx));
            rx.recv().unwrap();
        }
    }
}

fn check_metric_names_are_metric_ids(world: &mut crate::TestContext) {
    let registry = metrics::registry();
    for metrics in world.metrics.iter() {
        let metric_ids = metrics.keys().cloned().collect::<Vec<_>>();
        // MetricId alone is not the unique identifier for a metric
        // - thus multiple collectors may have descs with the same MetricId
        assert!(registry.collectors_for_metric_ids(&metric_ids).len() >= metric_ids.len());
        for metric_id in metrics.keys().cloned() {
            let metric_name = metric_id.name();
            let metric_name = metric_name.as_str();
            assert!(!registry.filter_descs(|desc| desc.fq_name == metric_name).is_empty());
            // ensure collectors can be looked via MetricId
            assert!(!registry.collectors_for_metric_id(metric_id).is_empty());
        }
    }
}

fn check_label_names_are_label_ids(world: &mut crate::TestContext) {
    let registry = metrics::registry();
    for metrics in world.metrics.iter() {
        for metric_id in metrics.keys() {
            let metric_name = metric_id.name();
            let metric_name = metric_name.as_str();
            let all_label_names_can_be_parsed_into_label_ids = registry.filter_descs(|desc| !desc.const_label_pairs.is_empty() &&  desc.fq_name == metric_name)
                .iter()
                .all(|desc| desc.const_label_pairs.iter().all(|label_pair| {
                    label_pair.get_name().parse::<metrics::LabelId>().is_ok()
                }));
            assert!(all_label_names_can_be_parsed_into_label_ids);
        }
    }
}

fn register_counter_with_const_labels(world: &mut crate::TestContext) {
    let metric_id = metrics::MetricId::generate();
    let mut labels = HashMap::new();
    labels.insert(metrics::LabelId::generate(), "A".to_string());
    let _counter = metrics::registry()
        .register_counter(metric_id, "counter", Some(labels))
        .unwrap();
    world.metric_id = Some(metric_id);
}

fn register_gauge_with_dup_desc(world: &mut crate::TestContext) {
    let metric_id = world.metric_id.unwrap();
    let desc = metrics::registry().filter_descs(|desc| desc.fq_name == metric_id.name().as_str());
    let desc = desc.first().unwrap();
    let labels = desc
        .const_label_pairs
        .iter()
        .fold(HashMap::new(), |mut map, label_pair| {
            map.insert(
                label_pair.get_name().parse::<metrics::LabelId>().unwrap(),
                label_pair.get_value().to_string(),
            );
            map
        });
    assert!(metrics::registry()
        .register_gauge(metric_id, desc.help.as_str(), Some(labels))
        .is_err());
}

fn register_gauge_with_different_const_label_values(world: &mut crate::TestContext) {
    let metric_id = world.metric_id.unwrap();
    let desc = metrics::registry().filter_descs(|desc| desc.fq_name == metric_id.name().as_str());
    let desc = desc.first().unwrap();
    let labels = desc
        .const_label_pairs
        .iter()
        .fold(HashMap::new(), |mut map, label_pair| {
            map.insert(
                label_pair.get_name().parse::<metrics::LabelId>().unwrap(),
                ULID::generate().to_string(),
            );
            map
        });

    if let Err(err) =
        metrics::registry().register_gauge(metric_id, desc.help.as_str(), Some(labels))
    {
        panic!("{}", err);
    }
}

fn check_metric_id_desc_count(world: &mut crate::TestContext, expected_count: usize) {
    match world.metric_id {
        Some(metric_id) => {
            let name = metric_id.name();
            assert_eq!(
                metrics::registry()
                    .filter_descs(|desc| desc.fq_name == name.as_str())
                    .len(),
                expected_count
            )
        }
        None => panic!("world.metric_id is required"),
    }
}

fn register_metrics(world: &mut crate::TestContext, step: &gherkin::Step) {
    let mut metrics = HashMap::<metrics::MetricId, Arc<dyn prometheus::core::Collector>>::new();
    for ref tables in step.table.as_ref() {
        for row in tables.rows.iter() {
            match row[0].as_str() {
                "IntCounter" => {
                    let metric_id = metrics::MetricId::generate();
                    let counter = metrics::registry()
                        .register_int_counter(metric_id, "IntCounter", Some(hashmap!{
                            metrics::LabelId::generate() => ULID::generate().to_string(),
                            metrics::LabelId::generate() => ULID::generate().to_string(),
                        }))
                        .unwrap();
                    counter.inc();
                    metrics.insert(metric_id, Arc::new(counter));
                }
                "Counter" => {
                    let metric_id = metrics::MetricId::generate();
                    let counter = metrics::registry()
                        .register_counter(metric_id, "Counter", Some(hashmap!{
                            metrics::LabelId::generate() => ULID::generate().to_string(),
                            metrics::LabelId::generate() => ULID::generate().to_string(),
                        }))
                        .unwrap();
                    counter.inc();
                    metrics.insert(metric_id, Arc::new(counter));
                }
                "CounterVec" => {
                    let metric_id = metrics::MetricId::generate();
                    let label_id = metrics::LabelId::generate();
                    let counter = metrics::registry()
                        .register_counter_vec(metric_id, "CounterVec", &[label_id], Some(hashmap!{
                            metrics::LabelId::generate() => ULID::generate().to_string(),
                            metrics::LabelId::generate() => ULID::generate().to_string(),
                        }))
                        .unwrap();
                    let counter = counter.with_label_values(&["A"]);
                    counter.inc();
                    metrics.insert(metric_id, Arc::new(counter));
                }
                "IntGauge" => {
                    let metric_id = metrics::MetricId::generate();
                    let gauge = metrics::registry()
                        .register_int_gauge(metric_id, "IntGauge", Some(hashmap!{
                            metrics::LabelId::generate() => ULID::generate().to_string(),
                            metrics::LabelId::generate() => ULID::generate().to_string(),
                        }))
                        .unwrap();
                    gauge.inc();
                    metrics.insert(metric_id, Arc::new(gauge));
                }
                "Gauge" => {
                    let metric_id = metrics::MetricId::generate();
                    let gauge = metrics::registry()
                        .register_int_gauge(metric_id, "Gauge", Some(hashmap!{
                            metrics::LabelId::generate() => ULID::generate().to_string(),
                            metrics::LabelId::generate() => ULID::generate().to_string(),
                        }))
                        .unwrap();
                    gauge.inc();
                    metrics.insert(metric_id, Arc::new(gauge));
                }
                "GaugeVec" => {
                    let metric_id = metrics::MetricId::generate();
                    let label_id = metrics::LabelId::generate();
                    let gauge = metrics::registry()
                        .register_gauge_vec(metric_id, "GaugeVec", &[label_id], Some(hashmap!{
                            metrics::LabelId::generate() => ULID::generate().to_string(),
                            metrics::LabelId::generate() => ULID::generate().to_string(),
                        }))
                        .unwrap();
                    let gauge = gauge.with_label_values(&["A"]);
                    gauge.inc();
                    metrics.insert(metric_id, Arc::new(gauge));
                }
                "Histogram" => {
                    let metric_id = metrics::MetricId::generate();
                    let histogram = metrics::registry()
                        .register_histogram(metric_id, "Histogram", vec![0.1, 0.5, 1.0], Some(hashmap!{
                            metrics::LabelId::generate() => ULID::generate().to_string(),
                            metrics::LabelId::generate() => ULID::generate().to_string(),
                        }))
                        .unwrap();
                    histogram.observe(0.001);
                    metrics.insert(metric_id, Arc::new(histogram));
                }
                "HistogramTimer" => {
                    let metric_id = metrics::MetricId::generate();
                    let histogram = metrics::registry()
                        .register_histogram_timer(
                            metric_id,
                            "HistogramTimer",
                            metrics::TimerBuckets::from(vec![
                                Duration::from_millis(50),
                                Duration::from_millis(100),
                                Duration::from_millis(500),
                            ]),
                            Some(hashmap!{
                            metrics::LabelId::generate() => ULID::generate().to_string(),
                            metrics::LabelId::generate() => ULID::generate().to_string(),
                        }),
                        )
                        .unwrap();
                    histogram.observe(0.001);
                    metrics.insert(metric_id, Arc::new(histogram));
                }
                "HistogramVec" => {
                    let metric_id = metrics::MetricId::generate();
                    let label_id = metrics::LabelId::generate();
                    let histogram = metrics::registry()
                        .register_histogram_vec(
                            metric_id,
                            "HistogramVec",
                            &[label_id],
                            vec![0.1, 0.5, 1.0],
                            Some(hashmap!{
                            metrics::LabelId::generate() => ULID::generate().to_string(),
                            metrics::LabelId::generate() => ULID::generate().to_string(),
                        }),
                        )
                        .unwrap();
                    let histogram = histogram.with_label_values(&[label_id.name().as_str()]);
                    histogram.observe(0.001);
                    metrics.insert(metric_id, Arc::new(histogram));
                }
                _ => panic!("unsupported metric type: {}", row[0]),
            }
        }
    }
    world.metrics = Some(metrics);
}

fn register_metric(world: &mut crate::TestContext) {
    let metric_id = metrics::MetricId::generate();
    world.metric_id = Some(metric_id);
    let counter = metrics::registry()
        .register_int_counter(metric_id, "counter", None)
        .unwrap();
    counter.inc();
}

fn register_duplicate_metric(world: &mut crate::TestContext) {
    for metric_id in world.metric_id {
        assert!(metrics::registry()
            .register_int_counter(metric_id, "counter", None)
            .is_err());
    }
}

fn get_metric_descs(world: &mut crate::TestContext) {
    assert!(world.metrics.is_some());
    let descs = metrics::registry().descs();
    for metrics in world.metrics.as_ref() {
        for metric_id in metrics.keys() {
            metrics::registry()
                .filter_descs(|desc| desc.fq_name == metric_id.name())
                .first()
                .unwrap();
            assert!(descs.iter().any(|desc| desc.fq_name == metric_id.name()));
        }
    }
}

fn gather_metrics(world: &mut crate::TestContext) {
    let metric_families = metrics::registry().gather();
    assert!(!metric_families.is_empty());
    assert!(world.metrics.is_some());
    for metrics in world.metrics.iter() {
        assert!(!metrics.is_empty());
        assert!(metric_families.len() >= metrics.len());
        for metric_id in metrics.keys() {
            let metric_name = metric_id.name();
            assert!(metric_families
                .iter()
                .any(|mf| mf.get_name() == metric_name.as_str()));
        }
    }
}

fn gather_metrics_using_desc_ids(world: &mut crate::TestContext) {
    let metric_families = metrics::registry().gather();
    assert!(!metric_families.is_empty());
    assert!(world.metrics.is_some());
    let registry = metrics::registry();
    for metrics in world.metrics.iter() {
        assert!(!metrics.is_empty());
        assert!(metric_families.len() >= metrics.len());
        for metric in metrics.values() {
            for desc in metric.desc() {
                let metric_families = registry.gather_metrics(&[desc.id]);
                assert_eq!(metric_families.len(), 1);
                assert_eq!(
                    metric_families.first().unwrap().get_name(),
                    desc.fq_name.as_str()
                );
            }
        }
    }
}

fn gather_metrics_by_name(world: &mut crate::TestContext) {
    let metric_families = metrics::registry().gather();
    assert!(!metric_families.is_empty());
    assert!(world.metrics.is_some());
    let registry = metrics::registry();
    for metrics in world.metrics.iter() {
        assert!(!metrics.is_empty());
        assert!(metric_families.len() >= metrics.len());
        for metric in metrics.values() {
            for desc in metric.desc() {
                let metric_families = registry.gather_metrics_by_name(&[desc.fq_name.as_str()]);
                assert_eq!(metric_families.len(), 1);
                assert_eq!(
                    metric_families.first().unwrap().get_name(),
                    desc.fq_name.as_str()
                );
            }
        }
    }
}

fn check_collectors_contain_metrics(world: &mut crate::TestContext) {
    let collectors = metrics::registry().collectors();
    for metrics in world.metrics.iter() {
        for registered_collector in metrics.values() {
            let registered_collector_descs = registered_collector.desc();
            // check if there is a collector with matching descs
            assert!(collectors.iter().any(|collector| {
                let descs = collector.desc();
                let matching_desc_count = descs
                    .iter()
                    .filter(|desc1| {
                        registered_collector_descs
                            .iter()
                            .any(|desc2| desc1.id == desc2.id)
                    })
                    .count();
                descs.len() == matching_desc_count
            }));
        }
    }
}

fn check_metric_family_count() {
    assert_eq!(
        metrics::registry().metric_family_count(),
        metrics::registry().gather().len()
    );
}
