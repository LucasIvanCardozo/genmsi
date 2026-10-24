# Roadmap

Order is deliberate: the engine and its format are validated by tests before
any GUI work exists to depend on them.

## Phase 0 — Inventory and oracle harness

Goal: know exactly what must be built.

- Parse all available `.gmdl` models and inventory the block classes and
  properties actually used.
- Back up the original `Genmsi_20230711` folder before any Wine use.
- Discover how to extract outputs from the original under Wine (CLI mode,
  generated files, or GUI automation); record the method in
  `docs/oracle-wine.md`.
- Collect baseline outputs for the example models.

Exit criteria: block inventory committed, harness can produce original
outputs for at least one model.

## Phase 1 — genmsi-format + genmsi-core (subset)

Goal: read, validate, and simulate the inventoried block subset.

- `.gmdl` reader/writer with validation and precise structural errors.
- Block semantic model with layout separate from semantics.
- pest grammar for the expression language.
- Seeded DES engine (scheduler, transactions, blocks, queues, userchains as
  needed by the inventory).
- Replications, confidence intervals, CSV/JSON export, event traces.

Exit criteria: example models simulate; oracle comparison within statistical
tolerance; proptest + insta suites green.

## Phase 2 — genmsi-cli

Goal: the engine is usable by humans, scripts and CI.

- Subcommands: `lint`, `simulate` (+ `--json`), `trace`.
- Deterministic output suitable for snapshot tests.

Exit criteria: CLI runs every example model; `--json` output schema documented.

## Phase 3 — genmsi-mcp

Goal: agents create, validate, simulate and inspect models.

- rmcp stdio server; tools: `simulate`, `lint`, `get_trace`, `explain_model`.
- Errors returned with line/column so agents can fix their own models.
- The catalogue answers for a list of classes, the class document says which
  entities the class declares and which it uses, and the explain document
  publishes the structure of the model; the procedure lives in the agent's
  guide (`skills/genmsi-modeling/`), installed with the product.

Exit criteria: an agent can build and simulate a model end to end via MCP.

## Phase 4 — genmsi-app (Tauri 2 + React)

Goal: GUI parity with the original, better.

- Node editor (React Flow) with real Ctrl+Z/Y, multi-select, copy/paste with
  stable ids (the original's structural-bug source).
- Live monitor, property grid, reports and charts from engine JSON.
- Generated TS types from the Rust schema.

Exit criteria: the workflow of the original app is reproducible in the GUI.
