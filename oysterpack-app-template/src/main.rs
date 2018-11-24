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

// TODO: Document crate
//!

// #![deny(missing_docs, missing_debug_implementations, warnings)]
#![deny(missing_docs, missing_debug_implementations)]
// TODO: update url
#![doc(html_root_url = "https://docs.rs/oysterpack_app_template/0.1.0")]

#[macro_use]
extern crate oysterpack;
extern crate serde_json;

#[cfg(test)]
#[macro_use]
extern crate oysterpack_testing;

use oysterpack::app_metadata;
use oysterpack::log;

op_build_mod!();

#[cfg(test)]
op_tests_mod!();

fn main() {
    let app_build = build::get();
    configure_logging(&app_build);

    info!("{}", concat!(env!("OUT_DIR"), "/built.rs"));
    info!(
        "{}",
        std::fs::read_to_string(concat!(env!("OUT_DIR"), "/built.rs")).unwrap()
    );
    info!("{}", serde_json::to_string_pretty(&app_build).unwrap(),);
}

fn configure_logging(build: &app_metadata::Build) {
    // TODO - for now it simply logs to stdout - long term, we want to be able to centrally log
    let log_config = oysterpack::log::config::LogConfigBuilder::new(log::Level::Warn)
        .crate_level(log::Level::Info)
        .build();
    oysterpack::log::init(log_config, oysterpack::log::manager::StdoutLogger);
}
