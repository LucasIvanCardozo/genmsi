#!/usr/bin/env bash
# Drive the original over one probe model and keep a screenshot of every step.
#
# It is the harness of `scripts/wine-oracle/run_oracle.sh` with screenshots
# added, so a run that does not finish can still be inspected (and the
# listing extracted even when the simulation failed).
#
# Usage: scripts/wine-oracle/probes/select/run-probe.sh <model.gmdl> <outdir> [display]

set -uo pipefail

REPO="$(cd "$(dirname "$0")/../../../.." && pwd)"
EXE="$REPO/Genmsi_20230711.backup/genmsi.exe"
WINEPREFIX="$REPO/scripts/wine-oracle/wineprefix"
MODEL="$(realpath "$1")"
OUT="$2"
XDISPLAY="${3:-:95}"
NAME="$(basename "$MODEL")"
NAME="${NAME%.*}"
SAFE_NAME="$(echo "$NAME" | tr ' ' '_')"
ZPATH="Z:$(echo "$MODEL" | sed 's|/|\\\\|g')"
mkdir -p "$OUT"
export WINEPREFIX DISPLAY="$XDISPLAY"

shot() { import -display "$XDISPLAY" -window root "$OUT/$SAFE_NAME.$1.png" 2>/dev/null; echo "  [shot] $1"; }

if ! xdpyinfo -display "$XDISPLAY" >/dev/null 2>&1; then
  nohup /usr/bin/Xvfb "$XDISPLAY" -screen 0 1280x800x24 -nolisten tcp > "$OUT/xvfb.log" 2>&1 &
  for _ in $(seq 1 15); do
    xdpyinfo -display "$XDISPLAY" >/dev/null 2>&1 && break
    sleep 0.5
  done
fi

app_window() { xdotool search --name "^Genmsi - " | head -1; }

wait_window() {
  local wid=""
  for _ in $(seq 1 30); do
    wid="$(app_window || true)"
    [ -n "$wid" ] && { echo "$wid"; return 0; }
    sleep 1
  done
  echo "error: GenMSI window not found on $XDISPLAY" >&2
  return 1
}

click_in() {
  local wid geo_x geo_y
  wid="$(app_window)"
  eval "$(xdotool getwindowgeometry --shell "$wid")"
  xdotool mousemove $((X + $1)) $((Y + $2)) click 1
  sleep "${3:-0.7}"
}

keys_in() {
  local wid geo_x geo_y
  wid="$(app_window)"
  eval "$(xdotool getwindowgeometry --shell "$wid")"
  xdotool mousemove $((X + $1)) $((Y + $2)) click 1
  sleep 0.4
  shift 2
  for k in "$@"; do
    xdotool key --clearmodifiers "$k"; sleep 0.3
  done
  sleep 0.5
}

extract_pane() {
  keys_in 456 294 ctrl+Home ctrl+shift+End ctrl+c
  python3 "$REPO/scripts/wine-oracle/read_clipboard.py" > "$1" 2>/dev/null
  if [ ! -s "$1" ]; then
    echo "  [warn] clipboard copy produced an empty $1" >&2
    return 1
  fi
  echo "  extracted $(wc -l < "$1") lines -> $1"
}

echo "== $SAFE_NAME on $XDISPLAY"
nohup wine "$EXE" "$ZPATH" > "$OUT/$SAFE_NAME.wine.log" 2>&1 &
WID="$(wait_window)" || { wineserver -k 2>/dev/null; exit 1; }
echo "GenMSI window: $WID"
sleep 5
shot 1-loaded

click_in 157 14    # "Modelo"
click_in 183 60    # "Simular"
sleep 1
shot 2-simulator
click_in 364 156 1.5   # run
sleep 3
shot 3-run
echo "--- windows now:"
xdotool search --name "." getwindowname %@ 2>/dev/null | sort | uniq -c | head -20

extract_pane "$OUT/$SAFE_NAME.report.txt" || true
shot 4-report
click_in 256 181   # "Código GPSS" tab
sleep 1
shot 5-gpss-tab
extract_pane "$OUT/$SAFE_NAME.gpss.txt" || true
shot 6-gpss

wineserver -k 2>/dev/null || true
echo "done"
