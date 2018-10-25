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

use super::{build_dependency_graph, metadata};
use petgraph::{
    dot::{Config, Dot},
    Graph,
};
use serde_json;
use tests::run_test;

#[test]
fn test_build_dependency_graph() {
    run_test("test_build_dependency_graph", || {
        let dependencies = build_dependency_graph(None);

        let dependencies_json = serde_json::to_string(&dependencies).unwrap();
        info!("dependency graph : {}", dependencies_json);

        let dependencies2: Graph<metadata::PackageId, metadata::dependency::Kind> =
            serde_json::from_str(&dependencies_json).unwrap();
        assert_eq!(
            Dot::with_config(&dependencies, &[Config::EdgeNoLabel]).to_string(),
            Dot::with_config(&dependencies2, &[Config::EdgeNoLabel]).to_string()
        );

        info!(
            "dependency Graphviz Dot diagram: {:?}",
            Dot::with_config(
                &dependencies.map(
                    |_, node| format!("{}-{}", node.name().to_string(), node.version()),
                    |_, edge| *edge
                ),
                &[Config::EdgeNoLabel]
            )
        );
    });
}
