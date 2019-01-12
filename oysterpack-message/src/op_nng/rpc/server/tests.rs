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

use super::*;
use oysterpack_uid::ULID;
use std::{
    iter::Iterator,
    num::NonZeroUsize,
    sync::Arc,
    thread,
    time::{Duration, Instant},
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
        match bincode::deserialize::<Request>(&*req.body()).unwrap() {
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
            Request::Panic(msg) => {
                error!("received Panic message on {:?}", thread::current().id());
                panic!(msg)
            }
        }
        req
    }
}

#[derive(Debug, Serialize, Deserialize)]
enum Request {
    Sleep(u32),
    Panic(String),
}

fn log_config() -> oysterpack_log::LogConfig {
    oysterpack_log::config::LogConfigBuilder::new(oysterpack_log::Level::Info).build()
}

fn send_sleep_request(url: &str, sleep_ms: u32) -> Result<Duration, nng::Error> {
    let mut s = Socket::new(nng::Protocol::Req0)?;
    let dialer = nng::dialer::DialerOptions::new(&s, url)?;
    let dialer = match dialer.start(true) {
        Ok(dialer) => dialer,
        Err((_, err)) => panic!(err),
    };

    let msg_bytes = bincode::serialize(&Request::Sleep(sleep_ms)).unwrap();
    let mut req = nng::Message::with_capacity(msg_bytes.len()).unwrap();
    req.push_back(&msg_bytes).unwrap();

    info!("sending client request ...");
    let start = Instant::now();
    s.send(req)?;
    s.recv()?;
    let dur = Instant::now().duration_since(start);
    info!("Request::Sleep({}) took {:?}", sleep_ms, dur);
    Ok(dur)
}

fn send_sleep_request_with_recv_timeout(
    url: &str,
    sleep_ms: u32,
    timeout: Duration,
) -> Result<Duration, nng::Error> {
    let mut s = Socket::new(nng::Protocol::Req0)?;
    s.set_opt::<nng::options::RecvTimeout>(Some(timeout))?;
    let dialer = nng::dialer::DialerOptions::new(&s, url)?;
    let dialer = match dialer.start(true) {
        Ok(dialer) => dialer,
        Err((_, err)) => panic!(err),
    };

    let msg_bytes = bincode::serialize(&Request::Sleep(sleep_ms)).unwrap();
    let mut req = nng::Message::with_capacity(msg_bytes.len()).unwrap();
    req.push_back(&msg_bytes).unwrap();

    info!("sending client request ...");
    let start = Instant::now();
    s.send(req)?;
    s.recv()?;
    let dur = Instant::now().duration_since(start);
    info!("Request::Sleep({}) took {:?}", sleep_ms, dur);
    Ok(dur)
}

fn send_panic_request(url: &str, msg: &str) -> Result<Duration, nng::Error> {
    let mut s = Socket::new(nng::Protocol::Req0)?;
    s.set_opt::<nng::options::RecvTimeout>(Some(Duration::from_millis(10)))?;
    let dialer = nng::dialer::DialerOptions::new(&s, url)?;
    let dialer = match dialer.start(true) {
        Ok(dialer) => dialer,
        Err((_, err)) => panic!(err),
    };

    let msg_bytes = bincode::serialize(&Request::Panic(msg.to_string())).unwrap();
    let mut req = nng::Message::with_capacity(msg_bytes.len()).unwrap();
    req.push_back(&msg_bytes).unwrap();

    info!("sending client request ...");
    let start = Instant::now();
    s.send(req)?;
    match s.recv() {
        Ok(_) => panic!("request should have panicked"),
        Err(err) => error!("Request:Panic failed as expected: {}", err),
    }
    let dur = Instant::now().duration_since(start);
    info!("Request::Panic took {:?}", dur);
    Ok(dur)
}

/// as long as aio contexts are available, client requests should be processed
#[test]
fn rpc_server() {
    oysterpack_log::init(log_config(), oysterpack_log::StderrLogger);

    let url = Arc::new(format!("inproc://{}", ULID::generate()));

    // the client should be able to connect async after the server has started
    let client_thread_handle = {
        let url = url.clone();
        thread::spawn(move || send_sleep_request(&*url.as_str(), 0).unwrap())
    };

    // start a server with 2 aio contexts
    let listener_settings =
        super::ListenerSettings::new(&*url.as_str()).set_aio_count(NonZeroUsize::new(2).unwrap());

    let server = Server::builder(listener_settings, TestProcessor)
        .spawn()
        .unwrap();

    // wait for the client background request completes
    client_thread_handle.join();
    assert!(server.running());

    for _ in 0..10 {
        send_sleep_request(&*url.as_str(), 0).unwrap();
    }

    // submit a long running request, which will block one of the aio contexts for 1 sec
    let (s, r) = crossbeam::channel::bounded(0);
    const SLEEP_TIME: u32 = 1000;
    {
        let url = url.clone();
        thread::spawn(move || {
            s.send(()).unwrap();
            send_sleep_request(&*url.as_str(), SLEEP_TIME).unwrap();
        });
    }
    r.recv().unwrap();
    info!("client with {} ms request has started", SLEEP_TIME);
    // give the client a chance to send the request
    thread::sleep_ms(10);

    // requests should still be able to flow through because one of aio contexts is available
    for _ in 0..10 {
        let duration = send_sleep_request(&*url.as_str(), 0).unwrap();
        assert!(duration < Duration::from_millis(50));
    }

    info!("client requests are done.");

    server.stop();
    server.join().unwrap();
}

