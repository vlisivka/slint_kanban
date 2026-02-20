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
    assert_eq!(
        metadata.updated_at, "2023-10-27",
        "Updated date should match YAML input"
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
    assert_eq!(
        metadata.updated_at, "",
        "Updated date should be empty if missing in YAML"
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
        description: "Check #abc123 and #def456. Also #123 is too short, and #abcdef78 is too long but should extract #abcdef. And #abc123 again.".to_string(),
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
