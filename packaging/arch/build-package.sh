#!/usr/bin/env bash
#
# Armar el paquete de Arch (CachyOS y la familia de pacman).
#
# El paquete sale del repositorio local: el script arma el tarball de la
# etiqueta (o del commit actual), le pone su suma al PKGBUILD y corre
# `makepkg`. El paquete queda en el directorio de trabajo de makepkg.
#
# El paquete se arma con el código que se está empaquetando: la versión del
# PKGBUILD relleno es la de ese código (la de la etiqueta, o la del
# workspace en `Cargo.toml`), no la que el PKGBUILD del repositorio tiene
# escrita.
#
# Escribe tres cosas, todas en el directorio de trabajo del paquete
# (`target/arch-package` por defecto, `$PKGDEST` cuando está puesto):
#
#   target/arch-package/PKGBUILD         el PKGBUILD relleno: la suma sha256
#                                        real del tarball de la fuente y la
#                                        versión del código empaquetado. La
#                                        AUR no acepta SKIP, así que este es
#                                        el archivo que hay que publicar.
#   target/arch-package/.SRCINFO       el .SRCINFO que sirve la AUR, hecho
#                                        a partir de ese PKGBUILD. El nombre es
#                                        el que la AUR espera.
#   target/arch-package/*.pkg.tar.zst    el paquete en sí.
#
# La CI publica los dos archivos junto al paquete, como assets del release.
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

# La versión del workspace: la fuente de la versión (Cargo.toml,
# [workspace.package] version).
workspace_version() {
  awk '
    /^\[workspace\.package\]/ { section = 1; next }
    /^\[/ { section = 0 }
    section && /^[[:space:]]*version[[:space:]]*=/ {
      gsub(/[[:space:]"]/, "", $0)
      sub(/^version=/, "", $0)
      print
      exit
    }
  ' "$repo/Cargo.toml"
}

cd "$repo"
# La versión del paquete: la de la etiqueta cuando hay etiqueta, la del
# código que se empaqueta (el workspace) cuando no.
version=""
archive="HEAD"
if [[ -n "$ref" ]] && git rev-parse "$ref" >/dev/null 2>&1; then
  version="${ref#v}"
  archive="$ref"
else
  version="$(workspace_version)"
  [[ -n "$version" ]] || die "la versión no está en Cargo.toml ([workspace.package] version)"
fi

rm -rf "$work"
mkdir -p "$work/src"
tarball="genmsi-$version.tar.gz"
# El tarball del repositorio: la etiqueta si existe, el commit actual si no.
git archive --format=tar --prefix="genmsi-$version/" "$archive" | gzip >"$work/$tarball"
sum="$(sha256sum "$work/$tarball" | cut -d' ' -f1)"

# La fuente del repositorio, con la versión de esta corrida: la URL es la de
# la etiqueta (la AUR baja esa) y el nombre es el archivo que está en el
# directorio de trabajo (el que arma esta corrida).
source_line="$(grep -m1 '^source=' "$pkgbuild")"
source_line="${source_line//\$pkgver/$version}"

# El PKGBUILD de esta corrida: el del repositorio con la versión del código
# empaquetado, la fuente de este tarball y su suma real.
sed -e "s|^pkgver=.*|pkgver=$version|" \
    -e "s|^source=.*|$source_line|" \
    -e "s|^sha256sums=.*|sha256sums=('$sum')|" \
    "$pkgbuild" >"$work/PKGBUILD"

# El .SRCINFO que sirve la AUR: el de ese PKGBUILD.
(cd "$work" && makepkg --printsrcinfo) >"$work/.SRCINFO"

echo "==> el paquete: genmsi $version (suma $sum)"
(cd "$work" && makepkg --nodeps)

package="$(ls -t "$work"/genmsi-*.pkg.tar.zst 2>/dev/null | head -1 || true)"
[[ -n "$package" ]] || die "makepkg no dejó ningún paquete"
echo "==> el paquete: $package"
echo "==> el PKGBUILD relleno: $work/PKGBUILD"
echo "==> el .SRCINFO: $work/.SRCINFO"

if [[ "$install" -eq 1 ]]; then
  sudo pacman -U "$package"
  echo "==> instalado"
fi
