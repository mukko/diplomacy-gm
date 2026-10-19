# ルールブックの例によるテストケース

> 英語版 [../en/test-cases.md](../en/test-cases.md) が正です。本書はその翻訳です。

ルールブックのすべての図を **命令＋期待結果** として書き直したもので、裁定器の単体テストに使います。
盤上には列挙したユニットのみが存在するものとします。ケース ID: `RB-Dnn`（図 nn）。
ルール ID は [rules.md](rules.md)、`PR-xx` は [adjudication.md](adjudication.md) §5 を参照。

凡例: ✅ 成功 · ❌ 失敗 · 💥 撃退 · ✂ 支援カット

## 1. 移動／マップ

| ID | 配置・命令 | 期待結果 | ルール |
|----|------------|----------|--------|
| RB-D01 | A Par | 合法な移動先: Bre, Pic, Bur, Gas | R-MOV-01 |
| RB-D02 | F Eng | 合法な移動先: Iri, Wal, Lon, Bel, Pic, Bre, Nth, Mid | R-MOV-02 |
| RB-D03 | F Rom | 合法: Tus, Nap, Tyn。不正: Ven, Apu | R-MAP-05 |
| RB-SPA | F Spa/NC → Wes, GoL, Mar | すべて不正 | R-MAP-06 |
| RB-BUL | F Con-Bul（海岸指定なし） | 不正 → 維持。`F Con-Bul EC` と `F Con-Bul SC` は合法 | R-MAP-07 |

## 2. スタンドオフ

| ID | 命令 | 期待結果 | ルール |
|----|------|----------|--------|
| RB-D04 | 独: A Ber-Sil · 露: A War-Sil | 両方 ❌、Sil は空（スタンドオフ） | R-STD-01 |
| RB-D04b | D04 ＋ Sil で維持するユニット | 両方 ❌、Sil のユニットは残る | R-STD-02 |
| RB-D05 | 独: F Kie-Ber, A Ber-Pru · 露: A Pru H | 何も動かない | R-STD-03 |
| RB-D06 | 独: F Ber-Pru, A Pru-Ber | どちらも動かない | R-STD-04 |
| RB-D07 | 英: A Hol-Bel, F Bel-Nth · 仏: F Nth-Hol | すべて ✅（回転） | R-STD-05 |

## 3. 支援

| ID | 命令 | 期待結果 | ルール |
|----|------|----------|--------|
| RB-D08 | 仏: A Mar-Bur, A Gas S A Mar-Bur · 独: A Bur H | Mar-Bur ✅、A Bur 💥 | R-SUP-01 |
| RB-D09 | 独: A Sil-Pru, F Bal S A Sil-Pru · 露: A Pru H | Sil-Pru ✅、A Pru 💥 | R-SUP-02 |
| RB-D10 | 仏: F GoL-Tyn, F Wes S F GoL-Tyn · 伊: F Nap-Tyn, F Rom S F Nap-Tyn | 両方 ❌（2 = 2）、Tyn は空 | R-STD-01 |
| RB-D11 | 仏: F GoL-Tyn, F Wes S F GoL-Tyn · 伊: F Tyn H, F Rom S F Tyn | GoL-Tyn ❌（2 対 2） | R-SUP-05 |
| RB-SUPMISMATCH | A Boh S A Mun-Sil · A Mun-Tyr | 支援は無効。維持支援には**ならない** | R-SUP-06 |
| RB-D12 | 墺: A Boh-Mun, A Tyr S A Boh-Mun · 独: A Mun-Sil, A Ber S A Mun-Sil · 露: A War-Sil, A Pru S A War-Sil | Boh-Mun ✅、A Mun 💥。Mun-Sil ❌ と War-Sil ❌（2 = 2、Sil は空） | R-STD-06 |
| RB-D13 | 土: A Bul-Rum · 露: A Rum-Bul, A Ser S A Rum-Bul, A Sev-Rum | Rum-Bul ✅、A Bul 💥。Sev-Rum ✅ | R-STD-07, PR-01 |
| RB-D14 | 土: A Bul-Rum, F Bla S A Bul-Rum · 露: A Rum-Bul, A Gre S A Rum-Bul, A Ser S A Rum-Bul, A Sev-Rum | Rum-Bul ✅（3 対 2）、A Bul 💥。Sev-Rum ✅ | R-STD-07 |
| RB-D13r | D13 から A Sev-Rum を除いたもの | Rum は空になるがスタンドオフ地域**ではない**（R-RET-04）。ただし攻撃者の出発地なので A Bul はそこへ撤退できない。 | R-RET-03/04 |

