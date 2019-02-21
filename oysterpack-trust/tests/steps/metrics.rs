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

//TODO: refactor to make it cleaner

pub mod collectors;
pub mod descriptors;

use cucumber_rust::*;
use maplit::*;
use oysterpack_trust::metrics::{self, *};
use oysterpack_uid::ULID;
use prometheus::core::Collector;
use std::io::BufWriter;
use std::{collections::HashMap, sync::Arc, thread, time::Duration};

steps!(TestContext => {
    given regex "01D3J441N6BM05NKCBQEVYTZY8" |world, _matches, step| {
        world.init();
        register_metrics(world, step);
    };

    when regex "01D3PPPT1ZNXPKKWM29R14V5ZT-2" |world, _matches, _step| {
        gather_all_metrics(world);
    };

    then regex "01D3PPPT1ZNXPKKWM29R14V5ZT-3" |world, _matches, _step| {
        check_metric_families_returned_for_registered_descs(world);
    };

    when regex "01D3PPY3E710BYY8DQDKVQ31KY-2" |world, _matches, _step| {
        gather_metrics_using_desc_ids(world);
    };

    then regex "01D3PPY3E710BYY8DQDKVQ31KY-3" |world, _matches, _step| {
        check_metric_returned_for_specified_desc_ids(world);
    };

    when regex "01D3PQ2KMBY07K48Q281SMPED6-2" |world, _matches, _step| {
        gather_metrics_by_name(world);
    };

    then regex "01D3PQ2KMBY07K48Q281SMPED6-3" |world, _matches, _step| {
        check_metric_returned_for_specified_desc_fq_names(world);
    };

    given regex "01D3PQBDWM4BAJQKXF9R0MQED7" |world, _matches, step| {
        world.init();
        register_metrics(world, step);
    };

    when regex "01D3PSPRYX7XHSGX0JFC8TT59H-2" |world, _matches, _step| {
        get_all_metric_collectors(world);
    };

    then regex "01D3PSPRYX7XHSGX0JFC8TT59H-3" |world, _matches, _step| {
        check_collector_descs_match_filter(world);
    };

    when regex "01D3PX3BGCMV2PS6FDXHH0ZEB1-2" |world, _matches, _step| {
        get_some_metric_collectors(world);
    };

    then regex "01D3PX3BGCMV2PS6FDXHH0ZEB1-3" |world, _matches, _step| {
        check_collector_descs_match_filter(world);
    };

    when regex "01D3PX3NRADQPMS95EB5C7ECD7-2" |world, _matches, _step| {
        get_metric_collectors_for_metric_ids(world);
    };

    then regex "01D3PX3NRADQPMS95EB5C7ECD7-3" |world, _matches, _step| {
        check_collector_descs_match_metric_ids(world);
    };

    when regex "01D3JAKE384RJA4FM9NJJNDPV6-1" |world, _matches, _step| {
        world.init();
        register_collector(world);
    };

    then regex "01D3JAKE384RJA4FM9NJJNDPV6-2" |world, _matches, _step| {
        check_collector_descs(world);
    };

    then regex "01D3JAKE384RJA4FM9NJJNDPV6-3" |world, _matches, _step| {
        check_collector_is_gathered(world);
    };

    then regex "01D3JAKE384RJA4FM9NJJNDPV6-4" |world, _matches, _step| {
        check_collector_is_registered(world);
    };

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
        world.init();
        register_metrics(world, step);
    };

    when regex "01D3SF3R0DTBTVRKC9PFHQEEM9-2" |world, _matches, _step| {
        // gather all descs
        world.descs = Some(metrics::registry().descs());
    };

    then regex "01D3SF3R0DTBTVRKC9PFHQEEM9-3" |world, _matches, _step| {
        check_metrics_gathered_for_all_descs(world);
    };

    when regex "01D3PSPCNHH6CSW08RTFKZZ8SP-2" |world, _matches, _step| {
        gather_descs_with_filter(world);
    };

    then regex "01D3PSPCNHH6CSW08RTFKZZ8SP-3" |world, _matches, _step| {
        check_descs_returned_match_filter(world);
    };

    when regex "01D3PSP4TQK6ESKSB6AEFWAAYF-2" |world, _matches, _step| {
        gather_descs_for_metric_ids(world);
    };

    then regex "01D3PSP4TQK6ESKSB6AEFWAAYF-3" |world, _matches, _step| {
        check_descs_returned_match_metric_ids(world);
    };

    given regex "01D3M9ZJQSTWFFMKBR3Z2DXJ9N-1" |world, _matches, step| {
        world.init();
        register_metrics(world,step);
    };

    when regex "01D3M9ZJQSTWFFMKBR3Z2DXJ9N-2" |world, _matches, _step| {
        gather_all_metrics(world);
    };

    then regex "01D3M9ZJQSTWFFMKBR3Z2DXJ9N-3" |world, _matches, _step| {
        check_text_encoded_metrics(world);
    };

    given regex "01D3JB9B4NP8T1PQ2Q85HY25FQ-1" |_world, _matches, _step| {
        // prometheus' ProcessCollector is automatically registered with the global metric registry
    };

    when regex "01D3JB9B4NP8T1PQ2Q85HY25FQ-2" |world, _matches, _step| {
        gather_all_metrics(world);
    };

    then regex "01D3JB9B4NP8T1PQ2Q85HY25FQ-3" |world, _matches, _step| {
        check_process_metrics_gathered(world);
    };

    when regex "01D3JB9B4NP8T1PQ2Q85HY25FQ-4" |world, _matches, _step| {
        gather_process_metric_descs(world);
    };

    then regex "01D3JB9B4NP8T1PQ2Q85HY25FQ-5" |world, _matches, _step| {
        check_process_metric_descs_gathered(world);
    };

    when regex "01D3JBCE21WYX6VMWCM4GW2ZTE-2" |world, _matches, _step| {
        gather_process_metrics(world)
    };

    then regex "01D3JBCE21WYX6VMWCM4GW2ZTE-3" |world, _matches, _step| {
        check_process_metrics_gathered(world);
    };

    when regex "01D3VC85Q8MVBJ543SHZ4RE9T2-2" |world, _matches, _step| {
        gather_metrics_for_metric_ids(world)
    };

    then regex "01D3VC85Q8MVBJ543SHZ4RE9T2-3" |world, _matches, _step| {
        check_metrics_gathered_for_metric_ids(world);
    };

    given regex "01D3VGSGCP9ZN9BX3BTB349FRJ-1" |world, _matches, _step| {
        world.init();
        register_metrics_collector(world);
    };

    when regex "01D3VGSGCP9ZN9BX3BTB349FRJ-2" |world, _matches, _step| {
        get_collector_descs(world);
    };

    then regex "01D3VGSGCP9ZN9BX3BTB349FRJ-3" |world, _matches, _step| {
        check_collector_descs(world);
    };

    when regex "01D3XX46RZ63QYR0AAWVBCHWGP-1" |_world, _matches, _step| {
        // when a function that sleeps for 1 ms is timed
    };

    then regex "01D3XX46RZ63QYR0AAWVBCHWGP-2" |_world, _matches, _step| {
        check_time_fn();
    };

    when regex "01D3XZ6GCY1ECSKMBC6870ZBS0-1" |_world, _matches, _step| {
        // when a function that sleeps for 1 ms is timed
    };

    then regex "01D3XZ6GCY1ECSKMBC6870ZBS0-2" |_world, _matches, _step| {
        check_time_with_result_fn();
    };

    when regex "01D43MQQ1H59ZGJ9G2AMEJB5RF-2" |world, _matches, _step| {
        world.init();
        gather_metrics_for_labels(world)
    };

    then regex "01D43MQQ1H59ZGJ9G2AMEJB5RF-3" |world, _matches, _step| {
        check_metrics_for_labels(world);
    };
});

