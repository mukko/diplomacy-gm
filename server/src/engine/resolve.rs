//! Decision-based resolution of the movement phase (adjudication.md § 3), after
//! L. B. Kruijswijk's "The Math of Adjudication": every move, support and convoy path is a
//! yes/no decision, resolved recursively with guess-and-check for dependency cycles (§ 3.4).

use std::collections::{HashMap, VecDeque};

use super::sea_borders;
use super::validate::Cmd;
use crate::model::Unit;

const MOVE: usize = 0;
const SUPPORT: usize = 1;
const PATH: usize = 2;
const KINDS: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Unresolved,
    Guessing,
    Resolved,
}

/// What a move is up against in its target province.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Opposition {
    /// Hold strength of the target province (§ 3.2).
    Hold(u32),
    /// Defend strength of the unit coming the other way (head-to-head, PR-01).
    Defend { unit: usize, strength: u32 },
}

#[derive(Debug, Clone)]
pub(crate) struct Attack {
    pub strength: u32,
    pub supports: Vec<usize>,
    /// Supports not counted because they come from the defender's power (R-SLF-02).
    pub excluded: Vec<usize>,
    /// The target holds a unit of the attacker's own power that stays (R-SLF-01).
    pub own_unit: bool,
    pub no_path: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct MoveEval {
    pub success: bool,
    pub attack: Attack,
    pub opposition: Opposition,
    /// Prevent strengths of the other moves into the same province.
    pub prevents: Vec<(usize, u32)>,
}

#[derive(Debug, Clone)]
pub(crate) enum CutReason {
    /// Attacked from a province other than the one supported into (R-CUT-01, R-CVY-10).
    Attacked { by: usize, rule: &'static str },
    /// The supporter is dislodged (R-CUT-02).
    Dislodged { by: usize },
}

/// An attack on the supporter that does not cut its support, and why.
#[derive(Debug, Clone)]
pub(crate) struct NotCut {
    pub by: usize,
    pub rule: &'static str,
}

#[derive(Debug, Clone)]
pub(crate) struct CutEval {
    pub cut: Option<CutReason>,
    pub not_cut: Vec<NotCut>,
}

pub(crate) struct Resolver<'a> {
    pub units: &'a [Unit],
    pub cmds: &'a [Cmd],
    at: HashMap<&'a str, usize>,
    moves_into: HashMap<&'a str, Vec<usize>>,
    /// Supports that match each unit's actual order (R-SUP-05/06), by supported unit.
    supports: Vec<Vec<usize>>,
    /// Matching convoy orders, by convoyed army (R-CVY-04).
    convoyers: Vec<Vec<usize>>,
    /// Opposing unit of a head-to-head battle without convoy (R-STD-04, PR-01).
    h2h: Vec<Option<usize>>,
    state: Vec<State>,
    value: Vec<bool>,
    deps: Vec<usize>,
    /// Convoyed armies stopped by the Szykman rule (PR-03).
    pub szykman: Vec<bool>,
    /// Moves that succeed as part of a circular movement (R-STD-05).
    pub circular: Vec<bool>,
}

impl<'a> Resolver<'a> {
    pub fn new(units: &'a [Unit], cmds: &'a [Cmd]) -> Self {
        let n = units.len();
        let at: HashMap<&str, usize> = units
            .iter()
            .enumerate()
            .map(|(i, u)| (u.location.as_str(), i))
            .collect();
        let mut moves_into: HashMap<&str, Vec<usize>> = HashMap::new();
        let mut supports = vec![Vec::new(); n];
        let mut convoyers = vec![Vec::new(); n];
        for (i, cmd) in cmds.iter().enumerate() {
            match cmd {
                Cmd::Move { to, .. } => moves_into.entry(to.as_str()).or_default().push(i),
                Cmd::Support { target, to } => {
                    let matches = match (to, &cmds[*target]) {
                        (None, Cmd::Move { .. }) => false,
                        (None, _) => true,
                        (Some(p), Cmd::Move { to, .. }) => p == to,
                        (Some(_), _) => false,
                    };
                    if matches {
                        supports[*target].push(i);
                    }
                }
                Cmd::Convoy { army, to } => {
                    if matches!(&cmds[*army], Cmd::Move { to: t, convoy: true, .. } if t == to) {
                        convoyers[*army].push(i);
                    }
                }
                Cmd::Hold => {}
            }
        }
        let h2h = (0..n)
            .map(|i| {
                let Cmd::Move { to, convoy: false, .. } = &cmds[i] else {
                    return None;
                };
                let &o = at.get(to.as_str())?;
                match &cmds[o] {
                    Cmd::Move {
                        to: back,
                        convoy: false,
                        ..
                    } if back == &units[i].location => Some(o),
                    _ => None,
                }
            })
            .collect();
        Resolver {
            units,
            cmds,
            at,
            moves_into,
            supports,
            convoyers,
            h2h,
            state: vec![State::Unresolved; n * KINDS],
            value: vec![false; n * KINDS],
            deps: Vec::new(),
            szykman: vec![false; n],
            circular: vec![false; n],
        }
    }

