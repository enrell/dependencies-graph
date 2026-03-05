# depg

A fast, interactive dependency graph visualizer for modern codebases. **depg** analyzes your project's dependencies and serves a beautiful, interactive local web UI to explore your architecture.

## Features
- ⚡ **Zero-config auto-detection**: Run `depg run` and it automatically detects your ecosystem.
- 📦 **Supported Ecosystems**: 
  - Rust (`Cargo.lock`, complete transitive dependencies)
  - Node.js/npm (`package-lock.json` v2/v3, fully supports npm's hoisting resolution)
  - Go (`go.mod`, exact transitive links)
  - Python (`poetry.lock`, drops language noise)
- 🕸️ **Advanced Visualization**: Uses Cytoscape.js and the `fcose` force-directed algorithm for an organic, highly-readable layout of thousands of nodes.
- 🎨 **Premium UI**: Cyber-minimalist dark mode built for engineers. Features node inspection, interactive neighbor highlighting, and search functionality.
- 🎚️ **Depth Control**: Limit your dependency graph using `--depth` to focus precisely on what matters.

## Installation

### 1. Using shell install scripts 

**Linux / macOS**
```bash
curl -fsSL https://raw.githubusercontent.com/your-username/depg/main/install.sh | sh
```

**Windows (PowerShell)**
```powershell
irm https://raw.githubusercontent.com/your-username/depg/main/install.ps1 | iex
```

### 2. Using Cargo

If you have Rust installed natively, you can install directly via Cargo:

```bash
cargo install --git https://github.com/your-username/depg.git
```

*Alternatively, clone the repository and run `cargo install --path .`*

## Usage
Navigate to any supported project directory (e.g., a Rust project with a `Cargo.lock` or a JS project with a `package-lock.json`), and run:

```bash
depg run
```

### Options
- `--port <PORT>`: Specify the web server port (default: 3000).
- `--depth <N>`: Limit the recursive depth of the graph to visualize only direct dependencies (`depth=1`) or deeper.
- `--open`: Automatically open the visualization in your default web browser.

Example:
```bash
depg run --depth 2 --port 8080 --open
```

## How It Works
1. `depg` searches your current directory for known lockfiles.
2. It parses the complete dependency tree, handling ecosystem-specific resolution semantics.
3. The graph data is serialized and served via an embedded, high-performance Axum web server.
4. The client fetches the graph and renders it instantly onto a physics-driven, responsive canvas.

## Architecture
- **Backend Core**: Rust (Clap, Anyhow, Serde)
- **Extensible Parser Engine**: Built using a dynamic traits system making it easy to plug in support for new languages.
- **Web Server**: Axum + Tokio (static assets compiled directly into the binary).
- **Frontend Engine**: Vanilla JS + Cytoscape.js.

## License
MIT
