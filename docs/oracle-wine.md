# Wine oracle harness (Phase 0)

How to extract simulation outputs from the original GenMSI (`genmsi.exe`,
Delphi GUI, 28 MB) running under Wine, so they can serve as oracle baselines
for the Rust engine (decisions D6/D9: Wine verifies; published GPSS
semantics lead).

Environment this was validated on: Wine **11.19**, KDE Plasma 6 (Wayland),
genmsi `v1.2020.07.28 BETA1!`.

## Safety rules

- Always work on the backup copy `Genmsi_20230711.backup/` (verified with
  `diff -r` against the original before any Wine use). Wine may write inside
  the exe folder; the faculty original must never be touched.
- Use an isolated `WINEPREFIX=scripts/wine-oracle/wineprefix/` (git-ignored),
  created with `wineboot -i`.
- Extraction artifacts stay in `scripts/wine-oracle/out/` (git-ignored); the
  curated baselines committed under `fixtures/outputs/` are small text files.

## Findings

1. **The `.gmdl` path is accepted on the command line.** Launching
   `genmsi.exe <path.gmdl>` opens the model directly (the main window title
   shows the model path). No GUI navigation is needed to load a model.
   Windows-style path: `Z:\home\...\fixtures\models\<model>.gmdl`.
2. **The app writes no output files.** After a full simulation there were no
   new files in the wineprefix or the exe folder (beyond registry state).
   The simulation results live only inside the GUI.
3. **Simulation is run from the GUI**: menu `Modelo -> Simular` (F9) opens a
   `Simulador` panel with a `Start:` counter, a run button (blue globe;
   the green one is only "Actualizar"/refresh), and output tabs:
   `Código GPSS` and `Reporte` (plus Facilities, Queues, Storages, QTables,
   CEC, FEC, Transacciones, Monitor).
4. **`Reporte` contains the full GPSS-style report** (`START_TIME END_TIME
   BLOCKS FACILITIES STORAGES`, the block table, entity statistics) and
   `Código GPSS` contains the generated GPSS source of the model — both are
   exactly what the oracle needs.
5. **Text extraction goes through the X11 clipboard.** The output pane is a
   Delphi rich edit: `Ctrl+A` does NOT work; use
   `Ctrl+Home`, `Ctrl+Shift+End`, `Ctrl+C`. The clipboard can then be read
   from the Linux side (see `read_clipboard.py`).

## Manual method (validated end to end for `TP2__problema1.gmdl`)

The same steps, useful for debugging the harness:

```sh
export WINEPREFIX=$PWD/scripts/wine-oracle/wineprefix
cp -a Genmsi_20230711 Genmsi_20230711.backup   # only once; then diff -r
wineboot -i                                    # only once

# 1. Launch with the model (virtual desktop keeps the app in one window)
wine explorer /desktop=genmsi,1280x800 \
  Genmsi_20230711.backup/genmsi.exe \
  'Z:\home\lucas\Documentos\Workspace\Genmsi\fixtures\models\TP2__problema1.gmdl'

# 2. Menu "Modelo" -> "Simular" (F9). In the Simulador panel press the run
#    button (Start: 1 is the default). Wait for "Simulación finalizada".

# 3. Select the "Reporte" tab, click inside the report text, then
#    Ctrl+Home, Ctrl+Shift+End, Ctrl+C, and dump the clipboard:
python3 scripts/wine-oracle/read_clipboard.py > out/model.report.txt

# 4. Same for the "Código GPSS" tab:
python3 scripts/wine-oracle/read_clipboard.py > out/model.gpss.txt

# 5. Close the Wine session
wineserver -k
```

## Automated harness

`scripts/wine-oracle/run_oracle.sh <model.gmdl> [outdir]` automates steps
1–4 with `xdotool` (input injection needs the KDE "Control remoto"
permission the first time) and `read_clipboard.py` for the clipboard.

Input injection over the live KDE/Wayland session is inherently fragile
while the desktop is in use: KWin focus-stealing prevention blocks
`xdotool windowactivate`, XWayland reports a 1×1 proxy window as
`_NET_ACTIVE_WINDOW` when a Wayland window holds focus, and real mouse
movement overrides injected moves. The script therefore:

- runs the app in a Wine virtual desktop window,
- raises it with a KWin scripting helper (`activate_genmsi.js`, loaded via
  the `org.kde.KWin` DBus Scripting API) before every input action, and
- aborts instead of typing if KWin does not confirm the activation.

**Headless mode (default, no interference with the desktop at all):**
`run_oracle.sh` runs Wine on a private Xvfb display (`:99`, started
automatically if absent). The GenMSI window is the only client there, always
focused; coordinates are computed relative to the window so window placement
does not matter. Set `XDISPLAY` to reuse a different private display.

## Gotchas discovered

- `pkill -f genmsi.exe` also matches the calling shell if its command line
  contains the string; use `pkill -x genmsi.exe` or the exact PID.
- Wine clipboard integration works with `tkinter`'s `clipboard_get()`
  against the same X display Wine runs on; no `xclip` needed.
- The right-click on simulator tabs opened an "Abrir" file dialog instead
  of a context menu — avoid right-clicks inside the Simulador panel.
- Delphi GUI apps keep hidden helper windows (e.g. 1×1 "Default IME")
  that can hold the X input focus while the main window looks active.
- The report's first character can be lost if the click that places the
  caret lands on it; `Ctrl+Home` after clicking inside the pane avoids it.

## Oracle baselines

Extracted baselines are committed under `fixtures/outputs/` as
`<model>.report.txt` and `<model>.gpss.txt`. These are single sample runs:
comparison against the Rust engine uses statistical tolerance (D6), not
bit-exact equality.
