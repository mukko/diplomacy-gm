//! Text order entry (adjudication.md § 2.1).

use diplomacy_gm_server::engine::parse::parse_orders;
use diplomacy_gm_server::model::{DislodgedUnit, GameState, Order, OrderKind, Phase, Power, Unit, UnitType};
use serde_json::{Value, json};

fn start() -> GameState {
    GameState::standard_start()
}

/// Parse and return each item as JSON (`order` or `error`) for compact assertions.
fn parsed(state: &GameState, text: &str) -> Vec<Value> {
    parse_orders(state, text, None)
        .into_iter()
        .map(|p| match (p.order, p.error) {
            (Some(o), _) => serde_json::to_value(o).unwrap(),
            (None, Some(e)) => json!({ "error": e }),
            _ => unreachable!(),
        })
        .collect()
}

fn orders(state: &GameState, text: &str) -> Vec<Order> {
    parse_orders(state, text, None)
        .into_iter()
        .map(|p| p.order.unwrap_or_else(|| panic!("`{}`: {:?}", p.text, p.error)))
        .collect()
}

#[test]
fn rulebook_sample_spring_1901_with_power_prefixes() {
    let text = "Austria: A Vie-Tri, A Bud-Gal, F Tri-Alb
England: A Lvp-Yor, F Lon-Nth, F Edi-Nrg
France: A Par-Bur, A Mar-Spa, F Bre-Pic
Germany: A Ber-Kie, A Mun-Ruh, F Kie-Den
Italy: A Ven-Pie, A Rom-Ven, F Nap-Ion
Russia: A Mos-Ukr, A War-Gal, F StP-Bot, F Sev-Bla
Turkey: A Con-Bul, A Smy-Con, F Ank-Bla";
    let os = orders(&start(), text);
    assert_eq!(os.len(), 22);
    assert!(os.iter().all(|o| matches!(o.kind, OrderKind::Move { .. })));
    assert_eq!(
        serde_json::to_value(&os[0]).unwrap(),
        json!({ "power": "AUS", "unit": "Vie", "type": "move", "to": "Tri" })
    );
    assert_eq!(os[18].power, Power::Russia);
    assert_eq!(os[19].power, Power::Turkey);
}

#[test]
fn notation_variants() {
    let s = start();
    let got = parsed(
        &s,
        "par - bur\nA Paris to Burgundy\nA Par–Bur\nA Par -> Bur\nF Lon Holds\nF Lon\nA Par H\n\
         A Mar S A Par-Bur\nA Mar supports Paris - Burgundy\nF Bre S A Par\n\
         F Kie-Den.\n# comment line\nA Ven-Pie; A Rom-Ven",
    );
    let mv = json!({ "power": "FRA", "unit": "Par", "type": "move", "to": "Bur" });
    assert_eq!(got[0], mv);
    assert_eq!(got[1], mv);
    assert_eq!(got[2], mv);
    assert_eq!(got[3], mv);
    assert_eq!(got[4], json!({ "power": "ENG", "unit": "Lon", "type": "hold" }));
    assert_eq!(got[5], json!({ "power": "ENG", "unit": "Lon", "type": "hold" }));
    assert_eq!(got[6], json!({ "power": "FRA", "unit": "Par", "type": "hold" }));
    let sup = json!({ "power": "FRA", "unit": "Mar", "type": "support", "target": "Par", "to": "Bur" });
    assert_eq!(got[7], sup);
    assert_eq!(got[8], sup);
    assert_eq!(
        got[9],
        json!({ "power": "FRA", "unit": "Bre", "type": "support", "target": "Par" })
    );
    assert_eq!(
        got[10],
        json!({ "power": "GER", "unit": "Kie", "type": "move", "to": "Den" })
    );
    assert_eq!(got.len(), 13, "comment skipped, `;` splits");
}

#[test]
fn coasts_full_names_and_aliases() {
    let mut s = start();
    s.units.push(Unit {
        power: Power::France,
        unit_type: UnitType::F,
        location: "Mid".into(),
        coast: None,
    });
    let got = parsed(
        &s,
        "F Mid-Spa NC\nF Mid-Spa/sc\nF Mid - Spain (nc)\nF Mid-Spa north coast\n\
         F Mid-Spa\nF Mao-Por\nF Mid-Atlantic Ocean - Portugal\nF StP/SC-Bot\nF St. Petersburg - Gulf of Bothnia",
    );
    let to = |v: &Value| v["to"].as_str().unwrap().to_string();
    assert_eq!(to(&got[0]), "Spa/NC");
    assert_eq!(to(&got[1]), "Spa/SC");
    assert_eq!(to(&got[2]), "Spa/NC");
    assert_eq!(to(&got[3]), "Spa/NC");
    assert_eq!(to(&got[4]), "Spa", "coast left to validation (R-MAP-07)");
    assert_eq!(to(&got[5]), "Por");
    assert_eq!(
        got[6],
        json!({ "power": "FRA", "unit": "Mid", "type": "move", "to": "Por" })
    );
    assert_eq!(
        got[7],
        json!({ "power": "RUS", "unit": "StP", "type": "move", "to": "Bot" })
    );
    assert_eq!(got[8], got[7]);
}

