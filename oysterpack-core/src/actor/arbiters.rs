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

//! The Arbiters actor serves as the Arbiter central registry.
//! - Arbiters implements AppService, i.e., it is registered as a SystemService.
//!
//! ## Messages
//! - [GetArbiter](struct.GetArbiter.html)
//! - [GetArbiterNames](struct.GetArbiterNames.html)
//!
//! ```rust
//! # extern crate oysterpack_core;
//! # extern crate actix;
//! # extern crate futures;
//! # use oysterpack_core::actor::arbiters::*;
//! # use actix::{msgs::{Execute, StartActor}, prelude::*, registry::SystemRegistry,spawn,};
//! # use futures::{future, prelude::*};
//! System::run(|| {
//!   let arbiters = System::current().registry().get::<Arbiters>();
//!   let future = arbiters
//!       .send(GetArbiterNames)
//!       .and_then(|names| {
//!           // There are no Arbiters registered
//!           assert!(names.is_none());
//!           let arbiters = System::current().registry().get::<Arbiters>();
//!           // A new Arbiter named "WEB" will be registered and its address will be returned
//!           arbiters.send(GetArbiter::from("WEB"))
//!       }).and_then(|arbiter_addr| {
//!           arbiter_addr.send(actix::msgs::Execute::new(|| -> Result<(), ()> {
//!               let arbiters = System::current().registry().get::<Arbiters>();
//!               let future = arbiters.send(GetArbiterNames).then(|result| {
//!                 // The "WEB" Arbiter is registered
//!                 assert_eq!(result.unwrap().unwrap(),vec![Name("WEB")]);
//!                 future::ok::<(), ()>(())
//!               });
//!               spawn(future);
//!               Ok(())
//!           }))
//!       }).and_then(|_| {
//!           System::current().stop();
//!           Ok(())
//!       }).map_err(|_| ());
//!   spawn(future);
//! });
//! ```

use actix::{
    msgs::Execute, registry::SystemService, Actor, Addr, Arbiter, Context, Handler, MailboxError,
    Message, MessageResult, Supervised, System,
};
use actor::{self, events, AppService, GetServiceInfo, ServiceInfo, DisplayName};
use futures::prelude::*;
use oysterpack_events::Eventful;
use std::{collections::HashMap, fmt};

/// Arbiters ServiceId (01CWSGYS79QQHAE6ZDRKB48F6S)
pub const SERVICE_ID: actor::Id = actor::Id(1865070194757304474174751345022876889);

/// Gets a service from the specified Arbiter
///
/// ## Panics
/// If invoked outside the context of a running actor System.
pub fn service<A: actor::Service>(
    arbiter: Name,
) -> impl Future<Item = Addr<A>, Error = MailboxError> {
    let arbiters = actor::app_service::<Arbiters>();
    arbiters
        .send(GetArbiter::from(arbiter))
        .and_then(|arbiter| {
            arbiter
                .send(Execute::new(|| -> Result<Addr<A>, ()> {
                    Ok(Arbiter::registry().get::<A>())
                })).map(|service| service.unwrap())
        })
}

/// Arbiter registry AppService
#[derive(Debug)]
pub struct Arbiters {
    service_info: ServiceInfo,
    arbiters: HashMap<Name, Addr<Arbiter>>,
}

impl Default for Arbiters {
    fn default() -> Self {
        Arbiters {
            arbiters: HashMap::new(),
            service_info: ServiceInfo::for_new_actor_instance(SERVICE_ID, Self::TYPE),
        }
    }
}

op_actor_service! {
    AppService(Arbiters)
}

impl crate::actor::LifeCycle for Arbiters {}

impl DisplayName for Arbiters {
    fn name() -> &'static str {"Arbiters"}
}

/// Arbiter name
#[derive(Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Clone, Copy)]
pub struct Name(pub &'static str);

impl From<&'static str> for Name {
    fn from(name: &'static str) -> Name {
        Name(name)
    }
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.0)
    }
}

/// GetArbiter request message.
///
/// If an Arbiter with the specified name does not exist, then it will be created.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct GetArbiter {
    name: Name,
}

impl Message for GetArbiter {
    type Result = Addr<Arbiter>;
}

impl GetArbiter {
    /// Name getter
    pub fn name(&self) -> Name {
        self.name
    }
}

impl From<Name> for GetArbiter {
    fn from(name: Name) -> Self {
        GetArbiter { name }
    }
}

impl From<&'static str> for GetArbiter {
    fn from(name: &'static str) -> Self {
        GetArbiter { name: Name(name) }
    }
}

impl Handler<GetArbiter> for Arbiters {
    type Result = MessageResult<GetArbiter>;

    fn handle(&mut self, request: GetArbiter, _: &mut Self::Context) -> Self::Result {
        let arbiter_addr = self
            .arbiters
            .entry(request.name)
            .or_insert_with(|| Arbiter::new(request.name.to_string()));

        MessageResult(arbiter_addr.clone())
    }
}

/// GetArbiterNames request message
#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct GetArbiterNames;

