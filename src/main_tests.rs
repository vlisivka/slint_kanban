//! main_tests.rs
//!
//! Purpose: Integration tests for the application, testing UI interactions and CLI flows.

use super::*;
use tempfile::tempdir;

#[test]
fn test_cli_add() -> anyhow::Result<()> {
    let dir = tempdir()?;
    let root = dir.path().to_path_buf();

    let args = CliArgs {
        root: Some(root.clone()),
        command: Some(Commands::Add {
            title: "Test Ticket".to_string(),
            description: "Test Description".to_string(),
            queue: "1. Incoming".to_string(),
            assign_to: "".to_string(),
        }),
    };

    run_main(args)?;

    let board = Board::load(root)?;
    let incoming = board.queues.iter().find(|q| q.id == "1. Incoming").unwrap();
    assert_eq!(
        incoming.tickets.len(),
        1,
        "Ticket should be added to Incoming queue"
    );
    assert_eq!(incoming.tickets[0].title, "Test Ticket", "The added ticket's title should match the CLI input. Verify ticket creation logic in Board::create_ticket.");

    Ok(())
}

#[test]
fn test_cli_update() -> anyhow::Result<()> {
    let dir = tempdir()?;
    let root = dir.path().to_path_buf();

    // 1. Add a ticket
    run_main(CliArgs {
        root: Some(root.clone()),
        command: Some(Commands::Add {
            title: "Old Title".to_string(),
            description: "Old Desc".to_string(),
            queue: "1. Incoming".to_string(),
            assign_to: "".to_string(),
        }),
    })?;

    let board = Board::load(root.clone())?;
    let id = board.queues[0].tickets[0].id.clone();

    // 2. Update it
    run_main(CliArgs {
        root: Some(root.clone()),
        command: Some(Commands::Update {
            id: id.clone(),
            title: Some("New Title".to_string()),
            description: None,
            assign_to: None,
            unassign: false,
        }),
    })?;

    let board = Board::load(root)?;
    let ticket = board.find_ticket_by_id(&id).unwrap();
    assert_eq!(ticket.title, "New Title", "The ticket title should be updated after CLI 'update' command. Check handle_command for Commands::Update.");
    assert_eq!(ticket.description, "Old Desc", "The ticket description should remain unchanged if not specified in CLI 'update' command. Check update_ticket logic in model.rs.");

    Ok(())
}

#[test]
fn test_cli_move() -> anyhow::Result<()> {
    let dir = tempdir()?;
    let root = dir.path().to_path_buf();

    // 1. Add a ticket
    run_main(CliArgs {
        root: Some(root.clone()),
        command: Some(Commands::Add {
            title: "Move Me".to_string(),
            description: "".to_string(),
            queue: "1. Incoming".to_string(),
            assign_to: "".to_string(),
        }),
    })?;

    let board = Board::load(root.clone())?;
    let id = board.queues[0].tickets[0].id.clone();

    // 2. Move it
    run_main(CliArgs {
        root: Some(root.clone()),
        command: Some(Commands::Move {
            id: id.clone(),
            queue: "2. ToDo".to_string(),
        }),
    })?;

    let board = Board::load(root)?;
    let todo = board.queues.iter().find(|q| q.id == "2. ToDo").unwrap();
    assert!(
        todo.tickets.iter().any(|t| t.id == id),
        "Ticket should be present in the target queue after CLI 'move' command. Verify move_ticket logic and handle_command mapping."
    );

    Ok(())
}

#[test]
fn test_cli_remove() -> anyhow::Result<()> {
    let dir = tempdir()?;
    let root = dir.path().to_path_buf();

    // 1. Add a ticket
    run_main(CliArgs {
        root: Some(root.clone()),
        command: Some(Commands::Add {
            title: "Delete Me".to_string(),
            description: "".to_string(),
            queue: "1. Incoming".to_string(),
            assign_to: "".to_string(),
        }),
    })?;

    let board = Board::load(root.clone())?;
    let id = board.queues[0].tickets[0].id.clone();

    // 2. Remove it
    run_main(CliArgs {
        root: Some(root.clone()),
        command: Some(Commands::Remove { id: id.clone() }),
    })?;

    let board = Board::load(root)?;
    assert!(
        board.find_ticket_by_id(&id).is_none(),
        "Ticket should no longer exist after CLI 'remove' command. Check delete_ticket in model.rs and its usage in handle_command."
    );

    Ok(())
}

#[test]
fn test_change_limit() -> anyhow::Result<()> {
    let dir = tempdir()?;
    let root = dir.path().to_path_buf();

    // 1. Initialize board
    Board::ensure_initialized(&root)?;
    let board = Board::load(root.clone())?;
    assert_eq!(board.config.get_limit("1. Incoming"), None, "Newly initialized board should have no queue limits by default. Check Config::default if this fails.");

    // 2. Simulate the logic of on_request_change_limit
    let queue_id = "1. Incoming";
    let new_limit = 10;

    let mut board = Board::load(root.clone())?;
    board.config.set_limit(queue_id.to_string(), new_limit);
    board.config.write(&root)?;

    // 3. Verify
    let board_after = Board::load(root)?;
    assert_eq!(
        board_after.config.get_limit(queue_id),
        Some(10),
        "Queue limit should be updated in config.toml after request_change_limit. Ensure on_request_change_limit and Config::write are working as expected."
    );

    Ok(())
}
