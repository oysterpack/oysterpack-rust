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

// This file is generated. Do not edit
// @generated

// https://github.com/Manishearth/rust-clippy/issues/702
#![allow(unknown_lints)]
#![allow(clippy)]

#![cfg_attr(rustfmt, rustfmt_skip)]

#![allow(box_pointers)]
#![allow(dead_code)]
#![allow(missing_docs)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(trivial_casts)]
#![allow(unsafe_code)]
#![allow(unused_imports)]
#![allow(unused_results)]

const METHOD_FOO_UNARY: ::grpcio::Method<super::foo::Request, super::foo::Response> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/oysterpack_trust_grpc.protos.foo.Foo/unary",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_FOO_CLIENT_STREAMING: ::grpcio::Method<super::foo::Request, super::foo::Response> = ::grpcio::Method {
    ty: ::grpcio::MethodType::ClientStreaming,
    name: "/oysterpack_trust_grpc.protos.foo.Foo/client_streaming",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_FOO_SERVER_STREAMING: ::grpcio::Method<super::foo::Request, super::foo::Response> = ::grpcio::Method {
    ty: ::grpcio::MethodType::ServerStreaming,
    name: "/oysterpack_trust_grpc.protos.foo.Foo/server_streaming",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_FOO_BIDI_STREAMING: ::grpcio::Method<super::foo::Request, super::foo::Response> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Duplex,
    name: "/oysterpack_trust_grpc.protos.foo.Foo/bidi_streaming",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

#[derive(Clone)]
pub struct FooClient {
    client: ::grpcio::Client,
}

impl FooClient {
    pub fn new(channel: ::grpcio::Channel) -> Self {
        FooClient {
            client: ::grpcio::Client::new(channel),
        }
    }

    pub fn unary_opt(&self, req: &super::foo::Request, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::foo::Response> {
        self.client.unary_call(&METHOD_FOO_UNARY, req, opt)
    }

    pub fn unary(&self, req: &super::foo::Request) -> ::grpcio::Result<super::foo::Response> {
        self.unary_opt(req, ::grpcio::CallOption::default())
    }

    pub fn unary_async_opt(&self, req: &super::foo::Request, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::foo::Response>> {
        self.client.unary_call_async(&METHOD_FOO_UNARY, req, opt)
    }

    pub fn unary_async(&self, req: &super::foo::Request) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::foo::Response>> {
        self.unary_async_opt(req, ::grpcio::CallOption::default())
    }

    pub fn client_streaming_opt(&self, opt: ::grpcio::CallOption) -> ::grpcio::Result<(::grpcio::ClientCStreamSender<super::foo::Request>, ::grpcio::ClientCStreamReceiver<super::foo::Response>)> {
        self.client.client_streaming(&METHOD_FOO_CLIENT_STREAMING, opt)
    }

    pub fn client_streaming(&self) -> ::grpcio::Result<(::grpcio::ClientCStreamSender<super::foo::Request>, ::grpcio::ClientCStreamReceiver<super::foo::Response>)> {
        self.client_streaming_opt(::grpcio::CallOption::default())
    }

    pub fn server_streaming_opt(&self, req: &super::foo::Request, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientSStreamReceiver<super::foo::Response>> {
        self.client.server_streaming(&METHOD_FOO_SERVER_STREAMING, req, opt)
    }

    pub fn server_streaming(&self, req: &super::foo::Request) -> ::grpcio::Result<::grpcio::ClientSStreamReceiver<super::foo::Response>> {
        self.server_streaming_opt(req, ::grpcio::CallOption::default())
    }

    pub fn bidi_streaming_opt(&self, opt: ::grpcio::CallOption) -> ::grpcio::Result<(::grpcio::ClientDuplexSender<super::foo::Request>, ::grpcio::ClientDuplexReceiver<super::foo::Response>)> {
        self.client.duplex_streaming(&METHOD_FOO_BIDI_STREAMING, opt)
    }

    pub fn bidi_streaming(&self) -> ::grpcio::Result<(::grpcio::ClientDuplexSender<super::foo::Request>, ::grpcio::ClientDuplexReceiver<super::foo::Response>)> {
        self.bidi_streaming_opt(::grpcio::CallOption::default())
    }
    pub fn spawn<F>(&self, f: F) where F: ::futures::Future<Item = (), Error = ()> + Send + 'static {
        self.client.spawn(f)
    }
}

pub trait Foo {
    fn unary(&mut self, ctx: ::grpcio::RpcContext, req: super::foo::Request, sink: ::grpcio::UnarySink<super::foo::Response>);
    fn client_streaming(&mut self, ctx: ::grpcio::RpcContext, stream: ::grpcio::RequestStream<super::foo::Request>, sink: ::grpcio::ClientStreamingSink<super::foo::Response>);
    fn server_streaming(&mut self, ctx: ::grpcio::RpcContext, req: super::foo::Request, sink: ::grpcio::ServerStreamingSink<super::foo::Response>);
    fn bidi_streaming(&mut self, ctx: ::grpcio::RpcContext, stream: ::grpcio::RequestStream<super::foo::Request>, sink: ::grpcio::DuplexSink<super::foo::Response>);
}

pub fn create_foo<S: Foo + Send + Clone + 'static>(s: S) -> ::grpcio::Service {
    let mut builder = ::grpcio::ServiceBuilder::new();
    let mut instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_FOO_UNARY, move |ctx, req, resp| {
        instance.unary(ctx, req, resp)
    });
    let mut instance = s.clone();
    builder = builder.add_client_streaming_handler(&METHOD_FOO_CLIENT_STREAMING, move |ctx, req, resp| {
        instance.client_streaming(ctx, req, resp)
    });
    let mut instance = s.clone();
    builder = builder.add_server_streaming_handler(&METHOD_FOO_SERVER_STREAMING, move |ctx, req, resp| {
        instance.server_streaming(ctx, req, resp)
    });
    let mut instance = s.clone();
    builder = builder.add_duplex_streaming_handler(&METHOD_FOO_BIDI_STREAMING, move |ctx, req, resp| {
        instance.bidi_streaming(ctx, req, resp)
    });
    builder.build()
}
