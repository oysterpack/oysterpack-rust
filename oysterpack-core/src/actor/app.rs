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

//! Application Actor is used to represent the application.
//! - each new instance is assigned a unique instance id, in the form of a TypedULID&lt;App&gt;
//! - run() is the key method used to run the application actor system
//!

use oysterpack_app_metadata::Build;
use oysterpack_log::LogConfig;
use oysterpack_uid::TypedULID;

use actor::{Id as ServiceId, InstanceId as ServiceInstanceId, ServiceInfo};

use actix::dev::{Handler, Message, MessageResult, System};
use futures::{future, prelude::Future};

/// App ServiceId (01CX5JGTT4VJE4XTJFD2564HTA)
pub const SERVICE_ID: ServiceId = ServiceId(1865558955258922375120216715788699466);

/// App represents an application instance.
#[derive(Debug)]
pub struct App {
    instance_id: TypedULID<App>,
    build: Option<Build>,
    service_info: ServiceInfo,
}

op_actor_service! {
    AppService(App)
}

impl Default for App {
    fn default() -> App {
        App {
            instance_id: TypedULID::generate(),
            build: None,
            service_info: ServiceInfo::for_new_actor_instance(SERVICE_ID),
        }
    }
}

impl App {
    /// Runs the App actor System.
    /// - log system is initialized
    /// - the Build is stored
    /// - the supplied future is spawned
    ///   - the supplied future is the application workflow
    /// - once the supplied future completes, then the System is stopped
    pub fn run<F>(build: Build, log_config: LogConfig, f: F) -> i32
    where
        F: Future<Item = (), Error = ()> + 'static,
    {
        System::run(move || {
            let task = crate::actor::logger::init_logging(log_config)
                .then(move |_| {
                    let app = System::current().registry().get::<App>();
                    app.send(SetBuild(build))
                }).then(|_| f)
                .then(|_| {
                    let app = System::current().registry().get::<App>();
                    app.send(StopApp)
                });
            crate::actor::spawn_task(task);
        })
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
    type Result = TypedULID<App>;
}

impl Handler<GetInstanceId> for App {
    type Result = MessageResult<GetInstanceId>;

    fn handle(&mut self, _: GetInstanceId, _: &mut Self::Context) -> Self::Result {
        MessageResult(self.instance_id)
    }
}

#[allow(warnings)]
#[cfg(test)]
mod tests {

    use super::*;
    use actix::dev::System;
    use crate::actor::logger::init_logging;
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
                app.send(SetBuild(::build::get()))
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
            ::build::get(),
            log_config(),
            future::lazy(|| {
                info!("The next wave is blockchain ...");
                let app = System::current().registry().get::<App>();
                app.send(GetInstanceId)
            }).then(|app_instance_id| {
                info!("app_instance_id = {}", app_instance_id.unwrap());
                future::ok::<(), ()>(())
            }),
        );
    }

}
