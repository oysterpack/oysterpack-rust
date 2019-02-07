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

//! This module provides support for building scalable, high performing messaging based server built
//! on proven [nng](https://nanomsg.github.io/nng/)  technology.

use crate::concurrent::execution::Executor;
use crate::concurrent::messaging::reqrep::{ReqRep, ReqRepId};
use crate::metrics;
use crate::opnng::config::{SocketConfig, SocketConfigError};
use failure::Fail;
use futures::{future::FutureExt, prelude::*, sink::SinkExt, stream::StreamExt, task::SpawnExt};
use lazy_static::lazy_static;
use nng::options::Options;
use oysterpack_log::*;
use oysterpack_uid::ULID;
use serde::{Deserialize, Serialize};
use std::{
    fmt,
    num::NonZeroUsize,
    sync::Mutex,
    sync::{Arc, RwLock},
};

lazy_static! {

    /// Global Executor registry
    static ref SERVER_HANDLES: RwLock<fnv::FnvHashMap<ULID, Arc<ServerHandle>>> = RwLock::new(fnv::FnvHashMap::default());

    /// the metric is incremented on nng::PipeEvent::AddPost and decremented on nng::PipeEvent::RemovePost
    static ref ACTIVE_CONN_COUNT: prometheus::IntGaugeVec = metrics::registry().register_int_gauge_vec(
        ACTIVE_CONN_COUNT_METRIC_ID,
        "Active number of socket connections".to_string(),
        &[REQREP_LABEL_ID],
        None
    ).unwrap();

    /// the metric is incremented on nng::PipeEvent::AddPost
    static ref TOT_CONN_COUNT: prometheus::IntCounterVec = metrics::registry().register_int_counter_vec(
        TOT_CONN_COUNT_METRIC_ID,
        "Total number of socket connections since the server was started".to_string(),
        &[REQREP_LABEL_ID],
        None
    ).unwrap();

    /// the metric is incremented on nng::PipeEvent::PrePost
    /// - this should normally match the number of nng::PipeEvent::PrePost events. When it's greater
    ///   then it means connections are being closed before being added to the socket.
    static ref TOT_CONN_INITIATE_COUNT: prometheus::IntCounterVec = metrics::registry().register_int_counter_vec(
        TOT_CONN_INITIATE_COUNT_METRIC_ID,
        "Total number of connections that have been initiated, but before being added to the socket, since the server was started.".to_string(),
        &[REQREP_LABEL_ID],
        None
    ).unwrap();

}

/// IntGaugeVec MetricId which is used to track the total number of active socket connections by ReqRepId
pub const ACTIVE_CONN_COUNT_METRIC_ID: metrics::MetricId =
    metrics::MetricId(1873168046490600819041194830632263157);
/// IntCounterVec MetricId which is used to track the total number of socket connections by ReqRepId
pub const TOT_CONN_COUNT_METRIC_ID: metrics::MetricId =
    metrics::MetricId(1873172323751845180087130844627387786);
/// IntCounterVec MetricId which is used to track the total number of connection that have been initiated by ReqRepId
pub const TOT_CONN_INITIATE_COUNT_METRIC_ID: metrics::MetricId =
    metrics::MetricId(1873172273925609759145190455058277250);

/// Metric LabelId which is used to store a ReqRepId
/// - this is used by the following metrics:
///   - IntGaugeVec(ACTIVE_CONN_COUNT_METRIC_ID)
///   - IntCounterVec(TOT_CONN_COUNT_METRIC_ID)
pub const REQREP_LABEL_ID: metrics::LabelId =
    metrics::LabelId(1873168278096570673538811977244540631);

