use std::{collections::HashSet, error::Error, sync::Arc, time::Duration};

use chrono::Utc;
use rusty_ulid::Ulid;

/// Represents a lazy or (possibly) async function.
/// The Task can only be executed once.
pub trait Task<Ref, Callback, Exec, T, E>
where
    Callback: ResultListener<T, E>,
    Exec: Execution,
    T: Send,
    E: Error + Send,
{
    /// After the Task is executed, it can no longer be referenced directly.
    /// It can only be referenced via its TaskRef, which can be used to track the task.
    fn execute(&mut self, callback: Callback) -> Exec;

    fn desc(&self) -> Descriptor;
}

/// Task ID - unique identifier
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct TaskId(u128);

impl TaskId {
    pub fn value(&self) -> u128 {
        self.0
    }
}

impl From<u128> for TaskId {
    fn from(id: u128) -> Self {
        TaskId(id)
    }
}

impl From<Ulid> for TaskId {
    fn from(id: Ulid) -> Self {
        TaskId(id.into())
    }
}

/// Task Instance ID - unique identifier
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct InstanceId(u128);

impl InstanceId {
    pub fn new() -> InstanceId {
        InstanceId::from(Ulid::new())
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

pub struct TaskResult<T, E>
where
    T: Send,
    E: Error + Send,
{
    result: Result<T, E>,
    task_id: TaskId,
    instance_id: InstanceId,
}

pub trait ResultListener<T, E>
where
    T: Send,
    E: Error + Send,
{
    fn on_result(result: TaskResult<T, E>);
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Descriptor {
    task_id: TaskId,
    config: Option<Config>,
}

impl Descriptor {
    /// Returns the unique Task identifier.
    pub fn id(&self) -> TaskId {
        self.task_id
    }

    /// Returns the Task config.
    fn config(&self) -> Option<&Config> {
        match self.config {
            Some(ref c) => Some(c),
            None => None,
        }
    }
}

/// Represents a one-time idempotent action that can be used to cancel async computations, or to release resources that active data sources are holding.
pub trait Cancelable {
    /// Cancels the unit of work represented by this reference.
    ///
    /// Guaranteed idempotency - calling it multiple times should have the same side-effect as calling it only once.
    /// Implementations of this method should also be thread-safe.
    fn cancel(&mut self);

    /// Returns true if cancellation was triggered.
    fn cancelled(&self) -> bool;
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ExecuteOptions {
    priority: Option<Priority>,
    timeout: Option<TimeoutConfig>,
    delay: Option<Duration>,
}

impl ExecuteOptions {
    /// returns the Task priority
    pub fn priority(&self) -> Option<Priority> {
        self.priority
    }

    /// Returns the Task timeout.
    /// None indicates, there is no timeout.
    fn timeout(&self) -> Option<TimeoutConfig> {
        self.timeout
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Config {
    execute_options: Option<ExecuteOptions>,
    tags: Option<HashSet<Tag>>,
    task_dependencies: Option<HashSet<TaskId>>,
}

/// Task config
impl Config {
    /// returns the default execution settings
    fn execute_options(&self) -> Option<&ExecuteOptions> {
        match self.execute_options {
            Some(ref opts) => Some(opts),
            None => None,
        }
    }

    /// Tasks can have optional tags.
    ///
    /// Example use cases:
    /// - ES
    /// - COUCH_DB
    /// - DB
    ///
    /// Tags may be used by implementations for logging and analytic purposes.
    fn tags(&self) -> Option<&HashSet<Tag>> {
        match self.tags {
            Some(ref tags) => Some(tags),
            None => None,
        }
    }

    /// Returns optional set of TaskId(s) for Tasks that this Task directly depends on.
    /// This indicates the potential types of sub-tasks that may be executed by this Task.
    fn task_dependencies(&self) -> Option<&HashSet<TaskId>> {
        match self.task_dependencies {
            Some(ref deps) => Some(deps),
            None => None,
        }
    }
}

/// Tag
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Tag(String);

impl Tag {
    /// Tag value
    pub fn value(&self) -> &str {
        &self.0
    }
}

/// Task timeout config.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct TimeoutConfig(Duration, State);

impl TimeoutConfig {
    /// relative_to_state defaults to Running
    pub fn new(
        duration: Duration,
        relative_to_state: Option<TimeoutRelativeState>,
    ) -> TimeoutConfig {
        match relative_to_state {
            Some(s) => TimeoutConfig(duration, s.into()),
            None => TimeoutConfig(duration, State::Running),
        }
    }

    /// timeout
    pub fn timeout(&self) -> Duration {
        self.0
    }

    /// relative to state
    pub fn relative_to_state(&self) -> State {
        self.1
    }
}

#[derive(Debug, Copy, Clone)]
/// The states that timeouts are relative to
pub enum TimeoutRelativeState {
    New,
    Scheduled,
    Running,
}

impl Into<State> for TimeoutRelativeState {
    fn into(self) -> State {
        match self {
            TimeoutRelativeState::New => State::New,
            TimeoutRelativeState::Scheduled => State::Scheduled,
            TimeoutRelativeState::Running => State::Running,
        }
    }
}

/// Used to define a task's priority. Higher priority tasks take precedence over lower priority tasks.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Priority(u8);

impl Priority {
    /// returns the priority value
    pub fn value(&self) -> u8 {
        self.0
    }
}

/// Tracks Task execution.
pub trait Execution: Send {
    /// An InstanceId is assigned once the task is scheduled for execution, i.e., when Task.execute() is invoked.
    fn instance_id(&self) -> Option<InstanceId>;

    fn task_desc(&self) -> Descriptor;

    /// Returns the Task's execution options.
    /// A Task can be configured with default options, which can be overridden.
    fn execute_options(&self) -> Option<&ExecuteOptions>;

    /// Returns ExecutionStatus
    fn status(&self) -> Box<ExecutionStatus>;

    /// None is returned if the task does not support cancellation
    fn cancelable(&self) -> Option<Box<Cancelable>>;

    /// None is returned if this task spawned no child tasks.
    fn child_tasks(&self) -> Option<&ChildTasks>;
}

pub trait ChildTasks {
    /// Returns total number of sub-tasks that have been spawned.
    fn count(&self) -> usize;

    /// Returns the number of sub-tasks that this task is waiting on
    fn blocked_on_count(&self) -> usize;

    /// Returns th number of sub-tasks that have been submitted async, with possible side effects.
    fn async_count(&self) -> usize;

    /// Returns child executions.
    fn executions(&self) -> Vec<Arc<Execution>>;
}

/// Used to track task execution.
pub trait ExecutionStatus: Send {
    /// Returns the current Task status
    fn state(&self) -> State;

    /// Returns the StateTransition for the task.
    fn state_transtions(&self) -> Vec<StateTransition>;

    /// Task progress. Not all tasks may support reporting progress. In that case, None is returned.
    /// Progress starts when the Task starts running.
    fn progress(&self) -> Option<Progress>;
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum State {
    New,
    Scheduled,
    Running,
    Success,
    Failure,
    Cancelled,
    TimedOut,
}

impl State {
    pub fn terminal_state(&self) -> bool {
        match *self {
            State::Success | State::Failure | State::Cancelled | State::TimedOut => true,
            _ => false,
        }
    }
}

/// Represents when state transition event.
pub struct StateTransition(State, Utc);

impl StateTransition {
    /// The state that was transitioned to
    pub fn state(&self) -> State {
        self.0
    }

    /// When the state transition occurred.
    pub fn timestamp(&self) -> Utc {
        self.1
    }
}

/// Represents a task's progress as a percentage between 0 - 100.
///
pub struct Progress(u8);

impl Progress {
    /// returns the priority value
    pub fn value(&self) -> u8 {
        self.0
    }

    /// increments progress
    ///
    /// If progress exceeds 100%, then progress is reset to 100%, which indicates it is done.
    pub fn inc(&mut self, progress: u8) -> u8 {
        if progress >= 100 {
            self.0 = 100;
            return self.0;
        }

        self.0 += progress;
        if self.0 > 100 {
            self.0 = 100;
        }
        self.0
    }

    pub fn done(&self) -> bool {
        self.0 == 100
    }
}
