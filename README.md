# diplomacy-gm

[日本語版 README](README.ja.md)

A game-master tool for the board game ***Diplomacy***, run by the GM on their own computer.
The GM enters the players' orders; the tool adjudicates them according to the official rules and
shows the result on the map — one unit at a time, with the reasoning and rule references behind
every outcome.

> Unofficial fan project. Not affiliated with or endorsed by Hasbro, Avalon Hill or Wizards of the
> Coast. *Diplomacy* is a trademark of Hasbro.

## Features

- **Rules engine** for the standard 7-player game: movement, retreats and adjustments, following the
  official rulebook (5th edition, 2008). Checked against every example in the rulebook and the
  DATC adjudicator test cases for movement, retreats and builds (6.A–6.I) (see [docs/en/test-cases.md](docs/en/test-cases.md)).
- **Map GUI** in the browser (Japanese / English): units, supply-center ownership, order arrows, and
  an adjacency **graph view**.
- **Order entry** by clicking units on the map (only legal orders are offered) or by **typing**
  orders in rulebook notation (`France: A Par-Bur, A Mar S A Par-Bur`). Each order is checked and
  invalid ones are explained.
- **Resolution replay**: step through a movement phase unit by unit on the map, or switch between
  the board before and after the phase. Each step shows why it succeeded or failed (strengths, cut
  supports, rule IDs).
- **History** of every phase, **save files** (one JSON file per game, saved automatically,
  downloadable and loadable), **civil disorder** handling, and an emergency **manual edit** of the
  board.
- Runs locally: the server only accepts connections from the same computer. No account, no
  internet connection needed.

## Getting started

### Option A — ready-to-run package (no development tools needed)

