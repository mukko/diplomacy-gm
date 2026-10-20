# Order Adjudication Specification

How the API server turns a set of orders into results. Rule IDs refer to [rules.md](rules.md).
Executable examples are in [test-cases.md](test-cases.md).

## 1. Overview

Adjudication is a pure function:

```
adjudicate(state: GameState, orders: Order[]) -> Resolution
```

- `state` — phase (`S1901M` …), unit positions (with coasts), SC ownership, dislodged units (retreat phase).
- `orders` — at most one order per unit, already attributed to the ordering power.
- `Resolution` — per-order outcome, the **step-by-step trace** used by the GUI, and the next `GameState`.

Three phase kinds have different adjudicators:

| Phase kind | Input | Output |
|------------|-------|--------|
| Movement (`M`) | Hold / Move / Support / Convoy | new positions, dislodged units + their legal retreats |
| Retreat (`R`) | Retreat / Disband | new positions; after Fall: SC ownership update, victory check |
| Adjustment (`A`) | Build / Disband / Waive | new units, removed units |

The adjudicator must be **deterministic** and must not depend on order input order.

## 2. Order parsing and validation

Validation happens **before** resolution. An invalid order is replaced by **Hold** (R-ORD-04) and
the reason is recorded in the trace.

### 2.1 Parsing (text input)

The GUI submits structured orders, but a text parser is provided for convenience and tests.

- Case-insensitive. Unit type (`A`/`F`) optional (R-ORD-06); if given and wrong, the order is invalid.
- Move separators: `-`, `–`, `->`, `to`. Keywords: `H`/`Hold(s)`, `S`/`Support(s)`, `C`/`Convoy(s)`,
  `R`/`Retreat`, `D`/`Disband`, `B`/`Build`, `via convoy`.
- Coasts: `NC SC EC`, also `/nc`, `(nc)`, `North Coast`, etc.
- Province aliases (accepted on input, normalised to rulebook abbreviations): e.g. `Lyo`→`GoL`,
  `Mao`/`Mat`→`Mid`, `Nao`→`NAt`, `Nwg`→`Nrg`, `Stp`→`StP`, `Naf`→`NAf`, `Port`→`Por`, full names.
- Anything ambiguous → invalid ("when in doubt, write it out").

### 2.2 Validation rules (movement phase)

| Check | Failure result |
|-------|----------------|
| Unit exists at the location and belongs to the ordering power | order ignored (PR-14) |
| Move: destination adjacent for that unit type/coast (R-MOV-01/02), or army move with a possible convoy route (R-CVY-03) | Hold |
| Move to split-coast province with two reachable coasts and no coast given | Hold (R-MAP-07) |
| Move with a coast that is not reachable | Hold (PR-04) |
| Support: supporter could move to the target province (R-SUP-02/03) | Hold |
| Support: a unit exists at the supported location | Hold |
| Convoy: ordering unit is a fleet in a water province (R-CVY-01); convoyed unit is an army (R-CVY-02) | Hold |
| Convoy: origin and destination are coastal | Hold |

Note: "support/convoy does not match the actual order" is **not** a validation failure — the order is
legal, it simply provides nothing during resolution (R-SUP-06, R-CVY-04).

## 3. Resolution algorithm (movement phase)

Use the standard **decision-based** approach (as described by L. B. Kruijswijk in *The Math of
Adjudication*, used by most modern judges) and validate against the DATC
(Diplomacy Adjudicator Test Cases). The rulebook's rules map onto these definitions:

### 3.1 Decisions

For each order the adjudicator determines:

| Decision | Applies to | Meaning |
|----------|-----------|---------|
| `path` | move | an army move has a valid route (adjacent, or ≥1 convoy route with no dislodged fleet) |
| `support` | support | support is *given* (not cut, not void) |
| `move` | move | the move succeeds |
| `dislodged` | every unit | the unit is dislodged |

### 3.2 Strengths

| Strength | Definition |
|----------|-----------|
| **Hold strength** of province P | 0 if P is empty, or if its unit moved away successfully. 1 if its unit tried to move but failed. Otherwise 1 + number of given hold-supports. |
| **Attack strength** of move X→P | 0 if no `path`. If P is empty, or P's unit moves away successfully (and is not in a head-to-head with X): 1 + given supports. Otherwise: 0 if P's unit belongs to X's power (R-SLF-01); else 1 + given supports **excluding supports from P's owner** (R-SLF-02). |
| **Defend strength** of move X→P (head-to-head only) | 1 + given supports. |
| **Prevent strength** of move X→P | 0 if no `path`. 0 if X is in a head-to-head battle and the opposing move succeeded (R-STD-07). Otherwise 1 + given supports (own-power supports count here — PR-02). |

