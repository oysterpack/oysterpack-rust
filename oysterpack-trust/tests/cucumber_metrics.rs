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

mod steps;

use oysterpack_trust::metrics;

#[derive(Default)]
pub struct TestContext {
    pub metric_id: Option<metrics::MetricId>,
    pub metric_ids: Option<Vec<metrics::MetricId>>,
    pub int_counter_registration_result: Option<Result<prometheus::IntCounter, prometheus::Error>>,
}

impl TestContext {
    fn init(&mut self) {
        self.metric_id = None;
        self.int_counter_registration_result = None;
        self.metric_ids = None;
    }
}

impl cucumber_rust::World for TestContext {}

cucumber! {
    features: "./features/metrics",
    world: crate::TestContext,
    steps: &[
        steps::metrics::steps
    ]
}