// Copyright 2018 OysterPack Inc.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[macro_use]
extern crate oysterpack;
#[macro_use]
extern crate log;
extern crate simple_logging;

use log::LevelFilter;
use oysterpack::uid;

struct User;
type UserId = uid::Uid<User>;

#[derive(Serialize,Deserialize)]
struct Foo(u128);

#[test]
fn test() {
    simple_logging::log_to_stderr(LevelFilter::Info);

    let user_id = UserId::new();
    info!("new: UserId({}) with datetime: {}",user_id, user_id.datetime());

    let user_id = user_id.increment().unwrap();
    info!("incremented: UserId({}) with datetime: {}",user_id, user_id.datetime());

    assert!(user_id.clone().increment().unwrap() > user_id);
}