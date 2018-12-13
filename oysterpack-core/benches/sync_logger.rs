/*
 * Copyright 2018 OysterPack Inc.
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

//! This benchmark is used as a baseline to compare with the sync_logger benchmark.
//!
//! This benchmarks logging to stderr synchronously on the same thread.

extern crate oysterpack_core;
#[macro_use]
extern crate oysterpack_log;
extern crate actix;
#[macro_use]
extern crate futures;

#[macro_use]
extern crate criterion;

use criterion::Criterion;
use oysterpack_log::log::*;

fn sync_stderr_logger_benchmark(c: &mut Criterion) {
    fn log_config() -> oysterpack_log::LogConfig {
        oysterpack_log::config::LogConfigBuilder::new(Level::Info).build()
    }

    oysterpack_log::init(log_config(), oysterpack_log::StderrLogger);

    c.bench_function("StderrLogger", |b| b.iter(|| error!("CIAO")));
}

criterion_group!(benches, sync_stderr_logger_benchmark);

fn main() {
    benches();

    criterion::Criterion::default()
        .configure_from_args()
        .final_summary();
}
