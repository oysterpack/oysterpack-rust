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

//! nng RPC client

use crate::op_nng::{
    errors::{AioCreateError, AioReceiveError, AioSendError},
    new_aio_context, try_from_nng_message, try_into_nng_message, SocketSettings,
};
use crossbeam::stack::TreiberStack;
use nng::{
    self,
    aio::{Aio, Context},
    dialer::{Dialer, DialerOptions},
    options::Options,
    Socket,
};
use oysterpack_errors::{op_error, Error};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{fmt, num::NonZeroUsize, panic::RefUnwindSafe, time::Duration};

pub mod errors;

#[allow(warnings)]
#[cfg(test)]
mod tests;

/// Async reply handler that is used as a callback by the AsyncClient
pub trait ReplyHandler<Rep>: Send + Sized + RefUnwindSafe + 'static
where
    Rep: Serialize + DeserializeOwned,
{
    /// reply callback
    fn on_reply(&mut self, result: Result<Rep, Error>);
}

/// nng async client
pub struct AsyncClient {
    dialer: nng::dialer::Dialer,
    socket: nng::Socket,
    aio_contexts: TreiberStack<AioContext>,
}

impl AsyncClient {
    /// Sends the request and invokes the callback with the reply asynchronously
    /// - the messages are snappy compressed and bincode serialized - see the [marshal]() module
    /// - if the req
    pub fn send_with_callback<Req, Rep, Callback>(
        &mut self,
        req: &Req,
        cb: Callback,
    ) -> Result<(), Error>
    where
        Req: Serialize + DeserializeOwned,
        Rep: Serialize + DeserializeOwned,
        Callback: ReplyHandler<Rep>,
    {
        let mut cb = cb;
        let msg = try_into_nng_message(req)?;
        let ctx: nng::aio::Context = new_aio_context(&self.socket)?;
        let aio = nng::aio::Aio::with_callback(move |aio| {
            match aio.result().unwrap() {
                Ok(_) => {
                    // since the aio receive operation was successful, then there will always be a message to get
                    // thus it is safe to invoke unwrap
                    let rep = aio.get_msg().unwrap();
                    match try_from_nng_message::<Rep>(&rep) {
                        Ok(rep) => {
                            cb.on_reply(Ok(rep));
                        }
                        Err(err) => {
                            cb.on_reply(Err(err));
                        }
                    }
                }
                Err(err) => {
                    cb.on_reply(Err(op_error!(AioReceiveError::from(err))));
                }
            }
        })
        .map_err(|err| op_error!(AioCreateError::from(err)))?;

        let mut aio_context = AioContext {
            aio,
            ctx
        };
        aio_context.send(msg)?;

        self.aio_contexts.push(aio_context);

        Ok(())
    }

    /// constructor
    pub fn dial(dialer_settings: DialerSettings) -> Result<Self, Error> {
        Builder::new(dialer_settings).async_client()
    }

    /// constructor
    pub fn dial_with_socket_settings(
        dialer_settings: DialerSettings,
        socket_settings: ClientSocketSettings,
    ) -> Result<Self, Error> {
        Builder::new(dialer_settings)
            .socket_settings(socket_settings)
            .async_client()
    }
}

impl fmt::Debug for AsyncClient {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.dialer.get_opt::<nng::options::Url>() {
            Ok(url) => write!(f, "AsyncClient(Socket({}), Url({}))", self.socket.id(), url),
            Err(err) => write!(f, "AsyncClient(Socket({}), Err({}))", self.socket.id(), err),
        }
    }
}

struct AioContext {
    aio: Aio,
    ctx: Context,
}

impl AioContext {

    fn send(&mut self, msg: nng::Message) -> Result<(), Error> {
        self.aio.send(&self.ctx, msg)
            .map_err(|(_msg, err)| op_error!(AioSendError::from(err)))
    }
}


impl fmt::Debug for AioContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "AioWorker({})", self.ctx.id())
    }
}

