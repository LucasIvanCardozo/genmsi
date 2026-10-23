#!/usr/bin/env bash
# Print a readable list of the forms that the original GenMSI executable stores.
#
# Usage: scripts/wine-oracle/dfm-summary.sh <executable>
#
# For every form it prints its class, its name and its caption, one per line.
# The executable is only read: it is never run, never modified, and nothing is
# written inside the repository.

set -euo pipefail

REPO="$(cd "$(dirname "$0")/../.." && pwd)"

if [ $# -ne 1 ]; then
  echo "uso: scripts/wine-oracle/dfm-summary.sh <ejecutable>" >&2
  exit 2
fi

EXE="$1"

if [ ! -f "$EXE" ]; then
  echo "error: $EXE: file not found" >&2
  exit 1
fi

exec python3 "$REPO/scripts/wine-oracle/dfm.py" --summary "$EXE"
