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
# The `ts` feature of `genmsi-core` and of `genmsi-format` is optional, so
# this generator does not change the default build of the workspace.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
out="$root/crates/genmsi-app/src/types/generated"

cd "$root"
# The types are generated into a scratch directory first: a generation that
# fails must not leave the app without the types it already had.
scratch="$root/target/ts-types"
rm -rf "$scratch"
mkdir -p "$scratch"
TS_RS_EXPORT_DIR="$scratch" cargo test -p genmsi-core -p genmsi-format \
  --features genmsi-core/ts,genmsi-format/ts --lib export_bindings

rm -rf "$out"
mkdir -p "$out"
cp "$scratch"/*.ts "$out"/
rm -rf "$scratch"

echo "generated:"
ls -1 "$out"
