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

use super::*;
use petgraph::dot::{Config, Dot};
use serde_json;
use tests::run_test;
use metadata;

op_build_mod!();

#[test]
fn build_dependency_graph_all_features() {
    run_test(|| {
        let dependencies = build_dependency_graph(None);
        info!(
            "dependencies Dot diagram: {:?}",
            Dot::with_config(&dependencies, &[Config::EdgeNoLabel])
        );
        info!("all dependencies: {:?}",super::dependencies::all(&dependencies));

        let dependencies_json = serde_json::to_string_pretty(&dependencies).unwrap();
        info!("dependencies : {}", dependencies_json);

        let dependencies2: Graph<metadata::PackageId,metadata::dependency::Kind> = serde_json::from_str(&dependencies_json).unwrap();
        assert_eq!(
            Dot::with_config(&dependencies, &[Config::EdgeNoLabel]).to_string(),
            Dot::with_config(&dependencies2, &[Config::EdgeNoLabel]).to_string()
        );
    });
}

#[test]
fn build_dependency_graph_default_features() {
    run_test(|| {
        let features = vec!["default".to_string()];
        let dependencies = build_dependency_graph(Some(features));
        info!(
            "dependencies Dot diagram: {:?}",
            Dot::with_config(&dependencies, &[Config::EdgeNoLabel])
        );

        let dependencies_json = serde_json::to_string_pretty(&dependencies).unwrap();
        info!("dependencies : {}", dependencies_json);

        let dependencies2: Graph<metadata::PackageId,metadata::dependency::Kind> = serde_json::from_str(&dependencies_json).unwrap();
        assert_eq!(
            Dot::with_config(&dependencies, &[Config::EdgeNoLabel]).to_string(),
            Dot::with_config(&dependencies2, &[Config::EdgeNoLabel]).to_string()
        );
    });
}
