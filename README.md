# diplomacy-gm

[日本語版 README](README.ja.md)

A game-master tool for the board game ***Diplomacy***: an **API server** that adjudicates orders
according to the official rules and returns results with step-by-step reasoning, and a **GUI**
that shows the map and game progress, accepts order input, and replays each resolution step.

> Status: **early development.** Map data, the Rust API server (game lifecycle, JSON save
> files, replay), the adjudicator (movement, retreat and adjustment phases, with a
> step-by-step trace) and a first version of the web GUI (Japanese/English) exist.

## Development

```bash
cd web && npm install && npm run build            # build the GUI once (web/dist)
cd server && cargo run -- --data-dir data/games   # API + GUI on http://127.0.0.1:8080
cd server && cargo test                           # server tests
npm install && npm test                           # map data tests (shared/map)
cd web && npm run dev                             # GUI dev server on http://127.0.0.1:5173 (API must run)
```

Open <http://127.0.0.1:8080/> in a browser. The server only accepts local connections.

Each game is saved as one JSON file in the data directory; see
[architecture.md § 5](docs/en/architecture.md#5-persistence-save-files).

## Documentation

| Document | English | 日本語 |
|----------|---------|--------|
| Rules specification (with rule IDs) | [docs/en/rules.md](docs/en/rules.md) | [docs/ja/rules.md](docs/ja/rules.md) |
| Map reference | [docs/en/map.md](docs/en/map.md) | [docs/ja/map.md](docs/ja/map.md) |
| Adjudication spec & project rulings | [docs/en/adjudication.md](docs/en/adjudication.md) | [docs/ja/adjudication.md](docs/ja/adjudication.md) |
| Rulebook examples as test cases | [docs/en/test-cases.md](docs/en/test-cases.md) | [docs/ja/test-cases.md](docs/ja/test-cases.md) |
| Architecture (draft) | [docs/en/architecture.md](docs/en/architecture.md) | [docs/ja/architecture.md](docs/ja/architecture.md) |
| Glossary (EN/JA) | [docs/glossary.md](docs/glossary.md) | ← |

English documents are the source of truth; Japanese documents mirror them.

## Rules source

*Diplomacy* Rulebook, 5th edition (2008), Avalon Hill / Wizards of the Coast —
<https://media.wizards.com/2015/downloads/ah/diplomacy_rules.pdf>.
The documents here paraphrase the rules for implementation purposes; the rulebook PDF itself is
not included in this repository. *Diplomacy* is a trademark of Hasbro.

## Map credits

The board map is adapted from "Diplomacy.svg" by Martin Asal (Wikimedia Commons), licensed under
CC BY-SA 3.0; the adapted map files are under the same licence. See [shared/map/CREDITS.md](shared/map/CREDITS.md).
