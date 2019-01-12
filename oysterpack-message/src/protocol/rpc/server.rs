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

//! Provides an RPC nng messaging server

use crate::protocol::rpc::{MessageProcessor, MessageProcessorFactory, ThreadConfig};
use log::{error, info};
use nng::{self, listener::Listener, options::Options, Socket};
use oysterpack_errors::{op_error, Error, ErrorMessage};
use serde::{Deserialize, Serialize};
use std::{fmt, num::NonZeroUsize, sync::Arc, thread, time::Duration};

/// nng RPC server
pub struct Server {
    stop_trigger: crossbeam::channel::Sender<()>,
    stopped_signal: crossbeam::channel::Receiver<()>,
}

impl Server {
    /// Spawns a new server instance in a background thread
    pub fn spawn<Factory, Processor>(
        listener_settings: ListenerSettings,
        socket_settings: Option<SocketSettings>,
        message_processor_factory: Arc<Factory>,
        thread_config: Option<ThreadConfig>,
    ) -> Result<Server, Error>
    where
        Factory: MessageProcessorFactory<Processor, nng::Message, nng::Message>,
        Processor: MessageProcessor<nng::Message, nng::Message>,
    {
        let socket = nng::Socket::new(nng::Protocol::Rep0)
            .map_err(|err| op_error!(errors::SocketCreateError(ErrorMessage(err.to_string()))))?;
        let socket = {
            match socket_settings {
                Some(socket_settings) => socket_settings.apply(socket)?,
                None => socket,
            }
        };

        let (stop_sender, stop_receiver) = crossbeam::channel::bounded(0);
        let (stopped_sender, stopped_receiver) = crossbeam::channel::bounded::<()>(1);

        let builder = thread_config.map_or_else(thread::Builder::new, |config| config.builder());

        builder
            .spawn(move || {
                let workers = (0..listener_settings.aio_context_count)
                    .map(|_| {
                        let mut state = AioState::Recv;
                        let mut message_processor = message_processor_factory.new();

                        let ctx: nng::aio::Context = Server::new_context(&socket)
                            .expect("failed to create aio socket context");
                        let ctx_clone = ctx.clone();
                        let aio = nng::aio::Aio::with_callback(move |aio| {
                            Server::handle_aio_event(
                                aio,
                                &ctx_clone,
                                &mut state,
                                &mut message_processor,
                            )
                        })
                        .expect("nng::aio::Aio::with_callback() failed");

                        (aio, ctx)
                    })
                    .collect::<Vec<(nng::aio::Aio, nng::aio::Context)>>();

                let _listener = listener_settings.start_listener(&socket).unwrap();
                info!("socket listener has been started");

                // Now start all of the workers listening.
                for (a, c) in &workers {
                    a.recv(c)
                        .map_err(|err| {
                            op_error!(errors::AioReceiveError(ErrorMessage(err.to_string())))
                        })
                        .unwrap();
                }
                info!("aio context receive operations have been initiated");

                // block until stop signal is received
                let _ = stop_receiver.recv();
                // send notification that the server has stopped
                let _ = stopped_sender.send(());
            })
            .expect("failed to spawn server thread");

        Ok(Server {
            stop_trigger: stop_sender,
            stopped_signal: stopped_receiver,
        })
    }

    /// Triggers the server to stop async
    pub fn stop(&self) {
        let _ = self.stop_trigger.send(());
    }

    /// Waits until the server stops, which will block the current thread
    pub fn wait(&self) {
        let _ = self.stopped_signal.recv();
    }

    /// Waits for the server to stop, but only for a limited time.
    pub fn wait_timeout(&self, timeout: Duration) -> bool {
        match self.stopped_signal.recv_timeout(timeout) {
            Ok(_) => true,
            Err(crossbeam::channel::RecvTimeoutError::Disconnected) => true,
            Err(crossbeam::channel::RecvTimeoutError::Timeout) => false,
        }
    }

    fn new_context(socket: &nng::Socket) -> Result<nng::aio::Context, Error> {
        nng::aio::Context::new(&socket)
            .map_err(|err| op_error!(errors::AioContextError(ErrorMessage(err.to_string()))))
    }

