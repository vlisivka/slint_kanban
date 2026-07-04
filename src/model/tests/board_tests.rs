//! src/model/tests/board_tests.rs
//!
//! Purpose: Unit tests for Board orchestration logic, including scanning, moving, creating, and deleting tickets.

use super::super::*;
use crate::model::queue::Queue;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use tempfile::tempdir;

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
        "The ticket ID in the target queue should match"
    );

    Ok(())
}

#[test]
fn test_delete_ticket() -> anyhow::Result<()> {
    let root = tempdir()?;
    let root_dir = root.path().to_path_buf();
    let tickets_dir = root_dir.join("Tickets");
    let queues_dir = root_dir.join("Queue");

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
        "The ticket folder in 'Tickets/' should be removed"
    );
    assert!(
        !q1_path.join("T1_link").exists(),
        "The ticket symlink in the queue folder should be removed"
    );

    let board_after = Board::load(root_dir)?;
    assert_eq!(
        board_after.queues[0].tickets.len(),
        0,
        "The board model should reflect the deletion"
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
    let tid = board.create_ticket("My New Task", "My Description", "Q1", "", "me", 0)?;

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
        "README.md should be created"
    );
    assert!(
        q1_path.join(&tid).exists(),
        "A symlink to the new ticket should be created in the target queue folder"
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
    assert_eq!(
        board2.queues[0].tickets[0].points, 0,
        "Ticket points should match input"
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
    let tid = board.create_ticket("Original", "Original Description", "Q1", "", "me", 5)?;

    board.update_ticket(&tid, "Updated Title", "Updated Description", "", 3)?;

    let board2 = Board::load(root_path)?;
    let t = &board2.queues[0].tickets[0];
    assert_eq!(t.title, "Updated Title", "Updated title should match input");
    assert_eq!(
        t.description, "Updated Description",
        "Updated description should match input"
    );
    assert_eq!(t.points, 3, "Updated points should match input");

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
    assert_eq!(board.queues[0].id, "1.Incoming");
    assert_eq!(board.queues[1].id, "2.ToDo");

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
        !root_path2.join("Queue/1.Incoming").exists(),
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
    board.config.set_limit("2.ToDo".to_string(), 1);
    board.config.write(&root_path)?;

    let board = Board::load(root_path)?;

    // Create first ticket - should succeed
    board.create_ticket("Task 1", "Desc 1", "2.ToDo", "", "me", 0)?;

    // Create second ticket - should fail
    let result = board.create_ticket("Task 2", "Desc 2", "2.ToDo", "", "me", 0);
    assert!(
        result.is_err(),
        "Should return an error if the queue has reached its limit"
    );
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("has reached its limit"),
        "Error message should mention limit"
    );

    Ok(())
}

#[test]
fn test_queue_limit_moving() -> anyhow::Result<()> {
    let root = tempdir()?;
    let root_path = root.path().to_path_buf();

    Board::ensure_initialized(&root_path)?;
    let mut board = Board::load(root_path.clone())?;
    board.config.set_limit("3.Doing".to_string(), 1);
    board.config.write(&root_path)?;

    let board = Board::load(root_path)?;

    // Create two tickets in To Dooo
    let tid1 = board.create_ticket("Task 1", "Desc 1", "2.ToDo", "", "me", 0)?;
    let tid2 = board.create_ticket("Task 2", "Desc 2", "2.ToDo", "", "me", 0)?;

    // Move first ticket to Doing - should succeed
    board.move_ticket(&tid1, "2.ToDo", "3.Doing")?;

    // Move second ticket to Doing - should fail
    let result = board.move_ticket(&tid2, "2.ToDo", "3.Doing");
    assert!(
        result.is_err(),
        "Should return an error if the target queue has reached its limit"
    );
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("has reached its limit"),
        "Error message should mention limit"
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
        ticket_index: std::collections::HashMap::new(),
    };

    assert_eq!(board.resolve_queue_id("Q1"), "Q1");
    assert_eq!(board.resolve_queue_id("index:0"), "Q1");
    assert_eq!(board.resolve_queue_id("index:1.5"), "Q2");
    assert_eq!(board.resolve_queue_id("index:5"), "Q2");
    assert_eq!(board.resolve_queue_id("random"), "random");

    // Test with hidden queues
    let board_hidden = Board {
        tickets_path: PathBuf::new(),
        queues_path: PathBuf::new(),
        config: Config::default(),
        queues: vec![
            Queue {
                id: "Q1".to_string(),
                name: "Q1".to_string(),
                tickets: vec![],
                limit: None,
                visible: true,
            },
            Queue {
                id: "B".to_string(),
                name: "B".to_string(),
                tickets: vec![],
                limit: None,
                visible: false, // HIDDEN
            },
            Queue {
                id: "Q2".to_string(),
                name: "Q2".to_string(),
                tickets: vec![],
                limit: None,
                visible: true,
            },
        ],
        ticket_index: std::collections::HashMap::new(),
    };

    assert_eq!(board_hidden.resolve_queue_id("index:0"), "Q1");
    assert_eq!(board_hidden.resolve_queue_id("index:1"), "Q2");

    Ok(())
}

