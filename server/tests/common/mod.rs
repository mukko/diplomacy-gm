//! Shared helpers for the engine tests: build a board and orders from rulebook notation, run the
//! adjudicator and query the resolution.
#![allow(dead_code)]

use diplomacy_gm_server::adjudicator::Adjudicator;
use diplomacy_gm_server::engine::{Engine, legal_moves};
use diplomacy_gm_server::model::{
    GameState, Order, OrderKind, OrderResult, Outcome, Power, Resolution, Unit, UnitType,
};

pub fn power(code: &str) -> Power {
    code.parse().unwrap()
}

/// `A Ber` / `F StP/SC` / `Ber` → (type, location, coast).
pub fn unit_spec(s: &str) -> (Option<UnitType>, String, Option<String>) {
    let s = s.trim();
    let (ty, loc) = match s.split_once(' ') {
        Some(("A", rest)) => (Some(UnitType::A), rest),
        Some(("F", rest)) => (Some(UnitType::F), rest),
        _ => (None, s),
    };
    match loc.split_once('/') {
        Some((p, c)) => (ty, p.to_string(), Some(c.to_string())),
        None => (ty, loc.to_string(), None),
    }
}

/// Parse one order in rulebook notation. A bare unit (`A Sil`) is placed on the board without
/// an order. Returns the ordering unit (for building a board) and the order.
pub fn parse_order(power: Power, text: &str) -> (Unit, Option<Order>) {
    let text = text.trim();
    let order = |unit: &str, kind| Order {
        power,
        unit: unit.to_string(),
        kind,
    };
    let place = |spec: &str| {
        let (ty, location, coast) = unit_spec(spec);
        Unit {
            power,
            unit_type: ty.unwrap_or(UnitType::A),
            location,
            coast,
        }
    };
    let strip_type = |s: &str| unit_spec(s).1 + &unit_spec(s).2.map(|c| format!("/{c}")).unwrap_or_default();
    if let Some(rest) = text.strip_prefix("B ") {
        let (ty, location, coast) = unit_spec(rest);
        let u = Unit {
            power,
            unit_type: ty.expect("build needs a unit type"),
            location,
            coast,
        };
        let o = order(
            &u.location,
            OrderKind::Build {
                unit_type: u.unit_type,
                coast: u.coast.clone(),
            },
        );
        return (u, Some(o));
    }
    if let Some(province) = text.strip_prefix("waive ") {
        let u = place(province);
        let o = order(&u.location, OrderKind::Waive);
        return (u, Some(o));
    }
    if let Some(me) = text.strip_suffix(" D") {
        let u = place(me);
        let o = order(&u.location, OrderKind::Disband);
        return (u, Some(o));
    }
    if let Some((me, rest)) = text.split_once(" S ") {
        let u = place(me);
        let kind = match rest.split_once('-') {
            Some((t, to)) => OrderKind::Support {
                target: strip_type(t),
                to: Some(to.trim().to_string()),
            },
            None => OrderKind::Support {
                target: strip_type(rest),
                to: None,
            },
        };
        let o = order(&u.location, kind);
        return (u, Some(o));
    }
    if let Some((me, rest)) = text.split_once(" C ") {
        let u = place(me);
        let (t, to) = rest.split_once('-').expect("convoy needs a move");
        let o = order(
            &u.location,
            OrderKind::Convoy {
                target: strip_type(t),
                to: to.trim().to_string(),
            },
        );
        return (u, Some(o));
    }
    if let Some(me) = text.strip_suffix(" H") {
        let u = place(me);
        let o = order(&u.location, OrderKind::Hold);
        return (u, Some(o));
    }
    let (text, via_convoy) = match text.strip_suffix(" via convoy") {
        Some(t) => (t, true),
        None => (text, false),
    };
    if let Some((me, to)) = text.split_once('-') {
        let u = place(me);
        let o = order(
            &u.location,
            OrderKind::Move {
                to: to.trim().to_string(),
                via_convoy,
            },
        );
        return (u, Some(o));
    }
    (place(text), None)
}