## 4. 支援カット

| ID | 命令 | 期待結果 | ルール |
|----|------|----------|--------|
| RB-D15 | 独: A Pru-War, A Sil S A Pru-War · 露: A War H, A Boh-Sil | Sil ✂。Pru-War ❌（1 対 1）。Boh-Sil ❌ | R-CUT-01 |
| RB-D16 | 独: A Pru-War, A Sil S A Pru-War · 露: A War-Sil | Sil の支援はカット**されない**。War-Sil ❌。Pru-War ✅（2 対 1）、A War 💥 | R-CUT-01 |
| RB-D17 | 独: F Ber-Pru, A Sil S F Ber-Pru · 露: A Pru-Sil, A War S A Pru-Sil, F Bal-Pru | Pru-Sil ✅、A Sil 💥 → Sil ✂。Ber-Pru ❌ と Bal-Pru ❌（1 = 1）、Pru は空 | R-CUT-02 |
| RB-D18 | 独: A Ber H, A Mun-Sil · 露: A Pru-Ber, A Sil S A Pru-Ber, A Boh-Mun, A Tyr S A Boh-Mun | Mun-Sil ❌ だが Sil ✂。Pru-Ber ❌。Boh-Mun ✅、A Mun 💥 | R-CUT-03 |

## 5. 輸送

| ID | 命令 | 期待結果 | ルール |
|----|------|----------|--------|
| RB-D19 | 英: A Lon-Nwy, F Nth C A Lon-Nwy | ✅ | R-CVY-03 |
| RB-CVYMISMATCH | A Rum-Arm · F Bla C A Rum-Ank | 輸送は無効。Rum と Arm は隣接していないため A Rum-Arm には輸送が必要だが、合致する輸送がないので失敗し、陸軍は Rum に留まる。(陸路で行ける移動先、例えば A Rum-Sev なら、移動は通常どおり判定される。) | R-CVY-04 |
| RB-SUPCVY | 英: A Pic-Bre, A Lon S A Pic-Bre, F Eng C A Lon S A Pic-Bre · 仏: F Bre H | Lon の支援と Eng の輸送は不正。Pic-Bre ❌（1 対 1） | R-SUP-07 |
| RB-D20 | 英: A Lon-Tun, F Eng C A Lon-Tun, F Mid C A Lon-Tun · 仏: F Wes C A Lon-Tun | ✅（陸軍は Tun に到着） | R-CVY-03 |
| RB-D21 | 仏: A Spa-Nap, F GoL C A Spa-Nap, F Tyn C A Spa-Nap · 伊: F Ion-Tyn, F Tun S F Ion-Tyn | F Tyn 💥。輸送失敗。A Spa は留まる | R-CVY-05 |

## 6. 自国ユニットの撃退と自国スタンドオフ

| ID | 命令 | 期待結果 | ルール |
|----|------|----------|--------|
| RB-D22 | 仏: A Par-Bur, A Mar S A Par-Bur, A Bur H | 何も動かない | R-SLF-01 |
| RB-D23 | 仏: A Par-Bur, A Bur-Mar · 独: A Ruh S A Par-Bur · 伊: A Mar-Bur | 何も動かない。仏 A Bur は撃退されない | R-SLF-01 |
| RB-D24 | 独: A Ruh-Bur, A Mun H · 仏: A Par S A Ruh-Bur, A Bur H | Ruh-Bur ❌（仏の支援は算入されない） | R-SLF-02 |
| RB-D24b | D24 で独を A Mun S A Ruh-Bur に変更 | Ruh-Bur ✅、仏 A Bur 💥 | R-SLF-02 |
| RB-D25 | 独: A Mun-Tyr, A Ruh-Mun, A Sil-Mun · 墺: A Tyr-Mun, A Boh S A Sil-Mun | 何も動かない | R-SLF-01 |
| RB-D26 | 英: F Den-Kie, F Nth-Den, F Hel S F Nth-Den · 露: A Ber-Kie, F Ska-Den, F Bal S F Ska-Den | 何も動かない（Nth-Den が Ska-Den とスタンドオフ） | R-SLF-03, PR-02 |
| RB-D27 | 墺: A Ser-Bud, A Vie-Bud · 露: A Gal S A Ser-Bud | Ser-Bud ✅ | R-SLF-04 |
| RB-D27b | D27 で Bud が空でなく墺 A Bud H がいる | Ser-Bud ❌（自国ユニットを撃退することになる） | R-SLF-04 |
| RB-CUTOWN | 自国ユニットが自国の支援ユニットを攻撃 | 支援はカットされない | R-CUT-04 |

