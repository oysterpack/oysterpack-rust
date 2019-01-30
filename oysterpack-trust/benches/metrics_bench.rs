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

#![feature(await_macro, async_await, futures_api, arbitrary_self_types)]
#![feature(duration_float)]

#[macro_use]
extern crate criterion;

use criterion::Criterion;

use oysterpack_uid::ULID;
use std::time::Duration;
use oysterpack_trust::{
    metrics::*,
    concurrent::execution::*
};
use futures::{sink::SinkExt, task::SpawnExt, stream::StreamExt};
use oysterpack_log::*;

criterion_group!(benches,
    prometheus_histogram_vec_observe_bench,
    metrics_local_counter_bench
);

criterion_main!(benches);

/// take away is that spawning async futures provides too much overhead
fn metrics_local_counter_bench(c: &mut Criterion) {
    /// Local counter is designed to be non-blocking
    #[derive(Debug, Clone)]
    pub struct LocalCounter {
        sender: futures::channel::mpsc::Sender<CounterMessage>,
        executor: Executor,
    }

    impl LocalCounter {

        /// constructor
        /// - spawns a bacground async task to update the metric
        ///
        pub fn new(counter: prometheus::core::GenericLocalCounter<prometheus::core::AtomicI64>, executor: Executor) -> Result<Self,futures::task::SpawnError> {
            let (sender, mut receiver) = futures::channel::mpsc::channel(1);
            let mut executor = executor;
            let mut counter = counter;
            executor.spawn(
                async move {
                    while let Some(msg) = await!(receiver.next()) {
                        match msg {
                            CounterMessage::Inc => counter.inc(),
                            CounterMessage::Flush(reply) => {
                                counter.flush();
                                if let Err(_) = reply.send(()) {
                                    warn!("Failed to send Flush reply");
                                }
                            },
                            CounterMessage::Close => break
                        }
                    }
                }
            )?;
            Ok(Self {
                sender,
                executor
            })
        }

        /// increment the counter
        pub fn inc(&mut self) -> Result<(), futures::task::SpawnError> {
            let mut sender = self.sender.clone();
            self.executor.spawn(
                async move {
                    if let Err(err) = await!(sender.send(CounterMessage::Inc)) {
                        warn!("Failed to send Inc message: {}", err);
                    }
                },
            )
        }
    }

    /// Counter message
    #[derive(Debug)]
    pub enum CounterMessage {
        /// increment the counter
        Inc,
        /// flush the local counter to the registered counter
        Flush(futures::channel::oneshot::Sender<()>),
        /// close the local counter receiver channel, which drops the local counter
        Close,
    }


    let metric_id = MetricId::generate();
    let counter = METRIC_REGISTRY.register_int_counter(metric_id, ULID::generate().to_string(), None).unwrap();
    let mut async_local_counter = LocalCounter::new(counter.local(), GLOBAL_EXECUTOR.clone()).unwrap();

    let mut local_counter = counter.local();
    c.bench_function("metrics_local_counter_bench - local", move |b| {
        b.iter(|| local_counter.inc())
    });

    c.bench_function("metrics_local_counter_bench - sync", move |b| {
        b.iter(|| counter.inc())
    });

    c.bench_function("metrics_local_counter_bench - async", move |b| {
        b.iter(|| async_local_counter.inc())
    });
}

