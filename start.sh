#!/usr/bin/env bash
# Start diplomacy-gm: build what is missing or out of date, run the server, open the browser.
#
#   ./start.sh                     # http://127.0.0.1:8080, games saved in data/games
#   ./start.sh --port 9000 --data-dir ~/diplomacy-games --no-open
#   ./start.sh --rebuild           # force a fresh build of the GUI and the server
#
# Needs Node.js (npm) and Rust (cargo). The server only accepts connections from this machine.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WEB="$ROOT/web"
DIST_INDEX="$WEB/dist/index.html"
PORT=8080
DATA_DIR="$ROOT/data/games"
OPEN=1
REBUILD=0

usage() {
  sed -n '2,8p' "$0" | sed 's/^# \{0,1\}//'
  exit "${1:-0}"
}

while [ $# -gt 0 ]; do
  case "$1" in
    --port) PORT="${2:?--port needs a value}"; shift 2 ;;
    --data-dir) DATA_DIR="${2:?--data-dir needs a value}"; shift 2 ;;
    --no-open) OPEN=0; shift ;;
    --rebuild) REBUILD=1; shift ;;
    -h|--help) usage 0 ;;
    *) echo "unknown option: $1" >&2; usage 2 ;;
  esac
done

for cmd in npm cargo; do
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "error: '$cmd' was not found. Install Node.js (npm) and Rust (cargo) first." >&2
    exit 1
  fi
done

# True if any GUI source is newer than the last build.
gui_changed() {
  [ -n "$(find "$WEB/src" "$WEB/index.html" "$WEB/package.json" "$WEB/vite.config.ts" \
    -type f -newer "$DIST_INDEX" -print 2>/dev/null | head -1)" ]
}

# 1. GUI: install dependencies once, build when its sources changed.
if [ ! -d "$WEB/node_modules" ]; then
  echo "==> Installing GUI dependencies (first run)"
  (cd "$WEB" && npm install --no-fund --no-audit)
fi
if [ "$REBUILD" = 1 ] || [ ! -f "$DIST_INDEX" ] || gui_changed; then
  echo "==> Building the GUI"
  (cd "$WEB" && npm run --silent build)
fi

# 2. Server: release build (cargo only rebuilds what changed).
echo "==> Building the server"
if [ "$REBUILD" = 1 ]; then
  (cd "$ROOT/server" && cargo clean --release -p diplomacy-gm-server >/dev/null 2>&1 || true)
fi
(cd "$ROOT/server" && cargo build --release --quiet)
BIN="$ROOT/server/target/release/diplomacy-gm-server"

# 3. Run, and open the browser once the server answers.
mkdir -p "$DATA_DIR"
URL="http://127.0.0.1:$PORT/"
echo "==> Starting diplomacy-gm on $URL (saves in $DATA_DIR) — press Ctrl+C to stop"

if [ "$OPEN" = 1 ]; then
  (
    for _ in $(seq 1 50); do
      if curl -fs -o /dev/null "${URL}maps/standard"; then
        if command -v open >/dev/null 2>&1; then open "$URL"
        elif command -v xdg-open >/dev/null 2>&1; then xdg-open "$URL" >/dev/null 2>&1
        else echo "Open $URL in a browser."
        fi
        exit 0
      fi
      sleep 0.2
    done
    echo "The server did not answer on $URL; see the messages above." >&2
  ) &
fi

exec "$BIN" --addr "127.0.0.1:$PORT" --data-dir "$DATA_DIR" --web-dir "$WEB/dist"
