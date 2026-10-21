# Fase 4 (asignación): genmsi-app — vertical slice

Coordinator assignment for the first Phase 4 session. This file is the coordinator's
input, not the phase's feature document: the session creates
`odd/tasks/genmsi-fase4-app.md` with these specs as `S1..Sn`, its own tasks and log, and a
verbatim `L1` holding the user's message that opens the phase. Product questions come to
the coordinator (D12); the ones below are already answered, so nothing here should be
re-decided in the session.

Engine baseline at assignment time: `main` @ `9ae78af`, 194 tests green, clippy clean,
oracle 20 of 20, CLI complete, MCP server complete.

## Scope of this phase

Per the user's answer (`S1`), Phase 4 opens with a **vertical slice**, not with workflow
parity: open a `.gmdl`, see the graph and the block properties, simulate it, and read the
report (blocks, facilities, queues, storages) from the engine's JSON. Editing nodes, the
live monitor and the charts are **out of scope for this closure** and become Phase 4b/4c
with their own assignments. Do not start them; do not stub them either.

## Specs

- S1 (verbatim, user answer): "Vertical slice primero (abrir + simular + reporte)" — "Abrir
  un .gmdl, ver el grafo y las propiedades, simular y leer reporte/cola/facility desde el
  JSON del motor. La edición de nodos viene después, sobre algo que ya se ve y se prueba."
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

## Engineering constraints (coordinator decisions, not product questions)

- **The frontend must run standalone under Vite, without Tauri.** `S5` is unachievable
  through a native webview: `agent-browser` drives a browser, so the app must be a Vite dev
  server on `localhost` that `agent-browser` can open, click and screenshot. The Tauri shell
  wraps the same frontend afterwards.
- **One engine adapter, two backends.** Define a single frontend module that exposes the
  engine operations the slice needs (load/validate, simulate, report). Behind it, exactly
  two implementations: `devBridge` (Vite dev middleware that spawns the `genmsi` binary with
  `--json` and returns its stdout verbatim) and `tauri` (a Tauri command calling the same
  core functions). Never a third path, and never logic in either backend: they move
  documents (S9), they do not build them. This is the defect phase 3 had to fix —
  two implementations of the same document — do not repeat it.
- **The dev bridge returns the CLI's bytes.** A test must compare the dev bridge's document
  for a reference model against `genmsi simulate --json` (and `lint --json`), byte for byte.
  If they can drift, the visual loop is testing something the release does not ship.
- **Types come from the Rust schema** (S7): generate them (`ts-rs` or `typify`) and commit
  the generator invocation; do not hand-write the document types.
- **The visual loop is part of the definition of done of every slice**: build, open, click,
  screenshot, look, fix. Document the exact commands in `docs/app-visual-loop.md`, including
  how to start the dev server in the background, how to wait for it to answer, and how to
  kill it. Load `agent-browser skills get core --full` before writing the loop; it is
  installed. Screenshots stay in `/tmp` (S4).
- **Do not slow down or break the existing workspace.** `cargo test --workspace` and
  `cargo clippy --workspace --all-targets -- -D warnings` must stay green and fast; if the
  Tauri crate would pull a heavy dependency tree into the default build, keep it out of the
  default members (or out of the workspace) and say so in the Log. Adding Node dependencies
  is allowed and is the item-5 surface: pin them in the lockfile.
- Environment facts: `bun` and `agent-browser` are installed, `webkit2gtk-4.1` 2.54.1 and
  `gtk3` are present, `tauri` CLI is not (use `bunx @tauri-apps/cli`).

## Allowed edit paths

- `crates/genmsi-app/` (the whole crate: frontend, `src-tauri/`, its own configs).
- `docs/` (new documents; `DECISIONS.md` only to append a new entry at the end).
- `odd/tasks/`.
- `Cargo.toml` (workspace `members`/`exclude`, and nothing else).
- Additive public API in `crates/genmsi-core/` or `crates/genmsi-format/` only if a schema
  export is missing (report it in the Log).

Prohibited: `crates/genmsi-cli/`, `crates/genmsi-mcp/` (behaviour and documents), `fixtures/`,
`scripts/wine-oracle/`, the comparison thresholds in `crates/genmsi-core/src/oracle.rs`.

## Reading order

1. `docs/DECISIONS.md` D1–D15.
2. `docs/ROADMAP.md` Phase 4 and `crates/genmsi-app/README.md`.
3. `docs/cli.md` — the `--json` documents the slice renders (`genmsi.report/1` and the
   entity tables inside it) and the exit codes.
4. `crates/genmsi-core/src/report.rs` and `crates/genmsi-core/src/documents.rs` — what the
   JSON already contains; do not recompute a metric in the UI.
5. `docs/format-and-engine.md` — the formats, the model semantic model and the containers.
6. `fixtures/README.md` — which models are the reference ones.

## Known traps — do not rediscover

- The report's `genmsi.report/1` body already carries blocks, facilities, queues, storages,
  tables and savevalues: the UI renders, it does not compute.
- Five reference models are refused by the engine by design (`invalid_model`, `no TERMINATE`
  and the like). The UI must show those as an error state, not as an empty model.
- The original prints zeroed queue averages for queues that end empty; those are baseline
  artefacts, not something the UI should "fix".
- `AGENTS.md`/`CLAUDE.md` style files do not exist here; the phase document is the source of
  truth for this phase.

## Deliverable

A vertical slice that opens a `.gmdl` from `fixtures/models/`, renders the block graph
read-only with the properties of the selected block, runs the simulation, and shows the
report tables; all text in Spanish (S3), the design modern (S2), the frontend runnable under
Vite (S5), both entry points (Vite and Tauri) sharing one adapter, tests green, `docs/app.md`
(design and structure) and `docs/app-visual-loop.md` (the repeatable `agent-browser` loop),
and the feature document with the evidence per task.

## Open questions (ask the coordinator; do not decide)

1. Where the graph layout comes from in this slice: the `.gmdl` carries canvas geometry
   (`Block::layout`), so rendering it is possible today; re-running a layout engine is a
   Phase 4b decision (it belongs to editing), not this one.
2. Whether the report view shows every table the document carries or a curated subset, if
   the volume turns out to be unreadable in a first design.
