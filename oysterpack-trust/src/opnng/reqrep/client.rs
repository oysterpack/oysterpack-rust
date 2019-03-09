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

//! Provides an ReqRep [Client](type.Client.html) application interface for nng clients.
//! - [register_client](fn.register_client.html) is used to register clients in a global registry
//! - [client](fn.client.html) is used to lookup Clients by ReqRepId
//!
//! - The client is fully async and supports parallelism. The level of parallelism is configured via
//!   [DialerConfig::parallelism()](struct.DialerConfig.html#method.parallelism).
//! - When all [Client(s)](type.Client.html) are unregistered and all references fall out of scope, then
//!   the backend ReqRep service will stop which will:
//!   - unregister its context
//!   - close the nng::Dialer and nng:Socket resources
//!   - close the Aio event loop channels, which will trigger the Aio event loop tasks to exit
//!
//! ## Design
//! The server is designed internally to be async and non-blocking leveraging nng's async capabilities.
//! coupled with [futures](https://crates.io/crates/futures-preview). The approach is to integrate using
//! [nng:Aio](https://docs.rs/nng/latest/nng/struct.Aio.html) and [nng::Context](https://docs.rs/nng/latest/nng/struct.Context.html).
//! via nng's [callback](https://docs.rs/nng/latest/nng/struct.Aio.html#method.with_callback) mechanism.
//! Parallelism is controlled by the number of [callbacks](https://docs.rs/nng/latest/nng/struct.Aio.html#method.with_callback)
//! that are registered. Each callback is linked to a future's based task via an async channel.
//! The callback's job is to simply forward async IO events to the backend end futures task to process.
//! The Aio callback tasks form a pool of Aio contexts which can handle multiple requests in parallel.
//! When a Client submits a request, the request / reply workflow is serviced by one of the Aio callback
//! tasks. If all Aio context tasks are busy, then requests will wait asynchronously in a non-blocking
//! manner for an Aio context task.

use crate::concurrent::{
    execution::Executor,
    messaging::reqrep::{self, ReqRep, ReqRepId},
};
use crate::opnng::{self, config::SocketConfigError};
use failure::Fail;
use futures::{
    channel::{mpsc, oneshot},
    future::FutureExt,
    sink::SinkExt,
    stream::StreamExt,
    task::SpawnExt,
};
use lazy_static::lazy_static;
use nng::options::Options;
use oysterpack_log::*;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::{fmt, num::NonZeroUsize, panic::AssertUnwindSafe, sync::Arc, time::Duration};
use hashbrown::HashMap;

lazy_static! {
     /// Global Client contexts
    static ref CLIENT_CONTEXTS: RwLock<HashMap<ReqRepId, Arc<NngClientContext>>> = RwLock::new(HashMap::new());

    /// Global ReqRep nng client registry
    static ref CLIENTS: RwLock<HashMap<ReqRepId, Client>> = RwLock::new(HashMap::new());
}

/// Client type alias
pub type Client = ReqRep<nng::Message, Result<nng::Message, RequestError>>;

/// The client's ReqRepId is used as the registry key. Thus, if a Client is already registered with
/// the same ReqRepId, then a [ClientRegistrationError::ClientAlreadyRegistered] error is returned.
pub fn register_client(
    reqrep_service_config: reqrep::ReqRepConfig,
    socket_config: Option<SocketConfig>,
    dialer_config: DialerConfig,
    executor: Executor,
) -> Result<Client, ClientRegistrationError> {
    let mut clients = CLIENTS.write();
    if clients.contains_key(&reqrep_service_config.reqrep_id()) {
        return Err(ClientRegistrationError::ClientAlreadyRegistered(
            reqrep_service_config.reqrep_id(),
        ));
    }
    let nng_client = NngClient::new(
        reqrep_service_config.reqrep_id(),
        socket_config,
        dialer_config,
        executor.clone(),
    )
    .map_err(ClientRegistrationError::NngError)?;
    let reqrep = reqrep_service_config
        .start_service(nng_client, executor)
        .map_err(|err| {
            ClientRegistrationError::NngError(NngClientError::ReqRepServiceStartFailed(
                err.is_shutdown(),
            ))
        })?;
    let _ = clients.insert(reqrep.id(), reqrep.clone());
    Ok(reqrep)
}

