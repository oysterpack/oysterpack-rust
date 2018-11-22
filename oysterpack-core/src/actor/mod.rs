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

//! Actors are core on the OysterPack platform. [Actix](https://crates.io/crates/actix) is used as the
//! underlying Actor framework. Top level actors are registered as services, either as an ArbiterService
//! or as a SystemService.
//!
//! ## Actor Design Guidelines
//! 1. Top level actors are registered as services, i.e., ArbiterService or SystemService
//! 2. Internally, normal actors are used as workers.
//! 3. Sync actors, i.e., actors with a SyncContext, are not exposed directly. They should be accessed
//!    via an actor service.
//!
//! ## Features
//! - service actors are assigned an Id
//! - each service actor instance is assigned an InstanceId
//! - service actor metrics are tracked
//!   - active service actor instance count
//!   - total service actor instance count
//!   - message count
//!   - last message received timestamp
//!   - message processing stats
//! - error tracking
//! - events
//!   - service started
//!   - service stopped

use actix::{
    dev::{
        Actor, ArbiterService, Context, Handler, Message, MessageResponse, ResponseChannel,
        SystemService,
    },
    sync::SyncContext,
};
use chrono::{DateTime, Utc};
use oysterpack_errors::Error;
use oysterpack_uid::{ulid::ulid_u128_into_string, TypedULID, ULID};
use std::fmt;

/// Service is an ArbiterService, which means a new instance is created per Arbiter.
pub trait Service: ArbiterService {
    /// Each Service is assigned an Id
    fn id(&self) -> Id;

    /// Each new instance is assigned a
    fn instance_id(&self) -> InstanceId;

    /// When the actor instance was created
    fn created_on(&self) -> DateTime<Utc> {
        self.instance_id().ulid().datetime()
    }
}

/// AppService is a SystemService, which means there is only 1 instance per actor system.
/// An actor system maps to an application, thus the name.
pub trait AppService: SystemService {
    /// Each Service is assigned an Id
    fn id(&self) -> Id;

    /// Each new instance is assigned a
    fn instance_id(&self) -> InstanceId;

    /// When the actor instance was created
    fn created_on(&self) -> DateTime<Utc> {
        self.instance_id().ulid().datetime()
    }
}

op_newtype! {
    /// Service identifier
    #[derive(Serialize, Deserialize, Clone, Copy, Eq, PartialEq, Hash, Ord, PartialOrd)]
    pub Id(pub u128)
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(ulid_u128_into_string(self.0).as_str())
    }
}

impl From<ULID> for Id {
    fn from(id: ULID) -> Id {
        Id(id.into())
    }
}

impl Into<ULID> for Id {
    fn into(self) -> ULID {
        ULID::from(self.0)
    }
}

/// ServiceActor is used to define InstanceId
#[derive(Debug)]
pub struct ServiceActor;
/// Service Actor Instance Id
pub type InstanceId = TypedULID<ServiceActor>;

/// Service info
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ServiceInfo {
    id: Id,
    instance_id: InstanceId,
}

impl fmt::Display for ServiceInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ServiceInfo({}:{})", self.id, self.instance_id)
    }
}

impl ServiceInfo {
    /// constructor which is meant to be used for new Actor instances.
    pub fn for_new_actor_instance(id: Id) -> ServiceInfo {
        ServiceInfo {
            id,
            instance_id: InstanceId::generate(),
        }
    }

    /// Returns the Actor Id
    pub fn id(&self) -> Id {
        self.id
    }

    /// Returns the Actor InstanceId
    pub fn instance_id(&self) -> InstanceId {
        self.instance_id
    }

    /// When the actor was created
    pub fn created_on(&self) -> DateTime<Utc> {
        self.instance_id.ulid().datetime()
    }
}

impl<T: Service> From<T> for ServiceInfo {
    fn from(service: T) -> Self {
        ServiceInfo {
            id: service.id(),
            instance_id: service.instance_id(),
        }
    }
}

impl<A, M> MessageResponse<A, M> for ServiceInfo
where
    A: Actor,
    M: Message<Result = ServiceInfo>,
{
    fn handle<R: ResponseChannel<M>>(self, _: &mut A::Context, tx: Option<R>) {
        if let Some(tx) = tx {
            tx.send(self);
        }
    }
}

/// GetServiceInfo request message
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GetServiceInfo;

impl Message for GetServiceInfo {
    type Result = ServiceInfo;
}

pub mod events;
pub mod arbiters;

#[allow(warnings)]
#[cfg(test)]
mod tests {
    use super::*;
    use actix::{msgs::Execute, spawn, Arbiter, Supervised, System};
    use futures::{future, prelude::*};
    use std::time::Duration;
    use tests::run_test;