## 7. 高度な輸送ケース

| ID | 命令 | 期待結果 | ルール |
|----|------|----------|--------|
| RB-D28 | 英: A Lon-Bel, F Nth C A Lon-Bel · 仏: A Bel-Lon, F Eng C A Bel-Lon | 両方 ✅（輸送による入れ替え） | R-CVY-07 |
| RB-D29 | 英: A Lon-Bel, F Eng C A Lon-Bel, F Nth C A Lon-Bel · 仏: F Bre-Eng, F Iri S F Bre-Eng | F Eng 💥。A Lon-Bel は Nth 経由で ✅ | R-CVY-08 |
| RB-D30 | 仏: A Tun-Nap, F Tyn C A Tun-Nap · 伊: F Ion-Tyn, F Nap S F Ion-Tyn | F Tyn 💥。輸送失敗。Nap の支援はカット**されない** | R-CVY-09 |
| RB-D31 | 仏: A Tun-Nap, F Tyn C A Tun-Nap, F Ion C A Tun-Nap · 伊: F Rom-Tyn, F Nap S F Rom-Tyn | Nap ✂（Ion 経由の経路が健在）。Rom-Tyn ❌（1 対 1）。Tun-Nap ❌（1 対 1） | R-CVY-10 |
| RB-D32 | D31 ＋ 仏: A Apu S A Tun-Nap | F Nap 💥（2 対 1）。陸軍は Nap に上陸。Rom-Tyn ❌ | R-CVY-10 |
| RB-D32b | D32 で伊の命令を逆に: F Nap-Tyn, F Rom S F Nap-Tyn | F Tyn 💥、F Nap が Tyn へ移動（ルールブックの注記） | R-CVY-05 |
| RB-LANDCVY1 | 陸路でも輸送でも到達可能。輸送艦隊のうち1つ以上が陸軍の所有者のもの | 輸送を使う | R-CVY-11 |
| RB-LANDCVY2 | 同上だが輸送艦隊はすべて他国のもの、"via convoy" 指定なし | 陸路を使う | R-CVY-11 |

## 8. 撤退

| ID | 状況 | 期待結果 | ルール |
|----|------|----------|--------|
| RB-RET1 | D12 の結果: A Mun が Boh からの攻撃で撃退 | Boh（攻撃者の出発地）と Sil（スタンドオフ）へは撤退できない | R-RET-03 |
| RB-RET2 | 2ユニットが同じ地域へ撤退 | 両方解散 | R-RET-06 |
| RB-RET3 | 合法な撤退先がない | 撤退命令なしで即座に解散 | R-RET-05 |

## 9. サンプルゲーム（統合テスト）

標準の初期配置から始まるルールブックのサンプルゲーム（1901年春〜1902年秋）。複数フェイズの回帰テストとして使います。
命令はルールブックの記法、期待結果は要約です。

### 1901年春

```
Austria: A Vie-Tri, A Bud-Gal, F Tri-Alb
England: A Lvp-Yor, F Lon-Nth, F Edi-Nrg
France:  A Par-Bur, A Mar-Spa, F Bre-Pic
Germany: A Ber-Kie, A Mun-Ruh, F Kie-Den
Italy:   A Ven-Pie, A Rom-Ven, F Nap-Ion
Russia:  A Mos-Ukr, A War-Gal, F StP-Bot, F Sev-Bla
Turkey:  A Con-Bul, A Smy-Con, F Ank-Bla
```
期待結果: F Sev-Bla / F Ank-Bla（スタンドオフ）と A Bud-Gal / A War-Gal（スタンドオフ）以外はすべて成功。撤退なし。
（注: `F StP-Bot` は `StP/SC` から出発。）

### 1901年秋

```
Austria: A Tri H, A Bud-Ser, F Alb-Gre
England: A Yor-Nwy, F Nth C A Yor-Nwy, F Nrg-Bar
France:  A Bur-Mar, A Spa-Por, F Pic-Bel
Germany: A Kie-Hol, A Ruh-Bel, F Den H
Italy:   A Ven H, A Pie-Mar, F Ion-Tun
Russia:  A Ukr S F Sev-Rum, A War-Gal, F Bot-Swe, F Sev-Rum
Turkey:  A Bul-Ser, A Con-Bul, F Ank-Bla
```
期待結果: Bel、Mar、Ser への移動はスタンドオフ。A Con-Bul は失敗（Bul が空かない）。それ以外は成功。撤退なし。
調整: 英 +1、土 +1、墺 +1、伊 +1、仏 +1、露 +2、独 +2。
建設: 英 F Edi · 独 F Kie, A Mun · 露 A StP, A Sev · 土 A Smy · 墺 A Vie · 伊 F Nap · 仏 F Mar。

