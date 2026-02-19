use super::*;
use std::fs::File;
use std::io::Write;
use tempfile::tempdir;

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
fn test_ticket_matches() {
    let ticket = Ticket {
        id: "T123".to_string(),
        title: "Buy Milk".to_string(),
        description: "Need whole milk for the coffee".to_string(),
        created_at: "now".to_string(),
        updated_at: "now".to_string(),
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
fn test_board_scanning() -> anyhow::Result<()> {
    let root = tempdir()?;
    let root_path = root.path().to_path_buf();
    let tickets_dir = root_path.join("Tickets");
    let queues_dir = root_path.join("Queue");

    std::fs::create_dir(&tickets_dir)?;
    std::fs::create_dir(&queues_dir)?;

    // Create Ticket T1
    let t1_path = tickets_dir.join("T1");
    std::fs::create_dir(&t1_path)?;
    let mut t1_readme = File::create(t1_path.join("README.md"))?;
    write!(
        t1_readme,
        "---\ntitle: Task 1\ncreated_at: 2023-01-01\nupdated_at: 2023-01-01\n---\nBody 1"
    )?;

    // Create Queue Q1
    let q1_path = queues_dir.join("Q1");
    std::fs::create_dir(&q1_path)?;

    // Symlink T1 to Q1
    #[cfg(unix)]
    std::os::unix::fs::symlink(&t1_path, q1_path.join("link_to_T1"))?;

    let board = Board::load(root_path)?;
    assert_eq!(
        board.queues.len(),
        1,
        "Board should have exactly one queue after scanning"
    );
    let q1 = &board.queues[0];
    assert_eq!(q1.id, "Q1", "Queue ID should match folder name");
    assert_eq!(q1.tickets.len(), 1, "Queue should contain one ticket");
    assert_eq!(
        q1.tickets[0].title, "Task 1",
        "Ticket title should match README content"
    );
    assert_eq!(q1.tickets[0].id, "T1", "Ticket ID should match folder name");

    Ok(())
}

#[test]
fn test_board_scanning_multiple_queues() -> anyhow::Result<()> {
    let root = tempdir()?;
    let root_path = root.path().to_path_buf();
    let tickets_dir = root_path.join("Tickets");
    let queues_dir = root_path.join("Queue");

    std::fs::create_dir_all(&tickets_dir)?;
    std::fs::create_dir_all(&queues_dir)?;

    // Ticket ttt123
    let t1_path = tickets_dir.join("ttt123");
    std::fs::create_dir(&t1_path)?;
    let mut f1 = File::create(t1_path.join("README.md"))?;
    write!(
        f1,
        "---\ntitle: T123\ncreated_at: 2023-01-01\nupdated_at: 2023-01-01\n---\nBody"
    )?;

    // Ticket ttt456
    let t2_path = tickets_dir.join("ttt456");
    std::fs::create_dir(&t2_path)?;
    let mut f2 = File::create(t2_path.join("README.md"))?;
    write!(
        f2,
        "---\ntitle: T456\ncreated_at: 2023-01-02\nupdated_at: 2023-01-02\n---\nBody"
    )?;

    // Queue q1
    let q1_path = queues_dir.join("q1");
    std::fs::create_dir(&q1_path)?;
    #[cfg(unix)]
    std::os::unix::fs::symlink(&t1_path, q1_path.join("link1"))?;

    // Queue q2
    let q2_path = queues_dir.join("q2");
    std::fs::create_dir(&q2_path)?;
    #[cfg(unix)]
    std::os::unix::fs::symlink(&t2_path, q2_path.join("link2"))?;

    let board = Board::load(root_path)?;

    // We can't guarantee order of directory reading, so we search
    let q1 = board
        .queues
        .iter()
        .find(|q| q.id == "q1")
        .expect("q1 not found");
    assert_eq!(q1.tickets.len(), 1, "q1 should have one ticket");
    assert_eq!(
        q1.tickets[0].id, "ttt123",
        "q1 should contain ticket ttt123"
    );

    let q2 = board
        .queues
        .iter()
        .find(|q| q.id == "q2")
        .expect("q2 not found");
    assert_eq!(q2.tickets.len(), 1, "q2 should have one ticket");
    assert_eq!(
        q2.tickets[0].id, "ttt456",
        "q2 should contain ticket ttt456"
    );

    Ok(())
}

#[test]
fn test_move_ticket() -> anyhow::Result<()> {
    let root = tempdir()?;
    let root_path = root.path().to_path_buf();
    let tickets_dir = root_path.join("Tickets");
    let queues_dir = root_path.join("Queue");

    std::fs::create_dir_all(&tickets_dir)?;
    std::fs::create_dir_all(&queues_dir)?;

    // Create Ticket T1
    let t1_path = tickets_dir
        .join("T1")
        .canonicalize()
        .unwrap_or(tickets_dir.join("T1"));
    if !t1_path.exists() {
        std::fs::create_dir(&t1_path)?;
    }

    let mut t1_readme = File::create(t1_path.join("README.md"))?;
    write!(
        t1_readme,
        "---\ntitle: T1\ncreated_at: 2023-01-01\nupdated_at: 2023-01-01\n---\nBody"
    )?;

    // Create Queues Q1, Q2
    let q1_path = queues_dir.join("Q1");
    let q2_path = queues_dir.join("Q2");
    std::fs::create_dir(&q1_path)?;
    std::fs::create_dir(&q2_path)?;

    // Symlink T1 to Q1
    #[cfg(unix)]
    std::os::unix::fs::symlink(&t1_path, q1_path.join("T1"))?;

    let board = Board::load(root_path.clone())?;
    assert_eq!(
        board
            .queues
            .iter()
            .find(|q| q.id == "Q1")
            .unwrap()
            .tickets
            .len(),
        1,
        "Q1 should initially have one ticket"
    );
    assert_eq!(
        board
            .queues
            .iter()
            .find(|q| q.id == "Q2")
            .unwrap()
            .tickets
            .len(),
        0,
        "Q2 should initially be empty"
    );

    // Move T1 from Q1 to Q2
    board.move_ticket("T1", "Q1", "Q2")?;

    let board_after = Board::load(root_path)?;
    assert_eq!(
        board_after
            .queues
            .iter()
            .find(|q| q.id == "Q1")
            .unwrap()
            .tickets
            .len(),
        0,
        "Q1 should be empty after moving the ticket"
    );
    assert_eq!(
        board_after
            .queues
            .iter()
            .find(|q| q.id == "Q2")
            .unwrap()
            .tickets
            .len(),
        1,
        "Q2 should have the moved ticket"
    );
    assert_eq!(
        board_after
            .queues
            .iter()
            .find(|q| q.id == "Q2")
            .unwrap()
            .tickets[0]
            .id,
        "T1",
        "The ticket ID in the target queue should match the moved ticket ID. Verify Board::move_ticket correctly re-links the ticket."
    );

    Ok(())
}

#[test]
fn test_delete_ticket() -> anyhow::Result<()> {
    let root = tempdir()?;
    let root_dir = root.path().to_path_buf();
    let tickets_dir = root_dir.join("Tickets");
    let queues_dir = root_dir.join("Queue");
    let deleted_dir = root_dir.join("Deleted");

    std::fs::create_dir_all(&tickets_dir)?;
    std::fs::create_dir_all(&queues_dir)?;

    let t1_path = tickets_dir
        .join("T1")
        .canonicalize()
        .unwrap_or(tickets_dir.join("T1"));
    if !t1_path.exists() {
        std::fs::create_dir(&t1_path)?;
    }
    let mut f1 = File::create(t1_path.join("README.md"))?;
    write!(
        f1,
        "---\ntitle: T1\ncreated_at: 2023-01-01\nupdated_at: 2023-01-01\n---\nBody"
    )?;

    let q1_path = queues_dir.join("Q1");
    std::fs::create_dir(&q1_path)?;
    #[cfg(unix)]
    std::os::unix::fs::symlink(&t1_path, q1_path.join("T1_link"))?;

    let board = Board::load(root_dir.clone())?;
    assert_eq!(
        board.queues[0].tickets.len(),
        1,
        "Queue should have one ticket before deletion"
    );

    board.delete_ticket("T1")?;

    assert!(
        !t1_path.exists(),
        "The ticket folder in 'Tickets/' should be removed after deletion. Ensure Board::delete_ticket correctly moves the folder."
    );
    assert!(
        deleted_dir.join("T1").exists(),
        "The ticket folder should be moved to 'Deleted/' folder after deletion. Check Board::delete_ticket implementation."
    );
    assert!(
        !q1_path.join("T1_link").exists(),
        "The ticket symlink in the queue folder should be removed after deletion. Check Board::delete_ticket logic for cleaning up symlinks."
    );

    let board_after = Board::load(root_dir)?;
    assert_eq!(
        board_after.queues[0].tickets.len(),
        0,
        "The board model should reflect the deletion by showing 0 tickets in the queue. Verify core scanning logic in Board::load."
    );

    Ok(())
}

#[test]
fn test_create_ticket() -> anyhow::Result<()> {
    let root = tempdir()?;
    let root_path = root.path().to_path_buf();

    std::fs::create_dir_all(root_path.join("Tickets"))?;
    let q1_path = root_path.join("Queue").join("Q1");
    std::fs::create_dir_all(&q1_path)?;

    let board = Board::load(root_path.clone())?;
    let tid = board.create_ticket("My New Task", "My Description", "Q1")?;

    assert_eq!(tid.len(), 6, "New ticket ID should be 6 characters long");
    assert!(
        tid.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()),
        "New ticket ID should be lowercase alphanumeric"
    );
    assert!(
        root_path.join("Tickets").join(&tid).exists(),
        "Ticket directory should be created"
    );
    assert!(
        root_path
            .join("Tickets")
            .join(&tid)
            .join("README.md")
            .exists(),
        "README.md should be created in ticket directory"
    );
    assert!(
        q1_path.join(&tid).exists(),
        "A symlink to the new ticket should be created in the target queue folder. Check symlink logic in Board::create_ticket."
    );

    let board2 = Board::load(root_path)?;
    assert_eq!(
        board2.queues[0].tickets.len(),
        1,
        "Board should have one ticket after creation"
    );
    assert_eq!(
        board2.queues[0].tickets[0].title, "My New Task",
        "Ticket title should match input"
    );
    assert_eq!(
        board2.queues[0].tickets[0].description, "My Description",
        "Ticket description should match input"
    );

    Ok(())
}

#[test]
fn test_update_ticket() -> anyhow::Result<()> {
    let root = tempdir()?;
    let root_path = root.path().to_path_buf();

    std::fs::create_dir_all(root_path.join("Tickets"))?;
    std::fs::create_dir_all(root_path.join("Queue").join("Q1"))?;

    let board = Board::load(root_path.clone())?;
    let tid = board.create_ticket("Original", "Original Description", "Q1")?;

    board.update_ticket(&tid, "Updated Title", "Updated Description")?;

    let board2 = Board::load(root_path)?;
    let t = &board2.queues[0].tickets[0];
    assert_eq!(t.title, "Updated Title", "Updated title should match input");
    assert_eq!(
        t.description, "Updated Description",
        "Updated description should match input"
    );

    Ok(())
}
#[test]
fn test_initialization() -> anyhow::Result<()> {
    let root = tempdir()?;
    let root_path = root.path();

    // 1. Initial run: should create default queues with numbers
    Board::ensure_initialized(root_path)?;

    let board = Board::load(root_path.to_path_buf())?;
    assert_eq!(
        board.queues.len(),
        7,
        "Default initialization should create 7 queues"
    );
    assert_eq!(board.queues[0].id, "1. Incoming", "Queue 0 ID mismatch. Board::ensure_initialized should create '1. Incoming' as the first queue.");
    assert_eq!(board.queues[1].id, "2. ToDo", "Queue 1 ID mismatch. Board::ensure_initialized should create '2. ToDo' as the second queue.");
    assert_eq!(board.queues[2].id, "3. Doing", "Queue 2 ID mismatch. Board::ensure_initialized should create '3. Doing' as the third queue.");
    assert_eq!(board.queues[3].id, "4. Reviewing", "Queue 3 ID mismatch. Board::ensure_initialized should create '4. Reviewing' as the fourth queue.");
    assert_eq!(board.queues[4].id, "5. Testing", "Queue 4 ID mismatch. Board::ensure_initialized should create '5. Testing' as the fifth queue.");
    assert_eq!(board.queues[5].id, "6. Done", "Queue 5 ID mismatch. Board::ensure_initialized should create '6. Done' as the sixth queue.");
    assert_eq!(board.queues[6].id, "7. Archive", "Queue 6 ID mismatch. Board::ensure_initialized should create '7. Archive' as the seventh queue.");

    // 2. Existing queue run: should NOT create defaults if something exists
    let root2 = tempdir()?;
    let root_path2 = root2.path();
    std::fs::create_dir_all(root_path2.join("Queue").join("CustomQueue"))?;

    Board::ensure_initialized(root_path2)?;
    assert!(
        root_path2.join("Queue/CustomQueue").exists(),
        "Custom queue should still exist"
    );
    assert!(
        !root_path2.join("Queue/1. Incoming").exists(),
        "Default queues should not be created if some exist"
    );

    Ok(())
}

#[test]
fn test_queue_limit_creation() -> anyhow::Result<()> {
    let root = tempdir()?;
    let root_path = root.path().to_path_buf();

    Board::ensure_initialized(&root_path)?;
    let mut board = Board::load(root_path.clone())?;

    // Set limit to 1 for "2. ToDo"
    board.config.set_limit("2. ToDo".to_string(), 1);
    board.config.write(&root_path)?;

    // Reload board to pick up config change if necessary, or just use the current board
    // Board::load re-reads the config.
    let board = Board::load(root_path)?;

    // Create first ticket - should succeed
    board.create_ticket("Task 1", "Desc 1", "2. ToDo")?;

    // Create second ticket - should fail
    let result = board.create_ticket("Task 2", "Desc 2", "2. ToDo");
    assert!(
        result.is_err(),
        "Board::create_ticket should return an error if the queue has reached its limit. Verify limit enforcement in create_ticket."
    );
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("has reached its limit"),
        "The error message from Board::create_ticket should clearly state that the queue limit has been reached. Check error message consistency."
    );

    Ok(())
}