/// `GER: A Ber-Sil, F Kie H; RUS: A War-Sil` → board units and orders.
pub fn parse(spec: &str) -> (Vec<Unit>, Vec<Order>) {
    let mut units = Vec::new();
    let mut orders = Vec::new();
    for part in spec.split(';').map(str::trim).filter(|p| !p.is_empty()) {
        let (code, list) = part.split_once(':').expect("`POW: orders`");
        let p = power(code.trim());
        for text in list.split(',').map(str::trim).filter(|t| !t.is_empty()) {
            let (unit, order) = parse_order(p, text);
            units.push(unit);
            orders.extend(order);
        }
    }
    (units, orders)
}

pub fn board(units: Vec<Unit>) -> GameState {
    let mut state = GameState::standard_start();
    state.units = units;
    state
}

/// Adjudicate a position consisting only of the units named in `spec` (Spring 1901 movement).
pub fn run(spec: &str) -> Res {
    let (units, orders) = parse(spec);
    run_on(&board(units), &orders)
}

pub fn run_on(state: &GameState, orders: &[Order]) -> Res {
    Res(Engine.adjudicate(state, orders).expect("movement phase resolves"))
}

pub struct Res(pub Resolution);

impl Res {
    pub fn result(&self, unit: &str) -> &OrderResult {
        self.0
            .orders
            .iter()
            .find(|r| r.order.unit.split('/').next() == Some(unit))
            .unwrap_or_else(|| panic!("no order for {unit}"))
    }

    pub fn outcome(&self, unit: &str) -> Outcome {
        self.result(unit).outcome
    }

    pub fn valid(&self, unit: &str) -> bool {
        self.result(unit).valid
    }

    pub fn rules(&self, unit: &str) -> Vec<String> {
        self.result(unit).rules.clone()
    }

    /// Owner and type of the unit ending in `province`, if any.
    pub fn at(&self, province: &str) -> Option<(Power, UnitType)> {
        self.0
            .next_state
            .units
            .iter()
            .find(|u| u.location == province)
            .map(|u| (u.power, u.unit_type))
    }

    pub fn coast_at(&self, province: &str) -> Option<String> {
        self.0
            .next_state
            .units
            .iter()
            .find(|u| u.location == province)
            .and_then(|u| u.coast.clone())
    }

    /// Dislodgement trace step for the unit that started in `province`.
    pub fn dislodgement(&self, province: &str) -> Option<&serde_json::Map<String, serde_json::Value>> {
        self.0
            .trace
            .iter()
            .filter(|s| s.step == "dislodgements")
            .map(|s| &s.details)
            .find(|d| {
                d["unit"]
                    .as_str()
                    .and_then(|u| u.split(' ').nth(1))
                    .map(|l| l.split('/').next().unwrap())
                    == Some(province)
            })
    }

    pub fn dislodged(&self, province: &str) -> bool {
        self.dislodgement(province).is_some()
    }

    pub fn retreat_options(&self, province: &str) -> Vec<String> {
        self.dislodgement(province).expect("dislodged")["retreatOptions"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect()
    }

    pub fn standoffs(&self) -> Vec<String> {
        let result = self.0.trace.iter().find(|s| s.step == "result").expect("result step");
        result.details["standoffProvinces"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().into())
            .collect()
    }

    pub fn nothing_moved(&self, state_units: &[&str]) {
        for p in state_units {
            assert!(self.at(p).is_some(), "unit in {p} should still be there");
        }
    }
}

#[allow(unused_imports)]
pub use Outcome::{Cut, Fail, Success, Void};

pub fn moves_of(unit: Unit) -> Vec<String> {
    let mut m = legal_moves(&unit);
    m.sort();
    m
}

pub fn unit(power: Power, unit_type: UnitType, location: &str, coast: Option<&str>) -> Unit {
    Unit {
        power,
        unit_type,
        location: location.into(),
        coast: coast.map(Into::into),
    }
}
