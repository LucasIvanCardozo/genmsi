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

## D16 — The lint document says what the format found, in `genmsi.lint/1`

The three clients said different things about the same model: the editor showed
the warning of the format (a block that uses an entity the model does not
declare), and the CLI and the MCP said `ok`, because `genmsi.lint/1` was built
from the compiler's observations alone. Resolution: the lint document keeps its
version, its fields and their types; `diagnostics` carries the compiler's
observations **and** the warnings of the format, and `status` is `diagnostics`
when there is any of them. Exit code `3` keeps its meaning (the model is
runnable and something was found), so a warning is never a refusal.

D13 says that within a major version the fields keep their name, type and
meaning. The meaning of `diagnostics` widens here, deliberately: the field was
already "the findings that do not stop the model", the value domain of `status`
does not change (`ok` or `diagnostics`), and the messages are the same kind of
string, so no consumer can tell a finding apart from the text alone. What a
consumer can see is that more models report `diagnostics` and exit `3`, and
that is the point of the change. A `/2` version would leave every consumer of
`/1` behind for a wording that did not change.

A finding the compiler already reported is not said twice: the compiler's
wording wins, because it carries what the run does about it (a storage nobody
declares has no capacity, and the run stops when a transaction reaches it).

## D17 — The agent's knowledge: facts in the catalogue, structure in the explain document, procedure in the guide

The tools answer three different questions and each one has its own home. The
facts of a class — its ports, its properties, the entities it declares and
uses, the XML of a block — live in `catalogue`: they are read from the class
specification, so they cannot drift from it, and the classes an agent needs
come back whole in one call. The structure of a model — every block with its
properties by name, every edge naming the two ends of a connection — lives in
`genmsi.explain/1`, next to the compiled program, in the `genmsi.model/1`
document the desktop app already reads. The procedure — how a model is
written, which entities it declares, what the expression language is, what
the engine does with a class — lives in the agent's guide,
`skills/genmsi-modeling/SKILL.md`, which the installer puts where the harness
of an agent finds it.

The alternative was to publish more in the tools: a table of the compiled
operand order of every class, and a tool that mutates a model. The first is
57 rows written by hand next to the compiler that decides the order, and it
drifts the day the compiler changes; the model's own XML already carries every
property with its name, and the explain document publishes it. The second
changes the tool surface, and it waits for the measurement that says the
agent needs it.

Rationale: the cold agent that built a nine-block model spent 61 tool batches
and read the engine's source, and what it lacked was information, not tools.
D11's guard applies: every addition here answers a question the agent could
not answer with what the tools already said.

## D18 — The engine composes the canvas

Where a block sits on the canvas and how big it is is the engine's business.
A model that does not carry that geometry is still a model: the engine
composes it (`genmsi_format::layout`), so the editor can draw it, and the
composition is what the file carries once the model is written.

The alternative was to keep the geometry in the file alone, as it was: the
editor read the geometry the file carried and drew nothing without it. A
model an agent builds comes from the catalogue's templates, and the templates
carry no geometry, so every model an agent wrote reached the app as an empty
canvas — the engine validated it perfectly (`lint: ok`) and the picture was
blank. The app's own button could not fix it either: its algorithm left the
blocks that had no geometry exactly where they were.

The composition is deterministic (the same model always produces the same
positions), it never touches the semantics (the geometry does not enter the
simulation), and it leaves a model that is already composed exactly as its
author placed it: only an incomplete geometry is composed, and only when
something asks for it (the app's `Reacomodar`, the MCP before it writes a
model, the CLI's `genmsi layout`).

There is one algorithm, not two. The frontend had its own copy of the
re-layout, and a copy is a second answer to the same question: the app
reordered the graph one way and the engine another. The button now asks the
engine, and the frontend's copy is gone.

Rationale: the session the user ran asked for a simpler model and wrote it
with two Python scripts, because the MCP had no way to build a model and the
file it wrote had no geometry to draw. The engine composing the canvas is
what makes an agent's model a model the reader can see.
