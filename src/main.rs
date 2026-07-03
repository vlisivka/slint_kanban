//! main.rs
//!
//! Purpose: Main entry point for the Slint Kanban application. Orchestrates UI and Backend.
//! Includes: UI event handlers, CLI command dispatch, reloading logic, type conversions,
//!           and NO_COLOR support (https://no-color.org/).
//! Constraints: Business logic should be in the model module, not here.

use notify::{RecursiveMode, Watcher};
use slint::ComponentHandle;
use slint_kanban::cli::{CliArgs, Commands, SprintAction};
use slint_kanban::controller::AppController;
use slint_kanban::model::Board;
use slint_kanban::*;
use std::path::PathBuf;
use std::sync::Arc;
use tr::tr;

/// Pushes the full board state into the UI, applying search/date/user filters.
/// Called on every reload (initial load, file watcher event, or filter change).
fn run_gui(root_path: PathBuf, admin: bool) -> anyhow::Result<()> {
    let ui = App::new()?;
    ui.set_is_admin(admin);
    let controller = Arc::new(AppController::new(ui.as_weak(), root_path.clone()));

    ui.set_board_queues(controller.board_queues_model().into());

    println!("{}", tr!("Using Kanban root: {}", root_path.display()));

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
            eprintln!("{}", tr!("Failed to watch directory: {}", e));
            return;
        }

        // Also watch user config file if possible
        if let Some(user_path) = slint_kanban::model::Config::user_config_path() {
            if let Some(parent) = user_path.parent() {
                let _ = std::fs::create_dir_all(parent);
                if let Err(e) = watcher.watch(parent, RecursiveMode::NonRecursive) {
                    eprintln!(
                        "{}",
                        tr!("Warning: Failed to watch user config directory: {}", e)
                    );
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

                    // Debounce: wait 500ms then drain any events that arrived
                    // during the sleep, so rapid saves trigger only one reload.
                    std::thread::sleep(Duration::from_millis(500));
                    while rx.try_recv().is_ok() {}

                    let c = c_watcher.clone();
                    eprintln!("{}", tr!("[WATCHER] Change detected, triggering reload..."));
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Err(e) = c.reload() {
                            eprintln!("{}", tr!("Error reloading board: {}", e));
                        }
                    });
                }
                Ok(Err(e)) => eprintln!("{}", tr!("Watch error: {}", e)),
                Err(e) => {
                    eprintln!("{}", tr!("Channel error: {}", e));
                    break;
                }
            }
        }
    });

    ui.run()?;
    Ok(())
}