/// Unregisters the client from the global registry
pub fn unregister_client(reqrep_id: ReqRepId) -> Option<Client> {
    let mut clients = CLIENTS.write();
    clients.remove(&reqrep_id)
}

/// Returns the client if it is registered
pub fn client(reqrep_id: ReqRepId) -> Option<Client> {
    CLIENTS.read().get(&reqrep_id).cloned()
}

/// Returns set of registered ReqRepId(s)
pub fn registered_client_ids() -> Vec<ReqRepId> {
    CLIENTS.read().keys().cloned().collect()
}

/// The context that is required by the NngClient's backend service.
#[derive(Clone)]
struct NngClientContext {
    id: ReqRepId,
    socket: Option<nng::Socket>,
    dialer: Option<nng::Dialer>,
    aio_context_pool_return: mpsc::Sender<mpsc::Sender<Request>>,
}

/// nng client
#[derive(Clone)]
struct NngClient {
    id: ReqRepId,
    borrow: mpsc::Sender<oneshot::Sender<mpsc::Sender<Request>>>,
    request_sender_pool_task_stop_tx: mpsc::Sender<()>,
}

impl NngClient {
    /// constructor
    ///
    /// ## Notes
    /// The Executor is used to spawn tasks for handling the nng request / reply processing.
    /// The parallelism defined by the DialerConfig corresponds to the number of Aio callbacks that
    /// will be registered, which corresponds to the number of Aio Context handler tasks spawned.
    fn new(
        id: ReqRepId,
        socket_config: Option<SocketConfig>,
        dialer_config: DialerConfig,
        mut executor: Executor,
    ) -> Result<Self, NngClientError> {
        let mut nng_client_executor = executor.clone();
        let parallelism = dialer_config.parallelism();
        let (aio_context_pool_return, aio_context_pool_borrow) =
            mpsc::channel::<mpsc::Sender<Request>>(parallelism);

        let create_context = move || {
            let socket = SocketConfig::create_socket(socket_config)
                .map_err(NngClientError::SocketCreateFailure)?;
            let dialer = dialer_config
                .start_dialer(&socket)
                .map_err(NngClientError::DialerStartError)?;

            Ok(NngClientContext {
                id,
                socket: Some(socket),
                dialer: Some(dialer),
                aio_context_pool_return,
            })
        };

        let mut start_workers = move |ctx: &NngClientContext| {
            for i in 0..parallelism {
                // used to notify the workers when an Aio event has occurred, i.e., the Aio callback has been invoked
                let (aio_tx, mut aio_rx) = futures::channel::mpsc::unbounded::<()>();
                let aio_tx = AssertUnwindSafe(aio_tx);
                let context = nng::Context::new(ctx.socket.as_ref().unwrap())
                    .map_err(NngClientError::NngContextCreateFailed)?;
                let callback_ctx = context.clone();
                let aio = nng::Aio::with_callback(move |_aio| {
                    if let Err(err) = aio_tx.unbounded_send(()) {
                        // means the channel has been disconnected because the worker Future task has completed
                        // the server is either being stopped, or the worker has crashed
                        // TODO: we need a way to know if the server is being shutdown
                        warn!("Failed to nofify worker of Aio event. This means the worker is not running. The Aio Context will be closed: {}", err);
                        // TODO: will cloning the Context work ? Context::close() cannot be invoked from the callback because it consumes the Context
                        //       and rust won't allow it because the Context is being referenced by the FnMut closure
                        callback_ctx.clone().close();
                        // TODO: send an alert - if the worker crashed, i.e., panicked, then it may need to be restarted
                    }
                }).map_err(NngClientError::NngAioCreateFailed)?;

                let (req_tx, mut req_rx) = futures::channel::mpsc::channel::<Request>(1);
                let mut aio_context_pool_return = ctx.aio_context_pool_return.clone();
                {
                    let req_tx = req_tx.clone();
                    let mut aio_context_pool_return = aio_context_pool_return.clone();
                    executor
                        .run(async move { await!(aio_context_pool_return.send(req_tx)) })
                        .map_err(|_err| NngClientError::AioContextPoolChannelClosed)?;
                }
                executor.spawn(async move {
                    debug!("[{}-{}] NngClient Aio Context task is running", id, i);
                    while let Some(mut req) = await!(req_rx.next()) {
                        debug!("[{}-{}] NngClient: processing request", id, i);
                        if let Some(msg) = req.msg.take() {
                            // send the request
                            match context.send(&aio, msg) {
                                Ok(_) => {
                                    if await!(aio_rx.next()).is_none() {
                                        debug!("[{}-{}] NngClient Aio callback channel is closed", id, i);
                                        break
                                    }
                                    match aio.result().unwrap() {
                                        Ok(_) => {
                                            // TODO: set a timeout - see Aio::set_timeout()
                                            // receive the reply
                                            match context.recv(&aio) {
                                                Ok(_) => {
                                                    if await!(aio_rx.next()).is_none() {
                                                        debug!("[{}-{}] NngClient Aio callback channel is closed", id, i);
                                                        break
                                                    }
                                                    match aio.result().unwrap() {
                                                        Ok(_) => {
                                                            match aio.get_msg() {
                                                                Some(reply) => {
                                                                    let _ = req.reply_chan.send(Ok(reply));
                                                                },
                                                                None => {
                                                                    let _ = req.reply_chan.send(Err(RequestError::NoReplyMessage));
                                                                }
                                                            }
                                                        }
                                                        Err(err) => {
                                                            let _ = req.reply_chan.send(Err(RequestError::RecvFailed(err)));
                                                            aio.cancel();
                                                        }
                                                    }
                                                },
                                                Err(err) => {
                                                    let _ = req.reply_chan.send(Err(RequestError::RecvFailed(err)));
                                                    aio.cancel();
                                                }
                                            }
                                        },
                                        Err(err) => {
                                            let _ = req.reply_chan.send(Err(RequestError::SendFailed(err)));
                                            aio.cancel();
                                        }
                                    }
                                },
                                Err((_msg, err)) =>  {
                                    let _ = req.reply_chan.send(Err(RequestError::SendFailed(err)));
                                    aio.cancel();
                                }
                            }
                        } else {
                            let _ = req.reply_chan.send(Err(RequestError::InvalidRequest("BUG: Request was received with no nng::Message".to_string())));
                        }
                        // add a request Sender back to the pool, indicating the worker is now available
                        if let Err(err) = await!(aio_context_pool_return.send(req_tx.clone())) {
                            error!("[{}-{}] Failed to return request sender back to the pool: {}",id, i, err)
                        }
                        debug!("[{}-{}] NngClient: request is done", id, i);
                    }
                    debug!("[{}-{}] NngClient Aio Context task is done", id, i);
                }).map_err(|err| NngClientError::AioContextTaskSpawnError(err.is_shutdown()))?;
            }

            Ok(())
        };

        // spawn a Sender<Request> resource pool
        // - the Sender<Request> resource pool is managed via a mpsc::channel: the Receiver<Sender<Request>>
        //   is owned by this task
        // - the ReqRep backend service is the consumer, and the Aio Context worker tasks are the producers
        let (borrow_tx, mut borrow_rx) = mpsc::channel::<oneshot::Sender<mpsc::Sender<Request>>>(1);
        // used to notify the task that the NngClient is being closed
        let (request_sender_pool_task_stop_tx, request_sender_pool_task_stop_rx) =
            mpsc::channel::<()>(0);
        let start_request_sender_pool_task = move || {
            nng_client_executor.spawn(async move {
                debug!("NngClient Aio Context Pool task is running: {}", id);

                // fuse the streams that will be polled via futures::select! - per the documentation
                let mut request_sender_pool_task_stop_rx = request_sender_pool_task_stop_rx.fuse();
                let mut aio_context_pool_borrow = aio_context_pool_borrow.fuse();

                while let Some(reply_chan) = await!(borrow_rx.next()) {
                    futures::select! {
                        request_sender = aio_context_pool_borrow.next() => match request_sender {
                            Some(request_sender) => {
                                let _ = reply_chan.send(request_sender);
                            },
                            None => {
                                debug!("`aio_context_pool_borrow` channel is disconnected - thus we are done");
                                break
                            }
                        },
                        _ = request_sender_pool_task_stop_rx.next() => break,
                    }
                }

                // drain the aio_context_pool_borrow channel and close the Aio Context handler channels
                // - this is required in order for the AIO Context handler tasks to exit
                while let Some(mut sender) = await!(aio_context_pool_borrow.next()) {
                    sender.close_channel();
                    debug!("closed Aio Context channel");
                }

                debug!("NngClient Aio Context Pool task is done: {}", id);
            }).map_err(|err| NngClientError::AioContextTaskSpawnError(err.is_shutdown()))
        };

        let ctx = create_context()?;
        start_workers(&ctx)?;
        start_request_sender_pool_task()?;

        {
            let mut clients = CLIENT_CONTEXTS.write();
            clients.insert(ctx.id, Arc::new(ctx));
        }

        Ok(Self {
            id,
            borrow: borrow_tx,
            request_sender_pool_task_stop_tx,
        })
    }
}

