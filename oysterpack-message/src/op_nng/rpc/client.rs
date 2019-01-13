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

use nng::{
    self,
    dialer::{Dialer, DialerOptions},
    Socket,
};
use oysterpack_errors::{
    Error, op_error
};
use serde::{Deserialize, Serialize};
use std::{num::NonZeroUsize, time::Duration};

/// nng RPC client
#[derive(Debug)]
pub struct Client {}

impl Client {}

/// Dialer Settings
#[derive(Debug, Serialize, Deserialize)]
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
            recv_max_size: None,
            no_delay: None,
            keep_alive: None,
            non_blocking: false,
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

pub mod errors {
    //! client related errors

    use super::*;
    use oysterpack_errors::IsError;
    use std::fmt;

    /// An error occurred when setting a dialer option.
    #[derive(Debug)]
    pub struct DialerSetOptError(nng::Error);

    impl DialerSetOptError {
        /// Error Id
        pub const ERROR_ID: oysterpack_errors::Id =
            oysterpack_errors::Id(1870617351358933523700534508070132261);
        /// Level::Alert
        pub const ERROR_LEVEL: oysterpack_errors::Level = oysterpack_errors::Level::Alert;
    }

    impl IsError for DialerSetOptError {
        fn error_id(&self) -> oysterpack_errors::Id {
            Self::ERROR_ID
        }

        fn error_level(&self) -> oysterpack_errors::Level {
            Self::ERROR_LEVEL
        }
    }

    impl fmt::Display for DialerSetOptError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "Failed to set dialer option: {}", self.0)
        }
    }

    impl From<nng::Error> for DialerSetOptError {
        fn from(err: nng::Error) -> DialerSetOptError {
            DialerSetOptError(err)
        }
    }

    /// Failed to create dialer instance
    #[derive(Debug)]
    pub struct DialerCreateError(nng::Error);

    impl DialerCreateError {
        /// Error Id
        pub const ERROR_ID: oysterpack_errors::Id =
            oysterpack_errors::Id(1870617814817456819801511817900043129);
        /// Level::Alert
        pub const ERROR_LEVEL: oysterpack_errors::Level = oysterpack_errors::Level::Alert;
    }

    impl IsError for DialerCreateError {
        fn error_id(&self) -> oysterpack_errors::Id {
            Self::ERROR_ID
        }

        fn error_level(&self) -> oysterpack_errors::Level {
            Self::ERROR_LEVEL
        }
    }

    impl fmt::Display for DialerCreateError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "Failed to create dialer instance: {}", self.0)
        }
    }

    impl From<nng::Error> for DialerCreateError {
        fn from(err: nng::Error) -> DialerCreateError {
            DialerCreateError(err)
        }
    }

    /// Failed to start dialer instance
    #[derive(Debug)]
    pub struct DialerStartError(nng::Error);

    impl DialerStartError {
        /// Error Id
        pub const ERROR_ID: oysterpack_errors::Id =
            oysterpack_errors::Id(1870618072331851255202721873004562985);
        /// Level::Alert
        pub const ERROR_LEVEL: oysterpack_errors::Level = oysterpack_errors::Level::Alert;
    }

    impl IsError for DialerStartError {
        fn error_id(&self) -> oysterpack_errors::Id {
            Self::ERROR_ID
        }

        fn error_level(&self) -> oysterpack_errors::Level {
            Self::ERROR_LEVEL
        }
    }

    impl fmt::Display for DialerStartError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "Failed to start dialer: {}", self.0)
        }
    }

    impl From<nng::Error> for DialerStartError {
        fn from(err: nng::Error) -> DialerStartError {
            DialerStartError(err)
        }
    }
}

#[allow(warnings)]
#[cfg(test)]
mod tests {

    use super::*;
    use crate::op_nng::rpc::{server::*, MessageProcessor, MessageProcessorFactory};
    use log::*;
    use oysterpack_uid::ULID;
    use serde::{Deserialize, Serialize};
    use std::{num::NonZeroUsize, sync::Arc, thread};

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
                Request::Sleep(_) => {
                    info!("received Sleep message on {:?}", thread::current().id())
                }
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

        // TODO

        server.stop();
        server.join();
    }

}
