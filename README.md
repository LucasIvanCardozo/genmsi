# genmsi

Native Linux reimplementation of GenMSI (a Delphi/Windows GPSS-style discrete
event simulation workshop) in Rust: headless engine, CLI, MCP server for AI
agents, and a Tauri 2 + React GUI.

## Layout

```
crates/
  genmsi-format/   .gmdl + gClasses.xml: parser, validation, writer
  genmsi-core/     DES engine: scheduler, seeded RNG, expressions, traces, reports
  genmsi-cli/      `genmsi` binary: lint, simulate (+ --json), trace
  genmsi-mcp/      `genmsi-mcp` binary: rmcp stdio server (simulate, lint, get_trace, explain_model)
  genmsi-app/      Tauri 2 + React GUI (joins in Phase 4; placeholder for now)
docs/              DECISIONS.md (decision log), ROADMAP.md (phases), scratch.md (the work area outside the repository)
fixtures/          example .gmdl models + expected outputs from the original
scripts/           Wine oracle harness, MCP probe, block inventory tooling
```

## Rules

- Only `genmsi-format` and `genmsi-core` may be dependencies of more than one
  crate; the CLI, MCP and GUI are thin layers over the engine.
- Every simulation run is seeded; a run with the same seed is reproducible.
- Semantic model and layout are stored separately.
- Compatibility with the original GenMSI is semantic equivalence within
  statistical tolerance, not bit-exactness.

See `docs/DECISIONS.md` for the full decision log and `docs/ROADMAP.md` for
the phase plan.

## Development

```sh
cargo build
cargo test
cargo run -p genmsi-cli -- lint model.gmdl   # Phase 1+
```
