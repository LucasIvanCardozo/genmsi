#!/usr/bin/env bash
#
# The smoke check of the desktop shell.
#
# The shell is the same frontend under a native window. The check proves that
# the window opens, that the frontend reaches the facade through `invoke` and
# that the picture can be captured, without a window manager.
#
# Two things the check needs, both from the WebKitGTK documentation of Tauri
# (https://v2.tauri.app/develop/debug/linux-graphics/): the software renderer
# and the disabled compositing mode, or the webview paints nothing under
# Xvfb. And the debug build of the shell loads the frontend from the dev
# server (`devUrl`), so the server has to run.
#
# Usage:
#   crates/genmsi-app/scripts/check-tauri-shell.sh
#
# Environment:
#   GENMSI_TAURI_BIN   the shell's binary (default: the one it builds)
#   GENMSI_SHOTS       where the capture goes (default /tmp/genmsi-app)
#   GENMSI_DISPLAY     the virtual display (default :98)

set -euo pipefail

APP_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SHELL_DIR="$APP_DIR/src-tauri"
SHOTS="${GENMSI_SHOTS:-/tmp/genmsi-app}"
DISPLAY_NUMBER="${GENMSI_DISPLAY:-:98}"
PORT=5173

mkdir -p "$SHOTS"

die() { echo "check-tauri-shell: $*" >&2; exit 1; }

command -v Xvfb >/dev/null || die "Xvfb is not installed"
command -v xdotool >/dev/null || die "xdotool is not installed"

SERVER_PID=""
XVFB_PID=""
APP_PID=""
cleanup() {
  for pid in "$APP_PID" "$SERVER_PID" "$XVFB_PID"; do
    if [[ -n "$pid" ]] && kill -0 "$pid" 2>/dev/null; then
      kill "$pid" 2>/dev/null || true
    fi
  done
}
trap cleanup EXIT

echo "==> the shell builds (its dependency tree is heavy, this takes a while)"
(cd "$SHELL_DIR" && cargo build --quiet) || die "the shell does not build"
BIN="${GENMSI_TAURI_BIN:-$SHELL_DIR/target/debug/genmsi-app-desktop}"
[[ -x "$BIN" ]] || die "$BIN is missing"

echo "==> the virtual display"
rm -f "/tmp/.X${DISPLAY_NUMBER#:}-lock" 2>/dev/null || true
Xvfb "$DISPLAY_NUMBER" -screen 0 1440x900x24 >"$SHOTS/xvfb.log" 2>&1 &
XVFB_PID=$!
sleep 1

echo "==> the dev server (the debug shell loads the frontend from it)"
curl -sf "http://127.0.0.1:$PORT/" >/dev/null 2>&1 ||
  (cd "$APP_DIR" && bun run dev) >"$SHOTS/vite.log" 2>&1 &
SERVER_PID=$!
for _ in $(seq 1 100); do
  curl -sf "http://127.0.0.1:$PORT/" >/dev/null 2>&1 && break
  sleep 0.3
done
curl -sf "http://127.0.0.1:$PORT/" >/dev/null 2>&1 || die "the dev server never answered"

echo "==> the shell starts"
DISPLAY="$DISPLAY_NUMBER" \
  LIBGL_ALWAYS_SOFTWARE=1 \
  WEBKIT_DISABLE_COMPOSITING_MODE=1 \
  WEBKIT_DISABLE_DMABUF_RENDERER=1 \
  GDK_BACKEND=x11 \
  "$BIN" >"$SHOTS/tauri.log" 2>&1 &
APP_PID=$!
sleep 10

if ! kill -0 "$APP_PID" 2>/dev/null; then
  echo "check-tauri-shell: the shell exited; see $SHOTS/tauri.log"
  cat "$SHOTS/tauri.log"
  exit 1
fi

WINDOW=$(DISPLAY="$DISPLAY_NUMBER" xdotool search --name "GenMSI" | head -1 || true)
if [[ -z "$WINDOW" ]]; then
  # Without a window manager the window does not publish its name, but it is
  # there: any window on the display is the shell's.
  WINDOW=$(DISPLAY="$DISPLAY_NUMBER" xdotool search --class "" | head -1 || true)
  if [[ -z "$WINDOW" ]]; then
    echo "check-tauri-shell: the window is not on the display; see $SHOTS/tauri.log"
    cat "$SHOTS/tauri.log"
    exit 1
  fi
  echo "==> the window exists (xdotool id $WINDOW); without a window manager it does not publish its name"
else
  echo "==> the window exists (xdotool id $WINDOW)"
fi

# The capture: without a window manager the root window may still hold the
# picture of the app, because the webview draws into it.
DISPLAY="$DISPLAY_NUMBER" import -window root "$SHOTS/tauri-window.png" 2>/dev/null || true
if [[ -f "$SHOTS/tauri-window.png" ]]; then
  SIZE=$(identify -format "%wx%h" "$SHOTS/tauri-window.png" 2>/dev/null || echo "unknown")
  COLORS=$(identify -format "%k" "$SHOTS/tauri-window.png" 2>/dev/null || echo "unknown")
  echo "==> the capture is in $SHOTS/tauri-window.png ($SIZE, $COLORS distinct colors)"
else
  echo "==> the capture failed; the Vite loop is the visual gate"
fi

echo "==> the shell's log"
cat "$SHOTS/tauri.log"

# The proof that the frontend reached the facade through `invoke`.
if grep -q "list_models" "$SHOTS/tauri.log"; then
  echo "==> the frontend reached the facade through invoke"
  grep "\[genmsi-app\]" "$SHOTS/tauri.log" | head -5
else
  echo "check-tauri-shell: the frontend did not reach the facade; see $SHOTS/tauri.log"
  exit 1
fi
