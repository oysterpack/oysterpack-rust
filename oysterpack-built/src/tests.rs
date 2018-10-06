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

//! provides unit test support

use chrono;
use fern;
use log;
use std::io;

pub const MODULE_NAME: &str = "oysterpack_built";

fn init_logging() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S%.6f]"),
                record.level(),
                record.target(),
                message
            ))
        }).level(log::LevelFilter::Warn)
        .level_for(MODULE_NAME, log::LevelFilter::Debug)
        .chain(io::stdout())
        .apply()?;

    Ok(())
}

lazy_static! {
    pub static ref INIT_FERN: Result<(), fern::InitError> = init_logging();
}

pub fn run_test<F: FnOnce() -> ()>(test: F) {
    let _ = *INIT_FERN;
    test()
}

use cargo::core::dependency::Kind;
use cargo::core::manifest::ManifestMetadata;
use cargo::core::package::PackageSet;
use cargo::core::registry::PackageRegistry;
use cargo::core::resolver::Method;
use cargo::core::shell::Shell;
use cargo::core::{Package, PackageId, Resolve, Workspace};
use cargo::ops;
use cargo::util::{self, important_paths, CargoResult, Cfg, Rustc};
use cargo::{CliResult, Config};
use petgraph;
use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeRef;
use petgraph::EdgeDirection;
use serde_json;
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::str::{self, FromStr};

op_build_mod!();

#[test]
fn build_info() {
    info!("{}", concat!(env!("OUT_DIR"), "/built.rs"));
    info!(
        "This is version {}{}, built for {} by {}.",
        build::PKG_VERSION,
        build::GIT_VERSION.map_or_else(|| "".to_owned(), |v| format!(" (git {})", v)),
        build::TARGET,
        build::RUSTC_VERSION
    );
    info!(
        "I was built with profile \"{}\", features \"{}\" on {}",
        build::PROFILE,
        build::FEATURES_STR,
        build::BUILT_TIME_UTC
    );
}

#[test]
fn test_cargo() {
    run_test(|| {
        let cargo_config = Config::default().unwrap();
        let workspace = workspace(&cargo_config, None).unwrap();
        let package = workspace.current().unwrap();
        info!("package: {}-{}", package.name(), package.version());
        let mut registry = registry(&cargo_config, &package).unwrap();
        let features = Some("build-time".to_string());
        let (packages, resolve) =
            resolve(&mut registry, &workspace, features, false, true, true).unwrap();
        info!("packages: {:?}", packages);
        info!("resolve: {:?}", resolve);

        let ids = packages.package_ids().cloned().collect::<Vec<_>>();
        let packages = registry.get(&ids);

        let root = package.package_id();
        info!("root: {}", root);

        let rustc = cargo_config.rustc(Some(&workspace)).unwrap();
        let target = Some(rustc.host.as_str());
        info!("target: {:?}", target);

        let cfgs = get_cfgs(&rustc, &target.map(|s| s.to_string())).unwrap();
        info!("cfgs: {:?}", cfgs);
        let graph = build_graph(
            &resolve,
            &packages,
            package.package_id(),
            target,
            cfgs.as_ref().map(|r| &**r),
        ).unwrap();
        //debug!("graph: {:?}", graph);

        //        let graph_json = serde_json::to_string_pretty(&graph);
        //        info!("graph : {}", graph_json);
    });
}

fn workspace(config: &Config, manifest_path: Option<PathBuf>) -> CargoResult<Workspace> {
    let root = match manifest_path {
        Some(path) => path,
        None => important_paths::find_root_manifest_for_wd(config.cwd())?,
    };
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
    all_features: bool,
    no_default_features: bool,
    no_dev_dependencies: bool,
) -> CargoResult<(PackageSet<'a>, Resolve)> {
    let features = Method::split_features(&features.into_iter().collect::<Vec<_>>());

    let (packages, resolve) = ops::resolve_ws(workspace)?;

    let method = Method::Required {
        dev_deps: !no_dev_dependencies,
        features: &features,
        all_features,
        uses_default_features: !no_default_features,
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

#[derive(Debug)]
struct Node<'a> {
    id: &'a PackageId,
    metadata: &'a ManifestMetadata,
}

#[derive(Debug)]
struct Graph<'a> {
    graph: petgraph::Graph<Node<'a>, Kind>,
    nodes: HashMap<&'a PackageId, NodeIndex>,
}

fn build_graph<'a>(
    resolve: &'a Resolve,
    packages: &'a PackageSet,
    root: &'a PackageId,
    target: Option<&str>,
    cfgs: Option<&[Cfg]>,
) -> CargoResult<Graph<'a>> {
    let mut graph = Graph {
        graph: petgraph::Graph::new(),
        nodes: HashMap::new(),
    };
    let node = Node {
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
                        let node = Node {
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
