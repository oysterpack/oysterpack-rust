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

//! Common messaging related errors

use failure::Fail;
use futures::channel;

/// Channel sending related errors
#[derive(Fail, Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum ChannelError {
    /// Sender channel is disconnected
    #[fail(display = "Sender channel is disconnected")]
    SenderDisconnected,
    /// Receiver channel is disconnected
    #[fail(display = "Receiver channel is disconnected")]
    ReceiverDisconnected,
}

impl From<channel::mpsc::SendError> for ChannelError {
    fn from(_: channel::mpsc::SendError) -> Self {
        ChannelError::SenderDisconnected
    }
}

impl From<futures::channel::oneshot::Canceled> for ChannelError {
    fn from(_: futures::channel::oneshot::Canceled) -> Self {
        ChannelError::ReceiverDisconnected
    }
}
