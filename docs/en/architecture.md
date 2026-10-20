# Architecture (draft)

> Status: **draft / proposal.** Technology choices marked *(proposed)* are not final; those marked *(decided)* are.

## 1. Goal

A game-master tool for *Diplomacy* that

- stores game state and history,
- lets the GM enter every power's orders on an interactive map,
- adjudicates orders according to the official rules ([rules.md](rules.md), [adjudication.md](adjudication.md)),
- returns **both the result and the reasoning steps**, so the GUI can replay resolution step by step.

Negotiation itself happens outside the tool (in person or elsewhere). The tool does not enforce agreements (R-GEN-05).

Scope *(decided)*: the standard 7-player game only. The variants for fewer players (rules.md § 14,
`R-VAR-*`) are not supported; `variant` is always `standard` and the server rejects any other value.

## 2. Components

```
┌──────────────────────────┐          JSON/HTTP          ┌───────────────────────────────┐
│ GUI (web)                │  ───── orders ──────────▶   │ API server                    │
│  - map (SVG)             │                             │  - game/phase management      │
│  - order input           │  ◀──── state, resolution,   │  - order validation           │
│  - phase/progress view   │         trace steps ─────   │  - adjudicator (pure)         │
│  - step-by-step replay   │                             │  - persistence                │
└──────────────────────────┘                             └───────────────────────────────┘
                     ▲                                                ▲
                     └──────────── shared: types, map data ───────────┘
```

