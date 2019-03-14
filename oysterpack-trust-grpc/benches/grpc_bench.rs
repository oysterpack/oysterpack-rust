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

#![feature(await_macro, async_await, futures_api, arbitrary_self_types)]
#![allow(warnings)]

#[macro_use]
extern crate criterion;

use criterion::{BatchSize, Criterion};

mod protos;

use protos::{
    foo::*,
    foo_grpc::{self, *},
};

use oysterpack_trust::concurrent::execution::{
    self,
    futures::{
        compat::{Compat, Compat01As03, Future01CompatExt, Stream01CompatExt},
        sink::SinkExt,
        stream::StreamExt,
        task::SpawnExt,
    },
    global_executor,
};
use oysterpack_uid::ULID;

use grpcio::{ChannelBuilder, EnvBuilder, Environment, ServerBuilder, WriteFlags};
use hashbrown::HashMap;
use parking_lot::Mutex;
use std::time::Duration;
use std::{num::NonZeroUsize, sync::Arc};

criterion_group!(
    benches,
    grpc_bench
);

criterion_main!(benches);

/// The benchmarks expose an issue: it appears the server cancels requests under load.
fn grpc_bench(c: &mut Criterion) {
    let env = Arc::new(Environment::new(1));
    let service = foo_grpc::create_foo(FooServer::default());
    let mut server = ServerBuilder::new(env)
        .register_service(service)
        .requests_slot_per_cq(1024*16)
        .bind("127.0.0.1", 0)
        .build()
        .unwrap();
    server.start();

    let statuses = Arc::new(Mutex::new((0, Vec::with_capacity(16))));
    let statuses_clone = statuses.clone();
    let print_statuses = move || {
        let mut statuses = statuses_clone.lock();
        println!(
            "grpc_unary_async_run_bench(): error count = {}",
            statuses.1.len()
        );
        println!("grpc_unary_async_run_bench(): statuses: {:#?}", *statuses);
        statuses.1.clear();
    };
    for &(ref host, port) in server.bind_addrs() {

        println!(
            "grpc_unary_run_bench(): grpc server listening on {}:{}",
            host, port
        );

        let env = Arc::new(EnvBuilder::new().build());
        let ch = ChannelBuilder::new(env).connect(format!("{}:{}", host, port).as_str());
        let client = foo_grpc::FooClient::new(ch);

        {
            let statuses = statuses.clone();
            let client = client.clone();
            c.bench_function("executor_run_bench", move |b| {

                b.iter(|| {
                    let request = Request::new();
                    let response = client.unary(&request);
                    let mut errs = statuses.lock();
                    errs.0 += 1;
                    if let Err(err) = response {
                        errs.1.push(err);
                    }
                });
            });
            print_statuses();
        }

        {
            let statuses = statuses.clone();
            let client = client.clone();
            c.bench_function("grpc_unary_async_run_bench", move |b| {
                b.iter(|| {
                    let request = Request::new();
                    let reply_receiver = client.unary_async(&request).unwrap();
                    let response =
                        global_executor().run(async move { await!(reply_receiver.compat()) });
                    let mut errs = statuses.lock();
                    errs.0 += 1;
                    if let Err(err) = response {
                        errs.1.push(err);
                    }
                });
            });
            print_statuses();
        }

        {
            let statuses = statuses.clone();
            let client = client.clone();
            c.bench_function("grpc_unary_async_response_run_bench", move |b| {
                b.iter(|| {
                    let request = Request::new();
                    let reply_receiver = client.unary_async(&request).unwrap();
                    global_executor()
                        .spawn(
                            async move {
                                await!(reply_receiver.compat());
                            },
                        )
                        .unwrap();
                });
            });
        }
    }

    println!("grpc_unary_run_bench(): server is shutting down ...");
    {
        use futures::Future;
        let _ = server.shutdown().wait();
    }
    println!("grpc_unary_run_bench(): server has been shutdown")
}

#[derive(Clone)]
struct FooServer {
    executor: execution::Executor,
}

impl Default for FooServer {
    fn default() -> Self {
        Self {
            executor: global_executor(),
        }
    }
}

impl Foo for FooServer {
    fn unary(
        &mut self,
        _ctx: grpcio::RpcContext,
        _req: Request,
        sink: grpcio::UnarySink<Response>,
    ) {
        let response = Response::new();
        self.executor
            .spawn(
                async move {
                    sink.success(response);
                },
            )
            .unwrap();
    }

    fn client_streaming(
        &mut self,
        _ctx: ::grpcio::RpcContext,
        stream: ::grpcio::RequestStream<Request>,
        sink: ::grpcio::ClientStreamingSink<Response>,
    ) {
        self.executor
            .spawn(
                async move {
                    // receive all client request messages
                    let mut stream = stream.compat();
                    while let Some(request) = await!(stream.next()) {
                        // drain the stream
                    }
                    // once all messages have been received, then send the response
                    let mut response = Response::new();
                    sink.success(response);
                },
            )
            .unwrap();
    }

    fn server_streaming(
        &mut self,
        _ctx: ::grpcio::RpcContext,
        _req: Request,
        sink: ::grpcio::ServerStreamingSink<Response>,
    ) {
        // the design is to stream messages using channels
        // ServerStreamingSink will send all messages received on the mpsc:Receiver stream
        let (mut tx, rx) = execution::futures::channel::mpsc::channel(0);
        {
            use futures::prelude::Sink;
            let send_all = sink.send_all(Compat::new(rx));
            self.executor
                .spawn(
                    async move {
                        let _ = await!(send_all.compat());
                    },
                )
                .unwrap();
        }

        // deliver messages via mpsc::Sender
        self.executor
            .spawn(
                async move {
                    let write_flags = WriteFlags::default();
                    for i in 0..10 {
                        let mut response = Response::new();
                        let msg = Result::<(Response, WriteFlags), grpcio::Error>::Ok((
                            response,
                            write_flags,
                        ));
                        let _ = await!(tx.send(msg));
                    }
                },
            )
            .unwrap();
    }

    fn bidi_streaming(
        &mut self,
        _ctx: ::grpcio::RpcContext,
        stream: ::grpcio::RequestStream<Request>,
        sink: ::grpcio::DuplexSink<Response>,
    ) {
        // used to stream response messages back to the client
        let (mut tx, rx) = execution::futures::channel::mpsc::channel(0);
        let write_flags = WriteFlags::default();

        let mut tx2 = tx.clone();
        // receive all client request messages that are streamed
        self.executor
            .spawn(
                async move {
                    let mut stream = stream.compat();
                    while let Some(request) = await!(stream.next()) {
                        match request {
                            Ok(request) => {
                                let mut response = Response::new();
                                let msg: Result<(Response, WriteFlags), grpcio::Error> =
                                    Ok((response, write_flags));
                                let _ = await!(tx2.send(msg));
                            }
                            Err(_) => return,
                        }
                    }
                },
            )
            .unwrap();

        {
            use futures::prelude::Sink;
            let send_all = sink.send_all(Compat::new(rx)).compat();
            self.executor
                .spawn(
                    async move {
                        let _ = await!(send_all);
                    },
                )
                .unwrap();
        }
        self.executor
            .spawn(
                async move {
                    for i in 0..10 {
                        let mut response = Response::new();
                        let msg: Result<(Response, WriteFlags), grpcio::Error> =
                            Ok((response, write_flags));
                        let _ = await!(tx.send(msg));
                    }
                },
            )
            .unwrap();
    }
}
