//! main.rs
//!
//! Purpose: Main entry point for the Slint Kanban application. Orchestrates UI and Backend.
//! Includes: UI event handlers, CLI command dispatch, reloading logic, type conversions,
//!           and NO_COLOR support (https://no-color.org/).
//! Constraints: Business logic should be in the model module, not here.

mod cli;
mod controller;
mod model;

use cli::{CliArgs, Commands, SprintAction};
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
    ui.on_request_stats(move || {
        c.handle_request_stats();
    });

    let c = controller.clone();
    ui.on_request_sprints_view(move || {
        c.handle_request_sprints_view();
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

fn handle_command(
    root_path: std::path::PathBuf,
    command: Commands,
    out: &mut dyn std::io::Write,
) -> anyhow::Result<()> {
    let board = Board::load(root_path.clone())?;

    match command {
        Commands::Add {
            title,
            description,
            queue,
            assign_to,
        } => {
            writeln!(out, "Adding ticket: {} to queue: {}", title, queue)?;
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
            writeln!(out, "Updating ticket: {}", id)?;
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
                    writeln!(out, "\n=== {} ===", queue.name)?;
                    for t in filtered_tickets {
                        let user_display = if t.assigned_to.is_empty() {
                            "<unassigned>".to_string()
                        } else {
                            t.assigned_to.clone()
                        };
                        writeln!(out, "[{}] {} (Assigned: {})", t.id, t.title, user_display)?;
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
                writeln!(out, "Setting active user to: {}", user)?;
                config.user.active_user = user;
            }
            if let Some(mine) = show_only_mine {
                writeln!(out, "Setting show_only_mine to: {}", mine)?;
                config.user.show_only_mine = mine;
            }
            if let Some(user) = add_user {
                if !config.kanban.users.contains(&user) {
                    writeln!(out, "Adding user: {}", user)?;
                    config.kanban.users.push(user);
                }
            }
            config.write(&root_path)?;
        }
        Commands::Stats { user } => {
            let summary = crate::model::stats::get_board_summary(&board);

            writeln!(out, "== Board Summary ==")?;
            writeln!(out, "Total tickets: {}", summary.total_tickets)?;
            writeln!(out, "Unassigned:    {}", summary.unassigned_tickets)?;
            writeln!(
                out,
                "Avg Lead Time: {}",
                summary
                    .avg_lead_time_days
                    .map(|d| format!("{:.1} days", d))
                    .unwrap_or_else(|| "-".to_string())
            )?;
            writeln!(
                out,
                "Avg Cycle Time: {}\n",
                summary
                    .avg_cycle_time_days
                    .map(|d| format!("{:.1} days", d))
                    .unwrap_or_else(|| "-".to_string())
            )?;

            writeln!(out, "== Tickets per Queue ==")?;
            writeln!(
                out,
                "{:<20} {:>5} {:>6} {:>5}",
                "Queue", "Count", "Limit", "Usage"
            )?;
            for q in summary.queues {
                let limit_str = q
                    .limit
                    .map(|l| l.to_string())
                    .unwrap_or_else(|| "-".to_string());
                let usage_str = if let Some(limit) = q.limit {
                    if limit > 0 {
                        format!("{}%", (q.count * 100) / limit)
                    } else {
                        "-".to_string()
                    }
                } else {
                    "-".to_string()
                };
                writeln!(
                    out,
                    "{:<20} {:>5} {:>6} {:>5}",
                    q.name, q.count, limit_str, usage_str
                )?;
            }
            writeln!(out)?;

            writeln!(out, "== Tickets per User ==")?;
            writeln!(out, "{:<20} {:>5}", "User", "Count")?;
            for u in summary.users {
                if user.as_ref().is_some_and(|f| f != &u.name) {
                    continue;
                }
                writeln!(out, "{:<20} {:>5}", u.name, u.count)?;
            }
        }
        Commands::Move { id, queue } => {
            writeln!(out, "Moving ticket: {} to queue: {}", id, queue)?;
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
            writeln!(out, "Removing ticket: {}", id)?;
            board.delete_ticket(&id)?;
        }
        Commands::Open { path } => {
            writeln!(out, "Opening GUI for path: {:?}", path)?;
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

            let ticket_dir = board.ticket_path(&id);
            let attach_dir = ticket_dir.join("attachment");
            let attachment_count = if attach_dir.exists() {
                std::fs::read_dir(&attach_dir)
                    .map(|entries| {
                        entries
                            .filter_map(Result::ok)
                            .filter(|e| e.path().is_file())
                            .count()
                    })
                    .unwrap_or(0)
            } else {
                0
            };

            writeln!(out, "ID:          {}", ticket.id)?;
            writeln!(out, "Title:       {}", ticket.title)?;
            writeln!(out, "Status:      {}", queue_name)?;
            writeln!(
                out,
                "Assigned to: {}",
                if ticket.assigned_to.is_empty() {
                    "<unassigned>"
                } else {
                    &ticket.assigned_to
                }
            )?;
            writeln!(out, "Author:      {}", ticket.author)?;
            writeln!(out, "Created at:  {}", ticket.created_at)?;
            writeln!(out, "Updated at:  {}", ticket.updated_at)?;
            writeln!(out, "Attachments: {}", attachment_count)?;
            writeln!(out, "\nDescription:\n{}", ticket.description)?;
            if !ticket.comments.is_empty() {
                writeln!(out, "\nComments:")?;
                for c in &ticket.comments {
                    writeln!(
                        out,
                        "- [{}] {} ({}): {}",
                        c.id, c.metadata.author, c.metadata.created_at, c.content
                    )?;
                }
            }
        }
        Commands::Comment { id, content } => {
            writeln!(out, "Adding comment to ticket: {}", id)?;
            let author = board.config.active_user();
            board.add_comment(&id, &content, author)?;
        }
        Commands::Attach {
            id,
            file,
            list,
            show,
            open,
        } => {
            let ticket_dir = board.ticket_path(&id);
            let attach_dir = ticket_dir.join("attachment");

            if !ticket_dir.exists() {
                anyhow::bail!("Ticket not found: {}", id);
            }

            if open {
                if !attach_dir.exists() {
                    std::fs::create_dir_all(&attach_dir)?;
                }
                writeln!(
                    out,
                    "Opening attachments directory: {}",
                    attach_dir.display()
                )?;
                #[cfg(not(test))]
                open::that(&attach_dir)?;
            } else if show {
                writeln!(out, "{}", attach_dir.display())?;
            } else if list {
                if attach_dir.exists() {
                    let mut found = false;
                    for entry in std::fs::read_dir(&attach_dir)? {
                        let entry = entry?;
                        if entry.path().is_file() {
                            found = true;
                            writeln!(out, "{}", entry.file_name().to_string_lossy())?;
                        }
                    }
                    if !found {
                        writeln!(out, "No attachments found.")?;
                    }
                } else {
                    writeln!(out, "No attachments found.")?;
                }
            } else if let Some(f) = file {
                let link = board.attach_file(&id, &f)?;
                writeln!(out, "{}", link)?;
            } else {
                anyhow::bail!("No action specified. Use --file, --list, --show, or --open.");
            }
        }
        Commands::Sprint { action } => {
            let mut sprint_board = board;
            match action {
                SprintAction::List => {
                    if sprint_board.config.kanban.sprints.is_empty() {
                        writeln!(out, "No sprints found.")?;
                    } else {
                        writeln!(
                            out,
                            "{:<6} {:<20} {:<12} {:<12}",
                            "Number", "Name", "Start", "End"
                        )?;
                        writeln!(
                            out,
                            "------------------------------------------------------"
                        )?;
                        for sprint in &sprint_board.config.kanban.sprints {
                            writeln!(
                                out,
                                "{:<6} {:<20} {:<12} {:<12}",
                                sprint.number, sprint.name, sprint.start_date, sprint.end_date
                            )?;
                        }
                    }
                }
                SprintAction::Current => {
                    if let Some(current) = sprint_board.config.get_current_sprint() {
                        writeln!(
                            out,
                            "Current Sprint: {} - {} ({} to {})",
                            current.number, current.name, current.start_date, current.end_date
                        )?;
                    } else {
                        writeln!(out, "No active sprint for today.")?;
                    }
                }
                SprintAction::Add {
                    number,
                    name,
                    start,
                    end,
                } => {
                    let next_number = if let Some(n) = number {
                        if sprint_board
                            .config
                            .kanban
                            .sprints
                            .iter()
                            .any(|s| s.number == n)
                        {
                            anyhow::bail!("Sprint with number {} already exists.", n);
                        }
                        n
                    } else {
                        sprint_board
                            .config
                            .kanban
                            .sprints
                            .iter()
                            .map(|s| s.number)
                            .max()
                            .map(|n| n + 1)
                            .unwrap_or(1)
                    };

                    sprint_board
                        .config
                        .kanban
                        .sprints
                        .push(crate::model::config::Sprint {
                            number: next_number,
                            name: name.clone(),
                            start_date: start.clone(),
                            end_date: end.clone(),
                        });
                    sprint_board.config.kanban.sprints.sort_by_key(|s| s.number);
                    sprint_board.config.write(&root_path)?;
                    writeln!(out, "Added sprint: {} - {}", next_number, name)?;
                }
                SprintAction::Update {
                    number,
                    name,
                    start,
                    end,
                } => {
                    let sprint = sprint_board
                        .config
                        .kanban
                        .sprints
                        .iter_mut()
                        .find(|s| s.number == number)
                        .ok_or_else(|| anyhow::anyhow!("Sprint {} not found.", number))?;
                    if let Some(ref n) = name {
                        sprint.name = n.clone();
                    }
                    if let Some(ref s) = start {
                        sprint.start_date = s.clone();
                    }
                    if let Some(ref e) = end {
                        sprint.end_date = e.clone();
                    }
                    sprint_board.config.write(&root_path)?;
                    writeln!(out, "Updated sprint {}.", number)?;
                }
                SprintAction::Remove { number } => {
                    let initial_len = sprint_board.config.kanban.sprints.len();
                    sprint_board
                        .config
                        .kanban
                        .sprints
                        .retain(|s| s.number != number);
                    if sprint_board.config.kanban.sprints.len() == initial_len {
                        anyhow::bail!("Sprint {} not found.", number);
                    }
                    sprint_board.config.write(&root_path)?;
                    writeln!(out, "Removed sprint {}.", number)?;
                }
            }
        }
    }

    Ok(())
}

/// Resolves the root path and dispatches to GUI or CLI command handler.
/// Root path priority: --root flag > KANBAN_HOME env var > ~/Kanban default.
fn run_cli(args: CliArgs, out: &mut dyn std::io::Write) -> anyhow::Result<()> {
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
        handle_command(root_path, command, out)
    } else {
        run_gui(root_path)
    }
}

pub(crate) fn into_slint_summary(
    summary: crate::model::stats::BoardSummary,
) -> crate::BoardSummaryStr {
    let slint_queues: Vec<crate::QueueStatStr> = summary
        .queues
        .into_iter()
        .map(|q| {
            let limit_str = q
                .limit
                .map(|l| l.to_string())
                .unwrap_or_else(|| "-".to_string());
            let usage_str = if let Some(limit) = q.limit {
                if limit > 0 {
                    format!("{}%", (q.count * 100) / limit)
                } else {
                    "-".to_string()
                }
            } else {
                "-".to_string()
            };
            crate::QueueStatStr {
                name: q.name.into(),
                count: q.count as i32,
                limit: limit_str.into(),
                usage: usage_str.into(),
            }
        })
        .collect();

    let slint_users: Vec<crate::UserStatStr> = summary
        .users
        .into_iter()
        .map(|u| crate::UserStatStr {
            name: u.name.into(),
            count: u.count as i32,
        })
        .collect();

    let lead_time_str = summary
        .avg_lead_time_days
        .map(|d| format!("{:.1} days", d))
        .unwrap_or_else(|| "-".to_string());
    let cycle_time_str = summary
        .avg_cycle_time_days
        .map(|d| format!("{:.1} days", d))
        .unwrap_or_else(|| "-".to_string());

    crate::BoardSummaryStr {
        total_tickets: summary.total_tickets as i32,
        unassigned_tickets: summary.unassigned_tickets as i32,
        queues: std::rc::Rc::new(slint::VecModel::from(slint_queues)).into(),
        users: std::rc::Rc::new(slint::VecModel::from(slint_users)).into(),
        avg_lead_time: lead_time_str.into(),
        avg_cycle_time: cycle_time_str.into(),
    }
}

fn main() -> anyhow::Result<()> {
    use clap::Parser;
    let args = CliArgs::parse();
    run_cli(args, &mut std::io::stdout())
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
