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

slint::include_modules!();

/// Converts a domain Ticket into the Slint-generated TicketStr for UI binding.
/// `snippet` is the first line of the description, shown on the card preview.
pub fn into_slint_ticket(ticket: &model::Ticket, board: &Board) -> TicketStr {
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

    let slint_comments: Vec<CommentStr> = ticket
        .comments
        .iter()
        .map(|c| {
            let crefs: Vec<RefStr> = c
                .references
                .iter()
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
            CommentStr {
                id: SharedString::from(&c.id),
                author: SharedString::from(&c.metadata.author),
                created_at: SharedString::from(&c.metadata.created_at),
                updated_at: SharedString::from(&c.metadata.updated_at),
                content: SharedString::from(&c.content),
                references: Rc::new(VecModel::from(crefs)).into(),
            }
        })
        .collect();

    let mut attachment_count = 0;
    let attach_dir = board.ticket_path(&ticket.id).join("attachment");
    if let Ok(entries) = std::fs::read_dir(attach_dir) {
        attachment_count = entries
            .flatten()
            .filter(|e| e.file_type().map(|ft| ft.is_file()).unwrap_or(false))
            .count() as i32;
    }

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
        comments: Rc::new(VecModel::from(slint_comments)).into(),
        attachment_count,
    }
}

/// Pushes the full board state into the UI, applying search/date/user filters.
/// Called on every reload (initial load, file watcher event, or filter change).
pub fn sync_board_to_ui(
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
            .filter(|t| {
                let user_filter = if show_only_mine {
                    Some(if active_user == "<unassigned>" {
                        ""
                    } else {
                        active_user
                    })
                } else {
                    None
                };
                t.matches_all(query, date_from, date_to, user_filter)
            })
            .collect();

        // Sort by updated_at: older at the top, newer at the bottom
        filtered_tickets.sort_by(|a, b| a.updated_at.cmp(&b.updated_at));

        let slint_tickets: Vec<TicketStr> = filtered_tickets
            .into_iter()
            .map(|t| into_slint_ticket(t, board))
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
        c.handle_move_ticket(ticket_id.into(), source_id.into(), target_id.into());
    });

    let c = controller.clone();
    ui.on_delete_ticket(move |ticket_id| {
        c.handle_delete_ticket(ticket_id.into());
    });

    let c = controller.clone();
    ui.on_create_ticket(move |queue_id, title, description, assigned_to| {
        c.handle_create_ticket(
            queue_id.into(),
            title.into(),
            description.into(),
            assigned_to.into(),
        );
    });

    let c = controller.clone();
    ui.on_update_ticket(move |ticket_id, title, description, assigned_to| {
        c.handle_update_ticket(
            ticket_id.into(),
            title.into(),
            description.into(),
            assigned_to.into(),
        );
    });

    let c = controller.clone();
    ui.on_add_comment(move |ticket_id, content| {
        c.handle_add_comment(ticket_id.into(), content.into());
    });

    let c = controller.clone();
    ui.on_attach_file(move |ticket_id| c.handle_attach_file(ticket_id.into()).into());

    let c = controller.clone();
    ui.on_open_attachment_folder(move |ticket_id| {
        c.handle_open_attachment_folder(ticket_id.into());
    });

    let c = controller.clone();
    ui.on_set_queue_limit(move |queue_id, limit| {
        c.handle_set_queue_limit(queue_id.into(), limit);
    });

    let c = controller.clone();
    ui.global::<UserGlobal>()
        .on_change_active_user(move |username| {
            c.handle_change_active_user(username.into());
        });

    let c = controller.clone();
    ui.global::<UserGlobal>()
        .on_toggle_show_only_mine(move |enabled| {
            c.handle_toggle_show_only_mine(enabled);
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
    ui.on_shortcut_open_new_ticket_dialog(move || {
        c.handle_shortcut_create_ticket();
    });

    let c = controller.clone();
    ui.on_toggle_queue_visibility(move |queue_id, visible| {
        c.handle_toggle_queue_visibility(queue_id.into(), visible);
    });

    let c = controller.clone();
    ui.on_accept_search(move |query| {
        c.handle_accept_search(query.into());
    });

    let c = controller.clone();
    ui.on_remove_search_item(move |query| {
        c.handle_remove_search_item(query.into());
    });

    let c = controller.clone();
    ui.on_navigate_to(move |target_id| {
        c.handle_navigate_to(target_id.into());
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
            println!("Adding ticket: {} to queue: {}", title, queue);
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
            println!("Updating ticket: {}", id);
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
                    .filter(|t| {
                        let user_filter = if unassigned {
                            Some("")
                        } else {
                            assigned_to_user.as_deref()
                        };
                        let matches_id = id.as_ref().is_none_or(|target| t.id == *target);
                        matches_id
                            && t.matches_all(&query, &date_from_str, &date_to_str, user_filter)
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
                config.user.active_user = user;
            }
            if let Some(mine) = show_only_mine {
                println!("Setting show_only_mine to: {}", mine);
                config.user.show_only_mine = mine;
            }
            if let Some(user) = add_user {
                if !config.kanban.users.contains(&user) {
                    println!("Adding user: {}", user);
                    config.kanban.users.push(user);
                }
            }
            config.write(&root_path)?;
        }
        Commands::Move { id, queue } => {
            println!("Moving ticket: {} to queue: {}", id, queue);
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
            println!("Author:      {}", ticket.author);
            println!("Created at:  {}", ticket.created_at);
            println!("Updated at:  {}", ticket.updated_at);
            println!("\nDescription:\n{}", ticket.description);
            if !ticket.comments.is_empty() {
                println!("\nComments:");
                for c in &ticket.comments {
                    println!(
                        "- [{}] {} ({}): {}",
                        c.id, c.metadata.author, c.metadata.created_at, c.content
                    );
                }
            }
        }
        Commands::Comment { id, content } => {
            println!("Adding comment to ticket: {}", id);
            let author = board.config.active_user();
            board.add_comment(&id, &content, author)?;
        }
        Commands::Attach { id, file } => {
            let link = board.attach_file(&id, &file)?;
            println!("{}", link);
        }
    }

    Ok(())
}

/// Resolves the root path and dispatches to GUI or CLI command handler.
/// Root path priority: --root flag > KANBAN_HOME env var > ~/Kanban default.
fn run_cli(args: CliArgs) -> anyhow::Result<()> {
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
    run_cli(args)
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
