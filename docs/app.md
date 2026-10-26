# genmsi-app

The desktop application of GenMSI: a thin client over the engine
(`genmsi-core`), the same way the CLI and the MCP server are.

This document describes the **third vertical slice** (Phase 4c): the app
opens a `.gmdl`, shows its graph, **edits it** — creating, deleting,
connecting and moving blocks, editing their properties, undoing, copying
and pasting — saves the model back to a `.gmdl` that the original editor
still opens, and **replays a finished run** in a monitor of its own, with the
clock, the state of the entities and the charts of the run.

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
                    │                          │
                    ▼                          ▼
        genmsi-app-engine (the facade)   the session process
        ├── the documents                   └── the model in memory
        └── the editing session                 ├── the undo stack
                                               └── the save
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
| `lint(path)` | `genmsi.lint/1` | What the model compiles to, and what it found. |
| `simulate(path, options)` | `genmsi.report/1` | The run's report. |
| `trace(path, options)` | `genmsi.trace/1` | The run's trace: every event of the run. |
| `openModel(path)` / `newModel()` | the session state | The model in memory, ready to edit. |
| `apply(command)` | the session state | One editing command, applied. |
| `relayout()` | the session state | The model composed on the canvas, by the engine. |
| `undo()` / `redo()` | the session state | One step of the command stack. |
| `save(path?)` | the outcome of a save | The model written back to a `.gmdl`. |
| `simulateCurrent(options)` | `genmsi.report/1` | The run of the model **in memory**. |
| `traceCurrent(options)` | `genmsi.trace/1` | The trace of the run of the model **in memory**. |
| `catalogue()` | `genmsi.catalogue/1` | The block classes the editor offers. |

Two backends implement it:

- **devBridge** (`src/engine/devBridge.ts`): the Vite middleware
  (`vite/devBridge.ts`) forwards the requests to the engine and returns its
  bytes. The editing session lives in a long-lived `genmsi-app-engine
  session` process, which the bridge keeps open while the user edits.
- **tauri** (`src/engine/tauri.ts`): the commands of the desktop shell
  (`src-tauri/src/commands.rs`) call the same facade
  (`crates/genmsi-app/engine`), which the dev bridge's binary also runs.

Neither backend builds a document. They move the engine's bytes, and the
bytes are the same in both paths: `tests/parity.test.ts` compares the dev
bridge's response with `genmsi simulate --json` and `genmsi lint --json`,
byte for byte, and compares the facade's output with the same documents.

### The session owns the model

The model that is being edited lives in the **facade**, not in the
frontend: the facade holds the model in memory, the undo stack with it and
the save. The frontend sends commands and draws what comes back.

That is what makes the editing honest: every mutation is a command that the
additive API of `genmsi-format` applies, so **the app never touches the
XML of a model**. A mutation that the API does not have is a defect of the
API, not a reason for string surgery in TypeScript.

The facade speaks one protocol with its backends: one JSON request per
line, one JSON answer per line. The dev bridge holds a `genmsi-app-engine
session` process open; the desktop shell holds the same session in memory.
Both call the same functions, so the two backends stay identical.

### The app watches the model it has open

The app asks for the **fingerprint** of the file it has open every half a
second: the date of the modification, in nanoseconds, and the size. When the
fingerprint changes, the file changed, and the app:

- **reloads the model** and **runs it again**, by the same path as the button
  *Ejecutar*: in the thread of the work, with its cancellation. The reader
  sees the new model and its reproduction without touching anything.
- **shows the compiler's message** when the model of the disk does not
  compile, and keeps the model it had. A broken model never empties the app.
- **does not reload** when the reader has changes that were not saved: the app
  says that the model of the disk changed and waits. The reader's work is
  never thrown away by a change somebody else made.

The fingerprint travels as a string because a JavaScript number cannot hold
nanoseconds since the epoch exactly, and two writes in the same millisecond
have to look different.

