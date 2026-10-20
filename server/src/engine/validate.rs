//! Movement-phase order validation (adjudication.md § 2.2). Every unit ends up with exactly one
//! command; invalid orders become Hold (R-ORD-04), units without orders hold (R-ORD-03).

use std::collections::{HashMap, HashSet, VecDeque};

use super::{army_moves, can_reach, coast, fleet_moves, province, province_type, sea_borders};
use crate::map::ProvinceType;
use crate::model::{GameState, Order, OrderKind, Power, UnitType};

/// A unit's effective order after validation. Unit references are indices into `state.units`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Cmd {
    Hold,
    Move {
        to: String,
        /// Destination coast for fleets moving to a split-coast province.
        coast: Option<String>,
        /// The army travels by convoy (R-CVY-03, R-CVY-11, PR-07).
        convoy: bool,
    },
    Support {
        target: usize,
        /// Destination province of the supported move; `None` for hold support.
        to: Option<String>,
    },
    Convoy {
        army: usize,
        to: String,
    },
}

/// What validation decided about one submitted order.
#[derive(Debug, Clone)]
pub(crate) enum Verdict {
    Accepted {
        unit: usize,
    },
    /// Legal format but not executable: the unit holds (R-ORD-04).
    Invalid {
        unit: usize,
        reason: String,
        rules: Vec<&'static str>,
    },
    /// Not applied to any unit (PR-05, PR-14, PR-16).
    Ignored {
        reason: String,
        rules: Vec<&'static str>,
    },
}

/// How an army with a land route came to travel by convoy, or not (R-CVY-11, PR-07).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RouteChoice {
    /// Destination not adjacent: only a convoy can get there.
    ConvoyOnly,
    /// Adjacent, but the order says "via convoy".
    ViaConvoy,
    /// Adjacent, and at least one matching convoy is the army's own.
    OwnFleet,
    /// Adjacent, matching convoys exist but are all foreign: land route used.
    LandOverForeignConvoy,
}

pub(crate) struct Checked {
    pub cmds: Vec<Cmd>,
    pub verdicts: Vec<Verdict>,
    pub routes: HashMap<usize, RouteChoice>,
}

fn unit_at(state: &GameState, location: &str) -> Option<usize> {
    let p = province(location);
    state.units.iter().position(|u| u.location == p)
}

pub(crate) fn check(state: &GameState, orders: &[Order]) -> Checked {
    let disorder: HashSet<Power> = state.civil_disorder.iter().map(|c| c.power).collect();

    // Attribute orders to units first: PR-14 (no such unit / not yours), PR-16, PR-05.
    let mut verdicts: Vec<Option<Verdict>> = vec![None; orders.len()];
    let mut by_unit: HashMap<usize, Vec<usize>> = HashMap::new();
    for (i, order) in orders.iter().enumerate() {
        let ignored = |reason: String, rules| Some(Verdict::Ignored { reason, rules });
        match unit_at(state, &order.unit) {
            None => verdicts[i] = ignored(format!("no unit in {}", province(&order.unit)), vec!["PR-14"]),
            Some(u) if state.units[u].power != order.power => {
                verdicts[i] = ignored(
                    format!(
                        "the unit in {} is not {}'s",
                        state.units[u].location,
                        order.power.code()
                    ),
                    vec!["PR-14"],
                )
            }
            Some(_) if disorder.contains(&order.power) => {
                verdicts[i] = ignored(
                    format!("{} is in civil disorder: its units hold", order.power.code()),
                    vec!["PR-16", "R-CD-01"],
                )
            }
            Some(u) => by_unit.entry(u).or_default().push(i),
        }
    }
    let mut accepted: HashMap<usize, usize> = HashMap::new();
    for (u, list) in by_unit {
        if let [only] = list[..] {
            accepted.insert(u, only);
        } else {
            for i in list {
                verdicts[i] = Some(Verdict::Ignored {
                    reason: format!(
                        "several orders for the unit in {}: all ignored, it holds",
                        state.units[u].location
                    ),
                    rules: vec!["PR-05"],
                });
            }
        }
    }

    let mut cmds = vec![Cmd::Hold; state.units.len()];
    for (&u, &i) in &accepted {
        verdicts[i] = Some(match validate(state, u, &orders[i].kind) {
            Ok(cmd) => {
                cmds[u] = cmd;
                Verdict::Accepted { unit: u }
            }
            Err((reason, rules)) => Verdict::Invalid { unit: u, reason, rules },
        });
    }

    // Land route vs convoy route (R-CVY-11, PR-07) needs everyone's convoy orders.
    let mut routes = HashMap::new();
    for (&u, &i) in &accepted {
        let Cmd::Move { to, convoy, .. } = &cmds[u] else {
            continue;
        };
        if state.units[u].unit_type != UnitType::A {
            continue;
        }
        let via = matches!(orders[i].kind, OrderKind::Move { via_convoy: true, .. });
        let choice = if *convoy {
            RouteChoice::ConvoyOnly
        } else {
            let convoyers: Vec<usize> = (0..cmds.len())
                .filter(|&f| matches!(&cmds[f], Cmd::Convoy { army, to: t } if *army == u && t == to))
                .collect();
            if via {
                RouteChoice::ViaConvoy
            } else if convoyers.iter().any(|&f| state.units[f].power == state.units[u].power) {
                RouteChoice::OwnFleet
            } else if !convoyers.is_empty() {
                RouteChoice::LandOverForeignConvoy
            } else {
                continue;
            }
        };
        if matches!(choice, RouteChoice::ViaConvoy | RouteChoice::OwnFleet)
            && let Cmd::Move { convoy, .. } = &mut cmds[u]
        {
            *convoy = true;
        }
        routes.insert(u, choice);
    }

    Checked {
        cmds,
        verdicts: verdicts
            .into_iter()
            .map(|v| v.expect("every order gets a verdict"))
            .collect(),
        routes,
    }
}