#[test]
fn test_queue_limit_moving() -> anyhow::Result<()> {
    let root = tempdir()?;
    let root_path = root.path().to_path_buf();

    Board::ensure_initialized(&root_path)?;
    let mut board = Board::load(root_path.clone())?;

    // Set limit to 1 for "3. Doing"
    board.config.set_limit("3. Doing".to_string(), 1);
    board.config.write(&root_path)?;

    let board = Board::load(root_path)?;

    // Create two tickets in ToDo
    let tid1 = board.create_ticket("Task 1", "Desc 1", "2. ToDo")?;
    let tid2 = board.create_ticket("Task 2", "Desc 2", "2. ToDo")?;

    // Move first ticket to Doing - should succeed
    board.move_ticket(&tid1, "2. ToDo", "3. Doing")?;

    // Move second ticket to Doing - should fail
    let result = board.move_ticket(&tid2, "2. ToDo", "3. Doing");
    assert!(
        result.is_err(),
        "Board::move_ticket should return an error if the target queue has reached its limit. Verify limit enforcement in move_ticket."
    );
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("has reached its limit"),
        "The error message from Board::move_ticket should clearly state that the target queue limit has been reached. Check error message consistency."
    );

    Ok(())
}

#[test]
fn test_create_ticket_invalid_queue() -> anyhow::Result<()> {
    let root = tempdir()?;
    let root_dir = root.path().to_path_buf();
    Board::ensure_initialized(&root_dir)?;
    let board = Board::load(root_dir)?;

    let result = board.create_ticket("Title", "Desc", "NonExistentQueue");
    assert!(result.is_err(), "Ticket creation in a non-existent queue should return an error. Verify that the queue ID passed exists and is correctly handled in Board::create_ticket.");
    Ok(())
}