/// Spawns a server background task
/// - the server runs as a Future
/// - returns a ServerHandle that can be used to stop the server
///   - the ServerHandle is registered globally
///   - when the server is stopped, the ServerHandle will be automatically unregistered
pub fn spawn(
    socket_config: SocketConfig,
    listener_config: ListenerConfig,
    service: ReqRep<nng::Message, nng::Message>,
    mut executor: Executor,
) -> Result<ServerHandle, SpawnError> {
    let (server_command_tx, mut server_command_rx) = futures::channel::mpsc::channel(1);

    let reqrep_id = service.reqrep_id();
    let url = listener_config.url.clone();
    let parallelism = listener_config.parallelism();
    let server_metrics = ServerMetrics::new(reqrep_id);
    let server_handle_id = ULID::generate();

    // TODO: replace with Debug format once https://gitlab.com/neachdainn/nng-rs/issues/23 is released
    fn pipe_event_display(event: nng::PipeEvent) -> &'static str {
        match event {
            nng::PipeEvent::AddPre => "PipeEvent::AddPre",
            nng::PipeEvent::AddPost => "PipeEvent::AddPost",
            nng::PipeEvent::RemovePost => "PipeEvent::RemovePost",
            nng::PipeEvent::Unknown(_) => "nng::PipeEvent::Unknown",
        }
    }

    let create_socket = || {
        let server_metrics = server_metrics.clone();
        let mut socket =
            nng::Socket::new(nng::Protocol::Rep0).map_err(SpawnError::SocketCreateFailure)?;
        socket.set_nonblocking(true);
        socket
            .pipe_notify(move |pipe, event| {
                match event {
                    nng::PipeEvent::AddPost => {
                        server_metrics.active_conn_count.inc();
                        server_metrics.tot_conn_count.inc();
                    }
                    nng::PipeEvent::RemovePost => server_metrics.active_conn_count.dec(),
                    nng::PipeEvent::AddPre => server_metrics.tot_conn_initiate_count.inc(),
                    _ => (),
                }
                debug!("{:?} {}", pipe, pipe_event_display(event));
            })
            .map_err(SpawnError::SocketCreateFailure)?;
        socket_config
            .apply(socket)
            .map_err(SpawnError::SocketConfigApplyFailed)
    };

    let start_listener = |socket: &nng::Socket| {
        listener_config
            .start_listener(socket)
            .map_err(SpawnError::ListenerStartFailure)
    };

    // spawns the worker tasks
    // - each Aio Context is serviced by its own private event loop running as a future
    // - the worker tasks will wait to be signalled via the returned channels to start listening on the Socket
    // - the worker's job is to integrate nng with the backend ReqRep service - it simply relays nng
    //   request messages to the ReqRep service, and then sends back the reply message returned from
    //   the ReqRep service
    //
    // Socket ---> Aio callback ---> worker --- nng::Message --> ReqRep service
    // Socket <----nng::message----- worker <-- nng::Message --- ReqRep service
    let mut create_workers =
        |socket: &nng::Socket| -> Result<Vec<futures::channel::oneshot::Sender<()>>, SpawnError> {
            let mut worker_start_chans = Vec::with_capacity(parallelism);
            for i in 0..parallelism {
                // used to signal the workers to start listening, i.e., start receiving messages
                let (start_tx, start_rx) = futures::channel::oneshot::channel::<()>();
                worker_start_chans.push(start_tx);
                // used to notify the workers when an Aio event has occurred, i.e., the Aio callback has been invoked
                let (aio_tx, mut aio_rx) = futures::channel::mpsc::unbounded::<()>();
                // wrap aio_tx within a Mutex in order to make it unwind safe and usable within  Aio callback
                let aio_tx = Mutex::new(aio_tx);
                let ctx = nng::Context::new(socket).map_err(SpawnError::ContextCreateFailure)?;
                let callback_ctx = ctx.clone();
                let aio = nng::Aio::with_callback(move |_aio| {
                    match aio_tx.lock() {
                        Ok(aio_tx) => {
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
                        }
                        Err(err) => {
                            // This should never happen
                            error!("Failed to obtain lock on Aio sender channel. The Aio Context will be closed: {}", err);
                            // TODO: will this work ?
                            callback_ctx.clone().close();
                            // TODO: trigger an alarm because this should never happen
                        }
                    };
                }).map_err(SpawnError::AioCreateWithCallbackFailure)?;
                let mut service_client = service.clone();
                executor
                    .spawn(
                        async move {
                            debug!(
                                "worker #{} is awaiting signal to start listening ...",
                                i
                            );
                            match await!(start_rx) {
                                Ok(_) => {
                                    debug!("worker #{} is starting ...", i);
                                    let mut state = AioState::Recv;

                                    let recv = |state: AioState| {
                                        if let Err(err) = ctx.recv(&aio) {
                                            // TODO: trigger alert - async I/O errors need to be investigated
                                            error!("{:?}: Context::recv() failed: {}",state, err);
                                        }
                                        AioState::Recv
                                    };

                                    let send = |state: AioState, msg: nng::Message| {
                                        if let Err((_msg,err)) = ctx.send(&aio, msg) {
                                            // TODO: trigger alert - async I/O errors need to be investigated
                                            error!("{:?}: Context::send() failed: {}",state, err);
                                            aio.cancel();
                                            return recv(state);
                                        }
                                        AioState::Send
                                    };

                                    let reply_recv_failed = |state, msg_id, err| {
                                        warn!("Reply was not received for MessageId({}): {}",msg_id, err);
                                        aio.cancel();
                                        recv(state)
                                    };

                                    let reqrep_send_failed = |state, err, reqrep_id| {
                                        error!("ReqRep::send() failed: ReqRepId({}) : {}", reqrep_id , err);
                                        aio.cancel();
                                        recv(state)
                                    };

                                    let no_msg_available = |state| {
                                        warn!("{:?} Expected a message to be available", state);
                                        aio.cancel();
                                        recv(state)
                                    };

                                    let handle_aio_error = |state, err: nng::Error| {
                                        match err.kind() {
                                            nng::ErrorKind::Closed => {
                                                AioState::Closed
                                            },
                                            _ => {
                                                error!("{:?}: Aio error: {}",state, err);
                                                aio.cancel();
                                                recv(state)
                                            }
                                        }
                                    };

                                    let no_io_operation_running = |state| {
                                        // TODO: trigger alert - unexpected behaivor
                                        warn!("{:?}: There is no I/O operation running ... this should never happen", state);
                                        aio.cancel();
                                        recv(state)
                                    };

                                    // start listening
                                    recv(state);
                                    debug!("worker #{} is listening ...", i);
                                    while let Some(_) = await!(aio_rx.next()) {
                                        state = match state {
                                            AioState::Recv => {
                                                match aio.result() {
                                                    Some(Ok(_)) => {
                                                        match aio.get_msg() {
                                                            Some(msg) => {
                                                                match await!(service_client.send(msg)) {
                                                                    Ok(reply_receiver) => {
                                                                        let msg_id = reply_receiver.message_id();
                                                                        match await!(reply_receiver.recv()) {
                                                                            Ok(reply) => send(state, reply),
                                                                            Err(err) => reply_recv_failed(state, msg_id, err)
                                                                        }
                                                                    },
                                                                    Err(err) => reqrep_send_failed(state, err, service_client.reqrep_id())
                                                                }
                                                            },
                                                            None => no_msg_available(state)
                                                        }
                                                    },
                                                    Some(Err(err)) => handle_aio_error(state, err),
                                                    None => no_io_operation_running(state)
                                                }
                                            }
                                            AioState::Send => {
                                                match aio.result() {
                                                    Some(Ok(_)) => recv(state),
                                                    Some(Err(err)) => handle_aio_error(state, err),
                                                    None => no_io_operation_running(state)
                                                }
                                            },
                                            // this state will never be matched against, but we must fulfill the match contract
                                            AioState::Closed => break
                                        };
                                        if state == AioState::Closed {
                                            break;
                                        }
                                    }
                                    debug!("worker #{} task is done", i);
                                }
                                Err(_) => {
                                    debug!("worker #{} task was cancelled", i);
                                }
                            }
                        },
                    )
                    .map_err(|err| SpawnError::ExecutorSpawnError {
                        is_executor_shutdown: err.is_shutdown(),
                    })?;
            }
            Ok(worker_start_chans)
        };

    let start_workers = |worker_start_chans: Vec<futures::channel::oneshot::Sender<()>>,
                         socket: nng::Socket,
                         listener: nng::Listener,
                         mut executor: Executor| {
        executor.spawn_with_handle(async move{
            for c in worker_start_chans {
                if c.send(()).is_err() {
                    // TODO: trigger alert - this should never happen
                    error!("Unable to send worker start signal because the channel has been disconnected");
                }
            }
            debug!("Server({}) is running ...", reqrep_id);
            while let Some(cmd) = await!(server_command_rx.next()) {
                match cmd {
                    ServerCommand::Ping(reply_chan) => {
                        let _ = reply_chan.send(());
                    },
                    ServerCommand::Stop => break
                }
            }
            debug!("Server({}) is shutting down ...", reqrep_id);
            listener.close();
            socket.close();
            debug!("Server({}) is shut down", reqrep_id);
            let mut server_handles = SERVER_HANDLES.write().unwrap();
            server_handles.remove(&server_handle_id);
        }).map_err(|err| SpawnError::ExecutorSpawnError {
            is_executor_shutdown: err.is_shutdown()
        })
    };

    let socket = create_socket()?;
    let worker_start_chans = create_workers(&socket)?;
    let listener = start_listener(&socket)?;
    let handle = start_workers(worker_start_chans, socket, listener, executor.clone())?;

    let server_handle = ServerHandle {
        id: server_handle_id,
        url,
        reqrep_id,
        parallelism: NonZeroUsize::new(parallelism).unwrap(),
        handle: Some(handle.shared()),
        server_command_channel: Some(server_command_tx),
        executor,
        metrics: server_metrics,
    };

    let mut server_handles = SERVER_HANDLES.write().unwrap();
    server_handles.insert(server_handle.id(), Arc::new(server_handle.clone()));

    Ok(server_handle)
}

