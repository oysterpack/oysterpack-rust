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

//! The execution module provides support to define execution contexts in which functions can be executed.

use std::time::Duration;

use tokio::prelude::*;

#[cfg(test)]
mod tests;

pub trait Scheduler {
    fn execute<F>(&mut self, f: F)
    where
        F: Future<Item = (), Error = ()> + Send;
}

/// Represents a one-time idempotent action that can be used to cancel async computations, or to
/// release resources that active data sources are holding.
pub trait Cancelable {
    /// Cancels the unit of work represented by this reference.
    //
    // Guaranteed idempotency - calling it multiple times should have the same side-effect as
    // calling it only once. Implementations of this method should also be thread-safe.
    fn cancel();
}

pub enum FutureError {
    /// The future cancelled
    Cancelled,
    /// The future did not complete in time, i.e., has timed out.
    Timeout(Duration),
    Panic,
}
