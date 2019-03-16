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
use oysterpack_trust_nng::reqrep::{
    client::*,
    server::{self, ServerHandle},
};
use oysterpack_uid::*;

use futures::{
    channel::oneshot,
    executor::ThreadPoolBuilder,
    future::{join_all, RemoteHandle},
    future::{Future, FutureExt},
    stream::StreamExt,
    task::{Spawn, SpawnExt},
};
use lazy_static::lazy_static;
use oysterpack_log::*;
use std::{
    num::NonZeroUsize,
    pin::Pin,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

criterion_group!(benches, nng_reqrep_inproc_bench, nng_reqrep_tcp_bench);

criterion_main!(benches);

fn start_server(url: url::Url, reqrep_id: ReqRepId) -> ServerHandle {
    let timer_buckets = metrics::exponential_timer_buckets(
        Duration::from_nanos(100),
        2.0,
        NonZeroUsize::new(20).unwrap(),
    )
    .unwrap();
    let reqrep_service = ReqRepConfig::new(reqrep_id, timer_buckets)
        .set_chan_buf_size(1)
        .start_service(EchoService, global_executor().clone())
        .unwrap();

    server::spawn(
        None,
        server::ListenerConfig::new(url),
        reqrep_service,
        execution::ExecutorBuilder::new(ExecutorId::generate())
            .register()
            .unwrap(),
    )
    .unwrap()
}

fn start_client(url: url::Url, reqrep_id: ReqRepId) -> Client {
    let timer_buckets = metrics::exponential_timer_buckets(
        Duration::from_nanos(100),
        2.0,
        NonZeroUsize::new(20).unwrap(),
    )
    .unwrap();

    register_client(
        ReqRepConfig::new(reqrep_id, timer_buckets).set_chan_buf_size(1),
        None,
        DialerConfig::new(url),
        execution::ExecutorBuilder::new(ExecutorId::generate())
            .register()
            .unwrap(),
    )
    .unwrap()
}

/// measures how long a request/reply message flow takes
fn nng_reqrep_inproc_bench(c: &mut Criterion) {
    let reqrep_id = ReqRepId::generate();
    let url = url::Url::parse(format!("inproc://{}", ULID::generate()).as_str()).unwrap();
    let mut server_handle = start_server(url.clone(), reqrep_id);
    assert!(server_handle.ping());

    let mut client = start_client(url, reqrep_id);

    c.bench_function("nng_reqrep_inproc_bench", move |b| {
        let mut executor = global_executor();
        b.iter(|| {
            executor.run(async { await!(client.send(nng::Message::new().unwrap())).unwrap() });
        })
    });
}

fn nng_reqrep_tcp_bench(c: &mut Criterion) {
    let reqrep_id = ReqRepId::generate();
    let url = url::Url::parse("tcp://127.0.0.1:4747").unwrap();
    let mut server_handle = start_server(url.clone(), reqrep_id);
    assert!(server_handle.ping());

    let mut client = start_client(url, reqrep_id);

    c.bench_function("nng_reqrep_tcp_bench", move |b| {
        let mut executor = global_executor();
        b.iter(|| {
            executor.run(async { await!(client.send(nng::Message::new().unwrap())).unwrap() });
        })
    });
}

struct EchoService;
impl Processor<nng::Message, nng::Message> for EchoService {
    fn process(&mut self, req: nng::Message) -> reqrep::FutureReply<nng::Message> {
        async move { req }.boxed()
    }
}
