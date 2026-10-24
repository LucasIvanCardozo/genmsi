# genmsi MCP server

`genmsi-mcp` is the MCP server of the GenMSI engine (ROADMAP Phase 3): it serves
the engine to an AI agent over stdio with the six tools of decision D7 —
`simulate`, `lint`, `get_trace` and `explain_model`, plus `save_model`, which
writes a model the agent built, and `catalogue`, which tells the agent which
block classes exist and what the XML of a block of each one looks like. The
binary is a thin layer over `genmsi-core` and `genmsi-format`; it carries no
simulation logic (decision D2), and the documents it returns are the ones the
CLI publishes (decision D13).

```
genmsi-mcp            # speaks MCP on stdin/stdout
```

Standard output belongs to the protocol: the server writes nothing else there.
A failure that stops the server goes to standard error and the process exits
with `1`.

## The six tools

| Tool | What it does | Returns |
|---|---|---|
| `catalogue` | Says which block classes exist and what a block of one of them looks like: its ports, its properties and the XML of the block. | `genmsi.classlist/1` or `genmsi.class/1` |
| `lint` | Loads the model, validates it and compiles it. It does not simulate. It reports what it found. | `genmsi.lint/1` |
| `simulate` | Runs the model once and returns its GPSS report. | `genmsi.report/1` |
| `get_trace` | Runs the model with the event trace enabled and returns the records. | `genmsi.trace/1` |
| `explain_model` | Describes the model: its blocks, its operands and what each statement does. | `genmsi.explain/1` |
| `save_model` | Writes a model the agent built to a file, after checking that it compiles. | `genmsi.lint/1` |

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

**Call this tool before writing a model.** It is the one that saves the agent
from guessing: it says which classes exist, which ports (`ioid`) and which
properties (`name`, `type`, `value`) each class declares, and it gives the
XML of a block of the class, exactly as the file stores it.

Without `class` it returns the `genmsi.classlist/1` document:

| Field | Type | Meaning |
|---|---|---|
| `schema` | string | `genmsi.classlist/1` |
| `classes` | array | Every class the editor offers: `class` (`moduleclass`) and `title` (what the class does). |
| `skeleton` | string | The skeleton of a model: the smallest `.gmdl` document the format reads, with the model root and nothing else. |

With `class` it returns the `genmsi.class/1` document of that class:

| Field | Type | Meaning |
|---|---|---|
| `schema` | string | `genmsi.class/1` |
| `class` | string | `moduleclass` the blocks of this class carry. |
| `title` | string | What the class does. |
| `inputs` | array | Input ports: `ioid`, `name`, `caption`. |
| `outputs` | array | Output ports: `ioid`, `name`, `caption`. |
| `properties` | array | The properties the class declares: `name`, `hint`, `ty`, `subtype`, `unique` and the `options` of a listed property. |
| `template` | string | The XML of a minimal, valid block of this class. |

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

Returns the `genmsi.trace/1` document, the same document `genmsi trace --json`
prints. The trace has no filters (decision of the coordinator in Phase 2); the
document says `truncated: true` when the bound may have cut it.

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
| `glossary` | object | What each GPSS statement of the model does, keyed by class. Only the classes the model uses appear. |

The glossary is the vocabulary an agent needs to write a model: it says what
each statement does and which operands it takes (`Media`, `Cantidad`,
`Parametro`, ...). It covers the classes the engine implements; the wording
follows the semantics of `genmsi-core`.

### `save_model`

| Field | Type | Default | Meaning |
|---|---|---|---|
| `path` | string | — | Path of the `.gmdl` file to write. |
| `model` | string | — | The whole `.gmdl` document as text. |

Writes the model to the file and returns the `genmsi.lint/1` document of what
was written.

**The model is checked before it is written**: a model that does not load,
does not validate or does not compile is **not** written, and the failure is
the same one the other tools give, with the block and its position. The write
is **atomic**: the text goes to a temporary file next to the target and that
file is renamed over it, so a reader that watches the model (the desktop
application does) never sees half of one.

The tool writes the **whole document**, not a patch: read the model with
`explain_model`, change the text and hand the complete document back.

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
| `invalid_arguments` | The call itself is wrong: no model, or both `path` and `model`. |
| `class_not_found` | The class named in `catalogue` is not one of the classes the catalogue lists. The `class` field carries the name that was asked for. |
| `model_not_found` | The file named by `path` does not exist. |
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
(`genmsi.catalogue/1`) is left alone, so the application does not download
the templates it does not use.

## Not in this phase

Decision D7 says `genmsi mcp` runs the server. This phase delivers the server
as its own binary, `genmsi-mcp`, because the Phase 3 assignment limits the
edit paths to `crates/genmsi-mcp/`, `crates/genmsi-core/`,
`crates/genmsi-format/`, `docs/` and `odd/tasks/`, and D2 forbids the CLI from
depending on a second client crate. Wiring the server as a `genmsi mcp`
subcommand is a coordinator decision that needs the CLI path and a reading of
D2.

## See also

- [`docs/cli.md`](cli.md): the documents the tools return, in full.
- [`docs/mcp-transcript.md`](mcp-transcript.md): an agent building, linting,
  simulating and tracing a model through the server.
- [`docs/DECISIONS.md`](DECISIONS.md): D7 (the tools), D13 (the documents) and
  D14 (the decisions of this phase).
