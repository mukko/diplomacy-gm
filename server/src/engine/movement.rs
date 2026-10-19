//! Movement phase: validation → resolution → trace and next position (adjudication.md § 3–4).

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use serde_json::{Value, json};

use super::resolve::{CutReason, MoveEval, Opposition, Resolver};
use super::validate::{self, Checked, Cmd, RouteChoice, Verdict};
use super::{legal_moves, province};
use crate::map;
use crate::model::{
    DislodgedUnit, GameState, Order, OrderKind, OrderResult, Outcome, Phase, PhaseKind, Power, Resolution, Season,
    TraceStep, Unit, UnitType,
};

/// SCs needed to win (R-GEN-03).
const VICTORY_SCS: usize = 18;

pub(crate) fn adjudicate(state: &GameState, orders: &[Order]) -> Resolution {
    let checked = validate::check(state, orders);
    let mut r = Resolver::new(&state.units, &checked.cmds);
    r.resolve_all();
    let mut t = Tracer {
        state,
        checked: &checked,
        steps: Vec::new(),
    };

    t.validate_steps(orders);
    t.convoy_route_steps(&mut r);
    let per_unit = t.support_and_move_steps(&mut r);
    let after = t.dislodgement_steps(&mut r);
    let next_state = t.result_steps(after);

    let results = orders
        .iter()
        .zip(&checked.verdicts)
        .map(|(order, verdict)| match verdict {
            Verdict::Ignored { reason, rules } | Verdict::Invalid { reason, rules, .. } => OrderResult {
                order: order.clone(),
                valid: false,
                outcome: Outcome::Void,
                reason: Some(reason.clone()),
                rules: strings(rules),
            },
            Verdict::Accepted { unit } => {
                let (outcome, reason, rules) =
                    per_unit
                        .get(unit)
                        .cloned()
                        .unwrap_or((Outcome::Success, None, Vec::new()));
                OrderResult {
                    order: order.clone(),
                    valid: true,
                    outcome,
                    reason,
                    rules: strings(&rules),
                }
            }
        })
        .collect();

    Resolution {
        phase: state.phase,
        orders: results,
        trace: t.steps,
        next_state,
    }
}

type UnitOutcome = (Outcome, Option<String>, Vec<&'static str>);

struct Tracer<'a> {
    state: &'a GameState,
    checked: &'a Checked,
    steps: Vec<TraceStep>,
}

/// Units and their fate at the end of resolution.
struct AfterMoves {
    units: Vec<Unit>,
    dislodged: Vec<DislodgedUnit>,
    standoffs: BTreeSet<String>,
}

