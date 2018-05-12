// Copyright 2018 OysterPack Inc.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! tests

extern crate actix;
extern crate chrono;
extern crate failure;
extern crate fern;
extern crate futures;
extern crate log;
extern crate oysterpack_id;

use super::*;

use self::actix::{msgs::*, prelude::*};
use self::futures::{future::ok, prelude::*};

use self::oysterpack_id::Oid;

use tests::run_test;

lazy_static! {
    pub static ref ARBITER_ID_1: ArbiterId = ArbiterId::new();
}

#[test]
fn arbiter_ids_can_be_stored_in_lazy_static() {
    fn test() {
        let id = ArbiterId::new();
        info!("id = {}", id);

        info!("ARBITER_ID_1 = {}", *ARBITER_ID_1);

        let id: ArbiterId = *ARBITER_ID_1;
        info!("ARBITER_ID_1 = {:?}", id);
    }
    run_test(test);
}

#[test]
fn arbiter_registry_is_defined_as_system_service() {
    fn test() {
        let mut sys = System::new("sys");
        let system_registry = Arbiter::system_registry();
        let _: Addr<Syn, _> = system_registry.get::<arbiters::Registry>();
        let _ = sys.run_until_complete(ok::<(), ()>(()));
    }
    run_test(test);
}

#[test]
fn arbiters_are_created_on_demand() {
    fn test() {
        let mut sys = System::new("sys");
        // When an Arbiter is looked up with an ArbiterId that is not registered
        let test = super::arbiter(ArbiterId::new()).and_then(|arbiter| {
            // Then the Arbiter's Addr is returned
            // And the Addr is connected
            assert!(arbiter.connected());
            ok(arbiter)
        });
        let result = sys.run_until_complete(test);
        match result {
            Ok(_) => (),
            Err(err) => panic!(err),
        }
    }
    run_test(test);
}

#[test]
fn arbiter() {
    fn test() {
        let mut sys = System::new("sys");
        let test = super::arbiter(*ARBITER_ID_1)
            .map(|arbiter| {
                assert!(arbiter.connected());
                super::arbiter(*ARBITER_ID_1).and_then(|arbiter| {
                    assert!(arbiter.connected());
                    ok(arbiter)
                })
            })
            .flatten();

        let result = sys.run_until_complete(test);
        match result {
            Ok(_) => (),
            Err(err) => panic!(err),
        }
    }
    run_test(test);
}

#[test]
fn stopped_arbiter_is_unregistered_on_demand() {
    fn test() {
        let mut sys = System::new("sys");
        // Given an Arbiter is registered
        let test = super::arbiter(*ARBITER_ID_1)
            .map(|arbiter| {
                assert!(arbiter.connected());
                // When it is stopped
                let _ = arbiter.send(StopArbiter(0)).wait();

                use std::thread::sleep;
                use std::time::Duration;
                while arbiter.connected() {
                    info!("arbiter is still connected ...");
                    sleep(Duration::from_millis(10));
                }
                info!("arbiter is not connected");

                super::arbiter_count()
                    .map(|count| {
                        // Then arbiter is no longer registered - count drops back to 0
                        assert_eq!(count.0, 0);
                        count
                    })
                    .and_then(|_| {
                        // Then new arbiter will be created on demand.
                        super::arbiter(*ARBITER_ID_1).and_then(|arbiter| {
                            assert!(arbiter.connected());
                            ok(arbiter)
                        })
                    })
            })
            .flatten();

        let result = sys.run_until_complete(test);
        match result {
            Ok(_) => (),
            Err(err) => panic!(err),
        }
    }
    run_test(test);
}

