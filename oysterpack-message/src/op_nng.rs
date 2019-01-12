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

//! provides nng support

use crate::errors;
use oysterpack_errors::{op_error, Error, ErrorMessage};
use serde::{de::DeserializeOwned, Serialize};

// TODO: implement TryFrom when it bocomes stable
/// Converts an nng:Message into a SealedEnvelope.
pub fn try_from_nng_message<T>(msg: &nng::Message) -> Result<T, Error>
where
    T: Serialize + DeserializeOwned,
{
    bincode::deserialize(&**msg).map_err(|err| {
        op_error!(errors::BincodeDeserializeError(ErrorMessage(
            err.to_string()
        )))
    })
}

// TODO: implement TryInto when it becomes stable
/// Converts itself into an nng:Message
pub fn try_into_nng_message<T>(msg: &T) -> Result<nng::Message, Error>
where
    T: Serialize + DeserializeOwned,
{
    let bytes = bincode::serialize(msg).map_err(|err| {
        op_error!(errors::BincodeSerializeError(ErrorMessage(format!(
            "SealedEnvelope : {}",
            err
        ))))
    })?;
    let mut msg = nng::Message::with_capacity(bytes.len()).map_err(|err| {
        op_error!(errors::NngMessageError::from(ErrorMessage(format!(
            "Failed to create an empty message with capacity = {}: {}",
            bytes.len(),
            err
        ))))
    })?;
    msg.push_back(&bytes).map_err(|err| {
        op_error!(errors::NngMessageError::from(ErrorMessage(format!(
            "Failed to append data to the back of the message body: {}",
            err
        ))))
    })?;
    Ok(msg)
}
