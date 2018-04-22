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
use self::futures::prelude::*;
use self::futures::future::{lazy, ok, AndThen, FutureResult};

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

    let registry: Addr<Syn, _> = system_registry.get::<ArbiterRegistry>();

    sys.run_until_complete(ok::<(), ()>(()));
}

#[test]
fn arbiter_not_exists() {
    let mut sys = System::new("sys");

    let check = super::arbiter(ArbiterId::new())
        .and_then(|arbiter| {
            assert!(arbiter.connected());
            ok(arbiter)
        });
    let result = sys.run_until_complete(check);
    match result {
        Ok(_) => (),
        Err(err) => panic!("failed to register arbiter : {:?}", err),
    }
}

#[test]
fn arbiter() {
    let mut sys = System::new("sys");
    let check = super::arbiter(*ARBITER_ID_1)
        .map(|arbiter| {
            assert!(arbiter.connected());
            super::arbiter(*ARBITER_ID_1).and_then(|arbiter| {
                assert!(arbiter.connected());
                ok(arbiter)
            })
        })
        .flatten();

    let result = sys.run_until_complete(check);
    match result {
        Ok(_) => (),
        Err(err) => panic!("failed to register arbiter : {:?}", err),
    }
}
