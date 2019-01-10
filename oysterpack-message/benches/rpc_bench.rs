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
};

use oysterpack_message::protocol::rpc::*;

criterion_group!(
    benches,
    nng_msg_req_rep_bench,
    nng_msg_req_rep_bench_custom_stack_size
);

criterion_main!(benches);

struct Echo;

impl MessageHandler<nng::Message, nng::Message> for Echo {
    fn handle(&mut self, req: nng::Message) -> nng::Message {
        req
    }
}

fn log_config() -> oysterpack_log::LogConfig {
    oysterpack_log::config::LogConfigBuilder::new(oysterpack_log::Level::Info).build()
}

fn nng_msg_req_rep_bench(c: &mut Criterion) {
    oysterpack_log::init(log_config(), oysterpack_log::StderrLogger);
    let (client, service) = channels::<nng::Message, nng::Message>(10, 10);

    fn msg() -> nng::Message {
        let msg = b"some data";
        let mut nng_msg = nng::Message::with_capacity(msg.len()).unwrap();
        nng_msg.push_back(&msg[..]);
        nng_msg
    }

    Echo.bind(service.clone(), None);
    let nng_msg = msg();
    let client_n = client.clone();
    c.bench_function("nng_msg_req_rep_bench(threads = 1)", move |b| {
        b.iter(|| {
            client_n.request_channel().send(nng_msg.clone());
            let _ = client_n.reply_channel().recv().unwrap();
        })
    });

    Echo.bind(service.clone(), None);
    let nng_msg = msg();
    let client_n = client.clone();
    c.bench_function("nng_msg_req_rep_bench(threads = 2)", move |b| {
        b.iter(|| {
            client_n.request_channel().send(nng_msg.clone());
            let _ = client_n.reply_channel().recv().unwrap();
        })
    });

    Echo.bind(service.clone(), None);
    let nng_msg = msg();
    let client_n = client.clone();
    c.bench_function("nng_msg_req_rep_bench(threads = 3)", move |b| {
        b.iter(|| {
            client_n.request_channel().send(nng_msg.clone());
            let _ = client_n.reply_channel().recv().unwrap();
        })
    });

    Echo.bind(service.clone(), None);
    let nng_msg = msg();
    let client_n = client.clone();
    c.bench_function("nng_msg_req_rep_bench(threads = 4)", move |b| {
        b.iter(|| {
            client_n.request_channel().send(nng_msg.clone());
            let _ = client_n.reply_channel().recv().unwrap();
        })
    });
}

fn nng_msg_req_rep_bench_custom_stack_size(c: &mut Criterion) {
    oysterpack_log::init(log_config(), oysterpack_log::StderrLogger);
    let (client, service) = channels::<nng::Message, nng::Message>(10, 10);

    fn msg() -> nng::Message {
        let msg = b"some data";
        let mut nng_msg = nng::Message::with_capacity(msg.len()).unwrap();
        nng_msg.push_back(&msg[..]);
        nng_msg
    }

    Echo.bind(
        service.clone(),
        Some(ThreadConfig::new("Echo").set_stack_size(1024)),
    );
    let nng_msg = msg();
    let client_n = client.clone();
    c.bench_function(
        "nng_msg_req_rep_bench(threads = 1, stack_size = 1024)",
        move |b| {
            b.iter(|| {
                client_n.request_channel().send(nng_msg.clone());
                let _ = client_n.reply_channel().recv().unwrap();
            })
        },
    );

    Echo.bind(
        service.clone(),
        Some(ThreadConfig::new("Echo").set_stack_size(1024)),
    );
    let nng_msg = msg();
    let client_n = client.clone();
    c.bench_function(
        "nng_msg_req_rep_bench(threads = 2, stack_size = 1024)",
        move |b| {
            b.iter(|| {
                client_n.request_channel().send(nng_msg.clone());
                let _ = client_n.reply_channel().recv().unwrap();
            })
        },
    );

    Echo.bind(
        service.clone(),
        Some(ThreadConfig::new("Echo").set_stack_size(1024)),
    );
    let nng_msg = msg();
    let client_n = client.clone();
    c.bench_function(
        "nng_msg_req_rep_bench(threads = 3, stack_size = 1024)",
        move |b| {
            b.iter(|| {
                client_n.request_channel().send(nng_msg.clone());
                let _ = client_n.reply_channel().recv().unwrap();
            })
        },
    );

    Echo.bind(
        service.clone(),
        Some(ThreadConfig::new("Echo").set_stack_size(1024)),
    );
    let nng_msg = msg();
    let client_n = client.clone();
    c.bench_function(
        "nng_msg_req_rep_bench(threads = 4, stack_size = 1024)",
        move |b| {
            b.iter(|| {
                client_n.request_channel().send(nng_msg.clone());
                let _ = client_n.reply_channel().recv().unwrap();
            })
        },
    );
}
