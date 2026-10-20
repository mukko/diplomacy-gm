//! Legal orders for order entry (architecture.md § 6, `…/options` routes) and the proposed
//! default disbands of an adjustment phase. Every candidate is checked with the adjudicator's
//! own validation, so the GUI never offers an order the adjudicator would reject.

use std::collections::{BTreeSet, HashSet};

use serde::Serialize;

use super::adjustment::{check_build, default_disbands as choose_disbands};
use super::common::{build_sites, in_civil_disorder, sc_count, unit_count, unit_label};
use super::validate::{self as validation, Cmd, Verdict, convoy_possible, convoy_possible_via, validate};
use super::{Engine, province, province_type};
use crate::adjudicator::Adjudicator;
use crate::map::{self, ProvinceType};
use crate::model::{GameState, Order, OrderKind, PhaseKind, Power, Unit, UnitType};

/// The orders one unit may be given in the current phase.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnitOptions {
    pub unit: Unit,
    /// The power is in civil disorder: its orders are ignored (movement, retreat) and only
    /// GM-entered disbands are used (adjustment) — PR-16.
    pub civil_disorder: bool,
    /// Ready-to-submit orders, in the same shape as `PUT …/orders/:power`.
    pub orders: Vec<Order>,
}

/// Everything a power may order in the current phase.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PowerOptions {
    pub power: Power,
    pub civil_disorder: bool,
    /// Adjustment phase only: supply centers minus units (> 0 builds, < 0 disbands).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adjustment: Option<i64>,
    pub units: Vec<UnitOptions>,
    /// Adjustment phase only: legal builds (and a waive order) when the power may build.
    pub builds: Vec<Order>,
}

/// Disbands of one power in an adjustment phase (R-CD-03, PR-08, PR-09).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DisbandProposal {
    pub power: Power,
    pub civil_disorder: bool,
    /// Number of units the power must remove.
    pub required: usize,
    /// Disband orders already submitted (e.g. entered by the GM) that will be used.
    pub ordered: Vec<String>,
    /// Units the engine will remove for the rest, farthest from home first.
    pub proposed: Vec<ProposedDisband>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProposedDisband {
    pub unit: String,
    /// PR-08 distance to the nearest home SC.
    pub distance: usize,
}

/// Options for the unit in `location` (a dislodged unit in a retreat phase). `None` if there
/// is no such unit.
pub fn unit_options(state: &GameState, location: &str) -> Option<UnitOptions> {
    let here = province(location);
    match state.phase.kind {
        PhaseKind::Movement => {
            let u = state.units.iter().position(|u| u.location == here)?;
            Some(movement_options(state, u))
        }
        PhaseKind::Retreat => {
            let d = state.dislodged.iter().find(|d| d.location == here)?;
            let unit = Unit {
                power: d.power,
                unit_type: d.unit_type,
                location: d.location.clone(),
                coast: d.coast.clone(),
            };
            let disorder = in_civil_disorder(state, d.power);
            let mut orders: Vec<Order> = d
                .retreat_options
                .iter()
                .map(|to| order(d.power, here, OrderKind::Retreat { to: to.clone() }))
                .collect();
            orders.push(order(d.power, here, OrderKind::Disband));
            if disorder {
                orders.clear();
            }
            Some(UnitOptions {
                unit,
                civil_disorder: disorder,
                orders,
            })
        }
        PhaseKind::Adjustment => {
            let unit = state.units.iter().find(|u| u.location == here)?.clone();
            let must_disband = sc_count(state, unit.power) < unit_count(state, unit.power);
            let orders = if must_disband {
                vec![order(unit.power, here, OrderKind::Disband)]
            } else {
                Vec::new()
            };
            Some(UnitOptions {
                civil_disorder: in_civil_disorder(state, unit.power),
                unit,
                orders,
            })
        }
    }
}

/// Options for every unit of `power`, plus its builds in an adjustment phase.
pub fn power_options(state: &GameState, power: Power) -> PowerOptions {
    let locations: Vec<&str> = match state.phase.kind {
        PhaseKind::Retreat => state
            .dislodged
            .iter()
            .filter(|d| d.power == power)
            .map(|d| d.location.as_str())
            .collect(),
        _ => state
            .units
            .iter()
            .filter(|u| u.power == power)
            .map(|u| u.location.as_str())
            .collect(),
    };
    let units = locations.into_iter().filter_map(|l| unit_options(state, l)).collect();
    let disorder = in_civil_disorder(state, power);
    let (adjustment, builds) = if state.phase.kind == PhaseKind::Adjustment {
        let delta = sc_count(state, power) as i64 - unit_count(state, power) as i64;
        let builds = if delta > 0 && !disorder {
            build_options(state, power)
        } else {
            Vec::new()
        };
        (Some(delta), builds)
    } else {
        (None, Vec::new())
    };
    PowerOptions {
        power,
        civil_disorder: disorder,
        adjustment,
        units,
        builds,
    }
}