| Component | Responsibility | Tech *(proposed)* |
|-----------|----------------|-------------------|
| `server/` | REST API, game lifecycle, persistence, calls adjudicator | **Rust** (axum, tokio, serde) *(decided)* |
| Adjudicator (`server/src/engine/`) | Pure adjudicator: `adjudicate(state, orders) → Resolution`. No I/O. Plugs into the server's `Adjudicator` trait. | Rust, `cargo test`; DATC + [test-cases.md](test-cases.md) |
| `web/` | Map rendering, order entry, progress, trace replay | React + Vite, SVG map |
| `shared/` | Map data (`standard.json`: provinces, coasts, adjacency), read by both server and GUI; map drawing assets for the GUI (`standard.svg`, `standard.layout.json`, `standard.graph.svg`, see [map.md § 7](map.md#7-drawing-the-map-gui-assets)); TypeScript map types/tests | JSON, SVG (+ TypeScript, Vitest) |
| Persistence | One JSON save file per game (§ 5) | Files *(decided)* |
| Deployment | Runs locally on the GM's machine for a single user; listens on `127.0.0.1` only; no authentication or roles (§ 6) | *(decided)* |

The adjudicator is the authority: the GUI may pre-validate for usability, but the server re-validates everything.

## 3. Game flow

```
create game ─▶ S1901M ─▶ [S1901R] ─▶ F1901M ─▶ [F1901R] ─▶ W1901A ─▶ S1902M ─▶ …
                 │            │          │           │           │
              orders      retreats    orders     retreats    builds/disbands
                 └── resolve ──┘                     └─ SC update + victory check (PR-11)
```

- Retreat phases (`R`) are skipped when no unit has a legal retreat.
- Adjustment phase (`A`) is skipped when no power needs to build or disband (PR-17).
- A phase is resolved when the GM triggers it (or all powers have submitted, depending on settings).
  Powers that submitted nothing are treated per R-ORD-03 / PR-09; civil disorder applies only when declared by the GM (PR-16).

### Phase ID format

`<S|F|W><year><M|R|A>` — e.g. `S1901M`, `F1901R`, `W1901A`.

## 4. Core data model (sketch)

```jsonc
// GameState (board position at the start of a phase; game ID and variant live in the save file, § 5)
{
  "phase": "F1901M",
  "units": [
    { "power": "RUS", "type": "F", "location": "StP", "coast": "SC" },
    { "power": "FRA", "type": "A", "location": "Bur" }
  ],
  "supplyCenters": { "Par": "FRA", "Bel": null },
  "dislodged": [ /* retreat phase only */
    { "power": "FRA", "type": "A", "location": "Bur", "attackerFrom": "Ruh", "retreatOptions": ["Gas", "Pic"] }
  ],
  "standoffProvinces": ["Sil"],
  "civilDisorder": [ { "power": "ITA", "since": "F1902M" } ],   // GM-declared (PR-16)
  "winner": null
}

// Order (structured, as sent by the GUI)
{ "power": "FRA", "unit": "Mar", "type": "move", "to": "Bur" }
{ "power": "FRA", "unit": "Gas", "type": "support", "target": "Mar", "to": "Bur" }   // omit "to" for hold support
{ "power": "ENG", "unit": "Nth", "type": "convoy", "target": "Lon", "to": "Nwy" }
{ "power": "ENG", "unit": "Lon", "type": "move", "to": "Nwy", "viaConvoy": true }
{ "power": "RUS", "unit": "StP", "type": "build", "unitType": "F", "coast": "NC" }

// Resolution
{
  "phase": "F1901M",
  "orders": [ { "order": { /* … */ }, "valid": true, "outcome": "success|fail|void|cut", "reason": "…", "rules": ["R-STD-01"] } ],
  "trace": [ { "step": "support-cuts", "order": "A Sil S A Pru-War", "outcome": "cut", "by": "A Boh-Sil", "rules": ["R-CUT-01"] } ],
  "nextState": { /* GameState */ }
}
```

Power IDs: `AUS ENG FRA GER ITA RUS TUR`. Province IDs: rulebook abbreviations ([map.md](map.md)).

## 5. Persistence (save files)

- **One game = one JSON file**, `<dataDir>/<gameId>.json` (server option `--data-dir`,
  env `DIPLOMACY_GM_DATA_DIR`, default `data/games`). The file is the source of truth.
- **Saved automatically:** every change (new game, order submission, resolution) is written back
  immediately and atomically (temporary file + rename), so a crash never leaves a half-written save.
- **Restoring:** a save file (on disk or uploaded) is parsed and **validated** before use: format and
  version, game ID, known provinces/coasts, at most one unit per province (R-UNIT-03), and that the
  history is consistent (each resolution's `nextState` equals the next recorded position, except for
  `civilDisorder`, which the GM may change between phases). Invalid
  data is rejected and nothing is written.
- **Replay** needs nothing else: the history keeps the starting position, orders and `Resolution`
  (incl. `trace`) of every resolved phase.

```jsonc
// <gameId>.json
{
  "format": "diplomacy-gm/game",
  "formatVersion": 1,
  "gameId": "g_…",
  "title": "Friday game",
  "variant": "standard",
  "createdAt": "2026-10-19T12:00:00Z",
  "updatedAt": "2026-10-19T13:10:00Z",
  "state": { /* GameState: current, unresolved phase */ },
  "orders": { "GER": [ /* Order */ ] },          // submitted so far for the current phase (secret)
  "history": [                                    // resolved phases, oldest first
    { "phase": "S1901M", "state": { /* GameState */ }, "orders": { /* per power */ },
      "resolution": { /* Resolution */ }, "resolvedAt": "…" }
  ]
}
```

A replay **frame** is one entry of `history` (position + orders + resolution), followed by a final
frame for the current position. Within a frame the GUI steps through `resolution.trace` (§ 7).

## 6. API (sketch)

Start screen: **New game** → `POST /games`; **Load** → `GET /games` then `GET /games/:id`
(or `POST /games/import` with a save file); **Replay** → `GET /games/:id/replay`.
Implemented so far: the routes marked ✔. `resolve` adjudicates movement, retreat and adjustment phases.

| Method & path | Purpose |
|---------------|---------|
| `GET  /maps/standard` ✔ | Map data for rendering and client-side hints |
| `GET  /games` ✔ | Saved games (ID, title, phase, last update), newest first, plus save files that could not be read |
| `POST /games` ✔ | Create a game and its save file (title; later: variant, power assignment, settings) |
| `GET  /games/:id` ✔ | Restore from the save file: current state, phase, which powers have submitted (orders stay secret) |
| `GET  /games/:id/save` ✔ | Download the complete save file (includes the unresolved orders) |
| `POST /games/import` ✔ | Restore a game from an uploaded save file (validated); `?asNew=true` stores it under a new ID |
| `GET  /games/:id/history` ✔ | All phases with orders and resolutions |
| `GET  /games/:id/replay` ✔ | Replay frames (§ 5) |
| `GET  /games/:id/replay/:index` ✔ | One replay frame |
| `GET  /maps/standard/svg` ✔, `…/layout` ✔, `…/graph` ✔ | Map drawing assets for the GUI ([map.md § 7](map.md#7-drawing-the-map-gui-assets)) |
| `GET  /games/:id/phases/:phase/orders` ✔ | Orders submitted so far for the current phase, per power, each with the same `{ order, status, reason?, rules }` check as `PUT …/orders` (local single-GM tool: not secret). `409 phase-mismatch` for another phase |
| `GET  /games/:id/units/:prov/options` ✔ | Legal orders for one unit in the current phase, as ready-to-submit `Order` objects checked with the adjudicator's own validation. Movement: hold, moves (split-coast destinations with their coast; `viaConvoy` variants where a convoy is possible), hold and move supports (only for moves the supported unit can legally be ordered to make), convoys (only for routes that can pass through this fleet). Retreat: the dislodged unit's retreat options and disband. Adjustment: disband, if its power must remove units. A power in civil disorder gets `civilDisorder: true` and no orders in movement and retreat phases (its orders are ignored, PR-16); in an adjustment phase its disbands are still offered, for the GM to enter (PR-08). `404` if there is no such unit |
| `GET  /games/:id/powers/:power/options` ✔ | The same for every unit of a power (dislodged units in a retreat phase). In an adjustment phase also `adjustment` (SCs − units) and `builds`: each legal build (home SC × unit type × coast) plus a waive order; empty in civil disorder |
| `POST /games/:id/phases/:phase/parse-orders` ✔ | Read order text `{ text }` ([adjudication.md § 2.1](adjudication.md#21-parsing-text-input)); returns `items`: `{ text, order?, error?, check? }` per item. Stores nothing |
| `PUT  /games/:id/phases/:phase/orders/:power` ✔ | Submit/replace a power's orders. Works for any power, including manual disbands for a power in civil disorder (PR-08). All orders are stored; the response has `accepted` (count) and `results`: per order, in submission order, `{ order, status, reason?, rules }` with `status` `valid` (executed as written; it may still fail during resolution), `hold` (movement: not executable, the unit holds — R-ORD-04), `disband` (retreat: not executable, the unit is disbanded) or `ignored` (no effect: not the power's unit, civil disorder, duplicate, beyond entitlement, wrong phase — PR-05/14/15/16, R-ADJ-01). Same checks as the adjudicator's validation (adjudication.md § 2) |
| `GET  /games/:id/phases/:phase/default-disbands` ✔ | For the current adjustment phase: each power that must remove units, with `required`, `ordered` (submitted disband orders that will be used, e.g. entered by the GM) and `proposed` (units the engine will remove for the rest, with their PR-08 distance) — R-CD-03 / PR-08 / PR-09. `409 phase-mismatch` if `:phase` is not the current phase, `409 not-adjustment-phase` for movement and retreat phases |
| `POST /games/:id/phases/:phase/resolve` ✔ | Adjudicate; returns `Resolution` (incl. `trace`) and advances the game |
| `PUT  /games/:id/civil-disorder/:power` ✔ | Declare civil disorder for a power from the current phase (PR-16). Stored in `state.civilDisorder` (`since` = current phase), so it is saved, carried into later phases and kept in the history and replay through each phase's starting position. Declaring again changes nothing (`changed: false`) |
| `DELETE /games/:id/civil-disorder/:power` ✔ | Lift civil disorder from the current phase on (e.g. replacement player); phases already resolved keep it in their records. `changed: false` if it was not declared |
| `POST /adjudicate` | Stateless: `{state, orders}` → `Resolution` (sandbox, tests, "what-if") |

Decided behaviour of these routes *(decided)*: move supports are offered only for moves the
supported unit can legally be ordered to make; convoys only for routes that can pass through that
fleet; `default-disbands` answers `409` outside the current adjustment phase; repeating a
civil-disorder declaration or lift returns `200` with `changed: false`; the options,
order-check and `default-disbands` routes call the engine directly (not through the `Adjudicator`
trait), which is the only engine.

Deployment and access *(decided)*: a single-user tool for the GM, run locally on the GM's machine.
The server listens on `127.0.0.1` only (the default `--addr 127.0.0.1:8080`; a non-loopback `--addr`
is refused at startup) and is not exposed to the network. There is no authentication, no roles and no permission checks: every route is open to
the local GM.

Secret simultaneous orders (R-ORD-01) are handled by how the tool is used, not by access control:
the GM enters the orders, and `GET /games/:id` does not return unresolved orders. Whether the GUI
hides one power's orders from the others on screen is a display question for the GUI (§ 7).

## 7. GUI

| Area | Features |
|------|----------|
| Start screen | New game / load a saved game (list or file upload) / replay a game |
| Map | SVG map (`standard.svg`) as the main view, with a switch to the adjacency **graph view** (`standard.graph.svg`) as an auxiliary view; units (A/F shapes, power colours), SC ownership, split coasts, click to select unit/province |
| Order input | Select unit → choose order type → click target(s); shows legal options from the server; text entry as alternative |
| Progress | Current phase, year/season, per-power submission status, SC counts, civil-disorder badges. No timers (R-TURN-05: the GM keeps time outside the tool) |
| GM controls | Resolve phase, declare / lift civil disorder per power (PR-16), review automatically proposed disbands and override them manually (PR-08) |
| Resolution replay | Step through `trace` (prev / next / play): highlight orders, show strengths, cut supports, bounces, dislodgements with rule IDs |
| History / replay | Browse past phases and their maps/orders frame by frame (§ 5); export the save file |

### Implemented so far

The server serves the built GUI (`web/dist`, override with `--web-dir`) at `/` on the same local
address as the API, so one `cargo run` starts everything. During development `npm run dev` in `web/`
runs Vite on 127.0.0.1:5173 and proxies `/games` and `/maps` to the API.

- Start screen: new game, saved-game list, load a save file (optionally under a new ID).
- Board: map view and graph view; SC ownership tint, units, dislodged units; arrows for the orders
  entered (green: valid when submitted, red: invalid, grey: not submitted / other powers).
- Order entry: click a unit (or pick it in the list) → legal orders from the server; click a highlighted
  province to move/retreat; submit per power and see each order's check (valid / holds / ignored with
  reason and rule IDs). Adjustment phase: builds per home SC, required disbands and the automatic
  disbands (PR-08).
- Progress: units, SCs, submission status and civil disorder declare/lift per power.
- Resolve with an inline confirmation, then step through the trace (first / prev / next / last, step
  list, details with rule IDs, orders of the step drawn on the board, board after the last step).
  Reopening a game shows its most recent resolution.
- Retreat phase: dislodged units are drawn next to their province with a dashed red outline; selecting one
  offers its retreat options (highlighted on the board) and disband. Adjustment phase: builds per home SC
  (unit type and, for St. Petersburg, coast), disbands, waive, and the automatic disbands (PR-08).
  Both phases were checked in a browser through a full Spring–Winter cycle.
- History: a **History** tab lists every phase of the game (replay frames, § 5); pick one (or step
  with prev / next) to see that phase's starting position and step through its resolution.
- Japanese / English switch (remembered in the browser).

- Text entry: in the Orders tab, "Enter orders as text" reads typed orders for any powers (preview with
  each order's check or error), then adds them to the drafts or adds and submits them.

Everything listed in this section is implemented; timers are intentionally not part of the tool.

## 8. Development principles

- **Rules are the spec.** Every engine behaviour must trace back to a rule ID or a project ruling (PR-xx).
- **Test first for the engine.** Each rulebook example in [test-cases.md](test-cases.md) and the relevant DATC cases become unit tests.
- **English docs are the source of truth;** Japanese docs mirror them and must be updated in the same change.