fn init_callbacks(ui: &App, controller: Arc<AppController>) {
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
    ui.on_create_ticket(move |queue_id, title, description, assigned_to, points| {
        c.handle_create_ticket(
            queue_id.into(),
            title.into(),
            description.into(),
            assigned_to.into(),
            points,
        );
    });

    let c = controller.clone();
    ui.on_update_ticket(move |ticket_id, title, description, assigned_to, points| {
        c.handle_update_ticket(
            ticket_id.into(),
            title.into(),
            description.into(),
            assigned_to.into(),
            points,
        );
    });

    let c = controller.clone();
    ui.on_add_comment(move |ticket_id, content| {
        c.handle_add_comment(ticket_id.into(), content.into());
    });

    let c = controller.clone();
    ui.on_attach_file(move |ticket_id| c.handle_attach_file(ticket_id.into()).into());

    let c = controller.clone();
    ui.on_request_full_ticket(move |ticket_id| c.handle_request_full_ticket(ticket_id.into()));

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
    ui.global::<UserGlobal>()
        .on_toggle_manage_only_mine(move |enabled| {
            c.handle_toggle_manage_only_mine(enabled);
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
    let search_timer = std::rc::Rc::new(std::cell::RefCell::new(slint::Timer::default()));
    ui.on_search_edited(move |_| {
        let timer = search_timer.clone();
        let c = c.clone();
        timer.borrow().stop();
        timer.borrow().start(
            slint::TimerMode::SingleShot,
            std::time::Duration::from_millis(300),
            move || {
                let _ = c.reload();
            },
        );
    });

    let c = controller.clone();
    ui.on_date_filter_changed(move || {
        let _ = c.reload();
    });

    let c = controller.clone();
    ui.on_select_history_item(move |_| {
        c.handle_select_history_item();
    });

    // Clipboard integration
    let clipboard = std::sync::Arc::new(std::sync::Mutex::new(arboard::Clipboard::new().ok()));
    let cb_clone = clipboard.clone();
    ui.global::<crate::Clipboard>()
        .on_copy_to_clipboard(move |text| {
            if let Ok(mut cb_lock) = cb_clone.lock() {
                if let Some(cb) = cb_lock.as_mut() {
                    if let Err(e) = cb.set_text(text.to_string()) {
                        eprintln!("Failed to copy to clipboard: {:?}", e);
                    } else {
                        println!("Copied to clipboard: {}", text);
                    }
                } else {
                    eprintln!("Clipboard not available");
                }
            }
        });

    let c = controller.clone();
    ui.on_request_admin_data(move || {
        c.handle_request_admin_data();
    });

    let c = controller.clone();
    ui.on_save_board_readme(move |content| {
        c.handle_save_board_readme(content.into());
    });

    let c = controller.clone();
    ui.on_admin_add_user(move |name| {
        c.handle_add_user(name.into());
    });

    let c = controller.clone();
    ui.on_admin_remove_user(move |name| {
        c.handle_remove_user(name.into());
    });

    let c = controller.clone();
    ui.on_admin_add_queue(move |name| {
        c.handle_add_queue(name.into());
    });

    let c = controller.clone();
    ui.on_admin_rename_queue(move |id, name| {
        c.handle_rename_queue(id.into(), name.into());
    });

    let c = controller.clone();
    ui.on_admin_delete_queue(move |id| {
        c.handle_delete_queue(id.into());
    });

    let c = controller.clone();
    ui.on_admin_add_sprint(move |name, start, end| {
        c.handle_add_sprint(name.into(), start.into(), end.into());
    });

    let c = controller.clone();
    ui.on_admin_update_sprint(move |number, name, start, end| {
        c.handle_update_sprint(number, name.into(), start.into(), end.into());
    });

    let c = controller.clone();
    ui.on_admin_remove_sprint(move |number| {
        c.handle_remove_sprint(number);
    });
}

fn handle_command(
    root_path: std::path::PathBuf,
    command: Commands,
    admin: bool,
    out: &mut dyn std::io::Write,
) -> anyhow::Result<()> {
    let board = Board::load(root_path.clone())?;

    match command {
        Commands::Add {
            title,
            description,
            description_file,
            queue,
            assign_to,
            points,
        } => {
            // Resolve description body: --description-file (file or stdin) overrides --description.
            // If both provided, concatenate description + "\n" + file/stdin content.
            let body = match (&description_file, &description) {
                (Some(path), _) if path == "-" => {
                    // Read from stdin
                    let mut input = String::new();
                    std::io::Read::read_to_string(&mut std::io::stdin(), &mut input)?;
                    input
                }
                (Some(path), _) => {
                    // Read from file
                    let file_content = std::fs::read_to_string(path).map_err(|e| {
                        anyhow::anyhow!("Failed to read description file '{}': {}", path, e)
                    })?;
                    match description {
                        Some(desc) => format!("{}\n{}", desc, file_content),
                        None => file_content,
                    }
                }
                (None, Some(desc)) => desc.clone(),
                (None, None) => String::new(),
            };

            writeln!(
                out,
                "{}",
                tr!(
                    "Adding ticket: {} to queue: {} with {} points",
                    title,
                    queue,
                    points
                )
            )?;
            let author = board.config.active_user();
            board.create_ticket(&title, &body, &queue, &assign_to, author, points)?;
        }
        Commands::Update {
            id,
            title,
            description,
            assign_to,
            unassign,
            points,
        } => {
            writeln!(out, "{}", tr!("Updating ticket: {}", id))?;
            let ticket = board
                .find_ticket_by_id(&id)
                .ok_or_else(|| anyhow::anyhow!(tr!("Ticket not found: {}", id)))?;

            if !board.can_manage_ticket(ticket, admin) {
                anyhow::bail!(tr!("Access Denied: You can only update tickets assigned to you. Use --admin to bypass."));
            }

            // Fields not provided on CLI are preserved from the existing ticket
            let title = title.unwrap_or(ticket.title.clone());
            let description = description.unwrap_or(ticket.description.clone());
            let assign_to = if unassign {
                "".to_string()
            } else {
                assign_to.unwrap_or(ticket.assigned_to.clone())
            };
            let points = points.unwrap_or(ticket.points);
            board.update_ticket(&id, &title, &description, &assign_to, points)?;
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
                let filtered_tickets: Vec<&slint_kanban::model::Ticket> = queue
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
                            tr!("<unassigned>").to_string()
                        } else {
                            t.assigned_to.clone()
                        };
                        let points_display = if t.points > 0 {
                            tr!(" [{} pts]", t.points)
                        } else {
                            "".to_string()
                        };
                        writeln!(
                            out,
                            "[{}]{} {} (Assigned: {})",
                            t.id, points_display, t.title, user_display
                        )?;
                    }
                }
            }
        }
        Commands::Configure {
            active_user,
            show_only_mine,
            manage_only_mine,
            add_user,
        } => {
            let mut config = board.config.clone();
            if let Some(user) = active_user {
                writeln!(out, "{}", tr!("Setting active user to: {}", user))?;
                config.user.active_user = user;
            }
            if let Some(mine) = show_only_mine {
                writeln!(out, "{}", tr!("Setting show_only_mine to: {}", mine))?;
                config.user.show_only_mine = mine;
            }
            if let Some(manage) = manage_only_mine {
                writeln!(out, "{}", tr!("Setting manage_only_mine to: {}", manage))?;
                config.user.manage_only_mine = manage;
            }
            if let Some(user) = add_user {
                if !config.kanban.users.contains(&user) {
                    writeln!(out, "{}", tr!("Adding user: {}", user))?;
                    config.kanban.users.push(user);
                }
            }
            config.write(&root_path)?;
        }
        Commands::Stats { user, csv } => {
            let mut filtered_board = board.clone();
            if let Some(ref u) = user {
                // Filter tickets by assigned user
                for queue in &mut filtered_board.queues {
                    queue.tickets.retain(|t| t.assigned_to == *u);
                }
                // Filter user list in config so the stats only show the requested user
                filtered_board
                    .config
                    .kanban
                    .users
                    .retain(|user_id| user_id == u);
            }
            let summary = slint_kanban::model::stats::get_board_summary(&filtered_board);

            if csv {
                print_stats_csv(&summary, out)?;
            } else {
                print_stats_human_readable(&summary, out)?;
            }
        }
        Commands::Move { id, queue } => {
            writeln!(out, "{}", tr!("Moving ticket: {} to queue: {}", id, queue))?;
            let ticket = board
                .find_ticket_by_id(&id)
                .ok_or_else(|| anyhow::anyhow!(tr!("Ticket not found: {}", id)))?;

            if !board.can_manage_ticket(ticket, admin) {
                anyhow::bail!(tr!("Access Denied: You can only move tickets assigned to you. Use --admin to bypass."));
            }

            let source_queue = board
                .queues
                .iter()
                .find(|q| q.tickets.iter().any(|t| t.id == id))
                .ok_or_else(|| anyhow::anyhow!(tr!("Ticket not found in any queue: {}", id)))?;
            board.move_ticket(&id, &source_queue.id, &queue)?;
        }
        Commands::Remove { id } => {
            writeln!(out, "{}", tr!("Removing ticket: {}", id))?;
            let ticket = board
                .find_ticket_by_id(&id)
                .ok_or_else(|| anyhow::anyhow!(tr!("Ticket not found: {}", id)))?;

            if !board.can_manage_ticket(ticket, admin) {
                anyhow::bail!(tr!("Access Denied: You can only remove tickets assigned to you. Use --admin to bypass."));
            }

            board.delete_ticket(&id)?;
        }
        Commands::Open { path } => {
            writeln!(out, "{}", tr!("Opening GUI for path: {}", path.display()))?;
            run_gui(path, admin)?;
        }
        Commands::Show { id } => {
            let ticket = board
                .load_full_ticket(&id)
                .map_err(|_| anyhow::anyhow!(tr!("Ticket not found: {}", id)))?;

            let queue_name = board
                .queues
                .iter()
                .find(|q| q.tickets.iter().any(|t| t.id == id))
                .map(|q| q.name.clone())
                .unwrap_or_else(|| tr!("Unknown").to_string());

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
            writeln!(out, "Points:      {}", ticket.points)?;
            writeln!(out, "Status:      {}", queue_name)?;
            writeln!(
                out,
                "Assigned to: {}",
                if ticket.assigned_to.is_empty() {
                    tr!("<unassigned>").to_string()
                } else {
                    ticket.assigned_to.clone()
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
        Commands::Comment {
            id,
            content,
            content_file,
        } => {
            // Resolve comment body: --content-file (file or stdin) overrides --content.
            // If both provided, concatenate content + "\n" + file/stdin content.
            let comment_body = match (&content_file, &content) {
                (Some(path), _) if path == "-" => {
                    // Read from stdin
                    let mut input = String::new();
                    std::io::Read::read_to_string(&mut std::io::stdin(), &mut input)?;
                    match content {
                        Some(desc) => format!("{}\n{}", desc, input),
                        None => input,
                    }
                }
                (Some(path), _) => {
                    // Read from file
                    let file_content = std::fs::read_to_string(path).map_err(|e| {
                        anyhow::anyhow!("Failed to read comment file '{}': {}", path, e)
                    })?;
                    match content {
                        Some(desc) => format!("{}\n{}", desc, file_content),
                        None => file_content,
                    }
                }
                (None, Some(desc)) => desc.clone(),
                (None, None) => String::new(),
            };

            writeln!(out, "{}", tr!("Adding comment to ticket: {}", id))?;
            let author = board.config.active_user();
            board.add_comment(&id, &comment_body, author)?;
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
                anyhow::bail!(tr!("Ticket not found: {}", id));
            }

            if open {
                if !attach_dir.exists() {
                    std::fs::create_dir_all(&attach_dir)?;
                }
                writeln!(
                    out,
                    "{}",
                    tr!("Opening attachments directory: {}", attach_dir.display())
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
                        writeln!(out, "{}", tr!("No attachments found."))?;
                    }
                } else {
                    writeln!(out, "{}", tr!("No attachments found."))?;
                }
            } else if let Some(f) = file {
                let link = board.attach_file(&id, &f)?;
                writeln!(out, "{}", link)?;
            } else {
                anyhow::bail!(tr!(
                    "No action specified. Use --file, --list, --show, or --open."
                ));
            }
        }
        Commands::Sprint { action } => {
            let mut sprint_board = board;
            match action {
                SprintAction::List => {
                    if sprint_board.config.kanban.sprints.is_empty() {
                        writeln!(out, "{}", tr!("No sprints found."))?;
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
                    if let Some(current) = sprint_board.config.get_current_sprint(None) {
                        writeln!(
                            out,
                            "Current Sprint: {} - {} ({} to {})",
                            current.number, current.name, current.start_date, current.end_date
                        )?;
                    } else {
                        writeln!(out, "{}", tr!("No active sprint for today."))?;
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
                            anyhow::bail!(tr!("Sprint with number {} already exists.", n));
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
                        .push(slint_kanban::model::config::Sprint {
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
                        .ok_or_else(|| anyhow::anyhow!(tr!("Sprint {} not found.", number)))?;
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
                        anyhow::bail!(tr!("Sprint {} not found.", number));
                    }
                    sprint_board.config.write(&root_path)?;
                    writeln!(out, "Removed sprint {}.", number)?;
                }
            }
        }
    }

    Ok(())
}

fn print_stats_human_readable(
    summary: &slint_kanban::model::stats::BoardSummary,
    out: &mut dyn std::io::Write,
) -> anyhow::Result<()> {
    writeln!(out, "{}", tr!("== Board Summary =="))?;
    writeln!(out, "{}", tr!("Total tickets: {}", summary.total_tickets))?;
    writeln!(
        out,
        "{}",
        tr!("Unassigned:    {}", summary.unassigned_tickets)
    )?;
    writeln!(
        out,
        "{}",
        tr!(
            "Avg Lead Time: {}",
            summary
                .avg_lead_time_days
                .map(|d| tr!("{:.1} days", d))
                .unwrap_or_else(|| "-".to_string())
        )
    )?;
    writeln!(
        out,
        "{}",
        tr!(
            "Avg Cycle Time: {}",
            summary
                .avg_cycle_time_days
                .map(|d| tr!("{:.1} days", d))
                .unwrap_or_else(|| "-".to_string())
        )
    )?;

    if let Some(rate) = summary.completion_rate {
        writeln!(out, "{}", tr!("Completion Rate: {:.1}%", rate))?;
    }

    writeln!(out, "{}", tr!("Total Points:    {}", summary.total_points))?;
    writeln!(
        out,
        "{}",
        tr!("Done Points:     {}", summary.total_done_points)
    )?;
    if summary.total_points > 0 {
        let p_rate = (summary.total_done_points as f64 / summary.total_points as f64) * 100.0;
        writeln!(out, "{}", tr!("Points Completion Rate: {:.1}%", p_rate))?;
    }

    if let Some(rate) = summary.sprint_completion_rate {
        writeln!(out, "{}", tr!("Sprint Completion: {:.1}% (Tickets)", rate))?;
    }
    writeln!(out)?;

    writeln!(out, "{}", tr!("== Tickets per Queue =="))?;
    writeln!(
        out,
        "{}",
        tr!(
            "{:<20} {:>5} {:>6} {:>5}",
            "Queue",
            "Count",
            "Limit",
            "Usage"
        )
    )?;
    for q in &summary.queues {
        let limit_str = q
            .limit
            .map(|l| l.to_string())
            .unwrap_or_else(|| "-".to_string());
        let usage_str = match q.limit {
            Some(limit) if limit > 0 => format!("{}%", (q.count * 100) / limit),
            _ => "-".to_string(),
        };
        writeln!(
            out,
            "{:<20} {:>5} {:>6} {:>5}",
            q.name, q.count, limit_str, usage_str
        )?;
    }
    writeln!(out)?;
    writeln!(out, "{}", tr!("== Tickets per User =="))?;
    writeln!(out, "{}", tr!("{:<20} {:>5}", "User", "Count"))?;
    for u in &summary.users {
        writeln!(out, "{:<20} {:>5}", u.name, u.count)?;
    }
    writeln!(out)?;

    writeln!(out, "== Trends (Debug) ==")?;
    writeln!(
        out,
        "{:<10} {:>8} {:>8} {:>8} {:>8}",
        "Date", "TotalT", "DoneT", "TotalP", "DoneP"
    )?;
    for tp in &summary.trend {
        writeln!(
            out,
            "{:<10} {:>8} {:>8} {:>8} {:>8}",
            if tp.timestamp.len() >= 10 {
                &tp.timestamp[5..10]
            } else {
                &tp.timestamp
            },
            tp.total_tickets,
            tp.done_tickets,
            tp.total_points,
            tp.done_points
        )?;
    }
    Ok(())
}

fn print_stats_csv(
    summary: &slint_kanban::model::stats::BoardSummary,
    out: &mut dyn std::io::Write,
) -> anyhow::Result<()> {
    writeln!(out, "Type,Category/Date,Metric,Value,Unit")?;

    // Summary
    writeln!(
        out,
        "Summary,General,Total Tickets,{},count",
        summary.total_tickets
    )?;
    writeln!(
        out,
        "Summary,General,Unassigned Tickets,{},count",
        summary.unassigned_tickets
    )?;
    if let Some(d) = summary.avg_lead_time_days {
        writeln!(out, "Summary,General,Avg Lead Time,{:.2},days", d)?;
    }
    if let Some(d) = summary.avg_cycle_time_days {
        writeln!(out, "Summary,General,Avg Cycle Time,{:.2},days", d)?;
    }
    if let Some(r) = summary.completion_rate {
        writeln!(out, "Summary,General,Completion Rate,{:.2},%", r)?;
    }
    writeln!(
        out,
        "Summary,General,Total Points,{},pts",
        summary.total_points
    )?;
    writeln!(
        out,
        "Summary,General,Done Points,{},pts",
        summary.total_done_points
    )?;
    if summary.total_points > 0 {
        let p_rate = (summary.total_done_points as f64 / summary.total_points as f64) * 100.0;
        writeln!(
            out,
            "Summary,General,Points Completion Rate,{:.2},%",
            p_rate
        )?;
    }
    if let Some(r) = summary.sprint_completion_rate {
        writeln!(out, "Summary,General,Sprint Completion Rate,{:.2},%", r)?;
    }

    // Queues
    for q in &summary.queues {
        writeln!(out, "Queue,{},Count,{},tickets", q.name, q.count)?;
        if let Some(l) = q.limit {
            writeln!(out, "Queue,{},Limit,{},tickets", q.name, l)?;
            if l > 0 {
                writeln!(
                    out,
                    "Queue,{},Usage,{:.2},%",
                    q.name,
                    (q.count as f64 / l as f64) * 100.0
                )?;
            }
        }
    }

    // Users
    for u in &summary.users {
        writeln!(out, "User,{},Count,{},tickets", u.name, u.count)?;
    }

    // Trends
    for tp in &summary.trend {
        let date = if tp.timestamp.len() >= 10 {
            &tp.timestamp[0..10]
        } else {
            &tp.timestamp
        };
        writeln!(
            out,
            "Trend,{},Total Tickets,{},tickets",
            date, tp.total_tickets
        )?;
        writeln!(
            out,
            "Trend,{},Done Tickets,{},tickets",
            date, tp.done_tickets
        )?;
        writeln!(out, "Trend,{},Total Points,{},pts", date, tp.total_points)?;
        writeln!(out, "Trend,{},Done Points,{},pts", date, tp.done_points)?;
    }

    // Burndown
    for bp in &summary.burndown {
        writeln!(
            out,
            "Burndown,{},Remaining Points,{},pts",
            bp.date, bp.remaining_points
        )?;
        writeln!(
            out,
            "Burndown,{},Ideal Points,{},pts",
            bp.date, bp.ideal_points
        )?;
    }

    Ok(())
}

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
        handle_command(root_path, command, args.admin, out)
    } else {
        run_gui(root_path, args.admin)
    }
}

fn main() -> anyhow::Result<()> {
    use clap::Parser;

    // Initialize translations for Rust code
    tr::tr_init!("i18n");

    // Initialize translations for Slint
    // Dynamically look for i18n directory relative to the manifest or in standard locations
    // For now, assume it's in the current working directory or relative to the manifest
    slint::init_translations!(concat!(env!("CARGO_MANIFEST_DIR"), "/i18n"));

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
