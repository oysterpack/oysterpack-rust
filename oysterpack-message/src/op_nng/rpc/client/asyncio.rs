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

//! Asynchronous client

use super::{ClientSocketSettings, DialerSettings};
use crate::op_nng::new_aio_context;
use errors::{
    AioContextAtMaxCapacity, AioContextChannelClosed, AioCreateError, AioReceiveError,
    AioSendError, AsyncReplyChannelDisconnected,
};
use log::*;
use nng::{aio, options::Options};
use oysterpack_errors::{op_error, Error};
use std::{
    fmt,
    panic::RefUnwindSafe,
    sync::{Arc, Mutex},
    thread,
    time::Instant,
};

/// Async reply handler that is used as a callback by the AsyncClient
pub trait ReplyHandler: Send + Sync + RefUnwindSafe + 'static {
    /// reply callback
    fn on_reply(&mut self, result: Result<nng::Message, Error>);
}

/// nng async client
///
/// # Design
/// - each request creates a new aio context
///   - aio contexts are tracked in a background thread - once the request is complete, then the aio
///     context will be closed
/// - the AsyncClient limits the number of concurrent requests using a channel to implement a ticketing system
///   - the AsyncClient is initialized with a ticket channel that is full
///   - each request obtains a ticket from the channel via a `recv()`
///     - if there are no tickets, then an AioContextAtMaxCapacity error is returned
///   - when the callback is done, it will return the ticket back to the channel via a `try_send(())`
pub struct AsyncClient {
    dialer: nng::dialer::Dialer,
    socket: nng::Socket,
    aio_context_ticket_rx: crossbeam::channel::Receiver<()>,
    aio_context_ticket_tx: crossbeam::channel::Sender<()>,
    aio_context_registry_chan: crossbeam::channel::Sender<AioContextMessage>,
}

impl AsyncClient {
    /// Sends the request and invokes the callback with the reply asynchronously
    ///
    /// ## Send Errors (means the async task failed to be submitted)
    /// - [AioContextAtMaxCapacity](errors/struct.AioContextAtMaxCapacity.html)
    /// - [AioCreateError](../../../../../oysterpack_message/op_nng/errors/struct.AioCreateError.html)
    /// - [AioSendError](../../../../../oysterpack_message/op_nng/errors/struct.AioSendError.html)
    /// - [AioContextChannelClosed](errors/struct.AioContextChannelClosed.html)
    ///
    /// ## Callback Errors
    /// - [AioReceiveError](../../../../../oysterpack_message/op_nng/errors/struct.AioReceiveError.html)
    pub fn send_with_callback<Callback>(&self, req: nng::Message, cb: Callback) -> Result<(), Error>
    where
        Callback: ReplyHandler,
    {
        match self.aio_context_ticket_rx.try_recv() {
            Ok(_) => {
                let context = new_aio_context(&self.socket)?;
                let aio_state = Arc::new(Mutex::new(AioState::Idle));

                let mut cb = cb;
                let callback_aio_state = Arc::clone(&aio_state);
                let callback_context = context.clone();
                let aio_context_chan = self.aio_context_registry_chan.clone();
                let context_key = ContextId::new(&context);
                let aio_context_ticket_tx = self.aio_context_ticket_tx.clone();
                let aio = nng::aio::Aio::with_callback(move |aio| {
                    let close = || {
                        debug!("closing context({}) ... ", context_key);
                        if let Err(err) = aio_context_ticket_tx.try_send(()) {
                            error!("Failed to return ticket, which implies the client has been dropped: {}", err);
                        }
                        if let Err(err) = aio_context_chan.send(AioContextMessage::Remove(context_key)) {
                            warn!("Failed to unregister aio context - ignore this warning if the app is shutting down: {}", err);
                        }
                        debug!("closed context({})", context_key);
                    };

                    match aio.result().unwrap() {
                        Ok(_) => {
                            let mut ctx_state = callback_aio_state.lock().unwrap();
                            match *ctx_state {
                                AioState::Send => {
                                    // sending the request was successful
                                    // now lets wait for the reply
                                    aio.recv(&callback_context).unwrap();
                                    *ctx_state = AioState::Recv;
                                }
                                AioState::Recv => {
                                    // reply has been successfully received
                                    // thus it is safe to invoke unwrap n
                                    let rep = aio.get_msg().unwrap();
                                    cb.on_reply(Ok(rep));
                                    *ctx_state = AioState::Idle;
                                    close();
                                }
                                AioState::Idle => {
                                    warn!("did not expect to be invoked while idle");
                                }
                            }
                        }
                        Err(err) => {
                            cb.on_reply(Err(op_error!(AioReceiveError::from(err))));
                            close();
                        }
                    }
                })
                    .map_err(|err| op_error!(AioCreateError::from(err)))?;

                // send the message
                {
                    let mut ctx_state = aio_state.lock().unwrap();
                    *ctx_state = AioState::Send;
                }
                aio.send(&context, req)
                    .map_err(|(_msg, err)| op_error!(AioSendError::from(err)))?;
                // register the aio context
                self.aio_context_registry_chan
                    .send(AioContextMessage::Insert((
                        context_key,
                        AioContext::from((aio, context)),
                    )))
                    .map_err(|_| op_error!(AioContextChannelClosed::new(&self.dialer)))?;
                Ok(())
            }
            Err(_) => Err(op_error!(AioContextAtMaxCapacity::new(
                &self.dialer,
                self.aio_context_ticket_rx.capacity().unwrap_or(0)
            ))),
        }
    }

