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
use failure::Fail;
use futures::{sink::SinkExt, stream::StreamExt, task::SpawnExt};
use oysterpack_log::*;
use std::{num::NonZeroUsize, sync::Mutex};

/// Spawns a server background task
/// - the server runs as a Future
/// - returns a channel that can be used to stop the server
pub fn spawn(
    url: String,
    parallelism: NonZeroUsize,
    service: ReqRep<nng::Message, nng::Message>,
    mut executor: Executor,
) -> Result<futures::channel::mpsc::Sender<ServerCommand>, SpawnError> {
    let (server_command_tx, mut server_command_rx) = futures::channel::mpsc::channel(1);
    let reqrep_id = service.reqrep_id();

    let create_socket = || {
        let mut socket =
            nng::Socket::new(nng::Protocol::Rep0).map_err(SpawnError::SocketCreateFailure)?;
        socket.set_nonblocking(true);
        Ok(socket)
    };

    let create_listener_options = |socket: &nng::Socket| {
        let listener_options = nng::ListenerOptions::new(socket, url.as_str())
            .map_err(SpawnError::ListenerOptionsCreateFailure)?;
        Ok(listener_options)
    };

    let start_listener = |listener_options: nng::ListenerOptions| {
        listener_options
            .start(true)
            .map_err(|(_, err)| SpawnError::ListenerStartFailure(err))
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
            let mut worker_start_chans = Vec::with_capacity(parallelism.get());
            for i in 0..parallelism.get() {
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
                                    debug!("worker #{} is listening ...", i);
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

                                    recv(state);

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
                                            AioState::Closed => {
                                                debug!("worker #{} task is exiting because Aio Context is closed.", i);
                                                break;
                                            }
                                        };
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
        executor.spawn(async move{
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
        }).map_err(|err| SpawnError::ExecutorSpawnError {
            is_executor_shutdown: err.is_shutdown()
        })
    };

    let socket = create_socket()?;
    let listener_options = create_listener_options(&socket)?;
    let worker_start_chans = create_workers(&socket)?;
    let listener = start_listener(listener_options)?;
    start_workers(worker_start_chans, socket, listener, executor)?;

    Ok(server_command_tx)
}

// TODO: need a way to wait for the server to shutdown cleanly
/// Server handle
#[derive(Debug)]
pub struct ServerHandle {
    url: String,
    reqrep_id: ReqRepId,
    parallelism: NonZeroUsize,
    handle: Option<futures::future::RemoteHandle<()>>,
    server_command_channel: Option<futures::channel::mpsc::Sender<ServerCommand>>,
    executor: Executor,
}

impl ServerHandle {
    /// returns true if the server has been signalled to stop
    pub fn stop_triggered(&self) -> bool {
        self.server_command_channel.is_none()
    }

    /// signals the server to shutdown
    pub async fn trigger_stop(&mut self) {
        if let Some(mut c) = self.server_command_channel.take() {
            // the result can be ignored because if the channel is disconnected then it means the
            // server has stopped
            let _ = await!(c.send(ServerCommand::Stop));
        }
    }

    /// Block the current thread until the server has shutdown
    ///
    /// ## Notes
    /// If the stop is not triggered before invoking this method, then the server can not be
    pub fn await_shutdown(mut self) {
        if let Some(handle) = self.handle.take() {
            let _ = self.executor.spawn_await(async { await!(handle) });
        }
    }
}

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
    #[fail(display = "Failed to start the listener: {}", _0)]
    ListenerStartFailure(nng::Error),
}

/// Aio state for socket context. 0OMDIgOKlS2c
#[derive(Debug, Copy, Clone)]
enum AioState {
    /// aio receive operation is in progress
    Recv,
    /// aio send operation is in progress
    Send,
    /// Closed
    Closed,
}

#[allow(warnings)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        concurrent::{execution::*, messaging::reqrep::*},
        configure_logging, metrics,
    };
    use oysterpack_uid::*;
    use std::{thread, time::Duration};

    #[test]
    fn nng_server_poc() {
        configure_logging();

        const REQREP_ID: ReqRepId = ReqRepId(1871557337320005579010710867531265404);
        let mut executor = global_executor();

        struct Service;
        impl Processor<nng::Message, nng::Message> for Service {
            fn process(&mut self, req: nng::Message) -> nng::Message {
                req
            }
        }

        let timer_buckets = metrics::TimerBuckets::from(
            vec![
                Duration::from_nanos(50),
                Duration::from_nanos(100),
                Duration::from_nanos(150),
                Duration::from_nanos(200),
            ]
            .as_slice(),
        );
        let mut service =
            ReqRep::start_service(REQREP_ID, 1, Service, executor.clone(), timer_buckets).unwrap();

        let url = format!("inproc://{}", ULID::generate());
        let parallelism = NonZeroUsize::new(num_cpus::get()).unwrap();
        let server_command_channel =
            super::spawn(url.clone(), parallelism, service, global_executor().clone()).unwrap();

        let mut executor = global_executor();

        let mut server_command_chan = server_command_channel.clone();
        executor.spawn_await(
            async move {
                let (tx, rx) = futures::channel::oneshot::channel();
                if await!(server_command_chan.send(ServerCommand::Ping(tx))).is_ok() {
                    info!("Sent Ping");
                    let _ = await!(rx);
                    info!("Received Ping reply");
                } else {
                    error!("Failed to send Ping");
                }
            },
        );

        let mut s = nng::Socket::new(nng::Protocol::Req0).unwrap();
        s.dial(url.as_str()).unwrap();

        for i in 1..=10 {
            s.send(nng::Message::new().unwrap()).unwrap();
            info!("[{}] Sent request", i);
            let _ = s.recv().unwrap();
            info!("[{}] Received response", i);
        }

        let mut server_command_chan = server_command_channel.clone();
        executor.spawn_await(
            async move {
                let _ = await!(server_command_chan.send(ServerCommand::Stop));
            },
        );
        thread::sleep_ms(200);
    }

}
