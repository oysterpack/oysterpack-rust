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
        registry::PackageRegistry, resolver::Method, Package, PackageId, Resolve,
        Workspace,
    },
    ops,
    util::{self, important_paths, CargoResult, Cfg, Rustc},
    Config,
};
use metadata::{self, dependency};
use petgraph::{
    self,
    graph::{Graph, NodeIndex},
    visit::EdgeRef,
};
use serde_json;
use std::{
    collections::{hash_map::Entry, HashMap},
    path,
    str::{self, FromStr},
    env,
    fs::{OpenOptions},
    io::prelude::*
};


/// Gathers build information and generates code to make it available at runtime.
///
/// # Panics
/// If build-time information failed to be gathered.
pub fn run() {
    let src = env::var("CARGO_MANIFEST_DIR").unwrap();
    let dst = path::Path::new(&env::var("OUT_DIR").unwrap()).join("built.rs");
    built::write_built_file_with_opts(&built::Options::default(), &src, &dst).expect("Failed to acquire build-time information");


    let write_dependencies = || {
        let env = build_env::get_environment();
        let features = build_env::features(&env);
        let features = if features.is_empty() {
            None
        } else {
            Some(features)
        };
        let dependency_graph = build_dependency_graph(features);
        let all_dependencies : Vec<metadata::PackageId> = dependencies::all(&dependency_graph).into_iter().map(|pkg_id|pkg_id.clone()).collect();
        let all_dependencies = serde_json::to_string(&all_dependencies).expect("Failed to serialize dependencies to JSON");

        let mut built_file = OpenOptions::new()
            .append(true)
            .open(&dst).expect("Failed to open file in append mode");


        built_file.write_all(b"/// An array of effective dependencies as a JSON array.\n").unwrap();
        writeln!(
            built_file,
            "pub const DEPENDENCIES_JSON: &str = r#\"{}\"#;",
            all_dependencies
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
    let (packages, resolve) = resolve(&mut registry, &workspace, features).unwrap();

    let ids = packages.package_ids().cloned().collect::<Vec<_>>();
    let packages = registry.get(&ids);
    let root = package.package_id();
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

    let graph = graph.graph.filter_map(
        |node_idx, node| {
            Some(metadata::PackageId::new(
                node.id.name().to_string(),
                node.id.version().clone(),
            ))
        },
        |edge_index, edge| match edge {
            Kind::Normal => Some(metadata::dependency::Kind::Normal),
            _ => None,
        },
    );

    let graph = graph.filter_map(
        |node_idx, node| {
            // remove nodes that have no edges
            match graph.neighbors_undirected(node_idx).detach().next(&graph) {
                Some(_) => Some(node.clone()),
                None => None,
            }
        },
        |edge_index, edge| Some(*edge),
    );

    graph
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
    use petgraph;
    use metadata;

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
    pub fn all(graph: &petgraph::Graph<metadata::PackageId, metadata::dependency::Kind>) -> Vec<&metadata::PackageId> {
        graph.raw_nodes().iter().map(|node|&node.weight).collect()
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



#[cfg(test)]
mod tests;
