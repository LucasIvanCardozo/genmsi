#!/usr/bin/env bash
#
# The visual loop of genmsi-app as one runnable script.
#
# This is the single source of truth of `docs/app-visual-loop.md`: the manual
# steps in the document are these commands, spelled out one by one. The script
# starts the Vite dev server, drives it with `agent-browser`, takes the
# screenshots of the loop in `${GENMSI_SHOTS:-/tmp/genmsi-app}` and exits
# non-zero if any step fails. The server is killed on every exit path.
#
# The loop covers the editing: it creates a node, connects it, drags it,
# undoes, redoes, saves, reopens the file and shows that the change is
# there, and then the monitor: it runs the model, replays the trace and
# shows the charts of the run. The model it edits is a COPY in the
# development work directory: the reference models of `fixtures/models` are
# read-only oracle material.
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
# The model of the monitor: it has queues, facilities and tables, so the
# charts of the run have something to draw.
MONITOR_MODEL="fixtures/models/TP4__supermercado.gmdl"
# The bench of the shapes: every class of the catalogue, unconnected, in a
# grid. It is made by `scripts/make-catalogue-model.py`.
CATALOGUE_MODEL="crates/genmsi-app/fixtures/catalogo-clases.gmdl"
REFUSED="fixtures/models/TP9__tp9.gmdl"
# The copy the loop edits: the development work directory is the only place
# the development path writes.
WORK="crates/genmsi-app/.devmodels"
EDITED="$WORK/visual.gmdl"
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

# The centre of an element, in the coordinates the mouse takes.
centre_of() {
  local selector="$1"
  agent-browser eval "(() => { const el = document.querySelector('$selector'); if (!el) return ''; const r = el.getBoundingClientRect(); return [Math.round(r.x + r.width / 2), Math.round(r.y + r.height / 2)].join(','); })()" |
    tr -d '"'
}

# Drag one element onto another with the mouse: React Flow listens to the
# pointer events, so the drag has to be a real one.
drag_handle() {
  local from="$1" to="$2"
  local from_centre to_centre
  from_centre=$(centre_of "$from")
  to_centre=$(centre_of "$to")
  [[ "$from_centre" == ,* || "$from_centre" == "" ]] && die "the handle $from is not on the picture"
  [[ "$to_centre" == ,* || "$to_centre" == "" ]] && die "the handle $to is not on the picture"
  agent-browser mouse move "${from_centre%,*}" "${from_centre#*,}"
  agent-browser mouse down left
  agent-browser mouse move "${to_centre%,*}" "${to_centre#*,}" --duration 400 --steps 12
  agent-browser mouse up left
}

# Where a block is on the canvas, as the graph draws it.
node_position() {
  agent-browser eval "(() => { const el = document.querySelector('.react-flow__node[data-id=\"$1\"]'); return el ? el.style.transform : ''; })()" |
    tr -d '"'
}

# Drag a node by its cover, the way a user does it.
drag_node() {
  local selector="$1" dx="$2" dy="$3"
  local centre
  centre=$(centre_of "$selector")
  [[ "$centre" == ,* || "$centre" == "" ]] && die "the node $selector is not on the picture"
  local x="${centre%,*}" y="${centre#*,}"
  agent-browser mouse move "$x" "$y"
  agent-browser mouse down left
  agent-browser mouse move "$((x + dx))" "$((y + dy))" --duration 400 --steps 12
  agent-browser mouse up left
}

# Move a range input, the way React sees it.
#
# React reads the value of the input from its own tracker, so the value is
# set through the native setter and the input event is dispatched: the
# component sees the change and the picture follows.
set_range_middle() {
  local selector="$1"
  agent-browser eval "(() => { const input = document.querySelector('$selector'); if (!input) return 'not found'; const max = Number(input.max); const value = String(Math.floor(max / 2)); const setter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value').set; setter.call(input, value); input.dispatchEvent(new Event('input', { bubbles: true })); return value; })()" >/dev/null
}

# Open a model in the browser.
#
# The app has no dropdown of models: it opens them with the dialog of the
# system, and the browser has no dialog. The recents are the way in — the
# browser keeps them in its own storage — so the loop leaves the path there,
# reloads the page and clicks the model in the menu.
open_model() {
  local path="$1"
  local file stem
  file="$(basename "$path")"
  stem="$(basename "$path" .gmdl)"
  agent-browser eval "localStorage.setItem('genmsi.recent-models', JSON.stringify(['$path']))" >/dev/null
  agent-browser reload >/dev/null
  agent-browser wait --text "GenMSI"
  # The recents live behind an icon button, not one with a name: it is the
  # "models opened lately" button of the header, and it stays disabled while
  # the list is empty, so the recents have to be written before the reload.
  agent-browser eval "(() => { const b = Array.from(document.querySelectorAll('button')).find((x) => (x.getAttribute('aria-label') || '').includes('modelos abiertos')); if (!b) return 'no recent button'; if (b.disabled) return 'the recent button is disabled'; b.click(); return 'menu open'; })()" >/dev/null
  agent-browser eval "(() => { const button = Array.from(document.querySelectorAll('button')).find((b) => b.textContent.includes('$file')); if (!button) return 'the model is not in the recents'; button.click(); return 'opened'; })()" >/dev/null
  # The header carries the name of the model that is open.
  agent-browser wait --text "$stem"
}