The reload is an operation of the session, like an edit: undoing it brings
back the session it left behind, with the undo stack it had.

This is what makes the cycle with an agent work: the agent writes the model
through the MCP's `save_model` (which writes it atomically, so a reader that
watches the file never sees half of one), and the app, which is watching,
picks it up by itself.

### The app publishes the model it has open

For the cycle to close, the agent has to know **which file** the app has
open. The app says it, in a file of its own data directory: `open-model.json`,
beside `recent-models.json`. The file carries the `genmsi.openmodel/1`
document:

| Field | Meaning |
|---|---|
| `path` | The **absolute path** of the file the app has open. |
| `pid` | The pid of the process that has it open. |

The app writes the file when it opens a model, replaces it when the session
reloads or saves another file, and **removes it** when the session starts a
new model (a model that is not open is not published) and when the app
closes. The write is atomic, like every other write of the app: a reader
never sees half of a document.

The MCP server reads that file and publishes it as the `open_model` tool, so
an agent asks the server instead of guessing where the app writes or asking
the reader which file is open. When the app is not running, the file is not
there (or the process it names is gone), and the tool answers with the same
document with `path` and `pid` empty: that is an answer, not a failure.

The directory is the one the shell of the app resolves with Tauri, and the
one the engine's core computes for the platform:

| Platform | The directory |
|---|---|
| Linux | `$XDG_DATA_HOME/org.gensmi.desktop` (or `$HOME/.local/share/org.gensmi.desktop`) |
| macOS | `$HOME/Library/Application Support/org.gensmi.desktop` |
| Windows | `%APPDATA%\org.gensmi.desktop` |

### The commands

The commands are the editing API of `genmsi-format`, and their types are
generated from the Rust ones (`src/types/generated/EditCommand.ts`):

| Command | What it does |
|---|---|
| `addBlock` | Creates a block of a catalogue class with a fresh id. |
| `removeBlock` / `removeBlocks` | Removes blocks, their children and the links that touch them. |
| `moveBlock` | Moves a block (a drag is one command, not one per pixel). |
| `setLayouts` | Moves many blocks at once: the re-layout is one undo entry. |
| `connect` / `disconnect` | Joins two ports of sibling blocks, or undoes it. |
| `setProperty` | Sets a declared property, keeping the file's encoding. |
| `setCaption` | Sets the title a block shows. |
| `paste` | Copies blocks with fresh ids, remapped links and fresh labels. |

The labels are the connection points: a link is a pair of ports that share
one label, and the block's own label is the label of its first labelled
input. The original editor reused ids and labels when it copied a block,
and that is where its structural bugs came from; the paste here gives the
copies fresh ids and a fresh label per remapped link.

### The documents come from the engine

The report is the engine's `genmsi.report/1`, not a re-computation: the app
renders the blocks, facilities, queues, storages, chains, logic switches,
savevalues and tables exactly as the engine published them.

The model's graph is the engine's `genmsi.model/1`: the blocks with the
canvas geometry the `.gmdl` holds, their decoded properties, their ports
and the links between them. It is built from the file's semantic model, so
a model the engine refuses still has a graph and the app can show the
blocks and the error at the same time.

The catalogue of the block classes (`genmsi.catalogue/1`) says what a block
looks like in the file: its ports with their ids and its properties with
the declared type the editor renders. The catalogue is pinned against the
reference models by a test.

### The types are generated

The TypeScript types come from the Rust types (`ts-rs`), never written by
hand:

```bash
crates/genmsi-app/scripts/generate-types.sh
```

The script turns on the optional `ts` feature of `genmsi-core` and of
`genmsi-format` and writes one file per published type into
`crates/genmsi-app/src/types/generated/`. The frontend imports them;
`src/types/index.ts` only adds the envelope every document shares and
`src/types/protocol.ts` the four fields of the session's answer.

## The interface