    // TODO: how to best handle aio errors
    fn handle_aio_event<T>(
        aio: &nng::aio::Aio,
        ctx: &nng::aio::Context,
        state: &mut AioState,
        message_processor: &mut T,
    ) where
        T: MessageProcessor<nng::Message, nng::Message>,
    {
        let new_state = match *state {
            AioState::Recv => match aio.result().unwrap() {
                Ok(_) => match aio.get_msg() {
                    Some(req) => {
                        let rep = message_processor.process(req);
                        match aio.send(&ctx, rep) {
                            Ok(_) => AioState::Send,
                            Err((_rep, err)) => {
                                error!("failed to send reply: {}", err);
                                aio.cancel();
                                aio.recv(&ctx).expect("aio.recv() failed");
                                AioState::Recv
                            }
                        }
                    }
                    None => {
                        error!("No message was found ... initiating aio.recv()");
                        aio.recv(&ctx).expect("aio.recv() failed");
                        AioState::Recv
                    }
                },
                Err(err) => {
                    match err.kind() {
                        nng::ErrorKind::Closed => info!("aio context is closed"),
                        _ => error!("aio receive error: {}", err),
                    }

                    AioState::Recv
                }
            },
            AioState::Send => {
                if let Err(err) = aio.result().unwrap() {
                    error!("aio send error: {}", err)
                }
                aio.recv(ctx).unwrap();
                AioState::Recv
            }
        };

        *state = new_state;
    }
}

impl fmt::Debug for Server {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("Server")
    }
}

/// Aio state for socket context.
#[derive(Debug, Copy, Clone)]
pub enum AioState {
    /// aio receive operation is in progress
    Recv,
    /// aio send operation is in progress
    Send,
}

/// Listener settings
#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct SocketSettings {
    reconnect_min_time: Option<Duration>,
    reconnect_max_time: Option<Duration>,
    max_ttl: Option<u8>,
}

impl SocketSettings {
    /// set socket options
    pub fn apply(&self, socket: Socket) -> Result<Socket, Error> {
        socket
            .set_opt::<nng::options::ReconnectMinTime>(self.reconnect_min_time)
            .map_err(|err| op_error!(errors::SocketSetOptError(ErrorMessage(err.to_string()))))?;

        socket
            .set_opt::<nng::options::ReconnectMaxTime>(self.reconnect_max_time)
            .map_err(|err| op_error!(errors::SocketSetOptError(ErrorMessage(err.to_string()))))?;

        if let Some(opt) = self.max_ttl {
            socket.set_opt::<nng::options::MaxTtl>(opt).map_err(|err| {
                op_error!(errors::SocketSetOptError(ErrorMessage(err.to_string())))
            })?;
        }

        Ok(socket)
    }

    /// The minimum amount of time to wait before attempting to establish a connection after a previous attempt has failed.
    /// This value becomes the default for new dialers. Individual dialers can then override the setting.
    pub fn reconnect_min_time(&self) -> Option<Duration> {
        self.reconnect_min_time
    }

    /// The maximum amount of time to wait before attempting to establish a connection after a previous
    /// attempt has failed.
    ///
    /// If this is non-zero, then the time between successive connection attempts will start at the
    /// value of ReconnectMinTime, and grow exponentially, until it reaches this value. If this value
    /// is zero, then no exponential back-off between connection attempts is done, and each attempt
    /// will wait the time specified by ReconnectMinTime.
    pub fn reconnect_max_time(&self) -> Option<Duration> {
        self.reconnect_max_time
    }

    /// The maximum number of "hops" a message may traverse.
    ///
    /// The intention here is to prevent forwarding loops in device chains. Note that not all protocols
    /// support this option and those that do generally have a default value of 8.
    ///
    /// Each node along a forwarding path may have its own value for the maximum time-to-live, and
    /// performs its own checks before forwarding a message. Therefore it is helpful if all nodes in
    /// the topology use the same value for this option.
    ///
    /// Sockets can use this with the following protocols:
    /// - Pair v1
    /// - Rep v0
    /// - Req v0
    /// - Surveyor v0
    /// - Respondent v0
    pub fn max_ttl(&self) -> Option<u8> {
        self.max_ttl
    }
}

/// Listener settings
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct ListenerSettings {
    url: String,
    recv_max_size: Option<usize>,
    no_delay: Option<bool>,
    keep_alive: Option<bool>,
    non_blocking: bool,
    aio_context_count: usize,
}

