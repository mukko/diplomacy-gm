# CLAUDE.md

Context for AI agents and developers working in this repository.

## What this is

A *Diplomacy* game-master tool: an API server (order validation + adjudication, returns results and a
step-by-step trace) and a web GUI (map, progress, order input, step replay). See
[docs/en/architecture.md](docs/en/architecture.md).

## Rules are the specification

- Read [docs/en/rules.md](docs/en/rules.md) before touching game logic. Every rule has an ID
  (`R-STD-01`, `R-CUT-01`, …). Reference these IDs in code comments, tests and trace output.
- Where the rulebook is ambiguous, follow the project rulings `PR-xx` in
  [docs/en/adjudication.md § 5](docs/en/adjudication.md#5-project-rulings-ambiguities-and-gaps).
  Do not invent new behaviour silently — add a PR entry instead (marked **(open)** if unconfirmed).
- The rulebook examples in [docs/en/test-cases.md](docs/en/test-cases.md) must all pass as engine tests.
- Use the terms in [docs/glossary.md](docs/glossary.md); Japanese text must use its Japanese terms.
- Province IDs are the rulebook abbreviations listed in [docs/en/map.md](docs/en/map.md).

## Documentation policy

- English docs (`docs/en/`) are the **source of truth**. Japanese docs (`docs/ja/`) mirror them
  with the same structure, rule IDs and anchors' meaning. Update both in the same change.
- Do not copy text from the official rulebook verbatim; paraphrase. Do not commit the rulebook PDF.

## Key gotchas (frequently mis-implemented)

- Support is cut by an attack from anywhere **except** the province the support is directed into — but
  is always cut if the supporter is dislodged (R-CUT-01/02).
- A power cannot dislodge its own unit; support from the defender's own power does not count for
  dislodging it, but still counts for bouncing others (R-SLF-01/02, PR-02).
- A dislodged unit has no effect on the province its attacker came from (R-STD-07).
- Retreats cannot go to the attacker's origin or to a province vacated by a standoff (R-RET-03).
- Split coasts: Bul (EC/SC), Spa (NC/SC), StP (NC/SC). Kiel and Constantinople have one coast.
