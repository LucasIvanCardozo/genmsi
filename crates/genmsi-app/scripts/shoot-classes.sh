#!/usr/bin/env bash
#
# The photo tour of the block classes.
#
# For every class the catalogue offers, a real picture of the block as the
# app draws it **during a run**, with the live figure it shows. The pictures
# are taken from the app itself, never from a drawing.
#
# The tour needs models that run: a block only has a live figure while the
# replay is playing. The script runs each model, lets the clock advance and
# photographs every block of it, one file per class (`<class>.png`).
#
# Usage:
#   crates/genmsi-app/scripts/shoot-classes.sh MODEL [MODEL...]
#
# Environment:
#   GENMSI_SHOTS           where the pictures go (default /tmp/fotos-clases)
#   GENMSI_ZOOM            the zoom of the canvas for the pictures (default 2)
#   AGENT_BROWSER_SESSION  the browser session (one per run)
#
# The models of `fixtures/models/` are read-only oracle material: the script
# never writes them.

set -euo pipefail

APP_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REPO_ROOT="$(cd "$APP_DIR/../.." && pwd)"
OUT="${GENMSI_SHOTS:-/tmp/fotos-clases}"
ZOOM="${GENMSI_ZOOM:-2}"
PORT=5173

die() { echo "shoot-classes: $*" >&2; exit 1; }

ENGINE="$REPO_ROOT/target/debug/genmsi-app-engine"
[[ -x "$ENGINE" ]] || die "$ENGINE is missing; run cargo build -p genmsi-app-engine"

mkdir -p "$OUT"

SERVER_PID=""
cleanup() {
  if [[ -n "$SERVER_PID" ]] && kill -0 "$SERVER_PID" 2>/dev/null; then
    kill "$SERVER_PID" 2>/dev/null || true
    wait "$SERVER_PID" 2>/dev/null || true
  fi
}
trap cleanup EXIT

