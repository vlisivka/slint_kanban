//! main.rs
//!
//! Purpose: Main entry point for the Slint Kanban application. Orchestrates UI and Backend.
//! Includes: UI event handlers, reloading logic, and type conversions.
//! Constraints: Business logic should be in the model module, not here.

mod cli;
mod model;

use cli::{CliArgs, Commands};
use model::{Board, Config};
use notify::{RecursiveMode, Watcher};
use slint::{ComponentHandle, SharedString, VecModel};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

slint::include_modules!();

static RELOAD_COUNT: AtomicU64 = AtomicU64::new(0);

fn ticket_to_slint(ticket: &model::Ticket, board: &Board) -> TicketStr {
    let snippet = ticket.description.lines().next().unwrap_or("").to_string();
    let refs: Vec<RefStr> = ticket
        .extract_references()
        .into_iter()
        .map(|id_with_hash| {
            let id = id_with_hash.trim_start_matches('#');
            let title = board
                .find_ticket_by_id(id)
                .map(|t| t.title.clone())
                .unwrap_or_else(|| "Unknown Ticket".to_string());
            RefStr {
                id: SharedString::from(id_with_hash),
                title: SharedString::from(title),
            }
        })
        .collect();
    TicketStr {
        id: SharedString::from(&ticket.id),
        title: SharedString::from(&ticket.title),
        description: SharedString::from(&ticket.description),
        snippet: SharedString::from(snippet),
        created_at: SharedString::from(&ticket.created_at),
        updated_at: SharedString::from(&ticket.updated_at),
        references: Rc::new(VecModel::from(refs)).into(),
    }
}

fn sync_ui_with_board(ui: &App, board: &Board, query: &str, date_from: &str, date_to: &str) {
    let mut slint_queues: Vec<QueueStr> = vec![];

    for queue in &board.queues {
        let slint_tickets: Vec<TicketStr> = queue
            .tickets
            .iter()
            .filter(|t| t.matches(query) && t.matches_date_range(date_from, date_to))
            .map(|t| ticket_to_slint(t, board))
            .collect();

        let ticket_count = slint_tickets.len() as i32;
        let limit = queue.limit.map(|l| l as i32).unwrap_or(-1);

        let tickets_model = Rc::new(VecModel::from(slint_tickets));

        slint_queues.push(QueueStr {
            id: SharedString::from(&queue.id),
            name: SharedString::from(&queue.name),
            tickets: tickets_model.into(),
            limit,
            ticket_count,
            visible: queue.visible,
        });
    }

    let queues_model = Rc::new(VecModel::from(slint_queues));
    ui.set_board_queues(queues_model.into());
}

fn reload_board(ui: &App, root_path: &Path) -> anyhow::Result<()> {
    let count = RELOAD_COUNT.fetch_add(1, Ordering::SeqCst);
    println!("Reloading board #{}...", count + 1);

    let board = Board::load(root_path.to_path_buf())?;
    let query = ui.get_search_query();
    let date_from = ui.get_date_from();
    let date_to = ui.get_date_to();
    sync_ui_with_board(
        ui,
        &board,
        query.as_str(),
        date_from.as_str(),
        date_to.as_str(),
    );

    let history: Vec<SharedString> = board
        .config
        .search_history
        .iter()
        .map(|s| SharedString::from(s))
        .collect();
    ui.set_search_history(Rc::new(VecModel::from(history)).into());

    Ok(())
}

