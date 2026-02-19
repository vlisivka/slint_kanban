//! queue.rs
//!
//! Purpose: Defines the Queue structure, which represents a column on the board.
//! Includes: Queue struct.
//! Constraints: Logic for cross-queue movements belongs in Board.

use crate::model::ticket::Ticket;

#[derive(Debug, Clone)]
pub struct Queue {
    pub id: String, // Directory name
    pub name: String,
    pub tickets: Vec<Ticket>,
    pub limit: Option<usize>,
    pub visible: bool,
}
