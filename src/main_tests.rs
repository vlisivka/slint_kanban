//! main_tests.rs
//!
//! Purpose: Integration tests for the application, testing UI interactions and CLI flows.

use super::*;
use clap::Parser;
use tempfile::{tempdir, TempDir};

struct TestEnv {
    _dir: TempDir,
    pub root: std::path::PathBuf,
}

impl TestEnv {
    fn new() -> anyhow::Result<Self> {
        let dir = tempdir()?;
        let root = dir.path().to_path_buf();
        Ok(Self { _dir: dir, root })
    }

    fn run(&self, args: &[&str]) -> anyhow::Result<String> {
        let mut full_args = vec!["kanban", "--root", self.root.to_str().unwrap()];
        full_args.extend_from_slice(args);

        let cli_args = CliArgs::try_parse_from(full_args)?;
        let mut out = Vec::new();
        run_cli(cli_args, &mut out)?;
        Ok(String::from_utf8(out).unwrap_or_default())
    }

    fn run_err(&self, args: &[&str]) -> String {
        let mut full_args = vec!["kanban", "--root", self.root.to_str().unwrap()];
        full_args.extend_from_slice(args);

        match CliArgs::try_parse_from(full_args) {
            Ok(cli_args) => {
                let mut out = Vec::new();
                let res = run_cli(cli_args, &mut out);
                assert!(res.is_err(), "Expected error but got success");
                res.unwrap_err().to_string()
            }
            Err(e) => e.to_string(),
        }
    }

    fn board(&self) -> anyhow::Result<Board> {
        Board::load(self.root.clone())
    }
}

#[test]
fn test_cli_add() -> anyhow::Result<()> {
    let env = TestEnv::new()?;

    // 1. Add ticket
    env.run(&[
        "add",
        "-t",
        "Test Ticket",
        "-d",
        "Test Description",
        "-q",
        "1. Incoming",
    ])?;

    let board = env.board()?;
    let incoming = board.queues.iter().find(|q| q.id == "1. Incoming").unwrap();
    assert_eq!(
        incoming.tickets.len(),
        1,
        "Ticket should be added to Incoming queue"
    );
    assert_eq!(incoming.tickets[0].title, "Test Ticket");

    Ok(())
}

#[test]
fn test_cli_update() -> anyhow::Result<()> {
    let env = TestEnv::new()?;

    env.run(&[
        "add",
        "-t",
        "Old Title",
        "-d",
        "Old Desc",
        "-q",
        "1. Incoming",
    ])?;

    let id = env.board()?.queues[0].tickets[0].id.clone();

    env.run(&["update", "-i", &id, "-t", "New Title"])?;

    let board = env.board()?;
    let ticket = board.find_ticket_by_id(&id).unwrap();
    assert_eq!(ticket.title, "New Title");
    assert_eq!(ticket.description, "Old Desc");

    Ok(())
}

#[test]
fn test_cli_move() -> anyhow::Result<()> {
    let env = TestEnv::new()?;

    env.run(&["add", "-t", "Move Me", "-q", "1. Incoming"])?;
    let id = env.board()?.queues[0].tickets[0].id.clone();

    // Valid move
    env.run(&["move", "-i", &id, "-q", "2. To Do"])?;
    let board = env.board()?;
    assert!(board
        .queues
        .iter()
        .find(|q| q.id == "2. To Do")
        .unwrap()
        .tickets
        .iter()
        .any(|t| t.id == id));

    // Invalid move
    env.run_err(&["move", "-i", &id, "-q", "NonExistent"]);

    Ok(())
}

#[test]
fn test_cli_remove() -> anyhow::Result<()> {
    let env = TestEnv::new()?;

    env.run(&["add", "-t", "Delete Me", "-q", "1. Incoming"])?;
    let id = env.board()?.queues[0].tickets[0].id.clone();

    // Valid remove
    env.run(&["remove", "-i", &id])?;
    assert!(env.board()?.find_ticket_by_id(&id).is_none());

    // Invalid remove
    env.run_err(&["remove", "-i", "invalid_id"]);

    Ok(())
}

#[test]
fn test_cli_change_limit() -> anyhow::Result<()> {
    let env = TestEnv::new()?;
    Board::ensure_initialized(&env.root)?;

    // change limit using model logic explicitly
    let mut board = env.board()?;
    board.config.set_limit("1. Incoming".to_string(), 10);
    board.config.write(&env.root)?;

    assert_eq!(env.board()?.config.get_limit("1. Incoming"), Some(10));
    Ok(())
}