fn run_gui(root_path: PathBuf) -> anyhow::Result<()> {
    let ui = App::new()?;

    println!("Using Kanban root: {:?}", root_path);

    // Initial load
    reload_board(&ui, &root_path)?;

    let ui_handle = ui.as_weak();
    let watcher_root = root_path.clone();

    // Set up callbacks
    let move_root = root_path.clone();
    let move_ui_handle = ui.as_weak();

    ui.on_move_ticket(move |ticket_id, source_id, target_id| {
        let board = match Board::load(move_root.clone()) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("Error loading board for move: {:?}", e);
                return;
            }
        };

        let resolved_target_id = board.resolve_queue_id(&target_id);

        if source_id == resolved_target_id {
            return;
        }

        println!(
            "Moving ticket {} from {} to {}",
            ticket_id, source_id, resolved_target_id
        );
        if let Err(e) = board.move_ticket(&ticket_id, &source_id, &resolved_target_id) {
            eprintln!("Error moving ticket: {:?}", e);
            if let Some(ui) = move_ui_handle.upgrade() {
                ui.invoke_show_warning_dialog(SharedString::from(e.to_string()));
            }
        }
    });

    let delete_root = root_path.clone();
    ui.on_delete_ticket(move |ticket_id| {
        let board = match Board::load(delete_root.clone()) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("Error loading board for delete: {:?}", e);
                return;
            }
        };

        println!("Deleting ticket {}", ticket_id);
        if let Err(e) = board.delete_ticket(&ticket_id) {
            eprintln!("Error deleting ticket: {:?}", e);
        }
    });

    std::thread::spawn(move || {
        let (tx, rx) = std::sync::mpsc::channel();
        let mut watcher = notify::recommended_watcher(tx).unwrap();

        if let Err(e) = watcher.watch(&watcher_root, RecursiveMode::Recursive) {
            eprintln!("Failed to watch directory: {:?}", e);
            return;
        }

        use std::time::Duration;

        loop {
            match rx.recv() {
                Ok(Ok(event)) => {
                    use notify::EventKind;

                    // Only reload on significant changes. Ignore Access, Metadata, etc.
                    let should_reload = match event.kind {
                        EventKind::Create(_) | EventKind::Remove(_) => true,
                        EventKind::Modify(m) => matches!(
                            m,
                            notify::event::ModifyKind::Data(_) | notify::event::ModifyKind::Name(_)
                        ),
                        _ => false,
                    };

                    if !should_reload {
                        continue;
                    }

                    // Fixed-window debounce: wait a bit and then drain everything
                    std::thread::sleep(Duration::from_millis(100));
                    while rx.try_recv().is_ok() {
                        // Drainage loop
                    }

                    let ui_handle = ui_handle.clone();
                    let root = watcher_root.clone();

                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(ui) = ui_handle.upgrade() {
                            if let Err(e) = reload_board(&ui, &root) {
                                eprintln!("Error reloading board: {:?}", e);
                            }
                        }
                    });
                }
                Ok(Err(e)) => {
                    eprintln!("Watch error: {:?}", e);
                }
                Err(e) => {
                    eprintln!("Channel error: {:?}", e);
                    break;
                }
            }
        }
    });

    let save_root = root_path.clone();
    ui.on_save_ticket(move |ticket_id, title, description| {
        let board = match Board::load(save_root.clone()) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("Error loading board for save: {:?}", e);
                return;
            }
        };

        println!("Saving ticket {}", ticket_id);
        if let Err(e) = board.update_ticket(&ticket_id, &title, &description) {
            eprintln!("Error saving ticket: {:?}", e);
        }
    });

    let nav_root = root_path.clone();
    let nav_ui_handle = ui.as_weak();
    ui.on_navigate_to(move |target_id| {
        let board = match Board::load(nav_root.clone()) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("Error loading board for navigation: {:?}", e);
                return;
            }
        };

        // If target_id starts with #, strip it for search
        let id_to_find = if target_id.starts_with('#') {
            &target_id[1..]
        } else {
            &target_id
        };

        if let Some(ticket) = board.find_ticket_by_id(id_to_find) {
            if let Some(ui) = nav_ui_handle.upgrade() {
                ui.set_active_ticket(ticket_to_slint(ticket, &board));
                ui.set_is_viewing_ticket(true);
            }
        } else {
            if let Some(ui) = nav_ui_handle.upgrade() {
                ui.invoke_show_warning_dialog(SharedString::from(format!(
                    "Ticket NOT FOUND: {}",
                    target_id
                )));
            }
        }
    });

    let search_root = root_path.clone();
    let search_ui_handle = ui.as_weak();
    ui.on_search_edited(move |query| {
        let board = match Board::load(search_root.clone()) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("Error loading board for search: {:?}", e);
                return;
            }
        };
        if let Some(ui) = search_ui_handle.upgrade() {
            let date_from = ui.get_date_from();
            let date_to = ui.get_date_to();
            sync_ui_with_board(
                &ui,
                &board,
                query.as_str(),
                date_from.as_str(),
                date_to.as_str(),
            );
        }
    });

    let accept_root = root_path.clone();
    let accept_ui_handle = ui.as_weak();
    ui.on_accept_search(move |query| {
        let mut config = match Config::load(&accept_root) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error loading config for history: {:?}", e);
                return;
            }
        };
        config.add_search_to_history(query.to_string());
        if let Err(e) = config.write(&accept_root) {
            eprintln!("Error writing config with history: {:?}", e);
        }
        // Reload history in UI
        if let Some(ui) = accept_ui_handle.upgrade() {
            let history: Vec<SharedString> = config
                .search_history
                .iter()
                .map(|s| SharedString::from(s))
                .collect();
            ui.set_search_history(Rc::new(VecModel::from(history)).into());
        }
    });

    let select_root = root_path.clone();
    let select_ui_handle = ui.as_weak();
    ui.on_select_history_item(move |query| {
        let board = match Board::load(select_root.clone()) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("Error loading board for history selection: {:?}", e);
                return;
            }
        };
        if let Some(ui) = select_ui_handle.upgrade() {
            let date_from = ui.get_date_from();
            let date_to = ui.get_date_to();
            sync_ui_with_board(
                &ui,
                &board,
                query.as_str(),
                date_from.as_str(),
                date_to.as_str(),
            );
        }
    });

    let remove_root = root_path.clone();
    let remove_ui_handle = ui.as_weak();
    ui.on_remove_search_item(move |query| {
        let mut config = match Config::load(&remove_root) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error loading config for history removal: {:?}", e);
                return;
            }
        };
        config.remove_search_from_history(&query);
        if let Err(e) = config.write(&remove_root) {
            eprintln!("Error writing config after history removal: {:?}", e);
        }
        // Reload history in UI
        if let Some(ui) = remove_ui_handle.upgrade() {
            let history: Vec<SharedString> = config
                .search_history
                .iter()
                .map(|s| SharedString::from(s))
                .collect();
            ui.set_search_history(Rc::new(VecModel::from(history)).into());
        }
    });

    let date_root = root_path.clone();
    let date_ui_handle = ui.as_weak();
    ui.on_date_filter_changed(move || {
        let board = match Board::load(date_root.clone()) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("Error loading board for date filter: {:?}", e);
                return;
            }
        };
        if let Some(ui) = date_ui_handle.upgrade() {
            let query = ui.get_search_query();
            let date_from = ui.get_date_from();
            let date_to = ui.get_date_to();
            sync_ui_with_board(
                &ui,
                &board,
                query.as_str(),
                date_from.as_str(),
                date_to.as_str(),
            );
        }
    });

    let toggle_root = root_path.clone();
    ui.on_toggle_queue_visibility(move |queue_id, visible| {
        let mut config = match Config::load(&toggle_root) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error loading config for toggle: {:?}", e);
                return;
            }
        };

        config.set_visible(queue_id.to_string(), visible);
        if let Err(e) = config.write(&toggle_root) {
            eprintln!("Error writing config: {:?}", e);
        }
        // No manual reload here - rely on file watcher
    });

    let create_root = root_path.clone();
    let create_ui_handle = ui.as_weak();
    ui.on_request_create_ticket(move |queue_id, title, description| {
        let board = match Board::load(create_root.clone()) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("Error loading board for create: {:?}", e);
                return;
            }
        };

        println!("Creating ticket in queue {}", queue_id);
        if let Err(e) = board.create_ticket(&title, &description, &queue_id) {
            eprintln!("Error creating ticket: {:?}", e);
            if let Some(ui) = create_ui_handle.upgrade() {
                ui.invoke_show_warning_dialog(SharedString::from(e.to_string()));
            }
        }
    });

    let limit_root = root_path.clone();
    ui.on_request_change_limit(move |queue_id, limit| {
        let mut board = match Board::load(limit_root.clone()) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("Error loading board for limit change: {:?}", e);
                return;
            }
        };

        println!("Changing limit for queue {} to {}", queue_id, limit);
        if limit < 0 {
            board.config.queue_limits.remove(&queue_id.to_string());
        } else {
            board.config.set_limit(queue_id.to_string(), limit as usize);
        }

        if let Err(e) = board.config.write(&limit_root) {
            eprintln!("Error saving config: {:?}", e);
        }
    });

    ui.run()?;
    Ok(())
}

