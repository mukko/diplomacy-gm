//! DATC (Diplomacy Adjudicator Test Cases, L. B. Kruijswijk, v3.0 numbering) movement-phase cases
//! 6.A–6.G as engine tests. Where DATC offers options, the rulebook (2000/2008) option is used
//! unless a project ruling (docs/en/adjudication.md § 5) says otherwise. Cases that our rules do
//! not settle are listed with a `// skipped:` comment instead of a test.
//!
//! Retreat and adjustment cases (6.H–6.J) and the paradoxes 6.F.16 / 6.F.22 are in rulebook.rs.

mod common;

use common::*;
use diplomacy_gm_server::model::{Power, UnitType};

const ENG: Power = Power::England;
const FRA: Power = Power::France;
const GER: Power = Power::Germany;
const ITA: Power = Power::Italy;
const AUS: Power = Power::Austria;
const RUS: Power = Power::Russia;
const TUR: Power = Power::Turkey;

/// Units from `units` (orders in it are dropped), orders from `orders` (their units are dropped).
fn run_split(units: &str, orders: &str) -> Res {
    run_on(&board(parse(units).0), &parse(orders).1)
}

// ----- 6.A. Basic checks -------------------------------------------------------------------------

#[test]
fn datc_6a01_moving_to_an_area_that_is_not_a_neighbour() {
    let r = run("ENG: F Nth-Pic");
    assert!(!r.valid("Nth"));
    assert_eq!(r.at("Nth"), Some((ENG, UnitType::F)));
}

#[test]
fn datc_6a02_move_army_to_sea() {
    let r = run("ENG: A Lvp-Iri");
    assert!(!r.valid("Lvp"), "R-MOV-01");
    assert_eq!(r.at("Lvp"), Some((ENG, UnitType::A)));
}

#[test]
fn datc_6a03_move_fleet_to_land() {
    let r = run("GER: F Kie-Mun");
    assert!(!r.valid("Kie"), "R-MOV-02");
    assert_eq!(r.at("Kie"), Some((GER, UnitType::F)));
}

#[test]
fn datc_6a04_move_to_own_sector() {
    let r = run("GER: F Kie-Kie");
    assert!(!r.valid("Kie"));
    assert_eq!(r.at("Kie"), Some((GER, UnitType::F)));
}

#[test]
fn datc_6a05_move_to_own_sector_with_convoy() {
    let r = run("ENG: F Nth C A Yor-Yor, A Yor-Yor, A Lvp S A Yor-Yor; GER: F Lon-Yor, A Wal S F Lon-Yor");
    for u in ["Nth", "Yor", "Lvp"] {
        assert!(!r.valid(u), "{u}");
    }
    assert_eq!(r.outcome("Lon"), Success);
    assert!(r.dislodged("Yor"));
    assert_eq!(r.at("Yor"), Some((GER, UnitType::F)));
}

#[test]
fn datc_6a06_ordering_a_unit_of_another_country() {
    let r = run_split("ENG: F Lon", "GER: F Lon-Nth");
    assert!(!r.valid("Lon"));
    assert!(r.rules("Lon").contains(&"PR-14".to_string()));
    assert_eq!(r.at("Lon"), Some((ENG, UnitType::F)));
}

#[test]
fn datc_6a07_only_armies_can_be_convoyed() {
    let r = run("ENG: F Lon-Bel, F Nth C A Lon-Bel");
    assert!(!r.valid("Lon"));
    assert!(!r.valid("Nth"));
    assert!(r.rules("Nth").contains(&"R-CVY-02".to_string()));
    assert_eq!(r.at("Lon"), Some((ENG, UnitType::F)));
}

#[test]
fn datc_6a08_support_to_hold_yourself_is_not_possible() {
    let r = run("ITA: A Ven-Tri, A Tyr S A Ven-Tri; AUS: F Tri S F Tri");
    assert!(!r.valid("Tri"));
    assert_eq!(r.outcome("Ven"), Success);
    assert!(r.dislodged("Tri"));
}

#[test]
fn datc_6a09_fleets_must_follow_coast_if_not_on_sea() {
    let r = run("ITA: F Rom-Ven");
    assert!(!r.valid("Rom"), "R-MAP-05");
    assert_eq!(r.at("Rom"), Some((ITA, UnitType::F)));
}

#[test]
fn datc_6a10_support_on_unreachable_destination_not_possible() {
    let r = run("AUS: A Ven H; ITA: F Rom S A Apu-Ven, A Apu-Ven");
    assert!(!r.valid("Rom"), "R-SUP-02");
    assert_eq!(r.outcome("Apu"), Fail);
    assert!(!r.dislodged("Ven"));
}

#[test]
fn datc_6a11_simple_bounce() {
    let r = run("AUS: A Vie-Tyr; ITA: A Ven-Tyr");
    assert_eq!((r.outcome("Vie"), r.outcome("Ven")), (Fail, Fail));
    assert_eq!(r.standoffs(), ["Tyr"]);
}

#[test]
fn datc_6a12_bounce_of_three_units() {
    let r = run("AUS: A Vie-Tyr; GER: A Mun-Tyr; ITA: A Ven-Tyr");
    for u in ["Vie", "Mun", "Ven"] {
        assert_eq!(r.outcome(u), Fail, "{u}");
    }
    assert_eq!(r.at("Tyr"), None);
}

// ----- 6.B. Coastal issues -----------------------------------------------------------------------

#[test]
fn datc_6b01_moving_with_unspecified_coast_when_coast_is_necessary() {
    let r = run("FRA: F Por-Spa");
    assert!(!r.valid("Por"));
    assert!(r.rules("Por").contains(&"R-MAP-07".to_string()));
    assert_eq!(r.at("Por"), Some((FRA, UnitType::F)));
}

#[test]
fn datc_6b02_moving_with_unspecified_coast_when_coast_is_not_necessary() {
    // Only the north coast is reachable from Gascony: the coast is inferred (PR-04).
    let r = run("FRA: F Gas-Spa");
    assert_eq!(r.outcome("Gas"), Success);
    assert_eq!(r.coast_at("Spa").as_deref(), Some("NC"));
}

#[test]
fn datc_6b03_moving_with_wrong_coast_when_coast_is_not_necessary() {
    let r = run("FRA: F Gas-Spa/SC");
    assert!(!r.valid("Gas"));
    assert!(r.rules("Gas").contains(&"PR-04".to_string()));
    assert_eq!(r.at("Gas"), Some((FRA, UnitType::F)));
}

#[test]
fn datc_6b04_support_to_unreachable_coast_allowed() {
    // Marseilles can reach Spain (south coast), so it may support into the north coast (R-SUP-03).
    let r = run("FRA: F Gas-Spa/NC, F Mar S F Gas-Spa/NC; ITA: F Wes-Spa/SC");
    assert!(r.valid("Mar"));
    assert_eq!(r.outcome("Gas"), Success);
    assert_eq!(r.outcome("Wes"), Fail);
    assert_eq!(r.coast_at("Spa").as_deref(), Some("NC"));
}

#[test]
fn datc_6b05_support_from_unreachable_coast_not_allowed() {
    let r = run("FRA: F Mar-GoL, F Spa/NC S F Mar-GoL; ITA: F GoL H");
    assert!(!r.valid("Spa"), "R-SUP-02, R-MAP-06");
    assert_eq!(r.outcome("Mar"), Fail);
    assert!(!r.dislodged("GoL"));
}

#[test]
fn datc_6b06_support_can_be_cut_with_other_coast() {
    let r = run("ENG: F Iri S F NAt-Mid, F NAt-Mid; FRA: F Spa/NC S F Mid, F Mid H; ITA: F GoL-Spa/SC");
    assert_eq!(r.outcome("Spa"), Cut);
    assert_eq!(r.outcome("NAt"), Success);
    assert!(r.dislodged("Mid"));
}

#[test]
fn datc_6b07_supporting_own_unit_with_unspecified_coast() {
    // A support names the province; the coast does not matter (R-SUP-03).
    let r = run("FRA: F Por S F Mid-Spa, F Mid-Spa/NC; ITA: F GoL S F Wes-Spa/SC, F Wes-Spa/SC");
    assert_eq!(r.outcome("Por"), Success);
    assert_eq!((r.outcome("Mid"), r.outcome("Wes")), (Fail, Fail));
    assert_eq!(r.at("Spa"), None);
}

