//! Adjudicator interface. The engine (rules.md / adjudication.md) plugs in here;
//! the server only records what it returns.

use thiserror::Error;

use crate::model::{GameState, Order, Resolution};

#[derive(Debug, Error)]
pub enum AdjudicationError {
    #[error("the adjudicator is not implemented yet")]
    NotImplemented,
    #[error("adjudication failed: {0}")]
    Failed(String),
}

/// Pure function `adjudicate(state, orders) → Resolution` (architecture.md § 2). No I/O.
pub trait Adjudicator: Send + Sync {
    fn adjudicate(&self, state: &GameState, orders: &[Order]) -> Result<Resolution, AdjudicationError>;
}

/// Placeholder until the engine exists: every resolve request fails with `NotImplemented`.
pub struct PendingEngine;

impl Adjudicator for PendingEngine {
    fn adjudicate(&self, _state: &GameState, _orders: &[Order]) -> Result<Resolution, AdjudicationError> {
        Err(AdjudicationError::NotImplemented)
    }
}
