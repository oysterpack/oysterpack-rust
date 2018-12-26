/*
 * Copyright 2018 OysterPack Inc.
 *
 *    Licensed under the Apache License, Version 2.0 (the "License");
 *    you may not use this file except in compliance with the License.
 *    You may obtain a copy of the License at
 *
 *        http://www.apache.org/licenses/LICENSE-2.0
 *
 *    Unless required by applicable law or agreed to in writing, software
 *    distributed under the License is distributed on an "AS IS" BASIS,
 *    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *    See the License for the specific language governing permissions and
 *    limitations under the License.
 */

extern crate oysterpack;
#[macro_use]
extern crate log;
extern crate simple_logging;

use log::LevelFilter;
use oysterpack::uid;

struct User;
type UserId = uid::TypedULID<User>;

fn main() {
    simple_logging::log_to_stderr(LevelFilter::Info);

    let user_id = UserId::generate();
    info!(
        "new: UserId({}) with datetime: {}",
        user_id,
        user_id.ulid().datetime()
    );

    let user_id = user_id.increment();
    info!(
        "incremented: UserId({}) with datetime: {}",
        user_id,
        user_id.ulid().datetime()
    );

    assert!(user_id.clone().increment() > user_id);
}
