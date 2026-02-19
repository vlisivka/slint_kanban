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
}