/// Server handle
///
/// There are 2 ways to stop the server:
/// 1. Explicitly signal the server to stop via [stop_async()](#method.stop_async)
/// 2. When all instances of the ServerHandle are dropped
#[derive(Debug, Clone)]
pub struct ServerHandle {
    id: ULID,
    url: String,
    reqrep_id: ReqRepId,
    parallelism: NonZeroUsize,
    handle: Option<future::Shared<future::RemoteHandle<()>>>,
    server_command_channel: Option<futures::channel::mpsc::Sender<ServerCommand>>,
    executor: Executor,
    metrics: ServerMetrics,
}

impl ServerHandle {
    /// Returns the ServerHandle ULID
    pub fn id(&self) -> ULID {
        self.id
    }

    /// Returns the URI that the server is listening on
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Returns the ReqRepId for the backend service
    pub fn reqrep_id(&self) -> ReqRepId {
        self.reqrep_id
    }

    /// Number of outstanding requests that the server can handle at a given time.
    ///
    /// This is *NOT* the number of threads in use, but instead represents outstanding work items.
    pub fn parallelism(&self) -> usize {
        self.parallelism.get()
    }

    /// returns true if the server has been signalled to stop
    pub fn stop_signalled(&self) -> bool {
        self.server_command_channel.is_none()
    }