#[test]
fn stopped_arbiter_is_replaced_on_demand() {
    fn test() {
        let mut sys = System::new("sys");
        // Given an Arbiter is registered
        let test = super::arbiter(*ARBITER_ID_1)
            .map(|arbiter| {
                assert!(arbiter.connected());
                // When it is stopped
                let _ = arbiter.send(StopArbiter(0)).wait();

                use std::thread::sleep;
                use std::time::Duration;
                while arbiter.connected() {
                    info!("arbiter is still connected ...");
                    sleep(Duration::from_millis(10));
                }
                info!("arbiter is not connected");

                // Then new arbiter will be created on demand.
                super::arbiter(*ARBITER_ID_1).and_then(|arbiter| {
                    assert!(arbiter.connected());
                    ok(arbiter)
                })
            })
            .flatten();

        let result = sys.run_until_complete(test);
        match result {
            Ok(_) => (),
            Err(err) => panic!(err),
        }
    }
    run_test(test);
}

#[test]
fn arbiter_count() {
    fn test() {
        let mut sys = System::new("sys");
        let test = super::arbiter_count()
            .map(|count| {
                info!("arbiter count = {:?}", count);
                assert_eq!(count.0, 0);
                count
            })
            .and_then(|_| {
                super::arbiter(*ARBITER_ID_1).and_then(|_| {
                    info!("arbiter registered");
                    super::arbiter_count().map(|count| {
                        info!("arbiter count = {:?}", count);
                        assert_eq!(count.0, 1);
                        count
                    })
                })
            });
        let result = sys.run_until_complete(test);
        match result {
            Ok(_) => (),
            Err(err) => panic!(err),
        }
    }
    run_test(test);
}

#[test]
fn arbiter_ids() {
    fn test() {
        let mut sys = System::new("sys");
        let test = super::arbiter_ids()
            .map(|arbiter_ids| {
                info!("arbiter_ids = {:?}", arbiter_ids);
                assert_eq!(arbiter_ids.len(), 0);
                arbiter_ids
            })
            .and_then(|_| {
                super::arbiter(*ARBITER_ID_1).and_then(|_| {
                    info!("arbiter registered");
                    super::arbiter_ids().map(|arbiter_ids| {
                        info!("arbiter_ids = {:?}", arbiter_ids);
                        assert_eq!(arbiter_ids.len(), 1);
                        arbiter_ids
                    })
                })
            });
        let result = sys.run_until_complete(test);
        match result {
            Ok(_) => (),
            Err(err) => panic!(err),
        }
    }
    run_test(test);
}

#[test]
fn contains_arbiter() {
    fn test() {
        let mut sys = System::new("sys");

        // When no Arbiter exists for a specified ArbiterId
        let arbiter_id = ArbiterId::new();
        let test = super::contains_arbiter(arbiter_id)
            .map(|contains_arbiter| {
                // Then no Arbiter should be found
                assert_eq!(contains_arbiter, false);
                contains_arbiter
            })
            .and_then(|_| {
                // After the Arbiter is created and registered
                super::arbiter(arbiter_id).and_then(|_| {
                    info!("arbiter registered");
                    super::contains_arbiter(arbiter_id).map(|contains_arbiter| {
                        // Then Arbiter will be found
                        assert_eq!(contains_arbiter, true);
                        contains_arbiter
                    })
                })
            });
        let result = sys.run_until_complete(test);
        match result {
            Ok(_) => (),
            Err(err) => panic!(err),
        }
    }
    run_test(test);
}

#[test]
fn register_actor_by_type() {
    struct Foo;

    impl Actor for Foo {
        type Context = Context<Self>;
    }

    impl<I: Send, E: Send> Handler<actix::msgs::Execute<I, E>> for Foo {
        type Result = Result<I, E>;

        fn handle(&mut self, msg: Execute<I, E>, _: &mut Context<Self>) -> Result<I, E> {
            msg.exec()
        }
    }

    fn test() {
        let mut sys = System::new("sys");
        // When no Arbiter exists for a specified ArbiterId
        let arbiter_id = ArbiterId::new();
        let test = super::register_actor_by_type(arbiter_id, |_| Foo).and_then(|foo| {
            foo.send(actix::msgs::Execute::new(|| -> Result<String, String> {
                Ok("SUCCESS !!!".to_string())
            })).map_err(
                |err| errors::ActorRegistrationError::MessageDeliveryFailed {
                    mailbox_error: err,
                    message_type: errors::MessageType("actix::msgs::Execute".to_string()),
                    actor_destination: errors::ActorDestination("Foo".to_string()),
                },
            )
        });
        let result = sys.run_until_complete(test);
        match result {
            Ok(msg) => info!("foo result : {:?}", msg),
            Err(err) => panic!(err),
        }
    }

    run_test(test);
}

