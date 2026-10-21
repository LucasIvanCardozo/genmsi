# Fase 2 (asignación): genmsi-cli

Coordinator assignment for the Phase 2 session. This file is the coordinator's
input, not the phase's feature document: the phase session creates
`odd/tasks/genmsi-fase2-cli.md` with these specs as `S1..S8`, its own `S#`/tasks/log,
and a verbatim `L1` holding the user's message that opens the phase. Product
questions go to the coordinator (D12), never decided here.

Engine baseline at assignment time: `main` @ `3c20f99`, 101 tests green, clippy clean,
oracle 20 of 20 models matching.

## Assignment

- Objective (ROADMAP Phase 2): the engine is usable by humans, scripts and CI.
- Allowed edit paths: `crates/genmsi-cli/`, `docs/` (new documents; `DECISIONS.md`
  only to append a new entry at the end), `odd/tasks/`.
- Additive-only review path: if the CLI needs a capability the engine already has
  internally, add the public API (`pub fn`/`pub struct`/export) in
  `crates/genmsi-core/` or `crates/genmsi-format/`. Never change simulation
  semantics, and record in the Log what was added there. Anything that changes what
  a model produces goes through the coordinator as a question first.
- Prohibited: `crates/genmsi-mcp/`, `crates/genmsi-app/`, `fixtures/`
  (`models` and `outputs` are read-only oracle material), `scripts/wine-oracle/`,
  and the comparison thresholds in `crates/genmsi-core/src/oracle.rs`.
- Deliverable: `cargo test` green and clippy clean for the workspace; every model in
  `fixtures/models/` runnable through the CLI with the five refused models failing
  with the same error class and a non-zero exit code; a snapshot test per
  subcommand; `docs/cli.md` documenting the `--json` schema.

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

## Reading order

1. `docs/DECISIONS.md` D1–D12.
2. `docs/ROADMAP.md` Phase 2.
3. `docs/format-and-engine.md` and `crates/genmsi-core/src/lib.rs` (the API the CLI
   wraps).
4. `crates/genmsi-cli/src/main.rs` (placeholder) and
   `crates/genmsi-core/examples/simulate.rs` (the reference invocation).
5. `docs/oracle-comparison.md`, sections "Baseline artifacts" and "Single-sample
   noise on small blocks".

## Known traps — do not rediscover

- `TRANSFER` sends its `Prob` share to `Salida1`; the GPSS export prints
  `TRANSFER P,<Salida2>,<Salida1>`. Pinned by
  `crates/genmsi-core/tests/transfer_branch.rs`; do not invert it to chase one metric.
- The original prints zeroed queue averages for every queue that ends empty, whatever
  its traffic. Those metrics never match and stay counted as mismatches.
- `fixtures/outputs/*.gpss.txt` is truncated for `Parcial 6-11-23`, `TP10`, `TP3`,
  `TP4`, `TP5`, `TP6_2` and `TP8`; the authoritative expansion is the block table of
  the matching `.report.txt`.
- The comparison rule is 20% for counts, 35% for times and occupancies, or a baseline
  inside the 95% CI of the replication mean. Never relax it.

## Product questions for the coordinator

Anything not written above: the exact `--json` field naming and schema stability
promise, exit-code conventions beyond non-zero-on-failure, whether `trace` needs
filters, and whether the CLI grows a `run-all` convenience command at all. Ask before
building.
