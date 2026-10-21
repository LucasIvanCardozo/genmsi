# Decision log

Every architecture decision made for genmsi lives here. Phase sessions read
this file and do not re-litigate settled decisions; a decision change requires
a new entry (or an explicit supersede) in this file.

## D1 — Language: Rust

One engine feeds CLI, MCP server and GUI without duplicated logic; a static
single binary lets agents run `genmsi mcp` with nothing else installed; memory
safety without a runtime. Go was the runner-up (faster to write) but lost on
GUI maturity (Wails vs Tauri) and single-binary distribution.

## D2 — Structure: Cargo workspace with layered crates

`genmsi-format`, `genmsi-core`, `genmsi-cli`, `genmsi-mcp`, `genmsi-app`.
Layer rule: only `genmsi-format` and `genmsi-core` may be dependencies of more
than one crate. CLI, MCP and GUI are thin layers with no simulation logic.
Each crate compiles and is tested independently.

## D3 — Model format: DOM parsing (roxmltree), not streaming

The largest input is `gClasses.xml` at 722 KB; it fits in memory trivially.
Streaming (quick-xml) adds complexity with no benefit at this size. Revisit
only if multi-hundred-MB inputs ever appear.

## D4 — Expression language: pest grammar

The GPSS-style expression language (SNA, `FN:MediaClientes`, ...) is defined
in a `.pest` file that is simultaneously grammar, documentation and parser.
Parse errors come with line/column, which agents and humans need.

## D5 — RNG: rand + rand_chacha, seeded per run

Same seed → bit-for-bit reproducible run. Basis for replications, confidence
intervals, and the semantic oracle regression tests. Compatibility with the
original is statistical, not bit-exact.

## D6 — Fidelity goal: semantic equivalence with statistical tolerance

`.gmdl` models from the original must open and run with results equivalent
within statistical tolerance. Replicating Delphi/GPSS internal RNG and
floating-point quirks is out of scope.

## D7 — Agent integration: MCP first-class via rmcp

`genmsi mcp` runs an rmcp stdio server with tools: `simulate`, `lint`,
`get_trace`, `explain_model`. The tool surface is a thin adapter over the same
API the CLI uses.

## D8 — GUI: Tauri 2 + React 19 + React Flow (Phase 4)

The node editor needs a best-in-class graph component; native GUI toolkits
(egui, Slint) would require hand-building it. Frontend: React Flow, Zustand,
Tailwind + shadcn/ui, Recharts (single charting library). Shared types are
generated from the Rust schema (ts-rs or typify). GUI work starts only after
the engine and format are frozen by tests.

## D9 — Oracle: Wine verifies, published GPSS semantics lead

The original `genmsi.exe` runs under Wine to *verify* equivalence. Semantic
ground truth comes from published GPSS documentation (GPSS/H, GPSS World);
black-box Wine deduction is the slow path, used only to resolve discrepancies.
The engine implements first the block subset actually used by the project's
models (inventory script, Phase 0).

## D10 — Verification: proptest for invariants, insta for snapshots

Property tests pin engine invariants (a transaction always ends in a valid
state, entity counts never negative). Snapshot tests pin full reports and
traces so regressions show up in the diff.

## D11 — Simplicity guard

Goal is minimum parity with the original, with better code quality — not a
maximally professional system. Every tool addition must justify itself against
that goal.

## D12 — Phase sessions protocol

This is the coordinator session. Each phase runs in a dedicated session that
receives its assignment (objective, allowed edit paths, S# specs, deliverable)
from the coordinator and consults it instead of making product decisions.
Product questions go to the user through the coordinator.

## D13 — CLI output contract: one deterministic document per subcommand

Every `genmsi` subcommand writes exactly one deterministic document to stdout and
nothing to stderr unless it fails, which is what makes the output snapshot-testable
and usable from CI (ROADMAP Phase 2). `--json` wraps the body in a versioned
envelope, `{"schema": "genmsi.<document>/<major>", ...}`; within one major version a
change is additive only (new fields may appear; existing names, types and meanings
stay), and a breaking change bumps the number. Exit codes: `0` success, `1` the
model does not load, validate or run (including the five models the original
refuses), `2` usage error, `3` `lint` found compiler diagnostics on a runnable
model. `trace` has no filters beyond `--limit`, and `run-all <dir>` exists as the
CI convenience that sweeps one directory. Rationale: the smallest surface that
covers the original's workflow (D11) while giving scripts a contract they can
depend on; the schema field lets each document grow without breaking consumers.