impl Message for GetArbiterNames {
    type Result = Option<Vec<Name>>;
}

impl Handler<GetArbiterNames> for Arbiters {
    type Result = MessageResult<GetArbiterNames>;

    fn handle(&mut self, _: GetArbiterNames, _: &mut Self::Context) -> Self::Result {
        if self.arbiters.is_empty() {
            return MessageResult(None);
        }
        let names = self.arbiters.keys().cloned().collect();
        MessageResult(Some(names))
    }
}

#[allow(warnings)]
#[cfg(test)]
mod tests {
    use super::*;
    use actix::{
        msgs::{Execute, StartActor},
        prelude::*,
        registry::SystemRegistry,
        spawn,
    };
    use actor::Service;
    use futures::{future, prelude::*};
    use tests::run_test;

    fn into_task(f: impl Future) -> impl Future<Item = (), Error = ()> {
        f.map(|_| ()).map_err(|_| ())
    }

    fn spawn_task(future: impl Future + 'static) {
        spawn(into_task(future));
    }

    fn stop_system() {
        warn!("Sending System stop signal ...");
        System::current().stop();
        warn!("System stop signalled");
    }

    struct Foo;

    impl Actor for Foo {
        type Context = Context<Self>;

        fn started(&mut self, ctx: &mut Self::Context) {
            info!("Foo started");
        }

        fn stopped(&mut self, ctx: &mut Self::Context) {
            info!("Foo stopped");
        }
    }

    struct Echo(String);

    impl Message for Echo {
        type Result = String;
    }

    impl Handler<Echo> for Foo {
        type Result = MessageResult<Echo>;

        fn handle(&mut self, request: Echo, _: &mut Self::Context) -> Self::Result {
            MessageResult(request.0.clone())
        }
    }

    #[test]
    fn get_arbiter_names() {
        run_test("arbiters", || {
            System::run(|| {
                let arbiters = System::current().registry().get::<Arbiters>();
                let future = arbiters
                    .send(GetArbiterNames)
                    .and_then(|names| {
                        assert!(names.is_none());
                        let arbiters = System::current().registry().get::<Arbiters>();
                        arbiters.send(GetArbiter::from("WEB"))
                    }).and_then(|arbiter_addr| {
                        arbiter_addr.send(actix::msgs::Execute::new(|| -> Result<(), ()> {
                            let arbiters = System::current().registry().get::<Arbiters>();
                            let future = arbiters.send(GetArbiterNames).then(|result| {
                                assert_eq!(result.unwrap().unwrap(), vec![Name("WEB")]);
                                future::ok::<(), ()>(())
                            });
                            spawn(future);
                            Ok(())
                        }))
                    }).and_then(|_| {
                        System::current().stop();
                        Ok(())
                    }).map_err(|_| ());
                spawn(future);
            });
        });

        run_test("GetArbiterNames - no Arbiters registered", || {
            System::run(|| {
                let arbiters = System::current().registry().get::<Arbiters>();
                let future = arbiters
                    .send(GetArbiterNames)
                    .then(|result| {
                        match result {
                            Ok(None) => info!("There are no Arbiters registered"),
                            Ok(Some(names)) => {
                                panic!("There should be no Arbiters registered: {:?}", names)
                            }
                            Err(e) => panic!("SHOULD NEVER HAPPEN: {}", e),
                        }
                        future::ok::<(), ()>(())
                    }).then(|_| {
                        stop_system();
                        future::ok::<(), ()>(())
                    });
                spawn_task(future);
            });
        });

        run_test("GetArbiterNames - Arbiter registered", || {
            System::run(|| {
                let arbiters = System::current().registry().get::<Arbiters>();
                let future = arbiters
                    .send(GetArbiter::from("A"))
                    .then(|result| match result {
                        Ok(arbiter_addr) => arbiter_addr.send(StartActor::new(|_| Foo)),
                        Err(e) => panic!("SHOULD NEVER HAPPEN: {}", e),
                    }).then(|result| {
                        match result {
                            Ok(foo_addr) => {
                                let addr = foo_addr.clone();
                                foo_addr.send(Echo("CIAO".to_string())).then(|result| {
                                    match result {
                                        Ok(msg) => info!("Echo response: {}", msg),
                                        Err(e) => panic!("SHOULD NEVER HAPPEN: {}", e),
                                    }
                                    // when all references to the Foo actor address get dropped,
                                    // the actor instance is stopped.
                                    future::ok::<_, ()>(addr)
                                })
                            }
                            Err(e) => panic!("SHOULD NEVER HAPPEN: {}", e),
                        }
                    }).then(move |_| arbiters.send(GetArbiterNames))
                    .then(|result| {
                        match result {
                            Ok(None) => panic!("Arbiters should be registered"),
                            Ok(Some(names)) => info!("Registered Arbiters: {:?}", names),
                            Err(e) => panic!("SHOULD NEVER HAPPEN: {}", e),
                        }
                        future::ok::<(), ()>(())
                    }).then(|_| {
                        stop_system();
                        future::ok::<(), ()>(())
                    });

                spawn_task(future);
            });
        });
    }

