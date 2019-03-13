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

use crate::protos::{
    message::*,
    metrics::*,
    metrics_grpc::{self, *},
};

use oysterpack_trust::concurrent::{
    execution::{
        self,
        futures::{
            compat::{Compat, Future01CompatExt, Stream01CompatExt},
            sink::{Sink, SinkExt},
            stream::{Stream, StreamExt},
            task::SpawnExt,
        },
        global_executor,
    },
    messaging::reqrep,
};

use grpcio::{ChannelBuilder, EnvBuilder, Environment, ServerBuilder, WriteFlags};
use std::sync::Arc;

use futures::future::Future;

#[derive(Clone)]
struct FooServer;

impl Foo for FooServer {
    fn unary(
        &mut self,
        ctx: grpcio::RpcContext,
        req: super::metrics::Request,
        sink: grpcio::UnarySink<super::metrics::Response>,
    ) {
        let mut response = Response::new();
        response.set_id(1);
        global_executor().spawn(
            async move {
                sink.success(response);
            },
        );
    }

    fn client_streaming(
        &mut self,
        ctx: ::grpcio::RpcContext,
        stream: ::grpcio::RequestStream<super::metrics::Request>,
        sink: ::grpcio::ClientStreamingSink<super::metrics::Response>,
    ) {
        global_executor().spawn(
            async move {
                let mut id = 0;
                let mut stream = stream.compat();
                while let Some(request) = await!(stream.next()) {
                    println!("client_streaming(): request = {:?}", request);
                    id = request.unwrap().id;
                }
                let mut response = Response::new();
                response.set_id(id);
                sink.success(response);
            },
        );
    }

    fn server_streaming(
        &mut self,
        ctx: ::grpcio::RpcContext,
        req: super::metrics::Request,
        sink: ::grpcio::ServerStreamingSink<super::metrics::Response>,
    ) {
        println!("server_streaming() request: {:?}", req);
        let (mut tx, rx) = execution::futures::channel::mpsc::channel(0);
        {
            use futures::prelude::{Sink, Stream};
            let send_all = sink.send_all(Compat::new(rx)).compat();
            global_executor().spawn(
                async move {
                    let _ = await!(send_all);
                },
            );
        }

        global_executor().spawn(
            async move {
                let write_flags = WriteFlags::default();
                for i in 0..10 {
                    let mut response = Response::new();
                    response.id = i as u64;
                    let msg: Result<(Response, WriteFlags), grpcio::Error> =
                        Ok((response, write_flags));
                    let _ = await!(tx.send(msg));
                }
            },
        );
    }

    fn bidi_streaming(
        &mut self,
        ctx: ::grpcio::RpcContext,
        stream: ::grpcio::RequestStream<super::metrics::Request>,
        sink: ::grpcio::DuplexSink<super::metrics::Response>,
    ) {
        global_executor().spawn(
            async move {
                let mut id = 0_u64;
                let mut stream = stream.compat();
                while let Some(request) = await!(stream.next()) {
                    println!("bidi_streaming(): server request = {:?}", request);
                    match request {
                        Ok(request) => {
                            id = request.id;
                        }
                        Err(_) => return,
                    }
                }
            },
        );

        let (mut tx, rx) = execution::futures::channel::mpsc::channel(0);
        {
            use futures::prelude::{Sink, Stream};
            let send_all = sink.send_all(Compat::new(rx)).compat();
            global_executor().spawn(
                async move {
                    let _ = await!(send_all);
                },
            );
        }

        global_executor().spawn(
            async move {
                let write_flags = WriteFlags::default();
                for i in 0..10 {
                    let mut response = Response::new();
                    response.id = i as u64;
                    let msg: Result<(Response, WriteFlags), grpcio::Error> =
                        Ok((response, write_flags));
                    let _ = await!(tx.send(msg));
                }
            },
        );
    }
}

#[test]
fn grpc_unary() {
    let env = Arc::new(Environment::new(1));
    let service = metrics_grpc::create_foo(FooServer);
    let mut server = ServerBuilder::new(env)
        .register_service(service)
        .bind("127.0.0.1", 0)
        .build()
        .unwrap();
    server.start();

    for &(ref host, port) in server.bind_addrs() {
        println!("listening on {}:{}", host, port);

        let env = Arc::new(EnvBuilder::new().build());
        let ch = ChannelBuilder::new(env).connect(format!("{}:{}", host, port).as_str());
        let client = metrics_grpc::FooClient::new(ch);
        let request = Request::new();
        let response = client.unary(&request).unwrap();
        println!("grpc_unary(): response = {:?}", response);
    }

    println!("server is shutting down ...");
    let _ = server.shutdown().wait();
    println!("server has been shutdown")
}

