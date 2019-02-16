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

mod steps;

use maplit::*;
use oysterpack_trust::metrics;
use oysterpack_trust::metrics::LabelId;
use oysterpack_trust::metrics::MetricId;
use oysterpack_uid::ULID;
use std::{collections::HashMap, sync::Arc, thread};

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

cucumber! {
    features: "./features/metrics",
    world: crate::TestContext,
    steps: &[
        steps::metrics::steps
    ]
}