    #[test]
    fn get_service_info() {
        run_test("GetServiceInfo", || {
            System::run(|| {
                fn get_service_info() -> impl Future {
                    let arbiters = System::current().registry().get::<Arbiters>();
                    arbiters.send(GetServiceInfo).then(|result| {
                        match result {
                            Ok(info) => info!("{}", info),
                            Err(err) => {
                                // this should only happen if the actor has been stopped, but in this case
                                // because the Actor is an ArbiterService, it will run as long as the Arbiter
                                // is running.
                                panic!("SHOULD NEVER HAPPEN: {}", err)
                            }
                        }
                        future::ok::<_, ()>(())
                    })
                }

                let future = get_service_info().then(|_| {
                    System::current().stop();
                    future::ok::<_, ()>(())
                });

                spawn(future);
            });
        });

        run_test("GetServiceInfo - within separate Arbiter", || {
            System::run(|| {
                fn get_service_info(msg: String) -> impl Future {
                    let arbiters = System::current().registry().get::<Arbiters>();
                    arbiters.send(GetServiceInfo).then(move |result| {
                        match result {
                            Ok(info) => info!("{}: {}", info, msg),
                            Err(err) => panic!("SHOULD NEVER HAPPEN {}", err),
                        }
                        future::ok::<_, ()>(())
                    })
                }

                const FRONTEND: Name = Name("FRONTEND");
                let arbiters = System::current().registry().get::<Arbiters>();
                let future = arbiters.send(GetArbiter { name: FRONTEND }).then(|result| {
                    let future = match result {
                        Ok(arbiter) => arbiter.send(Execute::new(|| -> Result<(), ()> {
                            spawn_task(get_service_info("FRONTEND - 1".to_string()));
                            spawn_task(get_service_info("FRONTEND - 2".to_string()));
                            Ok(())
                        })),
                        Err(err) => panic!("GetArbiter failed: {}", err),
                    };
                    spawn_task(future.then(|_| {
                        stop_system();
                        future::ok::<_, ()>(())
                    }));
                    future::ok::<_, ()>(())
                });
                spawn(future);
            });
        });

        run_test("GetServiceInfo - stop system tasks are spawned", || {
            System::run(|| {
                fn get_service_info() -> impl Future {
                    let arbiters = System::current().registry().get::<Arbiters>();
                    arbiters.send(GetServiceInfo).then(|result| {
                        match result {
                            Ok(info) => info!("{}", info),
                            Err(err) => {
                                // this should only happen if the actor has been stopped, but in this case
                                // because the Actor is an ArbiterService, it will run as long as the Arbiter
                                // is running.
                                panic!("SHOULD NEVER HAPPEN: {}", err)
                            }
                        }
                        future::ok::<_, ()>(())
                    })
                }

                for _ in 0..10 {
                    spawn_task(get_service_info());
                }

                let future = get_service_info().then(|_| {
                    stop_system();
                    future::ok::<_, ()>(())
                });

                spawn(future);
            });
        });
    }

    const BAR_ID: actor::Id = actor::Id(1864734280873114327279151769208160280);

    struct Bar {
        service_info: ServiceInfo,
    }

    impl Default for Bar {
        fn default() -> Self {
            Bar {
                service_info: ServiceInfo::for_new_actor_instance(BAR_ID, Self::TYPE),
            }
        }
    }

    op_actor_service! {
        Service(Bar)
    }

    impl actor::LifeCycle for Bar {}

    impl DisplayName for Bar {
        fn name() -> &'static str {"Bar"}
    }

    #[test]
    fn get_service_from_arbiter() {
        System::run(|| {
            let task = service::<Bar>(Name("ARBITER-1")).and_then(|bar_1| {
                service::<Bar>(Name("ARBITER-1")).and_then(move |bar_2| {
                    bar_1
                        .send(GetServiceInfo)
                        .and_then(move |bar_1_service_info| {
                            bar_2
                                .send(GetServiceInfo)
                                .and_then(move |bar_2_service_info| {
                                    assert_eq!(bar_1_service_info, bar_2_service_info);
                                    future::ok::<_, actix::MailboxError>(())
                                })
                        })
                })
            });

            let task = task.then(|_| {
                service::<Bar>(Name("ARBITER-1")).and_then(|bar_1| {
                    service::<Bar>(Name("ARBITER-2")).and_then(move |bar_2| {
                        bar_1
                            .send(GetServiceInfo)
                            .and_then(move |bar_1_service_info| {
                                bar_2
                                    .send(GetServiceInfo)
                                    .and_then(move |bar_2_service_info| {
                                        assert_ne!(bar_1_service_info, bar_2_service_info);
                                        future::ok::<_, actix::MailboxError>(())
                                    })
                            })
                    })
                })
            });

            let task = task.then(|_| {
                System::current().stop();
                future::ok::<_, ()>(())
            });

            spawn_task(task);
        });
    }
}
