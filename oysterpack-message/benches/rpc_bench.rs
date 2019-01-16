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
    fs,
    io::{prelude::*, BufReader},
    path::PathBuf,
    thread,
    sync::Arc,
    time::Duration,
    num::NonZeroUsize
};

use oysterpack_message::op_nng::rpc::{client::{
    DialerSettings,
    syncio::{
        SyncClient
    },
    asyncio::{
        AsyncClient, ReplyHandler
    }
}, server::*, MessageProcessor, MessageProcessorFactory};
use oysterpack_uid::ULID;
use oysterpack_errors::Error;
use log::*;

criterion_group!(
    benches,
    nng_sync_client_context_bench,
    nng_async_client_context_bench
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
    let listener_settings =
        ListenerSettings::new(&*url.as_str()).set_aio_count(NonZeroUsize::new(server_aio_context_count).unwrap());
    let server = Server::builder(listener_settings, EchoProcessor)
        .spawn()
        .unwrap();

    let dialer_settings = DialerSettings::new(url.as_str())
        .set_non_blocking(true)
        .set_reconnect_min_time(Duration::from_millis(10))
        .set_reconnect_max_time(Duration::from_millis(10));

    let mut client = SyncClient::dial(dialer_settings.clone()).unwrap();
    info!("received reply: {:?}",client.send(nng::Message::new().unwrap()));

    let bench_function_id = format!("nng_sync_client_bench(server aio context count = {})", server_aio_context_count);
    c.bench_function(bench_function_id.as_str(), move |b| {
        b.iter(|| {
            client.send(nng::Message::new().unwrap()).unwrap();
        })
    });

    server.stop();
    server.join();
}

fn nng_async_client_context_bench(c: &mut Criterion) {
    async_client_context_bench(c, 1,1);
    async_client_context_bench(c, 2,1);
    async_client_context_bench(c, num_cpus::get(),1);
    async_client_context_bench(c, num_cpus::get(),num_cpus::get()/2);
//    async_client_context_bench(c, 1,2);
//    async_client_context_bench(c, 1,4);
//    async_client_context_bench(c, 1,num_cpus::get());
//    async_client_context_bench(c, 2);
//    async_client_context_bench(c, num_cpus::get());
}

struct NoopReplyHandler;

impl ReplyHandler for NoopReplyHandler {
    fn on_reply(&mut self, result: Result<nng::Message, Error>) {
        if let Err(err) = result {
            error!("request failed: {}", err);
        }
    }
}

fn async_client_context_bench(c: &mut Criterion, server_aio_context_count: usize, client_aio_context_count: usize) {
    oysterpack_log::init(log_config(), oysterpack_log::StderrLogger);

    let url = Arc::new(format!("inproc://{}", ULID::generate()));

    // start a server with 2 aio contexts
    let listener_settings =
        ListenerSettings::new(&*url.as_str()).set_aio_count(NonZeroUsize::new(server_aio_context_count).unwrap());
    let server = Server::builder(listener_settings, EchoProcessor)
        .spawn()
        .unwrap();

    let dialer_settings = DialerSettings::new(url.as_str())
        .set_non_blocking(true)
        .set_reconnect_min_time(Duration::from_millis(10))
        .set_reconnect_max_time(Duration::from_millis(10))
        .set_capacity(NonZeroUsize::new(client_aio_context_count).unwrap());

    let mut client = AsyncClient::dial(dialer_settings.clone()).unwrap();
    client.send_with_callback(nng::Message::new().unwrap(),NoopReplyHandler).unwrap();
    info!("sent async request");
    thread::yield_now();
    loop {
        if client.available_capacity() > 0 {
            break;
        }
        thread::yield_now();
    }

    let bench_function_id = format!("nng_async_client_bench(aio context counts: server = {}, client = {} )", server_aio_context_count, client_aio_context_count);
    c.bench_function(bench_function_id.as_str(), move |b| {
        b.iter(|| {
            if let Err(err) = client.send_with_callback(nng::Message::new().unwrap(),NoopReplyHandler) {
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
