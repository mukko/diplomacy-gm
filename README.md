# diplomacy-gm

[日本語版 README](README.ja.md)

A game-master tool for the board game ***Diplomacy***: an **API server** that adjudicates orders
according to the official rules and returns results with step-by-step reasoning, and a **GUI**
that shows the map and game progress, accepts order input, and replays each resolution step.

> Status: **design / documentation phase.** No code yet.

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
