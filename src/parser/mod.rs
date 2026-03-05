mod cargo;
mod npm;
mod poetry;

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
         Supported: Rust (Cargo.lock), Node.js (package-lock.json), Python (poetry.lock)",
        project_path.display()
    )
}

pub(crate) fn bfs(
    root: &str,
    adjacency: &HashMap<String, Vec<String>>,
    max_depth: Option<usize>,
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bfs_full() {
        let mut adj = HashMap::new();
        adj.insert(
            "A 1".to_string(),
            vec!["B 1".to_string(), "C 1".to_string()],
        );
        adj.insert("B 1".to_string(), vec!["D 1".to_string()]);

        let graph = bfs("A 1", &adj, None);

        assert_eq!(graph.root, "A 1");
        assert_eq!(graph.nodes.len(), 4);
        assert_eq!(graph.edges.len(), 3);
    }

    #[test]
    fn test_bfs_depth_limited() {
        let mut adj = HashMap::new();
        adj.insert("root 1".to_string(), vec!["dep1 1".to_string()]);
        adj.insert("dep1 1".to_string(), vec!["transitive 1".to_string()]);

        let graph = bfs("root 1", &adj, Some(1));

        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(graph.edges.len(), 1);
        let dep1_node = graph.nodes.iter().find(|n| n.id == "dep1 1").unwrap();
        assert_eq!(dep1_node.depth, 1);
    }
}
