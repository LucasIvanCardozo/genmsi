# Fase 0: block inventory + Wine oracle harness

Phase 0 session for genmsi-rs. Coordinator: "GenMSI-rs: sesión principal coordinadora" (D12). Scope: scripts/, docs/oracle-wine.md, docs/BLOCK-INVENTORY.md, fixtures/. Never edit crates/ (Phase 1).

## Specs

- S1 (verbatim): "RESPALDO antes de cualquier uso de Wine: copiar Genmsi_20230711/ a Genmsi_20230711.backup/ (está git-ignored; Wine puede escribir dentro de la carpeta del exe). Trabajá siempre sobre la copia o con un WINEPREFIX aislado (ej. scripts/wine-oracle/wineprefix/, también git-ignored)."
- S2 (verbatim): "Inventario: script que recorra fixtures/models/*.gmdl (25 modelos, XML con <gmodel><gclasses><modules>) y reporte por clase de módulo: cuántas veces se usa y qué propiedades (atributos) usa con valores no vacíos. Salida: docs/BLOCK-INVENTORY.md con tabla por clase. Esta lista define el subconjunto de bloques de la Fase 1 (decisión D9)."
- S3 (verbatim): "Oráculo: descubrir cómo extraer salidas del original bajo Wine (wine 11.19 instalado). genmsi.exe es una GUI Delphi de 28 MB: primero probá si acepta el .gmdl por parámetro de línea de comandos o si escribe archivos de resultados legibles; la automatización de GUI es el último recurso. Documentá el método paso a paso en docs/oracle-wine.md y dejá el script ejecutable en scripts/wine-oracle/."
- S4 (verbatim deliverable): "(a) docs/BLOCK-INVENTORY.md completo sobre los 25 modelos, (b) docs/oracle-wine.md con el método de extracción reproducido para al menos un modelo, (c) scripts de inventario y oráculo commiteados."
- S5 (verbatim prohibitions): "Prohibido: subir nada a remotos, borrar o modificar los originales de la carpeta de la facultad, commitear binarios de más de 1 MB."
- S6: Report at end: which blocks the models use, how outputs are extracted from the original, and the commits evidencing each point. If Wine fails blocking, report the exact error instead of improvising.
- S7: backup dir and wineprefix must be git-ignored; never commit Wine droppings, binaries >1 MB, or model outputs larger than reasonable text.

## Tasks

- T1 (S1, S5, S7) Create Genmsi_20230711.backup/ (full copy, verified with diff -r), add git-ignore entries for backup + wineprefix. Route: inline.
- T2 (S2, S4a) Write scripts/inventory_blocks.py, run over 25 models, generate docs/BLOCK-INVENTORY.md (per-class table: uses, non-empty properties, per-model presence). Route: inline (deterministic scripting).
- T3 (S3, S4b) Wine oracle discovery: CLI-arg test, readable output files test, GUI automation last resort; WINEPREFIX at scripts/wine-oracle/wineprefix/; write docs/oracle-wine.md + executable scripts/wine-oracle/ extract script; reproduce for ≥1 model. Route: inline (interactive discovery).
- T4 (S4c, S5, S6) Commit inventory script + doc, oracle script + doc as reviewable work units; verify no >1 MB binaries staged; report. Route: inline.
- T5 (S3, S4b) Headless (Xvfb) harness validation + clean re-extraction of TP2 baselines into fixtures/outputs/. Blocked on: user installs xorg-server-xvfb (sudo pacman -S xorg-server-xvfb).

## Log

- L1 (verbatim, first message): "FASE 0 — GenMSI-rs: inventario de bloques + harness oráculo Wine. Sos la sesión encargada de la Fase 0 del proyecto genmsi-rs. La sesión coordinadora es 'GenMSI-rs: sesión principal coordinadora'; cualquier decisión de producto o duda de alcance, consultá por ese canal y no decidas por tu cuenta." + scope/paths/objectives as quoted in S1–S6; wine 11.19 installed; consult coordinator for product decisions.
- L2 (session evidence): Wine 11.19 findings — genmsi.exe opens the .gmdl from the CLI argument; writes no output files; simulation runs from Modelo→Simular (F9, blue-globe run button); Reporte + Código GPSS tabs extracted via X11 clipboard (Ctrl+Home, Ctrl+Shift+End, Ctrl+C — Delphi rich edit has no Ctrl+A) into tkinter clipboard_get. Reproduced manually for TP2__problema1.gmdl (report + GPSS source). Live-desktop xdotool automation is fragile under KDE/Wayland while the user works (focus-stealing prevention, XWayland 1x1 _NET_ACTIVE_WINDOW proxy, user mouse overrides); decision with user: headless Xvfb as the harness runtime (user installs xorg-server-xvfb). Commits: 33b5f26 (inventory), 758e7b9 (oracle docs+harness).
- L3 (session evidence): user installed xorg-server-xvfb; run_oracle.sh rewritten headless (private Xvfb :99 1280x800, no window manager, window-relative coordinates, no user-desktop interaction). Validated end to end for TP2__problema1.gmdl: report (129 lines) + Código GPSS (60 lines) extracted cleanly. Baselines committed under fixtures/outputs/.