impl ListenerSettings {
    /// constructor
    pub fn new(url: &str) -> ListenerSettings {
        ListenerSettings {
            url: url.to_string(),
            recv_max_size: None,
            no_delay: None,
            keep_alive: None,
            non_blocking: false,
            aio_context_count: 1,
        }
    }

    /// Cause the listener to start listening on the address with which it was created.
    ///
    /// Normally, the act of "binding" to the address indicated by url is done synchronously, including
    /// any necessary name resolution. As a result, a failure, such as if the address is already in use,
    /// will be returned immediately. However, if nonblocking is specified then this is done asynchronously;
    /// furthermore any failure to bind will be periodically reattempted in the background.
    ///
    /// The returned handle controls the life of the listener. If it is dropped, the listener is shut
    /// down and no more messages will be received on it.
    pub fn start_listener(self, socket: &Socket) -> Result<Listener, Error> {
        let options = nng::listener::ListenerOptions::new(socket, self.url())
            .map_err(|err| op_error!(errors::ListenerCreateError(ErrorMessage(err.to_string()))))?;

        if let Some(option) = self.recv_max_size.as_ref() {
            options
                .set_opt::<nng::options::RecvMaxSize>(*option)
                .map_err(|err| {
                    op_error!(errors::ListenerSetOptError(ErrorMessage(err.to_string())))
                })?;
        }

        if let Some(option) = self.no_delay.as_ref() {
            options
                .set_opt::<nng::options::transport::tcp::NoDelay>(*option)
                .map_err(|err| {
                    op_error!(errors::ListenerSetOptError(ErrorMessage(err.to_string())))
                })?;
        }

        if let Some(option) = self.keep_alive.as_ref() {
            options
                .set_opt::<nng::options::transport::tcp::KeepAlive>(*option)
                .map_err(|err| {
                    op_error!(errors::ListenerSetOptError(ErrorMessage(err.to_string())))
                })?;
        }

        options.start(self.non_blocking).map_err(|(_options, err)| {
            op_error!(errors::ListenerStartError(ErrorMessage(err.to_string())))
        })
    }

    /// the address that the server is listening on
    pub fn url(&self) -> &str {
        &self.url
    }

    /// if true, then it binds to the address asynchronously
    pub fn non_blocking(&self) -> bool {
        self.non_blocking
    }

    /// number of async IO operations that can be performed concurrently, which corresponds to the number
    /// of socket contexts that will be created
    pub fn aio_context_count(&self) -> usize {
        self.aio_context_count
    }

    /// The maximum message size that the will be accepted from a remote peer.
    ///
    /// If a peer attempts to send a message larger than this, then the message will be discarded.
    /// If the value of this is zero, then no limit on message sizes is enforced. This option exists
    /// to prevent certain kinds of denial-of-service attacks, where a malicious agent can claim to
    /// want to send an extraordinarily large message, without sending any data. This option can be
    /// set for the socket, but may be overridden for on a per-dialer or per-listener basis.
    pub fn recv_max_size(&self) -> Option<usize> {
        self.recv_max_size
    }

    /// When true (the default), messages are sent immediately by the underlying TCP stream without waiting to gather more data.
    /// When false, Nagle's algorithm is enabled, and the TCP stream may wait briefly in attempt to coalesce messages.
    ///
    /// Nagle's algorithm is useful on low-bandwidth connections to reduce overhead, but it comes at a cost to latency.
    pub fn no_delay(&self) -> Option<bool> {
        self.no_delay
    }

    /// Enable the sending of keep-alive messages on the underlying TCP stream.
    ///
    /// This option is false by default. When enabled, if no messages are seen for a period of time,
    /// then a zero length TCP message is sent with the ACK flag set in an attempt to tickle some traffic
    /// from the peer. If none is still seen (after some platform-specific number of retries and timeouts),
    /// then the remote peer is presumed dead, and the connection is closed.
    ///
    /// This option has two purposes. First, it can be used to detect dead peers on an otherwise quiescent
    /// network. Second, it can be used to keep connection table entries in NAT and other middleware
    /// from being expiring due to lack of activity.
    pub fn keep_alive(&self) -> Option<bool> {
        self.keep_alive
    }