/// when all aio contexts are busy, client requests will be blocked waiting for an aio context to
/// free up
#[test]
fn rpc_server_all_contexts_busy() {
    oysterpack_log::init(log_config(), oysterpack_log::StderrLogger);

    let url = format!("inproc://{}", ULID::generate());

    // the client should be able to connect async after the server has started
    let client_thread_handle = {
        let url = url.clone();
        thread::spawn(move || send_sleep_request(&*url.as_str(), 0).unwrap())
    };

    // start a server with 2 aio contexts
    let listener_settings =
        super::ListenerSettings::new(&*url.as_str()).set_aio_count(NonZeroUsize::new(2).unwrap());

    let server = super::Server::spawn(listener_settings, &TestProcessor, None, None).unwrap();

    // wait for the client background request completes
    client_thread_handle.join();

    // submit long running request, which will block one of the aio contexts for 1 sec
    let (s1, r1) = crossbeam::channel::bounded(0);
    let (s2, r2) = crossbeam::channel::bounded(0);
    const SLEEP_TIME: u32 = 1000;
    {
        let url = url.clone();
        thread::spawn(move || {
            s1.send(()).unwrap();
            send_sleep_request(&*url.as_str(), SLEEP_TIME).unwrap();
        });
    }
    {
        let url = url.clone();
        thread::spawn(move || {
            s2.send(()).unwrap();
            send_sleep_request(&*url.as_str(), SLEEP_TIME).unwrap();
        });
    }
    r1.recv().unwrap();
    r2.recv().unwrap();
    info!(
        "client requests with {} ms request have started",
        SLEEP_TIME
    );
    // give the client a chance to send the request
    thread::sleep_ms(10);

    let duration = send_sleep_request(&*url.as_str(), 0).unwrap();
    assert!(
        duration > Duration::from_millis(500),
        "client request should have been blocked waiting for aio context to become available"
    );

    server.stop();
    server.join().unwrap();
}

/// build a server via the server Builder
#[test]
fn rpc_server_builder() {
    oysterpack_log::init(log_config(), oysterpack_log::StderrLogger);

    let url = format!("inproc://{}", ULID::generate());
    info!("url = {}", url);

    // the client should be able to connect async after the server has started
    let client_thread_handle = {
        let url = url.clone();
        thread::spawn(move || send_sleep_request(url.as_str(), 0).unwrap())
    };

    // start a server with 2 aio contexts
    let listener_settings =
        super::ListenerSettings::new(&*url.as_str()).set_aio_count(NonZeroUsize::new(2).unwrap());

    let server = super::Builder::new(listener_settings, TestProcessor)
        .spawn()
        .unwrap();

    // wait for the client background request completes
    client_thread_handle.join();
    send_sleep_request(&*url.as_str(), 0).unwrap();

    server.stop_signal().stop();
    server.join().unwrap();
}

/// when a message processor panics, the aio context is terminated - this means that over time,
/// the server will become unresponsive, i.e., when all aio contexts have terminated
#[test]
fn message_processor_panics() {
    oysterpack_log::init(log_config(), oysterpack_log::StderrLogger);

    let url = format!("inproc://{}", ULID::generate());
    info!("url = {}", url);

    // the client should be able to connect async after the server has started
    let client_thread_handle = {
        let url = url.clone();
        thread::spawn(move || send_sleep_request(url.as_str(), 0).unwrap())
    };

    // start a server with 2 aio contexts
    let listener_settings =
        super::ListenerSettings::new(url.as_str()).set_aio_count(NonZeroUsize::new(2).unwrap());

    let server = super::Builder::new(listener_settings, TestProcessor)
        .spawn()
        .unwrap();

    // wait for the client background request completes
    client_thread_handle.join();

    send_sleep_request_with_recv_timeout(url.as_str(), 0, Duration::from_millis(50)).unwrap();

    (0..10)
        .map(|i| {
            let url = url.clone();
            thread::spawn(move || {
                send_panic_request(url.as_str(), &format!("panic #{}", i)).unwrap()
            })
        })
        .collect::<Vec<thread::JoinHandle<Duration>>>()
        .into_iter()
        .for_each(|h| {
            let _ = h.join().unwrap();
        });

    info!("all panic requests have completed - let's try to send a good request");
    match send_sleep_request_with_recv_timeout(url.as_str(), 0, Duration::from_millis(50)) {
        Ok(_) => panic!("should have failed because all aio contexts should be terminated"),
        Err(err) => assert_eq!(err.kind(), nng::ErrorKind::TimedOut),
    }

    info!(
        "all aio contexts are terminated, and server running = {}",
        server.running()
    );

    server.stop();
    info!(
        "after stop was signalled, server running = {}",
        server.running()
    );
    thread::sleep_ms(100);
    info!(
        "100 ms after stop was signalled, server running = {}",
        server.running()
    );
    server.join().unwrap();
}
