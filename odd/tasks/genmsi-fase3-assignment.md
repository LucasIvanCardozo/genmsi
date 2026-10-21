# Fase 3 (asignación): genmsi-mcp

Coordinator assignment for the Phase 3 session. This file is the coordinator's input,
not the phase's feature document: the phase session creates
`odd/tasks/genmsi-fase3-mcp.md` with these specs as `S1..Sn`, its own tasks and log, and a
verbatim `L1` holding the user's message that opens the phase. Product questions go to the
coordinator (D12), never decided here.

Engine baseline at assignment time: `main` @ `4a599a6`, 151 tests green, clippy clean,
oracle 20 of 20, CLI complete (fase 2 merged).

## Assignment

- Objective (ROADMAP Phase 3): "agents create, validate, simulate and inspect models".
  Tools: `simulate`, `lint`, `get_trace`, `explain_model`. Exit criteria: "an agent can
  build and simulate a model end to end via MCP".
- Allowed edit paths: `crates/genmsi-mcp/`, `docs/` (new documents; `DECISIONS.md` only to
  append a new entry at the end), `odd/tasks/`.
- Additive-only review path: if a tool needs something the engine already has internally,
  add public API in `crates/genmsi-core/` or `crates/genmsi-format/`. A *field* on the
  semantic model (see the line-number question below) is additive when the writer keeps
  producing byte-identical files; say so in the Log. Never change simulation semantics or
  the CLI's published output contract (`docs/cli.md`, `genmsi.*/1` schemas).
- Prohibited: `crates/genmsi-app/`, `crates/genmsi-cli/` behaviour changes, `fixtures/`
  (read-only oracle material), `scripts/wine-oracle/`, the comparison thresholds in
  `crates/genmsi-core/src/oracle.rs`.
- Dependency: adding `rmcp` to the workspace is expected by D7 and allowed; the crate's
  `Cargo.toml` already says so and keeps it out until the tool surface is defined. Any
  other new dependency is a question.
- Deliverable: the server runs over stdio with the four tools; `cargo test` green and
  clippy clean; a documented end-to-end transcript (an agent creates a small model, lints
  it, runs it, reads the trace) committed under `docs/`; `docs/mcp.md` with the tool
  schemas; every reference model reachable through the tools with the same accept/refuse
  split as the CLI.

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
  quality — not a maximally professional system. Every tool addition must justify
  itself against that goal."
- S6 (D13, verbatim): "`--json` wraps the body in a versioned envelope,
  `{"schema": "genmsi.<document>/<major>", ...}`; within one major version a change is
  additive only [...] and a breaking change bumps the number."
- S7 (D12, verbatim): "Each phase runs in a dedicated session that receives its
  assignment (objective, allowed edit paths, S# specs, deliverable) from the coordinator
  and consults it instead of making product decisions."

## Reading order

1. `docs/DECISIONS.md` D1–D13.
2. `docs/ROADMAP.md` Phase 3.
3. `docs/cli.md` — what the CLI already exposes and the JSON documents the MCP should
   reuse rather than reinvent.
4. `crates/genmsi-cli/src/{model,output,error}.rs` — the layer boundaries to mirror.
5. `crates/genmsi-core/src/lib.rs` and `crates/genmsi-format/src/{lib,validate}.rs` — the
   API surface, the error types and what they carry.
6. `docs/oracle-comparison.md`, sections "Baseline artifacts" and "Single-sample noise on
   small blocks", plus `odd/tasks/genmsi-fase2-cli.md` L16.

## Known traps — do not rediscover

- `TRANSFER` sends its `Prob` share to `Salida1`; pinned by
  `crates/genmsi-core/tests/transfer_branch.rs`. Do not invert it.
- The original prints zeroed queue averages for queues that end empty; those metrics never
  match and stay counted as mismatches.
- `fixtures/outputs/*.gpss.txt` is truncated for seven models; the authoritative expansion
  is the block table of the matching `.report.txt`.
- The report's operand column now matches the original except three documented
  differences (our `P:`/`V:`/`FN:` notation, `Multifurcacion`'s generated helper function,
  and `TERMINATE 0`). `explain_model` must not "helpfully" rewrite them.
- The five refused models fail with three error classes; the MCP must return them as
  tool errors, not as panics or empty reports.

## Product questions for the coordinator (ask before building)

1. **Line numbers.** `EngineError` names the block (`node`) and the class; the parser keeps
   source offsets only for its own XML errors (`genmsi_format::line_of`), and `Block` has no
   line. S1 asks for line/column: do we add a `line` to the semantic model (additive, the
   writer must stay byte-identical), or do we report `node` + class + name and let the agent
   locate the block?
2. **Tool result shape.** Reuse the CLI's `genmsi.report/1` and `genmsi.trace/1` documents
   verbatim inside the MCP result (one schema to maintain, S3/S6), or a separate MCP shape
   tuned for agents?
3. **Input shape.** Do the tools take a path only, or also a model document inline
   (agents building a model from scratch have no file yet — S2)?
4. **`explain_model`.** Is it `program_json` plus the block glossary (what each GPSS
   statement does), or something narrower?

Record the answers in `docs/DECISIONS.md` as `D14` and in `docs/mcp.md`.
