//! A game and its complete history. One `Game` is exactly one save file
//! (docs/en/architecture.md § 5, "Persistence").

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::adjudicator::{AdjudicationError, Adjudicator};
use crate::map;
use crate::model::{GameState, Order, Phase, PhaseKind, Power, Resolution};

pub const SAVE_FORMAT: &str = "diplomacy-gm/game";
pub const SAVE_FORMAT_VERSION: u32 = 1;

pub type OrderSet = BTreeMap<Power, Vec<Order>>;

#[derive(Debug, Error)]
pub enum GameError {
    #[error("phase {got} is not the current phase ({expected})")]
    PhaseMismatch { expected: Phase, got: Phase },
    #[error("the game is over")]
    GameOver,
    #[error("invalid order: {0}")]
    InvalidOrder(String),
    #[error(transparent)]
    Adjudication(#[from] AdjudicationError),
    #[error("invalid save data: {0}")]
    InvalidSave(String),
}

/// A resolved phase: the position it started from, all orders, and the resolution (incl. trace).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhaseRecord {
    pub phase: Phase,
    pub state: GameState,
    pub orders: OrderSet,
    pub resolution: Resolution,
    pub resolved_at: DateTime<Utc>,
}

/// Save file contents. Everything needed to restore the game (and replay it) is in here.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    pub format: String,
    pub format_version: u32,
    pub game_id: String,
    #[serde(default)]
    pub title: Option<String>,
    pub variant: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// Position at the start of the current (unresolved) phase.
    pub state: GameState,
    /// Orders submitted so far for the current phase (secret until resolved).
    #[serde(default)]
    pub orders: OrderSet,
    /// Resolved phases, oldest first.
    #[serde(default)]
    pub history: Vec<PhaseRecord>,
}

/// One point in the replay: a phase's starting position, plus its orders and resolution
/// if it has been resolved. The last frame is the current position.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplayFrame<'a> {
    pub index: usize,
    pub phase: Phase,
    pub state: &'a GameState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orders: Option<&'a OrderSet>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<&'a Resolution>,
}

pub fn is_valid_game_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= 64 && id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}

pub fn new_game_id() -> String {
    format!("g_{}", uuid::Uuid::new_v4().simple())
}

impl Game {
    /// New standard game at Spring 1901.
    pub fn new_standard(title: Option<String>) -> Self {
        let now = Utc::now();
        Game {
            format: SAVE_FORMAT.to_string(),
            format_version: SAVE_FORMAT_VERSION,
            game_id: new_game_id(),
            title,
            variant: "standard".to_string(),
            created_at: now,
            updated_at: now,
            state: GameState::standard_start(),
            orders: OrderSet::new(),
            history: Vec::new(),
        }
    }

    fn check_phase(&self, phase: Phase) -> Result<(), GameError> {
        if self.state.winner.is_some() {
            return Err(GameError::GameOver);
        }
        if phase != self.state.phase {
            return Err(GameError::PhaseMismatch {
                expected: self.state.phase,
                got: phase,
            });
        }
        Ok(())
    }

    /// Submit or replace a power's orders for the current phase. Only structural checks are
    /// done here; legality is decided by the adjudicator.
    pub fn submit_orders(&mut self, phase: Phase, power: Power, orders: Vec<Order>) -> Result<(), GameError> {
        self.check_phase(phase)?;
        let map = map::standard();
        for order in &orders {
            if order.power != power {
                return Err(GameError::InvalidOrder(format!(
                    "order for {} submitted as {}",
                    order.power.code(),
                    power.code()
                )));
            }
            let province = order.unit.split('/').next().unwrap_or_default();
            if map.province(province).is_none() {
                return Err(GameError::InvalidOrder(format!("unknown province `{}`", order.unit)));
            }
            if phase.kind == PhaseKind::Movement
                && !self
                    .state
                    .units
                    .iter()
                    .any(|u| u.power == power && u.location == province)
            {
                return Err(GameError::InvalidOrder(format!(
                    "{} has no unit in {province}",
                    power.code()
                )));
            }
        }
        self.orders.insert(power, orders);
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Adjudicate the current phase, append it to the history and advance to the next state.
    pub fn resolve(&mut self, phase: Phase, adjudicator: &dyn Adjudicator) -> Result<&PhaseRecord, GameError> {
        self.check_phase(phase)?;
        let all_orders: Vec<Order> = self.orders.values().flatten().cloned().collect();
        let resolution = adjudicator.adjudicate(&self.state, &all_orders)?;
        if resolution.phase != phase || resolution.next_state.phase <= phase {
            return Err(AdjudicationError::Failed(format!(
                "resolution for {} leads to {}, expected a phase after {phase}",
                resolution.phase, resolution.next_state.phase
            ))
            .into());
        }
        let now = Utc::now();
        let next = resolution.next_state.clone();
        self.history.push(PhaseRecord {
            phase,
            state: std::mem::replace(&mut self.state, next),
            orders: std::mem::take(&mut self.orders),
            resolution,
            resolved_at: now,
        });
        self.updated_at = now;
        Ok(self.history.last().expect("just pushed"))
    }

    pub fn replay_len(&self) -> usize {
        self.history.len() + 1
    }

    pub fn replay_frame(&self, index: usize) -> Option<ReplayFrame<'_>> {
        match self.history.get(index) {
            Some(r) => Some(ReplayFrame {
                index,
                phase: r.phase,
                state: &r.state,
                orders: Some(&r.orders),
                resolution: Some(&r.resolution),
            }),
            None if index == self.history.len() => Some(ReplayFrame {
                index,
                phase: self.state.phase,
                state: &self.state,
                orders: None,
                resolution: None,
            }),
            None => None,
        }
    }

