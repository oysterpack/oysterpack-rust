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

/// Refers to a source code location.
/// This can be used to include information regarding where an error or events occur in the code.
/// This will improve traceability.
#[derive(Debug, Clone)]
pub struct SourceCodeLocation {
    module_path: &'static str,
    line: u32,
}

impl SourceCodeLocation {

    /// constructor - use the module_path!() and line!() macros provided by rust.
    pub fn new(module_path: &'static str, line: u32) -> SourceCodeLocation { SourceCodeLocation {module_path,line}}

    /// refers source code line number
    pub fn line(&self) -> u32 {self.line}

    /// refers to the source code module path
    pub fn module_path(&self) -> u32 {self.line}
}