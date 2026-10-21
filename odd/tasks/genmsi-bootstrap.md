# Bootstrap: genmsi-rs workspace skeleton

Cargo workspace skeleton for the GenMSI reimplementation in Rust (headless engine + CLI/MCP/GUI clients).
Scope of this document: repository bootstrap only (structure, docs, placeholders, first commit).

## Specs

- S1 (verbatim): "quiero ademas una buena filosofia de escalabilidad, buena estructura de carpetas ordenadas y reuzabilidad!"
- S2 (verbatim): "lo que si quiero es que minimo en la nueva se pueda hacer lo que se hace en la version actual (pero mejor en terminos de optimizacion y codigo)"
- S3: Repository lives at `~/Documentos/Workspace/Genmsi` (user decision, 2026 session); the original `Genmsi_20230711/` folder stays inside, git-ignored.
- S4: Layers: `genmsi-format` (parse/validate/write .gmdl + gClasses.xml), `genmsi-core` (DES engine, RNG, expressions, traces, reports), `genmsi-cli` (clap binary), `genmsi-mcp` (rmcp stdio server, Fase 3), `genmsi-app` (Tauri 2 + React, Fase 4, placeholder only).
- S5: Only `genmsi-format` and `genmsi-core` may be dependencies of more than one crate; cli/mcp/app are thin layers with no simulation logic.
- S6: Git remote: none for now (user decision); conventional commits on `main`.
- S7: Language of technical artifacts (code, comments, commit messages, repo docs): English. Conversation with the user: Spanish.

## Tasks

- T1 (S3, S4) Create Cargo workspace with crates genmsi-format, genmsi-core, genmsi-cli, genmsi-mcp as members; genmsi-app as placeholder directory (not a member until Fase 4). Commit: initial.
- T2 (S1, S5) docs/DECISIONS.md + docs/ROADMAP.md + README.md with structure and layer rules. Commit: initial.
- T3 (S4) fixtures/ and scripts/ placeholder directories with README explaining Fase 0 purpose. Commit: initial.
- T4 (S2) `cargo build` and `cargo test` green, zero warnings. Commit: initial.
- T5 (S6) git init + first conventional commit. Commit: initial.

## Log

- L1 (verbatim, first message): "Hola! te doy una explicacion de lo que quiero hacer con la aplicacion que esta dentrod e la carpeta gemnsi: [...] Recrear GenMSI como aplicación nativa de Linux, en Rust, no como una copia 1:1 sino como una versión mejorada, con el motor desacoplado de la interfaz y con agentes de IA como ciudadanos de primera clase (vía MCP)." + five decisions: (1) .gmdl compatibility with statistical tolerance; (2) headless engine with CLI/MCP/GUI clients; (3) corrected design from day 1 (validated data model, layout separate from semantics, seeded RNG, no opaque global state); (4) modern simulation (replications, confidence intervals, CSV/JSON export, event traces); (5) improvement not replica (data-defined blocks, expression parser with clear errors, GUI with real Ctrl+Z/Y, multi-select, live monitor).
- L2 (verbatim): "perfecto, te parece si encaramos este proyecto? quiero ademas una buena filosofia de escalabilidad, buena estructura de carpetas ordenadas y reuzabilidad! que te parece si esta es la sesion principal y para cada fase abro una sesion nueva y vos le mandas un mensaje para que se centre unicamente en sui fase y cualquier cosa te consulte. si hay preguntas que hacerme me las debes hacer vos aqui en la sesiuon principal."
- L3 (user answers): repo at `~/Documentos/Workspace/genmsi`; .gmdl models location unknown (everything he has is in `/home/lucas/Documentos/Workspace/Genmsi/Genmsi_20230711/`, which contains none) — start with hand-written example models; git remote: local only; skeleton built in this session.
- L4 (tooling review decisions): streaming XML replaced by DOM (`roxmltree`) — 722 KB does not need streaming; one charting lib (Recharts) for the GUI; Wine is a verifier, not a semantic discoverer — GPSS published semantics are primary source; engine builds the block subset actually used by the models first.
