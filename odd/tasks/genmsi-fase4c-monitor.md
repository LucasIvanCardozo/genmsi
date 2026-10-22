# Fase 4c: genmsi-app — monitor por reproducción y gráficos

Phase 4c session of genmsi-rs. Coordinator: "GenMSI-rs: sesión principal coordinadora"
(D12). The peer catalog was unavailable in this session (`session discovery is
unavailable`), so the assignment's open questions and the two engineering
questions they raised were put to the user directly; the answers are recorded
as `S8`, `S9` and `S10`. Scope: `crates/genmsi-app/` (frontend, `engine/`,
`src-tauri/`, `scripts/`), additive-only `crates/genmsi-core/` (the generated
TypeScript types of the trace document, which do not change the document
itself), `docs/` and `odd/tasks/`.

## Specs

- S1 (verbatim, user answer): "Series de tiempo + histogramas" — "Colas y facilities a lo
  largo del tiempo (contenido medio, utilización) + histogramas de las tables/QTables del
  modelo. Es lo que el reporte ya trae."
- S2 (verbatim, user answer): "Pestaña propia (recomendado)" — "Una pestaña `Monitor` junto a
  `Grafo` y `Reporte`: el grafo queda limpio y el monitor tiene su propio espacio con reloj y
  métricas."
- S3 (verbatim, user answer to the previous phase, still binding): "todo el texto visible en
  español neutral" (no voseo).
- S4 (D8, verbatim): "Frontend: React Flow, Zustand, Tailwind + shadcn/ui, Recharts (single
  charting library)."
- S5 (D8 / the coordinator's decision on the live monitor): this phase **replays a finished
  run**; the stepping engine (a live monitor with `new`/`step`/snapshot) is phase 4d, a later
  and separate unit whose exit gate is the oracle 20/20. **The engine does not change in this
  phase** and nothing of the stepping API is stubbed here.
- S6 (D2, verbatim): "only `genmsi-format` and `genmsi-core` may be dependencies of more than
  one crate. CLI, MCP and GUI are thin layers with no simulation logic."
- S7 (Phase 4 S5, still binding, verbatim): "algo importante a tener en cuenta aqui es que
  mientras se vaya haciendo la ui debe poderse revisar visualmente con agent bropwser o algo
  similar no?"
- S8 (assignment question 1, answered by the user): "Solo estado por bloque + reloj" — the
  replay animates the **state per block** and the clock; the transactions do **not** travel
  along the edges as dots. The trace carries the data for both; the second is an order of
  magnitude more code.
- S9 (assignment question 2, answered by the user): "Agrupar con aviso visible" — a table with
  more than 200 intervals is drawn grouped, and the label says how many intervals each bar
  covers. Up to 200 intervals the table is drawn as it is.
- S10 (the accumulated percentage, answered by the user): "Calcularlo en el frontend" — the
  histogram's accumulated curve is the running total of the **published frequencies**. No
  engine metric is recomputed: the average content and the utilization are not derived (the
  document does not carry them, and the phase 4b review removed a column that implied
  otherwise).
- S11 (assignment, engineering constraint, verbatim): "**The replay is a projection of existing
  documents, computed in the frontend.**" The run emits `genmsi.report/1` and the trace
  carries every event; the intermediate states are rebuilt by walking the trace (clock, queue
  contents, facility ownership). "Do not recompute a metric the engine computes (queue
  averages, utilization): render what the documents carry."
- S12 (assignment, engineering constraint, verbatim): "**Determinism is part of the feature**:
  replaying the same model with the same seed shows the same animation. A test pins that the
  replay state at event *i* is a pure function of the trace prefix."
- S13 (assignment, engineering constraint, verbatim): "**One adapter command, one new
  document.** The facade gains a `trace` document command (`genmsi.trace/1`, already published
  by the CLI) next to `model`, `lint` and `simulate`. Both backends (devBridge and Tauri) call
  it the same way; no third path."
- S14 (assignment, engineering constraint, verbatim): "**Performance guard**: the trace of a big
  model (`Parcial 6-11-23`, 10k+ events) must replay smoothly. If walking the whole trace per
  frame is slow, precompute the state timeline once per run (a derived index), not per
  animation frame. The UI must not freeze; say how you handled it."
- S15 (assignment, verbatim): "**The visual loop grows again** (S7): `check-visual-loop.sh`
  gains the monitor steps — run, open the monitor tab, screenshot the graph mid-replay,
  screenshot the charts — with distinct screenshots. It stays the single source of truth."
- S16 (assignment, verbatim): "**Tauri smoke check** stays as it is in 4b: the window exists,
  the invoke path answers; the Vite loop is the visual gate."

