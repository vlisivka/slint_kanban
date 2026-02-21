//! board.rs
//!
//! Purpose: Orchestrates the Kanban board, including loading/saving tickets, moving them between queues, and creating new ones.
//! Includes: Board struct and its complex operational methods.
//! Constraints: Should rely on Ticket, Queue, and Config for data structures, but manages the coordination between them.

use crate::model::config::Config;
use crate::model::queue::Queue;
use crate::model::ticket::{Ticket, TicketMetadata};
use std::path::{Path, PathBuf};

/// Default queue names created when initializing a new board.
const DEFAULT_QUEUES: &[&str] = &[
    "1. Incoming",
    "2. ToDo",
    "3. Doing",
    "4. Reviewing",
    "5. Testing",
    "6. Done",
    "7. Archive",
];

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
}

impl Board {
    pub fn ensure_initialized(root_path: &Path) -> anyhow::Result<()> {
        let queues_path = root_path.join("Queue");
        let tickets_path = root_path.join("Tickets");

        std::fs::create_dir_all(&queues_path)?;
        std::fs::create_dir_all(&tickets_path)?;

        // Check if any queues already exist
        let mut has_queues = false;
        if let Ok(entries) = std::fs::read_dir(&queues_path) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    has_queues = true;
                    break;
                }
            }
        }

        if !has_queues {
            for q in DEFAULT_QUEUES {
                std::fs::create_dir_all(queues_path.join(q))?;
            }

            // Create default config file
            let config_path = root_path.join("config.toml");
            if !config_path.exists() {
                let mut default_config = Config::default();
                default_config.set_limit("2. ToDo".to_string(), 21);
                default_config.set_limit("3. Doing".to_string(), 5);
                default_config.write(root_path)?;
            }

            // Create default root README.md if it doesn't exist
            let readme_path = root_path.join("README.md");
            if !readme_path.exists() {
                let default_readme = "---\ntitle: The Project\nauthor: Kanban authors\ncreated_at: 2026-01-01 00:00:00\nupdated_at: 2026-01-01 00:00:00\n---\n# Board Overview\n\nWelcome to your new Kanban board!\n\n## Rules\n1. Move tickets between queues.\n2. Respect WIP limits.\n";
                std::fs::write(&readme_path, default_readme)?;
            }
        }

        Ok(())
    }

    /// Loads the root README.md content and metadata.
    pub fn load_board_info(root_path: &Path) -> anyhow::Result<(TicketMetadata, String)> {
        let default_meta = || TicketMetadata {
            title: "Board Overview".to_string(),
            ..Default::default()
        };

        let readme_path = root_path.join("README.md");
        if !readme_path.exists() {
            return Ok((
                default_meta(),
                "# Board Overview\n\nRoot README.md not found.".to_string(),
            ));
        }

        let content = std::fs::read_to_string(&readme_path)?;
        let parts: Vec<&str> = content.splitn(3, "---").collect();

        if parts.len() < 3 {
            return Ok((default_meta(), content));
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
        Ok((metadata, body))
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
        Ticket::load(path)
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
        };

        if board.queues_path.exists() {
            board.load_all_queues()?;
        }

        Ok(board)
    }

    fn load_all_queues(&mut self) -> anyhow::Result<()> {
        let mut queues = vec![];
        for entry in std::fs::read_dir(&self.queues_path)?.flatten() {
            let path = entry.path();
            if path.is_dir() {
                queues.push(self.load_queue(&path)?);
            }
        }
        queues.sort_by(|a, b| a.id.cmp(&b.id));
        self.queues = queues;
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
                let ticket_link_path = entry.path();

                // Resolve symlink to get the actual ticket directory
                let resolved_result =
                    Self::resolve_symlink(path, &ticket_link_path).and_then(|p| p.canonicalize());

                match resolved_result {
                    Ok(resolved_path) => {
                        if resolved_path.exists() && resolved_path.is_dir() {
                            match Ticket::load(&resolved_path) {
                                Ok(ticket) => tickets.push(ticket),
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
            return Err(anyhow::anyhow!(
                "Source ({:?}) or target ({:?}) queue not found",
                source_path,
                target_path
            ));
        }

        // Check if target queue has reached its limit
        self.check_queue_limit(target_queue_id)?;

        // Queue entries are symlinks; find the one that resolves to our ticket.
        // We compare canonical paths because the symlink target is a relative path.
        let mut link_to_move = None;
        for entry in std::fs::read_dir(&source_path)?.flatten() {
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
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Ticket {} not found in queue {}",
                ticket_id,
                source_queue_id
            ))
        }
    }

    /// Soft-deletes a ticket: moves its directory to the system Recycle Bin,
    /// then removes all queue symlinks that pointed to it.
    pub fn delete_ticket(&self, ticket_id: &str) -> anyhow::Result<()> {
        let ticket_path = self.ticket_path(ticket_id);
        if !ticket_path.exists() {
            return Err(anyhow::anyhow!("Ticket {} not found", ticket_id));
        }

        // We use canonicalized path for any advanced checking before moving
        let abs_ticket_path = ticket_path.canonicalize()?;

        // Move the directory to the system Recycle Bin
        #[cfg(not(test))]
        {
            if let Err(e) = trash::delete(&abs_ticket_path) {
                return Err(anyhow::anyhow!("Failed to move ticket to trash: {}", e));
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
            if queue_dir.is_dir() {
                for ticket_entry in std::fs::read_dir(&queue_dir)?.flatten() {
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
    ) -> anyhow::Result<String> {
        use rand::Rng;

        let queue_path = self.queue_path(queue_id);
        if !queue_path.exists() {
            return Err(anyhow::anyhow!("Queue {} not found", queue_id));
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
                anyhow::anyhow!("Failed to generate unique ticket ID after 5 attempts")
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
        };
        ticket.save(&ticket_dir)?;

        // Link ticket into the target queue (Queue/<name>/<id> → Tickets/<id>)
        #[cfg(unix)]
        std::os::unix::fs::symlink(&ticket_dir, queue_path.join(&ticket_id))?;

        Ok(ticket_id)
    }

    pub fn update_ticket(
        &self,
        ticket_id: &str,
        title: &str,
        description: &str,
        assigned_to: &str,
    ) -> anyhow::Result<()> {
        let ticket_dir = self.ticket_path(ticket_id);
        let mut ticket = Ticket::load(&ticket_dir)?;

        ticket.updated_at = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        ticket.title = title.to_string();
        ticket.description = description.to_string();
        ticket.assigned_to = assigned_to.to_string();

        ticket.save(&ticket_dir)?;

        Ok(())
    }

    /// Resolves a queue identifier. If `target_id` is "index:<N>", it maps
    /// the pixel-based column index from drag-and-drop to the Nth visible queue.
    /// Otherwise returns the ID as-is (direct queue name from CLI).
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
        for queue in &self.queues {
            if let Some(ticket) = queue.tickets.iter().find(|t| t.id == id) {
                return Some(ticket);
            }
        }
        None
    }

    pub fn check_queue_limit(&self, queue_id: &str) -> anyhow::Result<()> {
        if let Some(limit) = self.config.get_limit(queue_id) {
            let queue_path = self.queue_path(queue_id);
            if !queue_path.exists() {
                return Ok(());
            }
            let current_count = std::fs::read_dir(&queue_path)?
                .flatten()
                .filter(|e| e.path().is_symlink() || e.path().is_dir())
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
}