### 3.3 Decision rules

- **Move X→P succeeds** iff
  - attack strength > hold strength of P — or, in a head-to-head battle (P's unit moves to X without convoy), attack strength > defend strength of the opposing move; **and**
  - attack strength > prevent strength of every other move into P.
- **Support is cut** (R-CUT-01..04) iff the supporter is dislodged, **or** some move into the
  supporter's province with a valid `path`
  - comes from a different power than the supporter (R-CUT-04), and
  - does not come from the province the support is directed into (R-CUT-01),
  - and, for a convoyed attacker, is not excluded by R-CVY-09 (support against a fleet the attacker's convoy needs), unless R-CVY-10 applies.
- **Unit is dislodged** iff it did not move successfully and some move into its province succeeds.
- **Convoy path** exists iff there is a chain of convoying fleets, all with matching orders, all not
  dislodged, linking origin to destination (R-CVY-03/05/08). Land vs convoy choice: R-CVY-11 / PR-07.
- **Head-to-head without convoy** means two units ordered into each other's provinces, both by land
  (R-STD-04). If either move uses a convoy, they are independent moves (R-CVY-07).

### 3.4 Cycles and paradoxes

Decisions depend on each other. Resolve with guess-and-check recursion (or iteration to a fixed
point). When a dependency cycle is found:

1. **Circular movement** (A→B→C→A, all land, no head-to-head) — all moves succeed (R-STD-05).
2. **Convoy paradox** — apply R-CVY-09/10 first (the rulebook's own resolution).
3. Any **remaining paradox** — apply the *Szykman rule*: the convoyed armies involved are treated as
   not moving and have no effect (they do not cut support, do not bounce). See PR-03.

### 3.5 Output of the movement phase

- New positions of all non-dislodged units.
- Dislodged units, each with: attacker origin, and **legal retreat destinations** computed now
  (R-RET-02/03/04) — the set of adjacent provinces reachable by that unit type/coast, minus occupied
  provinces, minus the attacker's origin, minus provinces where a standoff occurred this turn.
- `standoffProvinces`: provinces left empty by a bounce (needed for R-RET-03(c)).
- If no dislodged unit has a legal retreat (or there are no dislodged units), the retreat phase is skipped.

## 4. Step-by-step trace (for the GUI)

The GUI must be able to replay resolution one step at a time. The adjudicator therefore emits an
ordered list of **trace steps**, each referencing orders, provinces and rule IDs. The internal
algorithm is recursive, so the trace is a *presentation order* computed after resolution:

| # | Step | Contents |
|---|------|----------|
| 1 | `validate` | each order: valid / replaced by Hold, with reason and rule ID |
| 2 | `convoy-routes` | candidate routes for each convoyed army; route chosen (land/convoy, R-CVY-11) |
| 3 | `support-cuts` | each support: given / cut (by whom, rule) / void (mismatch) |
| 4 | `strengths` | per contested province: hold/attack/defend/prevent strengths with contributing supports |
| 5 | `moves` | each move: success or failure and the deciding comparison (e.g. "2 > 1", "standoff 1 = 1") |
| 6 | `dislodgements` | dislodged units, attacker origin, retreat options |
| 7 | `result` | final positions; standoff provinces |

Retreat and adjustment phases emit similar but shorter traces (`validate`, `conflicts`, `result`,
and for Fall: `sc-ownership`, `victory-check`). The adjustment phase uses `default-disbands` (units
chosen by R-CD-03 / PR-08, with their distances) in place of `conflicts`.

Every trace entry should include the rule ID(s) that justify it, e.g.
`{ "step": "support-cuts", "order": "A Sil S A Pru-War", "outcome": "cut", "by": "A Boh-Sil", "rules": ["R-CUT-01"] }`.

## 5. Project rulings (ambiguities and gaps)

Decisions taken where the rulebook is silent or ambiguous. Changing one of these requires updating
this table (and the Japanese version) and the related tests. Items marked **(open)** should be
confirmed by the project owner.

| ID | Topic | Ruling | Basis |
|----|-------|--------|-------|
| PR-01 | Supported head-to-head battle | The stronger side wins and dislodges the other; equal → both stay. | Rulebook Diagrams 13–14; DATC |
| PR-02 | Support from the defender's own power | Not counted toward dislodging that power's unit, but **counts** for prevent strength (bouncing others). | Diagrams 24–26 |
| PR-03 | Paradoxes not covered by rules 21–22 | Szykman rule: involved convoyed armies do not move and have no effect. | DATC recommendation |
| PR-04 | Coast specification | Omitted but only one coast reachable → inferred. Unreachable coast given → invalid (Hold). Coast on an army order → ignored. **(open)** The same applies to retreats (a fleet's coast may be omitted when only one coast is among its retreat options) and to builds (coast ignored for an army; a fleet built in a split-coast province needs its coast, R-ADJ-04). | R-MAP-07 |
| PR-05 | Multiple orders for one unit | Not possible: the API stores one order per unit; a resubmission replaces it. If the adjudicator nevertheless receives several orders for one unit, all of them are ignored and the unit holds (independent of input order); in a retreat phase the unit is disbanded (R-RET-07). Confirmed by the project owner. | — |
| PR-06 | Retreat to the origin of a **convoyed** attacker | **Forbidden**, following the rulebook literally (R-RET-03(b)), even though DATC allows it. Confirmed by the project owner. | R-RET-03 |
| PR-07 | "via convoy" | Represented as `viaConvoy: true` on the move. If the destination is not adjacent, a convoy is used regardless. | R-CVY-11 |
| PR-08 | Civil-disorder distance | Minimum number of province steps from the unit to **any** home SC of its power (owned or not), moving through any province (land or water) regardless of unit type. Ties: fleets before armies, then alphabetical by province (R-CD-03); **(open)** "alphabetical" uses the English province name from map.md (e.g. *Livonia* before *Ukraine*). The automatic choice is only a **default**: before the phase is resolved the GM may enter or replace the disband orders of any power (see § 7), and GM-entered orders take precedence. Confirmed by the project owner. | R-CD-03 |
| PR-09 | Missing retreat / adjustment orders | Missing retreat → disband. Missing required disbands → chosen by R-CD-03. Missing builds → waived. | R-RET-07, R-CD |
| PR-10 | Elimination | A power with no units and no SCs is eliminated. A power with SCs but no units stays in the game. | — |
| PR-11 | Victory timing | Checked after the Fall retreat phase, after SC ownership update, before adjustments. | R-GEN-03 |
| PR-12 | Unmatched convoy order | Provides nothing; the fleet counts as not moving (can receive hold support). | R-CVY-04 |
| PR-13 | Hold support to a unit whose move order was invalid | Valid — that unit is treated as holding. | R-ORD-04, R-SUP-05 |
| PR-14 | Order for a unit not owned / not present | Ignored and reported in the trace. | — |
| PR-15 | Excess or invalid adjustment orders | Invalid ignored (R-ADJ-01); builds beyond entitlement ignored in submission order; build in an occupied or non-controlled home SC is invalid. **(open)** A waive order uses up one build in submission order, like a build. | R-ADJ |
| PR-16 | Civil disorder trigger | **Declared by the GM**, never automatic. The GM records the declaration (power, starting phase) in the tool and can lift it later (e.g. a replacement player joins). While declared: movement — all orders ignored, units hold, no mutual support (R-CD-01); retreat — dislodged units disbanded (R-CD-02); adjustment — no builds, required removals chosen by R-CD-03. **(open)** Disband orders filed for a power in civil disorder are treated as GM-entered and used (PR-08); its build orders are ignored. A power that merely failed to submit orders without a declaration is handled by R-ORD-03 / PR-09. | R-CD-01, R-CD-02 |
| PR-17 | Skipping the adjustment phase | **(open)** The adjustment phase is skipped when no power must disband and no power can build: a power owed builds counts only if it is not in civil disorder and has at least one unoccupied home SC it controls (R-ADJ-03/05, R-CD-02). | architecture.md § 3 |

## 6. Retreat phase

1. Validate: destination must be in the unit's precomputed retreat options; otherwise → disband.
2. Two or more retreats to the same province → all disbanded (R-RET-06).
3. Apply. After a **Fall** retreat phase: update SC ownership (R-SC-01/02), check victory (PR-11).

## 7. Adjustment phase

1. For each power: `delta = controlledSCs - units`.
2. `delta > 0`: accept up to `delta` valid builds (R-ADJ-03/04/05/06), others ignored.
3. `delta < 0`: require exactly `-delta` disbands; missing ones chosen by R-CD-03 / PR-08 (PR-09). The server exposes the automatically proposed disbands before resolution so the GM can review them and, if needed, enter different disband orders for that power (GM override).
4. `delta = 0`: no orders accepted.
5. Next phase is Spring of the following year.

## 8. References

- Official rulebook (5th ed., 2008): <https://media.wizards.com/2015/downloads/ah/diplomacy_rules.pdf>
- Diplomacy Adjudicator Test Cases (DATC), L. B. Kruijswijk — the de-facto conformance suite for
  adjudicators. Where DATC offers options, the rulebook (2000/2008 rules) option is preferred unless a
  project ruling above says otherwise.