    /// Returns ServerMetrics
    pub fn metrics(&self) -> &ServerMetrics {
        &self.metrics
    }

    /// pings the server to check if it is still alive
    ///
    /// ## ServerHandleError
    /// - Internally, the Ping message is sent via an async channel, i.e., Futures based. This requires
    ///   an Executor to spawn the task to send the Ping message. If the Executor fails to spawn the
    ///   task, then a ServerHandleError will be returned. Normally, this should never happen ...
    pub fn ping(&self) -> Result<bool, ServerHandleError> {
        match self.server_command_channel {
            Some(ref server_command_channel) => {
                let mut server_command_channel = server_command_channel.clone();
                let mut executor = self.executor.clone();
                executor
                    .spawn_await(
                        async move {
                            let (tx, rx) = futures::channel::oneshot::channel();
                            if await!(server_command_channel.send(ServerCommand::Ping(tx))).is_ok()
                            {
                                let _ = await!(rx);
                                true
                            } else {
                                false
                            }
                        },
                    )
                    .map_err(|err| ServerHandleError(err.to_string()))
            }
            None => Ok(false),
        }
    }

    /// signals the server to shutdown async
    pub fn stop_async(&mut self) -> Result<bool, ServerHandleError> {
        if let Some(mut c) = self.server_command_channel.take() {
            self.executor
                .spawn(
                    async move {
                        // the result can be ignored because if the channel is disconnected then it means the
                        // server has stopped
                        let _ = await!(c.send(ServerCommand::Stop));
                    },
                )
                .map_err(|err| {
                    if err.is_shutdown() {
                        ServerHandleError("executor is shutdown".to_string())
                    } else {
                        ServerHandleError("executor failed to spawn the task".to_string())
                    }
                })?;
            return Ok(true);
        }
        Ok(false)
    }

