//! board.rs
//!
//! Purpose: Orchestrates the Kanban board, including loading/saving tickets, moving them between queues, and creating new ones.
//! Includes: Board struct and its complex operational methods.
//! Constraints: Should rely on Ticket, Queue, and Config for data structures, but manages the coordination between them.

use crate::model::action::ActionPayload;
use crate::model::config::Config;
use crate::model::queue::Queue;
use crate::model::ticket::{Ticket, TicketMetadata};
use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::SystemTime;
use tr::tr;

static TICKET_CACHE: OnceLock<Mutex<HashMap<PathBuf, (SystemTime, Ticket)>>> = OnceLock::new();

fn get_ticket_cache() -> &'static Mutex<HashMap<PathBuf, (SystemTime, Ticket)>> {
    TICKET_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Default queue names created when initializing a new board.
const DEFAULT_QUEUES: &[&str] = &[
    "1.Incoming",
    "2.ToDo",
    "3.Doing",
    "4.Reviewing",
    "5.Testing",
    "6.Done",
    "7.Archive",
];
/// Returns true if the directory contains any entries whose name does NOT start with '.'.
/// Dotfiles (names starting with '.') are ignored — they don't count as "having content".
fn has_non_dot_entries(path: &Path) -> bool {
    std::fs::read_dir(path)
        .ok()
        .and_then(|entries| {
            entries.flatten().find(|e| {
                let name = e.file_name();
                !name.to_string_lossy().starts_with('.')
            })
        })
        .is_some()
}

const DEFAULT_README: &str = r#"---
title: Project Overview
author: Kanban Authors
created_at: 2026-01-01 00:00:00
updated_at: 2026-01-01 00:00:00
---
# Board Overview

Welcome to your new Kanban board! This system is designed for local-first, file-system-based project management.

## Quality Process (Definition of Done)
To ensure high quality and clarity:
1. **Verification**: A ticket is considered "Done" only when the **original author** confirms that all tasks, conditions, and acceptance criteria have been fully met.
2. **Review**: Tasks should move through the "Reviewing" and "Testing" queues before being finalized.
3. **Closing**: Only after confirmation should a ticket be moved to the "Done" queue. "Archive" is reserved for completed tasks that are no longer needed for daily tracking.

## Statistics & Analytics
Detailed analytics are available via the **Board Info** -> **Statistics** button.

### How Metrics are Calculated:
- **Board Completion Rate**: `(Done Tickets) / (Total Tickets - Archived Tickets)`. This represents the overall progress of the project, excluding tasks that have been archived.
- **Sprint Completion Rate**: This metric tracks performance during the active sprint. It is calculated as `(Completed in Sprint) / (Active in Sprint)`, where "Active" includes any ticket created or modified during the sprint period.
- **Lead Time**: The total time elapsed from the moment a ticket is created until it reaches the "Done" queue. It measures the customer's perspective of time.
- **Cycle Time**: The time spent actively working on a task. It measures the duration from when a ticket leaves the "starting" queues (e.g., To Dooooo) until it enters a "done" queue.

## Sprints
Sprints are time-boxed iterations (usually 1-2 weeks) that help the team focus on a specific set of tasks.
- **Detection**: The system automatically detects the current sprint based on today's date.
- **Tracking**: Use the "Sprint" display in the header to see the current sprint's progress.
- **Management**: You can add, update, or remove sprints using the CLI: `kanban sprint add --name "Sprint Name" --start YYYY-MM-DD --end YYYY-MM-DD`.

## Configuration
Customize your board by editing `config.toml`:
- **users**: Define team members to enable assignment.
- **queue_limits**: Set WIP (Work In Progress) limits to prevent bottlenecks.
- **workflows**: Customize which queues are considered "start" (e.g., To Dooooooo) and "done" (e.g., Done, Archive) for accurate time tracking.
- **points_scale**: Customize point values and their meaning (default setup is 1-10).

## Estimation (Points)
Each ticket can be assigned a "Point" value (from 0 to 10) to represent the estimated effort or complexity:
- **0 pts**: No estimation or trivial task.
- **1-4 pts**: Tasks taking 1 to 4 days.
- **5 pts**: 1 week.
- **6 pts**: 2 weeks.
- **7 pts**: 1 month.
- **8 pts**: 2-3 months.
- **9 pts**: 6 months.
- **10 pts**: 1 year.

The system uses these points to calculate the **Board Completion Rate (by points)**, which provides a more accurate view of progress than just ticket count.

Tip: The `config.toml` file uses the TOML format. The application will automatically reload when you save changes to this file.
"#;

