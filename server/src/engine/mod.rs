//! The adjudicator (docs/en/adjudication.md): pure functions from `(GameState, orders)` to a
//! `Resolution`. Only the movement phase is implemented so far; retreat and adjustment phases
//! return `AdjudicationError::NotImplemented`.

mod movement;
mod resolve;
mod validate;

use crate::adjudicator::{AdjudicationError, Adjudicator};
use crate::map::{self, ProvinceType};
use crate::model::{GameState, Order, PhaseKind, Resolution, Unit, UnitType};

/// The rules engine, plugged into the server's `Adjudicator` trait.
pub struct Engine;

impl Adjudicator for Engine {
    fn adjudicate(&self, state: &GameState, orders: &[Order]) -> Result<Resolution, AdjudicationError> {
        match state.phase.kind {
            PhaseKind::Movement => Ok(movement::adjudicate(state, orders)),
            PhaseKind::Retreat | PhaseKind::Adjustment => Err(AdjudicationError::NotImplemented),
        }
    }
}

/// Province part of a location: `Bul/EC` → `Bul`.
pub(crate) fn province(location: &str) -> &str {
    location.split('/').next().unwrap_or(location)
}

/// Coast part of a location: `Bul/EC` → `Some("EC")`.
pub(crate) fn coast(location: &str) -> Option<&str> {
    location.split_once('/').map(|(_, c)| c)
}

pub(crate) fn province_type(id: &str) -> Option<ProvinceType> {
    map::standard().province(id).map(|p| p.kind)
}

/// Key into the fleet adjacency table: the province, plus the coast for split-coast provinces.
fn fleet_key(location: &str, coast: Option<&str>) -> String {
    match coast {
        Some(c) => format!("{location}/{c}"),
        None => location.to_string(),
    }
}

/// Locations a fleet at `location`/`coast` can move to (R-MOV-02, R-MAP-05/06). Split-coast
/// destinations carry their coast, e.g. `Bul/EC`.
pub(crate) fn fleet_moves(location: &str, coast: Option<&str>) -> &'static [String] {
    map::standard()
        .adjacency
        .fleet
        .get(&fleet_key(location, coast))
        .map(Vec::as_slice)
        .unwrap_or_default()
}

/// Provinces an army in `location` can move to without a convoy (R-MOV-01).
pub(crate) fn army_moves(location: &str) -> &'static [String] {
    map::standard()
        .adjacency
        .army
        .get(location)
        .map(Vec::as_slice)
        .unwrap_or_default()
}

/// Destinations a unit could move to this turn, ignoring other units (RB-D01..03).
/// Fleet destinations on split coasts carry the coast (`Spa/SC`). Convoys are not included.
pub fn legal_moves(unit: &Unit) -> Vec<String> {
    match unit.unit_type {
        UnitType::A => army_moves(&unit.location).to_vec(),
        UnitType::F => fleet_moves(&unit.location, unit.coast.as_deref()).to_vec(),
    }
}

/// Whether the unit could move into `target` (any coast), as required of a supporter (R-SUP-02/03).
pub(crate) fn can_reach(unit: &Unit, target: &str) -> bool {
    match unit.unit_type {
        UnitType::A => army_moves(&unit.location).iter().any(|p| p == target),
        UnitType::F => fleet_moves(&unit.location, unit.coast.as_deref())
            .iter()
            .any(|l| province(l) == target),
    }
}

/// Whether the water province `sea` borders the province `land` (any coast).
pub(crate) fn sea_borders(sea: &str, land: &str) -> bool {
    fleet_moves(sea, None).iter().any(|l| province(l) == land)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn location_parts() {
        assert_eq!(province("Bul/EC"), "Bul");
        assert_eq!(coast("Bul/EC"), Some("EC"));
        assert_eq!(province("Par"), "Par");
        assert_eq!(coast("Par"), None);
    }
}
