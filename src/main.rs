//! main.rs
//!
//! Purpose: Main entry point for the Slint Kanban application. Orchestrates UI and Backend.
//! Includes: UI event handlers, reloading logic, and type conversions.
//! Constraints: Business logic should be in the model module, not here.

mod cli;
mod controller;
mod model;

use cli::{CliArgs, Commands};
use controller::AppController;
use model::Board;
use notify::{RecursiveMode, Watcher};
use slint::{ComponentHandle, SharedString, VecModel};
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;

slint::include_modules!();

pub fn ticket_to_slint(ticket: &model::Ticket, board: &Board) -> TicketStr {
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
        assigned_to: SharedString::from(&ticket.assigned_to),
        references: Rc::new(VecModel::from(refs)).into(),
    }
}

pub fn sync_ui_with_board(
    ui: &App,
    board: &Board,
    query: &str,
    date_from: &str,
    date_to: &str,
    show_only_mine: bool,
    active_user: &str,
) {
    let mut slint_queues: Vec<QueueStr> = vec![];

    for queue in &board.queues {
        let slint_tickets: Vec<TicketStr> = queue
            .tickets
            .iter()
            .filter(|t: &&model::Ticket| {
                let matches_search = t.matches(query);
                let matches_date = t.matches_date_range(date_from, date_to);
                let matches_user = if show_only_mine {
                    let target_user = if active_user == "<unassigned>" {
                        ""
                    } else {
                        active_user
                    };
                    t.assigned_to == target_user
                } else {
                    true
                };
                matches_search && matches_date && matches_user
            })
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

fn run_gui(root_path: PathBuf) -> anyhow::Result<()> {
    let ui = App::new()?;
    let controller = Arc::new(AppController::new(ui.as_weak(), root_path.clone()));

    println!("Using Kanban root: {:?}", root_path);

    // Initial load
    controller.reload()?;

    let watcher_root = root_path.clone();

    // Set up callbacks
    let c = controller.clone();
    ui.on_move_ticket(move |ticket_id, source_id, target_id| {
        c.handle_move(ticket_id.into(), source_id.into(), target_id.into());
    });

    let c = controller.clone();
    ui.on_delete_ticket(move |ticket_id| {
        c.handle_delete(ticket_id.into());
    });

    let c = controller.clone();
    ui.on_request_create_ticket(move |queue_id, title, description, assigned_to| {
        c.handle_create(
            queue_id.into(),
            title.into(),
            description.into(),
            assigned_to.into(),
        );
    });

    let c = controller.clone();
    ui.on_save_ticket(move |ticket_id, title, description, assigned_to| {
        c.handle_save(
            ticket_id.into(),
            title.into(),
            description.into(),
            assigned_to.into(),
        );
    });

    let c = controller.clone();
    ui.on_request_change_limit(move |queue_id, limit| {
        c.handle_change_limit(queue_id.into(), limit);
    });

    let c = controller.clone();
    ui.global::<UserGlobal>()
        .on_change_active_user(move |username| {
            c.handle_user_change(username.into());
        });

    let c = controller.clone();
    ui.global::<UserGlobal>()
        .on_toggle_show_only_mine(move |enabled| {
            c.handle_toggle_mine(enabled);
        });

    let c = controller.clone();
    ui.on_toggle_queue_visibility(move |queue_id, visible| {
        c.handle_queue_visibility(queue_id.into(), visible);
    });

    let c = controller.clone();
    ui.on_accept_search(move |query| {
        c.handle_search_history_add(query.into());
    });

    let c = controller.clone();
    ui.on_remove_search_item(move |query| {
        c.handle_search_history_remove(query.into());
    });

    let nav_root = root_path.clone(); // use local var
    let nav_ui = ui.as_weak();
    ui.on_navigate_to(move |target_id| {
        let board = Board::load(nav_root.clone())
            .unwrap_or_else(|_| Board::load(nav_root.clone()).unwrap());
        // ...
        // Implementation details...
        // To stay safe, I'll paste the original logic for these specific Read-Only callbacks
        // but updated to match the new structure if needed.

        let id_to_find = if target_id.starts_with('#') {
            &target_id[1..]
        } else {
            &target_id
        };

        if let Some(ticket) = board.find_ticket_by_id(id_to_find) {
            if let Some(ui) = nav_ui.upgrade() {
                ui.set_active_ticket(ticket_to_slint(ticket, &board));
                ui.set_is_viewing_ticket(true);
            }
        } else {
            if let Some(ui) = nav_ui.upgrade() {
                ui.invoke_show_warning_dialog(SharedString::from(format!(
                    "Ticket NOT FOUND: {}",
                    target_id
                )));
            }
        }
    });

    // Search/Filter callbacks just trigger reload/sync
    let c = controller.clone();
    ui.on_search_edited(move |_| {
        let _ = c.reload(); // Re-sync UI with new search query
    });

    let c = controller.clone();
    ui.on_date_filter_changed(move || {
        let _ = c.reload();
    });

    let c = controller.clone();
    ui.on_select_history_item(move |_| {
        let _ = c.reload();
    });

    // Watcher
    let c_watcher = controller.clone();
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

                    std::thread::sleep(Duration::from_millis(100));
                    while rx.try_recv().is_ok() {}

                    let c = c_watcher.clone();
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Err(e) = c.reload() {
                            eprintln!("Error reloading board: {:?}", e);
                        }
                    });
                }
                Ok(Err(e)) => eprintln!("Watch error: {:?}", e),
                Err(e) => {
                    eprintln!("Channel error: {:?}", e);
                    break;
                }
            }
        }
    });

    ui.run()?;
    Ok(())
}