impl fmt::Debug for NngClient {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "NngClient({})", self.id)
    }
}

impl reqrep::Processor<nng::Message, Result<nng::Message, RequestError>> for NngClient {
    fn process(
        &mut self,
        req: nng::Message,
    ) -> reqrep::FutureReply<Result<nng::Message, RequestError>> {
        let mut borrow = self.borrow.clone();

        async move {
            let (borrow_tx, borrow_rx) = oneshot::channel();
            if await!(borrow.send(borrow_tx)).is_err() {
                return Err(RequestError::NngAioContextPoolChannelDisconnected);
            }

            let (tx, rx) = oneshot::channel();
            let request = Request {
                msg: Some(req),
                reply_chan: tx,
            };

            match await!(borrow_rx) {
                Ok(ref mut sender) => match await!(sender.send(request)) {
                    Ok(_) => match await!(rx) {
                        Ok(result) => result,
                        Err(_) => Err(RequestError::ReplyChannelClosed),
                    },
                    Err(err) => Err(RequestError::AioContextChannelDisconnected(err)),
                },
                Err(_) => Err(RequestError::NngAioContextPoolChannelDisconnected),
            }
        }
            .boxed()
    }

    fn destroy(&mut self) {
        debug!("NngClient({}) is being destroyed ...", self.id);
        let mut client_contexts = CLIENT_CONTEXTS.write();
        if let Some(mut context) = client_contexts.remove(&self.id) {
            let context = Arc::get_mut(&mut context).unwrap();
            context.dialer.take().unwrap().close();
            debug!("NngClient({}): closed nng::Dialer", self.id);
            context.socket.take().unwrap().close();
            debug!("NngClient({}): closed nng::Socket ", self.id);
            // shutdown the Sender<Request> pool task
            self.borrow.close_channel();
            self.request_sender_pool_task_stop_tx.close_channel();
            debug!("NngClient({}): closed channels", self.id);
        }
        debug!("NngClient({}) is destroyed", self.id);
    }
}

