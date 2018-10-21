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

//! Provides support for DevOps tooling.

#[macro_use]
mod macros;

#[cfg(test)]
mod tests;

use std::fmt;

/// Refers to a source code location.
/// This can be used to include information regarding where an error or event occur in the code to
/// provide traceability.
#[derive(Debug, Clone)]
pub struct SourceCodeLocation {
    module_path: &'static str,
    line: u32,
}

impl SourceCodeLocation {
    /// constructor - use the module_path!() and line!() macros provided by rust.
    pub fn new(module_path: &'static str, line: u32) -> SourceCodeLocation {
        SourceCodeLocation { module_path, line }
    }

    /// refers source code line number
    pub fn line(&self) -> u32 {
        self.line
    }

    /// refers to the source code module path
    pub fn module_path(&self) -> &'static str {
        self.module_path
    }

    /// returns the crate name, which is extracted from the module path
    pub fn crate_name(&self) -> &'static str {
        self.module_path.split("::").next().unwrap()
    }
}

impl fmt::Display for SourceCodeLocation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.module_path, self.line)
    }
}
