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
//! - [Service]() actors are ArbiterService(s), i.e., they are assigned to run within a specific thread
//! - [AppService]() actors are SystemService(s), i.e., they run within the System arbiter
//! - the [op_actor_service!]() macro generates the boilerplate Actor service code
//!   - logs service lifecycle events
//!
//! ## TODO
//! - service actor metrics are tracked
//!   - active service actor instance count
//!   - total service actor instance count
//!   - message count
//!   - last message received timestamp
//!   - message processing stats
//! - error tracking

use actix::{
    self,
    dev::{
        Actor, Addr, ArbiterService, Context, Handler, MailboxError, Message, MessageResponse,
        Recipient, ResponseChannel, System, SystemService,
    },
    sync::SyncContext,
};
use chrono::{DateTime, Duration, Utc};
use futures::Future;
use oysterpack_errors::Error;
use oysterpack_uid::{ulid::ulid_u128_into_string, TypedULID, ULID};
use std::{fmt, time};

/// Returns the Actor Address for the specified AppService.
pub fn app_service<A>() -> Addr<A>
where
    A: AppService + Actor<Context = Context<A>>,
{
    System::current().registry().get::<A>()
}

/// Converts a Future into a Task compatible future that can be spawned.
pub fn into_task(f: impl Future) -> impl Future<Item = (), Error = ()> {
    f.map(|_| ()).map_err(|_| ())
}

/// The future will first be converted into Task compativle future, and then spawned on the current arbiter.
///
/// # Panics
/// This function panics if actix system is not running.
pub fn spawn_task(future: impl Future + 'static) {
    actix::spawn(into_task(future));
}

/// Service is an ArbiterService, which means a new instance is created per Arbiter.
pub trait Service: ArbiterService + LifeCycle {
    /// ServiceType
    const TYPE: ServiceType = ServiceType::ArbiterService;

    /// Each Service is assigned an Id
    fn id(&self) -> Id;

    /// Each new instance is assigned a
    fn instance_id(&self) -> InstanceId;
}

/// AppService is a SystemService, which means there is only 1 instance per actor system.
/// An actor system maps to an application, thus the name.
pub trait AppService: SystemService + LifeCycle {
    /// ServiceType
    const TYPE: ServiceType = ServiceType::SystemService;

    /// Each Service is assigned an Id
    fn id(&self) -> Id;

    /// Each new instance is assigned a
    fn instance_id(&self) -> InstanceId;

    /// When the actor instance was created
    fn created_on(&self) -> DateTime<Utc> {
        self.instance_id().ulid().datetime()
    }
}

/// ServiceType
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum ServiceType {
    /// Actor is an ArbiterService
    ArbiterService,
    /// Actor is a SystemService
    SystemService,
}

/// Service lifecycle
pub trait LifeCycle: Actor {
    /// Lifecycle method called when the Actor is started
    fn on_started(&mut self, _: &mut Self::Context) {}

    /// Lifecycle method called when the Actor service is started
    fn on_service_started(&mut self, _: &mut Self::Context) {}

    /// Lifecycle method called when the Actor is restarting
    fn on_restarting(&mut self, _: &mut Self::Context) {}

    /// Lifecycle method called when the Actor is stopping
    fn on_stopping(&mut self, _: &mut Self::Context) -> actix::Running {
        actix::Running::Stop
    }

    /// Lifecycle method called when the Actor is stopped
    fn on_stopped(&mut self, _: &mut Self::Context) {}
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
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct ServiceInfo {
    id: Id,
    instance_id: InstanceId,
    service_type: ServiceType,
}

impl fmt::Display for ServiceInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ServiceInfo({}:{})", self.id, self.instance_id)
    }
}

impl ServiceInfo {
    /// constructor which is meant to be used for new Actor instances.
    pub fn for_new_actor_instance(id: Id, service_type: ServiceType) -> ServiceInfo {
        ServiceInfo {
            id,
            instance_id: InstanceId::generate(),
            service_type,
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

/// Ping is used as a heartbeat.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Ping(DateTime<Utc>);

impl Ping {
    /// constructor
    pub fn new() -> Ping {
        Default::default()
    }

    /// When the Ping was created
    pub fn timestamp(&self) -> DateTime<Utc> {
        self.0
    }
}

impl Default for Ping {
    fn default() -> Ping {
        Ping(Utc::now())
    }
}

impl Message for Ping {
    type Result = Pong;
}

impl fmt::Display for Ping {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Ping({})", self.0.to_rfc3339())
    }
}

/// Ping response
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Pong {
    ping: Ping,
    received: DateTime<Utc>,
}

impl From<Ping> for Pong {
    fn from(ping: Ping) -> Pong {
        Pong {
            ping,
            received: Utc::now(),
        }
    }
}

impl Pong {
    /// The Ping request that this Pong is responding to
    pub fn ping(&self) -> Ping {
        self.ping
    }

    /// When the Ping was received
    pub fn ping_received_timestamp(&self) -> DateTime<Utc> {
        self.received
    }

    /// how long it took to receive the Ping
    pub fn duration(&self) -> Duration {
        self.received.signed_duration_since(self.ping.timestamp())
    }
}

impl fmt::Display for Pong {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Received {} at {} : Duration({})",
            self.ping,
            self.received.to_rfc3339(),
            self.duration()
        )
    }
}

/// GetServiceClient request message
#[derive(Debug, Clone)]
pub struct GetServiceClient;

