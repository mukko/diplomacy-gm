# diplomacy-gm

[English README](README.md)

ボードゲーム ***ディプロマシー*** のゲームマスターツールです。公式ルールに従って命令を裁定し、結果を
ステップごとの解決手順とともに返す **API サーバー** と、地図・ゲームの進行状況を表示し、命令の入力と
解決ステップの再生ができる **GUI** で構成されます。

> 状態: **開発初期。** マップデータと Rust の API サーバー（ゲームのライフサイクル、JSON セーブファイル、
> 再生）があります。裁定器と GUI はまだありません。

## 開発

```bash
cd server && cargo run -- --data-dir data/games   # API: http://127.0.0.1:8080
cd server && cargo test                           # サーバーのテスト
npm install && npm test                           # マップデータのテスト（shared/map）
```

ゲームは 1 つにつき 1 つの JSON ファイルとしてデータディレクトリに保存されます。
[architecture.md § 5](docs/ja/architecture.md#5-永続化セーブファイル) を参照。

## ドキュメント

| ドキュメント | English | 日本語 |
|--------------|---------|--------|
| ルール仕様（ルール ID 付き） | [docs/en/rules.md](docs/en/rules.md) | [docs/ja/rules.md](docs/ja/rules.md) |
| マップ リファレンス | [docs/en/map.md](docs/en/map.md) | [docs/ja/map.md](docs/ja/map.md) |
| 裁定仕様とプロジェクト裁定 | [docs/en/adjudication.md](docs/en/adjudication.md) | [docs/ja/adjudication.md](docs/ja/adjudication.md) |
| ルールブックの例によるテストケース | [docs/en/test-cases.md](docs/en/test-cases.md) | [docs/ja/test-cases.md](docs/ja/test-cases.md) |
| アーキテクチャ（ドラフト） | [docs/en/architecture.md](docs/en/architecture.md) | [docs/ja/architecture.md](docs/ja/architecture.md) |
| 用語集（英日） | [docs/glossary.md](docs/glossary.md) | ← |

英語ドキュメントが正で、日本語ドキュメントはその翻訳です。

## ルールの出典

*Diplomacy* ルールブック 第5版（2008年）、Avalon Hill / Wizards of the Coast —
<https://media.wizards.com/2015/downloads/ah/diplomacy_rules.pdf>
本リポジトリのドキュメントは実装のためにルールを言い換えたものであり、ルールブックの PDF 自体は含みません。
*Diplomacy* は Hasbro の商標です。
