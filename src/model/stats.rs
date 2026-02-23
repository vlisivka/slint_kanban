//! stats.rs
//!
//! Purpose: Provides statistics and analytics functionality for the Kanban board.
//! Includes: Snapshot-based metrics.
//! Constraints: Should only read from existing state or log files.

use crate::model::action::ActionPayload;
use crate::model::Board;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueueStat {
    pub name: String,
    pub count: usize,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserStat {
    pub name: String,
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BoardSummary {
    pub total_tickets: usize,
    pub unassigned_tickets: usize,
    pub queues: Vec<QueueStat>,
    pub users: Vec<UserStat>,
    pub avg_lead_time_days: Option<f64>,
    pub avg_cycle_time_days: Option<f64>,
    pub completion_rate: Option<f64>,
    pub sprint_completion_rate: Option<f64>,
}

pub fn get_board_summary(board: &Board) -> BoardSummary {
    let mut total_tickets = 0;
    let mut unassigned_tickets = 0;
    let mut queues = Vec::new();
    let mut user_counts: HashMap<String, usize> = HashMap::new();

    // Fill configured users to ensure they appear in the stats even with 0
    for user in &board.config.kanban.users {
        if user != "<unassigned>" {
            user_counts.insert(user.clone(), 0);
        }
    }

    // Default workflow if not configured
    let default_workflow = crate::model::config::Workflow {
        start_queues: vec!["3. Doing".to_string(), "Doing".to_string()],
        done_queues: vec![
            "6. Done".to_string(),
            "Done".to_string(),
            "7. Archive".to_string(),
        ],
    };

    let workflow = board
        .config
        .kanban
        .workflows
        .get("default")
        .unwrap_or(&default_workflow);

    let done_queues = &workflow.done_queues;
    let start_queues = &workflow.start_queues;

    let mut done_tickets_count = 0;
    let mut archived_tickets_count = 0;

    for queue in &board.queues {
        let count = queue.tickets.len();
        total_tickets += count;

        if done_queues.contains(&queue.name) {
            // Note: 7. Archive is in done_queues by default, but we should distinguish it
            // for the completion rate formula: (Done) / (Total - Archive)
            if queue.name.to_lowercase().contains("archive") {
                archived_tickets_count += count;
            } else {
                done_tickets_count += count;
            }
        }

        queues.push(QueueStat {
            name: queue.name.clone(),
            count,
            limit: queue.limit,
        });

        for ticket in &queue.tickets {
            if ticket.assigned_to.is_empty() {
                unassigned_tickets += 1;
            } else {
                *user_counts.entry(ticket.assigned_to.clone()).or_insert(0) += 1;
            }
        }
    }

    let completion_rate = if total_tickets > archived_tickets_count {
        Some((done_tickets_count as f64) / ((total_tickets - archived_tickets_count) as f64) * 100.0)
    } else {
        None
    };

    let mut users: Vec<UserStat> = user_counts
        .into_iter()
        .map(|(name, count)| UserStat { name, count })
        .collect();

    // Sort by name for deterministic order
    users.sort_by(|a, b| a.name.cmp(&b.name));

    let mut avg_lead_time_days = None;
    let mut avg_cycle_time_days = None;
    let mut sprint_completion_rate = None;

    if let Some(parent) = board.tickets_path.parent() {
        if let Ok(all_logs) = load_all_logs(parent) {
            let mut lead_times = Vec::new();
            let mut cycle_times = Vec::new();

            let mut ticket_ids = std::collections::HashSet::new();
            for q in &board.queues {
                for t in &q.tickets {
                    ticket_ids.insert(t.id.clone());
                }
            }

            for id in ticket_ids {
                if let Some(lt) = calculate_lead_time(&id, &all_logs, done_queues) {
                    lead_times.push(lt);
                }
                if let Some(ct) = calculate_cycle_time(&id, &all_logs, start_queues, done_queues)
                {
                    cycle_times.push(ct);
                }
            }

            if !lead_times.is_empty() {
                let sum: i64 = lead_times.iter().sum();
                avg_lead_time_days = Some((sum as f64) / 86400.0 / (lead_times.len() as f64));
            }
            if !cycle_times.is_empty() {
                let sum: i64 = cycle_times.iter().sum();
                avg_cycle_time_days = Some((sum as f64) / 86400.0 / (cycle_times.len() as f64));
            }

            // Calculate Sprint Completion Rate if there is an active sprint
            if let Some(sprint) = board.config.get_current_sprint(None) {
                let mut active_in_sprint = std::collections::HashSet::new();
                let mut completed_in_sprint = std::collections::HashSet::new();

                for entry in &all_logs {
                    if entry.timestamp >= sprint.start_date && entry.timestamp <= sprint.end_date {
                        match &entry.payload {
                            ActionPayload::CreateTicket { id, .. } => {
                                active_in_sprint.insert(id.clone());
                            }
                            ActionPayload::ChangeStatus { id, to, .. } => {
                                active_in_sprint.insert(id.clone());
                                if done_queues.contains(to) {
                                    completed_in_sprint.insert(id.clone());
                                }
                            }
                            _ => {}
                        }
                    }
                }

                if !active_in_sprint.is_empty() {
                    sprint_completion_rate = Some(
                        (completed_in_sprint.len() as f64) / (active_in_sprint.len() as f64) * 100.0,
                    );
                }
            }
        }
    }

    BoardSummary {
        total_tickets,
        unassigned_tickets,
        queues,
        users,
        avg_lead_time_days,
        avg_cycle_time_days,
        completion_rate,
        sprint_completion_rate,
    }
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: String,
    #[allow(dead_code)]
    pub action: String,
    #[allow(dead_code)]
    pub description: String,
    pub payload: ActionPayload,
}

pub fn load_all_logs(root_path: &Path) -> anyhow::Result<Vec<LogEntry>> {
    let logs_dir = root_path.join("logs");
    let mut all_entries = Vec::new();

    if !logs_dir.exists() {
        return Ok(all_entries);
    }

    for entry in std::fs::read_dir(logs_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().is_some_and(|ext| ext == "md") {
            if let Ok(mut entries) = parse_log_file(&path) {
                all_entries.append(&mut entries);
            }
        }
    }

    // Sort by timestamp
    all_entries.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

    Ok(all_entries)
}

pub fn parse_log_file(path: &Path) -> anyhow::Result<Vec<LogEntry>> {
    let content = std::fs::read_to_string(path)?;
    let mut entries = Vec::new();

    for line in content.lines() {
        if !line.starts_with('|') || line.contains("**Date**") || line.contains(":---") {
            continue;
        }

        // Format: | timestamp | action | description | `json` |
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() < 5 {
            continue;
        }

        let timestamp = parts[1].trim().to_string();
        let action = parts[2].trim().to_string();
        let description = parts[3].trim().to_string();
        let json_part = parts[4].trim();

        // Extract JSON from backticks
        let json = if json_part.starts_with('`') && json_part.ends_with('`') {
            &json_part[1..json_part.len() - 1]
        } else {
            json_part
        };

        if let Ok(payload) = serde_json::from_str::<ActionPayload>(json) {
            entries.push(LogEntry {
                timestamp,
                action,
                description,
                payload,
            });
        }
    }

    Ok(entries)
}

#[allow(dead_code)]
pub struct TicketLifecycle {
    pub ticket_id: String,
    pub created_at: String,
    pub completed_at: Option<String>,
    pub time_spent_per_queue: HashMap<String, i64>, // seconds
}

#[allow(dead_code)]
pub fn get_ticket_lifecycle(ticket_id: &str, all_entries: &[LogEntry]) -> TicketLifecycle {
    let mut lifecycle = TicketLifecycle {
        ticket_id: ticket_id.to_string(),
        created_at: String::new(),
        completed_at: None,
        time_spent_per_queue: HashMap::new(),
    };

    let mut current_queue: Option<String> = None;
    let mut last_timestamp: Option<chrono::DateTime<chrono::FixedOffset>> = None;

    let ticket_entries = all_entries.iter().filter(|e| match &e.payload {
        ActionPayload::CreateTicket { id, .. } => id == ticket_id,
        ActionPayload::UpdateTicket { id } => id == ticket_id,
        ActionPayload::ChangeStatus { id, .. } => id == ticket_id,
        ActionPayload::AddComment { id, .. } => id == ticket_id,
        ActionPayload::AssignTicket { id, .. } => id == ticket_id,
        ActionPayload::AttachFile { id, .. } => id == ticket_id,
    });

    for entry in ticket_entries {
        let ts = if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&entry.timestamp) {
            dt
        } else {
            continue;
        };

        if let Some(last_ts) = last_timestamp {
            if let Some(q) = &current_queue {
                let duration = ts.signed_duration_since(last_ts).num_seconds();
                *lifecycle.time_spent_per_queue.entry(q.clone()).or_insert(0) += duration;
            }
        }

        match &entry.payload {
            ActionPayload::CreateTicket { queue, .. } => {
                lifecycle.created_at = entry.timestamp.clone();
                current_queue = Some(queue.clone());
            }
            ActionPayload::ChangeStatus { to, .. } => {
                current_queue = Some(to.clone());
            }
            _ => {}
        }
        last_timestamp = Some(ts);
    }

    lifecycle
}

pub fn calculate_lead_time(
    ticket_id: &str,
    all_entries: &[LogEntry],
    done_queues: &[String],
) -> Option<i64> {
    let mut created_at: Option<chrono::DateTime<chrono::FixedOffset>> = None;
    let mut completed_at: Option<chrono::DateTime<chrono::FixedOffset>> = None;

    for entry in all_entries {
        match &entry.payload {
            ActionPayload::CreateTicket { id, .. } if id == ticket_id => {
                created_at = chrono::DateTime::parse_from_rfc3339(&entry.timestamp).ok();
            }
            ActionPayload::ChangeStatus { id, to, .. } if id == ticket_id => {
                if done_queues.contains(to) && completed_at.is_none() {
                    completed_at = chrono::DateTime::parse_from_rfc3339(&entry.timestamp).ok();
                } else if !done_queues.contains(to) {
                    completed_at = None; // Moved out of done
                }
            }
            _ => {}
        }
    }

    if let (Some(start), Some(end)) = (created_at, completed_at) {
        Some(end.signed_duration_since(start).num_seconds())
    } else {
        None
    }
}

pub fn calculate_cycle_time(
    ticket_id: &str,
    all_entries: &[LogEntry],
    start_queues: &[String],
    done_queues: &[String],
) -> Option<i64> {
    let mut started_at: Option<chrono::DateTime<chrono::FixedOffset>> = None;
    let mut completed_at: Option<chrono::DateTime<chrono::FixedOffset>> = None;

    for entry in all_entries {
        match &entry.payload {
            ActionPayload::ChangeStatus { id, to, .. } if id == ticket_id => {
                if start_queues.contains(to) && started_at.is_none() {
                    started_at = chrono::DateTime::parse_from_rfc3339(&entry.timestamp).ok();
                }

                if done_queues.contains(to) && completed_at.is_none() {
                    completed_at = chrono::DateTime::parse_from_rfc3339(&entry.timestamp).ok();
                } else if !done_queues.contains(to) {
                    completed_at = None; // Moved out of done
                }
            }
            _ => {}
        }
    }

    if let (Some(start), Some(end)) = (started_at, completed_at) {
        Some(end.signed_duration_since(start).num_seconds())
    } else {
        None
    }
}
