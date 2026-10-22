#!/usr/bin/env bash
# Generate the TypeScript types of the engine's documents.
#
# The types come from the Rust types (S7): `ts-rs` writes one file per
# published type into `crates/genmsi-app/src/types/generated/`. The frontend
# imports them; it never writes a document type by hand.
#
# Run it after a change in the engine's documents:
#
#     crates/genmsi-app/scripts/generate-types.sh
#
# The `ts` feature of `genmsi-core` is optional, so this generator does not
# change the default build of the workspace.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
out="$root/crates/genmsi-app/src/types/generated"

cd "$root"
rm -rf "$out"
mkdir -p "$out"
TS_RS_EXPORT_DIR="$out" cargo test -p genmsi-core --features ts --lib export_bindings

echo "generated:"
ls -1 "$out"