#[test]
fn datc_6b08_supporting_with_unspecified_coast_when_only_one_coast_is_possible() {
    let r = run("FRA: F Por S F Gas-Spa, F Gas-Spa/NC; ITA: F GoL S F Wes-Spa/SC, F Wes-Spa/SC");
    assert_eq!(r.outcome("Por"), Success);
    assert_eq!((r.outcome("Gas"), r.outcome("Wes")), (Fail, Fail));
}

// skipped: 6.B.9 (supporting with wrong coast). DATC wants `F Por S F Mid-Spa/NC` to be void when
// F Mid moves to Spa/SC. Our rules let a support into a split-coast province count "regardless of
// coast" (R-SUP-03) and R-SUP-06 does not say whether a coast in the support must match, so the
// engine (which only compares provinces) gives the support. Needs a ruling.

#[test]
fn datc_6b10_unit_ordered_with_wrong_coast() {
    // The fleet is on the south coast but the order names the north coast. There is only one
    // unit in Spain, so the order has one possible meaning and is executed (R-ORD-05/06).
    let r = run_split("FRA: F Spa/SC", "FRA: F Spa/NC-GoL");
    assert!(r.valid("Spa"));
    assert_eq!(r.outcome("Spa"), Success);
    assert_eq!(r.at("GoL"), Some((FRA, UnitType::F)));
}

#[test]
fn datc_6b11_coast_cannot_be_ordered_to_change() {
    let r = run_split("FRA: F Spa/NC", "FRA: F Spa/SC-GoL");
    assert!(!r.valid("Spa"));
    assert_eq!(r.coast_at("Spa").as_deref(), Some("NC"));
}

#[test]
fn datc_6b12_army_movement_with_coastal_specification() {
    // A coast on an army order is ignored (PR-04).
    let r = run("FRA: A Gas-Spa/NC");
    assert_eq!(r.outcome("Gas"), Success);
    assert_eq!(r.at("Spa"), Some((FRA, UnitType::A)));
    assert_eq!(r.coast_at("Spa"), None);
}

#[test]
fn datc_6b13_coastal_crawl_not_allowed() {
    // Still a head-to-head battle although the coasts differ (R-STD-04).
    let r = run("TUR: F Bul/SC-Con, F Con-Bul/EC");
    assert_eq!((r.outcome("Bul"), r.outcome("Con")), (Fail, Fail));
    assert_eq!(r.coast_at("Bul").as_deref(), Some("SC"));
    assert_eq!(r.at("Con"), Some((TUR, UnitType::F)));
}

// 6.B.14 (building with unspecified coast) is an adjustment case: see datc_6i04 in rulebook.rs.

#[test]
fn datc_6b15_supporting_foreign_unit_with_unspecified_coast() {
    let r = run("FRA: F Por S F Mid-Spa; ENG: F Mid-Spa/NC; ITA: F GoL S F Wes-Spa/SC, F Wes-Spa/SC");
    assert_eq!(r.outcome("Por"), Success);
    assert_eq!((r.outcome("Mid"), r.outcome("Wes")), (Fail, Fail));
}

// ----- 6.C. Circular movement --------------------------------------------------------------------

#[test]
fn datc_6c01_three_army_circular_movement() {
    let r = run("TUR: F Ank-Con, A Con-Smy, A Smy-Ank");
    for u in ["Ank", "Con", "Smy"] {
        assert_eq!(r.outcome(u), Success, "{u}");
    }
    assert_eq!(r.at("Con"), Some((TUR, UnitType::F)));
    assert_eq!(r.at("Ank"), Some((TUR, UnitType::A)));
    assert!(r.rules("Ank").contains(&"R-STD-05".to_string()));
}

#[test]
fn datc_6c02_three_army_circular_movement_with_support() {
    let r = run("TUR: F Ank-Con, A Con-Smy, A Smy-Ank, A Bul S F Ank-Con");
    for u in ["Ank", "Con", "Smy"] {
        assert_eq!(r.outcome(u), Success, "{u}");
    }
    assert_eq!(r.at("Con"), Some((TUR, UnitType::F)));
}

#[test]
fn datc_6c03_a_disrupted_three_army_circular_movement() {
    let r = run("TUR: F Ank-Con, A Con-Smy, A Smy-Ank, A Bul-Con");
    for u in ["Ank", "Con", "Smy", "Bul"] {
        assert_eq!(r.outcome(u), Fail, "{u}");
    }
    assert_eq!(r.at("Ank"), Some((TUR, UnitType::F)));
}

#[test]
fn datc_6c04_a_circular_movement_with_attacked_convoy() {
    let r = run("AUS: A Tri-Ser, A Ser-Bul; \
         TUR: A Bul-Tri, F Aeg C A Bul-Tri, F Ion C A Bul-Tri, F Adr C A Bul-Tri; ITA: F Nap-Ion");
    for u in ["Tri", "Ser", "Bul"] {
        assert_eq!(r.outcome(u), Success, "{u}");
    }
    assert_eq!(r.outcome("Nap"), Fail);
    assert_eq!(r.at("Tri"), Some((TUR, UnitType::A)));
    assert_eq!(r.at("Bul"), Some((AUS, UnitType::A)));
}

#[test]
fn datc_6c05_a_disrupted_circular_movement_due_to_dislodged_convoy() {
    let r = run("AUS: A Tri-Ser, A Ser-Bul; \
         TUR: A Bul-Tri, F Aeg C A Bul-Tri, F Ion C A Bul-Tri, F Adr C A Bul-Tri; \
         ITA: F Nap-Ion, F Tun S F Nap-Ion");
    assert!(r.dislodged("Ion"));
    for u in ["Tri", "Ser", "Bul"] {
        assert_eq!(r.outcome(u), Fail, "{u}");
    }
    assert_eq!(r.at("Bul"), Some((TUR, UnitType::A)));
    assert_eq!(r.at("Tri"), Some((AUS, UnitType::A)));
}

#[test]
fn datc_6c06_two_armies_with_two_convoys() {
    let r = run("ENG: F Nth C A Lon-Bel, A Lon-Bel; FRA: F Eng C A Bel-Lon, A Bel-Lon");
    assert_eq!((r.outcome("Lon"), r.outcome("Bel")), (Success, Success));
    assert_eq!(r.at("Bel"), Some((ENG, UnitType::A)));
    assert_eq!(r.at("Lon"), Some((FRA, UnitType::A)));
}

#[test]
fn datc_6c07_disrupted_unit_swap() {
    let r = run("ENG: F Nth C A Lon-Bel, A Lon-Bel; FRA: F Eng C A Bel-Lon, A Bel-Lon, A Bur-Bel");
    for u in ["Lon", "Bel", "Bur"] {
        assert_eq!(r.outcome(u), Fail, "{u}");
    }
    assert_eq!(r.at("Lon"), Some((ENG, UnitType::A)));
}

#[test]
fn datc_6c08_no_self_dislodgement_in_disrupted_circular_movement() {
    let r = run("TUR: F Con-Bla, A Bul-Con, A Smy S A Bul-Con; RUS: F Bla-Bul/EC; AUS: A Ser-Bul");
    for u in ["Con", "Bul", "Bla", "Ser"] {
        assert_eq!(r.outcome(u), Fail, "{u}");
    }
    assert!(!r.dislodged("Con"));
    assert_eq!(r.at("Con"), Some((TUR, UnitType::F)));
}

#[test]
fn datc_6c09_no_help_in_dislodgement_of_own_unit_in_disrupted_circular_movement() {
    let r = run("TUR: F Con-Bla, A Smy S A Bul-Con; RUS: F Bla-Bul/EC; AUS: A Ser-Bul, A Bul-Con");
    for u in ["Con", "Bul", "Bla", "Ser"] {
        assert_eq!(r.outcome(u), Fail, "{u}");
    }
    assert!(!r.dislodged("Con"));
}

// ----- 6.D. Supports and dislodges ---------------------------------------------------------------

#[test]
fn datc_6d01_supported_hold_can_prevent_dislodgement() {
    let r = run("AUS: F Adr S A Tri-Ven, A Tri-Ven; ITA: A Ven H, A Tyr S A Ven");
    assert_eq!(r.outcome("Tri"), Fail);
    assert!(!r.dislodged("Ven"));
}

