use anyhow::{Context, Result, bail};
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

use super::StackParser;
use crate::graph::DependencyGraph;

pub struct GoParser;

impl StackParser for GoParser {
    fn name(&self) -> &'static str {
        "Go (go mod)"
    }

    fn detect(&self, project_path: &Path) -> bool {
        project_path.join("go.mod").exists()
    }

    fn parse(&self, project_path: &Path, max_depth: Option<usize>) -> Result<DependencyGraph> {
        let output = Command::new("go")
            .arg("mod")
            .arg("graph")
            .current_dir(project_path)
            .output()
            .context("Failed to execute `go mod graph`. Is Go installed?")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("`go mod graph` failed: {}", stderr);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        let root_output = Command::new("go")
            .arg("list")
            .arg("-m")
            .current_dir(project_path)
            .output()
            .context("Failed to execute `go list -m`")?;

        let root_mod = if root_output.status.success() {
            String::from_utf8_lossy(&root_output.stdout)
                .trim()
                .to_string()
        } else {
            "root".to_string()
        };

        parse_content(&stdout, &root_mod, max_depth)
    }
}

pub fn parse_content(
    content: &str,
    root_mod: &str,
    max_depth: Option<usize>,
) -> Result<DependencyGraph> {
    let mut adjacency: HashMap<String, Vec<String>> = HashMap::new();

    for line in content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() == 2 {
            let source = format_id(parts[0]);
            let target = format_id(parts[1]);
            adjacency.entry(source).or_default().push(target);
        }
    }

    let root_id = format_id(root_mod);

    // Ensure root is in adjacency even if it has no dependencies,
    // to kick off the BFS cleanly.
    adjacency.entry(root_id.clone()).or_default();

    Ok(super::bfs(&root_id, &adjacency, max_depth, "Go (go mod)"))
}

fn format_id(raw: &str) -> String {
    if let Some((name, version)) = raw.split_once('@') {
        format!("{} {}", name, version)
    } else {
        format!("{} 0.0.0", raw)
    }
}
