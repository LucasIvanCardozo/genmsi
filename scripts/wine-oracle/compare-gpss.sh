#!/usr/bin/env bash
# Compare our GPSS with the GPSS the original editor produced for the same
# model.
#
# The comparison has two parts, because the program has two things to say:
#
# - What each block does: the verb and the operands of every statement,
#   compared block by block, matched by the block's label.
# - Where the transaction goes: the connections, compared as a set of pairs
#   (block, destination). The connections are compared apart from the blocks
#   because the order of the blocks is a matter of the walk, not of the model:
#   the same connection is a fall-through in one program and a `TRANSFER` in
#   the other, and that is not a difference of behaviour.
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

# Split a GPSS file into three kinds of line:
#
#   D <definition>            a storage or a queue table
#   B <label> <statements>    a block, with the statements that are not a
#                             connection
#   C <source> <destination>  a connection, with the block it leaves from
#
# A block starts at a statement with a name; the statements that continue it
# have none. A `TRANSFER` is never part of a block's statements: it says where
# the transaction goes, and that is a connection.
split() {
    awk '
        function flush() {
            if (count > 0) {
                printf "B\t%s\t%s\n", cur, body
            }
            body = ""
            count = 0
        }
        function connection(source, operands, first, rest, dest) {
            # The first operand is the probability; the rest are the
            # destinations, and an empty one is a port with no link.
            first = 1
            rest = operands
            while (rest != "") {
                if (index(rest, ",") > 0) {
                    dest = substr(rest, 1, index(rest, ",") - 1)
                    rest = substr(rest, index(rest, ",") + 1)
                } else {
                    dest = rest
                    rest = ""
                }
                if (first) {
                    first = 0
                } else if (dest != "") {
                    printf "C\t%s\t%s\n", source, dest
                }
            }
        }
        {
            sub(/\r$/, "")
            sub(/[ \t]+$/, "")
            if ($0 == "") { next }
            # The name of a block is in the first column; a statement that
            # continues a block leaves that column empty, so the verb is the
            # first thing on the line.
            if ($0 ~ /^[^ ]/) {
                name = $1
                verb = $2
                first_operand = 3
            } else {
                name = ""
                verb = $1
                first_operand = 2
            }
            text = ""
            for (i = first_operand; i <= NF; i++) {
                text = text (text == "" ? "" : " ") $i
            }
            if (verb == "STORAGE" || verb == "QTABLE") {
                flush()
                printf "D\t%s %s\n", verb, text
                next
            }
            # The units of an ENTER or a LEAVE are the same whether the
            # original writes them or not, so they do not make a difference.
            if (verb == "ENTER" || verb == "LEAVE") {
                sub(/,1$/, "", text)
            }
            if (verb == "TRANSFER") {
                # A TRANSFER says where the transaction goes: it is a
                # connection, not something a block does.
                if (name != "") {
                    flush()
                    cur = name
                }
                connection(cur, text)
                next
            }
            if (name != "") {
                flush()
                cur = name
            } else if (count == 0) {
                cur = "(" verb " " text ")"
            }
            body = body (body == "" ? "" : " ; ") verb " " text
            count++
        }
        END { flush() }
    ' "$1"
}

split "$OURS" > "$WORK/ours"
split "$THEIRS" > "$WORK/theirs"

problems=0

# The definitions, sorted: their order is not part of the model.
grep '^D' "$WORK/ours" | sort > "$WORK/ours.def" || true
grep '^D' "$WORK/theirs" | sort > "$WORK/theirs.def" || true

# The blocks, keyed by their label.
awk -F'\t' '$1 == "B" { print $2 "\t" $3 }' "$WORK/ours" | sort > "$WORK/ours.blocks"
awk -F'\t' '$1 == "B" { print $2 "\t" $3 }' "$WORK/theirs" | sort > "$WORK/theirs.blocks"

# The connections, as a set of pairs.
awk -F'\t' '$1 == "C" { print $2 " -> " $3 }' "$WORK/ours" | sort > "$WORK/ours.conn"
awk -F'\t' '$1 == "C" { print $2 " -> " $3 }' "$WORK/theirs" | sort > "$WORK/theirs.conn"

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

connections=$(diff "$WORK/ours.conn" "$WORK/theirs.conn" || true)
if [ -n "$connections" ]; then
    problems=$((problems + 1))
    echo "las conexiones difieren:"
    printf '%s\n' "$connections" | sed 's/^/  /'
fi

if [ "$problems" -eq 0 ]; then
    count=$(wc -l < "$WORK/ours.blocks")
    connections=$(wc -l < "$WORK/ours.conn")
    echo "el GPSS coincide: $count bloques y $connections conexiones"
    exit 0
fi

echo "el GPSS difiere"
exit 1
