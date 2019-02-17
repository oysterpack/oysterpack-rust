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
    opnng::{
        self,
        reqrep::{client::*, server},
    },
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
    pin::Pin,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

criterion_group!(benches, nng_reqrep_bench,);

criterion_main!(benches);

const REQREP_ID: ReqRepId = ReqRepId(1871557337320005579010710867531265404);

lazy_static! {
    static ref URL: url::Url = url::Url::parse(&format!("inproc://{}", ULID::generate())).unwrap();
    static ref CLIENT: Arc<Mutex<Client>> = Arc::new(Mutex::new(start_client()));
}

fn start_server() -> ReqRep<nng::Message, nng::Message> {
    let timer_buckets = metrics::TimerBuckets::from(
        vec![
            Duration::from_nanos(50),
            Duration::from_nanos(100),
            Duration::from_nanos(150),
            Duration::from_nanos(200),
        ]
        .as_slice(),
    );
    ReqRepConfig::new(REQREP_ID, timer_buckets)
        .set_chan_buf_size(1)
        .start_service(EchoService, global_executor().clone())
        .unwrap()
}

fn start_client() -> Client {
    let timer_buckets = metrics::TimerBuckets::from(
        vec![
            Duration::from_nanos(50),
            Duration::from_nanos(100),
            Duration::from_nanos(150),
            Duration::from_nanos(200),
        ]
        .as_slice(),
    );

    register_client(
        ReqRepConfig::new(REQREP_ID, timer_buckets).set_chan_buf_size(1),
        None,
        DialerConfig::new(URL.clone()),
        execution::ExecutorBuilder::new(ExecutorId::generate())
            .register()
            .unwrap(),
    )
    .unwrap()
}

/// measures how long a request/reply message flow takes
fn nng_reqrep_bench(c: &mut Criterion) {
    let server_executor_id = ExecutorId::generate();
    let mut server_handle = server::spawn(
        None,
        server::ListenerConfig::new(URL.clone()),
        start_server(),
        execution::ExecutorBuilder::new(server_executor_id)
            .register()
            .unwrap(),
    )
    .unwrap();
    assert!(server_handle.ping());

    c.bench_function("nng_reqrep_bench", move |b| {
        let mut executor = global_executor();
        b.iter(|| {
            executor.run(
                async {
                    let mut req_rep = CLIENT.lock().unwrap();
                    await!(req_rep.send(nng::Message::new().unwrap())).unwrap()
                },
            );
        })
    });
}

struct EchoService;
impl Processor<nng::Message, nng::Message> for EchoService {
    fn process(&mut self, req: nng::Message) -> reqrep::FutureReply<nng::Message> {
        async move { req }.boxed()
    }
}