    /// Resolve every decision so later queries only read results.
    pub fn resolve_all(&mut self) {
        for u in 0..self.units.len() {
            match &self.cmds[u] {
                Cmd::Move { .. } => {
                    self.path(u);
                    self.moves(u);
                }
                Cmd::Support { .. } => {
                    self.given(u);
                }
                _ => {}
            }
        }
        debug_assert!(
            self.state.iter().all(|s| *s != State::Guessing),
            "a decision was left on a guess"
        );
    }

    // ----- public queries --------------------------------------------------------------

    pub fn is_move(&self, u: usize) -> bool {
        matches!(self.cmds[u], Cmd::Move { .. })
    }

    pub fn is_convoyed(&self, u: usize) -> bool {
        matches!(self.cmds[u], Cmd::Move { convoy: true, .. })
    }

    pub fn head_to_head(&self, u: usize) -> Option<usize> {
        self.h2h[u]
    }

    pub fn matched_supports(&self, u: usize) -> &[usize] {
        &self.supports[u]
    }

    pub fn convoyers(&self, u: usize) -> &[usize] {
        &self.convoyers[u]
    }

    pub fn moves_into(&self, province: &str) -> Vec<usize> {
        self.moves_into.get(province).cloned().unwrap_or_default()
    }

    pub fn unit_at(&self, province: &str) -> Option<usize> {
        self.at.get(province).copied()
    }

    /// The move of unit `u` succeeds.
    pub fn moves(&mut self, u: usize) -> bool {
        self.is_move(u) && self.resolve(u * KINDS + MOVE)
    }

    /// The support of unit `s` is given (matched and not cut).
    pub fn given(&mut self, s: usize) -> bool {
        self.resolve(s * KINDS + SUPPORT)
    }

    /// The move of `u` has a route: always for land moves, otherwise an intact convoy chain.
    pub fn path(&mut self, u: usize) -> bool {
        !self.is_convoyed(u) || self.resolve(u * KINDS + PATH)
    }

    /// The unit that dislodges `u`, if any.
    pub fn dislodged_by(&mut self, u: usize) -> Option<usize> {
        if self.moves(u) {
            return None;
        }
        let location = self.units[u].location.as_str();
        self.moves_into(location).into_iter().find(|&m| self.moves(m))
    }

    pub fn hold_strength(&mut self, province: &str) -> (u32, Vec<usize>) {
        let Some(o) = self.unit_at(province) else {
            return (0, Vec::new());
        };
        if self.is_move(o) {
            return (if self.moves(o) { 0 } else { 1 }, Vec::new());
        }
        let given = self.given_supports(o);
        (1 + given.len() as u32, given)
    }

    pub fn attack(&mut self, m: usize) -> Attack {
        let mut attack = Attack {
            strength: 0,
            supports: Vec::new(),
            excluded: Vec::new(),
            own_unit: false,
            no_path: false,
        };
        if !self.path(m) {
            attack.no_path = true;
            return attack;
        }
        let given = self.given_supports(m);
        let occupant = self.target(m).and_then(|p| self.unit_at(p));
        let defender = match occupant {
            Some(o) if self.h2h[m] == Some(o) || !self.moves(o) => Some(o),
            _ => None,
        };
        match defender {
            None => attack.supports = given,
            Some(o) if self.units[o].power == self.units[m].power => {
                attack.own_unit = true;
                attack.supports = given;
                return attack;
            }
            Some(o) => {
                let power = self.units[o].power;
                (attack.excluded, attack.supports) = given.into_iter().partition(|&s| self.units[s].power == power);
            }
        }
        attack.strength = 1 + attack.supports.len() as u32;
        attack
    }