#[test]
fn test_move_ticket_invalid_queue() -> anyhow::Result<()> {
    let root = tempdir()?;
    let root_dir = root.path().to_path_buf();
    Board::ensure_initialized(&root_dir)?;
    let board = Board::load(root_dir)?;

    let tid = board.create_ticket("Title", "Desc", "1. Incoming")?;
    let result = board.move_ticket(&tid, "1. Incoming", "NonExistentQueue");
    assert!(result.is_err(), "Moving a ticket to a non-existent queue should return an error. Check Board::move_ticket logic for handling invalid target queue IDs.");
    Ok(())
}

#[test]
fn test_config_load_invalid_toml() -> anyhow::Result<()> {
    let root = tempdir()?;
    let root_path = root.path().to_path_buf();
    let config_path = root_path.join("config.toml");

    let mut f = File::create(config_path)?;
    write!(f, "invalid = toml = [")?;

    let result = Config::load(&root_path);
    assert!(result.is_err(), "Loading a configuration file with invalid TOML should return an error. Check Config::load error handling and toml::from_str integration.");
    Ok(())
}

#[test]
fn test_delete_non_existent_ticket() -> anyhow::Result<()> {
    let root = tempdir()?;
    let root_dir = root.path().to_path_buf();
    Board::ensure_initialized(&root_dir)?;
    let board = Board::load(root_dir)?;

    let result = board.delete_ticket("NonExistentID");
    assert!(result.is_err(), "Deleting a ticket with a non-existent ID should return an error. Verify Board::delete_ticket checks if the ticket exists before attempting deletion.");
    Ok(())
}

