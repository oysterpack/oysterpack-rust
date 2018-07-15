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

use crossbeam_channel as channel;
use rusty_ulid::Ulid;
use std::{
    fmt::Debug, time::{Duration, Instant, SystemTime},
};
use tokio::prelude::*;

/// Command is a Future that executes the underlying Future.
///
/// ### Features
/// - The Command result Item type must implement the Send + Debug traits
///   - the Send trait will enable the item to be delivered via channels
///   - Debug is useful for logging purposes
/// - The Command result Error type must implement the Send + Debug + Clone
///   - the Error type must be cloneable in order to enable Errors to be delivered as part of Progress
/// - The underlying future is fused.
///   - Normally futures can behave unpredictable once they're used
///     after a future has been resolved. The fused Future is always defined to return Async::NotReady
///     from poll after it has resolved successfully or returned an error.
/// - Commands are assigned a unique CommandId
///   - the idea is that all commands must be registered and documented.
/// - Every Command instance is assigned a unique InstanceId
/// - The future's execution progress is tracked
/// - Progress events can be reported via a channel
///
#[derive(Debug)]
pub struct Command<T, E, F>
where
    T: Send + Debug,
    E: Send + Debug + Clone,
    F: Future<Item = T, Error = E>,
{
    // the underlying future is fused
    fut: future::Fuse<F>,
    // tracks command future execution progress
    progress: Progress,
    // used to report progress
    progress_sender_chan: Option<channel::Sender<Progress>>,
}

impl<T, E, F> Future for Command<T, E, F>
where
    T: Send + Debug,
    E: Send + Debug + Clone,
    F: Future<Item = T, Error = E>,
{
    type Item = T;
    type Error = E;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if self.done() {
            warn!("Future was polled after it has completed, which will always return Async::NotReady");
            return self.fut.poll();
        }

        self.progress.poll_counter += 1;
        self.progress.last_polled = Some(SystemTime::now());
        if self.progress.status == Status::CREATED {
            self.progress.first_polled = self.progress.last_polled.clone();
            self.progress.status = Status::RUNNING;
        }

        let last_polled = Instant::now();
        let poll_result = self.fut.poll();
        self.progress.poll_duration += Instant::now().duration_since(last_polled);
        self.progress.completed = Some(SystemTime::now());
        let result = match poll_result {
            Ok(result @ Async::Ready(_)) => {
                self.progress.status = Status::SUCCESS;
                Ok(result)
            }
            result @ Ok(Async::NotReady) => result,
            result @ Err(_) => {
                self.progress.status = Status::FAILURE;
                result
            }
        };
        debug!("{:?}", self.progress);
        if let Some(ref subscriber_chan) = self.progress_sender_chan {
            select! {
                send(subscriber_chan,self.progress) => debug!("sent progress on subscriber_chan"),
                default => warn!("Unable to send progress on subscriber_chan: {:?}", self.progress)
            }
        }
        result
    }
}

impl<T, E, F> Command<T, E, F>
where
    T: Send + Debug,
    E: Send + Debug + Clone,
    F: Future<Item = T, Error = E>,
{
    /// Constructs a new Command using the specified future as its underlying future.
    /// The underlying future will be fused.
    pub fn new(id: CommandId, fut: F) -> Command<T, E, F> {
        Command {
            fut: Future::fuse(fut),
            progress: Progress::new(id),
            progress_sender_chan: None,
        }
    }

    /// Returns whether the underlying future has finished or not.
    ///
    /// If this method returns true, then all future calls to poll are guaranteed to return Ok(Async::NotReady).
    /// If this returns false, then the underlying future has not been driven to completion.
    pub fn done(&self) -> bool {
        self.fut.is_done()
    }

    /// Returns a snapshot of the command's progress
    pub fn progress(&self) -> Progress {
        self.progress
    }

    /// CommandId is the unique identifier for the command - across all instances.
    pub fn id(&self) -> CommandId {
        self.progress.id
    }

    /// InstanceID is the unique identifier for this instance of the command.
    /// Its main use case is for tracking purposes.
    pub fn instance_id(&self) -> InstanceId {
        self.progress.instance_id
    }
}

/// Command builder
pub struct Builder<T, E, F>
where
    T: Send + Debug,
    E: Send + Debug + Clone,
    F: Future<Item = T, Error = E>,
{
    cmd: Command<T, E, F>,
}