/// nng RPC client
pub struct SyncClient {
    // the order is important because Rust will drop fields in the order listed
    // the dialer must be dropped before the socket, otherwise the following error occurs
    //
    // thread 'op_nng::rpc::client::tests::sync_client' panicked at 'Unexpected error code while closing dialer (12)', /home/alfio/.cargo/registry/src/github.com-1ecc6299db9ec823/nng-0.3.0/src/dialer.rs:104:3
    //
    // i.e., the dialer must be closed before the socket is closed
    dialer: nng::dialer::Dialer,
    socket: nng::Socket,
}

impl SyncClient {
    /// Sends the request and wait for a reply synchronously
    /// - the messages are snappy compressed and bincode serialized - see the [marshal]() module
    pub fn send<Req, Rep>(&mut self, req: &Req) -> Result<Rep, Error>
    where
        Req: Serialize + DeserializeOwned,
        Rep: Serialize + DeserializeOwned,
    {
        let msg = try_into_nng_message(req)?;
        self.socket
            .send(msg)
            .map_err(|err| op_error!(errors::SocketSendError::from(err)))?;
        let rep = self
            .socket
            .recv()
            .map_err(|err| op_error!(errors::SocketRecvError::from(err)))?;
        try_from_nng_message(&rep)
    }

    /// constructor
    pub fn dial(dialer_settings: DialerSettings) -> Result<Self, Error> {
        Builder::new(dialer_settings).sync_client()
    }

    /// constructor
    pub fn dial_with_socket_settings(
        dialer_settings: DialerSettings,
        socket_settings: ClientSocketSettings,
    ) -> Result<Self, Error> {
        Builder::new(dialer_settings)
            .socket_settings(socket_settings)
            .sync_client()
    }
}

impl fmt::Debug for SyncClient {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.dialer.get_opt::<nng::options::Url>() {
            Ok(url) => write!(f, "SyncClient(Socket({}), Url({}))", self.socket.id(), url),
            Err(err) => write!(f, "SyncClient(Socket({}), Err({}))", self.socket.id(), err),
        }
    }
}

/// Client builder
#[derive(Debug)]
pub struct Builder {
    dialer_settings: DialerSettings,
    socket_settings: Option<ClientSocketSettings>,
}

impl Builder {
    /// constructor
    pub fn new(dialer_settings: DialerSettings) -> Builder {
        Builder {
            dialer_settings,
            socket_settings: None,
        }
    }

    /// Configures the socket
    pub fn socket_settings(self, socket_settings: ClientSocketSettings) -> Builder {
        let mut builder = self;
        builder.socket_settings = Some(socket_settings);
        builder
    }

    /// builds a new SyncClient
    pub fn sync_client(self) -> Result<SyncClient, Error> {
        let mut this = self;
        let socket = Builder::create_socket(this.socket_settings.take())?;
        let dialer = this.dialer_settings.start_dialer(&socket)?;
        Ok(SyncClient { socket, dialer })
    }

    /// builds a new AsyncClient
    pub fn async_client(self) -> Result<AsyncClient, Error> {
        let mut this = self;
        let socket = Builder::create_socket(this.socket_settings.take())?;
        let dialer = this.dialer_settings.start_dialer(&socket)?;
        Ok(AsyncClient {
            socket,
            dialer,
            aio_contexts: TreiberStack::new()
        })
    }

    fn create_socket(
        socket_settings: Option<ClientSocketSettings>,
    ) -> Result<nng::Socket, Error> {
        let socket = nng::Socket::new(nng::Protocol::Req0)
            .map_err(|err| op_error!(errors::SocketCreateError::from(err)))?;
        match socket_settings {
            Some(socket_settings) => socket_settings.apply(socket),
            None => Ok(socket),
        }
    }
}

/// Socket Settings
#[derive(Debug, Serialize, Deserialize)]
pub struct ClientSocketSettings {
    reconnect_min_time: Option<Duration>,
    reconnect_max_time: Option<Duration>,
    resend_time: Option<Duration>,
    socket_settings: Option<SocketSettings>,
}

