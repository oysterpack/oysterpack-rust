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
use errors;
use failure::Fail;
use oysterpack_uid::TypedULID;
use std::{
    fmt::{self, Debug},
    time::{Duration, Instant, SystemTime},
};
use tokio::prelude::*;

/// Command is a Future that executes the underlying Future.
///
/// # Features
/// - Command result Item type must implement the Send + Debug traits
///   - Send trait enables the item to be delivered via channels
///   - Debug trait is useful for logging purposes
/// - The Command adds a CommandFailure Error to the Command's underlying future's Error as context.
/// - The underlying future is fused.
///   - Normally futures can behave unpredictable once they're used
///     after a future has been resolved. The fused Future is always defined to return Async::NotReady
///     from poll after it has resolved successfully or returned an error.
/// - Commands are assigned a unique CommandId
///   - the idea is that all commands must be registered and documented, i.e., commands will be
///     registered via their CommandId
/// - Every Command instance is assigned a unique InstanceId
///   - command instance events, e.g., log events, should include the command InstanceId to help
///     with troubleshooting
/// - Future's execution progress is tracked
/// - Progress events can be reported via a channel
///
/// # Logging
///
/// ## WARN Events
/// - Future was polled after it has completed, which will always return Async::NotReady
/// - Unable to send progress on subscriber_chan
///
/// ## DEBUG Events
/// - Progress is logged after the underlying future is polled
/// - Progress is logged when the Command goes out of scope, i.e., when Drop::drop() is invoked
///
#[derive(Debug)]
pub struct Command<F: Future> {
    /// underlying future is fused
    fut: future::Fuse<F>,
    /// tracks command future execution progress
    progress: Progress,
    /// used to report progress
    progress_sender_chan: Option<channel::Sender<Progress>>,
}

impl<T, F> Future for Command<F>
where
    T: Send + Debug,
    F: Future<Item = T, Error = errors::Error>,
{
    type Item = T;
    type Error = errors::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if self.done() {
            warn!("Future was polled after it has completed, which will always return Async::NotReady");
            return self.fut.poll();
        }

        self.progress.poll_counter += 1;
        self.progress.last_polled = Some(SystemTime::now());
        if let Status::CREATED = self.progress.status {
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
            Err(err) => {
                self.progress.status = Status::FAILURE(err.clone());
                Err(self.command_error(err))
            }
        };
        debug!("{:?}", self.progress);
        if let Some(ref subscriber_chan) = self.progress_sender_chan {
            select! {
                send(subscriber_chan,self.progress.clone()) => debug!("sent progress on subscriber_chan"),
                default => warn!("Unable to send progress on subscriber_chan: {:?}", self.progress)
            }
        }
        result
    }
}

impl<T, F> Command<F>
where
    T: Send + Debug,
    F: Future<Item = T, Error = errors::Error>,
{
    /// Constructs a new Command using the specified future as its underlying future.
    /// The underlying future will be fused.
    pub fn new(id: CommandId, fut: F) -> Command<F> {
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
    pub fn progress(&self) -> &Progress {
        &self.progress
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

    fn command_error(&self, error: errors::Error) -> errors::Error {
        let command_failure = CommandFailure::new(self.id(), self.instance_id(), error.clone());
        op_error!(COMMAND_FAILURE_ERROR_ID, error.context(command_failure))
    }
}

impl<F: Future> Drop for Command<F> {
    fn drop(&mut self) {
        debug!("Command Dropped: {:?}", self.progress);
    }
}

/// Command builder
pub struct Builder<F: Future> {
    cmd: Command<F>,
}

impl<T, F> Builder<F>
where
    T: Send + Debug,
    F: Future<Item = T, Error = errors::Error>,
{
    /// Constructs a new Builder seeding it with the Command's underlying future.
    pub fn new(id: CommandId, fut: F) -> Builder<F> {
        Builder {
            cmd: Command::new(id, fut),
        }
    }

    /// Attaches a progress subscriber sender channel to the command
    pub fn progress_subscriber_chan(self, subscriber: channel::Sender<Progress>) -> Builder<F> {
        let mut builder = self;
        builder.cmd.progress_sender_chan = Some(subscriber);
        builder
    }

    /// Builds and returns the Command
    pub fn build(self) -> Command<F> {
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
#[derive(Debug, Clone)]
pub enum Status {
    /// Command future has been created, but has not started running
    CREATED,
    /// Command future has started running, i.e., it has been polled at least once
    RUNNING,
    /// Command has completed successfully
    SUCCESS,
    /// Command has completed with an error
    FAILURE(errors::Error),
    /// Command was cancelled
    CANCELLED,
}

impl Status {
    /// Returns true if status == CREATED
    pub fn created(&self) -> bool {
        if let Status::CREATED = *self {
            true
        } else {
            false
        }
    }

    /// Returns true if status == RUNNING
    pub fn running(&self) -> bool {
        if let Status::RUNNING = *self {
            true
        } else {
            false
        }
    }

    /// Returns true if status == SUCCESS
    pub fn success(&self) -> bool {
        if let Status::SUCCESS = *self {
            true
        } else {
            false
        }
    }

    /// Returns true if status == FAILURE
    pub fn failure(&self) -> bool {
        if let Status::FAILURE(_) = *self {
            true
        } else {
            false
        }
    }

    /// Returns true if status == CANCELLED
    pub fn cancelled(&self) -> bool {
        if let Status::CANCELLED = *self {
            true
        } else {
            false
        }
    }
}

/// Used to track the Command future execution progress
#[derive(Debug, Clone)]
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
            instance_id: InstanceId::generate(),
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
    pub fn status(&self) -> &Status {
        &self.status
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

op_newtype! {
    /// Unique Command ID
    #[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize)]
    pub CommandId(pub u128)
}

/// Represents a Command instance
#[derive(Debug)]
pub struct Instance;

/// Command instance ULID
pub type InstanceId = TypedULID<Instance>;

/// CommandFailure provides the context for command failures
#[derive(Debug, Clone)]
pub struct CommandFailure {
    command_id: CommandId,
    instance_id: InstanceId,
    error: errors::Error,
}

impl CommandFailure {
    /// CommandFailure constructor
    pub fn new(
        command_id: CommandId,
        instance_id: InstanceId,
        error: errors::Error,
    ) -> CommandFailure {
        CommandFailure {
            command_id,
            instance_id,
            error,
        }
    }

    /// CommandId for Command that failed.
    pub fn command_id(&self) -> CommandId {
        self.command_id
    }

    /// InstanceId for Command that failed.
    pub fn instance_id(&self) -> InstanceId {
        self.instance_id
    }

    /// Error that caused the Command failure. This is the Error that is returned by the Command's
    /// underlying future.
    pub fn error(&self) -> &errors::Error {
        &self.error
    }
}

impl fmt::Display for CommandFailure {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "CommandFailure({:?})/InstanceId({})/{})",
            self.command_id, self.instance_id, self.error
        )
    }
}

/// Indicates a failure occurred while executing a Command.
pub const COMMAND_FAILURE_ERROR_ID: errors::ErrorId = errors::ErrorId(1);
