use super::*;
use slint::Model;
use tempfile::tempdir;

#[test]
fn test_ui() -> anyhow::Result<()> {
    let ui = App::new()?;

    // 1. Test initialization
    assert_eq!(ui.get_board_queues().row_count(), 0, "Initially there should be no queues in the model. Ensure App initialization correctly sets an empty model.");
    assert!(
        !ui.get_is_dragging(),
        "is_dragging should be false by default."
    );
    assert!(
        !ui.get_is_editing(),
        "is_editing should be false by default."
    );
    assert!(
        !ui.get_is_viewing_ticket(),
        "is_viewing_ticket should be false by default."
    );

    // 2. Test interaction
    let queues_model = Rc::new(VecModel::from(vec![QueueStr {
        id: SharedString::from("q1"),
        name: SharedString::from("Queue 1"),
        tickets: Rc::new(VecModel::default()).into(),
        limit: -1,
        ticket_count: 0,
        visible: true,
    }]));
    ui.set_board_queues(queues_model.into());

    ui.invoke_test_trigger_add_ticket(SharedString::from("q1"));

    assert!(ui.get_is_editing(), "is_editing should be true after triggering add_ticket. Verify that test_trigger_add_ticket callback correctly updates the state.");
    assert_eq!(ui.get_target_queue_for_new(), "q1", "target_queue_for_new should match the queue ID where '+' was clicked. Check the value passed to test_trigger_add_ticket.");

    // 3. Test cancel edit
    ui.invoke_test_trigger_cancel_edit();
    assert!(
        !ui.get_is_editing(),
        "is_editing should be false after cancel."
    );

    // 4. Test view and close
    ui.set_active_ticket(TicketStr {
        id: "T1".into(),
        title: "Task 1".into(),
        description: "Desc 1".into(),
        snippet: "Desc 1".into(),
        created_at: "now".into(),
        updated_at: "now".into(),
        references: Rc::new(VecModel::from(vec![])).into(),
    });
    ui.set_is_viewing_ticket(true);
    assert!(ui.get_is_viewing_ticket(), "is_viewing_ticket should be true after explicit set. Check Slint property binding for is_viewing_ticket.");
    ui.invoke_test_trigger_close_view();
    assert!(
        !ui.get_is_viewing_ticket(),
        "is_viewing_ticket should be false after close. Verify that test_trigger_close_view callback correctly updates the state."
    );

    // 5. Test deletion callback
    let (tx, rx) = std::sync::mpsc::channel();
    ui.on_delete_ticket(move |tid| {
        tx.send(tid.to_string()).unwrap();
    });
    ui.invoke_test_trigger_delete_ticket("T-DELETE".into());
    let deleted_id = rx
        .recv_timeout(std::time::Duration::from_millis(100))
        .expect("Delete callback should be triggered");
    assert_eq!(deleted_id, "T-DELETE", "The ID passed to the delete callback should match the requested ID. Check how test_trigger_delete_ticket invokes the callback.");

    Ok(())
}

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
