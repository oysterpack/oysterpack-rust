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

//! Provides support for Future compatible messaging

pub mod errors;
pub mod reqrep;

use oysterpack_uid::macros::ulid;
use serde::{Deserialize, Serialize};

/// Message ULID
///
/// ## Use Case
/// 1. Used to track messages
#[ulid]
pub struct MessageId(u128);

#[allow(warnings)]
#[cfg(test)]
mod tests {
    use futures::{
        channel::oneshot,
        stream::StreamExt,
        task::{Spawn, SpawnExt},
    };

    #[test]
    fn try_recv_after_already_received_on_oneshot_channel() {
        let (p, mut c) = oneshot::channel();
        p.send(1);
        assert_eq!(c.try_recv().unwrap().unwrap(), 1);
        // trying to receive a message after it already received a message results in an error
        // at this point the Receiver is cancelled
        assert!(c.try_recv().is_err());
    }

}
