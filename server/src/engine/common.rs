//! Helpers shared by the phase adjudicators: trace steps, notation, the end-of-year update
//! (SC ownership, victory) and phase skipping (architecture.md § 3).

use std::collections::{BTreeMap, HashMap, VecDeque};

use serde_json::{Value, json};

use super::province;
use crate::map;
use crate::model::{GameState, Order, OrderKind, Phase, PhaseKind, Power, Season, TraceStep, Unit, UnitType};

/// SCs needed to win (R-GEN-03).
pub(crate) const VICTORY_SCS: usize = 18;

pub(crate) fn step(name: &str, rules: &[&str], details: Value) -> TraceStep {
    let Value::Object(details) = details else {
        unreachable!("trace details are objects")
    };
    TraceStep {
        step: name.to_string(),
        rules: strings(rules),
        details,
    }
}

pub(crate) fn strings(rules: &[&str]) -> Vec<String> {
    rules.iter().map(|r| r.to_string()).collect()
}

pub(crate) fn with(mut v: Value, key: &str, value: &str) -> Value {
    v[key] = json!(value);
    v
}

pub(crate) fn type_letter(t: UnitType) -> &'static str {
    match t {
        UnitType::A => "A",
        UnitType::F => "F",
    }
}

pub(crate) fn with_coast(location: &str, coast: Option<&str>) -> String {
    match coast {
        Some(c) => format!("{location}/{c}"),
        None => location.to_string(),
    }
}

pub(crate) fn unit_label(unit: &Unit) -> String {
    format!(
        "{} {}",
        type_letter(unit.unit_type),
        with_coast(&unit.location, unit.coast.as_deref())
    )
}

pub(crate) fn position_text(unit: &Unit) -> String {
    format!("{} {}", unit.power.code(), unit_label(unit))
}

/// A submitted order in rulebook notation, e.g. `A Gas S A Mar-Bur`. `unit_type` looks up the
/// type of the unit in a province (the board in movement phases, the dislodged units in retreats).
pub(crate) fn order_text(order: &Order, unit_type: impl Fn(&str) -> Option<UnitType>) -> String {
    let ty = |loc: &str| {
        unit_type(province(loc))
            .map(|t| format!("{} ", type_letter(t)))
            .unwrap_or_default()
    };
    let me = format!("{}{}", ty(&order.unit), order.unit);
    match &order.kind {
        OrderKind::Hold => format!("{me} H"),
        OrderKind::Move { to, via_convoy } => {
            format!("{me}-{to}{}", if *via_convoy { " via convoy" } else { "" })
        }
        OrderKind::Support { target, to: None } => format!("{me} S {}{target}", ty(target)),
        OrderKind::Support { target, to: Some(to) } => format!("{me} S {}{target}-{to}", ty(target)),
        OrderKind::Convoy { target, to } => format!("{me} C {}{target}-{to}", ty(target)),
        OrderKind::Retreat { to } => format!("{me}-{to}"),
        OrderKind::Disband => format!("{me} D"),
        OrderKind::Build { unit_type, coast } => {
            let coast = coast.as_deref().or_else(|| super::coast(&order.unit));
            format!(
                "B {} {}",
                type_letter(*unit_type),
                with_coast(province(&order.unit), coast)
            )
        }
        OrderKind::Waive => format!("{} waive", order.power.code()),
    }
}

pub(crate) fn sc_count(state: &GameState, power: Power) -> usize {
    state.supply_centers.values().filter(|o| **o == Some(power)).count()
}

pub(crate) fn unit_count(state: &GameState, power: Power) -> usize {
    state.units.iter().filter(|u| u.power == power).count()
}

pub(crate) fn in_civil_disorder(state: &GameState, power: Power) -> bool {
    state.civil_disorder.iter().any(|c| c.power == power)
}

/// Home SCs where `power` may build now: still controlled by it and unoccupied (R-ADJ-03/05/06).
pub(crate) fn build_sites(state: &GameState, power: Power) -> Vec<&'static str> {
    map::standard()
        .supply_centers()
        .filter(|sc| {
            sc.home == Some(power)
                && state.supply_centers.get(&sc.id) == Some(&Some(power))
                && !state.units.iter().any(|u| u.location == sc.id)
        })
        .map(|sc| sc.id.as_str())
        .collect()
}

/// The adjustment phase is skipped when no power must disband or can build (architecture.md § 3,
/// R-ADJ-03/05, R-CD-02).
pub(crate) fn adjustments_needed(state: &GameState) -> bool {
    Power::ALL.into_iter().any(|p| {
        let (scs, units) = (sc_count(state, p), unit_count(state, p));
        units > scs || (scs > units && !in_civil_disorder(state, p) && !build_sites(state, p).is_empty())
    })
}

/// End of a Fall turn (after retreats, or after movement if there were none): update SC
/// ownership (R-SC-01/02), check victory (R-GEN-03, PR-11) and pick the next phase — the
/// adjustment phase, or the next Spring if no adjustments are needed.
pub(crate) fn end_of_year(next: &mut GameState) -> (Vec<TraceStep>, Phase) {
    let mut changes = Vec::new();
    for unit in &next.units {
        if let Some(owner) = next.supply_centers.get_mut(&unit.location)
            && *owner != Some(unit.power)
        {
            changes.push(json!({ "province": unit.location, "from": owner.map(Power::code), "to": unit.power.code() }));
            *owner = Some(unit.power);
        }
    }
    let mut steps = vec![step(
        "sc-ownership",
        &["R-SC-01", "R-SC-02"],
        json!({ "changes": changes }),
    )];
    let counts: BTreeMap<&str, usize> = Power::ALL.iter().map(|p| (p.code(), sc_count(next, *p))).collect();
    if next.winner.is_none() {
        next.winner = Power::ALL.into_iter().find(|p| sc_count(next, *p) >= VICTORY_SCS);
    }
    steps.push(step(
        "victory-check",
        &["R-GEN-03", "PR-11"],
        json!({ "supplyCenters": counts, "winner": next.winner.map(Power::code) }),
    ));
    let year = next.phase.year;
    let phase = if adjustments_needed(next) {
        Phase {
            season: Season::Winter,
            year,
            kind: PhaseKind::Adjustment,
        }
    } else {
        Phase {
            season: Season::Spring,
            year: year + 1,
            kind: PhaseKind::Movement,
        }
    };
    (steps, phase)
}

/// Civil-disorder distance (PR-08): fewest province steps from `from` to any home SC of
/// `power`, moving through any province (land or water) regardless of unit type.
pub(crate) fn home_distance(from: &str, power: Power) -> usize {
    let map = map::standard();
    let mut graph: HashMap<&str, Vec<&str>> = HashMap::new();
    let adjacency = map.adjacency.army.iter().chain(map.adjacency.fleet.iter());
    for (a, list) in adjacency {
        graph
            .entry(province(a))
            .or_default()
            .extend(list.iter().map(|b| province(b)));
    }
    let homes: Vec<&str> = map
        .supply_centers()
        .filter(|sc| sc.home == Some(power))
        .map(|sc| sc.id.as_str())
        .collect();
    let mut dist: HashMap<&str, usize> = HashMap::from([(from, 0)]);
    let mut queue = VecDeque::from([from]);
    while let Some(p) = queue.pop_front() {
        let d = dist[p];
        if homes.contains(&p) {
            return d;
        }
        for &q in graph.get(p).map(Vec::as_slice).unwrap_or_default() {
            if !dist.contains_key(q) {
                dist.insert(q, d + 1);
                queue.push_back(q);
            }
        }
    }
    usize::MAX
}
