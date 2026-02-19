//! ticket.rs
//!
//! Purpose: Defines the Ticket structure and metadata, including parsing and search matching.
//! Includes: Ticket and TicketMetadata structs.
//! Constraints: Should not contain logic for managing multiple tickets or queues.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketMetadata {
    pub title: String,
    #[serde(default)]
    pub created_at: String, // ISO 8601 or similar
    #[serde(default)]
    pub updated_at: String,
    #[serde(default)]
    pub assigned_to: String,
}

#[derive(Debug, Clone)]
pub struct Ticket {
    pub id: String,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
    pub description: String,
    pub assigned_to: String,
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
        }
    }

    pub fn extract_references(&self) -> Vec<String> {
        let mut refs = Vec::new();
        let mut start = 0;
        while let Some(pos) = self.description[start..].find('#') {
            let actual_pos = start + pos;
            if actual_pos + 7 <= self.description.len() {
                let potential_id = &self.description[actual_pos + 1..actual_pos + 7];
                // Check if it's 6 lowercase alphanumeric chars
                if potential_id
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
                {
                    refs.push(format!("#{}", potential_id));
                }
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
        self.title.to_lowercase().contains(&query_lower)
            || self.description.to_lowercase().contains(&query_lower)
            || self.id.to_lowercase().contains(&query_lower)
    }

    pub fn matches_date_range(&self, from: &str, to: &str) -> bool {
        if !from.is_empty() && self.created_at.as_str() < from {
            return false;
        }
        if !to.is_empty() && self.created_at.as_str() > to && !self.created_at.starts_with(to) {
            return false;
        }
        true
    }

    pub fn load(path: &std::path::Path) -> anyhow::Result<Self> {
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

    pub fn save(&self, path: &std::path::Path) -> anyhow::Result<()> {
        if !path.exists() {
            std::fs::create_dir_all(path)?;
        }
        let readme_path = path.join("README.md");
        let mut f = std::fs::File::create(&readme_path)?;
        use std::io::Write;
        write!(
            f,
            "---\ntitle: {}\ncreated_at: {}\nupdated_at: {}\nassigned_to: \"{}\"\n---\n{}",
            self.title, self.created_at, self.updated_at, self.assigned_to, self.description
        )?;
        Ok(())
    }
}
