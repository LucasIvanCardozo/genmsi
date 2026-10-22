# Fase 4b (asignación): genmsi-app — edición de nodos y guardado

Coordinator assignment for the second Phase 4 session. Coordinator's input, not the phase's
feature document: the session creates `odd/tasks/genmsi-fase4b-edicion.md` with these specs
as `S1..Sn`, its own tasks and log, and a verbatim `L1` holding the user's message that
opens the phase. The product questions below are already answered; nothing here is
re-decided in the session.

Baseline at assignment time: `main` @ `fb10f85`, 203 tests + 8 frontend tests green, clippy
clean, oracle 20/20, vertical slice merged (graph, properties, report, visual loop).

## Scope of this phase

Full editing of a model in the app, saved back to `.gmdl`: create, delete, connect, move and
re-property blocks, with real undo/redo, copy/paste with stable ids, an explicit optional
re-layout, and save/save-as. The live monitor and the charts are **out of scope** (4c) and
must not be stubbed.

## Specs

- S1 (verbatim, user answer): "Guardar en .gmdl (compatible con el original)" — "El archivo
  editado se abre igual en la app Delphi: el writer ya hace round-trip byte a byte y conserva
  layout y elementos desconocidos. Compatibilidad total con lo que ya existe."
- S2 (verbatim, user answer): "Preservar, con reacomodo opcional" — "El grafo se ve como lo
  dejó el autor del archivo; los nodos nuevos se colocan donde se sueltan. Un botón explícito
  `Reacomodar` reordena y es deshacible."
- S3 (verbatim, user answer): "Edición completa: topología + propiedades + guardado" —
  "Crear, borrar, conectar, mover, editar propiedades, Ctrl+Z/Y y guardar .gmdl. Un modelo
  editable de punta a punta en un cierre, con loop visual por slice."