    #[test]
    fn service() {
        const FOO_ID: Id = Id(1864734280873114327279151769208160280);

        struct Foo {
            service_info: ServiceInfo,
        }

        impl Default for Foo {
            fn default() -> Self {
                Foo {
                    service_info: ServiceInfo::for_new_actor_instance(FOO_ID),
                }
            }
        }

        impl Actor for Foo {
            type Context = Context<Self>;

            fn started(&mut self, ctx: &mut Self::Context) {
                info!("started: {}", self.service_info);
            }

            fn stopped(&mut self, ctx: &mut Self::Context) {
                info!("stopped: {}", self.service_info);
            }
        }

        // TODO: macro
        impl Service for Foo {
            fn id(&self) -> Id {
                self.service_info.id
            }

            fn instance_id(&self) -> InstanceId {
                self.service_info.instance_id
            }
        }

        // TODO: macro
        impl ArbiterService for Foo {
            fn service_started(&mut self, ctx: &mut Context<Self>) {
                info!("service_started: {}", self.service_info);
            }
        }

        // TODO: macro
        impl Supervised for Foo {
            fn restarting(&mut self, ctx: &mut Self::Context) {
                info!("restarting: {}", self.service_info);
            }
        }

        // TODO: macro - relies on `service_info` field
        impl Handler<GetServiceInfo> for Foo {
            type Result = ServiceInfo;

            fn handle(&mut self, msg: GetServiceInfo, ctx: &mut Self::Context) -> Self::Result {
                self.service_info
            }
        }

        run_test("GetServiceInfo", || {
            System::run(|| {
                let foo = Arbiter::registry().get::<Foo>();
                let frontend = Arbiter::builder().name("frontend").build();
                let frontend2 = frontend.clone();

                Arbiter::spawn(
                    Arbiter::registry()
                        .get::<Foo>()
                        .send(GetServiceInfo)
                        .timeout(Duration::from_millis(10))
                        .then(|info| {
                            match info {
                                Ok(info) => info!("GetServiceInfo Response: {}", info),
                                Err(err) => error!("GetServiceInfo failed: {:?}", err),
                            }
                            future::ok::<_, ()>(())
                        }).then(move |_| {
                            frontend.send(Execute::new(|| -> Result<(), ()> {
                                let future = Arbiter::registry()
                                    .get::<Foo>()
                                    .send(GetServiceInfo)
                                    .timeout(Duration::from_millis(10))
                                    .then(|info| {
                                        match info {
                                            Ok(info) => {
                                                info!("frontend: GetServiceInfo Response: {}", info)
                                            }
                                            Err(err) => {
                                                error!("frontend: GetServiceInfo failed: {:?}", err)
                                            }
                                        }
                                        future::ok::<_, ()>(())
                                    });
                                spawn(future);
                                Ok(())
                            }))
                        }).then(move |_| {
                            frontend2.send(Execute::new(|| -> Result<(), ()> {
                                let future = Arbiter::registry()
                                    .get::<Foo>()
                                    .send(GetServiceInfo)
                                    .timeout(Duration::from_millis(10))
                                    .then(|info| {
                                        match info {
                                            Ok(info) => {
                                                info!("frontend: GetServiceInfo Response: {}", info)
                                            }
                                            Err(err) => {
                                                error!("frontend: GetServiceInfo failed: {:?}", err)
                                            }
                                        }
                                        future::ok::<_, ()>(())
                                    });
                                spawn(future);
                                Ok(())
                            }))
                        }).then(|_| {
                            System::current().stop();
                            future::ok::<_, ()>(())
                        }),
                );
            });
        });

        run_test("GetServiceInfo - 1024 tasks", || {
            System::run(|| {
                fn join_tasks(task_count: usize) -> impl Future {
                    let foo = Arbiter::registry().get::<Foo>();
                    let mut tasks = Vec::with_capacity(task_count);
                    for _ in 0..task_count {
                        let task = foo
                            .send(GetServiceInfo)
                            .timeout(Duration::from_millis(10))
                            .then(|_| {
                                System::current().stop();
                                future::ok::<_, ()>(())
                            });
                        tasks.push(task);
                    }
                    future::join_all(tasks)
                }

                // at around 300, actix panics because of a debug_assert within the mailbox:
                // ```
                // debug_assert!(
                //   n_polls.inc() < MAX_SYNC_POLLS,
                //   "Use Self::Context::notify() instead of direct use of address"
                // );
                // ```
                // where MAX_SYNC_POLLS = 256
                // i.e., it panics if there are more than 256 msgs queued in the mailbox, then a panic is triggered
                const TASK_BUCKET_COUNT: usize = 256;

                let task = join_tasks(TASK_BUCKET_COUNT)
                    .then(|_| join_tasks(TASK_BUCKET_COUNT))
                    .then(|_| join_tasks(TASK_BUCKET_COUNT))
                    .then(|_| join_tasks(TASK_BUCKET_COUNT));

                Arbiter::spawn(task.then(|_| {
                    System::current().stop();
                    future::ok::<_, ()>(())
                }));
            });
        });
    }
}


