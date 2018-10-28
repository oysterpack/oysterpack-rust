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

//! Standardizes logging for the OysterPack platform.
//!

// #![deny(missing_docs, missing_debug_implementations, warnings)]
#![deny(missing_docs, missing_debug_implementations)]
#![doc(html_root_url = "https://docs.rs/oysterpack_log/0.1.0")]

#[macro_use]
extern crate oysterpack_macros;
extern crate oysterpack_app_metadata;

extern crate chrono;
extern crate fern;
#[macro_use]
pub extern crate log;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

#[cfg(test)]
#[macro_use]
extern crate oysterpack_app_metadata_macros;

pub mod config;
pub mod manager;

pub use config::LogConfig;

/// re-export the log macros
pub use log::*;

pub use manager::*;

#[cfg(test)]
op_build_mod!();

#[cfg(test)]
mod tests {

    /// - ensures logging is configured and initialized
    /// - collects test execution time and logs it
    pub fn run_test<F: FnOnce() -> ()>(name: &str, test: F) {
        let log_config = ::config::LogConfigBuilder::new(log::Level::Warn)
            .crate_level(log::Level::Debug)
            .build();
        ::manager::init(log_config, &::build::get());
        let before = std::time::Instant::now();
        test();
        let after = std::time::Instant::now();
        info!(
            "{}: test run time: {:?}",
            name,
            after.duration_since(before)
        );
    }

    #[test]
    fn compiles() {
        run_test("compiles", || info!("it compiles :)"));
    }
}

#[cfg(test)]
pub use tests::run_test;
