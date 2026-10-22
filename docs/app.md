# genmsi-app

The desktop application of GenMSI: a thin client over the engine
(`genmsi-core`), the same way the CLI and the MCP server are.

This document describes the **first vertical slice** (Phase 4): open a
`.gmdl`, see its block graph and the properties of the selected block, run the
model and read the report's tables from the engine's JSON. Editing nodes,
the live monitor and the charts come later (Phase 4b and 4c).

## The shape of the app

```
┌──────────────── frontend (React 19 + TypeScript + Vite) ───────────────┐
│                                                                          │
│  the adapter (one interface)                                             │
│      ├── devBridge ── HTTP ──▶ the Vite middleware ──▶ the binaries     │
│      └── tauri    ── invoke ─▶ the Tauri commands ──▶ the facade       │
│                                                                          │
│  the views: the graph, the property panel, the report                    │
└──────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
                     genmsi-app-engine (the facade)
                                    │
                                    ▼
                              genmsi-core
```

### One adapter, two backends

The frontend knows one interface (`src/engine/adapter.ts`):

| Operation | Document | Meaning |
|---|---|---|
| `listModels()` | `genmsi.models/1` | The `.gmdl` files the app offers. |
| `loadModel(path)` | `genmsi.model/1` | The model's blocks, geometry and links. |
| `lint(path)` | `genmsi.lint/1` | What the model compiles to. |
| `simulate(path, options)` | `genmsi.report/1` | The run's report. |

Two backends implement it:

- **devBridge** (`src/engine/devBridge.ts`): the Vite middleware
  (`vite/devBridge.ts`) spawns the engine and returns its bytes. The lint and
  the report come from the `genmsi` CLI verbatim; the model's graph comes from
  the `genmsi-app-engine` binary, which builds it with the engine's own
  functions.
- **tauri** (`src/engine/tauri.ts`): the commands of the desktop shell
  (`src-tauri/src/commands.rs`) call the same facade
  (`crates/genmsi-app/engine`), the crate the dev bridge's binary also runs.

Neither backend builds a document. They move the engine's bytes, and the
bytes are the same in both paths: `tests/parity.test.ts` compares the dev
bridge's response with `genmsi simulate --json` and `genmsi lint --json`,
byte for byte, and compares the facade's output with the same documents.

### The documents come from the engine

The report is the engine's `genmsi.report/1`, not a re-computation: the app
renders the blocks, facilities, queues, storages, chains, logic switches,
savevalues and tables exactly as the engine published them.

The model's graph needed one document that did not exist, so `genmsi-core`
publishes it: `genmsi.model/1` carries the blocks with the canvas geometry
the `.gmdl` holds, their decoded properties and the links between them. It is
built from the file's semantic model, so a model the engine refuses still has
a graph and the app can show the blocks and the error at the same time.

### The types are generated

The TypeScript types come from the Rust types (`ts-rs`), never written by
hand:

```bash
crates/genmsi-app/scripts/generate-types.sh
```

The script turns on the optional `ts` feature of `genmsi-core` and writes one
file per published type into `crates/genmsi-app/src/types/generated/`. The
frontend imports them; `src/types/index.ts` only adds the envelope every
document shares.

## The interface

The design is the app's own (the phase's decision `S2`): clarity over
similarity with the original.

- **Header**: the app's name, the model picker, the seed and the horizon, and
  the button that runs the model. One row, always visible.
- **Two views**: *Grafo* and *Reporte*. The graph shows the blocks with the
  geometry the model carries; the report shows the engine's tables.
- **Property panel**: the selected block's identity and its declared
  properties, decoded. The app computes nothing.
- **Numbers**: the original's comma decimals, so a report read in the app and
  a report read in the CLI say the same thing.
- **Text**: Spanish, neutral. The code, the file names and the documents stay
  in English.

A model the engine refuses is an error the user reads (the engine's own
message), never an empty graph.

## The files

| Path | What it is |
|---|---|
| `crates/genmsi-app/src/engine/` | The adapter and its two backends. |
| `crates/genmsi-app/src/components/` | The graph, the property panel, the report, the picker. |
| `crates/genmsi-app/src/state/store.ts` | The app's state (Zustand). |
| `crates/genmsi-app/src/types/generated/` | The types generated from the Rust schema. |
| `crates/genmsi-app/vite/devBridge.ts` | The Vite middleware that serves the engine's documents. |
| `crates/genmsi-app/engine/` | `genmsi-app-engine`: the facade both backends share. |
| `crates/genmsi-app/src-tauri/` | The desktop shell (outside the workspace: its dependency tree is heavy). |
| `crates/genmsi-app/tests/parity.test.ts` | The dev bridge's bytes against the CLI's. |

## Running it

```bash
cargo build -p genmsi-cli -p genmsi-app-engine
cd crates/genmsi-app && bun install && bun run dev
```

The dev server answers at <http://127.0.0.1:5173>. The repeatable visual loop
is in [`app-visual-loop.md`](app-visual-loop.md).

## Checking it

```bash
cd crates/genmsi-app
bun test        # the dev bridge's bytes against `genmsi simulate --json`
bunx tsc --noEmit
```

From the repository's root, the workspace's own checks keep running:

```bash
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## What this slice does not do

Editing nodes (the node editor with undo, multi-select and copy/paste), the
live monitor and the charts are Phase 4b and 4c. Nothing of them is stubbed
here: the app opens, shows and runs.
