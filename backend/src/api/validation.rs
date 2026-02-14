use std::collections::HashMap;
use uuid::Uuid;

const MAX_KEYWORDS: usize = 10;
const MAX_KEYWORD_LEN: usize = 64;

pub fn validate_keywords(keywords: &[String]) -> Result<Vec<String>, String> {
    let sanitized: Vec<String> = keywords
        .iter()
        .map(|keyword| sanitize_text(keyword))
        .filter(|keyword| !keyword.is_empty())
        .collect();

    if sanitized.is_empty() || sanitized.len() > MAX_KEYWORDS {
        return Err(format!(
            "keywords must contain between 1 and {} non-empty values",
            MAX_KEYWORDS
        ));
    }

    if sanitized
        .iter()
        .any(|keyword| keyword.len() > MAX_KEYWORD_LEN)
    {
        return Err(format!(
            "each keyword must be {} characters or fewer",
            MAX_KEYWORD_LEN
        ));
    }

    Ok(sanitized)
}

pub fn sanitize_string_list(
    values: &[String],
    field_name: &str,
    max_items: usize,
    max_item_len: usize,
) -> Result<Vec<String>, String> {
    let sanitized: Vec<String> = values
        .iter()
        .map(|value| sanitize_text(value))
        .filter(|value| !value.is_empty())
        .collect();

    if sanitized.len() > max_items {
        return Err(format!(
            "{} supports at most {} values",
            field_name, max_items
        ));
    }

    if sanitized.iter().any(|value| value.len() > max_item_len) {
        return Err(format!(
            "{} values must be {} characters or fewer",
            field_name, max_item_len
        ));
    }

    Ok(sanitized)
}

pub fn validate_minecraft_version(version: Option<&str>) -> Result<Option<String>, String> {
    let Some(version) = version else {
        return Ok(None);
    };

    let sanitized = sanitize_text(version);
    if sanitized.is_empty() {
        return Ok(None);
    }

    if sanitized.len() > 20 {
        return Err("minecraft_version must be 20 characters or fewer".to_string());
    }

    if !sanitized
        .chars()
        .all(|ch| ch.is_ascii_digit() || ch == '.' || ch == '+' || ch == 'x' || ch == 'X')
    {
        return Err(
            "minecraft_version must only contain digits, dots, optional x-range, or + suffix"
                .to_string(),
        );
    }

    if !is_valid_minecraft_version(&sanitized) {
        return Err(
            "minecraft_version must match format like 1.20, 1.20.1, 1.20.x, or 1.20+".to_string(),
        );
    }

    Ok(Some(sanitized))
}

pub fn parse_uuid(value: &str, field_name: &str) -> Result<Uuid, String> {
    Uuid::parse_str(value).map_err(|_| {
        format!(
            "{} must be a valid UUID (xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx)",
            field_name
        )
    })
}

pub fn parse_uuid_list(
    values: &[String],
    field_name: &str,
    max_items: usize,
) -> Result<Vec<Uuid>, String> {
    if values.len() > max_items {
        return Err(format!(
            "{} supports at most {} values",
            field_name, max_items
        ));
    }

    values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            parse_uuid(value, field_name).map_err(|_| {
                format!(
                    "{}[{}] must be a valid UUID (xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx)",
                    field_name, index
                )
            })
        })
        .collect()
}

pub fn validate_comparison_notes(
    notes: &HashMap<String, String>,
    max_entries: usize,
    max_note_len: usize,
) -> Result<HashMap<Uuid, String>, String> {
    if notes.len() > max_entries {
        return Err(format!(
            "comparison_notes supports at most {} entries",
            max_entries
        ));
    }

    let mut validated = HashMap::new();
    for (mod_id, note) in notes {
        let parsed_id = parse_uuid(mod_id, "comparison_notes key")?;
        let sanitized_note = sanitize_text(note);
        if sanitized_note.len() > max_note_len {
            return Err(format!(
                "comparison note for {} exceeds {} characters",
                mod_id, max_note_len
            ));
        }
        if !sanitized_note.is_empty() {
            validated.insert(parsed_id, sanitized_note);
        }
    }

    Ok(validated)
}

pub fn sanitize_optional_text(
    value: Option<&str>,
    field_name: &str,
    max_len: usize,
) -> Result<Option<String>, String> {
    let Some(value) = value else {
        return Ok(None);
    };

    let sanitized = sanitize_text(value);
    if sanitized.is_empty() {
        return Ok(None);
    }

    if sanitized.len() > max_len {
        return Err(format!(
            "{} must be {} characters or fewer",
            field_name, max_len
        ));
    }

    Ok(Some(sanitized))
}

fn sanitize_text(value: &str) -> String {
    value
        .chars()
        .filter(|ch| !ch.is_control())
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn is_valid_minecraft_version(version: &str) -> bool {
    let without_plus = version.trim_end_matches('+');
    let normalized = without_plus
        .strip_suffix(".x")
        .or_else(|| without_plus.strip_suffix(".X"))
        .unwrap_or(without_plus);

    let parts: Vec<&str> = normalized.split('.').collect();
    matches!(parts.len(), 2 | 3)
        && parts
            .iter()
            .all(|part| !part.is_empty() && part.chars().all(|ch| ch.is_ascii_digit()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_keyword_count_and_sanitization() {
        let keywords = vec![
            "  sodium ".to_string(),
            "\n".to_string(),
            "fabric".to_string(),
        ];
        let validated = validate_keywords(&keywords).unwrap();
        assert_eq!(validated, vec!["sodium".to_string(), "fabric".to_string()]);
    }

    #[test]
    fn validates_minecraft_version_patterns() {
        assert_eq!(
            validate_minecraft_version(Some("1.20.1")).unwrap(),
            Some("1.20.1".to_string())
        );
        assert!(validate_minecraft_version(Some("1.20-rc1")).is_err());
    }

    #[test]
    fn validates_uuid_lists() {
        let values = vec![Uuid::new_v4().to_string()];
        let parsed = parse_uuid_list(&values, "ids", 5).unwrap();
        assert_eq!(parsed.len(), 1);
    }
}
