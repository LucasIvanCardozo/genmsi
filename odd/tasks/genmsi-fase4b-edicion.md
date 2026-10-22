# Fase 4b: genmsi-app — edición de nodos y guardado

Phase 4b session of genmsi-rs. Coordinator: "GenMSI-rs: sesión principal coordinadora"
(D12); the product questions were consulted with the user (the peer catalog was
unavailable in this session: `session discovery is unavailable`). Scope:
`crates/genmsi-app/` (frontend, `engine/`, `src-tauri/`, `scripts/`), additive
public API in `crates/genmsi-format/` and `crates/genmsi-core/`, `docs/` (new
documents; `DECISIONS.md` only to append an entry) and `odd/tasks/`.

## Specs

- S1 (verbatim, user answer): "Guardar en .gmdl (compatible con el original)" —
  "El archivo editado se abre igual en la app Delphi: el writer ya hace round-trip
  byte a byte y conserva layout y elementos desconocidos. Compatibilidad total con
  lo que ya existe."
- S2 (verbatim, user answer): "Preservar, con reacomodo opcional" — "El grafo se ve
  como lo dejó el autor del archivo; los nodos nuevos se colocan donde se sueltan. Un
  botón explícito `Reacomodar` reordena y es deshacible."
- S3 (verbatim, user answer): "Edición completa: topología + propiedades + guardado" —
  "Crear, borrar, conectar, mover, editar propiedades, Ctrl+Z/Y y guardar .gmdl. Un
  modelo editable de punta a punta en un cierre, con loop visual por slice."
