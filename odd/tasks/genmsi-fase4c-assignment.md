# Fase 4c (asignación): genmsi-app — monitor por reproducción y gráficos

Coordinator assignment for the third Phase 4 session. Coordinator's input, not the phase's
feature document: the session creates `odd/tasks/genmsi-fase4c-monitor.md` with these specs
as `S1..Sn`, its own tasks and log, and a verbatim `L1` holding the user's message that
opens the phase.

Baseline at assignment time: `main` @ `ee74c95`, 241 tests green (36 suites), clippy clean,
19/19 frontend tests, oracle 20/20, editing merged (byte-identical save, undo/redo,
re-layout).

## Scope of this phase

A Monitor tab next to `Grafo` and `Reporte` (S3): replaying a finished run over the graph —
clock, animated blocks, the current state of queues and facilities at replay time — and the
charts of the run (Recharts, S4 of D8), with the run's data coming from documents that
already exist.

**The engine does not change in this phase.** The stepping API (`new`/`step`/snapshot) that
a true live monitor would need is Phase 4d, planned separately; it is not started and not
stubbed here.

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
- S5 (verbatim, user decision on the live monitor): "entiendo que tu recomendacion es la
  mejor pero la primera opcion no nos daria mas flexibilidad y mejor calidad de vida?" —
  answered by the coordinator with a two-step plan the user accepted: this phase replays a
  finished run; the stepping engine (4d) is a later, separate unit with the oracle 20/20 as
  its exit gate.
- S6 (D2, verbatim): "only `genmsi-format` and `genmsi-core` may be dependencies of more than
  one crate. CLI, MCP and GUI are thin layers with no simulation logic."
- S7 (Phase 4 S5, still binding, verbatim): "algo importante a tener en cuenta aqui es que
  mientras se vaya haciendo la ui debe poderse revisar visualmente con agent bropwser o algo
  similar no?"

## Engineering constraints (coordinator decisions)

- **The replay is a projection of existing documents, computed in the frontend.** The run
  emits `genmsi.report/1` (the final state) and the trace already carries every event
  (`TraceEvent { time, xact, block, class, label, event, detail }`). Rebuild the intermediate
  states by walking the trace: clock, queue contents, facility ownership. Do not recompute a
  metric the engine computes (queue averages, utilization): render what the documents carry.
  If the trace turns out to miss something the replay needs (e.g. queue content at enter
  time), that is a coordinator question, not a TypeScript workaround.
- **One adapter command, one new document.** The facade gains a `trace` document command
  (`genmsi.trace/1`, already published by the CLI) next to `model`, `lint` and `simulate`.
  Both backends (devBridge and Tauri) call it the same way; no third path.
- **Determinism is part of the feature**: replaying the same model with the same seed shows
  the same animation. A test pins that the replay state at event *i* is a pure function of
  the trace prefix.
- **Charts read the report document**: time series over the tables the model declares and
  the entity summaries the report carries; histogram of a table with its intervals,
  frequencies and accumulated percentages. No metric is recomputed in TypeScript (the same
  rule the report view already follows — the review of phase 4 removed one column that
  implied otherwise).
- **Performance guard**: the trace of a big model (`Parcial 6-11-23`, 10k+ events) must
  replay smoothly. If walking the whole trace per frame is slow, precompute the state
  timeline once per run (a derived index), not per animation frame. The UI must not freeze;
  say how you handled it.
- **The visual loop grows again** (S7): `check-visual-loop.sh` gains the monitor steps —
  run, open the monitor tab, screenshot the graph mid-replay, screenshot the charts — with
  distinct screenshots. It stays the single source of truth.
- **Tauri smoke check** stays as it is in 4b: the window exists, the invoke path answers;
  the Vite loop is the visual gate.

## Allowed edit paths

- `crates/genmsi-app/` (frontend, `engine`, `scripts`, tests, `src-tauri`).
- `crates/genmsi-core/` and `crates/genmsi-format/`: additive only, and only if something the
  replay or the charts need is missing from a published document. A document shape change is
  a coordinator question first.
- `docs/` (new documents; `DECISIONS.md` only to append an entry at the end).
- `odd/tasks/`.

Prohibited: `crates/genmsi-cli/`, `crates/genmsi-mcp/`, `fixtures/`, `scripts/` at the repo
root, the oracle thresholds, and changes to what the existing documents emit for a run.

## Reading order

1. `docs/DECISIONS.md` D1–D15.
2. `docs/app.md` and `docs/app-visual-loop.md`.
3. `crates/genmsi-core/src/sim.rs` `TraceEvent` and `genmsi.trace/1` (`documents.rs`) — what
   an event carries, what the CLI already publishes.
4. `crates/genmsi-core/src/report.rs` — the tables section of `genmsi.report/1` (intervals,
   frequencies, accumulated percentages) that the histograms render.
5. `odd/tasks/genmsi-fase4b-edicion.md` — the review that closed the previous phase, and the
   store/session shape the monitor plugs into.

## Known traps — do not rediscover

- The trace has a limit (`--limit`, default 100 records in the CLI): a replay of a big model
  needs the facade to ask for a high limit, and the UI must show honestly when the trace is
  truncated (`truncated` is already in the document) instead of implying it replayed
  everything.
- Queue averages in the documents are run aggregates, not instants: the replay's "content at
  time t" comes from walking the events, never from dividing the aggregate.
- The original prints zeroed queue averages for queues that end empty; the charts render the
  document's numbers as they are.
- `GenMSI` titles and model captions are user text; do not transliterate them.

## Deliverable

The Monitor tab: run, replay with clock and animated state over the graph or its own canvas
(S2), the current queue/facility state at the replay instant, time-series and histogram
charts from the report document, deterministic replays, `check-visual-loop.sh` extended with
distinct screenshots per step, `docs/app.md` updated, feature document with per-task
evidence.

## Open questions (ask the coordinator; do not decide)

1. Whether the replay also moves the transactions visually (dots travelling along the edges)
   or only animates per-block state and the clock. The trace has the data for both; the
   second is one order of magnitude less code.
2. Whether the charts of a table with 1000+ intervals render as-is or the UI buckets them
   (bucketing is a presentation decision, but it changes what the user reads).
