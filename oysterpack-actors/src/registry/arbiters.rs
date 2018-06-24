// Copyright 2018 OysterPack Inc.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Arbiters registry

extern crate actix;
extern crate futures;
extern crate oysterpack_id;

use self::actix::prelude::*;
use self::futures::{future, prelude::*};

use super::{errors::*, ArbiterAddr, ArbiterId};

use self::oysterpack_id::Id;

use std::collections::HashMap;

/// Type alias used for Result Error types that should never result in an Error
type Never = ();

/// Arbiter registry
pub(crate) struct Registry {
    arbiters: HashMap<ArbiterId, Addr<Syn, Arbiter>>,
}

impl Supervised for Registry {}

impl SystemService for Registry {
    fn service_started(&mut self, _: &mut Context<Self>) {
        debug!("service started");
    }
}

impl Default for Registry {
    fn default() -> Self {
        Registry {
            arbiters: HashMap::new(),
        }
    }
}

impl Actor for Registry {
    type Context = Context<Self>;

    fn started(&mut self, _: &mut Self::Context) {
        debug!("started");
    }

    fn stopped(&mut self, _: &mut Self::Context) {
        debug!("stopped");
    }
}

#[derive(Debug)]
pub(crate) struct GetArbiter(pub ArbiterId);

impl Message for GetArbiter {
    type Result = Result<ArbiterAddr, Never>;
}

impl Handler<GetArbiter> for Registry {
    type Result = Result<ArbiterAddr, Never>;

    fn handle(&mut self, msg: GetArbiter, _: &mut Self::Context) -> Self::Result {
        let arbiter_id = msg.0;
        let arbiter = self.arbiters
            .entry(arbiter_id)
            .or_insert_with(|| Arbiter::new(arbiter_id.to_string()));
        if !arbiter.connected() {
            *arbiter = Arbiter::new(arbiter_id.to_string());
        }
        Ok(arbiter.clone())
    }
}

#[derive(Debug)]
pub(crate) struct ContainsArbiter(pub ArbiterId);

impl Message for ContainsArbiter {
    type Result = Result<bool, Never>;
}

impl Handler<ContainsArbiter> for Registry {
    type Result = Result<bool, Never>;

    fn handle(&mut self, msg: ContainsArbiter, _: &mut Self::Context) -> Self::Result {
        Ok(self.arbiters.contains_key(&msg.0))
    }
}

#[derive(Debug)]
pub(crate) struct GetArbiterIds;

impl Message for GetArbiterIds {
    type Result = Result<Vec<ArbiterId>, Never>;
}

impl Handler<GetArbiterIds> for Registry {
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
