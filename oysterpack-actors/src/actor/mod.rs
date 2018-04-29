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

//! The actor module provides the standard for OysterPack Actors:
//!
//! 1. All Actor instances will be assigned a unique ActorId

extern crate actix;
extern crate chrono;
extern crate futures;
extern crate oysterpack_id;

use self::actix::prelude::*;
use self::futures::prelude::*;
use self::chrono::prelude::*;

use self::oysterpack_id::Id;

/// Type alias for Actor message response futures.
/// The future error type is MailboxError, which indicates an error occurred while sending the request.
/// If a message can result in error, then the response type should be wrapped in a Result.
pub type ActorMessageResponse<T> = Box<Future<Item = T, Error = MailboxError>>;

/// Provides support for building new Actors following standards.
/// The StandardActor functionality is integrated via its lifecyle.
///
/// It provides the following functionality:
/// 1. Each actor is assigned a unique
pub struct ActorInstance {
    instance_id: ActorInstanceId,
    created_on: DateTime<Utc>,
}

impl ActorInstance {
    ///
    pub fn new() -> ActorInstance {
        ActorInstance {
            instance_id: ActorInstanceId::new(),
            created_on: Utc::now(),
        }
    }

    /// Returns the Actor's instance id.
    pub fn instance_id(&self) -> ActorInstanceId {
        self.instance_id
    }

    pub fn created_on(&self) -> DateTime<Utc> {
        self.created_on
    }
}

/// Each new Actor instance is assigned a unique ActorInstanceId.
pub type ActorInstanceId = Id<ActorInstance>;
