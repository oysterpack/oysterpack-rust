// Copyright 2018 OysterPack Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! This benchmarks the logging throughput.
//!
//! Log records are logged async on a separate thread via the Logger Service Actor.
//!
//! In comparison with the sync logger, the logging throughput on the logging thread is ~68x greater.
//! Thus, we do see a significant benefit to log async.

extern crate oysterpack_core;
#[macro_use]
extern crate oysterpack_log;
extern crate actix;
#[macro_use]
extern crate futures;

#[macro_use]
extern crate criterion;

use criterion::Criterion;

use oysterpack_core::actor;
use oysterpack_log::log::*;

use actix::System;
use futures::{future, prelude::*};

use std::{sync::mpsc, thread};

fn async_stderr_logger_benchmark(c: &mut Criterion) {
    fn log_config() -> oysterpack_log::LogConfig {
        oysterpack_log::config::LogConfigBuilder::new(Level::Info).build()
    }

    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        System::run(move || {
            let task = actor::logger::init_logging(log_config());
            let task = task
                .and_then(move |_| {
                    for i in 0..20 {
                        info!("LOG MSG #{}", i);
                    }
                    Ok(())
                }).then(move |_| {
                    let _ = tx.send(System::current());
                    future::ok::<(), ()>(())
                });
            actor::spawn_task(task);
        });
    });

    let system: System = rx.recv().unwrap();

    c.bench_function("Logger Service Actor", |b| b.iter(|| error!("CIAO")));

    system.stop();
}

criterion_group!(benches, async_stderr_logger_benchmark);

fn main() {
    benches();

    criterion::Criterion::default()
        .configure_from_args()
        .final_summary();
}
