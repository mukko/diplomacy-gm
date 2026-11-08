# diplomacy-gm

[English README](README.md)

ボードゲーム ***ディプロマシー*** のゲームマスター（GM）用ツールです。GM が自分のパソコンで動かします。
GM がプレイヤーの命令を入力すると、公式ルールに従って裁定し、結果を地図上に表示します。結果は1軍ずつ、
判定の根拠とルールの参照つきで確認できます。

> 非公式のファン製プロジェクトです。Hasbro、Avalon Hill、Wizards of the Coast とは関係がなく、承認も受けていません。
> *Diplomacy* は Hasbro の商標です。

## 主な機能

- **ルールエンジン**：標準の7人戦に対応します（移動・撤退・調整フェイズ）。公式ルールブック（第5版、2008年）に従い、
  ルールブックの全例と、DATC（裁定プログラム用のテストケース集）の移動・撤退・建設のケース（6.A〜6.I）で検証しています（[docs/ja/test-cases.md](docs/ja/test-cases.md)）。
- **地図 GUI**：ブラウザで動作し、日本語と英語を切り替えられます。ユニット、補給都市の支配、命令の矢印を表示し、
  隣接関係の**グラフ表示**にも切り替えられます。
- **命令の入力**：地図上のユニットをクリックして選ぶ方法（合法な命令だけが候補に出ます）と、ルールブックの記法で
  **文字入力**する方法（`France: A Par-Bur, A Mar S A Par-Bur`）があります。命令ごとにチェックし、無効な命令には理由を表示します。
- **解決の再生**：移動フェイズの結果を地図上で1軍ずつ確認したり、解決前と解決後の盤面を切り替えたりできます。
  各ステップに、成功・失敗の理由（戦力の比較、支援カット、ルール ID）が表示されます。
- **そのほか**：全フェイズの**履歴**、**セーブファイル**（1ゲーム1つの JSON。自動保存、書き出し・読み込み可）、
  **内乱**の扱い、緊急用の盤面の**手動変更**。
- **ローカルで完結**：サーバーは同じパソコンからの接続だけを受け付けます。アカウントもインターネット接続も不要です。

## 使い始める

### 方法A：配布用パッケージ（開発ツール不要）