    pub fn replay_frames(&self) -> Vec<ReplayFrame<'_>> {
        (0..self.replay_len()).filter_map(|i| self.replay_frame(i)).collect()
    }

    /// Check that restored save data is consistent before it is used.
    pub fn validate(&self) -> Result<(), GameError> {
        let bad = |msg: String| Err(GameError::InvalidSave(msg));
        if self.format != SAVE_FORMAT {
            return bad(format!("unknown format `{}`", self.format));
        }
        if self.format_version != SAVE_FORMAT_VERSION {
            return bad(format!("unsupported format version {}", self.format_version));
        }
        if !is_valid_game_id(&self.game_id) {
            return bad(format!("invalid game id `{}`", self.game_id));
        }
        if self.variant != "standard" {
            return bad(format!("unsupported variant `{}`", self.variant));
        }

        let mut states = self.history.iter().map(|r| &r.state).chain([&self.state]);
        if let Some(bad_state) = states.find_map(|s| check_state(s).err()) {
            return bad(bad_state);
        }

        for (i, record) in self.history.iter().enumerate() {
            if record.state.phase != record.phase || record.resolution.phase != record.phase {
                return bad(format!("history[{i}]: phase mismatch in {}", record.phase));
            }
            let next = self.history.get(i + 1).map(|r| &r.state).unwrap_or(&self.state);
            if &record.resolution.next_state != next {
                return bad(format!(
                    "history[{i}] ({}): result does not match the following position",
                    record.phase
                ));
            }
            if next.phase <= record.phase {
                return bad(format!(
                    "history[{i}]: phases out of order ({} → {})",
                    record.phase, next.phase
                ));
            }
            for (power, orders) in &record.orders {
                if orders.iter().any(|o| o.power != *power) {
                    return bad(format!(
                        "history[{i}]: order filed under the wrong power {}",
                        power.code()
                    ));
                }
            }
        }
        if self.history.is_empty() && self.state.phase < Phase::START {
            return bad(format!("game starts before {}", Phase::START));
        }
        for (power, orders) in &self.orders {
            if orders.iter().any(|o| o.power != *power) {
                return bad(format!("pending order filed under the wrong power {}", power.code()));
            }
        }
        Ok(())
    }
}

fn check_state(state: &GameState) -> Result<(), String> {
    let map = map::standard();
    let mut occupied = std::collections::HashSet::new();
    for unit in &state.units {
        let Some(province) = map.province(&unit.location) else {
            return Err(format!("{}: unknown province `{}`", state.phase, unit.location));
        };
        if let Some(coast) = &unit.coast
            && !province.coasts.as_ref().is_some_and(|c| c.contains(coast))
        {
            return Err(format!("{}: {} has no coast `{coast}`", state.phase, unit.location));
        }
        if !occupied.insert(&unit.location) {
            return Err(format!("{}: two units in {} (R-UNIT-03)", state.phase, unit.location));
        }
    }
    for sc in state.supply_centers.keys() {
        if !map.province(sc).is_some_and(|p| p.supply_center) {
            return Err(format!("{}: `{sc}` is not a supply center", state.phase));
        }
    }
    Ok(())
}
