# Diplomacy Rules Specification

> **Source:** *Diplomacy* Rulebook, 5th edition (2008), Avalon Hill / Wizards of the Coast.
> Official PDF: <https://media.wizards.com/2015/downloads/ah/diplomacy_rules.pdf>
>
> This document is a **paraphrased, implementation-oriented restatement** of the official rules,
> written so that developers (and AI agents) working on this repository share one precise
> understanding of the game. It is not a copy of the rulebook. When in doubt, consult the PDF.
> Where the rulebook is silent or ambiguous, the decision taken by this project is recorded in
> [adjudication.md § Project rulings](adjudication.md#5-project-rulings-ambiguities-and-gaps).
>
> Each rule has a stable ID (e.g. `R-MOV-03`). Use these IDs in code comments, tests, commit
> messages and API resolution logs.

Related documents:
[map.md](map.md) (board data) ·
[adjudication.md](adjudication.md) (order resolution algorithm) ·
[test-cases.md](test-cases.md) (rulebook examples as test fixtures) ·
[architecture.md](architecture.md) ·
[../glossary.md](../glossary.md)

---

## 1. Players, powers and goal

| ID | Rule |
|----|------|
| R-GEN-01 | The standard game has **7 players**, each controlling one *Great Power*: **Austria** (Austria-Hungary), **England**, **France**, **Germany**, **Italy**, **Russia**, **Turkey**. "Country" in the rules means Great Power. |
| R-GEN-02 | Powers are assigned randomly. This is the **only** element of chance in the game. |
| R-GEN-03 | **Victory:** a power that controls **18 or more supply centers** (of 34) wins. The check is made **after a Fall turn** (i.e. after Fall retreats, when ownership is updated). |
| R-GEN-04 | **Draw:** players may agree to end the game at any time. All powers that still have units on the board share the draw equally. |
| R-GEN-05 | Negotiation is free-form and **non-binding**: nothing said or written during diplomacy obliges a player. (The tool records game state and orders; it does not enforce agreements.) |

## 2. Units

| ID | Rule |
|----|------|
| R-UNIT-01 | There are two unit types: **Army (A)** and **Fleet (F)**. |
| R-UNIT-02 | **All units have equal strength (1).** Strength only increases through support. |
| R-UNIT-03 | **At most one unit may occupy a province at any time.** No exceptions. |
| R-UNIT-04 | The number of units a power may have equals the number of supply centers it controlled at the end of the last Fall turn (enforced in the Adjustment phase). Therefore there are never more than 34 units on the board. |
| R-UNIT-05 | Physical piece limits are irrelevant to the software (the rulebook allows borrowing pieces of eliminated powers). |

## 3. The board

Full province data is in [map.md](map.md).

| ID | Rule |
|----|------|
| R-MAP-01 | Provinces are of three types: **inland** (land, no sea border), **water** (sea), **coastal** (land adjacent to at least one water province). |
| R-MAP-02 | Armies may occupy inland and coastal provinces. Fleets may occupy water and coastal provinces. |
| R-MAP-03 | **34** inland/coastal provinces are **supply centers** (SC). 22 are *home* SCs of the seven powers, 12 are neutral at start. |
| R-MAP-04 | Unnamed areas cannot be occupied. **Switzerland is impassable.** Islands (other than Great Britain) cannot be occupied. |
| R-MAP-05 | **Fleet coastal movement:** a fleet in a coastal province may move to an adjacent coastal province **only if the two share a coastline** (i.e. the fleet could sail along the coast). Example: F Rome may go to Tuscany, Naples or the Tyrrhenian Sea, but **not** to Venice or Apulia (only a land border is shared). |
| R-MAP-06 | **Split coasts:** **Bulgaria (EC/SC)**, **Spain (NC/SC)** and **St. Petersburg (NC/SC)** have two separate coasts. A fleet there is on exactly one coast and may only move to provinces adjacent to *that* coast, but it still occupies (and blocks) the whole province. |
| R-MAP-07 | When a fleet moves to a split-coast province and **both coasts are reachable**, the order **must specify the coast**; otherwise the order is invalid and the fleet holds. (If only one coast is reachable, the coast is implied — see project ruling PR-04.) |
| R-MAP-08 | **Kiel and Constantinople** have a waterway through them and count as having **one** coast. Fleets may pass through (e.g. Holland → Kiel, later Kiel → Berlin; Black Sea → Constantinople, later Constantinople → Aegean). Armies may cross the waterway freely. Units still need one turn per province (no jumping over). |
| R-MAP-09 | **Denmark / Sweden:** units may move directly between Sweden and Denmark. A fleet cannot move directly between the **Baltic Sea and Skagerrak**; it must go via Sweden or Denmark. Sweden has a single coast. **Denmark does not border Berlin.** |

## 4. Turn structure

| ID | Rule |
|----|------|
| R-TURN-01 | The game starts in **Spring 1901**. Each year has a **Spring** turn and a **Fall** turn, each representing six months. |
| R-TURN-02 | **Spring turn phases:** 1. Diplomacy → 2. Order writing → 3. Order resolution → 4. Retreat & disband. |
| R-TURN-03 | **Fall turn phases:** the same four, plus 5. **Gaining & losing units (Adjustments)**. |
| R-TURN-04 | After the Fall turn (including retreats), supply-center ownership is updated, the victory condition (R-GEN-03) is checked, then adjustments are made. |
| R-TURN-05 | Suggested negotiation time: 30 min before the first turn, 15 min before each later turn; may end early if all agree. Order writing: no more than ~5 min. **No diplomacy is allowed** during order writing/reading, retreats, or adjustments. (The tool has no timers: the GM keeps time outside it. Decided by the project owner.) |
| R-TURN-06 | Orders are dated `Spring 1901`, `Fall 1901`, `Spring 1902`, … |

Phase identifiers used by this project: `S1901M` (Spring movement), `S1901R` (Spring retreat),
`F1901M`, `F1901R`, `W1901A` (Winter/adjustment after Fall 1901). See [architecture.md](architecture.md).

## 5. Orders — general

| ID | Rule |
|----|------|
| R-ORD-01 | Orders are written **secretly** and revealed **simultaneously**. |
| R-ORD-02 | Each power may order all, some, or none of its units. Every unit receives exactly one of: **Hold**, **Move**, **Support**, **Convoy** (convoy: fleets only). |
| R-ORD-03 | **A unit with no order holds.** |
| R-ORD-04 | **An illegal order is not executed; the unit holds.** (It can still receive support to hold.) |
| R-ORD-05 | A **legal** order must be executed even if written by mistake. A poorly worded order with only one possible meaning is executed with that meaning. |
| R-ORD-06 | Notation: unit type, province, order. The unit type (A/F) is a reminder only; omitting it does not invalidate the order because a province holds at most one unit. Abbreviations: see [map.md](map.md). |

Notation summary:

| Order | Notation | Example |
|-------|----------|---------|
| Hold | `<U> <prov> H` / `Holds` | `F Lon H` |
| Move | `<U> <from>-<to>` | `A Par-Bur`, `F Con-Bul EC` |
| Support hold | `<U> <prov> S <U> <prov>` | `F Den S F Bal` |
| Support move | `<U> <prov> S <U> <from>-<to>` | `A Par S A Mar-Bur` |
| Convoy | `F <sea> C A <from>-<to>` | `F Bla C A Ank-Sev` |
| Retreat | `<U> <from>-<to>` | `A Bur-Gas` |
| Disband | `<U> <prov> D` | `A Gal D` |
| Build | `B <U> <prov>[ <coast>]` | `B F StP NC` |

## 6. Hold

| ID | Rule |
|----|------|
| R-HLD-01 | A Hold order keeps the unit in place. It may be supported (R-SUP-05). |

## 7. Move

| ID | Rule |
|----|------|
| R-MOV-01 | An **army** may move to an adjacent inland or coastal province, never to a water province (except via convoy, §9). |
| R-MOV-02 | A **fleet** may move to an adjacent water province or an adjacent coastal province sharing a coastline (R-MAP-05/06). Never inland. |
| R-MOV-03 | A move into an occupied province is called an **attack**. |
| R-MOV-04 | A move succeeds if its strength (1 + valid supports) is **strictly greater** than every competing force: the defending strength of the unit in the target (if it is not leaving successfully) and the strength of every other move into the same target. |

### 7.1 Standoffs (bounces)

| ID | Rule |
|----|------|
| R-STD-01 | **Equal strength:** if two or more units with equal (greatest) strength try to enter the same province, none of them moves. (Rulebook rule 3.) |
| R-STD-02 | **A standoff does not dislodge** a unit already in the contested province. (Rule 4.) |
| R-STD-03 | **Blocking chain:** a unit that does not move (holds, or fails to move) prevents units ordered into its province from entering unless they dislodge it; this can propagate along a chain ("traffic jam"). (Rule 5.) |
| R-STD-04 | **No swapping without convoy:** two units ordered into each other's province cannot both move. (Rule 6.) See PR-01 for supported head-to-head battles. |
| R-STD-05 | **Rotation:** three or more units may rotate (A→B→C→A) as long as no two units swap directly. (Rule 7.) |
| R-STD-06 | **Dislodged units still bounce elsewhere:** a dislodged unit can still cause a standoff in a province *other than* the one its attacker came from. (Rule 10.) |
| R-STD-07 | **No effect on the attacker's origin:** a dislodged unit, even if supported, has **no effect** on the province from which the dislodging attack came. Another unit moving into that province is not bounced by it. (Rule 11.) Applies to head-to-head battles only: a unit dislodged by a convoyed attack can still bounce others in the attacker's origin (PR-20). |

## 8. Support

| ID | Rule |
|----|------|
| R-SUP-01 | A unit may support instead of moving. Support may be **offensive** (supporting a move) or **defensive** (supporting a unit that holds, supports or convoys). Each valid support adds **+1** strength. |
| R-SUP-02 | The supporting unit must be able to **legally move to the province it supports into** (ignoring other units). It need not be adjacent to the supported unit. E.g. an army in Brest cannot support into the English Channel; a fleet in Rome cannot support into Venice. |
| R-SUP-03 | A fleet able to reach either coast of a split-coast province may support a move into that province **regardless of coast**. The coast named in such a support is not checked (PR-18). |
| R-SUP-04 | Support may be given to any power's unit, **without its consent, and cannot be refused** (but see R-SLF-01/02). |
| R-SUP-05 | **Support to hold:** a unit not ordered to move (hold, support, convoy, or no order) can be supported by an order naming only its province (`S F Bal`). If that unit actually tries to move, the support is invalid. (Rule 8.) |
| R-SUP-06 | **Support to move:** must match the supported move exactly (origin and destination). If the supported unit moves elsewhere or holds, the support fails and does **not** turn into a hold support. (Rule 9.) |
| R-SUP-07 | Support cannot be convoyed. |

### 8.1 Cutting support

| ID | Rule |
|----|------|
| R-CUT-01 | **Support is cut** if the supporting unit is **attacked from any province other than the one into which it is supporting** — whether or not the attack succeeds. A cut support adds nothing. (Rule 13.) |
| R-CUT-02 | **Support is cut if the supporting unit is dislodged**, including by an attack from the province into which it is supporting. (Rule 14.) |
| R-CUT-03 | A unit that is itself being dislodged can still cut support elsewhere (as long as the dislodging attack does not come from the province it is attacking). (Rule 15.) |
| R-CUT-04 | **An attack by a power on its own unit does not cut support.** (Rule 16.) |
| R-CUT-05 | Exceptions for convoyed attacks: R-CVY-09 and R-CVY-10. |

> Rulebook tip: in complex positions, first determine which supports are cut, then resolve moves.

## 9. Convoy

| ID | Rule |
|----|------|
| R-CVY-01 | Only a **fleet in a water province** may convoy. Fleets in coastal provinces (including Kiel, Constantinople and Denmark) cannot convoy. |
| R-CVY-02 | Only **armies** can be convoyed. A fleet cannot be convoyed. |
| R-CVY-03 | An army may be convoyed from a coastal province, through a **chain of adjacent water provinces each containing a convoying fleet**, to a coastal province adjacent to the last fleet — all in one turn. |
| R-CVY-04 | Both orders are required: the army orders a move to the destination; each fleet orders `C A <from>-<to>`. The convoy order must match the army's move exactly; otherwise the convoy fails and the army stays put. A fleet may convoy at most one army per turn. |
| R-CVY-05 | **Disruption:** if a convoying fleet that is necessary for the route is **dislodged**, the convoy fails and the army remains in its province. An attack that does not dislodge the fleet has no effect on the convoy. (Rule 17.) |
| R-CVY-06 | **Bounce at destination:** a convoyed army that bounces at its destination stays in its origin (and can still be dislodged there). Convoyed armies may be supported. (Rule 18.) |
| R-CVY-07 | **Swapping via convoy:** two units may exchange places if either or both are convoyed. (Rule 19, exception to R-STD-04.) |
| R-CVY-08 | **Multiple routes:** if orders provide alternative convoy routes, the army arrives as long as **at least one route** remains intact. (Rule 20.) |
| R-CVY-09 | **Convoyed attack does not cut certain supports:** a convoyed army does **not** cut the support of a unit that is supporting an attack against one of the fleets **necessary** for that army's convoy. (Rule 21, supersedes R-CUT-01.) |
| R-CVY-10 | **…unless another route survives:** an army that has at least one successful convoy route **does** cut support given by a unit in the destination province, even if that support targets a fleet on an alternative route. (Rule 22, supersedes R-CVY-09.) |
| R-CVY-11 | **Land route vs convoy route** (when an army could reach its destination either way): if **at least one convoying fleet belongs to the army's owner**, the convoy is used. If none does, the **land route** is used, unless the army's order says **"via convoy"**. (Prevents foreign powers from "kidnapping" an army.) |

## 10. Self-dislodgement rules

| ID | Rule |
|----|------|
| R-SLF-01 | **A power cannot dislodge its own unit, nor support the dislodgement of its own unit**, even unexpectedly. Such a move fails to dislodge (the attacker does not enter). (Rule 12.) |
| R-SLF-02 | Support given by power P to an attack on P's own unit does **not count** toward dislodging that unit (Diagram 24). If the attacker has enough *other* support, the unit is dislodged. |
| R-SLF-03 | Self-attack orders are still meaningful: e.g. they can create standoffs (Diagram 26). |
| R-SLF-04 | **Self-standoff:** a power may deliberately bounce two of its own units in an empty province. If one attack unexpectedly has more support (own or foreign), it succeeds (Diagram 27). It would not succeed against an own unit already in that province. |

## 11. Retreat & disband phase

| ID | Rule |
|----|------|
| R-RET-01 | After resolution, each dislodged unit must retreat or be disbanded. Retreat orders are written without negotiation and revealed simultaneously. |
| R-RET-02 | A retreat must be to an **adjacent province the unit could normally move to** (fleet coast rules apply). Retreats cannot be convoyed or supported. |
| R-RET-03 | A unit **cannot** retreat to: (a) an occupied province; (b) the province **from which the attacker came**; (c) a province **left vacant by a standoff** this turn. |
| R-RET-04 | A province that is empty because the only unit bouncing there was dislodged by a unit coming *from* that province (R-STD-07) was **not** a standoff and **is** a valid retreat target (Diagrams 13/14). |
| R-RET-05 | If no valid retreat exists, the unit is **disbanded immediately**. |
| R-RET-06 | If two or more units retreat to the same province, **all of them are disbanded**. |
| R-RET-07 | A dislodged unit without a retreat order is disbanded. A unit may always choose to disband instead of retreating. |

## 12. Supply centers and adjustments (after Fall)

| ID | Rule |
|----|------|
| R-SC-01 | A power **gains control** of a supply center when one of its units occupies it at the **end of a Fall turn (after retreats)**. |
| R-SC-02 | Control persists while the SC is vacant or occupied by the owner at the end of each Fall turn. Passing through an SC in Spring and leaving in Fall does not change ownership. |
| R-ADJ-01 | Adjustments are written without negotiation and revealed simultaneously. Vague or invalid adjustment orders are ignored. |
| R-ADJ-02 | **Disband:** if a power has more units than SCs, it must disband the excess (owner's choice). See civil disorder for defaults. |
| R-ADJ-03 | **Build:** if a power has more SCs than units, it may build new units, one per **unoccupied home supply center it still controls**. Never outside its home SCs. |
| R-ADJ-04 | Inland SCs: army only. Coastal SCs: unit type must be specified. Fleet in St. Petersburg: coast (NC/SC) must be specified. |
| R-ADJ-05 | If all of a power's controlled home SCs are occupied, it cannot build that year (builds are lost, not deferred). |
| R-ADJ-06 | A power that has lost all its home SCs keeps fighting with its remaining units but cannot build until it controls a home SC again at the end of a Fall turn. |
| R-ADJ-07 | A power may **waive** builds it is entitled to. |
| R-ADJ-08 | A power with no units and no SCs is eliminated. *(Implied; see PR-10.)* |

## 13. Civil disorder

| ID | Rule |
|----|------|
| R-CD-01 | A power that leaves or fails to submit orders in a Spring/Fall turn is in **civil disorder**: its units all hold and do **not** support each other. *(In this tool civil disorder is declared by the GM — PR-16.)* |
| R-CD-02 | Its dislodged units are disbanded (no retreats). No builds are made. |
| R-CD-03 | If it must remove units, remove first the units **farthest from its home country**; ties: **fleets before armies**, then **alphabetical by province**. (Distance definition: PR-08.) |
| R-CD-04 | Replacing a departed player is recommended; policy is decided before the game. |

## 14. Variants for fewer players

> **Not supported by this tool**: it handles the standard 7-player game only (see
> [architecture.md § 1](architecture.md#1-goal)). These rules are kept for reference.

| ID | Players | Setup |
|----|---------|-------|
| R-VAR-01 | 6 | Italy is removed from play: Italian units hold, are not mutually supporting, may be supported to hold by anyone, and are disbanded if dislodged. |
| R-VAR-02 | 5 | As 6 players, removing Italy and Germany. |
| R-VAR-03 | 4 | England; Austria+France; Germany+Turkey; Italy+Russia. |
| R-VAR-04 | 3 | England+Germany+Austria; Russia+Italy; France+Turkey. |
| R-VAR-05 | 2 | WWI scenario: England+France+Russia vs Austria+Germany+Turkey. Italy neutral and impassable. Starts **1914**. Before Fall 1914 adjustments, a coin toss decides which side Italy joins in Spring 1915. Victory at **24 SCs**. |
| R-VAR-06 | 2–4 | SC ownership and adjustments are still computed **per power**, not per player. |

## 15. Starting position (Spring 1901)

| Power | Units | Home SCs |
|-------|-------|----------|
| Austria | A Vie, A Bud, F Tri | Vie, Bud, Tri |
| England | F Lon, F Edi, A Lvp | Lon, Edi, Lvp |
| France | A Par, A Mar, F Bre | Par, Mar, Bre |
| Germany | A Ber, A Mun, F Kie | Ber, Mun, Kie |
| Italy | A Rom, A Ven, F Nap | Rom, Ven, Nap |
| Russia | A Mos, F Sev, A War, F StP(SC) | Mos, Sev, War, StP |
| Turkey | F Ank, A Con, A Smy | Ank, Con, Smy |

The 12 neutral SCs start unoccupied: Nwy, Swe, Den, Hol, Bel, Spa, Por, Tun, Ser, Rum, Bul, Gre.

## 16. The rulebook's 22 resolution rules (cross-reference)

| # | Summary | Rule ID |
|---|---------|---------|
| 1 | All units have the same strength | R-UNIT-02 |
| 2 | One unit per province | R-UNIT-03 |
| 3 | Equal strength into same province → nobody moves | R-STD-01 |
| 4 | Standoff does not dislodge the occupant | R-STD-02 |
| 5 | A non-moving unit can block a chain | R-STD-03 |
| 6 | No swapping without convoy | R-STD-04 |
| 7 | Rotation of 3+ units allowed | R-STD-05 |
| 8 | Hold support names only the province | R-SUP-05 |
| 9 | Move support must match the move | R-SUP-06 |
| 10 | Dislodged unit can still bounce elsewhere | R-STD-06 |
| 11 | Dislodged unit has no effect on its attacker's origin | R-STD-07 |
| 12 | No self-dislodgement (nor support of it) | R-SLF-01 |
| 13 | Support cut by attack from anywhere but the target | R-CUT-01 |
| 14 | Support cut by dislodgement | R-CUT-02 |
| 15 | A unit being dislodged can still cut support | R-CUT-03 |
| 16 | Own attack doesn't cut own support | R-CUT-04 |
| 17 | Dislodging a needed convoy fleet breaks the convoy | R-CVY-05 |
| 18 | Convoyed army bouncing stays home | R-CVY-06 |
| 19 | Swapping allowed via convoy | R-CVY-07 |
| 20 | One intact route suffices | R-CVY-08 |
| 21 | Convoyed army doesn't cut support against its convoy | R-CVY-09 |
| 22 | …except when another route succeeds | R-CVY-10 |
