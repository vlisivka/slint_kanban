use crate::model::stats::get_board_summary;
use crate::model::{Board, Ticket};
use std::collections::HashMap;

#[test]
fn test_get_board_summary() {
    let mut board = Board {
        config: Default::default(),
        queues: vec![],
        tickets_path: "".into(),
        queues_path: "".into(),
        ticket_index: HashMap::new(),
    };

    // Add some users to config to ensure they show up in stats
    board.config.kanban.users = vec![
        "<unassigned>".to_string(),
        "alice".to_string(),
        "bob".to_string(),
        "charlie".to_string(),
    ];

    let mut q1 = crate::model::queue::Queue {
        id: "1".into(),
        name: "Incoming".into(),
        tickets: vec![],
        limit: None,
        visible: true,
    };

    let mut q2 = crate::model::queue::Queue {
        id: "2".into(),
        name: "ToDo".into(),
        tickets: vec![],
        limit: Some(5),
        visible: true,
    };

    let mut t1 = Ticket::from_metadata("t1".into(), Default::default(), "".into());
    t1.assigned_to = "alice".into();

    let mut t2 = Ticket::from_metadata("t2".into(), Default::default(), "".into());
    t2.assigned_to = "alice".into();

    let mut t3 = Ticket::from_metadata("t3".into(), Default::default(), "".into());
    t3.assigned_to = "".into(); // unassigned

    let mut t4 = Ticket::from_metadata("t4".into(), Default::default(), "".into());
    t4.assigned_to = "bob".into();

    q1.tickets.push(t1);
    q1.tickets.push(t3); // 2 tickets in q1
    q2.tickets.push(t2);
    q2.tickets.push(t4); // 2 tickets in q2

    board.queues.push(q1);
    board.queues.push(q2);

    let summary = get_board_summary(&board);

    assert_eq!(summary.total_tickets, 4, "Should have 4 total tickets");
    assert_eq!(
        summary.unassigned_tickets, 1,
        "Should have 1 unassigned ticket"
    );

    assert_eq!(summary.queues.len(), 2, "Should have 2 queue stats");
    assert_eq!(
        summary.queues[0].name, "Incoming",
        "First queue name should match"
    );
    assert_eq!(
        summary.queues[0].count, 2,
        "First queue should have 2 tickets"
    );
    assert_eq!(summary.queues[0].limit, None, "Incoming has no limit");

    assert_eq!(
        summary.queues[1].name, "ToDo",
        "Second queue name should match"
    );
    assert_eq!(summary.queues[1].count, 2, "ToDo should have 2 tickets");
    assert_eq!(summary.queues[1].limit, Some(5), "ToDo has limit 5");

    assert_eq!(
        summary.users.len(),
        3,
        "Should have alice, bob, and charlie reported"
    );

    // Convert to map for easy checking
    let mut mapped_users: HashMap<String, usize> = HashMap::new();
    for u in summary.users {
        mapped_users.insert(u.name, u.count);
    }

    assert_eq!(
        mapped_users.get("alice"),
        Some(&2),
        "alice should have 2 tickets"
    );
    assert_eq!(
        mapped_users.get("bob"),
        Some(&1),
        "bob should have 1 ticket"
    );
    assert_eq!(
        mapped_users.get("charlie"),
        Some(&0),
        "charlie should have 0 tickets"
    );

    assert_eq!(summary.completion_rate, Some(0.0), "0/4 tickets are done");

    // Add a done queue and re-check
    let q3 = crate::model::queue::Queue {
        id: "3".into(),
        name: "Done".into(),
        tickets: vec![Ticket::from_metadata(
            "t5".into(),
            Default::default(),
            "".into(),
        )],
        limit: None,
        visible: true,
    };
    board.queues.push(q3);

    let summary = get_board_summary(&board);
    assert_eq!(summary.completion_rate, Some(20.0), "1/5 tickets is done"); // (1) / (5 - 0) * 100 = 20%
}

#[test]
fn test_parse_log_file() -> anyhow::Result<()> {
    use crate::model::stats::parse_log_file;
    use std::fs;
    use tempfile::tempdir;

    let dir = tempdir()?;
    let log_path = dir.path().join("log_alice_123.md");

    let log_content = "# User Activity Log: alice\n\n\
| **Date** | **Action** | **Action description** | **JSON** |\n\
| :--- | :--- | :--- | :--- |\n\
| 2026-02-22T10:00:00Z | CREATE_TICKET | Created Task A | `{\"action\":\"CREATE_TICKET\", \"id\":\"123\", \"title\":\"Task A\", \"queue\":\"Incoming\"}` |\n\
| 2026-02-22T10:05:00Z | CHANGE_STATUS | Moved to ToDo | `{\"action\":\"CHANGE_STATUS\", \"id\":\"123\", \"from\":\"Incoming\", \"to\":\"ToDo\"}` |\n";

    fs::write(&log_path, log_content)?;

    let entries = parse_log_file(&log_path)?;
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].action, "CREATE_TICKET");
    assert_eq!(entries[0].timestamp, "2026-02-22T10:00:00Z");
    assert_eq!(entries[1].action, "CHANGE_STATUS");

    Ok(())
}