impl Message for GetServiceClient {
    type Result = ServiceClient;
}

/// ServiceClient
#[derive(Clone)]
pub struct ServiceClient {
    get_service_info: Recipient<GetServiceInfo>,
    ping: Recipient<Ping>,
    get_arbiter_name: Recipient<GetArbiterName>,
}

impl ServiceClient {
    /// constructor for an Actor Service
    pub fn for_service<A>(service: Addr<A>) -> ServiceClient
    where
        A: Service
            + actix::Handler<GetServiceInfo>
            + actix::Handler<Ping>
            + actix::Handler<GetArbiterName>,
    {
        ServiceClient {
            get_service_info: service.clone().recipient(),
            ping: service.clone().recipient(),
            get_arbiter_name: service.recipient(),
        }
    }

    /// constructor for an Actor AppService
    pub fn for_app_service<A>(service: Addr<A>) -> ServiceClient
    where
        A: AppService
            + actix::Handler<GetServiceInfo>
            + actix::Handler<Ping>
            + actix::Handler<GetArbiterName>,
    {
        ServiceClient {
            get_service_info: service.clone().recipient(),
            ping: service.clone().recipient(),
            get_arbiter_name: service.recipient(),
        }
    }

    /// Returns a future that will return ServiceInfo.
    /// - MailboxError should never happen
    pub fn get_service_info(
        &self,
        timeout: Option<time::Duration>,
    ) -> impl Future<Item = ServiceInfo, Error = MailboxError> {
        match timeout {
            Some(duration) => self.get_service_info.send(GetServiceInfo).timeout(duration),
            None => self.get_service_info.send(GetServiceInfo),
        }
    }

    /// Returns a future that will return ServiceInfo.
    /// - MailboxError should never happen
    pub fn ping(
        &self,
        timeout: Option<time::Duration>,
    ) -> impl Future<Item = Pong, Error = MailboxError> {
        match timeout {
            Some(duration) => self.ping.send(Ping::new()).timeout(duration),
            None => self.ping.send(Ping::new()),
        }
    }

    /// Returns a future that will return ServiceInfo.
    /// - MailboxError should never happen
    pub fn arbiter_name(
        &self,
        timeout: Option<time::Duration>,
    ) -> impl Future<Item = ArbiterName, Error = MailboxError> {
        match timeout {
            Some(duration) => self.get_arbiter_name.send(GetArbiterName).timeout(duration),
            None => self.get_arbiter_name.send(GetArbiterName),
        }
    }
}

impl fmt::Debug for ServiceClient {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("ServiceClient")
    }
}

/// GetArbiterName Message request
#[derive(Debug, Clone, Copy)]
pub struct GetArbiterName;

impl Message for GetArbiterName {
    type Result = ArbiterName;
}

/// ArbiterName
#[derive(Debug, Clone)]
pub struct ArbiterName(String);

impl ArbiterName {
    /// constructor
    pub fn new<T: fmt::Display>(name: T) -> ArbiterName {
        ArbiterName(name.to_string())
    }

    /// name getter
    pub fn name(&self) -> &str {
        &self.0
    }
}

impl From<&'static str> for ArbiterName {
    fn from(name: &'static str) -> ArbiterName {
        ArbiterName(name.to_string())
    }
}

impl fmt::Display for ArbiterName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.name())
    }
}

pub mod app;
pub mod arbiters;
pub mod eventlog;
pub mod events;
pub mod logger;

#[allow(warnings)]
#[cfg(test)]
mod tests {
    use super::*;
    use actix::{
        msgs::{Execute, StartActor},
        spawn, Arbiter, Supervised, System,
    };
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
                    service_info: ServiceInfo::for_new_actor_instance(
                        FOO_ID,
                        ServiceType::ArbiterService,
                    ),
                }
            }
        }

        op_actor_service! {
            Service(Foo)
        }

        impl LifeCycle for Foo {}

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
                                Err(err) => panic!("GetServiceInfo failed: {:?}", err),
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
                                                panic!("frontend: GetServiceInfo failed: {:?}", err)
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
                                                panic!("frontend: GetServiceInfo failed: {:?}", err)
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

        run_test("Ping", || {
            System::run(|| {
                let foo = Arbiter::registry().get::<Foo>();

                let task = foo
                    .send(Ping::new())
                    .then(|pong| {
                        let pong = pong.unwrap();
                        info!("{}", pong);
                        future::ok::<(), ()>(())
                    }).then(|_| {
                        System::current().stop();
                        future::ok::<_, ()>(())
                    });

                spawn_task(task);
            });
        });

        run_test("GetServiceClient", || {
            System::run(|| {
                let foo = Arbiter::registry().get::<Foo>();
                let task = foo
                    .send(GetServiceClient)
                    .then(|client| {
                        let client = client.unwrap();
                        let ping = client.ping(None);
                        let service_info = client.get_service_info(Some(Duration::from_millis(10)));
                        let arbiter_name = client.arbiter_name(None);

                        ping.then(move |pong| {
                            info!("{}", pong.unwrap());
                            service_info
                        }).then(|service_info| {
                            info!("{}", service_info.unwrap());
                            arbiter_name
                        }).then(|arbiter_name| {
                            info!("arbiter: {}", arbiter_name.unwrap());
                            future::ok::<_, ()>(())
                        })
                    }).then(|_| {
                        System::current().stop();
                        future::ok::<_, ()>(())
                    });

                spawn_task(task);
            });
        });
    }
}
