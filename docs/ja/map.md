# 標準マップ リファレンス

> 英語版 [../en/map.md](../en/map.md) が正です。本書はその翻訳です。

ルールブックで使われる標準の1901年ヨーロッパマップの参照データです（[rules.md](rules.md) §3 参照）。
略称はルールブックの一覧に従い、API で使う**正規の地域 ID** とします（入力は大文字小文字を区別せず、
出力は正規の表記）。地域名は曖昧さを避けるため英語表記のままとします。

> **隣接データはここには載せていません。** ルールブックには隣接表がないため、機械可読ファイル
> （予定: `shared/map/standard.json`）として作成し、ルールブックの例と §4 の特殊ケースから作る単体テストで検証します。

## 1. 集計

| 項目 | 数 |
|------|---:|
| 地域 | 75（＋通行不能のスイス） |
| 陸の地域 | 56（内陸14＋沿岸42） |
| 海域 | 19 |
| 補給都市 | 34（本国22＋中立12） |
| 勝利条件 | 18 |

## 2. 地域一覧

凡例 — **種別:** `L` 内陸、`C` 沿岸、`W` 海域。**SC:** ★ = 補給都市。**本国:** その地域を本国補給都市とする列強。

### 2.1 オーストリア

| 略称 | 名称 | 種別 | SC | 本国 |
|------|------|:----:|:--:|------|
| Boh | Bohemia | L | | |
| Bud | Budapest | L | ★ | オーストリア |
| Gal | Galicia | L | | |
| Tri | Trieste | C | ★ | オーストリア |
| Tyr | Tyrolia | L | | |
| Vie | Vienna | L | ★ | オーストリア |

### 2.2 イギリス

| 略称 | 名称 | 種別 | SC | 本国 |
|------|------|:----:|:--:|------|
| Cly | Clyde | C | | |
| Edi | Edinburgh | C | ★ | イギリス |
| Lvp | Liverpool | C | ★ | イギリス |
| Lon | London | C | ★ | イギリス |
| Wal | Wales | C | | |
| Yor | Yorkshire | C | | |

### 2.3 フランス

| 略称 | 名称 | 種別 | SC | 本国 |
|------|------|:----:|:--:|------|
| Bre | Brest | C | ★ | フランス |
| Bur | Burgundy | L | | |
| Gas | Gascony | C | | |
| Mar | Marseilles | C | ★ | フランス |
| Par | Paris | L | ★ | フランス |
| Pic | Picardy | C | | |

### 2.4 ドイツ

| 略称 | 名称 | 種別 | SC | 本国 |
|------|------|:----:|:--:|------|
| Ber | Berlin | C | ★ | ドイツ |
| Kie | Kiel | C | ★ | ドイツ |
| Mun | Munich | L | ★ | ドイツ |
| Pru | Prussia | C | | |
| Ruh | Ruhr | L | | |
| Sil | Silesia | L | | |

### 2.5 イタリア

| 略称 | 名称 | 種別 | SC | 本国 |
|------|------|:----:|:--:|------|
| Apu | Apulia | C | | |
| Nap | Naples | C | ★ | イタリア |
| Pie | Piedmont | C | | |
| Rom | Rome | C | ★ | イタリア |
| Tus | Tuscany | C | | |
| Ven | Venice | C | ★ | イタリア |

### 2.6 ロシア

| 略称 | 名称 | 種別 | SC | 本国 |
|------|------|:----:|:--:|------|
| Fin | Finland | C | | |
| Lvn | Livonia | C | | |
| Mos | Moscow | L | ★ | ロシア |
| Sev | Sevastopol | C | ★ | ロシア |
| StP | St. Petersburg | C（北岸/南岸） | ★ | ロシア |
| Ukr | Ukraine | L | | |
| War | Warsaw | L | ★ | ロシア |

### 2.7 トルコ

| 略称 | 名称 | 種別 | SC | 本国 |
|------|------|:----:|:--:|------|
| Ank | Ankara | C | ★ | トルコ |
| Arm | Armenia | C | | |
| Con | Constantinople | C | ★ | トルコ |
| Smy | Smyrna | C | ★ | トルコ |
| Syr | Syria | C | | |

### 2.8 中立地域（陸）

| 略称 | 名称 | 種別 | SC |
|------|------|:----:|:--:|
| Alb | Albania | C | |
| Bel | Belgium | C | ★ |
| Bul | Bulgaria | C（東岸/南岸） | ★ |
| Den | Denmark | C | ★ |
| Gre | Greece | C | ★ |
| Hol | Holland | C | ★ |
| Nwy | Norway | C | ★ |
| NAf | North Africa | C | |
| Por | Portugal | C | ★ |
| Rum | Rumania | C | ★ |
| Ser | Serbia | L | ★ |
| Spa | Spain | C（北岸/南岸） | ★ |
| Swe | Sweden | C | ★ |
| Tun | Tunis | C | ★ |

### 2.9 海域

