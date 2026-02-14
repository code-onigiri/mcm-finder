// MCM-Finder CLI
mod cache;
mod commands;
mod formatters;

use clap::{Parser, Subcommand};
use commands::details::{execute_details, DetailsCommandArgs};
use commands::load_session::{execute_load_session, LoadSessionCommandArgs};
use commands::save_session::{execute_save_session, SaveSessionCommandArgs};
use commands::search::{execute_search, SearchCommandArgs};

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

        /// Loader filter(s): fabric,forge,quilt,neoforge
        #[arg(short = 'l', long = "loader", value_delimiter = ',')]
        loader: Vec<String>,

        /// Category filter(s)
        #[arg(long = "category", value_delimiter = ',')]
        category: Vec<String>,

        /// Minimum downloads threshold
        #[arg(long = "min-downloads")]
        min_downloads: Option<u64>,

        /// Only include mods with open-source licenses
        #[arg(long = "open-source")]
        open_source: bool,

        /// Only include mods updated within N days
        #[arg(long = "updated-within")]
        updated_within: Option<u32>,

        /// Sort mode: relevance, update-recency, downloads, created
        #[arg(long, value_parser = ["relevance", "update-recency", "downloads", "created"])]
        sort: Option<String>,

        /// Provider filter(s): modrinth,curseforge
        #[arg(short = 'p', long = "provider", value_delimiter = ',')]
        provider: Vec<String>,

        /// Bypass local CLI cache and refresh provider data
        #[arg(long)]
        force_refresh: bool,
    },
    /// Show consolidated details for one mod
    Details {
        /// Consolidated mod UUID
        mod_id: String,
        /// API base URL (default: http://127.0.0.1:3000)
        #[arg(long)]
        api_base_url: Option<String>,
    },
    /// Save a session from a prior search
    SaveSession {
        /// Search query UUID returned by API search response
        query_id: String,
        /// Selected mod UUID(s)
        #[arg(long = "mod-id", value_delimiter = ',')]
        mod_ids: Vec<String>,
        /// Optional session description
        #[arg(long)]
        description: Option<String>,
        /// Comparison note as mod_id=note, repeatable
        #[arg(long = "note")]
        notes: Vec<String>,
        /// API base URL (default: http://127.0.0.1:3000)
        #[arg(long)]
        api_base_url: Option<String>,
    },
    /// Load an existing saved session
    LoadSession {
        /// Session UUID
        session_id: String,
        /// API base URL (default: http://127.0.0.1:3000)
        #[arg(long)]
        api_base_url: Option<String>,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Search {
            query,
            version,
            loader,
            category,
            min_downloads,
            open_source,
            updated_within,
            sort,
            provider,
            force_refresh,
        } => {
            let args = SearchCommandArgs {
                query,
                version,
                loaders: loader,
                categories: category,
                min_downloads,
                open_source_only: open_source,
                updated_within_days: updated_within,
                sort,
                providers: provider,
                force_refresh,
            };

            match execute_search(args).await {
                Ok(response) => {
                    formatters::print_results(&response);
                    if !response.metadata.provider_errors.is_empty() {
                        formatters::error::print_degraded_notice(
                            &response.metadata.provider_errors,
                        );
                    }
                }
                Err(err) => {
                    formatters::error::print_search_error(&err);
                    std::process::exit(1);
                }
            }
        }
        Commands::Details {
            mod_id,
            api_base_url,
        } => {
            let args = DetailsCommandArgs {
                mod_id,
                api_base_url,
            };
            match execute_details(args).await {
                Ok(details) => formatters::details::print_mod_details(&details),
                Err(err) => {
                    eprintln!("{}", err);
                    std::process::exit(1);
                }
            }
        }
        Commands::SaveSession {
            query_id,
            mod_ids,
            description,
            notes,
            api_base_url,
        } => {
            let args = SaveSessionCommandArgs {
                query_id,
                mod_ids,
                description,
                notes,
                api_base_url,
            };
            match execute_save_session(args).await {
                Ok(session) => formatters::details::print_session_summary(&session),
                Err(err) => {
                    eprintln!("{}", err);
                    std::process::exit(1);
                }
            }
        }
        Commands::LoadSession {
            session_id,
            api_base_url,
        } => {
            let args = LoadSessionCommandArgs {
                session_id,
                api_base_url,
            };
            match execute_load_session(args).await {
                Ok(session) => formatters::details::print_session_summary(&session),
                Err(err) => {
                    eprintln!("{}", err);
                    std::process::exit(1);
                }
            }
        }
    }
}
