pub mod cli;
pub mod controller;
pub mod model;

slint::include_modules!();

use model::{Board, Queue, Ticket};
use slint::SharedString;

/// Converts a domain Ticket into the Slint-generated TicketStr for UI binding.
/// `snippet` is the first line of the description, shown on the card preview.
pub fn into_slint_ticket(ticket: &Ticket, board: &Board) -> TicketStr {
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
                id: c.id.clone().into(),
                author: c.metadata.author.clone().into(),
                created_at: c.metadata.created_at.clone().into(),
                updated_at: c.metadata.updated_at.clone().into(),
                content: c.content.clone().into(),
                references: std::rc::Rc::new(slint::VecModel::from(crefs)).into(),
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
        id: ticket.id.clone().into(),
        title: ticket.title.clone().into(),
        description: ticket.description.clone().into(),
        snippet: snippet.into(),
        created_at: ticket.created_at.clone().into(),
        updated_at: ticket.updated_at.clone().into(),
        assigned_to: ticket.assigned_to.clone().into(),
        author: ticket.author.clone().into(),
        references: std::rc::Rc::new(slint::VecModel::from(refs)).into(),
        comments: std::rc::Rc::new(slint::VecModel::from(slint_comments)).into(),
        attachment_count,
        points: ticket.points as i32,
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
        let mut filtered_tickets: Vec<&Ticket> = queue
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

        let total_points: i32 = slint_tickets.iter().map(|t| t.points).sum();
        let tickets_model = std::rc::Rc::new(slint::VecModel::from(slint_tickets));

        slint_queues.push(QueueStr {
            id: slint::SharedString::from(&queue.id),
            name: slint::SharedString::from(&queue.name),
            tickets: tickets_model.into(),
            limit,
            ticket_count,
            total_points,
            visible: queue.visible,
        });
    }

    let queues_model = std::rc::Rc::new(slint::VecModel::from(slint_queues));
    ui.set_board_queues(queues_model.into());
}

pub fn into_slint_queue(queue: &Queue, board: &Board) -> QueueStr {
    let tickets: Vec<TicketStr> = queue
        .tickets
        .iter()
        .map(|t| into_slint_ticket(t, board))
        .collect();

    QueueStr {
        id: queue.id.clone().into(),
        name: queue.name.clone().into(),
        tickets: std::rc::Rc::new(slint::VecModel::from(tickets)).into(),
        limit: queue.limit.map(|l| l as i32).unwrap_or(-1),
        ticket_count: queue.tickets.len() as i32,
        total_points: queue.tickets.iter().map(|t| t.points).sum::<u32>() as i32,
        visible: queue.visible,
    }
}

pub fn into_slint_summary(summary: &model::stats::BoardSummary) -> BoardSummaryStr {
    let slint_queues: Vec<QueueStatStr> = summary
        .queues
        .iter()
        .map(|qs| QueueStatStr {
            name: qs.name.clone().into(),
            count: qs.count as i32,
            limit: qs
                .limit
                .map(|l| l.to_string())
                .unwrap_or("-".to_string())
                .into(),
            usage: if let Some(l) = qs.limit {
                format!("{:.0}%", (qs.count as f64 / l as f64) * 100.0).into()
            } else {
                "-".into()
            },
        })
        .collect();

    let slint_users: Vec<UserStatStr> = summary
        .users
        .iter()
        .map(|us| UserStatStr {
            name: us.name.clone().into(),
            count: us.count as i32,
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
    let completion_rate_str = summary
        .completion_rate
        .map(|r| format!("{:.1}%", r))
        .unwrap_or_else(|| "-".to_string());
    let sprint_completion_rate_str = summary
        .sprint_completion_rate
        .map(|r| format!("{:.1}%", r))
        .unwrap_or_else(|| "-".to_string());

    let points_completion_rate_str = if summary.total_points > 0 {
        format!(
            "{:.1}%",
            (summary.total_done_points as f64 / summary.total_points as f64) * 100.0
        )
    } else {
        "-".to_string()
    };

    let slint_trend: Vec<TrendPointStr> = summary
        .trend
        .iter()
        .map(|tp| TrendPointStr {
            timestamp: if tp.timestamp.len() >= 10 {
                tp.timestamp[5..10].to_string().into()
            } else {
                tp.timestamp.clone().into()
            },
            total_tickets: tp.total_tickets as i32,
            done_tickets: tp.done_tickets as i32,
            total_points: tp.total_points as i32,
            done_points: tp.done_points as i32,
        })
        .collect();

    BoardSummaryStr {
        total_tickets: summary.total_tickets as i32,
        unassigned_tickets: summary.unassigned_tickets as i32,
        queues: std::rc::Rc::new(slint::VecModel::from(slint_queues)).into(),
        users: std::rc::Rc::new(slint::VecModel::from(slint_users)).into(),
        total_points: summary.total_points as i32,
        total_done_points: summary.total_done_points as i32,
        points_completion_rate: points_completion_rate_str.into(),
        avg_lead_time: lead_time_str.into(),
        avg_cycle_time: cycle_time_str.into(),
        completion_rate: completion_rate_str.into(),
        sprint_completion_rate: sprint_completion_rate_str.into(),
        f_completion_rate: summary.completion_rate.unwrap_or(0.0) as f32,
        f_points_completion_rate: if summary.total_points > 0 {
            (summary.total_done_points as f32 / summary.total_points as f32) * 100.0
        } else {
            0.0
        },
        f_sprint_completion_rate: summary.sprint_completion_rate.unwrap_or(0.0) as f32,
        trend: std::rc::Rc::new(slint::VecModel::from(slint_trend)).into(),
    }
}
