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

//! Arbiter central registry

use super::{
    AppService, GetServiceInfo, Id as ServiceId, InstanceId as ServiceInstanceId, ServiceInfo,
};
use actix::{registry::SystemService, Actor, Addr, Arbiter, Context, Handler, Supervised};
use futures::prelude::*;
use std::collections::HashMap;

/// Arbiters ServiceId
pub const SERVICE_ID: ServiceId = ServiceId(1865070194757304474174751345022876889);

/// Arbiter registry
#[derive(Debug)]
pub struct Arbiters {
    service_info: ServiceInfo,
    arbiters: HashMap<Id, Addr<Arbiter>>,
}

impl Default for Arbiters {
    fn default() -> Self {
        Arbiters {
            arbiters: HashMap::new(),
            service_info: ServiceInfo::for_new_actor_instance(SERVICE_ID),
        }
    }
}

op_newtype! {
    /// Arbiter identifier
    #[derive(Serialize, Deserialize, Clone, Copy, Eq, PartialEq, Hash, Ord, PartialOrd)]
    pub Id(pub u8)
}

impl Actor for Arbiters {
    type Context = Context<Self>;

    fn started(&mut self, _: &mut Self::Context) {
        info!("started: {:?}", self.service_info);
    }

    fn stopped(&mut self, _: &mut Self::Context) {
        info!("stopped: {:?}", self.service_info);
    }
}

impl AppService for Arbiters {
    fn id(&self) -> ServiceId {
        self.service_info.id
    }

    fn instance_id(&self) -> ServiceInstanceId {
        self.service_info.instance_id
    }
}

impl SystemService for Arbiters {
    fn service_started(&mut self, _: &mut Context<Self>) {
        info!("service_started: {:?}", self.service_info);
    }
}

impl Supervised for Arbiters {
    fn restarting(&mut self, _: &mut Self::Context) {
        info!("restarting: {:?}", self.service_info);
    }
}

impl Handler<GetServiceInfo> for Arbiters {
    type Result = ServiceInfo;

    fn handle(&mut self, _: GetServiceInfo, _: &mut Self::Context) -> Self::Result {
        self.service_info
    }
}
