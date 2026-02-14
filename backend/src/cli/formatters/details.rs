use crate::commands::details::CliModDetailsResponse;
use colored::Colorize;
use mcm_finder::models::{CompatibilityStatus, MaintenanceStatus, SearchSessionSummary};

pub fn print_mod_details(details: &CliModDetailsResponse) {
    println!(
        "{} ({})",
        details.profile.canonical_name.bold(),
        details.profile.canonical_slug.dimmed()
    );
    println!(
        "{} {}",
        "Composite relevance:".dimmed(),
        format!("{:.2}", details.profile.composite_relevance).bright_green()
    );
    println!(
        "{} {}",
        "Confidence score:".dimmed(),
        format!("{:.2}", details.profile.confidence_score).bright_green()
    );

    println!("\n{}", "Integrated Summary".bold());
    println!("  {}", details.integrated_summary);
    println!(
        "  {} {}",
        "Discovery relationships:".dimmed(),
        details.discovery_relationship_count
    );
    println!(
        "  {} {}",
        "Metadata conflicts:".dimmed(),
        details.metadata_conflict_count
    );

    println!("\n{}", "Compatibility".bold());
    println!(
        "  {}",
        compatibility_text(&details.profile.compatibility_assessment)
    );

    println!("\n{}", "Maintenance".bold());
    println!(
        "  {}",
        maintenance_text(&details.profile.maintenance_signals)
    );

    println!("\n{}", "Risk Factors".bold());
    if details.profile.adoption_risks.is_empty() {
        println!("  {}", "No major risks detected".green());
    } else {
        for risk in &details.profile.adoption_risks {
            println!(
                "  - {} [{}] {}",
                format!("{:?}", risk.risk_type).yellow(),
                format!("{:?}", risk.severity),
                risk.description
            );
        }
    }

    println!("\n{}", "Discovery Evidence".bold());
    if details.profile.discovery_evidence.is_empty() {
        println!("  {}", "No relationships discovered".dimmed());
    } else {
        for evidence in &details.profile.discovery_evidence {
            println!(
                "  - {:?} -> {} ({:.2})",
                evidence.relationship_type, evidence.related_mod_name, evidence.confidence
            );
            println!("    {}", evidence.context.dimmed());
        }
    }

    println!("\n{}", "Metadata Conflicts".bold());
    if details.profile.metadata_conflicts.is_empty() {
        println!("  {}", "No conflicts detected".green());
    } else {
        for conflict in &details.profile.metadata_conflicts {
            println!(
                "  - {} [{}]",
                conflict.field.yellow(),
                format!("{:?}", conflict.severity)
            );
            for (source, value) in &conflict.values {
                println!("    {}: {}", source.dimmed(), value);
            }
        }
    }
}

pub fn print_session_summary(session: &SearchSessionSummary) {
    println!("{}", "Saved Session".bold());
    println!("  {} {}", "Session ID:".dimmed(), session.id);
    println!("  {} {}", "Created:".dimmed(), session.created_at);
    if let Some(description) = &session.query_description {
        println!("  {} {}", "Description:".dimmed(), description);
    }
    println!(
        "  {} {}",
        "Shortlisted mods:".dimmed(),
        session.shortlisted_mods.len()
    );
    for mod_id in &session.shortlisted_mods {
        let note = session
            .comparison_notes
            .get(mod_id)
            .cloned()
            .unwrap_or_else(|| "No note".to_string());
        println!("    - {} {}", mod_id, format!("({})", note).dimmed());
    }
}

fn compatibility_text(status: &CompatibilityStatus) -> String {
    match status {
        CompatibilityStatus::FullyCompatible => "Fully compatible".to_string(),
        CompatibilityStatus::PartiallyCompatible { issues } => {
            format!("Partially compatible: {}", issues.join("; "))
        }
        CompatibilityStatus::Incompatible { reasons } => {
            format!("Incompatible: {}", reasons.join("; "))
        }
    }
}

fn maintenance_text(status: &MaintenanceStatus) -> String {
    match status {
        MaintenanceStatus::ActivelyMaintained => "Actively maintained".to_string(),
        MaintenanceStatus::Maintenance { last_update_days } => {
            format!(
                "Maintenance mode (last update {} days ago)",
                last_update_days
            )
        }
        MaintenanceStatus::Abandoned => "Likely abandoned".to_string(),
        MaintenanceStatus::Unknown => "Unknown".to_string(),
    }
}
