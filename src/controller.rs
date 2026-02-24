//! controller.rs
//!
//! Purpose: Encapsulates the application logic and state, acting as the bridge between the UI (App) and the Data Model (Board).
//! Includes: AppController struct and methods for handling UI actions.

use crate::model::{Board, Config};
use crate::{App, QueueStr, SprintStr, TicketStr, UserGlobal};
use slint::{ComponentHandle, Model, SharedString, VecModel, Weak};
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Mutex;

/// Mediates between the Slint UI and the file-system Board model.
/// Each action handler re-loads the board from disk to ensure consistency
/// (the board is the source of truth, not in-memory state).
pub struct AppController {
    app_weak: Weak<App>,
    pub root_path: PathBuf,
    // Persistent models for incremental updates
    board_queues_model: Rc<VecModel<QueueStr>>,
    ticket_models: Mutex<HashMap<String, Rc<VecModel<TicketStr>>>>,
    ticket_cache: Mutex<HashMap<String, (String, TicketStr)>>, // ID -> (updated_at, TicketStr)
}

// Safety: The Slint-related fields (Rc<VecModel>) are only accessed
// on the main thread via reload() and UI callbacks.
unsafe impl Send for AppController {}
unsafe impl Sync for AppController {}

impl AppController {
    pub fn new(app_weak: Weak<App>, root_path: PathBuf) -> Self {
        Self {
            app_weak,
            root_path,
            board_queues_model: Rc::new(VecModel::default()),
            ticket_models: Mutex::new(HashMap::new()),
            ticket_cache: Mutex::new(HashMap::new()),
        }
    }

    pub fn board_queues_model(&self) -> Rc<VecModel<QueueStr>> {
        self.board_queues_model.clone()
    }

    /// Reloads the board from disk and synchronizes the UI.
    /// This is used for initial load and when the file watcher detects changes.
    pub fn reload(&self) -> anyhow::Result<()> {
        let start = std::time::Instant::now();
        let app = self
            .app_weak
            .upgrade()
            .ok_or_else(|| anyhow::anyhow!("UI dropped"))?;

        let t1 = std::time::Instant::now();
        let board = Board::load(self.root_path.clone())?;
        let load_duration = t1.elapsed();

        let t2 = std::time::Instant::now();
        self.sync_board_data(&app, &board);
        let sync_duration = t2.elapsed();

        self.sync_users(&app, &board);
        self.sync_config(&app, &board);

        let total_duration = start.elapsed();
        if total_duration.as_millis() > 16 {
            eprintln!(
                "[PERF] reload() took {:?} (load: {:?}, sync: {:?})",
                total_duration, load_duration, sync_duration
            );
        }

        Ok(())
    }

