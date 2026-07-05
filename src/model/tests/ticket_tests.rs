//! src/model/tests/ticket_tests.rs
//!
//! Purpose: Unit tests for Ticket and TicketMetadata logic.

use crate::model::ticket::{Ticket, TicketMetadata};

#[test]
fn test_ticket_metadata_deserialization() {
    let yaml = "
title: Buy Groceries
created_at: 2023-10-27
updated_at: 2023-10-27
";
    let metadata: TicketMetadata = serde_yaml::from_str(yaml).expect("Failed to parse YAML");
    assert_eq!(
        metadata.title, "Buy Groceries",
        "Ticket title should match YAML input"
    );
    assert_eq!(
        metadata.created_at, "2023-10-27",
        "Created date should match YAML input"
    );
}

#[test]
fn test_ticket_metadata_assigned_to() {
    let yaml = "
title: Assigned Task
assigned_to: alice
";
    let metadata: TicketMetadata = serde_yaml::from_str(yaml).expect("Failed to parse YAML");
    assert_eq!(metadata.assigned_to, "alice");

    let yaml_empty = "
title: Unassigned Task
";
    let metadata_empty: TicketMetadata =
        serde_yaml::from_str(yaml_empty).expect("Failed to parse YAML");
    assert_eq!(metadata_empty.assigned_to, "");

    let yaml_blank = "
title: Blank Assignment
assigned_to: 
";
    let metadata_blank: TicketMetadata =
        serde_yaml::from_str(yaml_blank).expect("Failed to parse YAML with blank assigned_to");
    assert_eq!(
        metadata_blank.assigned_to, "",
        "Blank assigned_to should be empty string, but got: '{}'",
        metadata_blank.assigned_to
    );

    let yaml_explicit_empty = "
title: Explicit Empty Assignment
assigned_to: \"\"
";
    let metadata_explicit: TicketMetadata = serde_yaml::from_str(yaml_explicit_empty)
        .expect("Failed to parse YAML with explicit empty string");
    assert_eq!(
        metadata_explicit.assigned_to, "",
        "Explicit empty string assigned_to should be empty string"
    );
}

#[test]
fn test_ticket_matches() {
    let ticket = Ticket {
        id: "T123".to_string(),
        title: "Buy Milk".to_string(),
        description: "Need whole milk for the coffee".to_string(),
        created_at: "now".to_string(),
        updated_at: "now".to_string(),
        assigned_to: "".to_string(),
        author: "me".to_string(),
        points: 0,
        attachment_count: 0,
        comments: vec![],
    };

    assert!(
        ticket.matches("milk"),
        "Should match title (case-insensitive)"
    );
    assert!(
        ticket.matches("MILK"),
        "Should match title (case-insensitive)"
    );
    assert!(ticket.matches("coffee"), "Should match description");
    assert!(ticket.matches("T123"), "Should match ID");
    assert!(ticket.matches("t123"), "Should match ID (case-insensitive)");
    assert!(ticket.matches(""), "Empty query should always match");
    assert!(!ticket.matches("bread"), "Should not match unrelated text");
}

