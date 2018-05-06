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

/// Registers an actor on the specified Arbiter.
/// - if the Arbiter for the specified ArbiterId does not exist, then it will be created on demand
/// - if actor_instance_id is None, then the actor is registered by type, i.e., only 1 instance of
///   the specified actor type can exist per arbiter. Otherwise, the actor is registered using the
///   specified ActorInstanceId. This allows multiple Actor instances of the same type to be registered.
pub fn register_actor<A, F>(
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

/// marker Id trait - used to define [ActorInstanceId](type.ActorInstanceId.html)
pub trait ActorInstance {}

/// Used to assign an Id to an Actor instance.
pub type ActorInstanceId = Id<ActorInstance + Send>;

/// registry related errors
pub mod errors {
    use super::*;

    /// Types of errors that can occur when registering Actors
    #[derive(Debug, Fail)]
    pub enum ActorRegistrationError {
        #[fail(display = "Actor is already registered.")]
        ActorAlreadyRegistered,
        #[fail(display = "Failed to deliver message [{}] to actor [{}] : {}", message_type,
               actor_destination, mailbox_error)]
        MessageDeliveryFailed {
            #[cause]
            mailbox_error: MailboxError,
            message_type: MessageType,
            actor_destination: ActorDestination,
        },
    }

    ///
    #[derive(Debug)]
    pub struct MessageType(pub String);

    impl fmt::Display for MessageType {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    #[derive(Debug)]
    pub struct ActorDestination(pub String);

    impl fmt::Display for ActorDestination {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl ActorRegistrationError {
        pub fn arbiter_message_delivery_failed(err: MailboxError) -> ActorRegistrationError {
            errors::ActorRegistrationError::MessageDeliveryFailed {
                mailbox_error: err,
                message_type: errors::MessageType("GetArbiter".to_string()),
                actor_destination: errors::ActorDestination("arbiters::Registry".to_string()),
            }
        }

        pub fn start_actor_message_delivery_failed(err: MailboxError) -> ActorRegistrationError {
            errors::ActorRegistrationError::MessageDeliveryFailed {
                mailbox_error: err,
                message_type: errors::MessageType("actix::msgs::StartActor".to_string()),
                actor_destination: errors::ActorDestination("actix::Arbiter".to_string()),
            }
        }

        pub fn register_actor_message_delivery_failed(err: MailboxError) -> ActorRegistrationError {
            errors::ActorRegistrationError::MessageDeliveryFailed {
                mailbox_error: err,
                message_type: errors::MessageType("RegisterActor".to_string()),
                actor_destination: errors::ActorDestination("actors::Registry".to_string()),
            }
        }
    }
}

mod arbiters {
    use super::*;
    use super::errors::*;

    /// Type alias used for Result Error types that should never result in an Error
    type Never = ();

    /// Arbiter registry
    pub(crate) struct Registry {
        arbiters: HashMap<ArbiterId, Addr<Syn, Arbiter>>,
        actors: HashMap<ArbiterId, TypeMap>,
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
                actors: HashMap::new(),
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

    pub(crate) struct SubmitActorRegistration<A: Actor<Context = Context<A>>> {
        arbiter_id: ArbiterId,
        start_actor: actix::msgs::StartActor<A>,
    }

    impl<A: Actor<Context = Context<A>>> Message for SubmitActorRegistration<A> {
        type Result = Result<Addr<Syn, A>, ActorRegistrationError>;
    }

    impl<A: Actor<Context = Context<A>>> Handler<SubmitActorRegistration<A>> for Registry {
        type Result = ResponseFuture<Addr<Syn, A>, ActorRegistrationError>;

        fn handle(
            &mut self,
            msg: SubmitActorRegistration<A>,
            ctx: &mut Self::Context,
        ) -> Self::Result {
            let actors = self.actors
                .entry(msg.arbiter_id)
                .or_insert_with(|| TypeMap::new());

            let result: ResponseFuture<Addr<Syn, A>, ActorRegistrationError> = if let Some(actor) =
                actors.get::<Addr<Syn, A>>()
            {
                Box::new(future::err(ActorRegistrationError::ActorAlreadyRegistered))
            } else {
                let arbiter_id = msg.arbiter_id;
                let self_addr: Addr<Syn, Self> = ctx.address::<Addr<Syn, Self>>().clone();
                let future = arbiter(arbiter_id)
                    .map_err(|err| ActorRegistrationError::MessageDeliveryFailed {
                        mailbox_error: err,
                        message_type: MessageType("GetArbiter".to_string()),
                        actor_destination: ActorDestination("arbiters::Registry".to_string()),
                    })
                    .and_then(|arbiter| {
                        arbiter.send(msg.start_actor).map_err(|err| {
                            ActorRegistrationError::MessageDeliveryFailed {
                                mailbox_error: err,
                                message_type: MessageType("StartActor".to_string()),
                                actor_destination: ActorDestination("actix::Arbiter".to_string()),
                            }
                        })
                    })
                    .and_then(move |actor| {
                        self_addr
                            .send(RegisterActor {
                                arbiter_id: arbiter_id,
                                actor: actor.clone(),
                            })
                            .map_err(|err| ActorRegistrationError::MessageDeliveryFailed {
                                mailbox_error: err,
                                message_type: MessageType("RegisterActor".to_string()),
                                actor_destination: ActorDestination(
                                    "arbiters::Registry".to_string(),
                                ),
                            })
                            .and_then(|result| match result {
                                Ok(addr) => future::ok(addr.clone()),
                                Err(e) => future::err(e),
                            })
                    });
                Box::new(future)
            };
            result
        }
    }

    pub(crate) struct RegisterActor<A: Actor> {
        arbiter_id: ArbiterId,
        actor: Addr<Syn, A>,
    }

    impl<A: Actor> Message for RegisterActor<A> {
        type Result = Result<Addr<Syn, A>, ActorRegistrationError>;
    }

    impl<A: Actor> Handler<RegisterActor<A>> for Registry {
        type Result = Result<Addr<Syn, A>, ActorRegistrationError>;

        fn handle(&mut self, msg: RegisterActor<A>, _: &mut Self::Context) -> Self::Result {
            let actors = self.actors
                .entry(msg.arbiter_id)
                .or_insert_with(|| TypeMap::new());
            match actors.entry::<Addr<Syn, A>>() {
                Entry::Occupied(_) => Err(ActorRegistrationError::ActorAlreadyRegistered),
                Entry::Vacant(entry) => {
                    entry.insert(msg.actor.clone());
                    Ok(msg.actor)
                }
            }
        }
    }
}

mod actors {
    use super::*;
    use super::errors::*;
    use std::sync::Arc;

    pub(crate) struct Registry {
        // ArbiterId -> TypeMap -> ActorRegistryEntry<A>
        actors_by_type: HashMap<ArbiterId, TypeMap>,
        // ActorInstanceId -> ActorRegistryEntry<A>
        actors_by_id: PolyMap<ActorInstanceId>,
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
                actors_by_type: HashMap::new(),
                actors_by_id: PolyMap::new(),
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

    struct ActorRegistryEntry<A: Actor<Context = Context<A>>> {
        // the arbiter that the actor is running on
        arbiter_id: ArbiterId,
        addr: Option<Addr<Syn, A>>,
    }

    impl<A: Actor<Context = Context<A>>> ActorRegistryEntry<A> {
        fn new(arbiter_id: ArbiterId) -> ActorRegistryEntry<A> {
            ActorRegistryEntry {
                arbiter_id: arbiter_id,
                addr: None,
            }
        }
    }

    pub(crate) struct RegisterActor<A: Actor<Context = Context<A>>> {
        arbiter_id: ArbiterId,
        actor_instance_id: Option<ActorInstanceId>,
        start_actor: actix::msgs::StartActor<A>,
    }

    impl<A: Actor<Context = Context<A>>> RegisterActor<A> {
        pub fn new(
            arbiter_id: ArbiterId,
            actor_instance_id: Option<ActorInstanceId>,
            start_actor: actix::msgs::StartActor<A>,
        ) -> RegisterActor<A> {
            RegisterActor {
                arbiter_id,
                actor_instance_id,
                start_actor,
            }
        }
    }

    impl<A: Actor<Context = Context<A>>> Message for RegisterActor<A> {
        type Result = Result<Addr<Syn, A>, ActorRegistrationError>;
    }

    impl<A: Actor<Context = Context<A>>> Handler<RegisterActor<A>> for Registry {
        type Result = ResponseFuture<Addr<Syn, A>, ActorRegistrationError>;

        fn handle(&mut self, msg: RegisterActor<A>, ctx: &mut Self::Context) -> Self::Result {
            // check if already registered
            if let Some(actor_instance_id) = msg.actor_instance_id {
                let entry: Option<&ActorRegistryEntry<A>> =
                    self.actors_by_id.get(&actor_instance_id);
                if entry.is_some() {
                    return Box::new(future::err(ActorRegistrationError::ActorAlreadyRegistered));
                }
            } else {
                if let Some(arbiter_actors) = self.actors_by_type.get(&msg.arbiter_id) {
                    if arbiter_actors.get::<ActorRegistryEntry<A>>().is_some() {
                        return Box::new(future::err(
                            ActorRegistrationError::ActorAlreadyRegistered,
                        ));
                    }
                }
            }

            let arbiter_id = msg.arbiter_id;
            let actor_instance_id = msg.actor_instance_id;
            let on_error_addr: Addr<Syn, Self> = ctx.address::<Addr<Syn, Self>>().clone();
            let on_success_addr: Addr<Syn, Self> = ctx.address::<Addr<Syn, Self>>().clone();

            // create the future for starting the actor
            let future = arbiter(msg.arbiter_id)
                .map_err(|err| ActorRegistrationError::arbiter_message_delivery_failed(err))
                .and_then(|arbiter| {
                    arbiter.send(msg.start_actor).map_err(|err| {
                        ActorRegistrationError::start_actor_message_delivery_failed(err)
                    })
                })
                .map_err(move |err| {
                    let addr: Option<Addr<Syn, A>> = None;
                    on_error_addr.send(ActorRegistrationResult {
                        arbiter_id,
                        actor_instance_id: actor_instance_id.clone(),
                        addr,
                    });
                    err
                })
                .map(move |addr| {
                    on_success_addr.send(ActorRegistrationResult {
                        arbiter_id,
                        actor_instance_id: actor_instance_id.clone(),
                        addr: Some(addr.clone()),
                    });
                    addr
                });

            // register the entry
            let entry = ActorRegistryEntry::<A>::new(arbiter_id);

            match actor_instance_id {
                Some(actor_instance_id) => {
                    self.actors_by_id.insert(actor_instance_id, entry);
                }
                None => {
                    let actors = self.actors_by_type
                        .entry(arbiter_id)
                        .or_insert_with(|| TypeMap::new());
                    actors.insert(entry);
                }
            }

            Box::new(future)
        }
    }

    struct ActorRegistrationResult<A: Actor<Context = Context<A>>> {
        arbiter_id: ArbiterId,
        actor_instance_id: Option<ActorInstanceId>,
        // Some = success
        // None = failure
        addr: Option<Addr<Syn, A>>,
    }

    impl<A: Actor<Context = Context<A>>> Message for ActorRegistrationResult<A> {
        type Result = Result<(), ()>;
    }

    impl<A: Actor<Context = Context<A>>> Handler<ActorRegistrationResult<A>> for Registry {
        type Result = Result<(), ()>;

        fn handle(
            &mut self,
            msg: ActorRegistrationResult<A>,
            ctx: &mut Self::Context,
        ) -> Self::Result {
            match msg.addr {
                Some(ref addr) => match msg.actor_instance_id {
                    Some(actor_instance_id) => {
                        self.actors_by_id.insert(
                            actor_instance_id,
                            ActorRegistryEntry {
                                arbiter_id: msg.arbiter_id,
                                addr: Some(addr.clone()),
                            },
                        );
                    }
                    None => {
                        self.actors_by_type.get_mut(&msg.arbiter_id).map(|actors| {
                            actors.insert(ActorRegistryEntry {
                                arbiter_id: msg.arbiter_id,
                                addr: Some(addr.clone()),
                            })
                        });
                    }
                },
                None => match msg.actor_instance_id {
                    Some(actor_instance_id) => {
                        self.actors_by_id
                            .remove::<ActorInstanceId, ActorRegistryEntry<A>>(&actor_instance_id);
                    }
                    None => {
                        self.actors_by_type
                            .get_mut(&msg.arbiter_id)
                            .map(|actors| actors.remove::<ActorRegistryEntry<A>>());
                    }
                },
            }
            Ok(())
        }
    }

}
