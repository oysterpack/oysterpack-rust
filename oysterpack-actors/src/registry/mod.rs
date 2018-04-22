// Copyright 2018 OysterPack Inc.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! The registry module provides registries for:
//! 1. Arbiter(s)
//! 2. Actor(s)
//! 3. SyncActor(s)
//!
//! The registries are provided as Actix SystemService(s)
//!
//! Arbiter Registry Features:
//! 1. Start and register a new Arbiter
//! 2. Arbiters are assigned a unique ArbiterId
//! 3. Arbiter can be looked up by ArbiterId
//!

#[cfg(test)]
mod tests;

extern crate actix;
extern crate futures;
extern crate oysterpack_id;

use self::actix::prelude::*;
use self::futures::prelude::*;
use self::oysterpack_id::Id;

use std::{fmt, collections::HashMap, fmt::{Display, Formatter}, hash::{Hash, Hasher}};

/// Arbiter registry
pub struct ArbiterRegistry {
    arbiters: HashMap<ArbiterId, Addr<Syn, Arbiter>>,
}

impl Supervised for ArbiterRegistry {}

impl SystemService for ArbiterRegistry {}

impl Default for ArbiterRegistry {
    fn default() -> Self {
        ArbiterRegistry {
            arbiters: HashMap::new(),
        }
    }
}

impl Actor for ArbiterRegistry {
    type Context = Context<Self>;
}

/// Unique Arbiter id.
///
/// ArbiterId(s) can be defined as static constansts leveraging the [lazy_static](https://docs.rs/crate/lazy_static).
pub type ArbiterId = Id<Arbiter>;

/// Type alias for an Arbiter Addr
pub type ArbiterAddr = Addr<Syn, Arbiter>;

/// Type alias used for Result Error types that should never result in an Error
pub type Never = ();

/// Looks up an Arbiter address. If one does not exist for the specified id, then a new one is created and registered on demand.
pub fn arbiter(id: ArbiterId) -> Box<Future<Item = ArbiterAddr, Error = MailboxError>> {
    let service = Arbiter::system_registry().get::<ArbiterRegistry>();
    let request = service.send(GetArbiter(id));

    let request = request.map(|result| result.unwrap());
    Box::new(request)
}

struct GetArbiter(ArbiterId);

impl Message for GetArbiter {
    type Result = Result<ArbiterAddr, Never>;
}

impl Handler<GetArbiter> for ArbiterRegistry {
    type Result = Result<ArbiterAddr, Never>;

    fn handle(&mut self, msg: GetArbiter, ctx: &mut Self::Context) -> Self::Result {
        if !self.arbiters.contains_key(&msg.0) {
            let arbiter = Arbiter::new(msg.0.to_string());
            self.arbiters.insert(msg.0, arbiter.clone());
        }
        Ok(self.arbiters.get(&msg.0).unwrap().clone())
    }
}