/// File-system–backed Kanban board.
///
/// Layout on disk:
///   <root>/Tickets/<id>/README.md  — ticket data (YAML frontmatter + markdown body)
///   <root>/Queue/<queue_name>/<id> — symlink → ../../Tickets/<id>
///   <root>/Deleted/<id>/           — soft-deleted tickets (moved from Tickets)
///   <root>/config.toml             — user prefs, queue limits, search history
#[derive(Debug, Clone)]
pub struct Board {
    pub queues: Vec<Queue>,
    pub tickets_path: PathBuf,
    pub queues_path: PathBuf,
    pub config: Config,
    pub ticket_index: std::collections::HashMap<String, Ticket>,
}

impl Board {
    pub fn ensure_initialized(root_path: &Path) -> anyhow::Result<()> {
        let queues_path = root_path.join("Queue");
        let tickets_path = root_path.join("Tickets");

        std::fs::create_dir_all(&queues_path)?;
        std::fs::create_dir_all(&tickets_path)?;
        std::fs::create_dir_all(root_path.join("logs"))?;

        // Check if any queues already exist (ignore dotfiles)
        let mut has_queues = false;
        if let Ok(entries) = std::fs::read_dir(&queues_path) {
            for entry in entries.flatten() {
                let name = entry.file_name();
                if !name.to_string_lossy().starts_with('.') && entry.path().is_dir() {
                    has_queues = true;
                    break;
                }
            }
        }

        if !has_queues {
            for q in DEFAULT_QUEUES {
                std::fs::create_dir_all(queues_path.join(q))?;
                // Create .keepme file for git tracking in each default queue
                Self::ensure_keepme(&queues_path.join(q))?;
            }
            // Create default config file
            let config_path = root_path.join("config.toml");
            if !config_path.exists() {
                let mut default_config = Config::default();
                default_config.set_limit("2.ToDo".to_string(), 21);
                default_config.set_limit("3.Doing".to_string(), 5);
                default_config.write(root_path)?;
            }

            // Create default root README.md if it doesn't exist
            let readme_path = root_path.join("README.md");
            if !readme_path.exists() {
                std::fs::write(&readme_path, DEFAULT_README)?;
            }
        }

        // Create .keepme files in empty root directories (for git tracking)
        Self::ensure_keepme(&queues_path)?;
        Self::ensure_keepme(&tickets_path)?;
        Self::ensure_keepme(&root_path.join("logs"))?;

        Ok(())
    }

    /// Write a .keepme file in the directory if it doesn't already exist.
    fn ensure_keepme(dir: &Path) -> anyhow::Result<()> {
        let keepme_path = dir.join(".keepme");
        if !keepme_path.exists() {
            std::fs::write(&keepme_path, "")?;
        }
        Ok(())
    }

    /// Loads the root README.md content and metadata.
    pub fn load_board_info(root_path: &Path) -> anyhow::Result<(TicketMetadata, String)> {
        let readme_path = root_path.join("README.md");
        let content = if readme_path.exists() {
            std::fs::read_to_string(&readme_path)?
        } else {
            DEFAULT_README.to_string()
        };

        Ok(Self::parse_readme_content(&content))
    }

    pub fn update_board_readme(&self, content: &str) -> anyhow::Result<()> {
        let root_path = self
            .tickets_path
            .parent()
            .ok_or_else(|| anyhow::anyhow!(tr!("Invalid root path")))?;
        let readme_path = root_path.join("README.md");
        std::fs::write(&readme_path, content)?;
        self.append_log_entry(ActionPayload::UpdateBoardInfo, "Updated board README.md")?;
        Ok(())
    }

    pub fn add_queue(&self, name: &str) -> anyhow::Result<()> {
        let root_path = self
            .tickets_path
            .parent()
            .ok_or_else(|| anyhow::anyhow!(tr!("Invalid root path")))?;
        let queue_path = root_path.join("Queue").join(name);
        if queue_path.exists() {
            anyhow::bail!(tr!("Queue already exists: {}", name));
        }
        std::fs::create_dir_all(&queue_path)?;
        // Create .keepme file for git tracking
        std::fs::write(queue_path.join(".keepme"), "")?;
        self.append_log_entry(
            ActionPayload::ManageQueues {
                op: "ADD".to_string(),
                queue: name.to_string(),
                new_name: None,
            },
            &format!("Added queue: {}", name),
        )?;
        Ok(())
    }

