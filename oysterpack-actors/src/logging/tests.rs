// Copyright 2018 OysterPack Inc.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! tests

extern crate erased_serde;
extern crate slog;
extern crate slog_extlog;

extern crate actix;
extern crate futures;
extern crate serde_json;

use super::*;

use slog_extlog::{ExtLoggable, stats::StatDefinition};
use slog_extlog::stats;
use erased_serde::Serialize;

define_stats! {
   FOO_STATS = {
       // Some simple counters
       FooNonEmptyCount(Counter, "FOO-1001", "Count of non-empty Foo requests", []),
       FooTotalBytesByUser(Counter, "FOO-1002",
                           "Total size of all Foo requests per user", ["user"])
   }
}

#[derive(Debug, Clone, Serialize, ExtLoggable)]
#[LogDetails(Id = "101", Text = "Foo Request received", Level = "Info")]
#[StatTrigger(StatName = "FooNonEmptyCount", Action = "Incr", Condition = "self.bytes > 0",
              Value = "1")]
#[StatTrigger(StatName = "FooTotalBytesByUser", Action = "Incr", ValueFrom = "self.bytes")]
struct FooReqRcvd {
    // The number of bytes in the request
    bytes: usize,
    // The user for the request.
    #[StatGroup(StatName = "FooTotalBytesByUser")]
    user: String,
}

#[derive(Debug, Clone, Serialize, SlogValue)]
enum FooRspCode {
    Success,
    InvalidUser,
}

#[derive(Debug, Clone, Serialize, SlogValue)]
enum FooMethod {
    GET,
    PATCH,
    POST,
    PUT,
}

#[derive(Debug, Clone, Serialize, SlogValue)]
struct FooContext {
    id: String,
    method: FooMethod,
}

#[test]
fn system_logger() {
    let mut sys = System::new("sys");
    let test = super::system_logger().map(|logger| {
        info!(logger, "root_logger SUCCESS #1"; o!("cxt" =>
                         FooContext {
                           id: "123456789".to_string(),
                           method: FooMethod::POST,
                         }));

        // stat
        let cfg = stats::StatsConfig {
            stats: FOO_STATS,
            ..Default::default()
        };
        let logger = stats::StatisticsLogger::new(logger, cfg);
        xlog!(
            logger,
            FooReqRcvd {
                bytes: 42,
                user: "andrew.carnegie".to_string(),
            }
        );
        xlog!(
            logger,
            FooReqRcvd {
                bytes: 0,
                user: "cornelius.vanderbilt".to_string(),
            }
        );
        xlog!(
            logger,
            FooReqRcvd {
                bytes: 97,
                user: "john.d.rockefeller".to_string(),
            }
        );
        logger
    });
    match sys.run_until_complete(test) {
        Ok(logger) => info!(logger, "{:?}", logger.get_stats()),
        Err(err) => panic!(err),
    }
}

#[test]
#[should_panic(expected = "System is not running")]
fn root_logger_outside_of_actor_system_panics() {
    let _ = super::system_logger().map(|logger| {
        info!(logger, "root_logger SUCCESS #1");
        logger
    });
}

#[test]
fn set_system_logger() {
    // TODO: write log to a string buffer to inspect

    let mut sys = System::new("sys");
    let init_root_logger = super::system_logger()
        .and_then(|logger| {
            let logger = logger.new(o!(ACTOR_ID => 456));
            super::set_system_logger(logger)
        })
        .and_then(|_| {
            super::system_logger().map(|logger| {
                info!(logger, "set_root_logger SUCCESS #1");
                logger
            })
        });
    match sys.run_until_complete(init_root_logger) {
        Ok(logger) => info!(logger, "set_root_logger SUCCESS #2"),
        Err(err) => panic!(err),
    }
}
