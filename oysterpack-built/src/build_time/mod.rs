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

//! used as a build dependency - activated via the build-time feature

use built;
use cargo::{
    core::{
        dependency::Kind, manifest::ManifestMetadata, package::PackageSet,
        registry::PackageRegistry, resolver::Method, Package, PackageId, Resolve, Workspace,
    },
    ops,
    util::{self, important_paths, CargoResult, Cfg, Rustc},
    Config,
};
use oysterpack_app_metadata::metadata::{self, dependency};
use petgraph::{
    self, dot,
    graph::{node_index, Graph, NodeIndex},
    visit::EdgeRef,
    Direction,
};
use serde_json;
use std::{
    collections::{hash_map::Entry, HashMap},
    env,
    fs::OpenOptions,
    io::prelude::*,
    path,
    str::{self, FromStr},
};

/// Gathers build information and generates code to make it available at runtime.
///
/// # Panics
/// If build-time information failed to be gathered.
pub fn run() {
    let src = env::var("CARGO_MANIFEST_DIR").unwrap();
    let dst = path::Path::new(&env::var("OUT_DIR").unwrap()).join("built.rs");
    built::write_built_file_with_opts(&built::Options::default(), &src, &dst)
        .expect("Failed to acquire build-time information");

    let write_dependencies = || {
        let env = build_env::get_environment();
        let features = build_env::features(&env);
        let features = if features.is_empty() {
            None
        } else {
            Some(features)
        };
        let mut dependency_graph = build_dependency_graph(features);

        let graphviz_dependency_graph = dot::Dot::with_config(
            &dependency_graph.map(
                |node_idx, node| format!("{}={}", node.name().to_string(), node.version()),
                |edge_index, edge| *edge,
            ),
            &[dot::Config::EdgeNoLabel],
        ).to_string();

        let mut built_file = OpenOptions::new()
            .append(true)
            .open(&dst)
            .expect("Failed to open file in append mode");

        writeln!(
            built_file,
            "/// graphviz .dot format for the dependency graph\npub const DEPENDENCIES_GRAPHVIZ_DOT: &str = r#\"{}\"#;",
            graphviz_dependency_graph
        ).unwrap();
    };

    write_dependencies();
}

/// resolves dependencies and constructs a dependency graph
///
/// # Panics
/// If dependency graph failed to be built.
pub fn build_dependency_graph(
    features: Option<Vec<String>>,
) -> Graph<metadata::PackageId, dependency::Kind> {
    let cargo_config = Config::default().unwrap();
    let workspace = workspace(&cargo_config).unwrap();
    let package = workspace.current().unwrap();
    let mut registry = registry(&cargo_config, &package).unwrap();
    let features = features.map(|features| features.join(" "));
    debug!("build_dependency_graph: features = {:?}", features);
    let (packages, resolve) = resolve(&mut registry, &workspace, features).unwrap();

    let ids = packages.package_ids().cloned().collect::<Vec<_>>();
    let packages = registry.get(&ids);
    let root = package.package_id();
    debug!("build_dependency_graph: root package id = {}", root);
    let rustc = cargo_config.rustc(Some(&workspace)).unwrap();
    let target = Some(rustc.host.as_str());
    let cfgs = get_cfgs(&rustc, &target.map(|s| s.to_string())).unwrap();
    let graph = build_graph(
        &resolve,
        &packages,
        package.package_id(),
        target,
        cfgs.as_ref().map(|r| &**r),
    ).unwrap();

    filter_dependencies(graph.graph)
}