The design is the app's own (the phase's decision `S2`): clarity over
similarity with the original.

- **Header**: the identity of the work and the **console of the run**. The
  identity comes first — the mark, the name of the app, the name of the
  model, the dot that says the work is not saved and the title the model
  carries — and the console closes the row, at the right: *Simular* (the
  model, and with it the report), *Ejecutar* (the model and the trace, with
  the replay), the pause and play of the replay, the speed, the clock and
  the slider of the replay. One row, always visible: the controls are there
  before a run, disabled, and the reader never has to start a run to find
  them.
- **Toolbar**: the commands, in one dense row, split by what they do and
  named. *Archivo* — `Nuevo`, `Abrir…`, `Recientes`, `Guardar`,
  `Guardar como…`; *Edición* — `Deshacer`, `Rehacer`, `Borrar`; *Vista* —
  `Encuadrar`, `Reacomodar`; and at the other end the three numbers the next
  run takes: the seed, with the lock that says whether it repeats, the
  horizon and the `START`. The commands that an icon explains keep only the
  icon; the ones that a word explains keep the word.
- **The lock of the seed**: a run that draws its own seed is a different run
  every time; a run with the reader's seed is the same one, and can be
  repeated. The checkbox next to the seed says which of the two it is: clear,
  the run draws its seed and the field shows which one it was; checked, the
  run repeats. Writing a seed checks the box, because a seed the reader
  chose is a seed the reader fixed.
- **One window, three panels**: *Reproducción*, *Gráficos* and *Reporte*. The
  reproduction is the graph of the model — with the geometry the model
  carries — with the run on it: the blocks the run is filling are marked,
  and the transactions that are on their way travel over the links. The
  charts are the numbers of the run, the series of the entities and the
  histograms of the tables; the report is
  the engine's tables, with the numbers of the instant the replay is at.
- **Lateral panel**: two tabs, *Bloque* (the selected block's identity and
  its declared properties, one editor per declared type) and *Bloques* (the
  classes that can be created, with what each one does and the ports and
  properties it declares). The panel is **of the model**: the state of the
  run is told by the report and by the charts, which say it with more
  detail and alive with the clock. The one thing the panel said that nobody
  else says —which transaction holds each facility— is the column *Ahora* of
  the indicators of the facilities.
- **Status line**: the foot of the window carries **every message of the
  app**, and there is no band of messages at the top: the top belongs to
  the header and to the toolbar. The line says what the model compiles to
  (recomputed after every change), the failure of the last run, the
  confirmation of a save and the question of the engine when it refuses to
  save a model. What needs the eye says so with a halo: it pulses three
  times, because the reader may be looking at the graph and not at the foot
  of the window.
- **Numbers**: the original's comma decimals, so a report read in the app
  and a report read in the CLI say the same thing.
- **Text**: Spanish, neutral. The code, the file names and the documents
  stay in English.

A model the engine refuses is an error the user reads (the engine's own
message), never an empty graph. The editing continues: the diagnostics say
what is wrong, and saving such a model is allowed after the user confirms
it.

### The property editors come from the declared type

The `.gmdl` declares the type of every property (`TSNA`, `PosInteger`,
`EntityUse`, `EntityDef`, `Listed`, `Expresion`, ...), and the editor
renders one editor per type instead of one free-text field for everything:

| Type | The editor | What it checks |
|---|---|---|
| `TSNA` | a number | the value is a number |
| `PosInteger` | a whole number | it is an integer greater than zero |
| `EntityUse`, `EntityDef` | a text field | the name has no whitespace (and no `:` when it defines an entity) |
| `Listed` | the options of its list | the value is one of them |
| `Expresion` | a text field | the value is not empty |

What cannot be checked without the engine (an expression's grammar, an
entity that exists) is left to the engine: the diagnostics panel says so
after every change.

### The monitor replays the run

The monitor is a tab of its own: the graph stays clean and the replay has its
own space, with the clock and the metrics.

