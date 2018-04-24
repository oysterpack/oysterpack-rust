// Copyright 2018 OysterPack Inc.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! tests

extern crate actix;
extern crate futures;

use super::*;

use self::actix::prelude::*;
use self::actix::msgs::*;
use self::futures::prelude::*;
use self::futures::future::ok;

lazy_static! {
 pub static ref ARBITER_ID_1 : ArbiterId = ArbiterId::new();
}

#[test]
fn arbiter_ids_can_be_stored_in_lazy_static() {
    let id = ArbiterId::new();
    println!("id = {}", id);

    println!("ARBITER_ID_1 = {}", *ARBITER_ID_1);

    let id: ArbiterId = *ARBITER_ID_1;
    println!("ARBITER_ID_1 = {:?}", id);
}

#[test]
fn arbiter_registry_is_defined_as_system_service() {
    let mut sys = System::new("sys");
    let system_registry = Arbiter::system_registry();
    let _: Addr<Syn, _> = system_registry.get::<ArbiterRegistry>();
    let _ = sys.run_until_complete(ok::<(), ()>(()));
}

#[test]
fn arbiters_are_created_on_demand() {
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

#[test]
fn arbiter() {
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

#[test]
fn stopped_arbiter_is_unregistered_on_demand() {
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
                println!("arbiter is still connected ...");
                sleep(Duration::from_millis(10));
            }
            println!("arbiter is not connected");

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

#[test]
fn stopped_arbiter_is_replaced_on_demand() {
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
                println!("arbiter is still connected ...");
                sleep(Duration::from_millis(10));
            }
            println!("arbiter is not connected");

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

#[test]
fn arbiter_count() {
    let mut sys = System::new("sys");
    let test = super::arbiter_count()
        .map(|count| {
            println!("arbiter count = {:?}", count);
            assert_eq!(count.0, 0);
            count
        })
        .and_then(|_| {
            super::arbiter(*ARBITER_ID_1).and_then(|_| {
                println!("arbiter registered");
                super::arbiter_count().map(|count| {
                    println!("arbiter count = {:?}", count);
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

#[test]
fn arbiter_ids() {
    let mut sys = System::new("sys");
    let test = super::arbiter_ids()
        .map(|arbiter_ids| {
            println!("arbiter_ids = {:?}", arbiter_ids);
            assert_eq!(arbiter_ids.len(), 0);
            arbiter_ids
        })
        .and_then(|_| {
            super::arbiter(*ARBITER_ID_1).and_then(|_| {
                println!("arbiter registered");
                super::arbiter_ids().map(|arbiter_ids| {
                    println!("arbiter_ids = {:?}", arbiter_ids);
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

#[test]
fn contains_arbiter() {
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
                println!("arbiter registered");
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
