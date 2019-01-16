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

//! Provides support for [nng](https://nanomsg.github.io/nng/index.html) messaging protocols.

use crate::{
    errors::{BincodeDeserializeError, NngMessageError},
    marshal,
};
use oysterpack_errors::{op_error, Error, ErrorMessage};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use nng::{options::Options, Socket};
use std::{
    num::{NonZeroU16, NonZeroUsize},
    time::Duration,
};

pub mod errors;
pub mod rpc;

/// Tries to decode the message into the specified type
pub fn try_from_nng_message<T>(msg: &nng::Message) -> Result<T, Error>
where
    T: Serialize + DeserializeOwned,
{
    marshal::decode(&**msg)
        .map_err(|err| op_error!(BincodeDeserializeError(ErrorMessage(err.to_string()))))
}

/// Tries to encode the message into an nng message
pub fn try_into_nng_message<T>(msg: &T) -> Result<nng::Message, Error>
where
    T: Serialize + DeserializeOwned,
{
    let bytes = marshal::encode(msg)?;

    let mut msg = nng::Message::with_capacity(bytes.len()).map_err(|err| {
        op_error!(NngMessageError::from(ErrorMessage(format!(
            "Failed to create an empty message with capacity = {}: {}",
            bytes.len(),
            err
        ))))
    })?;
    msg.push_back(&bytes).map_err(|err| {
        op_error!(NngMessageError::from(ErrorMessage(format!(
            "Failed to append data to the back of the message body: {}",
            err
        ))))
    })?;
    Ok(msg)
}

/// Creates a new aio context
pub fn new_aio_context(socket: &nng::Socket) -> Result<nng::aio::Context, Error> {
    nng::aio::Context::new(&socket)
        .map_err(|err| op_error!(errors::AioContextCreateError::from(err)))
}

/// Sends the message asynchronously and maps the nng error over to an OysterPack Error
pub fn send_request_async(
    aio: &nng::aio::Aio,
    ctx: &nng::aio::Context,
    msg: nng::Message,
) -> Result<(), Error> {
    aio.send(ctx, msg)
        .map_err(|(_msg, err)| op_error!(errors::AioSendError::from(err)))
}

/// Socket settings
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, Default)]
pub struct SocketSettings {
    recv_buffer_size: Option<NonZeroU16>,
    recv_max_size: Option<NonZeroUsize>,
    recv_timeout: Option<Duration>,
    send_timeout: Option<Duration>,
    send_buffer_size: Option<NonZeroU16>,
    max_ttl: Option<u8>,
    socket_name: Option<String>,
    tcp_no_delay: Option<bool>,
    tcp_keep_alive: Option<bool>,
}

impl SocketSettings {
    /// set socket options
    pub(crate) fn apply(&self, socket: Socket) -> Result<Socket, Error> {
        if let Some(opt) = self.recv_buffer_size {
            socket
                .set_opt::<nng::options::RecvBufferSize>(i32::from(opt.get()))
                .map_err(|err| op_error!(errors::SocketSetOptError::from(err)))?;
        }

        if let Some(opt) = self.send_buffer_size {
            socket
                .set_opt::<nng::options::SendBufferSize>(i32::from(opt.get()))
                .map_err(|err| op_error!(errors::SocketSetOptError::from(err)))?;
        }

        if let Some(opt) = self.recv_max_size {
            socket
                .set_opt::<nng::options::RecvMaxSize>(opt.get())
                .map_err(|err| op_error!(errors::SocketSetOptError::from(err)))?;
        }

        socket
            .set_opt::<nng::options::RecvTimeout>(self.recv_timeout)
            .map_err(|err| op_error!(errors::SocketSetOptError::from(err)))?;

        socket
            .set_opt::<nng::options::SendTimeout>(self.send_timeout)
            .map_err(|err| op_error!(errors::SocketSetOptError::from(err)))?;

        if let Some(opt) = self.max_ttl {
            socket
                .set_opt::<nng::options::MaxTtl>(opt)
                .map_err(|err| op_error!(errors::SocketSetOptError::from(err)))?;
        }

        if let Some(opt) = self.socket_name.as_ref() {
            socket
                .set_opt::<nng::options::SocketName>(opt.clone())
                .map_err(|err| op_error!(errors::SocketSetOptError::from(err)))?;
        }

        if let Some(opt) = self.tcp_no_delay {
            socket
                .set_opt::<nng::options::transport::tcp::NoDelay>(opt)
                .map_err(|err| op_error!(errors::SocketSetOptError::from(err)))?;
        }

        if let Some(opt) = self.tcp_keep_alive {
            socket
                .set_opt::<nng::options::transport::tcp::KeepAlive>(opt)
                .map_err(|err| op_error!(errors::SocketSetOptError::from(err)))?;
        }

        Ok(socket)
    }

    /// Enable the sending of keep-alive messages on the underlying TCP stream.
    ///
    /// This option is false by default. When enabled, if no messages are seen for a period of time,
    /// then a zero length TCP message is sent with the ACK flag set in an attempt to tickle some
    /// traffic from the peer. If none is still seen (after some platform-specific number of retries
    /// and timeouts), then the remote peer is presumed dead, and the connection is closed.
    ///
    /// his option has two purposes. First, it can be used to detect dead peers on an otherwise
    /// quiescent network. Second, it can be used to keep connection table entries in NAT and other
    /// middleware from being expiring due to lack of activity.
    pub fn tcp_keep_alive(&self) -> Option<bool> {
        self.tcp_keep_alive
    }