#[test]
fn register_actor_by_id() {
    struct Foo;

    impl Actor for Foo {
        type Context = Context<Self>;
    }

    impl<I: Send, E: Send> Handler<actix::msgs::Execute<I, E>> for Foo {
        type Result = Result<I, E>;

        fn handle(&mut self, msg: Execute<I, E>, _: &mut Context<Self>) -> Result<I, E> {
            msg.exec()
        }
    }

    fn test() {
        let mut sys = System::new("sys");
        // When no Arbiter exists for a specified ArbiterId
        let arbiter_id = ArbiterId::new();
        let actor_instance_id = ActorInstanceId::new();
        let test =
            super::register_actor_by_id(arbiter_id, actor_instance_id, |_| Foo).and_then(|foo| {
                foo.send(actix::msgs::Execute::new(|| -> Result<String, String> {
                    Ok("SUCCESS !!!".to_string())
                })).map_err(
                    |err| errors::ActorRegistrationError::MessageDeliveryFailed {
                        mailbox_error: err,
                        message_type: errors::MessageType("actix::msgs::Execute".to_string()),
                        actor_destination: errors::ActorDestination("Foo".to_string()),
                    },
                )
            });
        let result = sys.run_until_complete(test);
        match result {
            Ok(msg) => info!("foo result : {:?}", msg),
            Err(err) => panic!(err),
        }
    }

    run_test(test);
}

#[test]
fn register_multiple_actors_by_type() {
    struct Foo;

    impl Actor for Foo {
        type Context = Context<Self>;
    }

    impl<I: Send, E: Send> Handler<actix::msgs::Execute<I, E>> for Foo {
        type Result = Result<I, E>;

        fn handle(&mut self, msg: Execute<I, E>, _: &mut Context<Self>) -> Result<I, E> {
            msg.exec()
        }
    }

    fn test() {
        let mut sys = System::new("sys");
        // When no Arbiter exists for a specified ArbiterId
        let arbiter_id = ArbiterId::new();

        fn register_actor(
            arbiter_id: ArbiterId,
        ) -> Box<Future<Item = Result<String, String>, Error = errors::ActorRegistrationError>>
        {
            Box::new(
                super::register_actor_by_type(arbiter_id, |_| Foo).and_then(|foo| {
                    foo.send(actix::msgs::Execute::new(|| -> Result<String, String> {
                        Ok("SUCCESS !!!".to_string())
                    })).map_err(
                        |err| errors::ActorRegistrationError::MessageDeliveryFailed {
                            mailbox_error: err,
                            message_type: errors::MessageType("actix::msgs::Execute".to_string()),
                            actor_destination: errors::ActorDestination("Foo".to_string()),
                        },
                    )
                }),
            )
        }

        let test = register_actor(arbiter_id).and_then(|_| register_actor(arbiter_id));
        let result = sys.run_until_complete(test);
        match result {
            Err(errors::ActorRegistrationError::ActorAlreadyRegistered) => {
                info!("registration failed as expected with ActorAlreadyRegistered")
            }
            Ok(msg) => panic!(
                "registering multiple actors with the same type on the same arbiter should fail"
            ),
            Err(err) => panic!("failed with unexpected error : {:?}", err),
        }
    }

    run_test(test);
}

