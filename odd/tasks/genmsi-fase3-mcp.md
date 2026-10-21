# Fase 3: genmsi-mcp

Phase 3 session for genmsi-rs. Coordinator: "GenMSI-rs: sesión principal coordinadora"
(D12); product decisions are consulted, not decided here. Scope: `crates/genmsi-mcp/`,
`crates/genmsi-core/` and `crates/genmsi-format/` (additive public API only), `docs/`
(new documents; `DECISIONS.md` only to append an entry), `odd/tasks/`.

## Specs

- S1 (ROADMAP Phase 3, verbatim): "- rmcp stdio server; tools: `simulate`, `lint`,
  `get_trace`, `explain_model`. - Errors returned with line/column so agents can fix their
  own models."
- S2 (ROADMAP exit criteria, verbatim): "an agent can build and simulate a model end to
  end via MCP."
- S3 (D7, verbatim): "`genmsi mcp` runs an rmcp stdio server with tools: `simulate`,
  `lint`, `get_trace`, `explain_model`. The tool surface is a thin adapter over the same
  API the CLI uses."
- S4 (D2, verbatim): "only `genmsi-format` and `genmsi-core` may be dependencies of more
  than one crate. CLI, MCP and GUI are thin layers with no simulation logic."
- S5 (D11, verbatim): "Goal is minimum parity with the original, with better code
  quality — not a maximally professional system. Every tool addition must justify itself
  against that goal."
- S6 (D13, verbatim): "`--json` wraps the body in a versioned envelope,
  `{"schema": "genmsi.<document>/<major>", ...}`; within one major version a change is
  additive only [...] and a breaking change bumps the number."
