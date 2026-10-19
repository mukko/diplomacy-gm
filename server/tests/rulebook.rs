//! Rulebook examples (docs/en/test-cases.md §§ 1–7) as engine tests, one per case ID, plus the
//! sample game of § 9 (Spring 1901 – Fall 1902 movement).

use diplomacy_gm_server::adjudicator::Adjudicator;
use diplomacy_gm_server::engine::{Engine, legal_moves};
use diplomacy_gm_server::model::{GameState, Order, OrderKind, Outcome, Phase, Power, Resolution, Unit, UnitType};

// ----- helpers ---------------------------------------------------------------------------------

fn power(code: &str) -> Power {
    code.parse().unwrap()
}

/// `A Ber` / `F StP/SC` / `Ber` → (type, location, coast).
fn unit_spec(s: &str) -> (Option<UnitType>, String, Option<String>) {
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
fn parse_order(power: Power, text: &str) -> (Unit, Option<Order>) {
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
fn parse(spec: &str) -> (Vec<Unit>, Vec<Order>) {
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

fn board(units: Vec<Unit>) -> GameState {
    let mut state = GameState::standard_start();
    state.units = units;
    state
}

/// Adjudicate a position consisting only of the units named in `spec` (Spring 1901 movement).
fn run(spec: &str) -> Res {
    let (units, orders) = parse(spec);
    run_on(&board(units), &orders)
}

fn run_on(state: &GameState, orders: &[Order]) -> Res {
    Res(Engine.adjudicate(state, orders).expect("movement phase resolves"))
}

struct Res(Resolution);

impl Res {
    fn result(&self, unit: &str) -> &diplomacy_gm_server::model::OrderResult {
        self.0
            .orders
            .iter()
            .find(|r| r.order.unit.split('/').next() == Some(unit))
            .unwrap_or_else(|| panic!("no order for {unit}"))
    }

    fn outcome(&self, unit: &str) -> Outcome {
        self.result(unit).outcome
    }

    fn valid(&self, unit: &str) -> bool {
        self.result(unit).valid
    }

    fn rules(&self, unit: &str) -> Vec<String> {
        self.result(unit).rules.clone()
    }

    /// Owner and type of the unit ending in `province`, if any.
    fn at(&self, province: &str) -> Option<(Power, UnitType)> {
        self.0
            .next_state
            .units
            .iter()
            .find(|u| u.location == province)
            .map(|u| (u.power, u.unit_type))
    }

    fn coast_at(&self, province: &str) -> Option<String> {
        self.0
            .next_state
            .units
            .iter()
            .find(|u| u.location == province)
            .and_then(|u| u.coast.clone())
    }

    /// Dislodgement trace step for the unit that started in `province`.
    fn dislodgement(&self, province: &str) -> Option<&serde_json::Map<String, serde_json::Value>> {
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

    fn dislodged(&self, province: &str) -> bool {
        self.dislodgement(province).is_some()
    }

    fn retreat_options(&self, province: &str) -> Vec<String> {
        self.dislodgement(province).expect("dislodged")["retreatOptions"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect()
    }

    fn standoffs(&self) -> Vec<String> {
        let result = self.0.trace.iter().find(|s| s.step == "result").expect("result step");
        result.details["standoffProvinces"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().into())
            .collect()
    }

    fn nothing_moved(&self, state_units: &[&str]) {
        for p in state_units {
            assert!(self.at(p).is_some(), "unit in {p} should still be there");
        }
    }
}

use Outcome::{Cut, Fail, Success, Void};

fn moves_of(unit: Unit) -> Vec<String> {
    let mut m = legal_moves(&unit);
    m.sort();
    m
}

fn unit(power: Power, unit_type: UnitType, location: &str, coast: Option<&str>) -> Unit {
    Unit {
        power,
        unit_type,
        location: location.into(),
        coast: coast.map(Into::into),
    }
}

// ----- § 1. Movement / map -----------------------------------------------------------------------

#[test]
fn rb_d01() {
    assert_eq!(
        moves_of(unit(Power::France, UnitType::A, "Par", None)),
        ["Bre", "Bur", "Gas", "Pic"]
    );
}

#[test]
fn rb_d02() {
    assert_eq!(
        moves_of(unit(Power::England, UnitType::F, "Eng", None)),
        ["Bel", "Bre", "Iri", "Lon", "Mid", "Nth", "Pic", "Wal"]
    );
}

#[test]
fn rb_d03() {
    assert_eq!(
        moves_of(unit(Power::Italy, UnitType::F, "Rom", None)),
        ["Nap", "Tus", "Tyn"]
    );
    for bad in ["Ven", "Apu"] {
        let r = run(&format!("ITA: F Rom-{bad}"));
        assert!(!r.valid("Rom"), "F Rom-{bad} must be illegal (R-MAP-05)");
        assert!(r.at("Rom").is_some());
    }
}

#[test]
fn rb_spa() {
    for to in ["Wes", "GoL", "Mar"] {
        let r = run(&format!("FRA: F Spa/NC-{to}"));
        assert!(!r.valid("Spa"), "F Spa/NC-{to} must be illegal (R-MAP-06)");
        assert_eq!(r.coast_at("Spa").as_deref(), Some("NC"));
    }
}

#[test]
fn rb_bul() {
    let r = run("TUR: F Con-Bul");
    assert!(!r.valid("Con"));
    assert!(r.rules("Con").contains(&"R-MAP-07".to_string()));
    assert_eq!(r.at("Con"), Some((Power::Turkey, UnitType::F)));
    for coast in ["EC", "SC"] {
        let r = run(&format!("TUR: F Con-Bul/{coast}"));
        assert!(r.valid("Con"));
        assert_eq!(r.outcome("Con"), Success);
        assert_eq!(r.coast_at("Bul").as_deref(), Some(coast));
    }
}

// ----- § 2. Standoffs ----------------------------------------------------------------------------

#[test]
fn rb_d04() {
    let r = run("GER: A Ber-Sil; RUS: A War-Sil");
    assert_eq!((r.outcome("Ber"), r.outcome("War")), (Fail, Fail));
    assert_eq!(r.at("Sil"), None);
    assert_eq!(r.standoffs(), ["Sil"]);
}

#[test]
fn rb_d04b() {
    let r = run("GER: A Ber-Sil; RUS: A War-Sil; AUS: A Sil H");
    assert_eq!((r.outcome("Ber"), r.outcome("War")), (Fail, Fail));
    assert_eq!(r.at("Sil"), Some((Power::Austria, UnitType::A)));
    assert!(!r.dislodged("Sil"));
}

#[test]
fn rb_d05() {
    let r = run("GER: F Kie-Ber, A Ber-Pru; RUS: A Pru H");
    assert_eq!((r.outcome("Kie"), r.outcome("Ber")), (Fail, Fail));
    r.nothing_moved(&["Kie", "Ber", "Pru"]);
}

#[test]
fn rb_d06() {
    let r = run("GER: F Ber-Pru, A Pru-Ber");
    assert_eq!((r.outcome("Ber"), r.outcome("Pru")), (Fail, Fail));
    assert_eq!(r.at("Ber"), Some((Power::Germany, UnitType::F)));
    assert_eq!(r.at("Pru"), Some((Power::Germany, UnitType::A)));
}

#[test]
fn rb_d07() {
    let r = run("ENG: A Hol-Bel, F Bel-Nth; FRA: F Nth-Hol");
    for u in ["Hol", "Bel", "Nth"] {
        assert_eq!(r.outcome(u), Success, "{u}");
    }
    assert_eq!(r.at("Bel"), Some((Power::England, UnitType::A)));
    assert_eq!(r.at("Nth"), Some((Power::England, UnitType::F)));
    assert_eq!(r.at("Hol"), Some((Power::France, UnitType::F)));
    assert!(r.rules("Hol").contains(&"R-STD-05".to_string()));
}

// ----- § 3. Support ------------------------------------------------------------------------------

#[test]
fn rb_d08() {
    let r = run("FRA: A Mar-Bur, A Gas S A Mar-Bur; GER: A Bur H");
    assert_eq!(r.outcome("Mar"), Success);
    assert!(r.dislodged("Bur"));
    assert_eq!(r.at("Bur"), Some((Power::France, UnitType::A)));
}

#[test]
fn rb_d09() {
    let r = run("GER: A Sil-Pru, F Bal S A Sil-Pru; RUS: A Pru H");
    assert!(r.valid("Bal"), "a fleet in Bal can support into coastal Pru (R-SUP-02)");
    assert_eq!(r.outcome("Sil"), Success);
    assert!(r.dislodged("Pru"));
}

#[test]
fn rb_d10() {
    let r = run("FRA: F GoL-Tyn, F Wes S F GoL-Tyn; ITA: F Nap-Tyn, F Rom S F Nap-Tyn");
    assert_eq!((r.outcome("GoL"), r.outcome("Nap")), (Fail, Fail));
    assert_eq!(r.at("Tyn"), None);
    assert_eq!(r.standoffs(), ["Tyn"]);
}

#[test]
fn rb_d11() {
    let r = run("FRA: F GoL-Tyn, F Wes S F GoL-Tyn; ITA: F Tyn H, F Rom S F Tyn");
    assert_eq!(r.outcome("GoL"), Fail);
    assert_eq!(r.outcome("Rom"), Success);
    assert!(!r.dislodged("Tyn"));
}

#[test]
fn rb_supmismatch() {
    let r = run("GER: A Boh S A Mun-Sil, A Mun-Tyr; AUS: A Ber-Mun");
    assert_eq!(r.outcome("Boh"), Void);
    assert!(r.rules("Boh").contains(&"R-SUP-06".to_string()));
    assert_eq!(r.outcome("Mun"), Success);
    // The void support does not turn into a hold support for anyone.
    let strengths =
        r.0.trace
            .iter()
            .find(|s| s.step == "strengths" && s.details["province"] == "Mun")
            .unwrap();
    assert_eq!(strengths.details["holdSupports"], serde_json::json!([]));
}

#[test]
fn rb_d12() {
    let r =
        run("AUS: A Boh-Mun, A Tyr S A Boh-Mun; GER: A Mun-Sil, A Ber S A Mun-Sil; RUS: A War-Sil, A Pru S A War-Sil");
    assert_eq!(r.outcome("Boh"), Success);
    assert!(r.dislodged("Mun"));
    assert_eq!((r.outcome("Mun"), r.outcome("War")), (Fail, Fail));
    assert_eq!(r.at("Sil"), None);
    assert!(r.rules("War").contains(&"R-STD-06".to_string()));
    // RB-RET1: A Mun cannot retreat to Boh (attacker origin) or Sil (standoff).
    let options = r.retreat_options("Mun");
    assert!(
        !options.contains(&"Boh".to_string()) && !options.contains(&"Sil".to_string()),
        "{options:?}"
    );
    assert_eq!(options, ["Bur", "Kie", "Ruh"]);
}

#[test]
fn rb_d13() {
    let r = run("TUR: A Bul-Rum; RUS: A Rum-Bul, A Ser S A Rum-Bul, A Sev-Rum");
    assert_eq!(r.outcome("Rum"), Success);
    assert!(r.dislodged("Bul"));
    assert_eq!(r.outcome("Sev"), Success);
    assert_eq!(r.at("Rum"), Some((Power::Russia, UnitType::A)));
}

#[test]
fn rb_d14() {
    let r = run("TUR: A Bul-Rum, F Bla S A Bul-Rum; RUS: A Rum-Bul, A Gre S A Rum-Bul, A Ser S A Rum-Bul, A Sev-Rum");
    assert_eq!(r.outcome("Rum"), Success);
    assert!(r.dislodged("Bul"));
    assert_eq!(r.outcome("Sev"), Success);
    assert!(
        r.result("Rum")
            .reason
            .as_deref()
            .unwrap()
            .contains("attack 3 > defend 2")
    );
}

#[test]
fn rb_d13r() {
    let r = run("TUR: A Bul-Rum; RUS: A Rum-Bul, A Ser S A Rum-Bul");
    assert_eq!(r.at("Rum"), None);
    assert!(
        r.standoffs().is_empty(),
        "Rum is empty but not a standoff province (R-RET-04)"
    );
    let options = r.retreat_options("Bul");
    assert!(
        !options.contains(&"Rum".to_string()),
        "Rum is the attacker's origin (R-RET-03)"
    );
    assert_eq!(options, ["Con", "Gre"]);
}

// ----- § 4. Cutting support ----------------------------------------------------------------------

#[test]
fn rb_d15() {
    let r = run("GER: A Pru-War, A Sil S A Pru-War; RUS: A War H, A Boh-Sil");
    assert_eq!(r.outcome("Sil"), Cut);
    assert_eq!((r.outcome("Pru"), r.outcome("Boh")), (Fail, Fail));
    assert!(!r.dislodged("War"));
}

#[test]
fn rb_d16() {
    let r = run("GER: A Pru-War, A Sil S A Pru-War; RUS: A War-Sil");
    assert_eq!(
        r.outcome("Sil"),
        Success,
        "attack from the supported-into province does not cut (R-CUT-01)"
    );
    assert_eq!(r.outcome("War"), Fail);
    assert_eq!(r.outcome("Pru"), Success);
    assert!(r.dislodged("War"));
}

#[test]
fn rb_d17() {
    let r = run("GER: F Ber-Pru, A Sil S F Ber-Pru; RUS: A Pru-Sil, A War S A Pru-Sil, F Bal-Pru");
    assert_eq!(r.outcome("Pru"), Success);
    assert!(r.dislodged("Sil"));
    assert_eq!(r.outcome("Sil"), Cut);
    assert!(r.rules("Sil").contains(&"R-CUT-02".to_string()));
    assert_eq!((r.outcome("Ber"), r.outcome("Bal")), (Fail, Fail));
    assert_eq!(r.at("Pru"), None);
}

#[test]
fn rb_d18() {
    let r = run("GER: A Ber H, A Mun-Sil; RUS: A Pru-Ber, A Sil S A Pru-Ber, A Boh-Mun, A Tyr S A Boh-Mun");
    assert_eq!(r.outcome("Mun"), Fail);
    assert_eq!(r.outcome("Sil"), Cut);
    assert_eq!(r.outcome("Pru"), Fail);
    assert_eq!(r.outcome("Boh"), Success);
    assert!(r.dislodged("Mun"));
}

// ----- § 5. Convoy -------------------------------------------------------------------------------

#[test]
fn rb_d19() {
    let r = run("ENG: A Lon-Nwy, F Nth C A Lon-Nwy");
    assert_eq!((r.outcome("Lon"), r.outcome("Nth")), (Success, Success));
    assert_eq!(r.at("Nwy"), Some((Power::England, UnitType::A)));
}

#[test]
fn rb_cvymismatch() {
    // Rum and Arm are not adjacent, so the army needs a convoy; the only fleet convoys to Ank.
    let r = run("RUS: A Rum-Arm; TUR: F Bla C A Rum-Ank");
    assert_eq!(r.outcome("Bla"), Void);
    assert_eq!(r.outcome("Rum"), Fail);
    assert_eq!(r.at("Rum"), Some((Power::Russia, UnitType::A)));
    // With a land route the unmatched convoy is irrelevant.
    let r = run("RUS: A Rum-Sev; TUR: F Bla C A Rum-Ank");
    assert_eq!(r.outcome("Bla"), Void);
    assert_eq!(r.outcome("Rum"), Success);
}

#[test]
fn rb_supcvy() {
    // "F Eng C A Lon S A Pic-Bre" cannot be expressed: a convoy order names a move, so the
    // closest order is F Eng C A Lon-Bre, which matches nothing (R-CVY-04).
    let r = run("ENG: A Pic-Bre, A Lon S A Pic-Bre, F Eng C A Lon-Bre; FRA: F Bre H");
    assert!(!r.valid("Lon"), "support cannot be convoyed (R-SUP-07)");
    assert_eq!(r.outcome("Eng"), Void);
    assert_eq!(r.outcome("Pic"), Fail);
    assert!(!r.dislodged("Bre"));
}

#[test]
fn rb_d20() {
    let r = run("ENG: A Lon-Tun, F Eng C A Lon-Tun, F Mid C A Lon-Tun; FRA: F Wes C A Lon-Tun");
    assert_eq!(r.outcome("Lon"), Success);
    assert_eq!(r.at("Tun"), Some((Power::England, UnitType::A)));
}

#[test]
fn rb_d21() {
    let r = run("FRA: A Spa-Nap, F GoL C A Spa-Nap, F Tyn C A Spa-Nap; ITA: F Ion-Tyn, F Tun S F Ion-Tyn");
    assert!(r.dislodged("Tyn"));
    assert_eq!(r.outcome("Spa"), Fail);
    assert_eq!(r.at("Spa"), Some((Power::France, UnitType::A)));
    assert!(r.rules("Spa").contains(&"R-CVY-05".to_string()));
}

// ----- § 6. Self-dislodgement and self-standoff --------------------------------------------------

#[test]
fn rb_d22() {
    let r = run("FRA: A Par-Bur, A Mar S A Par-Bur, A Bur H");
    assert_eq!(r.outcome("Par"), Fail);
    assert!(r.rules("Par").contains(&"R-SLF-01".to_string()));
    assert!(!r.dislodged("Bur"));
    r.nothing_moved(&["Par", "Mar", "Bur"]);
}

#[test]
fn rb_d23() {
    let r = run("FRA: A Par-Bur, A Bur-Mar; GER: A Ruh S A Par-Bur; ITA: A Mar-Bur");
    for u in ["Par", "Bur", "Mar"] {
        assert_eq!(r.outcome(u), Fail, "{u}");
    }
    assert!(!r.dislodged("Bur"));
    assert_eq!(r.at("Bur"), Some((Power::France, UnitType::A)));
    assert_eq!(r.at("Mar"), Some((Power::Italy, UnitType::A)));
}

#[test]
fn rb_d24() {
    let r = run("GER: A Ruh-Bur, A Mun H; FRA: A Par S A Ruh-Bur, A Bur H");
    assert_eq!(r.outcome("Ruh"), Fail);
    assert!(r.rules("Ruh").contains(&"R-SLF-02".to_string()));
    assert!(!r.dislodged("Bur"));
}

#[test]
fn rb_d24b() {
    let r = run("GER: A Ruh-Bur, A Mun S A Ruh-Bur; FRA: A Par S A Ruh-Bur, A Bur H");
    assert_eq!(r.outcome("Ruh"), Success);
    assert!(r.dislodged("Bur"));
}

#[test]
fn rb_d25() {
    let r = run("GER: A Mun-Tyr, A Ruh-Mun, A Sil-Mun; AUS: A Tyr-Mun, A Boh S A Sil-Mun");
    for u in ["Mun", "Ruh", "Sil", "Tyr"] {
        assert_eq!(r.outcome(u), Fail, "{u}");
    }
    r.nothing_moved(&["Mun", "Ruh", "Sil", "Tyr", "Boh"]);
}

#[test]
fn rb_d26() {
    let r = run("ENG: F Den-Kie, F Nth-Den, F Hel S F Nth-Den; RUS: A Ber-Kie, F Ska-Den, F Bal S F Ska-Den");
    for u in ["Den", "Nth", "Ber", "Ska"] {
        assert_eq!(r.outcome(u), Fail, "{u}");
    }
    r.nothing_moved(&["Den", "Nth", "Hel", "Ber", "Ska", "Bal"]);
    assert!(!r.dislodged("Den"));
}

#[test]
fn rb_d27() {
    let r = run("AUS: A Ser-Bud, A Vie-Bud; RUS: A Gal S A Ser-Bud");
    assert_eq!(r.outcome("Ser"), Success);
    assert_eq!(r.outcome("Vie"), Fail);
    assert_eq!(r.at("Bud"), Some((Power::Austria, UnitType::A)));
    assert_eq!(r.at("Ser"), None);
}

#[test]
fn rb_d27b() {
    let r = run("AUS: A Ser-Bud, A Vie-Bud, A Bud H; RUS: A Gal S A Ser-Bud");
    assert_eq!(r.outcome("Ser"), Fail);
    assert!(r.rules("Ser").contains(&"R-SLF-01".to_string()));
    assert!(!r.dislodged("Bud"));
}

#[test]
fn rb_cutown() {
    let r = run("GER: A Pru-War, A Sil S A Pru-War, A Boh-Sil; RUS: A War H");
    assert_eq!(
        r.outcome("Sil"),
        Success,
        "an own attack does not cut support (R-CUT-04)"
    );
    assert_eq!(r.outcome("Pru"), Success);
    assert!(r.dislodged("War"));
    assert_eq!(r.outcome("Boh"), Fail);
}

// ----- § 7. Advanced convoy cases ----------------------------------------------------------------

#[test]
fn rb_d28() {
    let r = run("ENG: A Lon-Bel, F Nth C A Lon-Bel; FRA: A Bel-Lon, F Eng C A Bel-Lon");
    assert_eq!((r.outcome("Lon"), r.outcome("Bel")), (Success, Success));
    assert_eq!(r.at("Bel"), Some((Power::England, UnitType::A)));
    assert_eq!(r.at("Lon"), Some((Power::France, UnitType::A)));
}

#[test]
fn rb_d29() {
    let r = run("ENG: A Lon-Bel, F Eng C A Lon-Bel, F Nth C A Lon-Bel; FRA: F Bre-Eng, F Iri S F Bre-Eng");
    assert!(r.dislodged("Eng"));
    assert_eq!(r.outcome("Lon"), Success);
    assert_eq!(r.at("Bel"), Some((Power::England, UnitType::A)));
}

#[test]
fn rb_d30() {
    let r = run("FRA: A Tun-Nap, F Tyn C A Tun-Nap; ITA: F Ion-Tyn, F Nap S F Ion-Tyn");
    assert!(r.dislodged("Tyn"));
    assert_eq!(r.outcome("Tun"), Fail);
    assert_eq!(r.outcome("Nap"), Success, "support not cut (R-CVY-09)");
    assert!(
        r.0.trace
            .iter()
            .any(|s| s.step == "support-cuts" && s.rules.contains(&"R-CVY-09".to_string()))
    );
}

#[test]
fn rb_d31() {
    let r = run("FRA: A Tun-Nap, F Tyn C A Tun-Nap, F Ion C A Tun-Nap; ITA: F Rom-Tyn, F Nap S F Rom-Tyn");
    assert_eq!(r.outcome("Nap"), Cut);
    assert!(r.rules("Nap").contains(&"R-CVY-10".to_string()));
    assert_eq!((r.outcome("Rom"), r.outcome("Tun")), (Fail, Fail));
    assert!(!r.dislodged("Tyn") && !r.dislodged("Nap"));
}

#[test]
fn rb_d32() {
    let r = run(
        "FRA: A Tun-Nap, F Tyn C A Tun-Nap, F Ion C A Tun-Nap, A Apu S A Tun-Nap; ITA: F Rom-Tyn, F Nap S F Rom-Tyn",
    );
    assert!(r.dislodged("Nap"));
    assert_eq!(r.outcome("Tun"), Success);
    assert_eq!(r.at("Nap"), Some((Power::France, UnitType::A)));
    assert_eq!(r.outcome("Rom"), Fail);
}

#[test]
fn rb_d32b() {
    let r = run(
        "FRA: A Tun-Nap, F Tyn C A Tun-Nap, F Ion C A Tun-Nap, A Apu S A Tun-Nap; ITA: F Nap-Tyn, F Rom S F Nap-Tyn",
    );
    assert!(r.dislodged("Tyn"));
    assert_eq!(r.outcome("Nap"), Success);
    assert_eq!(r.at("Tyn"), Some((Power::Italy, UnitType::F)));
    // The route via Ion survives (R-CVY-08) and Nap has been vacated.
    assert_eq!(r.at("Nap"), Some((Power::France, UnitType::A)));
}

#[test]
fn rb_landcvy1() {
    // Bre-Pic is a land move, but France's own fleet convoys it: the convoy is used, so this is
    // not a head-to-head battle with A Pic-Bre and both units swap (R-CVY-07, R-CVY-11).
    let r = run("FRA: A Bre-Pic, F Eng C A Bre-Pic; ENG: A Pic-Bre");
    assert_eq!((r.outcome("Bre"), r.outcome("Pic")), (Success, Success));
    assert_eq!(r.at("Pic"), Some((Power::France, UnitType::A)));
    assert_eq!(r.at("Bre"), Some((Power::England, UnitType::A)));
}

#[test]
fn rb_landcvy2() {
    // The only convoying fleet is foreign and the order does not say "via convoy": the land
    // route is used, making it a head-to-head battle that bounces.
    let r = run("FRA: A Bre-Pic; ENG: F Eng C A Bre-Pic, A Pic-Bre");
    assert_eq!((r.outcome("Bre"), r.outcome("Pic")), (Fail, Fail));
    assert_eq!(r.outcome("Eng"), Void);
    assert!(r.rules("Eng").contains(&"R-CVY-11".to_string()));
    // "via convoy" forces the convoy route (PR-07).
    let r = run("FRA: A Bre-Pic via convoy; ENG: F Eng C A Bre-Pic, A Pic-Bre");
    assert_eq!((r.outcome("Bre"), r.outcome("Pic")), (Success, Success));
}

// ----- § 9. Sample game (Spring 1901 – Fall 1902 movement) --------------------------------------

fn sample_orders(spec: &str) -> Vec<Order> {
    parse(spec).1
}

fn add_units(state: &mut GameState, spec: &str) {
    state.units.extend(parse(spec).0);
}

#[test]
fn sample_game() {
    // Spring 1901
    let s1901 = GameState::standard_start();
    let r = run_on(
        &s1901,
        &sample_orders(
            "AUS: A Vie-Tri, A Bud-Gal, F Tri-Alb;
             ENG: A Lvp-Yor, F Lon-Nth, F Edi-Nrg;
             FRA: A Par-Bur, A Mar-Spa, F Bre-Pic;
             GER: A Ber-Kie, A Mun-Ruh, F Kie-Den;
             ITA: A Ven-Pie, A Rom-Ven, F Nap-Ion;
             RUS: A Mos-Ukr, A War-Gal, F StP-Bot, F Sev-Bla;
             TUR: A Con-Bul, A Smy-Con, F Ank-Bla",
        ),
    );
    for u in ["Sev", "Ank", "Bud", "War"] {
        assert_eq!(r.outcome(u), Fail, "S1901 {u}");
    }
    assert!(
        r.0.orders
            .iter()
            .filter(|o| !["Sev", "Ank", "Bud", "War"].contains(&o.order.unit.as_str()))
            .all(|o| o.outcome == Success)
    );
    assert_eq!(r.0.next_state.phase, "F1901M".parse::<Phase>().unwrap());

    // Fall 1901
    let r = run_on(
        &r.0.next_state,
        &sample_orders(
            "AUS: A Tri H, A Bud-Ser, F Alb-Gre;
             ENG: A Yor-Nwy, F Nth C A Yor-Nwy, F Nrg-Bar;
             FRA: A Bur-Mar, A Spa-Por, F Pic-Bel;
             GER: A Kie-Hol, A Ruh-Bel, F Den H;
             ITA: A Ven H, A Pie-Mar, F Ion-Tun;
             RUS: A Ukr S F Sev-Rum, A War-Gal, F Bot-Swe, F Sev-Rum;
             TUR: A Bul-Ser, A Con-Bul, F Ank-Bla",
        ),
    );
    for u in ["Pic", "Ruh", "Bur", "Pie", "Bud", "Bul", "Con"] {
        assert_eq!(r.outcome(u), Fail, "F1901 {u}");
    }
    for u in ["Alb", "Yor", "Nrg", "Spa", "Kie", "Ion", "War", "Bot", "Sev", "Ank"] {
        assert_eq!(r.outcome(u), Success, "F1901 {u}");
    }
    let next = &r.0.next_state;
    assert_eq!(next.phase, "W1901A".parse::<Phase>().unwrap());
    let scs = |p: Power| next.supply_centers.values().filter(|o| **o == Some(p)).count();
    let expected = [
        (Power::Austria, 4),
        (Power::England, 4),
        (Power::France, 4),
        (Power::Germany, 5),
        (Power::Italy, 4),
        (Power::Russia, 6),
        (Power::Turkey, 4),
    ];
    for (p, n) in expected {
        assert_eq!(scs(p), n, "SCs of {p:?} after Fall 1901");
    }

    // Winter 1901 builds (adjustments are not implemented yet: apply them by hand).
    let mut s1902 = next.clone();
    s1902.phase = "S1902M".parse().unwrap();
    add_units(
        &mut s1902,
        "ENG: F Edi; GER: F Kie, A Mun; RUS: A StP, A Sev; TUR: A Smy; AUS: A Vie; ITA: F Nap; FRA: F Mar",
    );
    let r = run_on(
        &s1902,
        &sample_orders(
            "AUS: A Tri-Bud, A Vie-Bud, A Bud-Ser, F Gre H;
             ENG: A Nwy-StP, F Nth-Nwy, F Bar S A Nwy-StP, F Edi-Nth;
             FRA: A Bur S F Pic-Bel, A Por-Spa, F Pic-Bel, F Mar H;
             GER: A Hol-Bel, A Ruh S A Hol-Bel, A Mun-Bur, F Den H, F Kie-Hol;
             ITA: A Ven H, A Pie-Mar, F Tun-Wes, F Nap-Tyn;
             RUS: A Ukr S F Rum, A Gal-Bud, A StP-Nwy, A Sev S F Rum, F Swe S A StP-Nwy, F Rum H;
             TUR: A Bul-Rum, A Con-Bul, A Smy-Arm, F Bla S A Bul-Rum",
        ),
    );
    for u in [
        "Nwy", "StP", "Nth", "Edi", "Tri", "Vie", "Gal", "Bul", "Con", "Mun", "Pic", "Pie",
    ] {
        assert_eq!(r.outcome(u), Fail, "S1902 {u}");
    }
    for u in ["Bud", "Smy", "Hol", "Kie", "Tun", "Nap", "Por"] {
        assert_eq!(r.outcome(u), Success, "S1902 {u}");
    }
    assert_eq!(r.outcome("Bur"), Cut);
    assert!(r.0.next_state.dislodged.is_empty() && !r.0.trace.iter().any(|s| s.step == "dislodgements"));
    assert_eq!(r.0.next_state.phase, "F1902M".parse::<Phase>().unwrap());

    // Fall 1902
    let r = run_on(
        &r.0.next_state,
        &sample_orders(
            "AUS: A Vie-Gal, A Tri-Bud, A Ser S A Bul-Rum, F Gre H;
             ENG: A Nwy-StP, F Bar S A Nwy-StP, F Nth-Nwy, F Edi-Nth;
             FRA: A Bur-Bel, F Pic S A Bur-Bel, A Spa S F Mar, F Mar S A Spa;
             GER: A Ruh-Bur, A Mun S A Ruh-Bur, A Bel S A Ruh-Bur, F Den-Swe, F Hol S A Bel;
             ITA: A Ven-Pie, A Pie-Mar, F Wes-NAf, F Tyn-GoL;
             RUS: A StP-Nwy, F Swe S StP-Nwy, F Rum S A Sev, A Sev S F Rum, A Gal S F Rum, A Ukr S A Sev;
             TUR: A Bul-Rum, A Con-Bul, A Arm-Sev, F Bla S A Bul-Rum",
        ),
    );
    for u in ["Swe", "Mar", "Sev", "Gal", "Rum"] {
        assert_eq!(r.outcome(u), Cut, "F1902 support of {u} is cut");
    }
    assert_eq!(
        r.outcome("Bel"),
        Success,
        "A Bel is attacked from Bur, its support target"
    );
    for u in ["Vie", "Ven", "Pie", "Bur", "Den", "Arm", "StP"] {
        assert_eq!(r.outcome(u), Fail, "F1902 {u}");
    }
    for u in ["Ruh", "Bul", "Con", "Nwy", "Nth", "Edi", "Wes", "Tyn"] {
        assert_eq!(r.outcome(u), Success, "F1902 {u}");
    }
    assert!(r.dislodged("Rum") && r.dislodged("StP") && r.dislodged("Bur"));
    assert!(
        r.retreat_options("Rum").is_empty(),
        "F Rum has no retreat and is disbanded"
    );
    let next = &r.0.next_state;
    assert_eq!(next.phase, "F1902R".parse::<Phase>().unwrap());
    let retreating: Vec<&str> = next.dislodged.iter().map(|d| d.location.as_str()).collect();
    assert_eq!(retreating, ["Bur", "StP"]);
    assert!(
        next.dislodged
            .iter()
            .find(|d| d.location == "StP")
            .unwrap()
            .retreat_options
            .contains(&"Mos".into())
    );
    assert!(
        next.dislodged
            .iter()
            .find(|d| d.location == "Bur")
            .unwrap()
            .retreat_options
            .contains(&"Gas".into())
    );
}

// ----- other phases ------------------------------------------------------------------------------

#[test]
fn retreat_and_adjustment_phases_are_not_implemented_yet() {
    let mut state = GameState::standard_start();
    for phase in ["S1901R", "W1901A"] {
        state.phase = phase.parse().unwrap();
        assert!(Engine.adjudicate(&state, &[]).is_err(), "{phase}");
    }
}

// ----- paradoxes beyond the rulebook (DATC 6.F) ----------------------------------------------------

#[test]
fn pr03_pandins_paradox() {
    // DATC 6.F.16: whatever the convoy does, no move succeeds and F Eng stays.
    let r =
        run("ENG: F Lon S F Wal-Eng, F Wal-Eng; FRA: A Bre-Lon, F Eng C A Bre-Lon; GER: F Nth S F Bel-Eng, F Bel-Eng");
    for u in ["Wal", "Bre", "Bel"] {
        assert_eq!(r.outcome(u), Fail, "{u}");
    }
    assert!(!r.dislodged("Eng"));
}

#[test]
fn pr03_second_order_paradox_with_two_resolutions() {
    // DATC 6.F.24: two consistent outcomes; by the Szykman rule both convoyed armies stay and
    // have no effect, so both fleet attacks succeed.
    let r = run("ENG: F Edi-Nth, F Lon S F Edi-Nth; FRA: A Bre-Lon, F Eng C A Bre-Lon; \
         GER: F Bel S F Pic-Eng, F Pic-Eng; RUS: A Nwy-Bel, F Nth C A Nwy-Bel");
    assert_eq!((r.outcome("Edi"), r.outcome("Pic")), (Success, Success));
    assert_eq!((r.outcome("Bre"), r.outcome("Nwy")), (Fail, Fail));
    assert!(r.dislodged("Nth") && r.dislodged("Eng"));
    assert!(r.rules("Bre").contains(&"PR-03".to_string()) || r.rules("Nwy").contains(&"PR-03".to_string()));
}
