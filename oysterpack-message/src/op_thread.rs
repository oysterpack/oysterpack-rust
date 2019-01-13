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

//! Provides thread support

use oysterpack_uid::ULID;
use std::{num::NonZeroUsize, thread};

/// Thread config
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ThreadConfig {
    name: String,
    stack_size: Option<usize>,
}

impl ThreadConfig {
    /// constructor
    /// - name is trimmed
    /// - if the name is blank then it will be replaced with a ULID
    ///   - creating a thread with blank name would trigger a panic. In order to avoid to always
    ///     having to handle the error case for this edge case, a random thread name will be used
    pub fn new(name: &str) -> ThreadConfig {
        let name = {
            if name.trim().is_empty() {
                ULID::generate().to_string()
            } else {
                name.to_string()
            }
        };

        ThreadConfig {
            name,
            stack_size: None,
        }
    }

    /// Sets the size of the stack (in bytes) for the new thread.
    /// The actual stack size may be greater than this value if the platform specifies minimal stack size.
    pub fn set_stack_size(self, stack_size: NonZeroUsize) -> ThreadConfig {
        let mut config = self;
        config.stack_size = Some(stack_size.get());
        config
    }

    /// thread builder constructor
    pub fn builder(&self) -> thread::Builder {
        match self.stack_size {
            None => thread::Builder::new().name(self.name.clone()),
            Some(stack_size) => thread::Builder::new()
                .name(self.name.clone())
                .stack_size(stack_size),
        }
    }
}

#[allow(warnings)]
#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn named_thread() {
        let name = ULID::generate().to_string();
        let handle = ThreadConfig::new(name.as_str())
            .builder()
            .spawn(|| thread::current().name().unwrap().to_string())
            .unwrap();

        assert_eq!(handle.join().unwrap().to_string(), name);
    }

    #[test]
    fn spawn_thread_with_custom_stack_size() {
        let name = ULID::generate().to_string();
        let handle = ThreadConfig::new(name.as_str())
            .set_stack_size(NonZeroUsize::new(1024).unwrap())
            .builder()
            .spawn(|| thread::current().name().unwrap().to_string())
            .unwrap();

        assert_eq!(handle.join().unwrap().to_string(), name);
    }

}