#[test]
fn datc_6d02_a_move_cuts_support_on_hold() {
    let r = run("AUS: F Adr S A Tri-Ven, A Tri-Ven, A Vie-Tyr; ITA: A Ven H, A Tyr S A Ven");
    assert_eq!(r.outcome("Tyr"), Cut);
    assert_eq!(r.outcome("Tri"), Success);
    assert!(r.dislodged("Ven"));
}

#[test]
fn datc_6d03_a_move_cuts_support_on_move() {
    let r = run("AUS: F Adr S A Tri-Ven, A Tri-Ven; ITA: A Ven H, F Ion-Adr");
    assert_eq!(r.outcome("Adr"), Cut);
    assert_eq!(r.outcome("Tri"), Fail);
    assert!(!r.dislodged("Ven"));
}

#[test]
fn datc_6d04_support_to_hold_on_unit_supporting_a_hold_allowed() {
    let r = run("GER: A Ber S F Kie, F Kie S A Ber; RUS: F Bal S A Pru-Ber, A Pru-Ber");
    assert_eq!(r.outcome("Pru"), Fail);
    assert!(!r.dislodged("Ber"));
}

#[test]
fn datc_6d05_support_to_hold_on_unit_supporting_a_move_allowed() {
    let r = run("GER: A Ber S A Mun-Sil, F Kie S A Ber, A Mun-Sil; RUS: F Bal S A Pru-Ber, A Pru-Ber");
    assert_eq!(r.outcome("Pru"), Fail);
    assert!(!r.dislodged("Ber"));
}

#[test]
fn datc_6d06_support_to_hold_on_convoying_unit_allowed() {
    let r = run("GER: A Ber-Swe, F Bal C A Ber-Swe, F Pru S F Bal; RUS: F Lvn-Bal, F Bot S F Lvn-Bal");
    assert_eq!(r.outcome("Lvn"), Fail);
    assert!(!r.dislodged("Bal"));
    assert_eq!(r.outcome("Ber"), Success);
    assert_eq!(r.at("Swe"), Some((GER, UnitType::A)));
}

#[test]
fn datc_6d07_support_to_hold_on_moving_unit_not_allowed() {
    let r = run("GER: F Bal-Swe, F Pru S F Bal; RUS: F Lvn-Bal, F Bot S F Lvn-Bal, A Fin-Swe");
    assert_eq!(r.outcome("Pru"), Void, "R-SUP-05");
    assert_eq!(r.outcome("Bal"), Fail);
    assert_eq!(r.outcome("Lvn"), Success);
    assert!(r.dislodged("Bal"));
}

#[test]
fn datc_6d08_failed_convoy_cannot_receive_hold_support() {
    // A Gre-Nap was possible by convoy via the Ionian Sea, so it is a valid move that fails,
    // not an illegal order turned into a hold (R-ORD-04, R-SUP-05).
    let r = run("AUS: F Ion H, A Ser S A Alb-Gre, A Alb-Gre; TUR: A Gre-Nap, A Bul S A Gre");
    assert!(r.valid("Gre"));
    assert_eq!(r.outcome("Gre"), Fail);
    assert_eq!(r.outcome("Bul"), Void);
    assert_eq!(r.outcome("Alb"), Success);
    assert!(r.dislodged("Gre"));
}

#[test]
fn datc_6d09_support_to_move_on_holding_unit_not_allowed() {
    let r = run("ITA: A Ven-Tri, A Tyr S A Ven-Tri; AUS: A Alb S A Tri-Ser, A Tri H");
    assert_eq!(r.outcome("Alb"), Void, "R-SUP-06");
    assert_eq!(r.outcome("Ven"), Success);
    assert!(r.dislodged("Tri"));
}

#[test]
fn datc_6d10_self_dislodgment_prohibited() {
    let r = run("GER: A Ber H, F Kie-Ber, A Mun S F Kie-Ber");
    assert_eq!(r.outcome("Kie"), Fail);
    assert!(r.rules("Kie").contains(&"R-SLF-01".to_string()));
    assert!(!r.dislodged("Ber"));
}

#[test]
fn datc_6d11_no_self_dislodgment_of_returning_unit() {
    let r = run("GER: A Ber-Pru, F Kie-Ber, A Mun S F Kie-Ber; RUS: A War-Pru");
    for u in ["Ber", "Kie", "War"] {
        assert_eq!(r.outcome(u), Fail, "{u}");
    }
    assert!(!r.dislodged("Ber"));
    assert_eq!(r.at("Ber"), Some((GER, UnitType::A)));
}

#[test]
fn datc_6d12_supporting_a_foreign_unit_to_dislodge_own_unit_prohibited() {
    let r = run("AUS: F Tri H, A Vie S A Ven-Tri; ITA: A Ven-Tri");
    assert_eq!(r.outcome("Ven"), Fail);
    assert!(r.rules("Ven").contains(&"R-SLF-02".to_string()));
    assert!(!r.dislodged("Tri"));
}

#[test]
fn datc_6d13_supporting_a_foreign_unit_to_dislodge_a_returning_own_unit_prohibited() {
    let r = run("AUS: F Tri-Adr, A Vie S A Ven-Tri; ITA: A Ven-Tri, F Apu-Adr");
    for u in ["Tri", "Ven", "Apu"] {
        assert_eq!(r.outcome(u), Fail, "{u}");
    }
    assert!(!r.dislodged("Tri"));
}

#[test]
fn datc_6d14_supporting_a_foreign_unit_is_not_enough_to_prevent_dislodgement() {
    let r = run("AUS: F Tri H, A Vie S A Ven-Tri; ITA: A Ven-Tri, A Tyr S A Ven-Tri, F Adr S A Ven-Tri");
    assert_eq!(r.outcome("Ven"), Success);
    assert!(r.dislodged("Tri"));
}

#[test]
fn datc_6d15_defender_cannot_cut_support_for_attack_on_itself() {
    let r = run("RUS: F Con S F Bla-Ank, F Bla-Ank; TUR: F Ank-Con");
    assert_eq!(r.outcome("Con"), Success, "R-CUT-01");
    assert_eq!(r.outcome("Bla"), Success);
    assert!(r.dislodged("Ank"));
}

#[test]
fn datc_6d16_convoying_a_unit_dislodging_a_unit_of_same_power_is_allowed() {
    let r = run("ENG: A Lon H, F Nth C A Bel-Lon; FRA: F Eng S A Bel-Lon, A Bel-Lon");
    assert_eq!(r.outcome("Bel"), Success);
    assert!(r.dislodged("Lon"));
    assert_eq!(r.at("Lon"), Some((FRA, UnitType::A)));
}

#[test]
fn datc_6d17_dislodgement_cuts_supports() {
    let r = run("RUS: F Con S F Bla-Ank, F Bla-Ank; TUR: F Ank-Con, A Smy S F Ank-Con, A Arm-Ank");
    assert!(r.dislodged("Con"));
    assert_eq!(r.outcome("Con"), Cut);
    assert!(r.rules("Con").contains(&"R-CUT-02".to_string()));
    assert_eq!((r.outcome("Bla"), r.outcome("Arm")), (Fail, Fail));
    assert_eq!(r.at("Con"), Some((TUR, UnitType::F)));
    assert_eq!(r.at("Ank"), None);
}

#[test]
fn datc_6d18_a_surviving_unit_will_sustain_support() {
    let r = run("RUS: F Con S F Bla-Ank, F Bla-Ank, A Bul S F Con; TUR: F Ank-Con, A Smy S F Ank-Con, A Arm-Ank");
    assert!(!r.dislodged("Con"));
    assert_eq!(r.outcome("Bla"), Success);
    assert!(r.dislodged("Ank"));
}

#[test]
fn datc_6d19_even_when_surviving_is_in_alternative_way() {
    let r = run("RUS: F Con S F Bla-Ank, F Bla-Ank, A Smy S F Ank-Con; TUR: F Ank-Con");
    assert_eq!(r.outcome("Ank"), Fail, "R-SLF-02");
    assert!(!r.dislodged("Con"));
    assert_eq!(r.outcome("Bla"), Success);
    assert!(r.dislodged("Ank"));
}

#[test]
fn datc_6d20_unit_cannot_cut_support_of_its_own_country() {
    let r = run("ENG: F Lon S F Nth-Eng, F Nth-Eng, A Yor-Lon; FRA: F Eng H");
    assert_eq!(r.outcome("Lon"), Success, "R-CUT-04");
    assert!(r.dislodged("Eng"));
}

