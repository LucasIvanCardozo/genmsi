# genmsi-app

**Do not add simulation logic here**: the app is a thin client over
`genmsi-core`.

The GenMSI desktop application: a Tauri 2 shell around a React 19 frontend
that draws a model, runs it and shows the numbers. Underneath the app there is
the engine (`genmsi-core`), the format (`genmsi-format`) and the command line
(`genmsi`), which lints, simulates and traces a model.

## What is in the crate

- `engine/` — the crate `genmsi-app-engine`, the facade both backends share.
  It loads the model, asks `genmsi-core` for the engine's documents and
  returns their bytes, and it owns the editing session: the model in memory,
  its undo stack and the save.
- `src-tauri/` — the desktop shell: the Tauri window, the commands it exposes
  to the frontend and the dialogs of the system.
- `src/` — the frontend:
  - `components/` — the panels and the charts (`ModelBar.tsx`, `Toolbar.tsx`,
    `RunConsole.tsx`, `MonitorView.tsx`, `ReportView.tsx`, `charts/`);
  - `graph/` — the graph of the model: the nodes, the edges, the frames and
    the shapes the canvas draws;
  - `monitor/` — the run: the replay, the series, the statistics;
  - `state/` — the store of the app and the loop of the proposal;
  - `engine/` — the bridge to the engine: the desktop shell and the dev
    bridge answer the same calls;
  - `edit/` — the rules of the editing;
  - `types/` — the types of the engine's documents, generated (see below).
- `scripts/generate-types.sh` — regenerates the types of
  `src/types/generated/`.
- `tests/` — the tests of the app.

## How it runs

In the browser, against the real engine, without installing the desktop
shell:

```sh
scripts/dev-app.sh
```

It compiles the engine, starts the dev bridge and serves the frontend at
`http://127.0.0.1:5173/`. To install the desktop application for the current
user, use `scripts/install-linux.sh`.

## The workspace

Only `crates/genmsi-app/engine` is a member of the Cargo workspace of the
repository (`Cargo.toml`). The desktop shell (`crates/genmsi-app/src-tauri`)
stays out of it (`exclude = ["crates/genmsi-app/src-tauri"]`) and is a
workspace of its own: its dependency tree is heavy and the build of the
repository must not slow down.

## The stack

React 19 + TypeScript, `@xyflow/react` (the node editor), Zustand (the state
of the UI), Tailwind (the styles), Recharts (the charts) and `lucide-react`
(the icons). The components are the ones of `src/components/`: the crate
depends on no component library of a third party.

The types of the engine's documents are **generated, never written by hand**:
`ts-rs` writes one file per published type into `src/types/generated/`, and
`crates/genmsi-app/scripts/generate-types.sh` runs it. The frontend imports
them.

## The rules that hold

- The shell (`src-tauri/`) carries no logic of the model either: it opens the
  window and moves the bytes of the engine's documents.
- The frontend draws what the engine answers; the only numbers it computes are
  the derived ones — ratios of the numbers the engine publishes
  (`src/monitor/derived.ts`).
- A type of the engine's documents is written in Rust and generated here, not
  written here.
