//! controller.rs
//!
//! Purpose: Encapsulates the application logic and state, acting as the bridge between the UI (App) and the Data Model (Board).
//! Includes: AppController struct and methods for handling UI actions.

use crate::model::{Board, Config};
use crate::{sync_board_to_ui, App, TicketStr, UserGlobal};
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

        self.sync_board_data(&app, &board);
        self.sync_users(&app, &board);
        self.sync_config(&app, &board);

        Ok(())
    }

    /// Pushes filtered board data (queues & tickets) into the UI.
    fn sync_board_data(&self, app: &App, board: &Board) {
        let query = app.get_search_query();
        let date_from = app.get_date_from();
        let date_to = app.get_date_to();

        let show_only_mine = board.config.show_only_mine();
        let active_user = board.config.active_user();

        sync_board_to_ui(
            app,
            board,
            query.as_str(),
            date_from.as_str(),
            date_to.as_str(),
            show_only_mine,
            active_user,
        );
    }

    /// Syncs the user list and active user into the UI global.
    /// Only updates the VecModel when the list actually changed to avoid
    /// resetting ComboBox selection and causing flicker.
    fn sync_users(&self, app: &App, board: &Board) {
        let user_global = app.global::<UserGlobal>();

        let mut new_users: Vec<SharedString> = Vec::new();
        if !board.config.users().iter().any(|u| u == "<unassigned>") {
            new_users.push(SharedString::from("<unassigned>"));
        }
        new_users.extend(board.config.users().iter().map(SharedString::from));

        let current_users_model = user_global.get_users();
        let users_changed = current_users_model.row_count() != new_users.len()
            || current_users_model
                .iter()
                .zip(new_users.iter())
                .any(|(a, b)| a != *b);

        if users_changed {
            println!("Controller: Updating users list in UI...");
            user_global.set_users(Rc::new(VecModel::from(new_users)).into());
        }

        // Sync active user
        let new_active_user = SharedString::from(board.config.active_user());
        if user_global.get_active_user() != new_active_user {
            user_global.set_active_user(new_active_user);
        }
    }

    /// Syncs config state (show_only_mine, search history) into the UI.
    fn sync_config(&self, app: &App, board: &Board) {
        app.global::<UserGlobal>()
            .set_show_only_mine(board.config.show_only_mine());

        let history: Vec<SharedString> = board
            .config
            .search_history()
            .iter()
            .map(SharedString::from)
            .collect();
        app.set_search_history(Rc::new(VecModel::from(history)).into());
    }

    fn load_board(&self, action: &str) -> Option<Board> {
        match Board::load(self.root_path.clone()) {
            Ok(b) => Some(b),
            Err(e) => {
                eprintln!("Error loading board for {}: {:?}", action, e);
                None
            }
        }
    }

    fn modify_config<F>(&self, action: &str, f: F)
    where
        F: FnOnce(&mut Config),
    {
        let mut config = match Config::load(&self.root_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error loading config for {}: {:?}", action, e);
                return;
            }
        };
        f(&mut config);
        if let Err(e) = config.write(&self.root_path) {
            eprintln!("Error writing config for {}: {:?}", action, e);
        }
    }

    // --- Action Handlers ---

    pub fn handle_move_ticket(&self, ticket_id: String, source_id: String, target_id: String) {
        let board = match self.load_board("move") {
            Some(b) => b,
            None => return,
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

    pub fn handle_show_board_info(&self) {
        let app = match self.app_weak.upgrade() {
            Some(a) => a,
            None => return,
        };

        match Board::load_board_info(&self.root_path) {
            Ok((metadata, content)) => {
                let info = TicketStr {
                    id: "board-info".into(),
                    title: metadata.title.into(),
                    description: content.into(),
                    snippet: "".into(),
                    created_at: metadata.created_at.into(),
                    updated_at: metadata.updated_at.into(),
                    assigned_to: metadata.assigned_to.into(),
                    author: metadata.author.into(),
                    references: Rc::new(VecModel::default()).into(),
                    comments: Rc::new(VecModel::default()).into(),
                };
                app.set_active_ticket(info);
                app.set_show_ticket_view_dialog(true);
            }
            Err(e) => {
                self.show_error(&format!("Error loading board info: {}", e));
            }
        }
    }

    pub fn handle_delete_ticket(&self, ticket_id: String) {
        let board = match self.load_board("delete") {
            Some(b) => b,
            None => return,
        };

        println!("Controller: Deleting ticket {}", ticket_id);
        if let Err(e) = board.delete_ticket(&ticket_id) {
            eprintln!("Error deleting: {:?}", e);
        }
    }

    pub fn handle_create_ticket(
        &self,
        queue_id: String,
        title: String,
        description: String,
        assigned_to: String,
    ) {
        let board = match self.load_board("create") {
            Some(b) => b,
            None => return,
        };

        println!("Controller: Creating ticket in {}", queue_id);
        let author = board.config.active_user();
        if let Err(e) = board.create_ticket(&title, &description, &queue_id, &assigned_to, author) {
            self.show_error(&e.to_string());
        }
    }

    pub fn handle_update_ticket(
        &self,
        ticket_id: String,
        title: String,
        description: String,
        assigned_to: String,
    ) {
        let board = match self.load_board("save") {
            Some(b) => b,
            None => return,
        };

        println!("Controller: Saving ticket {}", ticket_id);
        if let Err(e) = board.update_ticket(&ticket_id, &title, &description, &assigned_to) {
            eprintln!("Error saving ticket: {:?}", e);
        }
    }

    pub fn handle_add_comment(&self, ticket_id: String, content: String) {
        let board = match self.load_board("add_comment") {
            Some(b) => b,
            None => return,
        };

        println!("Controller: Adding comment to ticket {}", ticket_id);
        let author = board.config.active_user();
        if let Err(e) = board.add_comment(&ticket_id, &content, author) {
            eprintln!("Error adding comment: {:?}", e);
            self.show_error(&e.to_string());
        } else {
            if let Some(app) = self.app_weak.upgrade() {
                if let Ok(b) = Board::load(self.root_path.clone()) {
                    if let Some(t) = b.find_ticket_by_id(&ticket_id) {
                        app.set_active_ticket(crate::into_slint_ticket(t, &b));
                    }
                }
            }
        }
    }

    pub fn handle_set_queue_limit(&self, queue_id: String, limit: i32) {
        let mut board = match self.load_board("limit") {
            Some(b) => b,
            None => return,
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

    pub fn handle_change_active_user(&self, username: String) {
        self.modify_config("user change", |c| c.user.active_user = username.clone());

        // Update UI immediately so the user doesn't see stale state while
        // waiting for the file watcher to trigger a full reload.
        if let Some(app) = self.app_weak.upgrade() {
            app.global::<UserGlobal>()
                .set_active_user(SharedString::from(username));
        }
    }

    pub fn handle_toggle_show_only_mine(&self, enabled: bool) {
        self.modify_config("toggle mine", |c| c.user.show_only_mine = enabled);

        // Update UI immediately so the user doesn't see stale state while
        // waiting for the file watcher to trigger a full reload.
        if let Some(app) = self.app_weak.upgrade() {
            app.global::<UserGlobal>().set_show_only_mine(enabled);
            let _ = self.reload();
        }
    }

    pub fn handle_toggle_queue_visibility(&self, queue_id: String, visible: bool) {
        self.modify_config("queue visibility", |c| c.set_visible(queue_id, visible));
    }

    pub fn handle_accept_search(&self, query: String) {
        self.modify_config("search history add", |c| c.add_search_to_history(query));
        // UI history is synced on file-watcher-triggered reload
    }

    pub fn handle_remove_search_item(&self, query: String) {
        self.modify_config("search history remove", |c| {
            c.remove_search_from_history(&query)
        });
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

    pub fn handle_navigate_to(&self, target_id: String) {
        let board = match self.load_board("navigate") {
            Some(b) => b,
            None => return,
        };

        let id_to_find = target_id.strip_prefix('#').unwrap_or(&target_id);

        if let Some(ticket) = board.find_ticket_by_id(id_to_find) {
            if let Some(app) = self.app_weak.upgrade() {
                app.set_active_ticket(crate::into_slint_ticket(ticket, &board));
                app.set_show_ticket_view_dialog(true);
            }
        } else {
            self.show_error(&format!("Ticket NOT FOUND: {}", target_id));
        }
    }

    fn show_error(&self, msg: &str) {
        if let Some(app) = self.app_weak.upgrade() {
            app.invoke_open_warning_dialog(SharedString::from(msg));
        }
    }
}
