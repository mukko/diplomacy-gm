# Architecture (draft)

> Status: **draft / proposal.** Technology choices marked *(proposed)* are not final.

## 1. Goal

A game-master tool for *Diplomacy* that

- stores game state and history,
- lets players (or a GM) enter orders on an interactive map,
- adjudicates orders according to the official rules ([rules.md](rules.md), [adjudication.md](adjudication.md)),
- returns **both the result and the reasoning steps**, so the GUI can replay resolution step by step.

Negotiation itself happens outside the tool (in person or elsewhere). The tool does not enforce agreements (R-GEN-05).

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
| `server/` | REST API, game lifecycle, persistence, calls adjudicator | TypeScript, Node.js (Hono or Fastify) |
| `server/engine/` (or `engine/`) | Pure adjudicator: `adjudicate(state, orders) → Resolution`. No I/O. | TypeScript, Vitest; DATC + [test-cases.md](test-cases.md) |
| `web/` | Map rendering, order entry, progress, trace replay | React + Vite, SVG map |
| `shared/` | Types (`Order`, `GameState`, `Resolution`, `TraceStep`), map data (`standard.json`: provinces, coasts, adjacency, SVG coordinates) | TypeScript + JSON |
| Persistence | Games, phases, submitted orders, resolutions | SQLite to start *(proposed)* |

The adjudicator is the authority: the GUI may pre-validate for usability, but the server re-validates everything.

## 3. Game flow

```
create game ─▶ S1901M ─▶ [S1901R] ─▶ F1901M ─▶ [F1901R] ─▶ W1901A ─▶ S1902M ─▶ …
                 │            │          │           │           │
              orders      retreats    orders     retreats    builds/disbands
                 └── resolve ──┘                     └─ SC update + victory check (PR-11)
```

- Retreat phases (`R`) are skipped when no unit has a legal retreat.
- Adjustment phase (`A`) is skipped when no power needs to build or disband.
- A phase is resolved when the GM triggers it (or all powers have submitted, depending on settings).
  Powers that submitted nothing are treated per R-ORD-03 / PR-09; civil disorder applies only when declared by the GM (PR-16).

### Phase ID format

`<S|F|W><year><M|R|A>` — e.g. `S1901M`, `F1901R`, `W1901A`.

## 4. Core data model (sketch)

```jsonc
// GameState
{
  "gameId": "g_123",
  "variant": "standard",          // or "2p-ww1", "6p-no-italy", …
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

## 5. API (sketch)

| Method & path | Purpose |
|---------------|---------|
| `GET  /maps/standard` | Map data for rendering and client-side hints |
| `POST /games` | Create a game (variant, power assignment, settings) |
| `GET  /games/:id` | Current state and phase |
| `GET  /games/:id/history` | All phases with orders and resolutions |
| `GET  /games/:id/units/:prov/options` | Legal orders for a unit (helps order entry UI) |
| `PUT  /games/:id/phases/:phase/orders/:power` | Submit/replace a power's orders; returns per-order validation. The GM may also call this for any power (e.g. manual disbands for a power in civil disorder, PR-08) |
| `GET  /games/:id/phases/:phase/default-disbands` | GM only: automatically proposed disbands per power (R-CD-03 / PR-08), for review before resolving |
| `POST /games/:id/phases/:phase/resolve` | Adjudicate; returns `Resolution` (incl. `trace`) and advances the game |
| `GET  /games/:id/phases/:phase/resolution` | Fetch a past resolution for replay |
| `PUT  /games/:id/civil-disorder/:power` | GM only: declare civil disorder for a power from the current phase (PR-16) |
| `DELETE /games/:id/civil-disorder/:power` | GM only: lift civil disorder (e.g. replacement player) |
| `POST /adjudicate` | Stateless: `{state, orders}` → `Resolution` (sandbox, tests, "what-if") |

Secret simultaneous orders: until a phase is resolved, a power's orders are only visible to that power and the GM.
(Authentication/roles: TBD.)

## 6. GUI

| Area | Features |
|------|----------|
| Map | SVG map; units (A/F shapes, power colours), SC ownership, split coasts, click to select unit/province |
| Order input | Select unit → choose order type → click target(s); shows legal options from the server; text entry as alternative |
| Progress | Current phase, year/season, per-power submission status, SC counts, timers (R-TURN-05), civil-disorder badges |
| GM controls | Resolve phase, declare / lift civil disorder per power (PR-16), review automatically proposed disbands and override them manually (PR-08) |
| Resolution replay | Step through `trace` (prev / next / play): highlight orders, show strengths, cut supports, bounces, dislodgements with rule IDs |
| History | Browse past phases and their maps/orders |

## 7. Development principles

- **Rules are the spec.** Every engine behaviour must trace back to a rule ID or a project ruling (PR-xx).
- **Test first for the engine.** Each rulebook example in [test-cases.md](test-cases.md) and the relevant DATC cases become unit tests.
- **English docs are the source of truth;** Japanese docs mirror them and must be updated in the same change.