#[test]
fn datc_6d21_dislodging_does_not_cancel_a_support_cut() {
    let r = run("AUS: F Tri H; ITA: A Ven-Tri, A Tyr S A Ven-Tri; GER: A Mun-Tyr; RUS: A Sil-Mun, A Ber S A Sil-Mun");
    assert!(r.dislodged("Mun"));
    assert_eq!(r.outcome("Tyr"), Cut, "R-CUT-03");
    assert!(!r.dislodged("Tri"));
}

#[test]
fn datc_6d22_impossible_fleet_move_cannot_be_supported() {
    let r = run("GER: F Kie-Mun, A Bur S F Kie-Mun; RUS: A Mun-Kie, A Ber S A Mun-Kie");
    assert!(!r.valid("Kie"));
    assert_eq!(r.outcome("Bur"), Void);
    assert_eq!(r.outcome("Mun"), Success);
    assert!(r.dislodged("Kie"));
}

#[test]
fn datc_6d23_impossible_coast_move_cannot_be_supported() {
    let r = run("ITA: F GoL-Spa/SC, F Wes S F GoL-Spa/SC; FRA: F Spa/NC-GoL, F Mar S F Spa/NC-GoL");
    assert!(!r.valid("Spa"));
    assert_eq!(r.outcome("Mar"), Void);
    assert_eq!(r.outcome("GoL"), Success);
    assert!(r.dislodged("Spa"));
}

#[test]
fn datc_6d24_impossible_army_move_cannot_be_supported() {
    let r = run("FRA: A Mar-GoL, F Spa/SC S A Mar-GoL; ITA: F GoL H; TUR: F Tyn S F Wes-GoL, F Wes-GoL");
    assert!(!r.valid("Mar"));
    assert_eq!(r.outcome("Spa"), Void);
    assert_eq!(r.outcome("Wes"), Success);
    assert!(r.dislodged("GoL"));
}

#[test]
fn datc_6d25_failing_hold_support_can_be_supported() {
    let r = run("GER: A Ber S A Pru, F Kie S A Ber; RUS: F Bal S A Pru-Ber, A Pru-Ber");
    assert_eq!(r.outcome("Ber"), Void);
    assert_eq!(r.outcome("Pru"), Fail);
    assert!(!r.dislodged("Ber"));
}

#[test]
fn datc_6d26_failing_move_support_can_be_supported() {
    let r = run("GER: A Ber S A Pru-Sil, F Kie S A Ber; RUS: F Bal S A Pru-Ber, A Pru-Ber");
    assert_eq!(r.outcome("Ber"), Void);
    assert_eq!(r.outcome("Pru"), Fail);
    assert!(!r.dislodged("Ber"));
}

#[test]
fn datc_6d27_failing_convoy_can_be_supported() {
    let r = run("ENG: F Swe-Bal, F Den S F Swe-Bal; GER: A Ber H; RUS: F Bal C A Ber-Lvn, F Pru S F Bal");
    assert_eq!(r.outcome("Bal"), Void, "R-CVY-04, PR-12");
    assert_eq!(r.outcome("Swe"), Fail);
    assert!(!r.dislodged("Bal"));
}

#[test]
fn datc_6d28_impossible_move_and_support() {
    let r = run("AUS: A Bud S F Rum; RUS: F Rum-Hol; TUR: F Bla-Rum, A Bul S F Bla-Rum");
    assert!(!r.valid("Rum"));
    assert_eq!(r.outcome("Bud"), Success, "PR-13");
    assert_eq!(r.outcome("Bla"), Fail);
    assert!(!r.dislodged("Rum"));
}

#[test]
fn datc_6d29_move_to_impossible_coast_and_support() {
    let r = run("AUS: A Bud S F Rum; RUS: F Rum-Bul/SC; TUR: F Bla-Rum, A Bul S F Bla-Rum");
    assert!(!r.valid("Rum"));
    assert!(r.rules("Rum").contains(&"PR-04".to_string()));
    assert_eq!(r.outcome("Bla"), Fail);
    assert!(!r.dislodged("Rum"));
}

#[test]
fn datc_6d30_move_without_coast_and_support() {
    let r = run("ITA: F Aeg S F Con; RUS: F Con-Bul; TUR: F Bla-Con, A Bul S F Bla-Con");
    assert!(!r.valid("Con"));
    assert!(r.rules("Con").contains(&"R-MAP-07".to_string()));
    assert_eq!(r.outcome("Bla"), Fail);
    assert!(!r.dislodged("Con"));
}

// skipped: 6.D.31 (a tricky impossible support). DATC wants `F Bla S A Rum-Arm` to be illegal
// because the only convoy route runs through the supporter itself. Our validation (adjudication.md
// § 2.2, R-SUP-02) only asks whether the supporter could move to Armenia, so the support is legal
// (and gives nothing, as the army has no route). Needs a ruling.

#[test]
fn datc_6d32_a_missing_fleet() {
    // No fleet in the North Sea: A Yor-Hol is impossible, hence illegal, and the army holds.
    let r = run("ENG: F Edi S A Lvp-Yor, A Lvp-Yor; FRA: F Lon S A Yor; GER: A Yor-Hol");
    assert!(!r.valid("Yor"), "R-CVY-03");
    assert_eq!(r.outcome("Lon"), Success);
    assert_eq!(r.outcome("Lvp"), Fail);
    assert!(!r.dislodged("Yor"));
}

#[test]
fn datc_6d33_unwanted_support_allowed() {
    let r = run("AUS: A Ser-Bud, A Vie-Bud; RUS: A Gal S A Ser-Bud; TUR: A Bul-Ser");
    assert_eq!(r.outcome("Ser"), Success);
    assert_eq!(r.outcome("Vie"), Fail);
    assert_eq!(r.outcome("Bul"), Success);
    assert_eq!(r.at("Ser"), Some((TUR, UnitType::A)));
}

#[test]
fn datc_6d34_support_targeting_own_area_not_allowed() {
    let r = run("GER: A Ber-Pru, A Sil S A Ber-Pru, F Bal S A Ber-Pru; \
         ITA: A Pru S Lvn-Pru; RUS: A War S A Lvn-Pru, A Lvn-Pru");
    assert!(!r.valid("Pru"), "R-SUP-02");
    assert_eq!(r.outcome("Ber"), Success);
    assert_eq!(r.outcome("Lvn"), Fail);
    assert!(r.dislodged("Pru"));
    assert_eq!(r.at("Pru"), Some((GER, UnitType::A)));
}

// ----- 6.E. Head-to-head battles and beleaguered garrison ----------------------------------------

#[test]
fn datc_6e01_dislodged_unit_has_no_effect_on_attackers_area() {
    let r = run("GER: A Ber-Pru, F Kie-Ber, A Sil S A Ber-Pru; RUS: A Pru-Ber");
    assert_eq!(r.outcome("Ber"), Success);
    assert!(r.dislodged("Pru"));
    assert_eq!(r.outcome("Kie"), Success, "R-STD-07");
    assert_eq!(r.at("Ber"), Some((GER, UnitType::F)));
}

#[test]
fn datc_6e02_no_self_dislodgement_in_head_to_head_battle() {
    let r = run("GER: A Ber-Kie, F Kie-Ber, A Mun S A Ber-Kie");
    assert_eq!((r.outcome("Ber"), r.outcome("Kie")), (Fail, Fail));
    assert!(!r.dislodged("Kie"));
    assert_eq!(r.at("Kie"), Some((GER, UnitType::F)));
}

#[test]
fn datc_6e03_no_help_in_dislodging_own_unit() {
    let r = run("GER: A Ber-Kie, A Mun S F Kie-Ber; ENG: F Kie-Ber");
    assert_eq!((r.outcome("Ber"), r.outcome("Kie")), (Fail, Fail));
    assert!(!r.dislodged("Ber"));
}

#[test]
fn datc_6e04_non_dislodged_loser_still_has_effect() {
    let r = run(
        "GER: F Hol-Nth, F Hel S F Hol-Nth, F Ska S F Hol-Nth; FRA: F Nth-Hol, F Bel S F Nth-Hol; \
         ENG: F Edi S F Nrg-Nth, F Yor S F Nrg-Nth, F Nrg-Nth; AUS: A Kie S A Ruh-Hol, A Ruh-Hol",
    );
    for u in ["Hol", "Nth", "Nrg", "Ruh"] {
        assert_eq!(r.outcome(u), Fail, "{u}");
    }
    assert!(!r.dislodged("Nth") && !r.dislodged("Hol"));
}