#[test]
fn register_multiple_actors_by_type_on_different_arbiters() {
    struct Foo;

    impl Actor for Foo {
        type Context = Context<Self>;
    }

    impl<I: Send, E: Send> Handler<actix::msgs::Execute<I, E>> for Foo {
        type Result = Result<I, E>;

        fn handle(&mut self, msg: Execute<I, E>, _: &mut Context<Self>) -> Result<I, E> {
            msg.exec()
        }
    }

    fn test() {
        let mut sys = System::new("sys");

        fn register_actor(
            arbiter_id: ArbiterId,
        ) -> Box<Future<Item = Result<String, String>, Error = errors::ActorRegistrationError>>
        {
            Box::new(
                super::register_actor_by_type(arbiter_id, |_| Foo).and_then(|foo| {
                    foo.send(actix::msgs::Execute::new(|| -> Result<String, String> {
                        Ok("SUCCESS !!!".to_string())
                    })).map_err(
                        |err| errors::ActorRegistrationError::MessageDeliveryFailed {
                            mailbox_error: err,
                            message_type: errors::MessageType("actix::msgs::Execute".to_string()),
                            actor_destination: errors::ActorDestination("Foo".to_string()),
                        },
                    )
                }),
            )
        }

        let test = register_actor(ArbiterId::new()).and_then(|_| register_actor(ArbiterId::new()));
        let result = sys.run_until_complete(test);
        match result {
            Ok(msg) => info!("{:?}", msg),
            Err(err) => panic!(err),
        }
    }

    run_test(test);
}

#[test]
fn register_multiple_actors_by_id_with_same_id() {
    struct Foo;

    impl Actor for Foo {
        type Context = Context<Self>;
    }

    impl<I: Send, E: Send> Handler<actix::msgs::Execute<I, E>> for Foo {
        type Result = Result<I, E>;

        fn handle(&mut self, msg: Execute<I, E>, _: &mut Context<Self>) -> Result<I, E> {
            msg.exec()
        }
    }

    fn test() {
        let mut sys = System::new("sys");
        // When no Arbiter exists for a specified ArbiterId
        let arbiter_id = ArbiterId::new();
        let actor_instance_id = ActorInstanceId::new();

        fn register_actor(
            arbiter_id: ArbiterId,
            actor_instance_id: ActorInstanceId,
        ) -> Box<Future<Item = Result<String, String>, Error = errors::ActorRegistrationError>>
        {
            Box::new(
                super::register_actor_by_id(arbiter_id, actor_instance_id, |_| Foo).and_then(
                    |foo| {
                        foo.send(actix::msgs::Execute::new(|| -> Result<String, String> {
                            Ok("SUCCESS !!!".to_string())
                        })).map_err(
                            |err| errors::ActorRegistrationError::MessageDeliveryFailed {
                                mailbox_error: err,
                                message_type: errors::MessageType(
                                    "actix::msgs::Execute".to_string(),
                                ),
                                actor_destination: errors::ActorDestination("Foo".to_string()),
                            },
                        )
                    },
                ),
            )
        }

        let test = register_actor(arbiter_id, actor_instance_id)
            .and_then(|_| register_actor(arbiter_id, actor_instance_id));
        let result = sys.run_until_complete(test);
        match result {
            Err(errors::ActorRegistrationError::ActorAlreadyRegistered) => {
                info!("registration failed as expected with ActorAlreadyRegistered")
            }
            Ok(msg) => panic!(
                "registering multiple actors with the same type on the same arbiter should fail"
            ),
            Err(err) => panic!("failed with unexpected error : {:?}", err),
        }
    }

    run_test(test);
}