    pub fn rename_queue(&self, old_id: &str, new_name: &str) -> anyhow::Result<()> {
        let root_path = self
            .tickets_path
            .parent()
            .ok_or_else(|| anyhow::anyhow!(tr!("Invalid root path")))?;
        let old_path = root_path.join("Queue").join(old_id);
        let new_path = root_path.join("Queue").join(new_name);

        if !old_path.exists() {
            anyhow::bail!(tr!("Queue not found: {}", old_id));
        }
        if new_path.exists() {
            anyhow::bail!(tr!("Queue already exists: {}", new_name));
        }

        std::fs::rename(&old_path, &new_path)?;

        // Update limits if any
        let mut config = self.config.clone();
        if let Some(limit) = config.kanban.queue_limits.remove(old_id) {
            config
                .kanban
                .queue_limits
                .insert(new_name.to_string(), limit);
            config.write(root_path)?;
        }

        self.append_log_entry(
            ActionPayload::ManageQueues {
                op: "RENAME".to_string(),
                queue: old_id.to_string(),
                new_name: Some(new_name.to_string()),
            },
            &format!("Renamed queue: {} -> {}", old_id, new_name),
        )?;
        Ok(())
    }

    pub fn delete_queue(&self, id: &str) -> anyhow::Result<()> {
        let root_path = self
            .tickets_path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Invalid root path"))?;
        let queue_path = root_path.join("Queue").join(id);

        if !queue_path.exists() {
            anyhow::bail!(tr!("Queue not found: {}", id));
        }

        // Check if empty — only dotfiles count as empty
        if has_non_dot_entries(&queue_path) {
            anyhow::bail!(tr!("Queue is not empty: {}", id));
        }

        std::fs::remove_dir_all(&queue_path)?;
        // Update limits if any
        let mut config = self.config.clone();
        if config.kanban.queue_limits.remove(id).is_some() {
            config.write(root_path)?;
        }

        self.append_log_entry(
            ActionPayload::ManageQueues {
                op: "DELETE".to_string(),
                queue: id.to_string(),
                new_name: None,
            },
            &format!("Deleted queue: {}", id),
        )?;
        Ok(())
    }

    pub fn add_user(&mut self, user: &str) -> anyhow::Result<()> {
        if self.config.kanban.users.contains(&user.to_string()) {
            return Ok(());
        }
        self.config.kanban.users.push(user.to_string());
        let root_path = self
            .tickets_path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Invalid root path"))?;
        self.config.write(root_path)?;

        self.append_log_entry(
            ActionPayload::ManageUsers {
                op: "ADD".to_string(),
                user: user.to_string(),
            },
            &format!("Added user: {}", user),
        )?;
        Ok(())
    }

    pub fn remove_user(&mut self, user: &str) -> anyhow::Result<()> {
        self.config.kanban.users.retain(|u| u != user);
        let root_path = self
            .tickets_path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Invalid root path"))?;
        self.config.write(root_path)?;

        self.append_log_entry(
            ActionPayload::ManageUsers {
                op: "REMOVE".to_string(),
                user: user.to_string(),
            },
            &format!("Removed user: {}", user),
        )?;
        Ok(())
    }

    pub fn add_sprint(
        &mut self,
        name: String,
        start_date: String,
        end_date: String,
    ) -> anyhow::Result<()> {
        let number = self
            .config
            .kanban
            .sprints
            .iter()
            .map(|s| s.number)
            .max()
            .map(|n| n + 1)
            .unwrap_or(1);

        self.config
            .kanban
            .sprints
            .push(crate::model::config::Sprint {
                number,
                name: name.clone(),
                start_date: start_date.clone(),
                end_date: end_date.clone(),
            });
        self.config.kanban.sprints.sort_by_key(|s| s.number);

        let root_path = self
            .tickets_path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Invalid root path"))?;
        self.config.write(root_path)?;

        self.append_log_entry(
            ActionPayload::ManageSprints {
                op: "ADD".to_string(),
                sprint_number: number,
            },
            &format!("Added sprint: {} - {}", number, name),
        )?;
        Ok(())
    }

    pub fn update_sprint(
        &mut self,
        number: u32,
        name: Option<String>,
        start_date: Option<String>,
        end_date: Option<String>,
    ) -> anyhow::Result<()> {
        let sprint = self
            .config
            .kanban
            .sprints
            .iter_mut()
            .find(|s| s.number == number)
            .ok_or_else(|| anyhow::anyhow!(tr!("Sprint {} not found", number)))?;

        if let Some(n) = name {
            sprint.name = n;
        }
        if let Some(s) = start_date {
            sprint.start_date = s;
        }
        if let Some(e) = end_date {
            sprint.end_date = e;
        }

        let root_path = self
            .tickets_path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Invalid root path"))?;
        self.config.write(root_path)?;

        self.append_log_entry(
            ActionPayload::ManageSprints {
                op: "UPDATE".to_string(),
                sprint_number: number,
            },
            &format!("Updated sprint: {}", number),
        )?;
        Ok(())
    }