[[ $# -ge 1 ]] || die "usage: $0 MODEL [MODEL...]"

# The dev server: the pictures are of the app.
if ! curl -sf "http://127.0.0.1:$PORT/" >/dev/null 2>&1; then
  (cd "$APP_DIR" && bun run dev) >"$OUT/vite.log" 2>&1 &
  SERVER_PID=$!
  for _ in $(seq 1 100); do
    curl -sf "http://127.0.0.1:$PORT/" >/dev/null 2>&1 && break
    kill -0 "$SERVER_PID" 2>/dev/null || die "the dev server died; see $OUT/vite.log"
    sleep 0.3
  done
fi

export AGENT_BROWSER_SESSION="${AGENT_BROWSER_SESSION:-genmsi-fotos-$$}"
agent-browser open "http://127.0.0.1:$PORT/" >/dev/null
agent-browser set viewport 1500 950 >/dev/null
agent-browser wait --text "GenMSI" >/dev/null
# The picture of an element lines up with the page only when the page is not
# zoomed: a zoom left by another session would move every block.
agent-browser eval "document.documentElement.style.zoom = '1'" >/dev/null

# Open a model the way the reader does: through the recent models.
open_model() {
  local path="$1" file
  file="$(basename "$path")"
  agent-browser eval "localStorage.setItem('genmsi.recent-models', JSON.stringify(['$path']))" >/dev/null
  agent-browser reload >/dev/null
  agent-browser wait --text "GenMSI" >/dev/null
  agent-browser eval "(() => { var b = Array.from(document.querySelectorAll('button')).find((x) => (x.getAttribute('aria-label') || '').includes('modelos abiertos')); if (!b) return 'no recent button'; b.click(); return 'menu open'; })()" >/dev/null
  sleep 0.5
  agent-browser eval "(() => { var btn = Array.from(document.querySelectorAll('button')).find((b) => b.textContent.includes('$file')); if (!btn) return 'not in the recents'; btn.click(); return 'opened'; })()" >/dev/null
  sleep 2
}

# A point of the canvas with no block on it, to press the mouse on.
empty_spot() {
  agent-browser eval "(() => { var w = innerWidth, h = innerHeight; for (let y = 120; y < h - 120; y += 24) { for (let x = 8; x < 220; x += 24) { var el = document.elementFromPoint(x, y); if (el && el.classList.contains('react-flow__pane')) return x + ',' + y; } } return ''; })()" | tr -d '"'
}

# The place of the block in the picture: x, y, width and height.
node_rect() {
  agent-browser eval "(() => { var el = document.querySelector('.react-flow__node[data-id=\"$1\"]'); if (!el) return 'none'; var r = el.getBoundingClientRect(); return [Math.round(r.x), Math.round(r.y), Math.round(r.width), Math.round(r.height)].join(' '); })()" | tr -d '"'
}

# The place of the canvas in the picture: x, y, width and height.
pane_rect() {
  agent-browser eval "(() => { var el = document.querySelector('.react-flow__pane'); if (!el) return 'none'; var r = el.getBoundingClientRect(); return [Math.round(r.x), Math.round(r.y), Math.round(r.width), Math.round(r.height)].join(' '); })()" | tr -d '"'
}

# Where the block is and how much the canvas has to move for it to be seen.
block_rect() {
  agent-browser eval "(() => { var el = document.querySelector('.react-flow__node[data-id=\"$1\"]'); if (!el) return 'none'; var r = el.getBoundingClientRect(); var m = 16; var w = innerWidth, h = innerHeight; let dx = 0, dy = 0; if (r.left < m) dx = r.left - m; else if (r.right > w - m) dx = r.right - (w - m); if (r.top < m) dy = r.top - m; else if (r.bottom > h - m) dy = r.bottom - (h - m); return Math.round(dx) + ' ' + Math.round(dy); })()" | tr -d '"'
}

for model in "$@"; do
  path="$REPO_ROOT/$model"
  [[ -f "$path" ]] || path="$model"
  [[ -f "$path" ]] || die "the model $model does not exist"
  echo "==> $model"
  open_model "$path"
  # The run and the trace: the figures of the blocks come from the replay.
  agent-browser find role button click --name "Ejecutar" >/dev/null 2>&1 || true
  sleep 9
  # The blocks of the model, with the class of each one.
  "$ENGINE" model "$path" | python3 -c '
import json, sys
model = json.load(sys.stdin)
for block in model["blocks"]:
    if not block.get("layout"):
        continue
    print(block["id"], block["class"])
' | while read -r id class; do
    [[ -f "$OUT/$class.png" ]] && continue
    # The block is made bigger around its own middle, with the wheel of
    # the canvas: the zoom keeps the point under the pointer where it is,
    # so the block grows where it is and does not leave the picture.
    agent-browser eval "(() => { var el = document.querySelector('.react-flow__node[data-id=\"$id\"]'); var pane = document.querySelector('.react-flow__pane'); if (!el || !pane) return 'none'; var r = el.getBoundingClientRect(); var cx = r.x + r.width / 2, cy = r.y + r.height / 2; for (let i = 0; i < 3; i++) { pane.dispatchEvent(new WheelEvent('wheel', {deltaY: -120, clientX: cx, clientY: cy, bubbles: true, cancelable: true})); } return 'ok'; })()" >/dev/null
    sleep 0.8
    # The place of the block is read once the zoom has settled: while it
    # moves, the picture and the rectangle do not agree.
    rect="$(node_rect "$id")"
    [[ "$rect" == "none" || -z "$rect" ]] && continue
    read -r nx ny nw nh <<<"$rect"
    # The block alone in the picture: the other blocks and the links are
    # hidden while it is taken, so that the reader judges the block and
    # not its neighbours.
    agent-browser eval "(() => { var lienzo = document.querySelector('.react-flow'); if (!lienzo) return 'no flow'; let node = lienzo; while (node && node.parentElement) { var parent = node.parentElement; for (var child of parent.children) { if (child !== node) child.style.visibility = 'hidden'; } node = parent; } document.querySelectorAll('.react-flow__node').forEach((n) => { if (n.getAttribute('data-id') !== '$id') n.style.visibility = 'hidden'; }); var capa = document.querySelector('.react-flow__pane'); var raiz = document.querySelector('.react-flow'); if (capa && raiz) { for (var child of raiz.children) { if (!child.contains(capa)) child.style.visibility = 'hidden'; } } document.querySelectorAll('.react-flow__edges, .react-flow__controls, .react-flow__minimap, .react-flow__panel, .react-flow__attribution').forEach((e) => { e.style.visibility = 'hidden'; }); return 'aislado'; })()" >/dev/null
    # The picture of the whole window, and the block cut out of it: the
    # picture of an element does not line up when the page moves.
    agent-browser screenshot "$OUT/.ventana.png" >/dev/null 2>&1 || continue
    # The picture is cut with room around the block: the text of a block
    # can live a little outside its own box.
    m=14
    magick "$OUT/.ventana.png" -crop "$((nw + 2 * m))x$((nh + 2 * m))+$((nx - m))+$((ny - m))" +repage "$OUT/$class.png" 2>/dev/null || true
    agent-browser eval "(() => { document.querySelectorAll('*').forEach((e) => { if (e.style.visibility === 'hidden') e.style.visibility = ''; }); var el = document.querySelector('.react-flow__node[data-id=\"$id\"]'); var pane = document.querySelector('.react-flow__pane'); if (el && pane) { var r = el.getBoundingClientRect(); var cx = r.x + r.width / 2, cy = r.y + r.height / 2; for (let i = 0; i < 3; i++) { pane.dispatchEvent(new WheelEvent('wheel', {deltaY: 120, clientX: cx, clientY: cy, bubbles: true, cancelable: true})); } } return 'restaurado'; })()" >/dev/null
  done
done

echo "==> the pictures are in $OUT"
ls "$OUT"/*.png 2>/dev/null | wc -l