#[test]
fn test_cli_comment() -> anyhow::Result<()> {
    let env = TestEnv::new()?;

    env.run(&["add", "-t", "Test Comment Ticket", "-q", "1. Incoming"])?;
    let id = env.board()?.queues[0].tickets[0].id.clone();

    env.run(&["comment", "-i", &id, "-c", "CLI created comment"])?;

    let board = env.board()?;
    let ticket = board.load_full_ticket(&id).unwrap();
    assert_eq!(ticket.comments.len(), 1);
    assert_eq!(ticket.comments[0].content, "CLI created comment");

    Ok(())
}

#[test]
fn test_cli_attach() -> anyhow::Result<()> {
    let env = TestEnv::new()?;

    env.run(&["add", "-t", "Test Attach Ticket", "-q", "1. Incoming"])?;
    let id = env.board()?.queues[0].tickets[0].id.clone();

    let dummy_file_path = env.root.join("dummy.txt");
    std::fs::write(&dummy_file_path, "dummy content")?;

    env.run(&["attach", "-i", &id, "-f", dummy_file_path.to_str().unwrap()])?;

    let board = env.board()?;
    let attached_file = board.ticket_path(&id).join("attachment").join("dummy.txt");
    assert!(attached_file.exists());
    assert_eq!(std::fs::read_to_string(&attached_file)?, "dummy content");

    Ok(())
}

#[test]
fn test_cli_configure() -> anyhow::Result<()> {
    let env = TestEnv::new()?;
    let user_dir = tempfile::tempdir().unwrap();
    slint_kanban::model::Config::set_test_user_config_path(Some(user_dir.path().join("user.toml")));

    Board::ensure_initialized(&env.root)?;

    env.run(&["configure", "--add-user", "newusr"])?;
    assert!(env
        .board()?
        .config
        .kanban
        .users
        .contains(&"newusr".to_string()));

    env.run(&[
        "configure",
        "--active-user",
        "newusr",
        "--show-only-mine",
        "true",
    ])?;
    let board = env.board()?;
    assert_eq!(board.config.user.active_user, "newusr");
    assert_eq!(board.config.user.show_only_mine, true);

    Ok(())
}

#[test]
fn test_cli_list() -> anyhow::Result<()> {
    let env = TestEnv::new()?;

    env.run(&[
        "add",
        "-t",
        "Task A",
        "-d",
        "Desc A",
        "-q",
        "1. Incoming",
        "--assign-to",
        "Alice",
    ])?;
    env.run(&["add", "-t", "Task B", "-d", "KeywordX", "-q", "2. To Do"])?;

    // List all
    let out = env.run(&["list"])?;
    assert!(out.contains("Task A"));
    assert!(out.contains("Task B"));

    // List assigned to Alice
    let out = env.run(&["list", "--assigned-to-user", "Alice"])?;
    assert!(out.contains("Task A"));
    assert!(!out.contains("Task B"));

    // List unassigned
    let out = env.run(&["list", "--unassigned"])?;
    assert!(!out.contains("Task A"));
    assert!(out.contains("Task B"));

    // Search KeywordX
    let out = env.run(&["list", "--search", "KeywordX"])?;
    assert!(!out.contains("Task A"));
    assert!(out.contains("Task B"));

    Ok(())
}

#[test]
fn test_cli_show() -> anyhow::Result<()> {
    let env = TestEnv::new()?;

    env.run(&[
        "add",
        "-t",
        "Task Show",
        "-d",
        "Show Description",
        "-q",
        "1. Incoming",
        "--assign-to",
        "Bob",
    ])?;
    let id = env.board()?.queues[0].tickets[0].id.clone();

    // Valid show
    let out = env.run(&["show", "-i", &id])?;
    assert!(out.contains(&id));
    assert!(out.contains("Task Show"));
    assert!(out.contains("Show Description"));
    assert!(out.contains("Bob"));
    assert!(out.contains("Attachments: 0"));

    // Invalid show
    env.run_err(&["show", "-i", "invalid_id"]);

    Ok(())
}