#[test]
fn convoys_via_convoy_and_foreign_units() {
    let mut s = start();
    s.units.retain(|u| u.location != "Lon");
    s.units.push(Unit {
        power: Power::England,
        unit_type: UnitType::A,
        location: "Lon".into(),
        coast: None,
    });
    s.units.push(Unit {
        power: Power::England,
        unit_type: UnitType::F,
        location: "Nth".into(),
        coast: None,
    });
    s.units.push(Unit {
        power: Power::Turkey,
        unit_type: UnitType::A,
        location: "Bul".into(),
        coast: None,
    });
    s.units.push(Unit {
        power: Power::Austria,
        unit_type: UnitType::A,
        location: "Ser".into(),
        coast: None,
    });
    let got = parsed(
        &s,
        "F Nth C A Lon-Nwy\nF Nth convoys London to Norway\nA Lon-Nwy via convoy\nA Ser S Turkish A Bul-Rum",
    );
    let cvy = json!({ "power": "ENG", "unit": "Nth", "type": "convoy", "target": "Lon", "to": "Nwy" });
    assert_eq!(got[0], cvy);
    assert_eq!(got[1], cvy);
    assert_eq!(
        got[2],
        json!({ "power": "ENG", "unit": "Lon", "type": "move", "to": "Nwy", "viaConvoy": true })
    );
    assert_eq!(
        got[3],
        json!({ "power": "AUS", "unit": "Ser", "type": "support", "target": "Bul", "to": "Rum" })
    );
}

#[test]
fn errors_are_reported_per_item() {
    let s = start();
    let got = parsed(
        &s,
        "A Bur-Par\nF Par-Bur\nA Par-Xyz\nA Par flies\nFrance: A Vie-Tri\nA Par-Bur, A Mar-Spa",
    );
    assert!(got[0]["error"].as_str().unwrap().contains("no unit in Bur"));
    assert!(
        got[1]["error"].as_str().unwrap().contains("is an army"),
        "R-ORD-06: wrong unit type"
    );
    assert!(got[2]["error"].as_str().unwrap().contains("unknown province"));
    assert!(got[3]["error"].as_str().unwrap().contains("unknown order"));
    assert!(got[4]["error"].as_str().unwrap().contains("belongs to AUS"));
    assert_eq!(got[5]["unit"], "Par", "an error does not affect the next items");
    assert_eq!(got[6]["unit"], "Mar");
}

#[test]
fn retreat_phase() {
    let mut s = start();
    s.phase = "F1901R".parse::<Phase>().unwrap();
    s.units.retain(|u| u.location != "Ven");
    s.dislodged.push(DislodgedUnit {
        power: Power::Italy,
        unit_type: UnitType::A,
        location: "Ven".into(),
        coast: None,
        attacker_from: "Tyr".into(),
        retreat_options: vec!["Apu".into(), "Pie".into(), "Tus".into()],
    });
    let got = parsed(&s, "A Ven-Tus\nA Ven R Pie\nA Ven D\nA Par-Bur\nA Ven");
    assert_eq!(
        got[0],
        json!({ "power": "ITA", "unit": "Ven", "type": "retreat", "to": "Tus" })
    );
    assert_eq!(
        got[1],
        json!({ "power": "ITA", "unit": "Ven", "type": "retreat", "to": "Pie" })
    );
    assert_eq!(got[2], json!({ "power": "ITA", "unit": "Ven", "type": "disband" }));
    assert!(got[3]["error"].as_str().unwrap().contains("no dislodged unit"));
    assert!(got[4]["error"].is_string());
}

#[test]
fn adjustment_phase_builds_disbands_waives() {
    let mut s = start();
    s.phase = "W1901A".parse::<Phase>().unwrap();
    s.units.retain(|u| u.location != "StP" && u.location != "Vie");
    let got = parsed(
        &s,
        "B F StP NC\nBuild A Vie\nB A StP/SC\nA Par D\nRussia: waive\nW Vie\nB StP\nA Par-Bur\nRussia:\nW StP\nA Par D",
    );
    assert_eq!(
        got[0],
        json!({ "power": "RUS", "unit": "StP", "type": "build", "unitType": "F", "coast": "NC" })
    );
    assert_eq!(
        got[1],
        json!({ "power": "AUS", "unit": "Vie", "type": "build", "unitType": "A" })
    );
    assert_eq!(
        got[2],
        json!({ "power": "RUS", "unit": "StP", "type": "build", "unitType": "A" }),
        "coast ignored for an army"
    );
    assert_eq!(got[3], json!({ "power": "FRA", "unit": "Par", "type": "disband" }));
    assert_eq!(got[4]["type"], "waive");
    assert_eq!(got[4]["power"], "RUS");
    assert_eq!(got[5]["power"], "AUS", "a prefix applies to its own line only");
    assert!(got[6]["error"].as_str().unwrap().contains("A or F"));
    assert!(got[7]["error"].is_string());
    assert_eq!(got[8]["power"], "RUS", "`Russia:` alone applies to the following lines");
    assert!(got[9]["error"].as_str().unwrap().contains("belongs to FRA"));
}
