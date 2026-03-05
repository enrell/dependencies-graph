# depg — Dependency Graph Visualizer

## Overview
CLI tool that analyzes project dependency graphs and serves an interactive web visualization.

## Architecture

```
src/
  main.rs      — Entry point, CLI dispatch
  lib.rs       — Library interface for integration tests
  cli.rs       — Clap CLI definitions (run subcommand)
  graph.rs     — Core data model (Node, Edge, DependencyGraph)
  parser/      — Unified parser system with BFS traversal
    mod.rs     — Parser trait and detection logic
    cargo.rs   — Cargo.lock implementation
    npm.rs     — Node.js parser (package-lock.json + bun.lock/bun.lockb + node_modules fallback)
    poetry.rs  — poetry.lock implementation
    go.rs      &mdash; go.mod implementation
  server.rs    — Axum web server with embedded static assets
tests/         — External integration test suite
  parser_tests.rs
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
- [x] Node.js (package-lock.json, bun.lock/bun.lockb, node_modules fallback)
- [x] Go (go.mod)
- [x] Python (poetry.lock)
