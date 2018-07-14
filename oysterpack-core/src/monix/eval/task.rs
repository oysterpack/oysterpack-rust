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

//! Task represents a specification for a possibly lazy or asynchronous computation, which when
//! executed will produce Result[T,E], along with possible side-effects.
//!
//! Task does not execute anything when working with its builders or operators and it does not
//! submit any work into any thread-pool, the execution eventually taking place only after
//! Task::run_async is called and not before that.
//!
//! Note that Task is conservative in how it spawns logical threads. Transformations like map and
//! flat_map for example will default to being executed on the logical thread on which the
//! asynchronous computation was started. But one shouldn't make assumptions about how things will
//! end up executed, as ultimately it is the implementation's job to decide on the best execution
//! model. All you are guaranteed is asynchronous execution after executing runAsync.

use std::error::Error;

/// Task represents a specification for a possibly lazy or asynchronous computation
trait Task<T, E>
where
    T: Send,
    E: Error + Send,
{
    fn execute(&mut self) -> Result<T, E>;
}