    pub fn defend(&mut self, m: usize) -> u32 {
        1 + self.given_supports(m).len() as u32
    }

    pub fn prevent(&mut self, m: usize) -> u32 {
        if !self.path(m) {
            return 0;
        }
        if let Some(o) = self.h2h[m]
            && self.moves(o)
        {
            return 0; // R-STD-07
        }
        1 + self.given_supports(m).len() as u32
    }

    /// Move decision (§ 3.3). With `explain`, every comparison is evaluated for the trace.
    pub fn eval_move(&mut self, m: usize, explain: bool) -> MoveEval {
        let attack = self.attack(m);
        let opposition = match self.h2h[m] {
            Some(o) => Opposition::Defend {
                unit: o,
                strength: self.defend(o),
            },
            None => Opposition::Hold(self.hold_strength(self.target(m).unwrap_or_default()).0),
        };
        let against = match opposition {
            Opposition::Hold(s) | Opposition::Defend { strength: s, .. } => s,
        };
        let mut success = !attack.own_unit && attack.strength > against;
        let mut prevents = Vec::new();
        if success || explain {
            for o in self.moves_into(self.target(m).unwrap_or_default()) {
                if o == m {
                    continue;
                }
                let p = self.prevent(o);
                prevents.push((o, p));
                if attack.strength <= p {
                    success = false;
                    if !explain {
                        break;
                    }
                }
            }
        }
        MoveEval {
            success,
            attack,
            opposition,
            prevents,
        }
    }

    /// Support-cut decision (§ 3.3, R-CUT-01..04, R-CVY-09/10).
    pub fn eval_cut(&mut self, s: usize, explain: bool) -> CutEval {
        let Cmd::Support { target, to } = &self.cmds[s] else {
            return CutEval {
                cut: None,
                not_cut: Vec::new(),
            };
        };
        let into = to.as_deref().unwrap_or(&self.units[*target].location);
        let here = self.units[s].location.as_str();
        let mut eval = CutEval {
            cut: None,
            not_cut: Vec::new(),
        };
        for m in self.moves_into(here) {
            let rule = if self.units[m].power == self.units[s].power {
                Err("R-CUT-04")
            } else if self.units[m].location == into {
                Err("R-CUT-01")
            } else if self.is_convoyed(m) {
                // A convoyed attack does not cut support for an attack on a fleet its convoy
                // needs (R-CVY-09), unless a route avoiding that fleet survives (R-CVY-10).
                let against_convoy = to.is_some() && self.unit_at(into).is_some_and(|f| self.convoyers[m].contains(&f));
                if against_convoy {
                    if self.eval_path(m, Some(into)) {
                        Ok("R-CVY-10")
                    } else {
                        Err("R-CVY-09")
                    }
                } else if self.path(m) {
                    Ok("R-CUT-01")
                } else {
                    Err("R-CVY-05")
                }
            } else {
                Ok("R-CUT-01")
            };
            match rule {
                Ok(rule) => {
                    if eval.cut.is_none() {
                        eval.cut = Some(CutReason::Attacked { by: m, rule });
                    }
                    if !explain {
                        return eval;
                    }
                }
                Err(rule) => eval.not_cut.push(NotCut { by: m, rule }),
            }
        }
        if eval.cut.is_none()
            && let Some(by) = self.dislodged_by(s)
        {
            eval.cut = Some(CutReason::Dislodged { by });
        }
        eval
    }

    /// Convoy-path decision (R-CVY-03/05/08): a chain of matching, non-dislodged convoying
    /// fleets links origin and destination. `avoid` leaves out the fleet in that province.
    pub fn eval_path(&mut self, m: usize, avoid: Option<&str>) -> bool {
        if self.szykman[m] {
            return false;
        }
        let Some(dest) = self.target(m) else { return false };
        let origin = self.units[m].location.as_str();
        let fleets: Vec<usize> = self.convoyers[m]
            .iter()
            .copied()
            .filter(|&f| Some(self.units[f].location.as_str()) != avoid)
            .collect();
        let sea = |f: usize| self.units[f].location.as_str();
        let mut seen = vec![false; fleets.len()];
        let mut queue = VecDeque::new();
        for (i, &f) in fleets.iter().enumerate() {
            if sea_borders(sea(f), origin) {
                seen[i] = true;
                queue.push_back(i);
            }
        }
        while let Some(i) = queue.pop_front() {
            let f = fleets[i];
            if self.dislodged_by(f).is_some() {
                continue;
            }
            if sea_borders(sea(f), dest) {
                return true;
            }
            for (j, &g) in fleets.iter().enumerate() {
                if !seen[j] && sea_borders(sea(f), sea(g)) {
                    seen[j] = true;
                    queue.push_back(j);
                }
            }
        }
        false
    }

