#!/usr/bin/env bash
# Probe: which order does the original's engine follow for a `Transfer`?
#
# The original writes the destinations of a `TRANSFER` in the reverse of the
# port order, but the *engine* can follow either the port order (the
# probability goes to `Salida1`, as the property hint says) or the order it
# writes (the probability goes to the first destination it writes).
#
# The probe settles it with the numbers: a `Transfer` with `Prob` = 85% whose
# `Salida1` leads to a long `Advance` (1000) and whose `Salida2` leads to a
# short one (10).  The report of each run says how many transactions entered
# each advance.
#
# Usage: scripts/wine-oracle/probes/run-probe.sh [model ...]
#
# Needs wine, Xvfb and xdotool (the same things `run_oracle.sh` needs).

set -uo pipefail

REPO="$(cd "$(dirname "$0")/../../.." && pwd)"
PROBES="$REPO/scripts/wine-oracle/probes"
ORACLE="$REPO/scripts/wine-oracle/run_oracle.sh"
WINEPREFIX="$REPO/scripts/wine-oracle/wineprefix"
export WINEPREFIX

MODELS=("$@")
[ ${#MODELS[@]} -eq 0 ] && MODELS=(TransferProbOrder85 TransferProbOrder15 TransferProbOrder)

mkdir -p "$PROBES/oracle" "$PROBES/ours"

# The entry count of the advance a report line names.
count_of() { # count_of <report file> <LOC>
  awk -v loc="$2" '$0 ~ loc && $0 ~ /ADVANCE/ { print $2; exit }' "$1"
}

for name in "${MODELS[@]}"; do
  model="$PROBES/$name.gmdl"
  [ -f "$model" ] || { echo "error: $model not found" >&2; exit 1; }

  echo "=== $name"
  if command -v wine >/dev/null 2>&1; then
    # The harness leaves the original alive when it fails, and its window
    # would then be found by the next model, which would drive the wrong one.
    bash "$ORACLE" "$model" "$PROBES/oracle" > "$PROBES/oracle/$name.run.log" 2>&1 \
      || echo "  (the oracle failed, see $PROBES/oracle/$name.run.log)"
    wineserver -k 2>/dev/null || true
    sleep 2
  else
    echo "  (wine not found: the original cannot be run here)"
  fi

  cargo run -q -p genmsi-cli -- simulate "$model" > "$PROBES/ours/$name.report.txt" 2>&1
  cargo run -q -p genmsi-cli -- gpss "$model" > "$PROBES/ours/$name.gpss.txt" 2>&1

  echo "  original   : largo=$(count_of "$PROBES/oracle/$name.report.txt" A_AdvanceLargo)" \
       "corto=$(count_of "$PROBES/oracle/$name.report.txt" A_AdvanceCorto)"
  echo "  ours       : largo=$(count_of "$PROBES/ours/$name.report.txt" A_AdvanceLargo)" \
       "corto=$(count_of "$PROBES/ours/$name.report.txt" A_AdvanceCorto)"
  echo "  original GPSS: $(grep -i TRANSFER "$PROBES/oracle/$name.gpss.txt" | tr -s ' ')"
  echo "  ours GPSS    : $(grep -i TRANSFER "$PROBES/ours/$name.gpss.txt" | tr -s ' ')"
done
