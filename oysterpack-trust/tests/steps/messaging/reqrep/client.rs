/*
 * Copyright 2019 OysterPack Inc.
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

use cucumber_rust::*;

use futures::{channel::oneshot, prelude::*, task::SpawnExt};
use oysterpack_trust::concurrent::execution::{self, *};
use std::{thread, time::Duration};

steps!(World => {
    // Feature: [01D4RW7WRVBBGTBZEQCXMFN51V] The ReqRep client can be shared by cloning it.

    // Scenario: [01D4RW8V6K8HR8R1QR8DMN2AQC] Clone the ReqRep client and send requests from multiple threads
    given regex "01D4RW8V6K8HR8R1QR8DMN2AQC" | _world, _matches, _step | {
        unimplemented!()
    };

});

#[derive(Default)]
pub struct World {}
