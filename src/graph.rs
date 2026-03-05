use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Node {
    pub id: String,
    pub label: String,
    pub version: String,
    pub is_root: bool,
    pub depth: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct Edge {
    pub source: String,
    pub target: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DependencyGraph {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub root: String,
    pub parser: String,
}
