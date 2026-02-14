use crate::models::ConsolidatedModProfile;

const OPEN_SOURCE_LICENSE_HINTS: [&str; 12] = [
    "mit",
    "apache",
    "gpl",
    "lgpl",
    "agpl",
    "mpl",
    "bsd",
    "isc",
    "epl",
    "unlicense",
    "cc-by",
    "zlib",
];

pub fn apply_filter(
    profiles: Vec<ConsolidatedModProfile>,
    open_source_only: bool,
) -> Vec<ConsolidatedModProfile> {
    if !open_source_only {
        return profiles;
    }

    profiles
        .into_iter()
        .filter(|profile| {
            profile
                .provider_records
                .iter()
                .filter_map(|record| record.license.as_ref())
                .any(|license| is_open_source_license(license))
        })
        .collect()
}

fn is_open_source_license(license: &str) -> bool {
    let normalized = license.trim().to_lowercase();
    OPEN_SOURCE_LICENSE_HINTS
        .iter()
        .any(|hint| normalized.contains(hint))
}

#[cfg(test)]
mod tests {
    use super::is_open_source_license;

    #[test]
    fn recognizes_open_source_licenses() {
        assert!(is_open_source_license("MIT"));
        assert!(is_open_source_license("LGPL-3.0"));
        assert!(!is_open_source_license("All Rights Reserved"));
    }
}