#[test]
fn datc_6e05_loser_dislodged_by_another_army_still_has_effect() {
    let r = run(
        "GER: F Hol-Nth, F Hel S F Hol-Nth, F Ska S F Hol-Nth; FRA: F Nth-Hol, F Bel S F Nth-Hol; \
         ENG: F Edi S F Nrg-Nth, F Yor S F Nrg-Nth, F Nrg-Nth, F Lon S F Nrg-Nth; AUS: A Kie S A Ruh-Hol, A Ruh-Hol",
    );
    assert_eq!(r.outcome("Nrg"), Success);
    assert!(r.dislodged("Nth"));
    assert_eq!((r.outcome("Hol"), r.outcome("Ruh")), (Fail, Fail));
    assert!(!r.dislodged("Hol"));
    assert_eq!(r.at("Nth"), Some((ENG, UnitType::F)));
}

#[test]
fn datc_6e06_not_dislodge_because_of_own_support_still_has_effect() {
    let r = run(
        "GER: F Hol-Nth, F Hel S F Hol-Nth; FRA: F Nth-Hol, F Bel S F Nth-Hol, F Eng S F Hol-Nth; \
         AUS: A Kie S A Ruh-Hol, A Ruh-Hol",
    );
    for u in ["Hol", "Nth", "Ruh"] {
        assert_eq!(r.outcome(u), Fail, "{u}");
    }
    assert!(!r.dislodged("Nth") && !r.dislodged("Hol"));
}

#[test]
fn datc_6e07_no_self_dislodgement_with_beleaguered_garrison() {
    let r =
        run("ENG: F Nth H, F Yor S F Nwy-Nth; GER: F Hol S F Hel-Nth, F Hel-Nth; RUS: F Ska S F Nwy-Nth, F Nwy-Nth");
    assert_eq!((r.outcome("Hel"), r.outcome("Nwy")), (Fail, Fail));
    assert!(!r.dislodged("Nth"));
}

#[test]
fn datc_6e08_no_self_dislodgement_with_beleaguered_garrison_and_head_to_head_battle() {
    let r =
        run("ENG: F Nth-Nwy, F Yor S F Nwy-Nth; GER: F Hol S F Hel-Nth, F Hel-Nth; RUS: F Ska S F Nwy-Nth, F Nwy-Nth");
    for u in ["Nth", "Hel", "Nwy"] {
        assert_eq!(r.outcome(u), Fail, "{u}");
    }
    assert!(!r.dislodged("Nth"));
}

#[test]
fn datc_6e09_almost_self_dislodgement_with_beleaguered_garrison() {
    let r =
        run("ENG: F Nth-Nrg, F Yor S F Nwy-Nth; GER: F Hol S F Hel-Nth, F Hel-Nth; RUS: F Ska S F Nwy-Nth, F Nwy-Nth");
    assert_eq!((r.outcome("Nth"), r.outcome("Nwy")), (Success, Success));
    assert_eq!(r.outcome("Hel"), Fail);
    assert_eq!(r.at("Nth"), Some((RUS, UnitType::F)));
}

#[test]
fn datc_6e10_almost_circular_movement_with_no_self_dislodgement_with_beleaguered_garrison() {
    let r = run(
        "ENG: F Nth-Den, F Yor S F Nwy-Nth; GER: F Hol S F Hel-Nth, F Hel-Nth, F Den-Hel; \
         RUS: F Ska S F Nwy-Nth, F Nwy-Nth",
    );
    for u in ["Nth", "Hel", "Den", "Nwy"] {
        assert_eq!(r.outcome(u), Fail, "{u}");
    }
    assert!(!r.dislodged("Nth"));
}

#[test]
fn datc_6e11_no_self_dislodgement_with_beleaguered_garrison_unit_swap_with_adjacent_convoying_and_two_coasts() {
    let r = run("FRA: A Spa-Por via convoy, F Mid C A Spa-Por, F GoL S F Por-Spa/NC; \
         GER: A Mar S A Gas-Spa, A Gas-Spa; ITA: F Por-Spa/NC, F Wes S F Por-Spa/NC");
    assert_eq!((r.outcome("Spa"), r.outcome("Por")), (Success, Success));
    assert_eq!(r.outcome("Gas"), Fail);
    assert_eq!(r.at("Por"), Some((FRA, UnitType::A)));
    assert_eq!(r.at("Spa"), Some((ITA, UnitType::F)));
    assert_eq!(r.coast_at("Spa").as_deref(), Some("NC"));
}

#[test]
fn datc_6e12_support_on_attack_on_own_unit_can_be_used_for_other_means() {
    // The Austrian support does not help dislodge its own unit (R-SLF-02) but still adds to the
    // prevent strength of the Italian army (PR-02).
    let r = run("AUS: A Bud-Rum, A Ser S A Vie-Bud; ITA: A Vie-Bud; RUS: A Gal-Bud, A Rum S A Gal-Bud");
    for u in ["Bud", "Vie", "Gal"] {
        assert_eq!(r.outcome(u), Fail, "{u}");
    }
    assert!(!r.dislodged("Bud"));
}

#[test]
fn datc_6e13_three_way_beleaguered_garrison() {
    let r = run(
        "ENG: F Edi S F Yor-Nth, F Yor-Nth; FRA: F Bel-Nth, F Eng S F Bel-Nth; GER: F Nth H; \
         RUS: F Nrg-Nth, F Nwy S F Nrg-Nth",
    );
    for u in ["Yor", "Bel", "Nrg"] {
        assert_eq!(r.outcome(u), Fail, "{u}");
    }
    assert!(!r.dislodged("Nth"));
}

#[test]
fn datc_6e14_illegal_head_to_head_battle_can_still_defend() {
    let r = run("ENG: A Lvp-Edi; RUS: F Edi-Lvp");
    assert!(!r.valid("Edi"));
    assert_eq!(r.outcome("Lvp"), Fail);
    assert_eq!(r.at("Edi"), Some((RUS, UnitType::F)));
}

#[test]
fn datc_6e15_the_friendly_head_to_head_battle() {
    let r = run(
        "ENG: F Hol S A Ruh-Kie, A Ruh-Kie; FRA: A Kie-Ber, A Mun S A Kie-Ber, A Sil S A Kie-Ber; \
         GER: A Ber-Kie, F Den S A Ber-Kie, F Hel S A Ber-Kie; RUS: F Bal S A Pru-Ber, A Pru-Ber",
    );
    for u in ["Ruh", "Kie", "Ber", "Pru"] {
        assert_eq!(r.outcome(u), Fail, "{u}");
    }
    assert!(!r.dislodged("Kie") && !r.dislodged("Ber"));
}

// ----- 6.F. Convoys ------------------------------------------------------------------------------

#[test]
fn datc_6f01_no_convoy_in_coastal_areas() {
    let r = run("TUR: A Gre-Sev, F Aeg C A Gre-Sev, F Con C A Gre-Sev, F Bla C A Gre-Sev");
    assert!(!r.valid("Con"), "R-CVY-01");
    assert_ne!(r.outcome("Gre"), Success);
    assert_eq!(r.at("Gre"), Some((TUR, UnitType::A)));
    assert_eq!(r.at("Sev"), None);
}

#[test]
fn datc_6f02_an_army_being_convoyed_can_bounce_as_normal() {
    let r = run("ENG: F Eng C A Lon-Bre, A Lon-Bre; FRA: A Par-Bre");
    assert_eq!((r.outcome("Lon"), r.outcome("Par")), (Fail, Fail));
    assert_eq!(r.at("Lon"), Some((ENG, UnitType::A)));
    assert_eq!(r.standoffs(), ["Bre"]);
}

#[test]
fn datc_6f03_an_army_being_convoyed_can_receive_support() {
    let r = run("ENG: F Eng C A Lon-Bre, A Lon-Bre, F Mid S A Lon-Bre; FRA: A Par-Bre");
    assert_eq!((r.outcome("Lon"), r.outcome("Par")), (Success, Fail));
    assert_eq!(r.at("Bre"), Some((ENG, UnitType::A)));
    assert_eq!(r.at("Par"), Some((FRA, UnitType::A)));
}

