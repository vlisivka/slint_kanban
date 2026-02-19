//! mod.rs (model)
//!
//! Purpose: Module entry point for the data model. Re-exports core structures for external use.
//! Includes: ticket, queue, board, and config modules.
//! Constraints: Only re-exports and module definitions should be here.

pub mod board;
pub mod config;
pub mod queue;
pub mod ticket;

// Re-export core types for convenience
pub use board::Board;
pub use config::Config;
pub use ticket::Ticket;

#[cfg(test)]
mod tests;
