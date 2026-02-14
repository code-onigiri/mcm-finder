use crate::commands::search::CliSearchResponse;
use colored::Colorize;

pub mod details;
pub mod error;
pub mod progress;

pub fn print_results(response: &CliSearchResponse) {
    println!(
        "{} {}",
        "Found".bright_blue(),
        response.data.len().to_string().bright_blue()
    );

    if let Some(reason) = &response.metadata.degradation_reason {
        println!("{} {}", "Degraded mode:".yellow(), reason.yellow());
    }

    for profile in &response.data {
        let providers: Vec<String> = profile
            .provider_records
            .iter()
            .map(|record| record.source.as_str().to_string())
            .collect();

        let summary = profile
            .provider_records
            .first()
            .map(|record| record.summary.clone())
            .unwrap_or_else(|| "No summary".to_string());

        println!(
            "\n{} {}",
            profile.canonical_name.bold(),
            format!("[{}]", providers.join(", ")).dimmed()
        );
        println!("  {}", summary);
        println!(
            "  {} {}",
            "Downloads:".dimmed(),
            profile.total_downloads.to_string().green()
        );

        if !profile.all_supported_versions.is_empty() {
            println!(
                "  {} {}",
                "Versions:".dimmed(),
                profile.all_supported_versions.join(", ")
            );
        }

        if !profile.all_supported_loaders.is_empty() {
            let loaders: Vec<String> = profile
                .all_supported_loaders
                .iter()
                .map(|loader| format!("{:?}", loader))
                .collect();
            println!("  {} {}", "Loaders:".dimmed(), loaders.join(", "));
        }

        println!(
            "  {} {}",
            "URL:".dimmed(),
            profile.canonical_slug.underline()
        );
    }

    if let (Some(cached_at), Some(expires_at), Some(cache_version)) = (
        response.metadata.cached_at,
        response.metadata.expires_at,
        response.metadata.cache_version.as_ref(),
    ) {
        println!(
            "\n{} {} | {} {} | {} {}",
            "cache_version:".dimmed(),
            cache_version,
            "cached_at:".dimmed(),
            cached_at,
            "expires_at:".dimmed(),
            expires_at
        );
    }
}
