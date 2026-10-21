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

## D14 — MCP tool surface: the CLI's documents, a model by path or by document

The four tools of D7 are thin adapters over the same API the CLI uses, and they
return the CLI's documents verbatim: `lint` returns `genmsi.lint/1`,
`simulate` returns `genmsi.report/1` and `get_trace` returns `genmsi.trace/1`.
`explain_model` has no CLI counterpart, so it publishes `genmsi.explain/1`: the
engine's `program_json` plus the GPSS glossary of the classes the model uses.
Rationale: one schema per document and one contract to maintain (D13), so an
agent and a script read the same thing; the tool result and the CLI output are
the same bytes.

Every tool takes the model either as `path` (a `.gmdl` file) or as `model` (the
document as text), because an agent that builds a model from scratch has no
file yet. The inline document is compiled exactly like a file and never touches
the disk.

A model that does not work is a tool error with a JSON body: `error` (a stable
class), `message`, `path`, `line`, `column`, `node` and `class`. To make the
line and the column possible, the semantic model records the source position of
every block (`genmsi-format`: `Block::source`); the field is additive, the
writer never reads it, and the engine keeps producing byte-identical `.gmdl`
files. The position points at the opening tag of the `<module>` element in the
`.gmdl` source, not at the line of the GPSS program the original generated.

This phase delivers the server as its own binary, `genmsi-mcp`, speaking MCP
over stdio. D7's wording (`genmsi mcp` runs the server) needs the CLI path,
which the Phase 3 assignment does not allow, and D2 forbids the CLI from
depending on a second client crate; wiring the subcommand is a coordinator
decision.
Resolved by D15: the CLI hosts the same server behind `genmsi mcp`.

## D15 — One entry point for the MCP server: `genmsi mcp`, `genmsi-mcp` kept

D7 says `genmsi mcp` runs the server, and D1's single-binary rationale rests on that
sentence. Phase 3 delivered the server as its own `genmsi-mcp` binary and flagged the
conflict with D2's layer rule instead of deciding it. Resolution: `genmsi-mcp` exposes
`serve_stdio()` and a blocking `serve_stdio_blocking()` (it owns the runtime, the CLI
stays synchronous), and `genmsi-cli` hosts the same server behind `genmsi mcp`. It is
the workspace's only client-to-client dependency, and it is allowed because that crate
carries no simulation logic (D2): it adapts the engine's documents into tools, and the
CLI adds one dispatch arm and nothing else. The standalone `genmsi-mcp` binary stays for
hosts that pin a command path.