パッケージを一度ビルドするか（[パッケージのビルド](#パッケージのビルド)参照）、GitHub Actions の成果物から取得して使います。

- **Windows**
  1. `diplomacy-gm-x86_64-pc-windows-*.zip` を展開し、`diplomacy-gm.exe` をダブルクリックします。
  2. 黒いウィンドウ（サーバー）が開き、続いてブラウザで GUI が開きます。
  3. 終了するときはウィンドウを閉じます。ゲームはプログラムと同じフォルダの `data\games` に保存されます。
  4. 署名していないプログラムのため、初回に警告が出ることがあります。「詳細情報」→「実行」を選んでください。
- **macOS / Linux**
  1. 展開して、ターミナルで `./diplomacy-gm --open` を実行します。
  2. ゲームは、実行したフォルダの `data/games` に保存されます。
  3. macOS では、ダウンロードした署名なしのプログラムが止められることがあります。その場合は「システム設定」→
     「プライバシーとセキュリティ」で許可するか、`xattr -d com.apple.quarantine diplomacy-gm` を実行してください。

### 方法B：ソースから起動

[Node.js](https://nodejs.org/)（npm。バージョン 22 以降を推奨）と [Rust](https://www.rust-lang.org/)（cargo、stable）が必要です。

```bash
git clone <このリポジトリ>
cd diplomacy-gm
./start.sh
```

`start.sh` は、初回に GUI の依存パッケージをインストールし、変更があれば GUI とサーバーをビルドしてから、
<http://127.0.0.1:8080/> でサーバーを起動してブラウザで開きます。ゲームは `data/games/` に保存されます。
Ctrl+C で停止します。

| オプション | 意味 |
|------------|------|
| `--port 9000` | ポートを変える |
| `--data-dir DIR` | セーブファイルの保存先を `DIR` にする |
| `--no-open` | ブラウザを開かない |
| `--rebuild` | GUI とサーバーを作り直す |

## 使い方

### 1. スタート画面

- **新しいゲーム**：タイトルを入れて作成します（標準マップ、1901年春から）。
- **保存済みのゲーム**：選ぶと、続きから再開できます。
- **セーブファイルを読み込む**：書き出したセーブファイルからゲームを復元します。新しい ID で読み込むこともできます。
- 右上のボタンで日本語と英語を切り替えます。

### 2. 命令の入力（「命令」タブ）

- **地図でユニットをクリック**（または一覧から選択）すると、出せる命令（維持・移動・支援・輸送）がボタンで並びます。
  移動できる地域が黄色く光るので、その地域をクリックすれば移動命令になります。
- または **「テキストで入力」** を開き、複数の国の命令を1行に1つ（「,」「;」区切りでも可）で入力します。

  ```
  France: A Par-Bur, A Mar S A Par-Bur, F Bre-Mid
  Germany:
  A Mun-Bur
  F Kie-Den
  F Nth C A Lon-Nwy
  A Lon-Nwy via convoy
  ```

  - 行頭の `France:` はその行だけに効き、国名だけの行は以降の行に効きます。
  - 地域は略称、よく使われる別名、正式名のどれでも書けます。
  - **「読み取る」** で読み取り結果を確認し、**「下書きに反映」** または **「反映して提出」** を押します。
- 国ごとに **「提出」** を押します。各命令に「有効」「無効 → 維持」「無視」の判定が、理由とルール ID つきで出ます。
  命令のないユニットは維持します。

### 3. フェイズの解決

**「このフェイズを解決」** を押して確認すると、「解決結果」タブが開きます。

- **1軍ずつ**（移動フェイズの既定）
  - 列強ごとにユニットを1つずつ進めます。ボタンのほか、← / → キーでも進めます。
  - 各ステップに、そのユニットの命令と結果（成功／失敗／維持／撃退）、理由が出ます。
  - 盤面は累積で、表示済みのユニットは解決後の位置に移ります。
  - 「判定の詳細」を開くと、そのユニットについての裁定の根拠が見られます。
- **前期** / **結果**：解決前の盤面と、全ユニットを一度に動かした解決後の盤面です。どちらも全命令の矢印を、
  成功は緑、失敗は赤で表示します。

撤退フェイズ（撃退されたユニットの撤退・解散）と、調整フェイズ（冬の建設・解散）も、同じく「命令」タブで入力します。
必要な解散が入力されていない場合は自動で選ばれ、そのユニットは事前に表示されます。

### 4. そのほかの機能

| 場所 | できること |
|------|------------|
| **「進行状況」タブ** | 列強ごとのユニット数・補給都市数・提出状況の確認、**内乱**の宣言・解除 |
| **「履歴」タブ** | 過去の全フェイズについて、開始時の盤面と解決の手順をステップごとに確認 |
| **「地図」/「グラフ」** | 地図表示と、隣接関係のグラフ表示の切り替え（移動や支援の確認に便利） |
| **「手動で変更」**（緊急用） | 盤面の下書き上でユニットや補給都市の支配国を直接変更し、**「決定」** または **「キャンセル」**。決定すると、そのフェイズで提出済みの命令は消え、変更は履歴に記録されます |
| **「セーブファイルを書き出す」** | ゲーム全体を1つの JSON ファイルで保存（データフォルダにも自動保存されています） |

## パッケージのビルド

`scripts/package.sh [ターゲット]` で、GUI を埋め込んだ実行ファイル1つを作ります。説明書、ライセンス、地図のクレジット、
サードパーティのライセンスと一緒に `dist/` に zip でまとめます。

```bash
scripts/package.sh                      # このパソコン向け（例: dist/diplomacy-gm-aarch64-apple-darwin.zip）
npm run package:windows                 # macOS / Linux から Windows x64 向け（= scripts/package.sh x86_64-pc-windows-gnu）
```

- Windows 向けのクロスコンパイルには、初回だけ次の準備が必要です。
  - `brew install mingw-w64`（macOS。約 1.4GB。Linux ではパッケージマネージャーで入れます）
  - `rustup target add x86_64-pc-windows-gnu`
- Windows 上でビルドする場合は、Git Bash から `scripts/package.sh x86_64-pc-windows-msvc` を実行します。
- [.github/workflows/package.yml](.github/workflows/package.yml) は、GitHub Actions 上でテストを実行し、Windows 版と
  macOS 版のパッケージを作ります（手動実行、または `v*` タグの push 時）。

## 開発

```bash
npm install && npm test                 # マップデータのテスト（shared/map）
cd server && cargo test                 # ルールエンジンと API のテスト
cd web && npm install && npx vitest run # GUI のテスト
cd web && npm run dev                   # GUI の開発サーバー http://127.0.0.1:5173（:8080 の API が必要）
cd server && cargo run -- --data-dir data/games   # API とビルド済み GUI http://127.0.0.1:8080
```

| パス | 内容 |
|------|------|
| `server/` | Rust（axum）の API サーバーとルールエンジン（`src/engine/`） |
| `web/` | React + Vite の GUI |
| `shared/map/` | マップデータ（地域・隣接）と描画用アセット |
| `docs/` | ルール仕様、裁定仕様、テストケース、設計（英語・日本語） |
| `scripts/`、`packaging/` | パッケージ作成 |

| ドキュメント | English | 日本語 |
|--------------|---------|--------|
| ルール仕様（ルール ID 付き） | [docs/en/rules.md](docs/en/rules.md) | [docs/ja/rules.md](docs/ja/rules.md) |
| マップ リファレンス | [docs/en/map.md](docs/en/map.md) | [docs/ja/map.md](docs/ja/map.md) |
| 裁定仕様とプロジェクト裁定 | [docs/en/adjudication.md](docs/en/adjudication.md) | [docs/ja/adjudication.md](docs/ja/adjudication.md) |
| ルールブックの例によるテストケース | [docs/en/test-cases.md](docs/en/test-cases.md) | [docs/ja/test-cases.md](docs/ja/test-cases.md) |
| 設計 | [docs/en/architecture.md](docs/en/architecture.md) | [docs/ja/architecture.md](docs/ja/architecture.md) |
| 用語集（英日） | [docs/glossary.md](docs/glossary.md) | ← |

英語ドキュメントが正で、日本語ドキュメントはその翻訳です。

## ライセンスとクレジット

- **コードとドキュメント**：[MIT License](LICENSE)。ただし次のファイルを除きます。
- **地図**：`shared/map/standard.svg`、`standard.layout.json`、`standard.graph.svg` と `shared/map/source/` は、
  Martin Asal 作「Diplomacy.svg」（Wikimedia Commons）を改変したもので、[CC BY-SA 3.0](https://creativecommons.org/licenses/by-sa/3.0/deed.ja)
  で提供します。[shared/map/CREDITS.md](shared/map/CREDITS.md) を参照してください。
- **ルール**：ドキュメントは *Diplomacy* ルールブック第5版（2008年、Avalon Hill / Wizards of the Coast、
  <https://media.wizards.com/2015/downloads/ah/diplomacy_rules.pdf>）を言い換えたものです。ルールブック自体は含みません。
- **サードパーティのソフトウェア**：配布用パッケージには、プログラムに組み込んだ Rust のクレートと JavaScript ライブラリの
  ライセンスをまとめた `THIRD_PARTY_LICENSES.txt` を同梱しています。
- *Diplomacy* は Hasbro の商標です。本プロジェクトは非公式のファン製プロジェクトです。
