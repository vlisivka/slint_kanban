//! src/model/utils.rs
//!
//! Purpose: Shared utility functions extracted from Ticket and Comment modules
//! to eliminate code duplication.

use anyhow::Result;

/// Extract ticket references of the form `#xxxxxx` where x is `[a-z0-9]{6}`
/// from plain text. Returns sorted, deduplicated matches.
pub fn extract_ticket_references(text: &str) -> Vec<String> {
    let mut refs = Vec::new();
    let mut start = 0;
    while let Some((char_idx, _ch)) = text[start..].char_indices().find(|(_, c)| *c == '#') {
        let actual_pos = start + char_idx;
        // Collect the next 6 characters after '#'
        let after_hash: String = text[actual_pos + 1..].chars().take(6).collect();
        if after_hash.len() == 6
            && after_hash
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        {
            refs.push(format!("#{}", after_hash));
        }
        start = actual_pos + 1;
    }
    refs.sort();
    refs.dedup();
    refs
}

/// Parse YAML frontmatter from a document.
/// The expected format is:
/// ```text
/// ---
/// <yaml content>
/// ---
/// <body content>
/// ```
/// Returns `(frontmatter_yaml, body_markdown)`.
pub fn parse_frontmatter(content: &str) -> Result<(String, String)> {
    let parts: Vec<&str> = content.splitn(3, "---").collect();
    if parts.len() < 3 {
        return Err(anyhow::anyhow!("Invalid frontmatter format"));
    }

    let frontmatter = parts[1];
    let body = parts[2].trim().to_string();

    Ok((frontmatter.to_string(), body))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_ticket_references_basic() {
        let text = "Check #abc123 and #def456. Also #123 is too short, and #abcdef78 is too long but should extract #abcdef.";
        let refs = extract_ticket_references(text);
        assert_eq!(refs.len(), 3, "Should extract exactly 3 unique references.");
        assert!(
            refs.contains(&"#abc123".to_string()),
            "Should contain #abc123."
        );
        assert!(
            refs.contains(&"#def456".to_string()),
            "Should contain #def456."
        );
        assert!(
            refs.contains(&"#abcdef".to_string()),
            "Should contain #abcdef (first 6 chars after #)."
        );
    }

    #[test]
    fn test_extract_ticket_references_non_ascii() {
        let text = "Привіт #abc123 і #def456!";
        let refs = extract_ticket_references(text);
        assert_eq!(
            refs.len(),
            2,
            "Should extract exactly 2 references from non-ASCII text."
        );
        assert!(
            refs.contains(&"#abc123".to_string()),
            "Should contain #abc123."
        );
        assert!(
            refs.contains(&"#def456".to_string()),
            "Should contain #def456."
        );
    }

    #[test]
    fn test_extract_ticket_references_no_panic_on_unicode() {
        // '#' at byte 0, then 5 ASCII chars (bytes 1-5), then Cyrillic 'е' (bytes 6-7).
        // Previously this panicked because [1..7] split the Cyrillic 'е'.
        // Now char_indices() handles it correctly — no panic.
        let text = "#12345е";
        let refs = extract_ticket_references(text);
        // '12345е' is not all ASCII lowercase/digit (Cyrillic 'е'), so no reference extracted.
        assert!(
            refs.is_empty(),
            "Should extract no references when the 6 chars after '#' contain non-ASCII."
        );
    }

    #[test]
    fn test_parse_frontmatter_valid() {
        let content = "---\ntitle: \"Test\"\ncreated_at: 2026-07-04\n---\nHello world";
        let (fm, body) = parse_frontmatter(content).unwrap();
        assert_eq!(fm, "\ntitle: \"Test\"\ncreated_at: 2026-07-04\n");
        assert_eq!(body, "Hello world");
    }

    #[test]
    fn test_parse_frontmatter_missing() {
        let content = "---\ntitle: Test";
        let result = parse_frontmatter(content);
        assert!(result.is_err(), "Should error on missing closing ---");
    }
}
