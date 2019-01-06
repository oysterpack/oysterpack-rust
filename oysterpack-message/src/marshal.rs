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

//! provides marshalling functions for serialization / deserialization
//! - bincode is used for serialization
//! - snappy is used for compression

use crate::errors;
use oysterpack_errors::{op_error, Error, ErrorMessage};
use serde::{de::DeserializeOwned, Serialize};

/// serialized via bincode and then compressed via snappy
pub fn serialize<T: Serialize>(o: &T) -> Result<Vec<u8>, Error> {
    let bytes = bincode::serialize(o)
        .map_err(|err| op_error!(errors::BincodeSerializeError(ErrorMessage(err.to_string()))))?;
    Ok(parity_snappy::compress(&bytes))
}

/// decompressed via snappy and then deserialized via snappy
pub fn deserialize<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, Error> {
    let bytes = parity_snappy::decompress(bytes)
        .map_err(|err| op_error!(errors::SnappyDecompressError(ErrorMessage(err.to_string()))))?;
    bincode::deserialize(&bytes).map_err(|err| {
        op_error!(errors::BincodeDeserializeError(ErrorMessage(
            err.to_string()
        )))
    })
}
