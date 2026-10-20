//! Text order entry (adjudication.md § 2.1): turns order text such as `A Par-Bur`,
//! `F Nth C A Lon-Nwy` or `Austria: A Vie-Tri, A Bud-Gal` into structured `Order`s.
//!
//! Parsing only works out *which* order is meant and for which unit; whether the order is legal
//! is decided by the same validation as submitted orders (`options::check_orders`).

use serde::Serialize;

use super::province;
use crate::map;
use crate::model::{GameState, Order, OrderKind, PhaseKind, Power, UnitType};

/// One order item of the text, with the order it was read as or why it could not be read.
#[derive(Debug, Clone, Serialize)]
pub struct ParsedOrder {
    /// The item as written.
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<Order>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Parse order text for the current phase of `state`.
///
/// - One order per line, or several separated by `,` or `;`. Empty lines and lines starting
///   with `#` are skipped.
/// - A power name followed by `:` (`France:`, `FRA:`, `フランス:`) at the start of a line sets the
///   power for that line, or, alone on a line, for the following lines; otherwise the power is that
///   of the ordered unit (or, for builds and waives, of the home supply center). A unit of another
///   power is then an error.
/// - Case-insensitive; unit type optional (R-ORD-06) but checked when given; province IDs, common
///   aliases and full names; coasts as `StP/NC`, `StP NC`, `StP (nc)` or `StP north coast`.
pub fn parse_orders(state: &GameState, text: &str, default_power: Option<Power>) -> Vec<ParsedOrder> {
    let mut out = Vec::new();
    let mut current = default_power;
    for line in text.lines() {
        let mut rest = line.trim();
        if rest.is_empty() || rest.starts_with('#') {
            continue;
        }
        // `France:` alone on a line applies to the following lines; `France: A Par-Bur, …`
        // applies to that line only.
        let mut line_power = current;
        if let Some((head, tail)) = rest.split_once([':', '：'])
            && let Some(p) = power_named(head.trim())
        {
            rest = tail.trim();
            if rest.is_empty() {
                current = Some(p);
                continue;
            }
            line_power = Some(p);
        }
        for item in rest.split([',', ';', '、']) {
            let item = item.trim().trim_end_matches('.');
            if item.is_empty() {
                continue;
            }
            out.push(match parse_item(state, item, line_power) {
                Ok(order) => ParsedOrder {
                    text: item.to_string(),
                    order: Some(order),
                    error: None,
                },
                Err(e) => ParsedOrder {
                    text: item.to_string(),
                    order: None,
                    error: Some(e),
                },
            });
        }
    }
    out
}

fn power_named(s: &str) -> Option<Power> {
    let s = s.to_lowercase();
    let names: [(Power, &[&str]); 7] = [
        (
            Power::Austria,
            &[
                "aus",
                "austria",
                "austria-hungary",
                "オーストリア",
                "オーストリア＝ハンガリー",
            ],
        ),
        (
            Power::England,
            &["eng", "england", "britain", "イギリス", "イングランド"],
        ),
        (Power::France, &["fra", "france", "フランス"]),
        (Power::Germany, &["ger", "germany", "ドイツ"]),
        (Power::Italy, &["ita", "italy", "イタリア"]),
        (Power::Russia, &["rus", "russia", "ロシア"]),
        (Power::Turkey, &["tur", "turkey", "トルコ"]),
    ];
    names.iter().find(|(_, ns)| ns.contains(&s.as_str())).map(|(p, _)| *p)
}

/// Aliases accepted on input (adjudication.md § 2.1), lower case → canonical ID.
const ALIASES: &[(&str, &str)] = &[
    ("lyo", "GoL"),
    ("gol", "GoL"),
    ("mao", "Mid"),
    ("mat", "Mid"),
    ("nao", "NAt"),
    ("nat", "NAt"),
    ("nwg", "Nrg"),
    ("naf", "NAf"),
    ("port", "Por"),
    ("galacia", "Gal"),
    ("sevastapol", "Sev"),
    ("tys", "Tyn"),
];

/// Lower-case province ID, alias or one-word full name → canonical ID.
fn location_id(token: &str) -> Option<String> {
    let map = map::standard();
    if let Some(p) = map.provinces.iter().find(|p| p.id.eq_ignore_ascii_case(token)) {
        return Some(p.id.clone());
    }
    if let Some((_, id)) = ALIASES.iter().find(|(a, _)| *a == token) {
        return Some((*id).to_string());
    }
    map.provinces
        .iter()
        .find(|p| squash(&p.name) == token)
        .map(|p| p.id.clone())
}

/// `"St. Petersburg"` → `"stpetersburg"`: lower case, letters and digits only.
fn squash(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

/// Lower-case the item, turn multi-word names into single tokens and split it into tokens.
fn tokenize(item: &str) -> Vec<String> {
    let mut s = format!(" {} ", item.to_lowercase());
    for (from, to) in [
        ("–", "-"),
        ("—", "-"),
        ("−", "-"),
        ("→", "-"),
        ("->", "-"),
        ("(", " "),
        (")", " "),
        ("/", " "),
    ] {
        s = s.replace(from, to);
    }
    // Multi-word full names first (longest first), so "Mid-Atlantic Ocean" survives the dash split.
    let mut names: Vec<(String, String)> = map::standard()
        .provinces
        .iter()
        .flat_map(|p| {
            let lower = p.name.to_lowercase();
            let no_dot = lower.replace('.', "");
            [lower, no_dot]
                .into_iter()
                .map(|n| (n, p.id.to_lowercase()))
                .collect::<Vec<_>>()
        })
        .filter(|(n, _)| n.contains(' ') || n.contains('-'))
        .collect();
    names.extend([
        ("mid-atlantic".into(), "mid".into()),
        ("north atlantic".into(), "nat".into()),
    ]);
    names.sort_by_key(|(n, _)| std::cmp::Reverse(n.len()));
    for (name, id) in names {
        s = s.replace(&format!(" {name} "), &format!(" {id} "));
        s = s.replace(&format!(" {name}-"), &format!(" {id}-"));
        s = s.replace(&format!("-{name} "), &format!("-{id} "));
    }
    for (from, to) in [
        (" north coast ", " nc "),
        (" south coast ", " sc "),
        (" east coast ", " ec "),
        (" via convoy ", " viaconvoy "),
        (" by convoy ", " viaconvoy "),
    ] {
        s = s.replace(from, to);
    }
    s.replace('-', " - ")
        .split_whitespace()
        .filter(|t| !NATIONALITIES.contains(t))
        .map(str::to_string)
        .collect()
}

/// Ignored when naming a foreign unit, as in `A Ser S Turkish A Bul-Rum`.
const NATIONALITIES: &[&str] = &[
    "austrian", "english", "british", "french", "german", "italian", "russian", "turkish",
];

struct Tokens {
    t: Vec<String>,
    i: usize,
}

impl Tokens {
    fn peek(&self) -> Option<&str> {
        self.t.get(self.i).map(String::as_str)
    }
    fn next(&mut self) -> Option<String> {
        let t = self.t.get(self.i).cloned();
        self.i += 1;
        t
    }
    fn eat(&mut self, words: &[&str]) -> bool {
        if self.peek().is_some_and(|t| words.contains(&t)) {
            self.i += 1;
            true
        } else {
            false
        }
    }
    fn unit_type(&mut self) -> Option<UnitType> {
        match self.peek() {
            Some("a" | "army") => {
                self.i += 1;
                Some(UnitType::A)
            }
            Some("f" | "fleet") => {
                self.i += 1;
                Some(UnitType::F)
            }
            _ => None,
        }
    }
    /// A province, optionally followed by a coast.
    fn location(&mut self) -> Result<(String, Option<String>), String> {
        let tok = self.next().ok_or("a province is missing")?;
        let id = location_id(&tok).ok_or_else(|| format!("unknown province `{tok}`"))?;
        let coast = match self.peek() {
            Some(c @ ("nc" | "sc" | "ec")) => {
                let c = c.to_uppercase();
                self.i += 1;
                Some(c)
            }
            _ => None,
        };
        Ok((id, coast))
    }
    fn rest(&self) -> Option<String> {
        (self.i < self.t.len()).then(|| self.t[self.i..].join(" "))
    }
}

const HOLD: &[&str] = &["h", "hold", "holds", "stand", "stands"];
const MOVE: &[&str] = &["-", "to", "m", "move", "moves", "r", "retreat", "retreats"];
const SUPPORT: &[&str] = &["s", "support", "supports"];
const CONVOY: &[&str] = &["c", "convoy", "convoys"];
const DISBAND: &[&str] = &["d", "disband", "disbands", "remove"];
const BUILD: &[&str] = &["b", "build", "builds"];
const WAIVE: &[&str] = &["w", "waive", "waives"];

fn parse_item(state: &GameState, item: &str, current: Option<Power>) -> Result<Order, String> {
    let mut tk = Tokens {
        t: tokenize(item),
        i: 0,
    };
    let kind = state.phase.kind;
    let map = map::standard();

    // Adjustment orders that do not start with a unit: builds and waives.
    if tk.eat(BUILD) {
        if kind != PhaseKind::Adjustment {
            return Err("builds are only possible in an adjustment phase".into());
        }
        let unit_type = tk.unit_type().ok_or("say which unit to build (A or F)")?;
        let (site, coast) = tk.location()?;
        let power = current
            .or_else(|| map.province(&site).and_then(|p| p.home))
            .ok_or_else(|| format!("{site} is not a home supply center"))?;
        finish(&tk)?;
        let coast = if unit_type == UnitType::A { None } else { coast };
        return Ok(Order {
            power,
            unit: site,
            kind: OrderKind::Build { unit_type, coast },
        });
    }
    if tk.eat(WAIVE) {
        if kind != PhaseKind::Adjustment {
            return Err("waives are only possible in an adjustment phase".into());
        }
        let site = if tk.peek().is_some() {
            Some(tk.location()?.0)
        } else {
            None
        };
        let power = current
            .or_else(|| site.as_deref().and_then(|s| map.province(s)).and_then(|p| p.home))
            .ok_or("say which power waives a build (e.g. `France: waive`)")?;
        let site = site
            .or_else(|| {
                map.supply_centers()
                    .find(|p| p.home == Some(power))
                    .map(|p| p.id.clone())
            })
            .unwrap_or_default();
        finish(&tk)?;
        return Ok(Order {
            power,
            unit: site,
            kind: OrderKind::Waive,
        });
    }

    // Everything else names a unit first.
    let given_type = tk.unit_type();
    let (loc, _) = tk.location()?;
    let unit = if kind == PhaseKind::Retreat {
        state
            .dislodged
            .iter()
            .find(|u| u.location == loc)
            .map(|u| (u.power, u.unit_type))
            .ok_or_else(|| format!("no dislodged unit in {loc}"))?
    } else {
        state
            .units
            .iter()
            .find(|u| u.location == loc)
            .map(|u| (u.power, u.unit_type))
            .ok_or_else(|| format!("no unit in {loc}"))?
    };
    let (power, unit_type) = unit;
    if let Some(t) = given_type
        && t != unit_type
    {
        return Err(format!(
            "the unit in {loc} is {}, not {}",
            type_name(unit_type),
            type_name(t)
        ));
    }
    if let Some(p) = current
        && p != power
    {
        return Err(format!(
            "the unit in {loc} belongs to {}, not {}",
            power.code(),
            p.code()
        ));
    }
    let order = |kind| Order {
        power,
        unit: loc.clone(),
        kind,
    };

    let parsed = if tk.peek().is_none() || tk.eat(HOLD) {
        match kind {
            PhaseKind::Movement => order(OrderKind::Hold),
            _ => return Err("say what the unit does".into()),
        }
    } else if tk.eat(DISBAND) {
        order(OrderKind::Disband)
    } else if tk.eat(MOVE) {
        let (to, coast) = tk.location()?;
        let to = match (unit_type, coast) {
            (UnitType::F, Some(c)) => format!("{to}/{c}"),
            _ => to,
        };
        let via_convoy = tk.eat(&["viaconvoy"]);
        match kind {
            PhaseKind::Movement => order(OrderKind::Move { to, via_convoy }),
            PhaseKind::Retreat if !via_convoy => order(OrderKind::Retreat { to }),
            PhaseKind::Retreat => return Err("retreats cannot be convoyed (R-RET-02)".into()),
            PhaseKind::Adjustment => return Err("units cannot move in an adjustment phase".into()),
        }
    } else if tk.eat(SUPPORT) {
        if kind != PhaseKind::Movement {
            return Err("supports are only possible in a movement phase".into());
        }
        tk.unit_type();
        let (target, _) = tk.location()?;
        let to = if tk.eat(MOVE) {
            Some(province(&tk.location()?.0).to_string())
        } else {
            None
        };
        tk.eat(HOLD);
        order(OrderKind::Support { target, to })
    } else if tk.eat(CONVOY) {
        if kind != PhaseKind::Movement {
            return Err("convoys are only possible in a movement phase".into());
        }
        tk.unit_type();
        let (target, _) = tk.location()?;
        if !tk.eat(MOVE) {
            return Err("a convoy needs the army's destination (`C A Lon-Nwy`)".into());
        }
        let (to, _) = tk.location()?;
        order(OrderKind::Convoy { target, to })
    } else {
        return Err(format!("unknown order `{}`", tk.rest().unwrap_or_default()));
    };
    finish(&tk)?;
    Ok(parsed)
}

fn finish(tk: &Tokens) -> Result<(), String> {
    match tk.rest() {
        Some(rest) => Err(format!("unexpected `{rest}`")),
        None => Ok(()),
    }
}

fn type_name(t: UnitType) -> &'static str {
    match t {
        UnitType::A => "an army",
        UnitType::F => "a fleet",
    }
}
