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

use nng::{self, listener::Listener, options::Options, Socket};
use oysterpack_errors::{op_error, Error, ErrorMessage};
use serde::{Deserialize, Serialize};
use std::{fmt, sync::Arc, num::NonZeroUsize};
use super::MessageHandler;

/// nng RPC server
pub struct Server {
    listener: nng::listener::Listener,
    socket: Arc<Socket>,
}

impl Server {

    /// starts the server using the specified settings
    pub fn start<Handler>(
        listener_settings: ListenerSettings,
        socket: Arc<Socket>,
        message_handler_pool: Vec<Handler>
    ) -> Result<Server, Error> where Handler: MessageHandler<nng::Message, nng::Message> {
        let workers: Vec<_> = (0..listener_settings.aio_count)
            .map(|i| create_worker(i, &s))
            .collect::<Result<_, _>>()?;
        let listener =  listener_settings.start_listener(&socket)?;

        // Now start all of the workers listening.
        for (a, c) in &workers {
            a.recv(c)?;
        }

        Ok(Server {
            listener,
            socket,
        })
    }

    /// socket settings
    pub fn socket_settings(&self) -> SocketSettings {
        SocketSettings::from(&*self.socket)
    }

    fn create_worker(s: &Socket) -> Result<(nng::aio::Aio, nng::aio::Context), Error>
    {
        let mut state = State::Recv;

        let ctx = nng::aio:: Context::new(s)?;
        let ctx_clone = ctx.clone();
        let aio = nng::aio::Aio::with_callback(move |aio| worker_callback(aio, &ctx_clone, &mut state))
            .map_error(|err| {
                op_error!(CreateAioWorkerError(ErrorMessage(err.to_string())))
            })?;

        Ok((aio, ctx))
    }

}

impl fmt::Debug for Server {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Server(listener.id={})", self.listener.id())
    }
}

struct AioWorker {
    state: AioWorkerState
}

impl AioWorker {
    fn callback(&mut self, aio: nng::aio::Aio) {
        // TODO
    }
}

/// State of a request.
#[derive(Debug, Copy, Clone)]
enum AioWorkerState {
    Recv,
    Wait,
    Send,
}

/// Listener settings
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct SocketSettings {
    max_ttl: Option<u8>
}

impl SocketSettings {
    /// The maximum number of "hops" a message may traverse.
    ///
    /// The intention here is to prevent forwarding loops in device chains. Note that not all protocols
    /// support this option and those that do generally have a default value of 8.
    ///
    /// Each node along a forwarding path may have its own value for the maximum time-to-live, and
    /// performs its own checks before forwarding a message. Therefore it is helpful if all nodes in
    /// the topology use the same value for this option.
    pub fn max_ttl(&self) -> Option<u8> {
        self.max_ttl
    }

    // TODO - add the rest of the socket options
}

impl From<&Socket> for SocketSettings {

    fn from(socket: &Socket) -> SocketSettings {
        SocketSettings {
            max_ttl: socket.get_opt::<nng::options::MaxTtl>().ok()
        }
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
    aio_count: usize
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
            aio_count: 1
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
        let map_err = |err: nng::Error| -> errors::ListenerStartError {
            errors::ListenerStartError(self.clone(), ErrorMessage(err.to_string()))
        };

        let options = nng::listener::ListenerOptions::new(socket, self.url())
            .map_err(|err| op_error!(map_err(err)))?;

        if let Some(option) = self.recv_max_size.as_ref() {
            options
                .set_opt::<nng::options::RecvMaxSize>(*option)
                .map_err(|err| op_error!(map_err(err)))?;
        }

        if let Some(option) = self.no_delay.as_ref() {
            options
                .set_opt::<nng::options::transport::tcp::NoDelay>(*option)
                .map_err(|err| op_error!(map_err(err)))?;
        }

        if let Some(option) = self.keep_alive.as_ref() {
            options
                .set_opt::<nng::options::transport::tcp::KeepAlive>(*option)
                .map_err(|err| op_error!(map_err(err)))?;
        }

        options
            .start(self.non_blocking)
            .map_err(|(_options, err)| op_error!(map_err(err)))
    }

    /// the address that the server is listening on
    pub fn url(&self) -> &str {
        &self.url
    }

    /// if true, then it binds to the address asynchronously
    pub fn non_blocking(&self) -> bool {
        self.non_blocking
    }

    /// number of async IO operations that can be performed concurrently
    pub fn aio_count(&self) -> usize {
        self.aio_count
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
        settings.aio_count = count.get();
        settings
    }
}

pub mod errors {
    //! server errors

    use super::*;
    use oysterpack_errors::IsError;
    use std::fmt;

    /// Failed to start listener
    #[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
    pub struct ListenerStartError(pub ListenerSettings, pub ErrorMessage);

    impl ListenerStartError {
        /// Error Id
        pub const ERROR_ID: oysterpack_errors::Id =
            oysterpack_errors::Id(1870302624499038905208367552914704572);
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
            write!(f, "Failed to start listener: {} : {:?}", self.1, self.0)
        }
    }

    /// Failed to create Aio worker
    #[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
    pub struct CreateAioWorkerError(pub ErrorMessage);

    impl CreateAioWorkerError {
        /// Error Id
        pub const ERROR_ID: oysterpack_errors::Id =
            oysterpack_errors::Id(1870313585057930209197631174282877574);
        /// Level::Error
        pub const ERROR_LEVEL: oysterpack_errors::Level = oysterpack_errors::Level::Error;
    }

    impl IsError for CreateAioWorkerError {
        fn error_id(&self) -> oysterpack_errors::Id {
            Self::ERROR_ID
        }

        fn error_level(&self) -> oysterpack_errors::Level {
            Self::ERROR_LEVEL
        }
    }

    impl fmt::Display for CreateAioWorkerError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "Failed to create Aio worker: {}", self.0)
        }
    }

}