pub(crate) type Invalid = (String, Vec<&'static str>);

pub(crate) fn validate(state: &GameState, u: usize, kind: &OrderKind) -> Result<Cmd, Invalid> {
    let unit = &state.units[u];
    let here = unit.location.as_str();
    match kind {
        OrderKind::Hold => Ok(Cmd::Hold),
        OrderKind::Move { to, .. } => {
            let dest = province(to);
            if province_type(dest).is_none() {
                return Err((format!("unknown province `{to}`"), vec!["R-ORD-04"]));
            }
            if dest == here {
                return Err((format!("{here} cannot move to itself"), vec!["R-MOV-01"]));
            }
            match unit.unit_type {
                // A coast on an army order is ignored (PR-04).
                UnitType::A => {
                    if army_moves(here).iter().any(|p| p == dest) {
                        Ok(Cmd::Move {
                            to: dest.to_string(),
                            coast: None,
                            convoy: false,
                        })
                    } else if convoy_possible(state, here, dest) {
                        Ok(Cmd::Move {
                            to: dest.to_string(),
                            coast: None,
                            convoy: true,
                        })
                    } else {
                        Err((
                            format!("an army in {here} cannot reach {dest}, not even by convoy"),
                            vec!["R-MOV-01", "R-CVY-03"],
                        ))
                    }
                }
                UnitType::F => fleet_destination(here, unit.coast.as_deref(), to).map(|coast| Cmd::Move {
                    to: dest.to_string(),
                    coast,
                    convoy: false,
                }),
            }
        }
        OrderKind::Support { target, to } => {
            let Some(t) = unit_at(state, target) else {
                return Err((format!("no unit in {} to support", province(target)), vec!["R-SUP-01"]));
            };
            if t == u {
                return Err(("a unit cannot support itself".into(), vec!["R-SUP-01"]));
            }
            let into = to.as_deref().map(province);
            if into == Some(state.units[t].location.as_str()) {
                return Err(("the supported move does not go anywhere".into(), vec!["R-SUP-06"]));
            }
            if let Some(p) = into
                && province_type(p).is_none()
            {
                return Err((format!("unknown province `{p}`"), vec!["R-ORD-04"]));
            }
            let into_prov = into.unwrap_or(&state.units[t].location);
            if !can_reach(unit, into_prov) {
                return Err((
                    format!("{here} could not move to {into_prov} itself, so it cannot support into it"),
                    vec!["R-SUP-02", "R-SUP-03", "R-SUP-07"],
                ));
            }
            // The supported army could not get there on this board: not adjacent, and no convoy
            // route without the supporter, which cannot convoy at the same time (PR-19).
            let target = &state.units[t];
            if let Some(p) = into
                && target.unit_type == UnitType::A
                && !army_moves(&target.location).iter().any(|m| m == p)
                && !convoy_route_exists(state, &target.location, p, Some(here))
            {
                return Err((
                    format!(
                        "an army in {} cannot reach {p} on this board, so it cannot be supported there",
                        target.location
                    ),
                    vec!["PR-19", "R-CVY-03"],
                ));
            }
            Ok(Cmd::Support {
                target: t,
                to: into.map(str::to_string),
            })
        }
        OrderKind::Convoy { target, to } => {
            if unit.unit_type != UnitType::F || province_type(here) != Some(ProvinceType::Sea) {
                return Err(("only a fleet in a water province can convoy".into(), vec!["R-CVY-01"]));
            }
            let Some(a) = unit_at(state, target) else {
                return Err((format!("no unit in {} to convoy", province(target)), vec!["R-CVY-04"]));
            };
            if state.units[a].unit_type != UnitType::A {
                return Err(("only armies can be convoyed".into(), vec!["R-CVY-02"]));
            }
            let dest = province(to);
            let coastal = |p: &str| province_type(p) == Some(ProvinceType::Coast);
            if !coastal(&state.units[a].location) || !coastal(dest) || dest == state.units[a].location {
                return Err((
                    "a convoy must go from one coastal province to another".into(),
                    vec!["R-CVY-03"],
                ));
            }
            if !convoy_possible_via(state, &state.units[a].location, dest, here) {
                return Err((
                    format!(
                        "{here} is not on any convoy route from {} to {dest} on this board",
                        state.units[a].location
                    ),
                    vec!["PR-19", "R-CVY-03"],
                ));
            }
            Ok(Cmd::Convoy {
                army: a,
                to: dest.to_string(),
            })
        }
        OrderKind::Retreat { .. } | OrderKind::Disband | OrderKind::Build { .. } | OrderKind::Waive => Err((
            "not a movement-phase order (hold, move, support or convoy)".into(),
            vec!["R-ORD-02"],
        )),
    }
}

/// Resolve a fleet's destination coast (R-MAP-06/07, PR-04).
fn fleet_destination(here: &str, here_coast: Option<&str>, to: &str) -> Result<Option<String>, Invalid> {
    let dest = province(to);
    let reachable: Vec<Option<&str>> = fleet_moves(here, here_coast)
        .iter()
        .filter(|l| province(l) == dest)
        .map(|l| coast(l))
        .collect();
    if reachable.is_empty() {
        return Err((
            format!("a fleet in {here} cannot move to {dest}"),
            vec!["R-MOV-02", "R-MAP-05", "R-MAP-06"],
        ));
    }
    match (coast(to), &reachable[..]) {
        (Some(c), _) if reachable.contains(&Some(c)) => Ok(Some(c.to_string())),
        (Some(c), _) => Err((format!("coast {dest}/{c} cannot be reached from {here}"), vec!["PR-04"])),
        (None, [one]) => Ok(one.map(str::to_string)),
        (None, _) => Err((
            format!("both coasts of {dest} can be reached: the coast must be given"),
            vec!["R-MAP-07"],
        )),
    }
}

/// Validation-time convoy check (R-CVY-03): both ends coastal and a chain of water provinces,
/// each holding a fleet, links them. Whether those fleets actually convoy is decided later.
pub(crate) fn convoy_possible(state: &GameState, from: &str, to: &str) -> bool {
    convoy_route_exists(state, from, to, None)
}

/// As [`convoy_possible`], without the fleet in `without` (a supporter cannot also convoy).
fn convoy_route_exists(state: &GameState, from: &str, to: &str, without: Option<&str>) -> bool {
    let coastal = |p: &str| province_type(p) == Some(ProvinceType::Coast);
    let seas = fleet_seas(state, without);
    coastal(from) && coastal(to) && fleet_seas_from(&seas, from).iter().any(|s| sea_borders(s, to))
}

/// Whether the fleet in `sea` is necessary for some convoy route from `from` to `to` on this
/// board, i.e. lies on a chain of fleets with no shortcut (PR-19, DATC 6.G.19).
pub(crate) fn convoy_possible_via(state: &GameState, from: &str, to: &str, sea: &str) -> bool {
    let seas = fleet_seas(state, None);
    let coastal = |p: &str| province_type(p) == Some(ProvinceType::Coast);
    if !coastal(from) || !coastal(to) || !seas.contains(&sea) {
        return false;
    }
    let mut path = Vec::new();
    seas.iter()
        .filter(|s| sea_borders(s, from))
        .any(|&first| minimal_route_through(&seas, &mut path, first, from, to, sea))
}

/// Depth-first search over chains `path + [next]` in which only the first fleet borders `from`,
/// only the last borders `to` and only consecutive fleets are adjacent.
fn minimal_route_through<'a>(
    seas: &[&'a str],
    path: &mut Vec<&'a str>,
    next: &'a str,
    from: &str,
    to: &str,
    sea: &str,
) -> bool {
    path.push(next);
    let found = if sea_borders(next, to) {
        path.contains(&sea)
    } else {
        seas.iter().any(|&s| {
            !path.contains(&s)
                && sea_borders(next, s)
                && !sea_borders(s, from)
                && path[..path.len() - 1].iter().all(|p| !sea_borders(p, s))
                && minimal_route_through(seas, path, s, from, to, sea)
        })
    };
    path.pop();
    found
}

/// Water provinces holding a fleet, except `without`.
fn fleet_seas<'a>(state: &'a GameState, without: Option<&str>) -> Vec<&'a str> {
    state
        .units
        .iter()
        .filter(|u| u.unit_type == UnitType::F && province_type(&u.location) == Some(ProvinceType::Sea))
        .map(|u| u.location.as_str())
        .filter(|&s| Some(s) != without)
        .collect()
}

/// Water provinces among `seas` that a chain of such provinces connects to the coast `from`.
fn fleet_seas_from<'a>(seas: &[&'a str], from: &str) -> HashSet<&'a str> {
    let mut seen: HashSet<&str> = seas.iter().copied().filter(|s| sea_borders(s, from)).collect();
    let mut queue: VecDeque<&str> = seen.iter().copied().collect();
    while let Some(sea) = queue.pop_front() {
        for &next in seas {
            if !seen.contains(next) && sea_borders(sea, next) {
                seen.insert(next);
                queue.push_back(next);
            }
        }
    }
    seen
}
