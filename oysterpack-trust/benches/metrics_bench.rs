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

#![feature(duration_float)]

#[macro_use]
extern crate criterion;

use criterion::Criterion;

use oysterpack_uid::ULID;
use std::time::Duration;

criterion_group!(benches, prometheus_histogram_vec_observe_bench);

criterion_main!(benches);

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
