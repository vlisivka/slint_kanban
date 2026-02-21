#[cfg(test)]
use super::*;
use slint::{ComponentHandle, Model};
use tempfile::tempdir;
fn setup_test_app() -> (App, Arc<AppController>, PathBuf, tempfile::TempDir) {
    crate::init_test_backend();
    let temp_dir = tempdir().unwrap();
    let root_path = temp_dir.path().to_path_buf();

    // Initialize board with some data
    Board::ensure_initialized(&root_path).unwrap();
    let board = Board::load(root_path.clone()).unwrap();
    board
        .create_ticket("Ticket 1", "Desc 1", "2. ToDo", "user1", "author")
        .unwrap();
    board
        .create_ticket("Ticket 2", "Desc 2", "6. Done", "user2", "author")
        .unwrap();

    let ui = App::new().unwrap();
    let controller = Arc::new(AppController::new(ui.as_weak(), root_path.clone()));

    // Initialize logic
    controller.reload().unwrap();
    crate::init_callbacks(&ui, controller.clone());

    (ui, controller, root_path, temp_dir)
}

fn run_gui_interaction_cycle(ui: &App, controller: &Arc<AppController>, _root_path: &PathBuf) {
    // 1. Search (and rapid typing)
    for i in 0..5 {
        let query = format!("Ticket {}", i % 2 + 1);
        ui.set_search_query(query.clone().into());
        ui.invoke_search_edited(query.into());
    }
    ui.set_search_query("".into());
    ui.invoke_search_edited("".into());

    // 2. Filter & Visibility
    ui.global::<UserGlobal>().set_active_user("user1".into());
    ui.global::<UserGlobal>().invoke_toggle_show_only_mine(true);
    controller.reload().unwrap();
    ui.global::<UserGlobal>()
        .invoke_toggle_show_only_mine(false);
    controller.reload().unwrap();

    // 3. Creation & Cancel
    ui.invoke_test_trigger_add_ticket("2. ToDo".into());
    ui.invoke_test_trigger_cancel_edit();

    ui.invoke_test_trigger_add_ticket("2. ToDo".into());
    ui.invoke_create_ticket(
        "2. ToDo".into(),
        "Perf Task".into(),
        "Perf Details".into(),
        "user1".into(),
    );
    controller.reload().unwrap();

    // 4. View & Close
    let queues = ui.get_board_queues();
    let todo_queue = queues.iter().find(|q| q.id == "2. ToDo").unwrap();
    let t_view = todo_queue.tickets.row_data(0).unwrap();
    ui.set_active_ticket(t_view);
    ui.set_show_ticket_view_dialog(true);
    ui.invoke_test_trigger_close_view();

    // 5. Editing
    let t_edit = todo_queue.tickets.row_data(0).unwrap();
    ui.set_show_ticket_edit_dialog(true);
    ui.set_editing_ticket_id(t_edit.id.clone());
    ui.set_editing_ticket_title("Updated Perf Title".into());
    ui.invoke_update_ticket(
        t_edit.id.clone(),
        "Updated Perf Title".into(),
        t_edit.description.clone(),
        t_edit.assigned_to.clone(),
    );
    ui.set_show_ticket_edit_dialog(false); // Reset editing state
    controller.reload().unwrap();

    // 6. Deletion
    let t_to_del = todo_queue.tickets.row_data(0).unwrap().id;
    ui.invoke_test_trigger_delete_ticket(t_to_del.clone());
    controller.reload().unwrap();

    // 7. Limits
    ui.invoke_set_queue_limit("2. ToDo".into(), 5);
    controller.reload().unwrap();

    // 8. Search History Cycle
    ui.set_search_query("History Item".into());
    ui.invoke_accept_search("History Item".into());
    controller.reload().unwrap();
    ui.invoke_remove_search_item("History Item".into());
    controller.reload().unwrap();
}

