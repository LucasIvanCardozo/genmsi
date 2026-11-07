# genmsi MCP server

`genmsi-mcp` is the MCP server of the GenMSI engine (ROADMAP Phase 3): it serves
the engine to an AI agent over stdio with the eight tools of decision D7 —
`simulate`, `lint`, `get_trace` and `explain_model`, plus `save_model`, which
writes a model the agent built, `apply_edit`, which builds and changes one
block by block, `catalogue`, which tells the agent which block classes exist
and what the XML of a block of each one looks like, and `app_state`, which
tells the agent what the desktop app is showing right now (the model it has
open among it). The
binary is a thin layer over `genmsi-core` and `genmsi-format`; it carries no
simulation logic (decision D2), and the documents it returns are the ones the
CLI publishes (decision D13).

## The agent's guide

The tools carry the facts of a class (`catalogue`) and the structure of a
model (`explain_model`); the procedure — how a model is written, which
entities it has to declare, what the expression language is and what the
engine does with them — lives in the agent's guide:
[`skill/genmsi-modeling/SKILL.md`](../skill/genmsi-modeling/SKILL.md).

The guide travels with the product: `scripts/install-linux.sh --with-mcp`
installs it in `~/.agents/skills/genmsi-modeling`, where the harness of an
agent finds it. It is the sanctioned place to learn the format: an agent that
reads the guide does not read the engine's source, and it does not spend
twenty calls discovering what a link looks like.

```
genmsi-mcp            # speaks MCP on stdin/stdout
```

Standard output belongs to the protocol: the server writes nothing else there.
A failure that stops the server goes to standard error and the process exits
with `1`.

## The eight tools

| Tool | What it does | Returns |
|---|---|---|
| `catalogue` | Says which block classes exist and what a block of one of them looks like: what the class does, its ports, its properties, the XML of the block and the **contract** of the class (what it does to the flow, what it blocks, the rules of taking and releasing, and the traps). | `genmsi.classlist/1` or `genmsi.class/1` |
| `lint` | Loads the model, validates it and compiles it. It does not simulate. It reports what it found. | `genmsi.lint/1` |
| `simulate` | Runs the model once and returns its GPSS report. | `genmsi.report/1` |
| `get_trace` | Runs the model with the event trace enabled and returns the records. | `genmsi.trace/1` |
| `explain_model` | Describes the model: its blocks, its operands and what each statement does. | `genmsi.explain/1` |
| `save_model` | Writes a model the agent built to a file, after checking that it compiles. | `genmsi.lint/1` |
| `apply_edit` | Adds, edits and connects the blocks of a model: a batch of the engine's own editing commands. | `genmsi.edit/1` |
| `app_state` | Says what the desktop app is showing right now: the model it has open, whether the reader has unsaved changes on it, what it found and the last run it made. | `genmsi.live/1` |

Every tool but `catalogue` takes the model in one of two ways:

- `path`: the path of a `.gmdl` file on disk.
- `model`: the `.gmdl` document as text, for a model an agent is building and
  has not written to a file yet.

Exactly one of them is required; giving both, or neither, is an
`invalid_arguments` error.

### `catalogue`

| Field | Type | Default | Meaning |
|---|---|---|---|
| `class` | string or null | — | Name of the class to describe (`Generate`, `Queue`, ...). |
| `classes` | array of strings or null | — | Names of several classes to describe, in one call. Give `class` or `classes`, not both. |

**Call this tool before writing a model.** It is the one that saves the agent
from guessing: it says which classes exist, what each one does, which ports
(`ioid`) and which properties (`name`, `ty`, `value`) each class declares,
and it gives the XML of a block of the class, exactly as the file stores it.

Without `class` it returns the `genmsi.classlist/1` document:

