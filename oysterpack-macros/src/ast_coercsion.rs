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

//! Provides macro helpers for [AST coercion](https://danielkeep.github.io/tlborm/book/blk-ast-coercion.html)

/// Used to coerce `tt` tokens into an `expr`
#[macro_export]
macro_rules! op_tt_as_expr {
    ($e:expr) => {
        $e
    };
}

/// Used to coerce a `tt` tokens into an `item`
#[macro_export]
macro_rules! op_tt_as_item {
    ($i:item) => {
        $i
    };
}

/// Used to coerce a `tt` tokens into a `pat`
#[macro_export]
macro_rules! op_tt_as_pat {
    ($p:pat) => {
        $p
    };
}

/// Used to coerce a `tt` tokens into a `stmt`
#[macro_export]
macro_rules! op_tt_as_stmt {
    ($s:stmt) => {
        $s
    };
}
