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
    ui.invoke_request_create_ticket(
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
    ui.set_is_viewing_ticket(true);
    ui.invoke_test_trigger_close_view();

    // 5. Editing
    let t_edit = todo_queue.tickets.row_data(0).unwrap();
    ui.set_is_editing(true);
    ui.set_editing_id(t_edit.id.clone());
    ui.set_editing_title("Updated Perf Title".into());
    ui.invoke_save_ticket(
        t_edit.id.clone(),
        "Updated Perf Title".into(),
        t_edit.description.clone(),
        t_edit.assigned_to.clone(),
    );
    ui.set_is_editing(false); // Reset editing state
    controller.reload().unwrap();

    // 6. Deletion
    let t_to_del = todo_queue.tickets.row_data(0).unwrap().id;
    ui.invoke_test_trigger_delete_ticket(t_to_del.clone());
    controller.reload().unwrap();

    // 7. Limits
    ui.invoke_request_change_limit("2. ToDo".into(), 5);
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
    assert!(!ui.get_is_editing());
    assert!(!ui.get_is_viewing_ticket());

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
        total_elapsed < std::time::Duration::from_millis(1000),
        "Extreme performance regression: 10 full GUI cycles took over 1 second ({:?})",
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
}