- S4 (D8, verbatim): "- Node editor (React Flow) with real Ctrl+Z/Y, multi-select,
  copy/paste with stable ids (the original's structural-bug source)."
- S5 (Phase 1 S1, verbatim): "la reescritura de un modelo cargado debe producir
  diferencias mínimas verificables" — editing one property must change that property
  and nothing else.
- S6 (D2, verbatim): "only `genmsi-format` and `genmsi-core` may be dependencies of
  more than one crate. CLI, MCP and GUI are thin layers with no simulation logic."
- S7 (Phase 4 S3, still binding, verbatim): "Español" — "todo el texto visible en
  español neutral" (no voseo).
- S8 (Phase 4 S5, still binding, verbatim): "algo importante a tener en cuenta aqui es
  que mientras se vaya haciendo la ui debe poderse revisar visualmente con agent
  bropwser o algo similar no?"
- S9 (assignment question 1, answered by the user): "Sí, incluir «modelo nuevo»" —
  the phase offers an empty canvas with the root container, next to the editing of
  existing files.
- S10 (assignment question 2, answered by the user): "Sí, pedir confirmación" —
  when the diagnostics carry a modelling error the engine refuses to run, the
  interface asks for confirmation before saving; saving is never blocked.

## Tasks

- T1 (S1–S10, deliverable) The feature document, the branch `feat/phase4b-edicion` and
  the Engram mirror. Commit: `docs(odd): phase 4b feature document for the node editor`.
- T2 (S5, S6) `genmsi-format` gains the additive editing API (`edit.rs`): add/remove
  block, add/remove link, set property value, set layout, assign a fresh unique
  `moduleid`, relabel the two ends of a link, and paste (fresh ids, remapped links).
  With it, the `EditSession`: the model plus the command stack of undo/redo (one
  entry per user operation, redo cleared on a new edit, not persisted). Tests: the
  mutations keep the model's invariants, the undo/redo round trip, the paste's ids.
  Commit: `feat(format): the additive editing API and the undo stack`.
- T3 (S3, S5) `genmsi-core` gains the block catalogue (the classes the app offers to
  create, with their ports and their declared properties) and `genmsi.model/1` grows
  the fields the editor needs (the block's ports, the property's raw value, `unique`
  and `visible`), additively inside its major version. The snapshot of the document
  moves with it. Commit: `feat(core): the block catalogue and the editable model document`.
- T4 (S1, S3, S6) The facade (`crates/genmsi-app/engine`) grows the editing session:
  open, apply a command, undo, redo, save, save-as, plus the lint of the model in
  memory and the catalogue document. The thin binary speaks the session over
  stdin/stdout (one JSON per line); the Tauri commands call the same facade. The
  facade stays a thin client: no simulation logic. Commit: `feat(app): the editing session in the facade`.
- T5 (S1, S8) The dev bridge: one long-lived session process, the editing routes and
  the dev work directory (the only place the dev path writes, and the log says so).
  The read-only routes keep returning the engine's bytes verbatim. Commit: `feat(app): the dev bridge speaks the editing session`.
- T6 (S3, S4, S8) The graph becomes an editor: create a block, delete it, connect two
  ports, move it, multi-select. The mutations travel as commands; the app never
  touches XML. Commit: `feat(app): the editable graph`.
- T7 (S3, S5, S7) The property panel becomes an editor: one editor per declared type
  (`TSNA`, `PosInteger`, `EntityUse`, `EntityDef`, `Listed`, `Expresion`, ...), with
  the validation the type allows, and the diagnostics panel that follows every
  mutation. Commit: `feat(app): the property editors by type and the diagnostics panel`.
- T8 (S3, S4) Undo/redo (`Ctrl+Z`, `Ctrl+Shift+Z`, `Ctrl+Y`), copy/paste with fresh
  ids and remapped links, and the explicit `Reacomodar` (a deterministic layered
  layout, computed in the frontend, covered by a test, never run on open).
  Commit: `feat(app): undo/redo, copy/paste and the explicit re-layout`.
- T9 (S1, S3, S9, S10) Save and save-as through the writer of the CLI, the dirty
  state, the confirmation before saving a model the engine refuses, and «modelo nuevo»
  (an empty canvas with the root container). Commit: `feat(app): save, save-as and the new model`.
- T10 (S5, S4) The tests: the save of the 25 reference models without changes is
  byte-identical; pasting twice leaves every id unique and every link resolvable;
  the saved file is refused by the engine with the same class as the in-app
  diagnostic; the frontend's layout is deterministic and its editors validate.
  Commit: `test(app): the editing invariants and the byte-identical save`.
- T11 (S8) The visual loop grows with the phase: create, connect, drag, undo, redo,
  save, reopen and show the change, one screenshot per step; and the Tauri smoke
  check under `Xvfb` with `xdotool` and `import`. Commit: `test(app): the visual loop of the editing steps`.
- T12 (deliverable) `docs/app.md` updated (the editing, the session, the commands) and
  the close: `cargo test --workspace`, `cargo clippy --workspace --all-targets --
  -D warnings`, the frontend's own tests, and the evidence of every task.
  Commit: `docs(app): the editing, the session and the commands`.

## Decisions (engineering, from the assignment)

- **The app never touches XML.** Every mutation is a command that the facade applies
  through the additive API of `genmsi-format`. A missing mutation is a defect of the
  facade, not a reason for string surgery in TypeScript.
- **The session owns the model.** The facade holds the model in memory, the undo
  stack with it, and the save writes that model through the writer of the CLI. The
  dev bridge keeps one long-lived session process; the Tauri shell keeps the same
  session in memory. Both call the same facade functions.
- **The labels are the connection points.** A link is a pair of ports that share a
  label; connecting two ports gives both the same label, and the label is what the
  engine follows. The block's own label is the label of its first labelled input.
- **The undo stack is one entry per operation.** A drag is one entry (the final
  position), not one per pixel; a paste is one entry; the re-layout is one entry.
- **The re-layout lives in the frontend** (a deterministic layered layout over the
  model graph) and reaches the model as one command with every position, so it is
  one undo entry.

## Commits

| Task | Commit |
|---|---|
| T1 | `230e2ef` `docs(odd): phase 4b feature document for the node editor` |
| T2 | `c40d712` `feat(format): the catalogue of the block classes`, `e34d5b9` `feat(format): the additive editing API and the undo stack` |
| T3 | `2c67393` `feat(core): the block catalogue document and the editable model document` |
| T4 | `2fc3cef` `feat(app): the editing session in the facade` |
| T5 | `149bc56` `feat(app): the dev bridge speaks the editing session` |
| T6–T9 | `ab08e5a` `feat(app): the editable graph, the property editors and the undo stack`, `a50e87a` `fix(app): the graph looped on the selection and the frame cut the picture` |
| T10 | `9bfc2f8` `test(app): the editing invariants and the byte-identical save` |
| T11 | `f2171d2` `test(app): the visual loop of the editing steps`, `68902e7` `test(app): the smoke check of the desktop shell`, `6af8e3a` `fix(app): the desktop shell looked for the models one level too high` |
| T12 | `d22b7a3` `docs(app): the editing, the session and the commands` |

## Log

- L1 (verbatim, first message):

```
FASE 4b — GenMSI-rs: genmsi-app (edición de nodos). Sos la sesión de la Fase 4b del
   proyecto genmsi-rs. La coordinadora es "GenMSI-rs: sesión principal coordinadora"; las
   decisiones de producto se consultan, no se deciden.

   Leé primero odd/tasks/genmsi-fase4b-assignment.md: trae las tres decisiones ya respondidas
   (guardar en .gmdl compatible con el original, preservar layout con reacomodo opcional,
   edición completa en un cierre), el alcance exacto, y las restricciones de ingeniería — la app
   nunca toca XML, guardar sin cambios debe ser byte-idéntico, diagnóstico continuo con guardado
   nunca bloqueado, undo de una entrada por operación, copy/paste con ids frescos y links
   remapeados. Creá tu feature doc odd/tasks/genmsi-fase4b-edicion.md con esas specs, tus
   tareas y este mensaje como L1 verbatim, antes de la primera escritura de código. Entregable:
   edición end-to-end con el loop visual extendido a los pasos de edición, cargo test + clippy +
   frontend tests verdes, docs/app.md actualizado y evidencia por tarea. Prohibido:
   crates/genmsi-cli/, crates/genmsi-mcp/, fixtures/, scripts/ de la raíz, y cambiar la forma de
   los documentos genmsi.*/1.
```

- L2 (evidence, baseline): the assignment's baseline is `main` @ `fb10f85` (203 tests
  + 8 frontend tests green, clippy clean, oracle 20/20). This session starts from
  `main` @ `4d4219b` (the assignment document) on the branch `feat/phase4b-edicion`.