#[test]
fn datc_6f04_an_attacked_convoy_is_not_disrupted() {
    let r = run("ENG: F Nth C A Lon-Hol, A Lon-Hol; GER: F Ska-Nth");
    assert_eq!(r.outcome("Lon"), Success, "R-CVY-05");
    assert_eq!(r.at("Hol"), Some((ENG, UnitType::A)));
}

#[test]
fn datc_6f05_a_beleaguered_convoy_is_not_disrupted() {
    let r =
        run("ENG: F Nth C A Lon-Hol, A Lon-Hol; FRA: F Eng-Nth, F Bel S F Eng-Nth; GER: F Ska-Nth, F Den S F Ska-Nth");
    assert!(!r.dislodged("Nth"));
    assert_eq!(r.outcome("Lon"), Success);
    assert_eq!(r.at("Hol"), Some((ENG, UnitType::A)));
}

#[test]
fn datc_6f06_dislodged_convoy_does_not_cut_support() {
    let r = run("ENG: F Nth C A Lon-Hol, A Lon-Hol; \
         GER: A Hol S A Bel, A Bel S A Hol, F Hel S F Ska-Nth, F Ska-Nth; FRA: A Pic-Bel, A Bur S A Pic-Bel");
    assert!(r.dislodged("Nth"));
    assert_eq!(r.outcome("Lon"), Fail);
    assert_eq!(r.outcome("Hol"), Success, "not cut by a convoy that fails (R-CVY-05)");
    assert_eq!(r.outcome("Pic"), Fail);
    assert!(!r.dislodged("Bel"));
}

#[test]
fn datc_6f07_dislodged_convoy_does_not_cause_contested_area() {
    let r = run("ENG: F Nth C A Lon-Hol, A Lon-Hol; GER: F Hel S F Ska-Nth, F Ska-Nth");
    assert!(r.dislodged("Nth"));
    assert!(r.standoffs().is_empty());
    assert!(r.retreat_options("Nth").contains(&"Hol".to_string()));
}

#[test]
fn datc_6f08_dislodged_convoy_does_not_cause_a_bounce() {
    let r = run("ENG: F Nth C A Lon-Hol, A Lon-Hol; GER: F Hel S F Ska-Nth, F Ska-Nth, A Bel-Hol");
    assert!(r.dislodged("Nth"));
    assert_eq!(r.outcome("Bel"), Success);
    assert_eq!(r.at("Hol"), Some((GER, UnitType::A)));
}

#[test]
fn datc_6f09_dislodge_of_multi_route_convoy() {
    // One intact route is enough (R-CVY-08, 2000 rules).
    let r = run("ENG: F Eng C A Lon-Bel, F Nth C A Lon-Bel, A Lon-Bel; FRA: F Bre S F Mid-Eng, F Mid-Eng");
    assert!(r.dislodged("Eng"));
    assert_eq!(r.outcome("Lon"), Success);
    assert_eq!(r.at("Bel"), Some((ENG, UnitType::A)));
}

#[test]
fn datc_6f10_dislodge_of_multi_route_convoy_with_foreign_fleet() {
    let r = run("ENG: F Nth C A Lon-Bel, A Lon-Bel; GER: F Eng C A Lon-Bel; FRA: F Bre S F Mid-Eng, F Mid-Eng");
    assert!(r.dislodged("Eng"));
    assert_eq!(r.outcome("Lon"), Success, "R-CVY-08");
    assert_eq!(r.at("Bel"), Some((ENG, UnitType::A)));
}

#[test]
fn datc_6f11_dislodge_of_multi_route_convoy_with_only_foreign_fleets() {
    let r = run("ENG: A Lon-Bel; GER: F Eng C A Lon-Bel; RUS: F Nth C A Lon-Bel; FRA: F Bre S F Mid-Eng, F Mid-Eng");
    assert!(r.dislodged("Eng"));
    assert_eq!(r.outcome("Lon"), Success, "R-CVY-08");
    assert_eq!(r.at("Bel"), Some((ENG, UnitType::A)));
}

#[test]
fn datc_6f12_dislodged_convoying_fleet_not_on_route() {
    let r = run("ENG: F Eng C A Lon-Bel, A Lon-Bel, F Iri C A Lon-Bel; FRA: F NAt S F Mid-Iri, F Mid-Iri");
    assert!(r.dislodged("Iri"));
    assert_eq!(r.outcome("Lon"), Success);
    assert_eq!(r.at("Bel"), Some((ENG, UnitType::A)));
}

#[test]
fn datc_6f13_the_unwanted_alternative() {
    let r = run("ENG: A Lon-Bel, F Nth C A Lon-Bel; FRA: F Eng C A Lon-Bel; GER: F Hol S F Den-Nth, F Den-Nth");
    assert!(r.dislodged("Nth"));
    assert_eq!(r.outcome("Lon"), Success, "R-CVY-08");
    assert_eq!(r.at("Bel"), Some((ENG, UnitType::A)));
}

#[test]
fn datc_6f14_simple_convoy_paradox() {
    // The convoyed army does not cut the support against its convoy (R-CVY-09).
    let r = run("ENG: F Lon S F Wal-Eng, F Wal-Eng; FRA: A Bre-Lon, F Eng C A Bre-Lon");
    assert_eq!(r.outcome("Lon"), Success, "R-CVY-09");
    assert_eq!(r.outcome("Wal"), Success);
    assert!(r.dislodged("Eng"));
    assert_eq!(r.outcome("Bre"), Fail);
    assert!(!r.dislodged("Lon"));
}

#[test]
fn datc_6f15_simple_convoy_paradox_with_additional_convoy() {
    let r = run("ENG: F Lon S F Wal-Eng, F Wal-Eng; FRA: A Bre-Lon, F Eng C A Bre-Lon; \
         ITA: F Iri C A NAf-Wal, F Mid C A NAf-Wal, A NAf-Wal");
    assert_eq!(r.outcome("Wal"), Success);
    assert!(r.dislodged("Eng"));
    assert_eq!(r.outcome("NAf"), Success);
    assert_eq!(r.at("Wal"), Some((ITA, UnitType::A)));
    assert_eq!(r.at("Bre"), Some((FRA, UnitType::A)));
}

// 6.F.16 (Pandin's paradox): see pr03_pandins_paradox in rulebook.rs.

#[test]
fn datc_6f17_pandins_extended_paradox() {
    // No consistent resolution once a dislodged supporter loses its support (R-CUT-02): by the
    // Szykman rule the convoyed army does not move (PR-03).
    let r = run(
        "ENG: F Lon S F Wal-Eng, F Wal-Eng; FRA: A Bre-Lon, F Eng C A Bre-Lon, F Yor S A Bre-Lon; \
         GER: F Nth S F Bel-Eng, F Bel-Eng",
    );
    for u in ["Bre", "Wal", "Bel"] {
        assert_eq!(r.outcome(u), Fail, "{u}");
    }
    assert!(!r.dislodged("Lon") && !r.dislodged("Eng"));
    assert!(r.rules("Bre").contains(&"PR-03".to_string()));
}

#[test]
fn datc_6f18_betrayal_paradox() {
    // Not settled by R-CVY-09/10 (the support is for the convoying fleet, not an attack on it):
    // Szykman rule (PR-03).
    let r = run(
        "ENG: F Nth C A Lon-Bel, A Lon-Bel, F Eng S A Lon-Bel; FRA: F Bel S F Nth; \
         GER: F Hel S F Ska-Nth, F Ska-Nth",
    );
    assert_eq!(r.outcome("Lon"), Fail);
    assert!(r.rules("Lon").contains(&"PR-03".to_string()));
    assert_eq!(r.outcome("Bel"), Success);
    assert_eq!(r.outcome("Ska"), Fail);
    assert!(!r.dislodged("Nth") && !r.dislodged("Bel"));
}

#[test]
fn datc_6f19_multi_route_convoy_disruption_paradox() {
    // F Tyn is not necessary (route via Ion), so the support in Naples is cut (R-CVY-10).
    let r = run("FRA: A Tun-Nap, F Tyn C A Tun-Nap, F Ion C A Tun-Nap; ITA: F Nap S F Rom-Tyn, F Rom-Tyn");
    assert_eq!(r.outcome("Nap"), Cut);
    assert!(r.rules("Nap").contains(&"R-CVY-10".to_string()));
    assert_eq!((r.outcome("Rom"), r.outcome("Tun")), (Fail, Fail));
    assert!(!r.dislodged("Tyn"));
}

