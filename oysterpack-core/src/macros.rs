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

/// Generates Actor service boilerplate code, which enables the developer to focus on the business logic,
/// i.e., writing message handlers.
#[macro_export]
macro_rules! op_actor_service {
    ( Service($name:ident) ) => {
        impl Actor for $name {
            type Context = Context<Self>;

            fn started(&mut self, _: &mut Self::Context) {
                let event =
                    events::ServiceLifeCycleEvent::for_service(self, events::LifeCycle::Started)
                        .new_event(op_module_source!());
                event.log_pretty();
            }

            fn stopped(&mut self, _: &mut Self::Context) {
                let event =
                    events::ServiceLifeCycleEvent::for_service(self, events::LifeCycle::Stopped)
                        .new_event(op_module_source!());
                event.log_pretty();
            }
        }

        impl ArbiterService for $name {
            fn service_started(&mut self, _: &mut Context<Self>) {
                let event = events::ServiceLifeCycleEvent::for_service(
                    self,
                    events::LifeCycle::ServiceStarted,
                ).new_event(op_module_source!());
                event.log_pretty();
            }
        }

        impl Supervised for $name {
            fn restarting(&mut self, _: &mut Self::Context) {
                let event =
                    events::ServiceLifeCycleEvent::for_service(self, events::LifeCycle::Restarting)
                        .new_event(op_module_source!());
                event.log_pretty();
            }
        }

        impl Service for $name {
            fn id(&self) -> actor::Id {
                self.service_info.id
            }

            fn instance_id(&self) -> actor::InstanceId {
                self.service_info.instance_id
            }
        }

        impl Handler<GetServiceInfo> for $name {
            type Result = ServiceInfo;

            fn handle(&mut self, _: GetServiceInfo, _: &mut Self::Context) -> Self::Result {
                self.service_info
            }
        }
    };
    ( AppService($name:ident) ) => {
        impl Actor for $name {
            type Context = Context<Self>;

            fn started(&mut self, _: &mut Self::Context) {
                let event = events::ServiceLifeCycleEvent::for_app_service(
                    self,
                    events::LifeCycle::Started,
                ).new_event(op_module_source!());
                event.log_pretty();
            }

            fn stopped(&mut self, _: &mut Self::Context) {
                let event = events::ServiceLifeCycleEvent::for_app_service(
                    self,
                    events::LifeCycle::Stopped,
                ).new_event(op_module_source!());
                event.log_pretty();
            }
        }

        impl SystemService for $name {
            fn service_started(&mut self, _: &mut Context<Self>) {
                let event = events::ServiceLifeCycleEvent::for_app_service(
                    self,
                    events::LifeCycle::ServiceStarted,
                ).new_event(op_module_source!());
                event.log_pretty();
            }
        }

        impl Supervised for $name {
            fn restarting(&mut self, _: &mut Self::Context) {
                let event = events::ServiceLifeCycleEvent::for_app_service(
                    self,
                    events::LifeCycle::Restarting,
                ).new_event(op_module_source!());
                event.log_pretty();
            }
        }

        impl AppService for $name {
            fn id(&self) -> actor::Id {
                self.service_info.id
            }

            fn instance_id(&self) -> actor::InstanceId {
                self.service_info.instance_id
            }
        }

        impl Handler<GetServiceInfo> for $name {
            type Result = ServiceInfo;

            fn handle(&mut self, _: GetServiceInfo, _: &mut Self::Context) -> Self::Result {
                self.service_info
            }
        }
    };
}
