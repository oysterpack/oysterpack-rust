/*
 * Copyright 2018 OysterPack Inc.
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
        Actor, Addr, Arbiter, ArbiterService, Context, Handler, MailboxError, Message,
        MessageResponse, Recipient, Request, ResponseChannel, SendError, System, SystemService,
    },
    sync::SyncContext,
};
use chrono::{DateTime, Duration, Utc};
use futures::{Async, Future, Poll};
use oysterpack_app_metadata::Build;
use oysterpack_errors::Error;
use oysterpack_events::Id as EventId;
use oysterpack_uid::{ulid::ulid_u128_into_string, ULID, macros::{
    ulid, domain
}};
use std::{
    collections::{HashMap, HashSet},
    fmt,
    hash::{Hash, Hasher},
    time,
};

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
pub trait Service: ArbiterService + LifeCycle + DisplayName {
    /// ServiceType
    const TYPE: ServiceType = ServiceType::ArbiterService;

    /// Each Service is assigned an Id
    fn id(&self) -> Id;

    /// Each new instance is assigned a
    fn instance_id(&self) -> InstanceId;
}

/// AppService is a SystemService, which means there is only 1 instance per actor system.
/// An actor system maps to an application, thus the name.
pub trait AppService: SystemService + LifeCycle + DisplayName {
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

/// Provides a name
pub trait DisplayName {
    /// name getter
    fn name() -> &'static str;
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

#[domain(ServiceActor)]
#[ulid]
/// Service identifier
pub struct Id(pub u128);

#[domain(ServiceActorInstance)]
#[ulid]
/// Service Actor Instance Id
pub struct InstanceId(ULID);

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

    /// ServiceType getter
    pub fn service_type(&self) -> ServiceType {
        self.service_type
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

/// Heartbeat request message that simply responds with a Heartbeat.
///
/// # Use Case
/// This is meant to send to be used to test how fast an Actor can reply to a message with absolutely
/// no overhead.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Heartbeat;

impl Message for Heartbeat {
    type Result = Heartbeat;
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

/// GetDisplayName request message
#[derive(Debug, Clone)]
pub struct GetDisplayName;

impl Message for GetDisplayName {
    type Result = &'static str;
}

/// ServiceClient
#[derive(Clone)]
pub struct ServiceClient {
    get_service_info: Recipient<GetServiceInfo>,
    ping: Recipient<Ping>,
    heartbeat: Recipient<Heartbeat>,
    get_arbiter_name: Recipient<GetArbiterName>,
    name: &'static str,
}

impl ServiceClient {
    /// constructor for an Actor Service
    pub fn for_service<A>(service: Addr<A>) -> ServiceClient
    where
        A: Service
            + actix::Handler<GetServiceInfo>
            + actix::Handler<Ping>
            + actix::Handler<GetArbiterName>
            + actix::Handler<Heartbeat>
            + DisplayName,
    {
        ServiceClient {
            get_service_info: service.clone().recipient(),
            ping: service.clone().recipient(),
            heartbeat: service.clone().recipient(),
            get_arbiter_name: service.recipient(),
            name: <A as DisplayName>::name(),
        }
    }

    /// constructor for an Actor AppService
    pub fn for_app_service<A>(service: Addr<A>) -> ServiceClient
    where
        A: AppService
            + actix::Handler<GetServiceInfo>
            + actix::Handler<Ping>
            + actix::Handler<GetArbiterName>
            + actix::Handler<Heartbeat>
            + DisplayName,
    {
        ServiceClient {
            get_service_info: service.clone().recipient(),
            ping: service.clone().recipient(),
            heartbeat: service.clone().recipient(),
            get_arbiter_name: service.recipient(),
            name: <A as DisplayName>::name(),
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

    /// Returns a future that pings the service Actor.
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

    /// Returns a future that sends a heartbeat message to the service Actor.
    /// - MailboxError should never happen
    pub fn heartbeat(
        &self,
        timeout: Option<time::Duration>,
    ) -> impl Future<Item = Heartbeat, Error = MailboxError> {
        match timeout {
            Some(duration) => self.heartbeat.send(Heartbeat).timeout(duration),
            None => self.heartbeat.send(Heartbeat),
        }
    }

    /// Returns a future that will return the name of the Aribter that the service Actor is running in..
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

    /// Service name getter
    pub fn service_name(&self) -> &'static str {
        self.name
    }
}

impl fmt::Debug for ServiceClient {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ServiceClient({})", self.name)
    }
}

impl fmt::Display for ServiceClient {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ServiceClient({})", self.name)
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

pub mod config;
pub mod errors;
pub mod alarms;

/// AppClient provides a 1 stop shop to work with the App.
#[derive(Clone)]
pub struct AppClient {
    get_build: Recipient<app::GetBuild>,
    get_instance_id: Recipient<app::GetInstanceId>,
    get_app_instance_info: Recipient<app::GetAppInstanceInfo>,
    get_log_config: Recipient<app::GetLogConfig>,
    get_registered_services: Recipient<app::GetRegisteredServices>,

    get_arbiter_names: Recipient<arbiters::GetArbiterNames>,
    get_arbiter: Recipient<arbiters::GetArbiter>,

    get_registered_events: Recipient<eventlog::GetRegisteredEvents>,
    get_unregistered_events: Recipient<eventlog::GetUnregisteredEvents>,
}

impl AppClient {
    /// constructor
    ///
    /// # Panics
    /// if created outside the context of an actor System
    pub fn get() -> AppClient {
        let app = System::current().registry().get::<app::App>();
        let aribiters = System::current().registry().get::<arbiters::Arbiters>();
        let eventlog = System::current().registry().get::<eventlog::EventLog>();

        AppClient {
            get_build: app.clone().recipient(),
            get_instance_id: app.clone().recipient(),
            get_app_instance_info: app.clone().recipient(),
            get_log_config: app.clone().recipient(),
            get_registered_services: app.clone().recipient(),

            get_arbiter_names: aribiters.clone().recipient(),
            get_arbiter: aribiters.clone().recipient(),

            get_registered_events: eventlog.clone().recipient(),
            get_unregistered_events: eventlog.clone().recipient(),
        }
    }

    /// Returns a future that will return the app Build.
    /// - MailboxError should never happen
    pub fn get_registered_events(
        &self,
        timeout: Option<time::Duration>,
    ) -> impl Future<Item = Vec<EventId>, Error = MailboxError> {
        match timeout {
            Some(duration) => self
                .get_registered_events
                .send(eventlog::GetRegisteredEvents)
                .timeout(duration),
            None => self
                .get_registered_events
                .send(eventlog::GetRegisteredEvents),
        }
    }

    /// Returns a future that will return the app Build.
    /// - MailboxError should never happen
    pub fn get_unregistered_events(
        &self,
        timeout: Option<time::Duration>,
    ) -> impl Future<Item = Vec<EventId>, Error = MailboxError> {
        match timeout {
            Some(duration) => self
                .get_unregistered_events
                .send(eventlog::GetUnregisteredEvents)
                .timeout(duration),
            None => self
                .get_unregistered_events
                .send(eventlog::GetUnregisteredEvents),
        }
    }

    /// Returns a future that will return the app Build.
    /// - MailboxError should never happen
    pub fn get_build(
        &self,
        timeout: Option<time::Duration>,
    ) -> impl Future<Item = Option<Build>, Error = MailboxError> {
        match timeout {
            Some(duration) => self.get_build.send(app::GetBuild).timeout(duration),
            None => self.get_build.send(app::GetBuild),
        }
    }

    /// Returns a future that will return App instance id ULID.
    /// - MailboxError should never happen
    pub fn get_instance_id(
        &self,
        timeout: Option<time::Duration>,
    ) -> impl Future<Item = ULID, Error = MailboxError> {
        match timeout {
            Some(duration) => self
                .get_instance_id
                .send(app::GetInstanceId)
                .timeout(duration),
            None => self.get_instance_id.send(app::GetInstanceId),
        }
    }

    /// Returns a future that will return App instance info.
    /// - MailboxError should never happen
    pub fn get_app_instance_info(
        &self,
        timeout: Option<time::Duration>,
    ) -> impl Future<Item = app::AppInstanceInfo, Error = MailboxError> {
        match timeout {
            Some(duration) => self
                .get_app_instance_info
                .send(app::GetAppInstanceInfo)
                .timeout(duration),
            None => self.get_app_instance_info.send(app::GetAppInstanceInfo),
        }
    }

    /// Returns a future that will return App LogConfig.
    /// - MailboxError should never happen
    pub fn get_log_config(
        &self,
        timeout: Option<time::Duration>,
    ) -> impl Future<Item = Option<&'static oysterpack_log::LogConfig>, Error = MailboxError> {
        match timeout {
            Some(duration) => self
                .get_log_config
                .send(app::GetLogConfig)
                .timeout(duration),
            None => self.get_log_config.send(app::GetLogConfig),
        }
    }

    /// Returns a future that will return application registered services.
    /// - MailboxError should never happen
    pub fn get_registered_services(
        &self,
        timeout: Option<time::Duration>,
    ) -> impl Future<Item = HashMap<ServiceInfo, ServiceClient>, Error = MailboxError> {
        match timeout {
            Some(duration) => self
                .get_registered_services
                .send(app::GetRegisteredServices)
                .timeout(duration),
            None => self
                .get_registered_services
                .send(app::GetRegisteredServices),
        }
    }

    /// Returns a future that will return names of Arbiters that have been started
    /// - MailboxError should never happen
    pub fn get_arbiter_names(
        &self,
        timeout: Option<time::Duration>,
    ) -> impl Future<Item = Option<Vec<arbiters::Name>>, Error = MailboxError> {
        match timeout {
            Some(duration) => self
                .get_arbiter_names
                .send(arbiters::GetArbiterNames)
                .timeout(duration),
            None => self.get_arbiter_names.send(arbiters::GetArbiterNames),
        }
    }

    /// Returns a future that will return names of Arbiters that have been started
    /// - MailboxError should never happen
    pub fn get_arbiter(
        &self,
        name: arbiters::Name,
        timeout: Option<time::Duration>,
    ) -> impl Future<Item = Addr<Arbiter>, Error = MailboxError> {
        match timeout {
            Some(duration) => self
                .get_arbiter
                .send(arbiters::GetArbiter::from(name))
                .timeout(duration),
            None => self.get_arbiter.send(arbiters::GetArbiter::from(name)),
        }
    }
}

impl fmt::Debug for AppClient {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("AppClient")
    }
}

#[allow(warnings)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::run_test;
    use actix::{
        msgs::{Execute, StartActor},
        spawn, Arbiter, Supervised, System,
    };
    use futures::{future, prelude::*};
    use std::time::Duration;

    #[test]
    fn actor_service() {
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

        impl DisplayName for Foo {
            fn name() -> &'static str {
                "FOO"
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
                                Err(err) => panic!("GetServiceInfo failed: {:?}", err),
                            }
                            future::ok::<_, ()>(())
                        })
                        .then(move |_| {
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
                        })
                        .then(move |_| {
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
                        })
                        .then(|_| {
                            System::current().stop();
                            future::ok::<_, ()>(())
                        }),
                );
            });
        });

        run_test("GetServiceInfo - 256 tasks", || {
            System::run(|| {
                fn join_tasks(task_count: usize) -> impl Future {
                    let foo = Arbiter::registry().get::<Foo>();
                    let mut tasks = Vec::with_capacity(task_count);
                    for _ in 0..task_count {
                        tasks.push(foo.send(GetServiceInfo));
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

                Arbiter::spawn(join_tasks(TASK_BUCKET_COUNT).then(|_| {
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
                    })
                    .then(|_| {
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
                        let heartbeat = client.heartbeat(None);
                        let service_info = client.get_service_info(Some(Duration::from_millis(10)));
                        let arbiter_name = client.arbiter_name(None);

                        ping.then(move |pong| {
                            info!("{}", pong.unwrap());
                            service_info
                        })
                        .then(|service_info| {
                            info!("{}", service_info.unwrap());
                            heartbeat
                        })
                        .then(|heartbeat| {
                            info!("{:?}", heartbeat.unwrap());
                            arbiter_name
                        })
                        .then(|arbiter_name| {
                            info!("arbiter: {}", arbiter_name.unwrap());
                            future::ok::<_, ()>(())
                        })
                    })
                    .then(|_| {
                        System::current().stop();
                        future::ok::<_, ()>(())
                    });

                spawn_task(task);
            });
        });
    }

    #[test]
    fn app_client() {
        let log_config =
            oysterpack_log::config::LogConfigBuilder::new(oysterpack_log::Level::Info).build();
        app::App::run(
            crate::build::get(),
            log_config,
            future::lazy(|| {
                let app_client = AppClient::get();

                let get_app_instance_id = app_client.get_instance_id(None);
                let get_app_instance_info = app_client.get_app_instance_info(None);
                let get_build = app_client.get_build(None);
                let get_log_config = app_client.get_log_config(None);
                let get_registered_services = app_client.get_registered_services(None);

                let get_arbiter_names = app_client.get_arbiter_names(None);
                let get_arbiter = app_client.get_arbiter(arbiters::Name("FOO"), None);

                let get_registered_events = app_client.get_registered_events(None);
                let get_unregistered_events = app_client.get_unregistered_events(None);

                get_app_instance_id
                    .then(|app_instance_id| {
                        info!("app_instance_id: {:?}", app_instance_id.unwrap());
                        get_app_instance_info
                    })
                    .then(|app_instance_info| {
                        info!("app_instance_info: {:?}", app_instance_info.unwrap());
                        get_build
                    })
                    .then(|build| {
                        info!("build: {:?}", build.unwrap().unwrap().package().id());
                        get_log_config
                    })
                    .then(|log_config| {
                        info!("log_config: {:?}", log_config.unwrap());
                        get_registered_services
                    })
                    .then(|registered_services| {
                        info!("registered_services: {:?}", registered_services.unwrap());
                        get_arbiter_names
                    })
                    .then(|arbiter_names| {
                        info!("arbiter_names: {:?}", arbiter_names.unwrap());
                        get_arbiter
                    })
                    .then(|arbiter| {
                        info!("arbiter: {:?}", arbiter.unwrap());
                        get_registered_events
                    })
                    .then(|registered_events| {
                        info!("registered_events: {:?}", registered_events.unwrap());
                        get_unregistered_events
                    })
                    .then(|unregistered_events| {
                        info!("unregistered_events: {:?}", unregistered_events.unwrap());
                        future::ok::<(), ()>(())
                    })
            }),
        );
    }
}
