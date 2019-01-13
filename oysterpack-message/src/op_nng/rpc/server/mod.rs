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

use crate::{
    op_nng::rpc::{MessageProcessor, MessageProcessorFactory, SocketSettings},
    op_thread::ThreadConfig,
};

use log::{error, info};
use nng::{self, listener::Listener, options::Options, Socket};
use oysterpack_errors::{op_error, Error};
use serde::{Deserialize, Serialize};
use std::{
    fmt,
    marker::PhantomData,
    num::NonZeroUsize,
    sync::{Arc, Mutex},
    thread,
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
    running: Arc<Mutex<bool>>,
    join_handle: Option<thread::JoinHandle<()>>,
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
            3. the errors are reported via an nng client - pub/sub
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

        let aio_contexts = create_aio_contexts(
            &socket,
            message_processor_factory,
            listener_settings.aio_context_count,
        )?;

        #[allow(clippy::mutex_atomic)]
        let running = Arc::new(Mutex::new(false));
        let running_ref = Running(running.clone());
        // spawn the server in a background thread
        // - when the listener falls out of scope, then the listener will be closed
        // - when the aio context fall out of scope, then the context will be closed
        //   - the aio callback run in nng managed threads - if the thread panics, then the aio context
        //     will be closed. nng will log an error when the panic occurs, but there currently is
        //     is no mechanism for the app to be notified of the error.
        let join_handle = thread_config
            .map_or_else(thread::Builder::new, |config| config.builder())
            .spawn(move || {
                let _listener = listener_settings.start_listener(&socket).unwrap();
                info!("socket listener has been started");

                start_aio_contexts(&aio_contexts);
                {
                    let mut running = running_ref.0.lock().unwrap();
                    *running = true;
                }

                // block until stop signal is received
                let _ = stop_receiver.recv();
                // when the thread exits, the socket listener and aio contexts will will be closed
                info!("stopping server");
            })
            .expect("failed to spawn server thread");

        Ok(Server {
            stop_trigger: stop_sender,
            running,
            join_handle: Some(join_handle),
        })
    }

    /// Triggers the server to stop async
    pub fn stop(&self) {
        let _ = self.stop_trigger.send(());
    }

    /// Retuns a detached stop signal
    pub fn stop_signal(&self) -> StopSignal {
        StopSignal(self.stop_trigger.clone())
    }

    /// Attempts to join the server thread
    ///
    /// ## Note
    pub fn join(self) -> thread::Result<()> {
        let mut this = self;
        match this.join_handle.take() {
            Some(handle) => handle.join(),
            None => Ok(()),
        }
    }

    /// is the server running
    pub fn running(&self) -> bool {
        *self.running.lock().unwrap()
    }
}

impl fmt::Debug for Server {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("Server")
    }
}

/// Used to send the server a stop signal.
#[derive(Debug)]
pub struct StopSignal(crossbeam::channel::Sender<()>);

impl StopSignal {
    /// Signal the server to stop. It doesn't matter if the server is already stopped.
    pub fn stop(&self) {
        let _ = self.0.send(());
    }
}

#[derive(Debug)]
struct Running(Arc<Mutex<bool>>);

impl Drop for Running {
    fn drop(&mut self) {
        *self.0.lock().unwrap() = false;
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

    /// Starts a socket listener.
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
