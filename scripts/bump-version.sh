#!/usr/bin/env bash
#
# Write the version of the app in the six places that follow the source.
#
# The source is the `version` of `[workspace.package]` in the root
# `Cargo.toml`: it is the only place a person changes. This script writes
# the other six and then checks that every one of them holds the same
# version; when one of them does not, the script fails.
#
#   scripts/bump-version.sh 0.4.0
#
# The order of a release bump is the one of the source: the person changes
# `Cargo.toml` by hand, and then runs this script with that same version.
# The script never touches the source: when the source still says another
# version, the script says so and fails, because a release that carries two
# versions is a release nobody can install.
#
# The script is idempotent: running it twice with the same version does not
# change the tree.
#
# The `tauri.conf.json` is one of the six, and its copy is not a version:
# Tauri takes the version of the shell's `Cargo.toml`, so the script keeps
# the field out of the config.
set -euo pipefail

repo="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

die() {
  echo "bump-version: $*" >&2
  exit 1
}

usage() {
  cat >&2 <<'EOF'
uso: scripts/bump-version.sh <versión>

Escribe la versión en las seis copias que siguen la fuente y falla si
alguna queda con otra versión. La fuente —la `version` de
`[workspace.package]` en el Cargo.toml de la raíz— no la cambia el script:
la cambia la persona, a mano.
EOF
}

[[ $# -eq 1 ]] || {
  usage
  exit 2
}
case "$1" in
  -h | --help)
    usage
    exit 0
    ;;
esac

version="$1"
[[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+([-+][0-9A-Za-z.-]+)?$ ]] ||
  die "esto no parece una versión: $version"

cargo_root="$repo/Cargo.toml"
engine_cargo="$repo/crates/genmsi-app/engine/Cargo.toml"
manifest="$repo/crates/genmsi-app/package.json"
shell_cargo="$repo/crates/genmsi-app/src-tauri/Cargo.toml"
tauri_config="$repo/crates/genmsi-app/src-tauri/tauri.conf.json"
pkgbuild="$repo/packaging/arch/PKGBUILD"
readme="$repo/crates/genmsi-app/LEEME.txt"

for file in "$cargo_root" "$engine_cargo" "$manifest" "$shell_cargo" \
  "$tauri_config" "$pkgbuild" "$readme"; do
  [[ -f "$file" ]] || die "falta el archivo: ${file#"$repo"/}"
done

# The version of the source: the one place a person changes.
source_version() {
  sed -n '/^\[workspace\.package\]/,/^\[[^]]*\]/p' "$cargo_root" |
    grep -m1 -E '^[[:space:]]*version[[:space:]]*=' |
    sed -E 's/^[[:space:]]*version[[:space:]]*=[[:space:]]*"([^"]+)".*/\1/'
}

# The source is the last word, and it is checked BEFORE anything is
# written: when it says another version, the tree is out of date and the
# script says so without having touched a single file. A release that
# carries two versions is a release nobody can install.
current="$(source_version || true)"
[[ -n "$current" ]] || die "el Cargo.toml de la raíz no tiene versión"
if [[ "$current" != "$version" ]]; then
  die "el Cargo.toml de la raíz dice $current y pediste $version: la versión se cambia a mano en el Cargo.toml de la raíz"
fi

# The Cargo manifests: the version of their own package.
sed -i -E "s/^version[[:space:]]*=.*/version = \"$version\"/" \
  "$engine_cargo" "$shell_cargo"

# The manifest of the app.
sed -i -E "s/^([[:space:]]*\"version\"[[:space:]]*:[[:space:]]*)\"[^\"]*\"/\1\"$version\"/" \
  "$manifest"

# The config of Tauri: the version is not needed there. Tauri takes the one
# of the shell's `Cargo.toml`.
sed -i -E '/^[[:space:]]*"version"[[:space:]]*:/d' "$tauri_config"

# The PKGBUILD: one line, and only one.
sed -i -E "s/^pkgver=.*/pkgver=$version/" "$pkgbuild"

# The readme of the installer: the title and the names of the files carry
# the version.
sed -i -E "s/[0-9]+\.[0-9]+\.[0-9]+/$version/g" "$readme"

# The copies are checked one by one: a copy that does not hold the version
# is a failure, not a warning.
grep -qE "^version[[:space:]]*=[[:space:]]*\"$version\"" "$engine_cargo" ||
  die "el Cargo.toml del motor no quedó con la versión $version"
grep -qE "^version[[:space:]]*=[[:space:]]*\"$version\"" "$shell_cargo" ||
  die "el Cargo.toml del shell no quedó con la versión $version"
grep -qE "^[[:space:]]*\"version\"[[:space:]]*:[[:space:]]*\"$version\"" "$manifest" ||
  die "el package.json no quedó con la versión $version"
if grep -qE '"version"[[:space:]]*:' "$tauri_config"; then
  die "el tauri.conf.json volvió a llevar su propia versión"
fi
pkgver_lines="$(grep -cE '^pkgver=' "$pkgbuild" || true)"
[[ "$pkgver_lines" == "1" ]] ||
  die "el PKGBUILD tiene $pkgver_lines líneas pkgver y tiene que tener una"
grep -qE "^pkgver=$version\$" "$pkgbuild" ||
  die "el PKGBUILD no quedó con la versión $version"
head -n 1 "$readme" | grep -qF "GenMSI $version" ||
  die "el LEEME.txt no tiene la versión $version en el título"
others="$(grep -oE '[0-9]+\.[0-9]+\.[0-9]+' "$readme" | grep -vxF "$version" || true)"
[[ -z "$others" ]] ||
  die "el LEEME.txt lleva otra versión: $others"

echo "bump-version: las siete copias quedan en $version"
