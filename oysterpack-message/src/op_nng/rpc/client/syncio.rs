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

//! Synchronous client

use super::{errors, ClientSocketSettings, DialerSettings};
use nng::options::Options;
use oysterpack_errors::{op_error, Error};
use std::fmt;

/// nng RPC client
pub struct SyncClient {
    // the order is important because Rust will drop fields in the order listed
    // the dialer must be dropped before the socket, otherwise the following error occurs
    //
    // thread 'op_nng::rpc::client::tests::sync_client' panicked at 'Unexpected error code while closing dialer (12)', ... /nng-0.3.0/src/dialer.rs:104:3
    //
    // i.e., the dialer must be closed before the socket is closed
    dialer: nng::Dialer,
    socket: nng::Socket,
}

impl SyncClient {
    /// Sends the request and wait for a reply synchronously
    /// - the messages are snappy compressed and bincode serialized - see the [marshal](../../../../marshal/index.html) module
    pub fn send(&mut self, req: nng::Message) -> Result<nng::Message, Error> {
        self.socket
            .send(req)
            .map_err(|err| op_error!(errors::SocketSendError::from(err)))?;
        self.socket
            .recv()
            .map_err(|err| op_error!(errors::SocketRecvError::from(err)))
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
        let socket = ClientSocketSettings::create_socket(this.socket_settings.take())?;
        let dialer = this.dialer_settings.start_dialer(&socket)?;
        Ok(SyncClient { socket, dialer })
    }
}
