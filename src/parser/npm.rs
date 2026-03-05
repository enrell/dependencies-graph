use anyhow::{Context, Result, bail};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

use super::StackParser;
use crate::graph::DependencyGraph;

pub struct NpmParser;

impl StackParser for NpmParser {
    fn name(&self) -> &'static str {
        "Node.js (npm)"
    }

    fn detect(&self, project_path: &Path) -> bool {
        project_path.join("package-lock.json").exists()
    }

    fn parse(&self, project_path: &Path, max_depth: Option<usize>) -> Result<DependencyGraph> {
        let lock_path = project_path.join("package-lock.json");
        let content =
            std::fs::read_to_string(&lock_path).context("Failed to read package-lock.json")?;
        parse_content(&content, max_depth)
    }
}

#[derive(Deserialize)]
struct PackageLock {
    name: Option<String>,
    version: Option<String>,
    #[serde(rename = "lockfileVersion")]
    lockfile_version: Option<u32>,
    packages: Option<HashMap<String, PackageEntry>>,
}

#[derive(Deserialize)]
struct PackageEntry {
    name: Option<String>,
    version: Option<String>,
    dependencies: Option<HashMap<String, String>>,
}

fn parse_content(content: &str, max_depth: Option<usize>) -> Result<DependencyGraph> {
    let lock: PackageLock =
        serde_json::from_str(content).context("Failed to parse package-lock.json")?;

    let version = lock.lockfile_version.unwrap_or(1);
    if version < 2 {
        bail!(
            "package-lock.json v{version} is not supported. \
             Run `npm i --package-lock-only` to upgrade to v2+."
        );
    }

    let packages = lock
        .packages
        .context("No 'packages' field in package-lock.json")?;

    let root_entry = packages
        .get("")
        .context("No root entry in package-lock.json")?;

    let root_name = lock
        .name
        .or_else(|| root_entry.name.clone())
        .unwrap_or_else(|| "root".to_string());
    let root_version = lock
        .version
        .or_else(|| root_entry.version.clone())
        .unwrap_or_else(|| "0.0.0".to_string());
    let root_id = format!("{root_name} {root_version}");

    let resolved = build_resolved_index(&packages);
    let adjacency = build_adjacency(&root_id, root_entry, &packages, &resolved);

    Ok(super::bfs(&root_id, &adjacency, max_depth))
}

fn build_resolved_index(packages: &HashMap<String, PackageEntry>) -> HashMap<&str, (&str, String)> {
    packages
        .iter()
        .filter(|(path, _)| !path.is_empty())
        .map(|(path, entry)| {
            let name = extract_package_name(path);
            let ver = entry.version.as_deref().unwrap_or("0.0.0");
            let id = format!("{name} {ver}");
            (path.as_str(), (name, id))
        })
        .collect()
}

fn build_adjacency<'a>(
    root_id: &str,
    root_entry: &PackageEntry,
    packages: &'a HashMap<String, PackageEntry>,
    resolved: &HashMap<&'a str, (&str, String)>,
) -> HashMap<String, Vec<String>> {
    let mut adjacency: HashMap<String, Vec<String>> = HashMap::new();

    let root_targets: Vec<String> = dep_names(root_entry)
        .into_iter()
        .filter_map(|dep_name| {
            let dep_path = format!("node_modules/{dep_name}");
            resolved.get(dep_path.as_str()).map(|(_, id)| id.clone())
        })
        .collect();
    adjacency.insert(root_id.to_string(), root_targets);

    for (path, entry) in packages {
        if path.is_empty() {
            continue;
        }
        let (_, id) = &resolved[path.as_str()];
        let targets: Vec<String> = dep_names(entry)
            .into_iter()
            .filter_map(|dep_name| resolve_npm_dep(path, &dep_name, resolved))
            .collect();
        adjacency.insert(id.clone(), targets);
    }

    adjacency
}

fn extract_package_name(path: &str) -> &str {
    match path.rsplit_once("node_modules/") {
        Some((_, name)) => name,
        None => path,
    }
}

fn dep_names(entry: &PackageEntry) -> Vec<String> {
    entry
        .dependencies
        .as_ref()
        .map(|d| d.keys().cloned().collect())
        .unwrap_or_default()
}

fn resolve_npm_dep(
    parent_path: &str,
    dep_name: &str,
    resolved: &HashMap<&str, (&str, String)>,
) -> Option<String> {
    let nested = format!("{parent_path}/node_modules/{dep_name}");
    if let Some((_, id)) = resolved.get(nested.as_str()) {
        return Some(id.clone());
    }

    let mut search = parent_path.to_string();
    while let Some(idx) = search.rfind("/node_modules/") {
        search.truncate(idx);
        let try_path = if search.is_empty() {
            format!("node_modules/{dep_name}")
        } else {
            format!("{search}/node_modules/{dep_name}")
        };
        if let Some((_, id)) = resolved.get(try_path.as_str()) {
            return Some(id.clone());
        }
    }

    let top = format!("node_modules/{dep_name}");
    resolved.get(top.as_str()).map(|(_, id)| id.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_npm_v3() {
        let content = r#"{
    "name": "test-project",
    "version": "1.0.0",
    "lockfileVersion": 3,
    "packages": {
        "": {
            "name": "test-project",
            "version": "1.0.0",
            "dependencies": {
                "express": "^4.18.0"
            }
        },
        "node_modules/express": {
            "version": "4.18.2",
            "dependencies": {
                "cookie": "0.5.0"
            }
        },
        "node_modules/cookie": {
            "version": "0.5.0"
        }
    }
}"#;
        let graph = parse_content(content, None).unwrap();
        assert_eq!(graph.root, "test-project 1.0.0");
        assert_eq!(graph.nodes.len(), 3);
        assert_eq!(graph.edges.len(), 2);
    }

    #[test]
    fn test_parse_unsupported_v1() {
        let content = r#"{
    "name": "test",
    "version": "1.0.0",
    "lockfileVersion": 1,
    "dependencies": {}
}"#;
        let result = parse_content(content, None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("is not supported"));
    }
}