#[test]
fn test_ticket_matches_date_range() {
    let ticket = Ticket {
        id: "T123".to_string(),
        title: "Buy Milk".to_string(),
        description: "Need whole milk for the coffee".to_string(),
        created_at: "2024-02-18 12:00:00".to_string(),
        updated_at: "2024-02-18 12:00:00".to_string(),
        assigned_to: "".to_string(),
        author: "me".to_string(),
        points: 0,
        attachment_count: 0,
        comments: vec![],
    };

    // 1. No filters
    assert!(
        ticket.matches_date_range("", ""),
        "Empty filters should always match"
    );

    // 2. From filter
    assert!(
        ticket.matches_date_range("2024-01-01", ""),
        "Should match if created after 2024-01-01"
    );
    assert!(
        ticket.matches_date_range("2024-02-18", ""),
        "Should match if created on the same day"
    );
    assert!(
        ticket.matches_date_range("2024-02-18 12:00:00", ""),
        "Should match if created at exactly the same time"
    );
    assert!(
        !ticket.matches_date_range("2024-02-19", ""),
        "Should not match if created before 2024-02-19"
    );

    // 3. To filter
    assert!(
        ticket.matches_date_range("", "2024-03-01"),
        "Should match if created before 2024-03-01"
    );
    assert!(
        ticket.matches_date_range("", "2024-02-18"),
        "Should match if created on the same day"
    );
    assert!(
        ticket.matches_date_range("", "2024-02-18 12:00:00"),
        "Should match if created at exactly the same time"
    );
    assert!(
        !ticket.matches_date_range("", "2024-02-17"),
        "Should not match if created after 2024-02-17"
    );

    // 4. Range
    assert!(
        ticket.matches_date_range("2024-01-01", "2024-12-31"),
        "Within range"
    );
    assert!(
        !ticket.matches_date_range("2024-02-19", "2024-12-31"),
        "Out of range (too early)"
    );
    assert!(
        !ticket.matches_date_range("2023-01-01", "2024-02-17"),
        "Out of range (too late)"
    );
}

#[test]
fn test_ticket_metadata_missing_updated_at() {
    let yaml = "
title: Buy Groceries
created_at: 2023-10-27
";
    let metadata: TicketMetadata = serde_yaml::from_str(yaml).expect("Failed to parse YAML");
    assert_eq!(
        metadata.title, "Buy Groceries",
        "Ticket title should match YAML input even with missing fields"
    );
    assert_eq!(
        metadata.created_at, "2023-10-27",
        "Created date should match YAML input"
    );
}

