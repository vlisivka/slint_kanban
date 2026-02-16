use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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
}

#[derive(Debug, Clone)]
pub struct Board {
    pub queues: Vec<Queue>,
    pub tickets_path: PathBuf,
    pub queues_path: PathBuf,
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
                "4. Done",
                "5. Archive",
            ];
            for q in default_queues {
                std::fs::create_dir_all(queues_path.join(q))?;
            }
        }

        Ok(())
    }

    pub fn load(root_path: PathBuf) -> anyhow::Result<Self> {
        let queues_path = root_path.join("Queue");
        let tickets_path = root_path.join("Tickets");

        if !queues_path.exists() {
            return Ok(Board {
                queues: vec![],
                tickets_path,
                queues_path,
            });
        }

        let mut queues = vec![];
        for entry in std::fs::read_dir(&queues_path)?.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let queue_id = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or_default()
                    .to_string();
                let mut tickets = vec![];

                for ticket_entry in std::fs::read_dir(&path)?.flatten() {
                    let ticket_path = ticket_entry.path();

                    // In the architecture, tickets in queues are symlinks to ../../Tickets/<id>
                    // We need to resolve the symlink or read the target directory.
                    // The spec says "Tickets: Symlinks from a queue directory to ~/Kanban/Tickets".
                    // So we treat the entry as a ticket if it resolves to a directory containing README.md

                    let real_path = if ticket_path.is_symlink() {
                        std::fs::read_link(&ticket_path)?
                    } else {
                        ticket_path.clone()
                    };

                    // If relative symlink, verify it resolves relative to the ticket_path's parent?
                    // std::fs::read_link returns the content of the symlink.
                    // If it's absolute, fine. If relative, we need to join it.
                    let resolved_path = if real_path.is_relative() {
                        path.join(&real_path).canonicalize()?
                    } else {
                        real_path
                    };

                    if resolved_path.exists() && resolved_path.is_dir() {
                        let readme_path = resolved_path.join("README.md");
                        if readme_path.exists() {
                            let content = std::fs::read_to_string(&readme_path)?;
                            let parts: Vec<&str> = content.splitn(3, "---").collect();
                            if parts.len() >= 3 {
                                let frontmatter = parts[1];
                                let body = parts[2].trim().to_string();
                                let mut metadata: TicketMetadata =
                                    serde_yaml::from_str(frontmatter).unwrap_or(TicketMetadata {
                                        title: "Error parsing YAML".to_string(),
                                        created_at: "".to_string(),
                                        updated_at: "".to_string(),
                                    });

                                if metadata.updated_at.is_empty() && !metadata.created_at.is_empty()
                                {
                                    metadata.updated_at = metadata.created_at.clone();
                                }

                                let ticket_id = resolved_path
                                    .file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or_default()
                                    .to_string();
                                tickets.push(Ticket::from_metadata(ticket_id, metadata, body));
                            }
                        }
                    }
                }
                queues.push(Queue {
                    id: queue_id.clone(),
                    name: queue_id, // Use ID as name for now
                    tickets,
                });
            }
        }

        // Sort queues alphabetically by name
        queues.sort_by(|a, b| a.id.cmp(&b.id));

        Ok(Board {
            queues,
            tickets_path,
            queues_path,
        })
    }

    pub fn move_ticket(
        &self,
        ticket_id: &str,
        source_queue_id: &str,
        target_queue_id: &str,
    ) -> anyhow::Result<()> {
        let source_queue_path = self.queues_path.join(source_queue_id);
        let target_queue_path = self.queues_path.join(target_queue_id);
        let ticket_target_path = self.tickets_path.join(ticket_id).canonicalize()?;

        if !source_queue_path.exists() || !target_queue_path.exists() {
            return Err(anyhow::anyhow!("Source or target queue not found"));
        }

        // Find the symlink in source_queue that points to target_ticket_path
        let mut entry_to_move = None;
        for entry in std::fs::read_dir(&source_queue_path)?.flatten() {
            let path = entry.path();

            let real_path = if path.is_symlink() {
                std::fs::read_link(&path)?
            } else {
                path.clone()
            };

            let resolved_path = if real_path.is_relative() {
                source_queue_path.join(&real_path).canonicalize()?
            } else {
                real_path.canonicalize()?
            };

            if resolved_path == ticket_target_path {
                entry_to_move = Some(path);
                break;
            }
        }

        if let Some(source_path) = entry_to_move {
            let file_name = source_path
                .file_name()
                .ok_or_else(|| anyhow::anyhow!("Invalid file name"))?;
            let dest_path = target_queue_path.join(file_name);
            std::fs::rename(source_path, dest_path)?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Ticket not found in source queue"))
        }
    }

    pub fn delete_ticket(&self, ticket_id: &str) -> anyhow::Result<()> {
        let ticket_path = self.tickets_path.join(ticket_id);
        if !ticket_path.exists() {
            return Err(anyhow::anyhow!("Ticket not found"));
        }

        let deleted_root = self.tickets_path.parent().unwrap().join("Deleted");
        if !deleted_root.exists() {
            std::fs::create_dir_all(&deleted_root)?;
        }

        // 1. Move the ticket folder to Deleted
        let dest_path = deleted_root.join(ticket_id);

        // If it already exists in Deleted, we might want to overwrite or version it?
        // For now, let's just remove old deletion if it exists.
        if dest_path.exists() {
            std::fs::remove_dir_all(&dest_path)?;
        }
        std::fs::rename(&ticket_path, &dest_path)?;

        // 2. Cleanup symlinks in all queues
        for entry in std::fs::read_dir(&self.queues_path)?.flatten() {
            let queue_dir = entry.path();
            if queue_dir.is_dir() {
                for ticket_entry in std::fs::read_dir(&queue_dir)?.flatten() {
                    let symlink_path = ticket_entry.path();

                    let is_target = if symlink_path.is_symlink() {
                        if let Ok(target) = std::fs::read_link(&symlink_path) {
                            // Match either absolute or relative to the symlink's parent
                            let resolved = if target.is_relative() {
                                queue_dir.join(&target).canonicalize().unwrap_or(target)
                            } else {
                                target.canonicalize().unwrap_or(target)
                            };
                            resolved == dest_path || resolved == ticket_path
                        } else {
                            false
                        }
                    } else {
                        false
                    };

                    if is_target {
                        std::fs::remove_file(symlink_path)?;
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

        // 1. Generate unique ID
        let id: String = std::iter::repeat(())
            .map(|()| rand::thread_rng().sample(Alphanumeric))
            .map(char::from)
            .take(6)
            .collect();
        let ticket_id = format!("T-{}", id);

        // 2. Create ticket directory
        let ticket_dir = self.tickets_path.join(&ticket_id);
        if ticket_dir.exists() {
            return self.create_ticket(title, description, queue_id);
        }
        std::fs::create_dir_all(&ticket_dir)?;

        // 3. Create README.md with metadata
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let mut f = std::fs::File::create(ticket_dir.join("README.md"))?;
        use std::io::Write;
        write!(
            f,
            "---\ntitle: {}\ncreated_at: {}\nupdated_at: {}\n---\n{}",
            title, now, now, description
        )?;

        // 4. Create symlink in the target queue
        let queue_path = self.queues_path.join(queue_id);
        if !queue_path.exists() {
            return Err(anyhow::anyhow!("Queue {} not found", queue_id));
        }

        #[cfg(unix)]
        std::os::unix::fs::symlink(&ticket_dir, queue_path.join(&ticket_id))?;

        Ok(ticket_id)
    }

    pub fn update_ticket(
        &self,
        ticket_id: &str,
        title: &str,
        description: &str,
    ) -> anyhow::Result<()> {
        let ticket_dir = self.tickets_path.join(ticket_id);
        let readme_path = ticket_dir.join("README.md");
        if !readme_path.exists() {
            return Err(anyhow::anyhow!("Ticket {} not found", ticket_id));
        }

        let content = std::fs::read_to_string(&readme_path)?;
        let parts: Vec<&str> = content.splitn(3, "---").collect();
        let created_at = if parts.len() >= 3 {
            let metadata: TicketMetadata = serde_yaml::from_str(parts[1])?;
            metadata.created_at
        } else {
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
        };

        let updated_at = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        let mut f = std::fs::File::create(&readme_path)?;
        use std::io::Write;
        write!(
            f,
            "---\ntitle: {}\ncreated_at: {}\nupdated_at: {}\n---\n{}",
            title, created_at, updated_at, description
        )?;

        Ok(())
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
        assert_eq!(metadata.title, "Buy Groceries");
        assert_eq!(metadata.created_at, "2023-10-27");
        assert_eq!(metadata.updated_at, "2023-10-27");
    }

    #[test]
    fn test_ticket_metadata_missing_updated_at() {
        let yaml = "
title: Buy Groceries
created_at: 2023-10-27
";
        let metadata: TicketMetadata = serde_yaml::from_str(yaml).expect("Failed to parse YAML");
        assert_eq!(metadata.title, "Buy Groceries");
        assert_eq!(metadata.created_at, "2023-10-27");
        assert_eq!(metadata.updated_at, "");
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
        assert_eq!(board.queues.len(), 1);
        let q1 = &board.queues[0];
        assert_eq!(q1.id, "Q1");
        assert_eq!(q1.tickets.len(), 1);
        assert_eq!(q1.tickets[0].title, "Task 1");
        assert_eq!(q1.tickets[0].id, "T1");

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
        assert_eq!(q1.tickets.len(), 1);
        assert_eq!(q1.tickets[0].id, "ttt123");

        let q2 = board
            .queues
            .iter()
            .find(|q| q.id == "q2")
            .expect("q2 not found");
        assert_eq!(q2.tickets.len(), 1);
        assert_eq!(q2.tickets[0].id, "ttt456");

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
            1
        );
        assert_eq!(
            board
                .queues
                .iter()
                .find(|q| q.id == "Q2")
                .unwrap()
                .tickets
                .len(),
            0
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
            0
        );
        assert_eq!(
            board_after
                .queues
                .iter()
                .find(|q| q.id == "Q2")
                .unwrap()
                .tickets
                .len(),
            1
        );
        assert_eq!(
            board_after
                .queues
                .iter()
                .find(|q| q.id == "Q2")
                .unwrap()
                .tickets[0]
                .id,
            "T1"
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
        assert_eq!(board.queues[0].tickets.len(), 1);

        board.delete_ticket("T1")?;

        assert!(!t1_path.exists());
        assert!(deleted_dir.join("T1").exists());
        assert!(!q1_path.join("T1_link").exists());

        let board_after = Board::load(root_dir)?;
        assert_eq!(board_after.queues[0].tickets.len(), 0);

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

        assert!(tid.starts_with("T-"));
        assert!(root_path.join("Tickets").join(&tid).exists());
        assert!(root_path
            .join("Tickets")
            .join(&tid)
            .join("README.md")
            .exists());
        assert!(q1_path.join(&tid).exists());

        let board2 = Board::load(root_path)?;
        assert_eq!(board2.queues[0].tickets.len(), 1);
        assert_eq!(board2.queues[0].tickets[0].title, "My New Task");
        assert_eq!(board2.queues[0].tickets[0].description, "My Description");

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
        assert_eq!(t.title, "Updated Title");
        assert_eq!(t.description, "Updated Description");

        Ok(())
    }
    #[test]
    fn test_initialization() -> anyhow::Result<()> {
        let root = tempdir()?;
        let root_path = root.path();

        // 1. Initial run: should create default queues with numbers
        Board::ensure_initialized(root_path)?;

        let board = Board::load(root_path.to_path_buf())?;
        assert_eq!(board.queues.len(), 5);
        assert_eq!(board.queues[0].id, "1. Incoming");
        assert_eq!(board.queues[1].id, "2. ToDo");
        assert_eq!(board.queues[2].id, "3. Doing");
        assert_eq!(board.queues[3].id, "4. Done");
        assert_eq!(board.queues[4].id, "5. Archive");

        // 2. Existing queue run: should NOT create defaults if something exists
        let root2 = tempdir()?;
        let root_path2 = root2.path();
        std::fs::create_dir_all(root_path2.join("Queue").join("CustomQueue"))?;

        Board::ensure_initialized(root_path2)?;
        assert!(root_path2.join("Queue/CustomQueue").exists());
        assert!(!root_path2.join("Queue/1. Incoming").exists());

        Ok(())
    }
}
