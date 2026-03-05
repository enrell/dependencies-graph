use anyhow::{Context, Result, bail};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

use super::StackParser;
use crate::graph::DependencyGraph;

pub struct CargoParser;

impl StackParser for CargoParser {
    fn name(&self) -> &'static str {
        "Rust (Cargo)"
    }

    fn detect(&self, project_path: &Path) -> bool {
        project_path.join("Cargo.lock").exists()
    }

    fn parse(&self, project_path: &Path, max_depth: Option<usize>) -> Result<DependencyGraph> {
        let lock_path = project_path.join("Cargo.lock");
        let content = std::fs::read_to_string(&lock_path)
            .context("Failed to read Cargo.lock. Run `cargo generate-lockfile` first.")?;
        parse_content(&content, max_depth)
    }
}

#[derive(Deserialize)]
struct CargoLock {
    package: Option<Vec<LockPackage>>,
}

#[derive(Deserialize)]
struct LockPackage {
    name: String,
    version: String,
    source: Option<String>,
    dependencies: Option<Vec<String>>,
}

pub fn parse_content(content: &str, max_depth: Option<usize>) -> Result<DependencyGraph> {
    let lock: CargoLock = toml::from_str(content).context("Failed to parse Cargo.lock")?;

    let packages = lock.package.unwrap_or_default();
    if packages.is_empty() {
        bail!("No packages found in Cargo.lock");
    }

    let by_name = build_name_index(&packages);
    let adjacency = build_adjacency(&packages, &by_name);

    let root_id = packages
        .iter()
        .find(|p| p.source.is_none())
        .map(|p| pkg_id(&p.name, &p.version))
        .context("No root package found in Cargo.lock")?;

    Ok(super::bfs(&root_id, &adjacency, max_depth, "Rust (Cargo)"))
}

fn pkg_id(name: &str, version: &str) -> String {
    format!("{name} {version}")
}

fn build_name_index(packages: &[LockPackage]) -> HashMap<&str, Vec<(&str, String)>> {
    let mut index: HashMap<&str, Vec<(&str, String)>> = HashMap::new();
    for pkg in packages {
        let id = pkg_id(&pkg.name, &pkg.version);
        index.entry(&pkg.name).or_default().push((&pkg.version, id));
    }
    index
}

fn build_adjacency(
    packages: &[LockPackage],
    by_name: &HashMap<&str, Vec<(&str, String)>>,
) -> HashMap<String, Vec<String>> {
    let mut adj: HashMap<String, Vec<String>> = HashMap::new();
    for pkg in packages {
        let id = pkg_id(&pkg.name, &pkg.version);
        let deps = pkg
            .dependencies
            .as_ref()
            .map(|deps| {
                deps.iter()
                    .filter_map(|d| resolve_dep(d, by_name))
                    .collect()
            })
            .unwrap_or_default();
        adj.insert(id, deps);
    }
    adj
}

fn resolve_dep(dep_str: &str, by_name: &HashMap<&str, Vec<(&str, String)>>) -> Option<String> {
    let parts: Vec<&str> = dep_str.splitn(3, ' ').collect();
    let name = parts[0];

    by_name.get(name).and_then(|versions| {
        if parts.len() >= 2 {
            let target = parts[1];
            versions
                .iter()
                .find(|(v, _)| *v == target)
                .map(|(_, id)| id.clone())
        } else {
            versions.first().map(|(_, id)| id.clone())
        }
    })
}
