#!/usr/bin/env bash
#
# Run the app in the browser, without installing it.
#
# The desktop app is installed with `scripts/install-linux.sh`, but the
# frontend can also run in a browser against the real engine: the dev bridge
# serves the engine's documents and answers the session's commands. It is the
# same frontend the desktop app shows, and it is what the visual loop
# reviews, so a change can be seen without building and installing the
# desktop shell.
#
#   scripts/dev-app.sh
#
# The browser has no system dialog, so a model is opened with "Abrir…" (the
# browser asks for the path in a prompt) or from the "Recientes" menu.
#
set -euo pipefail

repo="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# The dev bridge spawns the engine's binaries: they must be there.
echo "==> compilando el motor"
(cd "${repo}" && cargo build -p genmsi-cli -p genmsi-app-engine)

cd "${repo}/crates/genmsi-app"
echo "==> la app en http://127.0.0.1:5173/ (Ctrl+C para salir)"
exec bun run dev
