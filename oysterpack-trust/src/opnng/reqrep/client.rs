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

//! Provides an ReqRep nng client

use crate::concurrent::messaging::reqrep;
use crate::opnng::{self, config::SocketConfigError};
use failure::Fail;
use futures::future::Future;
use nng::options::Options;
use serde::{Deserialize, Serialize};
use std::{fmt, num::NonZeroUsize, pin::Pin, time::Duration};

/// nng client
pub struct NngClient {
    socket: nng::Socket,
    dialer: nng::Dialer,
}

impl NngClient {
    /// constructor
    pub fn new(
        socket_config: Option<SocketConfig>,
        dialer_config: DialerConfig,
    ) -> Result<Self, NngClientError> {
        let socket = SocketConfig::create_socket(socket_config)
            .map_err(NngClientError::SockectCreateFailure)?;
        let dialer = dialer_config
            .start_dialer(&socket)
            .map_err(NngClientError::DialerStartError)?;
        Ok(Self { socket, dialer })
    }
}

impl reqrep::Processor<nng::Message, nng::Message> for NngClient {
    fn process(&mut self, req: nng::Message) -> Pin<Box<Future<Output = nng::Message> + Send>> {
        unimplemented!()
    }
}

impl fmt::Debug for NngClient {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "NngClient")
    }
}

/// NngClient related errors
#[derive(Debug, Fail)]
pub enum NngClientError {
    /// Failed to create DialerOptions
    #[fail(display = "Failed to create DialerOptions: {}", _0)]
    SockectCreateFailure(SocketConfigError),
    /// Failed to start Dialer
    #[fail(display = "Failed to start Dialer: {}", _0)]
    DialerStartError(DialerConfigError),
}

/// Socket Settings
#[derive(Debug, Serialize, Deserialize)]
pub struct SocketConfig {
    reconnect_min_time: Option<Duration>,
    reconnect_max_time: Option<Duration>,
    resend_time: Option<Duration>,
    socket_config: Option<opnng::config::SocketConfig>,
}

impl SocketConfig {
    pub(crate) fn create_socket(
        socket_config: Option<SocketConfig>,
    ) -> Result<nng::Socket, SocketConfigError> {
        let mut socket =
            nng::Socket::new(nng::Protocol::Req0).map_err(SocketConfigError::SocketCreateFailed)?;
        socket.set_nonblocking(true);
        match socket_config {
            Some(socket_config) => socket_config.apply(socket),
            None => Ok(socket),
        }
    }

    /// set socket options
    pub(crate) fn apply(&self, socket: nng::Socket) -> Result<nng::Socket, SocketConfigError> {
        let socket = if let Some(settings) = self.socket_config.as_ref() {
            settings.apply(socket)?
        } else {
            socket
        };

        socket
            .set_opt::<nng::options::ReconnectMinTime>(self.reconnect_min_time)
            .map_err(SocketConfigError::ReconnectMinTime)?;

        socket
            .set_opt::<nng::options::ReconnectMaxTime>(self.reconnect_max_time)
            .map_err(SocketConfigError::ReconnectMaxTime)?;

        socket
            .set_opt::<nng::options::protocol::reqrep::ResendTime>(self.resend_time)
            .map_err(SocketConfigError::ResendTime)?;

        Ok(socket)
    }

    /// Socket settings
    pub fn socket_config(&self) -> Option<&opnng::config::SocketConfig> {
        self.socket_config.as_ref()
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
    pub fn set_socket_config(self, config: opnng::config::SocketConfig) -> Self {
        let mut this = self;
        this.socket_config = Some(config);
        this
    }
}

/// Dialer Settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialerConfig {
    url: String,
    parallelism: usize,
    recv_max_size: Option<usize>,
    no_delay: Option<bool>,
    keep_alive: Option<bool>,
    reconnect_min_time: Option<Duration>,
    reconnect_max_time: Option<Duration>,
}

impl DialerConfig {
    /// constructor
    /// - parallelism = number of logical CPUs
    pub fn new(url: &str) -> DialerConfig {
        DialerConfig {
            url: url.to_string(),
            recv_max_size: None,
            no_delay: None,
            keep_alive: None,
            parallelism: num_cpus::get(),
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
    pub fn start_dialer(self, socket: &nng::Socket) -> Result<nng::Dialer, DialerConfigError> {
        let dialer_options = nng::DialerOptions::new(socket, self.url.as_str())
            .map_err(DialerConfigError::DialerOptionsCreateFailed)?;

        if let Some(recv_max_size) = self.recv_max_size {
            dialer_options
                .set_opt::<nng::options::RecvMaxSize>(recv_max_size)
                .map_err(DialerConfigError::RecvMaxSize)?;
        }

        if let Some(no_delay) = self.no_delay {
            dialer_options
                .set_opt::<nng::options::transport::tcp::NoDelay>(no_delay)
                .map_err(DialerConfigError::TcpNoDelay)?;
        }

        if let Some(keep_alive) = self.keep_alive {
            dialer_options
                .set_opt::<nng::options::transport::tcp::KeepAlive>(keep_alive)
                .map_err(DialerConfigError::TcpKeepAlive)?;
        }

        dialer_options
            .set_opt::<nng::options::ReconnectMinTime>(self.reconnect_min_time)
            .map_err(DialerConfigError::ReconnectMinTime)?;

        dialer_options
            .set_opt::<nng::options::ReconnectMaxTime>(self.reconnect_max_time)
            .map_err(DialerConfigError::ReconnectMaxTime)?;

        dialer_options
            .start(true)
            .map_err(|(_options, err)| DialerConfigError::DialerStartError(err))
    }

    /// the address that the server is listening on
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Max number of async IO operations that can be performed concurrently, which corresponds to the number
    /// of socket contexts that will be created.
    /// - if not specified, then it will default to the number of logical CPUs
    pub fn parallelism(&self) -> usize {
        self.parallelism
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

    /// set the max capacity of concurrent async requests
    pub fn set_parallelism(self, count: NonZeroUsize) -> Self {
        let mut settings = self;
        settings.parallelism = count.get();
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

/// Dialer config related errors
#[derive(Debug, Fail)]
pub enum DialerConfigError {
    /// Failed to create DialerOptions
    #[fail(display = "Failed to create DialerOptions: {}", _0)]
    DialerOptionsCreateFailed(nng::Error),
    /// Failed to set the RecvMaxSize option
    #[fail(display = "Failed to set the RecvMaxSize option: {}", _0)]
    RecvMaxSize(nng::Error),
    /// Failed to set the TcpNoDelay option
    #[fail(display = "Failed to set the TcpNoDelay option: {}", _0)]
    TcpNoDelay(nng::Error),
    /// Failed to set the TcpKeepAlive option
    #[fail(display = "Failed to set the TcpKeepAlive option: {}", _0)]
    TcpKeepAlive(nng::Error),
    /// Failed to set the ReconnectMinTime option
    #[fail(display = "Failed to set the ReconnectMinTime option: {}", _0)]
    ReconnectMinTime(nng::Error),
    /// Failed to set the ReconnectMaxTime option
    #[fail(display = "Failed to set the ReconnectMaxTime option: {}", _0)]
    ReconnectMaxTime(nng::Error),
    /// Failed to start Dialer
    #[fail(display = "Failed to start Dialer: {}", _0)]
    DialerStartError(nng::Error),
}
