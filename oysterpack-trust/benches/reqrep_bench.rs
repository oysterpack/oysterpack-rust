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

use criterion::Criterion;

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

criterion_group!(benches, reqrep_bench_bench,);

criterion_main!(benches);

lazy_static! {
    static ref CLIENT: Arc<Mutex<ReqRep<(), ()>>> = {
        let req_rep = ReqRep::start_service(
            ReqRepId::generate(),
            1,
            EchoService,
            ExecutorBuilder::new(ExecutorId::generate())
                .register()
                .unwrap(),
            timer_buckets(),
        )
        .unwrap();
        Arc::new(Mutex::new(req_rep))
    };
}

/// measures how long a request/reply message flow takes
fn reqrep_bench_bench(c: &mut Criterion) {
    c.bench_function("reqrep_bench_bench", move |b| {
        let mut executor = global_executor();
        b.iter(|| {
            executor.run(
                async {
                    let mut req_rep = CLIENT.lock().unwrap();
                    await!(req_rep.send(()))
                },
            );
        })
    });
}

struct EchoService;

impl Processor<(), ()> for EchoService {
    fn process(&mut self, req: ()) -> reqrep::FutureReply<()> {
        futures::future::ready(()).boxed()
    }
}

fn timer_buckets() -> metrics::TimerBuckets {
    metrics::TimerBuckets::from(
        vec![
            Duration::from_nanos(10),
            Duration::from_nanos(25),
            Duration::from_nanos(50),
            Duration::from_nanos(75),
            Duration::from_nanos(100),
            Duration::from_nanos(125),
            Duration::from_nanos(150),
            Duration::from_nanos(200),
            Duration::from_nanos(250),
        ]
        .as_slice(),
    )
}
