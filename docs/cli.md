# genmsi CLI

`genmsi` is the command-line client of the GenMSI engine (ROADMAP Phase 2): the
engine is usable by a human at a terminal, by a script and by CI. The binary is a
thin layer over `genmsi-core` and `genmsi-format`; it carries no simulation logic
(decision D2).

Every subcommand writes **one** document to stdout and nothing to stderr unless it
fails. The document is deterministic, which is what makes it snapshot-testable
(ROADMAP Phase 2) and comparable across machines.

```
genmsi <command> [options]

genmsi lint     <model.gmdl> [--json]
genmsi simulate <model.gmdl> [--seed <n>] [--until <time>] [--json]
genmsi trace    <model.gmdl> [--seed <n>] [--until <time>] [--limit <n>] [--json]
genmsi run-all  <directory>  [--seed <n>] [--until <time>] [--json]
```

## Common options

| Option | Applies to | Meaning |
|---|---|---|
| `--seed <n>` | `simulate`, `trace`, `run-all` | Base seed of every random stream. Default `1`. The same seed reproduces the same run, bit for bit. |
| `--until <time>` | `simulate`, `trace`, `run-all` | Stop once the simulated clock passes `<time>`, instead of running to the model's own termination. The engine stops at the first event after the horizon, so `end_time` can be larger than `<time>`. |
| `--limit <n>` | `trace` | Maximum number of trace records the engine keeps. Default `100`. |
| `--json` | all | Emit the versioned JSON document instead of the human-readable form (see [`--json` documents](#json-documents)). |

Without `--until`, a model runs until its termination counter reaches zero. All
25 reference models that the original accepts do that on their own; the five the
original refuses never start (see [Reference models](#reference-models)).

## Exit codes

| Code | Meaning |
|---|---|
| `0` | Success: the command produced its document. |
| `1` | The model does not load, does not validate, does not compile or does not run. `run-all` uses it when at least one model of the directory failed. |
| `2` | Usage error: `clap` rejected the arguments, or the argument cannot be used (a `run-all` target that is not a directory). |
| `3` | `lint` only: the model is runnable but the compiler reported diagnostics. |

Nothing else is produced. `--help` and `--version` exit `0`.

Failure messages go to stderr as one line, prefixed with the model path so a log
line says which model failed:

```
$ genmsi simulate "fixtures/models/TP9__tp9.gmdl"
error: fixtures/models/TP9__tp9.gmdl: (node 68): Unlink: the operand B must not be null; UNLINK needs a label for the unlinked transactions
$ echo $?
1
```

Exit code `3` is reachable only when `Program::diagnostics` is non-empty, which is
the engine's channel for non-fatal compile observations. **No reference model
produces one.** The only diagnostic the engine emits today is the runtime
`TABULATE <name> has no TABLE definition`, which shows up in the `simulate`
document under `diagnostics` and does not change the exit code.

## Commands

### `lint`

Loads the model (`.gmdl` parsing plus structural validation), validates that it is
runnable (a `CLOCK`, or a `TERMINATE` with a positive operand `A`, and a labelled
`UNLINK`), and compiles it. It does not simulate.

```
$ genmsi lint fixtures/models/TP2__problema1.gmdl
model: fixtures/models/TP2__problema1.gmdl
title: Modelo1
status: ok
blocks: 52
definitions: functions 0, variables 0, random_variables 0, storages 4, tables 0, qtables 1
diagnostics: none
```

`blocks` counts the executable GPSS statements the model compiles to. `status` is
`ok`, or `diagnostics` with the messages listed under `diagnostics:`.

### `simulate`

Runs the model once and prints the GPSS report: header counters, the block table
with `ENTRY`/`CURRENT`/`RETRY`, and the facilities, queues, storages, user chains,
tables, savevalues and diagnostics the model created.

Numbers use the comma decimal separator, like the Spanish original. `--json` uses
JSON numbers.

### `trace`

Runs the model with the structured event trace enabled and prints one line per
record:

```
 2700.000  T1    #1    Generate       generate
 2700.000  T1    #14   Queue          enter
 2700.000  T1    #14   Queue          leave      -> 4
# trace stopped at the 8 record limit (raise --limit)
```

The columns are simulation time, transaction, report line, block class, event
(`generate`, `enter`, `leave`, `wait`, `terminate`) and detail. `--limit` bounds
how many records the engine keeps; when the bound was reached the output ends with
the note shown above, so a truncated trace is never mistaken for a complete one.
There are no filters by transaction, block or time range.

### `run-all`

Runs every `.gmdl` file of one directory level (not recursive), sorted by file
name, with the same seed and horizon, and prints one line per model:

```
$ genmsi run-all fixtures/models
OK    TP2__problema1                                 3600.0  terminated
FAIL  TP9__tp9                                   (node 68): Unlink: the operand B must not be null; ...
25 models: 20 ok, 5 failed
$ echo $?
1
```

A failure does not stop the sweep. On `fixtures/models` the expected result is
`20 ok, 5 failed` and exit `1`; on a directory of valid models it exits `0`.

## `--json` documents

Every document is a **versioned envelope**: the field `schema` names the document
and its major version, and the body fields follow at the top level.

```json
{
  "schema": "genmsi.report/1",
  "title": "Puerta Banco",
  "...": "body fields"
}
```

| Schema id | Command |
|---|---|
| `genmsi.lint/1` | `lint` |
| `genmsi.report/1` | `simulate` |
| `genmsi.trace/1` | `trace` |
| `genmsi.run-all/1` | `run-all` |

**Stability promise.** Within one major version (`.../1`) changes are *additive*:
a new field may appear, and an existing field keeps its name, type and meaning. A
field is never renamed, retyped or removed without bumping the number. Consumers
must ignore fields they do not know.

Entity maps are JSON objects keyed by entity name, iterated in sorted order, so the
document is byte-identical for the same binary, model, seed and horizon. Paths
appear exactly as the user wrote them on the command line.

### `genmsi.lint/1`

| Field | Type | Meaning |
|---|---|---|
| `schema` | string | `genmsi.lint/1` |
| `path` | string | Model path, as given |
| `status` | string | `ok` or `diagnostics` |
| `title` | string | Model root name |
| `blocks` | number | Executable statements the model compiles to |
| `definitions` | object | `functions`, `variables`, `random_variables`, `storages`, `tables`, `qtables`, each a count |
| `diagnostics` | array of string | Non-fatal compiler observations |

### `genmsi.report/1`

| Field | Type | Meaning |
|---|---|---|
| `schema` | string | `genmsi.report/1` |
| `title` | string | Program title |
| `seed` | number | Seed used |
| `start_time`, `end_time` | number | Simulated time span |
| `end_reason` | string | `terminated`, `no_events` or `time_limit` |
| `transactions` | number | Transactions created |
| `events` | number | Events processed |
| `blocks` | array | One row per executable statement, in report order |
| `entities` | object | Entity statistics (below) |
| `diagnostics` | array of string | Compiler and runtime diagnostics |

`blocks[]`:

| Field | Type | Meaning |
|---|---|---|
| `line` | number | 1-based line the report shows |
| `node` | number | `moduleid` of the block in the model |
| `class` | string | Block class (`Generate`, `Queue`, `Advance`, ...) |
| `label` | string or null | GPSS label, when the block has one |
| `operands` | string | Operands rendered the way the report shows them |
| `entries`, `current`, `retries` | number | `ENTRY`, `CURRENT` and `RETRY` counters |

`entities` holds one object per entity kind, each keyed by entity name:

| Key | Entity fields |
|---|---|
| `facilities` | `name`, `entries`, `busy_time`, `busy`, `available`, `retry`, `inter` |
| `storages` | `name`, `capacity`, `in_use`, `area`, `min_use`, `max_use`, `entries`, `retry`, `delay` |
| `queues` | `name`, `content`, `max`, `entries`, `zero_entries`, `total_time`, `area`, `retry` |
| `chains` | `name`, `content`, `max`, `entries`, `total_time`, `area` |
| `logic` | Logic switch name → boolean state |
| `savevalues` | Savevalue name → value |
| `tables`, `qtables` | `name`, `first`, `width`, `intervals`, `samples`, `sum`, `sum_squares`, `max_value`, `frequencies`, `overflow` |

`area` is the content × time integral; the averages of the text report are derived
from it, so a consumer that wants `AVE.CONT.` divides `area` by
`end_time - start_time`.

### `genmsi.trace/1`

| Field | Type | Meaning |
|---|---|---|
| `schema` | string | `genmsi.trace/1` |
| `title` | string | Program title |
| `seed` | number | Seed used |
| `start_time`, `end_time` | number | Simulated time span |
| `end_reason` | string | `terminated`, `no_events` or `time_limit` |
| `limit` | number | Record limit the run used |
| `records` | number | Records kept |
| `truncated` | boolean | True when `records` reached `limit` |
| `events` | array | Trace records, in simulation order |

`events[]`: `time` (number), `xact` (number, the transaction id), `block` (number,
0-based statement index: the human output prints `block + 1`), `class` (string),
`label` (string or null), `event` (`generate`, `enter`, `leave`, `wait`,
`terminate`), `detail` (string, possibly empty).

### `genmsi.run-all/1`

| Field | Type | Meaning |
|---|---|---|
| `schema` | string | `genmsi.run-all/1` |
| `dir` | string | Directory scanned, as given |
| `seed` | number | Seed applied to every model |
| `until` | number | Horizon applied to every model; absent when none was given |
| `models_total`, `models_ok`, `models_failed` | number | Counts |
| `models` | array | One entry per model, in scan order |

`models[]`: `name` (file stem, spaces preserved), `path`, `status` (`ok` or
`failed`), and either `end_time` plus `end_reason` (when it ran) or `error` (the
engine message, when it failed).

## Determinism

- Same binary, model, seed and horizon → byte-identical stdout.
- Entity maps and directory scans are sorted; nothing depends on hash order.
- `--json` prints JSON numbers; the human report keeps the original's comma
  decimals. Neither depends on the machine locale.
- No timestamps, hostnames or absolute paths are added by the CLI.

`crates/genmsi-cli/tests/` pins one snapshot per subcommand (`INSTA_UPDATE=always
cargo test -p genmsi-cli` refreshes them) and checks that every reference model
runs or is refused as the original refused it.

## Reference models

`fixtures/models/` holds the 25 reference models. Twenty run; five are the models
whose oracle baseline is an error message, and the CLI fails with the same class:

| Model | `error:` message class | Exit |
|---|---|---|
| `Parcial 8-11-21__ParcialMikelModelos` | `invalid block` (definition name with whitespace) | 1 |
| `Problema 2__puenteLevadizo` | `no TERMINATE block with operand A and no CLOCK block` | 1 |
| `TP6__Clase 6_0_TestGate` | `no TERMINATE block with operand A and no CLOCK block` | 1 |
| `TP9__hack` | `no TERMINATE block with operand A and no CLOCK block` | 1 |
| `TP9__tp9` | `the operand B must not be null` (labelled `UNLINK` output) | 1 |

`TP9__hack` is the model where the original crashes with an internal error
(`Se intentó obtener la linea del fuente de un bloque inexistente`); the engine
reports the modelling error instead of replicating the crash (Phase 1 decision).

## Examples

```bash
# Validate a model in CI; a broken model fails the job.
genmsi lint model.gmdl

# Reproduce a run exactly and store the report.
genmsi simulate model.gmdl --seed 7 > report.txt

# Compare two designs over the same horizon.
genmsi simulate before.gmdl --until 3600 --json > before.json
genmsi simulate after.gmdl  --until 3600 --json > after.json

# Smoke-test a directory of models: 0 when all of them run.
genmsi run-all models/

# Inspect the first events of a model.
genmsi trace model.gmdl --limit 25
```
