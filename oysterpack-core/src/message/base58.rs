// Copyright 2018 OysterPack Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Provides support for base-58 encoding bytes.
//! - base-58 is used for the same reasons why Bitcoin uses it - to make it more user friendly

use oysterpack_errors::{Error, Id, IsError, Level};
use std::fmt;

/// encodes the bytes into base58 using the Bitcoin alphabet
pub fn encode(bytes: &[u8]) -> String {
    bs58::encode(bytes).into_string()
}

/// decodes a base58 encoding using the Bitcoin alphabet
pub fn decode(key: &str) -> Result<Vec<u8>, Error> {
    bs58::decode(key)
        .into_vec()
        .map_err(|err| op_error!(DecodeError(err)))
}

/// DecodeError
#[derive(Debug, Clone)]
pub struct DecodeError(bs58::decode::DecodeError);

impl DecodeError {
    /// Error ID(01CY9KFG3YQKD3WT1Y16DSZ74S)
    pub const ERR_ID: Id = Id(1867020503549016568488932545115692185);
    /// Error level
    pub const ERR_LEVEL: Level = Level::Error;
}

impl IsError for DecodeError {
    fn error_id(&self) -> Id {
        Self::ERR_ID
    }

    fn error_level(&self) -> Level {
        Self::ERR_LEVEL
    }
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.0.to_string().as_str())
    }
}