fn check_time_with_result_fn() {
    let clock = quanta::Clock::new();
    let (time, _result) = metrics::time_with_result(&clock, || {
        thread::sleep(Duration::from_millis(1));
        true
    });
    let time = metrics::as_float_secs(time);
    println!("time = {} s", time);
    const SECS_1MS: f64 = 0.001;
    assert!(time >= SECS_1MS && time <= SECS_1MS * 1.1);
}

fn check_time_fn() {
    let clock = quanta::Clock::new();
    let time = metrics::time(&clock, || thread::sleep(Duration::from_millis(1)));
    let time = metrics::as_float_secs(time);
    println!("time = {} s", time);
    const SECS_1MS: f64 = 0.001;
    assert!(time >= SECS_1MS && time <= SECS_1MS * 1.2);
}

fn get_collector_descs(world: &mut TestContext) {
    if let Some(ref collector) = world.collector {
        world.descs = Some(collector.desc().iter().cloned().cloned().collect());
    }
}

fn register_metrics_collector(world: &mut TestContext) {
    world.collector = Some(metrics::registry().register(Metrics::default()).unwrap());
}

fn gather_metrics_for_metric_ids(world: &mut TestContext) {
    let metric_ids = world
        .metrics
        .iter()
        .flat_map(|map| map.keys())
        .cloned()
        .collect::<Vec<_>>();
    world.metric_families = Some(metrics::registry().gather_for_metric_ids(&metric_ids));
}

fn register_metrics_with_labels(world: &mut TestContext) {
    let mut metrics = HashMap::<metrics::MetricId, Arc<dyn prometheus::core::Collector>>::new();
    let metric_id = metrics::MetricId::generate();
    let metric = metrics::registry()
        .register_int_counter(
            metric_id,
            "IntCounter with label",
            Some(hashmap! {
                metrics::LabelId::generate() => "A".to_string(),
                metrics::LabelId::generate() => "B".to_string()
            }),
        )
        .unwrap();
    metrics.insert(metric_id, Arc::new(metric));
    let metric_id = metrics::MetricId::generate();
    let metric = metrics::registry()
        .register_int_counter_vec(
            metric_id,
            "IntCounterVec with label",
            &[metrics::LabelId::generate(), metrics::LabelId::generate()],
            Some(hashmap! {
                metrics::LabelId::generate() => "C".to_string(),
                metrics::LabelId::generate() => "D".to_string()
            }),
        )
        .unwrap();
    metrics.insert(metric_id, Arc::new(metric));
    world.metrics = Some(metrics);
}