- L3 (decision, the open questions): the peer catalog was unavailable in this session
  (`session discovery is unavailable`), so the two open questions of the assignment
  were put to the user directly. Both answers are recorded as `S9` and `S10`: the
  phase offers «modelo nuevo», and the interface asks for confirmation before saving
  a model the engine refuses (the saving itself is never blocked).
- L4 (decision, the session): the model in memory lives in the facade, not in the
  frontend: the dev bridge keeps one long-lived `genmsi-app-engine session` process
  and the Tauri shell keeps the same session in a static. The alternative (the
  frontend carrying the model) would have made the frontend the owner of the format,
  which the assignment forbids.
- L5 (evidence, the format crate): the unit was delegated to a writer and came back
  with the catalogue (51 classes, their ports with their ids and their properties
  with the declared type) and the editing API (`EditCommand`, `EditSession`,
  `UndoStep`). Its own checks: `cargo test -p genmsi-format` 67 tests green,
  clippy clean, and the optional `ts` feature compiles. The writer decided four
  things the assignment left open and they are recorded here: the session has a
  `mark_saved` (the dirty flag is real, and undoing past a save diverges from the
  saved file); the paste refuses containers (a container copy without its children
  would be silent data loss); the wire form renames the multi-word fields by hand
  (`fromIoid`, `toIoid`) so serde and `ts-rs` agree; and the `Listed` options are
  ASCII only, because the file's encoding is CP1252 and the editor must not write
  a value it cannot encode the way the original did.
