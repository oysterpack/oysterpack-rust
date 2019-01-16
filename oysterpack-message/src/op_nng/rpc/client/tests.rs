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

//! client module tests

use super::{
    asyncio::{AioState, AsyncClient, ReplyHandler},
    syncio::{self, SyncClient},
    ClientSocketSettings, DialerSettings,
};
use crate::op_nng::{
    errors::AioReceiveError,
    new_aio_context,
    rpc::{
        server::{ListenerSettings, Server},
        MessageProcessor, MessageProcessorFactory,
    },
    try_from_nng_message, try_into_nng_message,
};
use log::*;
use oysterpack_errors::{op_error, Error};
use oysterpack_uid::ULID;
use serde::{Deserialize, Serialize};
use std::{
    num::NonZeroUsize,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

#[derive(Debug, Clone, Default)]
struct TestProcessor;

impl MessageProcessorFactory<TestProcessor, nng::Message, nng::Message> for TestProcessor {
    fn new(&self) -> TestProcessor {
        TestProcessor
    }
}

impl MessageProcessor<nng::Message, nng::Message> for TestProcessor {
    fn process(&mut self, req: nng::Message) -> nng::Message {
        match try_from_nng_message::<Request>(&req).unwrap() {
            Request::Sleep(sleep_ms) if sleep_ms > 0 => {
                info!(
                    "handler({:?}) sleeping for {} ms ...",
                    thread::current().id(),
                    sleep_ms
                );
                thread::sleep_ms(sleep_ms);
                info!("handler({:?}) has awaken !!!", thread::current().id());
            }
            Request::Sleep(_) => info!("received Sleep message on {:?}", thread::current().id()),
            Request::Panic => panic!("received Panic message on {:?}", thread::current().id()),
        }
        req
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
enum Request {
    Sleep(u32),
    Panic,
}

fn log_config() -> oysterpack_log::LogConfig {
    oysterpack_log::config::LogConfigBuilder::new(oysterpack_log::Level::Info).build()
}

#[test]
fn sync_client() {
    oysterpack_log::init(log_config(), oysterpack_log::StderrLogger);
    let url = Arc::new(format!("inproc://{}", ULID::generate()));

    // start a server with 2 aio contexts
    let listener_settings =
        ListenerSettings::new(&*url.as_str()).set_aio_count(NonZeroUsize::new(2).unwrap());
    let server = Server::builder(listener_settings, TestProcessor)
        .spawn()
        .unwrap();

    let dialer_settings = DialerSettings::new(url.as_str())
        .set_non_blocking(true)
        .set_reconnect_min_time(Duration::from_millis(100))
        .set_reconnect_max_time(Duration::from_millis(100));
    let mut client1 = SyncClient::dial(dialer_settings.clone()).unwrap();
    let mut client2 = SyncClient::dial(dialer_settings.clone()).unwrap();

    let req = Request::Sleep(0);
    for _ in 0..10 {
        info!(
            "client1 reply: {:?}",
            client1.send::<_, Request>(&req).unwrap()
        );
        info!(
            "client2 reply: {:?}",
            client2.send::<_, Request>(&req).unwrap()
        );
    }

    server.stop();
    server.join();
}

#[test]
fn sync_client_shared_between_threads() {
    use std::sync::{Arc, Mutex};

    oysterpack_log::init(log_config(), oysterpack_log::StderrLogger);
    let url = Arc::new(format!("inproc://{}", ULID::generate()));

    // start a server with 2 aio contexts
    let listener_settings =
        ListenerSettings::new(&*url.as_str()).set_aio_count(NonZeroUsize::new(2).unwrap());
    let server = Server::builder(listener_settings, TestProcessor)
        .spawn()
        .unwrap();

    let dialer_settings = DialerSettings::new(url.as_str())
        .set_non_blocking(true)
        .set_reconnect_min_time(Duration::from_millis(100))
        .set_reconnect_max_time(Duration::from_millis(100));
    let client = SyncClient::dial(dialer_settings.clone()).unwrap();
    // wrap the client in an Arc<Mutex<_>> to make access threadsafe
    let client = Arc::new(Mutex::new(client));

    let mut client_thread_handles = vec![];
    for _ in 0..10 {
        let client = client.clone();
        let handle = thread::spawn(move || {
            let mut client = client.lock().unwrap();
            let req = Request::Sleep(0);
            client.send::<_, Request>(&req).unwrap()
        });
        client_thread_handles.push(handle);
    }

    for handle in client_thread_handles {
        info!("{:?}", handle.join().unwrap());
    }

    server.stop();
    server.join();
}

struct FooHandler;

impl ReplyHandler for FooHandler {
    fn on_reply(&mut self, result: Result<nng::Message, Error>) {
        info!("Foo reply: {:?}", result);
    }
}

struct Bar {
    handler: Option<Box<dyn ReplyHandler>>,
}

#[test]
fn trait_object_invoked_across_threads() {
    oysterpack_log::init(log_config(), oysterpack_log::StderrLogger);
    let handler: Box<dyn ReplyHandler> = Box::new(FooHandler);
    let handler = Mutex::new(handler);

    let handle = thread::spawn(move || {
        let mut handler = handler.lock().unwrap();
        handler.on_reply(Ok(nng::Message::new().unwrap()));
    });

    handle.join().unwrap();
}

#[test]
fn chashmap_stored_trait_object_invoked_across_threads() {
    oysterpack_log::init(log_config(), oysterpack_log::StderrLogger);
    let handler: Box<dyn ReplyHandler> = Box::new(FooHandler);
    let bar = Bar {
        handler: Some(handler),
    };
    let map = Arc::new(chashmap::CHashMap::new());
    const KEY: usize = 1;
    map.insert(KEY, bar);

    let queue = Arc::new(crossbeam::queue::SegQueue::new());
    let queue_2 = Arc::clone(&queue);

    let handle = thread::spawn(move || {
        let mut bar = map.remove(&KEY).unwrap();
        let mut handler = bar.handler.take().unwrap();
        handler.on_reply(Ok(nng::Message::new().unwrap()));
        queue_2.push(bar);
    });

    handle.join().unwrap();
    let bar = queue.try_pop().unwrap();
    assert!(bar.handler.is_none());
}

struct ReplyForwarder {
    chan: crossbeam::channel::Sender<Result<nng::Message, Error>>,
}

impl ReplyHandler for ReplyForwarder {
    fn on_reply(&mut self, result: Result<nng::Message, Error>) {
        info!("reply: {:?}", result);
        if let Err(err) = self.chan.send(result) {
            error!("failed to forward message")
        }
    }
}

#[test]
fn async_client_poc() {
    oysterpack_log::init(log_config(), oysterpack_log::StderrLogger);
    let url = Arc::new(format!("inproc://{}", ULID::generate()));

    // start a server with 2 aio contexts
    let listener_settings =
        ListenerSettings::new(&*url.as_str()).set_aio_count(NonZeroUsize::new(2).unwrap());
    let server = Server::builder(listener_settings, TestProcessor)
        .spawn()
        .unwrap();

    let dialer_settings = DialerSettings::new(url.as_str())
        .set_non_blocking(true)
        .set_reconnect_min_time(Duration::from_millis(100))
        .set_reconnect_max_time(Duration::from_millis(100));
    let client = AsyncClient::dial(dialer_settings.clone()).unwrap();

    // POC
    let (tx, rx) = crossbeam::channel::bounded(10);
    let mut cb = ReplyForwarder { chan: tx };
    use std::sync::{Arc, Mutex};
    let mut ctx_state = Arc::new(Mutex::new(AioState::Idle));
    let mut call_back_ctx_state = Arc::clone(&ctx_state);
    let ctx: nng::aio::Context = new_aio_context(client.socket()).unwrap();
    let callback_ctx = ctx.clone();
    let aio = nng::aio::Aio::with_callback(move |aio| {
        info!("aio event on context({})", callback_ctx.id());
        match aio.result().unwrap() {
            Ok(_) => {
                let mut ctx_state = call_back_ctx_state.lock().unwrap();
                match *ctx_state {
                    AioState::Send => {
                        // sending the request was successful
                        // now lets wait for the reply
                        aio.recv(&callback_ctx).unwrap();
                        *ctx_state = AioState::Recv;
                    }
                    AioState::Recv => {
                        // reply has been successfully received
                        // thus it is safe to invoke unwrap n
                        let rep = aio.get_msg().unwrap();
                        cb.on_reply(Ok(rep));
                        *ctx_state = AioState::Idle;
                    }
                    AioState::Idle => {
                        warn!("did not expect to be invoked while idle");
                    }
                }
            }
            Err(err) => {
                cb.on_reply(Err(op_error!(AioReceiveError::from(err))));
            }
        }
    })
    .unwrap();

    for i in 0..10 {
        let req = Request::Sleep(0);
        let msg = try_into_nng_message(&req).unwrap();
        {
            let mut ctx_state = ctx_state.lock().unwrap();
            *ctx_state = AioState::Send;
            aio.send(&ctx, msg).unwrap();
        }
        match rx.recv() {
            Ok(rep) => info!("received forwarded reply #{} : {:?}", i, rep),
            Err(err) => panic!("recv failed: {}", err),
        }
    }

    server.stop();
    server.join();
}

#[test]
fn async_client_poc_restart_server() {
    oysterpack_log::init(log_config(), oysterpack_log::StderrLogger);
    let url = Arc::new(format!("inproc://{}", ULID::generate()));

    fn start_server(url: &str) -> Server {
        // start a server with 2 aio contexts
        let listener_settings =
            ListenerSettings::new(url).set_aio_count(NonZeroUsize::new(2).unwrap());
        Server::builder(listener_settings, TestProcessor)
            .spawn()
            .unwrap()
    }

    let server = start_server(&*url.as_str());

    let dialer_settings = DialerSettings::new(url.as_str())
        .set_non_blocking(true)
        .set_reconnect_min_time(Duration::from_millis(100))
        .set_reconnect_max_time(Duration::from_millis(100));
    let client = AsyncClient::dial(dialer_settings.clone()).unwrap();

    // POC
    let (tx, rx) = crossbeam::channel::bounded(10);
    let mut cb = ReplyForwarder { chan: tx };
    use std::sync::{Arc, Mutex};
    let mut ctx_state = Arc::new(Mutex::new(AioState::Idle));
    let mut call_back_ctx_state = Arc::clone(&ctx_state);
    let ctx: nng::aio::Context = new_aio_context(client.socket()).unwrap();
    let callback_ctx = ctx.clone();
    let aio = nng::aio::Aio::with_callback(move |aio| {
        info!("aio event on context({})", callback_ctx.id());
        match aio.result().unwrap() {
            Ok(_) => {
                let mut ctx_state = call_back_ctx_state.lock().unwrap();
                match *ctx_state {
                    AioState::Send => {
                        // sending the request was successful
                        // now lets wait for the reply
                        aio.recv(&callback_ctx).unwrap();
                        *ctx_state = AioState::Recv;
                    }
                    AioState::Recv => {
                        // reply has been successfully received
                        // thus it is safe to invoke unwrap n
                        let rep = aio.get_msg().unwrap();
                        cb.on_reply(Ok(rep));
                        *ctx_state = AioState::Idle;
                    }
                    AioState::Idle => {
                        warn!("did not expect to be invoked while idle");
                    }
                }
            }
            Err(err) => {
                cb.on_reply(Err(op_error!(AioReceiveError::from(err))));
            }
        }
    })
    .unwrap();

    let req = Request::Sleep(0);
    let msg = try_into_nng_message(&req).unwrap();
    {
        let mut ctx_state = ctx_state.lock().unwrap();
        *ctx_state = AioState::Send;
        aio.send(&ctx, msg).unwrap();
    }
    match rx.recv() {
        Ok(rep) => info!("received forwarded reply : {:?}", rep),
        Err(err) => panic!("recv failed: {}", err),
    }

    server.stop();
    server.join();

    let server = start_server(&*url.as_str());
    // after restarting the server, the client should reconnect automatically
    let req = Request::Sleep(0);
    let msg = try_into_nng_message(&req).unwrap();
    {
        let mut ctx_state = ctx_state.lock().unwrap();
        *ctx_state = AioState::Send;
        aio.send(&ctx, msg).unwrap();
    }
    match rx.recv() {
        Ok(rep) => info!(
            "received forwarded reply after restarting the server : {:?}",
            rep
        ),
        Err(err) => panic!("recv failed: {}", err),
    }

    thread::sleep_ms(100);

    server.stop();
    server.join();
}

#[test]
fn async_client_send_with_callback() {
    oysterpack_log::init(log_config(), oysterpack_log::StderrLogger);
    let url = Arc::new(format!("inproc://{}", ULID::generate()));

    // start a server with 2 aio contexts
    let listener_settings =
        ListenerSettings::new(&*url.as_str()).set_aio_count(NonZeroUsize::new(2).unwrap());
    let server = Server::builder(listener_settings, TestProcessor)
        .spawn()
        .unwrap();

    let dialer_settings = DialerSettings::new(url.as_str())
        .set_non_blocking(true)
        .set_reconnect_min_time(Duration::from_millis(100))
        .set_reconnect_max_time(Duration::from_millis(100));
    let mut client = AsyncClient::dial(dialer_settings.clone()).unwrap();

    for i in 0..10 {
        let msg = try_into_nng_message(&Request::Sleep(0)).unwrap();
        let (tx, rx) = crossbeam::channel::bounded(10);
        client.send_with_callback(msg, ReplyForwarder { chan: tx });
        match rx.recv() {
            Ok(rep) => info!("received forwarded reply #{} : {:?}", i, rep),
            Err(err) => panic!("recv #{} failed: {}", i, err),
        }
    }

    thread::yield_now();
    for i in 0..3 {
        let count = client.context_count();
        if count == 0 {
            break;
        }
        warn!("waiting for context to be closed ... count = {}", count);
        thread::sleep_ms(1);
    }
    assert_eq!(client.context_count(), 0);

    server.stop();
    server.join();
    info!("server has stopped");
}
