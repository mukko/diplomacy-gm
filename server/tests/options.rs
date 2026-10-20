//! Order options and default disbands (engine::options): what the GUI is offered must be exactly
//! what the adjudicator accepts.

use std::collections::BTreeMap;

use diplomacy_gm_server::adjudicator::Adjudicator;
use diplomacy_gm_server::engine::Engine;
use diplomacy_gm_server::engine::options::{OrderStatus, check_orders, default_disbands, power_options, unit_options};
use diplomacy_gm_server::model::{CivilDisorder, GameState, Order, OrderKind, Power, Unit, UnitType};

fn unit(power: Power, unit_type: UnitType, location: &str) -> Unit {
    Unit {
        power,
        unit_type,
        location: location.into(),
        coast: None,
    }
}

fn board(units: Vec<Unit>) -> GameState {
    let mut s = GameState::standard_start();
    s.units = units;
    s
}

fn moves(orders: &[Order]) -> Vec<String> {
    let mut out: Vec<String> = orders
        .iter()
        .filter_map(|o| match &o.kind {
            OrderKind::Move { to, via_convoy } => {
                Some(format!("{to}{}", if *via_convoy { " (via convoy)" } else { "" }))
            }
            _ => None,
        })
        .collect();
    out.sort();
    out
}

#[test]
fn every_offered_order_is_valid_for_the_adjudicator() {
    let mut boards = vec![GameState::standard_start()];
    boards.push(board(vec![
        unit(Power::England, UnitType::A, "Lon"),
        unit(Power::England, UnitType::F, "Nth"),
        unit(Power::France, UnitType::F, "Eng"),
        unit(Power::Germany, UnitType::A, "Hol"),
    ]));
    for state in boards {
        for u in &state.units {
            let opts = unit_options(&state, &u.location).unwrap();
            assert!(!opts.orders.is_empty());
            for order in &opts.orders {
                let res = Engine.adjudicate(&state, std::slice::from_ref(order)).unwrap();
                assert!(
                    res.orders[0].valid,
                    "{order:?} offered but rejected: {:?}",
                    res.orders[0].reason
                );
            }
        }
    }
}

#[test]
fn movement_options_at_the_start() {
    let s = GameState::standard_start();
    let par = unit_options(&s, "Par").unwrap();
    assert_eq!(moves(&par.orders), ["Bre", "Bur", "Gas", "Pic"]);
    assert!(par.orders.iter().any(|o| o.kind == OrderKind::Hold));
    // A Par may support A Mar-Bur (Par borders Bur) but not A Mar-Spa.
    let supports = |opts: &[Order], target: &str, to: Option<&str>| {
        opts.iter().any(|o| {
            o.kind
                == OrderKind::Support {
                    target: target.into(),
                    to: to.map(Into::into),
                }
        })
    };
    assert!(supports(&par.orders, "Mar", Some("Bur")));
    assert!(!supports(&par.orders, "Mar", Some("Spa")));
    assert!(supports(&par.orders, "Bre", None));

    // F StP/SC: only the south coast's neighbours.
    let stp = unit_options(&s, "StP").unwrap();
    assert_eq!(moves(&stp.orders), ["Bot", "Fin", "Lvn"]);
    // F Con can go to either coast of Bul, each named explicitly (R-MAP-07).
    let con = unit_options(&s, "Con").unwrap();
    assert!(
        moves(&con.orders).contains(&"Bul".to_string()),
        "A Con is an army: no coasts"
    );
    let ank = unit_options(&s, "Ank").unwrap();
    assert_eq!(moves(&ank.orders), ["Arm", "Bla", "Con"]);

    assert!(unit_options(&s, "Bel").is_none());
}

#[test]
fn convoy_options() {
    let s = board(vec![
        unit(Power::England, UnitType::A, "Lon"),
        unit(Power::England, UnitType::F, "Nth"),
        unit(Power::Germany, UnitType::A, "Hol"),
    ]);
    let lon = unit_options(&s, "Lon").unwrap();
    let m = moves(&lon.orders);
    for to in ["Nwy", "Bel", "Den", "Edi", "Yor (via convoy)", "Wal"] {
        assert!(m.contains(&to.to_string()), "{to} missing from {m:?}");
    }
    assert!(!m.contains(&"Wal (via convoy)".to_string()), "Nth does not border Wal");
    let nth = unit_options(&s, "Nth").unwrap();
    let convoys: Vec<String> = nth
        .orders
        .iter()
        .filter_map(|o| match &o.kind {
            OrderKind::Convoy { target, to } => Some(format!("{target}-{to}")),
            _ => None,
        })
        .collect();
    assert!(
        convoys.contains(&"Lon-Nwy".into()) && convoys.contains(&"Hol-Edi".into()),
        "{convoys:?}"
    );
    assert!(!convoys.iter().any(|c| c == "Lon-Wal"), "{convoys:?}");
}

