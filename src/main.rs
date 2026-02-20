//! main.rs
//!
//! Purpose: Main entry point for the Slint Kanban application. Orchestrates UI and Backend.
//! Includes: UI event handlers, CLI command dispatch, reloading logic, type conversions,
//!           and NO_COLOR support (https://no-color.org/).
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

/// Returns `true` when colored output should be suppressed.
/// Honors the NO_COLOR standard: https://no-color.org/
/// NO_COLOR is respected if the variable is set (to any value, including empty).
pub fn no_color() -> bool {
    std::env::var_os("NO_COLOR").is_some()
}

/// Print a line to stdout, stripping ANSI codes when NO_COLOR is set.
/// Usage: `cprintln!("text {}", value)` — same as println! but color-aware.
macro_rules! cprintln {
    ($fmt:literal $(, $arg:expr)* $(,)?) => {
        if crate::no_color() {
            // Strip ANSI codes: replace ESC[ sequences with nothing
            let raw = format!($fmt $(, $arg)*);
            // Remove ANSI escape sequences \x1b[...m
            let stripped = raw
                .split('\x1b')
                .enumerate()
                .map(|(i, part)| {
                    if i == 0 {
                        part.to_string()
                    } else {
                        // Drop up to and including the first 'm'
                        match part.find('m') {
                            Some(pos) => part[pos + 1..].to_string(),
                            None => part.to_string(),
                        }
                    }
                })
                .collect::<String>();
            println!("{}", stripped);
        } else {
            println!($fmt $(, $arg)*);
        }
    };
}

slint::include_modules!();

/// Converts a domain Ticket into the Slint-generated TicketStr for UI binding.
/// `snippet` is the first line of the description, shown on the card preview.
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
        author: SharedString::from(&ticket.author),
        references: Rc::new(VecModel::from(refs)).into(),
    }
}

