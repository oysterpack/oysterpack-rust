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
use std::time::Duration;
use std::{sync::Arc, thread};

/// converts a futures 0.1 Stream into a futures 0.3 Stream
pub fn into_stream03<S>(
    stream01: S,
) -> impl execution::futures::Stream<
    Item = Result<<S as futures::Stream>::Item, <S as futures::Stream>::Error>,
>
where
    S: futures::Stream,
{
    Compat01As03::new(stream01)
}

/// converts a futures 0.1 Future into a futures 0.3 Future
pub fn into_future03<F>(
    future01: F,
) -> impl execution::futures::Future<
    Output = Result<<F as futures::Future>::Item, <F as futures::Future>::Error>,
>
where
    F: futures::Future,
{
    Compat01As03::new(future01)
}

fn format_rpc_context(ctx: &grpcio::RpcContext) -> String {
    let request_headers =
        ctx.request_headers()
            .iter()
            .fold(HashMap::new(), |mut map, (key, value)| {
                map.insert(key.to_string(), String::from_utf8_lossy(value));
                map
            });
    format!(
        "[{:?}]: method = {}, host = {}, peer = {}, headers = {:#?}",
        thread::current()
            .name()
            .unwrap_or(format!("{:?}", thread::current().id()).as_str()),
        String::from_utf8_lossy(ctx.method()),
        String::from_utf8_lossy(ctx.host()),
        ctx.peer(),
        request_headers
    )
}

#[derive(Clone)]
struct FooServer;

impl Foo for FooServer {
    fn unary(&mut self, ctx: grpcio::RpcContext, req: Request, sink: grpcio::UnarySink<Response>) {
        println!("unary(): {}", format_rpc_context(&ctx));
        println!("unary(): request: {:?}", req);
        let ulid_key = ctx.request_headers().iter().find_map(|(key, value)| {
            if key == "key-bin" {
                Some(ULID::try_from_bytes(value).unwrap())
            } else {
                None
            }
        });
        println!("unary(): ulid_key = {:?}", ulid_key);

        let mut response = Response::new();
        response.set_id(req.id + 1);
        let sleep_duration = Duration::from_millis(req.sleep);
        global_executor()
            .spawn(
                async move {
                    println!("unary(): sleeping for {:?} ...", sleep_duration);
                    thread::sleep(sleep_duration);
                    use futures::Future;
                    let _ = await!(sink.success(response.clone()).compat());
                    println!(
                        "[{:?}]: unary(): sent response: {:?}",
                        thread::current().id(),
                        response
                    );
                },
            )
            .unwrap();
    }

    fn client_streaming(
        &mut self,
        ctx: ::grpcio::RpcContext,
        stream: ::grpcio::RequestStream<Request>,
        sink: ::grpcio::ClientStreamingSink<Response>,
    ) {
        println!("client_streaming(): {}", format_rpc_context(&ctx));
        global_executor()
            .spawn(
                async move {
                    let mut id = 0;
                    // receive all client request messages
                    let mut stream = stream.compat();
                    while let Some(request) = await!(stream.next()) {
                        println!("client_streaming(): request = {:?}", request);
                        id = request.unwrap().id;
                    }
                    // once all messages have been received, then send the response
                    let mut response = Response::new();
                    response.set_id(id);
                    sink.success(response);
                },
            )
            .unwrap();
    }

