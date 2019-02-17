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

steps!(TestContext => {

    given regex "01D3J441N6BM05NKCBQEVYTZY8" |world, _matches, step| {
        world.init();

    };

    when regex "01D3PPPT1ZNXPKKWM29R14V5ZT-2" |world, _matches, _step| {

    };

    then regex "01D3PPPT1ZNXPKKWM29R14V5ZT-3" |world, _matches, _step| {

    };

});

#[derive(Default)]
pub struct TestContext {

}

impl TestContext {
    pub fn init(&mut self) {

    }
}