## Tasks

- T1 (S1–S16, deliverable) The feature document, the branch `feat/phase4c-monitor` and the
  Engram mirror. Commit: `docs(odd): phase 4c feature document for the monitor and the charts`.
- T2 (S13) The facade publishes the trace: `trace_document_bytes(path, seed, until, limit)`
  returns the `genmsi.trace/1` document (the engine's bytes, the CLI's contract), the session
  protocol gains the operation that traces the model **in memory**, the thin binary gains the
  `trace` subcommand and the desktop shell the `trace_model` command. The facade stays a thin
  client: no simulation logic. Tests: the facade's bytes are the CLI's bytes (byte for byte),
  the limit is honoured and a truncated trace says so. Commit:
  `feat(app): the facade publishes the trace document`.
- T3 (S13) The trace document's TypeScript types are **generated** (`ts-rs` on `TraceEvent` and
  `TraceDocument`): the document itself does not change, only the types the frontend imports.
  Commit: `feat(core): the generated types of the trace document`.
- T4 (S13) The adapter grows `trace(path, options)` and `traceCurrent(options)`: the interface,
  the dev bridge (the route that returns the engine's bytes) and the Tauri backend call the
  same facade. Commit: `feat(app): the adapter asks the engine for the trace`.
- T5 (S11, S12, S14) The replay is a pure module (`src/monitor/replay.ts`): the timeline (the
  trace's events with the block's node, the entity's name and the units), the state at any
  prefix of the trace, and the series the charts draw. Tests: the same trace gives the same
  timeline, the state at event *i* depends only on the prefix, the derived final content of
  every queue is the one the report publishes, and a truncated trace is marked as such.
  Commit: `feat(app): the replay is a projection of the trace`.
- T6 (S1, S2, S8, S11) The Monitor tab: its own space next to `Grafo` and `Reporte`, the run
  (report + trace), the clock, the playback controls, the canvas with the model's blocks and
  the activity of each one, and the panel with the state of the queues and the facilities at
  the replay instant. The trace's truncation is shown honestly. Commit:
  `feat(app): the Monitor tab replays the run`.
- T7 (S1, S4, S9, S10) The charts (Recharts, the only charting library): the time series of
  the queues and the facilities over the run, and the histogram of every table and QTable of
  the model with its intervals, its frequencies and the accumulated curve; a table with more
  than 200 intervals is grouped and the label says so. Commit:
  `feat(app): the time series and the histograms`.
- T8 (S7, S15) The visual loop grows: run, open the monitor, the replay mid-run, the charts —
  one distinct screenshot per step. Commit: `test(app): the visual loop of the monitor`.
- T9 (S1–S16, deliverable) `docs/app.md` and `docs/app-visual-loop.md` updated, and the close:
  `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, the
  frontend's own tests, the visual loop, and the evidence of every task. Commit:
  `docs(app): the monitor, the replay and the charts`.

## Decisions (engineering, from the assignment and from the phase's own reading)

- **The replay never recomputes an engine metric.** The clock, the content of the queues and
  the ownership of the facilities are *state*, rebuilt by walking the trace; the average
  content and the utilization are *metrics* the engine computes, and the report document does
  not carry them, so the app does not show them (the same rule the report view follows).
- **The trace is the only timeline.** The report carries the final state and the aggregates;
  the trace carries every event. The replay joins the two through the program block index
  the trace carries and the report's block table (`line`), which maps a program block to the
  model's node.
- **The entity's name comes from the engine when the engine says it.** The trace's `wait`
  events carry the resolved name of the facility or the storage the transaction waits for; the
  model document carries the literal name of the queue of a block. A name that cannot be
  resolved statically (an indirect `*name` operand) is shown as such instead of being
  guessed.
- **One index per run.** The timeline (the events with what they change) is computed once when
  the run finishes; the animation reads the index, it never walks the whole trace per frame.
- **The chart shows what the document carries.** The histogram's bars are the published
  frequencies; the accumulated curve is their running total; the intervals and the sample count
  are the published ones.

## Commits

| Task | Commit |
|---|---|
| T1 | `03d75df` `docs(odd): phase 4c feature document for the monitor and the charts` |
| T2 | `716906e` `feat(app): the facade publishes the trace document` |
| T3 | `36358f6` `feat(core): the generated types of the trace document` |
| T4 | `13730f3` `feat(app): the adapter asks the engine for the trace` |
| T5 | `7c21a1e` `feat(app): the replay is a projection of the trace`, `2683a74` `test(app): the series of the entities tells every instant the state changes` |
| T6 | `ea7c7fa` `feat(app): the Monitor tab replays the run` |
| T7 | `91b8dbb` `feat(app): the time series and the histograms` |
| T8 | `fc50d71` `test(app): the visual loop of the monitor` |
| T9 | the documentation and the close of this document |

## Log

- L1 (verbatim, first message):

```
FASE 4c — GenMSI-rs: genmsi-app (monitor por reproducción y gráficos). Sos la sesión de la
   Fase 4c del proyecto genmsi-rs. La coordinadora es "GenMSI-rs: sesión principal
   coordinadora"; las decisiones de producto se consultan, no se deciden.

   Leé primero odd/tasks/genmsi-fase4c-assignment.md: trae las tres decisiones ya respondidas
   (series de tiempo + histogramas, pestaña Monitor propia, reproducción de la traza en lugar
   de un motor pausable), el alcance exacto — el motor NO cambia en esta fase, 4d es una unidad
   futura — y las restricciones de ingeniería: el replay es una proyección de documentos
   existentes caminando la traza, nunca se recalcula una métrica del motor, un comando de
   fachada nuevo (genmsi.trace/1) para los dos backends, determinismo con test, y el loop
   visual extendido con los pasos del monitor. Creá tu feature doc
   odd/tasks/genmsi-fase4c-monitor.md con esas specs, tus tareas y este mensaje como L1
   verbatim, antes de la primera escritura de código. Entregable: la pestaña Monitor con
   replay y gráficos, determinismo comprobado, check-visual-loop.sh extendido con capturas
   distintas por paso, cargo test + clippy + frontend tests verdes, docs/app.md actualizado y
   evidencia por tarea. Prohibido: crates/genmsi-cli/, crates/genmsi-mcp/, fixtures/,
   scripts/ de la raíz, y cambiar lo que los documentos existentes emiten.
```

- L2 (evidence, baseline): the assignment's baseline is `main` @ `ee74c95` (241 tests green in
  36 suites, clippy clean, 19/19 frontend tests, oracle 20/20). This session starts from
  `main` @ `fa66029` (the assignment document) on the branch `feat/phase4c-monitor`.
- L3 (decision, the open questions): the peer catalog was unavailable in this session
  (`session discovery is unavailable`), so the assignment's two open questions were put to the
  user directly, with a third one that the assignment's own wording raised. The answers are
  `S8` (the replay animates the state per block and the clock; the transactions do not travel
  along the edges), `S9` (a table with more than 200 intervals is grouped and the label says
  so) and `S10` (the accumulated curve is the running total of the published frequencies).
- L4 (evidence, what the trace carries): the trace's records are
  `TraceEvent { time, xact, block, class, label, event, detail }` with the events `generate`,
  `enter`, `wait`, `leave` and `terminate`. A transaction that enters a queue block is
  enqueued there and leaves the block right away, so the content of a queue is the number of
  entries minus the number of departures; a transaction that seizes a facility records a
  `wait` (with the facility's name) when the facility is busy and leaves the block (without a
  wait) when it takes it. The program block index the trace carries is the report's `line`
  minus one, and the report's block table carries the model's node of every program block, so
  the trace reaches the graph.
- L5 (evidence, the documents): `genmsi.report/1` carries the queues (`content`, `max`,
  `entries`, `zero_entries`, `total_time`, `area`, `retry`), the facilities (`entries`,
  `busy_time`, `busy`, `available`, `retry`, `inter`) and the tables (`first`, `width`,
  `intervals`, `samples`, `frequencies`, `overflow`). It does **not** carry the average
  content nor the utilization: the phase 4b review removed the report view's column that
  implied otherwise. The charts therefore show the state over time (from the trace) and the
  published frequencies, never a derived metric.
- L6 (decision, the trace's limit): the CLI's `trace` keeps 100 records by default; the
  facade asks for a high limit (the app asks for 200000) and the document's `truncated` flag
  says when the bound was reached. The UI says so instead of implying that the whole run was
  replayed.
- L7 (decision, one run, two documents): the monitor's run asks the engine for the report and
  for the trace of the same model and the same seed. The two runs are deterministic, so the
  trace belongs to the report the charts read.
- L8 (limits, recorded): the entity's name is resolved statically when the model's property
  is a literal (`EntityUse`); an indirect operand (`*name`) and a placeholder (`[name]`) are
  not resolved by the frontend, and the monitor says so instead of guessing. The number of
  units of a queue is the model's `Cantidad` property, which is a plain number in every
  reference model.
- L9 (evidence, the trace's grammar): the trace records `generate`, `enter`, `wait`, `leave`
  and `terminate`. The engine counts a block's transactions the same way the trace records
  them (an entry on `enter`, a departure on `leave` and on `terminate`), with one exception:
  a generator block holds *the generator itself* in the engine's count, so it is not part
  of the comparison between the projection and the report.
- L10 (evidence, the fidelity of the projection): the replay of the reference model and of
  the big model (`Parcial 6-11-23`, 82 325 events) ends in exactly the state the report
  publishes: the content of every queue and the transactions of every block (the generator
  blocks apart). The test compares both, so a divergence between the projection and the
  engine would fail the suite and not the picture.
- L11 (decision, the series): the series of the charts is the **exact** history of the
  entity (the instants its value changes), not a sample of the trace. The sample was tried
  first and it lied: a queue that fills and empties between two samples was invisible, and a
  facility that is free for a moment looked busy. The chart of the facilities was a comb of
  random teeth. With the exact series the queue's sawtooth and the facility's free moments
  are the ones the run had.
- L12 (fix, the desktop shell): the shell's document commands (`load_model`, `lint_model`,
  `simulate_model`) answer with the engine's bytes (a JSON string) and the frontend's Tauri
  backend took them for an object: the graph and the report of the desktop shell were a
  string where the frontend expected a document. The Vite loop (the visual gate) hid it.
  The backend now parses the bytes, as the session path always did.
- L13 (evidence, the visual loop): the loop grew with the monitor's steps (the run, the
  monitor tab, the replay at the middle of the run and the charts) and it found three
  things, all fixed: the canvas framed the containers as well as the flow, so the picture
  shrank until nothing could be read; the containers were drawn over the blocks they hold;
  and the axis of the time series showed the times with three decimals (the report's rule)
  where an axis needs a second. The replay is paused and put at the middle of the run by
  the loop, so the picture is the same on every run.
- L14 (verification, close): `cargo test --workspace` is green (245 tests) and
  `cargo clippy --workspace --all-targets -- -D warnings` is clean. The frontend's own
  checks are green too: `bun test` 30 of 30 (the dev bridge's bytes against the engine's
  for the report *and* the trace, the deterministic layout, the property editors and the
  replay) and `bunx tsc --noEmit` clean. The visual loop ran end to end with fourteen
  screenshots, and the desktop shell's smoke check passed (the window, the capture with
  1364 distinct colours and the `invoke` path).
- L15 (delivery, slices): the phase's commits are eleven. Three are bigger than the ~400
  authored lines and the reason is recorded instead of hidden: the replay module (the
  projection and its tests), the Monitor tab (the canvas, the controls and the state panel)
  and the charts (the series, the histograms and the dependency). They are one unit each
  — the projection without the tab is not a feature, and the charts without the series are
  not charts — and they are split by the behaviour they add, not by the file they touch.
  The unit that could be delegated was the facade's trace command (T2), and it was: a
  writer did it in the background while this session wrote the frontend, with disjoint
  surfaces.
- L16 (risk): the change adds a document to the app's own protocol (the trace) and a
  dependency (Recharts, the only charting library of D8), so item 3 applies to the protocol
  and item 5 to the dependency. Item 3 is covered by the parity: the facade's bytes and the
  dev bridge's bytes are the ones `genmsi trace --json` prints, byte for byte, and the
  document the engine publishes does not change. Item 5 is covered by the version of the
  dependency and by the lock file. Item 6 does not apply: the replay's logic has its own
  tests (the determinism, the purity of the prefix and the fidelity against the report),
  and the visual loop is the gate of the picture. Risk: item 3 (the app's protocol grows
  two operations) and item 5 (a new dependency).
- L17 (native review): the RDD switch is on, so the phase was assessed and reviewed. The
  assessment of the whole branch returned `high_risk` (the shell script of the visual loop
  is a `process_boundary` signal), so the review was started. The capture could **not** be
  made: the facade refuses the provider's own collect bindings as stale
  (`collectBinding is missing or stale for current STATUS`, and the group
  `collectBindings must be the complete distinct current reviewer group in exact provider
  order`), with the bindings the very same STATUS had just published — verified byte for
  byte against the published canonical form. The same wall stopped the phase 4b review
  (the native CLI reports the authority as needing a repair, which the facade does not
  support). The reviews of the work units say the same: the replay module (the projection
  and its tests) is `under_budget` and not due, and the visual loop's unit is due because
  of the same shell signal but its capture is refused in the same way. Two lineages stay
  open (`review-7372e6a7797e7ad1` over the branch and `review-eb06d5bcad7291e5` over the
  visual loop's unit): abandoning them is a decision of the maintainer, not of this
  session. **The review is therefore not an approval, and the phase's proof is the
  verification suite** (245 tests, clippy clean, 30 frontend tests, the visual loop with
  fourteen screenshots and the desktop shell's smoke check).
