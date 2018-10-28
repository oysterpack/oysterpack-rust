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

//! Standardizes logging for the OysterPack platform on top of [log](https://crates.io/crates/log).
//! Given a LogConfig, this crate will know how to initialize the logging system and how to shut it down.
//!
//! ```rust
//! #[macro_use]
//! extern crate oysterpack_app_metadata_macros;
//!
//! op_build_mod!();
//!
//! fn main() {
//!     let app_build = build::get();
//!     oysterpack_log::init(log_config(),&app_build);
//!     // The LogConfig used to initialize the log system can be retrieved.
//!     // This enables the LogConfig to be inspected.
//!     let log_config = oysterpack_log::config().unwrap();
//!
//!     run();
//!
//!     oysterpack_log::shutdown();
//! }
//!
//! /// This should be loaded from the app's configuration.
//! /// For this simple example, we are simply using the default LogConfig.
//! /// The default LogConfig sets the root log level to Warn and logs to stdout.
//! fn log_config() -> oysterpack_log::LogConfig {
//!     Default::default()
//! }
//!
//! fn run() {}
//! ```

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


pub use log::{
    // re-export the log macros
    debug,
    error,
    info,
    log,
    log_enabled,
    trace,
    warn,
    // re-export some other common log members
    Level,
    LevelFilter
};

pub use manager::{
    config,
    init,
    shutdown
};

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
