use chrono::{Duration, Utc};
use rand::Rng;
use slint_kanban::model::action::ActionPayload;
use slint_kanban::model::config::Sprint;
use slint_kanban::model::{Board, Ticket};
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: simulate_work <path_to_board>");
        std::process::exit(1);
    }

    let board_path = PathBuf::from(&args[1]);
    if board_path.exists() {
        println!("Cleaning up existing board at {:?}", board_path);
        std::fs::remove_dir_all(&board_path)?;
    }

    println!("Initializing board at {:?}", board_path);
    Board::ensure_initialized(&board_path)?;
    let board = Board::load(board_path.clone())?;

    let mut config = board.config.clone();

    // Create some sprints in the past
    let now = Utc::now();
    for i in 1..=4 {
        let end = now - Duration::days((14 * (4 - i as i32)) as i64);
        let start = end - Duration::days(13);
        config.kanban.sprints.push(Sprint {
            number: i as u32,
            name: format!("Sprint {}", i),
            start_date: start.format("%Y-%m-%d").to_string(),
            end_date: end.format("%Y-%m-%d").to_string(),
        });
    }
    config.write(&board_path)?;

    let users = vec!["Alice", "Bob", "Charlie"];
    let queues = vec![
        "1. Incoming",
        "2. To Do",
        "3. Doing",
        "4. Reviewing",
        "5. Testing",
        "6. Done",
    ];

    let mut rng = rand::thread_rng();
    let start_sim = now - Duration::days(60);

    let mut active_tickets = Vec::new();

    for day in 0..60 {
        let current_date = start_sim + Duration::days(day);
        let date_str = current_date.format("%Y-%m-%d %H:%M:%S").to_string();
        let ts_rfc = current_date.to_rfc3339_opts(chrono::SecondsFormat::Secs, true);

        println!("Simulating Day {}: {}", day, date_str);

        // Randomly add new tickets
        if rng.gen_bool(0.7) {
            let num_new = rng.gen_range(1..3);
            for _ in 0..num_new {
                let title = format!("Task from day {}", day);
                let points = rng.gen_range(1..11);
                let assigned_to = users[rng.gen_range(0..users.len())];

                // We bypass board.create_ticket to handle past timestamps manually
                // Or we can just use it and then patch the files, but it's easier to write manually
                let ticket_id = generate_id();
                let ticket_dir = board.tickets_path.join(&ticket_id);
                std::fs::create_dir_all(&ticket_dir)?;

                let ticket = Ticket {
                    id: ticket_id.clone(),
                    title: title.clone(),
                    created_at: date_str.clone(),
                    updated_at: date_str.clone(),
                    description: "Simulated task content".to_string(),
                    assigned_to: assigned_to.to_string(),
                    author: "SystemSim".to_string(),
                    points,
                    attachment_count: 0,
                    comments: vec![],
                };
                ticket.save(&board.tickets_path.join(&ticket_id))?;

                // Link to Incoming
                let queue_path = board.queues_path.join("1. Incoming");
                #[cfg(unix)]
                std::os::unix::fs::symlink(&ticket_dir, queue_path.join(&ticket_id))?;

                append_log(
                    &board_path,
                    ActionPayload::CreateTicket {
                        id: ticket_id.clone(),
                        title: title.clone(),
                        queue: "1. Incoming".to_string(),
                        points,
                    },
                    &format!("Created ticket {}", ticket_id),
                    &ts_rfc,
                    "SystemSim",
                );

                active_tickets.push((ticket_id, "1. Incoming".to_string(), points, current_date));
            }
        }

        // Simulating progress
        let mut i = 0;
        while i < active_tickets.len() {
            let (id, current_q, points, last_move) = active_tickets[i].clone();

            // Increase move chance and lower threshold for faster progress in simulation
            let days_since_move = (current_date - last_move).num_days();
            let move_threshold = (points as f64 * 0.3).max(0.0);

            if days_since_move >= move_threshold as i64 && rng.gen_bool(0.6) {
                let current_idx = queues.iter().position(|&q| q == current_q).unwrap();
                if current_idx < queues.len() - 1 {
                    let next_q = queues[current_idx + 1].to_string();

                    // Move symlink
                    let old_path = board.queues_path.join(&current_q).join(&id);
                    let new_path = board.queues_path.join(&next_q).join(&id);
                    std::fs::remove_file(old_path)?;
                    #[cfg(unix)]
                    std::os::unix::fs::symlink(board.tickets_path.join(&id), new_path)?;

                    append_log(
                        &board_path,
                        ActionPayload::ChangeStatus {
                            id: id.clone(),
                            from: current_q.clone(),
                            to: next_q.clone(),
                        },
                        &format!("Moved ticket {} to {}", id, next_q),
                        &ts_rfc,
                        "SystemSim",
                    );

                    active_tickets[i] = (id.clone(), next_q.clone(), points, current_date);

                    if next_q == "6. Done" {
                        active_tickets.remove(i);
                        continue;
                    }
                }
            }
            i += 1;
        }
    }

    println!("Simulation complete. Board created at {:?}", board_path);
    Ok(())
}

fn generate_id() -> String {
    let mut rng = rand::thread_rng();
    (0..6)
        .map(|_| {
            let chars = b"abcdefghijklmnopqrstuvwxyz0123456789";
            let idx = rng.gen_range(0..chars.len());
            chars[idx] as char
        })
        .collect()
}

fn append_log(
    board_path: &std::path::Path,
    payload: ActionPayload,
    description: &str,
    timestamp: &str,
    user: &str,
) {
    let logs_path = board_path.join("logs");
    let _ = std::fs::create_dir_all(&logs_path);
    let log_file = logs_path.join(format!("log_{}_sim.md", user));

    let exists = log_file.exists();
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_file)
        .unwrap();

    if !exists {
        use std::io::Write;
        writeln!(&mut file, "# User Activity Log: {}\n\n| **Date** | **Action** | **Action description** | **JSON** |\n| :--- | :--- | :--- | :--- |", user).unwrap();
    }

    let json = serde_json::to_string(&payload).unwrap();
    use std::io::Write;
    writeln!(
        &mut file,
        "| {} | {} | {} | `{}` |",
        timestamp,
        payload.to_string(),
        description,
        json
    )
    .unwrap();
}
