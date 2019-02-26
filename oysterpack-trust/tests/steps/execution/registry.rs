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

use futures::{prelude::*, task::SpawnExt};
use maplit::*;
use oysterpack_trust::{
    concurrent::execution::{self, *},
    metrics,
};
use std::{collections::HashSet, num::NonZeroUsize, panic, thread, time::Duration};

steps!(World => {
    // Feature: [01D3W0H2B7KNTBJTGDYP3CRB7K] A global Executor registry is provided.

    // Scenario: [01D3W0MDTMRJ6GNFCQCPTS55HG] Registering an Executor with default settings
    then regex "01D3W0MDTMRJ6GNFCQCPTS55HG-1" | _world, _matches, _step | {
        unimplemented!();
    };

    then regex "01D3W0MDTMRJ6GNFCQCPTS55HG-2" | _world, _matches, _step | {
        unimplemented!();
    };
});

#[derive(Clone, Default)]
pub struct World {

}