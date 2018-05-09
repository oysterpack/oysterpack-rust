// Copyright 2018 OysterPack Inc.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Actors registry

extern crate actix;
extern crate futures;
extern crate polymap;

use std::{collections::{HashMap, hash_map::Entry}, marker::PhantomData};

use self::actix::{msgs::StartActor, prelude::*};

use self::polymap::{PolyMap, TypeMap};
use self::futures::{future, prelude::*};

use super::{arbiter, ActorInstanceId, ArbiterId, errors::*};

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
    // if None, then this indicates that the registration is in progress
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

// ///////////////////
// UnregisterActor //
// /////////////////
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
            let entry: Option<&ActorRegistryEntry<A>> = self.actors_by_id.get(&actor_instance_id);
            if entry.is_some() {
                return Box::new(future::err(ActorRegistrationError::ActorAlreadyRegistered));
            }
        } else {
            if let Some(arbiter_actors) = self.actors_by_type.get(&msg.arbiter_id) {
                if arbiter_actors.get::<ActorRegistryEntry<A>>().is_some() {
                    return Box::new(future::err(ActorRegistrationError::ActorAlreadyRegistered));
                }
            }
        }

        let arbiter_id = msg.arbiter_id;
        let actor_instance_id = msg.actor_instance_id;
        let self_addr = ctx.address::<Addr<Unsync, Self>>().clone();

        // create the future for starting the actor
        let future = arbiter(msg.arbiter_id)
            .map_err(|err| ActorRegistrationError::arbiter_message_delivery_failed(err))
            .and_then(|arbiter| {
                arbiter
                    .send(msg.start_actor)
                    .map_err(|err| ActorRegistrationError::start_actor_message_delivery_failed(err))
            })
            .then(move |result| {
                // update the registry entry

                let future = match result {
                    Ok(ref addr) =>
                    // update the registry with the actors address
                        Box::new(
                        self_addr
                            .send(UpdateActor {
                                arbiter_id,
                                actor_instance_id: actor_instance_id,
                                addr: addr.clone(),
                            })
                            .map_err(|err| {
                                ActorRegistrationError::update_actor_message_delivery_failed(err)
                            }),
                    )
                        as Box<Future<Item = Result<(), ()>, Error = ActorRegistrationError>>,
                    Err(_) => {
                        // unregister the actor, i.e., remove the registry entry
                        Box::new(
                            self_addr
                                .send(UnregisterActor::<A> {
                                    arbiter_id,
                                    actor_instance_id: actor_instance_id,
                                    _type: PhantomData,
                                })
                                .map_err(|err| {
                                    ActorRegistrationError::unregister_actor_message_delivery_failed(err)
                                }),
                        )
                            as Box<Future<Item = Result<(), ()>, Error = ActorRegistrationError>>
                    }
                };

                future.then(|result2| match result2 {
                    Ok(_) => Box::new(future::result(result))
                        as Box<Future<Item = Addr<Syn, A>, Error = ActorRegistrationError>>,
                    Err(err) => Box::new(future::result(result).then(|_| future::err(err)))
                        as Box<Future<Item = Addr<Syn, A>, Error = ActorRegistrationError>>,
                })
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

// ///////////////
// UpdateActor //
// /////////////
pub(crate) struct UpdateActor<A: Actor<Context = Context<A>>> {
    arbiter_id: ArbiterId,
    actor_instance_id: Option<ActorInstanceId>,
    addr: Addr<Syn, A>,
}

impl<A: Actor<Context = Context<A>>> Message for UpdateActor<A> {
    type Result = Result<(), ()>;
}

impl<A: Actor<Context = Context<A>>> Handler<UpdateActor<A>> for Registry {
    type Result = Result<(), ()>;

    fn handle(&mut self, msg: UpdateActor<A>, ctx: &mut Self::Context) -> Self::Result {
        match msg.actor_instance_id {
            Some(actor_instance_id) => {
                self.actors_by_id.insert(
                    actor_instance_id,
                    ActorRegistryEntry {
                        arbiter_id: msg.arbiter_id,
                        addr: Some(msg.addr),
                    },
                );
            }
            None => {
                self.actors_by_type.get_mut(&msg.arbiter_id).map(|actors| {
                    actors.insert(ActorRegistryEntry {
                        arbiter_id: msg.arbiter_id,
                        addr: Some(msg.addr),
                    })
                });
            }
        }
        Ok(())
    }
}

// ///////////////////
// UnregisterActor //
// /////////////////
pub(crate) struct UnregisterActor<A: Actor<Context = Context<A>>> {
    arbiter_id: ArbiterId,
    actor_instance_id: Option<ActorInstanceId>,
    _type: PhantomData<A>,
}

impl<A: Actor<Context = Context<A>>> Message for UnregisterActor<A> {
    type Result = Result<(), ()>;
}

impl<A: Actor<Context = Context<A>>> Handler<UnregisterActor<A>> for Registry {
    type Result = Result<(), ()>;

    fn handle(&mut self, msg: UnregisterActor<A>, ctx: &mut Self::Context) -> Self::Result {
        match msg.actor_instance_id {
            Some(actor_instance_id) => {
                self.actors_by_id
                    .remove::<ActorInstanceId, ActorRegistryEntry<A>>(&actor_instance_id);
            }
            None => {
                self.actors_by_type
                    .get_mut(&msg.arbiter_id)
                    .map(|actors| actors.remove::<ActorRegistryEntry<A>>());
            }
        }
        Ok(())
    }
}
