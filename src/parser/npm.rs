use anyhow::{Context, Result, bail};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

use super::StackParser;
use crate::graph::DependencyGraph;

pub struct NpmParser;

impl StackParser for NpmParser {
    fn name(&self) -> &'static str {
        "Node.js (npm/bun)"
    }

    fn detect(&self, project_path: &Path) -> bool {
        project_path.join("package-lock.json").exists()
            || project_path.join("bun.lock").exists()
            || project_path.join("bun.lockb").exists()
            || (project_path.join("package.json").exists()
                && project_path.join("node_modules").is_dir())
    }

    fn parse(&self, project_path: &Path, max_depth: Option<usize>) -> Result<DependencyGraph> {
        let lock_path = project_path.join("package-lock.json");
        if lock_path.exists() {
            let content =
                std::fs::read_to_string(&lock_path).context("Failed to read package-lock.json")?;
            return parse_content(&content, max_depth);
        }

        parse_from_installed(project_path, max_depth)
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

pub fn parse_content(content: &str, max_depth: Option<usize>) -> Result<DependencyGraph> {
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

    Ok(super::bfs(&root_id, &adjacency, max_depth, "Node.js (npm)"))
}

#[derive(Deserialize)]
struct PackageJson {
    name: Option<String>,
    version: Option<String>,
    dependencies: Option<HashMap<String, String>>,
    #[serde(rename = "optionalDependencies")]
    optional_dependencies: Option<HashMap<String, String>>,
    #[serde(rename = "peerDependencies")]
    peer_dependencies: Option<HashMap<String, String>>,
}

#[derive(Clone)]
struct InstalledPackage {
    version: String,
    dependencies: Vec<String>,
}

fn parse_from_installed(project_path: &Path, max_depth: Option<usize>) -> Result<DependencyGraph> {
    let package_json_path = project_path.join("package.json");
    let root_content = std::fs::read_to_string(&package_json_path).with_context(|| {
        format!(
            "Failed to read {} (required when package-lock.json is missing)",
            package_json_path.display()
        )
    })?;
    let root_pkg: PackageJson = serde_json::from_str(&root_content)
        .context("Failed to parse package.json for root project")?;

    let root_name = root_pkg.name.clone().unwrap_or_else(|| "root".to_string());
    let root_version = root_pkg
        .version
        .clone()
        .unwrap_or_else(|| "0.0.0".to_string());
    let root_id = format!("{root_name} {root_version}");

    let mut installed = HashMap::new();
    collect_installed_packages(&project_path.join("node_modules"), "", &mut installed)?;

    if installed.is_empty() {
        bail!("No installed packages found in node_modules")
    }

    let resolved: HashMap<String, String> = installed
        .iter()
        .map(|(path, pkg)| (path.clone(), format!("{} {}", extract_package_name(path), pkg.version)))
        .collect();

    let mut adjacency: HashMap<String, Vec<String>> = HashMap::new();
    let root_targets = root_dep_names(&root_pkg)
        .into_iter()
        .filter_map(|dep_name| resolve_npm_dep_owned("", &dep_name, &resolved))
        .collect();
    adjacency.insert(root_id.clone(), root_targets);

    for (path, pkg) in &installed {
        if let Some(id) = resolved.get(path) {
            let targets = pkg
                .dependencies
                .iter()
                .filter_map(|dep_name| resolve_npm_dep_owned(path, dep_name, &resolved))
                .collect();
            adjacency.insert(id.clone(), targets);
        }
    }

    Ok(super::bfs(&root_id, &adjacency, max_depth, "Node.js (npm/bun)"))
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

fn collect_installed_packages(
    node_modules_dir: &Path,
    parent_path: &str,
    packages: &mut HashMap<String, InstalledPackage>,
) -> Result<()> {
    if !node_modules_dir.is_dir() {
        return Ok(());
    }

    for entry in std::fs::read_dir(node_modules_dir)
        .with_context(|| format!("Failed to read {}", node_modules_dir.display()))?
    {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if !file_type.is_dir() {
            continue;
        }

        let dir_name = entry.file_name().to_string_lossy().to_string();
        if dir_name == ".bin" {
            continue;
        }

        if dir_name.starts_with('@') {
            for scoped in std::fs::read_dir(entry.path())
                .with_context(|| format!("Failed to read scope directory {}", entry.path().display()))?
            {
                let scoped = scoped?;
                if !scoped.file_type()?.is_dir() {
                    continue;
                }
                let scoped_name = format!("{}/{}", dir_name, scoped.file_name().to_string_lossy());
                collect_one_package(&scoped.path(), parent_path, &scoped_name, packages)?;
            }
            continue;
        }

        collect_one_package(&entry.path(), parent_path, &dir_name, packages)?;
    }

    Ok(())
}

fn collect_one_package(
    package_dir: &Path,
    parent_path: &str,
    package_name: &str,
    packages: &mut HashMap<String, InstalledPackage>,
) -> Result<()> {
    let package_path = if parent_path.is_empty() {
        format!("node_modules/{package_name}")
    } else {
        format!("{parent_path}/node_modules/{package_name}")
    };

    let package_json_path = package_dir.join("package.json");
    if package_json_path.exists() {
        let content = std::fs::read_to_string(&package_json_path)
            .with_context(|| format!("Failed to read {}", package_json_path.display()))?;
        let pkg: PackageJson = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse {}", package_json_path.display()))?;

        let version = pkg.version.clone().unwrap_or_else(|| "0.0.0".to_string());
        let dependencies = root_dep_names(&pkg);

        packages.insert(
            package_path.clone(),
            InstalledPackage {
                version,
                dependencies,
            },
        );
    }

    collect_installed_packages(&package_dir.join("node_modules"), &package_path, packages)?;
    Ok(())
}

fn root_dep_names(pkg: &PackageJson) -> Vec<String> {
    let mut dep_names = Vec::new();

    if let Some(deps) = &pkg.dependencies {
        dep_names.extend(deps.keys().cloned());
    }
    if let Some(deps) = &pkg.optional_dependencies {
        dep_names.extend(deps.keys().cloned());
    }
    if let Some(deps) = &pkg.peer_dependencies {
        dep_names.extend(deps.keys().cloned());
    }

    dep_names.sort();
    dep_names.dedup();
    dep_names
}

fn resolve_npm_dep_owned(
    parent_path: &str,
    dep_name: &str,
    resolved: &HashMap<String, String>,
) -> Option<String> {
    if parent_path.is_empty() {
        let top = format!("node_modules/{dep_name}");
        return resolved.get(&top).cloned();
    }

    let nested = format!("{parent_path}/node_modules/{dep_name}");
    if let Some(id) = resolved.get(&nested) {
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
        if let Some(id) = resolved.get(&try_path) {
            return Some(id.clone());
        }
    }

    let top = format!("node_modules/{dep_name}");
    resolved.get(&top).cloned()
}