fn gather_metrics_for_labels(world: &mut TestContext) {
    register_metrics_with_labels(world);

    let label_pairs = world
        .metrics
        .iter()
        .map(|metrics| {
            metrics.values().fold(HashMap::new(), |mut map, collector| {
                collector
                    .desc()
                    .iter()
                    .flat_map(|desc| desc.const_label_pairs.clone())
                    .for_each(|label_pair| {
                        map.insert(
                            label_pair.get_name().to_string(),
                            label_pair.get_value().to_string(),
                        );
                    });
                map
            })
        })
        .collect::<Vec<_>>();
    let label_pairs = label_pairs.first().unwrap();

    let mfs = metrics::registry().gather_for_labels(label_pairs.clone());
    println!("{:#?}", world.metric_families);
    assert_eq!(mfs.len(), 2);

    let label_pairs: HashMap<_, _> = label_pairs
        .iter()
        .filter(|(_k, v)| v.as_str() == "A".to_string())
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    let mfs = metrics::registry().gather_for_labels(label_pairs.clone());
    world.metric_families = Some(mfs);
}

fn check_metrics_for_labels(world: &mut TestContext) {
    for mfs in world.metric_families.iter() {
        assert_eq!(mfs.len(), 1);
        assert_eq!(mfs.first().unwrap().get_metric().len(), 1);
    }
}

fn check_metrics_gathered_for_metric_ids(world: &mut TestContext) {
    let metric_ids = world
        .metrics
        .iter()
        .flat_map(|map| map.keys())
        .collect::<Vec<_>>();
    if let Some(ref mfs) = world.metric_families {
        assert_eq!(metric_ids.len(), mfs.len());
        assert!(metric_ids
            .iter()
            .all(|metric_id| mfs.iter().any(|mf| mf.get_name() == metric_id.name())));
    }
    if let Some(ref descs) = world.descs {
        for metric_name in ProcessMetrics::METRIC_NAMES.iter() {
            assert!(descs.iter().any(|desc| desc.fq_name == *metric_name));
        }
    }
}

fn gather_process_metrics(world: &mut TestContext) {
    world.metric_families =
        Some(metrics::registry().gather_for_desc_names(&metrics::ProcessMetrics::METRIC_NAMES));
}

fn check_process_metric_descs_gathered(world: &mut TestContext) {
    if let Some(ref descs) = world.descs {
        for metric_name in ProcessMetrics::METRIC_NAMES.iter() {
            assert!(descs.iter().any(|desc| desc.fq_name == *metric_name));
        }
    }
}

fn gather_process_metric_descs(world: &mut TestContext) {
    world.descs = Some(metrics::registry().find_descs(|desc| {
        ProcessMetrics::METRIC_NAMES
            .iter()
            .any(|name| *name == desc.fq_name)
    }));
}

fn check_process_metrics_gathered(world: &mut TestContext) {
    if let Some(ref mfs) = world.metric_families {
        for metric_name in metrics::ProcessMetrics::METRIC_NAMES.iter() {
            assert!(mfs.iter().any(|mf| mf.get_name() == *metric_name));
        }
    }
    let process_metrics = metrics::registry().gather_process_metrics();

    let mfs = metrics::registry()
        .gather_for_desc_names(&[metrics::ProcessMetrics::PROCESS_CPU_SECONDS_TOTAL]);
    let value = mfs.first().unwrap().get_metric()[0]
        .get_counter()
        .get_value();
    assert!(process_metrics.cpu_seconds_total() <= value);

    let mfs =
        metrics::registry().gather_for_desc_names(&[metrics::ProcessMetrics::PROCESS_OPEN_FDS]);
    let value = mfs.first().unwrap().get_metric()[0].get_gauge().get_value();
    assert!(process_metrics.open_fds() <= value);

    let mfs =
        metrics::registry().gather_for_desc_names(&[metrics::ProcessMetrics::PROCESS_MAX_FDS]);
    let value = mfs.first().unwrap().get_metric()[0].get_gauge().get_value();
    assert!(process_metrics.max_fds() <= value);

    let mfs = metrics::registry()
        .gather_for_desc_names(&[metrics::ProcessMetrics::PROCESS_VIRTUAL_MEMORY_BYTES]);
    let value = mfs.first().unwrap().get_metric()[0].get_gauge().get_value();
    assert!(process_metrics.virtual_memory_bytes() <= value);

    let mfs = metrics::registry()
        .gather_for_desc_names(&[metrics::ProcessMetrics::PROCESS_RESIDENT_MEMORY_BYTES]);
    let value = mfs.first().unwrap().get_metric()[0].get_gauge().get_value();
    assert!(process_metrics.resident_memory_bytes() <= value);

    let mfs = metrics::registry()
        .gather_for_desc_names(&[metrics::ProcessMetrics::PROCESS_START_TIME_SECONDS]);
    let value = mfs.first().unwrap().get_metric()[0].get_gauge().get_value();
    assert!(process_metrics.start_time_seconds() <= value);
}

fn check_text_encoded_metrics(world: &mut TestContext) {
    if let Some(ref mfs) = world.metric_families {
        let mut buf = BufWriter::new(Vec::<u8>::with_capacity(2048));
        metrics::registry().text_encode_metrics(&mut buf).unwrap();
        let buf = buf.into_inner().unwrap();
        let text = std::str::from_utf8(&buf).unwrap();
        println!("{}", text);
        for mf in mfs {
            let re = regex::RegexBuilder::new(format!("^{}.+$", mf.get_name()).as_str())
                .multi_line(true)
                .build()
                .unwrap();
            assert!(re.is_match(text));
        }
    }
}

