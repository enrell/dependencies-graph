use depg::parser::{bfs, cargo, go, npm, poetry};
use std::collections::HashMap;

#[test]
fn test_bfs_full() {
    let mut adj = HashMap::new();
    adj.insert(
        "A 1".to_string(),
        vec!["B 1".to_string(), "C 1".to_string()],
    );
    adj.insert("B 1".to_string(), vec!["D 1".to_string()]);

    let graph = bfs("A 1", &adj, None, "TestParser");

    assert_eq!(graph.root, "A 1");
    assert_eq!(graph.nodes.len(), 4);
    assert_eq!(graph.edges.len(), 3);
}

#[test]
fn test_bfs_depth_limited() {
    let mut adj = HashMap::new();
    adj.insert("root 1".to_string(), vec!["dep1 1".to_string()]);
    adj.insert("dep1 1".to_string(), vec!["transitive 1".to_string()]);

    let graph = bfs("root 1", &adj, Some(1), "TestParser");

    assert_eq!(graph.nodes.len(), 2);
    assert_eq!(graph.edges.len(), 1);
    let dep1_node = graph.nodes.iter().find(|n| n.id == "dep1 1").unwrap();
    assert_eq!(dep1_node.depth, 1);
}

#[test]
fn test_parse_cargo_lock() {
    let content = r#"
version = 3

[[package]]
name = "depg"
version = "0.1.0"
dependencies = [
 "anyhow",
]

[[package]]
name = "anyhow"
version = "1.0.102"
source = "registry+https://github.com/rust-lang/crates.io-index"
"#;
    let graph = cargo::parse_content(content, None).unwrap();
    assert_eq!(graph.root, "depg 0.1.0");
    assert_eq!(graph.nodes.len(), 2);
    assert_eq!(graph.edges.len(), 1);
    assert_eq!(graph.edges[0].source, "depg 0.1.0");
    assert_eq!(graph.edges[0].target, "anyhow 1.0.102");
}

#[test]
fn test_parse_cargo_empty() {
    let content = r#"version = 3"#;
    let result = cargo::parse_content(content, None);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "No packages found in Cargo.lock"
    );
}

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
    let graph = npm::parse_content(content, None).unwrap();
    assert_eq!(graph.root, "test-project 1.0.0");
    assert_eq!(graph.nodes.len(), 3);
    assert_eq!(graph.edges.len(), 2);
}

#[test]
fn test_parse_npm_unsupported_v1() {
    let content = r#"{
"name": "test",
"version": "1.0.0",
"lockfileVersion": 1,
"dependencies": {}
}"#;
    let result = npm::parse_content(content, None);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("is not supported"));
}

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

    let graph = poetry::parse_content(proj, lock, None).unwrap();
    assert_eq!(graph.root, "depg-py 0.1.0");
    assert_eq!(graph.nodes.len(), 4);
    assert_eq!(graph.edges.len(), 3);
}

#[test]
fn test_parse_go_mod() {
    let content = "example.com/mymod example.com/dep@v1.0.0\nexample.com/dep@v1.0.0 example.com/transitive@v2.0.0";
    let graph = go::parse_content(content, "example.com/mymod", None).unwrap();

    assert_eq!(graph.root, "example.com/mymod 0.0.0");
    assert_eq!(graph.nodes.len(), 3);
    assert_eq!(graph.edges.len(), 2);
}