#[test]
fn grpc_unary_async() {
    let env = Arc::new(Environment::new(1));
    let service = metrics_grpc::create_foo(FooServer);
    let mut server = ServerBuilder::new(env)
        .register_service(service)
        .bind("127.0.0.1", 0)
        .build()
        .unwrap();
    server.start();

    for &(ref host, port) in server.bind_addrs() {
        println!("listening on {}:{}", host, port);

        let env = Arc::new(EnvBuilder::new().build());
        let ch = ChannelBuilder::new(env).connect(format!("{}:{}", host, port).as_str());
        let client = metrics_grpc::FooClient::new(ch);
        let request = Request::new();
        let reply_receiver = client.unary_async(&request).unwrap();

        let (tx, rx) = execution::futures::channel::oneshot::channel();
        global_executor().spawn(
            async move {
                let response = await!(reply_receiver.compat()).unwrap();
                tx.send(response);
            },
        );

        let response = global_executor().run(rx).unwrap();
        println!("grpc_unary_async(): response = {:?}", response);
    }

    println!("server is shutting down ...");
    let _ = server.shutdown().wait();
    println!("server has been shutdown")
}

#[test]
fn client_streaming() {
    let env = Arc::new(Environment::new(1));
    let service = metrics_grpc::create_foo(FooServer);
    let mut server = ServerBuilder::new(env)
        .register_service(service)
        .bind("127.0.0.1", 0)
        .build()
        .unwrap();
    server.start();

    for &(ref host, port) in server.bind_addrs() {
        println!("listening on {}:{}", host, port);

        let env = Arc::new(EnvBuilder::new().build());
        let ch = ChannelBuilder::new(env).connect(format!("{}:{}", host, port).as_str());
        let client = metrics_grpc::FooClient::new(ch);
        let (sender, receiver) = client.client_streaming().unwrap();
        let (mut tx, rx) = execution::futures::channel::mpsc::channel(0);
        {
            use futures::prelude::{Sink, Stream};
            let send_all = sender.send_all(Compat::new(rx)).compat();
            global_executor().spawn(
                async move {
                    let _ = await!(send_all);
                },
            );
        }
        global_executor().spawn(
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
        );
        let receiver = receiver.compat();
        let response = global_executor().run(receiver).unwrap();
        println!("client_streaming(): response = {:?}", response);
    }

    println!("server is shutting down ...");
    let _ = server.shutdown().wait();
    println!("server has been shutdown")
}

#[test]
fn server_streaming() {
    let env = Arc::new(Environment::new(1));
    let service = metrics_grpc::create_foo(FooServer);
    let mut server = ServerBuilder::new(env)
        .register_service(service)
        .bind("127.0.0.1", 0)
        .build()
        .unwrap();
    server.start();

    for &(ref host, port) in server.bind_addrs() {
        println!("listening on {}:{}", host, port);

        let env = Arc::new(EnvBuilder::new().build());
        let ch = ChannelBuilder::new(env).connect(format!("{}:{}", host, port).as_str());
        let client = metrics_grpc::FooClient::new(ch);
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

    println!("server is shutting down ...");
    let _ = server.shutdown().wait();
    println!("server has been shutdown")
}

#[test]
fn bidi_streaming() {
    let env = Arc::new(Environment::new(1));
    let service = metrics_grpc::create_foo(FooServer);
    let mut server = ServerBuilder::new(env)
        .register_service(service)
        .bind("127.0.0.1", 0)
        .build()
        .unwrap();
    server.start();

    for &(ref host, port) in server.bind_addrs() {
        println!("listening on {}:{}", host, port);

        let env = Arc::new(EnvBuilder::new().build());
        let ch = ChannelBuilder::new(env).connect(format!("{}:{}", host, port).as_str());
        let client = metrics_grpc::FooClient::new(ch);
        let (sender, receiver) = client.bidi_streaming().unwrap();

        let (mut tx, rx) = execution::futures::channel::mpsc::channel(0);
        {
            use futures::prelude::{Sink, Stream};
            let send_all = sender.send_all(Compat::new(rx)).compat();
            global_executor().spawn(
                async move {
                    let _ = await!(send_all);
                    println!("bidi_streaming(): client sent all requests");
                },
            );
        }
        global_executor().spawn(
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
        );

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

    println!("server is shutting down ...");
    let _ = server.shutdown().wait();
    println!("server has been shutdown")
}