fn check_descs_returned_match_metric_ids(world: &mut TestContext) {
    let metric_ids = world
        .metrics
        .as_ref()
        .unwrap()
        .keys()
        .cloned()
        .collect::<Vec<_>>();
    match world.descs {
        Some(ref descs) => {
            for metric_id in metric_ids.iter() {
                assert!(descs.iter().any(|desc| desc.fq_name == metric_id.name()));
            }
            for desc in descs.iter() {
                assert!(metric_ids
                    .iter()
                    .any(|metric_id| metric_id.name() == desc.fq_name));
            }
        }
        None => panic!("no descs were found"),
    }
}

fn gather_descs_for_metric_ids(world: &mut TestContext) {
    let metric_ids = world
        .metrics
        .as_ref()
        .unwrap()
        .keys()
        .cloned()
        .collect::<Vec<_>>();
    world.descs = Some(metrics::registry().descs_for_metric_ids(metric_ids.as_slice()));
}

fn desc_matches(desc: &prometheus::core::Desc) -> bool {
    desc.help.contains("Histogram")
}

fn gather_descs_with_filter(world: &mut TestContext) {
    world.descs = Some(metrics::registry().find_descs(desc_matches));
}

fn check_descs_returned_match_filter(world: &mut TestContext) {
    match world.descs {
        Some(ref descs) => assert!(descs.iter().all(desc_matches)),
        None => panic!("descs should have been found"),
    }
}

fn check_metrics_gathered_for_all_descs(world: &mut TestContext) {
    if let Some(ref descs) = world.descs {
        let mfs = metrics::registry().gather();
        for mf in mfs.iter() {
            assert!(descs
                .iter()
                .any(|desc| mf.get_name() == desc.fq_name.as_str()));
        }
        // if no metrics have been observed for dimensions on a metric vec, then no metrics will be collected
        // i.e., there may be no metrics to ge gathered for metric vecs
        assert!(descs
            .iter()
            .filter(|desc| mfs.iter().all(|mf| mf.get_name() != desc.fq_name.as_str()))
            .all(|desc| !desc.variable_labels.is_empty()));
    }
}

fn check_metric_returned_for_specified_desc_fq_names(world: &mut TestContext) {
    if let (Some(descs), Some(mfs)) = (world.descs.as_ref(), world.metric_families.as_ref()) {
        for desc in descs.iter() {
            assert!(mfs.iter().any(|mf| mf.get_name() == desc.fq_name.as_str()));
            if !desc.const_label_pairs.is_empty() {
                assert!(mfs.iter().any(|mf| mf.get_name() == desc.fq_name.as_str()));
            }
        }
    }
}

fn check_metric_returned_for_specified_desc_ids(world: &mut TestContext) {
    if let (Some(descs), Some(mfs)) = (world.descs.as_ref(), world.metric_families.as_ref()) {
        for desc in descs.iter() {
            assert!(mfs.iter().any(|mf| mf.get_name() == desc.fq_name.as_str()));
            if !desc.const_label_pairs.is_empty() {
                let mf = mfs
                    .iter()
                    .find(|mf| mf.get_name() == desc.fq_name.as_str())
                    .unwrap();
                assert!(desc.const_label_pairs.iter().all(|label_pair| mf
                    .get_metric()
                    .iter()
                    .any(|metric| metric
                        .get_label()
                        .iter()
                        .any(|label_pair_2| label_pair_2 == label_pair))));
            }
        }
    }
}

fn check_metric_families_returned_for_registered_descs(world: &mut TestContext) {
    if let Some(ref mfs) = world.metric_families {
        let descs = metrics::registry().descs();
        assert_eq!(
            mfs.iter().map(|mf| mf.get_metric().len()).sum::<usize>(),
            descs.len()
        );
        for mf in mfs.iter() {
            assert!(descs
                .iter()
                .any(|desc| if desc.fq_name.as_str() == mf.get_name() {
                    if !desc.const_label_pairs.is_empty() {
                        mf.get_metric().iter().any(|metric| {
                            desc.const_label_pairs.iter().any(|label_pair| {
                                metric.get_label().iter().any(|label_pair_2| {
                                    label_pair_2.get_name() == label_pair.get_name()
                                        && label_pair_2.get_value() == label_pair.get_value()
                                })
                            })
                        })
                    } else {
                        true
                    }
                } else {
                    false
                }));
        }
    }
}

fn check_collector_descs_match_filter(world: &mut TestContext) {
    if let Some(ref collectors) = world.collectors {
        if let Some(ref descs) = world.descs {
            let collector_descs = collectors
                .iter()
                .flat_map(|collector| collector.desc())
                .collect::<Vec<_>>();
            assert_eq!(descs.len(), collector_descs.len());
            for desc in descs.iter() {
                assert!(collector_descs.iter().any(|desc2| desc2.id == desc.id));
            }
        }
    }
}

fn get_all_metric_collectors(world: &mut TestContext) {
    world.collectors = Some(metrics::registry().collectors());
    world.descs = Some(metrics::registry().descs());
}

fn get_some_metric_collectors(world: &mut TestContext) {
    let mut descs = metrics::registry().descs();
    let descs = descs.split_off(descs.len() / 2);
    world.collectors = Some(metrics::registry().find_collectors(|c| {
        c.desc()
            .iter()
            .any(|desc| descs.iter().any(|desc2| desc.id == desc2.id))
    }));
    world.descs = Some(descs);
}

