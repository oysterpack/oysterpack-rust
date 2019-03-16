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

use oysterpack_trust::{
    concurrent::{
        execution::global_executor,
        messaging::reqrep::{ReqRepConfig, ReqRepId},
    },
    metrics,
};
use oysterpack_trust_nng::reqrep::client::{self, Client, ClientRegistrationError};
use std::time::Duration;
use url::Url;

pub mod registry;

fn server_url(reqrep_id: ReqRepId) -> Url {
    Url::parse(format!("inproc://{}", reqrep_id).as_str()).unwrap()
}

fn timer_buckets() -> Vec<f64> {
    metrics::timer_buckets(vec![
        Duration::from_micros(100),
        Duration::from_micros(200),
        Duration::from_micros(300),
        Duration::from_micros(500),
        Duration::from_micros(800),
        Duration::from_micros(1300),
        Duration::from_micros(2100),
        Duration::from_micros(3400),
        Duration::from_micros(5500),
        Duration::from_micros(8900),
    ])
    .unwrap()
}

fn register_basic_client(reqrep_id: ReqRepId) -> Client {
    client::register_client(
        ReqRepConfig::new(reqrep_id, timer_buckets()),
        None,
        client::DialerConfig::new(server_url(reqrep_id)),
        global_executor(),
    )
    .unwrap()
}

fn try_register_basic_client(reqrep_id: ReqRepId) -> Result<Client, ClientRegistrationError> {
    client::register_client(
        ReqRepConfig::new(reqrep_id, timer_buckets()),
        None,
        client::DialerConfig::new(server_url(reqrep_id)),
        global_executor(),
    )
}

#[derive(Default)]
pub struct World {
    reqrep_id: Option<ReqRepId>,
}