#[test]
fn test_load_ticket_missing_readme() -> anyhow::Result<()> {
    let root = tempdir()?;
    let ticket_path = root.path().join("T1");
    std::fs::create_dir(&ticket_path)?;

    let board = Board {
        tickets_path: root.path().join("Tickets"),
        queues_path: root.path().join("Queue"),
        queues: vec![],
        config: Config::default(),
    };
    let result = board.load_ticket(&ticket_path);
    assert!(
        result.is_err(),
        "Loading a ticket with missing README.md should return an error."
    );
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("README.md not found"),
        "Error message should mention missing README.md"
    );
    Ok(())
}

#[test]
fn test_load_ticket_invalid_format() -> anyhow::Result<()> {
    let root = tempdir()?;
    let ticket_path = root.path().join("T1");
    std::fs::create_dir(&ticket_path)?;
    std::fs::write(
        ticket_path.join("README.md"),
        "Invalid format - no separators",
    )?;

    let board = Board {
        tickets_path: root.path().join("Tickets"),
        queues_path: root.path().join("Queue"),
        queues: vec![],
        config: Config::default(),
    };
    let result = board.load_ticket(&ticket_path);
    assert!(
        result.is_err(),
        "Loading a ticket with invalid format (missing separators) should return an error."
    );
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Invalid ticket format"),
        "Error message should mention invalid ticket format"
    );
    Ok(())
}

