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
pub struct TrendPoint {
    pub timestamp: String,
    pub total_tickets: usize,
    pub done_tickets: usize,
    pub total_points: u32,
    pub done_points: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BoardSummary {
    pub total_tickets: usize,
    pub unassigned_tickets: usize,
    pub queues: Vec<QueueStat>,
    pub users: Vec<UserStat>,
    pub total_points: u32,
    pub total_done_points: u32,
    pub avg_lead_time_days: Option<f64>,
    pub avg_cycle_time_days: Option<f64>,
    pub completion_rate: Option<f64>,
    pub sprint_completion_rate: Option<f64>,
    pub trend: Vec<TrendPoint>,
}

pub fn get_board_summary(board: &Board) -> BoardSummary {
    let mut total_tickets = 0;
    let mut unassigned_tickets = 0;
    let mut total_points = 0;
    let mut total_done_points = 0;
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
            total_points += ticket.points;
            if done_queues.contains(&queue.name) {
                total_done_points += ticket.points;
            }

            if ticket.assigned_to.is_empty() {
                unassigned_tickets += 1;
            } else {
                *user_counts.entry(ticket.assigned_to.clone()).or_insert(0) += 1;
            }
        }
    }

    let completion_rate = if total_tickets > archived_tickets_count {
        Some(
            (done_tickets_count as f64) / ((total_tickets - archived_tickets_count) as f64) * 100.0,
        )
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
    let mut trend = Vec::new();

    if let Some(parent) = board.tickets_path.parent() {
        if let Ok(all_logs) = load_all_logs(parent) {
            let mut lead_times = Vec::new();
            let mut cycle_times = Vec::new();

            // Map to track ticket timing: ID -> (created, started, completed) in seconds
            let mut ticket_timings: HashMap<String, (Option<i64>, Option<i64>, Option<i64>)> =
                HashMap::new();

            let sprint = board.config.get_current_sprint(None);
            let mut active_in_sprint = std::collections::HashSet::new();
            let mut completed_in_sprint = std::collections::HashSet::new();

            let (sprint_start, sprint_end) = if let Some(s) = &sprint {
                (
                    Some(s.start_date.clone()),
                    Some(format!("{}T23:59:59Z", s.end_date)),
                )
            } else {
                (None, None)
            };

            for entry in &all_logs {
                let ts = chrono::DateTime::parse_from_rfc3339(&entry.timestamp)
                    .ok()
                    .map(|dt| dt.timestamp());

                let in_sprint = if let (Some(start), Some(end)) = (&sprint_start, &sprint_end) {
                    &entry.timestamp >= start && &entry.timestamp <= end
                } else {
                    false
                };

                match &entry.payload {
                    ActionPayload::CreateTicket { id, .. } => {
                        let timing = ticket_timings
                            .entry(id.clone())
                            .or_insert((None, None, None));
                        timing.0 = ts;
                        if in_sprint {
                            active_in_sprint.insert(id.clone());
                        }
                    }
                    ActionPayload::ChangeStatus { id, to, .. } => {
                        let timing = ticket_timings
                            .entry(id.clone())
                            .or_insert((None, None, None));
                        if start_queues.contains(to) && timing.1.is_none() {
                            timing.1 = ts;
                        }
                        if done_queues.contains(to) && timing.2.is_none() {
                            timing.2 = ts;
                        } else if !done_queues.contains(to) {
                            timing.2 = None;
                        }

                        if in_sprint {
                            active_in_sprint.insert(id.clone());
                            if done_queues.contains(to) {
                                completed_in_sprint.insert(id.clone());
                            }
                        }
                    }
                    _ => {}
                }
            }

            // After single pass, collect results
            for (created, started, completed) in ticket_timings.values() {
                if let (Some(c), Some(comp)) = (created, completed) {
                    lead_times.push(comp - c);
                }
                if let (Some(s), Some(comp)) = (started, completed) {
                    cycle_times.push(comp - s);
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

            if !active_in_sprint.is_empty() {
                sprint_completion_rate = Some(
                    (completed_in_sprint.len() as f64) / (active_in_sprint.len() as f64) * 100.0,
                );
            }

            // Trend calculation - reusing pre-loaded logs
            trend = get_trend_data(&all_logs, done_queues, 15);
        }
    }

    BoardSummary {
        total_tickets,
        unassigned_tickets,
        queues,
        users,
        total_points,
        total_done_points,
        avg_lead_time_days,
        avg_cycle_time_days,
        completion_rate,
        sprint_completion_rate,
        trend,
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

    // Sort by timestamp (parsed)
    all_entries.sort_by(|a, b| {
        let da = chrono::DateTime::parse_from_rfc3339(&a.timestamp).ok();
        let db = chrono::DateTime::parse_from_rfc3339(&b.timestamp).ok();
        da.cmp(&db)
    });

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
        ActionPayload::UpdateTicket { id, .. } => id == ticket_id,
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

pub fn get_trend_data(
    all_logs: &[LogEntry],
    done_queues: &[String],
    intervals: usize,
) -> Vec<TrendPoint> {
    if all_logs.is_empty() || intervals == 0 {
        return Vec::new();
    }

    let first_ts = if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&all_logs[0].timestamp) {
        dt.with_timezone(&chrono::Utc)
    } else {
        return Vec::new();
    };
    let last_ts = chrono::Utc::now();
    let duration = last_ts.signed_duration_since(first_ts).num_seconds();

    if duration <= 0 {
        return Vec::new();
    }

    let step = duration / (intervals as i64);
    let mut trend = Vec::new();

    let mut ticket_states: HashMap<String, (String, u32)> = HashMap::new(); // ID -> (Queue, Points)
    let mut log_idx = 0;

    for i in 1..=intervals {
        let current_target_ts = first_ts + chrono::Duration::seconds(step * (i as i64));

        // Process logs up to this target timestamp
        while log_idx < all_logs.len() {
            let entry = &all_logs[log_idx];
            let entry_ts = if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&entry.timestamp) {
                dt.with_timezone(&chrono::Utc)
            } else {
                log_idx += 1;
                continue;
            };

            if entry_ts > current_target_ts {
                break;
            }

            match &entry.payload {
                ActionPayload::CreateTicket {
                    id, queue, points, ..
                } => {
                    ticket_states.insert(id.clone(), (queue.clone(), *points));
                }
                ActionPayload::ChangeStatus { id, to, .. } => {
                    if let Some(state) = ticket_states.get_mut(id) {
                        state.0 = to.clone();
                    }
                }
                ActionPayload::UpdateTicket { id, points } => {
                    if let Some(state) = ticket_states.get_mut(id) {
                        state.1 = *points;
                    }
                }
                _ => {}
            }
            log_idx += 1;
        }

        let mut total_tickets = 0;
        let mut done_tickets = 0;
        let mut total_points = 0;
        let mut done_points = 0;

        for (queue, points) in ticket_states.values() {
            total_tickets += 1;
            total_points += *points;
            if done_queues.contains(queue) {
                done_tickets += 1;
                done_points += *points;
            }
        }

        trend.push(TrendPoint {
            timestamp: current_target_ts.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            total_tickets,
            done_tickets,
            total_points,
            done_points,
        });
    }

    trend
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::action::ActionPayload;

    #[test]
    fn test_get_trend_data() {
        let now = chrono::Utc::now();
        let first_ts = (now - chrono::Duration::minutes(60))
            .to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        let mid_ts = (now - chrono::Duration::minutes(15))
            .to_rfc3339_opts(chrono::SecondsFormat::Secs, true);

        let logs = vec![
            LogEntry {
                timestamp: first_ts.clone(),
                action: "CREATE_TICKET".to_string(),
                description: "Created T1".to_string(),
                payload: ActionPayload::CreateTicket {
                    id: "t1".to_string(),
                    title: "T1".to_string(),
                    queue: "To Doo".to_string(),
                    points: 5,
                },
            },
            LogEntry {
                timestamp: mid_ts.clone(),
                action: "CHANGE_STATUS".to_string(),
                description: "Moved T1".to_string(),
                payload: ActionPayload::ChangeStatus {
                    id: "t1".to_string(),
                    from: "To Do".to_string(),
                    to: "Done".to_string(),
                },
            },
        ];

        let done_queues = vec!["Done".to_string()];
        // 2 intervals over 60 minutes:
        // Point 1: now - 30 min
        // Point 2: now
        let trend = get_trend_data(&logs, &done_queues, 2);

        assert_eq!(trend.len(), 2);

        // Point 1 (now - 30 min)
        // Tickets: 1 (created at -60 min)
        // Done: 0 (not done until -15 min)
        assert_eq!(trend[0].total_tickets, 1);
        assert_eq!(trend[0].done_tickets, 0);

        // Point 2 (now)
        // Tickets: 1
        // Done: 1
        assert_eq!(trend[1].total_tickets, 1);
        assert_eq!(trend[1].done_tickets, 1);
        assert_eq!(trend[1].total_points, 5);
        assert_eq!(trend[1].done_points, 5);
    }
}
