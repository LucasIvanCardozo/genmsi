# Fase 2: genmsi-cli

Phase 2 session for genmsi-rs. Coordinator: "GenMSI-rs: sesión principal coordinadora"
(D12); product decisions are consulted, not decided here. Scope: `crates/genmsi-cli/`,
`docs/` (new documents; `DECISIONS.md` only to append an entry), `odd/tasks/`.

## Specs

- S1 (verbatim): "lo que si quiero es que minimo en la nueva se pueda hacer lo que se
  hace en la version actual (pero mejor en terminos de optimizacion y codigo)"
- S2 (verbatim): "quiero ademas una buena filosofia de escalabilidad, buena estructura
  de carpetas ordenadas y reuzabilidad!"
- S3 (ROADMAP Phase 2, verbatim): "- Subcommands: `lint`, `simulate` (+ `--json`),
  `trace`. - Deterministic output suitable for snapshot tests."
- S4 (ROADMAP exit criteria, verbatim): "CLI runs every example model; `--json` output
  schema documented."
- S5 (D2, verbatim): "only `genmsi-format` and `genmsi-core` may be dependencies of
  more than one crate. CLI, MCP and GUI are thin layers with no simulation logic."
- S6 (D11, verbatim): "Goal is minimum parity with the original, with better code
  quality — not a maximally professional system. Every tool addition must justify
  itself against that goal."
- S7 (D6, verbatim): "`.gmdl` models from the original must open and run with results
  equivalent within statistical tolerance."
