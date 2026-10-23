#!/usr/bin/env bash
# Compare our GPSS with the GPSS the original editor produced for the same
# model. The two files are compared statement by statement: the name, the verb
# and the operands. Only the differences that mean something are reported; the
# padding between the columns is not.
#
#   compare-gpss.sh <our gpss> <the original's gpss>
#
# Exit 0 when the two agree, 1 when they differ, 2 when a file is missing.

set -euo pipefail

if [ "$#" -ne 2 ]; then
    echo "uso: compare-gpss.sh <nuestro gpss> <gpss del original>" >&2
    exit 2
fi

OURS="$1"
THEIRS="$2"

for file in "$OURS" "$THEIRS"; do
    if [ ! -f "$file" ]; then
        echo "no existe el archivo: $file" >&2
        exit 2
    fi
done

# A statement is the name, the verb and the operands. The name may be empty:
# the statements that continue a block carry no name of their own.
normalize() {
    awk '
        {
            sub(/\r$/, "")
            sub(/[ \t]+$/, "")
            if ($0 == "") { next }
            name = $1
            verb = $2
            rest = ""
            for (i = 3; i <= NF; i++) {
                rest = rest (rest == "" ? "" : " ") $i
            }
            printf "%s|%s|%s\n", name, verb, rest
        }
    ' "$1"
}

normalize "$OURS" > "${TMPDIR:-/tmp}/gpss-ours.$$"
normalize "$THEIRS" > "${TMPDIR:-/tmp}/gpss-theirs.$$"
trap 'rm -f "${TMPDIR:-/tmp}/gpss-ours.$$" "${TMPDIR:-/tmp}/gpss-theirs.$$"' EXIT

differences=$(diff --unchanged-line-format= --old-line-format='n ours: %L' --new-line-format='n original: %L' \
    "${TMPDIR:-/tmp}/gpss-ours.$$" "${TMPDIR:-/tmp}/gpss-theirs.$$" || true)

if [ -z "$differences" ]; then
    statements=$(wc -l < "${TMPDIR:-/tmp}/gpss-ours.$$")
    echo "el GPSS coincide: $statements sentencias iguales"
    exit 0
fi

count=$(printf '%s\n' "$differences" | grep -c . || true)
echo "el GPSS difiere en $count sentencias:"
printf '%s\n' "$differences" | sed 's/^/  /'
exit 1