#[test]
fn test_load_ticket_invalid_yaml() -> anyhow::Result<()> {
    let root = tempdir()?;
    let ticket_path = root.path().join("T1");
    std::fs::create_dir(&ticket_path)?;
    std::fs::write(
        ticket_path.join("README.md"),
        "---\ninvalid: yaml: [\n---\nBody",
    )?;

    let board = Board {
        tickets_path: root.path().join("Tickets"),
        queues_path: root.path().join("Queue"),
        queues: vec![],
        config: Config::default(),
    };
    let result = board.load_ticket(&ticket_path);
    assert!(
        result.is_err(),
        "Loading a ticket with invalid YAML should return an error."
    );
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Failed to parse YAML"),
        "Error message should mention YAML parsing failure"
    );
    Ok(())
}

#[test]
fn test_resolve_queue_id() -> anyhow::Result<()> {
    let board = Board {
        tickets_path: PathBuf::new(),
        queues_path: PathBuf::new(),
        config: Config::default(),
        queues: vec![
            Queue {
                id: "Q1".to_string(),
                name: "Queue 1".to_string(),
                tickets: vec![],
                limit: None,
                visible: true,
            },
            Queue {
                id: "Q2".to_string(),
                name: "Queue 2".to_string(),
                tickets: vec![],
                limit: None,
                visible: true,
            },
        ],
    };

    assert_eq!(
        board.resolve_queue_id("Q1"),
        "Q1",
        "Direct ID should resolve to itself"
    );
    assert_eq!(
        board.resolve_queue_id("index:0"),
        "Q1",
        "index:0 should resolve to the first queue"
    );
    assert_eq!(
        board.resolve_queue_id("index:1"),
        "Q2",
        "index:1 should resolve to the second queue"
    );
    assert_eq!(
        board.resolve_queue_id("index:5"),
        "Q2",
        "index:OOB should resolve to the last queue"
    );
    assert_eq!(
        board.resolve_queue_id("random"),
        "random",
        "Non-index strings should resolve as-is"
    );

    Ok(())
}

