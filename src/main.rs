mod model;

use model::Board;
use notify::{RecursiveMode, Watcher};
use slint::{ComponentHandle, SharedString, VecModel};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

slint::include_modules!();

static RELOAD_COUNT: AtomicU64 = AtomicU64::new(0);

fn reload_board(ui: &App, root_path: &Path) -> anyhow::Result<()> {
    let count = RELOAD_COUNT.fetch_add(1, Ordering::SeqCst);
    println!("Reloading board #{}...", count + 1);

    let board = Board::load(root_path.to_path_buf())?;

    // Convert Board to Slint Models
    let mut slint_queues: Vec<QueueStr> = vec![];

    for queue in board.queues {
        let mut slint_tickets: Vec<TicketStr> = vec![];
        for ticket in queue.tickets {
            let snippet = ticket.description.lines().next().unwrap_or("").to_string();
            slint_tickets.push(TicketStr {
                id: SharedString::from(ticket.id),
                title: SharedString::from(ticket.title),
                description: SharedString::from(ticket.description),
                snippet: SharedString::from(snippet),
                created_at: SharedString::from(ticket.created_at),
                updated_at: SharedString::from(ticket.updated_at),
            });
        }

        let tickets_model = Rc::new(VecModel::from(slint_tickets));

        slint_queues.push(QueueStr {
            id: SharedString::from(queue.id),
            name: SharedString::from(queue.name),
            tickets: tickets_model.into(),
        });
    }

    let queues_model = Rc::new(VecModel::from(slint_queues));
    ui.set_board_queues(queues_model.into());
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let ui = App::new()?;

    // Determine the root path: from command-line argument or default to ~/Kanban
    let args: Vec<String> = std::env::args().collect();
    let root_path = if args.len() > 1 {
        PathBuf::from(&args[1])
    } else {
        let home_dir = std::env::var("HOME").expect("HOME directory not set");
        PathBuf::from(home_dir).join("Kanban")
    };

    println!("Using Kanban root: {:?}", root_path);

    // Ensure directory exists and default queues are created for initial run
    Board::ensure_initialized(&root_path)?;

    // Initial load
    reload_board(&ui, &root_path)?;

    let ui_handle = ui.as_weak();
    let watcher_root = root_path.clone();

    // Set up callbacks
    let move_root = root_path.clone();
    ui.on_move_ticket(move |ticket_id, source_id, target_id| {
        let board = match Board::load(move_root.clone()) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("Error loading board for move: {:?}", e);
                return;
            }
        };

        let resolved_target_id = if let Some(idx_str) = target_id.strip_prefix("index:") {
            // Slint might send a float string like "1.36...". Parse as f64 and floor it.
            if let Ok(idx_f) = idx_str.parse::<f64>() {
                let idx = idx_f.floor() as usize;
                if idx < board.queues.len() {
                    board.queues[idx].id.clone()
                } else {
                    board
                        .queues
                        .last()
                        .map(|q| q.id.clone())
                        .unwrap_or_default()
                }
            } else {
                target_id.to_string()
            }
        } else {
            target_id.to_string()
        };

        if source_id == resolved_target_id {
            return; // No move needed
        }

        println!(
            "Moving ticket {} from {} to {}",
            ticket_id, source_id, resolved_target_id
        );
        if let Err(e) = board.move_ticket(&ticket_id, &source_id, &resolved_target_id) {
            eprintln!("Error moving ticket: {:?}", e);
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

                    // Consume any other events that occur within a small time window (debounce)
                    let debounce_duration = Duration::from_millis(200);
                    while rx.recv_timeout(debounce_duration).is_ok() {
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

    let create_root = root_path.clone();
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
        }
    });

    ui.run()?;
    Ok(())
}