| Field | Type | Meaning |
|---|---|---|
| `schema` | string | `genmsi.classlist/1` |
| `classes` | array | Every class the editor offers: `class` (`moduleclass`), `title` (what the class does, in the words of the interface) and `description` (what the class does, in the words of the engine's glossary). |
| `skeleton` | string | The skeleton of a model: the smallest `.gmdl` document the format reads, with the model root and nothing else. |

With `class` it returns the `genmsi.class/1` document of that class:

| Field | Type | Meaning |
|---|---|---|
| `schema` | string | `genmsi.class/1` |
| `class` | string | `moduleclass` the blocks of this class carry. |
| `title` | string | What the class does. |
| `description` | string | What the class does, in the words of the engine's glossary: the same text the list carries. |
| `inputs` | array | Input ports: `ioid`, `name`, `caption`. |
| `outputs` | array | Output ports: `ioid`, `name`, `caption`. |
| `properties` | array | The properties the class declares: `name`, `hint`, `ty`, `subtype`, `unique` and the `options` of a listed property. |
| `declares` | array | The entities the class **declares**: the storages, functions, tables, variables and logic switches a model has to define before it uses them. Each entry carries the `property` that names the entity and its `kind`. |
| `uses` | array | The entities the class **uses**: the facilities, queues and labels a model creates by using them. Each entry carries the `property` that names the entity and its `kind`. |
| `template` | string | The XML of a minimal, valid block of this class. |

The `description` is empty for a class the glossary does not cover: the
document never invents one.

With `classes` it returns the `genmsi.classes/1` document: the body of every
class asked for, in the order they were asked for, under one `classes` key.
An agent that needs eight classes asks for the eight in one call, instead of
calling the tool once per class. A class the catalogue does not offer fails
the call with the same `class_not_found` failure a single class fails with.

The `hint` of a property is **decoded**: the catalogue stores it
percent-encoded, the way the file does, and the document hands it over as the
text a reader reads (`"Cantidad de Transacciones a generar"`, not
`"Cantidad%20de%20Transacciones%20a%20generar"`). The `template` keeps the
encoded text, because the template is the text the file stores.

A class the catalogue does not offer is a `class_not_found` failure.

**How a model is written with the templates.** The template is a `<module>`
element with the ports, the properties and the default value of each property
the class declares. Three things the agent has to do with it:

1. **Renumber the block.** The template carries `moduleid="0"`, the id of the
   model root: every block of the model needs its own id.
2. **Drop the block into the skeleton's `<modules>` section.** The skeleton is
   the model root; the blocks are its children, and they name it in `owner`.
3. **Fill in the operands the class declares no default for.** The `Terminate`
   class, for instance, declares no default for its operand A, and the run
   never ends without it.

The links between the blocks live in the `<links>` section of the model root,
and each one names the two ends of the connection: the module and the port it
leaves from, and the module and the port it arrives at.

```xml
<link frommoduleid="1" fromioid="1" tomoduleid="2" toioid="1"/>
```

With that, the model is one the engine reads, compiles and runs: `lint` says
`"status": "ok"` and `simulate` returns the report.

### `lint`

| Field | Type | Default | Meaning |
|---|---|---|---|
| `path` | string or null | — | Path of the `.gmdl` model file. |
| `model` | string or null | — | The `.gmdl` document as text. |

Returns the `genmsi.lint/1` document: the model path (`<memory>` for an inline
document), the status (`ok` or `diagnostics`), the title, the number of
executable statements, the counts of the definitions and the findings: the
compiler's observations and the warnings of the format (an entity the model
uses without declaring it). A finding is not a failure: the model loads,
validates and compiles, and the document says what it found. The fields are
described in [`docs/cli.md`](cli.md#genmsilint1).

### `simulate`

| Field | Type | Default | Meaning |
|---|---|---|---|
| `path` | string or null | — | Path of the `.gmdl` model file. |
| `model` | string or null | — | The `.gmdl` document as text. |
| `seed` | number | drawn at random | Seed of every random stream; the run says which it used, and the same seed reproduces the same run. |
| `until` | number or null | — | Stop once the simulated clock passes this time. |
| `start` | number | `1` | How many terminations the run waits for, like the original's `Start` field. The run ends when the counter reaches zero or below. The model does not declare it. A value below one is refused. |

Returns the `genmsi.report/1` document, the same document
`genmsi simulate --json` prints.

### `get_trace`

| Field | Type | Default | Meaning |
|---|---|---|---|
| `path` | string or null | — | Path of the `.gmdl` model file. |
| `model` | string or null | — | The `.gmdl` document as text. |
| `seed` | number | drawn at random | Seed of every random stream; the run says which it used. |
| `until` | number or null | — | Stop once the simulated clock passes this time. |
| `start` | number | `1` | How many terminations the run waits for, like the original's `Start` field. The run ends when the counter reaches zero or below. The model does not declare it. A value below one is refused. |
| `limit` | number | `100` | Maximum number of trace records to keep. |
| `classes` | array of strings | — | Keep only the events of these classes (empty: all of them). |
| `nodes` | array of numbers | — | Keep only the events of these blocks of the document (their `moduleid`; empty: all of them). |
| `xact` | number | — | Keep only the events of this transaction. |
| `from` | number | — | Keep only the events from this simulated time on. |
| `to` | number | — | Keep only the events until this simulated time. |
| `summary` | boolean | `false` | Answer with the summary of the run instead of the events. |

Returns the `genmsi.trace/1` document, the same document `genmsi trace --json`
prints. The document says `truncated: true` when the bound may have cut it.

**The filters are applied when the event is recorded**, so the limit counts
the events that **pass** the filter: an agent that asks for the events of one
class gets those events, not the first ones of the run. The document carries
the filter it used, so the agent knows what it is not seeing.

`until` is the horizon of the **run**; `from` and `to` are the window of the
**filter**: they cut the events of a run that goes on.

**The summary** (`summary: true`) answers the `genmsi.trace_summary/1`
document instead: what each block did — how many times a transaction entered
it, the first and the last time — and not the events. It is the summary of
the **whole run**, not of the events that were kept, and it is not cut by the
limit. A block nobody entered is in it, with zero entries.

### `explain_model`

| Field | Type | Default | Meaning |
|---|---|---|---|
| `path` | string or null | — | Path of the `.gmdl` model file. |
| `model` | string or null | — | The `.gmdl` document as text. |

Returns the `genmsi.explain/1` document:

| Field | Type | Meaning |
|---|---|---|
| `schema` | string | `genmsi.explain/1` |
| `program` | object | The engine's description of the compiled program: `title`, `start_counter`, `blocks[]` (`node`, `class`, `label`, `operands`, `ports`), `storages`, `functions`, `variables`, `random_variables`, `tables`, `qtables` and `diagnostics`. |
| `model` | object | The model as the document writes it: `path`, `title`, `blocks[]` (`id`, `class`, `kind`, `name`, `caption`, `label`, `container`, `layout`, `properties[]` (`name`, `value`, `ty`, `subtype`, `hint`, `unique`, `visible`), `inputs[]` and `outputs[]` (`ioid`, `name`, `caption`, `label`)) and `edges[]` (`from_module`, `from_ioid`, `from_port`, `to_module`, `to_ioid`, `to_port`, `label`, `container`). It is the `genmsi.model/1` document the desktop app reads. |
| `glossary` | object | What each GPSS statement of the model does, keyed by class. Only the classes the model uses appear. |
| `map` | array | The **map between the compiled statements and the blocks of the document**: one row per statement, with `statement` (its index in the program), `node` (the `moduleid` of the block it came from), `class`, `caption` (the name the agent gave the block) and `line` and `column` (where the block is in the `.gmdl`). |

**The map is what makes the trace legible.** The trace numbers the
*statements* the model compiles to, and the report numbers them the other
way round; the `node` of an event and the `map` of this document are the
bridge to the block of the document and to its place in the file: "what block
is statement 218" is one lookup, not a piece of reverse engineering. A
module that compiles to several statements (the `Clock` is a generator and a
terminate) has one row per statement, and the `node` repeats.

The `program` is the compiled GPSS: its `operands` are positional, in the
order the original's listing uses. The `model` is the document itself: every
property carries its name and its value, and every edge names the two ends of
a connection. An agent reads a model — and writes the next one — from the
`model`, not from the positions of an operand string.

The glossary is the vocabulary an agent needs to write a model: it says what
each statement does and which operands it takes (`Media`, `Cantidad`,
`Parametro`, ...). It covers the classes the engine implements; the wording
follows the semantics of `genmsi-core`. The same descriptions travel with the
catalogue: `catalogue` says what a class does without asking for a model.

### `save_model`

| Field | Type | Default | Meaning |
|---|---|---|---|
| `path` | string or null | the model the app has open | Path of the `.gmdl` file to write. |
| `model` | string or null | — | The whole `.gmdl` document as text. |
| `target` | string | `"file"` | Where the model goes: `"file"` writes it to `path`; `"proposal"` leaves it in the staging area of the app, without touching the disk, and the person decides. |
| `summary` | string or null | — | One line about what the agent did. It travels with the proposal, so the person reads it before deciding. |

Writes the model to the file and returns the `genmsi.lint/1` document of what
was written.

**The trial of a document is `lint`.** To know whether a document compiles
before writing it, hand it to `lint` as `model`: it loads, validates and
compiles it and reports what it found, and writes nothing.

**The `path` may be left out.** When it is, the model the app has open is
the one that is written: the agent works on what the reader is looking at
instead of guessing a destination. With no app, or with an app that has no
model open, the failure is `no_open_model` and it says what to do.

**The model is checked before it is written**: a model that does not load,
does not validate or does not compile is **not** written, and the failure is
the same one the other tools give, with the block and its position. The write
is **atomic**: the text goes to a temporary file next to the target and that
file is renamed over it, so a reader that watches the model (the desktop
application does) never sees half of one.

**A model the reader is editing is not written**: when the desktop app has
unsaved changes on it, the tool writes nothing and answers `model_locked`
(see `app_state`).

The tool writes the **whole document**, not a patch: read the model with
`explain_model`, change the text and hand the complete document back.

**A model without canvas geometry is composed before it is written.** The
geometry is the engine's business (D18), and a model the editor cannot draw
is of no use to the reader: the engine places the blocks in layers and the
file carries the geometry. A model that already carries its geometry keeps
the disposition its author left.

The composition is **not a finding**: the lint document of a model that
compiles is `ok`, with no findings, whether the engine composed the model or
not. `genmsi layout` is the command that says what the composition placed.

### `apply_edit`

| Field | Type | Default | Meaning |
|---|---|---|---|
| `path` | string or null | the model the app has open | Path of the `.gmdl` model file. |
| `model` | string or null | — | The `.gmdl` document as text. |
| `commands` | array | — | The batch of editing commands, in order. |
| `layout` | string | `"auto"` | Where the blocks the batch adds go: `"auto"` or `"explicit"`. |
| `target` | string | `"file"` | Where the result goes: `"file"` writes it to the file the batch edited; `"proposal"` leaves it in the staging area of the app, without touching the disk, and the person decides. |
| `summary` | string or null | — | One line about what the batch did. It travels with the proposal, so the person reads it before deciding. |
| `dry_run` | boolean | `false` | Try the batch **without writing it**: the commands are applied and the model is compiled, nothing is written, and the answer carries the model it would have written. |

**The `path` may be left out.** When it is, the batch works on the model the
app has open: the agent works on what the reader is looking at instead of
guessing a destination. With no app, or with an app that has no model open,
the failure is `no_open_model` and it says what to do.

**The commands are the engine's own.** They are the same `EditCommand` values
the desktop application speaks over its own protocol
(`apply(command: EditCommand)`), and the schema the tool publishes is
**derived from that type**: the vocabulary of the commands is written in one
place, and the agent, the application and the engine speak the same one.

| Command | Fields | What it does |
|---|---|---|
| `addBlock` | `class`, `container`, `x`, `y` | Creates a block of a catalogue class inside a container. |
| `connect` | `from`, `fromIoid`, `to`, `toIoid` | Wires an output port to an input port. |
| `disconnect` | `container`, `from`, `fromIoid` | Removes the link that starts at a port of a container's child. |
| `setProperty` | `id`, `property`, `value` | Sets a declared property. |
| `setCaption` | `id`, `caption` | Names a block. |
| `moveBlock` | `id`, `x`, `y` | Moves a block, keeping its size. |
| `removeBlock` | `id` | Removes a block, its descendants and the links that touch them. |
| `removeBlocks` | `ids` | Removes several blocks as ONE operation. |
| `setLayouts` | `placements` | Places many things at once. |
| `movePort` | `block`, `ioid`, `input`, `x`, `y` | Moves a door of a container. |
| `removePort` | `block`, `ioid`, `input` | Removes a door of a container. |
| `paste` | `ids`, `dx`, `dy` | Copies blocks with fresh ids and remapped links. |

**The batch is atomic.** The commands are applied in order; if one fails, the
ones that were applied are undone and the model stays exactly as it was. The
failure is an `invalid_command` that names the command that failed —its index
and its `op`— so the agent corrects that command and sends the batch again.
The batch is replayable: the commands are the source of truth, not the text.

**The failure names what the agent wrote.** Besides the index and the `op`,
the failure says the **block** the command was about, its **class**, the
**name the agent gave it** and the **property** the command was setting: the
`name` and the `property` are fields of the failure, and they are in the
message too. An agent that built a model with fifty blocks named them, and
the name is what it has to look for — not the number the engine counts
blocks with.

**The trial.** `dry_run` applies the batch and compiles the model **without
writing anything**: the answer says what would have happened (`written:
false`) and carries the model it would have written, in `source`. The engine
compiles the model before it writes it, so the trial is the same road with
the last step left out — and a trial of a batch that fails fails the same
way.

**The model is checked before it is written.** A model that does not compile
is **not** written, and the failure is the one of the format or of the
compiler —the same classes the other tools give— with the block and its
position. The file the agent named keeps the model it had.

**A model the reader is editing is not written.** When the desktop app has
**unsaved changes** on the model the call named, the tool writes **nothing**
and answers `model_locked`: the app does not reload a model that changes under
it while the reader has changes that are not saved, so the work would stay
invisible on the canvas and the reader's save would overwrite it. Ask
`app_state` before writing, tell the reader, and send the same batch again
once the model is saved.

**Where the blocks go.** `layout: "auto"` (the default) puts every block an
`addBlock` adds in the free spot of its container, the same place the desktop
application gives a block that is dropped; `layout: "explicit"` uses the `x`
and the `y` of the command. A `paste` keeps the place of its source plus the
offset of the command, in both modes.

**The places of the commands survive the composition.** When the model the
batch edits carries no geometry, the engine composes its canvas —and it does
it **before** the batch, so the blocks the batch places keep the place they
were given.

**The ids are predictable.** The blocks a batch creates take the next free
ids, in the order of the commands: a model that starts from the skeleton
numbers them 1, 2, 3... A command that refers to a block of the same batch
(the `connect` that wires a block the batch just added) uses that number.

**A `path` that is not on the disk yet is a model the batch creates.** It
starts from the skeleton the catalogue publishes, and the file is written
when the model compiles —the same rule `save_model` follows. An agent does
not have to build the model in memory and write it somewhere else.

Returns the `genmsi.edit/1` document:

| Field | Type | Meaning |
|---|---|---|
| `schema` | string | `genmsi.edit/1` |
| `path` | string | The model path (`<memory>` for an inline document). |
| `applied` | number | How many commands the batch had. The batch is atomic: they are all applied, or none is. |
| `written` | boolean | Whether the model was written to the file. |
| `status` | string | `ok` or `diagnostics`, as in `genmsi.lint/1`. |
| `title` | string | The model title. |
| `blocks` | number | Executable statements the model compiles to. |
| `definitions` | object | The definitions the model declares, as in `genmsi.lint/1`. |
| `diagnostics` | array | The findings, as in `genmsi.lint/1`. |
| `created` | array | The ids of the blocks the batch created. |
| `source` | string | The model as text, when there was no file to write it to. The field is absent when the model was written. |

A model given as text (`model`) has no file to write to: the text of the
result travels back in `source`, and that text is the only place the edit
lives. A model given by `path` is written, and the file is the state.

### `app_state`

The tool takes no arguments.

It returns the `genmsi.live/1` document: **what the reader is looking at
right now**.

| Field | Type | Meaning |
|---|---|---|
| `schema` | string | `genmsi.live/1` |
| `path` | string or null | The **absolute path** of the model the app has open; null when there is none. |
| `pid` | number or null | The pid of the app that has it open; null when the app is not running. |
| `dirty` | boolean | Whether the reader has changes that are **not saved**. |
| `diagnostics` | array | What the app found in the model. |
| `model` | object or null | The model **as it is in memory**, when it differs from the disk. Null when the model of the disk is the model of the app: the agent reads that one from the file. |
| `run` | object or null | The last run the app made: `seed`, `until`, `start` and `report` (the `genmsi.report/1` document). |
| `proposal` | object or null | The proposal the person has to decide, or the one they decided: `id`, `state` (`pending`, `accepted` or `rejected`), `summary` (what the agent said it did) and `note` (what the person answered). Null when there is no proposal on the table. |

**The answer is never a failure.** When the app is not running, the answer is
the same document with `path` and `pid` null: the question has an answer
either way. A document left behind by an app that is gone says nothing: the
pid says whether the process is alive.

**Why the agent asks for it.** The app does not reload a model that changes
under it while the reader has changes that are not saved —the reader's work is
not thrown away by a change somebody else made— so the work of an agent that
wrote anyway would stay invisible on the canvas, and the reader's save would
overwrite it. The agent asks first, tells the reader, and sends the same edit
again once the model is saved. `apply_edit` and `save_model` **refuse** that
write on their own (see `model_locked` below): the refusal is mechanical, not
a rule the agent has to remember.

## Errors

A model that does not work is a **tool error**: the call succeeds as a call
and the result carries `isError: true` with a JSON body. There is no panic, no
empty report and no prose message an agent has to parse.

```json
{
  "error": "invalid_block",
  "message": "Queue is required",
  "path": "<memory>",
  "line": 1,
  "column": 662,
  "node": 2,
  "class": "Queue"
}
```

| Field | Type | Meaning |
|---|---|---|
| `error` | string | Stable class of the failure; an agent can branch on it. |
| `message` | string | What went wrong, in one sentence. |
| `path` | string | The model the failure belongs to. |
| `line` | number | 1-based line of the offending block in the `.gmdl` source. |
| `column` | number | 1-based column of the offending block in the `.gmdl` source. |
| `node` | number | `moduleid` of the offending block. |
| `class` | string | GPSS class of the offending block. |
| `command` | number | Index of the command that failed, when the failure is a command. |
| `op` | string | The `op` of the command that failed. |
| `name` | string | The name the agent gave the block the failure is about (its `caption`). |
| `property` | string | The property the command that failed was about. |

The fields that do not apply are absent — never `null`, never `0`. **The line
and the column point at the opening tag of the offending element of the
`.gmdl` file**, which is where an agent has to look; the reference models are
one very long line, so the column is the one that tells two blocks apart. The
line and the column are not the line of the GPSS program the original tool
generated.

A failure the parser finds while reading the document points at the element it
was reading: the opening tag of the `<module>` when the failure is inside one
(a duplicate port id, a malformed property), and the root element when it is
not (`node` is absent then). A file the server opens but cannot decode, for
example one that is not UTF-8, is an `invalid_model` with the same
`cannot read model: …` message the CLI prints; it carries no position.

`class` names the GPSS class of a compiled block, so it is filled only for
`invalid_block` and `unsupported_block`. The engine reports
`invalid_expression`, `invalid_graph` and `runtime_error` without a block, so
those carry no position at all.

The classes:

| Class | When |
|---|---|
| `invalid_arguments` | The call itself is wrong: both `path` and `model`, or no `model` to write. |
| `no_open_model` | The app has **no model open** and the call named neither a file nor a document, so there is no destination to guess. Name a `path`, or open a model in the app. |
| `invalid_command` | A command of the batch could not be applied. The batch is atomic, so no command is applied: the failure names the command that failed (its index in `command` and its `op` in `op`), the block it was about with the class and the name the agent gave it (`node`, `class`, `name`) and the property it was setting (`property`), and the model stays as it was. |
| `class_not_found` | The class named in `catalogue` is not one of the classes the catalogue lists. The `class` field carries the name that was asked for. |
| `model_not_found` | The file named by `path` does not exist. (`apply_edit` is the exception: it creates the model, see above.) |
| `model_locked` | The desktop app has **unsaved changes** on the model the call named, so nothing was written. The `path` field carries the model. The reader has to save it (or discard it) and the agent sends the same edit again: the batch is replayable, because the commands are the source of truth and not the text. The agent can also leave a **proposal** instead (`target: "proposal"`), which does not touch the disk. |
| `model_unreadable` | Reserved for a file the tool cannot read outside the model load path; a model the server loads goes through the same `parse_file` call the CLI uses, so its read failures are `invalid_model` (a missing file is still `model_not_found`). |
| `model_unwritable` | The file could not be written: the permissions, the disk, or the path. A model that does not compile is never written at all, so this class means the write itself failed. |
| `invalid_xml` | The document is not well-formed XML. The line and the column are known, the block is not. |
| `invalid_model` | The document breaks a structural or runnable rule (a name with whitespace, no `TERMINATE` and no `CLOCK`, an `UNLINK` without a label), or cannot be decoded at all (see above). Parse failures carry the position of the offending element; rule failures carry the position of the block that breaks the rule. |
| `invalid_block` | A block does not compile (a missing operand, an empty entity name). |
| `unsupported_block` | The engine does not implement that block class. |
| `invalid_expression` | An expression does not parse or does not evaluate. The engine reports it without a block, so it carries no position. |
| `invalid_graph` | The links do not form a valid graph. The engine reports it without a block, so it carries no position. |
| `runtime_error` | The run failed (for example a zero-time loop). The engine reports it without a block, so it carries no position. |

## Parity with the CLI

The MCP server is the same engine behind the same API (decision D7), so:

- The documents are the CLI's documents, byte for byte. `genmsi lint --json`,
  `genmsi simulate --json` and `genmsi trace --json` print exactly what the
  matching tool returns for the same model, the same seed and the same
  horizon.
- The accept/refuse split of the reference models is the CLI's split: twenty
  models run and five are refused, each with the same message class
  (`docs/cli.md` publishes the table).
- The five refused models fail as tool errors with the same text the CLI
  prints on stderr, plus the position of the block to fix.

## Decisions

The four questions of the Phase 3 assignment were answered by the coordinator
and are recorded as `D14` in [`DECISIONS.md`](DECISIONS.md):

1. The semantic model records the source position of every block
   (`genmsi-format`: `Block::source`). The writer never reads the field, so the
   engine keeps producing byte-identical `.gmdl` files.
2. The tools return the CLI's documents verbatim; `explain_model` has no CLI
   counterpart and publishes `genmsi.explain/1`.
3. Every tool takes the model as `path` or as an inline `model` document.
4. `explain_model` is the engine's `program_json` plus the GPSS glossary.

The `catalogue` tool is the one the agent calls **before** it writes a model:
the engine already knows every block class (`genmsi-format`: the catalogue),
and a client that cannot see it has to guess the element names of the file.
It publishes two documents of its own, `genmsi.classlist/1` and
`genmsi.class/1`; the catalogue the desktop application uses
(`genmsi.catalogue/1`) keeps its own shape, so the application does not
download the templates it does not use.

The `app_state` tool has no CLI counterpart either, for the same reason: the
file that says what the app is showing belongs to the app, not to the engine.
It publishes one document of its own, `genmsi.live/1` — and it is the **only**
one: the model the app has open travels in it, next to the `dirty` flag and
the last run, so the two facts cannot come from two files and disagree.

## The verb: `apply_edit`

Decision D21 records why the tool exists and why it speaks the engine's own
vocabulary. In short:

- the commands are `EditCommand`, the type the desktop application already
  speaks, and the schema of the tool is derived from it;
- the batch is atomic, and the failure names the command that failed;
- the model is checked before it is written, and it is written;
- the canvas belongs to the engine: the blocks the batch adds land in the free
  spot of their container.

The document it publishes, `genmsi.edit/1`, carries the lint of the model it
edited (the same fields and the same meanings as `genmsi.lint/1`) and what the
edit did: the commands applied, whether the model was written, the blocks the
batch created, and the text of the model when there was no file to write it
to.

## The entry point

The server runs as its own binary, `genmsi-mcp`, and as the subcommand
`genmsi mcp` (decision D15): both speak the same protocol over the same
stdio, and the CLI's subcommand is the one the installer registers.

## Configuring the server in a client

The server is the same everywhere: it is the same binary, the same engine and
the same protocol. What changes is the file each client reads and the key it
uses.

The installer registers the server with Pi and puts the agent's guide in
place:

```sh
./scripts/install-linux.sh --with-mcp
```

**Pi** — the command writes the file wherever that Pi reads it
(`~/.pi/agent/mcp.json`, and `.pi/mcp.json` in a trusted project):

```sh
pi mcp add genmsi -- ~/.local/bin/genmsi mcp
```

**Claude Desktop** — `~/.config/Claude/claude_desktop_config.json`, key
`mcpServers`:

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

**Cursor** — `~/.cursor/mcp.json`, key `mcpServers`:

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

## Two things that bite

**The working directory.** The server inherits the directory the client
launches it in, and a relative `path` is resolved against it: the same call
works in one client and fails in another. Pin it when the client can
(`pi mcp add genmsi --cwd /path/to/Genmsi -- …`), and use **absolute paths**
in the calls — `app_state` already returns one: the `path` of the
`genmsi.live/1` document is the absolute path of the model the app has open.

**The guide is a file, not the protocol.** The catalogue, the tool
descriptions and the published schema travel in the protocol, so an agent can
build a model with the MCP alone. The *procedure* (how a model is written,
which entities it has to declare, what the expression language is) lives in
the guide, and it is installed with the product:
[`skill/genmsi-modeling/SKILL.md`](../skill/genmsi-modeling/SKILL.md).
`skill/` is a **skills root**: copying its contents into `~/.agents/skills/`
is the whole installation.

```sh
cp -r skill/genmsi-modeling ~/.agents/skills/    # or: scripts/install-linux.sh --with-mcp
```

A write is refused for one more reason: while the desktop app has
**unsaved changes** on the model the call names, `save_model` and
`apply_edit` refuse with `model_locked` and write nothing: the app does not
reload a model that changes under it while the reader has changes that are
not saved. Ask `app_state` before you write, tell the reader, and send the
same call again once the model is saved.

## See also

- [`docs/cli.md`](cli.md): the documents the tools return, in full.
- [`docs/mcp-transcript.md`](mcp-transcript.md): an agent building, linting,
  simulating and tracing a model through the server.
- [`docs/DECISIONS.md`](DECISIONS.md): D7 (the tools), D13 (the documents) and
  D14 (the decisions of this phase).
