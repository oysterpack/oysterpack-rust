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

//! macros

/// internally used by the op_actor_service!() macro to generate code to log events
#[macro_export]
macro_rules! __op_log_event_for_service {
    ( $self:expr, $event:expr ) => {
        use $crate::oysterpack_events::Eventful;
        let event = $crate::actor::events::ServiceLifeCycleEvent::for_service($self, $event)
            .new_event(op_module_source!());

        use $crate::actor::eventlog::{EventLog, LogEvent};
        let eventlog = $crate::actix::dev::System::current()
            .registry()
            .get::<EventLog>();
        eventlog.do_send(LogEvent(event));
    };
}

/// internally used by the op_actor_service!() macro to generate code to log events
#[macro_export]
macro_rules! __op_log_event_for_app_service {
    ( $self:expr, $event:expr ) => {
        use $crate::oysterpack_events::Eventful;
        let event = $crate::actor::events::ServiceLifeCycleEvent::for_app_service($self, $event)
            .new_event(op_module_source!());

        use $crate::actor::eventlog::{EventLog, LogEvent};
        let eventlog = $crate::actix::dev::System::current()
            .registry()
            .get::<EventLog>();
        eventlog.do_send(LogEvent(event));
    };
}

/// internally used by the op_actor_service!() macro to generate code to log events
#[macro_export]
macro_rules! __op_service_handlers {
    ( $name:ident ) => {
        impl $crate::actix::dev::Handler<$crate::actor::GetServiceInfo> for $name {
            type Result = $crate::actix::MessageResult<$crate::actor::GetServiceInfo>;

            fn handle(
                &mut self,
                _: $crate::actor::GetServiceInfo,
                _: &mut Self::Context,
            ) -> Self::Result {
                $crate::actix::MessageResult(self.service_info)
            }
        }

        impl $crate::actix::dev::Handler<$crate::actor::Ping> for $name {
            type Result = $crate::actix::MessageResult<$crate::actor::Ping>;

            fn handle(&mut self, ping: $crate::actor::Ping, _: &mut Self::Context) -> Self::Result {
                $crate::actix::MessageResult($crate::actor::Pong::from(ping))
            }
        }

        impl $crate::actix::dev::Handler<$crate::actor::GetArbiterName> for $name {
            type Result = $crate::actix::MessageResult<$crate::actor::GetArbiterName>;

            fn handle(
                &mut self,
                _: $crate::actor::GetArbiterName,
                _: &mut Self::Context,
            ) -> Self::Result {
                $crate::actix::MessageResult($crate::actor::ArbiterName::new(
                    $crate::actix::Arbiter::name(),
                ))
            }
        }

        impl $crate::actix::dev::Handler<$crate::actor::GetDisplayName> for $name {
            type Result = $crate::actix::MessageResult<$crate::actor::GetDisplayName>;

            fn handle(&mut self, _: $crate::actor::GetDisplayName, _: &mut Self::Context) -> Self::Result {
                $crate::actix::MessageResult(<Self as $crate::actor::DisplayName>::name())
            }
        }
    };
}

