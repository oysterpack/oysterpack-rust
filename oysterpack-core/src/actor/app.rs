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

//! Application Actor is used to represent the application.
//! - each new instance is assigned a unique instance id, in the form of a TypedULID&lt;App&gt;
//! - run() is the key method used to run the application actor system

use oysterpack_app_metadata::{Build, PackageId};
use oysterpack_events::Eventful;
use oysterpack_log::{self, LogConfig};
use oysterpack_uid::ULID;

use crate::actor::{
    eventlog::{EventLog, LogEvent},
    events, AppClient, AppService, DisplayName, Id as ServiceId, InstanceId as ServiceInstanceId,
    ServiceClient, ServiceInfo,
};

use actix::dev::{Handler, Message, MessageResult, System};
use futures::{future, prelude::Future};
use std::{collections::HashMap, fmt, time};

/// App ServiceId (01CX5JGTT4VJE4XTJFD2564HTA)
pub const SERVICE_ID: ServiceId = ServiceId(1865558955258922375120216715788699466);

// TODO: handle OS signals - see https://crates.io/crates/signal-hook
/// App represents an application instance.
#[derive(Debug)]
pub struct App {
    build: Option<Build>,
    service_info: ServiceInfo,
    service_registry: HashMap<ServiceInfo, ServiceClient>,
}

op_actor_service! {
    AppService(App)
}

impl Default for App {
    fn default() -> App {
        App {
            build: None,
            service_info: ServiceInfo::for_new_actor_instance(SERVICE_ID, Self::TYPE),
            service_registry: HashMap::new(),
        }
    }
}

impl App {
    /// Runs the App actor System.
    /// - initializes the sodium library and chooses faster versions of the primitives if possible.
    ///   - also makes the random number generation functions (gen_key, gen_keypair, gen_nonce,
    ///     randombytes, randombytes_into) thread-safe
    /// - log system is initialized
    /// - the Build is stored
    /// - AppLifeCycleEvent Started event is logged
    /// - the supplied future is spawned
    ///   - the supplied future is the application workflow
    /// - after the supplied future completes:
    ///   - AppLifeCycleEvent Stopped event is logged
    ///   - the System is stopped
    ///
    /// ## Panics
    /// if sodium library initialization fails
    pub fn run<F>(build: Build, log_config: LogConfig, f: F) -> i32
    where
        F: Future<Item = (), Error = ()> + 'static,
    {
        sodiumoxide::init().expect("Failed to initialize the sodium library");
        System::run(move || {
            let task = crate::actor::logger::init_logging(log_config)
                .then(move |_| {
                    let app = System::current().registry().get::<App>();
                    app.send(SetBuild(build))
                        .then(move |_| app.send(GetAppInstanceInfo))
                })
                .then(|appinstance_info| {
                    let appinstance_info = appinstance_info.unwrap();
                    let eventlog = System::current().registry().get::<EventLog>();
                    let event = events::AppLifeCycleEvent::started(
                        appinstance_info.package_id().clone(),
                        appinstance_info.instance_id(),
                    );
                    eventlog.send(LogEvent(op_event!(event)))
                })
                .then(|_| f)
                .then(|_| {
                    let app = System::current().registry().get::<App>();
                    app.send(GetAppInstanceInfo)
                        .then(|appinstance_info| {
                            let appinstance_info = appinstance_info.unwrap();
                            let eventlog = System::current().registry().get::<EventLog>();
                            let event = events::AppLifeCycleEvent::stopped(
                                appinstance_info.package_id().clone(),
                                appinstance_info.instance_id(),
                            );
                            eventlog.send(LogEvent(op_event!(event)))
                        })
                        .then(move |_| app.send(StopApp))
                });
            crate::actor::spawn_task(task);
        })
    }

    /// AppInstanceInfo getter
    pub fn app_instance_info(&self) -> AppInstanceInfo {
        AppInstanceInfo {
            package_id: self
                .build
                .as_ref()
                .map_or_else(PackageId::for_this_crate, |build| {
                    build.package().id().clone()
                }),
            instance_id: self.service_info.instance_id().ulid(),
        }
    }
}

impl crate::actor::LifeCycle for App {}

impl DisplayName for App {
    fn name() -> &'static str {
        "App"
    }
}

/// SetBuild Request
#[derive(Debug, Clone)]
pub struct SetBuild(Build);

impl From<Build> for SetBuild {
    fn from(build: Build) -> SetBuild {
        SetBuild(build)
    }
}

impl Message for SetBuild {
    type Result = ();
}

impl Handler<SetBuild> for App {
    type Result = MessageResult<SetBuild>;

    fn handle(&mut self, msg: SetBuild, _: &mut Self::Context) -> Self::Result {
        self.build = Some(msg.0);
        MessageResult(())
    }
}

/// GetBuild Request
#[derive(Debug, Copy, Clone)]
pub struct GetBuild;

impl Message for GetBuild {
    type Result = Option<Build>;
}

impl Handler<GetBuild> for App {
    type Result = MessageResult<GetBuild>;

    fn handle(&mut self, _: GetBuild, _: &mut Self::Context) -> Self::Result {
        match self.build.as_ref() {
            None => MessageResult(None),
            Some(build) => MessageResult(Some(build.clone())),
        }
    }
}

/// StopApp Request
#[derive(Debug, Copy, Clone)]
pub struct StopApp;

impl Message for StopApp {
    type Result = ();
}

impl Handler<StopApp> for App {
    type Result = MessageResult<StopApp>;

    fn handle(&mut self, _: StopApp, _: &mut Self::Context) -> Self::Result {
        System::current().stop();
        MessageResult(())
    }
}

