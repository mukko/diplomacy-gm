# Rulebook Examples as Test Cases

Every diagram in the rulebook, restated as **orders + expected result**, for use as adjudicator
unit tests. Only the units listed are on the board. Case IDs: `RB-Dnn` (diagram nn).
Rule IDs refer to [rules.md](rules.md); `PR-xx` to [adjudication.md § 5](adjudication.md#5-project-rulings-ambiguities-and-gaps).

Legend: ✅ succeeds · ❌ fails · 💥 dislodged · ✂ support cut

## 1. Movement / map

| ID | Setup / orders | Expected | Rules |
|----|----------------|----------|-------|
| RB-D01 | A Par | Legal moves: Bre, Pic, Bur, Gas | R-MOV-01 |
| RB-D02 | F Eng | Legal moves: Iri, Wal, Lon, Bel, Pic, Bre, Nth, Mid | R-MOV-02 |
| RB-D03 | F Rom | Legal: Tus, Nap, Tyn. Illegal: Ven, Apu | R-MAP-05 |
| RB-SPA | F Spa/NC → Wes, GoL, Mar | all illegal | R-MAP-06 |
| RB-BUL | F Con-Bul (no coast) | illegal → Hold; `F Con-Bul EC` and `F Con-Bul SC` legal | R-MAP-07 |

## 2. Standoffs

| ID | Orders | Expected | Rules |
|----|--------|----------|-------|
| RB-D04 | Ger: A Ber-Sil · Rus: A War-Sil | both ❌, Sil empty (standoff) | R-STD-01 |
| RB-D04b | as D04 + a unit holding in Sil | both ❌, A Sil stays | R-STD-02 |
| RB-D05 | Ger: F Kie-Ber, A Ber-Pru · Rus: A Pru H | nothing moves | R-STD-03 |
| RB-D06 | Ger: F Ber-Pru, A Pru-Ber | neither moves | R-STD-04 |
| RB-D07 | Eng: A Hol-Bel, F Bel-Nth · Fra: F Nth-Hol | all ✅ (rotation) | R-STD-05 |

## 3. Support

| ID | Orders | Expected | Rules |
|----|--------|----------|-------|
| RB-D08 | Fra: A Mar-Bur, A Gas S A Mar-Bur · Ger: A Bur H | Mar-Bur ✅, A Bur 💥 | R-SUP-01 |
| RB-D09 | Ger: A Sil-Pru, F Bal S A Sil-Pru · Rus: A Pru H | Sil-Pru ✅, A Pru 💥 | R-SUP-02 |
| RB-D10 | Fra: F GoL-Tyn, F Wes S F GoL-Tyn · Ita: F Nap-Tyn, F Rom S F Nap-Tyn | both ❌ (2 = 2), Tyn empty | R-STD-01 |
| RB-D11 | Fra: F GoL-Tyn, F Wes S F GoL-Tyn · Ita: F Tyn H, F Rom S F Tyn | GoL-Tyn ❌ (2 vs 2) | R-SUP-05 |
| RB-SUPMISMATCH | A Boh S A Mun-Sil · A Mun-Tyr | support void; does **not** become hold support | R-SUP-06 |
| RB-D12 | Aus: A Boh-Mun, A Tyr S A Boh-Mun · Ger: A Mun-Sil, A Ber S A Mun-Sil · Rus: A War-Sil, A Pru S A War-Sil | Boh-Mun ✅, A Mun 💥; Mun-Sil ❌ and War-Sil ❌ (2 = 2, Sil empty) | R-STD-06 |
| RB-D13 | Tur: A Bul-Rum · Rus: A Rum-Bul, A Ser S A Rum-Bul, A Sev-Rum | Rum-Bul ✅, A Bul 💥; Sev-Rum ✅ | R-STD-07, PR-01 |
| RB-D14 | Tur: A Bul-Rum, F Bla S A Bul-Rum · Rus: A Rum-Bul, A Gre S A Rum-Bul, A Ser S A Rum-Bul, A Sev-Rum | Rum-Bul ✅ (3 vs 2), A Bul 💥; Sev-Rum ✅ | R-STD-07 |
| RB-D13r | D13 without A Sev-Rum | Rum ends empty but is **not** a standoff province (R-RET-04). A Bul still cannot retreat there because it is the attacker's origin. | R-RET-03/04 |

## 4. Cutting support

| ID | Orders | Expected | Rules |
|----|--------|----------|-------|
| RB-D15 | Ger: A Pru-War, A Sil S A Pru-War · Rus: A War H, A Boh-Sil | Sil ✂; Pru-War ❌ (1 vs 1); Boh-Sil ❌ | R-CUT-01 |
| RB-D16 | Ger: A Pru-War, A Sil S A Pru-War · Rus: A War-Sil | Sil support **not** cut; War-Sil ❌; Pru-War ✅ (2 vs 1), A War 💥 | R-CUT-01 |
| RB-D17 | Ger: F Ber-Pru, A Sil S F Ber-Pru · Rus: A Pru-Sil, A War S A Pru-Sil, F Bal-Pru | Pru-Sil ✅, A Sil 💥 → Sil ✂; Ber-Pru ❌ and Bal-Pru ❌ (1 = 1), Pru empty | R-CUT-02 |
| RB-D18 | Ger: A Ber H, A Mun-Sil · Rus: A Pru-Ber, A Sil S A Pru-Ber, A Boh-Mun, A Tyr S A Boh-Mun | Mun-Sil ❌ but Sil ✂; Pru-Ber ❌; Boh-Mun ✅, A Mun 💥 | R-CUT-03 |

## 5. Convoy

| ID | Orders | Expected | Rules |
|----|--------|----------|-------|
| RB-D19 | Eng: A Lon-Nwy, F Nth C A Lon-Nwy | ✅ | R-CVY-03 |
| RB-CVYMISMATCH | A Rum-Arm · F Bla C A Rum-Ank | convoy void. Rum and Arm are not adjacent, so A Rum-Arm needs a convoy and has none: it fails and the army stays in Rum. (With a land destination, e.g. A Rum-Sev, the move would be judged on its own merits.) | R-CVY-04 |
| RB-SUPCVY | Eng: A Pic-Bre, A Lon S A Pic-Bre, F Eng C A Lon S A Pic-Bre · Fra: F Bre H | Lon support and Eng convoy invalid; Pic-Bre ❌ (1 vs 1) | R-SUP-07 |
| RB-D20 | Eng: A Lon-Tun, F Eng C A Lon-Tun, F Mid C A Lon-Tun · Fra: F Wes C A Lon-Tun | ✅ (army reaches Tun) | R-CVY-03 |
| RB-D21 | Fra: A Spa-Nap, F GoL C A Spa-Nap, F Tyn C A Spa-Nap · Ita: F Ion-Tyn, F Tun S F Ion-Tyn | F Tyn 💥; convoy fails; A Spa stays | R-CVY-05 |

## 6. Self-dislodgement and self-standoff

| ID | Orders | Expected | Rules |
|----|--------|----------|-------|
| RB-D22 | Fra: A Par-Bur, A Mar S A Par-Bur, A Bur H | nothing moves | R-SLF-01 |
| RB-D23 | Fra: A Par-Bur, A Bur-Mar · Ger: A Ruh S A Par-Bur · Ita: A Mar-Bur | nothing moves; French A Bur not dislodged | R-SLF-01 |
| RB-D24 | Ger: A Ruh-Bur, A Mun H · Fra: A Par S A Ruh-Bur, A Bur H | Ruh-Bur ❌ (French support not counted) | R-SLF-02 |
| RB-D24b | as D24 but Ger: A Mun S A Ruh-Bur | Ruh-Bur ✅, French A Bur 💥 | R-SLF-02 |
| RB-D25 | Ger: A Mun-Tyr, A Ruh-Mun, A Sil-Mun · Aus: A Tyr-Mun, A Boh S A Sil-Mun | nothing moves | R-SLF-01 |
| RB-D26 | Eng: F Den-Kie, F Nth-Den, F Hel S F Nth-Den · Rus: A Ber-Kie, F Ska-Den, F Bal S F Ska-Den | nothing moves (Nth-Den bounces Ska-Den) | R-SLF-03, PR-02 |
| RB-D27 | Aus: A Ser-Bud, A Vie-Bud · Rus: A Gal S A Ser-Bud | Ser-Bud ✅ | R-SLF-04 |
| RB-D27b | as D27 + Aus A Bud H instead of empty Bud | Ser-Bud ❌ (would dislodge own unit) | R-SLF-04 |
| RB-CUTOWN | Own unit attacks own supporting unit | support not cut | R-CUT-04 |

## 7. Advanced convoy cases

| ID | Orders | Expected | Rules |
|----|--------|----------|-------|
| RB-D28 | Eng: A Lon-Bel, F Nth C A Lon-Bel · Fra: A Bel-Lon, F Eng C A Bel-Lon | both ✅ (swap via convoy) | R-CVY-07 |
| RB-D29 | Eng: A Lon-Bel, F Eng C A Lon-Bel, F Nth C A Lon-Bel · Fra: F Bre-Eng, F Iri S F Bre-Eng | F Eng 💥; A Lon-Bel ✅ via Nth | R-CVY-08 |
| RB-D30 | Fra: A Tun-Nap, F Tyn C A Tun-Nap · Ita: F Ion-Tyn, F Nap S F Ion-Tyn | F Tyn 💥; convoy fails; Nap support **not** cut | R-CVY-09 |
| RB-D31 | Fra: A Tun-Nap, F Tyn C A Tun-Nap, F Ion C A Tun-Nap · Ita: F Rom-Tyn, F Nap S F Rom-Tyn | Nap ✂ (route via Ion survives); Rom-Tyn ❌ (1 vs 1); Tun-Nap ❌ (1 vs 1) | R-CVY-10 |
| RB-D32 | as D31 + Fra: A Apu S A Tun-Nap | F Nap 💥 (2 vs 1); army lands in Nap; Rom-Tyn ❌ | R-CVY-10 |
| RB-D32b | as D32 but Italy reversed: F Nap-Tyn, F Rom S F Nap-Tyn | F Tyn 💥, F Nap moves to Tyn (rulebook note) | R-CVY-05 |
| RB-LANDCVY1 | Army could go by land or convoy; ≥1 convoying fleet is its owner's | convoy used | R-CVY-11 |
| RB-LANDCVY2 | Same, all convoying fleets foreign, no "via convoy" | land route used | R-CVY-11 |

## 8. Retreats

| ID | Situation | Expected | Rules |
|----|-----------|----------|-------|
| RB-RET1 | D12 result: A Mun dislodged from Boh | cannot retreat to Boh (attacker origin) or Sil (standoff) | R-RET-03 |
| RB-RET2 | Two units retreat to the same province | both disbanded | R-RET-06 |
| RB-RET3 | No legal retreat | disbanded immediately, no retreat order needed | R-RET-05 |

## 9. Sample game (integration test)

The rulebook's sample game (Spring 1901 – Fall 1902) from the standard start. Use it as a
multi-phase regression test. Orders are given in rulebook notation; expected outcomes are summarised.

### Spring 1901

```
Austria: A Vie-Tri, A Bud-Gal, F Tri-Alb
England: A Lvp-Yor, F Lon-Nth, F Edi-Nrg
France:  A Par-Bur, A Mar-Spa, F Bre-Pic
Germany: A Ber-Kie, A Mun-Ruh, F Kie-Den
Italy:   A Ven-Pie, A Rom-Ven, F Nap-Ion
Russia:  A Mos-Ukr, A War-Gal, F StP-Bot, F Sev-Bla
Turkey:  A Con-Bul, A Smy-Con, F Ank-Bla
```
Expected: all succeed except F Sev-Bla / F Ank-Bla (standoff) and A Bud-Gal / A War-Gal (standoff). No retreats.
(Note: `F StP-Bot` starts from `StP/SC`.)

### Fall 1901

```
Austria: A Tri H, A Bud-Ser, F Alb-Gre
England: A Yor-Nwy, F Nth C A Yor-Nwy, F Nrg-Bar
France:  A Bur-Mar, A Spa-Por, F Pic-Bel
Germany: A Kie-Hol, A Ruh-Bel, F Den H
Italy:   A Ven H, A Pie-Mar, F Ion-Tun
Russia:  A Ukr S F Sev-Rum, A War-Gal, F Bot-Swe, F Sev-Rum
Turkey:  A Bul-Ser, A Con-Bul, F Ank-Bla
```
Expected: moves to Bel, Mar, Ser bounce; A Con-Bul fails (Bul not vacated). Others succeed. No retreats.
Adjustments: Eng +1, Tur +1, Aus +1, Ita +1, Fra +1, Rus +2, Ger +2.
Builds: Eng F Edi · Ger F Kie, A Mun · Rus A StP, A Sev · Tur A Smy · Aus A Vie · Ita F Nap · Fra F Mar.

### Spring 1902

```
Austria: A Tri-Bud, A Vie-Bud, A Bud-Ser, F Gre H
England: A Nwy-StP, F Nth-Nwy, F Bar S A Nwy-StP, F Edi-Nth
France:  A Bur S F Pic-Bel, A Por-Spa, F Pic-Bel, F Mar H
Germany: A Hol-Bel, A Ruh S A Hol-Bel, A Mun-Bur, F Den H, F Kie-Hol
Italy:   A Ven H, A Pie-Mar, F Tun-Wes, F Nap-Tyn
Russia:  A Ukr S F Rum, A Gal-Bud, A StP-Nwy, A Sev S F Rum, F Swe S A StP-Nwy, F Rum H
Turkey:  A Bul-Rum, A Con-Bul, A Smy-Arm, F Bla S A Bul-Rum
```
Expected: Nwy-StP / StP-Nwy fail (2 vs 2 head-to-head), so F Nth-Nwy and F Edi-Nth fail.
Tri/Vie/Gal-Bud all bounce; A Bud-Ser ✅. Bul-Rum ❌ (2 vs 3); Con-Bul ❌; Smy-Arm ✅.
Mun-Bur ❌ but cuts A Bur's support → Hol-Bel ✅ (2 vs 1), Pic-Bel ❌; F Kie-Hol ✅.
Pie-Mar ❌; Tun-Wes ✅; Nap-Tyn ✅; Por-Spa ✅. No retreats.

### Fall 1902

```
Austria: A Vie-Gal, A Tri-Bud, A Ser S Turkish A Bul-Rum, F Gre H
England: A Nwy-StP, F Bar S A Nwy-StP, F Nth-Nwy, F Edi-Nth
France:  A Bur-Bel, F Pic S A Bur-Bel, A Spa S F Mar, F Mar S A Spa
Germany: A Ruh-Bur, A Mun S A Ruh-Bur, A Bel S A Ruh-Bur, F Den-Swe, F Hol S A Bel
Italy:   A Ven-Pie, A Pie-Mar, F Wes-NAf, F Tyn-GoL
Russia:  A StP-Nwy, F Swe S StP-Nwy, F Rum S A Sev, A Sev S F Rum, A Gal S F Rum, A Ukr S A Sev
Turkey:  A Bul-Rum, A Con-Bul, A Arm-Sev, F Bla S A Bul-Rum
```
Expected:
- Cut supports: F Swe (by Den), F Mar (by Pie), A Sev (by Arm), A Gal (by Vie), F Rum (by Bul). A Bel **not** cut (attacked from Bur, its support target).
- Mar and Sev hold off their attackers; Vie-Gal ❌; Ven-Pie ❌; Pie-Mar ❌.
- Bur-Bel ❌ (1 vs 2). Ruh-Bur ✅ (2 vs 1), French A Bur 💥.
- Bul-Rum ✅, Russian **F** Rum 💥 (no retreat possible → disbanded); Con-Bul ✅.
- Nwy-StP ✅ (Swe support cut), Russian A StP 💥; F Nth-Nwy ✅; F Edi-Nth ✅.
- Den-Swe ❌. Wes-NAf ✅, Tyn-GoL ✅.
- Retreats: Rus A StP-Mos, Fra A Bur-Gas.
- Adjustments: Russia 4 SCs / 5 units → disbands A Gal. Builds: Ger F Kie, Tur F Smy, Aus A Tri, Fra A Par, Eng F Lon. Italy unchanged.

### Known errata in the rulebook text (do not encode as expected behaviour)

- Fall 1901 builds: the text mentions a French build "A Por"; a build in Portugal is illegal (not a home SC, R-ADJ-03). The actual French build is F Mar; Portugal is merely the newly gained SC that entitles it.
- Fall 1902 commentary calls the Russian unit in Rumania an "Army"; it is a **Fleet**.
- Fall 1902 Russian order `F Swe S StP-Nwy` omits the unit type — valid per R-ORD-06 (good parser test).
- Abbreviations in the sample (`Port`, `Galacia`) are spelling variants; the parser should accept or the test should normalise them.

> These sample-game expectations were derived by hand from the rulebook commentary and should be
> double-checked once the engine runs them.