impl ClientSocketSettings {
    /// set socket options
    pub(crate) fn apply(&self, socket: Socket) -> Result<Socket, Error> {
        let socket = if let Some(settings) = self.socket_settings.as_ref() {
            settings.apply(socket)?
        } else {
            socket
        };

        socket
            .set_opt::<nng::options::ReconnectMinTime>(self.reconnect_min_time)
            .map_err(|err| op_error!(errors::SocketSetOptError::from(err)))?;

        socket
            .set_opt::<nng::options::ReconnectMaxTime>(self.reconnect_max_time)
            .map_err(|err| op_error!(errors::SocketSetOptError::from(err)))?;

        socket
            .set_opt::<nng::options::protocol::reqrep::ResendTime>(self.resend_time)
            .map_err(|err| op_error!(errors::SocketSetOptError::from(err)))?;

        Ok(socket)
    }

    /// Socket settings
    pub fn socket_settings(&self) -> Option<&SocketSettings> {
        self.socket_settings.as_ref()
    }

    /// Amount of time to wait before sending a new request.
    ///
    /// When a new request is started, a timer of this duration is also started. If no reply is
    /// received before this timer expires, then the request will be resent. (Requests are also
    /// automatically resent if the peer to whom the original request was sent disconnects, or if a
    /// peer becomes available while the requester is waiting for an available peer.)
    pub fn resend_time(&self) -> Option<Duration> {
        self.resend_time
    }

    /// The minimum amount of time to wait before attempting to establish a connection after a previous
    /// attempt has failed.
    ///
    /// If set on a Socket, this value becomes the default for new dialers. Individual dialers can
    /// then override the setting.
    pub fn reconnect_min_time(&self) -> Option<Duration> {
        self.reconnect_min_time
    }

    ///The maximum amount of time to wait before attempting to establish a connection after a previous
    /// attempt has failed.
    ///
    /// If this is non-zero, then the time between successive connection attempts will start at the
    /// value of ReconnectMinTime, and grow exponentially, until it reaches this value. If this value
    /// is zero, then no exponential back-off between connection attempts is done, and each attempt
    /// will wait the time specified by ReconnectMinTime. This can be set on a socket, but it can
    /// also be overridden on an individual dialer.
    pub fn reconnect_max_time(&self) -> Option<Duration> {
        self.reconnect_max_time
    }

    /// The minimum amount of time to wait before attempting to establish a connection after a previous
    /// attempt has failed.
    pub fn set_reconnect_min_time(self, reconnect_min_time: Duration) -> Self {
        let mut this = self;
        this.reconnect_min_time = Some(reconnect_min_time);
        this
    }

    ///The maximum amount of time to wait before attempting to establish a connection after a previous
    /// attempt has failed.
    pub fn set_reconnect_max_time(self, reconnect_max_time: Duration) -> Self {
        let mut this = self;
        this.reconnect_max_time = Some(reconnect_max_time);
        this
    }

    /// Amount of time to wait before sending a new request.
    pub fn set_resend_time(self, resend_time: Duration) -> Self {
        let mut this = self;
        this.resend_time = Some(resend_time);
        this
    }

    /// Apply socket settings
    pub fn set_socket_settings(self, settings: SocketSettings) -> Self {
        let mut this = self;
        this.socket_settings = Some(settings);
        this
    }
}

/// Dialer Settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialerSettings {
    url: String,
    non_blocking: bool,
    aio_context_count: Option<usize>,
    recv_max_size: Option<usize>,
    no_delay: Option<bool>,
    keep_alive: Option<bool>,
    reconnect_min_time: Option<Duration>,
    reconnect_max_time: Option<Duration>,
}

impl DialerSettings {
    /// constructor
    pub fn new(url: &str) -> DialerSettings {
        DialerSettings {
            url: url.to_string(),
            non_blocking: false,
            recv_max_size: None,
            no_delay: None,
            keep_alive: None,
            aio_context_count: None,
            reconnect_min_time: None,
            reconnect_max_time: None,
        }
    }