    /// The request is sent asynchronously
    ///
    /// ## Notes
    /// - this depends on tokio's `async-await-preview` feature
    pub async fn send(&self, req: nng::Message) -> Result<nng::Message, Error> {
        let (reply_sender, reply_receiver) =
            futures::sync::oneshot::channel::<Result<nng::Message, Error>>();
        let reply_sender = Mutex::new(Some(reply_sender));
        self.send_with_callback(req, AyncReplyHandler { reply_sender })?;
        match tokio::await!(reply_receiver)
            .map_err(|_| op_error!(AsyncReplyChannelDisconnected::new(&self.dialer)))
        {
            Ok(reply) => reply,
            Err(err) => Err(err),
        }
    }

    /// constructor
    pub fn dial(dialer_settings: DialerSettings) -> Result<Self, Error> {
        Builder::new(dialer_settings).build()
    }

    /// constructor
    pub fn dial_with_socket_settings(
        dialer_settings: DialerSettings,
        socket_settings: ClientSocketSettings,
    ) -> Result<Self, Error> {
        Builder::new(dialer_settings)
            .socket_settings(socket_settings)
            .build()
    }

    /// Returns the number of aio contexts that are currently active.
    pub fn context_count(&self) -> usize {
        let (tx, rx) = crossbeam::channel::bounded(1);
        self.aio_context_registry_chan
            .send(AioContextMessage::Count(tx))
            .map(|_| match rx.recv() {
                Ok(count) => count,
                Err(_) => 0,
            })
            .unwrap_or(0)
    }

    /// returns the max number of concurrent async requests
    /// - this never return 0 - if it does, then there is a bug
    pub fn max_capacity(&self) -> usize {
        self.aio_context_ticket_rx.capacity().unwrap_or(0)
    }

    /// returns the available capacity for submitting additional requests
    pub fn available_capacity(&self) -> usize {
        self.aio_context_ticket_rx.len()
    }

    /// returns the available capacity for submitting additional requests
    pub fn used_capacity(&self) -> usize {
        self.max_capacity() - self.available_capacity()
    }

    /// Returns the URL that the client connects to
    pub fn url(&self) -> String {
        self.dialer.get_opt::<nng::options::Url>().unwrap()
    }
}

impl fmt::Debug for AsyncClient {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "AsyncClient(Socket({}), Url({}))",
            self.socket.id(),
            self.url()
        )
    }
}

/// Aio state for socket context.
#[derive(Debug, Copy, Clone)]
pub(crate) enum AioState {
    /// aio receive operation is in progress
    Recv,
    /// aio send operation is in progress
    Send,
    /// aio context is idle
    Idle,
}

/// Client builder
#[derive(Debug)]
pub struct Builder {
    dialer_settings: DialerSettings,
    socket_settings: Option<ClientSocketSettings>,
}