- S7 (D12, verbatim): "Each phase runs in a dedicated session that receives its
  assignment (objective, allowed edit paths, S# specs, deliverable) from the coordinator
  and consults it instead of making product decisions."

## Tasks

- T1 (S1) `genmsi-format`: every `Block` records its source position (`SourcePos`: 1-based
  line and column), filled by the parser from the element's byte offset. The writer does
  not read the field, so a rewritten model stays byte-identical. Tests: the parser pins
  line/column of a block; the roundtrip test pins byte-identity.
  Commit: `feat(format): record the source position of every block`.
- T2 (S3, S6) `genmsi-core`: the canonical client documents live next to the engine, so
  the CLI and the MCP server emit the same bytes: `documents::LintDocument`,
  `documents::TraceDocument`, the schema ids, and `EngineError::node()`/`class()` so a
  client can map a failure back to the block that caused it. Additive public API only.
  Commit: `feat(core): publish the lint and trace documents for every client`.
- T3 (S3, S4) `genmsi-mcp`: the crate becomes a real rmcp stdio server: `rmcp` plus
  `tokio` in `Cargo.toml`, a library target (`server`, `tools`, `engine`, `errors`) and a
  thin binary that serves stdio. No simulation logic in the crate (S4).
  Commit: `feat(mcp): rmcp stdio server skeleton`.
- T4 (S2, S3) `genmsi-mcp`: the engine adapter — load a model from a path or from an
  inline document, compile it, run it, and render the engine's own documents.
  Commit: `feat(mcp): model adapter over the engine API`.
- T5 (S3, S6) `genmsi-mcp`: the tools `lint`, `simulate` and `get_trace` return the CLI's
  documents verbatim (`genmsi.lint/1`, `genmsi.report/1`, `genmsi.trace/1`), so there is
  one schema per document and no second contract to maintain.
  Commit: `feat(mcp): lint, simulate and get_trace tools`.
- T6 (S1) `genmsi-mcp`: failures are tool errors, never panics and never empty reports:
  a JSON body with `error` (stable class), `message`, `path`, `line`, `column`, `node`
  and `class`, taken from the engine error and the block's source position.
  Commit: same as T5.
- T7 (S3, S4) `genmsi-mcp`: `explain_model` returns the engine's `program_json` plus the
  GPSS glossary of the classes the model uses, so an agent can read a model and learn the
  vocabulary it needs to write one.
  Commit: `feat(mcp): explain_model with the GPSS glossary`.
- T8 (S2, deliverable) `genmsi-mcp` integration tests: the 25 reference models go through
  the tools with the same accept/refuse split as the CLI (20 accepted, 5 refused with the
  same error class), an inline model is linted, simulated and traced without a file, and
  the error body pins line/column.
  Commit: `test(mcp): cover the reference models through the tools`.
- T9 (S2, S6, deliverable) `docs/mcp.md`: the four tools with their input schemas, the
  documents they return, the error shape and the parity rules.
  Commit: `docs(mcp): document the tools, the schemas and the errors`.
- T10 (S2, deliverable) `docs/mcp-transcript.md`: the end-to-end transcript of an agent
  that creates a small model, lints it, simulates it and reads its trace.
  Commit: same as T9.
- T11 (D12, S7) Append `D14` to `docs/DECISIONS.md` with the four answers this phase
  implements, and record them in `docs/mcp.md`.
  Commit: same as T9.
- T12 (deliverable) Close: `cargo test --workspace` green and `cargo clippy --workspace
  --all-targets -- -D warnings` clean; every task carries the commit that holds it.

Commit slices: the slices follow the seams of the code (format, core documents, server
skeleton, tools, glossary, tests, docs), each one a reviewable behaviour of about 400
authored lines or less with its tests. The branch is `feat/phase3-mcp`; push, PR and
merge stay with the user.

## Commits

| Task | Commit |
|---|---|
| Feature document | `e116d54` `docs(odd): phase 3 feature document for the MCP session` |
| T1 | `10eb859` `feat(format): record the source position of every block` |
| T2 | `e8a7665` `feat(core): publish the lint and trace documents for every client` |
| T3, T4, T6 | `07f64e5` `feat(mcp): rmcp stdio server with the model adapter` |
| T5 | `b370d76` `feat(mcp): lint, simulate and get_trace tools` |
| T7 | `8516025` `feat(mcp): explain_model with the GPSS glossary` |
| T8 | `092a327` `test(mcp): cover the reference models through the tools` |
| T9, T10, T11 | `84912a7` `docs(mcp): document the tools, the schemas and the transcript` |
| T12 | `18967d7` `fix(mcp): tell a missing model from a model that cannot be read`, `1d10f51` `refactor(mcp): one place that turns a tool outcome into a result` |

## Decisions (answered by the coordinator, D12)

The four assignment questions were answered; these answers are the contract this phase
implements, `docs/DECISIONS.md` records them as `D14` and `docs/mcp.md` publishes them.

1. **Line and column.** The semantic model gains the source position of every block
   (`Block::source`: 1-based line and column). The field is additive: the writer does not
   read it, so the engine keeps producing byte-identical `.gmdl` files. A tool error
   carries `line` and `column` next to `node` and `class`.
2. **Tool result shape.** The tools return the CLI's documents verbatim
   (`genmsi.lint/1`, `genmsi.report/1`, `genmsi.trace/1`) inside the tool result: one
   schema per document, one contract to maintain (S3, S6). `explain_model` has no CLI
   counterpart, so it publishes its own `genmsi.explain/1`.
3. **Input shape.** Every tool takes the model either as `path` or as an inline `model`
   document (the `.gmdl` XML as text). An agent that builds a model from scratch has no
   file yet (S2); the inline document is compiled exactly like a file, without touching
   the disk.
4. **`explain_model`.** `program_json` (title, blocks with node/class/label/operands/
   ports, definitions, diagnostics) plus the GPSS glossary of the classes the model
   actually uses.

## Log

- L1 (verbatim, first message):

```
FASE 3 — GenMSI-rs: genmsi-mcp. Sos la sesión de la Fase 3 del proyecto genmsi-rs.
   La coordinadora es "GenMSI-rs: sesión principal coordinadora"; las decisiones de
   producto se consultan, no se deciden.

   Leé primero odd/tasks/genmsi-fase3-assignment.md (objetivo, rutas permitidas, specs
   S1-S7, orden de lectura, trampas conocidas y las preguntas que debés consultar). Creá
   tu propio feature doc odd/tasks/genmsi-fase3-mcp.md con esas specs, tu lista de tareas
   y este mensaje como L1 verbatim, antes de la primera escritura de código. Entregable:
   servidor rmcp por stdio con los cuatro tools, cargo test en verde + clippy limpio, un
   transcript end-to-end commiteado en docs/ y docs/mcp.md con los esquemas. Prohibido:
   crates/genmsi-app/, cambiar el comportamiento del CLI o el contrato genmsi.*/1,
   fixtures/, scripts/wine-oracle/ y los umbrales del oráculo.
```

- L2 (evidence, engine baseline at phase start): `main` @ `df95f3e`; 151 tests green and
  clippy clean per the assignment. `crates/genmsi-mcp/` exists with a stub `main.rs` and a
  `Cargo.toml` that keeps `rmcp` out until the tool surface is defined; this phase turns
  it into the server.
- L3 (decision, D14): the four product questions were answered by the coordinator
  (relayed through the user, since no coordinator session was reachable from this
  session's peer catalog): line/column on the semantic model, the CLI's documents
  verbatim as tool results, path or inline model as input, and `explain_model` as
  `program_json` plus the GPSS glossary. The answers are recorded in `D14`.
- L4 (scope note, D7 wording): D7 says `genmsi mcp` runs the server, but this phase's
  allowed edit paths exclude `crates/genmsi-cli/`, and D2 forbids the CLI from depending
  on a second client crate. This phase therefore delivers the `genmsi-mcp` binary speaking
  MCP over stdio; wiring it as the `genmsi mcp` subcommand is a coordinator decision that
  needs the CLI path and a D2 reading. Flagged at close, not decided here.
- L5 (scope note, documents): the canonical `lint` and `trace` documents live in
  `genmsi-core` (T2) because `crates/genmsi-cli/` is outside this phase's allowed edit
  paths; the CLI keeps its own copies of the same two structs until it is allowed to
  delegate to the core. Both copies are pinned by snapshot tests, and the phase closes
  with a differential check that the MCP result and the CLI result are the same bytes.
- L6 (evidence, differential check): the tool result and the CLI output are the same
  bytes. `genmsi lint/simulate/trace --json` and the matching tool call over
  `fixtures/models/TP6__Clase 6_1_PuertaBanco.gmdl` (the trace with `--limit 3`)
  produce byte-identical documents; the three `diff` runs are empty. The documents live
  in `genmsi-core` (T2), so this is one code path, not two that happen to agree.
- L7 (evidence, reference models through the tools): `crates/genmsi-mcp/tests/tools.rs`
  drives the server with an MCP client over a duplex transport: it lists the four tools
  and runs the 25 reference models through them. Twenty run and report; the five the
  original refuses fail as tool errors with the same message class as the CLI table of
  `docs/cli.md`, and each of them carries the `line`, the `column`, the `node` and the
  `class` of the block to fix.
- L8 (delivery, slices): the first two slices (the server with the adapter, and the
  three tools) are larger than the ~400-line guideline, because the crate, its
  dependency on `rmcp` and the engine boundary arrive together; the slices after them
  (glossary, tests, docs) are inside the guideline. The boundary is recorded here
  rather than hidden.
- L9 (evidence, transcript): `docs/mcp-transcript.md` is a real session: the server was
  started as `./target/debug/genmsi-mcp` and driven over stdio, one request at a time,
  with the model written inline. The transcript shows the handshake, `tools/list`, the
  four calls and the error an agent gets when it forgets the name of the queue.
- L10 (decision, tools): the tools return the CLI's documents as the text of the tool
  result and the failures as a tool error (`isError: true`) with the JSON body, not as
  a protocol error: the request was well-formed, the model was not, and the agent has
  to read the message to fix it.
- L11 (verification, close): `cargo test --workspace` green (151 tests before this
  phase, 24 more with it: 19 unit and 5 integration of the MCP crate) and
  `cargo clippy --workspace --all-targets -- -D warnings` clean. The oracle keeps its
  20 of 20 and the CLI snapshots are untouched: `crates/genmsi-cli/` was not modified in
  this phase.
- L12 (risk): the change touches the shared semantic model (`genmsi-format`) and the
  shared engine API (`genmsi-core`), so the native review candidate is the accumulated
  branch, not a single commit. Risk item 3 (a contract others consume: the `.gmdl`
  format and the `genmsi.*/1` documents) and item 6 (no test would catch a regression
  in what changes) both apply; the mitigations are the byte-identical roundtrip test
  over the 25 reference models, the differential check of L6 and the snapshot tests of
  the CLI, which did not move.
