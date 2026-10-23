#!/usr/bin/env bash
# Run the ORIGINAL GenMSI (under Wine) on a probe model and save its answer.
#
# This is a thin copy of `scripts/wine-oracle/run_oracle.sh` (same private
# Xvfb display, same clicks, same clipboard extraction) with one difference:
# it runs a *copy* of the original executable under a different file name.
#
# Why: another session working on the same machine runs
# `pkill -f "[g]enmsi.exe"` between its own probes, which kills every
# `genmsi.exe` process on the machine, including the one this harness
# starts.  The copy is byte-identical (the script checks the md5) and keeps
# the original's resources (`gClasses.xml`, `gmacroconfig.gmcfg`, ...) next
# to it, so the program that answers is the same program.
#
# Usage: scripts/wine-oracle/probes/reporte/run-probe-reporte.sh <model.gmdl> <outdir>

set -euo pipefail

REPO="$(cd "$(dirname "$0")"/../../../.. && pwd)"
ORIGINAL="$REPO/Genmsi_20230711.backup/genmsi.exe"
APPDIR="${GENMSI_APP_DIR:-/tmp/probe-reporte/app}"
EXE="$APPDIR/genmsi_original.exe"
WINEPREFIX="${GENMSI_WINEPREFIX:-$REPO/scripts/wine-oracle/wineprefix}"
OUT="${2:-/tmp/probe-reporte/out}"
XDISPLAY="${XDISPLAY:-:96}"

MODEL="$(realpath "$1")"
NAME="$(basename "$MODEL")"
NAME="${NAME%.*}"
SAFE_NAME="$(echo "$NAME" | tr ' ' '_')"
ZPATH="Z:$(echo "$MODEL" | sed 's|/|\\\\|g')"
mkdir -p "$OUT"
export WINEPREFIX DISPLAY="$XDISPLAY"

[ -f "$ORIGINAL" ] || { echo "error: $ORIGINAL not found" >&2; exit 1; }

# The copy of the original, with its resources, under a name that the
# concurrent session's `pkill -f genmsi.exe` does not match.
if [ ! -f "$EXE" ]; then
  mkdir -p "$APPDIR"
  cp -a "$REPO/Genmsi_20230711.backup/." "$APPDIR/"
  mv "$APPDIR/genmsi.exe" "$EXE"
fi
if ! cmp -s "$ORIGINAL" "$EXE"; then
  echo "error: $EXE is not a copy of $ORIGINAL" >&2
  exit 1
fi

start_xvfb() {
  if xdpyinfo -display "$XDISPLAY" >/dev/null 2>&1; then
    return 1
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

click_in() {  # click_in <rel_x> <rel_y> [settle_seconds]
  local wid geo_x geo_y
  wid="$(app_window)"
  eval "$(xdotool getwindowgeometry --shell "$wid")"
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

extract_pane() {  # extract_pane <file>
  keys_in 456 294 ctrl+Home ctrl+shift+End ctrl+c
  python3 "$REPO/scripts/wine-oracle/read_clipboard.py" > "$1" 2>/dev/null
  if [ ! -s "$1" ]; then
    echo "error: clipboard copy produced an empty $1" >&2
    return 1
  fi
  echo "extracted $(wc -l < "$1") lines -> $1"
}

nohup wine "$EXE" "$ZPATH" > "$OUT/$SAFE_NAME.wine.log" 2>&1 &
WID="$(wait_window)"
echo "GenMSI window: $WID on $XDISPLAY"
sleep 5

click_in 157 14    # "Modelo"
click_in 183 60    # "Simular"
sleep 1
click_in 364 156 1.5   # blue globe = run
sleep 2

extract_pane "$OUT/$SAFE_NAME.report.txt"
click_in 256 181   # "Código GPSS" tab
extract_pane "$OUT/$SAFE_NAME.gpss.txt"

wineserver -k 2>/dev/null || true
echo "done: $OUT/$SAFE_NAME.{report,gpss}.txt"