    /// Block the current thread until the server has shutdown
    ///
    /// ## Notes
    /// The server must be signaled to stop in order to shutdown.
    pub fn await_shutdown(mut self) -> Result<(), ServerHandleError> {
        if let Some(handle) = self.handle.take() {
            return self
                .executor
                .spawn_await(async { await!(handle) })
                .map_err(|err| ServerHandleError(err.to_string()));
        }
        Ok(())
    }

    /// Returns the ServerHandle - only if the server is still alive
    /// - ServerHandle(s) are globally registered when the server is spawned
    pub fn get(id: ULID) -> Option<Arc<ServerHandle>> {
        let server_handle = {
            let server_handles = SERVER_HANDLES.read().unwrap();
            server_handles.get(&id).cloned()
        };

        // check if the server is still alive
        if let Some(server_handle) = server_handle {
            if let Ok(true) = server_handle.ping() {
                Some(server_handle)
            } else {
                // unregister the Serverhandle because pinging the server failed
                {
                    let mut server_handles = SERVER_HANDLES.write().unwrap();
                    server_handles.remove(&id);
                }
                None
            }
        } else {
            None
        }
    }

    /// Returns the list of registered ServerHandle ULIDs along with the server's ReqRepId
    pub fn ids() -> Vec<(ULID, ReqRepId)> {
        let server_handles = SERVER_HANDLES.read().unwrap();
        server_handles
            .values()
            .map(|server_handle| (server_handle.id, server_handle.reqrep_id))
            .collect()
    }

    /// Returns ServerHandle(s) that are registered for the specified ReqRepId
    pub fn get_by_reqrep_id(reqrep_id: ReqRepId) -> Vec<Arc<ServerHandle>> {
        let server_handles = SERVER_HANDLES.read().unwrap();
        server_handles
            .values()
            .filter(|server_handle| server_handle.reqrep_id == reqrep_id)
            .cloned()
            .collect()
    }
}

/// ServerHandle error
#[derive(Fail, Debug, Clone)]
#[fail(display = "ServerHandle error: {}", _0)]
pub struct ServerHandleError(String);

/// Server commands
#[derive(Debug)]
pub enum ServerCommand {
    /// Ping the server to check if it is still alive
    Ping(futures::channel::oneshot::Sender<()>),
    /// Signals the server to shutdown
    Stop,
}

/// Errors that could happen while trying to spawn a server
#[derive(Debug, Fail)]
pub enum SpawnError {
    /// Failed to create Socket
    #[fail(display = "Failed to create Socket: {}", _0)]
    SocketCreateFailure(nng::Error),
    /// Failed to create ListenerOptions
    #[fail(display = "Failed to create ListenerOptions: {}", _0)]
    ListenerOptionsCreateFailure(nng::Error),
    /// Failed to create Context
    #[fail(display = "Failed to create Context: {}", _0)]
    ContextCreateFailure(nng::Error),
    /// Failed to create Context
    #[fail(display = "Failed to create Aio with callback: {}", _0)]
    AioCreateWithCallbackFailure(nng::Error),
    /// An error that occurred during spawning.
    #[fail(
        display = "Spawning Future failed: executor shutdown = {}",
        is_executor_shutdown
    )]
    ExecutorSpawnError {
        /// whether spawning failed because the executor is shut down
        is_executor_shutdown: bool,
    },
    /// Failed to start the listener
    #[fail(display = "{}", _0)]
    ListenerStartFailure(ListenerConfigError),
    /// Failed to apply SocketConfig options
    #[fail(display = "{}", _0)]
    SocketConfigApplyFailed(SocketConfigError),
}

/// Aio state for socket context
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
enum AioState {
    /// aio receive operation is in progress
    Recv,
    /// aio send operation is in progress
    Send,
    /// Closed
    Closed,
}

/// Server metrics
#[derive(Clone)]
pub struct ServerMetrics {
    active_conn_count: prometheus::IntGauge,
    tot_conn_count: prometheus::IntCounter,
    tot_conn_initiate_count: prometheus::IntCounter,
}