Build a package once (see [Building packages](#building-packages)) or take one from the GitHub
Actions artifacts, then:

- **Windows**: unzip `diplomacy-gm-x86_64-pc-windows-*.zip` and double-click `diplomacy-gm.exe`.
  A console window opens (that is the server) and the GUI opens in your browser. Close the window to
  stop. Games are saved in `data\games` next to the program. Windows may warn about an unsigned
  program the first time: choose "More info" → "Run anyway".
- **macOS / Linux**: unzip and run `./diplomacy-gm --open` in a terminal. Games are saved in
  `data/games` under the current folder. macOS may refuse a downloaded unsigned program; allow it in
  System Settings → Privacy & Security, or run `xattr -d com.apple.quarantine diplomacy-gm`.

### Option B — from source

Requires [Node.js](https://nodejs.org/) (npm; version 22 or later recommended) and
[Rust](https://www.rust-lang.org/) (cargo, stable).

```bash
git clone <this repository>
cd diplomacy-gm
./start.sh
```

`start.sh` installs the GUI's dependencies on the first run, builds the GUI and the server when
something changed, starts the server on <http://127.0.0.1:8080/> and opens it in your browser.
Games are saved in `data/games/`. Stop with Ctrl+C.

| Option | Meaning |
|--------|---------|
| `--port 9000` | use another port |
| `--data-dir DIR` | keep save files in `DIR` |
| `--no-open` | do not open the browser |
| `--rebuild` | rebuild the GUI and the server from scratch |

## Using the application

### 1. Start screen

- **New game** — enter a title and create it (standard map, Spring 1901).
- **Saved games** — reopen a game; it continues where you left off.
- **Load a save file** — restore a game from a downloaded save file (optionally under a new ID).
- The language switch (top right) toggles Japanese / English.

### 2. Entering orders (Orders tab)

- **Click a unit** on the map (or in the list). Its legal orders appear as buttons: hold, moves,
  supports, convoys. Provinces it can move to are highlighted — click one to order the move.
- Or open **Enter orders as text** and type orders for any powers, one per line or separated by
  `,` / `;`:

  ```
  France: A Par-Bur, A Mar S A Par-Bur, F Bre-Mid
  Germany:
  A Mun-Bur
  F Kie-Den
  F Nth C A Lon-Nwy
  A Lon-Nwy via convoy
  ```

  `France:` at the start of a line applies to that line; a power name alone applies to the lines that
  follow. Province abbreviations, common aliases and full names are accepted. **Read** shows how each
  line was understood; **Add to drafts** or **Add and submit**.
- **Submit** each power's orders. Each order is then marked *valid*, *invalid → holds* or *ignored*,
  with the reason and rule IDs. Units without an order hold.

### 3. Resolving a phase

Press **Resolve this phase** and confirm. The **Resolution** tab opens:

- **Unit by unit** (default for movement phases): units are taken power by power. Each step shows the
  unit's order and outcome (succeeded / failed / holds / dislodged, with the reason). The board is
  cumulative: units already shown stand where they ended up. "How it was decided" lists the
  adjudicator's reasoning for that unit. Use the buttons or the ← / → keys.
- **Before** / **After**: the board before the phase, or after all units moved at once, with every
  order drawn and coloured by outcome (green succeeded, red failed).

Retreat phases (dislodged units retreat or disband) and adjustment phases (builds and disbands in
winter) are entered the same way in the Orders tab. Required disbands that are not ordered are chosen
automatically and shown in advance.

### 4. Other tools

| Where | What |
|-------|------|
| **Progress** tab | units and supply centers per power, who has submitted, declare / lift **civil disorder** |
| **History** tab | every past phase: its starting board and its resolution, step by step |
| **Map / Graph** switch | the map, or the adjacency graph (useful to check moves and supports) |
| **Edit manually** (emergency) | change units and supply-center owners directly on a draft of the board, then **Confirm** or **Cancel**. Confirming discards the orders already submitted for that phase; the change is recorded in the history |
| **Download save file** | the complete game as one JSON file (also saved automatically in the data folder) |

## Building packages

`scripts/package.sh [TARGET]` builds one executable with the GUI built in and zips it, with a README,
the license, the map credits and the third-party licenses, into `dist/`.

```bash
scripts/package.sh                      # for this computer, e.g. dist/diplomacy-gm-aarch64-apple-darwin.zip
npm run package:windows                 # Windows x64 from macOS/Linux (= scripts/package.sh x86_64-pc-windows-gnu)
```

Cross-compiling for Windows needs, once: `brew install mingw-w64` (macOS; about 1.4 GB, or your
Linux package manager) and `rustup target add x86_64-pc-windows-gnu`. On Windows itself, run
`scripts/package.sh x86_64-pc-windows-msvc` from Git Bash.
[.github/workflows/package.yml](.github/workflows/package.yml) runs the tests and builds the Windows
and macOS packages on GitHub Actions (manually, or when a `v*` tag is pushed).

## Development

```bash
npm install && npm test                 # map data tests (shared/map)
cd server && cargo test                 # rules engine and API tests
cd web && npm install && npx vitest run # GUI tests
cd web && npm run dev                   # GUI dev server on http://127.0.0.1:5173 (needs the API on :8080)
cd server && cargo run -- --data-dir data/games   # API + built GUI on http://127.0.0.1:8080
```

| Path | Contents |
|------|----------|
| `server/` | Rust (axum) API server and the rules engine (`src/engine/`) |
| `web/` | React + Vite GUI |
| `shared/map/` | map data (provinces, adjacency) and the drawing assets |
| `docs/` | rules specification, adjudication spec, test cases, architecture (EN / JA) |
| `scripts/`, `packaging/` | packaging |

| Document | English | 日本語 |
|----------|---------|--------|
| Rules specification (with rule IDs) | [docs/en/rules.md](docs/en/rules.md) | [docs/ja/rules.md](docs/ja/rules.md) |
| Map reference | [docs/en/map.md](docs/en/map.md) | [docs/ja/map.md](docs/ja/map.md) |
| Adjudication spec & project rulings | [docs/en/adjudication.md](docs/en/adjudication.md) | [docs/ja/adjudication.md](docs/ja/adjudication.md) |
| Rulebook examples as test cases | [docs/en/test-cases.md](docs/en/test-cases.md) | [docs/ja/test-cases.md](docs/ja/test-cases.md) |
| Architecture | [docs/en/architecture.md](docs/en/architecture.md) | [docs/ja/architecture.md](docs/ja/architecture.md) |
| Glossary (EN/JA) | [docs/glossary.md](docs/glossary.md) | ← |

English documents are the source of truth; Japanese documents mirror them.

## License and credits

- **Code and documentation**: [MIT License](LICENSE), except the files below.
- **Map**: `shared/map/standard.svg`, `standard.layout.json`, `standard.graph.svg` and
  `shared/map/source/` are adapted from "Diplomacy.svg" by Martin Asal (Wikimedia Commons) and are
  licensed under [CC BY-SA 3.0](https://creativecommons.org/licenses/by-sa/3.0/). See
  [shared/map/CREDITS.md](shared/map/CREDITS.md).
- **Rules**: the documents paraphrase the *Diplomacy* Rulebook, 5th edition (2008), Avalon Hill /
  Wizards of the Coast (<https://media.wizards.com/2015/downloads/ah/diplomacy_rules.pdf>). The
  rulebook itself is not included.
- **Third-party software**: the packages include `THIRD_PARTY_LICENSES.txt` with the licenses of the
  Rust crates and JavaScript libraries built into the program.
- *Diplomacy* is a trademark of Hasbro. This is an unofficial fan project.
