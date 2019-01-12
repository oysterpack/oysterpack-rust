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

use crate::op_nng::rpc::{MessageProcessor, MessageProcessorFactory, ThreadConfig};
use log::{error, info};
use nng::{self, listener::Listener, options::Options, Socket};
use oysterpack_errors::{op_error, Error};
use serde::{Deserialize, Serialize};
use std::{
    fmt,
    marker::PhantomData,
    num::{NonZeroU16, NonZeroUsize},
    thread,
    time::Duration,
};

pub mod errors;

#[allow(warnings)]
#[cfg(test)]
mod tests;

/// nng RPC server
/// - if MessageProcessor(s) panic, then the aio context that contains the MessageProcessor will terminate
///   - each aio context represents a logical request handler thread. When all aio contexts terminate,
///     then the server will no longer be able to serve requests
///   - MessageProcessor(s) should never panic - that is either a bug, resource issue, or configuration
///     issue (which may cause the resource issue)
pub struct Server {
    stop_trigger: crossbeam::channel::Sender<()>,
    stopped_signal: crossbeam::channel::Receiver<()>,
}

impl Server {
    /// returns a new Server Builder instance
    pub fn builder<Factory, Processor>(
        listener_settings: ListenerSettings,
        message_processor_factory: Factory,
    ) -> Builder<Factory, Processor>
    where
        Factory: MessageProcessorFactory<Processor, nng::Message, nng::Message>,
        Processor: MessageProcessor<nng::Message, nng::Message>,
    {
        Builder::new(listener_settings, message_processor_factory)
    }