#[test]
fn test_gui_suite() {
    let (ui, controller, root_path, _temp_dir) = setup_test_app();

    // I. Correctness: Run one cycle and verify end state
    println!("Checking GUI correctness...");
    run_gui_interaction_cycle(&ui, &controller, &root_path);
    assert_eq!(ui.get_board_queues().row_count(), 7); // Still 7 queues
    assert!(!ui.get_show_ticket_edit_dialog());
    assert!(!ui.get_show_ticket_view_dialog());

    // II. Heavy Performance: Run 10 full interaction cycles
    println!("Starting heavy GUI performance test (10 full cycles)...");
    let start = std::time::Instant::now();
    for i in 0..10 {
        let cycle_start = std::time::Instant::now();
        run_gui_interaction_cycle(&ui, &controller, &root_path);
        println!("Cycle {} took: {:?}", i + 1, cycle_start.elapsed());
    }
    let total_elapsed = start.elapsed();
    println!(
        "Total time for 10 full interaction cycles: {:?}",
        total_elapsed
    );

    assert!(
        total_elapsed < std::time::Duration::from_millis(10000),
        "Extreme performance regression: 10 full GUI cycles took over 10 seconds ({:?})",
        total_elapsed
    );

    // III. Fuzzy Stability: 100 random steps
    println!("Starting fuzzy stability test (100 random steps)...");
    let mut rng = rand::thread_rng();
    use rand::Rng;
    let qids = vec!["1. Incoming", "2. ToDo", "3. Doing", "6. Done"];
    for i in 0..100 {
        let step_start = std::time::Instant::now();
        match rng.gen_range(0..5) {
            0 => ui.set_search_query("Fuzz".into()),
            1 => ui
                .global::<UserGlobal>()
                .invoke_toggle_show_only_mine(rng.gen_bool(0.5)),
            2 => ui.invoke_search_edited("".into()),
            3 => {
                let qid = qids[rng.gen_range(0..qids.len())];
                ui.invoke_toggle_queue_visibility(qid.into(), rng.gen_bool(0.5));
            }
            _ => {
                controller.reload().unwrap();
            }
        }
        let step_elapsed = step_start.elapsed();
        assert!(
            step_elapsed < std::time::Duration::from_millis(200),
            "GUI freeze detected at step {}: took {:?}",
            i,
            step_elapsed
        );
    }

    // IV. Test specific bugs
    println!("Testing regression bugs...");
    test_gui_user_desync();

    // V. Test ID visibility
    println!("Checking Ticket ID visibility...");
    test_ticket_id_visibility();

    // VI. Test comments addition
    println!("Checking GUI Add Comment...");
    test_gui_add_comment();
}

fn test_ticket_id_visibility() {
    let (ui, _controller, _root_path, _temp_dir) = setup_test_app();

    // Check if we can get the ID from the UI-exposed callback
    let id = ui.invoke_test_get_first_ticket_id();
    println!("First ticket ID in UI: #{}", id);

    assert!(!id.is_empty(), "First ticket ID should not be empty in UI");
    assert!(id.len() >= 4, "Ticket ID '{}' seems too short", id);
}

fn test_gui_user_desync() {
    let temp_dir = tempdir().unwrap();
    let root_path = temp_dir.path().to_path_buf();

    Board::ensure_initialized(&root_path).unwrap();

    let ui = App::new().unwrap();
    let controller = Arc::new(AppController::new(ui.as_weak(), root_path.clone()));

    let user_global = ui.global::<UserGlobal>();
    println!(
        "Before reload: users={:?}, active='{}'",
        user_global.get_users().row_count(),
        user_global.get_active_user()
    );

    controller.reload().unwrap();
    crate::init_callbacks(&ui, controller.clone());

    println!(
        "After reload: users={:?}, active='{}'",
        user_global.get_users().row_count(),
        user_global.get_active_user()
    );

    ui.invoke_test_trigger_add_ticket("1. Incoming".into());
    let assigned = ui.get_editing_ticket_assignee();
    println!("Final editing assigned to: '{}'", assigned);

    assert_eq!(assigned, "user", "Assigned user should be 'user'");
}

fn test_gui_add_comment() {
    let (ui, controller, root_path, _temp_dir) = setup_test_app();

    // 1. Get first ticket ID
    let id = ui.invoke_test_get_first_ticket_id();
    assert!(!id.is_empty(), "Need a ticket to add a comment to");

    // 2. Add comment via GUI callback
    ui.invoke_add_comment(id.clone().into(), "Hello from GUI test".into());

    // 3. Verify it was written to Board
    let board = Board::load(root_path).unwrap();
    let ticket = board.find_ticket_by_id(&id).unwrap();

    assert_eq!(
        ticket.comments.len(),
        1,
        "There should be exactly one comment added via GUI callback"
    );
    assert_eq!(ticket.comments[0].content, "Hello from GUI test");
    assert_eq!(
        ticket.comments[0].metadata.author, "user",
        "Author should be the mock active user"
    );
}