fn get_metric_collectors_for_metric_ids(world: &mut TestContext) {
    if let Some(ref metrics) = world.metrics {
        let metric_ids = metrics.keys().cloned().collect::<Vec<_>>();
    }
}

fn check_collector_descs_match_metric_ids(world: &mut TestContext) {
    if let Some(ref collectors) = world.collectors {
        if let Some(ref metrics) = world.metrics {
            let metric_ids = metrics.keys().cloned().collect::<Vec<_>>();
            let collector_descs = collectors
                .iter()
                .flat_map(|collector| collector.desc())
                .collect::<Vec<_>>();
            assert_eq!(metric_ids.len(), collector_descs.len());
            for desc in collector_descs.iter() {
                assert!(metric_ids
                    .iter()
                    .any(|metric_id| metric_id.name() == desc.fq_name));
            }
        }
    }
}

fn check_collector_is_registered(world: &mut TestContext) {
    if let Some(ref collector) = world.collector {
        metrics::registry()
            .find_collectors(|registered_collector| {
                let registered_descs = registered_collector.desc();
                let descs = collector.desc();
                if registered_descs.len() == descs.len() {
                    registered_descs
                        .iter()
                        .all(|desc| descs.iter().any(|desc2| desc.id == desc2.id))
                } else {
                    false
                }
            })
            .first()
            .unwrap();
    }
}

fn check_collector_is_gathered(world: &mut TestContext) {
    if let Some(ref collector) = world.collector {
        let descs = collector.desc();
        let desc_ids = descs.iter().map(|desc| desc.id).collect::<Vec<_>>();
        assert_eq!(
            metrics::registry().gather_for_desc_ids(&desc_ids).len(),
            collector.desc().len()
        );
    }
}

fn check_collector_descs(world: &mut TestContext) {
    if let Some(ref collector) = world.collector {
        let desc_ids = collector
            .desc()
            .iter()
            .map(|desc| desc.id)
            .collect::<fnv::FnvHashSet<_>>();

        let expected_desc_count = desc_ids.len();
        let actual_desc_count = metrics::registry()
            .find_descs(|desc| desc_ids.contains(&desc.id))
            .len();
        assert_eq!(actual_desc_count, expected_desc_count);
    }
}

fn register_collector(world: &mut TestContext) {
    world.collector = Some(
        metrics::registry()
            .register(RequestMetrics::default())
            .unwrap(),
    );
}

fn send_register_metric_command(world: &mut TestContext) {
    if let Some(ref sender) = world.command_sender {
        let (tx, rx) = crossbeam::channel::unbounded();
        sender.send(Command::RegisterMetrics(tx)).unwrap();
        let metric_id = rx.recv().unwrap();
        world.metric_id = Some(metric_id);
    }
}

fn send_check_metric_command(world: &mut TestContext) {
    if let Some(metric_id) = world.metric_id {
        if let Some(ref sender) = world.command_sender {
            let (tx, rx) = crossbeam::channel::unbounded();
            sender.send(Command::CheckMetric(metric_id, tx)).unwrap();
            let _ = rx.recv().unwrap();
        }
    }
}

fn check_metric_names_are_metric_ids(world: &mut TestContext) {
    let registry = metrics::registry();
    if let Some(ref metrics) = world.metrics {
        let metric_ids = metrics.keys().cloned().collect::<Vec<_>>();
        // MetricId alone is not the unique identifier for a metric
        // - thus multiple collectors may have descs with the same MetricId
        assert!(registry.collectors_for_metric_ids(&metric_ids).len() >= metric_ids.len());
        for metric_id in metrics.keys().cloned() {
            let metric_name = metric_id.name();
            let metric_name = metric_name.as_str();
            assert!(!registry
                .find_descs(|desc| desc.fq_name == metric_name)
                .is_empty());
            // ensure collectors can be looked via MetricId
            assert!(!registry.collectors_for_metric_id(metric_id).is_empty());
        }
    }
}

fn check_label_names_are_label_ids(world: &mut TestContext) {
    let registry = metrics::registry();
    if let Some(ref metrics) = world.metrics {
        for metric_id in metrics.keys() {
            let metric_name = metric_id.name();
            let metric_name = metric_name.as_str();
            let all_label_names_can_be_parsed_into_label_ids = registry
                .find_descs(|desc| {
                    !desc.const_label_pairs.is_empty() && desc.fq_name == metric_name
                })
                .iter()
                .all(|desc| {
                    desc.const_label_pairs
                        .iter()
                        .all(|label_pair| label_pair.get_name().parse::<metrics::LabelId>().is_ok())
                });
            assert!(all_label_names_can_be_parsed_into_label_ids);
        }
    }
}

fn register_counter_with_const_labels(world: &mut TestContext) {
    let metric_id = metrics::MetricId::generate();
    let mut labels = HashMap::new();
    labels.insert(metrics::LabelId::generate(), "A".to_string());
    let _counter = metrics::registry()
        .register_counter(metric_id, "counter", Some(labels))
        .unwrap();
    world.metric_id = Some(metric_id);
}