fn filter_dependencies(
    graph: petgraph::Graph<dependencies::Node, Kind>,
) -> Graph<metadata::PackageId, dependency::Kind> {
    debug!(
        "build_dependency_graph: initial node count = {}",
        graph.node_count()
    );

    // convert Graph<Node, Kind> -> Graph<metadata::PackageId,metadata::dependency::Kind> in order to
    // have a graph that we can serialize/deserialize via serde
    let graph = graph.filter_map(
        |node_idx, node| {
            // drop nodes that are only used build dependencies but not as normal dependencies
            match graph
                .edges_directed(node_idx, Direction::Incoming)
                .find(|edge| match edge.weight() {
                    Kind::Build => true,
                    _ => false,
                }) {
                Some(_) => {
                    match graph.edges_directed(node_idx, Direction::Incoming).find(
                        |edge| match edge.weight() {
                            Kind::Normal => true,
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
        |edge_index, edge| match edge {
            Kind::Normal => Some(metadata::dependency::Kind::Normal),
            _ => {
                debug!("build_dependency_graph: dropping edge: {:?}", edge);
                None
            }
        },
    );

    // remove nodes that have no edges
    let graph = graph.filter_map(
        |node_idx, node| {
            // remove nodes that have no edges
            match graph.neighbors_undirected(node_idx).detach().next(&graph) {
                Some(_) => Some(node.clone()),
                None => {
                    debug!(
                        "build_dependency_graph: dropping node with no edges: {}",
                        node
                    );
                    None
                }
            }
        },
        |edge_index, edge| Some(*edge),
    );

    remove_nodes_with_no_incoming_edges(graph)
}

/// remove nodes that have no incoming edges except for the root node
fn remove_nodes_with_no_incoming_edges(
    graph: Graph<metadata::PackageId, dependency::Kind>,
) -> Graph<metadata::PackageId, dependency::Kind> {
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
        |edge_index, edge| Some(*edge),
    );

    if removed_nodes {
        remove_nodes_with_no_incoming_edges(graph)
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
    root: &'a PackageId,
    target: Option<&str>,
    cfgs: Option<&[Cfg]>,
) -> CargoResult<dependencies::Graph<'a>> {
    let mut graph = dependencies::Graph {
        graph: petgraph::Graph::new(),
        nodes: HashMap::new(),
    };
    let node = dependencies::Node {
        id: root,
        metadata: packages.get(root)?.manifest().metadata(),
    };
    graph.nodes.insert(root, graph.graph.add_node(node));

    let mut pending = vec![root];

    while let Some(pkg_id) = pending.pop() {
        let idx = graph.nodes[&pkg_id];
        let pkg = packages.get(pkg_id)?;

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
                        let node = dependencies::Node {
                            id: dep_id,
                            metadata: packages.get(dep_id)?.manifest().metadata(),
                        };
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
    use oysterpack_app_metadata::metadata;
    use petgraph;

    #[derive(Debug)]
    pub struct Node<'a> {
        pub id: &'a PackageId,
        pub metadata: &'a ManifestMetadata,
    }

    #[derive(Debug)]
    pub struct Graph<'a> {
        pub graph: petgraph::Graph<Node<'a>, Kind>,
        pub nodes: HashMap<&'a PackageId, NodeIndex>,
    }

    /// Returns all dependency package ids
    pub fn all(
        graph: &petgraph::Graph<metadata::PackageId, metadata::dependency::Kind>,
    ) -> Vec<&metadata::PackageId> {
        let mut dependencies: Vec<&metadata::PackageId> =
            graph.raw_nodes().iter().map(|node| &node.weight).collect();
        dependencies.sort();
        dependencies
    }
}

mod build_env {
    use std::collections;
    use std::env;

    type EnvironmentMap = collections::HashMap<String, String>;

    pub fn get_environment() -> EnvironmentMap {
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

    pub fn features(envmap: &EnvironmentMap) -> Vec<String> {
        let prefix = "CARGO_FEATURE_";
        let mut features = Vec::new();
        for name in envmap.keys() {
            if name.starts_with(&prefix) {
                features.push(name[prefix.len()..].to_owned());
            }
        }
        features.sort();
        features
    }
}

#[cfg(all(test, feature = "build-time"))]
mod tests;