### 1902年春

```
Austria: A Tri-Bud, A Vie-Bud, A Bud-Ser, F Gre H
England: A Nwy-StP, F Nth-Nwy, F Bar S A Nwy-StP, F Edi-Nth
France:  A Bur S F Pic-Bel, A Por-Spa, F Pic-Bel, F Mar H
Germany: A Hol-Bel, A Ruh S A Hol-Bel, A Mun-Bur, F Den H, F Kie-Hol
Italy:   A Ven H, A Pie-Mar, F Tun-Wes, F Nap-Tyn
Russia:  A Ukr S F Rum, A Gal-Bud, A StP-Nwy, A Sev S F Rum, F Swe S A StP-Nwy, F Rum H
Turkey:  A Bul-Rum, A Con-Bul, A Smy-Arm, F Bla S A Bul-Rum
```
期待結果: Nwy-StP / StP-Nwy は失敗（2 対 2 の正面衝突）、そのため F Nth-Nwy と F Edi-Nth も失敗。
Tri/Vie/Gal-Bud はすべてスタンドオフ。A Bud-Ser ✅。Bul-Rum ❌（2 対 3）。Con-Bul ❌。Smy-Arm ✅。
Mun-Bur ❌ だが A Bur の支援をカット → Hol-Bel ✅（2 対 1）、Pic-Bel ❌。F Kie-Hol ✅。
Pie-Mar ❌。Tun-Wes ✅。Nap-Tyn ✅。Por-Spa ✅。撤退なし。

### 1902年秋

```
Austria: A Vie-Gal, A Tri-Bud, A Ser S Turkish A Bul-Rum, F Gre H
England: A Nwy-StP, F Bar S A Nwy-StP, F Nth-Nwy, F Edi-Nth
France:  A Bur-Bel, F Pic S A Bur-Bel, A Spa S F Mar, F Mar S A Spa
Germany: A Ruh-Bur, A Mun S A Ruh-Bur, A Bel S A Ruh-Bur, F Den-Swe, F Hol S A Bel
Italy:   A Ven-Pie, A Pie-Mar, F Wes-NAf, F Tyn-GoL
Russia:  A StP-Nwy, F Swe S StP-Nwy, F Rum S A Sev, A Sev S F Rum, A Gal S F Rum, A Ukr S A Sev
Turkey:  A Bul-Rum, A Con-Bul, A Arm-Sev, F Bla S A Bul-Rum
```
期待結果:
- カットされる支援: F Swe（Den から）、F Mar（Pie から）、A Sev（Arm から）、A Gal（Vie から）、F Rum（Bul から）。A Bel はカット**されない**（攻撃が支援先の Bur から来ているため）。
- Mar と Sev は攻撃を防ぐ。Vie-Gal ❌。Ven-Pie ❌。Pie-Mar ❌。
- Bur-Bel ❌（1 対 2）。Ruh-Bur ✅（2 対 1）、仏 A Bur 💥。
- Bul-Rum ✅、露 **F** Rum 💥（撤退先なし → 解散）。Con-Bul ✅。
- Nwy-StP ✅（Swe の支援がカット）、露 A StP 💥。F Nth-Nwy ✅。F Edi-Nth ✅。
- Den-Swe ❌。Wes-NAf ✅、Tyn-GoL ✅。
- 撤退: 露 A StP-Mos、仏 A Bur-Gas。
- 調整: ロシアは補給都市4・ユニット5 → A Gal を解散。建設: 独 F Kie、土 F Smy、墺 A Tri、仏 A Par、英 F Lon。イタリアは変化なし。

### ルールブック本文の既知の誤り（期待動作として実装しないこと）

- 1901年秋の建設: 本文にフランスの建設「A Por」とあるが、ポルトガルへの建設は不正（本国補給都市ではない。R-ADJ-03）。実際のフランスの建設は F Mar で、ポルトガルは建設権の根拠となった新規獲得補給都市にすぎない。
- 1902年秋の解説はルーマニアのロシアユニットを「陸軍」と呼んでいるが、実際は**艦隊**。
- 1902年秋のロシアの命令 `F Swe S StP-Nwy` はユニット種別を省略している — R-ORD-06 により有効（パーサーのよいテスト）。
- サンプル中の略称（`Port`、`Galacia`）は綴りの揺れ。パーサーで受け付けるか、テスト側で正規化すること。

> サンプルゲームの期待結果はルールブックの解説から手作業で導いたものです。エンジンで実行できるようになったら再確認してください。
