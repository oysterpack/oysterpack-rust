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
    oysterpack_log::config::LogConfigBuilder::new(oysterpack_log::Level::Info)
        .target_level(
            oysterpack_log::Target::from(env!("CARGO_PKG_NAME")),
            Level::Debug,
        )
        .build()
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
            client1.send(try_into_nng_message(&req).unwrap()).unwrap()
        );
        info!(
            "client2 reply: {:?}",
            client2.send(try_into_nng_message(&req).unwrap()).unwrap()
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
            let req = try_into_nng_message(&req).unwrap();
            client.send(req).unwrap()
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
fn async_client_send_with_callback() {
    oysterpack_log::init(log_config(), oysterpack_log::StderrLogger);
    let url = Arc::new(format!("inproc://{}", ULID::generate()));

    // start a server with 2 aio contexts
    let listener_settings =
        ListenerSettings::new(&*url.as_str()).set_aio_count(NonZeroUsize::new(2).unwrap());
    let server = Server::builder(listener_settings, TestProcessor)
        .spawn()
        .unwrap();

    const AIO_CONTEXT_CAPACITY: usize = 10;
    let dialer_settings = DialerSettings::new(url.as_str())
        .set_non_blocking(true)
        .set_reconnect_min_time(Duration::from_millis(100))
        .set_reconnect_max_time(Duration::from_millis(100))
        .set_max_concurrent_request_capacity(NonZeroUsize::new(AIO_CONTEXT_CAPACITY).unwrap());
    let mut client = AsyncClient::dial(dialer_settings.clone()).unwrap();

    for i in 0..AIO_CONTEXT_CAPACITY {
        let msg = try_into_nng_message(&Request::Sleep(0)).unwrap();
        let (tx, rx) = crossbeam::channel::bounded(10);
        client.send_with_callback(msg, ReplyForwarder { chan: tx });
        match rx.recv() {
            Ok(rep) => info!("received forwarded reply #{} : {:?}", i, rep),
            Err(err) => panic!("recv #{} failed: {}", i, err),
        }
    }

    thread::yield_now();
    for i in 0..10 {
        let count = client.context_count();
        if count == 0 {
            break;
        }
        warn!(
            "({}) waiting for context to be closed ... count = {}",
            i, count
        );
        thread::sleep_ms(1);
    }
    assert_eq!(client.context_count(), 0);
    assert_eq!(client.max_capacity(), client.available_capacity());
    assert_eq!(client.used_capacity(), 0);

    server.stop();
    server.join();
    info!("server has stopped");
}

#[test]
fn async_client_send_with_callback_with_max_capacity_exceeded() {
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

    let (tx, rx) = crossbeam::channel::bounded(10);
    let msg = try_into_nng_message(&Request::Sleep(10)).unwrap();
    assert!(client
        .send_with_callback(msg, ReplyForwarder { chan: tx.clone() })
        .is_ok());
    let msg = try_into_nng_message(&Request::Sleep(10)).unwrap();
    match client.send_with_callback(msg, ReplyForwarder { chan: tx.clone() }) {
        Ok(_) => panic!("this should have failed because we are max capacity"),
        Err(err) => assert_eq!(
            crate::op_nng::rpc::client::asyncio::errors::AioContextAtMaxCapacity::ERROR_ID,
            err.id()
        ),
    }

    assert!(rx.recv().is_ok());

    server.stop();
    server.join();
}

#[test]
fn async_client_send_with_callback_restart_server() {
    oysterpack_log::init(log_config(), oysterpack_log::StderrLogger);
    let url = Arc::new(format!("inproc://{}", ULID::generate()));

    // start a server with 2 aio contexts
    let listener_settings =
        ListenerSettings::new(&*url.as_str()).set_aio_count(NonZeroUsize::new(2).unwrap());
    let server = Server::builder(listener_settings, TestProcessor)
        .spawn()
        .unwrap();

    const AIO_CONTEXT_CAPACITY: usize = 10;
    let dialer_settings = DialerSettings::new(url.as_str())
        .set_non_blocking(true)
        .set_reconnect_min_time(Duration::from_millis(100))
        .set_reconnect_max_time(Duration::from_millis(100))
        .set_max_concurrent_request_capacity(NonZeroUsize::new(AIO_CONTEXT_CAPACITY).unwrap());
    let mut client = AsyncClient::dial(dialer_settings.clone()).unwrap();

    for i in 0..AIO_CONTEXT_CAPACITY {
        let msg = try_into_nng_message(&Request::Sleep(0)).unwrap();
        let (tx, rx) = crossbeam::channel::bounded(10);
        client.send_with_callback(msg, ReplyForwarder { chan: tx });
        match rx.recv() {
            Ok(rep) => info!("received forwarded reply #{} : {:?}", i, rep),
            Err(err) => panic!("recv #{} failed: {}", i, err),
        }
    }

    thread::yield_now();
    for i in 0..10 {
        let count = client.context_count();
        if count == 0 {
            break;
        }
        warn!(
            "({}) waiting for context to be closed ... count = {}",
            i, count
        );
        thread::sleep_ms(1);
    }
    assert_eq!(client.context_count(), 0);
    assert_eq!(client.max_capacity(), client.available_capacity());
    assert_eq!(client.used_capacity(), 0);

    server.stop();
    server.join();
    info!("server has stopped");

    info!("restarting server ...");
    // start a server with 2 aio contexts
    let listener_settings =
        ListenerSettings::new(&*url.as_str()).set_aio_count(NonZeroUsize::new(2).unwrap());
    let server = Server::builder(listener_settings, TestProcessor)
        .spawn()
        .unwrap();

    info!("... restarted server");

    for i in 0..AIO_CONTEXT_CAPACITY {
        let msg = try_into_nng_message(&Request::Sleep(0)).unwrap();
        let (tx, rx) = crossbeam::channel::bounded(10);
        client.send_with_callback(msg, ReplyForwarder { chan: tx });
        match rx.recv() {
            Ok(rep) => info!("received forwarded reply #{} : {:?}", i, rep),
            Err(err) => panic!("recv #{} failed: {}", i, err),
        }
    }

    thread::yield_now();
    for i in 0..10 {
        let count = client.context_count();
        if count == 0 {
            break;
        }
        warn!(
            "({}) waiting for context to be closed ... count = {}",
            i, count
        );
        thread::sleep_ms(1);
    }
    assert_eq!(client.context_count(), 0);
    assert_eq!(client.max_capacity(), client.available_capacity());
    assert_eq!(client.used_capacity(), 0);
}
