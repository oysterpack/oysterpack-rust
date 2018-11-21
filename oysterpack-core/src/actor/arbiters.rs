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
use actix::{
    registry::SystemService, Actor, Addr, Arbiter, Context, Handler, Message, MessageResult,
    Supervised,
};
use futures::prelude::*;
use std::{collections::HashMap, fmt};

/// Arbiters ServiceId (01CWSGYS79QQHAE6ZDRKB48F6S)
pub const SERVICE_ID: ServiceId = ServiceId(1865070194757304474174751345022876889);

/// Arbiter registry
#[derive(Debug)]
pub struct Arbiters {
    service_info: ServiceInfo,
    arbiters: HashMap<Name, Addr<Arbiter>>,
}

impl Default for Arbiters {
    fn default() -> Self {
        Arbiters {
            arbiters: HashMap::new(),
            service_info: ServiceInfo::for_new_actor_instance(SERVICE_ID),
        }
    }
}

/// Arbiter name
#[derive(Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Clone, Copy)]
pub struct Name(pub &'static str);

impl From<&'static str> for Name {
    fn from(name: &'static str) -> Name {
        Name(name)
    }
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.0)
    }
}

impl Actor for Arbiters {
    type Context = Context<Self>;

    fn started(&mut self, _: &mut Self::Context) {
        info!("started: {}", self.service_info);
    }

    fn stopped(&mut self, _: &mut Self::Context) {
        info!("stopped: {}", self.service_info);
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
        info!("service_started: {}", self.service_info);
    }
}

impl Supervised for Arbiters {
    fn restarting(&mut self, _: &mut Self::Context) {
        info!("restarting: {}", self.service_info);
    }
}

impl Handler<GetServiceInfo> for Arbiters {
    type Result = ServiceInfo;

    fn handle(&mut self, _: GetServiceInfo, _: &mut Self::Context) -> Self::Result {
        self.service_info
    }
}

/// GetArbiter request message.
///
/// If an Arbiter with the specified name does not exist, then it will be created.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct GetArbiter {
    name: Name,
}

impl Message for GetArbiter {
    type Result = Addr<Arbiter>;
}

impl Handler<GetArbiter> for Arbiters {
    type Result = MessageResult<GetArbiter>;

    fn handle(&mut self, request: GetArbiter, _: &mut Self::Context) -> Self::Result {
        let arbiter_addr = self
            .arbiters
            .entry(request.name)
            .or_insert_with(|| Arbiter::new(request.name.to_string()));

        MessageResult(arbiter_addr.clone())
    }
}

#[allow(warnings)]
#[cfg(test)]
mod tests {
    use super::*;
    use actix::{msgs::Execute, prelude::*, registry::SystemRegistry, spawn};
    use futures::{future, prelude::*};
    use tests::run_test;

    #[test]
    fn get_service_info() {
        run_test("GetServiceInfo", || {
            System::run(|| {
                fn get_service_info() -> impl Future {
                    let arbiters = System::current().registry().get::<Arbiters>();
                    arbiters.send(GetServiceInfo).then(|result| {
                        match result {
                            Ok(info) => info!("{}", info),
                            Err(err) => panic!("{}", err),
                        }
                        future::ok::<_, ()>(())
                    })
                }

                let future = get_service_info().then(|_| {
                    System::current().stop();
                    future::ok::<_, ()>(())
                });

                spawn(future);
            });
        });

        fn into_task<>(f: impl Future) -> impl Future<Item=(), Error=()> {
            f.map(|_| ()).map_err(|_| ())
        }

        run_test("GetServiceInfo - within separate Arbiter", || {
            System::run(|| {
                fn get_service_info() -> impl Future {
                    let arbiters = System::current().registry().get::<Arbiters>();
                    arbiters.send(GetServiceInfo).then(|result| {
                        match result {
                            Ok(info) => info!("{}", info),
                            Err(err) => panic!("{}", err),
                        }
                        future::ok::<_, ()>(())
                    })
                }

                const FRONTEND: Name = Name("FRONTEND");
                let arbiters = System::current().registry().get::<Arbiters>();
                let future =
                    arbiters
                        .send(GetArbiter { name: FRONTEND })
                        .then(|result| {
                            let future = match result {
                                Ok(arbiter) => arbiter.send(Execute::new(|| -> Result<(), ()> {
                                    let future = get_service_info().then(|_| {
                                        System::current().stop();
                                        future::ok::<_, ()>(())
                                    });
                                    spawn(future);
                                    Ok(())
                                })),
                                Err(err) => panic!("GetArbiter failed: {}", err),
                            };
                            spawn(into_task(future));
                            future::ok::<_, ()>(())
                        });
                spawn(future);
            });
        });
    }
}
