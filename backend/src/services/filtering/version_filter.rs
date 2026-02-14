use crate::models::ConsolidatedModProfile;

pub fn supports_requested_version(supported_versions: &[String], requested_version: &str) -> bool {
    let requested = requested_version.trim();
    if requested.is_empty() {
        return true;
    }

    let strict_requested = requested.ends_with('+');
    let normalized_requested = requested.trim_end_matches('+').to_lowercase();

    supported_versions
        .iter()
        .any(|supported| version_matches(supported, &normalized_requested, strict_requested))
}

pub fn apply_filter(
    profiles: Vec<ConsolidatedModProfile>,
    requested_version: Option<&str>,
) -> Vec<ConsolidatedModProfile> {
    let Some(version) = requested_version.map(str::trim).filter(|v| !v.is_empty()) else {
        return profiles;
    };

    profiles
        .into_iter()
        .filter(|profile| supports_requested_version(&profile.all_supported_versions, version))
        .collect()
}

fn version_matches(supported: &str, requested: &str, strict_requested: bool) -> bool {
    let supported = supported.trim().to_lowercase();
    if supported.is_empty() || requested.is_empty() {
        return false;
    }

    if supported == requested {
        return true;
    }

    if strict_requested {
        return false;
    }

    if let Some(prefix) = requested.strip_suffix(".x") {
        return supported == prefix || supported.starts_with(&format!("{}.", prefix));
    }

    if let Some(prefix) = supported.strip_suffix(".x") {
        return requested == prefix || requested.starts_with(&format!("{}.", prefix));
    }

    let requested_parts = requested.split('.').count();
    let supported_parts = supported.split('.').count();

    if requested_parts == 2 {
        return supported == requested || supported.starts_with(&format!("{}.", requested));
    }

    if supported_parts == 2 {
        return requested == supported || requested.starts_with(&format!("{}.", supported));
    }

    false
}

#[cfg(test)]
mod tests {
    use super::supports_requested_version;

    #[test]
    fn matches_patch_versions_for_minor_request() {
        assert!(supports_requested_version(
            &["1.20.1".to_string(), "1.19.4".to_string()],
            "1.20"
        ));
    }

    #[test]
    fn expands_x_ranges() {
        assert!(supports_requested_version(
            &["1.20.1".to_string(), "1.19.4".to_string()],
            "1.20.x"
        ));
    }

    #[test]
    fn treats_plus_as_exact() {
        assert!(!supports_requested_version(
            &["1.20.1".to_string()],
            "1.20+"
        ));
        assert!(supports_requested_version(&["1.20".to_string()], "1.20+"));
    }
}
