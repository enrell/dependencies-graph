use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

use super::StackParser;
use crate::graph::DependencyGraph;

pub struct PoetryParser;

impl StackParser for PoetryParser {
    fn name(&self) -> &'static str {
        "Python (Poetry)"
    }

    fn detect(&self, project_path: &Path) -> bool {
        project_path.join("poetry.lock").exists() && project_path.join("pyproject.toml").exists()
    }

    fn parse(&self, project_path: &Path, max_depth: Option<usize>) -> Result<DependencyGraph> {
        let lock_path = project_path.join("poetry.lock");
        let proj_path = project_path.join("pyproject.toml");

        let lock_content =
            std::fs::read_to_string(&lock_path).context("Failed to read poetry.lock")?;
        let proj_content =
            std::fs::read_to_string(&proj_path).context("Failed to read pyproject.toml")?;

        parse_content(&proj_content, &lock_content, max_depth)
    }
}

#[derive(Deserialize)]
struct PyProject {
    tool: Option<ToolGroup>,
}

#[derive(Deserialize)]
struct ToolGroup {
    poetry: Option<ProjectMeta>,
}

#[derive(Deserialize)]
struct ProjectMeta {
    name: Option<String>,
    version: Option<String>,
    dependencies: Option<HashMap<String, toml::Value>>,
}

#[derive(Deserialize)]
struct PoetryLock {
    package: Option<Vec<LockPackage>>,
}

#[derive(Deserialize)]
struct LockPackage {
    name: String,
    version: String,
    dependencies: Option<HashMap<String, toml::Value>>,
}

fn parse_content(
    proj_content: &str,
    lock_content: &str,
    max_depth: Option<usize>,
) -> Result<DependencyGraph> {
    let proj: PyProject = toml::from_str(proj_content).context("Failed to parse pyproject.toml")?;
    let lock: PoetryLock = toml::from_str(lock_content).context("Failed to parse poetry.lock")?;

    let packages = lock.package.unwrap_or_default();
    let meta = proj.tool.and_then(|t| t.poetry).unwrap_or(ProjectMeta {
        name: None,
        version: None,
        dependencies: None,
    });

    let root_name = meta.name.unwrap_or_else(|| "project".to_string());
    let root_version = meta.version.unwrap_or_else(|| "0.0.0".to_string());
    let root_id = format!("{root_name} {root_version}");

    let mut direct_deps = meta
        .dependencies
        .map(|d| d.keys().cloned().collect::<Vec<_>>())
        .unwrap_or_default();

    // Ignore Python itself as a dependency to reduce noise
    direct_deps.retain(|d| d.to_lowercase() != "python");

    let mut by_name: HashMap<String, String> = HashMap::new();
    for pkg in &packages {
        // Poetry lockfile names are often lowercase but might be mixed.
        by_name.insert(
            pkg.name.to_lowercase(),
            format!("{} {}", pkg.name, pkg.version),
        );
    }

    let mut adjacency: HashMap<String, Vec<String>> = HashMap::new();

    // Attach direct dependencies to root
    let mut root_edges = Vec::new();
    for dep in direct_deps {
        if let Some(id) = by_name.get(&dep.to_lowercase()) {
            root_edges.push(id.clone());
        }
    }
    adjacency.insert(root_id.clone(), root_edges);

    // Attach transitive dependencies
    for pkg in packages {
        let id = format!("{} {}", pkg.name, pkg.version);
        let mut edges = Vec::new();
        if let Some(deps) = pkg.dependencies {
            for dep_name in deps.keys() {
                if dep_name.to_lowercase() == "python" {
                    continue;
                }
                if let Some(target_id) = by_name.get(&dep_name.to_lowercase()) {
                    edges.push(target_id.clone());
                }
            }
        }
        adjacency.insert(id, edges);
    }

    Ok(super::bfs(&root_id, &adjacency, max_depth))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_poetry() {
        let proj = r#"
[tool.poetry]
name = "depg-py"
version = "0.1.0"

[tool.poetry.dependencies]
python = "^3.9"
requests = "^2.31.0"
"#;

        let lock = r#"
[[package]]
name = "requests"
version = "2.31.0"
[package.dependencies]
certifi = ">=2017.4.17"
urllib3 = ">=1.21.1,<3"

[[package]]
name = "certifi"
version = "2023.7.22"

[[package]]
name = "urllib3"
version = "2.0.4"
"#;

        let graph = parse_content(proj, lock, None).unwrap();
        assert_eq!(graph.root, "depg-py 0.1.0");
        assert_eq!(graph.nodes.len(), 4); // root + requests + certifi + urllib3
        assert_eq!(graph.edges.len(), 3); // root->requests, requests->certifi, requests->urllib3
    }
}