#[test]
fn test_extract_references() {
    let t = Ticket {
        id: "t1".to_string(),
        title: "T".to_string(),
        created_at: "now".to_string(),
        updated_at: "now".to_string(),
        assigned_to: "".to_string(),
        author: "me".to_string(),
        points: 0,
        attachment_count: 0,
        description: "Check #abc123 and #def456. Also #123 is too short, and #abcdef78 is too long but should extract #abcdef. And #abc123 again.".to_string(),
        comments: vec![],
    };
    let refs = t.extract_references();
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
fn test_update_ticket_unassign() {
    let temp_dir = tempfile::tempdir().unwrap();
    let ticket_path = temp_dir.path().join("TUnassign");
    std::fs::create_dir(&ticket_path).unwrap();

    let mut ticket = Ticket {
        id: "TUnassign".to_string(),
        title: "Test".to_string(),
        created_at: "now".to_string(),
        updated_at: "now".to_string(),
        description: "Desc".to_string(),
        assigned_to: "Alice".to_string(),
        author: "Bob".to_string(),
        points: 0,
        attachment_count: 0,
        comments: vec![],
    };
    ticket.save(&ticket_path).unwrap();

    // Verify written
    let loaded = Ticket::load(&ticket_path).unwrap();
    assert_eq!(loaded.assigned_to, "Alice");

    // Unassign
    ticket.assigned_to = "".to_string();
    ticket.save(&ticket_path).unwrap();

    // Verify unassigned
    let reloaded = Ticket::load(&ticket_path).unwrap();
    assert_eq!(reloaded.assigned_to, "", "Should be unassigned");
}
#[test]
fn test_ticket_points_serialization() {
    let temp_dir = tempfile::tempdir().unwrap();
    let ticket_path = temp_dir.path().join("TPoints");
    std::fs::create_dir(&ticket_path).unwrap();

    let ticket = Ticket {
        id: "TPoints".to_string(),
        title: "Point Test".to_string(),
        created_at: "now".to_string(),
        updated_at: "now".to_string(),
        description: "Desc".to_string(),
        assigned_to: "Alice".to_string(),
        author: "Bob".to_string(),
        points: 7,
        attachment_count: 0,
        comments: vec![],
    };
    ticket.save(&ticket_path).unwrap();

    let loaded = Ticket::load(&ticket_path).unwrap();
    assert_eq!(
        loaded.points, 7,
        "Points should be preserved after save/load"
    );

    // Test default points
    let yaml_no_points = "
title: No Points
created_at: 2023-10-27
";
    let metadata: TicketMetadata =
        serde_yaml::from_str(yaml_no_points).expect("Failed to parse YAML");
    assert_eq!(metadata.points, 0, "Points should default to 0 if missing");
}
#[test]
fn test_ticket_save_load_with_colon_in_title() {
    let temp_dir = tempfile::tempdir().unwrap();
    let ticket_path = temp_dir.path().join("TColon");
    std::fs::create_dir(&ticket_path).unwrap();

    // Title with a colon — this is the exact pattern from the bug report:
    // "Помилка: блок коментарів (#~) перекладається як повідомлення з пустим msgid."
    let ticket = Ticket {
        id: "TColon".to_string(),
        title: "Error: colon in title".to_string(),
        created_at: "2024-01-01 12:00:00".to_string(),
        updated_at: "2024-01-01 12:00:00".to_string(),
        description: "Test description".to_string(),
        assigned_to: "".to_string(),
        author: "test".to_string(),
        points: 0,
        attachment_count: 0,
        comments: vec![],
    };

    // Save should succeed
    ticket.save(&ticket_path).unwrap();

    // Load should succeed — this is the bug: it currently fails with
    // "mapping values are not allowed in this context at line 1 column X"
    let loaded = Ticket::load(&ticket_path).unwrap();
    assert_eq!(loaded.title, "Error: colon in title");
}

#[test]
fn test_extract_references_with_non_ascii() {
    let t = Ticket {
        id: "t1".to_string(),
        title: "T".to_string(),
        created_at: "now".to_string(),
        updated_at: "now".to_string(),
        assigned_to: "".to_string(),
        author: "me".to_string(),
        points: 0,
        attachment_count: 0,
        description: "Привіт #abc123 і #def456!".to_string(),
        comments: vec![],
    };
    let refs = t.extract_references();
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
fn test_extract_references_with_non_ascii_in_comment() {
    use crate::model::Comment;
    let comment = Comment {
        id: "tc001abc".to_string(),
        metadata: crate::model::comment::CommentMetadata::default(),
        content: "Привіт #abc123 і #def456!".to_string(),
        references: vec![],
    };
    let refs = comment.extract_references();
    assert_eq!(
        refs.len(),
        2,
        "Should extract exactly 2 references from non-ASCII comment."
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
fn test_extract_references_no_panic_on_unicode_byte_boundary() {
    // '#' at byte 0, then 5 ASCII chars (bytes 1-5), then Cyrillic 'е' (bytes 6-7).
    // Previously this panicked because [1..7] split the Cyrillic 'е'.
    // Now char_indices() handles it correctly — no panic.
    let t = Ticket {
        id: "t1".to_string(),
        title: "T".to_string(),
        created_at: "now".to_string(),
        updated_at: "now".to_string(),
        assigned_to: "".to_string(),
        author: "me".to_string(),
        points: 0,
        attachment_count: 0,
        description: "#12345е".to_string(),
        comments: vec![],
    };
    let refs = t.extract_references();
    // '12345е' is not all ASCII lowercase/digit (Cyrillic 'е'), so no reference extracted.
    assert!(
        refs.is_empty(),
        "Should extract no references when the 6 chars after '#' contain non-ASCII."
    );
}
/// When saving a ticket, the `updated_at` field should NOT be written to the YAML frontmatter.
/// Criteria of success: after saving and reading raw file content, no line starting with "updated_at:" exists.
/// Criteria of failure: updated_at is present in the frontmatter.
#[test]
fn test_ticket_save_no_updated_at() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir().unwrap();
    let ticket_path = temp_dir.path().join("TNoUpdate");
    std::fs::create_dir(&ticket_path).unwrap();

    let ticket = Ticket {
        id: "TNoUpdate".to_string(),
        title: "Test ticket".to_string(),
        created_at: "2024-01-01 12:00:00".to_string(),
        updated_at: "2024-01-01 13:00:00".to_string(),
        description: "Test description".to_string(),
        assigned_to: "".to_string(),
        author: "test".to_string(),
        points: 0,
        attachment_count: 0,
        comments: vec![],
    };

    ticket.save(&ticket_path)?;

    let content = std::fs::read_to_string(ticket_path.join("README.md"))?;
    assert!(
        !content.contains("updated_at:"),
        "YAML frontmatter should not contain updated_at field"
    );

    Ok(())
}
/// Loading a ticket should compute `updated_at` from the README.md file's mtime.
/// Criteria of success: ticket.updated_at is populated from file mtime, not from YAML.
/// Criteria of failure: updated_at is empty or comes from YAML frontmatter.
#[test]
fn test_ticket_load_updated_at_from_mtime() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir().unwrap();
    let ticket_path = temp_dir.path().join("TMtime");
    std::fs::create_dir(&ticket_path).unwrap();

    // Create README.md without updated_at in frontmatter
    let readme_path = ticket_path.join("README.md");
    let mut file = std::fs::File::create(&readme_path)?;
    use std::io::Write;
    writeln!(file, "---")?;
    writeln!(file, "title: \"Mtime Test\"")?;
    writeln!(file, "created_at: 2024-06-15 10:30:00")?;
    writeln!(file, "---")?;
    writeln!(file)?;
    writeln!(file, "Test body")?;
    drop(file);

    // Get the file's mtime
    let metadata = std::fs::metadata(&readme_path)?;
    let _mtime = metadata.modified()?;

    // Load the ticket
    let ticket = Ticket::load(&ticket_path)?;

    // updated_at should NOT be empty — it's computed from mtime
    assert!(
        !ticket.updated_at.is_empty(),
        "updated_at should be populated from file mtime"
    );

    // Load and verify the ticket loads correctly
    assert_eq!(ticket.title, "Mtime Test");

    Ok(())
}
/// Loading a ticket that has `updated_at` in frontmatter (old format) works correctly.
/// The field is ignored — updated_at is computed from mtime instead.
/// Criteria of success: ticket loads without error, updated_at comes from mtime.
/// Criteria of failure: parse error or updated_at comes from YAML.
#[test]
fn test_ticket_load_backward_compat_with_updated_at() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir().unwrap();
    let ticket_path = temp_dir.path().join("TOld");
    std::fs::create_dir(&ticket_path).unwrap();

    // Create README.md with old-style updated_at in frontmatter
    let readme_path = ticket_path.join("README.md");
    let mut file = std::fs::File::create(&readme_path)?;
    use std::io::Write;
    writeln!(file, "---")?;
    writeln!(file, "title: \"Old Ticket\"")?;
    writeln!(file, "created_at: 2024-01-01 08:00:00")?;
    writeln!(file, "updated_at: 2024-06-01 15:00:00")?; // old field — must be ignored
    writeln!(file, "---")?;
    writeln!(file)?;
    writeln!(file, "Body of old ticket")?;
    drop(file);

    // Load should succeed (backward compatible)
    let ticket = Ticket::load(&ticket_path)?;

    assert_eq!(ticket.title, "Old Ticket");
    assert_eq!(ticket.created_at, "2024-01-01 08:00:00");

    // updated_at should come from mtime, NOT from YAML's "2024-06-01 15:00:00"
    assert!(
        ticket.updated_at != "2024-06-01 15:00:00",
        "updated_at should be computed from mtime, not read from YAML"
    );
    assert!(
        !ticket.updated_at.is_empty(),
        "updated_at must be populated"
    );

    Ok(())
}
