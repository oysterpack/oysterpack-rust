// Copyright 2018 OysterPack Inc.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! POC tests

extern crate actix;
extern crate chrono;
extern crate futures;

use self::actix::prelude::*;
use self::actix::msgs::*;
use self::actix::registry::*;
use self::chrono::prelude::*;
use self::futures::prelude::*;
use self::futures::future::ok;

#[test]
fn multiple_systems() {
    use std::thread;
    use std::time;

    struct Timer {
        dur: time::Duration,
    }

    impl actix::Actor for Timer {
        type Context = actix::Context<Self>;

        // stop system after `self.dur` seconds
        fn started(&mut self, ctx: &mut actix::Context<Self>) {
            ctx.run_later(self.dur, |act, _ctx| {
                // send `SystemExit` to `System` actor.
                actix::Arbiter::system().do_send(actix::msgs::SystemExit(0));
            });
        }
    }

    let mgmt = thread::spawn(|| {
        let mgmt = actix::System::new("mgmt");

        let _: () = Timer {
            dur: time::Duration::new(0, 1),
        }.start();
        println!("mgmt return code = {}", mgmt.run());
    });

    let app = thread::spawn(|| {
        let app = actix::System::new("app");

        let _: () = Timer {
            dur: time::Duration::new(0, 1),
        }.start();
        println!("app return code = {}", app.run());
    });

    let _ = mgmt.join();
    let _ = app.join();
}

#[test]
fn multiple_arbiters() {
    struct Heartbeat {
        id: String,
    };

    impl Actor for Heartbeat {
        type Context = Context<Self>;

        fn started(&mut self, ctx: &mut Self::Context) {
            println!(
                "*** started : {} : Heartbeat state : {:?}",
                self.id,
                ctx.state()
            );
        }

        fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
            println!(
                "*** stopping : {} : Heartbeat state : {:?}",
                self.id,
                ctx.state()
            );
            Running::Stop
        }

        fn stopped(&mut self, ctx: &mut Self::Context) {
            println!(
                "*** stopped: {} : Heartbeat state : {:?}",
                self.id,
                ctx.state()
            );
        }
    }

    impl ArbiterService for Heartbeat {
        fn service_started(&mut self, ctx: &mut Context<Self>) {
            println!(
                "*** service_started : {} : {} : Heartbeat state : {:?} ",
                Arbiter::name(),
                self.id,
                ctx.state()
            );
        }
    }

    impl SystemService for Heartbeat {
        fn service_started(&mut self, ctx: &mut Context<Self>) {
            self.id = format!("{:?}", Utc::now());
            println!(
                "*** service_started : {} : {} : Heartbeat state : {:?} ",
                Arbiter::name(),
                self.id,
                ctx.state()
            );
        }
    }

    impl Supervised for Heartbeat {
        fn restarting(&mut self, ctx: &mut <Self as Actor>::Context) {
            println!(
                "*** restarting: {} : Heartbeat state : {:?}",
                self.id,
                ctx.state()
            );
        }
    }

    impl Default for Heartbeat {
        fn default() -> Self {
            Heartbeat { id: "".to_string() }
        }
    }

    struct Ping;

    impl Message for Ping {
        type Result = Result<Pong, ()>;
    }

    #[derive(Debug)]
    struct Pong(String);

    impl Handler<Ping> for Heartbeat {
        type Result = Result<Pong, ()>;

        fn handle(&mut self, msg: Ping, ctx: &mut Self::Context) -> Self::Result {
            Ok(Pong(format!("*** {} : {}", self.id, Utc::now())))
        }
    }

    let sys = System::new("sys1");

    // start actor on separate thread
    let heartbeat_arbiter = Arbiter::new("heartbeat-arbiter");
    let heartbeat = heartbeat_arbiter.send(StartActor::new(|_| Heartbeat {
        id: format!("{}::{}", Arbiter::system_name(), Arbiter::name()),
    }));

    //        let heartbeat1 : () = Heartbeat {
    //            id: format!("{}::{}", Arbiter::system_name(), Arbiter::name()),
    //        }.start();

    let heartbeat1 = Arbiter::system_registry().get::<Heartbeat>();

    let ping = Ping;
    let pong = heartbeat1.send(ping);
    Arbiter::handle().spawn(
        pong.map(|res| match res {
            Ok(result) => println!("Got result: {:?}", result),
            Err(err) => println!("Got error: {:?}", err),
        }).map_err(|e| {
                println!("Actor is probably died: {:?}", e);
            })
            .and_then(|result| {
                // stop system and exit
                Arbiter::system().do_send(actix::msgs::SystemExit(0));
                ok(result)
            }),
    );

    sys.run();
}
