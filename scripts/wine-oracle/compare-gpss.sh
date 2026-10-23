#!/usr/bin/env bash
# Compare our GPSS with the GPSS the original editor produced for the same
# model.
#
# The two files are compared group by group: a group is a block, with the
# statement that names it and the statements that continue it. The groups are
# matched by their label, so a block that moved in the model does not look
# like a difference of behaviour, and the report says which block differs and
# how.
#
# The definitions — the storages and the queue tables — are compared as sets:
# their order is not part of what the model means.
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

WORK="${TMPDIR:-/tmp}/gpss-compare.$$"
mkdir -p "$WORK"
trap 'rm -rf "$WORK"' EXIT

# Split a GPSS file into its definitions and its blocks. A definition is a
# line whose verb is STORAGE or QTABLE; a block starts at a line with a name
# and continues with the lines that have none.
split() {
    awk '
        function flush() {
            if (label != "" || count > 0) {
                if (label == "") { label = "(sin nombre)" }
                printf "B\t%s\t%s\n", label, body
            }
            body = ""
        }
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
            if (verb == "STORAGE" || verb == "QTABLE") {
                flush()
                printf "D\t%s %s\n", verb, rest
                next
            }
            # The units of an ENTER or a LEAVE are the same whether the
            # original writes them or not, so they do not make a difference.
            if (verb == "ENTER" || verb == "LEAVE") {
                sub(/,1$/, "", rest)
            }
            if (name != "") {
                flush()
                label = name
            }
            if (label == "") { label = "(sin nombre)" }
            body = body (body == "" ? "" : " ; ") verb " " rest
        }
        END { flush() }
    ' "$1"
}

split "$OURS" > "$WORK/ours"
split "$THEIRS" > "$WORK/theirs"

# The definitions, sorted: their order is not part of the model.
grep '^D' "$WORK/ours" | sort > "$WORK/ours.def"
grep '^D' "$WORK/theirs" | sort > "$WORK/theirs.def"

# The blocks, keyed by their label.
awk -F'\t' '$1 == "B" { print $2 "\t" $3 }' "$WORK/ours" | sort > "$WORK/ours.blocks"
awk -F'\t' '$1 == "B" { print $2 "\t" $3 }' "$WORK/theirs" | sort > "$WORK/theirs.blocks"

problems=0

definitions=$(diff "$WORK/ours.def" "$WORK/theirs.def" || true)
if [ -n "$definitions" ]; then
    problems=$((problems + 1))
    echo "las definiciones difieren:"
    printf '%s\n' "$definitions" | sed 's/^/  /'
fi

blocks=$(diff "$WORK/ours.blocks" "$WORK/theirs.blocks" || true)
if [ -n "$blocks" ]; then
    problems=$((problems + 1))
    echo "los bloques difieren:"
    printf '%s\n' "$blocks" | sed 's/^/  /'
fi

if [ "$problems" -eq 0 ]; then
    count=$(wc -l < "$WORK/ours.blocks")
    echo "el GPSS coincide: $count bloques con las mismas sentencias"
    exit 0
fi

echo "el GPSS difiere"
exit 1
