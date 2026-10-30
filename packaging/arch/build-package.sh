#!/usr/bin/env bash
#
# Armar el paquete de Arch (CachyOS y la familia de pacman).
#
# El paquete sale del repositorio local: el script arma el tarball de la
# etiqueta (o del commit actual), le pone su suma al PKGBUILD y corre
# `makepkg`. El paquete queda en el directorio de trabajo de makepkg.
#
#   packaging/arch/build-package.sh              # del commit actual
#   packaging/arch/build-package.sh v0.3.0       # de la etiqueta
#   packaging/arch/build-package.sh --install    # y lo instala
#
# Hace falta `makepkg` (el grupo `base-devel`) y el toolchain de Rust.
set -euo pipefail

repo="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
pkgbuild="$repo/packaging/arch/PKGBUILD"
work="${PKGDEST:-$repo/target/arch-package}"
install=0
ref=""

for arg in "$@"; do
  case "$arg" in
    --install) install=1 ;;
    v*) ref="$arg" ;;
    *) echo "opción desconocida: $arg" >&2; exit 2 ;;
  esac
done

die() { echo "build-package: $*" >&2; exit 1; }
command -v makepkg >/dev/null || die "makepkg no está (el grupo base-devel)"
command -v git >/dev/null || die "git no está"

cd "$repo"
# La versión del paquete: la de la etiqueta, o la del commit actual.
version="$(grep -m1 '^pkgver=' "$pkgbuild" | cut -d= -f2)"
[[ -n "$ref" ]] && version="${ref#v}"

rm -rf "$work"
mkdir -p "$work/src"
tarball="genmsi-$version.tar.gz"
# El tarball del repositorio: la etiqueta si existe, el commit actual si no.
if [[ -n "$ref" ]] && git rev-parse "$ref" >/dev/null 2>&1; then
  git archive --format=tar --prefix="genmsi-$version/" "$ref" | gzip >"$work/$tarball"
else
  git archive --format=tar --prefix="genmsi-$version/" HEAD | gzip >"$work/$tarball"
fi
sum="$(sha256sum "$work/$tarball" | cut -d' ' -f1)"

# El PKGBUILD de esta corrida: el del repositorio con la fuente y la suma
# de este tarball.
sed -e "s|^source=.*|source=(\"$tarball\")|" \
    -e "s|^sha256sums=.*|sha256sums=('$sum')|" \
    "$pkgbuild" >"$work/PKGBUILD"

echo "==> el paquete: genmsi $version (suma $sum)"
(cd "$work" && makepkg --nodeps)

package="$(ls -t "$work"/genmsi-*.pkg.tar.zst 2>/dev/null | head -1 || true)"
[[ -n "$package" ]] || die "makepkg no dejó ningún paquete"
echo "==> el paquete: $package"

if [[ "$install" -eq 1 ]]; then
  sudo pacman -U "$package"
  echo "==> instalado"
fi
