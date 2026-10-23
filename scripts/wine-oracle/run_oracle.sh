#!/usr/bin/env bash
# Wine oracle harness for GenMSI (Phase 0).
#
# Runs genmsi.exe fully headless on a private Xvfb display, opens a .gmdl
# model (accepted on the command line), runs the simulation, and extracts the
# "Reporte" and "Código GPSS" simulator tabs through the X11 clipboard into
# text files. Nothing is rendered on or touches the user's desktop.
#
# Usage: scripts/wine-oracle/run_oracle.sh <model.gmdl> [outdir]
#
# Requirements: wine, xvfb (Xvfb), xdotool, python3 + tkinter.
# Validated against genmsi v1.2020.07.28 BETA1! under Wine 11.19
# (see docs/oracle-wine.md).

set -euo pipefail

REPO="$(cd "$(dirname "$0")/../.." && pwd)"
EXE="$REPO/Genmsi_20230711.backup/genmsi.exe"   # always the backup copy
# The prefix the original runs under.  `GENMSI_WINEPREFIX` lets a second
# session run the original at the same time without sharing the prefix (the
# `wineserver -k` of one run would kill the other's simulation).
WINEPREFIX="${GENMSI_WINEPREFIX:-$REPO/scripts/wine-oracle/wineprefix}"
OUT="${2:-$REPO/scripts/wine-oracle/out}"
XDISPLAY="${XDISPLAY:-:99}"

MODEL="$(realpath "$1")"
NAME="$(basename "$MODEL")"
NAME="${NAME%.*}"
SAFE_NAME="$(echo "$NAME" | tr ' ' '_')"
ZPATH="Z:$(echo "$MODEL" | sed 's|/|\\\\|g')"
mkdir -p "$OUT"
export WINEPREFIX DISPLAY="$XDISPLAY"

[ -f "$EXE" ] || { echo "error: $EXE not found (backup the original first)" >&2; exit 1; }

# --- private Xvfb display ---------------------------------------------------

start_xvfb() {
  if xdpyinfo -display "$XDISPLAY" >/dev/null 2>&1; then
    return 1   # already running
  fi
  nohup Xvfb "$XDISPLAY" -screen 0 1280x800x24 -nolisten tcp \
    > "$OUT/xvfb.log" 2>&1 &
  for _ in $(seq 1 15); do
    xdpyinfo -display "$XDISPLAY" >/dev/null 2>&1 && return 0
    sleep 0.5
  done
  echo "error: Xvfb failed to start on $XDISPLAY" >&2
  return 1
}

WE_OWN_XVFB=0
if start_xvfb; then WE_OWN_XVFB=1; fi
trap '[ "$WE_OWN_XVFB" = 1 ] && pkill -f "Xvfb $XDISPLAY" 2>/dev/null || true' EXIT

# --- window helpers ---------------------------------------------------------

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

# Coordinates are relative to the app window, so window placement does not
# matter (there is no window manager on the private display).
click_in() {  # click_in <rel_x> <rel_y> [settle_seconds]
  local wid geo_x geo_y
  wid="$(app_window)"
  eval "$(xdotool getwindowgeometry --shell "$wid")"   # sets X, Y, ...
  xdotool mousemove $((X + $1)) $((Y + $2)) click 1
  sleep "${3:-0.7}"
}

keys_in() {  # keys_in <rel_x> <rel_y> <keys...>
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

# Select the whole text of the simulator output pane and copy it via the X11
# clipboard (Delphi rich edit: Ctrl+A unsupported, use Ctrl+Home + Ctrl+Shift+End).
extract_pane() {  # extract_pane <file>
  keys_in 456 294 ctrl+Home ctrl+shift+End ctrl+c
  python3 "$REPO/scripts/wine-oracle/read_clipboard.py" > "$1" 2>/dev/null
  if [ ! -s "$1" ]; then
    echo "error: clipboard copy produced an empty $1" >&2
    return 1
  fi
  echo "extracted $(wc -l < "$1") lines -> $1"
}

# --- run --------------------------------------------------------------------

nohup wine "$EXE" "$ZPATH" > "$OUT/$SAFE_NAME.wine.log" 2>&1 &
WID="$(wait_window)"
echo "GenMSI window: $WID on $XDISPLAY"
sleep 5   # give the Delphi app time to finish loading the model

# Menu Modelo -> Simular (F9): opens the Simulador panel
click_in 157 14    # "Modelo"
click_in 183 60    # "Simular"
sleep 1
# Simulator toolbar: blue globe = run (Start defaults to 1)
click_in 364 156 1.5
sleep 2   # small models finish in well under a second

# Simulator output tabs: Reporte | Código GPSS
extract_pane "$OUT/$SAFE_NAME.report.txt"
click_in 256 181   # "Código GPSS" tab
extract_pane "$OUT/$SAFE_NAME.gpss.txt"

# Stop the Wine session cleanly (keep Xvfb if we did not start it)
wineserver -k 2>/dev/null || true
echo "done: $OUT/$SAFE_NAME.{report,gpss}.txt"
