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

//! bench test summary

#![allow(warnings)]

#[macro_use]
extern crate criterion;

use criterion::Criterion;
use sodiumoxide::crypto::{box_, secretbox};

use std::{
    collections::HashMap,
    fmt, fs,
    io::{prelude::*, BufReader},
    num::NonZeroUsize,
    path::PathBuf,
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

use log::*;
use nng::aio;
use oysterpack_errors::Error;
use oysterpack_message::op_nng::{
    new_aio_context,
    rpc::{
        client::{
            asyncio::{AsyncClient, ReplyHandler},
            syncio::SyncClient,
            DialerSettings,
        },
        server::*,
        MessageProcessor, MessageProcessorFactory,
    },
};
use oysterpack_uid::ULID;

criterion_group!(
    benches,
    nng_sync_client_context_bench,
    nng_async_client_context_bench,
    aio_context_std_hashmap_storage_bench,
    aio_context_fnv_hashmap_storage_bench,
    aio_context_smallvec_storage_bench
);

criterion_main!(benches);

#[derive(Debug, Clone, Default)]
struct EchoProcessor;

impl MessageProcessorFactory<EchoProcessor, nng::Message, nng::Message> for EchoProcessor {
    fn new(&self) -> EchoProcessor {
        EchoProcessor
    }
}

impl MessageProcessor<nng::Message, nng::Message> for EchoProcessor {
    fn process(&mut self, req: nng::Message) -> nng::Message {
        req
    }
}

fn log_config() -> oysterpack_log::LogConfig {
    oysterpack_log::config::LogConfigBuilder::new(oysterpack_log::Level::Info).build()
}

fn nng_sync_client_context_bench(c: &mut Criterion) {
    sync_client_context_bench(c, 1);
    sync_client_context_bench(c, 2);
    sync_client_context_bench(c, num_cpus::get());
}

fn sync_client_context_bench(c: &mut Criterion, server_aio_context_count: usize) {
    oysterpack_log::init(log_config(), oysterpack_log::StderrLogger);

    let url = Arc::new(format!("inproc://{}", ULID::generate()));

    // start a server with 2 aio contexts
    let listener_settings = ListenerSettings::new(&*url.as_str())
        .set_aio_count(NonZeroUsize::new(server_aio_context_count).unwrap());
    let server = Server::builder(listener_settings, EchoProcessor)
        .spawn()
        .unwrap();

    let dialer_settings = DialerSettings::new(url.as_str())
        .set_non_blocking(true)
        .set_reconnect_min_time(Duration::from_millis(10))
        .set_reconnect_max_time(Duration::from_millis(10));

    let mut client = SyncClient::dial(dialer_settings.clone()).unwrap();
    info!(
        "received reply: {:?}",
        client.send(nng::Message::new().unwrap())
    );

    let bench_function_id = format!(
        "nng_sync_client_bench(server aio context count = {})",
        server_aio_context_count
    );
    c.bench_function(bench_function_id.as_str(), move |b| {
        b.iter(|| {
            client.send(nng::Message::new().unwrap()).unwrap();
        })
    });

    server.stop();
    server.join();
}

fn nng_async_client_context_bench(c: &mut Criterion) {
    async_client_context_bench(c, 1, 1);
    async_client_context_bench(c, 2, 1);
    async_client_context_bench(c, num_cpus::get(), 1);
    async_client_context_bench(c, num_cpus::get(), num_cpus::get() / 2);
}

struct NoopReplyHandler;

impl ReplyHandler for NoopReplyHandler {
    fn on_reply(&mut self, result: Result<nng::Message, Error>) {
        if let Err(err) = result {
            error!("request failed: {}", err);
        }
    }
}

fn async_client_context_bench(
    c: &mut Criterion,
    server_aio_context_count: usize,
    client_aio_context_count: usize,
) {
    oysterpack_log::init(log_config(), oysterpack_log::StderrLogger);

    let url = Arc::new(format!("inproc://{}", ULID::generate()));

    // start a server with 2 aio contexts
    let listener_settings = ListenerSettings::new(&*url.as_str())
        .set_aio_count(NonZeroUsize::new(server_aio_context_count).unwrap());
    let server = Server::builder(listener_settings, EchoProcessor)
        .spawn()
        .unwrap();

    let dialer_settings = DialerSettings::new(url.as_str())
        .set_non_blocking(true)
        .set_reconnect_min_time(Duration::from_millis(10))
        .set_reconnect_max_time(Duration::from_millis(10))
        .set_max_concurrent_request_capacity(NonZeroUsize::new(client_aio_context_count).unwrap());

    let mut client = AsyncClient::dial(dialer_settings.clone()).unwrap();
    client
        .send_with_callback(nng::Message::new().unwrap(), NoopReplyHandler)
        .unwrap();
    info!("sent async request");
    thread::yield_now();
    loop {
        if client.available_capacity() > 0 {
            break;
        }
        thread::yield_now();
    }

    let bench_function_id = format!(
        "nng_async_client_bench(aio context counts: server = {}, client = {})",
        server_aio_context_count, client_aio_context_count
    );
    c.bench_function(bench_function_id.as_str(), move |b| {
        b.iter(|| {
            if let Err(err) =
                client.send_with_callback(nng::Message::new().unwrap(), NoopReplyHandler)
            {
                loop {
                    if client.available_capacity() > 0 {
                        break;
                    }
                    thread::yield_now();
                }
            }
        })
    });

    server.stop();
    server.join();
}

fn aio_context_std_hashmap_storage_bench(c: &mut Criterion) {
    let url = format!("inproc://{}", ULID::generate());

    let mut socket = nng::Socket::new(nng::Protocol::Rep0).unwrap();
    socket.set_nonblocking(true);
    let context = new_aio_context(&socket).unwrap();
    let aio = nng::aio::Aio::with_callback(move |aio| info!("invoked")).unwrap();

    let context_id = ContextId::new(&context);
    let aio_context = AioContext::from((aio, context));
    let mut aio_contexts = HashMap::with_capacity(16);
    aio_contexts.insert(context_id, aio_context);
    c.bench_function("aio_context_std_hashmap_storage_bench", move |b| {
        b.iter(|| {
            let aio_context = aio_contexts.remove(&context_id).unwrap();
            aio_contexts.insert(context_id, aio_context);
        })
    });
}

fn aio_context_fnv_hashmap_storage_bench(c: &mut Criterion) {
    let url = format!("inproc://{}", ULID::generate());

    let mut socket = nng::Socket::new(nng::Protocol::Rep0).unwrap();
    socket.set_nonblocking(true);

    {
        let context = new_aio_context(&socket).unwrap();
        let aio = nng::aio::Aio::with_callback(move |aio| info!("invoked")).unwrap();
        let context_id = ContextId::new(&context);
        let aio_context = AioContext::from((aio, context));
        let mut aio_contexts = fnv::FnvHashMap::<ContextId, AioContext>::with_capacity_and_hasher(
            16,
            fnv::FnvBuildHasher::default(),
        );
        aio_contexts.insert(context_id, aio_context);
        c.bench_function("aio_context_fnv_hashmap_storage_bench(1 entry)", move |b| {
            b.iter(|| {
                let aio_context = aio_contexts.remove(&context_id).unwrap();
                aio_contexts.insert(context_id, aio_context);
            })
        });
    }

    let mut bench = |capacity: usize| {
        let mut aio_contexts = fnv::FnvHashMap::<ContextId, AioContext>::with_capacity_and_hasher(
            capacity,
            fnv::FnvBuildHasher::default(),
        );
        for _ in 0..capacity {
            let context = new_aio_context(&socket).unwrap();
            let aio = nng::aio::Aio::with_callback(move |aio| info!("invoked")).unwrap();
            let context_id = ContextId::new(&context);
            let aio_context = AioContext::from((aio, context));
            aio_contexts.insert(context_id, aio_context);
        }
        let context_id = *aio_contexts.keys().nth(0).unwrap();
        c.bench_function(
            format!(
                "aio_context_fnv_hashmap_storage_bench({} entries)",
                capacity
            )
            .as_str(),
            move |b| {
                b.iter(|| {
                    let aio_context = aio_contexts.remove(&context_id).unwrap();
                    aio_contexts.insert(context_id, aio_context);
                })
            },
        );
    };

    bench(16);
    bench(128);
    bench(256);
    bench(1024);
}

fn aio_context_smallvec_storage_bench(c: &mut Criterion) {
    type AioContext = (nng::aio::Aio, nng::aio::Context);

    type ContextId = i32;

    struct AioContexts(
        smallvec::SmallVec<[Option<(ContextId, AioContext)>; AioContexts::CACHE_SIZE]>,
    );

    impl AioContexts {
        const CACHE_SIZE: usize = 16;

        fn push(&mut self, entry: (ContextId, AioContext)) {
            for i in 0..AioContexts::CACHE_SIZE {
                if self.0[i].is_none() {
                    self.0[i] = Some(entry);
                    return;
                }
            }
            self.0.push(Some(entry));
        }

        fn remove(&mut self, context_id: ContextId) -> Option<AioContext> {
            for i in 0..AioContexts::CACHE_SIZE {
                if let Some((key, _)) = self.0[i] {
                    if context_id == key {
                        let (_, value) = self.0[i].take().unwrap();
                        return Some(value);
                    }
                }
            }
            None
        }
    }

    impl Default for AioContexts {
        fn default() -> AioContexts {
            let mut aio_contexts = smallvec::SmallVec::<
                [Option<(ContextId, AioContext)>; AioContexts::CACHE_SIZE],
            >::new();
            for _ in 0..AioContexts::CACHE_SIZE {
                aio_contexts.push(None);
            }
            AioContexts(aio_contexts)
        }
    }

    let url = format!("inproc://{}", ULID::generate());

    let mut socket = nng::Socket::new(nng::Protocol::Rep0).unwrap();
    socket.set_nonblocking(true);

    let mut aio_contexts = AioContexts::default();

    let mut context_ids = Vec::new();
    for i in 0..AioContexts::CACHE_SIZE {
        let context = new_aio_context(&socket).unwrap();
        let aio = nng::aio::Aio::with_callback(move |aio| info!("invoked")).unwrap();
        context_ids.push(context.id());
        aio_contexts.push((context.id(), (aio, context)))
    }
    let context_id = context_ids.pop().unwrap();

    c.bench_function(
        format!(
            "aio_context_smallvec_storage_bench({} entry)",
            AioContexts::CACHE_SIZE
        )
        .as_str(),
        move |b| {
            b.iter(|| {
                let aio_context = aio_contexts.remove(context_id).unwrap();
                aio_contexts.push((context_id, aio_context));
            })
        },
    );
}

struct AioContext {
    _aio: aio::Aio,
    context: aio::Context,
}

impl From<(aio::Aio, aio::Context)> for AioContext {
    fn from((aio, context): (aio::Aio, aio::Context)) -> Self {
        AioContext { _aio: aio, context }
    }
}

impl fmt::Debug for AioContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "AioContext({})", self.context.id())
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Ord, PartialOrd)]
struct ContextId(Instant, i32);

impl ContextId {
    fn new(context: &aio::Context) -> ContextId {
        ContextId(Instant::now(), context.id())
    }
}
