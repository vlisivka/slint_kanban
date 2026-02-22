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

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Add a new ticket
    Add {
        /// Ticket title
        #[arg(short, long)]
        title: String,

        /// Ticket description
        #[arg(short, long, default_value = "")]
        description: String,

        /// Target queue ID (e.g., "1. Incoming")
        #[arg(short, long)]
        queue: String,

        /// Assigned user
        #[arg(short, long, default_value = "")]
        assign_to: String,
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
    },

    /// Change configuration
    Configure {
        /// Set active user
        #[arg(long)]
        active_user: Option<String>,

        /// Set show only mine tickets (true/false)
        #[arg(long)]
        show_only_mine: Option<bool>,

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

        /// Comment content
        #[arg(short, long)]
        content: String,
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
