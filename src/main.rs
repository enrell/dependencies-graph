mod cli;
mod graph;
mod parser;
mod server;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run { port, depth, open } => {
            let project_path = std::env::current_dir()?;

            println!("🔍 Analyzing dependencies...");
            let graph = parser::parse_cargo_lock(&project_path, depth)?;
            println!(
                "✅ Found {} packages with {} dependency links",
                graph.nodes.len(),
                graph.edges.len()
            );

            println!("🚀 Starting server at http://127.0.0.1:{port}");
            server::start(graph, port, open).await?;
        }
    }

    Ok(())
}
