// Copyright 2018 OysterPack Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

extern crate oysterpack_core;
extern crate oysterpack_log;

extern crate actix;
extern crate futures;

use oysterpack_core::actor;
use oysterpack_log::log::*;

use actix::System;
use futures::{future, prelude::*};

#[test]
fn init_logging() {
    use oysterpack_log;

    fn log_config() -> oysterpack_log::LogConfig {
        oysterpack_log::config::LogConfigBuilder::new(Level::Info).build()
    }

    System::run(|| {
        let task = actor::logger::init_logging(log_config());
        let task = task
            .and_then(|_| {
                for i in 0..10 {
                    info!("LOG MSG #{}", i);
                }
                Ok(())
            })
            .then(|_| {
                // Not all log messages may have been processed. Queued messages will simply get dropped.
                info!("STOPPING ACTOR SYSTEM");
                System::current().stop();
                future::ok::<(), ()>(())
            });
        actor::spawn_task(task);
    });
}