fn register_gauge_with_dup_desc(world: &mut TestContext) {
    let metric_id = world.metric_id.unwrap();
    let desc = metrics::registry().find_descs(|desc| desc.fq_name == metric_id.name().as_str());
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

fn register_gauge_with_different_const_label_values(world: &mut TestContext) {
    let metric_id = world.metric_id.unwrap();
    let desc = metrics::registry().find_descs(|desc| desc.fq_name == metric_id.name().as_str());
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

fn check_metric_id_desc_count(world: &mut TestContext, expected_count: usize) {
    match world.metric_id {
        Some(metric_id) => {
            let name = metric_id.name();
            assert_eq!(
                metrics::registry()
                    .find_descs(|desc| desc.fq_name == name.as_str())
                    .len(),
                expected_count
            )
        }
        None => panic!("world.metric_id is required"),
    }
}

fn register_metrics(world: &mut TestContext, step: &gherkin::Step) {
    let mut metrics = HashMap::<metrics::MetricId, Arc<dyn prometheus::core::Collector>>::new();
    if let Some(ref tables) = step.table {
        for row in tables.rows.iter() {
            match row[0].as_str() {
                "IntCounter" => {
                    let metric_id = metrics::MetricId::generate();
                    let counter = metrics::registry()
                        .register_int_counter(
                            metric_id,
                            "IntCounter",
                            Some(hashmap! {
                                metrics::LabelId::generate() => ULID::generate().to_string(),
                                metrics::LabelId::generate() => ULID::generate().to_string(),
                            }),
                        )
                        .unwrap();
                    counter.inc();
                    metrics.insert(metric_id, Arc::new(counter));
                }
                "Counter" => {
                    let metric_id = metrics::MetricId::generate();
                    let counter = metrics::registry()
                        .register_counter(
                            metric_id,
                            "Counter",
                            Some(hashmap! {
                                metrics::LabelId::generate() => ULID::generate().to_string(),
                                metrics::LabelId::generate() => ULID::generate().to_string(),
                            }),
                        )
                        .unwrap();
                    counter.inc();
                    metrics.insert(metric_id, Arc::new(counter));
                }
                "CounterVec" => {
                    let metric_id = metrics::MetricId::generate();
                    let label_id = metrics::LabelId::generate();
                    let counter = metrics::registry()
                        .register_counter_vec(
                            metric_id,
                            "CounterVec",
                            &[label_id],
                            Some(hashmap! {
                                metrics::LabelId::generate() => ULID::generate().to_string(),
                                metrics::LabelId::generate() => ULID::generate().to_string(),
                            }),
                        )
                        .unwrap();
                    let counter = counter.with_label_values(&["A"]);
                    counter.inc();
                    metrics.insert(metric_id, Arc::new(counter));
                }
                "IntGauge" => {
                    let metric_id = metrics::MetricId::generate();
                    let gauge = metrics::registry()
                        .register_int_gauge(
                            metric_id,
                            "IntGauge",
                            Some(hashmap! {
                                metrics::LabelId::generate() => ULID::generate().to_string(),
                                metrics::LabelId::generate() => ULID::generate().to_string(),
                            }),
                        )
                        .unwrap();
                    gauge.inc();
                    metrics.insert(metric_id, Arc::new(gauge));
                }
                "Gauge" => {
                    let metric_id = metrics::MetricId::generate();
                    let gauge = metrics::registry()
                        .register_int_gauge(
                            metric_id,
                            "Gauge",
                            Some(hashmap! {
                                metrics::LabelId::generate() => ULID::generate().to_string(),
                                metrics::LabelId::generate() => ULID::generate().to_string(),
                            }),
                        )
                        .unwrap();
                    gauge.inc();
                    metrics.insert(metric_id, Arc::new(gauge));
                }
                "GaugeVec" => {
                    let metric_id = metrics::MetricId::generate();
                    let label_id = metrics::LabelId::generate();
                    let gauge = metrics::registry()
                        .register_gauge_vec(
                            metric_id,
                            "GaugeVec",
                            &[label_id],
                            Some(hashmap! {
                                metrics::LabelId::generate() => ULID::generate().to_string(),
                                metrics::LabelId::generate() => ULID::generate().to_string(),
                            }),
                        )
                        .unwrap();
                    let gauge = gauge.with_label_values(&["A"]);
                    gauge.inc();
                    metrics.insert(metric_id, Arc::new(gauge));
                }
                "Histogram" => {
                    let metric_id = metrics::MetricId::generate();
                    let histogram = metrics::registry()
                        .register_histogram(
                            metric_id,
                            "Histogram",
                            vec![0.1, 0.5, 1.0],
                            Some(hashmap! {
                                metrics::LabelId::generate() => ULID::generate().to_string(),
                                metrics::LabelId::generate() => ULID::generate().to_string(),
                            }),
                        )
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
                            Some(hashmap! {
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
                            Some(hashmap! {
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

fn register_metric(world: &mut TestContext) {
    let metric_id = metrics::MetricId::generate();
    world.metric_id = Some(metric_id);
    let counter = metrics::registry()
        .register_int_counter(metric_id, "counter", None)
        .unwrap();
    counter.inc();
}

fn register_duplicate_metric(world: &mut TestContext) {
    if let Some(metric_id) = world.metric_id {
        assert!(metrics::registry()
            .register_int_counter(metric_id, "counter", None)
            .is_err());
    }
}

fn gather_all_metrics(world: &mut TestContext) {
    world.metric_families = Some(metrics::registry().gather());
}

fn gather_metrics_using_desc_ids(world: &mut TestContext) {
    let mut descs = metrics::registry().descs();
    let descs = descs.split_off(descs.len() / 2);
    let desc_ids = descs.iter().map(|desc| desc.id).collect::<Vec<u64>>();
    world.metric_families = Some(metrics::registry().gather_for_desc_ids(desc_ids.as_slice()));
    world.descs = Some(descs);
}

fn gather_metrics_by_name(world: &mut TestContext) {
    let mut descs = metrics::registry().descs();
    let descs = descs.split_off(descs.len() / 2);
    let desc_names = descs
        .iter()
        .map(|desc| desc.fq_name.as_str())
        .collect::<Vec<_>>();
    world.metric_families = Some(metrics::registry().gather_for_desc_names(desc_names.as_slice()));
    world.descs = Some(descs);
}

#[derive(Default)]
pub struct TestContext {
    pub metric_id: Option<metrics::MetricId>,
    pub metrics: Option<HashMap<metrics::MetricId, Arc<dyn prometheus::core::Collector>>>,
    pub command_sender: Option<crossbeam::Sender<Command>>,
    pub collector: Option<metrics::ArcCollector>,
    pub collectors: Option<Vec<metrics::ArcCollector>>,
    pub descs: Option<Vec<prometheus::core::Desc>>,
    pub metric_families: Option<Vec<prometheus::proto::MetricFamily>>,
}

impl TestContext {
    fn init(&mut self) {
        self.metric_id = None;
        self.metrics = None;
        self.command_sender = None;
        self.collector = None;
        self.collectors = None;
        self.descs = None;
        self.metric_families = None;
    }

    fn spawn_command_handlers(&mut self) {
        let (tx, rx) = crossbeam::channel::bounded(0);
        self.command_sender = Some(tx.clone());
        for _ in 0..2 {
            let rx = rx.clone();
            thread::spawn(move || {
                for command in rx {
                    match command {
                        Command::RegisterMetrics(reply_chan) => {
                            let metric_id = metrics::MetricId::generate();
                            metrics::registry()
                                .register_counter(metric_id, "counter", None)
                                .unwrap();
                            reply_chan.send(metric_id).unwrap();
                        }
                        Command::CheckMetric(metric_id, reply_chan) => {
                            if metrics::registry()
                                .gather_for_desc_names(&[metric_id.name().as_str()])
                                .is_empty()
                            {
                                reply_chan.send(Err("no metrics gathered")).unwrap();
                                break;
                            }
                            if metrics::registry()
                                .descs_for_metric_id(metric_id)
                                .is_empty()
                            {
                                reply_chan.send(Err("no Desc(s) found")).unwrap();
                                break;
                            }
                            if metrics::registry()
                                .collectors_for_metric_id(metric_id)
                                .is_empty()
                            {
                                reply_chan.send(Err("no Collector(s) found")).unwrap();
                                break;
                            }

                            reply_chan.send(Ok(())).unwrap();
                        }
                        Command::Stop => break,
                    }
                }
            });
        }
    }

    fn stop_command_handlers(&mut self) {
        loop {
            for sender in self.command_sender.iter() {
                if sender.send(Command::Stop).is_err() {
                    return;
                }
            }
        }
    }
}

pub enum Command {
    RegisterMetrics(crossbeam::channel::Sender<metrics::MetricId>),
    CheckMetric(
        metrics::MetricId,
        crossbeam::channel::Sender<Result<(), &'static str>>,
    ),
    Stop,
}

impl cucumber_rust::World for TestContext {}

pub struct RequestMetrics {
    request_counter: prometheus::IntCounter,
    error_counter: prometheus::IntCounter,
}

impl RequestMetrics {
    pub const REQ_COUNTER_METRIC_ID: metrics::MetricId =
        metrics::MetricId(1874064177657531783668017676596473713);
    pub const ERR_COUNTER_METRIC_ID: metrics::MetricId =
        metrics::MetricId(1874064202949590498235699520354975202);
}

impl prometheus::core::Collector for RequestMetrics {
    fn desc(&self) -> Vec<&prometheus::core::Desc> {
        let mut descs = Vec::with_capacity(2);
        descs.extend(self.request_counter.desc());
        descs.extend(self.error_counter.desc());
        descs
    }

    fn collect(&self) -> Vec<prometheus::proto::MetricFamily> {
        let mut descs = Vec::with_capacity(2);
        descs.extend(self.request_counter.collect());
        descs.extend(self.error_counter.collect());
        descs
    }
}

impl Default for RequestMetrics {
    fn default() -> Self {
        Self {
            request_counter: metrics::new_int_counter::<fnv::FnvBuildHasher>(
                Self::REQ_COUNTER_METRIC_ID,
                "request counter",
                None,
            )
            .unwrap(),
            error_counter: metrics::new_int_counter::<fnv::FnvBuildHasher>(
                Self::ERR_COUNTER_METRIC_ID,
                "error counter",
                None,
            )
            .unwrap(),
        }
    }
}

#[derive(Clone)]
pub struct Metrics {
    counter: prometheus::Counter,
    int_counter: prometheus::IntCounter,
    counter_vec: prometheus::CounterVec,
    int_counter_vec: prometheus::IntCounterVec,

    gauge: prometheus::Gauge,
    int_gauge: prometheus::IntGauge,
    gauge_vec: prometheus::GaugeVec,
    int_gauge_vec: prometheus::IntGaugeVec,

    histogram: prometheus::Histogram,
    histogram_vec: prometheus::HistogramVec,
}

impl prometheus::core::Collector for Metrics {
    fn desc(&self) -> Vec<&prometheus::core::Desc> {
        let mut descs = Vec::with_capacity(10);
        descs.extend(self.counter.desc());
        descs.extend(self.int_counter.desc());
        descs.extend(self.counter_vec.desc());
        descs.extend(self.int_counter_vec.desc());

        descs.extend(self.gauge.desc());
        descs.extend(self.int_gauge.desc());
        descs.extend(self.gauge_vec.desc());
        descs.extend(self.int_gauge_vec.desc());

        descs.extend(self.histogram.desc());
        descs.extend(self.histogram_vec.desc());

        descs
    }

    fn collect(&self) -> Vec<prometheus::proto::MetricFamily> {
        let mut mfs = Vec::with_capacity(15);

        // simulates metrics being collected
        self.counter_vec.with_label_values(&["a"]).inc();
        self.int_counter_vec.with_label_values(&["a"]).inc();
        self.gauge_vec.with_label_values(&["a"]).inc();
        self.int_gauge_vec.with_label_values(&["a"]).inc();
        self.histogram_vec.with_label_values(&["a"]).observe(0.5);

        mfs.extend(self.counter.collect());
        mfs.extend(self.int_counter.collect());
        mfs.extend(self.counter_vec.collect());
        mfs.extend(self.int_counter_vec.collect());

        mfs.extend(self.gauge.collect());
        mfs.extend(self.int_gauge.collect());
        mfs.extend(self.gauge_vec.collect());
        mfs.extend(self.int_gauge_vec.collect());

        mfs.extend(self.histogram.collect());
        mfs.extend(self.histogram_vec.collect());

        mfs
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self {
            counter: metrics::new_counter(
                MetricId::generate(),
                "Metrics::counter",
                Some(hashmap! {
                    metrics::LabelId::generate() => ULID::generate().to_string(),
                    metrics::LabelId::generate() => ULID::generate().to_string(),
                }),
            )
            .unwrap(),
            int_counter: metrics::new_int_counter(
                MetricId::generate(),
                "Metrics::int_counter",
                Some(hashmap! {
                    metrics::LabelId::generate() => ULID::generate().to_string(),
                    metrics::LabelId::generate() => ULID::generate().to_string(),
                }),
            )
            .unwrap(),
            counter_vec: metrics::new_counter_vec(
                MetricId::generate(),
                "Metrics::counter_vec",
                &[LabelId::generate()],
                Some(hashmap! {
                    metrics::LabelId::generate() => ULID::generate().to_string(),
                    metrics::LabelId::generate() => ULID::generate().to_string(),
                }),
            )
            .unwrap(),
            int_counter_vec: metrics::new_int_counter_vec(
                MetricId::generate(),
                "Metrics::int_counter_vec",
                &[LabelId::generate()],
                Some(hashmap! {
                    metrics::LabelId::generate() => ULID::generate().to_string(),
                    metrics::LabelId::generate() => ULID::generate().to_string(),
                }),
            )
            .unwrap(),
            gauge: metrics::new_gauge(
                MetricId::generate(),
                "Metrics::gauge",
                Some(hashmap! {
                    metrics::LabelId::generate() => ULID::generate().to_string(),
                    metrics::LabelId::generate() => ULID::generate().to_string(),
                }),
            )
            .unwrap(),
            int_gauge: metrics::new_int_gauge(
                MetricId::generate(),
                "Metrics::int_gauge",
                Some(hashmap! {
                    metrics::LabelId::generate() => ULID::generate().to_string(),
                    metrics::LabelId::generate() => ULID::generate().to_string(),
                }),
            )
            .unwrap(),
            gauge_vec: metrics::new_gauge_vec(
                MetricId::generate(),
                "Metrics::gauge_vec",
                &[LabelId::generate()],
                Some(hashmap! {
                    metrics::LabelId::generate() => ULID::generate().to_string(),
                    metrics::LabelId::generate() => ULID::generate().to_string(),
                }),
            )
            .unwrap(),
            int_gauge_vec: metrics::new_int_gauge_vec(
                MetricId::generate(),
                "Metrics::int_gauge_vec",
                &[LabelId::generate()],
                Some(hashmap! {
                    metrics::LabelId::generate() => ULID::generate().to_string(),
                    metrics::LabelId::generate() => ULID::generate().to_string(),
                }),
            )
            .unwrap(),
            histogram: metrics::new_histogram(
                MetricId::generate(),
                "Metrics::gauge_vec",
                vec![0.1, 0.5, 1.0],
                Some(hashmap! {
                    metrics::LabelId::generate() => ULID::generate().to_string(),
                    metrics::LabelId::generate() => ULID::generate().to_string(),
                }),
            )
            .unwrap(),
            histogram_vec: metrics::new_histogram_vec(
                MetricId::generate(),
                "Metrics::int_gauge_vec",
                &[LabelId::generate()],
                vec![0.1, 0.5, 1.0],
                Some(hashmap! {
                    metrics::LabelId::generate() => ULID::generate().to_string(),
                    metrics::LabelId::generate() => ULID::generate().to_string(),
                }),
            )
            .unwrap(),
        }
    }
}