impl Builder {
    /// constructor
    pub fn new(dialer_settings: DialerSettings) -> Self {
        Self {
            dialer_settings,
            socket_settings: None,
        }
    }

    /// Configures the socket
    pub fn socket_settings(self, socket_settings: ClientSocketSettings) -> Self {
        let mut builder = self;
        builder.socket_settings = Some(socket_settings);
        builder
    }

    /// builds a new AsyncClient
    ///
    /// ## Errors
    /// - [SocketCreateError](../../../../../oysterpack_message/op_nng/errors/struct.SocketCreateError.html)
    /// - [SocketSetOptError](../../../../../oysterpack_message/op_nng/errors/struct.SocketSetOptError.html)
    /// - [DialerCreateError](../errors/struct.DialerCreateError.html)
    /// - [DialerSetOptError](../errors/struct.DialerSetOptError.html)
    /// - [DialerStartError](../errors/struct.DialerStartError.html)
    pub fn build(self) -> Result<AsyncClient, Error> {
        let mut this = self;

        let max_concurrent_request_capacity = this
            .dialer_settings
            .max_concurrent_request_capacity
            .unwrap_or(1);
        let socket = ClientSocketSettings::create_socket(this.socket_settings.take())?;
        let dialer = this.dialer_settings.start_dialer(&socket)?;

        // the channel is used to store request tickets, which limit the number of concurrent requests
        // a ticket is required in order to submit a request
        // once the request is complete, the ticket is returned
        let (tx_tickets, rx_tickets) = crossbeam::channel::bounded(max_concurrent_request_capacity);
        for _ in 0..max_concurrent_request_capacity {
            tx_tickets.send(()).unwrap();
        }
        let (tx, rx) = crossbeam::channel::unbounded();

        // each AsyncClient runs a background thread to track aio contexts
        thread::Builder::new()
            .stack_size(1024)
            .spawn(move || {
                let mut aio_contexts =
                // for this use case, benchmarks show that FNV hash is ~10% faster than SipHash (Rust's default)
                fnv::FnvHashMap::<ContextId, AioContext>::with_capacity_and_hasher(
                    max_concurrent_request_capacity,
                    fnv::FnvBuildHasher::default(),
                );
                for msg in rx {
                    match msg {
                        AioContextMessage::Insert((key, aio_context)) => {
                            aio_contexts.insert(key, aio_context);
                        }
                        AioContextMessage::Remove(ref key) => {
                            aio_contexts.remove(key);
                        }
                        AioContextMessage::Count(sender) => {
                            if let Err(err) = sender.try_send(aio_contexts.len()) {
                                error!("Failed send context count on reply channel: {}", err);
                            }
                        }
                    }
                }
            })
            .expect("Failed to spawn AsyncClient thread");

        Ok(AsyncClient {
            socket,
            dialer,
            aio_context_ticket_rx: rx_tickets,
            aio_context_ticket_tx: tx_tickets,
            aio_context_registry_chan: tx,
        })
    }
}

struct AioContext {
    _aio: aio::Aio,
    context: aio::Context,
}

impl From<(aio::Aio, aio::Context)> for AioContext {
    fn from((aio, context): (aio::Aio, aio::Context)) -> Self {
        Self { _aio: aio, context }
    }
}

impl fmt::Debug for AioContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "AioContext({})", self.context.id())
    }
}

enum AioContextMessage {
    Insert((ContextId, AioContext)),
    Remove(ContextId),
    Count(crossbeam::channel::Sender<usize>),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Ord, PartialOrd)]
struct ContextId(Instant, i32);

impl ContextId {
    fn new(context: &aio::Context) -> Self {
        Self(Instant::now(), context.id())
    }
}

impl fmt::Display for ContextId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}-{}", self.0, self.1)
    }
}

struct AyncReplyHandler {
    reply_sender: Mutex<Option<futures::sync::oneshot::Sender<Result<nng::Message, Error>>>>,
}

impl ReplyHandler for AyncReplyHandler {
    fn on_reply(&mut self, result: Result<nng::Message, Error>) {
        let mut reply_sender = self.reply_sender.lock().unwrap();
        let reply_sender = reply_sender.take().unwrap();
        if reply_sender.send(result).is_err() {
            error!("unable to send reply");
        }
    }
}

