use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "depg",
    version,
    about = "Analyze and visualize project dependency graphs"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Analyze dependencies and start the visualization server
    Run {
        /// Port for the web server
        #[arg(short, long, default_value_t = 3000)]
        port: u16,

        /// Maximum dependency depth to resolve
        #[arg(short, long)]
        depth: Option<usize>,

        /// Open browser automatically
        #[arg(long, default_value_t = false)]
        open: bool,
    },
}
