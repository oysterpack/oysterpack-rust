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

/// Constructs a new errors::Error from the specified errors::ErrorId and failure::Fail.
/// This provides a shorthand notation for creating an errors::Error.
/// The error is logged using the [log](https://crates.io/crates/log) crate.
///
/// This means that the [log](https://crates.io/crates/log) crate need to be imported by external crates using this macro:
/// - `#[macro_use] extern crate log`
///
/// # Example
/// `op_failure!(ERR_AUTHZ_FAILED, failure);`
///
/// where ERR_AUTHZ_FAILED is an `errors::ErrorId` and failure's type is `failure::Fail`
///
#[macro_export]
macro_rules! op_failure {
    ($err_id:expr, $fail:expr) => {{
        use $crate::errors::Error;
        let err = Error::new($err_id, $fail, op_src_loc!());
        error!("{}", err);
        err
    }};
}

/// This is a higher-order macro that generates a macro for creating an errors::Error.
///
/// # Example
/// `error_macro(ErrAuthz,ERR_AUTHZ_FAILED);`
///
///  This generates a macro named `ErrAuthz`. It gets used like: `ErrAuthz!(failure);`
///
///  The pattern is to define all ErrorIds and Error macros in a single module per crate.
///
#[macro_export]
macro_rules! error_macro {
    ($name:ident, $err_id:expr) => {
        macro_rules! $name {
            ($failure: expr) => {
                op_failure!($err_id, $failure)
            };
        }
    };
}