pub mod errors {
    //! AsyncClient specific errors

    pub use crate::op_nng::errors::{AioCreateError, AioReceiveError, AioSendError};

    use nng::options::Options;
    use oysterpack_errors::IsError;
    use std::fmt;

    /// The channel receiver is owned by the AsyncClient's aio context registry thread.
    /// Thus, the only way this error scenario can occur is if the thread panics and exits.
    /// *This should never happen*. If it does, then there is a pretty serious bug ;(
    ///
    /// If this error does occur, then it renders the AsyncClient useless.
    /// The only way application code can handle this error is to discard the AsyncClient and create
    /// a new instance.
    #[derive(Debug, Clone, Eq, PartialEq)]
    pub struct AioContextChannelClosed {
        url: String,
    }

    impl fmt::Display for AioContextChannelClosed {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "The aio context channel is closed: {}", self.url)
        }
    }

    impl AioContextChannelClosed {
        /// Error Id
        pub const ERROR_ID: oysterpack_errors::Id =
            oysterpack_errors::Id(1870911491777855603714943020812532997);
        /// Level::Alert
        pub const ERROR_LEVEL: oysterpack_errors::Level = oysterpack_errors::Level::Alert;

        /// constructor
        pub fn new(dialer: &nng::dialer::Dialer) -> Self {
            let url = dialer.get_opt::<nng::options::Url>().unwrap();
            Self { url }
        }
    }

    impl IsError for AioContextChannelClosed {
        fn error_id(&self) -> oysterpack_errors::Id {
            Self::ERROR_ID
        }

        fn error_level(&self) -> oysterpack_errors::Level {
            Self::ERROR_LEVEL
        }
    }

    /// The number of open aio contexts is at max capacity
    #[derive(Debug, Clone, Eq, PartialEq)]
    pub struct AioContextAtMaxCapacity {
        url: String,
        capacity: usize,
    }

    impl fmt::Display for AioContextAtMaxCapacity {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(
                f,
                "The number of open aio contexts is at max capacity ({}) for client: {}",
                self.capacity, self.url
            )
        }
    }

    impl AioContextAtMaxCapacity {
        /// Error Id
        pub const ERROR_ID: oysterpack_errors::Id =
            oysterpack_errors::Id(1870938774060056887721031847045251443);
        /// Level::Alert
        pub const ERROR_LEVEL: oysterpack_errors::Level = oysterpack_errors::Level::Alert;

        /// constructor
        pub fn new(dialer: &nng::dialer::Dialer, capacity: usize) -> Self {
            let url = dialer.get_opt::<nng::options::Url>().unwrap();
            Self { url, capacity }
        }
    }

    impl IsError for AioContextAtMaxCapacity {
        fn error_id(&self) -> oysterpack_errors::Id {
            Self::ERROR_ID
        }

        fn error_level(&self) -> oysterpack_errors::Level {
            Self::ERROR_LEVEL
        }
    }

    /// The reply was received, but the reply receiver channel is disconnected
    #[derive(Debug, Clone, Eq, PartialEq)]
    pub struct AsyncReplyChannelDisconnected {
        url: String,
    }

    impl fmt::Display for AsyncReplyChannelDisconnected {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(
                f,
                "The reply was received, but the reply receiver channel is disconnected: {}",
                self.url
            )
        }
    }

    impl AsyncReplyChannelDisconnected {
        /// Error Id
        pub const ERROR_ID: oysterpack_errors::Id =
            oysterpack_errors::Id(1871311201574300224103938713131150267);
        /// Level::Alert
        pub const ERROR_LEVEL: oysterpack_errors::Level = oysterpack_errors::Level::Alert;

        /// constructor
        pub fn new(dialer: &nng::dialer::Dialer) -> Self {
            let url = dialer.get_opt::<nng::options::Url>().unwrap();
            Self { url }
        }
    }

    impl IsError for AsyncReplyChannelDisconnected {
        fn error_id(&self) -> oysterpack_errors::Id {
            Self::ERROR_ID
        }

        fn error_level(&self) -> oysterpack_errors::Level {
            Self::ERROR_LEVEL
        }
    }
}
