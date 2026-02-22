//! config.rs
//!
//! Purpose: Handles application configuration, including queue limits, visibility, and search history.
//! Includes: Config struct and its persistence logic.
//! Constraints: Should not contain board-specific or ticket-specific logic.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Sprint {
    pub number: u32,
    pub name: String,
    pub start_date: String,
    pub end_date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Workflow {
    #[serde(default)]
    pub start_queues: Vec<String>,
    #[serde(default)]
    pub done_queues: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KanbanConfig {
    #[serde(default)]
    pub queue_limits: HashMap<String, usize>,
    #[serde(default = "default_users")]
    pub users: Vec<String>,
    #[serde(default)]
    pub sprints: Vec<Sprint>,
    #[serde(default)]
    pub workflows: HashMap<String, Workflow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfig {
    #[serde(default = "default_user")]
    pub active_user: String,
    pub machine_id: Option<String>,
    #[serde(default)]
    pub show_only_mine: bool,
    #[serde(default)]
    pub hidden_queues: Vec<String>,
    #[serde(default)]
    pub search_history: Vec<String>,
    #[serde(default)]
    pub date_from: String,
    #[serde(default)]
    pub date_to: String,
}

#[derive(Debug, Clone, Default)]
pub struct Config {
    pub kanban: KanbanConfig,
    pub user: UserConfig,
}

fn default_users() -> Vec<String> {
    vec!["<unassigned>".to_string(), "user".to_string()]
}

fn default_user() -> String {
    "user".to_string()
}

impl Default for KanbanConfig {
    fn default() -> Self {
        Self {
            queue_limits: HashMap::new(),
            users: default_users(),
            sprints: Vec::new(),
            workflows: HashMap::new(),
        }
    }
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            active_user: default_user(),
            machine_id: None,
            show_only_mine: false,
            hidden_queues: Vec::new(),
            search_history: Vec::new(),
            date_from: "".to_string(),
            date_to: "".to_string(),
        }
    }
}

impl Config {
    pub fn load(root_path: &std::path::Path) -> anyhow::Result<Self> {
        // 1. Load Kanban Config (Board root)
        let kanban_path = root_path.join("config.toml");
        let kanban: KanbanConfig = if kanban_path.exists() {
            let content = std::fs::read_to_string(&kanban_path)?;
            toml::from_str(&content).unwrap_or_default()
        } else {
            KanbanConfig::default()
        };

        // 2. Load User Config (~/.config/slint-kanban/user.toml)
        let user_path = Self::user_config_path();
        let mut user: UserConfig = if let Some(ref path) = user_path {
            if path.exists() {
                let content = std::fs::read_to_string(path)?;
                toml::from_str(&content).unwrap_or_default()
            } else {
                UserConfig::default()
            }
        } else {
            UserConfig::default()
        };

        // Ensure machine_id
        let mut should_save_user = false;
        if user.machine_id.is_none() || user.machine_id.as_ref().unwrap().is_empty() {
            user.machine_id = Some(Self::generate_machine_id());
            should_save_user = true;
        }

        if should_save_user {
            if let Some(path) = &user_path {
                if let Some(parent) = path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                if let Ok(content) = toml::to_string_pretty(&user) {
                    let _ = std::fs::write(path, content);
                }
            }
        }

        Ok(Config { kanban, user })
    }

    fn generate_machine_id() -> String {
        use rand::Rng;
        let charset: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
        let mut rng = rand::thread_rng();
        (0..8)
            .map(|_| {
                let idx = rng.gen_range(0..charset.len());
                charset[idx] as char
            })
            .collect()
    }

    pub fn user_config_path() -> Option<std::path::PathBuf> {
        dirs::config_dir().map(|p| p.join("slint-kanban").join("user.toml"))
    }

    // Facade methods for backward compatibility
    pub fn queue_limits(&self) -> &HashMap<String, usize> {
        &self.kanban.queue_limits
    }
    pub fn users(&self) -> &[String] {
        &self.kanban.users
    }
    pub fn active_user(&self) -> &str {
        &self.user.active_user
    }

    pub fn get_current_sprint(&self) -> Option<&Sprint> {
        let today = chrono::Local::now().naive_local().date().to_string();
        self.kanban
            .sprints
            .iter()
            .find(|s| s.start_date <= today && today <= s.end_date)
    }
    pub fn machine_id(&self) -> Option<&str> {
        self.user.machine_id.as_deref()
    }
    pub fn show_only_mine(&self) -> bool {
        self.user.show_only_mine
    }
    pub fn hidden_queues(&self) -> &Vec<String> {
        &self.user.hidden_queues
    }
    pub fn search_history(&self) -> &Vec<String> {
        &self.user.search_history
    }

    pub fn get_limit(&self, queue_id: &str) -> Option<usize> {
        self.kanban.queue_limits.get(queue_id).copied()
    }

    pub fn set_limit(&mut self, queue_id: String, limit: usize) {
        self.kanban.queue_limits.insert(queue_id, limit);
    }

    pub fn is_visible(&self, queue_id: &str) -> bool {
        !self.user.hidden_queues.contains(&queue_id.to_string())
    }

    pub fn set_visible(&mut self, queue_id: String, visible: bool) {
        if visible {
            self.user.hidden_queues.retain(|id| id != &queue_id);
        } else if !self.user.hidden_queues.contains(&queue_id) {
            self.user.hidden_queues.push(queue_id);
        }
    }

    pub fn add_search_to_history(&mut self, query: String) {
        if query.trim().is_empty() {
            return;
        }
        self.user.search_history.retain(|q| q != &query);
        self.user.search_history.insert(0, query);
        if self.user.search_history.len() > 10 {
            self.user.search_history.pop();
        }
    }

    pub fn remove_search_from_history(&mut self, query: &str) {
        self.user.search_history.retain(|q| q != query);
    }

    pub fn write(&self, root_path: &std::path::Path) -> anyhow::Result<()> {
        // Write Kanban config
        let kanban_path = root_path.join("config.toml");
        let kanban_content = toml::to_string_pretty(&self.kanban)?;
        std::fs::write(&kanban_path, kanban_content)?;

        // Write User config
        if let Some(user_path) = Self::user_config_path() {
            if let Some(parent) = user_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let user_content = toml::to_string_pretty(&self.user)?;
            std::fs::write(&user_path, user_content)?;
        }

        Ok(())
    }
}
