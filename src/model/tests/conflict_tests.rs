use crate::model::board::Board;
use crate::model::ticket::Ticket;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_conflict_resolution_duplicates() {
    let root = tempdir().unwrap();
    let root_path = root.path();
    Board::ensure_initialized(root_path).unwrap();

    let tickets_dir = root_path.join("Tickets");
    let t1_dir = tickets_dir.join("abc123");
    fs::create_dir_all(&t1_dir).unwrap();
    let t1 = Ticket {
        id: "abc123".to_string(),
        title: "Test 1".to_string(),
        created_at: "2026-01-01 00:00:00".to_string(),
        updated_at: "2026-01-01 00:00:00".to_string(),
        description: "Desc".to_string(),
        assigned_to: "".to_string(),
        author: "user".to_string(),
        points: 0,
        attachment_count: 0,
        comments: vec![],
    };
    t1.save(&t1_dir).unwrap();

    // Place symlink in two queues
    let q1_path = root_path.join("Queue").join("1.Incoming");
    let q2_path = root_path.join("Queue").join("2.ToDo");

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&t1_dir, q1_path.join("abc123")).unwrap();
        std::os::unix::fs::symlink(&t1_dir, q2_path.join("abc123")).unwrap();
    }

    // Load board - should resolve conflict
    let _board = Board::load(root_path.to_path_buf()).unwrap();

    // Should be removed from q1 (earlier) and kept in q2 (later)
    assert!(!q1_path.join("abc123").exists());
    assert!(q2_path.join("abc123").exists());
}

#[test]
fn test_conflict_resolution_orphans() {
    let root = tempdir().unwrap();
    let root_path = root.path();
    Board::ensure_initialized(root_path).unwrap();

    let tickets_dir = root_path.join("Tickets");
    let t1_dir = tickets_dir.join("orphan");
    fs::create_dir_all(&t1_dir).unwrap();
    let t1 = Ticket {
        id: "orphan".to_string(),
        title: "Orphan".to_string(),
        created_at: "2026-01-01 00:00:00".to_string(),
        updated_at: "2026-01-01 00:00:00".to_string(),
        description: "Desc".to_string(),
        assigned_to: "".to_string(),
        author: "user".to_string(),
        points: 0,
        attachment_count: 0,
        comments: vec![],
    };
    t1.save(&t1_dir).unwrap();

    // Load board - should find orphan and add to first queue
    let _board = Board::load(root_path.to_path_buf()).unwrap();

    let first_q_path = root_path.join("Queue").join("1.Incoming");
    assert!(first_q_path.join("orphan").exists());
}

#[test]
fn test_conflict_resolution_broken_links() {
    let root = tempdir().unwrap();
    let root_path = root.path();
    Board::ensure_initialized(root_path).unwrap();

    let q1_path = root_path.join("Queue").join("1.Incoming");
    let broken_link = q1_path.join("broken");

    // Create a symlink to non-existent path
    #[cfg(unix)]
    std::os::unix::fs::symlink(root_path.join("Tickets").join("nonexistent"), &broken_link)
        .unwrap();

    // Load board - should remove broken link
    let _board = Board::load(root_path.to_path_buf()).unwrap();

    assert!(!broken_link.exists() && !broken_link.is_symlink());
}

#[test]
fn test_can_manage_ticket_logic() {
    let root = tempdir().unwrap();
    let root_path = root.path();
    Board::ensure_initialized(root_path).unwrap();

    let mut board = Board::load(root_path.to_path_buf()).unwrap();
    board.config.user.manage_only_mine = true;
    board.config.user.active_user = "Alice".to_string();

    let t_mine = Ticket {
        id: "mine".to_string(),
        assigned_to: "Alice".to_string(),
        ..Default::default()
    };
    let t_others = Ticket {
        id: "others".to_string(),
        assigned_to: "Bob".to_string(),
        ..Default::default()
    };

    // Admin can manage everything
    assert!(board.can_manage_ticket(&t_others, true));

    // Manage only mine is ON
    assert!(board.can_manage_ticket(&t_mine, false));
    assert!(!board.can_manage_ticket(&t_others, false));

    // Admin user name can manage everything
    board.config.user.active_user = "admin".to_string();
    assert!(board.can_manage_ticket(&t_others, false));

    // Unassigned active user can manage everything
    board.config.user.active_user = "<unassigned>".to_string();
    assert!(board.can_manage_ticket(&t_others, false));

    // Manage only mine is OFF
    board.config.user.manage_only_mine = false;
    board.config.user.active_user = "Alice".to_string();
    assert!(board.can_manage_ticket(&t_others, false));
}

impl Default for Ticket {
    fn default() -> Self {
        Self {
            id: "".to_string(),
            title: "".to_string(),
            created_at: "".to_string(),
            updated_at: "".to_string(),
            description: "".to_string(),
            assigned_to: "".to_string(),
            author: "".to_string(),
            points: 0,
            attachment_count: 0,
            comments: vec![],
        }
    }
}