impl<T, E, F> Builder<T, E, F>
where
    T: Send + Debug,
    E: Send + Debug + Clone,
    F: Future<Item = T, Error = E>,
{
    /// Constructs a new Builder seeding it with the Command's underlying future.
    pub fn new(id: CommandId, fut: F) -> Builder<T, E, F> {
        Builder {
            cmd: Command::new(id, fut),
        }
    }

    /// Attaches a progress subscriber sender channel to the command
    pub fn progress_subscriber_chan(
        self,
        subscriber: channel::Sender<Progress>,
    ) -> Builder<T, E, F> {
        let mut builder = self;
        builder.cmd.progress_sender_chan = Some(subscriber);
        builder
    }

    /// Builds and returns the Command
    pub fn build(self) -> Command<T, E, F> {
        self.cmd
    }
}

/// Command transitions:
///
/// ```
/// //          |----------->|-> CANCELLED
/// // CREATED -|-> RUNNING -|-> SUCCESS
/// //                       |-> FAILURE
/// ```
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Status {
    /// Command future has been created, but has not started running
    CREATED,
    /// Command future has started running, i.e., it has been polled at least once
    RUNNING,
    /// Command has completed successfully
    SUCCESS,
    /// Command has completed with an error
    FAILURE,
    /// Command was cancelled
    CANCELLED,
}

/// Used to track the Command future execution progress
#[derive(Debug, Copy, Clone)]
pub struct Progress {
    id: CommandId,
    instance_id: InstanceId,
    status: Status,
    // used to track the number of times the future has been polled
    poll_counter: usize,
    // when the future instance was created
    created: SystemTime,
    // when the future instance was first polled
    first_polled: Option<SystemTime>,
    // when the future instance was last polled
    last_polled: Option<SystemTime>,
    // when the future completed
    completed: Option<SystemTime>,
    // the cumulative amount of time spent polling
    poll_duration: Duration,
}

impl Progress {
    /// constructs a new Progress with status = CREATED, and the created timestamp to now
    fn new(id: CommandId) -> Progress {
        Progress {
            id,
            instance_id: InstanceId::new(),
            status: Status::CREATED,
            poll_counter: 0,
            created: SystemTime::now(),
            first_polled: None,
            last_polled: None,
            completed: None,
            poll_duration: Duration::new(0, 0),
        }
    }

    /// CommandId is the unique identifier for the command - across all instances.
    pub fn id(&self) -> CommandId {
        self.id
    }

    /// InstanceID is the unique identifier for this instance of the command.
    /// Its main use case is for tracking purposes.
    pub fn instance_id(&self) -> InstanceId {
        self.instance_id
    }

    /// Command status
    pub fn status(&self) -> Status {
        self.status
    }

    /// the number of times the future has been polled
    pub fn poll_counter(&self) -> usize {
        self.poll_counter
    }

    /// when the future instance was created
    pub fn created(&self) -> SystemTime {
        self.created
    }

    /// when the future instance was first polled
    pub fn first_polled(&self) -> Option<SystemTime> {
        self.first_polled
    }

    /// when the future instance was last polled
    pub fn last_polled(&self) -> Option<SystemTime> {
        self.last_polled
    }

    /// when the future completed
    pub fn completed(&self) -> Option<SystemTime> {
        self.completed
    }

    /// the cumulative amount of time spent polling
    pub fn poll_duration(&self) -> Duration {
        self.poll_duration
    }
}

/// Command ID - unique identifier
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct CommandId(u128);

impl CommandId {
    pub fn new(id: u128) -> CommandId {
        CommandId(id)
    }

    pub fn value(&self) -> u128 {
        self.0
    }
}

impl From<u128> for CommandId {
    fn from(id: u128) -> Self {
        CommandId(id)
    }
}

impl From<Ulid> for CommandId {
    fn from(id: Ulid) -> Self {
        CommandId(id.into())
    }
}

/// Command Instance ID - unique identifier
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct InstanceId(u128);

impl InstanceId {
    pub fn new() -> InstanceId {
        <InstanceId as From<Ulid>>::from(Ulid::new())
    }
}

impl InstanceId {
    pub fn value(&self) -> u128 {
        self.0
    }
}

impl From<u128> for InstanceId {
    fn from(id: u128) -> Self {
        InstanceId(id)
    }
}

impl From<Ulid> for InstanceId {
    fn from(id: Ulid) -> Self {
        InstanceId(id.into())
    }
}
