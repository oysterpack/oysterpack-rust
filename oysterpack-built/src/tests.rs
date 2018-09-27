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

/// provides build-time information
mod build {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

#[test]
fn build_info() {
    println!("{}", concat!(env!("OUT_DIR"), "/built.rs"));
    println!(
        "This is version {}{}, built for {} by {}.",
        build::PKG_VERSION,
        build::GIT_VERSION.map_or_else(|| "".to_owned(), |v| format!(" (git {})", v)),
        build::TARGET,
        build::RUSTC_VERSION
    );
    println!(
        "I was built with profile \"{}\", features \"{}\" on {}",
        build::PROFILE,
        build::FEATURES_STR,
        build::BUILT_TIME_UTC
    );
}
