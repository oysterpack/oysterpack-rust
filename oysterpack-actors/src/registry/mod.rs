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

use std::collections::HashMap;
use actor::ActorMessageResponse;
use self::arbiters::*;

/// Unique Arbiter id.
///
/// ArbiterId(s) can be defined as static constansts leveraging the [lazy_static](https://docs.rs/crate/lazy_static).
pub type ArbiterId = Id<Arbiter>;

/// Type alias for an Arbiter Addr
pub type ArbiterAddr = Addr<Syn, Arbiter>;

/// Looks up an Arbiter address. If one does not exist for the specified id, then a new one is created and registered on demand.
/// If the registered Arbiter addr is not connected, then a new Arbiter will be created to take its place.
pub fn arbiter(id: ArbiterId) -> ActorMessageResponse<ArbiterAddr> {
    let service = Arbiter::system_registry().get::<ArbiterRegistry>();
    let request = service.send(GetArbiter(id)).map(|result| result.unwrap());
    Box::new(request)
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ArbiterCount(pub usize);

/// Returns the number of registered Arbiters
pub fn arbiter_count() -> ActorMessageResponse<ArbiterCount> {
    Box::new(arbiter_ids().map(|ids| ArbiterCount(ids.len())))
}

/// Returns the number of registered Arbiters
pub fn arbiter_ids() -> ActorMessageResponse<Vec<ArbiterId>> {
    let service = Arbiter::system_registry().get::<ArbiterRegistry>();
    let request = service.send(GetArbiterIds).map(|result| result.unwrap());
    Box::new(request)
}

mod arbiters {
    use super::*;

    /// Type alias used for Result Error types that should never result in an Error
    type Never = ();

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

    #[derive(Debug)]
    pub struct GetArbiter(pub ArbiterId);

    impl Message for GetArbiter {
        type Result = Result<ArbiterAddr, Never>;
    }

    impl Handler<GetArbiter> for ArbiterRegistry {
        type Result = Result<ArbiterAddr, Never>;

        fn handle(&mut self, msg: GetArbiter, _: &mut Self::Context) -> Self::Result {
            let arbiter = self.arbiters
                .entry(msg.0)
                .or_insert_with(|| Arbiter::new(msg.0.to_string()));
            if !arbiter.connected() {
                *arbiter = Arbiter::new(msg.0.to_string());
            }
            Ok(arbiter.clone())
        }
    }

    #[derive(Debug)]
    pub struct GetArbiterIds;

    impl Message for GetArbiterIds {
        type Result = Result<Vec<ArbiterId>, Never>;
    }

    impl Handler<GetArbiterIds> for ArbiterRegistry {
        type Result = Result<Vec<ArbiterId>, Never>;

        fn handle(&mut self, _: GetArbiterIds, _: &mut Self::Context) -> Self::Result {
            let mut ids = Vec::with_capacity(self.arbiters.len());
            let mut disconnected = vec![];
            for (id, addr) in &self.arbiters {
                if addr.connected() {
                    ids.push(*id);
                } else {
                    disconnected.push(*id);
                }
            }

            for ref id in disconnected {
                self.arbiters.remove(id);
            }
            Ok(ids)
        }
    }
}
