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

//! collects application metadata at build time

use built;
use cargo::{
    self,
    core::{
        package::PackageSet, registry::PackageRegistry, resolver::Method, Package, Resolve,
        Workspace,
    },
    ops,
    util::{self, important_paths, CargoResult, Cfg, Rustc},
    Config,
};

use petgraph::{
    self, dot,
    graph::{Graph, NodeIndex},
    Direction,
};
use std::{
    collections::{hash_map::Entry, HashMap},
    env,
    fs::{self, OpenOptions},
    io::prelude::*,
    path,
    str::{self, FromStr},
};

/// build metadata
pub(crate) mod metadata {
    use semver;
    use serde_derive::{Deserialize, Serialize};
    use std::fmt;

    /// Identifier for a specific version of a package.
    #[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, Ord, PartialOrd)]
    pub struct PackageId {
        name: String,
        version: semver::Version,
    }

    impl PackageId {
        /// PackageId constructor
        pub fn new(name: String, version: semver::Version) -> PackageId {
            PackageId { name, version }
        }

        /// Package name
        pub fn name(&self) -> &str {
            &self.name
        }

        /// Package version
        pub fn version(&self) -> &semver::Version {
            &self.version
        }
    }

    impl fmt::Display for PackageId {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{}-{}", self.name, self.version)
        }
    }

    pub mod dependency {
        use serde_derive::{Deserialize, Serialize};
        use std::fmt;

        /// Represents the kind of dependency
        #[derive(
            PartialEq, Eq, Hash, Ord, PartialOrd, Clone, Debug, Copy, Serialize, Deserialize,
        )]
        pub enum Kind {
            /// Normal compile time dependency
            Normal,
            /// Dependency is used for testing purposes
            Development,
            /// Dependency is used at build time
            Build,
        }

        impl fmt::Display for Kind {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                let label = match *self {
                    Kind::Normal => "Normal",
                    Kind::Development => "Development",
                    Kind::Build => "Build",
                };
                f.write_str(label)
            }
        }
    }
}

/// Gathers build information and generates code to make it available at runtime.
/// It writes a file named **built.rs** into Cargo's build output directory.
///
/// # Panics
/// If build-time information failed to be gathered.
pub fn run() {
    let src = env::var("CARGO_MANIFEST_DIR").unwrap();
    let dst = path::Path::new(&env::var("OUT_DIR").unwrap()).join("built.rs");
    built::write_built_file_with_opts(&built::Options::default(), &src, &dst)
        .expect("Failed to acquire build-time information");

    let write_dependencies = || {
        let (dependency_graph, features) = build_dependency_graph();

        let graphviz_dependency_graph = dot::Dot::with_config(
            &dependency_graph.map(
                |_, node| format!("{}={}", node.name().to_string(), node.version()),
                |_, edge| *edge,
            ),
            &[dot::Config::EdgeNoLabel],
        )
        .to_string();

        let mut built_file = OpenOptions::new()
            .append(true)
            .open(&dst)
            .expect("Failed to open file in append mode");

        replace_features_with_actual_names(&dst, features.as_slice());

        writeln!(
            built_file,
            "/// graphviz .dot format for the dependency graph\npub const DEPENDENCIES_GRAPHVIZ_DOT: &str = r#\"{}\"#;",
            graphviz_dependency_graph
        ).unwrap();
    };

    write_dependencies();
}

/// The `built` crate extracts features from the `CARGO_FEATURE_<name>` environment variables.
/// The problem with that approach is that the feature names are not the actual feature names.
///
/// This replaces the features with the actual feature names.
///
/// see: https://github.com/oysterpack/oysterpack/issues/6
fn replace_features_with_actual_names(dst: &path::PathBuf, compile_features: &[String]) {
    let file_contents = fs::read_to_string(dst).unwrap();
    let features: &str = "features";
    let features_uppercased = features.to_uppercase();
    let lines = file_contents
        .lines()
        .filter(|line| {
            !(line.contains(features)
                || line.contains(features_uppercased.as_str())
                || line.starts_with("// EVERYTHING ABOVE THIS POINT"))
        })
        .collect::<Vec<&str>>();

    let file_contents = format!(
        r#"
{}
/// The features that were enabled during compilation."
{}
/// The features as a comma-separated string."
{}
"#,
        lines.join("\n"),
        format!(
            "pub const FEATURES: [&str; {}] = {:?};",
            compile_features.len(),
            compile_features
        ),
        format!(
            r#"pub const FEATURES_STR: &str = {:?};"#,
            compile_features.join(",")
        )
    );

    let mut file = fs::File::create(dst).unwrap();
    writeln!(file, "{}", file_contents).expect("Failed to write to file");
}

