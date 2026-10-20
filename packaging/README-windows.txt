diplomacy-gm (Windows)
======================

[日本語]

起動
  diplomacy-gm.exe をダブルクリックします。黒いウィンドウ（サーバー）が開き、
  少し待つとブラウザでディプロマシー GM の画面が開きます。
  開かない場合は、ウィンドウに表示されたアドレス（例: http://127.0.0.1:8080/）を
  ブラウザで開いてください。

終了
  黒いウィンドウを閉じるか、ウィンドウで Ctrl+C を押します。

セーブデータ
  diplomacy-gm.exe と同じフォルダの data\games に、ゲームごとに 1 つの JSON
  ファイルとして自動で保存されます。バックアップはこのフォルダをコピーしてください。
  新しい版に入れ替えるときは、diplomacy-gm.exe だけを置き換えれば data は引き継がれます。

初回の警告
  署名していないプログラムのため、初回に「Windows によって PC が保護されました」と
  表示されることがあります。「詳細情報」→「実行」で起動できます。

その他
  ・自分のパソコンからの接続だけを受け付けます（ほかのパソコンからは使えません）。
  ・8080 番ポートが使用中の場合は 8081〜8089 番を自動で使います。
  ・オプション: diplomacy-gm.exe --help


[English]

Start
  Double-click diplomacy-gm.exe. A console window (the server) opens, and shortly after the
  Diplomacy GM page opens in your browser. If it does not, open the address shown in the
  window (e.g. http://127.0.0.1:8080/) yourself.

Stop
  Close the console window, or press Ctrl+C in it.

Saved games
  Saved automatically as one JSON file per game in data\games next to diplomacy-gm.exe.
  Copy that folder to back up. To update, replace only diplomacy-gm.exe; data is kept.

First-run warning
  The program is not signed, so Windows may show "Windows protected your PC" the first time.
  Choose "More info" -> "Run anyway".

Notes
  - Only connections from this computer are accepted.
  - If port 8080 is busy, ports 8081-8089 are tried.
  - Options: diplomacy-gm.exe --help


Credits / ライセンス
  The board map is adapted from "Diplomacy.svg" by Martin Asal (Wikimedia Commons),
  CC BY-SA 3.0 — see CREDITS.md. Unofficial fan tool; "Diplomacy" is a trademark of Hasbro.
  盤面の地図は Martin Asal 作「Diplomacy.svg」（CC BY-SA 3.0）を改変したものです（CREDITS.md 参照）。