#[test]
fn datc_6f20_unwanted_multi_route_convoy_paradox() {
    let r = run(
        "FRA: A Tun-Nap, F Tyn C A Tun-Nap; ITA: F Nap S F Ion, F Ion C A Tun-Nap; \
         TUR: F Aeg S F Eas-Ion, F Eas-Ion",
    );
    assert_eq!(r.outcome("Nap"), Cut);
    assert_eq!(r.outcome("Eas"), Success);
    assert!(r.dislodged("Ion"));
    assert_eq!(r.outcome("Tun"), Fail);
}

#[test]
fn datc_6f21_dads_army_convoy() {
    let r = run(
        "RUS: A Edi S A Nwy-Cly, F Nrg C A Nwy-Cly, A Nwy-Cly; FRA: F Iri S F Mid-NAt, F Mid-NAt; \
         ENG: A Lvp-Cly via convoy, F NAt C A Lvp-Cly, F Cly S F NAt",
    );
    assert_eq!(r.outcome("Cly"), Cut);
    assert_eq!(r.outcome("Mid"), Success);
    assert!(r.dislodged("NAt"));
    assert_eq!(r.outcome("Lvp"), Fail);
    assert_eq!(r.outcome("Nwy"), Success);
    assert!(r.dislodged("Cly"));
}

// 6.F.22 (second order paradox with two resolutions): see
// pr03_second_order_paradox_with_two_resolutions in rulebook.rs.

#[test]
fn datc_6f23_second_order_paradox_with_two_exclusive_convoys() {
    // Szykman rule (PR-03): both convoyed armies stay, the supports are not cut, no fleet moves.
    let r = run("ENG: F Edi-Nth, F Yor S F Edi-Nth; FRA: A Bre-Lon, F Eng C A Bre-Lon; \
         GER: F Bel S F Eng, F Lon S F Nth; ITA: F Mid-Eng, F Iri S F Mid-Eng; RUS: A Nwy-Bel, F Nth C A Nwy-Bel");
    for u in ["Edi", "Mid", "Bre", "Nwy"] {
        assert_eq!(r.outcome(u), Fail, "{u}");
    }
    assert_eq!((r.outcome("Bel"), r.outcome("Lon")), (Success, Success));
    assert!(!r.dislodged("Eng") && !r.dislodged("Nth"));
}

#[test]
fn datc_6f24_second_order_paradox_with_no_resolution() {
    // Szykman rule (PR-03): the convoyed armies stay and the supports are not cut.
    let r = run("ENG: F Edi-Nth, F Lon S F Edi-Nth, F Iri-Eng, F Mid S F Iri-Eng; \
         FRA: A Bre-Lon, F Eng C A Bre-Lon, F Bel S F Eng; RUS: A Nwy-Bel, F Nth C A Nwy-Bel");
    assert_eq!((r.outcome("Bre"), r.outcome("Nwy")), (Fail, Fail));
    assert_eq!((r.outcome("Lon"), r.outcome("Bel")), (Success, Success));
    assert_eq!(r.outcome("Iri"), Fail);
    assert!(!r.dislodged("Eng"));
    assert_eq!(r.outcome("Edi"), Success);
    assert!(r.dislodged("Nth"));
}

#[test]
fn datc_6f25_cut_support_last() {
    let r = run(
        "GER: A Ruh-Bel, A Hol S Ruh-Bel, A Den-Nwy, F Ska C Den-Nwy, A Fin S Den-Nwy; \
         ENG: A Yor-Hol, F Nth C Yor-Hol, F Hel S Yor-Hol, A Bel H; \
         RUS: F Nrg-Nth, F Nwy S Nrg-Nth, F Swe-Ska",
    );
    assert_eq!((r.outcome("Swe"), r.outcome("Nrg")), (Fail, Fail));
    assert_eq!(r.outcome("Den"), Success);
    assert!(r.dislodged("Nwy"));
    assert_eq!(r.outcome("Yor"), Success);
    assert!(r.dislodged("Hol"));
    assert_eq!(r.outcome("Hol"), Cut);
    assert_eq!(r.outcome("Ruh"), Fail);
    assert!(!r.dislodged("Bel"));
}

// ----- 6.G. Convoying to adjacent provinces ------------------------------------------------------
//
// Land route vs convoy route follows R-CVY-11 / PR-07: an own convoying fleet or "via convoy"
// means the convoy is used, and there is no fallback to the land route.

#[test]
fn datc_6g01_two_units_can_swap_provinces_by_convoy() {
    let r = run("ENG: A Nwy-Swe, F Ska C A Nwy-Swe; RUS: A Swe-Nwy");
    assert_eq!((r.outcome("Nwy"), r.outcome("Swe")), (Success, Success));
    assert_eq!(r.at("Swe"), Some((ENG, UnitType::A)));
    assert_eq!(r.at("Nwy"), Some((RUS, UnitType::A)));
}

#[test]
fn datc_6g02_kidnapping_an_army() {
    let r = run("ENG: A Nwy-Swe; RUS: F Swe-Nwy; GER: F Ska C A Nwy-Swe");
    assert_eq!(r.outcome("Ska"), Void);
    assert!(r.rules("Ska").contains(&"R-CVY-11".to_string()));
    assert_eq!((r.outcome("Nwy"), r.outcome("Swe")), (Fail, Fail));
    assert_eq!(r.at("Nwy"), Some((ENG, UnitType::A)));
}

#[test]
fn datc_6g03_an_unwanted_disrupted_convoy_to_adjacent_province() {
    let r = run("FRA: F Bre-Eng, A Pic-Bel, A Bur S A Pic-Bel, F Mid S F Bre-Eng; ENG: F Eng C A Pic-Bel");
    assert!(r.dislodged("Eng"));
    assert_eq!(r.outcome("Pic"), Success);
    assert_eq!(r.at("Bel"), Some((FRA, UnitType::A)));
}

#[test]
fn datc_6g04_an_unwanted_disrupted_convoy_to_adjacent_province_and_opposite_move() {
    let r = run("FRA: F Bre-Eng, A Pic-Bel, A Bur S A Pic-Bel, F Mid S F Bre-Eng; ENG: F Eng C A Pic-Bel, A Bel-Pic");
    assert!(r.dislodged("Eng"));
    assert_eq!(r.outcome("Pic"), Success);
    assert_eq!(r.outcome("Bel"), Fail);
    assert!(r.dislodged("Bel"));
    assert_eq!(r.at("Bel"), Some((FRA, UnitType::A)));
}

#[test]
fn datc_6g05_swapping_with_multiple_fleets_with_one_own_fleet() {
    let r = run("ITA: A Rom-Apu, F Tyn C A Apu-Rom; TUR: A Apu-Rom, F Ion C A Apu-Rom");
    assert_eq!((r.outcome("Rom"), r.outcome("Apu")), (Success, Success));
    assert_eq!(r.at("Rom"), Some((TUR, UnitType::A)));
    assert_eq!(r.at("Apu"), Some((ITA, UnitType::A)));
}

#[test]
fn datc_6g06_swapping_with_unintended_intent() {
    // The own fleet in the Channel shows the intent to convoy (R-CVY-11); the route used is
    // the Russian one.
    let r = run(
        "ENG: A Lvp-Edi, F Eng C A Lvp-Edi; GER: A Edi-Lvp; FRA: F Iri H, F Nth H; \
         RUS: F Nrg C A Lvp-Edi, F NAt C A Lvp-Edi",
    );
    assert_eq!((r.outcome("Lvp"), r.outcome("Edi")), (Success, Success));
    assert_eq!(r.at("Edi"), Some((ENG, UnitType::A)));
    assert_eq!(r.at("Lvp"), Some((GER, UnitType::A)));
}

// skipped: 6.G.7 (swapping with illegal intent). DATC treats `F Bot C A Swe-Nwy` as illegal (the
// Gulf of Bothnia cannot be on any route) so it shows no intent and the armies bounce. Our
// validation (adjudication.md § 2.2) only checks that both ends are coastal, so the order is legal,
// the own fleet makes the army take the convoy (R-CVY-11) and the units swap. Needs a ruling.