    /// Pushes filtered board data (queues & tickets) into the UI.
    fn sync_board_data(&self, app: &App, board: &Board) {
        let query = app.get_search_query();
        let date_from = app.get_date_from();
        let date_to = app.get_date_to();

        let show_only_mine = board.config.show_only_mine();
        let active_user = board.config.active_user();

        let mut ticket_models = self.ticket_models.lock().unwrap();
        let mut ticket_cache = self.ticket_cache.lock().unwrap();

        let mut slint_queues = Vec::new();

        for queue in &board.queues {
            let mut filtered_tickets: Vec<&crate::model::ticket::Ticket> = queue
                .tickets
                .iter()
                .filter(|t| {
                    let user_filter = if show_only_mine {
                        Some(if active_user == "<unassigned>" {
                            ""
                        } else {
                            active_user
                        })
                    } else {
                        None
                    };
                    t.matches_all(
                        query.as_str(),
                        date_from.as_str(),
                        date_to.as_str(),
                        user_filter,
                    )
                })
                .collect();

            // Sort by updated_at: older at the top, newer at the bottom
            filtered_tickets.sort_by(|a, b| a.updated_at.cmp(&b.updated_at));

            let tickets_model = ticket_models
                .entry(queue.id.clone())
                .or_insert_with(|| Rc::new(VecModel::default()));

            // Update tickets model incrementally
            let mut new_slint_tickets = Vec::new();
            for t in filtered_tickets {
                let s_ticket = if let Some((updated_at, cached)) = ticket_cache.get(&t.id) {
                    if *updated_at == t.updated_at {
                        cached.clone()
                    } else {
                        let new_s = crate::into_slint_ticket(t, board);
                        ticket_cache.insert(t.id.clone(), (t.updated_at.clone(), new_s.clone()));
                        new_s
                    }
                } else {
                    let new_s = crate::into_slint_ticket(t, board);
                    ticket_cache.insert(t.id.clone(), (t.updated_at.clone(), new_s.clone()));
                    new_s
                };
                new_slint_tickets.push(s_ticket);
            }

            // Patch the tickets model
            let current_len = tickets_model.row_count();
            let new_len = new_slint_tickets.len();

            for i in 0..current_len.min(new_len) {
                let old = tickets_model.row_data(i).unwrap();
                let new = &new_slint_tickets[i];

                // Only update if metadata that affects the card display has changed.
                // Comparing the whole struct is still cheaper than letting Slint re-render everything.
                if old.id != new.id
                    || old.updated_at != new.updated_at
                    || old.title != new.title
                    || old.assigned_to != new.assigned_to
                    || old.points != new.points
                {
                    tickets_model.set_row_data(i, new.clone());
                }
            }

            if new_len > current_len {
                for i in current_len..new_len {
                    tickets_model.push(new_slint_tickets[i].clone());
                }
            } else if new_len < current_len {
                for _ in new_len..current_len {
                    tickets_model.remove(new_len);
                }
            }

            let ticket_count = new_len as i32;
            let limit = queue.limit.map(|l| l as i32).unwrap_or(-1);
            let total_points: i32 = new_slint_tickets.iter().map(|t| t.points).sum();

            slint_queues.push(QueueStr {
                id: SharedString::from(&queue.id),
                name: SharedString::from(&queue.name),
                tickets: tickets_model.clone().into(),
                limit,
                ticket_count,
                total_points,
                visible: queue.visible,
            });
        }

        // Patch the board_queues_model
        let current_q_len = self.board_queues_model.row_count();
        let new_q_len = slint_queues.len();

        for i in 0..current_q_len.min(new_q_len) {
            self.board_queues_model
                .set_row_data(i, slint_queues[i].clone());
        }

        if new_q_len > current_q_len {
            for i in current_q_len..new_q_len {
                self.board_queues_model.push(slint_queues[i].clone());
            }
        } else if new_q_len < current_q_len {
            for _ in new_q_len..current_q_len {
                self.board_queues_model.remove(new_q_len);
            }
        }
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
        let user_global = app.global::<UserGlobal>();
        user_global.set_show_only_mine(board.config.show_only_mine());
        user_global.set_manage_only_mine(board.config.manage_only_mine());

        let history: Vec<SharedString> = board
            .config
            .search_history()
            .iter()
            .map(SharedString::from)
            .collect();
        app.set_search_history(Rc::new(VecModel::from(history)).into());

        // Sync sprint info
        if let Some(sprint) = board.config.get_current_sprint(None) {
            app.set_active_sprint(SprintStr {
                number: sprint.number as i32,
                name: sprint.name.clone().into(),
                start_date: sprint.start_date.clone().into(),
                end_date: sprint.end_date.clone().into(),
            });
            app.set_has_active_sprint(true);
        } else {
            app.set_has_active_sprint(false);
        }
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

    fn can_manage_ticket(&self, ticket: &crate::model::ticket::Ticket, board: &Board) -> bool {
        board.can_manage_ticket(ticket, false)
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

        if let Some(ticket) = board.find_ticket_by_id(&ticket_id) {
            if !self.can_manage_ticket(ticket, &board) {
                self.show_error("Access Denied: You can only manage tickets assigned to you.");
                return;
            }
        }

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
                    attachment_count: 0,
                    points: metadata.points as i32,
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
        if let Some(ticket) = board.find_ticket_by_id(&ticket_id) {
            if !self.can_manage_ticket(ticket, &board) {
                self.show_error("Access Denied: You can only delete tickets assigned to you.");
                return;
            }
        }

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
        points: i32,
    ) {
        let board = match self.load_board("create") {
            Some(b) => b,
            None => return,
        };

        println!("Controller: Creating ticket in {}", queue_id);
        let author = board.config.active_user();
        if let Err(e) = board.create_ticket(
            &title,
            &description,
            &queue_id,
            &assigned_to,
            author,
            points as u32,
        ) {
            self.show_error(&e.to_string());
        }
    }

    pub fn handle_update_ticket(
        &self,
        ticket_id: String,
        title: String,
        description: String,
        assigned_to: String,
        points: i32,
    ) {
        let board = match self.load_board("save") {
            Some(b) => b,
            None => return,
        };

        println!("Controller: Saving ticket {}", ticket_id);
        if let Some(ticket) = board.find_ticket_by_id(&ticket_id) {
            if !self.can_manage_ticket(ticket, &board) {
                self.show_error("Access Denied: You can only update tickets assigned to you.");
                return;
            }
        }

        if let Err(e) = board.update_ticket(
            &ticket_id,
            &title,
            &description,
            &assigned_to,
            points as u32,
        ) {
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
                    if let Ok(t) = b.load_full_ticket(&ticket_id) {
                        app.set_active_ticket(crate::into_slint_ticket(&t, &b));
                    }
                }
            }
        }
    }

    pub fn handle_attach_file(&self, ticket_id: String) -> String {
        let board = match self.load_board("attach_file") {
            Some(b) => b,
            None => return String::new(),
        };

        if let Some(path) = rfd::FileDialog::new().pick_file() {
            println!(
                "Controller: Attaching file {:?} to ticket {}",
                path, ticket_id
            );
            match board.attach_file(&ticket_id, &path) {
                Ok(markdown_link) => {
                    // Update the active ticket to reflect the latest attachment_count
                    if let Some(app) = self.app_weak.upgrade() {
                        if let Ok(b) = Board::load(self.root_path.clone()) {
                            if let Ok(t) = b.load_full_ticket(&ticket_id) {
                                app.set_active_ticket(crate::into_slint_ticket(&t, &b));
                            }
                        }
                    }
                    markdown_link
                }
                Err(e) => {
                    eprintln!("Error attaching file: {:?}", e);
                    self.show_error(&e.to_string());
                    String::new()
                }
            }
        } else {
            String::new() // User cancelled
        }
    }

    pub fn handle_open_attachment_folder(&self, ticket_id: String) {
        let board = match self.load_board("open_attachment_folder") {
            Some(b) => b,
            None => return,
        };
        let attach_dir = board.ticket_path(&ticket_id).join("attachment");

        if attach_dir.exists() {
            if let Err(e) = open::that(&attach_dir) {
                eprintln!("Error opening attachment folder: {:?}", e);
                self.show_error(&format!("Could not open folder: {}", e));
            }
        } else {
            self.show_error("Attachment folder doesn't exist yet.");
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

    pub fn handle_toggle_manage_only_mine(&self, enabled: bool) {
        self.modify_config("toggle manage mine", |c| c.user.manage_only_mine = enabled);

        if let Some(app) = self.app_weak.upgrade() {
            app.global::<UserGlobal>().set_manage_only_mine(enabled);
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

    pub fn handle_request_admin_data(&self) {
        if let Some(_board) = self.load_board("admin data") {
            if let Some(app) = self.app_weak.upgrade() {
                let (_, readme) = Board::load_board_info(&self.root_path).unwrap_or_default();
                app.set_board_readme_content(readme.into());
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

        if let Some(_) = board.find_ticket_by_id(id_to_find) {
            if let Ok(ticket) = board.load_full_ticket(id_to_find) {
                if let Some(app) = self.app_weak.upgrade() {
                    app.set_active_ticket(crate::into_slint_ticket(&ticket, &board));
                    app.set_show_ticket_view_dialog(true);
                }
            }
        } else {
            self.show_error(&format!("Ticket NOT FOUND: {}", target_id));
        }
    }

    pub fn handle_request_full_ticket(&self, ticket_id: String) -> TicketStr {
        let board = match self.load_board("request_full_ticket") {
            Some(b) => b,
            None => return TicketStr::default(),
        };

        match board.load_full_ticket(&ticket_id) {
            Ok(t) => crate::into_slint_ticket(&t, &board),
            Err(e) => {
                eprintln!("Error loading full ticket {}: {:?}", ticket_id, e);
                TicketStr::default()
            }
        }
    }

    pub fn handle_request_stats(&self) {
        let board = match self.load_board("stats") {
            Some(b) => b,
            None => return,
        };

        if let Some(app) = self.app_weak.upgrade() {
            let summary = crate::model::stats::get_board_summary(&board);
            let slint_summary = crate::into_slint_summary(&summary);
            app.set_board_stats(slint_summary);
            app.set_show_stats_view(true);
        }
    }

    pub fn handle_request_sprints_view(&self) {
        let board = match self.load_board("sprints") {
            Some(b) => b,
            None => return,
        };

        if let Some(app) = self.app_weak.upgrade() {
            let slint_sprints: Vec<SprintStr> = board
                .config
                .kanban
                .sprints
                .iter()
                .map(|s| SprintStr {
                    number: s.number as i32,
                    name: s.name.clone().into(),
                    start_date: s.start_date.clone().into(),
                    end_date: s.end_date.clone().into(),
                })
                .collect();
            app.set_all_sprints(Rc::new(VecModel::from(slint_sprints)).into());
            app.set_show_sprints_view(true);
        }
    }

    pub fn handle_save_board_readme(&self, content: String) {
        if let Some(board) = self.load_board("save readme") {
            if let Err(e) = board.update_board_readme(&content) {
                self.show_error(&format!("Failed to save board README: {}", e));
            }
        }
    }

    pub fn handle_add_queue(&self, name: String) {
        if let Some(board) = self.load_board("add queue") {
            if let Err(e) = board.add_queue(&name) {
                self.show_error(&format!("Failed to add queue: {}", e));
            } else {
                let _ = self.reload();
            }
        }
    }

    pub fn handle_rename_queue(&self, id: String, new_name: String) {
        if let Some(board) = self.load_board("rename queue") {
            if let Err(e) = board.rename_queue(&id, &new_name) {
                self.show_error(&format!("Failed to rename queue: {}", e));
            } else {
                let _ = self.reload();
            }
        }
    }

    pub fn handle_delete_queue(&self, id: String) {
        if let Some(board) = self.load_board("delete queue") {
            if let Err(e) = board.delete_queue(&id) {
                self.show_error(&format!("Failed to delete queue: {}", e));
            } else {
                let _ = self.reload();
            }
        }
    }

    pub fn handle_add_user(&self, username: String) {
        if let Some(mut board) = self.load_board("add user") {
            if let Err(e) = board.add_user(&username) {
                self.show_error(&format!("Failed to add user: {}", e));
            } else {
                let _ = self.reload();
            }
        }
    }

    pub fn handle_remove_user(&self, username: String) {
        if let Some(mut board) = self.load_board("remove user") {
            if let Err(e) = board.remove_user(&username) {
                self.show_error(&format!("Failed to remove user: {}", e));
            } else {
                let _ = self.reload();
            }
        }
    }

    fn show_error(&self, msg: &str) {
        if let Some(app) = self.app_weak.upgrade() {
            app.invoke_open_warning_dialog(SharedString::from(msg));
        }
    }
}