fn handle_command(root_path: PathBuf, command: Commands) -> anyhow::Result<()> {
    let board = Board::load(root_path.clone())?;

    match command {
        Commands::Add {
            title,
            description,
            queue,
            assign_to,
        } => {
            println!("Adding ticket: {} to queue: {}", title, queue);
            board.create_ticket(&title, &description, &queue, &assign_to)?;
        }
        Commands::Update {
            id,
            title,
            description,
            assign_to,
            unassign,
        } => {
            println!("Updating ticket: {}", id);
            let ticket = board
                .find_ticket_by_id(&id)
                .ok_or_else(|| anyhow::anyhow!("Ticket not found: {}", id))?;
            let title = title.unwrap_or(ticket.title.clone());
            let description = description.unwrap_or(ticket.description.clone());
            let assign_to = if unassign {
                "".to_string()
            } else {
                assign_to.unwrap_or(ticket.assigned_to.clone())
            };
            board.update_ticket(&id, &title, &description, &assign_to)?;
        }
        Commands::List {
            assigned_to_user,
            unassigned,
            search,
            id,
            date_from,
            date_to,
        } => {
            let query = search.unwrap_or_default();
            let date_from_str = date_from.unwrap_or_default();
            let date_to_str = date_to.unwrap_or_default();

            for queue in &board.queues {
                let filtered_tickets: Vec<&model::Ticket> = queue
                    .tickets
                    .iter()
                    .filter(|t: &&model::Ticket| {
                        let matches_search = t.matches(&query);
                        let matches_date = t.matches_date_range(&date_from_str, &date_to_str);
                        let matches_id = if let Some(ref target_id) = id {
                            t.id == *target_id
                        } else {
                            true
                        };
                        let matches_user = if unassigned {
                            t.assigned_to.is_empty()
                        } else if let Some(ref user) = assigned_to_user {
                            t.assigned_to == *user
                        } else {
                            true
                        };
                        matches_search && matches_date && matches_id && matches_user
                    })
                    .collect();

                if !filtered_tickets.is_empty() {
                    println!("\n=== {} ===", queue.name);
                    for t in filtered_tickets {
                        let user_display = if t.assigned_to.is_empty() {
                            "<unassigned>".to_string()
                        } else {
                            t.assigned_to.clone()
                        };
                        println!("[{}] {} (Assigned: {})", t.id, t.title, user_display);
                    }
                }
            }
        }
        Commands::Configure {
            active_user,
            show_only_mine,
            add_user,
        } => {
            let mut config = board.config.clone();
            if let Some(user) = active_user {
                println!("Setting active user to: {}", user);
                config.active_user = user;
            }
            if let Some(mine) = show_only_mine {
                println!("Setting show_only_mine to: {}", mine);
                config.show_only_mine = mine;
            }
            if let Some(user) = add_user {
                if !config.users.contains(&user) {
                    println!("Adding user: {}", user);
                    config.users.push(user);
                }
            }
            config.write(&root_path)?;
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
        Commands::Show { id } => {
            let ticket = board
                .find_ticket_by_id(&id)
                .ok_or_else(|| anyhow::anyhow!("Ticket not found: {}", id))?;

            let queue_name = board
                .queues
                .iter()
                .find(|q| q.tickets.iter().any(|t| t.id == id))
                .map(|q| q.name.as_str())
                .unwrap_or("Unknown");

            println!("ID:          {}", ticket.id);
            println!("Title:       {}", ticket.title);
            println!("Status:      {}", queue_name);
            println!(
                "Assigned to: {}",
                if ticket.assigned_to.is_empty() {
                    "<unassigned>"
                } else {
                    &ticket.assigned_to
                }
            );
            println!("Created at:  {}", ticket.created_at);
            println!("Updated at:  {}", ticket.updated_at);
            println!("\nDescription:\n{}", ticket.description);
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
