use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub queue_limits: HashMap<String, usize>,
}

impl Default for Config {
    fn default() -> Self {
        let queue_limits = HashMap::new();
        Self { queue_limits }
    }
}

impl Config {
    pub fn load(root_path: &std::path::Path) -> anyhow::Result<Self> {
        let config_path = root_path.join("config.toml");

        if !config_path.exists() {
            // Create default config file
            let default_config = Self::default();
            Ok(default_config)
        } else {
            let content = std::fs::read_to_string(&config_path)?;
            let config: Config = toml::from_str(&content)?;
            Ok(config)
        }
    }

    pub fn get_limit(&self, queue_id: &str) -> Option<usize> {
        self.queue_limits.get(queue_id).copied()
    }

    pub fn set_limit(&mut self, queue_id: String, limit: usize) {
        self.queue_limits.insert(queue_id, limit);
    }

    pub fn write(&self, root_path: &std::path::Path) -> anyhow::Result<()> {
        let config_path = root_path.join("config.toml");
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&config_path, content)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketMetadata {
    pub title: String,
    #[serde(default)]
    pub created_at: String, // ISO 8601 or similar
    #[serde(default)]
    pub updated_at: String,
}

#[derive(Debug, Clone)]
pub struct Ticket {
    pub id: String,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct Queue {
    pub id: String,
    pub name: String,
    pub tickets: Vec<Ticket>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct Board {
    pub queues: Vec<Queue>,
    pub tickets_path: PathBuf,
    pub queues_path: PathBuf,
    pub config: Config,
}

impl Ticket {
    pub fn from_metadata(id: String, metadata: TicketMetadata, description: String) -> Self {
        Self {
            id,
            title: metadata.title,
            created_at: metadata.created_at,
            updated_at: metadata.updated_at,
            description,
        }
    }

    pub fn extract_references(&self) -> Vec<String> {
        let mut refs = Vec::new();
        let mut start = 0;
        while let Some(pos) = self.description[start..].find("#T-") {
            let actual_pos = start + pos;
            if actual_pos + 9 <= self.description.len() {
                let potential_id = &self.description[actual_pos..actual_pos + 9];
                // Check if it's #T- followed by 6 alphanumeric chars
                if potential_id.chars().skip(3).all(|c| c.is_alphanumeric()) {
                    refs.push(potential_id.to_string());
                }
            }
            start = actual_pos + 3;
        }
        refs.sort();
        refs.dedup();
        refs
    }
}

impl Board {
    pub fn ensure_initialized(root_path: &std::path::Path) -> anyhow::Result<()> {
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

    fn load_ticket(&self, path: &std::path::Path) -> anyhow::Result<Ticket> {
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

    fn load_queue(&self, path: &std::path::Path) -> anyhow::Result<Queue> {
        let queue_id = path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid queue path: {:?}", path))?
            .to_string();

        let mut tickets = vec![];
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

        let limit = self.config.get_limit(&queue_id);
        Ok(Queue {
            id: queue_id.clone(),
            name: queue_id,
            tickets,
            limit,
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
        use rand::{distributions::Alphanumeric, Rng};

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
            .map(|()| rand::thread_rng().sample(Alphanumeric))
            .map(char::from)
            .take(6)
            .collect();
        let ticket_id = format!("T-{}", id);

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
        dir: &std::path::Path,
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
                return self
                    .queues
                    .get(idx)
                    .or(self.queues.last())
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_ticket_metadata_deserialization() {
        let yaml = "
title: Buy Groceries
created_at: 2023-10-27
updated_at: 2023-10-27
";
        let metadata: TicketMetadata = serde_yaml::from_str(yaml).expect("Failed to parse YAML");
        assert_eq!(metadata.title, "Buy Groceries", "Ticket title should match YAML input");
        assert_eq!(metadata.created_at, "2023-10-27", "Created date should match YAML input");
        assert_eq!(metadata.updated_at, "2023-10-27", "Updated date should match YAML input");
    }

    #[test]
    fn test_ticket_metadata_missing_updated_at() {
        let yaml = "
title: Buy Groceries
created_at: 2023-10-27
";
        let metadata: TicketMetadata = serde_yaml::from_str(yaml).expect("Failed to parse YAML");
        assert_eq!(metadata.title, "Buy Groceries", "Ticket title should match YAML input even with missing fields");
        assert_eq!(metadata.created_at, "2023-10-27", "Created date should match YAML input");
        assert_eq!(metadata.updated_at, "", "Updated date should be empty if missing in YAML");
    }

    #[test]
    fn test_board_scanning() -> anyhow::Result<()> {
        let root = tempdir()?;
        let root_path = root.path().to_path_buf();
        let tickets_dir = root_path.join("Tickets");
        let queues_dir = root_path.join("Queue");

        std::fs::create_dir(&tickets_dir)?;
        std::fs::create_dir(&queues_dir)?;

        // Create Ticket T1
        let t1_path = tickets_dir.join("T1");
        std::fs::create_dir(&t1_path)?;
        let mut t1_readme = File::create(t1_path.join("README.md"))?;
        write!(
            t1_readme,
            "---\ntitle: Task 1\ncreated_at: 2023-01-01\nupdated_at: 2023-01-01\n---\nBody 1"
        )?;

        // Create Queue Q1
        let q1_path = queues_dir.join("Q1");
        std::fs::create_dir(&q1_path)?;

        // Symlink T1 to Q1
        #[cfg(unix)]
        std::os::unix::fs::symlink(&t1_path, q1_path.join("link_to_T1"))?;

        let board = Board::load(root_path)?;
        assert_eq!(board.queues.len(), 1, "Board should have exactly one queue after scanning");
        let q1 = &board.queues[0];
        assert_eq!(q1.id, "Q1", "Queue ID should match folder name");
        assert_eq!(q1.tickets.len(), 1, "Queue should contain one ticket");
        assert_eq!(q1.tickets[0].title, "Task 1", "Ticket title should match README content");
        assert_eq!(q1.tickets[0].id, "T1", "Ticket ID should match folder name");

        Ok(())
    }

    #[test]
    fn test_board_scanning_multiple_queues() -> anyhow::Result<()> {
        let root = tempdir()?;
        let root_path = root.path().to_path_buf();
        let tickets_dir = root_path.join("Tickets");
        let queues_dir = root_path.join("Queue");

        std::fs::create_dir_all(&tickets_dir)?;
        std::fs::create_dir_all(&queues_dir)?;

        // Ticket ttt123
        let t1_path = tickets_dir.join("ttt123");
        std::fs::create_dir(&t1_path)?;
        let mut f1 = File::create(t1_path.join("README.md"))?;
        write!(
            f1,
            "---\ntitle: T123\ncreated_at: 2023-01-01\nupdated_at: 2023-01-01\n---\nBody"
        )?;

        // Ticket ttt456
        let t2_path = tickets_dir.join("ttt456");
        std::fs::create_dir(&t2_path)?;
        let mut f2 = File::create(t2_path.join("README.md"))?;
        write!(
            f2,
            "---\ntitle: T456\ncreated_at: 2023-01-02\nupdated_at: 2023-01-02\n---\nBody"
        )?;

        // Queue q1
        let q1_path = queues_dir.join("q1");
        std::fs::create_dir(&q1_path)?;
        #[cfg(unix)]
        std::os::unix::fs::symlink(&t1_path, q1_path.join("link1"))?;

        // Queue q2
        let q2_path = queues_dir.join("q2");
        std::fs::create_dir(&q2_path)?;
        #[cfg(unix)]
        std::os::unix::fs::symlink(&t2_path, q2_path.join("link2"))?;

        let board = Board::load(root_path)?;

        // We can't guarantee order of directory reading, so we search
        let q1 = board
            .queues
            .iter()
            .find(|q| q.id == "q1")
            .expect("q1 not found");
        assert_eq!(q1.tickets.len(), 1, "q1 should have one ticket");
        assert_eq!(q1.tickets[0].id, "ttt123", "q1 should contain ticket ttt123");

        let q2 = board
            .queues
            .iter()
            .find(|q| q.id == "q2")
            .expect("q2 not found");
        assert_eq!(q2.tickets.len(), 1, "q2 should have one ticket");
        assert_eq!(q2.tickets[0].id, "ttt456", "q2 should contain ticket ttt456");

        Ok(())
    }

    #[test]
    fn test_move_ticket() -> anyhow::Result<()> {
        let root = tempdir()?;
        let root_path = root.path().to_path_buf();
        let tickets_dir = root_path.join("Tickets");
        let queues_dir = root_path.join("Queue");

        std::fs::create_dir_all(&tickets_dir)?;
        std::fs::create_dir_all(&queues_dir)?;

        // Create Ticket T1
        let t1_path = tickets_dir
            .join("T1")
            .canonicalize()
            .unwrap_or(tickets_dir.join("T1"));
        if !t1_path.exists() {
            std::fs::create_dir(&t1_path)?;
        }
        // tempdir path might not be canonicalizable if it doesn't exist yet, but here it does.
        // Actually, canonicalize fails if the path doesn't exist.

        let mut t1_readme = File::create(t1_path.join("README.md"))?;
        write!(
            t1_readme,
            "---\ntitle: T1\ncreated_at: 2023-01-01\nupdated_at: 2023-01-01\n---\nBody"
        )?;

        // Create Queues Q1, Q2
        let q1_path = queues_dir.join("Q1");
        let q2_path = queues_dir.join("Q2");
        std::fs::create_dir(&q1_path)?;
        std::fs::create_dir(&q2_path)?;

        // Symlink T1 to Q1
        #[cfg(unix)]
        std::os::unix::fs::symlink(&t1_path, q1_path.join("T1"))?;

        let board = Board::load(root_path.clone())?;
        assert_eq!(
            board
                .queues
                .iter()
                .find(|q| q.id == "Q1")
                .unwrap()
                .tickets
                .len(),
            1,
            "Q1 should initially have one ticket"
        );
        assert_eq!(
            board
                .queues
                .iter()
                .find(|q| q.id == "Q2")
                .unwrap()
                .tickets
                .len(),
            0,
            "Q2 should initially be empty"
        );

        // Move T1 from Q1 to Q2
        board.move_ticket("T1", "Q1", "Q2")?;

        let board_after = Board::load(root_path)?;
        assert_eq!(
            board_after
                .queues
                .iter()
                .find(|q| q.id == "Q1")
                .unwrap()
                .tickets
                .len(),
            0,
            "Q1 should be empty after moving the ticket"
        );
        assert_eq!(
            board_after
                .queues
                .iter()
                .find(|q| q.id == "Q2")
                .unwrap()
                .tickets
                .len(),
            1,
            "Q2 should have the moved ticket"
        );
        assert_eq!(
            board_after
                .queues
                .iter()
                .find(|q| q.id == "Q2")
                .unwrap()
                .tickets[0]
                .id,
            "T1",
            "Moved ticket ID should be T1"
        );

        Ok(())
    }

    #[test]
    fn test_delete_ticket() -> anyhow::Result<()> {
        let root = tempdir()?;
        let root_dir = root.path().to_path_buf();
        let tickets_dir = root_dir.join("Tickets");
        let queues_dir = root_dir.join("Queue");
        let deleted_dir = root_dir.join("Deleted");

        std::fs::create_dir_all(&tickets_dir)?;
        std::fs::create_dir_all(&queues_dir)?;

        let t1_path = tickets_dir
            .join("T1")
            .canonicalize()
            .unwrap_or(tickets_dir.join("T1"));
        if !t1_path.exists() {
            std::fs::create_dir(&t1_path)?;
        }
        let mut f1 = File::create(t1_path.join("README.md"))?;
        write!(
            f1,
            "---\ntitle: T1\ncreated_at: 2023-01-01\nupdated_at: 2023-01-01\n---\nBody"
        )?;

        let q1_path = queues_dir.join("Q1");
        std::fs::create_dir(&q1_path)?;
        #[cfg(unix)]
        std::os::unix::fs::symlink(&t1_path, q1_path.join("T1_link"))?;

        let board = Board::load(root_dir.clone())?;
        assert_eq!(board.queues[0].tickets.len(), 1, "Queue should have one ticket before deletion");

        board.delete_ticket("T1")?;

        assert!(!t1_path.exists(), "Ticket directory should be deleted from Tickets/");
        assert!(deleted_dir.join("T1").exists(), "Ticket directory should be moved to Deleted/");
        assert!(!q1_path.join("T1_link").exists(), "Ticket symlink should be removed from the queue");

        let board_after = Board::load(root_dir)?;
        assert_eq!(board_after.queues[0].tickets.len(), 0, "Queue should be empty after deletion");

        Ok(())
    }

    #[test]
    fn test_create_ticket() -> anyhow::Result<()> {
        let root = tempdir()?;
        let root_path = root.path().to_path_buf();

        std::fs::create_dir_all(root_path.join("Tickets"))?;
        let q1_path = root_path.join("Queue").join("Q1");
        std::fs::create_dir_all(&q1_path)?;

        let board = Board::load(root_path.clone())?;
        let tid = board.create_ticket("My New Task", "My Description", "Q1")?;

        assert!(tid.starts_with("T-"), "New ticket ID should start with T-");
        assert!(root_path.join("Tickets").join(&tid).exists(), "Ticket directory should be created");
        assert!(root_path
            .join("Tickets")
            .join(&tid)
            .join("README.md")
            .exists(), "README.md should be created in ticket directory");
        assert!(q1_path.join(&tid).exists(), "Symlink to ticket should be created in the queue");

        let board2 = Board::load(root_path)?;
        assert_eq!(board2.queues[0].tickets.len(), 1, "Board should have one ticket after creation");
        assert_eq!(board2.queues[0].tickets[0].title, "My New Task", "Ticket title should match input");
        assert_eq!(board2.queues[0].tickets[0].description, "My Description", "Ticket description should match input");

        Ok(())
    }

    #[test]
    fn test_update_ticket() -> anyhow::Result<()> {
        let root = tempdir()?;
        let root_path = root.path().to_path_buf();

        std::fs::create_dir_all(root_path.join("Tickets"))?;
        std::fs::create_dir_all(root_path.join("Queue").join("Q1"))?;

        let board = Board::load(root_path.clone())?;
        let tid = board.create_ticket("Original", "Original Description", "Q1")?;

        board.update_ticket(&tid, "Updated Title", "Updated Description")?;

        let board2 = Board::load(root_path)?;
        let t = &board2.queues[0].tickets[0];
        assert_eq!(t.title, "Updated Title", "Updated title should match input");
        assert_eq!(t.description, "Updated Description", "Updated description should match input");

        Ok(())
    }
    #[test]
    fn test_initialization() -> anyhow::Result<()> {
        let root = tempdir()?;
        let root_path = root.path();

        // 1. Initial run: should create default queues with numbers
        Board::ensure_initialized(root_path)?;

        let board = Board::load(root_path.to_path_buf())?;
        assert_eq!(board.queues.len(), 7, "Default initialization should create 7 queues");
        assert_eq!(board.queues[0].id, "1. Incoming", "Queue 0 ID mismatch");
        assert_eq!(board.queues[1].id, "2. ToDo", "Queue 1 ID mismatch");
        assert_eq!(board.queues[2].id, "3. Doing", "Queue 2 ID mismatch");
        assert_eq!(board.queues[3].id, "4. Reviewing", "Queue 3 ID mismatch");
        assert_eq!(board.queues[4].id, "5. Testing", "Queue 4 ID mismatch");
        assert_eq!(board.queues[5].id, "6. Done", "Queue 5 ID mismatch");
        assert_eq!(board.queues[6].id, "7. Archive", "Queue 6 ID mismatch");

        // 2. Existing queue run: should NOT create defaults if something exists
        let root2 = tempdir()?;
        let root_path2 = root2.path();
        std::fs::create_dir_all(root_path2.join("Queue").join("CustomQueue"))?;

        Board::ensure_initialized(root_path2)?;
        assert!(root_path2.join("Queue/CustomQueue").exists(), "Custom queue should still exist");
        assert!(!root_path2.join("Queue/1. Incoming").exists(), "Default queues should not be created if some exist");

        Ok(())
    }

    #[test]
    fn test_queue_limit_creation() -> anyhow::Result<()> {
        let root = tempdir()?;
        let root_path = root.path().to_path_buf();

        Board::ensure_initialized(&root_path)?;
        let mut board = Board::load(root_path.clone())?;

        // Set limit to 1 for "2. ToDo"
        board.config.set_limit("2. ToDo".to_string(), 1);
        board.config.write(&root_path)?;

        // Reload board to pick up config change if necessary, or just use the current board
        // Board::load re-reads the config.
        let board = Board::load(root_path)?;

        // Create first ticket - should succeed
        board.create_ticket("Task 1", "Desc 1", "2. ToDo")?;

        // Create second ticket - should fail
        let result = board.create_ticket("Task 2", "Desc 2", "2. ToDo");
        assert!(result.is_err(), "Creation should fail as queue limit is reached");
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("has reached its limit"), "Error message should mention the queue limit");

        Ok(())
    }

    #[test]
    fn test_queue_limit_moving() -> anyhow::Result<()> {
        let root = tempdir()?;
        let root_path = root.path().to_path_buf();

        Board::ensure_initialized(&root_path)?;
        let mut board = Board::load(root_path.clone())?;

        // Set limit to 1 for "3. Doing"
        board.config.set_limit("3. Doing".to_string(), 1);
        board.config.write(&root_path)?;

        let board = Board::load(root_path)?;

        // Create two tickets in ToDo
        let tid1 = board.create_ticket("Task 1", "Desc 1", "2. ToDo")?;
        let tid2 = board.create_ticket("Task 2", "Desc 2", "2. ToDo")?;

        // Move first ticket to Doing - should succeed
        board.move_ticket(&tid1, "2. ToDo", "3. Doing")?;

        // Move second ticket to Doing - should fail
        let result = board.move_ticket(&tid2, "2. ToDo", "3. Doing");
        assert!(result.is_err(), "Moving should fail as target queue limit is reached");
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("has reached its limit"), "Error message should mention the queue limit");

        Ok(())
    }

    #[test]
    fn test_create_ticket_invalid_queue() -> anyhow::Result<()> {
        let root = tempdir()?;
        let root_dir = root.path().to_path_buf();
        Board::ensure_initialized(&root_dir)?;
        let board = Board::load(root_dir)?;
        
        let result = board.create_ticket("Title", "Desc", "NonExistentQueue");
        assert!(result.is_err(), "Ticket creation in a non-existent queue should return an error. Verify that the queue ID passed exists and is correctly handled in Board::create_ticket.");
        Ok(())
    }

    #[test]
    fn test_move_ticket_invalid_queue() -> anyhow::Result<()> {
        let root = tempdir()?;
        let root_dir = root.path().to_path_buf();
        Board::ensure_initialized(&root_dir)?;
        let board = Board::load(root_dir)?;
        
        let tid = board.create_ticket("Title", "Desc", "1. Incoming")?;
        let result = board.move_ticket(&tid, "1. Incoming", "NonExistentQueue");
        assert!(result.is_err(), "Moving a ticket to a non-existent queue should return an error. Check Board::move_ticket logic for handling invalid target queue IDs.");
        Ok(())
    }

    #[test]
    fn test_config_load_invalid_toml() -> anyhow::Result<()> {
        let root = tempdir()?;
        let root_path = root.path().to_path_buf();
        let config_path = root_path.join("config.toml");
        
        let mut f = File::create(config_path)?;
        write!(f, "invalid = toml = [")?;
        
        let result = Config::load(&root_path);
        assert!(result.is_err(), "Loading a configuration file with invalid TOML should return an error. Check Config::load error handling and toml::from_str integration.");
        Ok(())
    }

    #[test]
    fn test_delete_non_existent_ticket() -> anyhow::Result<()> {
        let root = tempdir()?;
        let root_dir = root.path().to_path_buf();
        Board::ensure_initialized(&root_dir)?;
        let board = Board::load(root_dir)?;
        
        let result = board.delete_ticket("NonExistentID");
        assert!(result.is_err(), "Deleting a ticket with a non-existent ID should return an error. Verify Board::delete_ticket checks if the ticket exists before attempting deletion.");
        Ok(())
    }

    #[test]
    fn test_load_ticket_missing_readme() -> anyhow::Result<()> {
        let root = tempdir()?;
        let ticket_path = root.path().join("T1");
        std::fs::create_dir(&ticket_path)?;
        
        let board = Board {
            tickets_path: root.path().join("Tickets"),
            queues_path: root.path().join("Queue"),
            queues: vec![],
            config: Config::default(),
        };
        let result = board.load_ticket(&ticket_path);
        assert!(result.is_err(), "Loading a ticket with missing README.md should return an error.");
        assert!(result.unwrap_err().to_string().contains("README.md not found"), "Error message should mention missing README.md");
        Ok(())
    }

    #[test]
    fn test_load_ticket_invalid_format() -> anyhow::Result<()> {
        let root = tempdir()?;
        let ticket_path = root.path().join("T1");
        std::fs::create_dir(&ticket_path)?;
        std::fs::write(ticket_path.join("README.md"), "Invalid format - no separators")?;
        
        let board = Board {
            tickets_path: root.path().join("Tickets"),
            queues_path: root.path().join("Queue"),
            queues: vec![],
            config: Config::default(),
        };
        let result = board.load_ticket(&ticket_path);
        assert!(result.is_err(), "Loading a ticket with invalid format (missing separators) should return an error.");
        assert!(result.unwrap_err().to_string().contains("Invalid ticket format"), "Error message should mention invalid ticket format");
        Ok(())
    }

    #[test]
    fn test_load_ticket_invalid_yaml() -> anyhow::Result<()> {
        let root = tempdir()?;
        let ticket_path = root.path().join("T1");
        std::fs::create_dir(&ticket_path)?;
        std::fs::write(ticket_path.join("README.md"), "---\ninvalid: yaml: [\n---\nBody")?;
        
        let board = Board {
            tickets_path: root.path().join("Tickets"),
            queues_path: root.path().join("Queue"),
            queues: vec![],
            config: Config::default(),
        };
        let result = board.load_ticket(&ticket_path);
        assert!(result.is_err(), "Loading a ticket with invalid YAML should return an error.");
        assert!(result.unwrap_err().to_string().contains("Failed to parse YAML"), "Error message should mention YAML parsing failure");
        Ok(())
    }

    #[test]
    fn test_resolve_queue_id() -> anyhow::Result<()> {
        let board = Board {
            tickets_path: PathBuf::new(),
            queues_path: PathBuf::new(),
            config: Config::default(),
            queues: vec![
                Queue { id: "Q1".to_string(), name: "Queue 1".to_string(), tickets: vec![], limit: None },
                Queue { id: "Q2".to_string(), name: "Queue 2".to_string(), tickets: vec![], limit: None },
            ],
        };

        assert_eq!(board.resolve_queue_id("Q1"), "Q1", "Direct ID should resolve to itself");
        assert_eq!(board.resolve_queue_id("index:0"), "Q1", "index:0 should resolve to the first queue");
        assert_eq!(board.resolve_queue_id("index:1"), "Q2", "index:1 should resolve to the second queue");
        assert_eq!(board.resolve_queue_id("index:5"), "Q2", "index:OOB should resolve to the last queue");
        assert_eq!(board.resolve_queue_id("random"), "random", "Non-index strings should resolve as-is");
        
        Ok(())
    }

    #[test]
    fn test_extract_references() {
        let t = Ticket {
            id: "T1".to_string(),
            title: "T".to_string(),
            created_at: "now".to_string(),
            updated_at: "now".to_string(),
            description: "Check #T-abc123 and #T-def456. Also #T-123 is too short, and #T-abcdef78 is too long but should extract #T-abcdef7. And #T-abc123 again.".to_string(),
        };
        let refs = t.extract_references();
        assert_eq!(refs.len(), 3, "Should extract exactly 3 unique valid references. Check extract_references logic.");
        assert!(refs.contains(&"#T-abc123".to_string()), "Should contain #T-abc123");
        assert!(refs.contains(&"#T-def456".to_string()), "Should contain #T-def456");
        assert!(refs.contains(&"#T-abcdef".to_string()), "Should contain #T-abcdef (first 8 chars of a longer match starting with #T-).");
    }

    #[test]
    fn test_find_ticket_by_id() {
        let board = Board {
            tickets_path: PathBuf::new(),
            queues_path: PathBuf::new(),
            config: Config::default(),
            queues: vec![
                Queue {
                    id: "Q1".to_string(),
                    name: "Q1".to_string(),
                    tickets: vec![
                        Ticket { id: "T1".to_string(), title: "T1".to_string(), created_at: "".to_string(), updated_at: "".to_string(), description: "".to_string() }
                    ],
                    limit: None,
                },
                Queue {
                    id: "Q2".to_string(),
                    name: "Q2".to_string(),
                    tickets: vec![
                        Ticket { id: "T2".to_string(), title: "T2".to_string(), created_at: "".to_string(), updated_at: "".to_string(), description: "".to_string() }
                    ],
                    limit: None,
                }
            ],
        };

        assert!(board.find_ticket_by_id("T1").is_some(), "Ticket T1 should be found in Q1. Ensure find_ticket_by_id iterates over all queues.");
        assert!(board.find_ticket_by_id("T2").is_some(), "Ticket T2 should be found in Q2. Ensure find_ticket_by_id iterates over all queues.");
        assert!(board.find_ticket_by_id("T3").is_none(), "Non-existent ticket T3 should not be found.");
    }
}

