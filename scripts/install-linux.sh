#!/usr/bin/env bash
#
# Instala la aplicación de escritorio para Linux (x86_64) en la carpeta de
# aplicaciones del usuario, sin necesidad de administrador, y deja el lanzador
# y el icono donde el escritorio los encuentra.
#
# La instalación anterior se hacía a mano, dos veces, y con lo que quedó
# instalado era una copia vieja del motor: los números de los modelos que
# mostraba la app no eran los de la última versión. Este script deja el
# paso repetible y siempre con el motor del día.
#
#   ./scripts/install-linux.sh              # compila e instala
#   ./scripts/install-linux.sh --no-build   # sólo reinstala lo ya compilado
#
set -euo pipefail

repo="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
app_dir="${HOME}/Applications/GenMSI"
launcher="${HOME}/.local/share/applications/GenMSI.desktop"
icon_dir="${HOME}/.local/share/icons/hicolor/128x128/apps"
icon="${icon_dir}/genmsi-app-desktop.png"

# La compilación de Tauri es la parte pesada de este script: el limitador de
# trabajos de .cargo/config.toml no alcanza cuando el enlazado pide toda la
# memoria de la máquina, así que aquí se pide -j 2 a mano.
build=1
[[ "${1:-}" == "--no-build" ]] && build=0

if [[ $build -eq 1 ]]; then
  echo "== compilando la aplicación =="
  (cd "${repo}/crates/genmsi-app" && cargo tauri build -j 2)
fi

binary="${repo}/crates/genmsi-app/src-tauri/target/release/genmsi-app-desktop"
if [[ ! -x "${binary}" ]]; then
  echo "no está el binario en ${binary}: compila primero o quita --no-build" >&2
  exit 1
fi

echo "== instalando en ${app_dir} =="
mkdir -p "${app_dir}" "${icon_dir}" "$(dirname "${launcher}")"
install -m 755 "${binary}" "${app_dir}/genmsi-app-desktop"
install -m 644 "${repo}/crates/genmsi-app/src-tauri/icons/128x128.png" "${app_dir}/GenMSI.png"
install -m 644 "${repo}/crates/genmsi-app/src-tauri/icons/128x128.png" "${icon}"

cat > "${app_dir}/GenMSI.desktop" <<DESKTOP
[Desktop Entry]
Type=Application
Name=GenMSI
Comment=Simulador GPSS
Exec=${app_dir}/genmsi-app-desktop
Icon=genmsi-app-desktop
Terminal=false
Categories=Education;
DESKTOP
chmod 644 "${app_dir}/GenMSI.desktop"

# El lanzador del menú: la ruta es absoluta, así que el escritorio lo
# encuentra desde cualquier lugar.
cat > "${launcher}" <<DESKTOP
[Desktop Entry]
Type=Application
Name=GenMSI
Comment=Simulador GPSS
Exec=${app_dir}/genmsi-app-desktop
Icon=genmsi-app-desktop
Terminal=false
Categories=Education;
DESKTOP
chmod 644 "${launcher}"

# El escritorio cachea los iconos: sin esto, el icono nuevo no aparece
# hasta que se vuelva a iniciar la sesión.
if command -v update-desktop-database >/dev/null 2>&1; then
  update-desktop-database "${HOME}/.local/share/applications" >/dev/null 2>&1 || true
fi
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
  gtk-update-icon-cache -f -t "${HOME}/.local/share/icons/hicolor" >/dev/null 2>&1 || true
fi

echo
echo "== instalado =="
ls -l "${app_dir}/genmsi-app-desktop" "${launcher}" "${icon}"
echo
echo "Se abre con: ${app_dir}/genmsi-app-desktop"