#[test]
fn register_multiple_actors_by_id_with_unique_id() {
    struct Foo;

    impl Actor for Foo {
        type Context = Context<Self>;
    }

    impl<I: Send, E: Send> Handler<actix::msgs::Execute<I, E>> for Foo {
        type Result = Result<I, E>;

        fn handle(&mut self, msg: Execute<I, E>, _: &mut Context<Self>) -> Result<I, E> {
            msg.exec()
        }
    }

    fn test() {
        let mut sys = System::new("sys");
        // When no Arbiter exists for a specified ArbiterId
        let arbiter_id = ArbiterId::new();

        fn register_actor(
            arbiter_id: ArbiterId,
            actor_instance_id: ActorInstanceId,
        ) -> Box<Future<Item = Result<String, String>, Error = errors::ActorRegistrationError>>
        {
            Box::new(
                super::register_actor_by_id(arbiter_id, actor_instance_id, |_| Foo).and_then(
                    |foo| {
                        foo.send(actix::msgs::Execute::new(|| -> Result<String, String> {
                            Ok("SUCCESS !!!".to_string())
                        })).map_err(
                            |err| errors::ActorRegistrationError::MessageDeliveryFailed {
                                mailbox_error: err,
                                message_type: errors::MessageType(
                                    "actix::msgs::Execute".to_string(),
                                ),
                                actor_destination: errors::ActorDestination("Foo".to_string()),
                            },
                        )
                    },
                ),
            )
        }

        let test = register_actor(arbiter_id, ActorInstanceId::new())
            .and_then(|_| register_actor(arbiter_id, ActorInstanceId::new()));
        let result = sys.run_until_complete(test);
        match result {
            Ok(msg) => info!("{:?}", msg),
            Err(err) => panic!(err),
        }
    }

    run_test(test);
}

