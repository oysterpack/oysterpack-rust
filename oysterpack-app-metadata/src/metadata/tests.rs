// Copyright 2018 OysterPack Inc.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! unit tests

use semver;

use super::PackageId;
use tests::*;

#[test]
fn parsing_dependencies_graphviz_dot_into_package_ids() {
    let dot = r#"
    digraph {
    0 [label="oysterpack_app_template=0.1.0"]
    1 [label="log=0.4.5"]
    2 [label="serde=1.0.79"]
    3 [label="oysterpack_app_metadata=0.1.0"]
    4 [label="serde_derive=1.0.79"]
    5 [label="fern=0.5.6"]
    6 [label="semver=0.9.0"]
    7 [label="chrono=0.4.6"]
    8 [label="serde_json=1.0.31"]
    9 [label="ryu=0.2.6"]
    10 [label="itoa=0.4.3"]
    11 [label="num-integer=0.1.39"]
    12 [label="time=0.1.40"]
    13 [label="num-traits=0.2.6"]
    14 [label="libc=0.2.43"]
    15 [label="semver-parser=0.7.0"]
    16 [label="proc-macro2=0.4.19"]
    17 [label="syn=0.15.6"]
    18 [label="quote=0.6.8"]
    19 [label="unicode-xid=0.1.0"]
    20 [label="cfg-if=0.1.5"]
    0 -> 1
    0 -> 2
    0 -> 3
    0 -> 4
    0 -> 5
    0 -> 6
    0 -> 7
    0 -> 8
    8 -> 2
    8 -> 9
    8 -> 10
    7 -> 11
    7 -> 2
    7 -> 12
    7 -> 13
    12 -> 14
    11 -> 13
    6 -> 15
    6 -> 2
    5 -> 1
    4 -> 16
    4 -> 17
    4 -> 18
    18 -> 16
    17 -> 19
    17 -> 18
    17 -> 16
    16 -> 19
    3 -> 2
    3 -> 7
    3 -> 6
    3 -> 4
    1 -> 20
}"#;

    run_test(|| {
        let mut package_ids: Vec<PackageId> = dot
            .lines()
            .filter(|line| !line.contains("->") && line.contains("["))
            .skip(1)
            .map(|line| {
                let line = &line[line.find('"').unwrap() + 1..];
                let line = &line[..line.find('"').unwrap()];
                let tokens: Vec<&str> = line.split("=").collect();
                PackageId::new(
                    tokens.get(0).unwrap().to_string(),
                    semver::Version::parse(tokens.get(1).unwrap()).unwrap(),
                )
            }).collect();
        package_ids.sort();
        let package_ids: Vec<String> = package_ids.iter().map(|id| id.to_string()).collect();
        info!("package_ids : {}", package_ids.join("\n"));
    });
}