- L6 (decision, where the catalogue lives): the catalogue says what a block looks
  like in the file (its ports, its properties), so it lives in `genmsi-format`; the
  engine's semantics stay in `genmsi-core`, which publishes the classes it
  compiles and a test pins the catalogue inside that list. A class the compiler
  does not know would create a block that simulates nothing.
- L7 (evidence, the documents): `genmsi.model/1` grew the ports of every block, the
  tooltip of every property and whether the property defines an entity — additive
  inside the major version, so the shape of the document is not broken for the
  clients that read it (D13). The snapshot of the document moved with it, and the
  new `genmsi.catalogue/1` carries the classes an editor offers to create.
- L8 (evidence, the visual loop): the loop ran on every step of the editing and
  found five things, all fixed: the graph reported the selection on every render
  and the actions were a new object every time, so the report and the store
  answered each other in a loop; the first view framed the containers as well as
  the flow and the picture shrank until nothing could be read; the new blocks
  landed far to the right, outside the picture; the list of classes scrolls, so a
  class has to be on the picture before it can be clicked; and the drag is a real
  mouse drag, because React Flow listens to the pointer events. The loop ends with
  eleven distinct screenshots, including the file reopened and the change in it.
- L9 (evidence, the desktop shell): the shell's window opens under `Xvfb` and the
  picture can be captured (1440×900 with 1345 distinct colours), and the log proves
  the frontend reaches the facade through `invoke` (`list_models: 25 modelos`). Three
  causes were behind the black window, and only one of them was the code: the
  WebKitGTK notes of Tauri (`v2.tauri.app/develop/debug/linux-graphics`) give the
  two flags the webview needs under Xvfb (`LIBGL_ALWAYS_SOFTWARE=1` and
  `WEBKIT_DISABLE_COMPOSITING_MODE=1`), the debug build of the shell loads the
  frontend from the dev server and not from the embedded assets, and the shell
  moved to the root of the repository climbing two levels instead of three, so the
  model list came back empty.
- L10 (verification, close): `cargo test --workspace` is green (239 tests) and
  `cargo clippy --workspace --all-targets -- -D warnings` is clean. The frontend's
  own checks are green too: `bun test` 19 of 19 (the dev bridge's bytes against the
  CLI's, the deterministic layout and the property editors) and `bunx tsc --noEmit`
  clean. The visual loop ran end to end with eleven screenshots and the desktop
  shell's smoke check passed.
- L11 (delivery, slices): the phase's commits are thirteen. Two of them are bigger
  than the ~400 authored lines and the reason is recorded instead of hidden: the
  catalogue (962 lines of data for 51 classes) and the editing API (1.5k lines with
  its tests) are one delegated unit of the format crate, and they are the two
  halves of it — the catalogue first, because the API builds blocks from it. The
  rest of the slices stay inside the guideline.
- L12 (limits, recorded): the paste refuses containers (a container carries its
  children, and copying one without them would lose data silently); the `Listed`
  options are the ones the reference models use, so a model with another value
  keeps it (the editor shows the current value among the options); the re-layout
  grows the containers to hold their blocks, which changes the size of a container
  in the file — it is one command, so it is one undo entry.
- L13 (risk): the change writes files (the save) and grows a document other
  clients read (`genmsi.model/1`), so risk items 1 and 3 apply. Item 1 is covered
  by the byte-identical save: a save without changes of the reference models
  produces the same bytes, and the paste test proves that the saved file loads,
  validates and compiles. Item 3 is covered by the versioned envelope (D13): the
  document only grew fields, and the snapshot pins the new shape. Risk item 6 does
  not apply: the bytes the app shows are compared with the CLI's by a test, and
  the frontend's pure logic has its own tests.
