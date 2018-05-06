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

//! # ServiceActor module
//!
//! ## Features
//!
//! ### Service Instance Tracking
//! 1. Service Actor instances register themselved in order to be tracked
//!
//! ### Service State
//! 1. Services can have persistent state
//! 2. Service state can be saved to persistent storage.
//! 3. Service state is stored as encrypted
//!     - each service has a separate encryption key, i.e., mapped to its ServiceId
//!     - while a service is running, it will generate a new temporary encryption key to store state
//!         - this will prevent others from trying to read or write the service state
//!         - before stopping the service will re-encrypt the state with the service encryption key
//!         - this will enable the service state to be restored into a new service instance
//! 3. Service state can be restored from persistent storage
//!     - the service key is used to decrypt the service state
//! 4. Service state can be deleted from persistent storage.
//!
//! ### Service Config
//!
//! ### Service Context
//!
//! ### Service Events
//!
//! ### Service Metrics
//!
//! ### Service HealthChecks
//!

extern crate actix;
extern crate chrono;
extern crate oysterpack_platform;
extern crate serde;

use self::serde::{Serialize, de::DeserializeOwned};
use std::{fmt::Debug, marker::PhantomData};
use self::oysterpack_platform::{Service, ServiceInstance};
use self::chrono::prelude::*;
use self::actix::*;

/// Represents a Service running as an Actor
///
/// T: Actor type marker. This is used to assign unique types to Actors in a generic manner.
/// ```rust
/// # use oysterpack_actors::*;
/// struct Foo;
/// struct Bar;
///
/// type FooActor = service::ServiceActor<Foo, Nil, Nil, Nil>;
/// type BarActor = service::ServiceActor<Bar, Nil, Nil, Nil>;
/// ```
/// State: persistent service state
/// Cfg: persistent service config
/// Ctx: additional context that is required by the service, e.g., actor addresses
pub struct ServiceActor<T, State, Cfg, Ctx>
where
    T: 'static,
    State: ServiceState,
    Cfg: ServiceConfig,
    Ctx: ServiceContext,
{
    _type: PhantomData<T>,
    context: ServiceActorContext<State, Cfg, Ctx>,
    service_instance: ServiceInstance,
    created_on: DateTime<Utc>,
    started_on: Option<DateTime<Utc>>,
    lifecycle: Option<Box<ServiceActorLifecycle<T, State, Cfg, Ctx>>>,
}

impl<T, State, Cfg, Ctx> ServiceActor<T, State, Cfg, Ctx>
where
    T: 'static,
    State: ServiceState,
    Cfg: ServiceConfig,
    Ctx: ServiceContext,
{
    /// Returns ServiceActorContext<State, Cfg, Ctx>
    pub fn context(&self) -> &ServiceActorContext<State, Cfg, Ctx> {
        &self.context
    }

    pub fn service_instance(&self) -> &ServiceInstance {
        &self.service_instance
    }
}

impl<T, State, Cfg, Ctx> Actor for ServiceActor<T, State, Cfg, Ctx>
where
    T: 'static,
    State: ServiceState,
    Cfg: ServiceConfig,
    Ctx: ServiceContext,
{
    type Context = Context<Self>;

    /// - records the start timestamp
    /// - logs a debug log message indicated that it is started
    fn started(&mut self, ctx: &mut Self::Context) {
        self.started_on = Some(Utc::now());
        if let Some(ref mut lifecycle) = self.lifecycle {
            lifecycle.started(ctx, &mut self.context);
        }
        debug!("{} started", self.service_instance);
    }

    fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
        let running = if let Some(ref mut lifecycle) = self.lifecycle {
            lifecycle.stopping(ctx, &mut self.context)
        } else {
            Running::Stop
        };
        debug!("{} stopping", self.service_instance);
        running
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        if let Some(ref mut lifecycle) = self.lifecycle {
            lifecycle.stopped(ctx, &mut self.context)
        }
        debug!("{} stopped", self.service_instance);
    }
}

pub struct ServiceActorBuilder<T, State, Cfg, Ctx>
where
    T: 'static,
    State: ServiceState,
    Cfg: ServiceConfig,
    Ctx: ServiceContext,
{
    actor: ServiceActor<T, State, Cfg, Ctx>,
}

