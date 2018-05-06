// Copyright 2018 OysterPack Inc.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! tests

extern crate oysterpack_platform;
extern crate polymap;
extern crate rmp_serde as rmps;
extern crate semver;
extern crate serde;

use self::polymap::{PolyMap, TypeMap};

use super::*;
use super::service::*;

use self::serde::{Deserialize, Serialize};
use self::rmps::{Deserializer, Serializer};
use self::oysterpack_platform::*;
use super::actix::msgs::*;

use tests::run_test;

use registry::*;

#[derive(Serialize, Deserialize, Debug)]
struct FooState {
    counter: usize,
}

#[test]
fn stateless_actor_service() {
    struct Echo {
        msg: String,
    }

    impl Message for Echo {
        type Result = String;
    }

    struct Foo;

    type FooActor = service::StatelessServiceActor<Foo>;

    impl Handler<Echo> for FooActor {
        type Result = String;

        fn handle(&mut self, msg: Echo, _: &mut Self::Context) -> Self::Result {
            debug!("{} {:?}", self.service_instance(), self.context());
            msg.msg
        }
    }

    fn test() {
        let foo_service = Service::new(
            ServiceId::new(),
            ServiceName::new("foo").unwrap(),
            semver::Version::parse("0.0.1").unwrap(),
        );

        let sys = System::new("sys");

        let service: Addr<Syn, _> = FooActor::create(|_| {
            let builder = ServiceActorBuilder::<Foo, Nil, Nil, Nil>::new(foo_service);
            builder.build()
        });
        let task = service
            .send(Echo {
                msg: "Hello".to_string(),
            })
            .and_then(|msg| {
                info!("Received echo back : {}", msg);
                Arbiter::system().send(actix::msgs::SystemExit(0))
            });

        Arbiter::handle().spawn(task.map(|_| ()).map_err(|_| ()));
        sys.run();
    }

    run_test(test);
}

#[test]
fn stateless_actor_service_running_on_arbiter() {
    struct Echo {
        msg: String,
    }

    impl Message for Echo {
        type Result = String;
    }

    struct Foo;

    type FooActor = service::StatelessServiceActor<Foo>;

    impl Handler<Echo> for FooActor {
        type Result = String;

        fn handle(&mut self, msg: Echo, _: &mut Self::Context) -> Self::Result {
            debug!("{} {:?}", self.service_instance(), self.context());
            msg.msg
        }
    }

    fn test() {
        let foo_arbiter_id = ArbiterId::new();

        let foo_service = Service::new(
            ServiceId::new(),
            ServiceName::new("foo").unwrap(),
            semver::Version::parse("0.0.1").unwrap(),
        );

        let sys = System::new("sys");

        let task = arbiter(foo_arbiter_id)
            .and_then(|arbiter| {
                arbiter.send(StartActor::new(|_| {
                    let builder = ServiceActorBuilder::<Foo, Nil, Nil, Nil>::new(foo_service);
                    builder.build()
                }))
            })
            .and_then(|actor| {
                actor.send(Echo {
                    msg: "Hello".to_string(),
                })
            })
            .and_then(|msg| {
                info!("Received echo back : {}", msg);
                Arbiter::system().send(actix::msgs::SystemExit(0))
            });

        Arbiter::handle().spawn(task.map(|_| ()).map_err(|_| ()));
        sys.run();
    }

    run_test(test);
}