- S4 (D8, verbatim): "- Node editor (React Flow) with real Ctrl+Z/Y, multi-select, copy/paste
  with stable ids (the original's structural-bug source)."
- S5 (Phase 1 S1, verbatim): "la reescritura de un modelo cargado debe producir diferencias
  mínimas verificables" — editing one property must change that property and nothing else.
- S6 (D2, verbatim): "only `genmsi-format` and `genmsi-core` may be dependencies of more than
  one crate. CLI, MCP and GUI are thin layers with no simulation logic."
- S7 (Phase 4 S3, still binding, verbatim): "Español" — "todo el texto visible en español
  neutral" (no voseo).
- S8 (Phase 4 S5, still binding, verbatim): "algo importante a tener en cuenta aqui es que
  mientras se vaya haciendo la ui debe poderse revisar visualmente con agent bropwser o algo
  similar no?"

## Engineering constraints (coordinator decisions)

- **The app never touches XML.** Every mutation goes through additive API in
  `genmsi-format` (add/remove block, add/remove link, set property value, set layout, assign
  a new unique `moduleid`, remap a link) exposed by the facade crate
  (`crates/genmsi-app/engine`). If a mutation is missing there, it is a defect in the facade,
  not a reason for string surgery in TypeScript.
- **Save is a write of the loaded model**, through the same writer the CLI uses, so the file
  stays byte-identical where it was not edited (S5). A "save without changes" of each of the
  25 fixtures must produce a byte-identical file — that is a test, not an observation.
- **Diagnostics are continuous, saving is never blocked.** A model can be momentarily
  invalid while being edited; the app re-runs `lint` after each mutation and shows the
  diagnostics, and saving an invalid model is allowed. A test pins that the saved file is
  refused by the engine with the same class as the in-app diagnostic.
- **Undo/redo is a command stack** with one entry per user operation (a drag is one entry,
  not one per pixel), redo cleared on a new edit, and the stack is not persisted. `Ctrl+Z`,
  `Ctrl+Shift+Z` and `Ctrl+Y` all work.
- **Copy/paste assigns fresh ids** to the pasted blocks, remaps their links atomically, and
  offsets them so they do not overlap the originals. This is the original's structural-bug
  source (S4) and needs a dedicated test: paste twice, then save, then load, and every id is
  unique and every link resolves.
- **Property editors come from the declared type**, not one free-text field: the `.gmdl`
  declares `TSNA`, `PosInteger`, `EntityUse`, `Listed` and friends. Render an editor that
  matches the type, and validate what can be validated before it reaches the model.
- **Re-layout (`Reacomodar`) is explicit and undoable** (S2): a deterministic layered layout
  over the model graph, computed in the frontend, covered by a test (same input, same
  positions), never run on open. If you add a JS layout dependency it is the item-5 surface:
  pin it and say so.
- **The two backends stay identical**: `devBridge` (Vite middleware, dev only) and `tauri`
  both call the same facade operations, including save. The dev bridge writes files only in
  dev and must say so in its log line.
- **The visual loop grows with the phase** (S8): `crates/genmsi-app/scripts/check-visual-loop.sh`
  must also create a node, connect it, drag it, undo, redo, save, reopen and show the change,
  with distinct screenshots per step. It stays the single source of truth the document
  points to.
- **Tauri smoke check without a window manager**: `Xvfb`, `xdotool` and ImageMagick `import`
  are installed; there is no WM. Build and launch the shell, assert the window exists with
  `xdotool search --name`, and try `import -window root`. If the capture is empty, document
  it and move on — the Vite loop is the visual gate, the Tauri path only needs to prove the
  `invoke` commands work (a log line from the app is enough).

## Allowed edit paths

- `crates/genmsi-app/` (frontend, `engine`, `scripts`, tests, `src-tauri`).
- `crates/genmsi-format/` and `crates/genmsi-core/`: additive API only (new public functions,
  new public types, `#[cfg_attr(feature = "ts", ...)]` on new types). No change to parsing,
  to what the writer emits for an unedited model, or to simulation semantics.
- `docs/` (new documents; `DECISIONS.md` only to append an entry at the end).
- `odd/tasks/`.

Prohibited: `crates/genmsi-cli/`, `crates/genmsi-mcp/`, `fixtures/` (read-only oracle
material), `scripts/` at the repo root, the comparison thresholds in `oracle.rs`, and
changing the documents `genmsi.report/1` / `lint/1` / `trace/1` / `model/1` shapes.

## Reading order

1. `docs/DECISIONS.md` D1–D15.
2. `docs/app.md` and `docs/app-visual-loop.md` — the slice that exists.
3. `docs/format-and-engine.md` — the semantic model, containers, links and the writer.
4. `crates/genmsi-format/src/{model,writer,validate}.rs` — what a mutation has to keep true.
5. `crates/genmsi-format/tests/fixtures_roundtrip.rs` — the invariant you must not break.
6. `odd/tasks/genmsi-fase4-app.md` — the review that closed the previous phase, including
   the four defects it fixed.

## Known traps — do not rediscover

- Labels are the connection points: a block's `label()` is the label of its first labelled
  input port, and links find their target by label, not by id. Renaming must keep the
  invariant or the model stops compiling.
- Containers own their links; a link that ends at a container's output port is an exit and
  continues in the parent. Editing across container boundaries needs that rule.
- The original's structural bugs came from reusing ids on paste (S4) and from links that the
  editor forgot to remap; do not reproduce them.
- Five reference models are refused by the engine by design; the app must keep showing the
  error state, and saving such a model must still round-trip it byte-identically.
- `genmsi.model/1` is the document the graph renders; if editing needs more than it carries
  (e.g. the property type), extend the document additively and bump nothing.

## Deliverable

Editing end to end with the visual loop as evidence: create/delete/connect/move blocks,
edit properties by type, undo/redo, copy/paste with fresh ids, explicit re-layout, save and
save-as to `.gmdl` with byte-identical output for untouched content, continuous diagnostics,
all text in Spanish, `check-visual-loop.sh` extended to the editing steps, `docs/app.md`
updated, and the feature document with the evidence per task.

## Open questions (ask the coordinator; do not decide)

1. Whether this phase also offers "modelo nuevo" (an empty canvas with the root container
   and no blocks) or only edits existing files. It is one small step from "create a node",
   but it changes the save flow (there is no path yet), so ask before building it.
2. What the app does when the user saves a model whose diagnostics include a modelling error
   the engine refuses to run (it saves anyway per the constraint above — the open question is
   whether the UI asks for confirmation first).