#[test]
fn test_find_ticket_by_id() {
    let tickets = vec![
        Ticket {
            id: "T1".to_string(),
            title: "T1".to_string(),
            created_at: "".to_string(),
            updated_at: "".to_string(),
            description: "".to_string(),
            assigned_to: "".to_string(),
            author: "".to_string(),
            points: 0,
            attachment_count: 0,
            comments: vec![],
        },
        Ticket {
            id: "T2".to_string(),
            title: "T2".to_string(),
            created_at: "".to_string(),
            updated_at: "".to_string(),
            description: "".to_string(),
            assigned_to: "".to_string(),
            author: "".to_string(),
            points: 0,
            attachment_count: 0,
            comments: vec![],
        },
    ];

    let mut ticket_index = std::collections::HashMap::new();
    for t in &tickets {
        ticket_index.insert(t.id.clone(), t.clone());
    }

    let board = Board {
        tickets_path: PathBuf::new(),
        queues_path: PathBuf::new(),
        config: Config::default(),
        queues: vec![
            Queue {
                id: "Q1".to_string(),
                name: "Q1".to_string(),
                tickets: vec![tickets[0].clone()],
                limit: None,
                visible: true,
            },
            Queue {
                id: "Q2".to_string(),
                name: "Q2".to_string(),
                tickets: vec![tickets[1].clone()],
                limit: None,
                visible: true,
            },
        ],
        ticket_index,
    };

    assert!(board.find_ticket_by_id("T1").is_some());
    assert!(board.find_ticket_by_id("T2").is_some());
    assert!(board.find_ticket_by_id("T3").is_none());
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
        ticket_index: std::collections::HashMap::new(),
    };
    let result = board.load_ticket(&ticket_path);
    assert!(result.is_err());
    Ok(())
}

#[test]
fn test_load_ticket_invalid_format() -> anyhow::Result<()> {
    let root = tempdir()?;
    let ticket_path = root.path().join("T1");
    std::fs::create_dir(&ticket_path)?;
    std::fs::write(ticket_path.join("README.md"), "No separators here")?;

    let board = Board {
        tickets_path: root.path().join("Tickets"),
        queues_path: root.path().join("Queue"),
        queues: vec![],
        config: Config::default(),
        ticket_index: std::collections::HashMap::new(),
    };
    let result = board.load_ticket(&ticket_path);
    assert!(result.is_err());
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
        ticket_index: std::collections::HashMap::new(),
    };
    let result = board.load_ticket(&ticket_path);
    assert!(result.is_err());
    Ok(())
}

#[test]
fn test_delete_non_existent_ticket() -> anyhow::Result<()> {
    let root = tempdir()?;
    Board::ensure_initialized(root.path())?;
    let board = Board::load(root.path().to_path_buf())?;

    let result = board.delete_ticket("nonexistent");
    assert!(result.is_err(), "Deleting non-existent ticket should fail");
    Ok(())
}

#[test]
fn test_move_ticket_invalid_queue() -> anyhow::Result<()> {
    let root = tempdir()?;
    Board::ensure_initialized(root.path())?;
    let board = Board::load(root.path().to_path_buf())?;
    let tid = board.create_ticket("T1", "D", "1.Incoming", "", "me", 0)?;

    let result = board.move_ticket(&tid, "invalid_src", "2.ToDo");
    assert!(result.is_err(), "Moving from invalid source should fail");

    let result = board.move_ticket(&tid, "1.Incoming", "invalid_target");
    assert!(result.is_err(), "Moving to invalid target should fail");

    Ok(())
}