/// Proposed disbands per power that must remove units (adjustment phase only; empty
/// otherwise). Submitted disbands are validated as the adjudicator would and taken first.
pub fn default_disbands(state: &GameState, orders: &[Order]) -> Vec<DisbandProposal> {
    if state.phase.kind != PhaseKind::Adjustment {
        return Vec::new();
    }
    let Ok(resolution) = Engine.adjudicate(state, orders) else {
        return Vec::new();
    };
    Power::ALL
        .into_iter()
        .filter_map(|p| {
            let required = unit_count(state, p)
                .checked_sub(sc_count(state, p))
                .filter(|&n| n > 0)?;
            let removed: HashSet<usize> = resolution
                .orders
                .iter()
                .filter(|r| r.valid && r.order.power == p && r.order.kind == OrderKind::Disband)
                .filter_map(|r| state.units.iter().position(|u| u.location == province(&r.order.unit)))
                .collect();
            let mut ordered: Vec<String> = removed.iter().map(|&u| unit_label(&state.units[u])).collect();
            ordered.sort();
            let proposed = choose_disbands(state, p, &removed, required - removed.len())
                .into_iter()
                .map(|(u, distance)| ProposedDisband {
                    unit: unit_label(&state.units[u]),
                    distance,
                })
                .collect();
            Some(DisbandProposal {
                power: p,
                civil_disorder: in_civil_disorder(state, p),
                required,
                ordered,
                proposed,
            })
        })
        .collect()
}

/// How the adjudicator will treat a submitted order (architecture.md § 6, `PUT …/orders`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum OrderStatus {
    /// Executed as written (it may still fail or bounce during resolution).
    Valid,
    /// Movement phase: not executable, the unit holds instead (R-ORD-04).
    Hold,
    /// Retreat phase: not executable, the unit is disbanded (§ 6, R-RET-07).
    Disband,
    /// Applies to no unit or exceeds what the power may order; it has no effect
    /// (PR-05, PR-14, PR-15, PR-16, R-ADJ-01).
    Ignored,
}

#[derive(Debug, Clone, Serialize)]
pub struct OrderCheck {
    pub order: Order,
    pub status: OrderStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    pub rules: Vec<String>,
}

/// Validate one power's submitted orders for the current phase, exactly as the adjudicator
/// will (adjudication.md § 2). Results are in submission order.
pub fn check_orders(state: &GameState, orders: &[Order]) -> Vec<OrderCheck> {
    let check = |order: &Order, status, reason: Option<String>, rules: Vec<String>| OrderCheck {
        order: order.clone(),
        status,
        reason,
        rules,
    };
    if state.phase.kind == PhaseKind::Movement {
        let checked = validation::check(state, orders);
        return orders
            .iter()
            .zip(checked.verdicts)
            .map(|(order, verdict)| match verdict {
                Verdict::Accepted { .. } => check(order, OrderStatus::Valid, None, Vec::new()),
                Verdict::Invalid { reason, mut rules, .. } => {
                    rules.push("R-ORD-04");
                    check(order, OrderStatus::Hold, Some(reason), super::common::strings(&rules))
                }
                Verdict::Ignored { reason, rules } => check(
                    order,
                    OrderStatus::Ignored,
                    Some(reason),
                    super::common::strings(&rules),
                ),
            })
            .collect();
    }
    let Ok(resolution) = Engine.adjudicate(state, orders) else {
        return Vec::new();
    };
    resolution
        .orders
        .into_iter()
        .map(|r| {
            let own_dislodged = state.phase.kind == PhaseKind::Retreat
                && !in_civil_disorder(state, r.order.power)
                && state
                    .dislodged
                    .iter()
                    .any(|d| d.power == r.order.power && d.location == province(&r.order.unit));
            let status = match (r.valid, own_dislodged) {
                (true, _) => OrderStatus::Valid,
                (false, true) => OrderStatus::Disband,
                (false, false) => OrderStatus::Ignored,
            };
            // A valid order keeps no reason here: conflicts are decided only at resolution.
            let (reason, rules) = if r.valid {
                (None, Vec::new())
            } else {
                (r.reason, r.rules)
            };
            check(&r.order, status, reason, rules)
        })
        .collect()
}

