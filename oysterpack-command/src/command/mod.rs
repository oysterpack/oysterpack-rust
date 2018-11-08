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

//! Command package

use chrono::{DateTime, Duration, Utc};
use crossbeam_channel as channel;
use failure::Fail;
use oysterpack_errors::Error;
use oysterpack_events::Eventful;
use oysterpack_uid::{Domain, HasDomain, TypedULID};
use std::fmt::Debug;
use tokio::prelude::*;

/// Represents a command future.
/// - commands are identified by CommandId
#[derive(Debug)]
pub struct Command<F: Future> {
    /// underlying future is fused
    fut: future::Fuse<F>,
    /// tracks command future execution progress
    progress: Progress,
    /// used to report progress
    progress_sender_chan: Option<channel::Sender<Progress>>,
}

//TODO: Cancellation results in a command cancelled event
impl<F, T> Future for Command<F>
where
    F: Future<Item = T, Error = Error>,
    T: Send + Debug,
{
    type Item = F::Item;
    type Error = F::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if self.done() {
            warn!("Future was polled after it has completed, which will always return Async::NotReady");
            return self.fut.poll();
        }

        self.progress.poll_counter += 1;
        self.progress.last_polled = Some(Utc::now());
        if let Status::CREATED = self.progress.status {
            self.progress.first_polled = self.progress.last_polled;
            self.progress.status = Status::RUNNING;
        }

        let last_polled = Utc::now();
        let poll_result = self.fut.poll();
        let poll_duration = Utc::now().signed_duration_since(last_polled);
        if let Some(poll_duration) = self.progress.poll_duration.checked_add(&poll_duration) {
            self.progress.poll_duration = poll_duration;
        }

        let result = match poll_result {
            Ok(result @ Async::Ready(_)) => {
                self.progress.completed = Some(Utc::now());
                self.progress.status = Status::SUCCESS;
                Ok(result)
            }
            result @ Ok(Async::NotReady) => result,
            Err(err) => {
                self.progress.completed = Some(Utc::now());
                self.progress.status = Status::FAILURE(err.clone());
                Err(err)
            }
        };
        debug!("{:?}", self.progress);
        if let Some(ref subscriber_chan) = self.progress_sender_chan {
            select! {
                send(subscriber_chan,self.progress.clone()) -> result => debug!("sent progress on subscriber_chan: {:?}", result),
                default => warn!("Unable to send progress on subscriber_chan: {:?}", self.progress)
            }
        }
        result
    }
}

impl<F, T> Command<F>
where
    F: Future<Item = T, Error = Error>,
    T: Send + Debug,
{
    /// Constructs a new Command using the specified future as its underlying future.
    /// The underlying future will be fused.
    pub fn new(id: Id, fut: F) -> Command<F> {
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
    pub fn id(&self) -> Id {
        self.progress.id
    }

    /// InstanceID is the unique identifier for this instance of the command.
    /// Its main use case is for tracking purposes.
    pub fn instance_id(&self) -> InstanceId {
        self.progress.instance_id
    }
}

op_newtype! {
    /// Command unique identifier
    #[derive(Serialize, Deserialize, Clone, Copy, Eq, PartialEq, Hash, Ord, PartialOrd)]
    pub Id(pub u128)
}

impl HasDomain for Id {
    const DOMAIN: Domain = Domain("Command");
}

/// Marker type for an Event instance, which is used to define [InstanceId](type.InstanceId.html)
#[allow(missing_debug_implementations)]
pub struct Instance;

/// Event instance IDs are generated for each new Event instance that is created.
pub type InstanceId = TypedULID<Instance>;

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
    FAILURE(Error),
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
    id: Id,
    instance_id: InstanceId,
    status: Status,
    // used to track the number of times the future has been polled
    poll_counter: usize,
    // when the future instance was created
    created: DateTime<Utc>,
    // when the future instance was first polled
    first_polled: Option<DateTime<Utc>>,
    // when the future instance was last polled
    last_polled: Option<DateTime<Utc>>,
    // when the future completed, whether it succeeded ot failed
    completed: Option<DateTime<Utc>>,
    // the cumulative amount of time spent polling
    poll_duration: Duration,
}

impl Progress {
    /// constructs a new Progress with status = CREATED, and the created timestamp to now
    fn new(id: Id) -> Progress {
        Progress {
            id,
            instance_id: InstanceId::generate(),
            status: Status::CREATED,
            poll_counter: 0,
            created: Utc::now(),
            first_polled: None,
            last_polled: None,
            completed: None,
            poll_duration: Duration::zero(),
        }
    }

    /// CommandId is the unique identifier for the command - across all instances.
    pub fn id(&self) -> Id {
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
    pub fn created(&self) -> DateTime<Utc> {
        self.created
    }

    /// when the future instance was first polled
    pub fn first_polled(&self) -> Option<DateTime<Utc>> {
        self.first_polled
    }

    /// when the future instance was last polled
    pub fn last_polled(&self) -> Option<DateTime<Utc>> {
        self.last_polled
    }

    /// when the future completed
    pub fn completed(&self) -> Option<DateTime<Utc>> {
        self.completed
    }

    /// the cumulative amount of time spent polling
    pub fn poll_duration(&self) -> Duration {
        self.poll_duration
    }
}

#[allow(warnings)]
#[cfg(test)]
mod tests;