/// Generates Actor service boilerplate code, which enables the developer to focus on the business logic,
/// i.e., writing message handlers. It provides implementations for the following traits:
/// - actix::dev::Actor
///   - logs lifecycle event
///   - invokes any actor lifecycle method, i.e., actor::LifeCycle
/// - actix::dev::ArbiterService / actix::dev::SystemService
///   - actor::Service -&gt; ArbiterService
///   - actor::AppService -&gt; SystemService
/// - actor::Service / actor::AppServiceService
/// - actix::dev::Supervised
/// - actix::dev::Handler<actor::GetServiceInfo>
/// - actix::dev::Handler<actor::Ping>
/// - actix::dev::Handler<actor::GetArbiterName>
/// - actix::dev::Handler<actor::GetDisplayName>
#[macro_export]
macro_rules! op_actor_service {
    ( Service($name:ident) ) => {
        impl $crate::actix::dev::Actor for $name {
            type Context = $crate::actix::dev::Context<Self>;

            fn started(&mut self, c: &mut Self::Context) {
                $crate::__op_log_event_for_service!(
                    self,
                    $crate::actor::events::ServiceLifeCycle::Started
                );

                let app: $crate::actix::Addr<$crate::actor::app::App> = $crate::actor::app_service();
                use $crate::actix::prelude::AsyncContext;
                let service_client = $crate::actor::ServiceClient::for_service(c.address());
                let register_service = app.send($crate::actor::app::RegisterService::new(self.service_info, service_client));
                $crate::actor::spawn_task(register_service);

                use $crate::actor::LifeCycle;
                self.on_started(c);
            }

            fn stopped(&mut self, c: &mut Self::Context) {
                $crate::__op_log_event_for_service!(
                    self,
                    $crate::actor::events::ServiceLifeCycle::Stopped
                );
                use $crate::actor::LifeCycle;
                self.on_stopped(c);
            }

            fn stopping(&mut self, c: &mut Self::Context) -> actix::Running {
                use $crate::actor::LifeCycle;
                self.on_stopping(c)
            }
        }

        impl $crate::actix::dev::ArbiterService for $name {
            fn service_started(&mut self, c: &mut Self::Context) {
                $crate::__op_log_event_for_service!(
                    self,
                    $crate::actor::events::ServiceLifeCycle::ServiceStarted
                );
                use $crate::actor::LifeCycle;
                self.on_service_started(c);
            }
        }

        impl $crate::actix::dev::Supervised for $name {
            fn restarting(&mut self, c: &mut Self::Context) {
                $crate::__op_log_event_for_service!(
                    self,
                    $crate::actor::events::ServiceLifeCycle::Restarting
                );
                use $crate::actor::LifeCycle;
                self.on_restarting(c);
            }
        }

        impl $crate::actor::Service for $name {
            fn id(&self) -> $crate::actor::Id {
                self.service_info.id
            }

            fn instance_id(&self) -> $crate::actor::InstanceId {
                self.service_info.instance_id
            }
        }

        $crate::__op_service_handlers! { $name }

        impl $crate::actix::dev::Handler<$crate::actor::GetServiceClient> for $name {
            type Result = $crate::actix::MessageResult<$crate::actor::GetServiceClient>;

            fn handle(
                &mut self,
                _: $crate::actor::GetServiceClient,
                c: &mut Self::Context,
            ) -> Self::Result {
                use $crate::actix::prelude::AsyncContext;
                $crate::actix::MessageResult($crate::actor::ServiceClient::for_service(c.address()))
            }
        }
    };
    ( AppService($name:ident) ) => {
        impl $crate::actix::dev::Actor for $name {
            type Context = $crate::actix::dev::Context<Self>;

            fn started(&mut self, c: &mut Self::Context) {
                $crate::__op_log_event_for_app_service!(
                    self,
                    $crate::actor::events::ServiceLifeCycle::Started
                );

                if self.service_info.id() != $crate::actor::app::SERVICE_ID {
                    let app: $crate::actix::Addr<$crate::actor::app::App> = $crate::actor::app_service();
                    use $crate::actix::prelude::AsyncContext;
                    let service_client = $crate::actor::ServiceClient::for_app_service(c.address());
                    let register_service = app.send($crate::actor::app::RegisterService::new(self.service_info, service_client));
                    $crate::actor::spawn_task(register_service);
                }

                use $crate::actor::LifeCycle;
                self.on_started(c);
            }

            fn stopped(&mut self, c: &mut Self::Context) {
                $crate::__op_log_event_for_app_service!(
                    self,
                    $crate::actor::events::ServiceLifeCycle::Stopped
                );
                use $crate::actor::LifeCycle;
                self.on_stopped(c);
            }

            fn stopping(&mut self, c: &mut Self::Context) -> actix::Running {
                use $crate::actor::LifeCycle;
                self.on_stopping(c)
            }
        }

        impl $crate::actix::dev::SystemService for $name {
            fn service_started(&mut self, c: &mut Self::Context) {
                $crate::__op_log_event_for_app_service!(
                    self,
                    $crate::actor::events::ServiceLifeCycle::ServiceStarted
                );
                use $crate::actor::LifeCycle;
                self.on_service_started(c);
            }
        }

        impl $crate::actix::dev::Supervised for $name {
            fn restarting(&mut self, c: &mut Self::Context) {
                $crate::__op_log_event_for_app_service!(
                    self,
                    $crate::actor::events::ServiceLifeCycle::Restarting
                );
                use $crate::actor::LifeCycle;
                self.on_restarting(c);
            }
        }

        impl $crate::actor::AppService for $name {
            fn id(&self) -> $crate::actor::Id {
                self.service_info.id
            }

            fn instance_id(&self) -> $crate::actor::InstanceId {
                self.service_info.instance_id
            }
        }

        $crate::__op_service_handlers! { $name }

        impl $crate::actix::dev::Handler<$crate::actor::GetServiceClient> for $name {
            type Result = $crate::actix::MessageResult<$crate::actor::GetServiceClient>;

            fn handle(
                &mut self,
                _: $crate::actor::GetServiceClient,
                c: &mut Self::Context,
            ) -> Self::Result {
                use $crate::actix::prelude::AsyncContext;
                $crate::actix::MessageResult($crate::actor::ServiceClient::for_app_service(
                    c.address(),
                ))
            }
        }
    };
}