#[test]
fn test_activity_logging() -> anyhow::Result<()> {
    let board_dir = tempdir()?;
    let root_path = board_dir.path().to_path_buf();

    let user_dir = tempdir()?;
    Config::set_test_user_config_path(Some(user_dir.path().join("user.toml")));

    Board::ensure_initialized(&root_path)?;
    let board = Board::load(root_path.clone())?;

    // Create ticket should produce a log entry
    let tid = board.create_ticket("Log Task", "Log Desc", "2.ToDo", "user1", "author1", 8)?;

    // Move ticket should produce a log entry
    board.move_ticket(&tid, "2.ToDo", "3.Doing")?;

    // Check logs
    let machine_id = board.config.machine_id().unwrap();
    let log_file_name = format!("log_{}_{}.md", board.config.active_user(), machine_id);
    let log_path = root_path.join("logs").join(log_file_name);

    assert!(log_path.exists(), "Log file should be created");

    let log_content = std::fs::read_to_string(&log_path)?;

    assert!(
        log_content.contains("# User Activity Log:"),
        "Should contain header"
    );
    assert!(
        log_content.contains("| **Date** | **Action** | **Action description** | **JSON** |"),
        "Should contain table header"
    );
    assert!(
        log_content.contains("CREATE_TICKET"),
        "Should contain CREATE_TICKET action"
    );
    assert!(
        log_content.contains("CHANGE_STATUS"),
        "Should contain CHANGE_STATUS action"
    );
    assert!(
        log_content.contains("Log Task"),
        "Should contain ticket title"
    );

    Ok(())
}
/// Tests for .keepme file creation during initialization and queue operations.
///
/// Criteria of success: ensure_initialized creates .keepme in root dirs (Queue, Tickets, logs)
/// Criteria of failure: no .keepme files created after initialization
#[test]
fn test_ensure_initialized_creates_keepme() -> anyhow::Result<()> {
    let root = tempdir()?;
    let root_path = root.path().to_path_buf();

    Board::ensure_initialized(&root_path)?;

    // Check .keepme files in root directories
    assert!(
        root_path.join("Queue/.keepme").exists(),
        "Queue should have .keepme file after initialization"
    );
    assert!(
        root_path.join("Tickets/.keepme").exists(),
        "Tickets should have .keepme file after initialization"
    );
    assert!(
        root_path.join("logs/.keepme").exists(),
        "logs should have .keepme file after initialization"
    );

    // Check .keepme files in each default queue directory
    for q_id in &[
        "1.Incoming",
        "2.ToDo",
        "3.Doing",
        "4.Reviewing",
        "5.Testing",
        "6.Done",
        "7.Archive",
    ] {
        assert!(
            root_path.join(format!("Queue/{q_id}/.keepme")).exists(),
            "Default queue {q_id} should have .keepme file after initialization"
        );
    }

    Ok(())
}
/// Tests that add_queue creates a .keepme file in the new queue directory.
/// Tests that add_queue creates a .keepme file in the new queue directory.
///
/// Criteria of success: new queue has .keepme inside it
#[test]
fn test_add_queue_creates_keepme() -> anyhow::Result<()> {
    let root = tempdir()?;
    let root_path = root.path().to_path_buf();

    Board::ensure_initialized(&root_path)?;
    let board = Board::load(root_path.clone())?;

    board.add_queue("8.Custom")?;

    assert!(
        root_path.join("Queue/8.Custom/.keepme").exists(),
        "New queue should have .keepme file"
    );

    Ok(())
}

/// Tests that delete_queue considers a directory with only dotfiles as empty.
///
/// Criteria of success: delete_queue succeeds when only dotfiles remain
/// Criteria of failure: delete_queue fails on non-dotfile entries in the queue
#[test]
fn test_delete_queue_with_only_dotfiles() -> anyhow::Result<()> {
    let root = tempdir()?;
    let root_path = root.path().to_path_buf();

    // Create minimal structure with a default queue so Board::load works
    std::fs::create_dir_all(root_path.join("Queue"))?;
    std::fs::create_dir_all(root_path.join("Tickets"))?;
    std::fs::create_dir_all(root_path.join("logs"))?;
    std::fs::create_dir_all(root_path.join("Queue").join("1.Incoming"))?;
    std::fs::write(root_path.join("Queue/.keepme"), "")?;
    std::fs::write(root_path.join("Tickets/.keepme"), "")?;
    std::fs::write(root_path.join("logs/.keepme"), "")?;
    let board = Board::load(root_path.clone())?;

    // Create a queue dir with only dotfiles
    let empty_queue_path = root_path.join("Queue").join("9.Empty");
    std::fs::create_dir_all(&empty_queue_path)?;
    std::fs::write(empty_queue_path.join(".keepme"), "")?;
    std::fs::write(empty_queue_path.join(".gitignore"), "*")?;

    board.delete_queue("9.Empty")?;

    assert!(
        !empty_queue_path.exists(),
        "Queue directory should be deleted"
    );

    Ok(())
}

