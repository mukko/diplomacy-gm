# diplomacy-gm

[English README](README.md)

ボードゲーム ***ディプロマシー*** のゲームマスターツールです。公式ルールに従って命令を裁定し、結果を
ステップごとの解決手順とともに返す **API サーバー** と、地図・ゲームの進行状況を表示し、命令の入力と
解決ステップの再生ができる **GUI** で構成されます。

> 状態: **開発初期。** マップデータ、Rust の API サーバー（ゲームのライフサイクル、JSON セーブファイル、
> 再生）、裁定器（移動・撤退・調整フェイズ、ステップごとの解決手順）、Web GUI の最初の版（日本語／英語）があります。

## 起動方法

[Node.js](https://nodejs.org/)（npm）と [Rust](https://www.rust-lang.org/)（cargo）が必要です。

```bash
./start.sh
```

初回は GUI の依存パッケージをインストールし、変更があれば GUI とサーバーをビルドしてから、
<http://127.0.0.1:8080/> でサーバーを起動してブラウザで開きます。ゲームは `data/games/` に保存されます。
Ctrl+C で停止します。オプション: `--port 9000`（ポート変更）、`--data-dir DIR`（保存先）、`--no-open`（ブラウザを開かない）、
`--rebuild`（作り直す）。`./start.sh --help` で一覧を表示します。

## 配布用パッケージ（Windows / macOS）

`scripts/package.sh [ターゲット]` で、GUI を埋め込んだ実行ファイル1つを作り、簡単な説明書と地図のクレジットと一緒に
`dist/` に zip でまとめます。使う人は Node.js も Rust も不要です。Windows では zip を展開して `diplomacy-gm.exe` を
ダブルクリックすると、サーバーが起動してブラウザが開き、ゲームはプログラムと同じフォルダの `data\games` に保存されます
（[packaging/README-windows.txt](packaging/README-windows.txt) 参照）。

- macOS で Windows 版をビルドする手順（クロスコンパイル。最初の2行は初回のみ）:

  ```bash
  brew install mingw-w64                      # Windows 用のリンカー（約 1.4GB）
  rustup target add x86_64-pc-windows-gnu
  npm run package:windows                     # = scripts/package.sh x86_64-pc-windows-gnu
                                              # → dist/diplomacy-gm-x86_64-pc-windows-gnu.zip
  ```

  Linux ではパッケージマネージャーで mingw-w64 を入れます。Windows 上では Node.js と Rust を入れたうえで、
  Git Bash から `scripts/package.sh x86_64-pc-windows-msvc` を実行します。
- 今使っているマシン向けのパッケージ: `scripts/package.sh`（例: `dist/diplomacy-gm-aarch64-apple-darwin.zip`）。
- GitHub Actions: [.github/workflows/package.yml](.github/workflows/package.yml) がテストを実行し、Windows 版と
  macOS 版のパッケージを作ります（手動実行、または `v*` タグの push 時）。

## 開発

```bash
cd web && npm install && npm run build            # GUI をビルド（web/dist、初回と GUI 変更時）
cd server && cargo run -- --data-dir data/games   # API と GUI: http://127.0.0.1:8080
cd server && cargo test                           # サーバーのテスト
npm install && npm test                           # マップデータのテスト（shared/map）
cd web && npm run dev                             # GUI の開発サーバー: http://127.0.0.1:5173（API の起動が必要）
```

ブラウザで <http://127.0.0.1:8080/> を開きます。サーバーは自分のマシンからの接続だけを受け付けます。

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

## 地図のクレジット

盤面の地図は Martin Asal 作「Diplomacy.svg」（Wikimedia Commons、CC BY-SA 3.0）を改変したもので、
改変した地図ファイルも同じライセンスです。[shared/map/CREDITS.md](shared/map/CREDITS.md) を参照してください。