    /// Sets the maximum message size that the will be accepted from a remote peer.
    pub fn set_recv_max_size(self, recv_max_size: usize) -> Self {
        let mut settings = self;
        settings.recv_max_size = Some(recv_max_size);
        settings
    }

    /// Sets no delay setting on TCP connection
    pub fn set_no_delay(self, no_delay: bool) -> Self {
        let mut settings = self;
        settings.no_delay = Some(no_delay);
        settings
    }

    /// Sets keep alive setting on TCP connection
    pub fn set_keep_alive(self, keep_alive: bool) -> Self {
        let mut settings = self;
        settings.keep_alive = Some(keep_alive);
        settings
    }

    /// Normally, the act of "binding" to the address indicated by url is done synchronously, including
    /// any necessary name resolution. As a result, a failure, such as if the address is already in use,
    /// will be returned immediately. However, if nonblocking is specified then this is done asynchronously;
    /// furthermore any failure to bind will be periodically reattempted in the background.
    pub fn set_non_blocking(self, non_blocking: bool) -> Self {
        let mut settings = self;
        settings.non_blocking = non_blocking;
        settings
    }

    /// set the number of async IO operations that can be performed concurrently
    pub fn set_aio_count(self, count: NonZeroUsize) -> Self {
        let mut settings = self;
        settings.aio_context_count = count.get();
        settings
    }
}

pub mod errors {
    //! server errors

    use super::*;
    use oysterpack_errors::IsError;
    use std::fmt;

    /// Failed to create socket
    #[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
    pub struct SocketCreateError(pub ErrorMessage);

    impl SocketCreateError {
        /// Error Id
        pub const ERROR_ID: oysterpack_errors::Id =
            oysterpack_errors::Id(1870511279758140964159435436428736321);
        /// Level::Error
        pub const ERROR_LEVEL: oysterpack_errors::Level = oysterpack_errors::Level::Error;
    }

    impl IsError for SocketCreateError {
        fn error_id(&self) -> oysterpack_errors::Id {
            Self::ERROR_ID
        }

        fn error_level(&self) -> oysterpack_errors::Level {
            Self::ERROR_LEVEL
        }
    }

