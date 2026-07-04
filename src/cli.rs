//! cli.rs
//!
//! Purpose: Handles command-line argument parsing and CLI mode operations.
//! Includes: CliArgs and Commands enums.
//! Constraints: Should not contain UI or heavy board logic.

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct CliArgs {
    /// Kanban root directory (defaults to ~/Kanban)
    #[arg(short, long)]
    pub root: Option<PathBuf>,

    /// Administrator mode (bypasses manage_only_mine)
    #[arg(long)]
    pub admin: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum QueueAction {
    /// List all queues with settings
    List,
    /// Add a new queue (admin only)
    Add {
        #[arg(long)]
        name: String,
    },
    /// Rename a queue (admin only)
    Rename {
        #[arg(short, long)]
        id: String,
        #[arg(long)]
        name: String,
    },
    /// Delete an empty queue (admin only)
    Delete {
        #[arg(short, long)]
        id: String,
    },
    /// View or set queue settings
    Settings {
        #[arg(short, long)]
        id: String,
        #[arg(short, long)]
        limit: Option<usize>,
    },
    /// List tickets in a queue with optional filters
    Tickets {
        #[arg(short, long)]
        id: String,
        /// Show detailed ticket info (id, created/updated at, points, author, assignee)
        #[arg(short, long)]
        verbose: bool,
        /// Filter by date after (YYYY-MM-DD or YYYY-MM-DD_HH:MM)
        #[arg(long)]
        after: Option<String>,
        /// Filter by last hour
        #[arg(long)]
        last_hour: bool,
        /// Filter by last day
        #[arg(long)]
        last_day: bool,
        /// Filter by assigned user
        #[arg(long)]
        assigned_to: Option<String>,
        /// Show only tickets assigned to active user
        #[arg(long)]
        assigned_to_me: bool,
    },
}
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Add a new ticket
    Add {
        /// Ticket title
        #[arg(short, long)]
        title: String,

        /// Ticket description (if empty, reads from --description-file or stdin)
        #[arg(short = 'd', long)]
        description: Option<String>,

        /// File to read description from (use '-' for stdin)
        #[arg(short = 'D', long)]
        description_file: Option<String>,

        /// Target queue ID (e.g., "1.Incoming")
        #[arg(short, long)]
        queue: String,

        /// Assigned user
        #[arg(short, long, default_value = "")]
        assign_to: String,

        /// Estimation points (1-10)
        #[arg(short, long, default_value = "0")]
        points: u32,
    },

    /// Update an existing ticket
    Update {
        /// Ticket ID (short ID)
        #[arg(short, long)]
        id: String,

        /// New ticket title
        #[arg(short, long)]
        title: Option<String>,

        /// New ticket description
        #[arg(short, long)]
        description: Option<String>,

        /// New assigned user
        #[arg(short, long)]
        assign_to: Option<String>,

        /// Unassign user
        #[arg(short, long)]
        unassign: bool,

        /// New estimation points (1-10)
        #[arg(short, long)]
        points: Option<u32>,
    },

    /// List tickets
    List {
        /// Filter by assigned user
        #[arg(long)]
        assigned_to_user: Option<String>,

        /// Show only unassigned tickets
        #[arg(long)]
        unassigned: bool,

        /// Search query (matches title, description or ID)
        #[arg(short, long)]
        search: Option<String>,

        /// Show only specified ticket ID
        #[arg(long)]
        id: Option<String>,

        /// Filter by date from (YYYY-MM-DD)
        #[arg(long)]
        date_from: Option<String>,

        /// Filter by date to (YYYY-MM-DD)
        #[arg(long)]
        date_to: Option<String>,
    },

    /// Show board statistics
    Stats {
        /// Filter by user
        #[arg(long)]
        user: Option<String>,

        /// Export in CSV format
        #[arg(long)]
        csv: bool,
    },

    /// Change configuration
    Configure {
        /// Set active user
        #[arg(long)]
        active_user: Option<String>,

        /// Set show only mine tickets (true/false)
        #[arg(long)]
        show_only_mine: Option<bool>,

        /// Set manage only mine tickets (true/false)
        #[arg(long)]
        manage_only_mine: Option<bool>,

        /// Add a new user to the list
        #[arg(long)]
        add_user: Option<String>,
    },

    /// Move a ticket to another queue
    Move {
        /// Ticket ID (short ID)
        #[arg(short, long)]
        id: String,

        /// Target queue ID
        #[arg(short, long)]
        queue: String,
    },

    /// Remove (delete) a ticket
    Remove {
        /// Ticket ID (short ID)
        #[arg(short, long)]
        id: String,
    },

    /// Open specific path in GUI
    Open {
        /// Path to open
        path: PathBuf,
    },

    /// Show ticket details
    Show {
        /// Ticket ID (short ID)
        #[arg(short, long)]
        id: String,
    },

    /// Manage sprints
    Sprint {
        #[command(subcommand)]
        action: SprintAction,
    },

    /// Add a comment to a ticket
    Comment {
        /// Ticket ID (short ID)
        #[arg(short, long)]
        id: String,

        /// Comment content (if empty, reads from --content-file or stdin)
        #[arg(short, long)]
        content: Option<String>,

        /// File to read comment content from (use '-' for stdin)
        #[arg(short = 'f', long)]
        content_file: Option<String>,
    },

    /// Attach a file to a ticket
    Attach {
        /// Ticket ID (short ID)
        #[arg(short, long)]
        id: String,

        /// Path to the file to attach
        #[arg(short, long)]
        file: Option<PathBuf>,

        /// List all attachments
        #[arg(short, long)]
        list: bool,

        /// Show the path to the attachments directory
        #[arg(short, long)]
        show: bool,

        /// Open the attachments directory in the file manager
        #[arg(short, long)]
        open: bool,
    },

    /// Manage queues
    Queue {
        #[command(subcommand)]
        action: QueueAction,
    },
}

#[derive(Subcommand, Debug)]
pub enum SprintAction {
    /// List all sprints
    List,
    /// Show the current sprint (by today's date)
    Current,
    /// Add a new sprint
    Add {
        #[arg(long)]
        number: Option<u32>,
        #[arg(long)]
        name: String,
        #[arg(long)]
        start: String,
        #[arg(long)]
        end: String,
    },
    /// Update an existing sprint
    Update {
        #[arg(long)]
        number: u32,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        start: Option<String>,
        #[arg(long)]
        end: Option<String>,
    },
    /// Remove a sprint
    Remove {
        #[arg(long)]
        number: u32,
    },
}
