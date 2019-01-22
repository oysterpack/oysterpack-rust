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
#[derive(Fail, Debug)]
pub enum ChannelSendError {
    /// Failed to send because the channel is full
    #[fail(display = "Failed to send message because the channel is full")]
    Full,
    /// Failed to send because the channel is disconnected
    #[fail(display = "Failed to send message because the channel is disconnected")]
    Disconnected,
}

impl From<channel::mpsc::SendError> for ChannelSendError {
    fn from(err: channel::mpsc::SendError) -> Self {
        if err.is_disconnected() {
            return ChannelSendError::Disconnected;
        }
        ChannelSendError::Full
    }
}
