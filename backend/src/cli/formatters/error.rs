use crate::commands::search::{CliProviderError, SearchCommandError};
use colored::Colorize;

pub fn print_search_error(error: &SearchCommandError) {
    match error {
        SearchCommandError::Validation(message) => {
            eprintln!("{} {}", "Validation error:".red().bold(), message);
        }
        SearchCommandError::Cache(err) => {
            eprintln!("{} {}", "Cache error:".red().bold(), err);
        }
        SearchCommandError::ProvidersUnavailable { failures } => {
            eprintln!(
                "{} {}",
                "Search failed:".red().bold(),
                "all providers unavailable"
            );
            for failure in failures {
                eprintln!("  - {}: {}", failure.provider.yellow(), failure.error);
            }
        }
    }
}

pub fn print_degraded_notice(provider_errors: &[CliProviderError]) {
    if provider_errors.is_empty() {
        return;
    }

    eprintln!(
        "\n{}",
        "Degraded mode: some providers failed. Showing partial results.".yellow()
    );

    for provider_error in provider_errors {
        let severity = if provider_error.severity.eq_ignore_ascii_case("warning") {
            provider_error.severity.yellow()
        } else {
            provider_error.severity.red()
        };

        eprintln!(
            "  - {} [{}]: {} ({})",
            provider_error.provider, severity, provider_error.error, provider_error.reason
        );
    }
}
