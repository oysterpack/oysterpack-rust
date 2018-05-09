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
extern crate failure;
extern crate futures;
extern crate oysterpack_id;
extern crate polymap;

use self::actix::prelude::*;
use self::futures::{future, prelude::*};

use std::{fmt, collections::HashMap};

use self::polymap::{PolyMap, TypeMap, typemap::Entry};

use self::oysterpack_id::Id;
use actor::MessageProcessingResult;

/// registry related errors
pub mod errors;
mod arbiters;
mod actors;

/// Unique Arbiter id.
///
/// ArbiterId(s) can be defined as static constants leveraging the [lazy_static](https://docs.rs/crate/lazy_static).
pub type ArbiterId = Id<Arbiter>;

/// Type alias for an Arbiter Addr
pub type ArbiterAddr = Addr<Syn, Arbiter>;

/// Looks up an Arbiter address. If one does not exist for the specified id, then a new one is created and registered on demand.
/// If the registered Arbiter addr is not connected, then a new Arbiter will be created to take its place.
pub fn arbiter(id: ArbiterId) -> MessageProcessingResult<ArbiterAddr> {
    let service = Arbiter::system_registry().get::<arbiters::Registry>();
    let request = service
        .send(arbiters::GetArbiter(id))
        .map(|result| result.unwrap());
    Box::new(request)
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ArbiterCount(pub usize);

/// Returns the number of registered Arbiters
pub fn arbiter_count() -> MessageProcessingResult<ArbiterCount> {
    Box::new(arbiter_ids().map(|ids| ArbiterCount(ids.len())))
}

/// Returns the number of registered Arbiters
pub fn arbiter_ids() -> MessageProcessingResult<Vec<ArbiterId>> {
    let service = Arbiter::system_registry().get::<arbiters::Registry>();
    let request = service
        .send(arbiters::GetArbiterIds)
        .map(|result| result.unwrap());
    Box::new(request)
}

/// Looks up an Arbiter address. If one does not exist for the specified id, then a new one is created and registered on demand.
/// If the registered Arbiter addr is not connected, then a new Arbiter will be created to take its place.
pub fn contains_arbiter(id: ArbiterId) -> MessageProcessingResult<bool> {
    let service = Arbiter::system_registry().get::<arbiters::Registry>();
    let request = service
        .send(arbiters::ContainsArbiter(id))
        .map(|result| result.unwrap());
    Box::new(request)
}

/// marker Id trait - used to define [ActorInstanceId](type.ActorInstanceId.html)
pub trait ActorInstance {}

/// Used to assign an Id to an Actor instance.
pub type ActorInstanceId = Id<ActorInstance + Send>;

/// Registers an actor on the specified Arbiter using the specified ActorInstanceId
/// - if the Arbiter for the specified ArbiterId does not exist, then it will be created on demand
pub fn register_actor_by_id<A, F>(
    arbiter_id: ArbiterId,
    actor_instance_id: ActorInstanceId,
    actor: F,
) -> Box<Future<Item = Addr<Syn, A>, Error = errors::ActorRegistrationError>>
where
    A: Actor<Context = Context<A>>,
    F: FnOnce(&mut Context<A>) -> A + Send + 'static,
{
    register_actor(arbiter_id, Some(actor_instance_id), actor)
}

/// Registers an actor on the specified Arbiter using the Actor's type as the registry key.
/// - only 1 instance of the Actor type can be registered per Arbiter
/// - if the Arbiter for the specified ArbiterId does not exist, then it will be created on demand
pub fn register_actor_by_type<A, F>(
    arbiter_id: ArbiterId,
    actor: F,
) -> Box<Future<Item = Addr<Syn, A>, Error = errors::ActorRegistrationError>>
where
    A: Actor<Context = Context<A>>,
    F: FnOnce(&mut Context<A>) -> A + Send + 'static,
{
    register_actor(arbiter_id, None, actor)
}

fn register_actor<A, F>(
    arbiter_id: ArbiterId,
    actor_instance_id: Option<ActorInstanceId>,
    actor: F,
) -> Box<Future<Item = Addr<Syn, A>, Error = errors::ActorRegistrationError>>
where
    A: Actor<Context = Context<A>>,
    F: FnOnce(&mut Context<A>) -> A + Send + 'static,
{
    let service = Arbiter::system_registry().get::<actors::Registry>();
    let request = service
        .send(actors::RegisterActor::new(
            arbiter_id,
            actor_instance_id,
            actix::msgs::StartActor::new(actor),
        ))
        .map_err(|err| errors::ActorRegistrationError::register_actor_message_delivery_failed(err))
        .and_then(|result| match result {
            Ok(addr) => future::ok(addr),
            Err(err) => future::err(err),
        });
    Box::new(request)
}
