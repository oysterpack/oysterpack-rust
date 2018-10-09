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
extern crate oysterpack_app_metadata;
#[macro_use]
extern crate log;
extern crate chrono;
extern crate fern;
extern crate semver;
extern crate serde;
#[macro_use]
extern crate serde_derive;

#[macro_use]
#[cfg(test)]
extern crate lazy_static;
#[cfg(test)]
extern crate serde_json;

op_build_mod!();

#[cfg(test)]
mod tests;

fn main() {
    configure_logging();
    info!("{}-{}", build::PKG_NAME, build::PKG_VERSION);
}

fn configure_logging() {
    // TODO

    use std::io;

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
        .level_for(build::PKG_NAME, log::LevelFilter::Info)
        .chain(io::stdout())
        .apply()
        .expect("Failed to configure logging");
}
