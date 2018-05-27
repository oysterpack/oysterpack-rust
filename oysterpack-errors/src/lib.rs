// Copyright 2018 OysterPack Inc.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern crate chrono;
extern crate oysterpack_id;

use chrono::prelude::*;
use oysterpack_id::Id;

pub struct Error<T> {
    err: T,
    timestamp: Utc
}

pub enum Severity {
    LOW,
    MEDIUM,
    HIGH,
    CRITICAL,
    EMERGENCY
}


#[cfg(test)]
mod tests {
    #[test]
    fn quick_test() {}
}
