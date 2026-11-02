# GenMSI

A native Linux reimplementation of **GenMSI** — the Delphi/Windows GPSS-style
discrete-event simulation workshop — in Rust: the file format, the engine, a
command-line client, an **MCP server for AI agents**, and a Tauri 2 + React
desktop application.

The goal is *semantic parity with better engineering*, not a bit-exact clone:
the engine answers the same questions the original does, and every model is
compared against it with a Wine harness
([`docs/oracle-comparison.md`](docs/oracle-comparison.md)).

```
cargo build --release
cargo test --workspace
./target/release/genmsi simulate fixtures/models/TP2__problema1.gmdl
```

## What's in the box

| Crate | What it is |
|---|---|
| `genmsi-format` | The `.gmdl` format: parser, validator, writer, the catalogue of the 66 block classes, the canvas composition, and the editing API (`EditCommand`). |
| `genmsi-core` | The engine: seeded scheduler, expressions, statistics, traces, reports. It **publishes** the documents every client reads; nobody computes a figure of its own. |
| `genmsi-cli` | The `genmsi` binary: `lint`, `simulate`, `trace`, `gpss`, `layout`, `run-all` and `mcp`. |
| `genmsi-mcp` | The MCP server: the engine served to AI agents over stdio, with eight tools. |
| `genmsi-app` | The desktop application: node editor, live monitor, reports and charts, over the same engine. |

```
crates/          the five crates above
skill/           the agent's guide (a skills root: genmsi-modeling/SKILL.md)
docs/            DECISIONS.md (the decision log), ROADMAP.md, mcp.md, cli.md, app.md…
fixtures/        25 example .gmdl models and the outputs of the original
scripts/         the installer, the Wine oracle, the MCP probe
Genmsi_20230711/ the original Delphi application (kept for the comparison)
```

## The MCP server

The MCP server is how an AI agent talks to GenMSI: it reads the catalogue of
the block classes, builds and changes models, validates them, runs them and
reads the results — all over one stdio process.

It is the **same binary** as the CLI (`genmsi mcp`), the same engine and the
same documents, so an agent and a script read the same thing.

| Tool | What it does | Returns |
|---|---|---|
| `catalogue` | Which block classes exist, what each one does, its ports, its properties and the XML of a block of it. | `genmsi.classlist/1`, `genmsi.class/1` |
| `apply_edit` | Adds, edits and connects blocks: a batch of the engine's own editing commands. | `genmsi.edit/1` |
| `lint` | Loads, validates and compiles a model and reports what it found. | `genmsi.lint/1` |
| `simulate` | Runs the model once and returns its GPSS report. | `genmsi.report/1` |
| `get_trace` | Runs the model with the event trace on and returns the records. | `genmsi.trace/1` |
| `explain_model` | Describes the model: its blocks, its properties by name and the links that wire them. | `genmsi.explain/1` |
| `save_model` | Writes a model the agent wrote, after checking that it compiles. | `genmsi.lint/1` |
| `open_model` | Which model the desktop app has open, so nobody has to guess where to write. | `genmsi.openmodel/1` |

Every model is taken as a `path` (a `.gmdl` file) or as a `model` (the document
as text). A failure names its class and the block that caused it, with the
line and the column, so an agent can fix its own model.

Full reference: [`docs/mcp.md`](docs/mcp.md).

### Installing it

```sh
./scripts/install-linux.sh --with-mcp     # the app, the CLI and the MCP
```

That installs the CLI in `~/.local/bin/genmsi`, registers the server with Pi
and puts the agent's guide in `~/.agents/skills/genmsi-modeling`.

### Configuring it in a client

The **server is the same everywhere**; what changes is the file each client
reads and the key it uses.

**Pi** — the command writes the file wherever that Pi reads it
(`~/.pi/agent/mcp.json`, and `.pi/mcp.json` in a trusted project):

```sh
pi mcp add genmsi -- ~/.local/bin/genmsi mcp
```

**Claude Desktop** — `~/.config/Claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "genmsi": {
      "command": "/home/you/.local/bin/genmsi",
      "args": ["mcp"]
    }
  }
}
```

**Cursor** — `~/.cursor/mcp.json`:

```json
{
  "mcpServers": {
    "genmsi": {
      "command": "/home/you/.local/bin/genmsi",
      "args": ["mcp"]
    }
  }
}
```

**VS Code (Copilot)** — `.vscode/mcp.json`, and the key is `servers`:

```json
{
  "servers": {
    "genmsi": {
      "type": "stdio",
      "command": "/home/you/.local/bin/genmsi",
      "args": ["mcp"]
    }
  }
}
```

**Windsurf** — `~/.codeium/windsurf/mcp_config.json`, key `mcpServers`.
**Zed** — `~/.config/zed/settings.json`, key `context_servers`.

### Two things that bite

**The working directory.** The server inherits the directory the client
launches it in, and a relative `path` is resolved against it: the same call
works in one client and fails in another. Pin it when the client can
(`pi mcp add genmsi --cwd /path/to/Genmsi -- …`), and use **absolute paths**
in the calls — `open_model` already returns one.

**The guide is a file, not the protocol.** The catalogue, the tool
descriptions and the published schema travel in the protocol, so an agent can
build a model with the MCP alone. The *procedure* (how a model is written,
which entities it has to declare, what the expression language is) lives in
the guide, and it is installed with the product:

```sh
cp -r skill/genmsi-modeling ~/.agents/skills/    # or: scripts/install-linux.sh --with-mcp
```

### What an agent does with it

```
catalogue(classes=["Generate", "Queue", "Terminate"])   # the vocabulary
apply_edit(path, [addBlock, addBlock, connect, setProperty])   # the model
lint(path)          # does it compile? what did the engine find?
simulate(path, seed)  # the report
```

A cold agent built a 35-block model that way in **12 MCP calls**, with no
sondas and without writing XML by hand
([`odd/tasks/medida3/notas.md`](odd/tasks/medida3/notas.md)).

## The agent's guide

[`skill/genmsi-modeling/SKILL.md`](skill/genmsi-modeling/SKILL.md) is the
procedure and the vocabulary for building a GenMSI model with the MCP. It is
the sanctioned place to learn the format: the catalogue holds the facts of a
class, the tools hold the interface, and the guide holds the rest.

`skill/` is a **skills root**: copying its contents into the directory where
the harness of an agent looks for skills (`~/.agents/skills/`) is the whole
installation.

## The desktop application

A Tauri 2 + React application: a node editor with real undo, a live monitor
that runs the model as it changes, and the reports and charts of the engine.

```sh
scripts/dev-app.sh            # in the browser, against the real engine
./scripts/install-linux.sh    # the desktop app, installed
```

The frontend reads the documents the engine publishes (the TypeScript types
are generated from the Rust ones), so it cannot drift from the engine. See
[`docs/app.md`](docs/app.md).

## The documents

| Document | What it holds |
|---|---|
| [`docs/DECISIONS.md`](docs/DECISIONS.md) | The decision log: why the engine owns the geometry, why the documents are versioned, what the MCP's verb is. |
| [`docs/ROADMAP.md`](docs/ROADMAP.md) | The phases of the work and where each one stands. |
| [`docs/format-and-engine.md`](docs/format-and-engine.md) | The `.gmdl` format and the semantics of the engine. |
| [`docs/cli.md`](docs/cli.md) | The CLI and the documents it prints. |
| [`docs/mcp.md`](docs/mcp.md) | The MCP server: the tools, the documents, the errors. |
| [`docs/mcp-transcript.md`](docs/mcp-transcript.md) | An agent building a model through the server, end to end. |
| [`docs/app.md`](docs/app.md) | The desktop application. |
| [`docs/oracle-comparison.md`](docs/oracle-comparison.md) | The comparison against the original, model by model. |
| [`docs/oracle-wine.md`](docs/oracle-wine.md) | How the original is run under Wine to produce that comparison. |

## Development

```sh
cargo build --release
cargo test --workspace                                   # the engine, the format, the CLI, the MCP
cargo clippy --workspace --all-targets -- -D warnings    # clean, it is a gate
(cd crates/genmsi-app && npm run typecheck && npm test)   # the frontend
```

The rules of the house, in one place:

- only `genmsi-format` and `genmsi-core` may be dependencies of more than one
  crate: the CLI, the MCP and the app are thin layers with no simulation
  logic of their own;
- every run is seeded, so a run with the same seed is reproducible;
- the engine publishes the documents and the clients invent nothing;
- the semantic model and the canvas geometry are stored separately;
- compatibility with the original is semantic equivalence within a
  statistical tolerance, not bit-exactness.

## The original

`Genmsi_20230711/` is the Delphi application this is a reimplementation of.
It is kept in the repository because it is the oracle: the harness in
`scripts/wine-oracle/` runs it under Wine and compares its numbers with ours.

## License

MIT (see the `license` field of the workspace). The original Delphi
application is not part of this code.