impl ServerMetrics {
    fn new(reqrep_id: ReqRepId) -> Self {
        let reqrep_id_label = reqrep_id.to_string();
        Self {
            active_conn_count: ACTIVE_CONN_COUNT.with_label_values(&[reqrep_id_label.as_str()]),
            tot_conn_count: TOT_CONN_COUNT.with_label_values(&[reqrep_id_label.as_str()]),
            tot_conn_initiate_count: TOT_CONN_INITIATE_COUNT
                .with_label_values(&[reqrep_id_label.as_str()]),
        }
    }

    /// Active number of socket connections
    pub fn active_conn_count(&self) -> usize {
        self.active_conn_count.get() as usize
    }

    /// Total number of socket connections since the server was started
    pub fn tot_conn_count(&self) -> usize {
        self.tot_conn_count.get() as usize
    }

    /// Total number of connections that have been initiated, but before being added to the socket, since the server was started.
    pub fn tot_conn_initiate_count(&self) -> usize {
        self.tot_conn_initiate_count.get() as usize
    }
}

impl fmt::Debug for ServerMetrics {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"ServerMetrics(active_conn_count = {}, tot_conn_count = {}, tot_conn_initiate_count = {})",
               self.active_conn_count.get(),
               self.tot_conn_count.get(),
               self.tot_conn_initiate_count.get()
        )
    }
}

/// Listener settings
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct ListenerConfig {
    url: String,
    recv_max_size: Option<usize>,
    no_delay: Option<bool>,
    keep_alive: Option<bool>,
    non_blocking: bool,
    parallelism: usize,
}

