#!/usr/bin/env bash
# Compare our GPSS with the original's for every bench.
#
# A bench is a model that exercises one class of the original and nothing
# else; the original's own answer for it (its GPSS and its report) is what
# `run-benches.sh` leaves in the results directory. This script asks the
# original's question of our engine, class by class, and says which classes
# agree and which do not.
#
#   compare-benches.sh <benches dir> <results dir>
#
# Exit 0 when every bench that has an original answer agrees, 1 when one does
# not, 2 when a directory is missing.

set -euo pipefail

if [ "$#" -ne 2 ]; then
    echo "uso: compare-benches.sh <directorio de bancos> <directorio de resultados>" >&2
    exit 2
fi

BENCHES="$1"
RESULTS="$2"

for dir in "$BENCHES" "$RESULTS"; do
    if [ ! -d "$dir" ]; then
        echo "no existe el directorio: $dir" >&2
        exit 2
    fi
done

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"

WORK="${TMPDIR:-/tmp}/compare-benches.$$"
mkdir -p "$WORK"
trap 'rm -rf "$WORK"' EXIT

# The engine, built once: the comparison asks it for every bench.
BIN="$ROOT/target/release/genmsi"
echo "compilando el motor..." >&2
cargo build --quiet --release --manifest-path "$ROOT/Cargo.toml" -p genmsi-cli >&2

total=0
matched=0
declare -a differing=()
declare -a missing=()

for bench in "$BENCHES"/*.gmdl; do
    [ -e "$bench" ] || continue
    name="$(basename "$bench" .gmdl)"
    original="$RESULTS/$name.gpss.txt"
    if [ ! -f "$original" ]; then
        missing+=("$name")
        continue
    fi
    total=$((total + 1))
    "$BIN" gpss "$bench" > "$WORK/ours.gpss" 2> "$WORK/ours.err" || true
    if bash "$HERE/compare-gpss.sh" "$WORK/ours.gpss" "$original" > "$WORK/report" 2>&1; then
        matched=$((matched + 1))
    else
        differing+=("$name")
        echo "=== $name"
        sed 's/^/  /' "$WORK/report"
    fi
done

echo
echo "bancos con respuesta del original: $total"
echo "coinciden: $matched"
echo "difieren: ${#differing[@]}"
if [ "${#differing[@]}" -gt 0 ]; then
    printf '  %s\n' "${differing[@]}"
fi
if [ "${#missing[@]}" -gt 0 ]; then
    echo "sin respuesta del original: ${#missing[@]}"
    printf '  %s\n' "${missing[@]}"
fi

if [ "${#differing[@]}" -gt 0 ]; then
    exit 1
fi
exit 0
