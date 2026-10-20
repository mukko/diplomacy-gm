//! Adjustment phase (adjudication.md § 7, R-ADJ-01..08): builds, disbands and waives, with
//! missing disbands chosen by R-CD-03 / PR-08 (PR-09) and civil disorder per PR-16.

use std::cmp::Reverse;
use std::collections::{BTreeMap, HashSet};

use serde_json::json;

use super::common::{
    build_sites, home_distance, in_civil_disorder, order_text, position_text, sc_count, step, strings, unit_count,
    unit_label,
};
use super::{coast, province};
use crate::map::{self, ProvinceType};
use crate::model::{
    GameState, Order, OrderKind, OrderResult, Outcome, Phase, PhaseKind, Power, Resolution, Season, TraceStep, Unit,
    UnitType,
};

type Verdict = (bool, Outcome, Option<String>, Vec<&'static str>);

fn ignored(reason: impl Into<String>, rules: Vec<&'static str>) -> Verdict {
    (false, Outcome::Void, Some(reason.into()), rules)
}

pub(crate) fn adjudicate(state: &GameState, orders: &[Order]) -> Resolution {
    let delta: BTreeMap<Power, i64> = Power::ALL
        .into_iter()
        .map(|p| (p, sc_count(state, p) as i64 - unit_count(state, p) as i64))
        .collect();
    let mut steps: Vec<TraceStep> = Vec::new();
    let mut verdicts = Vec::with_capacity(orders.len());
    let mut builds: Vec<Unit> = Vec::new();
    let mut used: BTreeMap<Power, i64> = BTreeMap::new(); // builds/waives or disbands accepted so far
    let mut waived: BTreeMap<Power, i64> = BTreeMap::new();
    let mut removed: HashSet<usize> = HashSet::new(); // indices into state.units

    // Orders are taken in submission order: once a power's entitlement is used up, the rest are
    // ignored (PR-15).
    for order in orders {
        let p = order.power;
        let d = delta[&p];
        let n = used.entry(p).or_default();
        let verdict = match &order.kind {
            OrderKind::Build { unit_type, coast: c } => {
                if d <= 0 {
                    ignored(
                        format!("{} has no builds this year", p.code()),
                        vec!["R-ADJ-03", "PR-15"],
                    )
                } else if in_civil_disorder(state, p) {
                    ignored(
                        format!("{} is in civil disorder: no builds", p.code()),
                        vec!["R-CD-02", "PR-16"],
                    )
                } else if *n >= d {
                    ignored("beyond the number of builds allowed", vec!["PR-15"])
                } else {
                    let c = c.as_deref().or_else(|| coast(&order.unit));
                    match check_build(state, &builds, p, province(&order.unit), *unit_type, c) {
                        Ok(unit) => {
                            *n += 1;
                            builds.push(unit);
                            (true, Outcome::Success, None, vec!["R-ADJ-03"])
                        }
                        Err((reason, rules)) => ignored(reason, rules),
                    }
                }
            }
            OrderKind::Waive => {
                if d <= 0 {
                    ignored(
                        format!("{} has no builds to waive", p.code()),
                        vec!["R-ADJ-07", "PR-15"],
                    )
                } else if *n >= d {
                    ignored("beyond the number of builds allowed", vec!["PR-15"])
                } else {
                    *n += 1;
                    *waived.entry(p).or_default() += 1;
                    (true, Outcome::Success, None, vec!["R-ADJ-07"])
                }
            }
            OrderKind::Disband => {
                let here = province(&order.unit);
                let unit = state.units.iter().position(|u| u.location == here);
                if d >= 0 {
                    ignored(
                        format!("{} does not need to disband", p.code()),
                        vec!["R-ADJ-02", "PR-15"],
                    )
                } else if *n >= -d {
                    ignored("beyond the number of disbands required", vec!["PR-15"])
                } else {
                    match unit {
                        Some(u) if state.units[u].power == p && !removed.contains(&u) => {
                            *n += 1;
                            removed.insert(u);
                            (true, Outcome::Success, None, vec!["R-ADJ-02"])
                        }
                        Some(u) if state.units[u].power == p => {
                            ignored("this unit is already disbanded", vec!["PR-15"])
                        }
                        _ => ignored(format!("{} has no unit in {here}", p.code()), vec!["PR-14"]),
                    }
                }
            }
            _ => ignored(
                "not an adjustment-phase order (build, disband or waive)",
                vec!["R-ADJ-01"],
            ),
        };
        verdicts.push(verdict);
    }

    let unit_type = |p: &str| state.units.iter().find(|u| u.location == p).map(|u| u.unit_type);
    for (order, (valid, _, reason, rules)) in orders.iter().zip(&verdicts) {
        let outcome = if *valid { "valid" } else { "ignored" };
        steps.push(step(
            "validate",
            rules,
            json!({ "order": order_text(order, unit_type), "outcome": outcome, "reason": reason }),
        ));
    }

    // Missing disbands are chosen by R-CD-03 / PR-08 (PR-09; PR-16 for civil disorder).
    let mut auto = Vec::new();
    for (&p, &d) in &delta {
        let missing = -d - used.get(&p).copied().unwrap_or(0);
        if d >= 0 || missing <= 0 {
            continue;
        }
        let chosen = default_disbands(state, p, &removed, missing as usize);
        let rules: &[&str] = if in_civil_disorder(state, p) {
            &["R-CD-03", "PR-08", "PR-16"]
        } else {
            &["R-CD-03", "PR-08", "PR-09"]
        };
        steps.push(step(
            "default-disbands",
            rules,
            json!({
                "power": p.code(),
                "missing": missing,
                "disbanded": chosen.iter().map(|&(u, dist)| json!({ "unit": unit_label(&state.units[u]), "distance": dist })).collect::<Vec<_>>(),
            }),
        ));
        for (u, _) in chosen {
            removed.insert(u);
            auto.push(unit_label(&state.units[u]));
        }
    }

    // Unused builds are waived (PR-09).
    let unused: BTreeMap<&str, i64> = delta
        .iter()
        .filter(|&(_, &d)| d > 0)
        .map(|(p, &d)| (p.code(), d - used.get(p).copied().unwrap_or(0)))
        .filter(|&(_, n)| n > 0)
        .collect();

    let mut units: Vec<Unit> = state
        .units
        .iter()
        .enumerate()
        .filter(|(i, _)| !removed.contains(i))
        .map(|(_, u)| u.clone())
        .collect();
    let built: Vec<String> = builds.iter().map(position_text).collect();
    units.extend(builds);
    let next = GameState {
        phase: Phase {
            season: Season::Spring,
            year: state.phase.year + 1,
            kind: PhaseKind::Movement,
        },
        units,
        supply_centers: state.supply_centers.clone(),
        dislodged: Vec::new(),
        standoff_provinces: Vec::new(),
        civil_disorder: state.civil_disorder.clone(),
        winner: state.winner,
    };
    // A power with no units and no SCs is out of the game (R-ADJ-08, PR-10).
    let eliminated: Vec<&str> = Power::ALL
        .into_iter()
        .filter(|&p| sc_count(&next, p) == 0 && unit_count(&next, p) == 0)
        .map(Power::code)
        .collect();
    let mut disbanded: Vec<String> = state
        .units
        .iter()
        .enumerate()
        .filter(|(i, _)| removed.contains(i))
        .map(|(_, u)| position_text(u))
        .collect();
    disbanded.sort();
    steps.push(step(
        "result",
        if eliminated.is_empty() {
            &["PR-09"]
        } else {
            &["PR-09", "PR-10"]
        },
        json!({
            "built": built,
            "disbanded": disbanded,
            "defaultDisbands": auto,
            "waived": waived.iter().map(|(p, n)| (p.code(), *n)).collect::<BTreeMap<_, _>>(),
            "unusedBuilds": unused,
            "eliminated": eliminated,
            "positions": next.units.iter().map(position_text).collect::<Vec<_>>(),
            "nextPhase": next.phase.to_string(),
        }),
    ));

    let results = orders
        .iter()
        .zip(verdicts)
        .map(|(order, (valid, outcome, reason, rules))| OrderResult {
            order: order.clone(),
            valid,
            outcome,
            reason,
            rules: strings(&rules),
        })
        .collect();
    Resolution {
        phase: state.phase,
        orders: results,
        trace: steps,
        next_state: next,
    }
}

/// A build is legal in an unoccupied home SC the power still controls (R-ADJ-03/05/06), with a
/// unit type the province allows and, for a fleet on a split coast, the coast (R-ADJ-04).
pub(crate) fn check_build(
    state: &GameState,
    builds: &[Unit],
    power: Power,
    here: &str,
    unit_type: UnitType,
    coast: Option<&str>,
) -> Result<Unit, (String, Vec<&'static str>)> {
    let Some(province) = map::standard().province(here) else {
        return Err((format!("unknown province `{here}`"), vec!["R-ADJ-01"]));
    };
    if province.home != Some(power) || !province.supply_center {
        return Err((
            format!("{here} is not a home supply center of {}", power.code()),
            vec!["R-ADJ-03"],
        ));
    }
    if builds.iter().any(|b| b.location == here) {
        return Err((
            format!("a unit is already being built in {here}"),
            vec!["R-UNIT-03", "PR-15"],
        ));
    }
    if !build_sites(state, power).contains(&here) {
        return Err((
            format!("{here} is occupied or no longer controlled by {}", power.code()),
            vec!["R-ADJ-03", "R-ADJ-05", "PR-15"],
        ));
    }
    let coast = match unit_type {
        UnitType::A => None, // a coast on an army build is ignored (PR-04)
        UnitType::F => {
            if province.kind != ProvinceType::Coast {
                return Err((format!("a fleet cannot be built in inland {here}"), vec!["R-ADJ-04"]));
            }
            match (&province.coasts, coast) {
                (Some(coasts), Some(c)) if coasts.iter().any(|x| x == c) => Some(c.to_string()),
                (Some(_), _) => {
                    return Err((format!("a fleet built in {here} needs its coast"), vec!["R-ADJ-04"]));
                }
                (None, Some(c)) => return Err((format!("{here} has no coast {c}"), vec!["R-ADJ-04", "PR-04"])),
                (None, None) => None,
            }
        }
    };
    Ok(Unit {
        power,
        unit_type,
        location: here.to_string(),
        coast,
    })
}

/// Units removed by default (R-CD-03, PR-08): farthest from any home SC first; ties fleets
/// before armies, then alphabetically by province name. Returns (unit index, distance).
pub(crate) fn default_disbands(
    state: &GameState,
    power: Power,
    removed: &HashSet<usize>,
    count: usize,
) -> Vec<(usize, usize)> {
    let map = map::standard();
    let name = |id: &str| {
        map.province(id)
            .map(|p| p.name.clone())
            .unwrap_or_else(|| id.to_string())
    };
    let mut candidates: Vec<(usize, usize)> = state
        .units
        .iter()
        .enumerate()
        .filter(|(i, u)| u.power == power && !removed.contains(i))
        .map(|(i, u)| (i, home_distance(&u.location, power)))
        .collect();
    candidates.sort_by_key(|&(i, dist)| {
        let u = &state.units[i];
        (Reverse(dist), u.unit_type == UnitType::A, name(&u.location))
    });
    candidates.truncate(count);
    candidates
}