impl Tracer<'_> {
    fn push(&mut self, step: &str, rules: &[&str], details: Value) {
        let Value::Object(details) = details else {
            unreachable!("trace details are objects")
        };
        self.steps.push(TraceStep {
            step: step.to_string(),
            rules: strings(rules),
            details,
        });
    }

    fn label(&self, u: usize) -> String {
        unit_label(&self.state.units[u])
    }

    /// The order a unit actually executes, in rulebook notation.
    fn cmd_text(&self, u: usize) -> String {
        let units = &self.state.units;
        let me = self.label(u);
        match &self.checked.cmds[u] {
            Cmd::Hold => format!("{me} H"),
            Cmd::Move { to, coast, .. } => format!("{me}-{}", with_coast(to, coast.as_deref())),
            Cmd::Support { target, to: None } => format!("{me} S {}", self.label(*target)),
            Cmd::Support { target, to: Some(p) } => format!("{me} S {}-{p}", self.label(*target)),
            Cmd::Convoy { army, to } => format!("{me} C A {}-{to}", units[*army].location),
        }
    }

    // ----- 1. validate --------------------------------------------------------------------

    fn validate_steps(&mut self, orders: &[Order]) {
        let mut ordered = HashSet::new();
        for (order, verdict) in orders.iter().zip(&self.checked.verdicts) {
            let text = order_text(self.state, order);
            match verdict {
                Verdict::Accepted { unit } => {
                    ordered.insert(*unit);
                    self.push("validate", &[], json!({ "order": text, "outcome": "valid" }));
                }
                Verdict::Invalid { unit, reason, rules } => {
                    ordered.insert(*unit);
                    let mut rules = rules.clone();
                    rules.push("R-ORD-04");
                    self.push(
                        "validate",
                        &rules,
                        json!({ "order": text, "outcome": "invalid", "reason": reason, "replacedBy": format!("{} H", self.label(*unit)) }),
                    );
                }
                Verdict::Ignored { reason, rules } => {
                    self.push(
                        "validate",
                        rules,
                        json!({ "order": text, "outcome": "ignored", "reason": reason }),
                    );
                }
            }
        }
        let unordered: Vec<String> = (0..self.state.units.len())
            .filter(|u| !ordered.contains(u))
            .map(|u| self.label(u))
            .collect();
        if !unordered.is_empty() {
            self.push(
                "validate",
                &["R-ORD-03"],
                json!({ "outcome": "hold", "units": unordered, "reason": "no order: the unit holds" }),
            );
        }
    }

    // ----- 2. convoy-routes -----------------------------------------------------------------

    fn convoy_route_steps(&mut self, r: &mut Resolver) {
        let routes: BTreeMap<usize, RouteChoice> = self.checked.routes.iter().map(|(k, v)| (*k, v.clone())).collect();
        for (u, choice) in routes {
            let order = self.cmd_text(u);
            if choice == RouteChoice::LandOverForeignConvoy {
                self.push(
                    "convoy-routes",
                    &["R-CVY-11"],
                    json!({ "order": order, "route": "land", "reason": "all convoying fleets are foreign and the order does not say via convoy" }),
                );
                continue;
            }
            let mut rules = vec![match choice {
                RouteChoice::ConvoyOnly => "R-CVY-03",
                RouteChoice::ViaConvoy => "PR-07",
                _ => "R-CVY-11",
            }];
            let fleets: Vec<String> = r.convoyers(u).iter().map(|&f| self.label(f)).collect();
            let broken: Vec<String> = r
                .convoyers(u)
                .to_vec()
                .into_iter()
                .filter(|&f| r.dislodged_by(f).is_some())
                .map(|f| self.label(f))
                .collect();
            let outcome = if r.szykman[u] {
                rules.push("PR-03");
                "paradox"
            } else if r.path(u) {
                if !broken.is_empty() {
                    rules.push("R-CVY-08");
                }
                "intact"
            } else if fleets.is_empty() {
                rules.push("R-CVY-04");
                "no-convoy"
            } else {
                rules.push("R-CVY-05");
                "disrupted"
            };
            self.push(
                "convoy-routes",
                &rules,
                json!({ "order": order, "route": "convoy", "outcome": outcome, "fleets": fleets, "dislodgedFleets": broken }),
            );
        }
    }

    // ----- 3–5. support-cuts, strengths, moves ------------------------------------------------

    fn support_and_move_steps(&mut self, r: &mut Resolver) -> HashMap<usize, UnitOutcome> {
        let n = self.state.units.len();
        let mut per_unit: HashMap<usize, UnitOutcome> = HashMap::new();

        for s in 0..n {
            let Cmd::Support { target, to } = &self.checked.cmds[s] else {
                continue;
            };
            let order = self.cmd_text(s);
            if !r.matched_supports(*target).contains(&s) {
                let (reason, rule) = if to.is_none() {
                    (
                        "the supported unit moves, so it cannot be supported to hold",
                        "R-SUP-05",
                    )
                } else {
                    ("the supported unit was not ordered to make that move", "R-SUP-06")
                };
                self.push(
                    "support-cuts",
                    &[rule],
                    json!({ "order": order, "outcome": "void", "reason": reason }),
                );
                per_unit.insert(s, (Outcome::Void, Some(reason.into()), vec![rule]));
                continue;
            }
            let eval = r.eval_cut(s, true);
            let not_cut: Vec<Value> = eval
                .not_cut
                .iter()
                .map(|nc| json!({ "by": self.cmd_text(nc.by), "rules": [nc.rule] }))
                .collect();
            match eval.cut {
                Some(CutReason::Attacked { by, rule }) => {
                    let reason = format!("cut by {}", self.cmd_text(by));
                    self.push(
                        "support-cuts",
                        &[rule],
                        json!({ "order": order, "outcome": "cut", "by": self.cmd_text(by), "notCutBy": not_cut }),
                    );
                    per_unit.insert(s, (Outcome::Cut, Some(reason), vec![rule]));
                }
                Some(CutReason::Dislodged { by }) => {
                    let reason = format!("supporter dislodged by {}", self.cmd_text(by));
                    self.push("support-cuts", &["R-CUT-02"], json!({ "order": order, "outcome": "cut", "by": self.cmd_text(by), "reason": "dislodged", "notCutBy": not_cut }));
                    per_unit.insert(s, (Outcome::Cut, Some(reason), vec!["R-CUT-02"]));
                }
                None => {
                    let rules: Vec<&str> = eval.not_cut.iter().map(|nc| nc.rule).collect();
                    self.push(
                        "support-cuts",
                        &rules,
                        json!({ "order": order, "outcome": "given", "notCutBy": not_cut }),
                    );
                    per_unit.insert(s, (Outcome::Success, None, rules));
                }
            }
        }

        // Strengths per contested province.
        let targets: BTreeSet<String> = (0..n)
            .filter_map(|u| match &self.checked.cmds[u] {
                Cmd::Move { to, .. } => Some(to.clone()),
                _ => None,
            })
            .collect();
        for p in &targets {
            let (hold, hold_supports) = r.hold_strength(p);
            let mut rules = vec!["R-MOV-04", "R-SUP-01"];
            let mut moves = Vec::new();
            for m in r.moves_into(p) {
                let attack = r.attack(m);
                if !attack.excluded.is_empty() {
                    rules.extend(["R-SLF-02", "PR-02"]);
                }
                let mut entry = json!({
                    "order": self.cmd_text(m),
                    "attack": attack.strength,
                    "supports": attack.supports.iter().map(|&s| self.cmd_text(s)).collect::<Vec<_>>(),
                    "prevent": r.prevent(m),
                });
                if !attack.excluded.is_empty() {
                    entry["excludedSupports"] =
                        json!(attack.excluded.iter().map(|&s| self.cmd_text(s)).collect::<Vec<_>>());
                }
                if r.head_to_head(m).is_some() {
                    entry["defend"] = json!(r.defend(m));
                }
                moves.push(entry);
            }
            rules.dedup();
            let holder = r.unit_at(p).map(|u| self.label(u));
            self.push(
                "strengths",
                &rules,
                json!({
                    "province": p,
                    "holder": holder,
                    "hold": hold,
                    "holdSupports": hold_supports.iter().map(|&s| self.cmd_text(s)).collect::<Vec<_>>(),
                    "moves": moves,
                }),
            );
        }

        // Move results.
        for m in 0..n {
            if !r.is_move(m) {
                continue;
            }
            let eval = r.eval_move(m, true);
            let (comparison, rules) = self.explain_move(r, m, &eval);
            let outcome = if eval.success { Outcome::Success } else { Outcome::Fail };
            self.push(
                "moves",
                &rules,
                json!({ "order": self.cmd_text(m), "outcome": if eval.success { "success" } else { "fail" }, "comparison": comparison }),
            );
            per_unit.insert(m, (outcome, Some(comparison), rules));
        }

        // Holds and convoys: their outcome depends on dislodgement and the convoyed army.
        for u in 0..n {
            match &self.checked.cmds[u] {
                Cmd::Hold => {
                    per_unit.insert(
                        u,
                        match r.dislodged_by(u) {
                            Some(by) => (
                                Outcome::Fail,
                                Some(format!("dislodged by {}", self.cmd_text(by))),
                                vec![],
                            ),
                            None => (Outcome::Success, None, vec!["R-HLD-01"]),
                        },
                    );
                }
                Cmd::Convoy { army, .. } => {
                    per_unit.insert(
                        u,
                        if !r.convoyers(*army).contains(&u) {
                            if self.checked.routes.get(army) == Some(&RouteChoice::LandOverForeignConvoy) {
                                (
                                    Outcome::Void,
                                    Some("the army takes the land route".into()),
                                    vec!["R-CVY-11"],
                                )
                            } else {
                                (
                                    Outcome::Void,
                                    Some("the army was not ordered to make that move".into()),
                                    vec!["R-CVY-04", "PR-12"],
                                )
                            }
                        } else if let Some(by) = r.dislodged_by(u) {
                            (
                                Outcome::Fail,
                                Some(format!("dislodged by {}", self.cmd_text(by))),
                                vec!["R-CVY-05"],
                            )
                        } else if r.szykman[*army] {
                            (Outcome::Fail, Some("convoy paradox".into()), vec!["PR-03"])
                        } else if r.path(*army) {
                            (Outcome::Success, None, vec!["R-CVY-03"])
                        } else {
                            (
                                Outcome::Fail,
                                Some("the convoy route is broken elsewhere".into()),
                                vec!["R-CVY-05"],
                            )
                        },
                    );
                }
                _ => {}
            }
        }
        per_unit
    }

    fn explain_move(&self, r: &mut Resolver, m: usize, eval: &MoveEval) -> (String, Vec<&'static str>) {
        let a = &eval.attack;
        if a.no_path {
            return if r.szykman[m] {
                ("convoy paradox: the army does not move".into(), vec!["PR-03"])
            } else if r.convoyers(m).is_empty() {
                ("no matching convoy orders".into(), vec!["R-CVY-04"])
            } else {
                ("no intact convoy route".into(), vec!["R-CVY-05"])
            };
        }
        if a.own_unit {
            return ("cannot dislodge a unit of its own power".into(), vec!["R-SLF-01"]);
        }
        let (label, against) = match &eval.opposition {
            Opposition::Hold(s) => ("hold", *s),
            Opposition::Defend { strength, .. } => ("defend", *strength),
        };
        let prevents: Vec<String> = eval
            .prevents
            .iter()
            .map(|&(o, p)| format!("prevent {p} ({})", self.cmd_text(o)))
            .collect();
        if eval.success {
            let mut text = format!("attack {} > {label} {against}", a.strength);
            for p in &prevents {
                text.push_str(&format!(", > {p}"));
            }
            let mut rules = vec!["R-MOV-04"];
            if r.circular[m] {
                rules.push("R-STD-05");
            }
            if r.head_to_head(m).is_some() {
                rules.push("PR-01");
            }
            if r.is_convoyed(m) {
                rules.push("R-CVY-03");
            }
            return (text, rules);
        }
        if a.strength <= against {
            let rule = match &eval.opposition {
                Opposition::Defend { .. } if a.strength == against => "R-STD-04",
                Opposition::Defend { .. } => "PR-01",
                Opposition::Hold(_) if !a.excluded.is_empty() => "R-SLF-02",
                Opposition::Hold(_) if r.unit_at(province(&self.target(m))).is_some_and(|o| r.is_move(o)) => "R-STD-03",
                Opposition::Hold(_) => "R-MOV-04",
            };
            return (format!("attack {} ≤ {label} {against}", a.strength), vec![rule]);
        }
        let (o, p) = eval
            .prevents
            .iter()
            .copied()
            .find(|&(_, p)| a.strength <= p)
            .expect("a prevent strength decided it");
        let mut rules = vec!["R-STD-01"];
        if r.dislodged_by(o).is_some() {
            rules.push("R-STD-06");
        }
        (
            format!("standoff: attack {} ≤ prevent {p} ({})", a.strength, self.cmd_text(o)),
            rules,
        )
    }

    fn target(&self, m: usize) -> String {
        match &self.checked.cmds[m] {
            Cmd::Move { to, .. } => to.clone(),
            _ => String::new(),
        }
    }

    // ----- 6. dislodgements ---------------------------------------------------------------

    fn dislodgement_steps(&mut self, r: &mut Resolver) -> AfterMoves {
        let n = self.state.units.len();
        let disorder: HashSet<Power> = self.state.civil_disorder.iter().map(|c| c.power).collect();
        let mut units = Vec::new();
        let mut beaten = Vec::new();
        for u in 0..n {
            let unit = &self.state.units[u];
            if let Some(by) = r.dislodged_by(u) {
                beaten.push((u, by));
            } else if r.moves(u) {
                let Cmd::Move { to, coast, .. } = &self.checked.cmds[u] else {
                    unreachable!()
                };
                units.push(Unit {
                    location: to.clone(),
                    coast: coast.clone(),
                    ..unit.clone()
                });
            } else {
                units.push(unit.clone());
            }
        }
        let occupied: HashSet<&str> = units.iter().map(|u| u.location.as_str()).collect();

        // A province is a standoff province if it ends empty although a move with a non-zero
        // prevent strength failed to enter it (R-RET-03(c), R-RET-04).
        let mut standoffs = BTreeSet::new();
        for m in 0..n {
            if r.is_move(m) && !r.moves(m) {
                let to = self.target(m);
                if !occupied.contains(to.as_str()) && r.prevent(m) > 0 {
                    standoffs.insert(to);
                }
            }
        }

        let mut dislodged = Vec::new();
        for (u, by) in beaten {
            let unit = &self.state.units[u];
            let attacker_from = self.state.units[by].location.clone();
            let options: Vec<String> = legal_moves(unit)
                .into_iter()
                .filter(|l| {
                    let p = province(l);
                    !occupied.contains(p) && p != attacker_from && !standoffs.contains(p)
                })
                .collect();
            let details = json!({
                "unit": self.label(u),
                "by": self.cmd_text(by),
                "attackerFrom": attacker_from,
                "retreatOptions": options,
            });
            if disorder.contains(&unit.power) {
                self.push(
                    "dislodgements",
                    &["R-CD-02", "PR-16"],
                    with(details, "outcome", "disbanded"),
                );
            } else if options.is_empty() {
                self.push("dislodgements", &["R-RET-05"], with(details, "outcome", "disbanded"));
            } else {
                self.push(
                    "dislodgements",
                    &["R-RET-02", "R-RET-03"],
                    with(details, "outcome", "must-retreat"),
                );
                dislodged.push(DislodgedUnit {
                    power: unit.power,
                    unit_type: unit.unit_type,
                    location: unit.location.clone(),
                    coast: unit.coast.clone(),
                    attacker_from,
                    retreat_options: options,
                });
            }
        }
        AfterMoves {
            units,
            dislodged,
            standoffs,
        }
    }

    // ----- 7. result (+ Fall SC update when there is no retreat phase) -------------------------

    fn result_steps(&mut self, after: AfterMoves) -> GameState {
        let phase = self.state.phase;
        let mut next = GameState {
            phase,
            units: after.units,
            supply_centers: self.state.supply_centers.clone(),
            dislodged: after.dislodged,
            standoff_provinces: Vec::new(),
            civil_disorder: self.state.civil_disorder.clone(),
            winner: self.state.winner,
        };
        let positions: Vec<String> = next.units.iter().map(position_text).collect();
        let standoffs: Vec<String> = after.standoffs.iter().cloned().collect();
        let mut fall_steps = Vec::new();

        next.phase = if !next.dislodged.is_empty() {
            next.standoff_provinces = standoffs.clone();
            Phase {
                kind: PhaseKind::Retreat,
                ..phase
            }
        } else if phase.season == Season::Spring {
            Phase {
                season: Season::Fall,
                ..phase
            }
        } else {
            let before = self.steps.len();
            self.update_supply_centers(&mut next);
            fall_steps = self.steps.split_off(before);
            if adjustments_needed(&next) {
                Phase {
                    season: Season::Winter,
                    kind: PhaseKind::Adjustment,
                    ..phase
                }
            } else {
                Phase {
                    season: Season::Spring,
                    year: phase.year + 1,
                    kind: PhaseKind::Movement,
                }
            }
        };
        let mut rules = vec![];
        if !standoffs.is_empty() {
            rules.push("R-STD-02");
        }
        self.push(
            "result",
            &rules,
            json!({ "positions": positions, "standoffProvinces": standoffs, "nextPhase": next.phase.to_string() }),
        );
        self.steps.extend(fall_steps);
        next
    }

    /// After Fall with no retreat phase: SC ownership (R-SC-01/02) and victory check (PR-11).
    fn update_supply_centers(&mut self, next: &mut GameState) {
        let mut changes = Vec::new();
        for unit in &next.units {
            if let Some(owner) = next.supply_centers.get_mut(&unit.location)
                && *owner != Some(unit.power)
            {
                changes.push(
                    json!({ "province": unit.location, "from": owner.map(Power::code), "to": unit.power.code() }),
                );
                *owner = Some(unit.power);
            }
        }
        self.push("sc-ownership", &["R-SC-01", "R-SC-02"], json!({ "changes": changes }));
        let counts: BTreeMap<&str, usize> = Power::ALL
            .iter()
            .map(|p| {
                (
                    p.code(),
                    next.supply_centers.values().filter(|o| **o == Some(*p)).count(),
                )
            })
            .collect();
        if next.winner.is_none() {
            next.winner = Power::ALL
                .into_iter()
                .find(|p| next.supply_centers.values().filter(|o| **o == Some(*p)).count() >= VICTORY_SCS);
        }
        self.push(
            "victory-check",
            &["R-GEN-03", "PR-11"],
            json!({ "supplyCenters": counts, "winner": next.winner.map(Power::code) }),
        );
    }
}

