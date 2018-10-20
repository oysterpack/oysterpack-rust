// Copyright 2018 OysterPack Inc.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern crate oysterpack;
#[macro_use]
extern crate log;
extern crate simple_logging;

use log::LevelFilter;
use oysterpack::ulid;

struct User;
type UserId = ulid::Uid<User>;

fn main() {
    simple_logging::log_to_stderr(LevelFilter::Info);

    let uid = UserId::new();
    info!("new: UserId({}) with datetime: {}",uid, uid.datetime());

    let uid = uid.increment().unwrap();
    info!("incremented: UserId({}) with datetime: {}",uid, uid.datetime());

    assert!(uid.clone().increment().unwrap() > uid);
}