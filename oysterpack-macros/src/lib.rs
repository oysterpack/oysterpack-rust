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

//! Provides a one stop shop for OysterPack macros.
//!
//! ## New Type Pattern
//! - [op_newtype](macro.op_newtype.html)
//!
//! ## Macro Building Blocks
//!
//! ### [AST Coercion](https://danielkeep.github.io/tlborm/book/blk-ast-coercion.html)
//! - [op_tt_as_expr](macro.op_tt_as_expr.html)
//! - [op_tt_as_item](macro.op_tt_as_item.html)
//! - [op_tt_as_pat](macro.op_tt_as_pat.html)
//! - [op_tt_as_stmt](macro.op_tt_as_stmt.html)

#![deny(missing_docs, missing_debug_implementations)]
#![doc(html_root_url = "https://docs.rs/oysterpack_macros/0.1.0")]

#[cfg(test)]
#[macro_use]
extern crate log;
#[cfg(test)]
extern crate fern;
#[macro_use]
#[cfg(test)]
extern crate lazy_static;
#[cfg(test)]
extern crate chrono;
#[cfg(test)]
extern crate serde;
#[cfg(test)]
#[macro_use]
extern crate serde_derive;
#[cfg(test)]
extern crate serde_json;

#[cfg(test)]
mod tests;

#[macro_use]
mod ast_coercsion;
#[macro_use]
mod newtype;