#[test]
fn test_extract_references() {
    let t = Ticket {
        id: "t1".to_string(),
        title: "T".to_string(),
        created_at: "now".to_string(),
        updated_at: "now".to_string(),
        description: "Check #abc123 and #def456. Also #123 is too short, and #abcdef78 is too long but should extract #abcdef. And #abc123 again.".to_string(),
    };
    let refs = t.extract_references();
    assert_eq!(
        refs.len(),
        3,
        "Should extract exactly 3 unique valid references. Check extract_references logic."
    );
    assert!(
        refs.contains(&"#abc123".to_string()),
        "Should contain #abc123"
    );
    assert!(
        refs.contains(&"#def456".to_string()),
        "Should contain #def456"
    );
    assert!(
        refs.contains(&"#abcdef".to_string()),
        "Should contain #abcdef (first 6 chars after #)."
    );
}

#[test]
fn test_find_ticket_by_id() {
    let board = Board {
        tickets_path: PathBuf::new(),
        queues_path: PathBuf::new(),
        config: Config::default(),
        queues: vec![
            Queue {
                id: "Q1".to_string(),
                name: "Q1".to_string(),
                tickets: vec![Ticket {
                    id: "T1".to_string(),
                    title: "T1".to_string(),
                    created_at: "".to_string(),
                    updated_at: "".to_string(),
                    description: "".to_string(),
                }],
                limit: None,
                visible: true,
            },
            Queue {
                id: "Q2".to_string(),
                name: "Q2".to_string(),
                tickets: vec![Ticket {
                    id: "T2".to_string(),
                    title: "T2".to_string(),
                    created_at: "".to_string(),
                    updated_at: "".to_string(),
                    description: "".to_string(),
                }],
                limit: None,
                visible: true,
            },
        ],
    };

    assert!(
        board.find_ticket_by_id("T1").is_some(),
        "Ticket T1 should be found in Q1. Ensure find_ticket_by_id iterates over all queues."
    );
    assert!(
        board.find_ticket_by_id("T2").is_some(),
        "Ticket T2 should be found in Q2. Ensure find_ticket_by_id iterates over all queues."
    );
    assert!(
        board.find_ticket_by_id("T3").is_none(),
        "Non-existent ticket T3 should not be found."
    );
}

#[test]
fn test_search_history() {
    let mut config = Config::default();

    // 1. Add some items
    config.add_search_to_history("rust".to_string());
    config.add_search_to_history("slint".to_string());
    assert_eq!(config.search_history, vec!["slint", "rust"]);

    // 2. Add duplicate - should move to top
    config.add_search_to_history("rust".to_string());
    assert_eq!(config.search_history, vec!["rust", "slint"]);

    // 3. Limit to 10 items
    for i in 0..15 {
        config.add_search_to_history(format!("search {}", i));
    }
    assert_eq!(config.search_history.len(), 10);
    assert_eq!(config.search_history[0], "search 14");

    // 4. Ignore empty
    config.add_search_to_history("".to_string());
    assert_eq!(config.search_history.len(), 10);

    // 5. Remove item
    config.remove_search_from_history("search 14");
    assert_eq!(config.search_history.len(), 9);
    assert!(!config.search_history.contains(&"search 14".to_string()));
}