fn order(power: Power, unit: &str, kind: OrderKind) -> Order {
    Order {
        power,
        unit: unit.to_string(),
        kind,
    }
}

/// Every location a unit could be ordered to move to: provinces, plus each coast of split-coast
/// provinces for fleets.
fn move_candidates(unit_type: UnitType) -> Vec<String> {
    let mut out = Vec::new();
    for p in &map::standard().provinces {
        match (&p.coasts, unit_type) {
            (Some(coasts), UnitType::F) => out.extend(coasts.iter().map(|c| format!("{}/{c}", p.id))),
            _ => out.push(p.id.clone()),
        }
    }
    out
}

/// Destinations `u` may legally be ordered to (validation only, as `validate` decides).
fn legal_moves(state: &GameState, u: usize) -> Vec<(String, bool)> {
    let unit = &state.units[u];
    move_candidates(unit.unit_type)
        .into_iter()
        .filter_map(|to| {
            let kind = OrderKind::Move {
                to: to.clone(),
                via_convoy: false,
            };
            match validate(state, u, &kind) {
                Ok(Cmd::Move { convoy, .. }) => Some((to, convoy)),
                _ => None,
            }
        })
        .collect()
}

fn movement_options(state: &GameState, u: usize) -> UnitOptions {
    let unit = state.units[u].clone();
    let here = unit.location.as_str();
    let power = unit.power;
    let disorder = in_civil_disorder(state, power);
    let mut orders = vec![order(power, here, OrderKind::Hold)];

    // Moves, and "via convoy" variants of land moves that a convoy could also make (PR-07).
    for (to, convoy_only) in legal_moves(state, u) {
        let via_possible = !convoy_only && unit.unit_type == UnitType::A && convoy_possible(state, here, &to);
        orders.push(order(
            power,
            here,
            OrderKind::Move {
                to: to.clone(),
                via_convoy: false,
            },
        ));
        if via_possible {
            orders.push(order(power, here, OrderKind::Move { to, via_convoy: true }));
        }
    }

    // Supports: hold support for any other unit; move support for the moves that unit may make.
    for (t, other) in state.units.iter().enumerate() {
        if t == u {
            continue;
        }
        let hold = OrderKind::Support {
            target: other.location.clone(),
            to: None,
        };
        if validate(state, u, &hold).is_ok() {
            orders.push(order(power, here, hold));
        }
        let destinations: BTreeSet<String> = legal_moves(state, t)
            .into_iter()
            .map(|(to, _)| province(&to).to_string())
            .collect();
        for to in destinations {
            let support = OrderKind::Support {
                target: other.location.clone(),
                to: Some(to),
            };
            if validate(state, u, &support).is_ok() {
                orders.push(order(power, here, support));
            }
        }
    }

    // Convoys: a fleet at sea may convoy any army whose convoy route could pass through it.
    if unit.unit_type == UnitType::F && province_type(here) == Some(ProvinceType::Sea) {
        for (a, army) in state.units.iter().enumerate() {
            if army.unit_type != UnitType::A {
                continue;
            }
            for (to, _) in legal_moves(state, a) {
                let kind = OrderKind::Convoy {
                    target: army.location.clone(),
                    to: to.clone(),
                };
                if validate(state, u, &kind).is_ok() && convoy_possible_via(state, &army.location, &to, here) {
                    orders.push(order(power, here, kind));
                }
            }
        }
    }

    if disorder {
        orders.clear();
    }
    UnitOptions {
        unit,
        civil_disorder: disorder,
        orders,
    }
}

/// Legal builds for `power` (R-ADJ-03..06), each home SC with every allowed unit type and
/// coast, followed by a waive order (R-ADJ-07).
fn build_options(state: &GameState, power: Power) -> Vec<Order> {
    let sites = build_sites(state, power);
    let mut out = Vec::new();
    for &site in &sites {
        let coasts: Vec<Option<String>> = match &map::standard().province(site).and_then(|p| p.coasts.clone()) {
            Some(cs) => cs.iter().cloned().map(Some).collect(),
            None => vec![None],
        };
        for unit_type in [UnitType::A, UnitType::F] {
            let coasts = if unit_type == UnitType::A {
                vec![None]
            } else {
                coasts.clone()
            };
            for coast in coasts {
                if check_build(state, &[], power, site, unit_type, coast.as_deref()).is_ok() {
                    out.push(order(power, site, OrderKind::Build { unit_type, coast }));
                }
            }
        }
    }
    if let Some(&site) = sites.first() {
        out.push(order(power, site, OrderKind::Waive));
    }
    out
}