/// The adjustment phase is skipped when no power must disband or can build (architecture.md § 3,
/// R-ADJ-03/05, R-CD-02).
fn adjustments_needed(state: &GameState) -> bool {
    let map = map::standard();
    Power::ALL.into_iter().any(|p| {
        let scs = state.supply_centers.values().filter(|o| **o == Some(p)).count();
        let units = state.units.iter().filter(|u| u.power == p).count();
        let in_disorder = state.civil_disorder.iter().any(|c| c.power == p);
        units > scs
            || (scs > units
                && !in_disorder
                && map.supply_centers().any(|sc| {
                    sc.home == Some(p)
                        && state.supply_centers.get(&sc.id) == Some(&Some(p))
                        && !state.units.iter().any(|u| u.location == sc.id)
                }))
    })
}

fn strings(rules: &[&str]) -> Vec<String> {
    rules.iter().map(|r| r.to_string()).collect()
}

fn with(mut v: Value, key: &str, value: &str) -> Value {
    v[key] = json!(value);
    v
}

fn type_letter(t: UnitType) -> &'static str {
    match t {
        UnitType::A => "A",
        UnitType::F => "F",
    }
}

fn with_coast(location: &str, coast: Option<&str>) -> String {
    match coast {
        Some(c) => format!("{location}/{c}"),
        None => location.to_string(),
    }
}