    pub fn remove_sprint(&mut self, number: u32) -> anyhow::Result<()> {
        self.config.kanban.sprints.retain(|s| s.number != number);

        let root_path = self
            .tickets_path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Invalid root path"))?;
        self.config.write(root_path)?;

        self.append_log_entry(
            ActionPayload::ManageSprints {
                op: "REMOVE".to_string(),
                sprint_number: number,
            },
            &format!("Removed sprint: {}", number),
        )?;
        Ok(())
    }

    fn parse_readme_content(content: &str) -> (TicketMetadata, String) {
        let default_meta = || TicketMetadata {
            title: "Board Overview".to_string(),
            ..Default::default()
        };

        let parts: Vec<&str> = content.splitn(3, "---").collect();

        if parts.len() < 3 {
            return (default_meta(), content.to_string());
        }

        let frontmatter = parts[1];
        let body = parts[2].trim().to_string();
        let mut metadata: TicketMetadata =
            serde_yaml::from_str(frontmatter).unwrap_or_else(|_| default_meta());
        if metadata.title.is_empty() {
            metadata.title = "Board Overview".to_string();
        }
        // Backfill updated_at
        if metadata.updated_at.is_empty() && !metadata.created_at.is_empty() {
            metadata.updated_at = metadata.created_at.clone();
        }
        (metadata, body)
    }

    pub fn can_manage_ticket(&self, ticket: &Ticket, admin_bypass: bool) -> bool {
        if admin_bypass || !self.config.manage_only_mine() {
            return true;
        }

        let active_user = self.config.active_user();
        if active_user == "admin" || active_user == "<unassigned>" {
            return true;
        }

        ticket.assigned_to == active_user || ticket.author == active_user
    }

    pub fn queue_path(&self, queue_id: &str) -> PathBuf {
        self.queues_path.join(queue_id)
    }

    pub fn ticket_path(&self, ticket_id: &str) -> PathBuf {
        self.tickets_path.join(ticket_id)
    }

    /// Loads a ticket from disk. Thin wrapper over [`Ticket::load`] kept
    /// for backward-compatibility with existing tests.
    pub fn load_ticket(&self, path: &Path) -> anyhow::Result<Ticket> {
        Ticket::load_full(path)
    }

    pub fn load(root_path: PathBuf) -> anyhow::Result<Self> {
        let queues_path = root_path.join("Queue");
        let tickets_path = root_path.join("Tickets");
        let config = Config::load(&root_path)?;

        let mut board = Board {
            queues: vec![],
            tickets_path,
            queues_path,
            config,
            ticket_index: std::collections::HashMap::new(),
        };

        if board.queues_path.exists() {
            board.load_all_queues()?;
        }

        Ok(board)
    }

    fn load_all_queues(&mut self) -> anyhow::Result<()> {
        let mut sorted_queue_ids: Vec<String> = std::fs::read_dir(&self.queues_path)?
            .flatten()
            .filter(|e| {
                let name = e.file_name();
                !name.to_string_lossy().starts_with('.') && e.path().is_dir()
            })
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect();
        sorted_queue_ids.sort();

        let mut ticket_to_link: HashMap<String, PathBuf> = HashMap::new();

        // 1. Scan queues for duplicates and broken links
        for q_id in &sorted_queue_ids {
            let q_path = self.queues_path.join(q_id);
            if let Ok(entries) = std::fs::read_dir(&q_path) {
                for entry in entries.flatten() {
                    // Skip dotfile entries (e.g., .keepme, .DS_Store)
                    let name = entry.file_name();
                    if name.to_string_lossy().starts_with('.') {
                        continue;
                    }
                    let link_path = entry.path();

                    match Self::resolve_symlink(&q_path, &link_path).and_then(|p| p.canonicalize())
                    {
                        Ok(resolved_path) if resolved_path.exists() && resolved_path.is_dir() => {
                            let actual_id = resolved_path
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or_default()
                                .to_string();
                            if actual_id.is_empty() {
                                continue;
                            }

                            if let Some(prev_link) = ticket_to_link.insert(actual_id, link_path) {
                                // Duplicate: remove from previous (earlier in sorted list)
                                let _ = std::fs::remove_file(prev_link);
                            }
                        }
                        _ => {
                            // Broken or target missing
                            let _ = std::fs::remove_file(link_path);
                        }
                    }
                }
            }
        }

        // 2. Scan Tickets/ for orphans
        if let Some(first_q) = sorted_queue_ids.first() {
            if let Ok(entries) = std::fs::read_dir(&self.tickets_path) {
                for entry in entries.flatten() {
                    // Skip dotfile directories (e.g., .keepme, .DS_Store)
                    let name = entry.file_name();
                    if name.to_string_lossy().starts_with('.') {
                        continue;
                    }
                    let ticket_path = entry.path();
                    if ticket_path.is_dir() {
                        let ticket_id = entry.file_name().to_string_lossy().to_string();
                        if let std::collections::hash_map::Entry::Vacant(e) =
                            ticket_to_link.entry(ticket_id.clone())
                        {
                            // Orphan!
                            let link_path = self.queues_path.join(first_q).join(&ticket_id);
                            #[cfg(unix)]
                            if std::os::unix::fs::symlink(&ticket_path, link_path.clone()).is_ok() {
                                e.insert(link_path);
                            }
                        }
                    }
                }
            }
        }

        // 3. Build actual Queue objects
        let mut queues = vec![];
        for q_id in &sorted_queue_ids {
            let path = self.queues_path.join(q_id);
            queues.push(self.load_queue(&path)?);
        }
        self.queues = queues;

        // Populate ticket index for O(1) lookup
        self.ticket_index.clear();
        for queue in &self.queues {
            for ticket in &queue.tickets {
                self.ticket_index.insert(ticket.id.clone(), ticket.clone());
            }
        }

        Ok(())
    }