- S8 (D12, verbatim): "Each phase runs in a dedicated session that receives its
  assignment (objective, allowed edit paths, S# specs, deliverable) from the
  coordinator and consults it instead of making product decisions."

## Tasks

- T1 (S2, S5) CLI skeleton and layer split: `main.rs` stays a thin dispatcher; args in
  `cli.rs`; one module per subcommand under `commands/`; shared rendering in `output.rs`;
  exit-code mapping in `error.rs`; loading in `model.rs`. No simulation logic in the CLI.
  Commit: `feat(cli): subcommand skeleton with layered modules`.
- T2 (T1) Shared run options (`--seed`, `--until`, `--json`) and model loading that keeps
  the engine's line/node context. Commit: same as T1.
- T3 (S1, S3) `lint`: structural validation, runnable validation and compilation, with a
  readable report, `--json` and exit codes 1/3. Commit: `feat(cli): lint subcommand`.
- T4 (S1, S3) `simulate`: seeded run plus the GPSS report from `genmsi_core::render`.
  Commit: `feat(cli): simulate subcommand`.
- T5 (S3) `simulate --json`: versioned envelope over the engine's typed document. Commit:
  same as T4.
- T6 (S3) `trace`: structured event trace with `--limit`, human lines and `--json`.
  Commit: `feat(cli): trace subcommand`.
- T7 (S3, coordinator decision) `run-all <dir>`: one line per model, `--json`, exit 1 when
  any model fails. Commit: `feat(cli): run-all subcommand`.
- T8 (S3) One insta snapshot per subcommand, plus a determinism test (same seed, same
  bytes). Commit: `test(cli): snapshot and determinism tests`.
- T9 (S4) CLI integration test: all 25 fixture models through the binary; the five the
  original refuses fail with the same error class and a non-zero exit code. Commit:
  same as T8.
- T10 (S4) `docs/cli.md`: commands, options, exit codes and the `--json` schema. Commit:
  `docs(cli): document commands, exit codes and the json schema`.
- T11 (D12) Append `DECISIONS.md` D13 recording the CLI contract; update this doc with the
  commit holding each task. Commit: same as T10.
- T12 (deliverable) Close: `cargo test` and `cargo clippy --all-targets -- -D warnings`
  green for the workspace; no task left pending.

Commit slices: the CLI is new code in one crate; slices follow subcommands so each commit
is one reviewable behaviour (~400 authored lines or less) with its tests. The branch is
`feat/phase2-cli`; push, PR and merge stay with the user.

## CLI contract (answered by the coordinator, D12)

The four assignment questions were answered; these answers are the contract this phase
implements and `docs/cli.md` publishes.

1. **`--json` schema**: the CLI emits a *versioned envelope*: a `schema` string followed by
   the body, `{"schema": "genmsi.report/1", ...}`. One schema id per document
   (`genmsi.lint/1`, `genmsi.report/1`, `genmsi.trace/1`, `genmsi.run-all/1`). The fields
   inside `genmsi.report/1` are the engine's own report document (no renaming). Stability
   promise: within a major schema version (`.../1`) changes are additive only — new fields
   may appear, existing fields keep their name, type and meaning; a breaking change bumps
   the number.
2. **Exit codes**: `0` success; `1` the model does not load, does not validate or does not
   run (includes the five models the original refuses); `2` usage error (reserved for
   clap's own argument errors); `3` `lint` found compiler diagnostics on a model that is
   still runnable. No other code is produced.
3. **`trace`**: no filters. `--limit <N>` bounds the number of records the engine keeps
   (default 100) and the human output ends with a truncation note when the limit was hit.
4. **`run-all <dir>`**: exists. One line per `.gmdl` file of a single directory level,
   sorted by file name; `--json` emits one object per model; exit `1` when any model fails
   (which is the expected outcome on `fixtures/models/`, where five models are refused).

## Commit map

Slices are one reviewable behaviour each; every commit compiles and its tests pass
on its own. The crate's first appearance is a compile unit, so the slices are cut
by subcommand rather than by file.

| Task | Commit subject | Hash |
|---|---|---|
| T1, T2, T3 | `feat(cli): lint subcommand on a layered skeleton` | `4d76d55` |
| T4, T5 | `feat(cli): simulate subcommand with the versioned report` | `a3321bc` |
| T6 | `feat(cli): trace subcommand with a record limit` | `eccac1b` |
| T7 | `feat(cli): run-all subcommand for directory sweeps` | `2689f80` |
| T8, T9 | `test(cli): every reference model, snapshots and determinism` | `0948087` |
| T10, T11 | `docs(cli): commands, exit codes and the json schema` | `c376097` |

Commit slice boundary: the Phase 2 slice is 7 commits on `feat/phase2-cli` (~2.2k
authored lines, lock files excluded). The phase turns a placeholder binary into a
CLI, so the work is one compile unit; the slices follow subcommands and the docs
rather than the ~400-line guideline, and every slice compiles and passes its own
tests. Native review applies to the work-unit commits, not to the accumulated
branch.

## Log

- L1 (verbatim, first message):

```
FASE 2 — GenMSI-rs: genmsi-cli. Sos la sesión de la Fase 2 del proyecto genmsi-rs.
   La coordinadora es "GenMSI-rs: sesión principal coordinadora"; las decisiones de
   producto se consultan, no se deciden.

   Leé primero odd/tasks/genmsi-fase2-assignment.md (objetivo, rutas permitidas, specs
   S1-S8, orden de lectura, trampas conocidas del oráculo y las preguntas que debés
   consultar). Creá tu propio feature doc odd/tasks/genmsi-fase2-cli.md con esas specs,
   tu lista de tareas y este mensaje como L1 verbatim, antes de la primera escritura de
   código. Entregable: cargo test en verde + clippy limpio, los 25 modelos corriendo por
   el CLI (los 5 que el original rechaza fallando con la misma clase de error y exit code
   distinto de cero), un snapshot test por subcomando y docs/cli.md con el esquema --json.
   Prohibido: tocar crates/genmsi-mcp/, crates/genmsi-app/, fixtures/, scripts/wine-oracle/
   y los umbrales de comparación.
```

- L2 (evidence, engine baseline at phase start): `main` @ `ca71ed1`; `cargo test
  --workspace` green and `cargo clippy --workspace --all-targets -- -D warnings` clean
  before any CLI change.
- L3 (evidence, natural termination): with `SimConfig::default()` (no `stop_time`) the 20
  runnable models all end with `EndReason::Terminated` in under 60 ms each, at their own
  `END_TIME` (3600 s for most, 1800 s `ejercicio_3`, 6097 s `TP6_2`, 28800 s `TP10`,
  36000 s `Parcial 6-11-23`). So `simulate` needs no mandatory horizon: `--until` is an
  optional override, not the default path.
- L4 (evidence, the five refusals through the engine API): `Parcial 8-11-21` fails at load
  (`QTable: invalid block: definition name "tiempo demora"`); `Problema 2__puenteLevadizo`,
  `TP6__Clase 6_0_TestGate` and `TP9__hack` fail with `no TERMINATE block with operand A
  and no CLOCK block`; `TP9__tp9` fails with `Unlink: the operand B must not be null`. All
  five are engine errors, so the CLI maps them to one non-zero exit code without
  replicating the original's crash for `TP9__hack`.
- L5 (decision, coordinator, D12 — verbatim answer tokens of the four assignment
  questions): `--json` = "Envoltorio CLI versionado" (`{"schema":"genmsi.report/1", ...}`);
  exit codes = "0 / 1 / 2 / 3" with `0` "modelo válido y limpio", `1` "el modelo no carga,
  no valida o no corre", `2` "error de uso", `3` "lint con diagnósticos del compilador,
  modelo ejecutable"; `trace` = "Sin filtros, con --limit"; `run-all` = "run-all <dir>" with
  the human `OK/FAIL` lines of the preview plus `--json` with one object per model.
  Recorded as the CLI contract above and as `DECISIONS.md` D13 (T11).
- L6 (evidence, exit code 3 reachability): the engine's `Program::diagnostics` ("non-fatal
  observations made while compiling") is the only compile-time diagnostic channel and no
  reference model produces one; the only diagnostic the engine emits today is the runtime
  `TABULATE <name> has no TABLE definition` (`crates/genmsi-core/src/sim.rs`). So `lint`
  maps a non-empty `Program::diagnostics` to exit `3`, the mapping is unit-tested, and
  `docs/cli.md` states that no reference model reaches it. Adding compiler diagnostics is
  out of scope for this phase (it would change engine output) and would need the
  coordinator.
- L7 (evidence, RED and differential check): on the base commit (`ca71ed1`) the CLI
  is the placeholder `main.rs`: `lint` and `simulate` exit 2 with "not implemented
  yet" and `trace`/`run-all` are unknown subcommands, so every test of this phase is
  RED there. After the change, the GPSS text report of `TP4__supermercado`,
  `TP10__tp10` and `monorriel__monorriel` is byte-identical to the base commit's
  (same seed) and the `genmsi.report/1` body is field-identical to the base
  `to_json`, with `schema` as the only added field: the `EndReason::as_str`
  refactor changed no output.
- L8 (observation, out of scope): `genmsi_core::report::operands` prints Rust `Debug`
  for a gate condition (`GATEL  Logic { switch: ..., state: true }`) instead of GPSS
  operands. That is Phase 1 report cosmetics, not a Phase 2 contract; the snapshot
  pins today's output so a later fix shows up as a diff.
- L9 (decision, S6/D11): six commit slices by subcommand (see the commit map), each
  with its tests. `cargo build` and `cargo test -p genmsi-cli` were run on every
  intermediate stage of the slice reconstruction.
- L10 (close): `cargo test --workspace` green, `cargo clippy --workspace
  --all-targets -- -D warnings` clean, 25 reference models through the binary (20
  reports, 5 refusals with exit 1), one snapshot per subcommand, `docs/cli.md` with
  the four schema ids, and `DECISIONS.md` D13 appended. Push, PR and merge stay
  with the user.
- L11 (evidence, commit map): the six slices are `4d76d55` (skeleton + lint),
  `a3321bc` (simulate), `eccac1b` (trace), `2689f80` (run-all), `0948087` (tests),
  `c376097` (docs). Every slice was built and tested before committing; the branch
  is `feat/phase2-cli`, based on `ca71ed1`.
