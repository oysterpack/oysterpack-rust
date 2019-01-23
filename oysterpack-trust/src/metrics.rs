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

//! Provides metrics support

use oysterpack_uid::macros::ulid;
use serde::{Deserialize, Serialize};

/// Metric Id
///
/// ### Why use a number as a metric name ?
/// Because names change over time, which can break components that depend on metric names ...
/// Assigning unique numerical identifiers is much more stable. Human friendly metric labels and any
/// additional information can be mapped externally to the MetricId.
#[ulid]
pub struct MetricId(pub u128);

/// Metric descriptor.
/// - the main purpose is to provide human friendly metric description
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricDesc {
    id: MetricId,
    name: String,
    short_description: Option<String>,
}

#[allow(warnings)]
#[cfg(test)]
mod tests {

    use super::*;
    use crate::configure_logging;
    use oysterpack_log::*;
    use std::thread;

    #[test]
    fn hotmic_metrics() {
        configure_logging();

        const REQREP: MetricId = MetricId(1871663033774396702741818028382227928);
        const COUNTER: MetricId = MetricId(1871663111240856363309308798871376715);


    }

}
