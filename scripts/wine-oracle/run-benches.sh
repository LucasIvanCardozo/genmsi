#!/usr/bin/env bash
# Run the original over every bench and keep its GPSS and its report.
#
# A bench is one model per class of the original (see `make-benches.py`).  For
# each of them the wine oracle harness opens the model in the original
# editor, runs the simulation and extracts the "Reporte" and the "Código
# GPSS" tabs.  The two files land next to each other, named after the bench:
#
#   <results>/Advance.report.txt    what the class does
#   <results>/Advance.gpss.txt       how the class compiles
#
# The run does not stop at the first bench that fails: every bench is tried
# and the summary at the end says how many ran and how many did not.
#
# Usage: scripts/wine-oracle/run-benches.sh [benches-dir] [results-dir]

set -uo pipefail

REPO="$(cd "$(dirname "$0")/../.." && pwd)"
ORACLE="$REPO/scripts/wine-oracle/run_oracle.sh"
WINEPREFIX="$REPO/scripts/wine-oracle/wineprefix"
export WINEPREFIX
BENCHES="${1:-$REPO/scripts/wine-oracle/benches}"
RESULTS="${2:-/tmp/genmsi-bench-out}"

BENCHES="$(cd "$BENCHES" 2>/dev/null && pwd || echo "$BENCHES")"
mkdir -p "$RESULTS"
RESULTS="$(cd "$RESULTS" && pwd)"

if [ ! -d "$BENCHES" ]; then
  echo "error: benches directory $BENCHES not found" >&2
  echo "hint: run 'python3 scripts/wine-oracle/make-benches.py' first" >&2
  exit 1
fi

if ! command -v wine >/dev/null 2>&1; then
  echo "error: wine not found; the original cannot be run here" >&2
  exit 1
fi

total=0
ran=0
failed=0
failed_names=""

for bench in "$BENCHES"/*.gmdl; do
  [ -e "$bench" ] || continue
  name="$(basename "$bench" .gmdl)"
  total=$((total + 1))
  printf '%-18s ' "$name"
  if bash "$ORACLE" "$bench" "$RESULTS" > "$RESULTS/$name.run.log" 2>&1; then
    if [ -s "$RESULTS/$name.gpss.txt" ] && [ -s "$RESULTS/$name.report.txt" ]; then
      ran=$((ran + 1))
      echo "ok"
      wineserver -k 2>/dev/null || true
      continue
    fi
  fi
  failed=$((failed + 1))
  failed_names="$failed_names $name"
  echo "failed (see $RESULTS/$name.run.log)"
  # The harness leaves the original alive when it fails; its window would be
  # found by the next bench, which would then drive the wrong window.
  wineserver -k 2>/dev/null || true
done

echo
echo "benches: $total   ran: $ran   did not run: $failed"
if [ "$failed" -gt 0 ]; then
  echo "did not run:$failed_names"
fi
[ "$failed" -eq 0 ]