# 1. The copy the loop edits. The reference model stays untouched.
mkdir -p "$REPO_ROOT/$WORK"
cp "$REPO_ROOT/$MODEL" "$REPO_ROOT/$EDITED"
# The ids the new blocks get: the session assigns the next free id, which is
# the largest id of the model plus one.
BASE_ID=$("$GENMSI_APP_BIN" model "$EDITED" |
  python3 -c 'import json,sys; print(max(b["id"] for b in json.load(sys.stdin)["blocks"]) + 1)')
# The blocks the picture shows: the ones the model places on the canvas.
NODE_COUNT=$("$GENMSI_APP_BIN" model "$EDITED" |
  python3 -c 'import json,sys; print(len([b for b in json.load(sys.stdin)["blocks"] if b["layout"]]))')
GENERATE_ID=$BASE_ID
TERMINATE_ID=$((BASE_ID + 1))
echo "==> the loop edits $EDITED (the new blocks are $GENERATE_ID and $TERMINATE_ID)"

# 2. The dev server in the background. The port is fixed (strictPort), so a
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

# 3. The browser, with one session for this run.
export AGENT_BROWSER_SESSION="genmsi-visual-loop-$$"
agent-browser open "http://127.0.0.1:$PORT/"
# A tall window: the graph of a reference model is a tall column, and the
# picture has to be readable in the screenshot.
agent-browser set viewport 1440 1000
agent-browser wait --text "GenMSI"

# 4. The reviewed model: the graph, framed with the model's own geometry.
open_model "$MODEL"
agent-browser wait --text "Puerta Banco"
agent-browser screenshot "$SHOTS/01-grafo.png"

# The first block of the flow, selected by id (a click by text lands on the
# node's cover), and its properties panel.
agent-browser click ".react-flow__node$FLOW_NODE"
agent-browser screenshot "$SHOTS/02-propiedades.png"

# The catalogue of the classes: the list the reader creates blocks from, in
# two columns, each entry with the silhouette of its class as its thumbnail.
agent-browser click '[data-monitor-aside="bloques"]'
agent-browser wait --text "Decisi"
agent-browser screenshot "$SHOTS/03-catalogo.png"

# The run and its report: the report is one of the panels of the window.
agent-browser find role button click --name "Simular"
agent-browser click '[data-monitor-panel="reporte"]'
agent-browser wait --text "Bloques"
agent-browser screenshot "$SHOTS/04-reporte.png"
# The editing below happens on the graph, so the window goes back to it.
agent-browser click '[data-monitor-panel="reproduccion"]'
agent-browser wait --fn "document.querySelectorAll('.react-flow__node').length > 0"

# 5. The editing steps, on the copy of the model.
open_model "$EDITED"
agent-browser wait --text "Puerta Banco"

# A new node, created from the catalogue. The classes live in the lateral
# panel, in the tab "Bloques".
agent-browser click '[data-monitor-aside="bloques"]'
agent-browser wait --text "Generar"
# The list of classes scrolls: the class has to be on the picture before it
# can be clicked.
agent-browser scrollintoview '[data-block-class="Generate"]'
agent-browser click '[data-block-class="Generate"]'
agent-browser wait --fn "document.querySelectorAll('.react-flow__node').length > $NODE_COUNT"
agent-browser screenshot "$SHOTS/05-nodo-nuevo.png"

# A second node, and the connection between the two: the new Generate feeds
# the new Terminate, so the link does not replace any other one.
agent-browser scrollintoview '[data-block-class="Terminate"]'
agent-browser click '[data-block-class="Terminate"]'
agent-browser wait --fn "document.querySelectorAll('.react-flow__node').length > $((NODE_COUNT + 1))"
# The picture is framed again, so the new blocks are on it.
agent-browser find role button click --name "Encuadrar"
agent-browser wait 300

drag_handle ".react-flow__node[data-id=\"$GENERATE_ID\"] .react-flow__handle-bottom" \
  ".react-flow__node[data-id=\"$TERMINATE_ID\"] .react-flow__handle-top"
agent-browser wait --fn "document.querySelectorAll('.react-flow__edge').length > 9"
agent-browser screenshot "$SHOTS/06-conectado.png"

