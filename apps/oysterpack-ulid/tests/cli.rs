/*
 * Copyright 2018 OysterPack Inc.
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

use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::{env, process::Command};

#[test]
fn run_cmd_with_no_args() {
    let mut cmd = Command::main_binary().unwrap();
    cmd.assert().failure().stderr(
        predicate::str::contains(format!(
            "oysterpack-ulid {}",
            env::var("CARGO_PKG_VERSION").unwrap()
        ))
        .and(predicate::str::contains("USAGE:"))
        .and(predicate::str::contains("FLAGS:"))
        .and(predicate::str::contains("SUBCOMMANDS:"))
        .and(predicate::str::contains("generate"))
        .and(predicate::str::contains("parse")),
    );
}

#[test]
fn generate_ulid() {
    let mut cmd = Command::main_binary().unwrap();
    cmd.arg("generate")
        .assert()
        .success()
        .stdout(predicate::str::is_match(r#"^\(".{26}", \d+, .+\)"#).unwrap());
}

#[test]
fn parse_ulid_str() {
    let mut cmd = Command::main_binary().unwrap();
    cmd.arg("parse").arg("01CZPB8GYMK8GX1CS7JJQDQ30G")
        .assert()
        .success()
        .stdout(predicate::str::is_match(r#"("01CZPB8GYMK8GX1CS7JJQDQ30G", 1868835502942233280451011019043212304, 2018-12-26T22:48:16.084Z)"#).unwrap());
}

#[test]
fn parse_ulid_u128() {
    let mut cmd = Command::main_binary().unwrap();
    cmd.arg("parse").arg("1868835502942233280451011019043212304")
        .assert()
        .success()
        .stdout(predicate::str::is_match(r#"("01CZPB8GYMK8GX1CS7JJQDQ30G", 1868835502942233280451011019043212304, 2018-12-26T22:48:16.084Z)"#).unwrap());
}

#[test]
fn parse_ulid_str_invalid() {
    let mut cmd = Command::main_binary().unwrap();
    cmd.arg("parse")
        .arg("sdfdsf")
        .assert()
        .failure()
        .stderr(predicate::str::is_match("Error: invalid length").unwrap());
}