fn handle_command(root_path: PathBuf, command: Commands) -> anyhow::Result<()> {
    let board = Board::load(root_path)?;

    match command {
        Commands::Add {
            title,
            description,
            queue,
        } => {
            println!("Adding ticket: {} to queue: {}", title, queue);
            board.create_ticket(&title, &description, &queue)?;
        }
        Commands::Update {
            id,
            title,
            description,
        } => {
            println!("Updating ticket: {}", id);
            let ticket = board
                .find_ticket_by_id(&id)
                .ok_or_else(|| anyhow::anyhow!("Ticket not found: {}", id))?;
            let title = title.unwrap_or(ticket.title.clone());
            let description = description.unwrap_or(ticket.description.clone());
            board.update_ticket(&id, &title, &description)?;
        }
        Commands::Move { id, queue } => {
            println!("Moving ticket: {} to queue: {}", id, queue);
            let _ticket = board
                .find_ticket_by_id(&id)
                .ok_or_else(|| anyhow::anyhow!("Ticket not found: {}", id))?;
            // We need the source queue ID
            let source_queue = board
                .queues
                .iter()
                .find(|q| q.tickets.iter().any(|t| t.id == id))
                .ok_or_else(|| anyhow::anyhow!("Ticket not found in any queue: {}", id))?;
            board.move_ticket(&id, &source_queue.id, &queue)?;
        }
        Commands::Remove { id } => {
            println!("Removing ticket: {}", id);
            board.delete_ticket(&id)?;
        }
        Commands::Open { path } => {
            println!("Opening GUI for path: {:?}", path);
            run_gui(path)?;
        }
    }

    Ok(())
}

fn run_main(args: CliArgs) -> anyhow::Result<()> {
    let root_path = if let Some(path) = args.root {
        path
    } else {
        let home_dir = std::env::var("HOME").expect("HOME directory not set");
        PathBuf::from(home_dir).join("Kanban")
    };

    // Ensure directory exists and default queues are created for initial run
    Board::ensure_initialized(&root_path)?;

    if let Some(command) = args.command {
        handle_command(root_path, command)
    } else {
        run_gui(root_path)
    }
}

fn main() -> anyhow::Result<()> {
    use clap::Parser;
    let args = CliArgs::parse();
    run_main(args)
}

#[cfg(test)]
mod main_tests;
