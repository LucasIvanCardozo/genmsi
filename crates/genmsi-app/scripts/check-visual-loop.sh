#!/usr/bin/env bash
#
# The visual loop of genmsi-app as one runnable script.
#
# This is the single source of truth of `docs/app-visual-loop.md`: the manual
# steps in the document are these commands, spelled out one by one. The script
# starts the Vite dev server, drives it with `agent-browser`, takes the four
# screenshots of the loop in `${GENMSI_SHOTS:-/tmp/genmsi-app}` and exits
# non-zero if any step fails. The server is killed on every exit path.
#
# Usage:
#   cargo build -p genmsi-cli -p genmsi-app-engine   # once, before the loop
#   crates/genmsi-app/scripts/check-visual-loop.sh
#
# Environment:
#   GENMSI_SHOTS           where the screenshots go (default /tmp/genmsi-app)
#   GENMSI_BIN, GENMSI_APP_BIN  engine binaries, as in `vite.config.ts`
#   AGENT_BROWSER_SESSION  is set by the script (one session per run)

set -euo pipefail

APP_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$APP_DIR/../../.." && pwd)"
PORT=5173
SHOTS="${GENMSI_SHOTS:-/tmp/genmsi-app}"
LOG="$SHOTS/vite.log"
MODEL="fixtures/models/TP6__Clase 6_1_PuertaBanco.gmdl"
REFUSED="fixtures/models/TP9__tp9.gmdl"
# The first block of the model's flow (`Generate`, caption "Clientes").
FLOW_NODE='[data-id="6"]'
# Only the engine's refusal notice carries this headline; a node label never
# does, which is what made a bare "Unlink" wait match the graph too early.
REFUSAL_TEXT="El modelo no compila"

mkdir -p "$SHOTS"

SERVER_PID=""
cleanup() {
  if [[ -n "$SERVER_PID" ]] && kill -0 "$SERVER_PID" 2>/dev/null; then
    kill "$SERVER_PID" 2>/dev/null || true
    wait "$SERVER_PID" 2>/dev/null || true
  fi
}
trap cleanup EXIT

die() { echo "check-visual-loop: $*" >&2; exit 1; }

# The binaries the dev bridge spawns must be there: the loop reviews the app
# against the real engine, never against a stale or missing one.
GENMSI_BIN="${GENMSI_BIN:-$REPO_ROOT/target/debug/genmsi}"
GENMSI_APP_BIN="${GENMSI_APP_BIN:-$REPO_ROOT/target/debug/genmsi-app-engine}"
[[ -x "$GENMSI_BIN" ]] || die "$GENMSI_BIN is missing; run cargo build -p genmsi-cli -p genmsi-app-engine"
[[ -x "$GENMSI_APP_BIN" ]] || die "$GENMSI_APP_BIN is missing; run cargo build -p genmsi-cli -p genmsi-app-engine"

# 1. The dev server in the background. The port is fixed (strictPort), so a
#    second server fails loudly instead of moving the address.
curl -sf "http://127.0.0.1:$PORT/" >/dev/null 2>&1 &&
  die "port $PORT is already in use; close the other dev server first"
(cd "$APP_DIR" && bun run dev) >"$LOG" 2>&1 &
SERVER_PID=$!
echo "==> dev server on port $PORT (pid $SERVER_PID, log $LOG)"
for _ in $(seq 1 100); do
  curl -sf "http://127.0.0.1:$PORT/" >/dev/null 2>&1 && break
  kill -0 "$SERVER_PID" 2>/dev/null || die "the dev server died; see $LOG"
  sleep 0.3
done
curl -sf "http://127.0.0.1:$PORT/" >/dev/null 2>&1 || die "the dev server never answered; see $LOG"

# 2. The browser, with one session for this run.
export AGENT_BROWSER_SESSION="genmsi-visual-loop-$$"
agent-browser open "http://127.0.0.1:$PORT/"
agent-browser wait --text "GenMSI"

# 3. The model picker. The options arrive after `GET /api/models`, so the loop
#    waits for one model's name before selecting: selecting earlier is a
#    silent no-op.
agent-browser wait --text "TP2__problema1"

# The reviewed model: the graph, framed with the model's own geometry.
agent-browser select '[aria-label="Modelo"]' "$MODEL"
agent-browser wait --text "Puerta Banco"
agent-browser screenshot "$SHOTS/01-grafo.png"

# The first block of the flow, selected by id (a click by text lands on the
# node's cover), and its properties panel.
agent-browser click ".react-flow__node$FLOW_NODE"
agent-browser screenshot "$SHOTS/02-propiedades.png"

# The run and its report.
agent-browser find role button click --name "Simular"
agent-browser wait --text "Bloques"
agent-browser screenshot "$SHOTS/03-reporte.png"

# A model the engine refuses: the engine's message, not an empty graph.
agent-browser select '[aria-label="Modelo"]' "$REFUSED"
agent-browser wait --text "$REFUSAL_TEXT"
# The screenshot must show the settled state, not the transition: the previous
# model stays on screen while the next one loads.
agent-browser wait --fn '!document.body.innerText.includes("Cargando")'
agent-browser screenshot "$SHOTS/04-error.png"

agent-browser close
echo "==> the visual loop ran; the screenshots are in $SHOTS"
