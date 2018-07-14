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

//! The command pattern is the core OysterPack pattern. All program logic should be designed and
//! implemented as commands.
//!
//! Command execution is standardized and provides support for :
//! - logging
//! - metrics
//! - healthchecks
//! - events
//! - config
//! - security via command based permissioning
//! - error logging
//! - cancelling execution
//! - timeouts
//! - retries
//! - sync execution
//! - async execution
//! - scheduled execution
//!
//! Commands are also self describing and have metadata:
//! - documentation
//! - metrics
//! - events
//! - errors
//! - healthchecks
//! - config
//! - tags
//! - type : Query, Mutation
//! - default timeout
//! - retry config
//!
//! Commands are functional. Commands can be composed using other commands.
//!
//!
//! Related commands are organized into catalogs. Catalogs can be organized into hierarchies in the
//! same way that modules can form a hierarchy. Think of commands belonging to a catalog path analagous
//! to a file system.
//!

use std::{
    fmt::Debug, time::{Duration, Instant},
};
use tokio::prelude::*;

/// Command is a Future that executes the underlying Future.
///
/// The underlying future is fused. Normally futures can behave unpredictable once they're used
/// after a future has been resolved. The fused Future is always defined to return Async::NotReady
/// from poll after it has resolved successfully or returned an error.
///
/// The following additional information is collected:
/// - poll counter
///   - is incremented each time the future is polled until it is done
///   - polling the command future after it is done will not increment the counter
/// - created timestamp
///   - when the command future instance was created
/// - first polled timestamp
///   - when the command future was first polled
/// - last polled timestamp
///   - when the command future was last polled
/// - completed timestamp
///   - when the command future completed
/// - success
///   - did the command resolve successfully
///
///
#[derive(Debug)]
pub struct Command<T, E, F>
where
    T: Send + Debug,
    E: Send + Debug,
    F: Future<Item = T, Error = E>,
{
    // the underlying future is fused
    fut: future::Fuse<F>,
    status: CommandStatus,

    // used to track the number of times the future has been polled
    poll_counter: usize,
    // when the future instance was created
    created: Instant,
    // when the future instance was last polled
    first_polled: Option<Instant>,
    // when the future instance was last polled
    last_polled: Option<Instant>,
    // when the future completed
    completed: Option<Instant>,
    poll_duration: Duration,
}

impl<T, E, F> Future for Command<T, E, F>
where
    T: Send + Debug,
    E: Send + Debug,
    F: Future<Item = T, Error = E>,
{
    type Item = T;
    type Error = E;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if self.done() {
            return self.fut.poll();
        }

        self.poll_counter += 1;
        let last_polled = Instant::now();
        self.last_polled = Some(last_polled);
        if self.status == CommandStatus::CREATED {
            self.first_polled = Some(last_polled);
            self.status = CommandStatus::RUNNING;
        }

        match self.fut.poll() {
            Ok(result) => match result {
                Async::Ready(_) => {
                    let now = Instant::now();
                    self.completed = Some(now);
                    self.poll_duration += now.duration_since(last_polled);
                    self.status = CommandStatus::SUCCESS;
                    Ok(result)
                }
                Async::NotReady => Ok(result),
            },
            result @ Err(_) => {
                let now = Instant::now();
                self.completed = Some(now);
                self.poll_duration += now.duration_since(last_polled);
                self.status = CommandStatus::FAILURE;
                result
            }
        }
    }
}

impl<T, E, F> Command<T, E, F>
where
    T: Send + Debug,
    E: Send + Debug,
    F: Future<Item = T, Error = E>,
{
    /// Returns whether the underlying future has finished or not.
    ///
    /// If this method returns true, then all future calls to poll are guaranteed to return Ok(Async::NotReady).
    /// If this returns false, then the underlying future has not been driven to completion.
    pub fn done(&self) -> bool {
        self.fut.is_done()
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum CommandStatus {
    CREATED,
    RUNNING,
    SUCCESS,
    FAILURE,
}
