//! ticket.rs
//!
//! Purpose: Defines the Ticket structure and metadata, including parsing and search matching.
//! Includes: Ticket and TicketMetadata structs.
//! Constraints: Should not contain logic for managing multiple tickets or queues.

use serde::{Deserialize, Serialize};
use tr::tr;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TicketMetadata {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub created_at: String, // "YYYY-MM-DD HH:MM:SS" format
    #[serde(default)]
    pub updated_at: String,
    #[serde(default)]
    pub assigned_to: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub points: u32,
    #[serde(default)]
    pub attachment_count: u32,
}

#[derive(Debug, Clone)]
pub struct Ticket {
    pub id: String,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
    pub description: String,
    pub assigned_to: String,
    pub author: String,
    pub points: u32,
    pub attachment_count: u32,
    pub comments: Vec<crate::model::Comment>,
}

impl Ticket {
    pub fn from_metadata(id: String, metadata: TicketMetadata, description: String) -> Self {
        Self {
            id,
            title: metadata.title,
            created_at: metadata.created_at,
            updated_at: metadata.updated_at,
            description,
            assigned_to: metadata.assigned_to,
            author: metadata.author,
            points: metadata.points,
            attachment_count: metadata.attachment_count,
            comments: Vec::new(),
        }
    }

    /// Finds ticket cross-references in the description text.
    /// References use the format `#xxxxxx` where x is a 6-char alphanumeric ticket ID.
    pub fn extract_references(&self) -> Vec<String> {
        let mut refs = Vec::new();
        let mut start = 0;
        while let Some((char_idx, _ch)) = self.description[start..]
            .char_indices()
            .find(|(_, c)| *c == '#')
        {
            let actual_pos = start + char_idx;
            // Collect the next 6 characters after '#'
            let after_hash: String = self.description[actual_pos + 1..].chars().take(6).collect();
            if after_hash.len() == 6
                && after_hash
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
            {
                refs.push(format!("#{}", after_hash));
            }
            start = actual_pos + 1;
        }
        refs.sort();
        refs.dedup();
        refs
    }

    pub fn matches(&self, query: &str) -> bool {
        if query.is_empty() {
            return true;
        }
        let query_lower = query.to_lowercase();
        let query_clean = if let Some(stripped) = query_lower.strip_prefix('#') {
            stripped
        } else {
            &query_lower
        };

        self.title.to_lowercase().contains(&query_lower)
            || self.description.to_lowercase().contains(&query_lower)
            || self.id.to_lowercase().contains(query_clean)
    }

    /// Checks whether this ticket's created_at falls within [from, to].
    /// Comparison is lexicographic, which works for "YYYY-MM-DD" prefixes.
    /// The `starts_with(to)` check allows matching same-day tickets when
    /// `to` contains only a date and `created_at` includes a time component.
    pub fn matches_date_range(&self, from: &str, to: &str) -> bool {
        if !from.is_empty() && self.created_at.as_str() < from {
            return false;
        }
        if !to.is_empty() && self.created_at.as_str() > to && !self.created_at.starts_with(to) {
            return false;
        }
        true
    }

    /// Combined filter: search query + date range + user assignment.
    /// Use `assigned_to_filter` = `Some("username")` to filter by user,
    /// or `None` to skip user filtering.
    pub fn matches_all(
        &self,
        query: &str,
        date_from: &str,
        date_to: &str,
        assigned_to_filter: Option<&str>,
    ) -> bool {
        self.matches(query)
            && self.matches_date_range(date_from, date_to)
            && assigned_to_filter.is_none_or(|user| self.assigned_to == user)
    }

    /// Loads a ticket from its directory. Expected format of README.md:
    /// ```text
    /// ---
    /// title: ...
    /// created_at: YYYY-MM-DD HH:MM:SS
    /// updated_at: YYYY-MM-DD HH:MM:SS
    /// assigned_to: "..."
    /// ---
    /// <markdown body>
    /// ```
    /// Loads a ticket header and body from its directory, WITHOUT comments.
    pub fn load(path: &std::path::Path) -> anyhow::Result<Self> {
        let ticket_id = path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow::anyhow!(tr!("Invalid ticket path: {}", path.display())))?
            .to_string();

        let readme_path = path.join("README.md");
        if !readme_path.exists() {
            return Err(anyhow::anyhow!(tr!(
                "README.md not found in {}",
                path.display()
            )));
        }