**The replay is a projection of the documents the engine published**, never a
second simulation. The run emits the report (the final state) and the trace
(every event of the run); the monitor walks the trace and rebuilds what the
run had at every instant:

| What the monitor shows | Where it comes from |
|---|---|
| The clock | The time of the event. |
| The content of the queues | A transaction enters a queue when it enters the queue's block and leaves it when it enters the block that departs from it. |
| The owner of the facilities | A transaction takes a facility when it leaves the seize block, and frees it when it enters the release block. |
| The units in use of the storages | The same walk, on the enter and leave blocks. |
| The transactions inside every block | The entries and the departures the trace records. |

**No metric the engine computes is derived.** The average content and the
utilization are *not* rebuilt: the report document does not carry them (the
phase 4b review removed the column that implied otherwise), so the app does
not show them. What the monitor shows is the *state*, which is what a monitor
shows.

The replay is **deterministic**: the state at an event is a pure function of
the prefix of the trace, and a test pins it (the same prefix always gives the
same state).

The **index is built once** per run: the frames of the trace with what each
one changes, and the instants each entity's value changes. The animation
reads the index, so the replay of a big trace (`Parcial 6-11-23`, 82k events)
never walks the whole trace again per frame.

The trace has a **record limit** (the CLI keeps 100 by default; the app asks
for 200000), and the document says when the bound cut the trace: the monitor
says so instead of implying that it replayed everything.

**The figures of the blocks are the engine's** (D19). A block of the model is
one or several statements of the listing, and the report marks the row that
counts its block: the first one, the transactions that entered it. The app
shows that figure and adds nothing up — adding the rows of a block up would
count one transaction once per statement, five times for a facility use. A
generator shows no figure: no transaction stays inside one.

**A crossing is always seen.** A transaction that crosses blocks with no
waiting time crosses them in no time either, so the ball that shows the
crossing would blink: the flight has a floor in the clock of the reader (the
speed of the replay times the shortest a ball may fly), and the clock stops
at the next crossing instead of skipping it. The clock of the run is still
the run's clock: the floor is in the time of the reader, not in the model's.

### The charts read the documents

The section of the charts is one column that scrolls: the summary of the run
first, and then one group of charts per family of entities.

| Group | What it shows |
|---|---|
| **Resumen** | The run in numbers (transactions, events, clock, how it ended, throughput, population) and the ranking of the resources that squeeze it (the utilization of the facilities, the mean content of the queues, the mean utilization of the storages). |
| **Colas** | The content of the queues over the time, the distribution of the waits and the indicators of every queue. |
| **Facilities** | The occupancy of every facility over the time (the intervals it was held) and its indicators. |
| **Almacenes y cadenas** | The units in use of a storage against its capacity, and the size of a chain over the time. |
| **Tablas** | The histogram of a table of the model (the frequencies the engine publishes, the accumulated curve, the mean and the deviation) and the list of the tables. |
| **Bloques** | The transactions that went through each block, and the retries of each block: where the model is contended. |

The charts are drawn with **Recharts** (the only charting library, as D8
says), except the occupancy of the facilities, which is a row of colored
intervals: a line of ones and zeros says less and takes more.

- the **time series** are the *exact* history of the entity (the instants its
  value changes), not a sample of the trace: a queue that fills and empties
  between two samples would be invisible in a sample, and a facility that is
  free for a moment would look busy.
- the **axis of the time is the time of the run, from zero to its end**,
  whatever the first change of an entity is: an axis that started at the
  first datum would hide the time before it. The vertical line that marks
  the instant the reader is at crosses every chart.
- the **histogram** of every table and QTable of the model: the bars are the
  frequencies the report publishes and the curve is their running total. A
  table with more than 200 intervals is grouped, and the label says how many
  intervals each bar covers.

