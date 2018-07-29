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

//! error related macros

/// Invokes Error::new($err_id, $fail, src_loc!())
///
/// # Example
///
#[macro_export]
macro_rules! op_failure {
    ($err_id:expr, $fail:expr) => {{
        use $crate::errors::Error;
        Error::new($err_id, $fail, op_src_loc!())
    }};
}