/// resolves dependencies and constructs a dependency graph for the current crate
///
/// # Panics
/// If dependency graph failed to be built.
fn build_dependency_graph() -> (
    Graph<metadata::PackageId, metadata::dependency::Kind>,
    Vec<String>, /* features */
) {
    let cargo_config = Config::default().unwrap();
    let workspace = workspace(&cargo_config).unwrap();
    let package = workspace.current().unwrap();
    let features = build_env::features(package);

    let mut registry = registry(&cargo_config, &package).unwrap();
    let features_option = if features.is_empty() {
        None
    } else {
        Some(features.join(" "))
    };

    let (packages, resolve) = resolve(&mut registry, &workspace, features_option).unwrap();

    let ids = packages.package_ids().cloned().collect::<Vec<_>>();
    let packages = registry.get(&ids).unwrap();
    let rustc = cargo_config.rustc(Some(&workspace)).unwrap();
    let target = Some(rustc.host.as_str());
    let cfgs = get_cfgs(&rustc, &target.map(|s| s.to_string())).unwrap();
    let graph = build_graph(
        &resolve,
        &packages,
        package.package_id(),
        target,
        cfgs.as_ref().map(|r| &**r),
    )
    .unwrap();
    (filter_dependencies(&graph.graph), features)
}

fn filter_dependencies(
    graph: &petgraph::Graph<dependencies::Node, cargo::core::dependency::Kind>,
) -> Graph<metadata::PackageId, metadata::dependency::Kind> {
    // convert Graph<Node, Kind> -> Graph<metadata::PackageId,metadata::dependency::Kind> in order to
    // have a graph that we can serialize/deserialize via serde
    let graph = graph.filter_map(
        |node_idx, node| {
            // drop nodes that are only used build dependencies but not as normal dependencies
            match graph
                .edges_directed(node_idx, Direction::Incoming)
                .find(|edge| match edge.weight() {
                    cargo::core::dependency::Kind::Build => true,
                    _ => false,
                }) {
                Some(_) => {
                    match graph.edges_directed(node_idx, Direction::Incoming).find(
                        |edge| match edge.weight() {
                            cargo::core::dependency::Kind::Normal => true,
                            _ => false,
                        },
                    ) {
                        Some(_) => {
                            // keep the node because it is used as a normal dependency
                            Some(metadata::PackageId::new(
                                node.id.name().to_string(),
                                node.id.version().clone(),
                            ))
                        }
                        // drop the node because it is only used as a build dependency
                        None => None,
                    }
                }
                None => Some(metadata::PackageId::new(
                    node.id.name().to_string(),
                    node.id.version().clone(),
                )),
            }
        },
        |_, edge| match edge {
            cargo::core::dependency::Kind::Normal => Some(metadata::dependency::Kind::Normal),
            _ => None,
        },
    );

    // remove nodes that have no edges
    let graph = graph.filter_map(
        |node_idx, node| {
            // remove nodes that have no edges
            graph
                .neighbors_undirected(node_idx)
                .detach()
                .next(&graph)
                .map(|_| node.clone())
        },
        |_, edge| Some(*edge),
    );

    remove_nodes_with_no_incoming_edges(&graph)
}

/// remove nodes that have no incoming edges except for the root node
fn remove_nodes_with_no_incoming_edges(
    graph: &Graph<metadata::PackageId, metadata::dependency::Kind>,
) -> Graph<metadata::PackageId, metadata::dependency::Kind> {
    let mut removed_nodes = false;
    let graph = graph.filter_map(
        |node_idx, node| {
            if node_idx.index() == 0 {
                Some(node.clone())
            } else {
                // remove nodes that have no edges
                match graph
                    .neighbors_directed(node_idx, Direction::Incoming)
                    .detach()
                    .next(&graph)
                {
                    Some(_) => Some(node.clone()),
                    None => {
                        debug!(
                            "build_dependency_graph: dropping node with no edges: {}",
                            node
                        );
                        removed_nodes = true;
                        None
                    }
                }
            }
        },
        |_, edge| Some(*edge),
    );

    if removed_nodes {
        remove_nodes_with_no_incoming_edges(&graph)
    } else {
        graph
    }
}

fn workspace(config: &Config) -> CargoResult<Workspace> {
    let root = important_paths::find_root_manifest_for_wd(config.cwd())?;
    Workspace::new(&root, config)
}

fn registry<'a>(config: &'a Config, package: &Package) -> CargoResult<PackageRegistry<'a>> {
    let mut registry = PackageRegistry::new(config)?;
    registry.add_sources(&[package.package_id().source_id().clone()])?;
    Ok(registry)
}

fn resolve<'a, 'cfg>(
    registry: &mut PackageRegistry<'cfg>,
    workspace: &'a Workspace<'cfg>,
    features: Option<String>,
) -> CargoResult<(PackageSet<'a>, Resolve)> {
    let features = Method::split_features(&features.into_iter().collect::<Vec<_>>());

    let (packages, resolve) = ops::resolve_ws(workspace)?;

    let method = Method::Required {
        dev_deps: false,
        features: &features,
        all_features: false,
        uses_default_features: false,
    };

    let resolve = ops::resolve_with_previous(
        registry,
        workspace,
        method,
        Some(&resolve),
        None,
        &[],
        true,
        true,
    )?;
    Ok((packages, resolve))
}

