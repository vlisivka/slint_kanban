//! controller.rs
//!
//! Purpose: Encapsulates the application logic and state, acting as the bridge between the UI (App) and the Data Model (Board).
//! Includes: AppController struct and methods for handling UI actions.

use crate::model::{Board, Config};
use crate::{sync_ui_with_board, App, UserGlobal};
use slint::{ComponentHandle, Model, SharedString, VecModel, Weak};
use std::path::PathBuf;
use std::rc::Rc;

/// Mediates between the Slint UI and the file-system Board model.
/// Each action handler re-loads the board from disk to ensure consistency
/// (the board is the source of truth, not in-memory state).
pub struct AppController {
    app_weak: Weak<App>,
    pub root_path: PathBuf,
}

impl AppController {
    pub fn new(app_weak: Weak<App>, root_path: PathBuf) -> Self {
        Self {
            app_weak,
            root_path,
        }
    }

    /// Reloads the board from disk and synchronizes the UI.
    /// This is used for initial load and when the file watcher detects changes.
    pub fn reload(&self) -> anyhow::Result<()> {
        let app = self
            .app_weak
            .upgrade()
            .ok_or_else(|| anyhow::anyhow!("UI dropped"))?;
        let board = Board::load(self.root_path.clone())?;

        // 1. Sync Board Data (Queues & Tickets)
        let query = app.get_search_query();
        let date_from = app.get_date_from();
        let date_to = app.get_date_to();
        let user_global = app.global::<UserGlobal>();

        let show_only_mine = board.config.show_only_mine();
        let active_user = board.config.active_user();

        sync_ui_with_board(
            &app,
            &board,
            query.as_str(),
            date_from.as_str(),
            date_to.as_str(),
            show_only_mine,
            active_user,
        );

        // 2. Sync Users — only update the model when the list actually changed,
        //    because resetting VecModel resets ComboBox selection and causes flicker.
        let mut new_users: Vec<SharedString> = Vec::new();
        // Ensure <unassigned> is always available
        if !board.config.users().iter().any(|u| u == "<unassigned>") {
            new_users.push(SharedString::from("<unassigned>"));
        }
        new_users.extend(board.config.users().iter().map(|s| SharedString::from(s)));

        let current_users_model = user_global.get_users();
        let users_changed = if current_users_model.row_count() != new_users.len() {
            true
        } else {
            current_users_model
                .iter()
                .zip(new_users.iter())
                .any(|(a, b)| a != *b)
        };

        if users_changed {
            println!("Controller: Updating users list in UI...");
            user_global.set_users(Rc::new(VecModel::from(new_users)).into());
        }

        // 3. Sync Active User (ensure consistency)
        let new_active_user = SharedString::from(board.config.active_user());
        if user_global.get_active_user() != new_active_user {
            user_global.set_active_user(new_active_user);
        }

        // 4. Sync Other Config
        user_global.set_show_only_mine(board.config.show_only_mine());

        let history: Vec<SharedString> = board
            .config
            .search_history()
            .iter()
            .map(|s| SharedString::from(s))
            .collect();
        app.set_search_history(Rc::new(VecModel::from(history)).into());

        Ok(())
    }

    // --- Action Handlers ---

