<div align="center">
  <img src="crates/genmsi-app/src-tauri/icons/128x128.png" width="96" alt="GenMSI">
  <h1>GenMSI</h1>
  <p><b>A discrete-event simulation workshop: draw a model, run it, and watch the numbers.</b></p>
  <p><sub>The engine, the format and the tools are in Rust; the desktop application is Tauri 2 + React.</sub></p>
  <p>
    <img alt="CI" src="https://github.com/LucasIvanCardozo/genmsi/actions/workflows/checks.yml/badge.svg">
    <img alt="MIT" src="https://img.shields.io/badge/license-MIT-059669">
    <img alt="v0.5.1" src="https://img.shields.io/badge/version-0.5.1-6366f1">
    <img alt="linux · macos · windows" src="https://img.shields.io/badge/platform-linux%20%C2%B7%20macos%20%C2%B7%20windows-0d9488">
  </p>
</div>

## What it is

GenMSI simulates a queueing system: transactions are generated, they wait in a
queue, they take a resource, they wait some time and they leave. A model is a
graph of blocks — `GENERATE`, `QUEUE`, `SEIZE`, `ADVANCE`, `TERMINATE` and
sixty more — that you draw, edit and run.

The desktop application is one window with three panels and a side panel:

- **Reproducción** — the graph of the model with the run on it: the blocks the
  flow fills are marked, and the transactions travel over the links.
- **Gráficos** — the numbers of the run: the series of the entities, the
  occupancy of the resources, the histograms of the tables.
- **Reporte** — the engine's tables (queues, resources, storages) with the
  numbers of the instant the replay is at.

On the side, the **block panel** shows the selected block and its properties,
and the **catalogue** lists the classes that can be created, each one with the
shape the canvas draws.

Underneath the application there is the engine (`genmsi-core`), the format
(`genmsi-format`) and the command line (`genmsi`), which lints, simulates and
traces a model.

```mermaid
graph LR
  APP["genmsi-app<br/>the desktop application"] --> E["genmsi-core<br/>the engine"]
  CLI["genmsi-cli"] --> E
  E --> F["genmsi-format<br/>the .gmdl format"]
  classDef clientes fill:none,stroke:#ea580c
  classDef motor fill:none,stroke:#4f46e5
  classDef fuente fill:none,stroke:#cbd5e1
  class APP,CLI clientes
  class E motor
  class F fuente
```

## Install it

The application is installed with the installers of the
[releases](https://github.com/LucasIvanCardozo/genmsi/releases): every release
carries the packages of the systems, all of the same version.

| System | What the release carries |
|---|---|
| Linux | `.deb` and `.rpm` (x86_64 and arm64), the Flatpak, and a `.tar.gz` portable |
| Arch and CachyOS | `.pkg.tar.zst` |
| macOS | `.dmg` (Intel and Apple Silicon) |
| Windows | `-setup.exe` and `.msi` (x64) |

The sources of the packages are in [`packaging/`](packaging).

From the source, the application runs in the browser against the real engine
(it needs Rust stable and Bun):

```sh
git clone https://github.com/LucasIvanCardozo/genmsi.git
cd genmsi
scripts/dev-app.sh
```

And the engine on its own, with `cargo build --release`:
`./target/release/genmsi --help`.

## The MCP server (beta)

The same engine is served to AI agents over stdio (`genmsi mcp`): eight tools
to read the catalogue of the classes, build and edit models, lint them, run
them and read the results.

**It is in beta**: the tools and the documents they return may still change.
The details are in [`docs/mcp.md`](docs/mcp.md).

## Documentation

- [`docs/app.md`](docs/app.md) — the desktop application.
- [`docs/format-and-engine.md`](docs/format-and-engine.md) — the format and
  the engine.
- [`docs/cli.md`](docs/cli.md) — the command line.
- [`docs/mcp.md`](docs/mcp.md) — the MCP server.
- [`docs/DECISIONS.md`](docs/DECISIONS.md) — the decision log.
- [`docs/ROADMAP.md`](docs/ROADMAP.md) — where the work stands.

## Status

**v0.5.1** — the work is in progress and the interfaces are not frozen yet.

**MIT.** See the `license` field of the workspace.
