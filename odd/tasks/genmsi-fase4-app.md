# Fase 4: genmsi-app — vertical slice

Phase 4 session for genmsi-rs. Coordinator: "GenMSI-rs: sesión principal coordinadora"
(D12); product decisions are consulted, not decided here. Scope: `crates/genmsi-app/`
(frontend, `engine/`, `src-tauri/`), additive public API in `crates/genmsi-core/` and
`crates/genmsi-format/`, `docs/` (new documents; `DECISIONS.md` only to append an entry),
`odd/tasks/`, and the workspace `Cargo.toml` (`members`/`exclude` only).

## Specs

- S1 (verbatim, user answer): "Vertical slice primero (abrir + simular + reporte)" —
  "Abrir un `.gmdl`, ver el grafo y las propiedades, simular y leer reporte/cola/facility
  desde el JSON del motor. La edición de nodos viene después, sobre algo que ya se ve y se
  prueba."
- S2 (verbatim, user answer): "Rediseño moderno" — "Diseño propio desde cero, priorizando
  claridad sobre parecido con el original."
- S3 (verbatim, user answer): "Español" — "Como el original y como los modelos de la
  facultad: todo el texto visible en español neutral." (No voseo. El código, nombres de
  archivos, commits y docs del repo siguen en inglés.)
- S4 (verbatim, user answer): "Loop documentado, sin imágenes en el repo" — "El loop queda
  documentado y se ejecuta en cada slice, pero las capturas no se commitean (quedan en
  /tmp): el repo no carga imágenes."
- S5 (verbatim, the requirement that opened this phase): "algo importante a tener en cuenta
  aqui es que mientras se vaya haciendo la ui debe poderse revisar visualmente con agent
  bropwser o algo similar no?" — the UI must be reviewable visually with `agent-browser`
  while it is being built, not only at the end.