fn unit_label(unit: &Unit) -> String {
    format!(
        "{} {}",
        type_letter(unit.unit_type),
        with_coast(&unit.location, unit.coast.as_deref())
    )
}

fn position_text(unit: &Unit) -> String {
    format!("{} {}", unit.power.code(), unit_label(unit))
}

/// A submitted order in rulebook notation, e.g. `A Gas S A Mar-Bur` (unit types from the board).
fn order_text(state: &GameState, order: &Order) -> String {
    let unit_type = |loc: &str| {
        let p = province(loc);
        state
            .units
            .iter()
            .find(|u| u.location == p)
            .map(|u| format!("{} ", type_letter(u.unit_type)))
            .unwrap_or_default()
    };
    let me = format!("{}{}", unit_type(&order.unit), order.unit);
    match &order.kind {
        OrderKind::Hold => format!("{me} H"),
        OrderKind::Move { to, via_convoy } => {
            format!("{me}-{to}{}", if *via_convoy { " via convoy" } else { "" })
        }
        OrderKind::Support { target, to: None } => format!("{me} S {}{target}", unit_type(target)),
        OrderKind::Support { target, to: Some(to) } => format!("{me} S {}{target}-{to}", unit_type(target)),
        OrderKind::Convoy { target, to } => format!("{me} C {}{target}-{to}", unit_type(target)),
        OrderKind::Retreat { to } => format!("{me} R {to}"),
        OrderKind::Disband => format!("{me} D"),
        OrderKind::Build { unit_type, coast } => {
            format!(
                "B {} {}",
                type_letter(*unit_type),
                with_coast(&order.unit, coast.as_deref())
            )
        }
        OrderKind::Waive => format!("{} waive", order.power.code()),
    }
}