    /// enable / disable tcp keep alive
    pub fn set_tcp_keep_alive(self, opt: bool) -> SocketSettings {
        let mut this = self;
        this.tcp_keep_alive = Some(opt);
        this
    }

    /// Disable (or enable) the use of Nagle's algorithm for TCP connections.
    ///
    /// When true (the default), messages are sent immediately by the underlying TCP stream without
    /// waiting to gather more data. When false, Nagle's algorithm is enabled, and the TCP stream may
    /// wait briefly in attempt to coalesce messages. Nagle's algorithm is useful on low-bandwidth
    /// connections to reduce overhead, but it comes at a cost to latency.
    pub fn tcp_no_delay(&self) -> Option<bool> {
        self.tcp_no_delay
    }

    /// enable / disable tcp no delay
    pub fn set_tcp_no_delay(self, opt: bool) -> SocketSettings {
        let mut this = self;
        this.tcp_no_delay = Some(opt);
        this
    }

    /// By default this is a string corresponding to the value of the socket.
    /// The string must fit within 63-bytes but it can be changed for other application uses.
    pub fn socket_name(&self) -> Option<&str> {
        self.socket_name.as_ref().map(|s| &*s.as_str())
    }

    /// max socket name length
    pub const MAX_SOCKET_NAME_LEN: usize = 63;

    /// sets the socket name and must fit within 63-bytes. It will be truncated if longer than 63 bytes.
    pub fn set_socket_name(self, name: &str) -> SocketSettings {
        let mut this = self;
        if name.len() > SocketSettings::MAX_SOCKET_NAME_LEN {
            this.socket_name = Some(name[..63].to_string());
        } else {
            this.socket_name = Some(name.to_string());
        }
        this
    }

    /// The maximum message size that the will be accepted from a remote peer.
    /// If a peer attempts to send a message larger than this, then the message will be discarded.
    /// This option exists to prevent certain kinds of denial-of-service attacks, where a malicious
    /// agent can claim to want to send an extraordinarily large message, without sending any data.
    pub fn recv_max_size(&self) -> Option<usize> {
        self.recv_max_size.map(|n| n.get())
    }

    /// configures the maximum message size that the will be accepted from a remote peer.
    pub fn set_recv_max_size(self, size: NonZeroUsize) -> SocketSettings {
        let mut this = self;
        this.recv_max_size = Some(size);
        this
    }

    /// The depth of the socket's receive buffer as a number of messages.
    /// Messages received by the transport may be buffered until the application has accepted them for delivery.
    pub fn recv_buffer_size(&self) -> Option<u16> {
        self.recv_buffer_size.map(|n| n.get())
    }

    /// configures the depth of the socket's receive buffer as a number of messages.
    pub fn set_recv_buffer_size(self, size: NonZeroU16) -> SocketSettings {
        let mut this = self;
        this.recv_buffer_size = Some(size);
        this
    }

    /// The depth of the socket send buffer as a number of messages.
    ///
    /// Messages sent by an application may be buffered by the socket until a transport is ready to
    /// accept them for delivery. This value must be an integer between 1 and 8192, inclusive.
    pub fn send_buffer_size(&self) -> Option<u16> {
        self.send_buffer_size.map(|n| n.get())
    }

    /// maximum allowed setting for send buffer size
    pub const MAX_SEND_BUFFER_SIZE: u16 = 8192;

    /// if the size is greater than 8192, then it will be set to 8192
    pub fn set_send_buffer_size(self, size: NonZeroU16) -> SocketSettings {
        let mut this = self;
        if size.get() > SocketSettings::MAX_SEND_BUFFER_SIZE {
            this.send_buffer_size =
                Some(NonZeroU16::new(SocketSettings::MAX_SEND_BUFFER_SIZE).unwrap());
        } else {
            this.send_buffer_size = Some(size);
        }

        this
    }

    /// When no message is available for receiving at the socket for this period of time, receive operations
    /// will fail with a timeout error.
    pub fn recv_timeout(&self) -> Option<Duration> {
        self.recv_timeout
    }

    /// configures receive timeout
    pub fn set_recv_timeout(self, timeout: Duration) -> SocketSettings {
        let mut this = self;
        this.recv_timeout = Some(timeout);
        this
    }

    /// The socket send timeout.
    ///
    /// When a message cannot be queued for delivery by the socket for this period of time (such as
    /// if send buffers are full), the operation will fail with with a timeout error.
    pub fn send_timeout(&self) -> Option<Duration> {
        self.send_timeout
    }

    /// configures send timeout
    pub fn set_send_timeout(self, timeout: Duration) -> SocketSettings {
        let mut this = self;
        this.send_timeout = Some(timeout);
        this
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

    /// configures send timeout
    pub fn set_max_ttl(self, ttl: u8) -> SocketSettings {
        let mut this = self;
        this.max_ttl = Some(ttl);
        this
    }
}