| 略称 | 名称 |
|------|------|
| Adr | Adriatic Sea（アドリア海） |
| Aeg | Aegean Sea（エーゲ海） |
| Bal | Baltic Sea（バルト海） |
| Bar | Barents Sea（バレンツ海） |
| Bla | Black Sea（黒海） |
| Eas | Eastern Mediterranean（東地中海） |
| Eng | English Channel（イギリス海峡） |
| Bot | Gulf of Bothnia（ボスニア湾） |
| GoL | Gulf of Lyon（リヨン湾） |
| Hel | Helgoland Bight（ヘルゴラント湾） |
| Ion | Ionian Sea（イオニア海） |
| Iri | Irish Sea（アイリッシュ海） |
| Mid | Mid-Atlantic Ocean（中部大西洋） |
| NAt | North Atlantic Ocean（北大西洋） |
| Nth | North Sea（北海） |
| Nrg | Norwegian Sea（ノルウェー海） |
| Ska | Skagerrak（スカゲラク海峡） |
| Tyn | Tyrrhenian Sea（ティレニア海） |
| Wes | Western Mediterranean（西地中海） |

注: ツールによってはリヨン湾を `Lyo`、大西洋を `Mao`/`Nao`、北海/ノルウェー海を `Nth`/`Nwg` と表記します。
**本プロジェクトは上記のルールブック表記を使います。** 命令パーサーは一般的な別名を受け付けます
（[adjudication.md](adjudication.md) §2 参照）。

## 3. 分割海岸

| 地域 | 海岸 | 海岸 ID |
|------|------|---------|
| Bulgaria | 東岸（黒海側）、南岸（エーゲ海側） | `Bul/EC`, `Bul/SC` |
| Spain | 北岸（大西洋側）、南岸（地中海側） | `Spa/NC`, `Spa/SC` |
| St. Petersburg | 北岸（バレンツ海側）、南岸（ボスニア湾側） | `StP/NC`, `StP/SC` |

これらの地域にいる艦隊の位置は常に海岸付きで保持します。陸軍は海岸を持ちません。

## 4. 特殊なマップケース（隣接テストで必ず検証する）

| ケース | ルール |
|--------|--------|
| スイス | 通行不能。地域ではない。 |
| キール | 海岸は1つ。艦隊は Hol ↔ Kie ↔ Ber/Bal/Hel/Den を直接移動できる。 |
| コンスタンティノープル | 海岸は1つ。艦隊は Bla ↔ Con ↔ Aeg/Bul/Smy/Ank を移動できる。 |
| デンマーク–スウェーデン | 直接隣接（陸軍・艦隊とも）。 |
| バルト海–スカゲラク海峡 | 隣接**しない**。 |
| デンマーク–ベルリン | 隣接**しない**。 |
| スウェーデン–ノルウェー | 海岸線を共有（艦隊の移動・支援が可能）。 |
| ローマ | 艦隊: Tus, Nap, Tyn のみ（Ven, Apu は不可）。 |
| イギリス海峡 | Iri, Wal, Lon, Bel, Pic, Bre, Nth, Mid に隣接（ルールブック図2）。 |
| パリ（陸軍） | Bre, Pic, Bur, Gas に隣接（ルールブック図1）。 |
| スペイン北岸 | Wes, GoL, Mar とは隣接しない。 |

## 5. 初期ユニット

[rules.md](rules.md) §15 参照。ロシアの艦隊は `StP/SC` から開始します。

## 6. ユニットの色（ルールブック）

| 列強 | 色 |
|------|----|
| オーストリア | 赤 |
| イギリス | 紺 |
| フランス | 水色 |
| ドイツ | 黒 |
| イタリア | 緑 |
| ロシア | 白 |
| トルコ | 黄 |

GUI では視認性のために異なる色を使ってもよいが、この対応が分かるようにしてください。

## 7. 地図の描画（GUI 用アセット）

GUI は同じデータから作った2種類の盤面表示を持ちます:

| 表示 | ファイル | 用途 |
|------|----------|------|
| **地図**（メイン） | `shared/map/standard.svg` | プレイヤーが見る盤面。 |
| **グラフ**（補助） | `shared/map/standard.graph.svg` | 地域をノード、隣接を線（陸軍／艦隊／両方）で表示。分割海岸は別ノード。隣接データの確認と裁定結果のデバッグに使う。 |

- `standard.svg` は Martin Asal 作「Diplomacy.svg」（Wikimedia Commons、**CC BY-SA 3.0**）を改変したもの。
  改変した地図ファイルは同じライセンスとし、リポジトリのほかの部分には影響しない。
  [`shared/map/CREDITS.md`](../../shared/map/CREDITS.md) 参照。
- `standard.svg` の構造: 地域ごとに `<g id="prov-<ID>" class="province land|coast|sea" data-province="<ID>">`
  （ほかに通行不能の `prov-Swi`）。分割海岸は別の領域 `id="coast-<ID>-<NC|SC|EC>"`、補給都市の印は
  `id="sc-<ID>"`、ラベルは `id="label-<ID>"`。地域と補給都市の印の色分けとユニットの描画は GUI が行う。
- `standard.layout.json`: 地域ごとのユニット・ラベル・補給都市の印の位置と、分割海岸ごとの艦隊の位置。
  1地域に置けるユニットは1つ（R-UNIT-03）なので、陸軍の位置と海岸の位置が重なることはあり得る。
- 3ファイルとも、固定した元ファイル（`shared/map/source/Diplomacy.commons.svg`）と `standard.json` から
  `tools/map/build_map.py` で**生成**する。手で編集しないこと。`standard.json` を変えたら
  `python3 -I tools/map/build_map.py` を再実行する。テスト: `shared/map/layout.test.ts`。
- 分割海岸のどちらの部分がどの海岸かは目視で確認済み: ブルガリアの北側（黒海側）が東岸、スペインと
  サンクトペテルブルクの北側が北岸。