#[test]
fn civil_disorder_units_get_no_orders() {
    let mut s = GameState::standard_start();
    s.civil_disorder.push(CivilDisorder {
        power: Power::Italy,
        since: s.phase,
    });
    let rom = unit_options(&s, "Rom").unwrap();
    assert!(rom.civil_disorder && rom.orders.is_empty());
    assert!(!unit_options(&s, "Par").unwrap().civil_disorder);
}

#[test]
fn retreat_options_are_the_precomputed_ones_plus_disband() {
    let s = board(vec![
        unit(Power::France, UnitType::A, "Mar"),
        unit(Power::France, UnitType::A, "Gas"),
        unit(Power::Germany, UnitType::A, "Bur"),
    ]);
    let orders = vec![
        Order {
            power: Power::France,
            unit: "Mar".into(),
            kind: OrderKind::Move {
                to: "Bur".into(),
                via_convoy: false,
            },
        },
        Order {
            power: Power::France,
            unit: "Gas".into(),
            kind: OrderKind::Support {
                target: "Mar".into(),
                to: Some("Bur".into()),
            },
        },
    ];
    let r = Engine.adjudicate(&s, &orders).unwrap().next_state;
    assert_eq!(r.phase.to_string(), "S1901R");
    let bur = unit_options(&r, "Bur").unwrap();
    assert_eq!(bur.unit.power, Power::Germany);
    let kinds: Vec<&OrderKind> = bur.orders.iter().map(|o| &o.kind).collect();
    assert!(kinds.contains(&&OrderKind::Disband));
    let retreats: Vec<String> = kinds
        .iter()
        .filter_map(|k| match k {
            OrderKind::Retreat { to } => Some(to.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(retreats, r.dislodged[0].retreat_options);
    assert!(
        unit_options(&r, "Par").is_none(),
        "only dislodged units take retreat orders"
    );
}

fn winter(units: Vec<Unit>, owners: &[(&str, Option<Power>)]) -> GameState {
    let mut s = board(units);
    s.phase = "W1901A".parse().unwrap();
    for (sc, owner) in owners {
        s.supply_centers.insert(sc.to_string(), *owner);
    }
    s
}

#[test]
fn build_options() {
    let s = winter(vec![unit(Power::Russia, UnitType::A, "Mos")], &[]);
    let opts = power_options(&s, Power::Russia);
    assert_eq!(opts.adjustment, Some(3));
    let builds: Vec<String> = opts
        .builds
        .iter()
        .map(|o| match &o.kind {
            OrderKind::Build { unit_type, coast } => {
                format!(
                    "{unit_type:?} {}{}",
                    o.unit,
                    coast.as_deref().map(|c| format!("/{c}")).unwrap_or_default()
                )
            }
            OrderKind::Waive => "waive".into(),
            k => panic!("{k:?}"),
        })
        .collect();
    assert_eq!(
        builds,
        ["A Sev", "F Sev", "A StP", "F StP/NC", "F StP/SC", "A War", "waive"]
    );
    // Each offered build is accepted by the adjudicator.
    for order in &opts.builds {
        let res = Engine.adjudicate(&s, std::slice::from_ref(order)).unwrap();
        assert!(res.orders[0].valid, "{order:?}");
    }
    let mut disorder = s.clone();
    disorder.civil_disorder.push(CivilDisorder {
        power: Power::Russia,
        since: s.phase,
    });
    assert!(
        power_options(&disorder, Power::Russia).builds.is_empty(),
        "no builds in civil disorder"
    );
}

#[test]
fn default_disbands_take_submitted_disbands_first() {
    let s = winter(
        vec![
            unit(Power::Russia, UnitType::A, "Lvn"),
            unit(Power::Russia, UnitType::A, "Swe"),
            unit(Power::Russia, UnitType::A, "Ukr"),
        ],
        &[("StP", None), ("Mos", None), ("War", None)],
    );
    let proposals = default_disbands(&s, &[]);
    assert_eq!(proposals.len(), 1);
    let p = &proposals[0];
    assert_eq!((p.power, p.required), (Power::Russia, 2));
    let proposed: BTreeMap<&str, usize> = p.proposed.iter().map(|d| (d.unit.as_str(), d.distance)).collect();
    assert_eq!(
        proposed,
        BTreeMap::from([("A Swe", 2), ("A Lvn", 1)]),
        "Swe is farthest; Livonia before Ukraine"
    );

    let gm = Order {
        power: Power::Russia,
        unit: "Ukr".into(),
        kind: OrderKind::Disband,
    };
    let p = &default_disbands(&s, &[gm])[0];
    assert_eq!(p.ordered, ["A Ukr"]);
    assert_eq!(p.proposed.len(), 1);
    assert_eq!(p.proposed[0].unit, "A Swe");

    assert!(default_disbands(&GameState::standard_start(), &[]).is_empty());
}

fn mv(power: Power, unit: &str, to: &str) -> Order {
    Order {
        power,
        unit: unit.into(),
        kind: OrderKind::Move {
            to: to.into(),
            via_convoy: false,
        },
    }
}

fn statuses(state: &GameState, orders: &[Order]) -> Vec<OrderStatus> {
    check_orders(state, orders).into_iter().map(|c| c.status).collect()
}

#[test]
fn order_checks_in_a_movement_phase() {
    let s = GameState::standard_start();
    let orders = [
        mv(Power::Italy, "Ven", "Pie"),
        mv(Power::Italy, "Nap", "Ven"), // fleet: Nap does not border Ven → holds
        mv(Power::Italy, "Par", "Bur"), // not Italy's unit → ignored
    ];
    let checks = check_orders(&s, &orders);
    assert_eq!(
        checks.iter().map(|c| c.status).collect::<Vec<_>>(),
        [OrderStatus::Valid, OrderStatus::Hold, OrderStatus::Ignored]
    );
    assert!(checks[1].rules.contains(&"R-ORD-04".to_string()) && checks[1].reason.is_some());
    assert!(checks[2].rules.contains(&"PR-14".to_string()));
    assert!(checks[0].reason.is_none());
}

#[test]
fn order_checks_in_retreat_and_adjustment_phases() {
    let s = board(vec![
        unit(Power::France, UnitType::A, "Mar"),
        unit(Power::France, UnitType::A, "Gas"),
        unit(Power::Germany, UnitType::A, "Bur"),
    ]);
    let orders = vec![
        mv(Power::France, "Mar", "Bur"),
        Order {
            power: Power::France,
            unit: "Gas".into(),
            kind: OrderKind::Support {
                target: "Mar".into(),
                to: Some("Bur".into()),
            },
        },
    ];
    let r = Engine.adjudicate(&s, &orders).unwrap().next_state;
    let retreat = |to: &str| Order {
        power: Power::Germany,
        unit: "Bur".into(),
        kind: OrderKind::Retreat { to: to.into() },
    };
    let not_dislodged = Order {
        power: Power::France,
        unit: "Gas".into(),
        kind: OrderKind::Retreat { to: "Spa".into() },
    };
    assert_eq!(
        statuses(&r, &[retreat("Ruh"), not_dislodged]),
        [OrderStatus::Valid, OrderStatus::Ignored]
    );
    assert_eq!(
        statuses(&r, &[retreat("Mar")]),
        [OrderStatus::Disband],
        "Mar is the attacker's origin"
    );
    assert_eq!(
        statuses(&r, &[retreat("Ruh"), retreat("Kie")]),
        [OrderStatus::Disband, OrderStatus::Disband],
        "PR-05"
    );

    let w = winter(
        vec![unit(Power::Russia, UnitType::A, "Ukr")],
        &[("StP", None), ("Mos", None)],
    );
    let build = |at: &str| Order {
        power: Power::Russia,
        unit: at.into(),
        kind: OrderKind::Build {
            unit_type: UnitType::A,
            coast: None,
        },
    };
    // Russia: SCs Sev, War (2) − 1 unit = one build.
    assert_eq!(
        statuses(&w, &[build("Sev"), build("War")]),
        [OrderStatus::Valid, OrderStatus::Ignored]
    );
}

#[test]
fn civil_disorder_powers_are_offered_disbands_in_adjustments() {
    // GM-entered disbands are used for a power in civil disorder (PR-08), so they are offered.
    let mut s = winter(
        vec![
            unit(Power::Russia, UnitType::A, "Lvn"),
            unit(Power::Russia, UnitType::A, "Swe"),
        ],
        &[("StP", None), ("Mos", None), ("War", None)],
    );
    s.civil_disorder.push(CivilDisorder {
        power: Power::Russia,
        since: s.phase,
    });
    let lvn = unit_options(&s, "Lvn").unwrap();
    assert!(lvn.civil_disorder);
    assert_eq!(
        lvn.orders.iter().map(|o| &o.kind).collect::<Vec<_>>(),
        [&OrderKind::Disband]
    );
    let opts = power_options(&s, Power::Russia);
    assert!(opts.units.iter().all(|u| u.orders.len() == 1));
    assert!(opts.builds.is_empty());
}