    impl fmt::Display for SocketCreateError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "Failed to create socket: {:?}", self.0)
        }
    }

    /// An error occurred when setting a socket option.
    #[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
    pub struct SocketSetOptError(pub ErrorMessage);

    impl SocketSetOptError {
        /// Error Id
        pub const ERROR_ID: oysterpack_errors::Id =
            oysterpack_errors::Id(1870511354278148346409496152407634279);
        /// Level::Error
        pub const ERROR_LEVEL: oysterpack_errors::Level = oysterpack_errors::Level::Error;
    }

    impl IsError for SocketSetOptError {
        fn error_id(&self) -> oysterpack_errors::Id {
            Self::ERROR_ID
        }

        fn error_level(&self) -> oysterpack_errors::Level {
            Self::ERROR_LEVEL
        }
    }

    impl fmt::Display for SocketSetOptError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "Failed to set socket option: {:?}", self.0)
        }
    }

    /// Failed to start listener instance
    #[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
    pub struct ListenerStartError(pub ErrorMessage);

    impl ListenerStartError {
        /// Error Id
        pub const ERROR_ID: oysterpack_errors::Id =
            oysterpack_errors::Id(1870510777469481547545613773325104910);
        /// Level::Error
        pub const ERROR_LEVEL: oysterpack_errors::Level = oysterpack_errors::Level::Error;
    }

    impl IsError for ListenerStartError {
        fn error_id(&self) -> oysterpack_errors::Id {
            Self::ERROR_ID
        }

        fn error_level(&self) -> oysterpack_errors::Level {
            Self::ERROR_LEVEL
        }
    }

    impl fmt::Display for ListenerStartError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "Failed to start listener: {:?}", self.0)
        }
    }

    /// Failed to create listener instance
    #[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
    pub struct ListenerCreateError(pub ErrorMessage);

    impl ListenerCreateError {
        /// Error Id
        pub const ERROR_ID: oysterpack_errors::Id =
            oysterpack_errors::Id(1870302624499038905208367552914704572);
        /// Level::Error
        pub const ERROR_LEVEL: oysterpack_errors::Level = oysterpack_errors::Level::Error;
    }

    impl IsError for ListenerCreateError {
        fn error_id(&self) -> oysterpack_errors::Id {
            Self::ERROR_ID
        }

        fn error_level(&self) -> oysterpack_errors::Level {
            Self::ERROR_LEVEL
        }
    }

    impl fmt::Display for ListenerCreateError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "Failed to create listener instance: {:?}", self.0)
        }
    }

    /// An error occurred when setting a listener option.
    #[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
    pub struct ListenerSetOptError(pub ErrorMessage);

    impl ListenerSetOptError {
        /// Error Id
        pub const ERROR_ID: oysterpack_errors::Id =
            oysterpack_errors::Id(1870302624499038905208367552914704572);
        /// Level::Error
        pub const ERROR_LEVEL: oysterpack_errors::Level = oysterpack_errors::Level::Error;
    }

    impl IsError for ListenerSetOptError {
        fn error_id(&self) -> oysterpack_errors::Id {
            Self::ERROR_ID
        }

        fn error_level(&self) -> oysterpack_errors::Level {
            Self::ERROR_LEVEL
        }
    }

    impl fmt::Display for ListenerSetOptError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "Failed to set listener option: {:?}", self.0)
        }
    }

    /// Failed to create new asynchronous I/O handle
    #[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
    pub struct AioCreateError(pub ErrorMessage);

    impl AioCreateError {
        /// Error Id
        pub const ERROR_ID: oysterpack_errors::Id =
            oysterpack_errors::Id(1870510443603468311033495279443790945);
        /// Level::Error
        pub const ERROR_LEVEL: oysterpack_errors::Level = oysterpack_errors::Level::Error;
    }

    impl IsError for AioCreateError {
        fn error_id(&self) -> oysterpack_errors::Id {
            Self::ERROR_ID
        }

        fn error_level(&self) -> oysterpack_errors::Level {
            Self::ERROR_LEVEL
        }
    }

    impl fmt::Display for AioCreateError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "Failed to create new aio handle: {}", self.0)
        }
    }

    /// Aio receive operation failed
    #[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
    pub struct AioReceiveError(pub ErrorMessage);

    impl AioReceiveError {
        /// Error Id
        pub const ERROR_ID: oysterpack_errors::Id =
            oysterpack_errors::Id(1870374078796088086815067802169113773);
        /// Level::Error
        pub const ERROR_LEVEL: oysterpack_errors::Level = oysterpack_errors::Level::Error;
    }

    impl IsError for AioReceiveError {
        fn error_id(&self) -> oysterpack_errors::Id {
            Self::ERROR_ID
        }

        fn error_level(&self) -> oysterpack_errors::Level {
            Self::ERROR_LEVEL
        }
    }

    impl fmt::Display for AioReceiveError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "Aio receive operation failed: {}", self.0)
        }
    }

    /// Failed to create new socket context
    #[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
    pub struct AioContextError(pub ErrorMessage);

    impl AioContextError {
        /// Error Id
        pub const ERROR_ID: oysterpack_errors::Id =
            oysterpack_errors::Id(1870374278155759380545373361718947172);
        /// Level::Error
        pub const ERROR_LEVEL: oysterpack_errors::Level = oysterpack_errors::Level::Error;
    }

    impl IsError for AioContextError {
        fn error_id(&self) -> oysterpack_errors::Id {
            Self::ERROR_ID
        }

        fn error_level(&self) -> oysterpack_errors::Level {
            Self::ERROR_LEVEL
        }
    }

    impl fmt::Display for AioContextError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "Failed to create new socket context: {}", self.0)
        }
    }

}

#[allow(warnings)]
#[cfg(test)]
mod test {
    use super::*;
    use std::{
        num::NonZeroUsize,
        thread,
        time::{Duration, Instant},
    };

    #[derive(Clone)]
    struct Sleep;

    impl MessageProcessorFactory<Sleep, nng::Message, nng::Message> for Sleep {
        fn new(&self) -> Sleep {
            Sleep
        }
    }

    impl MessageProcessor<nng::Message, nng::Message> for Sleep {
        fn process(&mut self, req: nng::Message) -> nng::Message {
            info!("received message on {:?}", thread::current().id());
            let sleep_ms: u32 = bincode::deserialize(&*req.body()).unwrap();
            if sleep_ms > 0 {
                info!(
                    "handler({:?}) sleeping for {} ms ...",
                    thread::current().id(),
                    sleep_ms
                );
                thread::sleep_ms(sleep_ms);
                info!("handler({:?}) has awaken !!!", thread::current().id());
            }
            req
        }
    }

