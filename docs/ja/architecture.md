# アーキテクチャ（ドラフト）

> 英語版 [../en/architecture.md](../en/architecture.md) が正です。本書はその翻訳です。
>
> 状態: **ドラフト／提案。** *（提案）* と記した技術選定は確定ではありません。

## 1. 目的

*ディプロマシー*のゲームマスターツールとして、以下を行う:

- ゲームの状態と履歴を保存する
- プレイヤー（または GM）がインタラクティブな地図上で命令を入力できる
- 公式ルール（[rules.md](rules.md)、[adjudication.md](adjudication.md)）に従って命令を裁定する
- **結果と解決の手順の両方**を返し、GUI が解決をステップごとに再生できるようにする

交渉自体はツールの外（対面など）で行います。ツールは合意の履行を強制しません（R-GEN-05）。

## 2. 構成要素

```
┌──────────────────────────┐          JSON/HTTP          ┌───────────────────────────────┐
│ GUI（Web）               │  ───── 命令 ────────────▶   │ API サーバー                  │
│  - 地図（SVG）           │                             │  - ゲーム／フェイズ管理       │
│  - 命令入力              │  ◀──── 状態・解決結果・     │  - 命令の検証                 │
│  - フェイズ／進行状況    │         トレース ───────    │  - 裁定器（純粋関数）         │
│  - ステップ再生          │                             │  - 永続化                     │
└──────────────────────────┘                             └───────────────────────────────┘
                     ▲                                                ▲
                     └────────── 共通: 型定義・マップデータ ──────────┘
```

| 構成要素 | 責務 | 技術 *（提案）* |
|----------|------|-----------------|
| `server/` | REST API、ゲームのライフサイクル、永続化、裁定器の呼び出し | TypeScript、Node.js（Hono または Fastify） |
| `server/engine/`（または `engine/`） | 純粋な裁定器: `adjudicate(state, orders) → Resolution`。I/O なし。 | TypeScript、Vitest。DATC ＋ [test-cases.md](test-cases.md) |
| `web/` | 地図描画、命令入力、進行状況、トレース再生 | React ＋ Vite、SVG 地図 |
| `shared/` | 型（`Order`、`GameState`、`Resolution`、`TraceStep`）、マップデータ（`standard.json`: 地域・海岸・隣接・SVG 座標） | TypeScript ＋ JSON |
| 永続化 | ゲーム、フェイズ、提出された命令、解決結果 | まずは SQLite *（提案）* |

裁定器が唯一の権威です。GUI は使い勝手のために事前検証してもよいが、サーバーがすべてを再検証します。

## 3. ゲームの流れ

```
ゲーム作成 ─▶ S1901M ─▶ [S1901R] ─▶ F1901M ─▶ [F1901R] ─▶ W1901A ─▶ S1902M ─▶ …
                 │            │          │           │           │
               命令          撤退       命令        撤退       建設／解散
                 └── 解決 ────┘                      └─ 補給都市更新＋勝利判定（PR-11）
```

- 合法な撤退先を持つユニットがなければ撤退フェイズ（`R`）はスキップ。
- 建設・解散が必要な列強がなければ調整フェイズ（`A`）はスキップ。
- フェイズは GM の操作で（または設定により全列強の提出完了時に）解決する。
  何も提出しなかった列強は R-ORD-03 / PR-09 に従って扱う。内乱は GM が宣言した場合のみ適用する（PR-16）。

### フェイズ ID 形式

`<S|F|W><年><M|R|A>` — 例: `S1901M`、`F1901R`、`W1901A`。

## 4. 中核データモデル（スケッチ）

```jsonc
// GameState
{
  "gameId": "g_123",
  "variant": "standard",          // または "2p-ww1", "6p-no-italy", …
  "phase": "F1901M",
  "units": [
    { "power": "RUS", "type": "F", "location": "StP", "coast": "SC" },
    { "power": "FRA", "type": "A", "location": "Bur" }
  ],
  "supplyCenters": { "Par": "FRA", "Bel": null },
  "dislodged": [ /* 撤退フェイズのみ */
    { "power": "FRA", "type": "A", "location": "Bur", "attackerFrom": "Ruh", "retreatOptions": ["Gas", "Pic"] }
  ],
  "standoffProvinces": ["Sil"],
  "civilDisorder": [ { "power": "ITA", "since": "F1902M" } ],   // GM が宣言（PR-16）
  "winner": null
}

// Order（GUI から送信される構造化形式）
{ "power": "FRA", "unit": "Mar", "type": "move", "to": "Bur" }
{ "power": "FRA", "unit": "Gas", "type": "support", "target": "Mar", "to": "Bur" }   // 維持支援なら "to" を省略
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

列強 ID: `AUS ENG FRA GER ITA RUS TUR`。地域 ID: ルールブックの略称（[map.md](map.md)）。

## 5. API（スケッチ）

| メソッドとパス | 目的 |
|----------------|------|
| `GET  /maps/standard` | 描画とクライアント側ヒント用のマップデータ |
| `POST /games` | ゲーム作成（バリアント、列強の割り当て、設定） |
| `GET  /games/:id` | 現在の状態とフェイズ |
| `GET  /games/:id/history` | 全フェイズの命令と解決結果 |
| `GET  /games/:id/units/:prov/options` | ユニットの合法な命令一覧（命令入力 UI の補助） |
| `PUT  /games/:id/phases/:phase/orders/:power` | 列強の命令を提出／置換。命令ごとの検証結果を返す |
| `POST /games/:id/phases/:phase/resolve` | 裁定。`Resolution`（`trace` を含む）を返し、ゲームを進める |
| `GET  /games/:id/phases/:phase/resolution` | 再生用に過去の解決結果を取得 |
| `PUT  /games/:id/civil-disorder/:power` | GM のみ: 現在のフェイズから列強の内乱を宣言（PR-16） |
| `DELETE /games/:id/civil-disorder/:power` | GM のみ: 内乱を解除（交代プレイヤー参加時など） |
| `POST /adjudicate` | ステートレス: `{state, orders}` → `Resolution`（サンドボックス、テスト、「もしも」検討） |

秘密の同時命令: フェイズが解決されるまで、列強の命令はその列強と GM にのみ見える。
（認証・ロール: 未定。）

## 6. GUI

| 領域 | 機能 |
|------|------|
| 地図 | SVG 地図。ユニット（A/F の形、列強の色）、補給都市の支配、分割海岸。クリックでユニット／地域を選択 |
| 命令入力 | ユニット選択 → 命令種別選択 → 対象をクリック。サーバーから得た合法な選択肢を表示。テキスト入力も可 |
| 進行状況 | 現在のフェイズ、年／季節、列強ごとの提出状況、補給都市数、タイマー（R-TURN-05）、内乱バッジ |
| GM 操作 | フェイズの解決、列強ごとの内乱の宣言／解除（PR-16） |
| 解決の再生 | `trace` をステップ送り（前へ／次へ／再生）: 命令の強調表示、戦力、支援カット、スタンドオフ、撃退をルール ID 付きで表示 |
| 履歴 | 過去のフェイズの地図と命令を閲覧 |

## 7. 開発方針

- **ルールが仕様。** エンジンのあらゆる挙動はルール ID かプロジェクト裁定（PR-xx）に遡れること。
- **エンジンはテストファースト。** [test-cases.md](test-cases.md) のルールブックの例と関連する DATC ケースを単体テストにする。
- **英語ドキュメントが正**。日本語ドキュメントはそれを反映し、同じ変更で更新する。