    /// Start a socket dialer.
    ///
    /// Normally, the first attempt to connect to the dialer's address is done synchronously, including
    /// any necessary name resolution. As a result, a failure, such as if the connection is refused,
    /// will be returned immediately, and no further action will be taken.
    ///
    /// However, if nonblocking is specified, then the connection attempt is made asynchronously.
    ///
    /// Furthermore, if the connection was closed for a synchronously dialed connection, the dialer
    /// will still attempt to redial asynchronously.
    ///
    /// The returned handle controls the life of the dialer. If it is dropped, the dialer is shut down
    /// and no more messages will be received on it.
    pub fn start_dialer(self, socket: &Socket) -> Result<Dialer, Error> {
        let dialer_options = DialerOptions::new(socket, self.url.as_str())
            .map_err(|err| op_error!(errors::DialerCreateError::from(err)))?;

        if let Some(recv_max_size) = self.recv_max_size {
            dialer_options
                .set_opt::<nng::options::RecvMaxSize>(recv_max_size)
                .map_err(|err| op_error!(errors::DialerSetOptError::from(err)))?;
        }

        if let Some(no_delay) = self.no_delay {
            dialer_options
                .set_opt::<nng::options::transport::tcp::NoDelay>(no_delay)
                .map_err(|err| op_error!(errors::DialerSetOptError::from(err)))?;
        }

        if let Some(keep_alive) = self.keep_alive {
            dialer_options
                .set_opt::<nng::options::transport::tcp::KeepAlive>(keep_alive)
                .map_err(|err| op_error!(errors::DialerSetOptError::from(err)))?;
        }

        dialer_options
            .set_opt::<nng::options::ReconnectMinTime>(self.reconnect_min_time)
            .map_err(|err| op_error!(errors::DialerSetOptError::from(err)))?;

        dialer_options
            .set_opt::<nng::options::ReconnectMaxTime>(self.reconnect_max_time)
            .map_err(|err| op_error!(errors::DialerSetOptError::from(err)))?;

        dialer_options
            .start(self.non_blocking)
            .map_err(|(_options, err)| op_error!(errors::DialerStartError::from(err)))
    }

    /// the address that the server is listening on
    pub fn url(&self) -> &str {
        &self.url
    }

    /// if true, then it binds to the address asynchronously
    pub fn non_blocking(&self) -> bool {
        self.non_blocking
    }

    /// Number of async IO operations that can be performed concurrently, which corresponds to the number
    /// of socket contexts that will be created.
    ///
    /// If None is returned or count is zero, then IO will be synchronous.
    pub fn aio_context_count(&self) -> Option<usize> {
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

    /// The minimum amount of time to wait before attempting to establish a connection after a previous
    /// attempt has failed.
    ///
    /// If set on a Socket, this value becomes the default for new dialers. Individual dialers can
    /// then override the setting.
    pub fn reconnect_min_time(&self) -> Option<Duration> {
        self.reconnect_min_time
    }

    ///The maximum amount of time to wait before attempting to establish a connection after a previous
    /// attempt has failed.
    ///
    /// If this is non-zero, then the time between successive connection attempts will start at the
    /// value of ReconnectMinTime, and grow exponentially, until it reaches this value. If this value
    /// is zero, then no exponential back-off between connection attempts is done, and each attempt
    /// will wait the time specified by ReconnectMinTime. This can be set on a socket, but it can
    /// also be overridden on an individual dialer.
    pub fn reconnect_max_time(&self) -> Option<Duration> {
        self.reconnect_max_time
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
        settings.aio_context_count = Some(count.get());
        settings
    }

    /// The minimum amount of time to wait before attempting to establish a connection after a previous
    /// attempt has failed.
    pub fn set_reconnect_min_time(self, reconnect_min_time: Duration) -> Self {
        let mut this = self;
        this.reconnect_min_time = Some(reconnect_min_time);
        this
    }

    ///The maximum amount of time to wait before attempting to establish a connection after a previous
    /// attempt has failed.
    pub fn set_reconnect_max_time(self, reconnect_max_time: Duration) -> Self {
        let mut this = self;
        this.reconnect_max_time = Some(reconnect_max_time);
        this
    }
}