#[test]
fn datc_6g08_explicit_convoy_that_isnt_there() {
    // "via convoy" forces the convoy (PR-07); there is no fallback to the land route.
    let r = run("FRA: A Bel-Hol via convoy; ENG: F Nth-Hel, A Hol-Kie");
    assert_eq!(r.outcome("Bel"), Fail);
    assert_eq!(r.at("Bel"), Some((FRA, UnitType::A)));
    assert_eq!(r.outcome("Nth"), Success);
    assert_eq!(r.outcome("Hol"), Success);
    assert_eq!(r.at("Hol"), None);
}

#[test]
fn datc_6g09_swapped_or_dislodged() {
    let r = run("ENG: A Nwy-Swe, F Ska C A Nwy-Swe, F Fin S A Nwy-Swe; RUS: A Swe-Nwy");
    assert_eq!((r.outcome("Nwy"), r.outcome("Swe")), (Success, Success));
    assert!(!r.dislodged("Swe"));
    assert_eq!(r.at("Nwy"), Some((RUS, UnitType::A)));
}

// 6.G.10, 6.G.14, 6.G.15: whether the unit dislodged by a convoyed attacker still bounces a third
// unit in the attacker's origin (DATC issue 4.A.7). adjudication.md § 3.2 zeroes the prevent
// strength only in a head-to-head battle (choice b, DATC's preference), but R-STD-07 read
// literally ("no effect on the province the attack came from") gives choice a. The tests check
// everything else and leave that move unasserted until a ruling exists.

#[test]
fn datc_6g10_swapped_or_an_head_to_head_battle() {
    let r = run(
        "ENG: A Nwy-Swe via convoy, F Den S A Nwy-Swe, F Fin S A Nwy-Swe; GER: F Ska C A Nwy-Swe; \
         RUS: A Swe-Nwy, F Bar S A Swe-Nwy; FRA: F Nrg-Nwy, F Nth S F Nrg-Nwy",
    );
    assert_eq!(r.outcome("Nwy"), Success);
    assert_eq!(r.outcome("Swe"), Fail);
    assert!(r.dislodged("Swe"));
    assert_eq!(r.at("Swe"), Some((ENG, UnitType::A)));
    // Unasserted (4.A.7): F Nrg-Nwy.
}

#[test]
fn datc_6g11_a_convoy_to_an_adjacent_province_with_a_paradox() {
    // Own fleet → convoy route, no fallback; then the simple convoy paradox (R-CVY-09).
    let r = run("ENG: F Nwy S F Nth-Ska, F Nth-Ska; RUS: A Swe-Nwy, F Ska C A Swe-Nwy, F Bar S A Swe-Nwy");
    assert_eq!(r.outcome("Nwy"), Success, "R-CVY-09");
    assert_eq!(r.outcome("Nth"), Success);
    assert!(r.dislodged("Ska"));
    assert_eq!(r.outcome("Swe"), Fail);
    assert!(!r.dislodged("Nwy"));
}

#[test]
fn datc_6g12_swapping_two_units_with_two_convoys() {
    let r = run("ENG: A Lvp-Edi via convoy, F NAt C A Lvp-Edi, F Nrg C A Lvp-Edi; \
         GER: A Edi-Lvp via convoy, F Nth C A Edi-Lvp, F Eng C A Edi-Lvp, F Iri C A Edi-Lvp");
    assert_eq!((r.outcome("Lvp"), r.outcome("Edi")), (Success, Success));
    assert_eq!(r.at("Edi"), Some((ENG, UnitType::A)));
    assert_eq!(r.at("Lvp"), Some((GER, UnitType::A)));
}

#[test]
fn datc_6g13_support_cut_on_attack_on_itself_via_convoy() {
    // The convoyed attack comes from Trieste, the province Venice supports into (R-CUT-01).
    let r = run("AUS: F Adr C A Tri-Ven, A Tri-Ven via convoy; ITA: A Ven S F Alb-Tri, F Alb-Tri");
    assert_eq!(r.outcome("Ven"), Success, "R-CUT-01");
    assert_eq!(r.outcome("Alb"), Success);
    assert!(r.dislodged("Tri"));
}

#[test]
fn datc_6g14_bounce_by_convoy_to_adjacent_province() {
    let r = run(
        "ENG: A Nwy-Swe, F Den S A Nwy-Swe, F Fin S A Nwy-Swe; FRA: F Nrg-Nwy, F Nth S F Nrg-Nwy; \
         GER: F Ska C A Swe-Nwy; RUS: A Swe-Nwy via convoy, F Bar S A Swe-Nwy",
    );
    assert_eq!(r.outcome("Nwy"), Success);
    assert_eq!(r.outcome("Swe"), Fail);
    assert!(r.dislodged("Swe"));
    // Unasserted (4.A.7): F Nrg-Nwy.
}

#[test]
fn datc_6g15_bounce_and_dislodge_with_double_convoy() {
    let r = run(
        "ENG: F Nth C A Lon-Bel, A Hol S A Lon-Bel, A Yor-Lon, A Lon-Bel via convoy; \
         FRA: F Eng C A Bel-Lon, A Bel-Lon via convoy",
    );
    assert_eq!(r.outcome("Lon"), Success);
    assert_eq!(r.outcome("Bel"), Fail);
    assert!(r.dislodged("Bel"));
    assert_eq!(r.at("Bel"), Some((ENG, UnitType::A)));
    // Unasserted (4.A.7): A Yor-Lon.
}

#[test]
fn datc_6g16_the_two_unit_in_one_area_bug_moving_by_convoy() {
    let r = run("ENG: A Nwy-Swe, A Den S A Nwy-Swe, F Bal S A Nwy-Swe, F Nth-Nwy; \
         RUS: A Swe-Nwy via convoy, F Ska C A Swe-Nwy, F Nrg S A Swe-Nwy");
    assert_eq!((r.outcome("Nwy"), r.outcome("Swe")), (Success, Success));
    assert_eq!(r.outcome("Nth"), Fail);
    assert_eq!(r.at("Nwy"), Some((RUS, UnitType::A)));
    assert_eq!(r.at("Nth"), Some((ENG, UnitType::F)));
}

#[test]
fn datc_6g17_the_two_unit_in_one_area_bug_moving_over_land() {
    let r = run(
        "ENG: A Nwy-Swe via convoy, A Den S A Nwy-Swe, F Bal S A Nwy-Swe, F Ska C A Nwy-Swe, F Nth-Nwy; \
         RUS: A Swe-Nwy, F Nrg S A Swe-Nwy",
    );
    assert_eq!((r.outcome("Nwy"), r.outcome("Swe")), (Success, Success));
    assert_eq!(r.outcome("Nth"), Fail);
    assert_eq!(r.at("Nwy"), Some((RUS, UnitType::A)));
    assert_eq!(r.at("Nth"), Some((ENG, UnitType::F)));
}

#[test]
fn datc_6g18_the_two_unit_in_one_area_bug_with_double_convoy() {
    let r = run(
        "ENG: F Nth C A Lon-Bel, A Hol S A Lon-Bel, A Yor-Lon, A Lon-Bel, A Ruh S A Lon-Bel; \
         FRA: F Eng C A Bel-Lon, A Bel-Lon, A Wal S A Bel-Lon",
    );
    assert_eq!((r.outcome("Lon"), r.outcome("Bel")), (Success, Success));
    assert_eq!(r.outcome("Yor"), Fail);
    assert_eq!(r.at("Lon"), Some((FRA, UnitType::A)));
    assert_eq!(r.at("Yor"), Some((ENG, UnitType::A)));
}

// skipped: 6.G.19 (swapping with intent of unnecessary convoy). DATC treats the French
// `F Wes C A Mar-Spa` as illegal (not necessary for any route), so there is no own-fleet intent
// and the armies bounce. Our validation accepts the order, R-CVY-11 then uses the convoy (via the
// Italian fleet in GoL) and the armies swap. Same open question as 6.G.7.

#[test]
fn datc_6g20_explicit_convoy_to_adjacent_province_disrupted() {
    // No fallback to the land route (PR-07).
    let r = run("FRA: F Bre-Eng, A Pic-Bel via convoy, A Bur S A Pic-Bel, F Mid S F Bre-Eng; ENG: F Eng C A Pic-Bel");
    assert!(r.dislodged("Eng"));
    assert_eq!(r.outcome("Pic"), Fail);
    assert_eq!(r.at("Pic"), Some((FRA, UnitType::A)));
    assert_eq!(r.at("Bel"), None);
}