    pub fn handle_move(&self, ticket_id: String, source_id: String, target_id: String) {
        let board = match Board::load(self.root_path.clone()) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("Error loading board for move: {:?}", e);
                return;
            }
        };

        let resolved_target_id = board.resolve_queue_id(&target_id);
        // Ignore no-op drops (same queue)
        if source_id == resolved_target_id {
            return;
        }

        println!(
            "Controller: Moving {} from {} to {}",
            ticket_id, source_id, resolved_target_id
        );
        if let Err(e) = board.move_ticket(&ticket_id, &source_id, &resolved_target_id) {
            self.show_error(&e.to_string());
        }
    }

    pub fn handle_delete(&self, ticket_id: String) {
        let board = match Board::load(self.root_path.clone()) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("Error loading board for delete: {:?}", e);
                return;
            }
        };

        println!("Controller: Deleting ticket {}", ticket_id);
        if let Err(e) = board.delete_ticket(&ticket_id) {
            eprintln!("Error deleting: {:?}", e);
        }
    }

    pub fn handle_create(
        &self,
        queue_id: String,
        title: String,
        description: String,
        assigned_to: String,
    ) {
        let board = match Board::load(self.root_path.clone()) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("Error loading board for create: {:?}", e);
                return;
            }
        };

        println!("Controller: Creating ticket in {}", queue_id);
        if let Err(e) = board.create_ticket(&title, &description, &queue_id, &assigned_to) {
            self.show_error(&e.to_string());
        }
    }

    pub fn handle_save(
        &self,
        ticket_id: String,
        title: String,
        description: String,
        assigned_to: String,
    ) {
        let board = match Board::load(self.root_path.clone()) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("Error loading board for save: {:?}", e);
                return;
            }
        };

        println!("Controller: Saving ticket {}", ticket_id);
        if let Err(e) = board.update_ticket(&ticket_id, &title, &description, &assigned_to) {
            eprintln!("Error saving ticket: {:?}", e);
        }
    }

    pub fn handle_change_limit(&self, queue_id: String, limit: i32) {
        let mut board = match Board::load(self.root_path.clone()) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("Error loading board for limit: {:?}", e);
                return;
            }
        };

        // limit < 0 from UI means "remove limit" (the Slint sends -1)
        if limit < 0 {
            board.config.kanban.queue_limits.remove(&queue_id);
        } else {
            board.config.set_limit(queue_id, limit as usize);
        }

        if let Err(e) = board.config.write(&self.root_path) {
            eprintln!("Error saving config: {:?}", e);
        }
    }

    pub fn handle_user_change(&self, username: String) {
        let mut config = match Config::load(&self.root_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error loading config for user change: {:?}", e);
                return;
            }
        };
        config.user.active_user = username.clone();
        if let Err(e) = config.write(&self.root_path) {
            eprintln!("Error writing config: {:?}", e);
        }

        // Update UI immediately so the user doesn't see stale state while
        // waiting for the file watcher to trigger a full reload.
        if let Some(app) = self.app_weak.upgrade() {
            app.global::<UserGlobal>()
                .set_active_user(SharedString::from(username));
        }
    }

    pub fn handle_toggle_mine(&self, enabled: bool) {
        let mut config = match Config::load(&self.root_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error loading config: {:?}", e);
                return;
            }
        };
        config.user.show_only_mine = enabled;
        if let Err(e) = config.write(&self.root_path) {
            eprintln!("Error writing config: {:?}", e);
        }

        // Update UI immediately so the user doesn't see stale state while
        // waiting for the file watcher to trigger a full reload.
        if let Some(app) = self.app_weak.upgrade() {
            app.global::<UserGlobal>().set_show_only_mine(enabled);
            let _ = self.reload();
        }
    }

    pub fn handle_queue_visibility(&self, queue_id: String, visible: bool) {
        let mut config = match Config::load(&self.root_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error loading config: {:?}", e);
                return;
            }
        };
        config.set_visible(queue_id, visible);
        if let Err(e) = config.write(&self.root_path) {
            eprintln!("Error writing config: {:?}", e);
        }
    }

    pub fn handle_search_history_add(&self, query: String) {
        let mut config = match Config::load(&self.root_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error config: {:?}", e);
                return;
            }
        };
        config.add_search_to_history(query);
        if let Err(e) = config.write(&self.root_path) {
            eprintln!("Error writing config: {:?}", e);
        }
        // UI history is synced on file-watcher-triggered reload
    }

    pub fn handle_search_history_remove(&self, query: String) {
        let mut config = match Config::load(&self.root_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error config: {:?}", e);
                return;
            }
        };
        config.remove_search_from_history(&query);
        if let Err(e) = config.write(&self.root_path) {
            eprintln!("Error writing config: {:?}", e);
        }
    }

    // --- Helpers ---
    pub fn handle_focus_search(&self) {
        if let Some(_app) = self.app_weak.upgrade() {
            // Logic to clear search history flag if it's open,
            // but primarily focuses the search field via Slint logic.
            // We can also trigger a reload if needed.
            let _ = self.reload();
        }
    }

    pub fn handle_shortcut_create_ticket(&self) {
        if let Some(app) = self.app_weak.upgrade() {
            let queues = app.get_board_queues();
            for i in 0..queues.row_count() {
                if let Some(q) = queues.row_data(i) {
                    if q.visible {
                        app.invoke_test_trigger_add_ticket(q.id.clone());
                        break;
                    }
                }
            }
        }
    }

    pub fn handle_select_history_item(&self) {
        if let Some(app) = self.app_weak.upgrade() {
            app.set_show_search_history(false);
            let _ = self.reload();
        }
    }

    fn show_error(&self, msg: &str) {
        if let Some(app) = self.app_weak.upgrade() {
            app.invoke_show_warning_dialog(SharedString::from(msg));
        }
    }
}