/// Tests that delete_queue fails when queue has non-dotfile entries.
///
/// Criteria of success: delete_queue returns error for non-empty queue
#[test]
fn test_delete_queue_with_non_dotfiles_fails() -> anyhow::Result<()> {
    let root = tempdir()?;
    let root_path = root.path().to_path_buf();

    std::fs::create_dir_all(root_path.join("Queue"))?;
    std::fs::create_dir_all(root_path.join("Tickets"))?;
    std::fs::write(root_path.join("Queue/.keepme"), "")?;
    std::fs::write(root_path.join("Tickets/.keepme"), "")?;
    Config::default().write(&root_path)?;

    let board = Board::load(root_path.clone())?;

    // Create a queue with a regular file
    let non_empty_queue_path = root_path.join("Queue").join("9.NonEmpty");
    std::fs::create_dir_all(&non_empty_queue_path)?;
    std::fs::write(non_empty_queue_path.join(".keepme"), "")?;
    std::fs::write(non_empty_queue_path.join("some_file.txt"), "data")?;

    let result = board.delete_queue("9.NonEmpty");
    assert!(result.is_err(), "Should fail to delete non-empty queue");

    Ok(())
}

/// Tests that scanning queues ignores dotfile directories.
///
/// Criteria of success: dotfile queue names are not loaded as queues
/// Criteria of failure: .hidden_queue appears in loaded queue list
#[test]
fn test_scan_queues_ignores_dotfile_dirs() -> anyhow::Result<()> {
    let root = tempdir()?;
    let root_path = root.path().to_path_buf();

    std::fs::create_dir_all(root_path.join("Queue"))?;
    std::fs::create_dir_all(root_path.join("Tickets"))?;
    Config::default().write(&root_path)?;

    // Create a real queue and a dotfile "queue"
    std::fs::create_dir_all(root_path.join("Queue/1.Incoming"))?;
    std::fs::create_dir_all(root_path.join("Queue/.hidden_queue"))?;

    let board = Board::load(root_path.clone())?;

    let queue_ids: Vec<&str> = board.queues.iter().map(|q| q.id.as_str()).collect();
    assert!(
        queue_ids.contains(&"1.Incoming"),
        "Real queue should be loaded"
    );
    assert!(
        !queue_ids.contains(&".hidden_queue"),
        "Dotfile queue should NOT be loaded"
    );

    Ok(())
}

/// Tests that scanning ticket directories ignores dotfile entries.
///
/// Criteria of success: .dotfile entries in queues are not loaded as tickets
#[test]
fn test_scan_tickets_ignores_dotfile_entries() -> anyhow::Result<()> {
    let root = tempdir()?;
    let root_path = root.path().to_path_buf();

    Board::ensure_initialized(&root_path)?;
    let board = Board::load(root_path.clone())?;
    let tid = board.create_ticket("Test Ticket", "desc", "1.Incoming", "", "me", 0)?;

    // Create a dotfile entry in the queue (simulating a hidden file)
    let queue_path = root_path.join("Queue/1.Incoming");
    std::fs::write(queue_path.join(".hidden_file"), "data")?;

    // Reload the board to pick up changes
    let board2 = Board::load(root_path)?;

    let incoming_queue = board2.queues.iter().find(|q| q.id == "1.Incoming").unwrap();
    let ticket_ids: Vec<&str> = incoming_queue
        .tickets
        .iter()
        .map(|t| t.id.as_str())
        .collect();

    assert!(
        ticket_ids.contains(&tid.as_str()),
        "Real ticket should be loaded"
    );
    assert!(
        !ticket_ids.contains(&".hidden_file"),
        "Dotfile should NOT be loaded as ticket"
    );

    Ok(())
}