The only numbers the charts compute are the **ratios of the engine's own
numbers** —the mean content of a queue, the utilization of a facility, the
mean and the deviation of a table— and every one of them says so: they are
marked with an asterisk and the card says the formula. Each set of numbers is
divided by **its own** time: the numbers of the instant by the clock of the
replay, the published ones by the length of the run.

### The re-layout is explicit

`Reacomodar` recomposes the whole canvas with **dagre**, the Sugiyama
pipeline, **from top to bottom**: the input of a block is at its top and its
output at its bottom, so the flow the reader follows goes down, and the links
are short and straight. The flow is counted over the whole model (a link that
crosses the border of a container says where the flow goes), and the
containers are the frames around the blocks they hold.

The layout is computed by the **engine** (`genmsi_format::layout`, D18), it is
deterministic (the same model always produces the same positions) and it
travels as **one** command, so the whole re-layout is one undo entry. It is
never run on open: the picture the author of the file left stays until the
user asks for a new one.

The engine's composition also places the blocks that carry no geometry of
their own (with the default size), and the containers grow to hold what they
own. The frontend has no copy of the algorithm: the button asks the engine,
so the app and the MCP place a model the same way.

A container is the **frame** around what it holds: it is drawn behind its
content and it is not a thing the reader drags (the blocks inside it are).
The picture is framed whole, even when the model is wide enough to need a
zoom out to be seen.

## The files

| Path | What it is |
|---|---|
| `crates/genmsi-app/src/engine/` | The adapter and its two backends. |
| `crates/genmsi-app/src/components/` | The graph, the property panel, the report, the monitor, the picker, the toolbar. |
| `crates/genmsi-app/src/components/charts/` | The charts of the run: one file per card of the section, and the pieces they share (the card, the table, the bars, the series). |
| `crates/genmsi-app/src/edit/` | The pure parts of the editor: the property editors. |
| `crates/genmsi-app/src/monitor/` | The replay: the projection of the trace and the state it rebuilds. |
| `crates/genmsi-app/src/monitor/derived.ts` | The metrics the app derives from the numbers the engine publishes. |
| `crates/genmsi-app/src/monitor/series.ts` | The series, the intervals and the histograms the charts draw. |
| `crates/genmsi-app/src/state/store.ts` | The app's state (Zustand). |
| `crates/genmsi-app/src/types/generated/` | The types generated from the Rust schema. |
| `crates/genmsi-app/src/format.ts` | The numbers of the app, with the original's comma decimals. |
| `crates/genmsi-app/vite/devBridge.ts` | The Vite middleware that serves the engine's documents. |
| `crates/genmsi-app/engine/` | `genmsi-app-engine`: the facade both backends share. |
| `crates/genmsi-app/src-tauri/` | The desktop shell (outside the workspace: its dependency tree is heavy). |
| `crates/genmsi-app/tests/` | The parity of the bytes and the pure logic of the editor. |

## Running it

```bash
cargo build -p genmsi-cli -p genmsi-app-engine
cd crates/genmsi-app && bun install && bun run dev
```

The dev server answers at <http://127.0.0.1:5173>. The repeatable visual
loop is in [`app-visual-loop.md`](app-visual-loop.md).

The desktop shell is the same frontend under a native window:

```bash
cd crates/genmsi-app/src-tauri && cargo build
crates/genmsi-app/scripts/check-tauri-shell.sh
```

## Checking it

```bash
cd crates/genmsi-app
bun test        # the dev bridge's bytes against the engine's, the property
                # editors and the replay of the trace
bunx tsc --noEmit
```

From the repository's root, the workspace's own checks keep running:

```bash
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## What this slice does not do

The **live monitor is not here yet**: this phase replays a *finished* run.
The stepping engine (a model that runs and waits, `new`/`step`/snapshot) is
Phase 4d, a separate unit whose exit gate is the oracle 20/20.

The engine does not change in this phase: the trace the monitor replays is
the `genmsi.trace/1` document the CLI already published.