/// Pushes the full board state into the UI, applying search/date/user filters.
/// Called on every reload (initial load, file watcher event, or filter change).
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
        let mut filtered_tickets: Vec<&model::Ticket> = queue
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
            .collect();

        // Sort by updated_at: older at the top, newer at the bottom
        filtered_tickets.sort_by(|a, b| a.updated_at.cmp(&b.updated_at));

        let slint_tickets: Vec<TicketStr> = filtered_tickets
            .into_iter()
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

    init_callbacks(&ui, controller.clone());

    let watcher_root = root_path.clone();

    // File watcher: debounced reload on filesystem changes
    let c_watcher = controller.clone();
    std::thread::spawn(move || {
        let (tx, rx) = std::sync::mpsc::channel();
        let mut watcher = notify::recommended_watcher(tx).unwrap();

        if let Err(e) = watcher.watch(&watcher_root, RecursiveMode::Recursive) {
            eprintln!("Failed to watch directory: {:?}", e);
            return;
        }

        // Also watch user config file if possible
        if let Some(user_path) = model::Config::user_config_path() {
            if let Some(parent) = user_path.parent() {
                let _ = std::fs::create_dir_all(parent);
                if let Err(e) = watcher.watch(parent, RecursiveMode::NonRecursive) {
                    eprintln!("Warning: Failed to watch user config directory: {:?}", e);
                }
            }
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

                    // Debounce: wait 100ms then drain any events that arrived
                    // during the sleep, so rapid saves trigger only one reload.
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

pub(crate) fn init_callbacks(ui: &App, controller: Arc<AppController>) {
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
    ui.on_show_board_info(move || {
        c.handle_show_board_info();
    });

    let c = controller.clone();
    ui.on_focus_search(move || {
        c.handle_focus_search();
    });

    let c = controller.clone();
    ui.on_shortcut_create_ticket(move || {
        c.handle_shortcut_create_ticket();
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

    let nav_root = controller.root_path.clone();
    let nav_ui = ui.as_weak();
    ui.on_navigate_to(move |target_id| {
        let board = Board::load(nav_root.clone())
            .unwrap_or_else(|_| Board::load(nav_root.clone()).unwrap());

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

    // Search/filter callbacks trigger board reload to apply new filters
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
        c.handle_select_history_item();
    });
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
            cprintln!("Adding ticket: {} to queue: {}", title, queue);
            let author = board.config.active_user();
            board.create_ticket(&title, &description, &queue, &assign_to, author)?;
        }
        Commands::Update {
            id,
            title,
            description,
            assign_to,
            unassign,
        } => {
            cprintln!("Updating ticket: {}", id);
            let ticket = board
                .find_ticket_by_id(&id)
                .ok_or_else(|| anyhow::anyhow!("Ticket not found: {}", id))?;
            // Fields not provided on CLI are preserved from the existing ticket
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
                    cprintln!("\n=== {} ===", queue.name);
                    for t in filtered_tickets {
                        let user_display = if t.assigned_to.is_empty() {
                            "<unassigned>".to_string()
                        } else {
                            t.assigned_to.clone()
                        };
                        cprintln!("[{}] {} (Assigned: {})", t.id, t.title, user_display);
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
                cprintln!("Setting active user to: {}", user);
                config.user.active_user = user;
            }
            if let Some(mine) = show_only_mine {
                cprintln!("Setting show_only_mine to: {}", mine);
                config.user.show_only_mine = mine;
            }
            if let Some(user) = add_user {
                if !config.kanban.users.contains(&user) {
                    cprintln!("Adding user: {}", user);
                    config.kanban.users.push(user);
                }
            }
            config.write(&root_path)?;
        }
        Commands::Move { id, queue } => {
            cprintln!("Moving ticket: {} to queue: {}", id, queue);
            let _ticket = board
                .find_ticket_by_id(&id)
                .ok_or_else(|| anyhow::anyhow!("Ticket not found: {}", id))?;
            let source_queue = board
                .queues
                .iter()
                .find(|q| q.tickets.iter().any(|t| t.id == id))
                .ok_or_else(|| anyhow::anyhow!("Ticket not found in any queue: {}", id))?;
            board.move_ticket(&id, &source_queue.id, &queue)?;
        }
        Commands::Remove { id } => {
            cprintln!("Removing ticket: {}", id);
            board.delete_ticket(&id)?;
        }
        Commands::Open { path } => {
            cprintln!("Opening GUI for path: {:?}", path);
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

            cprintln!("ID:          {}", ticket.id);
            cprintln!("Title:       {}", ticket.title);
            cprintln!("Status:      {}", queue_name);
            cprintln!(
                "Assigned to: {}",
                if ticket.assigned_to.is_empty() {
                    "<unassigned>"
                } else {
                    &ticket.assigned_to
                }
            );
            cprintln!("Author:      {}", ticket.author);
            cprintln!("Created at:  {}", ticket.created_at);
            cprintln!("Updated at:  {}", ticket.updated_at);
            cprintln!("\nDescription:\n{}", ticket.description);
        }
    }

    Ok(())
}

/// Resolves the root path and dispatches to GUI or CLI command handler.
/// Root path priority: --root flag > KANBAN_HOME env var > ~/Kanban default.
fn run_main(args: CliArgs) -> anyhow::Result<()> {
    let root_path = if let Some(path) = args.root {
        path
    } else if let Ok(kanban_home) = std::env::var("KANBAN_HOME") {
        PathBuf::from(kanban_home)
    } else {
        let home_dir = std::env::var("HOME").expect("HOME directory not set");
        PathBuf::from(home_dir).join("Kanban")
    };

    // Ensure board directory and default queues exist
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
static TEST_INIT: std::sync::Once = std::sync::Once::new();

#[cfg(test)]
pub fn init_test_backend() {
    TEST_INIT.call_once(|| {
        i_slint_backend_testing::init_integration_test_with_system_time();
    });
}

#[cfg(test)]
mod gui_tests;
#[cfg(test)]
mod main_tests;
