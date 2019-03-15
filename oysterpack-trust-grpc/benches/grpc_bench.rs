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

criterion_group!(benches, grpc_bench, grpc_secure_bench);

criterion_main!(benches);

fn grpc_bench(c: &mut Criterion) {
    let server = start_server();

    for &(ref host, port) in server.bind_addrs() {
        println!(
            "grpc_unary_run_bench(): grpc server listening on {}:{}",
            host, port
        );

        let env = Arc::new(EnvBuilder::new().build());
        let ch = ChannelBuilder::new(env).connect(format!("{}:{}", host, port).as_str());
        let client = foo_grpc::FooClient::new(ch);

        {
            let client = client.clone();
            c.bench_function("grpc_unary_futures_01_bench", move |b| {
                b.iter(|| {
                    let request = Request::new();
                    client.unary(&request).unwrap();
                });
            });
        }

        {
            let client = client.clone();
            c.bench_function("grpc_unary_futures_03_bench", move |b| {
                b.iter(|| {
                    let mut request = Request::new();
                    request.futures_version = Request_Futures::THREE;
                    client.unary(&request).unwrap();
                });
            });
        }

        {
            let client = client.clone();
            c.bench_function("grpc_unary_async_01_bench", move |b| {
                b.iter(|| {
                    let request = Request::new();
                    let reply_receiver = client.unary_async(&request).unwrap();
                    global_executor()
                        .run(async move { await!(reply_receiver.compat()) })
                        .unwrap();
                });
            });
        }

        {
            let client = client.clone();
            c.bench_function("grpc_unary_async_03_bench", move |b| {
                b.iter(|| {
                    let mut request = Request::new();
                    request.futures_version = Request_Futures::THREE;
                    let reply_receiver = client.unary_async(&request).unwrap();
                    global_executor()
                        .run(async move { await!(reply_receiver.compat()) })
                        .unwrap();
                });
            });
        }
    }

    stop_server(server);
}

fn grpc_secure_bench(c: &mut Criterion) {
    let (server, cert_pem) = start_secure_server();

    for &(ref host, port) in server.bind_addrs() {
        println!(
            "grpc_unary_run_bench(): grpc server listening on {}:{}",
            host, port
        );

        let channel_credentials = grpcio::ChannelCredentialsBuilder::new()
            .root_cert(cert_pem.as_bytes().to_vec())
            .build();

        let env = Arc::new(EnvBuilder::new().build());
        let ch = ChannelBuilder::new(env)
            .secure_connect(format!("{}:{}", host, port).as_str(), channel_credentials);
        let client = foo_grpc::FooClient::new(ch);

        {
            let client = client.clone();
            c.bench_function("grpc_secure_unary_futures_01_bench", move |b| {
                b.iter(|| {
                    let request = Request::new();
                    client.unary(&request).unwrap();
                });
            });
        }

        {
            let client = client.clone();
            c.bench_function("grpc_secure_unary_futures_03_bench", move |b| {
                b.iter(|| {
                    let mut request = Request::new();
                    request.futures_version = Request_Futures::THREE;
                    client.unary(&request).unwrap();
                });
            });
        }

        {
            let client = client.clone();
            c.bench_function("grpc_secure_unary_async_futures_01_bench", move |b| {
                b.iter(|| {
                    let request = Request::new();
                    let reply_receiver = client.unary_async(&request).unwrap();
                    global_executor()
                        .run(async move { await!(reply_receiver.compat()) })
                        .unwrap();
                });
            });
        }

        {
            let client = client.clone();
            c.bench_function("grpc_secure_unary_async_futures_03_bench", move |b| {
                b.iter(|| {
                    let mut request = Request::new();
                    request.futures_version = Request_Futures::THREE;
                    let reply_receiver = client.unary_async(&request).unwrap();
                    global_executor()
                        .run(async move { await!(reply_receiver.compat()) })
                        .unwrap();
                });
            });
        }
    }

    stop_server(server);
}

fn start_server() -> grpcio::Server {
    let env = Arc::new(Environment::new(num_cpus::get()));
    let service = foo_grpc::create_foo(FooServer::default());
    let mut server = ServerBuilder::new(env)
        .register_service(service)
        .bind("127.0.0.1", 0)
        .build()
        .unwrap();
    server.start();
    server
}

fn start_secure_server() -> (grpcio::Server, String) {
    let subject_alt_names: &[_] = &["127.0.0.1".to_string()];

    let cert = rcgen::generate_simple_self_signed(subject_alt_names);
    let cert_pem = cert.serialize_pem();
    let cert_private_key_pem = cert.serialize_private_key_pem();
    let server_credentials = grpcio::ServerCredentialsBuilder::new()
        .add_cert(
            cert_pem.as_bytes().to_vec(),
            cert_private_key_pem.as_bytes().to_vec(),
        )
        .build();

    let env = Arc::new(Environment::new(1));
    let service = foo_grpc::create_foo(FooServer::default());
    let mut server = ServerBuilder::new(env)
        .register_service(service)
        .bind_secure("127.0.0.1", 0, server_credentials)
        .build()
        .unwrap();
    server.start();
    (server, cert_pem)
}

fn stop_server(mut server: grpcio::Server) {
    use futures::Future;
    if let Err(err) = server.shutdown().wait() {
        println!("Error occurred while shutting down server: {:?}", err);
    }
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
    fn unary(&mut self, ctx: grpcio::RpcContext, req: Request, sink: grpcio::UnarySink<Response>) {
        match req.futures_version {
            Request_Futures::ONE => {
                use futures::Future;
                ctx.spawn(sink.success(Response::new()).map_err(|err| {
                    println!("unary(): failed to send response: {:?}", err);
                }));
            }
            Request_Futures::THREE => {
                self.executor
                    .spawn(
                        async move {
                            use futures::Future;
                            let _ = await!(sink.success(Response::new()).compat());
                        },
                    )
                    .unwrap();
            }
        }
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
