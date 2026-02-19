//! config.rs
//!
//! Purpose: Handles application configuration, including queue limits, visibility, and search history.
//! Includes: Config struct and its persistence logic.
//! Constraints: Should not contain board-specific or ticket-specific logic.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub queue_limits: HashMap<String, usize>,
    #[serde(default)]
    pub hidden_queues: Vec<String>,
    #[serde(default)]
    pub search_history: Vec<String>,
    #[serde(default = "default_users")]
    pub users: Vec<String>,
    #[serde(default = "default_user")]
    pub active_user: String,
    #[serde(default)]
    pub show_only_mine: bool,
}

fn default_users() -> Vec<String> {
    vec!["<unassigned>".to_string(), "user".to_string()]
}

fn default_user() -> String {
    "user".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            queue_limits: HashMap::new(),
            hidden_queues: Vec::new(),
            search_history: Vec::new(),
            users: default_users(),
            active_user: default_user(),
            show_only_mine: false,
        }
    }
}

impl Config {
    pub fn load(root_path: &std::path::Path) -> anyhow::Result<Self> {
        let config_path = root_path.join("config.toml");

        if !config_path.exists() {
            // No config file yet; use defaults
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

    pub fn is_visible(&self, queue_id: &str) -> bool {
        !self.hidden_queues.contains(&queue_id.to_string())
    }

    pub fn set_visible(&mut self, queue_id: String, visible: bool) {
        if visible {
            self.hidden_queues.retain(|id| id != &queue_id);
        } else if !self.hidden_queues.contains(&queue_id) {
            self.hidden_queues.push(queue_id);
        }
    }

    pub fn add_search_to_history(&mut self, query: String) {
        if query.trim().is_empty() {
            return;
        }
        // Remove if exists to move to top
        self.search_history.retain(|q| q != &query);
        self.search_history.insert(0, query);
        // Cap history at 10 entries
        if self.search_history.len() > 10 {
            self.search_history.pop();
        }
    }

    pub fn remove_search_from_history(&mut self, query: &str) {
        self.search_history.retain(|q| q != query);
    }

    pub fn write(&self, root_path: &std::path::Path) -> anyhow::Result<()> {
        let config_path = root_path.join("config.toml");
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&config_path, content)?;
        Ok(())
    }
}
