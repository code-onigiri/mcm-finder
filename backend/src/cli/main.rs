// MCM-Finder CLI
use clap::{Parser, Subcommand};
use colored::Colorize;

#[derive(Parser)]
#[command(name = "mcm")]
#[command(about = "Minecraft Mod Finder CLI", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Search for mods
    Search {
        /// Search keywords
        query: String,
        
        /// Minecraft version filter
        #[arg(short, long)]
        version: Option<String>,
        
        /// Loader filter (fabric/forge/quilt/neoforge)
        #[arg(short, long)]
        loader: Option<String>,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Search { query, version, loader } => {
            println!("{}", "Searching for mods...".bright_blue());
            println!("Query: {}", query);
            if let Some(v) = version {
                println!("Version: {}", v);
            }
            if let Some(l) = loader {
                println!("Loader: {}", l);
            }
            
            // Stub - will connect to backend services in Phase 3
            println!("{}", "✓ Found 0 mods (implementation pending)".green());
        }
    }
}