    fn load_queue(&self, path: &Path) -> anyhow::Result<Queue> {
        let queue_id = path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid queue path: {:?}", path))?
            .to_string();

        let visible = self.config.is_visible(&queue_id);
        let mut tickets = vec![];

        // Skip disk I/O for hidden queues — they still appear in the model
        // (for the visibility toggle UI) but carry no ticket data.
        if visible {
            for entry in std::fs::read_dir(path)?.flatten() {
                // Skip dotfile entries (e.g., .keepme, .DS_Store)
                let name = entry.file_name();
                if name.to_string_lossy().starts_with('.') {
                    continue;
                }
                let ticket_link_path = entry.path();

                // Resolve symlink to get the actual ticket directory
                let resolved_result =
                    Self::resolve_symlink(path, &ticket_link_path).and_then(|p| p.canonicalize());

                match resolved_result {
                    Ok(resolved_path) => {
                        if resolved_path.exists() && resolved_path.is_dir() {
                            let mut cache = get_ticket_cache().lock().unwrap();
                            let readme_path = resolved_path.join("README.md");
                            let mtime = std::fs::metadata(&readme_path)
                                .and_then(|m| m.modified())
                                .unwrap_or(SystemTime::UNIX_EPOCH);

                            if let Some((cached_mtime, ticket)) = cache.get(&resolved_path) {
                                if *cached_mtime == mtime {
                                    tickets.push(ticket.clone());
                                    continue;
                                }
                            }

                            match Ticket::load(&resolved_path) {
                                Ok(ticket) => {
                                    cache.insert(resolved_path.clone(), (mtime, ticket.clone()));
                                    tickets.push(ticket);
                                }
                                Err(e) => eprintln!(
                                    "Warning: Failed to load ticket at {:?}: {}",
                                    resolved_path, e
                                ),
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!(
                            "Warning: Skipping broken or inaccessible ticket link {:?}: {}",
                            ticket_link_path, e
                        );
                    }
                }
            }
        }

        let limit = self.config.get_limit(&queue_id);

        // Sort tickets by updated_at descending (newest first)
        tickets.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

        Ok(Queue {
            id: queue_id.clone(),
            name: queue_id,
            tickets,
            limit,
            visible,
        })
    }

    pub fn move_ticket(
        &self,
        ticket_id: &str,
        source_queue_id: &str,
        target_queue_id: &str,
    ) -> anyhow::Result<()> {
        let source_path = self.queue_path(source_queue_id);
        let target_path = self.queue_path(target_queue_id);
        let ticket_dir = self.ticket_path(ticket_id).canonicalize()?;

        if !source_path.exists() || !target_path.exists() {
            return Err(anyhow::anyhow!(tr!(
                "Source ({}) or target ({}) queue not found",
                source_path.display(),
                target_path.display()
            )));
        }

        // Check if target queue has reached its limit
        self.check_queue_limit(target_queue_id)?;

        // Queue entries are symlinks; find the one that resolves to our ticket.
        // We compare canonical paths because the symlink target is a relative path.
        let mut link_to_move = None;
        for entry in std::fs::read_dir(&source_path)?.flatten() {
            // Skip dotfile entries
            let name = entry.file_name();
            if name.to_string_lossy().starts_with('.') {
                continue;
            }
            let path = entry.path();
            let resolved = Self::resolve_symlink(&source_path, &path)?.canonicalize()?;

            if resolved == ticket_dir {
                link_to_move = Some(path);
                break;
            }
        }

        if let Some(source_link) = link_to_move {
            let file_name = source_link.file_name().unwrap();
            let dest_link = target_path.join(file_name);
            std::fs::rename(source_link, dest_link)?;

            let payload = ActionPayload::ChangeStatus {
                id: ticket_id.to_string(),
                from: source_queue_id.to_string(),
                to: target_queue_id.to_string(),
            };
            let _ = self.append_log_entry(
                payload,
                &format!(
                    "Moved ticket #{} from \"{}\" to \"{}\"",
                    ticket_id, source_queue_id, target_queue_id
                ),
            );

            Ok(())
        } else {
            Err(anyhow::anyhow!(tr!(
                "Ticket {} not found in queue {}",
                ticket_id,
                source_queue_id
            )))
        }
    }

    /// Soft-deletes a ticket: moves its directory to the system Recycle Bin,
    /// then removes all queue symlinks that pointed to it.
    pub fn delete_ticket(&self, ticket_id: &str) -> anyhow::Result<()> {
        let ticket_path = self.ticket_path(ticket_id);
        if !ticket_path.exists() {
            return Err(anyhow::anyhow!(tr!("Ticket {} not found", ticket_id)));
        }

        // We use canonicalized path for any advanced checking before moving
        let abs_ticket_path = ticket_path.canonicalize()?;

        // Move the directory to the system Recycle Bin
        #[cfg(not(test))]
        {
            if let Err(e) = trash::delete(&abs_ticket_path) {
                return Err(anyhow::anyhow!(tr!(
                    "Failed to move ticket to trash: {}",
                    e
                )));
            }
        }

        // During tests we perform hard deletion to avoid polluting the system trash
        #[cfg(test)]
        {
            std::fs::remove_dir_all(&abs_ticket_path)?;
        }

        // Remove symlinks in all queues that pointed to the deleted ticket.
        // Since the target is now deleted or moved to trash, the links are broken.
        // We identify them primarily by their file stem matching the ticket_id.
        for entry in std::fs::read_dir(&self.queues_path)?.flatten() {
            let queue_dir = entry.path();
            // Skip dotfile directories
            let name = entry.file_name();
            if name.to_string_lossy().starts_with('.') {
                continue;
            }
            if queue_dir.is_dir() {
                for ticket_entry in std::fs::read_dir(&queue_dir)?.flatten() {
                    // Skip dotfile entries
                    let tname = ticket_entry.file_name();
                    if tname.to_string_lossy().starts_with('.') {
                        continue;
                    }
                    let symlink_path = ticket_entry.path();
                    if symlink_path.is_symlink() {
                        let matches_deleted = if let Some(stem) = symlink_path.file_stem() {
                            stem.to_string_lossy() == ticket_id
                        } else {
                            false
                        };

                        if matches_deleted {
                            if let Err(e) = std::fs::remove_file(&symlink_path) {
                                eprintln!(
                                    "Failed to remove broken symlink {:?}: {}",
                                    symlink_path, e
                                );
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub fn create_ticket(
        &self,
        title: &str,
        description: &str,
        queue_id: &str,
        assigned_to: &str,
        author: &str,
        points: u32,
    ) -> anyhow::Result<String> {
        use rand::Rng;

        let queue_path = self.queue_path(queue_id);
        if !queue_path.exists() {
            return Err(anyhow::anyhow!(tr!("Queue {} not found", queue_id)));
        }

        self.check_queue_limit(queue_id)?;

        // Generate unique ID, retrying on (extremely rare) collisions
        let ticket_id = (0..5)
            .map(|_| {
                std::iter::repeat(())
                    .map(|()| {
                        let chars = b"abcdefghijklmnopqrstuvwxyz0123456789";
                        let idx = rand::thread_rng().gen_range(0..chars.len());
                        chars[idx] as char
                    })
                    .take(6)
                    .collect::<String>()
            })
            .find(|id| !self.ticket_path(id).exists())
            .ok_or_else(|| {
                anyhow::anyhow!(tr!("Failed to generate unique ticket ID after 5 attempts"))
            })?;

        let ticket_dir = self.ticket_path(&ticket_id);
        std::fs::create_dir_all(&ticket_dir)?;

        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let ticket = Ticket {
            id: ticket_id.clone(),
            title: title.to_string(),
            created_at: now.clone(),
            updated_at: now.clone(),
            description: description.to_string(),
            assigned_to: assigned_to.to_string(),
            author: author.to_string(),
            points,
            attachment_count: 0,
            comments: vec![],
        };
        ticket.save(&ticket_dir)?;

        // Link ticket into the target queue (Queue/<name>/<id> → Tickets/<id>)
        #[cfg(unix)]
        std::os::unix::fs::symlink(&ticket_dir, queue_path.join(&ticket_id))?;

        let payload = ActionPayload::CreateTicket {
            id: ticket_id.clone(),
            title: title.to_string(),
            queue: queue_id.to_string(),
            points,
        };
        let _ = self.append_log_entry(
            payload,
            &format!(
                "Created ticket \"{}\" (#{}) in queue \"{}\"",
                title, ticket_id, queue_id
            ),
        );

        Ok(ticket_id)
    }

    pub fn update_ticket(
        &self,
        ticket_id: &str,
        title: &str,
        description: &str,
        assigned_to: &str,
        points: u32,
    ) -> anyhow::Result<()> {
        let ticket_dir = self.ticket_path(ticket_id);
        let mut ticket = Ticket::load(&ticket_dir)?;

        let old_assigned = ticket.assigned_to.clone();

        ticket.updated_at = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        ticket.title = title.to_string();
        ticket.description = description.to_string();
        ticket.assigned_to = assigned_to.to_string();
        ticket.points = points;

        ticket.save(&ticket_dir)?;

        if old_assigned != assigned_to && !assigned_to.is_empty() {
            let payload = ActionPayload::AssignTicket {
                id: ticket_id.to_string(),
                assignee: Some(assigned_to.to_string()),
            };
            let _ = self.append_log_entry(
                payload,
                &format!("Assigned ticket #{} to {}", ticket_id, assigned_to),
            );
        } else if old_assigned != assigned_to && assigned_to.is_empty() {
            let payload = ActionPayload::AssignTicket {
                id: ticket_id.to_string(),
                assignee: None,
            };
            let _ = self.append_log_entry(payload, &format!("Unassigned ticket #{}", ticket_id));
        }

        let payload = ActionPayload::UpdateTicket {
            id: ticket_id.to_string(),
            points,
        };
        let _ = self.append_log_entry(payload, &format!("Updated ticket #{}", ticket_id));

        Ok(())
    }

    pub fn add_comment(
        &self,
        ticket_id: &str,
        content: &str,
        author: &str,
    ) -> anyhow::Result<String> {
        let ticket_dir = self.ticket_path(ticket_id);
        if !ticket_dir.exists() {
            return Err(anyhow::anyhow!("Ticket {} not found", ticket_id));
        }

        // Find next integer NNN
        let mut max_n: u32 = 0;
        if let Ok(entries) = std::fs::read_dir(&ticket_dir) {
            for entry in entries.flatten() {
                // Skip dotfile entries
                let name = entry.file_name();
                if name.to_string_lossy().starts_with('.') {
                    continue;
                }
                let name = entry.file_name().to_string_lossy().into_owned();
                if name.starts_with("tc") && name.ends_with(".md") && name.len() >= 5 {
                    if let Ok(n) = name[2..5].parse::<u32>() {
                        if n > max_n {
                            max_n = n;
                        }
                    }
                }
            }
        }
        let next_n = max_n + 1;

        // Generate UID
        use rand::Rng;
        let mut rng = rand::rngs::OsRng;
        let uid_chars: String = (0..5)
            .map(|_| {
                let chars = b"abcdefghijklmnopqrstuvwxyz0123456789";
                let idx = rng.gen_range(0..chars.len());
                chars[idx] as char
            })
            .collect();

        let comment_id = format!("tc{:03}{}", next_n, uid_chars);
        let comment_path = ticket_dir.join(format!("{}.md", comment_id));

        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let comment = crate::model::comment::Comment {
            id: comment_id.clone(),
            metadata: crate::model::comment::CommentMetadata {
                author: author.to_string(),
                created_at: now.clone(),
                updated_at: now.clone(),
                attachments: None,
            },
            content: content.to_string(),
            references: vec![],
        };

        comment.save(&comment_path)?;

        // Update ticket's updated_at
        let mut ticket = Ticket::load(&ticket_dir)?;
        ticket.updated_at = now;
        ticket.save(&ticket_dir)?;

        let payload = ActionPayload::AddComment {
            id: ticket_id.to_string(),
            comment_id: comment_id.clone(),
        };
        let _ = self.append_log_entry(payload, &format!("Added comment to ticket #{}", ticket_id));

        Ok(comment_id)
    }

    pub fn attach_file(
        &self,
        ticket_id: &str,
        source_path: &std::path::Path,
    ) -> anyhow::Result<String> {
        let ticket_dir = self.ticket_path(ticket_id);
        if !ticket_dir.exists() {
            return Err(anyhow::anyhow!("Ticket {} not found", ticket_id));
        }

        let attach_dir = ticket_dir.join("attachment");
        if !attach_dir.exists() {
            std::fs::create_dir_all(&attach_dir)?;
        }

        let file_name = source_path
            .file_name()
            .ok_or_else(|| anyhow::anyhow!("Source path has no filename"))?
            .to_string_lossy()
            .to_string();

        let mut target_name = file_name.clone();
        let mut target_path = attach_dir.join(&target_name);

        // Handle collisions
        if target_path.exists() {
            let stem = source_path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let ext = source_path
                .extension()
                .map(|e| format!(".{}", e.to_string_lossy()))
                .unwrap_or_default();

            let mut i = 1;
            loop {
                target_name = format!("{} ({}){}", stem, i, ext);
                target_path = attach_dir.join(&target_name);
                if !target_path.exists() {
                    break;
                }
                i += 1;
            }
        }

        std::fs::copy(source_path, &target_path)?;

        // Update ticket's attachment_count and updated_at
        if let Ok(mut ticket) = Ticket::load(&ticket_dir) {
            ticket.attachment_count += 1;
            ticket.updated_at = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
            let _ = ticket.save(&ticket_dir);
        }

        let payload = ActionPayload::AttachFile {
            id: ticket_id.to_string(),
            filename: target_name.clone(),
        };
        let _ = self.append_log_entry(
            payload,
            &format!("Attached file \"{}\" to ticket #{}", target_name, ticket_id),
        );

        Ok(format!("[{}](attachment/{})", target_name, target_name))
    }

    pub fn check_queue_limit(&self, queue_id: &str) -> anyhow::Result<()> {
        if let Some(limit) = self.config.get_limit(queue_id) {
            let queue_path = self.queue_path(queue_id);
            if !queue_path.exists() {
                return Ok(());
            }
            let current_count = std::fs::read_dir(&queue_path)?
                .flatten()
                .filter(|e| {
                    let name = e.file_name();
                    !name.to_string_lossy().starts_with('.')
                        && (e.path().is_symlink() || e.path().is_dir())
                })
                .count();

            if current_count >= limit {
                return Err(anyhow::anyhow!(
                    "Queue '{}' has reached its limit of {} tickets",
                    queue_id,
                    limit
                ));
            }
        }
        Ok(())
    }

    pub fn resolve_queue_id(&self, target_id: &str) -> String {
        if let Some(idx_str) = target_id.strip_prefix("index:") {
            if let Ok(idx_f) = idx_str.parse::<f64>() {
                let idx = idx_f.floor() as usize;
                let visible_queues: Vec<_> = self.queues.iter().filter(|q| q.visible).collect();
                return visible_queues
                    .get(idx)
                    .or(visible_queues.last())
                    .map(|q| q.id.clone())
                    .unwrap_or_else(|| target_id.to_string());
            }
        }
        target_id.to_string()
    }

    pub fn find_ticket_by_id(&self, id: &str) -> Option<&Ticket> {
        self.ticket_index.get(id)
    }

    pub fn load_full_ticket(&self, ticket_id: &str) -> anyhow::Result<Ticket> {
        let path = self.ticket_path(ticket_id);
        Ticket::load_full(&path)
    }

    pub fn resolve_symlink(base_dir: &Path, link_path: &Path) -> std::io::Result<PathBuf> {
        if link_path.is_symlink() {
            let target = std::fs::read_link(link_path)?;
            if target.is_relative() {
                Ok(base_dir.join(&target))
            } else {
                Ok(target)
            }
        } else {
            Ok(link_path.to_path_buf())
        }
    }

    pub fn append_log_entry(
        &self,
        payload: ActionPayload,
        description: &str,
    ) -> anyhow::Result<()> {
        let active_user = self.config.active_user();
        if active_user.is_empty() {
            return Ok(());
        }
        let machine_id = self.config.machine_id().unwrap_or("unknown");

        let root_path = self
            .tickets_path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Invalid board root path"))?;

        let logs_path = root_path.join("logs");
        if !logs_path.exists() {
            std::fs::create_dir_all(&logs_path)?;
        }

        let log_file_name = format!("log_{}_{}.md", active_user, machine_id);
        let log_path = logs_path.join(log_file_name);

        let should_write_header = !log_path.exists();

        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)?;

        if should_write_header {
            writeln!(
                &mut file,
                "# User Activity Log: {}\n\n| **Date** | **Action** | **Action description** | **JSON** |\n| :--- | :--- | :--- | :--- |",
                active_user
            )?;
        }

        // Use second precision without fractional part for better readability
        let now = chrono::Local::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        let action_name = payload.to_string();
        let json = serde_json::to_string(&payload)?;

        writeln!(
            &mut file,
            "| {} | {} | {} | `{}` |",
            now, action_name, description, json
        )?;

        Ok(())
    }
}
