use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct CommentMetadata {
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub updated_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attachments: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Comment {
    pub id: String,
    pub metadata: CommentMetadata,
    pub content: String,
    pub references: Vec<String>,
}

impl Comment {
    pub fn extract_references(&self) -> Vec<String> {
        crate::model::utils::extract_ticket_references(&self.content)
    }

    pub fn load(path: &std::path::Path) -> anyhow::Result<Self> {
        let id_with_ext = path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid comment path: {:?}", path))?;

        let id = id_with_ext.trim_end_matches(".md").to_string();

        let content_raw = std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("Failed to read comment {:?}: {}", path, e))?;

        let parts: Vec<&str> = content_raw.splitn(3, "---").collect();
        let (metadata, content) = if parts.len() >= 3 {
            let frontmatter = parts[1];
            let body = parts[2].trim().to_string();
            let mut meta: CommentMetadata = serde_yaml::from_str(frontmatter)
                .map_err(|e| anyhow::anyhow!("Failed to parse YAML in {:?}: {}", path, e))?;

            if meta.updated_at.is_empty() && !meta.created_at.is_empty() {
                meta.updated_at = meta.created_at.clone();
            }
            (meta, body)
        } else {
            let meta = CommentMetadata {
                author: "Unknown".to_string(),
                created_at: "".to_string(),
                updated_at: "".to_string(),
                attachments: None,
            };
            (meta, content_raw.trim().to_string())
        };

        let mut comment = Self {
            id,
            metadata,
            content,
            references: vec![],
        };
        comment.references = comment.extract_references();
        Ok(comment)
    }

    pub fn save(&self, path: &std::path::Path) -> anyhow::Result<()> {
        let mut f = std::fs::File::create(path)?;
        let frontmatter = serde_yaml::to_string(&self.metadata)?;
        use std::io::Write;
        write!(
            f,
            "---\n{}---\n{}",
            frontmatter.trim_start_matches("---\n"),
            self.content
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_references_from_comment() {
        let comment = Comment {
            id: "tc001abc".to_string(),
            metadata: CommentMetadata::default(),
            content: "Check this out: #abc123 and #def456 but not #123 (too short)".to_string(),
            references: vec![],
        };
        let refs = comment.extract_references();
        assert_eq!(
            refs,
            vec!["#abc123", "#def456"],
            "Should extract exactly two valid references"
        );
    }

    #[test]
    fn test_load_comment_valid() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tc001abc.md");
        std::fs::write(
            &path,
            "---\nauthor: User\ncreated_at: 2026-01-01\n---\nHello #abc123!",
        )
        .unwrap();

        let comment = Comment::load(&path).unwrap();
        assert_eq!(comment.id, "tc001abc");
        assert_eq!(comment.metadata.author, "User");
        assert_eq!(comment.metadata.created_at, "2026-01-01");
        assert_eq!(comment.metadata.updated_at, "2026-01-01");
        assert_eq!(comment.content, "Hello #abc123!");
        assert_eq!(comment.references, vec!["#abc123"]);
    }
}