impl ListenerConfig {
    /// constructor
    ///
    /// ## Default settings
    /// - non_blocking = true
    /// - parallelism = num of available CPUs + 1
    pub fn new(url: &str) -> ListenerConfig {
        ListenerConfig {
            url: url.to_string(),
            recv_max_size: None,
            no_delay: None,
            keep_alive: None,
            non_blocking: true,
            parallelism: num_cpus::get() + 1,
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
    pub fn start_listener(
        &self,
        socket: &nng::Socket,
    ) -> Result<nng::Listener, ListenerConfigError> {
        let options = nng::ListenerOptions::new(socket, self.url())
            .map_err(ListenerConfigError::ListenerOptionsCreateFailed)?;

        if let Some(option) = self.recv_max_size.as_ref() {
            options
                .set_opt::<nng::options::RecvMaxSize>(*option)
                .map_err(ListenerConfigError::RecvMaxSize)?;
        }

        if let Some(option) = self.no_delay.as_ref() {
            options
                .set_opt::<nng::options::transport::tcp::NoDelay>(*option)
                .map_err(ListenerConfigError::TcpNoDelay)?;
        }

        if let Some(option) = self.keep_alive.as_ref() {
            options
                .set_opt::<nng::options::transport::tcp::KeepAlive>(*option)
                .map_err(ListenerConfigError::TcpKeepAlive)?;
        }

        options
            .start(self.non_blocking)
            .map_err(|(_options, err)| ListenerConfigError::ListenerStartFailed(err))
    }

    /// the address that the server is listening on
    pub fn url(&self) -> &str {
        &self.url
    }

    /// if true, then it binds to the address asynchronously
    pub fn non_blocking(&self) -> bool {
        self.non_blocking
    }

    /// Number of outstanding requests that the server can handle at a given time.
    ///
    /// This is *NOT* the number of threads in use, but instead represents outstanding work items.
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

    /// Sets the maximum message size that the will be accepted from a remote peer.
    pub fn set_recv_max_size(mut self, recv_max_size: usize) -> Self {
        self.recv_max_size = Some(recv_max_size);
        self
    }

    /// Sets no delay setting on TCP connection
    pub fn set_no_delay(mut self, no_delay: bool) -> Self {
        self.no_delay = Some(no_delay);
        self
    }

    /// Sets keep alive setting on TCP connection
    pub fn set_keep_alive(mut self, keep_alive: bool) -> Self {
        self.keep_alive = Some(keep_alive);
        self
    }

    /// Normally, the act of "binding" to the address indicated by url is done synchronously, including
    /// any necessary name resolution. As a result, a failure, such as if the address is already in use,
    /// will be returned immediately. However, if nonblocking is specified then this is done asynchronously;
    /// furthermore any failure to bind will be periodically reattempted in the background.
    pub fn set_non_blocking(mut self, non_blocking: bool) -> Self {
        self.non_blocking = non_blocking;
        self
    }

    /// set the number of async IO operations that can be performed concurrently
    pub fn set_aio_count(mut self, count: NonZeroUsize) -> Self {
        self.parallelism = count.get();
        self
    }
}

/// Socket config related errors
#[derive(Debug, Fail)]
pub enum ListenerConfigError {
    /// Failed to create ListenerOpion
    #[fail(display = "Failed to create ListenerOpions: {}", _0)]
    ListenerOptionsCreateFailed(nng::Error),
    /// Failed start the Listener
    #[fail(display = "Failed start the Listener: {}", _0)]
    ListenerStartFailed(nng::Error),
    ///Failed to set the RecvMaxSize Socket option
    #[fail(display = "Failed to set the RecvMaxSize Socket option: {}", _0)]
    RecvMaxSize(nng::Error),
    /// Failed to set the TcpNoDelay Socket option
    #[fail(display = "Failed to set the TcpNoDelay Socket option: {}", _0)]
    TcpNoDelay(nng::Error),
    /// Failed to set the TcpKeepAlive Socket option
    #[fail(display = "Failed to set the TcpKeepAlive Socket option: {}", _0)]
    TcpKeepAlive(nng::Error),
}

#[allow(warnings)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        concurrent::{
            execution::*,
            messaging::reqrep::{self, *},
        },
        configure_logging, metrics,
    };
    use oysterpack_uid::ULID;
    use oysterpack_uid::*;
    use std::{thread, time::Duration};

    struct EchoService;
    impl Processor<nng::Message, nng::Message> for EchoService {
        fn process(&mut self, req: nng::Message) -> reqrep::FutureReply<nng::Message> {
            async move { req }.boxed()
        }
    }

    const REQREP_ID: ReqRepId = ReqRepId(1871557337320005579010710867531265404);

    fn start_service() -> ReqRep<nng::Message, nng::Message> {
        let timer_buckets = metrics::TimerBuckets::from(
            vec![
                Duration::from_nanos(50),
                Duration::from_nanos(100),
                Duration::from_nanos(150),
                Duration::from_nanos(200),
            ]
            .as_slice(),
        );
        ReqRep::start_service(
            REQREP_ID,
            1,
            EchoService,
            global_executor().clone(),
            timer_buckets,
        )
        .unwrap()
    }

    #[test]
    fn nng_server_single_client() {
        configure_logging();

        // GIVEN: the server is running
        let url = format!("inproc://{}", ULID::generate());
        let mut server_handle = super::spawn(
            SocketConfig::default(),
            ListenerConfig::new(url.as_str()),
            start_service(),
            global_executor().clone(),
        )
        .unwrap();
        assert!(server_handle.ping().unwrap());

        // GIVEN: a client that connects to the server
        let mut s = nng::Socket::new(nng::Protocol::Req0).unwrap();
        s.dial(url.as_str()).unwrap();

        for i in 1..=10 {
            // WHEN: the client submits requests
            s.send(nng::Message::new().unwrap()).unwrap();
            info!("[{}] Sent request", i);
            // THEN: the client successfully receives a response
            let _ = s.recv().unwrap();
            info!("[{}] Received response", i);
        }

        // THEN: the server handle is registered
        let server_handle_ref = ServerHandle::get(server_handle.id()).unwrap();
        assert!(server_handle_ref.ping().unwrap());
        assert!(ServerHandle::ids()
            .iter()
            .find(|(id, reqrep_id)| *id == server_handle_ref.id()
                && *reqrep_id == server_handle_ref.reqrep_id())
            .is_some());
        let server_handles = ServerHandle::get_by_reqrep_id(REQREP_ID);
        assert!(server_handles
            .iter()
            .find(|server_handle| server_handle.reqrep_id() == REQREP_ID
                && server_handle.id() == server_handle_ref.id())
            .is_some());

        // WHEN: the server is signalled to stop
        assert!(server_handle.stop_async().unwrap());
        // THEN: the server shuts down
        server_handle.await_shutdown();

        // AND: the server handle becomes invalid
        assert!(!server_handle_ref.ping().unwrap());
        assert!(ServerHandle::get(server_handle_ref.id()).is_none());
        assert!(ServerHandle::ids()
            .iter()
            .find(|(id, reqrep_id)| *id == server_handle_ref.id()
                && *reqrep_id == server_handle_ref.reqrep_id())
            .is_none());
        let server_handles = ServerHandle::get_by_reqrep_id(REQREP_ID);
        assert!(server_handles
            .iter()
            .find(|server_handle| server_handle.reqrep_id() == REQREP_ID
                && server_handle.id() == server_handle_ref.id())
            .is_none());

        let mut executor = global_executor();
        // GIVEN: the server is not running
        // WHEN: the client submits requests
        let handle = executor
            .spawn_with_handle(
                async move {
                    s.send(nng::Message::new().unwrap()).unwrap();
                    let _ = s.recv().unwrap();
                    s.send(nng::Message::new().unwrap()).unwrap();
                    info!("Sent request while server was shutdown ...");
                    let reply = s.recv().unwrap();
                    info!("... Received response after server was restarted");
                    reply
                },
            )
            .unwrap();

        // WHEN: the server is restarted
        let mut server_handle = super::spawn(
            SocketConfig::default(),
            ListenerConfig::new(url.as_str()),
            start_service(),
            global_executor().clone(),
        )
        .unwrap();
        assert!(server_handle.ping().unwrap());

        // THEN: the client will be able to connect and be serviced
        let reply = executor.spawn_await(handle).unwrap();
        info!("Reply was received: {:?}", reply);

        // WHEN: the server handle is dropped
        drop(server_handle);

        // THEN: the server continues running because a ServerHandle reference is registered
        // AND: clients are still able to connect to the server
        let mut s = nng::Socket::new(nng::Protocol::Req0).unwrap();
        let result = s.dial(url.as_str());
        assert!(result.is_ok());
    }

    #[test]
    fn nng_server_multi_client() {
        configure_logging();

        // GIVEN: the server is running
        let url = format!("inproc://{}", ULID::generate());
        let mut server_handle = super::spawn(
            SocketConfig::default(),
            ListenerConfig::new(url.as_str()),
            start_service(),
            global_executor().clone(),
        )
        .unwrap();
        assert!(server_handle.ping().unwrap());

        let mut client_task_handles = Vec::new();

        // The clients need their own dedicated Executor, i.e., thread pool because the client tasks
        // will block the threads. If they were to share the server executor then the clients will
        // consume all the threads in the pool and block waiting for a reply. The server cannot reply
        // because there wouldn't be any free threads available in the pool.
        const CLIENT_COUNT: usize = 100;
        let mut executor = ExecutorBuilder::new(ExecutorId::generate())
            .set_pool_size(NonZeroUsize::new(CLIENT_COUNT).unwrap())
            .register()
            .unwrap();
        for _ in 0..CLIENT_COUNT {
            let url = url.clone();
            // GIVEN: a client that connects to the server
            let handle = executor
                .spawn_with_handle(
                    async move {
                        let mut s = nng::Socket::new(nng::Protocol::Req0).unwrap();
                        s.dial(url.as_str()).unwrap();

                        let client_id = ULID::generate();
                        for i in 1..=10 {
                            // WHEN: the client submits requests
                            s.send(nng::Message::new().unwrap()).unwrap();
                            info!("[{}::{}] Sent request", client_id, i);
                            // THEN: the client successfully receives a response
                            let _ = s.recv().unwrap();
                            info!("[{}::{}] Received response", client_id, i);
                        }
                        client_id
                    },
                )
                .unwrap();
            client_task_handles.push(handle);
        }

        info!("server metrics: {:?}", server_handle.metrics());

        assert_eq!(client_task_handles.len(), 100);
        let mut executor = global_executor();
        for handle in client_task_handles {
            info!("waiting for client to be done ...");
            let client_id = executor.spawn_await(handle).unwrap();
            info!(
                "client is done: {} : {:?}",
                client_id,
                server_handle.metrics()
            );
        }

        info!("all clients are done: {:#?}", server_handle.metrics());

        // WHEN: the server is signalled to stop
        assert!(server_handle.stop_async().unwrap());
        // THEN: the server shuts down
        server_handle.await_shutdown();
    }

}