# The node is dragged: one drag is one undo entry.
POSITION_BEFORE=$(node_position "$GENERATE_ID")
drag_node ".react-flow__node[data-id=\"$GENERATE_ID\"]" -120 -60
agent-browser wait --fn "document.querySelector('.react-flow__node[data-id=\"$GENERATE_ID\"]').style.transform !== '$POSITION_BEFORE'"
agent-browser screenshot "$SHOTS/07-arrastrado.png"

# Undo takes the drag back.
agent-browser press "Control+z"
agent-browser wait --fn "document.querySelector('.react-flow__node[data-id=\"$GENERATE_ID\"]').style.transform === '$POSITION_BEFORE'"
agent-browser screenshot "$SHOTS/08-deshacer.png"

# And redo puts it back.
agent-browser press "Control+Shift+z"
agent-browser wait --fn "document.querySelector('.react-flow__node[data-id=\"$GENERATE_ID\"]').style.transform !== '$POSITION_BEFORE'"
agent-browser screenshot "$SHOTS/09-rehacer.png"

# The re-layout: the button asks the engine to compose the model, and the
# blocks move to the layers of the flow. The composition is the engine's, the
# same one the MCP applies before it writes a model.
POSITION_BEFORE=$(node_position "$GENERATE_ID")
agent-browser find role button click --name "Reordenar el grafo en capas"
agent-browser wait --fn "document.querySelector('.react-flow__node[data-id=\"$GENERATE_ID\"]').style.transform !== '$POSITION_BEFORE'"
agent-browser screenshot "$SHOTS/10-reacomodado.png"

# The model is saved, and the app says so.
agent-browser find role button click --name "Guardar"
agent-browser wait --text "Se guardó"
agent-browser screenshot "$SHOTS/11-guardado.png"

# The file is reopened: the change is in the file, not only in the session.
agent-browser open "http://127.0.0.1:$PORT/"
agent-browser wait --text "GenMSI"
open_model "$EDITED"
agent-browser wait --text "Puerta Banco"
agent-browser wait --fn "document.querySelectorAll('.react-flow__node').length > $((NODE_COUNT + 1))"
agent-browser screenshot "$SHOTS/12-reabierto.png"

# A model the engine refuses: the engine's message, not an empty graph.
open_model "$REFUSED"
agent-browser wait --text "$REFUSAL_TEXT"
# The screenshot must show the settled state, not the transition: the previous
# model stays on screen while the next one loads.
agent-browser wait --fn '!document.body.innerText.includes("Cargando")'
agent-browser screenshot "$SHOTS/13-error.png"

# 6. The monitor: the run, the replay and the charts.
#
# The model of the monitor is the one with tables, queues and facilities:
# the charts need a model that has all of them.
open_model "$MONITOR_MODEL"
agent-browser wait --text "supermercado"
# The model has to be in memory before the run: the button waits for it, and
# the run of a model that is still loading would have nothing to run.
agent-browser wait --fn '!document.body.innerText.includes("Cargando")'
agent-browser wait --text "Ejecutar"
agent-browser find role button click --name "Ejecutar"
# The clock of the run advances: the monitor is playing the trace.
agent-browser wait --fn "!document.body.innerText.includes('0:00:00')"
agent-browser screenshot "$SHOTS/14-monitor.png"

# The replay is paused and put at the middle of the run, so the picture is
# the same on every run: the blocks that hold transactions are marked and the
# clock says where the replay is.
agent-browser find role button click --name "Pausar la reproducción"
set_range_middle '[aria-label="Posición de la reproducción"]'
agent-browser wait --fn 'document.querySelector("input[aria-label=\"Posición de la reproducción\"]").value === String(Math.floor(Number(document.querySelector("input[aria-label=\"Posición de la reproducción\"]").max) / 2))'
agent-browser screenshot "$SHOTS/15-monitor-reproduccion.png"

# The charts of the run: the time series of the entities and the histogram of
# the tables of the model.
agent-browser click '[data-monitor-panel="graficos"]'
agent-browser wait --text "Histograma"
agent-browser screenshot "$SHOTS/16-monitor-graficos.png"

# The report of the run, with the numbers of the instant the replay is at.
agent-browser click '[data-monitor-panel="reporte"]'
agent-browser wait --text "Colas"
agent-browser screenshot "$SHOTS/17-monitor-reporte.png"

# The bench of the shapes: the 66 classes of the catalogue, unconnected, so
# that every silhouette is seen with the same eye. The model does not
# compile (a GPSS with no links is not a model), and the app says so in
# the status line while it draws every block.
open_model "$CATALOGUE_MODEL"
agent-browser wait --text "catalogo-clases"
agent-browser screenshot "$SHOTS/18-banco-clases.png"

agent-browser close
echo "==> the visual loop ran; the screenshots are in $SHOTS"
