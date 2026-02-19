//! board.rs
//!
//! Purpose: Orchestrates the Kanban board, including loading/saving tickets, moving them between queues, and creating new ones.
//! Includes: Board struct and its complex operational methods.
//! Constraints: Should rely on Ticket, Queue, and Config for data structures, but manages the coordination between them.

use crate::model::config::Config;
use crate::model::queue::Queue;
use crate::model::ticket::{Ticket, TicketMetadata};
use std::path::{Path, PathBuf};

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
            let default_queues = vec![
                "1. Incoming",
                "2. ToDo",
                "3. Doing",
                "4. Reviewing",
                "5. Testing",
                "6. Done",
                "7. Archive",
            ];
            for q in default_queues {
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
        }

        Ok(())
    }

    pub fn queue_path(&self, queue_id: &str) -> PathBuf {
        self.queues_path.join(queue_id)
    }

    pub fn ticket_path(&self, ticket_id: &str) -> PathBuf {
        self.tickets_path.join(ticket_id)
    }

    pub(crate) fn load_ticket(&self, path: &Path) -> anyhow::Result<Ticket> {
        let ticket_id = path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid ticket path: {:?}", path))?
            .to_string();

        let readme_path = path.join("README.md");
        if !readme_path.exists() {
            return Err(anyhow::anyhow!("README.md not found in {:?}", path));
        }

        let content = std::fs::read_to_string(&readme_path)
            .map_err(|e| anyhow::anyhow!("Failed to read README.md in {:?}: {}", path, e))?;

        let parts: Vec<&str> = content.splitn(3, "---").collect();
        if parts.len() < 3 {
            return Err(anyhow::anyhow!(
                "Invalid ticket format in {:?}",
                readme_path
            ));
        }

        let frontmatter = parts[1];
        let body = parts[2].trim().to_string();

        let mut metadata: TicketMetadata = serde_yaml::from_str(frontmatter)
            .map_err(|e| anyhow::anyhow!("Failed to parse YAML in {:?}: {}", readme_path, e))?;

        if metadata.updated_at.is_empty() && !metadata.created_at.is_empty() {
            metadata.updated_at = metadata.created_at.clone();
        }

        Ok(Ticket::from_metadata(ticket_id, metadata, body))
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

        if visible {
            for entry in std::fs::read_dir(path)?.flatten() {
                let ticket_link_path = entry.path();

                // Resolve the symlink to get the actual ticket directory
                let resolved_result = if ticket_link_path.is_symlink() {
                    let link_target = std::fs::read_link(&ticket_link_path)?;
                    if link_target.is_relative() {
                        path.join(&link_target).canonicalize()
                    } else {
                        link_target.canonicalize()
                    }
                } else {
                    ticket_link_path.canonicalize()
                };

                match resolved_result {
                    Ok(resolved_path) => {
                        if resolved_path.exists() && resolved_path.is_dir() {
                            match self.load_ticket(&resolved_path) {
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
        let visible = self.config.is_visible(&queue_id);

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
        if let Some(limit) = self.config.get_limit(target_queue_id) {
            let current_count = std::fs::read_dir(&target_path)?
                .flatten()
                .filter(|e| e.path().is_symlink() || e.path().is_dir())
                .count();

            if current_count >= limit {
                return Err(anyhow::anyhow!(
                    "Queue '{}' has reached its limit of {} tickets",
                    target_queue_id,
                    limit
                ));
            }
        }

        // Find the symlink in source_queue that points to the ticket
        let mut link_to_move = None;
        for entry in std::fs::read_dir(&source_path)?.flatten() {
            let path = entry.path();
            let resolved = if path.is_symlink() {
                let target = std::fs::read_link(&path)?;
                if target.is_relative() {
                    source_path.join(&target).canonicalize()?
                } else {
                    target.canonicalize()?
                }
            } else {
                path.canonicalize()?
            };

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

    pub fn delete_ticket(&self, ticket_id: &str) -> anyhow::Result<()> {
        let ticket_path = self.ticket_path(ticket_id);
        if !ticket_path.exists() {
            return Err(anyhow::anyhow!("Ticket {} not found", ticket_id));
        }

        let root_dir = self.tickets_path.parent().unwrap();
        let deleted_root = root_dir.join("Deleted");
        if !deleted_root.exists() {
            std::fs::create_dir_all(&deleted_root)?;
        }

        let dest_path = deleted_root.join(ticket_id);
        if dest_path.exists() {
            std::fs::remove_dir_all(&dest_path)?;
        }

        // Canonicalize paths for comparison (before rename)
        let abs_ticket_path = ticket_path.canonicalize()?;

        std::fs::rename(&ticket_path, &dest_path)?;

        // Update abs_dest_path after rename
        let abs_dest_path = dest_path.canonicalize()?;

        // Cleanup symlinks in all queues
        for entry in std::fs::read_dir(&self.queues_path)?.flatten() {
            let queue_dir = entry.path();
            if queue_dir.is_dir() {
                for ticket_entry in std::fs::read_dir(&queue_dir)?.flatten() {
                    let symlink_path = ticket_entry.path();
                    if symlink_path.is_symlink() {
                        if let Ok(target) = std::fs::read_link(&symlink_path) {
                            // Resolve target without canonicalize if it's broken
                            let resolved = if target.is_relative() {
                                queue_dir.join(&target)
                            } else {
                                target.clone()
                            };

                            // Check if it matches either old or new location
                            // We use absolute paths for more reliable comparison
                            let resolved_abs = resolved.canonicalize().unwrap_or(resolved);

                            if resolved_abs == abs_dest_path || resolved_abs == abs_ticket_path {
                                std::fs::remove_file(symlink_path)?;
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
    ) -> anyhow::Result<String> {
        use rand::Rng;

        let queue_path = self.queue_path(queue_id);
        if !queue_path.exists() {
            return Err(anyhow::anyhow!("Queue {} not found", queue_id));
        }

        if let Some(limit) = self.config.get_limit(queue_id) {
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

        // Generate unique ID
        let id: String = std::iter::repeat(())
            .map(|()| {
                let chars = b"abcdefghijklmnopqrstuvwxyz0123456789";
                let idx = rand::thread_rng().gen_range(0..chars.len());
                chars[idx] as char
            })
            .take(6)
            .collect();
        let ticket_id = id;

        let ticket_dir = self.ticket_path(&ticket_id);
        if ticket_dir.exists() {
            return self.create_ticket(title, description, queue_id);
        }
        std::fs::create_dir_all(&ticket_dir)?;

        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        self.write_ticket_readme(&ticket_dir, title, &now, &now, description)?;

        #[cfg(unix)]
        std::os::unix::fs::symlink(&ticket_dir, queue_path.join(&ticket_id))?;

        Ok(ticket_id)
    }

    fn write_ticket_readme(
        &self,
        dir: &Path,
        title: &str,
        created_at: &str,
        updated_at: &str,
        description: &str,
    ) -> anyhow::Result<()> {
        let readme_path = dir.join("README.md");
        let mut f = std::fs::File::create(&readme_path)?;
        use std::io::Write;
        write!(
            f,
            "---\ntitle: {}\ncreated_at: {}\nupdated_at: {}\n---\n{}",
            title, created_at, updated_at, description
        )?;
        Ok(())
    }

    pub fn update_ticket(
        &self,
        ticket_id: &str,
        title: &str,
        description: &str,
    ) -> anyhow::Result<()> {
        let ticket_dir = self.ticket_path(ticket_id);
        let ticket = self.load_ticket(&ticket_dir)?;

        let updated_at = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        self.write_ticket_readme(
            &ticket_dir,
            title,
            &ticket.created_at,
            &updated_at,
            description,
        )?;

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
        for queue in &self.queues {
            if let Some(ticket) = queue.tickets.iter().find(|t| t.id == id) {
                return Some(ticket);
            }
        }
        None
    }
}
