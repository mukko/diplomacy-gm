//! Core data model (docs/en/architecture.md § 4). Serialized as camelCase JSON.

use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Power {
    #[serde(rename = "AUS")]
    Austria,
    #[serde(rename = "ENG")]
    England,
    #[serde(rename = "FRA")]
    France,
    #[serde(rename = "GER")]
    Germany,
    #[serde(rename = "ITA")]
    Italy,
    #[serde(rename = "RUS")]
    Russia,
    #[serde(rename = "TUR")]
    Turkey,
}

impl Power {
    pub const ALL: [Power; 7] = [
        Power::Austria,
        Power::England,
        Power::France,
        Power::Germany,
        Power::Italy,
        Power::Russia,
        Power::Turkey,
    ];

    pub fn code(self) -> &'static str {
        match self {
            Power::Austria => "AUS",
            Power::England => "ENG",
            Power::France => "FRA",
            Power::Germany => "GER",
            Power::Italy => "ITA",
            Power::Russia => "RUS",
            Power::Turkey => "TUR",
        }
    }
}

impl FromStr for Power {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Power::ALL
            .into_iter()
            .find(|p| p.code().eq_ignore_ascii_case(s))
            .ok_or_else(|| format!("unknown power `{s}`"))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnitType {
    A,
    F,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Season {
    Spring,
    Fall,
    Winter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PhaseKind {
    Movement,
    Retreat,
    Adjustment,
}

/// Phase ID `<S|F|W><year><M|R|A>`, e.g. `S1901M` (R-TURN-01..06).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Phase {
    pub season: Season,
    pub year: u16,
    pub kind: PhaseKind,
}

impl Phase {
    /// First phase of the standard game (R-TURN-01).
    pub const START: Phase = Phase {
        season: Season::Spring,
        year: 1901,
        kind: PhaseKind::Movement,
    };

    fn order_key(&self) -> (u16, u8) {
        let step = match (self.season, self.kind) {
            (Season::Spring, PhaseKind::Movement) => 0,
            (Season::Spring, PhaseKind::Retreat) => 1,
            (Season::Fall, PhaseKind::Movement) => 2,
            (Season::Fall, PhaseKind::Retreat) => 3,
            _ => 4,
        };
        (self.year, step)
    }
}

impl PartialOrd for Phase {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Phase {
    fn cmp(&self, other: &Self) -> Ordering {
        self.order_key().cmp(&other.order_key())
    }
}

impl fmt::Display for Phase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self.season {
            Season::Spring => 'S',
            Season::Fall => 'F',
            Season::Winter => 'W',
        };
        let k = match self.kind {
            PhaseKind::Movement => 'M',
            PhaseKind::Retreat => 'R',
            PhaseKind::Adjustment => 'A',
        };
        write!(f, "{s}{}{k}", self.year)
    }
}

impl FromStr for Phase {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let err = || format!("invalid phase `{s}` (expected e.g. S1901M, F1901R, W1901A)");
        let bytes = s.as_bytes();
        if bytes.len() != 6 {
            return Err(err());
        }
        let season = match bytes[0].to_ascii_uppercase() {
            b'S' => Season::Spring,
            b'F' => Season::Fall,
            b'W' => Season::Winter,
            _ => return Err(err()),
        };
        let kind = match bytes[5].to_ascii_uppercase() {
            b'M' => PhaseKind::Movement,
            b'R' => PhaseKind::Retreat,
            b'A' => PhaseKind::Adjustment,
            _ => return Err(err()),
        };
        let year: u16 = s[1..5].parse().map_err(|_| err())?;
        let valid = matches!(
            (season, kind),
            (Season::Spring | Season::Fall, PhaseKind::Movement | PhaseKind::Retreat)
                | (Season::Winter, PhaseKind::Adjustment)
        );
        if !valid {
            return Err(err());
        }
        Ok(Phase { season, year, kind })
    }
}