/// GetInstanceId Request
/// - when an App is started, it assigns itself a new instance id.
#[derive(Debug, Copy, Clone)]
pub struct GetInstanceId;

impl Message for GetInstanceId {
    type Result = ULID;
}

impl Handler<GetInstanceId> for App {
    type Result = MessageResult<GetInstanceId>;

    fn handle(&mut self, _: GetInstanceId, _: &mut Self::Context) -> Self::Result {
        MessageResult(self.service_info.instance_id().ulid())
    }
}

/// GetInstanceId Request
/// - when an App is started, it assigns itself a new instance id.
#[derive(Debug, Copy, Clone)]
pub struct GetAppInstanceInfo;

/// App instance info
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct AppInstanceInfo {
    package_id: PackageId,
    instance_id: ULID,
}

impl AppInstanceInfo {
    /// PackageId getter
    pub fn package_id(&self) -> &PackageId {
        &self.package_id
    }

    /// is the same as the service instance ID
    pub fn instance_id(&self) -> ULID {
        self.instance_id
    }
}

impl Message for GetAppInstanceInfo {
    type Result = AppInstanceInfo;
}

impl fmt::Display for AppInstanceInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({},{})", self.package_id, self.instance_id)
    }
}

impl Handler<GetAppInstanceInfo> for App {
    type Result = MessageResult<GetAppInstanceInfo>;

    fn handle(&mut self, _: GetAppInstanceInfo, _: &mut Self::Context) -> Self::Result {
        MessageResult(self.app_instance_info())
    }
}

/// GetLogConfig Request
/// - returns the LogConfig that is in use
#[derive(Debug, Copy, Clone)]
pub struct GetLogConfig;

impl Message for GetLogConfig {
    /// When using App::run(), this will always return some LogConfig.
    type Result = Option<&'static LogConfig>;
}

impl Handler<GetLogConfig> for App {
    type Result = MessageResult<GetLogConfig>;

    fn handle(&mut self, _: GetLogConfig, _: &mut Self::Context) -> Self::Result {
        MessageResult(oysterpack_log::config())
    }
}

/// RegisterService Message request
#[derive(Debug, Clone)]
pub struct RegisterService {
    key: ServiceInfo,
    service_client: ServiceClient,
}

impl Message for RegisterService {
    type Result = ();
}

impl RegisterService {
    /// constructor
    pub fn new(key: ServiceInfo, service_client: ServiceClient) -> RegisterService {
        RegisterService {
            key,
            service_client,
        }
    }
}

impl Handler<RegisterService> for App {
    type Result = MessageResult<RegisterService>;

    fn handle(&mut self, req: RegisterService, _: &mut Self::Context) -> Self::Result {
        self.service_registry.insert(req.key, req.service_client);
        MessageResult(())
    }
}

/// GetRegisteredServices
#[derive(Debug, Clone, Copy)]
pub struct GetRegisteredServices;

impl Message for GetRegisteredServices {
    type Result = HashMap<ServiceInfo, ServiceClient>;
}

impl Handler<GetRegisteredServices> for App {
    type Result = MessageResult<GetRegisteredServices>;

    fn handle(&mut self, _: GetRegisteredServices, _: &mut Self::Context) -> Self::Result {
        MessageResult(self.service_registry.clone())
    }
}

#[allow(warnings)]
#[cfg(test)]
mod tests {

    use super::*;
    use crate::actor::logger::init_logging;
    use actix::dev::System;
    use futures::{future, prelude::*};

    use oysterpack_log;
    use serde_json;

    fn log_config() -> oysterpack_log::LogConfig {
        oysterpack_log::config::LogConfigBuilder::new(oysterpack_log::Level::Info).build()
    }

    #[test]
    fn app_build_msgs() {
        let code = System::run(|| {
            let task = init_logging(log_config());
            let task = task.then(|_| {
                let app = System::current().registry().get::<App>();
                app.send(GetBuild)
            });
            let task = task.then(|build| {
                let build = build.unwrap();
                if build.is_some() {
                    panic!("There should be no Build set");
                }
                let app = System::current().registry().get::<App>();
                app.send(SetBuild(crate::build::get()))
            });
            let task = task.then(|_| {
                let app = System::current().registry().get::<App>();
                app.send(GetBuild)
            });
            let task = task.then(|build| {
                let build = build.unwrap().unwrap();
                info!(
                    "build: {}",
                    serde_json::to_string_pretty(&build.package().id()).unwrap()
                );
                future::ok::<(), ()>(())
            });
            let task = task.then(|_| {
                System::current().stop();
                future::ok::<(), ()>(())
            });
            // it's ok if the system is stopped again
            let task = task.then(|_| {
                System::current().stop();
                future::ok::<(), ()>(())
            });
            crate::actor::spawn_task(task);
        });
    }

    #[test]
    fn app_run() {
        App::run(
            crate::build::get(),
            log_config(),
            future::lazy(|| {
                info!("The next wave is blockchain ...");
                let app = System::current().registry().get::<App>();
                app.send(GetInstanceId)
            })
            .then(|app_instance_id| {
                info!("app_instance_id = {}", app_instance_id.unwrap());
                let app = System::current().registry().get::<App>();
                app.send(GetAppInstanceInfo)
            })
            .then(|app_instance_info| {
                info!("app_instance_info = {}", app_instance_info.unwrap());
                let app = System::current().registry().get::<App>();
                app.send(GetLogConfig)
            })
            .then(|logconfig| {
                info!("logconfig = {}", logconfig.unwrap().unwrap());
                let app = System::current().registry().get::<App>();
                app.send(GetRegisteredServices)
            })
            .then(|registered_services| {
                info!("registered_services = {:?}", registered_services.unwrap());
                future::ok::<(), ()>(())
            }),
        );
    }

}
