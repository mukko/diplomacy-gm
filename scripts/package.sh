#!/usr/bin/env bash
# Build a ready-to-run package: one executable with the GUI built in, plus README and credits.
#
#   scripts/package.sh                          # for this machine
#   scripts/package.sh x86_64-pc-windows-gnu    # Windows x64, cross-compiled (needs mingw-w64)
#   scripts/package.sh x86_64-pc-windows-msvc   # Windows x64 (on Windows)
#
# Output: dist/diplomacy-gm-<target>.zip
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TARGET="${1:-}"

echo "==> Building the GUI"
(cd "$ROOT/web" && npm ci --no-fund --no-audit && npm run --silent build)

echo "==> Building the server${TARGET:+ for $TARGET}"
if [ -n "$TARGET" ]; then
  rustup target add "$TARGET" >/dev/null
  (cd "$ROOT/server" && cargo build --release --locked --target "$TARGET")
  OUT="$ROOT/server/target/$TARGET/release"
  NAME="$TARGET"
else
  (cd "$ROOT/server" && cargo build --release --locked)
  OUT="$ROOT/server/target/release"
  NAME="$(rustc -vV | sed -n 's/^host: //p')"
fi

case "$NAME" in
  *windows*) EXE="diplomacy-gm-server.exe"; DEST_EXE="diplomacy-gm.exe"; README="README-windows.txt" ;;
  *) EXE="diplomacy-gm-server"; DEST_EXE="diplomacy-gm"; README="" ;;
esac

STAGE="$ROOT/dist/diplomacy-gm-$NAME"
rm -rf "$STAGE" "$STAGE.zip"
mkdir -p "$STAGE"
cp "$OUT/$EXE" "$STAGE/$DEST_EXE"
cp "$ROOT/shared/map/CREDITS.md" "$STAGE/CREDITS.md"
if [ -n "$README" ]; then
  # Windows Notepad copes best with CRLF line endings.
  sed 's/$/\r/' "$ROOT/packaging/$README" > "$STAGE/README.txt"
else
  cp "$ROOT/README.md" "$STAGE/README.md"
fi

cd "$ROOT/dist"
if command -v zip >/dev/null 2>&1; then
  zip -qr "diplomacy-gm-$NAME.zip" "diplomacy-gm-$NAME"
elif command -v powershell >/dev/null 2>&1; then
  powershell -NoProfile -Command "Compress-Archive -Path 'diplomacy-gm-$NAME' -DestinationPath 'diplomacy-gm-$NAME.zip'"
else
  echo "(no zip tool found: the package is the folder $STAGE)"
  exit 0
fi
echo "==> $ROOT/dist/diplomacy-gm-$NAME.zip"
