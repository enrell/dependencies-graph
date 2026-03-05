# depg — Dependency Graph Visualizer

## Overview
CLI tool that analyzes project dependency graphs and serves an interactive web visualization.

## Architecture

```
src/
  main.rs      — Entry point, CLI dispatch
  cli.rs       — Clap CLI definitions (run subcommand)
  graph.rs     — Core data model (Node, Edge, DependencyGraph)
  parser.rs    — Cargo.lock parser with BFS depth-limited traversal
  server.rs    — Axum web server with embedded static assets
web/
  index.html   — HTML shell
  style.css    — Dark theme design system
  app.js       — Cytoscape.js graph rendering
```

## Tech Stack
- **Language**: Rust (edition 2024)
- **CLI**: clap v4 (derive)
- **HTTP**: axum v0.8 + tokio
- **Parsing**: toml + serde
- **Frontend**: Vanilla JS + Cytoscape.js (CDN)
- **Error handling**: anyhow

## Conventions
- Static web files are embedded at compile time via `include_str!`
- Graph uses BFS from root with optional `--depth` limit
- Root package = package with no `source` field in Cargo.lock
- Node IDs use format `"name version"`

## Supported Ecosystems
- [x] Rust (Cargo.lock)
- [ ] Node.js (package-lock.json)
- [ ] Go (go.mod)
- [ ] Python (requirements.txt / poetry.lock)
