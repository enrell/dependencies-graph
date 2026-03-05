pub mod cargo;
pub mod go;
pub mod npm;
pub mod poetry;

use anyhow::{Result, bail};
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::Path;

use crate::graph::{DependencyGraph, Edge, Node};

pub trait StackParser {
    fn name(&self) -> &'static str;
    fn detect(&self, project_path: &Path) -> bool;
    fn parse(&self, project_path: &Path, max_depth: Option<usize>) -> Result<DependencyGraph>;
}

fn parsers() -> Vec<Box<dyn StackParser>> {
    vec![
        Box::new(cargo::CargoParser),
        Box::new(npm::NpmParser),
        Box::new(poetry::PoetryParser),
        Box::new(go::GoParser),
    ]
}

pub fn detect_and_parse(project_path: &Path, max_depth: Option<usize>) -> Result<DependencyGraph> {
    for parser in parsers() {
        if parser.detect(project_path) {
            println!("📦 Detected {} project", parser.name());
            return parser.parse(project_path, max_depth);
        }
    }

    bail!(
        "No supported project detected at {}.\n\
         Supported: Rust (Cargo.lock), Node.js (package-lock.json), Python (poetry.lock), Go (go.mod)",
        project_path.display()
    )
}

pub fn bfs(
    root: &str,
    adjacency: &HashMap<String, Vec<String>>,
    max_depth: Option<usize>,
    parser_name: &str,
) -> DependencyGraph {
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    let mut nodes = Vec::new();
    let mut edges = Vec::new();

    visited.insert(root.to_string());
    queue.push_back((root.to_string(), 0_usize));

    while let Some((id, depth)) = queue.pop_front() {
        let (name, version) = id.split_once(' ').unwrap_or((&id, ""));

        nodes.push(Node {
            id: id.clone(),
            label: name.to_string(),
            version: version.to_string(),
            is_root: id == root,
            depth,
        });

        let can_traverse = max_depth.map_or(true, |max| depth < max);
        if can_traverse {
            if let Some(deps) = adjacency.get(&id) {
                for dep in deps {
                    edges.push(Edge {
                        source: id.clone(),
                        target: dep.clone(),
                    });
                    if visited.insert(dep.clone()) {
                        queue.push_back((dep.clone(), depth + 1));
                    }
                }
            }
        }
    }

    DependencyGraph {
        root: root.to_string(),
        nodes,
        edges,
        parser: parser_name.to_string(),
    }
}
