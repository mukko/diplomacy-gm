# Standard Map Reference

Reference data for the standard 1901 Europe map, as used by the rulebook
(see [rules.md § 3](rules.md#3-the-board)). Abbreviations follow the rulebook's list
and are the **canonical province IDs** used by the API (case-insensitive on input,
canonical casing on output).

> **Adjacency data is intentionally not listed here.** The rulebook does not contain an
> adjacency table; it will live in a machine-readable file (planned: `shared/map/standard.json`)
> with unit tests derived from the rulebook examples and the special cases in § 4.

## 1. Totals

| Item | Count |
|------|------:|
| Provinces | 75 (+ impassable Switzerland) |
| Land provinces | 56 (14 inland + 42 coastal) |
| Water provinces | 19 |
| Supply centers | 34 (22 home + 12 neutral) |
| Victory threshold | 18 |

## 2. Provinces

Legend — **Type:** `L` inland, `C` coastal, `W` water. **SC:** ★ = supply center.
**Home:** the power whose home SC it is.

### 2.1 Austria

| Abbr | Name | Type | SC | Home |
|------|------|:----:|:--:|------|
| Boh | Bohemia | L | | |
| Bud | Budapest | L | ★ | Austria |
| Gal | Galicia | L | | |
| Tri | Trieste | C | ★ | Austria |
| Tyr | Tyrolia | L | | |
| Vie | Vienna | L | ★ | Austria |

### 2.2 England

| Abbr | Name | Type | SC | Home |
|------|------|:----:|:--:|------|
| Cly | Clyde | C | | |
| Edi | Edinburgh | C | ★ | England |
| Lvp | Liverpool | C | ★ | England |
| Lon | London | C | ★ | England |
| Wal | Wales | C | | |
| Yor | Yorkshire | C | | |

### 2.3 France

| Abbr | Name | Type | SC | Home |
|------|------|:----:|:--:|------|
| Bre | Brest | C | ★ | France |
| Bur | Burgundy | L | | |
| Gas | Gascony | C | | |
| Mar | Marseilles | C | ★ | France |
| Par | Paris | L | ★ | France |
| Pic | Picardy | C | | |

### 2.4 Germany

| Abbr | Name | Type | SC | Home |
|------|------|:----:|:--:|------|
| Ber | Berlin | C | ★ | Germany |
| Kie | Kiel | C | ★ | Germany |
| Mun | Munich | L | ★ | Germany |
| Pru | Prussia | C | | |
| Ruh | Ruhr | L | | |
| Sil | Silesia | L | | |

### 2.5 Italy

| Abbr | Name | Type | SC | Home |
|------|------|:----:|:--:|------|
| Apu | Apulia | C | | |
| Nap | Naples | C | ★ | Italy |
| Pie | Piedmont | C | | |
| Rom | Rome | C | ★ | Italy |
| Tus | Tuscany | C | | |
| Ven | Venice | C | ★ | Italy |

### 2.6 Russia

| Abbr | Name | Type | SC | Home |
|------|------|:----:|:--:|------|
| Fin | Finland | C | | |
| Lvn | Livonia | C | | |
| Mos | Moscow | L | ★ | Russia |
| Sev | Sevastopol | C | ★ | Russia |
| StP | St. Petersburg | C (NC/SC) | ★ | Russia |
| Ukr | Ukraine | L | | |
| War | Warsaw | L | ★ | Russia |

### 2.7 Turkey

| Abbr | Name | Type | SC | Home |
|------|------|:----:|:--:|------|
| Ank | Ankara | C | ★ | Turkey |
| Arm | Armenia | C | | |
| Con | Constantinople | C | ★ | Turkey |
| Smy | Smyrna | C | ★ | Turkey |
| Syr | Syria | C | | |

### 2.8 Neutral land

| Abbr | Name | Type | SC |
|------|------|:----:|:--:|
| Alb | Albania | C | |
| Bel | Belgium | C | ★ |
| Bul | Bulgaria | C (EC/SC) | ★ |
| Den | Denmark | C | ★ |
| Gre | Greece | C | ★ |
| Hol | Holland | C | ★ |
| Nwy | Norway | C | ★ |
| NAf | North Africa | C | |
| Por | Portugal | C | ★ |
| Rum | Rumania | C | ★ |
| Ser | Serbia | L | ★ |
| Spa | Spain | C (NC/SC) | ★ |
| Swe | Sweden | C | ★ |
| Tun | Tunis | C | ★ |

### 2.9 Water

| Abbr | Name |
|------|------|
| Adr | Adriatic Sea |
| Aeg | Aegean Sea |
| Bal | Baltic Sea |
| Bar | Barents Sea |
| Bla | Black Sea |
| Eas | Eastern Mediterranean |
| Eng | English Channel |
| Bot | Gulf of Bothnia |
| GoL | Gulf of Lyon |
| Hel | Helgoland Bight |
| Ion | Ionian Sea |
| Iri | Irish Sea |
| Mid | Mid-Atlantic Ocean |
| NAt | North Atlantic Ocean |
| Nth | North Sea |
| Nrg | Norwegian Sea |
| Ska | Skagerrak |
| Tyn | Tyrrhenian Sea |
| Wes | Western Mediterranean |

Note: the abbreviation lists in some tools use `Lyo` for Gulf of Lyon, `Mao`/`Nao` for the Atlantic
oceans and `Nth`/`Nwg` for North/Norwegian Sea. **This project uses the rulebook forms above**; the
order parser should accept common aliases (see [adjudication.md § 2](adjudication.md#2-order-parsing-and-validation)).

## 3. Split coasts

| Province | Coasts | Coast IDs |
|----------|--------|-----------|
| Bulgaria | East coast (Black Sea side), South coast (Aegean side) | `Bul/EC`, `Bul/SC` |
| Spain | North coast (Atlantic/Biscay side), South coast (Mediterranean side) | `Spa/NC`, `Spa/SC` |
| St. Petersburg | North coast (Barents side), South coast (Gulf of Bothnia side) | `StP/NC`, `StP/SC` |

A fleet location is always stored with its coast for these provinces; armies never have a coast.

## 4. Special map cases (must be covered by adjacency tests)

| Case | Rule |
|------|------|
| Switzerland | Impassable, not a province. |
| Kiel | Single coast; fleets can go Hol ↔ Kie ↔ Ber/Bal/Hel/Den directly. |
| Constantinople | Single coast; fleets can go Bla ↔ Con ↔ Aeg/Bul/Smy/Ank. |
| Denmark–Sweden | Directly adjacent (armies and fleets). |
| Baltic–Skagerrak | **Not** adjacent. |
| Denmark–Berlin | **Not** adjacent. |
| Sweden–Norway | Share a coastline (fleet move/support possible). |
| Rome | Fleet: Tus, Nap, Tyn only (not Ven, not Apu). |
| English Channel | Adjacent to Iri, Wal, Lon, Bel, Pic, Bre, Nth, Mid (rulebook Diagram 2). |
| Paris (army) | Adjacent to Bre, Pic, Bur, Gas (rulebook Diagram 1). |
| Spain NC | Not adjacent to Wes, GoL, Mar. |

## 5. Starting units

See [rules.md § 15](rules.md#15-starting-position-spring-1901). Russia's fleet starts at `StP/SC`.

## 6. Unit colours (rulebook)

| Power | Colour |
|-------|--------|
| Austria | red |
| England | dark blue |
| France | light blue |
| Germany | black |
| Italy | green |
| Russia | white |
| Turkey | yellow |

The GUI may use different colours for readability, but should keep this mapping recognisable.

## 7. Drawing the map (GUI assets)

The GUI has two views of the board, built from the same data:

| View | File | Use |
|------|------|-----|
| **Map** (main) | `shared/map/standard.svg` | The board players look at. |
| **Graph** (auxiliary) | `shared/map/standard.graph.svg` | Provinces as nodes and adjacencies as edges (army, fleet or both); split coasts are separate nodes. Used to check adjacency and to debug resolutions. |

- `standard.svg` is adapted from "Diplomacy.svg" by Martin Asal (Wikimedia Commons, **CC BY-SA 3.0**).
  The adapted map files are under the same licence; the rest of the repository is not affected.
  See [`shared/map/CREDITS.md`](../../shared/map/CREDITS.md).
- Structure of `standard.svg`: one `<g id="prov-<ID>" class="province land|coast|sea" data-province="<ID>">`
  per province (plus `prov-Swi`, impassable); split coasts are separate areas
  `id="coast-<ID>-<NC|SC|EC>"`; supply-centre markers `id="sc-<ID>"`; labels `id="label-<ID>"`.
  The GUI colours provinces and SC markers by owner and draws units itself.
- `standard.layout.json`: for each province the position of a unit, its label and its SC marker; for each
  split coast the position of a fleet there. One province holds at most one unit (R-UNIT-03), so the army
  position and a coast position may coincide.
- All three files are **generated** by `tools/map/build_map.py` from the pinned source file
  (`shared/map/source/Diplomacy.commons.svg`) and `standard.json`; do not edit them by hand. Re-run
  `python3 -I tools/map/build_map.py` after changing `standard.json`. Tests: `shared/map/layout.test.ts`.
- Which part of a split province is which coast was checked visually: Bulgaria's northern part
  (Black Sea side) is EC, Spain's and St. Petersburg's northern parts are NC.