- S6 (ROADMAP Phase 4, verbatim): "- Node editor (React Flow) with real Ctrl+Z/Y,
  multi-select, copy/paste with stable ids (the original's structural-bug source). - Live
  monitor, property grid, reports and charts from engine JSON. - Generated TS types from
  the Rust schema." Exit criteria: "the workflow of the original app is reproducible in the
  GUI." This phase delivers the read-and-simulate half of it.
- S7 (D8, verbatim): "The node editor needs a best-in-class graph component; native GUI
  toolkits (egui, Slint) would require hand-building it. Frontend: React Flow, Zustand,
  Tailwind + shadcn/ui, Recharts (single charting library). Shared types are generated from
  the Rust schema (ts-rs or typify). GUI work starts only after the engine and format are
  frozen by tests."
- S8 (D2, verbatim): "only `genmsi-format` and `genmsi-core` may be dependencies of more
  than one crate. CLI, MCP and GUI are thin layers with no simulation logic."
- S9 (D13, verbatim): "`--json` wraps the body in a versioned envelope,
  `{"schema": "genmsi.<document>/<major>", ...}`; within one major version a change is
  additive only [...] and a breaking change bumps the number."
- S10 (D12, verbatim): "Each phase runs in a dedicated session that receives its assignment
  (objective, allowed edit paths, S# specs, deliverable) from the coordinator and consults
  it instead of making product decisions."

## Tasks

- T1 (S10, deliverable) The feature document, the branch `feat/phase4-app` and the Engram
  mirror. Commit: `docs(odd): phase 4 feature document for the app session`.
- T2 (S1, S9) `genmsi-core` publishes the one document the slice was missing: the model's
  graph (`genmsi.model/1`) — blocks with their canvas geometry, their decoded properties
  and the links between them. Additive public API only; the CLI's documents do not move.
  Tests: the document of a reference model pins the blocks, the geometry and the edges; a
  model that does not compile still yields its graph.
  Commit: `feat(core): publish the model graph document`.
- T3 (S7) The TypeScript types are generated from the Rust types, not written by hand:
  `ts-rs` derives on the published documents (optional `ts` feature, so the default
  workspace build does not change), the generator invocation committed, the generated
  bindings committed under the frontend.
  Commit: `feat(app): generate the TypeScript types from the Rust schema`.
- T4 (S8, S9) `crates/genmsi-app/engine` (`genmsi-app-engine`): the Rust facade both
  backends share — it loads a model, builds the documents and returns their bytes. A thin
  binary (`genmsi-app-engine model|lint|simulate|models`) is what the Vite dev bridge
  spawns. No simulation logic of its own: it calls `genmsi-core`.
  Commit: `feat(app): the engine facade shared by both backends`.
- T5 (S5) The frontend runs under Vite, standalone: React 19 + TypeScript + Tailwind, with
  the dev bridge as a Vite middleware that spawns the engine binaries and returns the
  engine's bytes verbatim. One adapter interface, two backends (dev bridge and Tauri).
  Commit: `feat(app): the frontend under Vite with the dev bridge`.
- T6 (S1) The graph view: React Flow renders the model's blocks with the geometry the
  `.gmdl` carries, read-only; selecting a block selects it.
  Commit: `feat(app): the block graph from the model's own geometry`.
- T7 (S1) The property panel: the selected block's properties, decoded, with its identity.
  Commit: `feat(app): the property panel of the selected block`.
- T8 (S1) The report view: the tables of `genmsi.report/1` (blocks, facilities, queues,
  storages, chains, logic, savevalues, tables) rendered, not recomputed.
  Commit: `feat(app): the report tables from the engine's JSON`.
- T9 (S2, S3) The modern design and the Spanish text, the model picker and the error
  state: a model the engine refuses is an error the user reads, never an empty graph.
  Commit: `feat(app): the model picker, the error state and the Spanish interface`.
- T10 (S5, S8) The Tauri shell: `src-tauri/` with commands that call the same facade, and
  the frontend's Tauri backend behind the same adapter. The Tauri crate stays out of the
  default workspace members so the workspace build does not slow down.
  Commit: `feat(app): the Tauri shell behind the same adapter`.
- T11 (S5, S4) The visual loop, documented and run: `docs/app-visual-loop.md` with the
  exact commands, and the evidence (screenshots in `/tmp`, never committed).
  Commit: `docs(app): the visual loop with agent-browser`.
- T12 (deliverable) `docs/app.md`: the design, the structure and the adapter contract.
  Commit: `docs(app): the design and the structure of the app`.
- T13 (deliverable) The parity test and the close: the dev bridge's bytes against
  `genmsi simulate --json` and `genmsi lint --json`, `cargo test --workspace` green,
  `cargo clippy --workspace --all-targets -- -D warnings` clean, and the feature document
  closed with the commit of every task.
  Commit: `test(app): the dev bridge returns the engine's bytes`.

## Decisions (engineering, from the assignment; product questions go to the coordinator)

- **The frontend runs under Vite, without Tauri in the loop.** `agent-browser` drives a
  browser, so the reviewable surface is a Vite dev server on `localhost`; the Tauri shell
  wraps the same frontend afterwards.
- **One adapter, two backends.** `devBridge` (a Vite dev middleware that spawns the
  engine binaries and returns their stdout verbatim) and `tauri` (commands that call the
  same Rust facade). Neither backend builds a document: they move the engine's bytes.
- **The documents come from the engine.** The report is `genmsi.report/1` and the lint is
  `genmsi.lint/1`, byte for byte what the CLI prints. The graph needed one document that
  did not exist (`genmsi.model/1`), published by `genmsi-core` (T2).
- **The types are generated.** `ts-rs` derives on the published Rust types; the frontend
  imports the generated bindings and never writes a document type by hand.
- **The Tauri crate is out of the default workspace members.** Its dependency tree is
  heavy; the workspace build stays as it was.

## Open questions of the assignment, answered here (the coordinator was not reachable)

The assignment asks two questions and says to consult the coordinator. The peer catalog
was unavailable in this session (`session discovery is unavailable`), so both are answered
with the reading the assignment itself supports, and both are reversible:

1. **The graph layout comes from the `.gmdl`.** The file carries the canvas geometry of
   every block (`Block::layout`), so the slice renders it as it is; re-running a layout
   engine belongs to editing (Phase 4b).
2. **The report view shows every table the document carries.** The volume turns out to be
   readable as a set of tabs (the block table is the long one and scrolls); nothing is
   curated away in this slice.

## Log

- L1 (verbatim, first message):

```
FASE 4 — GenMSI-rs: genmsi-app (vertical slice). Sos la sesión de la Fase 4 del proyecto
   genmsi-rs. La coordinadora es "GenMSI-rs: sesión principal coordinadora"; las decisiones
   de producto se consultan, no se deciden.

   Leé primero odd/tasks/genmsi-fase4-assignment.md: trae las cuatro decisiones de producto
   ya respondidas por el usuario (vertical slice, rediseño moderno, UI en español, loop
   visual sin imágenes en el repo), el alcance exacto de este cierre y las restricciones de
   ingeniería — el frontend debe correr suelto bajo Vite para poder revisarse con
   agent-browser, un solo adaptador del motor con dos backends, y un test que compare los
   bytes del dev bridge contra `genmsi simulate --json`. Creá tu feature doc
   odd/tasks/genmsi-fase4-app.md con esas specs, tus tareas y este mensaje como L1 verbatim,
   antes de la primera escritura de código. Antes de escribir el loop visual, cargá
   `agent-browser skills get core --full`. Entregable: el vertical slice funcionando, cargo
   test + clippy verdes, docs/app.md y docs/app-visual-loop.md, y evidencia visual por slice.
   Prohibido: crates/genmsi-cli/, crates/genmsi-mcp/, fixtures/, scripts/wine-oracle/ y los
   umbrales del oráculo.
```

- L2 (evidence, baseline): the assignment's baseline is `main` @ `9ae78af` (194 tests
  green, clippy clean, oracle 20 of 20). This session starts from `main` @ `69e6016` (the
  assignment document itself) on the branch `feat/phase4-app`.
- L3 (evidence, toolchain): `bun` 1.3.14, `node` 24.16.0, `agent-browser` installed,
  `cargo`/`rustc` present, `webkit2gtk-4.1` 2.54.1 and `gtk3` present, `tauri` CLI absent
  (`bunx @tauri-apps/cli`). `agent-browser skills get core --full` was loaded before the
  visual loop was written, as the assignment requires.
- L4 (decision, type generation): `ts-rs` 12 was chosen over `typify` after a probe
  (`/tmp/tsprobe`): it handles the borrowed shapes the engine's documents use
  (`&'a str`, `&'a [String]`, `&'a Entities`, `Option<&'a str>`) without a second copy of
  the document types, and `#[ts(as = "f64")]` maps the `u64` counters to the JSON `number`
  they are on the wire. The `ts` feature is optional, so the default workspace build does
  not change.
- L5 (scope note, additive API): the engine's published documents did not include the
  model's graph, so `genmsi-core` gains `genmsi.model/1` (T2). That is the additive public
  API the assignment allows; nothing else in `genmsi-core` or `genmsi-format` changes
  behaviour, and `crates/genmsi-cli/` and `crates/genmsi-mcp/` are not touched.