    // ----- internals --------------------------------------------------------------------

    fn target(&self, m: usize) -> Option<&'a str> {
        match &self.cmds[m] {
            Cmd::Move { to, .. } => Some(to.as_str()),
            _ => None,
        }
    }

    fn given_supports(&mut self, u: usize) -> Vec<usize> {
        let candidates = self.supports[u].clone();
        candidates.into_iter().filter(|&s| self.given(s)).collect()
    }

    fn adjudicate(&mut self, d: usize) -> bool {
        let u = d / KINDS;
        match d % KINDS {
            MOVE => self.eval_move(u, false).success,
            SUPPORT => self.eval_cut(u, false).cut.is_none(),
            _ => self.eval_path(u, None),
        }
    }

    /// Guess-and-check resolution with cycle detection (§ 3.4).
    fn resolve(&mut self, d: usize) -> bool {
        match self.state[d] {
            State::Resolved => return self.value[d],
            State::Guessing => {
                // Always record the dependency (even if `d` is listed already): every decision
                // evaluated on top of a guess must see that it depends on one, or it would be
                // marked resolved from a guessed value.
                self.deps.push(d);
                return self.value[d];
            }
            State::Unresolved => {}
        }
        let old = self.deps.len();
        self.value[d] = false;
        self.state[d] = State::Guessing;
        let first = self.adjudicate(d);
        if self.deps.len() == old {
            // No dependency on any guess: the result is final.
            if self.state[d] != State::Resolved {
                self.value[d] = first;
                self.state[d] = State::Resolved;
            }
            return first;
        }
        if self.deps[old] != d {
            // Depends on a cycle that `d` is not part of; the caller up the stack resolves it.
            self.deps.push(d);
            self.value[d] = first;
            return first;
        }
        // `d` is part of a cycle: try the other guess.
        self.reset_deps(old);
        self.value[d] = true;
        self.state[d] = State::Guessing;
        let second = self.adjudicate(d);
        if first == second {
            // Only one consistent outcome.
            self.reset_deps(old);
            self.value[d] = first;
            self.state[d] = State::Resolved;
            return first;
        }
        // Zero or two consistent outcomes: apply the backup rule, then resolve again.
        self.backup_rule(old);
        self.resolve(d)
    }

    fn reset_deps(&mut self, old: usize) {
        for &x in &self.deps[old..] {
            self.state[x] = State::Unresolved;
        }
        self.deps.truncate(old);
    }

    /// Backup rule (§ 3.4): a cycle without convoys is a circular movement and every move in it
    /// succeeds (R-STD-05); a cycle through a convoy is a paradox, and by the Szykman rule the
    /// convoyed armies in it do not move and have no effect (PR-03).
    fn backup_rule(&mut self, old: usize) {
        let mut cycle = self.deps[old..].to_vec();
        cycle.sort_unstable();
        cycle.dedup();
        self.reset_deps(old);
        let paths: Vec<usize> = cycle
            .iter()
            .filter(|&&d| d % KINDS == PATH)
            .map(|d| d / KINDS)
            .collect();
        if !paths.is_empty() {
            for u in paths {
                self.szykman[u] = true;
                self.value[u * KINDS + PATH] = false;
                self.state[u * KINDS + PATH] = State::Resolved;
            }
            return;
        }
        let moves: Vec<usize> = cycle
            .iter()
            .filter(|&&d| d % KINDS == MOVE)
            .map(|d| d / KINDS)
            .collect();
        if moves.is_empty() {
            // Not expected without moves or convoys; settle the cycle as guessed to terminate.
            for d in cycle {
                self.value[d] = false;
                self.state[d] = State::Resolved;
            }
            return;
        }
        for u in moves {
            self.circular[u] = true;
            self.value[u * KINDS + MOVE] = true;
            self.state[u * KINDS + MOVE] = State::Resolved;
        }
    }
}
