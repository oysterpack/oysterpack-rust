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

use std::thread;

/// Message handler that implements a request/reply protocol pattern
pub trait MessageHandler {
    /// processes the request and returns a response
    fn handle(&mut self, req: nng::Message) -> nng::Message;
}

/// Represents a logical socket connection
pub struct Connection {
    channel_capacity: usize,
    message_handler_thread: Option<thread::JoinHandle<()>>,
    message_handler_thread_config: Option<ThreadConfig>
}

/// Thread config
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ThreadConfig {
    name: String,
    stack_size: usize
}
