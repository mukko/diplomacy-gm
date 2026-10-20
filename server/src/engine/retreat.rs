//! Retreat phase (adjudication.md § 6, R-RET-01..07): each dislodged unit retreats to one of
//! its precomputed retreat options or is disbanded.

use std::collections::{BTreeMap, HashMap};

use serde_json::json;

use super::common::{end_of_year, in_civil_disorder, order_text, position_text, step, strings, unit_label};
use super::{coast, province};
use crate::model::{
    DislodgedUnit, GameState, Order, OrderKind, OrderResult, Outcome, Phase, PhaseKind, Resolution, Season, TraceStep,
    Unit, UnitType,
};

fn as_unit(d: &DislodgedUnit) -> Unit {
    Unit {
        power: d.power,
        unit_type: d.unit_type,
        location: d.location.clone(),
        coast: d.coast.clone(),
    }
}

type Verdict = (bool, Outcome, Option<String>, Vec<&'static str>);

pub(crate) fn adjudicate(state: &GameState, orders: &[Order]) -> Resolution {
    let dislodged = &state.dislodged;
    let label = |k: usize| unit_label(&as_unit(&dislodged[k]));
    let mut steps: Vec<TraceStep> = Vec::new();
    let mut verdicts: Vec<Option<Verdict>> = vec![None; orders.len()];
    let ignored = |reason: String, rules: Vec<&'static str>| Some((false, Outcome::Void, Some(reason), rules));

    // Attribute orders to dislodged units (PR-14, PR-16, PR-05).
    let mut by_unit: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for (i, order) in orders.iter().enumerate() {
        let here = province(&order.unit);
        match dislodged.iter().position(|d| d.location == here) {
            None => verdicts[i] = ignored(format!("no dislodged unit in {here}"), vec!["PR-14"]),
            Some(k) if dislodged[k].power != order.power => {
                verdicts[i] = ignored(
                    format!("the dislodged unit in {here} is not {}'s", order.power.code()),
                    vec!["PR-14"],
                )
            }
            Some(_) if in_civil_disorder(state, order.power) => {
                verdicts[i] = ignored(
                    format!(
                        "{} is in civil disorder: its dislodged units are disbanded",
                        order.power.code()
                    ),
                    vec!["R-CD-02", "PR-16"],
                )
            }
            Some(k) => by_unit.entry(k).or_default().push(i),
        }
    }

    // Each dislodged unit's plan: Some(destination) to retreat, None to disband.
    let mut plan: Vec<Option<String>> = vec![None; dislodged.len()];
    let mut order_of: HashMap<usize, usize> = HashMap::new();
    for (&k, list) in &by_unit {
        let [i] = list[..] else {
            for &i in list {
                verdicts[i] = ignored(
                    format!(
                        "several orders for the unit in {}: all ignored, it is disbanded",
                        dislodged[k].location
                    ),
                    vec!["PR-05", "R-RET-07"],
                );
            }
            continue;
        };
        order_of.insert(k, i);
        verdicts[i] = Some(match &orders[i].kind {
            OrderKind::Disband => (true, Outcome::Success, None, vec!["R-RET-07"]),
            OrderKind::Retreat { to } => match retreat_target(&dislodged[k], to) {
                Ok(dest) => {
                    plan[k] = Some(dest);
                    (true, Outcome::Success, None, vec!["R-RET-02"])
                }
                Err((reason, rules)) => (
                    false,
                    Outcome::Void,
                    Some(format!("{reason}; the unit is disbanded")),
                    rules,
                ),
            },
            _ => (
                false,
                Outcome::Void,
                Some("not a retreat-phase order (retreat or disband); the unit is disbanded".into()),
                vec!["R-RET-01"],
            ),
        });
    }

    let unit_type = |p: &str| dislodged.iter().find(|d| d.location == p).map(|d| d.unit_type);
    for (i, (order, verdict)) in orders.iter().zip(&verdicts).enumerate() {
        let (valid, _, reason, rules) = verdict.as_ref().expect("every order gets a verdict");
        let text = order_text(order, unit_type);
        let outcome = if *valid {
            "valid"
        } else if order_of.values().any(|&j| j == i) {
            "invalid"
        } else {
            "ignored"
        };
        steps.push(step(
            "validate",
            rules,
            json!({ "order": text, "outcome": outcome, "reason": reason }),
        ));
    }
    let unordered: Vec<String> = (0..dislodged.len())
        .filter(|k| !order_of.contains_key(k))
        .map(label)
        .collect();
    if !unordered.is_empty() {
        steps.push(step(
            "validate",
            &["R-RET-07", "PR-09"],
            json!({ "outcome": "disband", "units": unordered, "reason": "no retreat order: the unit is disbanded" }),
        ));
    }

    // Two or more retreats into the same province: all are disbanded (R-RET-06).
    let mut by_dest: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for (k, dest) in plan.iter().enumerate() {
        if let Some(dest) = dest {
            by_dest.entry(province(dest).to_string()).or_default().push(k);
        }
    }
    for (dest, ks) in &by_dest {
        if ks.len() < 2 {
            continue;
        }
        steps.push(step(
            "conflicts",
            &["R-RET-06"],
            json!({ "province": dest, "units": ks.iter().map(|&k| label(k)).collect::<Vec<_>>(), "outcome": "all disbanded" }),
        ));
        for &k in ks {
            plan[k] = None;
            if let Some(&i) = order_of.get(&k) {
                verdicts[i] = Some((
                    true,
                    Outcome::Fail,
                    Some(format!("another unit also retreated to {dest}: both are disbanded")),
                    vec!["R-RET-06"],
                ));
            }
        }
    }

    // Apply.
    let mut next = GameState {
        phase: state.phase,
        units: state.units.clone(),
        supply_centers: state.supply_centers.clone(),
        dislodged: Vec::new(),
        standoff_provinces: Vec::new(),
        civil_disorder: state.civil_disorder.clone(),
        winner: state.winner,
    };
    let mut retreated = Vec::new();
    let mut disbanded = Vec::new();
    for (k, d) in dislodged.iter().enumerate() {
        match &plan[k] {
            Some(dest) => {
                let unit = Unit {
                    location: province(dest).to_string(),
                    coast: coast(dest).map(str::to_string),
                    ..as_unit(d)
                };
                retreated.push(format!("{}-{}", label(k), dest));
                next.units.push(unit);
            }
            None => disbanded.push(label(k)),
        }
    }

    let mut fall_steps = Vec::new();
    next.phase = if state.phase.season == Season::Spring {
        Phase {
            season: Season::Fall,
            kind: PhaseKind::Movement,
            ..state.phase
        }
    } else {
        let (s, phase) = end_of_year(&mut next);
        fall_steps = s;
        phase
    };
    steps.push(step(
        "result",
        &[],
        json!({
            "retreated": retreated,
            "disbanded": disbanded,
            "positions": next.units.iter().map(position_text).collect::<Vec<_>>(),
            "nextPhase": next.phase.to_string(),
        }),
    ));
    steps.extend(fall_steps);

    let results = orders
        .iter()
        .zip(verdicts)
        .map(|(order, v)| {
            let (valid, outcome, reason, rules) = v.expect("every order gets a verdict");
            OrderResult {
                order: order.clone(),
                valid,
                outcome,
                reason,
                rules: strings(&rules),
            }
        })
        .collect();
    Resolution {
        phase: state.phase,
        orders: results,
        trace: steps,
        next_state: next,
    }
}

/// Match a retreat destination against the unit's retreat options (R-RET-02/03, § 6.1). A coast
/// on an army order is ignored; a fleet's coast may be omitted when only one is open (PR-04).
fn retreat_target(d: &DislodgedUnit, to: &str) -> Result<String, (String, Vec<&'static str>)> {
    let dest = province(to);
    let candidates: Vec<&String> = d.retreat_options.iter().filter(|o| province(o) == dest).collect();
    let not_allowed = || {
        Err((
            format!("{dest} is not among the retreat options {:?}", d.retreat_options),
            vec!["R-RET-02", "R-RET-03"],
        ))
    };
    match (d.unit_type, coast(to), &candidates[..]) {
        (_, _, []) => not_allowed(),
        (UnitType::A, _, [one]) => Ok((*one).clone()),
        (UnitType::F, Some(c), _) => match candidates.iter().find(|o| coast(o) == Some(c)) {
            Some(o) => Ok((*o).clone()),
            None => Err((format!("coast {dest}/{c} is not a retreat option"), vec!["PR-04"])),
        },
        (UnitType::F, None, [one]) => Ok((*one).clone()),
        _ => Err((
            format!("both coasts of {dest} are open: the coast must be given"),
            vec!["R-MAP-07"],
        )),
    }
}
