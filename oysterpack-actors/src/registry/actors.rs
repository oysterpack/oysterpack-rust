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

use std::collections::{HashMap, hash_map::Entry};

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
        let self_addr: Addr<Syn, Self> = ctx.address::<Addr<Syn, Self>>().clone();
                let on_error_addr: Addr<Syn, Self> = ctx.address::<Addr<Syn, Self>>().clone();
                let on_success_addr: Addr<Syn, Self> = ctx.address::<Addr<Syn, Self>>().clone();

        // create the future for starting the actor
        let future = arbiter(msg.arbiter_id)
            .map_err(|err| ActorRegistrationError::arbiter_message_delivery_failed(err))
            .and_then(|arbiter| {
                arbiter
                    .send(msg.start_actor)
                    .map_err(|err| ActorRegistrationError::start_actor_message_delivery_failed(err))
            })
            .then(move|result| {
                let addr = match result {
                    Ok(ref addr) => Some(addr.clone()),
                    Err(_) => None
                };

                self_addr.send(ActorRegistrationResult {
                    arbiter_id,
                    actor_instance_id: actor_instance_id.clone(),
                    addr,
                }).map_err(|err| {
                    ActorRegistrationError::actor_registration_result_message_delivery_failed(err)
                }).then(|actor_registration_result| {
                    match actor_registration_result {
                        Ok(_) => Box::new(future::result(result)) as Box<Future<Item=Addr<Syn,A>, Error=ActorRegistrationError>>,
                        Err(err) => {
                            Box::new(future::result(result)
                                .then(|_| future::err(err)))
                                as Box<Future<Item=Addr<Syn,A>, Error=ActorRegistrationError>>
                        }
                    }
                })

//                future::result(result)

//                match result {
//                    Ok(addr) => {
//                        let future = self_addr.send(ActorRegistrationResult {
//                            arbiter_id,
//                            actor_instance_id: actor_instance_id.clone(),
//                            addr: Some(addr.clone()),
//                        }).map_err(|err| {
//                            ActorRegistrationError::actor_registration_result_message_delivery_failed(err)
//                        }).then(|actor_registration_result| {
//                            if result.is_err() {
//                                future::err(result.err().unwrap())
//                            } else {
//                                future::ok(addr.clone())
//                            }
//                        });
//                        future
//                    },
//                    Err(err) => {
//                        let future = self_addr.send(ActorRegistrationResult {
//                            arbiter_id,
//                            actor_instance_id: actor_instance_id.clone(),
//                            addr: None,
//                        }).map_err(|err| {
//                            ActorRegistrationError::actor_registration_result_message_delivery_failed(err)
//                        }).then(|actor_registration_result| {
//                            match actor_registration_result {
//                                Ok(_) => future::result(result),
//                                Err(err) => {
//                                    future::result(result).map_err(|_| err)
//                                }
//                            }
//                        });
//                        future
//                    }
//                }

//                let future = self_addr.send(ActorRegistrationResult {
//                    arbiter_id,
//                    actor_instance_id: actor_instance_id.clone(),
//                    addr,
//                })
//                .map_err(|err| {
//                    ActorRegistrationError::actor_registration_result_message_delivery_failed(err)
//                })
//                .then(|actor_registration_result| {
//                    match actor_registration_result {
//                        Ok(_) => {
//
//                        },
//                        Err(err) => future::err(err)
//                    }
//                });
//                future
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

    fn handle(&mut self, msg: ActorRegistrationResult<A>, ctx: &mut Self::Context) -> Self::Result {
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
