// Copyright 2018 OysterPack Inc.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! unit test support

use chrono;
use fern;
use log;
use std::io;

fn init_logging() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S%.6f]"),
                record.level(),
                record.target(),
                message
            ))
        }).level(log::LevelFilter::Warn)
        .level_for("oysterpack_core", log::LevelFilter::Debug)
        .chain(io::stdout())
        .apply()?;

    Ok(())
}

lazy_static! {
    pub static ref INIT_FERN: Result<(), fern::InitError> = init_logging();
}

pub fn run_test<F: FnOnce() -> ()>(test: F) {
    let _ = *INIT_FERN;
    test()
}

pub struct Id(pub u128);

impl Id {
    /// returns the id
    pub fn id(&self) -> u128 {
        self.0
    }
}

impl ::std::fmt::Display for Id {
    /// Displays the id in lower hex format
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "{:x}", self.0)
    }
}

impl ::std::ops::Deref for Id {
    type Target = u128;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<u128> for Id {
    fn from(value: u128) -> Id {
        Id(value)
    }
}

#[test]
fn id_test() {
    run_test(|| {
        let id = Id(1);
        let id: u128 = *id;
        match id {
            1 => info!("id_test(): MATCHED"),
            _ => panic!("id_test() did not match"),
        }

        let id: Id = 1.into();
    });
}