fn get_cfgs(rustc: &Rustc, target: &Option<String>) -> CargoResult<Option<Vec<Cfg>>> {
    let mut process = util::process(&rustc.path);
    process.arg("--print=cfg").env_remove("RUST_LOG");
    if let Some(ref s) = *target {
        process.arg("--target").arg(s);
    }

    let output = match process.exec_with_output() {
        Ok(output) => output,
        Err(_) => return Ok(None),
    };
    let output = str::from_utf8(&output.stdout).unwrap();
    let lines = output.lines();
    Ok(Some(
        lines.map(Cfg::from_str).collect::<CargoResult<Vec<_>>>()?,
    ))
}

fn build_graph<'a>(
    resolve: &'a Resolve,
    packages: &'a PackageSet,
    root: &'a cargo::core::PackageId,
    target: Option<&str>,
    cfgs: Option<&[Cfg]>,
) -> CargoResult<dependencies::Graph<'a>> {
    let mut graph = dependencies::Graph {
        graph: petgraph::Graph::new(),
        nodes: HashMap::new(),
    };
    let node = dependencies::Node { id: root };
    graph.nodes.insert(root, graph.graph.add_node(node));

    let mut pending = vec![root];

    while let Some(pkg_id) = pending.pop() {
        let idx = graph.nodes[&pkg_id];
        let pkg = packages.get_one(pkg_id)?;

        for raw_dep_id in resolve.deps_not_replaced(pkg_id) {
            let it = pkg
                .dependencies()
                .iter()
                .filter(|d| d.matches_id(raw_dep_id))
                .filter(|d| {
                    d.platform()
                        .and_then(|p| target.map(|t| p.matches(t, cfgs)))
                        .unwrap_or(true)
                });
            let dep_id = match resolve.replacement(raw_dep_id) {
                Some(id) => id,
                None => raw_dep_id,
            };
            for dep in it {
                let dep_idx = match graph.nodes.entry(dep_id) {
                    Entry::Occupied(e) => *e.get(),
                    Entry::Vacant(e) => {
                        pending.push(dep_id);
                        let node = dependencies::Node { id: dep_id };
                        *e.insert(graph.graph.add_node(node))
                    }
                };
                graph.graph.add_edge(idx, dep_idx, dep.kind());
            }
        }
    }

    Ok(graph)
}

mod dependencies {

    use super::*;
    use petgraph;

    #[derive(Debug)]
    pub struct Node<'a> {
        pub id: &'a cargo::core::PackageId,
    }

    #[derive(Debug)]
    pub struct Graph<'a> {
        pub graph: petgraph::Graph<Node<'a>, cargo::core::dependency::Kind>,
        pub nodes: HashMap<&'a cargo::core::PackageId, NodeIndex>,
    }
}

mod build_env {
    use cargo::core::Package;
    use std::collections;
    use std::env;

    type EnvironmentMap = collections::HashMap<String, String>;

    fn get_environment() -> EnvironmentMap {
        let mut envmap = EnvironmentMap::new();
        for (k, v) in env::vars_os() {
            let k = k.into_string();
            let v = v.into_string();
            if let (Ok(k), Ok(v)) = (k, v) {
                envmap.insert(k, v);
            }
        }
        envmap
    }

    /// CARGO_FEATURE_<name> - For each activated feature of the package being built, this environment
    /// variable will be present where <name> is the name of the feature uppercased and having - translated to _
    ///
    /// Since features
    fn features_from_cargo_env() -> Vec<String> {
        let prefix = "CARGO_FEATURE_";
        let mut features = Vec::new();
        for name in get_environment().keys() {
            if name.starts_with(&prefix) {
                let feature = name[prefix.len()..].to_owned();
                features.push(feature);
            }
        }
        features.sort();
        features
    }

    pub fn features(package: &Package) -> Vec<String> {
        use std::collections::HashMap;
        let mut available_features = HashMap::new();
        for name in package.summary().features().keys() {
            available_features.insert(
                name.to_string().to_uppercase().replace("-", "_"),
                name.to_string(),
            );
        }
        for dependency in package.dependencies().iter().filter(|d| d.is_optional()) {
            available_features.insert(
                dependency
                    .package_name()
                    .to_string()
                    .to_uppercase()
                    .replace("-", "_"),
                dependency.package_name().to_string(),
            );
        }

        let features: Vec<String> = features_from_cargo_env()
            .iter()
            .filter(|feature| available_features.contains_key(*feature))
            .map(|feature| available_features[feature].clone())
            .collect();

        if !features.is_empty() {
            println!("*** available features = {:?}", available_features);
        }
        if !features.is_empty() {
            println!("*** built with features = {:?}", features);
        }

        features
    }
}

#[cfg(test)]
mod tests;