#[test]
fn test_cli_attach_extended() -> anyhow::Result<()> {
    let env = TestEnv::new()?;

    env.run(&["add", "-t", "Attach Test", "-q", "1. Incoming"])?;
    let id = env.board()?.queues[0].tickets[0].id.clone();

    // 1. Test --show (path)
    let out = env.run(&["attach", "-i", &id, "--show"])?;
    assert!(out.contains("attachment"));
    assert!(out.contains(&id));

    // 2. Test --list (empty)
    let out = env.run(&["attach", "-i", &id, "--list"])?;
    assert!(out.contains("No attachments found."));

    // 3. Attach a file
    let dummy = env.root.join("test.txt");
    std::fs::write(&dummy, "hello")?;
    env.run(&["attach", "-i", &id, "-f", dummy.to_str().unwrap()])?;

    // 4. Test --list (one file)
    let out = env.run(&["attach", "-i", &id, "--list"])?;
    assert!(out.contains("test.txt"));

    // 5. Test show command includes count
    let out = env.run(&["show", "-i", &id])?;
    assert!(out.contains("Attachments: 1"));

    // 6. Test --open (check message)
    let out = env.run(&["attach", "-i", &id, "--open"])?;
    assert!(out.contains("Opening attachments directory"));

    Ok(())
}

#[test]
fn test_cli_stats() -> anyhow::Result<()> {
    let env = TestEnv::new()?;
    Board::ensure_initialized(&env.root)?;

    env.run(&[
        "add",
        "-t",
        "Task 1",
        "-q",
        "1. Incoming",
        "--assign-to",
        "Alice",
    ])?;
    env.run(&[
        "add",
        "-t",
        "Task 2",
        "-q",
        "2. To Do",
        "--assign-to",
        "Bob",
    ])?;
    env.run(&["add", "-t", "Task 3", "-q", "1. Incoming"])?;

    let out = env.run(&["stats"])?;
    assert!(out.contains("== Board Summary =="));
    assert!(out.contains("Total tickets: 3"));
    assert!(out.contains("Unassigned:    1"));

    assert!(out.contains("== Tickets per Queue =="));
    assert!(out.contains("1. Incoming              2      -     -"));
    assert!(out.contains("2. To Do                 1     21    4%"));

    assert!(out.contains("== Tickets per User =="));
    assert!(out.contains("Alice                    1"));
    assert!(out.contains("Bob                      1"));

    // Test filtering by user
    let out = env.run(&["stats", "--user", "Alice"])?;
    assert!(out.contains("Alice                    1"));
    assert!(!out.contains("Bob"));

    Ok(())
}

#[test]
fn test_cli_stats_csv() -> anyhow::Result<()> {
    let env = TestEnv::new()?;
    Board::ensure_initialized(&env.root)?;

    env.run(&[
        "add",
        "-t",
        "Task 1",
        "-q",
        "1. Incoming",
        "--assign-to",
        "Alice",
    ])?;

    let out = env.run(&["stats", "--csv"])?;
    assert!(out.contains("Type,Category/Date,Metric,Value,Unit"));
    assert!(out.contains("Summary,General,Total Tickets,1,count"));
    assert!(out.contains("Queue,1. Incoming,Count,1,tickets"));
    assert!(out.contains("User,Alice,Count,1,tickets"));

    Ok(())
}

#[test]
fn test_cli_sprint_crud() -> anyhow::Result<()> {
    let env = TestEnv::new()?;
    Board::ensure_initialized(&env.root)?;

    env.run(&[
        "sprint",
        "add",
        "--number",
        "1",
        "--name",
        "Sprint 1",
        "--start",
        "2026-02-01",
        "--end",
        "2026-02-14",
    ])?;

    let out = env.run(&["sprint", "list"])?;
    assert!(out.contains("Sprint 1"));

    env.run(&["sprint", "update", "--number", "1", "--name", "Sprint 2"])?;
    let out = env.run(&["sprint", "list"])?;
    assert!(out.contains("Sprint 2"));

    env.run(&["sprint", "remove", "--number", "1"])?;
    let out = env.run(&["sprint", "list"])?;
    assert!(out.contains("No sprints found."));

    // Test automatic numbering
    env.run(&[
        "sprint",
        "add",
        "--name",
        "Auto Sprint",
        "--start",
        "2026-03-01",
        "--end",
        "2026-03-14",
    ])?;
    let out = env.run(&["sprint", "list"])?;
    assert!(out.contains("1      Auto Sprint"));

    Ok(())
}
