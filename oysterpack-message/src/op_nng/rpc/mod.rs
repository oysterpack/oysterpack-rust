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

//! Provides support for a request/reply RPC-like services.

use std::panic::{RefUnwindSafe, UnwindSafe};

pub mod client;
pub mod server;

/// MessageProcessor factory
pub trait MessageProcessorFactory<T, Req, Rep>: Send + Sync + 'static
where
    Req: Send + 'static,
    Rep: Send + 'static,
    T: MessageProcessor<Req, Rep>,
{
    /// returns a new MessageProcessor instance
    fn new(&self) -> T;
}

/// Message handler that implements a request/reply protocol pattern
pub trait MessageProcessor<Req, Rep>:
    Send + Sync + RefUnwindSafe + UnwindSafe + 'static
where
    Req: Send + 'static,
    Rep: Send + 'static,
{
    /// processes the request message and returns a reply message
    fn process(&mut self, req: Req) -> Rep;
}