impl<T, State, Cfg, Ctx> ServiceActorBuilder<T, State, Cfg, Ctx>
where
    T: 'static,
    State: ServiceState,
    Cfg: ServiceConfig,
    Ctx: ServiceContext,
{
    /// Creates new ServiceActorBuilder
    pub fn new(service: Service) -> Self {
        ServiceActorBuilder {
            actor: ServiceActor {
                _type: PhantomData,
                context: Default::default(),
                service_instance: ServiceInstance::new(service),
                created_on: Utc::now(),
                started_on: None,
                lifecycle: None,
            },
        }
    }

    /// Sets service state
    pub fn state(&mut self, state: State) -> &Self {
        self.actor.context.state = Some(state);
        self
    }

    /// Sets service config
    pub fn config(&mut self, config: Cfg) -> &Self {
        self.actor.context.config = Some(config);
        self
    }

    /// Sets service context
    pub fn context(&mut self, context: Ctx) -> &Self {
        self.actor.context.context = Some(context);
        self
    }

    /// Sets service context
    pub fn lifecycle(
        &mut self,
        lifecycle: Box<ServiceActorLifecycle<T, State, Cfg, Ctx>>,
    ) -> &Self {
        self.actor.lifecycle = Some(lifecycle);
        self
    }

    /// Returns a new ServiceActor instance
    pub fn build(self) -> ServiceActor<T, State, Cfg, Ctx> {
        self.actor
    }
}

/// Marker trait for service state.
/// - is serializable because this will enable the service state to be persisted
/// - state is able to be sent across threads
/// - it can be cloned
/// - its lifetime is marked as static because actix::Actor is marked as static
pub trait ServiceState
    : 'static + Serialize + DeserializeOwned + Debug + Clone + Send {
}

/// Marker trait for service configuration.
/// - is serializable because this will enable the service state to be persisted
/// - state is able to be sent across threads
/// - it can be cloned
/// - its lifetime is marked as static because actix::Actor is marked as static
pub trait ServiceConfig
    : 'static + Serialize + DeserializeOwned + Debug + Clone + Send {
}

/// Represents additional context that is required by the service, e.g., actor addresses,
/// DB Connection Pool
/// - its lifetime is marked as static because actix::Actor is marked as static
pub trait ServiceContext: 'static + Debug {}

/// ServiceActor context provides state, config, and actor specific context.
#[derive(Debug)]
pub struct ServiceActorContext<State, Cfg, Ctx>
where
    State: ServiceState,
    Cfg: ServiceConfig,
    Ctx: ServiceContext,
{
    state: Option<State>,
    config: Option<Cfg>,
    context: Option<Ctx>,
}

impl<State, Cfg, Ctx> ServiceActorContext<State, Cfg, Ctx>
where
    State: ServiceState,
    Cfg: ServiceConfig,
    Ctx: ServiceContext,
{
    /// Returns ServiceState
    pub fn state(&self) -> &Option<State> {
        &self.state
    }

    /// Returns ServiceConfig
    pub fn config(&self) -> &Option<Cfg> {
        &self.config
    }

    /// Returns ServiceContext
    pub fn context(&self) -> &Option<Ctx> {
        &self.context
    }
}

impl<State, Cfg, Ctx> Default for ServiceActorContext<State, Cfg, Ctx>
where
    State: ServiceState,
    Cfg: ServiceConfig,
    Ctx: ServiceContext,
{
    fn default() -> Self {
        ServiceActorContext {
            state: None,
            config: None,
            context: None,
        }
    }
}

/// ServiceActor lifecycle
pub trait ServiceActorLifecycle<T, State, Cfg, Ctx>
where
    T: 'static,
    State: ServiceState,
    Cfg: ServiceConfig,
    Ctx: ServiceContext,
    Self: 'static,
{
    fn started(
        &mut self,
        actor_ctx: &mut Context<ServiceActor<T, State, Cfg, Ctx>>,
        service_ctx: &mut ServiceActorContext<State, Cfg, Ctx>,
    ) {
    }

    fn stopping(
        &mut self,
        actor_ctx: &mut Context<ServiceActor<T, State, Cfg, Ctx>>,
        service_ctx: &mut ServiceActorContext<State, Cfg, Ctx>,
    ) -> Running {
        Running::Stop
    }

    fn stopped(
        &mut self,
        actor_ctx: &mut Context<ServiceActor<T, State, Cfg, Ctx>>,
        service_ctx: &mut ServiceActorContext<State, Cfg, Ctx>,
    ) {
    }
}

/// Used for Nil Service state, config, or context.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Nil;

impl ServiceState for Nil {}

impl ServiceConfig for Nil {}

impl ServiceContext for Nil {}

/// Has no state, config, or context
pub type StatelessServiceActor<T> = ServiceActor<T, Nil, Nil, Nil>;