    fn server_streaming(
        &mut self,
        ctx: ::grpcio::RpcContext,
        req: Request,
        sink: ::grpcio::ServerStreamingSink<Response>,
    ) {
        println!("server_streaming(): {}", format_rpc_context(&ctx));
        println!("server_streaming() request: {:?}", req);
        // the design is to stream messages using channels
        // ServerStreamingSink will send all messages received on the mpsc:Receiver stream
        let (mut tx, rx) = execution::futures::channel::mpsc::channel(0);
        {
            use futures::prelude::Sink;
            let send_all = sink.send_all(Compat::new(rx));
            global_executor()
                .spawn(
                    async move {
                        let _ = await!(send_all.compat());
                    },
                )
                .unwrap();
        }

        // deliver messages via mpsc::Sender
        global_executor()
            .spawn(
                async move {
                    let write_flags = WriteFlags::default();
                    for i in 0..10 {
                        let mut response = Response::new();
                        response.id = i as u64;
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
        ctx: ::grpcio::RpcContext,
        stream: ::grpcio::RequestStream<Request>,
        sink: ::grpcio::DuplexSink<Response>,
    ) {
        println!("bidi_streaming(): {}", format_rpc_context(&ctx));
        // used to stream response messages back to the client
        let (mut tx, rx) = execution::futures::channel::mpsc::channel(0);
        let write_flags = WriteFlags::default();

        let mut tx2 = tx.clone();
        // receive all client request messages that are streamed
        global_executor()
            .spawn(
                async move {
                    let mut stream = stream.compat();
                    while let Some(request) = await!(stream.next()) {
                        println!("bidi_streaming(): server request = {:?}", request);
                        match request {
                            Ok(request) => {
                                let mut response = Response::new();
                                response.id = request.id + 100;
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
            global_executor()
                .spawn(
                    async move {
                        let _ = await!(send_all);
                    },
                )
                .unwrap();
        }
        global_executor()
            .spawn(
                async move {
                    for i in 0..10 {
                        let mut response = Response::new();
                        response.id = i as u64;
                        let msg: Result<(Response, WriteFlags), grpcio::Error> =
                            Ok((response, write_flags));
                        let _ = await!(tx.send(msg));
                    }
                },
            )
            .unwrap();
    }
}

fn start_server() -> grpcio::Server {
    let env = Arc::new(Environment::new(num_cpus::get()));
    let service = foo_grpc::create_foo(FooServer);
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
    let service = foo_grpc::create_foo(FooServer);
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

#[test]
fn grpc_unary() {
    let server = start_server();

    for &(ref host, port) in server.bind_addrs() {
        println!("listening on {}:{}", host, port);

        let env = Arc::new(EnvBuilder::new().build());
        let ch = ChannelBuilder::new(env).connect(format!("{}:{}", host, port).as_str());
        let client = foo_grpc::FooClient::new(ch);
        let request = Request::new();
        let response = client.unary(&request).unwrap();
        println!("grpc_unary(): response = {:?}", response);
    }

    stop_server(server);
}

#[test]
fn grpc_unary_secure() {
    let (server, cert_pem) = start_secure_server();

    for &(ref host, port) in server.bind_addrs() {
        println!("listening on {}:{}", host, port);

        let channel_credentials = grpcio::ChannelCredentialsBuilder::new()
            .root_cert(cert_pem.as_bytes().to_vec())
            .build();

        let env = Arc::new(EnvBuilder::new().build());
        let ch = ChannelBuilder::new(env)
            .secure_connect(format!("{}:{}", host, port).as_str(), channel_credentials);
        let client = foo_grpc::FooClient::new(ch);
        let request = Request::new();
        let response = client.unary(&request).unwrap();
        println!("grpc_unary(): response = {:?}", response);
    }

    stop_server(server);
}

#[test]
fn grpc_unary_async() {
    let server = start_server();

    for &(ref host, port) in server.bind_addrs() {
        println!("listening on {}:{}", host, port);

        let env = Arc::new(EnvBuilder::new().build());
        let ch = ChannelBuilder::new(env).connect(format!("{}:{}", host, port).as_str());
        let client = foo_grpc::FooClient::new(ch);
        let request = Request::new();

        let call_opt = {
            let call_opt = grpcio::CallOption::default();
            let mut headers = grpcio::MetadataBuilder::new();
            headers.add_str("key", "value").unwrap();
            let ulid = ULID::generate();
            headers.add_bytes("key-bin", &ulid.to_bytes()).unwrap();
            let call_opt = call_opt.headers(headers.build());
            call_opt
        };

        let reply_receiver = client.unary_async_opt(&request, call_opt).unwrap();

        let (tx, rx) = execution::futures::channel::oneshot::channel();
        global_executor()
            .spawn(
                async move {
                    let response = await!(reply_receiver.compat()).unwrap();
                    let _ = tx.send(response);
                },
            )
            .unwrap();

        let response = global_executor().run(rx).unwrap();
        println!("grpc_unary_async(): response = {:?}", response);
    }

    stop_server(server);
}

#[test]
fn grpc_unary_async_send_next_req_before_receiving_reply() {
    let server = start_server();

    for &(ref host, port) in server.bind_addrs() {
        println!("listening on {}:{}", host, port);

        let env = Arc::new(EnvBuilder::new().build());
        let ch = ChannelBuilder::new(env).connect(format!("{}:{}", host, port).as_str());
        let client = foo_grpc::FooClient::new(ch);

        for i in 1..=10 {
            let mut request = Request::new();
            request.id = i;

            // Given: an async request has been sent
            request.sleep = 0;
            let reply_receiver = client.unary_async(&request).unwrap();
            // And: a sync request is sent before receiving the async reply
            request.id = i + 1;
            request.sleep = 10;
            let response = client.unary(&request).unwrap();
            println!(
                "grpc_unary_async_send_next_req_before_receiving_reply(): sync response = {:?}",
                response
            );

            // Then: the async response can be retrieved after receiving the sync response
            let response = global_executor().run(reply_receiver.compat()).unwrap();
            println!(
                "grpc_unary_async_send_next_req_before_receiving_reply(): async response = {:?}",
                response
            );
        }
    }

    stop_server(server);
}

#[test]
fn grpc_unary_async_timeout() {
    let server = start_server();

    for &(ref host, port) in server.bind_addrs() {
        println!("listening on {}:{}", host, port);

        let env = Arc::new(EnvBuilder::new().build());
        let ch = ChannelBuilder::new(env).connect(format!("{}:{}", host, port).as_str());
        let client = foo_grpc::FooClient::new(ch);

        let mut request = Request::new();
        request.sleep = 50;

        let call_opt = {
            let call_opt = grpcio::CallOption::default();
            let mut headers = grpcio::MetadataBuilder::new();
            headers.add_str("key", "value").unwrap();
            let ulid = ULID::generate();
            headers.add_bytes("key-bin", &ulid.to_bytes()).unwrap();
            let call_opt = call_opt.headers(headers.build());
            let call_opt = call_opt.timeout(Duration::from_millis(10));
            call_opt
        };

        let reply_receiver = client.unary_async_opt(&request, call_opt).unwrap();

        let (tx, rx) = execution::futures::channel::oneshot::channel();
        global_executor()
            .spawn(
                async move {
                    let response = await!(reply_receiver.compat());
                    let _ = tx.send(response);
                },
            )
            .unwrap();
        let response = global_executor().run(rx).unwrap();
        println!("grpc_unary_async_timeout(): response = {:?}", response);
    }

    stop_server(server);
}

#[test]
fn grpc_unary_async_no_timeout() {
    let server = start_server();

    for &(ref host, port) in server.bind_addrs() {
        println!("listening on {}:{}", host, port);

        let env = Arc::new(EnvBuilder::new().build());
        let ch = ChannelBuilder::new(env).connect(format!("{}:{}", host, port).as_str());
        let client = foo_grpc::FooClient::new(ch);

        let mut request = Request::new();
        request.sleep = 1;

        let call_opt = {
            let call_opt = grpcio::CallOption::default();
            let mut headers = grpcio::MetadataBuilder::new();
            headers.add_str("key", "value").unwrap();
            let ulid = ULID::generate();
            headers.add_bytes("key-bin", &ulid.to_bytes()).unwrap();
            let call_opt = call_opt.headers(headers.build());
            let call_opt = call_opt.timeout(Duration::from_millis(100));
            call_opt
        };

        let reply_receiver = client.unary_async_opt(&request, call_opt).unwrap();

        let (tx, rx) = execution::futures::channel::oneshot::channel();
        global_executor()
            .spawn(
                async move {
                    let response = await!(reply_receiver.compat());
                    let _ = tx.send(response);
                },
            )
            .unwrap();
        let response = global_executor().run(rx).unwrap();
        println!(
            "grpc_unary_async_no_timeout(): response = {:?}",
            response.unwrap()
        );
    }

    stop_server(server);
}

#[test]
fn client_streaming() {
    let server = start_server();

    for &(ref host, port) in server.bind_addrs() {
        println!("listening on {}:{}", host, port);

        let env = Arc::new(EnvBuilder::new().build());
        let ch = ChannelBuilder::new(env).connect(format!("{}:{}", host, port).as_str());
        let client = foo_grpc::FooClient::new(ch);
        let (sender, receiver) = client.client_streaming().unwrap();
        let (mut tx, rx) = execution::futures::channel::mpsc::channel(0);
        {
            use futures::prelude::Sink;
            let send_all = sender.send_all(Compat::new(rx)).compat();
            global_executor()
                .spawn(
                    async move {
                        let _ = await!(send_all);
                    },
                )
                .unwrap();
        }
        global_executor()
            .spawn(
                async move {
                    let write_flags = WriteFlags::default();
                    for i in 0..10 {
                        let mut request = Request::new();
                        request.id = i;
                        let msg: Result<(Request, WriteFlags), grpcio::Error> =
                            Ok((request, write_flags));
                        let _ = await!(tx.send(msg));
                    }
                },
            )
            .unwrap();
        let receiver = receiver.compat();
        let response = global_executor().run(receiver).unwrap();
        println!("client_streaming(): response = {:?}", response);
    }

    stop_server(server);
}

#[test]
fn server_streaming() {
    let server = start_server();

    for &(ref host, port) in server.bind_addrs() {
        println!("listening on {}:{}", host, port);

        let env = Arc::new(EnvBuilder::new().build());
        let ch = ChannelBuilder::new(env).connect(format!("{}:{}", host, port).as_str());
        let client = foo_grpc::FooClient::new(ch);
        let request = Request::new();
        let receiver = client.server_streaming(&request).unwrap();
        let mut receiver = receiver.compat();
        global_executor().run(
            async move {
                while let Some(response) = await!(receiver.next()) {
                    println!("server_streaming(): response = {:?}", response);
                }
                println!("server_streaming(): DONE");
            },
        );
    }

    stop_server(server);
}

#[test]
fn bidi_streaming() {
    let server = start_server();

    for &(ref host, port) in server.bind_addrs() {
        println!("listening on {}:{}", host, port);

        let env = Arc::new(EnvBuilder::new().build());
        let ch = ChannelBuilder::new(env).connect(format!("{}:{}", host, port).as_str());
        let client = foo_grpc::FooClient::new(ch);
        let (sender, receiver) = client.bidi_streaming().unwrap();

        let (mut tx, rx) = execution::futures::channel::mpsc::channel(0);
        {
            use futures::prelude::Sink;
            let send_all = sender.send_all(Compat::new(rx)).compat();
            global_executor()
                .spawn(
                    async move {
                        let _ = await!(send_all);
                        println!("bidi_streaming(): client sent all requests");
                    },
                )
                .unwrap();
        }
        global_executor()
            .spawn(
                async move {
                    let write_flags = WriteFlags::default();
                    for i in 0..10 {
                        let mut request = Request::new();
                        request.id = i;
                        let msg: Result<(Request, WriteFlags), grpcio::Error> =
                            Ok((request.clone(), write_flags));
                        let _ = await!(tx.send(msg));
                        println!("bidi_streaming(): client sent request: {:?}", request);
                    }
                },
            )
            .unwrap();

        let mut receiver = receiver.compat();
        global_executor().run(
            async move {
                while let Some(response) = await!(receiver.next()) {
                    println!("bidi_streaming(): client response = {:?}", response);
                }
                println!("bidi_streaming(): client received all responses");
            },
        );
    }

    stop_server(server);
}