#[test]
fn register_actors_on_separate_arbiters() {
    struct OpActor<State: Default> {
        id: Oid,
        created: chrono::DateTime<chrono::Utc>,
        state: State,
    }

    impl<State: Default> Default for OpActor<State> {
        fn default() -> Self {
            OpActor {
                id: Oid::new(),
                created: chrono::Utc::now(),
                state: Default::default(),
            }
        }
    }

    #[derive(Clone)]
    struct Command {
        reply_to: Recipient<Syn, Command>,
        action: Action,
        sender: Oid,
    };

    #[derive(Debug, Copy, Clone)]
    enum Action {
        SystemExit,
        Ping,
    }

    impl Message for Command {
        type Result = ();
    }

    type FooActor = OpActor<Foo>;

    struct Foo {
        msg_count: u8,
    };

    impl Default for Foo {
        fn default() -> Self {
            Foo { msg_count: 0 }
        }
    }

    impl Actor for FooActor {
        type Context = Context<Self>;

        fn started(&mut self, _: &mut Self::Context) {
            info!("[Foo] started : {:?}", self.created);
        }

        fn stopping(&mut self, _: &mut Self::Context) -> Running {
            info!("[Foo] stopping");
            Running::Stop
        }

        fn stopped(&mut self, _: &mut Self::Context) {
            info!("[Foo] stopped");
        }
    }

    impl Handler<Command> for FooActor {
        type Result = ();

        fn handle(&mut self, cmd: Command, ctx: &mut Self::Context) -> Self::Result {
            self.state.msg_count += 1;
            info!(
                "[Foo] msg #{} : {:?} from {:?}",
                self.state.msg_count, cmd.action, cmd.sender
            );
            if self.state.msg_count >= 3 {
                let _ = cmd.reply_to.try_send(Command {
                    reply_to: ctx.address::<Addr<Syn, FooActor>>().recipient(),
                    action: Action::SystemExit,
                    sender: self.id,
                });
            } else {
                let _ = cmd.reply_to.try_send(Command {
                    reply_to: ctx.address::<Addr<Syn, FooActor>>().recipient(),
                    action: cmd.action,
                    sender: self.id,
                });
            }

            ()
        }
    }

    struct Bar {
        id: Oid,
        actor: Recipient<Syn, Command>,
    }

    impl Actor for Bar {
        type Context = Context<Self>;

        fn started(&mut self, _: &mut Self::Context) {
            info!("[Bar] started");
        }

        fn stopping(&mut self, _: &mut Self::Context) -> Running {
            info!("[Bar] stopping");
            Running::Stop
        }

        fn stopped(&mut self, _: &mut Self::Context) {
            info!("[Bar] stopped");
        }
    }

    struct Run;

    impl Message for Run {
        type Result = Result<(), ()>;
    }

    impl Handler<Run> for Bar {
        type Result = Result<(), ()>;

        fn handle(&mut self, _: Run, _: &mut Self::Context) -> Self::Result {
            struct ActionHandler {
                id: Oid,
            };

            impl Actor for ActionHandler {
                type Context = Context<Self>;

                fn started(&mut self, _: &mut Self::Context) {
                    info!("[ActionHandler] started");
                }

                fn stopping(&mut self, _: &mut Self::Context) -> Running {
                    info!("[ActionHandler] stopping");
                    Running::Stop
                }

                fn stopped(&mut self, _: &mut Self::Context) {
                    info!("[ActionHandler] stopped");
                }
            }

            impl Handler<Command> for ActionHandler {
                type Result = ();

                fn handle(&mut self, cmd: Command, ctx: &mut Self::Context) -> Self::Result {
                    info!("[ActionHandler] {:?} from {:?}", cmd.action, cmd.sender);
                    match cmd.action {
                        Action::Ping => {
                            info!("[ActionHandler] received Ping back");
                            let _ = cmd.reply_to.try_send(Command {
                                reply_to: ctx.address::<Addr<Syn, ActionHandler>>().recipient(),
                                action: cmd.action,
                                sender: self.id,
                            });
                        }
                        Action::SystemExit => {
                            info!("[ActionHandler] terminating actor system ...");
                            let _ = Arbiter::system().do_send(actix::msgs::SystemExit(0));
                            info!("[ActionHandler] sent actix::msgs::SystemExit(0)");
                        }
                    }
                    ()
                }
            }

            let reply_to = Arbiter::start(|_| ActionHandler { id: Oid::new() });
            let reply_to: Recipient<Syn, Command> = reply_to.recipient();

            let _ = self.actor.try_send(Command {
                reply_to: reply_to,
                sender: self.id,
                action: Action::Ping {},
            });
            info!("[Bar] sent initial ping");
            Ok(())
        }
    }

    fn test() {
        let sys = System::new("sys");

        let task = super::arbiter(ArbiterId::new())
            .and_then(|arbiter| {
                info!("Foo Arbiter has been registered");
                let request = arbiter.send(StartActor::new(|_| FooActor::default()));
                info!("Starting Foo actor");
                request
            })
            .and_then(|foo| {
                info!("Foo actor has been started");
                info!("Registering Bar Arbiter ...");
                super::arbiter(ArbiterId::new())
                    .and_then(|arbiter| {
                        info!("Bar Arbiter has been registered");
                        let request = arbiter.send(StartActor::new(move |_| Bar {
                            actor: foo.recipient(),
                            id: Oid::new(),
                        }));
                        info!("Starting Boo actor");
                        request
                    })
                    .and_then(|bar| {
                        info!("Bar actor has been started");
                        let request = bar.send(Run);
                        info!("Sent Run message to Bar");
                        request
                    })
            });

        Arbiter::handle().spawn(task.map(|_| ()).map_err(|_| ()));

        sys.run();
    }

    run_test(test);
}

#[test]
fn response_future_example() {
    struct Foo;

    impl Actor for Foo {
        type Context = Context<Self>;
    }

    struct FooRequest;

    struct FooResponse;

    impl Message for FooRequest {
        type Result = Result<FooResponse, ()>;
    }

    impl Handler<FooRequest> for Foo {
        type Result = ResponseFuture<FooResponse, ()>;

        fn handle(&mut self, _: FooRequest, _: &mut Self::Context) -> Self::Result {
            Box::new(future::ok(FooResponse))
        }
    }
}