fn prometheus_histogram_vec_observe_bench(c: &mut Criterion) {
    {
        let reqrep_timer = format!("OP{}", ULID::generate());
        let reqrep_service_id_label = format!("OP{}", ULID::generate());

        let registry = prometheus::Registry::new();
        let opts = prometheus::HistogramOpts::new(reqrep_timer, "reqrep timer".to_string());

        let reqrep_timer =
            prometheus::HistogramVec::new(opts, &[reqrep_service_id_label.as_str()]).unwrap();
        registry.register(Box::new(reqrep_timer.clone())).unwrap();

        c.bench_function("prometheus_histogram_vec_observe", move |b| {
            let mut reqrep_timer_local = reqrep_timer.local();
            let reqrep_timer =
                reqrep_timer_local.with_label_values(&[ULID::generate().to_string().as_str()]);
            let clock = quanta::Clock::new();

            b.iter(|| {
                let f = || {};
                let start = clock.start();
                f();
                let end = clock.end();
                let delta = clock.delta(start, end);
                reqrep_timer.observe(delta as f64);
                reqrep_timer.flush();
            })
        });
    }

    {
        let reqrep_timer = format!("OP{}", ULID::generate());
        let reqrep_service_id_label = format!("OP{}", ULID::generate());

        let registry = prometheus::Registry::new();
        let opts = prometheus::HistogramOpts::new(reqrep_timer, "reqrep timer".to_string());

        let reqrep_timer =
            prometheus::HistogramVec::new(opts, &[reqrep_service_id_label.as_str()]).unwrap();
        registry.register(Box::new(reqrep_timer.clone())).unwrap();

        c.bench_function("prometheus_histogram_vec_observe_no_flush", move |b| {
            let mut reqrep_timer_local = reqrep_timer.local();
            let reqrep_timer =
                reqrep_timer_local.with_label_values(&[ULID::generate().to_string().as_str()]);
            let clock = quanta::Clock::new();

            b.iter(|| {
                let f = || {};
                let start = clock.start();
                f();
                let end = clock.end();
                let delta = clock.delta(start, end);
                reqrep_timer.observe(delta as f64);
            })
        });
    }

    {
        let reqrep_timer = format!("OP{}", ULID::generate());
        let reqrep_service_id_label = format!("OP{}", ULID::generate());

        let registry = prometheus::Registry::new();
        let opts = prometheus::HistogramOpts::new(reqrep_timer, "reqrep timer".to_string());

        let reqrep_timer =
            prometheus::HistogramVec::new(opts, &[reqrep_service_id_label.as_str()]).unwrap();
        registry.register(Box::new(reqrep_timer.clone())).unwrap();

        c.bench_function("prometheus_histogram_vec_observe_float_secs", move |b| {
            let mut reqrep_timer_local = reqrep_timer.local();
            let reqrep_timer =
                reqrep_timer_local.with_label_values(&[ULID::generate().to_string().as_str()]);
            let clock = quanta::Clock::new();

            b.iter(|| {
                let f = || {};
                let start = clock.start();
                f();
                let end = clock.end();
                let delta = clock.delta(start, end);
                let delta = Duration::from_nanos(delta);
                reqrep_timer.observe(delta.as_float_secs());
                reqrep_timer.flush();
            })
        });
    }

    {
        let reqrep_timer = format!("OP{}", ULID::generate());
        let reqrep_service_id_label = format!("OP{}", ULID::generate());

        let registry = prometheus::Registry::new();
        let opts = prometheus::HistogramOpts::new(reqrep_timer, "reqrep timer".to_string());

        let reqrep_timer =
            prometheus::HistogramVec::new(opts, &[reqrep_service_id_label.as_str()]).unwrap();
        registry.register(Box::new(reqrep_timer.clone())).unwrap();

        c.bench_function(
            "prometheus_histogram_vec_observe_float_secs_direct",
            move |b| {
                let mut reqrep_timer_local = reqrep_timer.local();
                let reqrep_timer =
                    reqrep_timer_local.with_label_values(&[ULID::generate().to_string().as_str()]);
                let clock = quanta::Clock::new();

                b.iter(|| {
                    let f = || {};
                    let start = clock.start();
                    f();
                    let end = clock.end();
                    let delta = clock.delta(start, end);
                    reqrep_timer.observe(as_float_secs(delta));
                    reqrep_timer.flush();
                })
            },
        );
    }

    {
        let reqrep_timer = format!("OP{}", ULID::generate());
        let reqrep_service_id_label = format!("OP{}", ULID::generate());

        let registry = prometheus::Registry::new();
        let opts = prometheus::HistogramOpts::new(reqrep_timer, "reqrep timer".to_string());

        let reqrep_timer =
            prometheus::HistogramVec::new(opts, &[reqrep_service_id_label.as_str()]).unwrap();
        registry.register(Box::new(reqrep_timer.clone())).unwrap();

        c.bench_function(
            "prometheus_histogram_vec_observe_float_secs_direct_timed",
            move |b| {
                let mut reqrep_timer_local = reqrep_timer.local();
                let reqrep_timer =
                    reqrep_timer_local.with_label_values(&[ULID::generate().to_string().as_str()]);
                let clock = quanta::Clock::new();

                b.iter(|| {
                    let delta = time(&clock, || {});
                    reqrep_timer.observe(as_float_secs(delta));
                    reqrep_timer.flush();
                })
            },
        );
    }
}

const NANOS_PER_SEC: u32 = 1_000_000_000;

pub fn as_float_secs(nanos: u64) -> f64 {
    (nanos as f64) / (NANOS_PER_SEC as f64)
}

fn time<F>(clock: &quanta::Clock, f: F) -> u64
where
    F: FnOnce(),
{
    let start = clock.start();
    f();
    let end = clock.end();
    clock.delta(start, end)
}