    fn log_config() -> oysterpack_log::LogConfig {
        oysterpack_log::config::LogConfigBuilder::new(oysterpack_log::Level::Info).build()
    }

    /// Run the client portion of the program.
    fn client(url: &str, sleep_ms: u32) -> Result<Duration, nng::Error> {
        let mut s = Socket::new(nng::Protocol::Req0)?;
        let dialer = nng::dialer::DialerOptions::new(&s, url)?;
        let dialer = match dialer.start(true) {
            Ok(dialer) => dialer,
            Err((_, err)) => panic!(err),
        };

        let msg_bytes = bincode::serialize(&sleep_ms).unwrap();
        let mut req = nng::Message::with_capacity(msg_bytes.len()).unwrap();
        req.push_back(&msg_bytes).unwrap();

        info!("sending client request ...");
        let start = Instant::now();
        s.send(req)?;
        s.recv()?;
        let dur = Instant::now().duration_since(start);
        info!("Request({}) took {:?}", sleep_ms, dur);
        Ok(dur)
    }

    #[test]
    fn rpc_server() {
        oysterpack_log::init(log_config(), oysterpack_log::StderrLogger);

        const url: &str = "inproc://test";

        // the client should be able to connect async after the server has started
        let client_thread_handle = thread::spawn(|| client(url, 0).unwrap());

        // start a server with 2 aio contexts
        let listener_settings =
            super::ListenerSettings::new(url).set_aio_count(NonZeroUsize::new(2).unwrap());

        let server = super::Server::spawn(listener_settings, None, Arc::new(Sleep), None).unwrap();

        // wait for the client background request completes
        client_thread_handle.join();

        for _ in 0..10 {
            client(url, 0).unwrap();
        }

        // submit a long running request, which will block one of the aio contexts for 1 sec
        let (s, r) = crossbeam::channel::bounded(0);
        const SLEEP_TIME: u32 = 1000;
        thread::spawn(move || {
            s.send(()).unwrap();
            client(url, SLEEP_TIME).unwrap();
        });
        r.recv().unwrap();
        info!("client with {} ms request has started", SLEEP_TIME);
        // give the client a chance to send the request
        thread::sleep_ms(10);

        // requests should still be able to flow through because one of aio contexts is available
        for _ in 0..10 {
            let duration = client(url, 0).unwrap();
            assert!(duration < Duration::from_millis(50));
        }

        info!("client requests are done.");

        server.stop();
        server.wait();
    }

    #[test]
    fn rpc_server_all_contexts_busy() {
        oysterpack_log::init(log_config(), oysterpack_log::StderrLogger);

        const url: &str = "inproc://test";

        // the client should be able to connect async after the server has started
        let client_thread_handle = thread::spawn(|| client(url, 0).unwrap());

        // start a server with 2 aio contexts
        let listener_settings =
            super::ListenerSettings::new(url).set_aio_count(NonZeroUsize::new(2).unwrap());

        let server = super::Server::spawn(listener_settings, None, Arc::new(Sleep), None).unwrap();

        // wait for the client background request completes
        client_thread_handle.join();

        // submit long running request, which will block one of the aio contexts for 1 sec
        let (s1, r1) = crossbeam::channel::bounded(0);
        let (s2, r2) = crossbeam::channel::bounded(0);
        const SLEEP_TIME: u32 = 1000;
        thread::spawn(move || {
            s1.send(()).unwrap();
            client(url, SLEEP_TIME).unwrap();
        });
        thread::spawn(move || {
            s2.send(()).unwrap();
            client(url, SLEEP_TIME).unwrap();
        });
        r1.recv().unwrap();
        r2.recv().unwrap();
        info!(
            "client requests with {} ms request have started",
            SLEEP_TIME
        );
        // give the client a chance to send the request
        thread::sleep_ms(10);

        let duration = client(url, 0).unwrap();
        assert!(
            duration > Duration::from_millis(500),
            "client request should have been blocked waiting for aio context to become available"
        );

        server.stop();
        server.wait();
    }

}