        let file = std::fs::File::open(&readme_path).map_err(|e| {
            anyhow::anyhow!(tr!("Failed to open README.md in {}: {}", path.display(), e))
        })?;
        let reader = std::io::BufReader::new(file);
        use std::io::BufRead;

        let mut frontmatter = String::new();
        let mut body_snippet = String::new();
        let mut state = 0; // 0: before first ---, 1: inside frontmatter, 2: after second --- (body)

        for line in reader.lines() {
            let line = line?;
            if line.trim() == "---" {
                state += 1;
                if state == 3 {
                    break;
                }
                continue;
            }

            match state {
                1 => {
                    frontmatter.push_str(&line);
                    frontmatter.push('\n');
                }
                2 => {
                    if !line.trim().is_empty() {
                        body_snippet = line.trim().to_string();
                        break;
                    }
                }
                _ => {}
            }
        }

        if state < 2 {
            return Err(anyhow::anyhow!(tr!(
                "Invalid ticket format (missing frontmatter) in {}",
                readme_path.display()
            )));
        }

        let mut metadata: TicketMetadata = serde_yaml::from_str(&frontmatter).map_err(|e| {
            anyhow::anyhow!(tr!(
                "Failed to parse YAML in {}: {}",
                readme_path.display(),
                e
            ))
        })?;

        // Backfill updated_at for tickets created before this field was added
        if metadata.updated_at.is_empty() && !metadata.created_at.is_empty() {
            metadata.updated_at = metadata.created_at.clone();
        }

        let ticket = Ticket::from_metadata(ticket_id, metadata, body_snippet);
        Ok(ticket)
    }

    /// Loads comments for an already loaded ticket.
    pub fn load_comments(&mut self, path: &std::path::Path) -> anyhow::Result<()> {
        let mut comments = Vec::new();
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().into_owned();
                if name.starts_with("tc") && name.ends_with(".md") {
                    if let Ok(comment) = crate::model::Comment::load(&entry.path()) {
                        comments.push(comment);
                    }
                }
            }
        }
        comments.sort_by(|a, b| a.metadata.created_at.cmp(&b.metadata.created_at));
        self.comments = comments;
        Ok(())
    }

    /// Loads a ticket from its directory, including full body and all comments.
    pub fn load_full(path: &std::path::Path) -> anyhow::Result<Self> {
        let ticket_id = path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow::anyhow!(tr!("Invalid ticket path: {}", path.display())))?
            .to_string();

        let readme_path = path.join("README.md");
        if !readme_path.exists() {
            return Err(anyhow::anyhow!(tr!(
                "README.md not found in {}",
                path.display()
            )));
        }

        let content = std::fs::read_to_string(&readme_path).map_err(|e| {
            anyhow::anyhow!(tr!("Failed to read README.md in {}: {}", path.display(), e))
        })?;

        let parts: Vec<&str> = content.splitn(3, "---").collect();
        if parts.len() < 3 {
            return Err(anyhow::anyhow!(tr!(
                "Invalid ticket format (missing frontmatter) in {}",
                readme_path.display()
            )));
        }

        let frontmatter = parts[1];
        let body = parts[2].trim().to_string();

        let mut metadata: TicketMetadata = serde_yaml::from_str(frontmatter).map_err(|e| {
            anyhow::anyhow!(tr!(
                "Failed to parse YAML in {}: {}",
                readme_path.display(),
                e
            ))
        })?;

        // Backfill updated_at for tickets created before this field was added
        if metadata.updated_at.is_empty() && !metadata.created_at.is_empty() {
            metadata.updated_at = metadata.created_at.clone();
        }

        let mut ticket = Ticket::from_metadata(ticket_id, metadata, body);
        ticket.load_comments(path)?;
        Ok(ticket)
    }

    pub fn save(&self, path: &std::path::Path) -> anyhow::Result<()> {
        if !path.exists() {
            std::fs::create_dir_all(path)?;
        }
        let readme_path = path.join("README.md");
        let mut f = std::fs::File::create(&readme_path)?;
        use std::io::Write;
        write!(
            f,
            "---\ntitle: \"{}\"\ncreated_at: {}\nupdated_at: {}\nassigned_to: \"{}\"\nauthor: \"{}\"\npoints: {}\nattachment_count: {}\n---\n{}",
            self.title, self.created_at, self.updated_at, self.assigned_to, self.author, self.points, self.attachment_count, self.description
        )?;
        Ok(())
    }
}
