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
#   ./scripts/install-linux.sh --with-mcp   # además instala la CLI y
#                                             configura el MCP de Pi
#
# El MCP es lo que permite que un agente (Pi, u otro cliente MCP) lea,
# valide y simule los modelos: lo publica la misma CLI, con `genmsi mcp`.
#
set -euo pipefail

repo="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
app_dir="${HOME}/Applications/GenMSI"
launcher="${HOME}/.local/share/applications/GenMSI.desktop"
icon_dir="${HOME}/.local/share/icons/hicolor/128x128/apps"
# El icono lleva el nombre que pide el lanzador (Icon=GenMSI), el mismo con
# que lo instala el paquete de pacman.
icon="${icon_dir}/GenMSI.png"

# La compilación de Tauri es la parte pesada de este script: el limitador de
# trabajos de .cargo/config.toml no alcanza cuando el enlazado pide toda la
# memoria de la máquina, así que aquí se pide -j 2 a mano.
build=1
with_mcp=0
for arg in "$@"; do
  case "$arg" in
    --no-build) build=0 ;;
    --with-mcp) with_mcp=1 ;;
    *) echo "opción desconocida: $arg" >&2; exit 2 ;;
  esac
done

if [[ $build -eq 1 ]]; then
  echo "== compilando la aplicación =="
  # `cargo tauri build` no acepta `-j`: el paralelismo lo acota el
  # `.cargo/config.toml` del repositorio (4 trabajos).
  (cd "${repo}/crates/genmsi-app" && cargo tauri build)
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
install -m 644 "${repo}/crates/genmsi-app/LEEME.txt" "${app_dir}/LEEME.txt"
install -m 644 "${repo}/crates/genmsi-app/src-tauri/icons/128x128.png" "${icon}"

# El lanzador: el del repositorio, con la ruta del programa cambiada (el
# programa vive en la carpeta del usuario y no en el PATH).
sed "s|^Exec=.*|Exec=${app_dir}/genmsi-app-desktop|" \
  "${repo}/packaging/linux/GenMSI.desktop" > "${app_dir}/GenMSI.desktop"
chmod 644 "${app_dir}/GenMSI.desktop"

# El lanzador del menú: la ruta es absoluta, así que el escritorio lo
# encuentra desde cualquier lugar.
sed "s|^Exec=.*|Exec=${app_dir}/genmsi-app-desktop|" \
  "${repo}/packaging/linux/GenMSI.desktop" > "${launcher}"
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

# El MCP: la misma CLI, con el subcomando `mcp`, habla el protocolo y
# publica seis herramientas (lint, simulate, get_trace, explain_model,
# catalogue y save_model). Sin esto, un agente no puede trabajar con los
# modelos.
if [[ $with_mcp -eq 1 ]]; then
  echo
  echo "== el MCP =="
  if [[ $build -eq 1 ]]; then
    (cd "${repo}" && cargo build --release -j 2 -p genmsi-cli)
  fi
  cli="${repo}/target/release/genmsi"
  if [[ ! -x "${cli}" ]]; then
    echo "no está la CLI en ${cli}" >&2
  else
    install -Dm 755 "${cli}" "${HOME}/.local/bin/genmsi"
    echo "CLI instalada en ${HOME}/.local/bin/genmsi"
    if command -v pi >/dev/null 2>&1; then
      pi mcp add genmsi -- "${HOME}/.local/bin/genmsi" mcp || true
      echo "MCP configurado. En una sesión de Pi, /reload para que aparezcan sus herramientas."
    else
      echo "Para conectarlo a Pi:"
      echo "  pi mcp add genmsi -- ${HOME}/.local/bin/genmsi" mcp
    fi
  fi

  # La guía del agente: el archivo que enseña a construir un modelo con el
  # MCP sin leer el código del motor. Viaja con el producto, en el
  # directorio de skills del usuario.
  skill_src="${repo}/skills/genmsi-modeling"
  if [[ -d "${skill_src}" ]]; then
    skill_dir="${HOME}/.agents/skills/genmsi-modeling"
    mkdir -p "${skill_dir}"
    install -m 644 "${skill_src}/SKILL.md" "${skill_dir}/SKILL.md"
    echo "Guía del agente instalada en ${skill_dir}"
  fi
fi
