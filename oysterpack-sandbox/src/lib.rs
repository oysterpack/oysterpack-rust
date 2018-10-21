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

extern crate oysterpack_uid;
#[macro_use]
extern crate oysterpack_macros;
#[macro_use]
extern crate log;
#[cfg(test)]
extern crate fern;
#[macro_use]
extern crate lazy_static;
extern crate chrono;
extern crate rusty_ulid;
#[macro_use]
extern crate tokio;
#[macro_use]
extern crate crossbeam_channel;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate failure_derive;
#[macro_use]
extern crate serde_derive;
extern crate semver;
extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate juniper;

// The module declaration order matters because of macro dependencies.
// The errors module depends on the macros defined within devops and uid modules.
// Thus, the devops and uid modules need to be brought into scope before the errors module.
#[macro_use]
pub mod devops;
#[macro_use]
pub mod errors;

pub mod reactive;
pub mod time;

mod juniper_poc;

#[cfg(test)]
mod tests;