    /// Spawns a new server instance in a background thread
    fn spawn<Factory, Processor>(
        listener_settings: ListenerSettings,
        message_processor_factory: &Factory,
        socket_settings: Option<SocketSettings>,
        thread_config: Option<ThreadConfig>,
    ) -> Result<Server, Error>
    where
        Factory: MessageProcessorFactory<Processor, nng::Message, nng::Message>,
        Processor: MessageProcessor<nng::Message, nng::Message>,
    {
        fn create_aio_contexts<Factory, Processor>(
            socket: &nng::Socket,
            message_processor_factory: &Factory,
            aio_context_count: usize,
        ) -> Result<Vec<(nng::aio::Aio, nng::aio::Context)>, Error>
        where
            Factory: MessageProcessorFactory<Processor, nng::Message, nng::Message>,
            Processor: MessageProcessor<nng::Message, nng::Message>,
        {
            // TODO: for now errors are simply being logged, but how to best handle errors ?
            /* Options
            1. the errors are handed off to an AioErrorCallback
            2. the errors are reported on a channel
            */
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

            let aio_contexts: Vec<(nng::aio::Aio, nng::aio::Context)> = (0..aio_context_count)
                .map(|_| {
                    let mut state = AioState::Recv;
                    let mut message_processor = message_processor_factory.new();

                    let ctx: nng::aio::Context = new_context(socket)?;
                    let ctx_clone = ctx.clone();
                    let aio = nng::aio::Aio::with_callback(move |aio| {
                        handle_aio_event(aio, &ctx_clone, &mut state, &mut message_processor)
                    })
                    .map_err(|err| op_error!(errors::AioCreateError::from(err)))?;

                    Ok((aio, ctx))
                })
                .collect::<Result<_, _>>()?;

            Ok(aio_contexts)
        }

        fn create_socket(socket_settings: Option<SocketSettings>) -> Result<nng::Socket, Error> {
            let socket = nng::Socket::new(nng::Protocol::Rep0)
                .map_err(|err| op_error!(errors::SocketCreateError::from(err)))?;
            match socket_settings {
                Some(socket_settings) => socket_settings.apply(socket),
                None => Ok(socket),
            }
        }

        fn start_aio_contexts(aio_contexts: &[(nng::aio::Aio, nng::aio::Context)]) {
            for (a, c) in aio_contexts {
                a.recv(c)
                    .map_err(|err| op_error!(errors::AioReceiveError::from(err)))
                    .unwrap();
            }
            info!("aio context receive operations have been initiated");
        }

        fn new_context(socket: &nng::Socket) -> Result<nng::aio::Context, Error> {
            nng::aio::Context::new(&socket)
                .map_err(|err| op_error!(errors::AioContextCreateError::from(err)))
        }

        /***************************/
        /***** function logic ******/
        /***************************/

        let socket = create_socket(socket_settings)?;

        // used to send a stop signal to the server
        let (stop_sender, stop_receiver) = crossbeam::channel::bounded(0);
        // used to send a notification when the server has stopped
        let (stopped_sender, stopped_receiver) = crossbeam::channel::bounded::<()>(1);

        let aio_contexts = create_aio_contexts(
            &socket,
            message_processor_factory,
            listener_settings.aio_context_count,
        )?;

        // spawn the server in a background thread
        // - when the listener falls out of scope, then the listener will be closed
        // - when the aio context fall out of scope, then the context will be closed
        //   - the aio callback run in nng managed threads - if the thread panics, then the aio context
        //     will be closed. nng will log an error when the panic occurs, but there currently is
        //     is no mechanism for the app to be notified of the error.
        thread_config
            .map_or_else(thread::Builder::new, |config| config.builder())
            .spawn(move || {
                let _listener = listener_settings.start_listener(&socket).unwrap();
                info!("socket listener has been started");

                start_aio_contexts(&aio_contexts);

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
}

impl fmt::Debug for Server {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("Server")
    }
}

/// Server builder
#[derive(Debug)]
pub struct Builder<Factory, Processor>
where
    Factory: MessageProcessorFactory<Processor, nng::Message, nng::Message>,
    Processor: MessageProcessor<nng::Message, nng::Message>,
{
    listener_settings: Option<ListenerSettings>,
    message_processor_factory: Option<Factory>,
    socket_settings: Option<SocketSettings>,
    thread_config: Option<ThreadConfig>,
    _processor_phantom_data: PhantomData<Processor>,
}

impl<Factory, Processor> Builder<Factory, Processor>
where
    Factory: MessageProcessorFactory<Processor, nng::Message, nng::Message>,
    Processor: MessageProcessor<nng::Message, nng::Message>,
{
    /// constructor
    pub fn new(
        listener_settings: ListenerSettings,
        message_processor_factory: Factory,
    ) -> Builder<Factory, Processor> {
        Builder {
            listener_settings: Some(listener_settings),
            message_processor_factory: Some(message_processor_factory),
            socket_settings: None,
            thread_config: None,
            _processor_phantom_data: PhantomData,
        }
    }

    /// Configures the socket
    pub fn socket_settings(self, socket_settings: SocketSettings) -> Builder<Factory, Processor> {
        let mut builder = self;
        builder.socket_settings = Some(socket_settings);
        builder
    }

    /// Configures the thread that will be used to host the server
    pub fn thread_config(self, thread_config: ThreadConfig) -> Builder<Factory, Processor> {
        let mut builder = self;
        builder.thread_config = Some(thread_config);
        builder
    }

    /// Spawns a new server instance in a background thread
    ///
    /// ## Panics
    pub fn spawn(self) -> Result<Server, Error> {
        let mut builder = self;
        Server::spawn(
            builder.listener_settings.take().unwrap(),
            &builder.message_processor_factory.take().unwrap(),
            builder.socket_settings.take(),
            builder.thread_config.take(),
        )
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
    pub fn apply(&self, socket: Socket) -> Result<Socket, Error> {
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
        let mut s = self;
        s.tcp_keep_alive = Some(opt);
        s
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
        let mut s = self;
        s.tcp_no_delay = Some(opt);
        s
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
        let mut s = self;
        if name.len() > SocketSettings::MAX_SOCKET_NAME_LEN {
            s.socket_name = Some(name[..63].to_string());
        } else {
            s.socket_name = Some(name.to_string());
        }
        s
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
        let mut s = self;
        s.recv_max_size = Some(size);
        s
    }

    /// The depth of the socket's receive buffer as a number of messages.
    /// Messages received by the transport may be buffered until the application has accepted them for delivery.
    pub fn recv_buffer_size(&self) -> Option<u16> {
        self.recv_buffer_size.map(|n| n.get())
    }

    /// configures the depth of the socket's receive buffer as a number of messages.
    pub fn set_recv_buffer_size(self, size: NonZeroU16) -> SocketSettings {
        let mut s = self;
        s.recv_buffer_size = Some(size);
        s
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
        let mut settings = self;
        if size.get() > SocketSettings::MAX_SEND_BUFFER_SIZE {
            settings.send_buffer_size =
                Some(NonZeroU16::new(SocketSettings::MAX_SEND_BUFFER_SIZE).unwrap());
        } else {
            settings.send_buffer_size = Some(size);
        }

        settings
    }

    /// When no message is available for receiving at the socket for this period of time, receive operations
    /// will fail with a timeout error.
    pub fn recv_timeout(&self) -> Option<Duration> {
        self.recv_timeout
    }

    /// configures receive timeout
    pub fn set_recv_timeout(self, timeout: Duration) -> SocketSettings {
        let mut s = self;
        s.recv_timeout = Some(timeout);
        s
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
        let mut s = self;
        s.send_timeout = Some(timeout);
        s
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
        let mut s = self;
        s.max_ttl = Some(ttl);
        s
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
            .map_err(|err| op_error!(errors::ListenerCreateError::from(err)))?;

        if let Some(option) = self.recv_max_size.as_ref() {
            options
                .set_opt::<nng::options::RecvMaxSize>(*option)
                .map_err(|err| op_error!(errors::ListenerSetOptError::from(err)))?;
        }

        if let Some(option) = self.no_delay.as_ref() {
            options
                .set_opt::<nng::options::transport::tcp::NoDelay>(*option)
                .map_err(|err| op_error!(errors::ListenerSetOptError::from(err)))?;
        }

        if let Some(option) = self.keep_alive.as_ref() {
            options
                .set_opt::<nng::options::transport::tcp::KeepAlive>(*option)
                .map_err(|err| op_error!(errors::ListenerSetOptError::from(err)))?;
        }

        options
            .start(self.non_blocking)
            .map_err(|(_options, err)| op_error!(errors::ListenerStartError::from(err)))
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