impl Serialize for Phase {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Phase {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Unit {
    pub power: Power,
    #[serde(rename = "type")]
    pub unit_type: UnitType,
    pub location: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub coast: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DislodgedUnit {
    pub power: Power,
    #[serde(rename = "type")]
    pub unit_type: UnitType,
    pub location: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub coast: Option<String>,
    pub attacker_from: String,
    pub retreat_options: Vec<String>,
}

/// GM-declared civil disorder (PR-16).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CivilDisorder {
    pub power: Power,
    pub since: Phase,
}

/// Board position at the start of a phase.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameState {
    pub phase: Phase,
    pub units: Vec<Unit>,
    /// SC province ID → owner (`null` = unowned).
    pub supply_centers: BTreeMap<String, Option<Power>>,
    #[serde(default)]
    pub dislodged: Vec<DislodgedUnit>,
    #[serde(default)]
    pub standoff_provinces: Vec<String>,
    #[serde(default)]
    pub civil_disorder: Vec<CivilDisorder>,
    #[serde(default)]
    pub winner: Option<Power>,
}

impl GameState {
    /// Standard starting position, Spring 1901 (rules.md § 15).
    pub fn standard_start() -> Self {
        use Power::*;
        use UnitType::*;
        let units = [
            (Austria, A, "Vie", None),
            (Austria, A, "Bud", None),
            (Austria, F, "Tri", None),
            (England, F, "Lon", None),
            (England, F, "Edi", None),
            (England, A, "Lvp", None),
            (France, A, "Par", None),
            (France, A, "Mar", None),
            (France, F, "Bre", None),
            (Germany, A, "Ber", None),
            (Germany, A, "Mun", None),
            (Germany, F, "Kie", None),
            (Italy, A, "Rom", None),
            (Italy, A, "Ven", None),
            (Italy, F, "Nap", None),
            (Russia, A, "Mos", None),
            (Russia, F, "Sev", None),
            (Russia, A, "War", None),
            (Russia, F, "StP", Some("SC")),
            (Turkey, F, "Ank", None),
            (Turkey, A, "Con", None),
            (Turkey, A, "Smy", None),
        ]
        .into_iter()
        .map(|(power, unit_type, location, coast)| Unit {
            power,
            unit_type,
            location: location.to_string(),
            coast: coast.map(str::to_string),
        })
        .collect();

        let supply_centers = crate::map::standard()
            .supply_centers()
            .map(|p| (p.id.clone(), p.home))
            .collect();

        GameState {
            phase: Phase::START,
            units,
            supply_centers,
            dislodged: Vec::new(),
            standoff_provinces: Vec::new(),
            civil_disorder: Vec::new(),
            winner: None,
        }
    }
}

/// A structured order as sent by the GUI (architecture.md § 4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Order {
    pub power: Power,
    /// Province the ordered unit is in (or, for builds, the province to build in).
    pub unit: String,
    #[serde(flatten)]
    pub kind: OrderKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum OrderKind {
    Hold,
    Move {
        /// Target location; may carry a coast, e.g. `Bul/EC`.
        to: String,
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        via_convoy: bool,
    },
    Support {
        target: String,
        /// Omitted for hold support.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        to: Option<String>,
    },
    Convoy {
        target: String,
        to: String,
    },
    Retreat {
        to: String,
    },
    Disband,
    Build {
        unit_type: UnitType,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        coast: Option<String>,
    },
    Waive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Outcome {
    Success,
    Fail,
    Void,
    Cut,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrderResult {
    pub order: Order,
    pub valid: bool,
    pub outcome: Outcome,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(default)]
    pub rules: Vec<String>,
}

/// One step of the adjudication reasoning, replayed by the GUI.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraceStep {
    pub step: String,
    #[serde(default)]
    pub rules: Vec<String>,
    /// Step-specific fields (`order`, `outcome`, `by`, strengths, …).
    #[serde(flatten)]
    pub details: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Resolution {
    pub phase: Phase,
    pub orders: Vec<OrderResult>,
    #[serde(default)]
    pub trace: Vec<TraceStep>,
    pub next_state: GameState,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn phase_round_trip_and_order() {
        for s in ["S1901M", "S1901R", "F1901M", "F1901R", "W1901A"] {
            assert_eq!(s.parse::<Phase>().unwrap().to_string(), s);
        }
        for bad in ["W1901M", "S1901A", "X1901M", "S19O1M", "S1901"] {
            assert!(bad.parse::<Phase>().is_err(), "{bad}");
        }
        let p = |s: &str| s.parse::<Phase>().unwrap();
        assert!(p("S1901M") < p("S1901R"));
        assert!(p("F1901R") < p("W1901A"));
        assert!(p("W1901A") < p("S1902M"));
    }

    #[test]
    fn standard_start_matches_rules_section_15() {
        let s = GameState::standard_start();
        assert_eq!(s.phase.to_string(), "S1901M");
        assert_eq!(s.units.len(), 22);
        assert_eq!(s.supply_centers.len(), 34);
        assert_eq!(s.supply_centers.values().filter(|o| o.is_none()).count(), 12);
        let stp = s.units.iter().find(|u| u.location == "StP").unwrap();
        assert_eq!((stp.unit_type, stp.coast.as_deref()), (UnitType::F, Some("SC")));
    }

    #[test]
    fn orders_use_architecture_json_shape() {
        let cases = [
            json!({ "power": "FRA", "unit": "Mar", "type": "move", "to": "Bur" }),
            json!({ "power": "FRA", "unit": "Gas", "type": "support", "target": "Mar", "to": "Bur" }),
            json!({ "power": "ENG", "unit": "Nth", "type": "convoy", "target": "Lon", "to": "Nwy" }),
            json!({ "power": "ENG", "unit": "Lon", "type": "move", "to": "Nwy", "viaConvoy": true }),
            json!({ "power": "RUS", "unit": "StP", "type": "build", "unitType": "F", "coast": "NC" }),
            json!({ "power": "AUS", "unit": "Vie", "type": "hold" }),
        ];
        for value in cases {
            let order: Order = serde_json::from_value(value.clone()).unwrap();
            assert_eq!(serde_json::to_value(&order).unwrap(), value);
        }
    }
}
