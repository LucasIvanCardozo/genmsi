#!/usr/bin/env bash
#
# Refresca el PKGBUILD de la AUR con la suma real del tarball de la
# etiqueta.
#
# La AUR no acepta `SKIP` en `sha256sums`: el paquete se construye en la
# máquina de la AUR, que baja la fuente ella misma y comprueba la suma.
# Esa suma es la del tarball que GitHub arma para la etiqueta, y sólo se
# puede calcular una vez que la etiqueta existe.
#
#   packaging/arch/aur-pkgbuild.sh              # la versión del PKGBUILD
#   packaging/arch/aur-pkgbuild.sh 0.3.2       # la de una versión
#
# El script deja el PKGBUILD con la suma puesta y dice cuál es, para que
# se pueda revisar antes de subirlo a la AUR.
set -euo pipefail

repo="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
pkgbuild="$repo/packaging/arch/PKGBUILD"
owner="LucasIvanCardozo/genmsi"

version="${1:-}"
if [[ -z "$version" ]]; then
  version="$(grep -m1 '^pkgver=' "$pkgbuild" | cut -d= -f2)"
fi

die() { echo "aur-pkgbuild: $*" >&2; exit 1; }

[[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] ||
  die "esto no parece una versión (se pide X.Y.Z): $version"

tag="v$version"
url="https://github.com/$owner/archive/refs/tags/$tag.tar.gz"

# El tarball, en un directorio de trabajo propio (fuera del repositorio).
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
echo "==> el tarball de $tag"
curl -fsSL -o "$work/$tag.tar.gz" "$url" || die "no se pudo bajar $url"
sum="$(sha256sum "$work/$tag.tar.gz" | cut -d' ' -f1)"

# La suma va en el PKGBUILD, y la versión también (por si se pasó otra).
sed -e "s|^pkgver=.*|pkgver=$version|" \
    -e "s|^sha256sums=.*|sha256sums=('$sum')|" \
    "$pkgbuild" >"$work/PKGBUILD"
mv "$work/PKGBUILD" "$pkgbuild"

echo "==> la versión es $version y su suma es"
echo "    $sum"
echo "==> el PKGBUILD quedó con la suma puesta; falta subirlo a la AUR:"
echo "    (en el repositorio de la AUR: aur.archlinux.org/$owner.git no;"
echo "     el de este paquete es aur.archlinux.org/genmsi.git)"