/// Client registration errors
#[derive(Debug, Fail)]
pub enum ClientRegistrationError {
    /// Client is already registered
    #[fail(display = "Client is already registered: {}", _0)]
    ClientAlreadyRegistered(ReqRepId),
    /// nng error
    #[fail(display = "nng error: {}", _0)]
    NngError(#[cause] NngClientError),
}

/// NngClient related errors
#[derive(Debug, Fail)]
pub enum NngClientError {
    /// Failed to create Socket
    #[fail(display = "Failed to create Socket: {}", _0)]
    SocketCreateFailure(#[cause] SocketConfigError),
    /// Failed to start Dialer
    #[fail(display = "Failed to start Dialer: {}", _0)]
    DialerStartError(#[cause] DialerConfigError),
    /// Failed to create nng::Context
    #[fail(display = "Failed to create nng::Context: {}", _0)]
    NngContextCreateFailed(#[cause] nng::Error),
    /// Failed to create nng::Aio
    #[fail(display = "Failed to create nng::Aio: {}", _0)]
    NngAioCreateFailed(#[cause] nng::Error),
    /// The Aio Context pool channel is closed
    #[fail(display = "The Aio Context pool channel is closed")]
    AioContextPoolChannelClosed,
    /// Failed to spawn Aio Context request handler task
    #[fail(
        display = "Failed to spawn Aio Context request handler task: executor is shutdown = {}",
        _0
    )]
    AioContextTaskSpawnError(bool),
    /// Failed to spawn Aio Context request handler task
    #[fail(
        display = "Failed to spawn ReqRep service: executor is shutdown = {}",
        _0
    )]
    ReqRepServiceStartFailed(bool),
}

/// Request related errors
#[derive(Debug, Fail, Clone)]
pub enum RequestError {
    /// The nng Aio Context pool channel is disconnected
    #[fail(display = "The nng Aio Context pool channel is disconnected.")]
    NngAioContextPoolChannelDisconnected,
    /// The nng Aio Context channel is disconnected
    #[fail(display = "The nng Aio Context channel is disconnected: {}", _0)]
    AioContextChannelDisconnected(#[cause] futures::channel::mpsc::SendError),
    /// Reply channel closed
    #[fail(display = "Reply channel closed")]
    ReplyChannelClosed,
    /// Failed to send the request
    #[fail(display = "Failed to send request: {}", _0)]
    SendFailed(#[cause] nng::Error),
    /// Failed to receive the reply
    #[fail(display = "Failed to receive reply: {}", _0)]
    RecvFailed(#[cause] nng::Error),
    /// Empty message
    #[fail(display = "Invalid request: {}", _0)]
    InvalidRequest(String),
    /// No reply message
    #[fail(display = "BUG: No reply message was found - this should never happen")]
    NoReplyMessage,
}

struct Request {
    msg: Option<nng::Message>,
    reply_chan: oneshot::Sender<Result<nng::Message, RequestError>>,
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
    #[serde(with = "url_serde")]
    url: url::Url,
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
    pub fn new(url: url::Url) -> DialerConfig {
        DialerConfig {
            url,
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
    pub fn url(&self) -> &url::Url {
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
    DialerOptionsCreateFailed(#[cause] nng::Error),
    /// Failed to set the RecvMaxSize option
    #[fail(display = "Failed to set the RecvMaxSize option: {}", _0)]
    RecvMaxSize(#[cause] nng::Error),
    /// Failed to set the TcpNoDelay option
    #[fail(display = "Failed to set the TcpNoDelay option: {}", _0)]
    TcpNoDelay(#[cause] nng::Error),
    /// Failed to set the TcpKeepAlive option
    #[fail(display = "Failed to set the TcpKeepAlive option: {}", _0)]
    TcpKeepAlive(#[cause] nng::Error),
    /// Failed to set the ReconnectMinTime option
    #[fail(display = "Failed to set the ReconnectMinTime option: {}", _0)]
    ReconnectMinTime(#[cause] nng::Error),
    /// Failed to set the ReconnectMaxTime option
    #[fail(display = "Failed to set the ReconnectMaxTime option: {}", _0)]
    ReconnectMaxTime(#[cause] nng::Error),
    /// Failed to start Dialer
    #[fail(display = "Failed to start Dialer: {}", _0)]
    DialerStartError(#[cause] nng::Error),
}

#[allow(warnings)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::opnng::config::{SocketConfig, SocketConfigError};
    use crate::{
        concurrent::{
            execution::{self, *},
            messaging::reqrep::{self, *},
        },
        configure_logging, metrics,
        opnng::reqrep::server,
    };
    use futures::executor::ThreadPoolBuilder;
    use oysterpack_uid::ULID;
    use oysterpack_uid::*;
    use std::{thread, time::Duration};

    struct EchoService;
    impl Processor<nng::Message, nng::Message> for EchoService {
        fn process(&mut self, req: nng::Message) -> reqrep::FutureReply<nng::Message> {
            async move { req }.boxed()
        }
    }

    fn start_server() -> ReqRep<nng::Message, nng::Message> {
        start_server_with_reqrep_id(ReqRepId::generate())
    }

    fn start_server_with_reqrep_id(reqrep_id: ReqRepId) -> ReqRep<nng::Message, nng::Message> {
        let timer_buckets = metrics::TimerBuckets::from(
            vec![
                Duration::from_nanos(50),
                Duration::from_nanos(100),
                Duration::from_nanos(150),
                Duration::from_nanos(200),
            ]
            .as_slice(),
        );
        ReqRepConfig::new(reqrep_id, timer_buckets)
            .start_service(EchoService, global_executor().clone())
            .unwrap()
    }

    fn start_client(reqrep_id: ReqRepId, url: url::Url) -> (Client, ExecutorId) {
        start_client_with_dialer_config(reqrep_id, DialerConfig::new(url))
    }

    fn start_client_with_dialer_config(
        reqrep_id: ReqRepId,
        dialer_config: DialerConfig,
    ) -> (Client, ExecutorId) {
        let timer_buckets = metrics::TimerBuckets::from(
            vec![
                Duration::from_nanos(50),
                Duration::from_nanos(100),
                Duration::from_nanos(150),
                Duration::from_nanos(200),
            ]
            .as_slice(),
        );

        let client_executor_id = ExecutorId::generate();
        let client = super::register_client(
            ReqRepConfig::new(reqrep_id, timer_buckets),
            None,
            dialer_config,
            execution::ExecutorBuilder::new(client_executor_id)
                .register()
                .unwrap(),
        )
        .unwrap();
        (client, client_executor_id)
    }

    #[test]
    fn nng_client_single_client() {
        configure_logging();
        let mut executor = execution::global_executor();

        // GIVEN: the server is running
        let url = url::Url::parse(&format!("inproc://{}", ULID::generate())).unwrap();
        let server_executor_id = ExecutorId::generate();
        let server_reqrep = start_server();
        let reqrep_id = server_reqrep.id();
        let mut server_handle = server::spawn(
            None,
            server::ListenerConfig::new(url.clone()),
            server_reqrep,
            execution::ExecutorBuilder::new(server_executor_id)
                .register()
                .unwrap(),
        )
        .unwrap();
        assert!(server_handle.ping());

        // GIVEN: the NngClient is registered
        let (mut client, client_executor_id) = start_client(reqrep_id, url.clone());
        // THEN: the client ReqRepId should match
        assert_eq!(client.id(), reqrep_id);
        // WHEN: the client is dropped
        drop(client);
        const REQUEST_COUNT: usize = 100;
        let replies: Vec<nng::Message> = executor.run(
            async move {
                // Then: the client can still be retrieved from the global registry
                let mut client = super::client(reqrep_id).unwrap();
                // AND: the client is still functional
                let mut replies = Vec::with_capacity(REQUEST_COUNT);
                for _ in 0..REQUEST_COUNT {
                    let reply_receiver: ReplyReceiver<Result<nng::Message, RequestError>> =
                        await!(client.send(nng::Message::new().unwrap())).unwrap();
                    replies.push(await!(reply_receiver.recv()).unwrap().unwrap());
                }
                replies
            },
        );
        // THEN: all requests were successfully processed
        assert_eq!(replies.len(), REQUEST_COUNT);

        // WHEN: the client is unregistered
        let client = super::unregister_client(reqrep_id).unwrap();
        assert!(super::unregister_client(reqrep_id).is_none());
        assert!(super::client(reqrep_id).is_none());

        // WHEN: the last client reference is dropped
        drop(client);
        thread::yield_now();
        let executor = execution::executor(client_executor_id).unwrap();
        for _ in 0..10 {
            if executor.task_active_count() == 0 {
                info!("all client tasks have completed");
                break;
            }
            info!("waiting for NngClient Aio Context handler tasks to exit: executor.active_task_count() = {}", executor.task_active_count());
            thread::sleep_ms(5);
            thread::yield_now();
        }
        // TODO: this sometimes fails, which means there is a bug
        // It has failed where the active task count = 1
        assert_eq!(executor.task_active_count(), 0);
    }

    #[test]
    fn nng_client_multithreaded_usage() {
        configure_logging();
        let mut executor = execution::ExecutorBuilder::new(ExecutorId::generate())
            .register()
            .unwrap();

        // GIVEN: the server is running
        let url = url::Url::parse(&format!("inproc://{}", ULID::generate())).unwrap();
        let server_executor_id = ExecutorId::generate();
        let server_reqrep = start_server();
        let reqrep_id = server_reqrep.id();
        let mut server_handle = server::spawn(
            None,
            server::ListenerConfig::new(url.clone()),
            server_reqrep,
            execution::ExecutorBuilder::new(server_executor_id)
                .register()
                .unwrap(),
        )
        .unwrap();
        assert!(server_handle.ping());

        // GIVEN: the NngClient is registered
        let (mut client, client_executor_id) = start_client(reqrep_id, url.clone());

        const TASK_COUNT: usize = 10;
        const REQUEST_COUNT: usize = 100;
        let mut handles = Vec::new();
        for _ in 0..TASK_COUNT {
            let handle = executor
                .spawn_with_handle(
                    async move {
                        // Then: the client can still be retrieved from the global registry
                        let mut client = super::client(reqrep_id).unwrap();
                        // AND: the client is still functional
                        let mut replies = Vec::with_capacity(REQUEST_COUNT);
                        for _ in 0..REQUEST_COUNT {
                            let reply_receiver: ReplyReceiver<Result<nng::Message, RequestError>> =
                                await!(client.send(nng::Message::new().unwrap())).unwrap();
                            replies.push(await!(reply_receiver.recv()).unwrap().unwrap());
                        }
                        replies
                    },
                )
                .unwrap();
            handles.push(handle);
        }

        executor.run(
            async move {
                for handle in handles {
                    let replies: Vec<nng::Message> = await!(handle);
                    assert_eq!(replies.len(), REQUEST_COUNT);
                }
            },
        );
    }

    #[test]
    fn check_client_internal_task_count() {
        configure_logging();
        let mut executor = execution::ExecutorBuilder::new(ExecutorId::generate())
            .register()
            .unwrap();

        let reqrep_id = ReqRepId::generate();
        let url = url::Url::parse(&format!("inproc://{}", reqrep_id)).unwrap();

        // WHEN: the client is started
        let (mut client, client_executor_id) = start_client(reqrep_id, url.clone());
        // THEN: we expect the Client to have N number of tasks running = 1 Aio worker per logical cpu + 1 ReqRep backend service task + 1 request sender pool task
        let expected_task_count = num_cpus::get() as u64 + 2;
        let executor = execution::executor(client_executor_id).unwrap();
        info!("active task count = {}", executor.task_active_count());
        for _ in 0..10 {
            if executor.task_active_count() == expected_task_count {
                break;
            }
            thread::sleep_ms(1);
        }
        assert_eq!(executor.task_active_count(), expected_task_count);
    }

    #[test]
    fn dialer_config_reconnect_time_min_max() {
        configure_logging();
        let mut executor = execution::ExecutorBuilder::new(ExecutorId::generate())
            .register()
            .unwrap();

        let reqrep_id = ReqRepId::generate();
        let url = url::Url::parse(&format!("inproc://{}", reqrep_id)).unwrap();

        // GIVEN: a DialerConfig that is configured with a reconnect min and max time of 50 ms
        let dialer_config = DialerConfig::new(url.clone())
            .set_reconnect_min_time(Duration::from_millis(50))
            .set_reconnect_max_time(Duration::from_millis(100));

        // WHEN: the client is started before the server
        let (mut client, client_executor_id) =
            start_client_with_dialer_config(reqrep_id, dialer_config);
        // AND: the client sends a request before the server is running
        let client_request_handle = {
            let mut client = client.clone();
            executor.spawn_with_handle(
                async move { await!(client.send_recv(nng::Message::new().unwrap())) },
            )
        }
        .unwrap();

        // WHEN: the server starts
        let server_reqrep = start_server_with_reqrep_id(reqrep_id);
        let mut server_handle = server::spawn(
            None,
            server::ListenerConfig::new(url.clone()),
            server_reqrep,
            global_executor(),
        )
        .unwrap();
        assert!(server_handle.ping());

        // THEN: the request is processed successfully
        let reply = executor.run(async move { await!(client_request_handle) });
        info!("reply = {:?}", reply.unwrap().unwrap());
    }

    #[test]
    fn dialer_config_reconnect_time_min() {
        configure_logging();

        let mut executor = execution::ExecutorBuilder::new(ExecutorId::generate())
            .register()
            .unwrap();

        let reqrep_id = ReqRepId::generate();
        let url = url::Url::parse(&format!("inproc://{}", reqrep_id)).unwrap();

        // GIVEN: a DialerConfig that is configured with a reconnect min and max time of 50 ms
        let dialer_config =
            DialerConfig::new(url.clone()).set_reconnect_min_time(Duration::from_millis(50));

        // WHEN: the client is started before the server
        let (mut client, client_executor_id) =
            start_client_with_dialer_config(reqrep_id, dialer_config);
        // AND: the client sends a request before the server is running
        let client_request_handle = {
            let mut client = client.clone();
            executor.spawn_with_handle(
                async move { await!(client.send_recv(nng::Message::new().unwrap())) },
            )
        }
        .unwrap();

        // WHEN: the server starts
        let server_reqrep = start_server_with_reqrep_id(reqrep_id);
        let mut server_handle = server::spawn(
            None,
            server::ListenerConfig::new(url.clone()),
            server_reqrep,
            global_executor(),
        )
        .unwrap();
        assert!(server_handle.ping());

        // THEN: the request is processed successfully
        let reply = executor
            .run(async move { await!(client_request_handle) })
            .unwrap();
        info!("reply = {:?}", reply.unwrap());
    }
}
