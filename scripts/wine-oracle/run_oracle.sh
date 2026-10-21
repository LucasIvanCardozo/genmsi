#!/usr/bin/env bash
# Wine oracle harness for GenMSI (Phase 0).
#
# Opens a .gmdl model in genmsi.exe under Wine (isolated WINEPREFIX, virtual
# desktop), runs the simulation, and extracts the Report and "GPSS Code" tabs
# through the X11 clipboard into text files.
#
# Usage: scripts/wine-oracle/run_oracle.sh <model.gmdl> [outdir]
#
# Requirements: wine, xdotool (KDE remote-control permission granted),
# spectacle (KDE) only for optional screenshots, python3 + tkinter for the
# clipboard read. Coordinates assume the Wine virtual desktop window renders
# at 1920x1080 on screen 0 (see docs/oracle-wine.md for the discovery steps).

set -euo pipefail

REPO="$(cd "$(dirname "$0")/../.." && pwd)"
EXE="$REPO/Genmsi_20230711.backup/genmsi.exe"   # always the backup copy
WINEPREFIX="$REPO/scripts/wine-oracle/wineprefix"
OUT="${2:-$REPO/scripts/wine-oracle/out}"
export WINEPREFIX
export DISPLAY="${DISPLAY:-:0}"

MODEL="$(realpath "$1")"
NAME="$(basename "$MODEL")"
NAME="${NAME%.*}"
ZPATH="Z:$(echo "$MODEL" | sed 's|/|\\\\|g')"
mkdir -p "$OUT"

# --- helpers ---------------------------------------------------------------

winedesk_window() {
  xdotool search --name "genmsi - Wine Desktop" | head -1
}

wait_window() {
  local wid=""
  for _ in $(seq 1 30); do
    wid="$(winedesk_window || true)"
    [ -n "$wid" ] && { echo "$wid"; return 0; }
    sleep 1
  done
  echo "error: Wine desktop window not found" >&2
  return 1
}

# Every input action is preceded by a KWin-script activation + verification, so
# keys and clicks can never land on another application. Plain X11 activation
# is unreliable here: KWin focus-stealing prevention blocks xdotool when a
# Wayland-native window is active, and with the Wine desktop focused, X11's
# _NET_ACTIVE_WINDOW reports Wine's 1x1 IME helper instead of the desktop, so
# "is the Wine window active?" is only answerable by KWin itself.
ACTIVATE_JS="$REPO/scripts/wine-oracle/activate_genmsi.js"

activate() {
  gdbus call --session --dest org.kde.KWin --object-path /Scripting \
    --method org.kde.kwin.Scripting.unloadScript activate_genmsi >/dev/null 2>&1 || true
  timeout 10 gdbus call --session --dest org.kde.KWin --object-path /Scripting \
    --method org.kde.kwin.Scripting.loadScript "$ACTIVATE_JS" activate_genmsi >/dev/null
  timeout 10 gdbus call --session --dest org.kde.KWin --object-path /Scripting \
    --method org.kde.kwin.Scripting.start >/dev/null
  sleep 0.6
  local flag
  flag="$(timeout 10 journalctl --user -n 8 --no-pager 2>/dev/null | grep -o 'GENMSI-ACTIVE:[a-z]*' | tail -1)"
  if [ "$flag" != "GENMSI-ACTIVE:yes" ]; then
    echo "error: could not activate the Wine desktop window (KWin flag: '$flag')" >&2
    return 1
  fi
}

click_at() { activate && xdotool mousemove "$1" "$2" click 1 && sleep "${3:-0.6}"; }

# Select the whole text of the simulator output pane and copy it via the X11
# clipboard (Delphi rich edit: Ctrl+A unsupported, use Ctrl+Home + Ctrl+Shift+End).
extract_pane() {
  local file="$1"
  click_at 950 630
  xdotool key --clearmodifiers ctrl+Home;  sleep 0.3
  xdotool key --clearmodifiers ctrl+shift+End; sleep 0.3
  xdotool key --clearmodifiers ctrl+c;     sleep 0.6
  python3 "$REPO/scripts/wine-oracle/read_clipboard.py" > "$file" 2>/dev/null
  if [ ! -s "$file" ]; then
    echo "error: clipboard copy produced an empty $file" >&2
    return 1
  fi
  echo "extracted $(wc -l < "$file") lines -> $file"
}

# --- run -------------------------------------------------------------------

[ -f "$EXE" ] || { echo "error: $EXE not found (backup the original first)" >&2; exit 1; }

nohup wine explorer /desktop=genmsi,1280x800 "$EXE" "$ZPATH" \
  > "$OUT/$(echo "$NAME" | tr ' ' '_').wine.log" 2>&1 &
WID="$(wait_window)"
echo "wine desktop window: $WID"
sleep 5   # give the Delphi app time to finish loading the model
activate   # first activation also verifies the KWin scripting route works

# Modelo menu -> Simular (F9)
click_at 624 311   # "Modelo"
click_at 650 357   # "Simular"
sleep 1
# Simulator toolbar: green circle = refresh, blue globe = run (Start defaults to 1)
click_at 826 452 1.5
sleep 2   # small models finish in well under a second

# Simulator output tabs: Report | GPSS Code
extract_pane "$OUT/$(echo "$NAME" | tr ' ' '_').report.txt"
click_at 721 476   # "Código GPSS" tab
extract_pane "$OUT/$(echo "$NAME" | tr ' ' '_').gpss.txt"

# Stop the Wine session cleanly
wineserver -k 2>/dev/null || true
echo "done: $OUT/$(echo "$NAME" | tr ' ' '_').{report,gpss}.txt"
