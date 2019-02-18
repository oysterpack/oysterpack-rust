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

//! request/reply messaging bench tests

#![feature(await_macro, async_await, futures_api, arbitrary_self_types)]
#![allow(warnings)]

#[macro_use]
extern crate criterion;

use criterion::{BatchSize, Criterion};

use oysterpack_trust::{
    concurrent::{
        execution::{self, *},
        messaging::reqrep::{self, *},
    },
    metrics,
};

use futures::{
    channel::oneshot,
    future::{join_all, RemoteHandle},
    future::{Future, FutureExt},
    sink::SinkExt,
    stream::StreamExt,
    task::{Spawn, SpawnExt},
};
use lazy_static::lazy_static;
use oysterpack_log::*;
use std::{
    pin::Pin,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

criterion_group!(benches, executor_run_bench, executor_spawn_bench,);

criterion_main!(benches);

fn executor_run_bench(c: &mut Criterion) {
    c.bench_function("executor_run_bench", move |b| {
        let mut executor = ExecutorBuilder::new(ExecutorId::generate())
            .set_catch_unwind(false)
            .register()
            .unwrap();
        b.iter(|| {
            executor.run(async {});
        });
    });
}

/// ## Summary
/// - catch_unwind adds no performance overhead
fn executor_spawn_bench(c: &mut Criterion) {
    let executor_id = ExecutorId::generate();
    let _ = ExecutorBuilder::new(executor_id)
        .set_catch_unwind(false)
        .register()
        .unwrap();

    c.bench_function("executor_spawn_no_catch_unwind_bench", move |b| {
        let mut executor = execution::executor(executor_id).unwrap();
        assert!(!executor.catch_unwind());
        b.iter(|| {
            let (mut tx, rx) = futures::channel::oneshot::channel();
            executor.spawn(
                async move {
                    tx.send(()).unwrap();
                },
            );
            executor.run(
                async move {
                    await!(rx).unwrap();
                },
            );
        });
    });

    let executor_id = ExecutorId::generate();
    let _ = ExecutorBuilder::new(executor_id)
        .set_catch_unwind(true)
        .register()
        .unwrap();

    c.bench_function("executor_spawn_catch_unwind_bench", move |b| {
        let mut executor = execution::executor(executor_id).unwrap();
        assert!(executor.catch_unwind());
        b.iter(|| {
            let (mut tx, rx) = futures::channel::oneshot::channel();
            executor.spawn(
                async move {
                    tx.send(()).unwrap();
                },
            );
            executor.run(
                async move {
                    await!(rx).unwrap();
                },
            );
        });
    });
}
