---
name: genmsi-modeling
description: Read this when an agent has to build, read or change a GenMSI model (a `.gmdl` document) with the GenMSI MCP server.
---

# GenMSI modelling (agent guide)

This file is the procedure and the vocabulary for building a GenMSI model with the GenMSI MCP server.
It is the only sanctioned place to learn the format: the catalogue holds the facts of a class, the tools hold the interface, and this file holds the rest — read it instead of the engine's Rust source.

## The workflow

1. **One catalogue call.** `catalogue({ classes: ["Generate", "Queue", "FacilityUse", "Terminate", "Clock"] })` — every class the model needs, in **one call**, not one call per class. The answer carries, for each class: the title and the description, the input ports (`ioid`, `name`), the output ports (`ioid`, `name`), the properties (`name`, `hint`, `ty`, `subtype`, and the `options` of a listed property), the entity kinds the class `declares` and the ones it `uses`, the XML template of a minimal valid block of that class, and the **contract** of the class: what it does to the flow of the transaction, what it blocks, the rules of taking and releasing, and the traps.
   - `catalogue()` with no argument returns the list of every class and the skeleton of a model.
   - `catalogue({ class: "Generate" })` returns one class.
2. **Design before writing.** Decide the flow of the transactions, the entities the model needs and the class of every block. The catalogue tells you what a class does; the design is yours.
3. **Build the model with `apply_edit`.** One call, one batch of the engine's editing commands: an `addBlock` per block, a `connect` per link, a `setProperty` per operand. The engine gives every new block a place on the canvas, checks that the model compiles and writes it. The batch is **atomic**: if a command fails, no command is applied and the error names the command that failed, the block it was about (with its class and **the name you gave it**) and the property it was setting. See *The verb* below. You do not have to name a file: when you name neither a `path` nor a `model`, the batch works on **the model the app has open**.
4. **`lint({ path })`** — the model as it is now on disk. It loads, validates and compiles the model and reports what it found. A model that does not compile is a tool error, not an empty report. The lint also names the **property a block carries that its class does not declare** (`Tamano` for `Tamaño`, with the name it probably meant): a typo is silent in the engine **when the model was written by hand**, so the lint is where it shows up — as a finding of a model that compiles, because a missing *operand* is a refusal and never reaches the findings. **Through the editing path it is not silent**: `setProperty` with a property the class does not declare fails, and the failure names the property and the block.
5. **`explain_model({ path })`** — the model as the document writes it: every block with its class, its properties **by name** and its ports, and the `edges` that wire the blocks. It is the cheapest way to see that the model is the model you meant. (It compiles the model first, so read it after `lint`.) It also carries the **map** between the compiled statements and the blocks of the document: one row per statement, with the block, the name you gave it and where it is in the `.gmdl`.
6. **`simulate({ path, seed })`** — the GPSS report. Read its `blocks` (the statements the model compiles to, with their counts) and its `entities` (the queues, facilities and storages it used).
   - **Always pass the `seed` explicitly.** The default is drawn at random, so two calls with the same arguments are two different runs. The seed you pass is the seed you can repeat.
7. **`get_trace({ path, seed, ... })`** — the events of a run, and the **summary** of it. The filters (by class, by block, by transaction, by window of time) are applied when the event is recorded, so the `limit` counts the events that pass them: ask for the events of one class and you get those, not the first ones of the run. With `summary: true` the answer is the summary of the run — what each block did, how many times a transaction entered it and when the first and the last one entered it. See *Reading a run* below.
8. **`app_state()`** — what the reader is seeing right now (see *The person reading* below): the model the app has open, whether that file has changes that are not saved, and the **proposal** on the table, if you left one.
9. **`save_model({ path, model })`** — only for a document you wrote by hand: it takes the **whole** document as text. To change a model that is already on disk, use `apply_edit` on its `path`, not `save_model`.

The order matters: `lint` before `simulate` (a model that does not compile is a tool error, not an empty report), and `simulate` before you are done (never keep a model you have not run).

## The verb

`apply_edit` is the tool that changes a model. It takes the model (`path` or
`model`) and a **batch** of commands, and applies them in order:

```json
{
  "path": "/tmp/modelo.gmdl",
  "commands": [
    { "op": "addBlock", "class": "Generate", "container": 0, "x": 0, "y": 0 },
    { "op": "addBlock", "class": "Terminate", "container": 0, "x": 0, "y": 0 },
    { "op": "connect", "from": 1, "fromIoid": 1, "to": 2, "toIoid": 1 },
    { "op": "setProperty", "id": 1, "property": "Media", "value": "60" }
  ]
}
```

- **The commands are the engine's own**, and the schema the tool publishes is
  the one of that type:

| Command | Fields | What it does |
|---|---|---|
| `addBlock` | `class`, `container`, `x`, `y` | Creates a block of a catalogue class inside a container. |
| `connect` | `from`, `fromIoid`, `to`, `toIoid` | Wires an output port to an input port. |
| `disconnect` | `container`, `from`, `fromIoid` | Removes the link that starts at a port of a container's child. |
| `setProperty` | `id`, `property`, `value` | Sets a declared property. |
| `setCaption` | `id`, `caption` | Names a block (the field is `caption`, not `value`). |
| `moveBlock` | `id`, `x`, `y` | Moves a block, keeping its size. |
| `removeBlock` | `id` | Removes a block, its descendants and the links that touch them. |
| `removeBlocks` | `ids` | Removes several blocks as ONE operation. |
| `setLayouts` | `placements` | Places many things at once. |
| `movePort` | `block`, `ioid`, `input`, `x`, `y` | Moves a door of a container. |
| `removePort` | `block`, `ioid`, `input` | Removes a door of a container. |
| `paste` | `ids`, `dx`, `dy` | Copies blocks with fresh ids and remapped links. |

- **The batch is atomic.** A command that fails undoes the ones that were
  applied: the model stays as it was, and the error (`invalid_command`) says
  which command failed — its index and its `op`. Correct that command and
  send the batch again: the commands are the source of truth, not the text.
- **The ids are predictable.** The blocks a batch creates take the next free
  ids, in the order of the commands: a model that starts from the skeleton
  numbers them 1, 2, 3... A `connect` that wires a block of the same batch
  uses that number, and the answer carries the ids it created in `created`.
- **A `path` that is not on the disk yet is a model the batch creates.** It
  starts from the skeleton the catalogue returned, and the file is written
  when the model compiles. You do not have to build the model in memory and
  write it somewhere else.
- **The canvas is the engine's business.** Every block the batch adds lands in
  the free spot of its container, like a drop in the application. With
  `"layout": "explicit"` the `x` and the `y` of the command are used instead.
- **`target` says where the result goes.** `"file"` (the default) writes the
  model to the file the batch edited. `"proposal"` leaves it in the staging
  area of the app, **without touching the disk**, and the reader sees it on
  their canvas and decides — see *The person reading* below. A `summary` of
  what you did travels with the proposal.

## Reading a run

A run has two faces: the **events** (what happened, one by one) and the
**summary** (what each block did). The trace is the first and the summary is
the second, and most of the questions an agent asks are about the second.

**The filters.** `get_trace` takes `classes` (the classes of the blocks),
`nodes` (the blocks of the document, by their `moduleid`), `xact` (a
transaction), and `from` and `to` (a window of simulated time). They are
applied **when the event is recorded**, so the `limit` counts the events
that pass them: asking for the events of one class gives those events, not
the first ones of the run. The answer says which filter it used.

**The summary.** `get_trace({ summary: true })` answers the summary of the
**whole run** (not of the events that were kept, and not cut by the limit):
one row per statement, with the block of the document, its class, how many
times a transaction entered it and when the first and the last one entered
it. A block nobody entered is in it, with zero entries.

**The map.** The trace numbers the *statements* the model compiles to (a
`FacilityUse` is five statements), and the summary does the same. The
`map` of `explain_model` is the bridge: one row per statement, with the
`node` of the block of the document, the **name you gave it** and where it
is in the `.gmdl` (its line and column). With it, "what block is statement
218" is one lookup: the event's `node` is the block, and the map says where
that block is in the file.

The summary and the map are crossed by their number: the `block` of a row of
the summary is the `statement` of a row of the map. The summary says how
many times the flow went through there; the map says which block that was,
what you called it and where it is in the file.

**Trying a batch without writing it.** `apply_edit({ dry_run: true })`
applies the commands and compiles the model, and **writes nothing**: the
answer says what would have happened (`written: false`) and carries the
model it would have written, in `source`. A batch that would fail fails the
same way. Use it before a big batch, and when the model is not yours to
overwrite.

## The person reading

The desktop application and the MCP are two ends of the same loop. Two roads
connect them, and both of them end with the reader seeing your work on the
canvas.

**Where you work.** The tools do not need a path: when you ask for a model
and name neither a file nor a document, they work on **the model the app has
open** — the one the reader is looking at. That is what you mean, unless you
say otherwise. With no app running, or with no model open, the tools that
take a model answer `no_open_model` and tell you to name a file: there is no
destination to guess. (`app_state` never fails: it answers with its document
and `path: null` when there is no app — a `null` path is what "nobody has a
model open" looks like.)

**`app_state()`** says what the reader is seeing right now: the model the
application has open, whether they have changes that are **not saved**, what
it found in the model, the last run it made (the report and the seed it ran
with), and **the proposal on the table**. Without the application running it
answers with `path: null`.

The tools do not need a path: when you ask for a model and name neither a
file nor a document, they work on **the model the app has open** — the one
the reader is looking at. That is what you mean, unless you say otherwise.
With no app running, or with no model open, the tools that take a model
answer `no_open_model` and tell you to name a file: there is no destination
to guess. (`app_state` never fails: it answers with its document and
`path: null` when there is no app — a `null` path is what "nobody has a
model open" looks like.)

**Look before you write.** If the model you are about to change is the one
the application has open, ask `app_state()` first:

- the reader has **no** changes pending → write it (the default target,
  `"file"`): the application picks the change up on its own, within half a
  second, and the reader sees it on the canvas;
- the reader **has** changes pending → **do not write**. Say so and wait:
  *"the application has unsaved changes on `modelo.gmdl`; shall I wait for you
  to save it?"* Their work is not on the disk, and writing over the file would
  leave your work invisible on the canvas and theirs overwritten.

**`apply_edit` and `save_model` refuse that write for you.** When the
application has unsaved changes on the model you named, they answer
`model_locked` and **write nothing**. That error is not a failure of the
model: it is the application saying "this one is being edited".

**The proposal: the road that stays open.** Leave a **proposal** when the
reader is busy, or when you want the work reviewed before it lands: the same
call with `target: "proposal"` and a `summary` of what you did. The model
does **not** touch the disk. It waits in the staging area of the app, and the
app puts it on the reader's canvas — dirty, and undoable — so they see
exactly what you built and decide: *Aceptar* writes it to the file,
*Descartar* throws it away. The note they leave when they discard it comes
back to you.

The proposal waits too, while the reader has changes that are not saved: the
app does not put a proposal on the canvas over the work of the reader. It is
adopted when they save, and until then `app_state()` says `pending`.

**The verdict comes back in `app_state()`.** The `proposal` field carries the
identity of the proposal, its `state` (`pending`, `accepted` or `rejected`)
and the `note`. A proposal that is still `pending` means the reader has not
decided yet: wait for it, and do not send the same batch again — the
proposal is the batch, replayed on the reader's canvas.

**With the app closed, nobody decides.** The verdict lives in the document
of the proposal, beside the model it carries, and the answer of the tool
that left it says where that document is (`proposal.document`). Read it when
there is no app to ask: its `state` is `pending` until the app takes the
proposal and the person decides.

And if the reader asks you what you did, `app_state()` also tells you whether
your change actually landed on their canvas.
- **The model is checked before it is written.** One that does not compile is
  **not** written, and the failure is the compiler's, with the block and its
  position. The file keeps the model it had.
- The answer is the `genmsi.edit/1` document: the lint of the model it edited
  (the same fields as `genmsi.lint/1`), the commands applied, whether the
  model was written, and the blocks the batch created. With `model` (no
  file) the text of the result comes back in `source`.

`apply_edit` and `save_model` are two ways to write, and they do not overlap:
`apply_edit` changes a model **block by block**, `save_model` writes a
document **you wrote**. Prefer `apply_edit`: it carries the class names, the
ports and the properties of the engine, it checks what it writes, and it never
leaves a half-built model behind.

## The document

A `.gmdl` file is one XML document: a root, the (empty) custom classes, and the modules.

| Element | What it is |
|---|---|
| `<gmodel versioncode="1.00.2008.03.23">` | the file |
| `<gclasses/>` | the custom macros of the original editor; the engine does not interpret them — do not use them |
| the root `<module moduleid="0" moduleclass="main" owner="0" kind="C">` | the model; it owns the `<links>` that wire the blocks and has no ports |
| a block `<module moduleid="N" moduleclass="Class" owner="0" kind="A">` | one block of that class; `N` is unique in the model; `owner="0"` says the model root holds it |
| `<props>` | the properties of the class, one `<property>` each |
| `<inputs>` / `<outputs>` | the ports of the class, one `<input>` / `<output>` each, with the `ioid` the catalogue publishes |
| `caption` / `name` | the label of the block on the canvas and its name; free text |

### The properties

```xml
<property name="Media" hint="..." value="60" type="TSNA" subType="SNA" visible="1" unique="0"/>
```

- `name` must be the catalogue's name, verbatim (`StorageDef` declares `Tamaño`, accent and all).
- `value` is what the block does: a number, an entity name or an expression.
- `type` and `subType` are not decoration: the format reads them to decide whether the property **declares** an entity (`EntityDef`) or **uses** one (`EntityUse`). This is the whole entity rule; see *Entities* below.
- `hint`, `visible` and `unique` are optional; copy them from the template when you have one.
- **A property the block does not set is read as zero (or empty), not as the default the template carries.** The template of a class holds the default value of every property (`Media` is 60, `Tamaño` is 10), but the engine reads what the block says: a `StorageDef` without a `Tamaño` keeps no capacity, and a `Generate` without a `Media` arrives at time 0. Copy the template, or set every property the class needs.
- The order of the `<property>` elements does not change the model: the engine reads the properties **by name**. (The writer replays the order it found, so keep the schema order when you rewrite a file you did not write yourself.)
- Some text in the file is CP1252 percent-encoded (`%20` for a space, in the hints and in some entity names of the reference models). The catalogue's template keeps that text; plain text is read correctly, so write plain names and values.

### How two blocks are connected

A link is a `<link>` element inside the `<links>` of the container that owns the blocks — for the blocks of the model, that is the model root.

```xml
<link frommoduleid="2" fromioid="1" tomoduleid="3" toioid="1"/>
```

- `frommoduleid` is the block that **sends**; `tomoduleid` is the block that **receives**.
- `fromioid` and `toioid` are the `ioid` of the port, as the catalogue publishes it **for that class**. The ids are per class and are not "1 and 2" by convention: `Generate` has a single output with `ioid="1"`, and most other classes have `Entrada1` with `ioid="1"` and `Salida1` with `ioid="2"`.
- A link starts at an **output** port and ends at an **input** port. A block with no output port (a `Clock`, a `StorageDef`, a `Function`) can never be the source of a link: the engine refuses the model with `link starts at missing output port 1`.
- The flow of the transactions is the flow of the links. A block whose output is not linked sends the transaction nowhere.
- A nested container owns its own `<links>`. A link that crosses a container uses the container's own ports.

### A worked example

Five blocks: a clock, a generator, a queue, a facility and a terminate. The queue `Cola` and the facility `Servidor` are created by use; nothing declares them.

```xml
<gmodel versioncode="1.00.2008.03.23"><gclasses/><modules>
<module moduleid="0" moduleclass="main" owner="0" caption="Modelo" name="Modelo1" kind="C" genprefer="1,1,1,1,1,0,0,0,0,0,0,0,0" exehead="" nonexehead="" exefoot="" nonexefoot=""><links><link frommoduleid="2" fromioid="1" tomoduleid="3" toioid="1"/><link frommoduleid="3" fromioid="2" tomoduleid="4" toioid="1"/><link frommoduleid="4" fromioid="2" tomoduleid="5" toioid="1"/></links><inputs/><outputs/></module>
<module moduleid="1" moduleclass="Clock" owner="0" caption="Clock" name="Clock" kind="A"><props><property name="Frecuencia" value="3600" type="TSNA" subType="SNA"/></props></module>
<module moduleid="2" moduleclass="Generate" owner="0" caption="Generate" name="Generate" kind="A"><outputs><output ioid="1" name="Salida1" caption="" mylabel=""/></outputs><props><property name="Media" value="60" type="TSNA" subType="SNA"/><property name="Modificador" value="30" type="TSNA" subType="SNA"/></props></module>
<module moduleid="3" moduleclass="Queue" owner="0" caption="Queue" name="Queue" kind="A"><inputs><input ioid="1" name="Entrada1" caption="Ingreso" mylabel=""/></inputs><outputs><output ioid="2" name="Salida1" caption="" mylabel=""/></outputs><props><property name="Queue" value="Cola" type="EntityUse" subType="Queue"/></props></module>
<module moduleid="4" moduleclass="FacilityUse" owner="0" caption="FacilityUse" name="FacilityUse" kind="A"><inputs><input ioid="1" name="Entrada1" caption="Ingreso%20al%20recurso" mylabel=""/></inputs><outputs><output ioid="2" name="Salida1" caption="" mylabel=""/></outputs><props><property name="Facility" value="Servidor" type="EntityUse" subType="Facility"/><property name="Queue" value="Cola" type="EntityUse" subType="Queue"/><property name="Media" value="10" type="TSNA" subType="SNA"/><property name="Modificador" value="5" type="TSNA" subType="SNA"/></props></module>
<module moduleid="5" moduleclass="Terminate" owner="0" caption="Terminate" name="Terminate" kind="A"><inputs><input ioid="1" name="Entrada1" caption="Ingreso" mylabel=""/></inputs><props><property name="Unidades" value="1" type="TSNA" subType="SNA"/></props></module>
</modules></gmodel>
```

With `seed: 42` this model: one transaction arrives, joins `Cola`, takes `Servidor`, waits 9 units and terminates. The report says `end_time` 79 and the queue `Cola` has 2 entries.

## Entities

Two kinds of entity, and the difference is the `type` of the property that names them.

| | Declared | Used |
|---|---|---|
| `type` | `EntityDef` | `EntityUse` |
| what it does | the entity exists: the model knows its capacity, its frequencies, its value | the entity comes into being where it is used |

The catalogue says which is which for every class: `declares` lists the kinds the class declares, `uses` the kinds it uses.

These must be declared before they are used:

| Kind | Declared by |
|---|---|
| Storage | `StorageDef` (the capacity, `Tamaño`) |
| Table | `Table` |
| Function | `Function`, `FunctionDist`, `FunctionExpon` (and the multifurcations, which declare the functions they need) |
| Variable | `Variable`, `VariableA`, `VariableN` |
| Logic switch | `InitialLS` |
| Label | `Match` (its `RotuloPropio`) |
| Queue table | `QTable` |

The last four are only checked when the model declares that kind of entity somewhere; the first three are always checked.

These are created by use, and are never declared: queues, facilities, user chains, transaction groups, numeric groups, parameters, random seeds and save values.

Two that are easy to mix up:

- a **save value** is created where it is used; `InitiaSV` only sets its initial value.
- a **label** is declared by the `Match` that carries it (`RotuloPropio`); `RotuloMatch` uses the conjugate one.

And one that is not declared but has a name:

- a **user chain** is named by the `Cadena` property of the `Link` and the `Unlink` blocks, and the two blocks name the same chain when they are about the same chain. The name is yours: `setProperty` on `Cadena` renames it, and nothing has to declare it. (The name a block gets by itself, the `A_Queue` of a connection, is a different thing: it is the label of the connection, the `LOC` of the statement.)

Two rules about names: an entity name may not contain whitespace or `:`; and the check only fires for a plain name — a computed name (`*Cuadra`) or a placeholder (`[storage]`) is not checked.

`lint` reports a missing declaration as a finding and the model still compiles; `simulate` refuses the run: `the storage 'SalaEspera' is used but no StorageDef declares it`.

## Expressions

The value of a property that carries an expression is written in the language of the engine.

| Form | Meaning |
|---|---|
| `12`, `12.5` | a number |
| `a+b`, `a-b`, `a*b`, `a/b` | arithmetic |
| `a#b` | **multiplication** — `#` is not division: the original's editor has no `*` key, it offers `/`, `%`, `#` and `@` |
| `a@b` | modulo |
| `( )` | grouping; `-a` is the negation |
| `70%` | the editor's per-mille form: it evaluates to **700** |
| `P:name` | a parameter of the transaction |
| `X:name` | a save value |
| `V:name` | a variable |
| `FN:name` (or `F:name`) | a function |
| `Q:name` | the content of a queue |
| `S:name` | the units in use of a storage |
| `R:name` | 1 while the facility is free |
| `CH:name` | the size of a user chain |
| `LS:name` (or `L:name`) | a logic switch |
| `N:name` | the transactions that passed a label |
| `W:name` | the transactions waiting at a label |
| `*name` | indirect: the entity whose name is the **value** of the parameter `name` |
| `name` | a bare name is a parameter read |
| `AC1` | the model clock |
| `C1` | the absolute clock |
| `TC1` | the remaining termination count |
| `RN3` | a uniform draw from the random stream 3 |
| `[name]` | a placeholder: the value of the enclosing container, resolved at compile time |

`:` and `$` are the same separator (`P:name` and `P$name` parse alike; the generated GPSS text uses `$`).

A value that depends on the state of the model is written with those prefixes: `Q:Cola`, `S:SalaEspera`, `AC1`, `P:Edad`.

Do not mix the two percentage forms: in an expression `70%` is 700, but in `Transfer.Prob` a literal with `%` is a fraction (`85%` → 0.85) while a bare number is per-mille (`100` → 10%).

## Runtime semantics worth knowing

- **The clock is discrete.** The simulated time is whole units and a delay is truncated, not rounded: a delay of `10.7` moves the transaction 10 units. Choose the time unit before the numbers.
- **The run ends when the start counter reaches zero.** `start` is a parameter of the run (the option of `simulate` and `get_trace`), not of the model: the model does not declare it.
- **A model needs a way to end**: a `Clock` or a `Terminate` with a positive operand. Without one the engine refuses the model.
- **The `Clock` is the horizon.** It takes one unit of the start counter every `Frecuencia` units, so a `Clock` with `Frecuencia=60` ends a run with `start=1` at time 60, and with `start=2` at time 120. The `Clock` compiles into a generator and a terminate: the report shows two rows for it.
- **`Terminate.Unidades`** takes that many units from the counter for every transaction that arrives. `Unidades=0` never fires, and the class declares no default for it: a terminate without an operand never ends the run.
- **A generator creates as many transactions as `Cantidad` says.** An **empty** `Cantidad` creates them forever, a number limits the total, and **zero** creates none. The `OffSet` moves the first arrival.
- **The chain of delay is served by priority, and then by arrival.** The waiting transactions of a facility and of a storage are served by priority (descending) and, among the ones of the same priority, in the order they arrived.
- **`Match` needs two blocks.** The rendezvous pairs a block with the block that carries the conjugate label (`RotuloPropio` of one, `RotuloMatch` of the other); a block with no conjugate never pairs and its transaction waits there (the trace shows a `backup`).
- **A `Clock` has no output port**, so it cannot send a transaction anywhere: it is the horizon of the run, not an event. An event at a given time is a `Generate` with an `OffSet`, or a `Logic` that a `GateL` waits for.
- **A `Terminate` with `Unidades=0` removes the transaction and never ends the run**; with a number it takes that many units from the counter of the run for every transaction that arrives (the run ends when the counter reaches zero).
- **A `Table` tabulates an expression.** Its `Atributo` is evaluated at the `Tabulate` block, not when the table is declared: `Atributo="AC1-P:HoraLlegada"` records the time each transaction waited.
- **A generator does not arrive at time 0**: the first transaction arrives one interval later — unless `OffSet` shifts it, and then the first arrival is *at* the offset (`OffSet=120` arrives at time 120, not at 130). **An `OffSet` of zero arrives at time 0**; an `OffSet` that is empty leaves the first arrival drawn from the mean. A numeric `Modificador` is a symmetric spread, `uniform(Media-Modificador, Media+Modificador)`; `Modificador=0` means exactly `Media`; a **function** as the modifier multiplies the mean (`Media * f(u)`), the idiom for an exponential service. **With a spread, the first interval is drawn too**, like the ones that follow: the original does that and so does the engine (`crates/genmsi-core/tests/primera_llegada.rs` pins the three cases).
- **`FacilityUse` is five statements**: queue, seize, depart, advance, release. The transaction waits in the `Queue` while the facility is busy.
- **`Queue` joins, it does not dequeue**: the transaction enters the queue and goes on its way; it stays in the queue until a `Depart` takes it out. A queue whose model has no `Depart` never empties, and the report says so (`content` grows, `zero_entries` stays at zero).
- **`Match` is a rendezvous**: the transaction waits at the block (the trace shows a `backup` event) for the transaction that carries the conjugate label; the pair then continues together — **both** of them, so **both** keep whatever they had taken. A rendezvous is not a handover of one item to the other.
- **`Transfer` splits**: the first output takes the share `Prob`, the second takes the rest.
- **A storage needs its capacity**: `StorageDef` declares it. `Enter` takes units and keeps them until a `Leave` gives them back; a transaction that does not fit **waits at the `Enter`**, and a `Leave` never waits. `StorageUse` is the pair in one block: it joins the queue, takes the units, leaves the queue, waits the `Media` and gives the units back. The unit is **held, not consumed**, so a storage of capacity N admits N transactions at a time and the next one waits.
- **The gate blocks** (`GateF`, `GateL`, `GateS`, `GateM`): the pass goes to `Salida1`; when `Salida2` is not connected, the transaction waits in the block and every change of the state retries the waiters.
- **`Split` creates copies**: the original continues to its own output and the copies go to `Salida2`.


## Probe before you rely on it

A class whose semantics you have not seen is worth one probe, not twenty.

1. Build a model of two or three blocks: a `Clock`, the class itself and a `Terminate`. Use `apply_edit` with the skeleton the catalogue returned: `{ "op": "addBlock", ... }` for each block, `{ "op": "connect", ... }` for each link. The answer carries the text of the model in `source`.
2. `lint({ model })` — it must compile.
3. `simulate({ model, seed: 42 })` and read the `blocks` of the report: the rows are the statements the class compiles to, with their operands and their counts.
4. `get_trace({ model, seed: 42, limit: 40 })` — the events in order, with the time, the block and the detail (`-> 13` is the block the transaction goes to).
5. Change one operand and run again with the same seed: the difference is what the operand does.

Probe the classes whose behaviour you are not sure of, not the ones the catalogue already describes.

## Traps

- **A model the reader is editing.** `apply_edit` and `save_model` answer `model_locked` when the desktop app has unsaved changes on the model you named, and they write nothing. That is not a broken model: it is the reader's work. Save it and send the same batch again.
- **A link that starts at a port the block does not have.** The engine refuses the model (`link starts at missing output port 1`). The `ioid` of a port is the one the catalogue publishes for that class.
- **An entity used and never declared.** `lint` says so as a finding; `simulate` refuses the run. Declare a storage with `StorageDef`, a function with `Function`/`FunctionDist`/`FunctionExpon`, a table with `Table`.
- **The time unit.** The clock is whole units and the delays are truncated: a model whose numbers are in seconds and whose clock is in minutes is a model whose numbers are wrong.
- **`save_model` writes the WHOLE document.** It is not a patch. To change a model that is on disk, use `apply_edit` on its `path`; to write a document you wrote by hand, read the model with `explain_model` first and hand back the complete text.
- **`start` is a parameter of the run, not of the model.** `simulate({ start: 2 })` runs the model until two units of termination.
- **`%` means two different things.** In an expression `70%` is 700; in `Transfer.Prob` `85%` is 0.85 and a bare `100` is 10%.
- **`#` is multiplication**, not division: `160+(P:Tipo-1)#30` is a price.
- **An entity name may not contain whitespace or `:`**, and the declaration check only fires for a plain name.
- **A queue that never empties.** The `Queue` block joins the queue and continues; only a `Depart` takes the transaction out of it. A model that wants a queue to drain needs the `Depart`.
- **The report has one row per statement, not per block.** The `FacilityUse` block is five rows and the `Clock` is two: do not read the row count as the block count. `lint`'s `blocks` counts the same statements, not the modules of the document.
- **The trace numbers the statements, not the modules.** The block of an event is the line of the compiled program, and the `-> N` of a detail is the statement the transaction goes to; neither is the `moduleid` of the document.
- **`save_model` writes the engine's form of the document.** The writer normalises the whitespace and fills the empty attributes (`hint=""`, `genprefer`): the file it writes is not the text you sent, and the next agent that reads it sees the engine's bytes.
- **A `Match` needs the two labels to be equal.** The template carries
  `RotuloPropio=AAA` and `RotuloMatch=BBB`, and a `Match` with the defaults
  does not compile (`MATCH Match refers to unknown label "BBB"`). Set both
  labels to the same text, in the same batch that creates the block.
- **The canvas geometry is the engine's, not yours.** A model you build carries no `x`, `y`, `width` or `height`, and it does not need them: `apply_edit` and `save_model` compose the model before writing it, so the desktop app draws it. Do not place the blocks by hand — the engine's composition is the one the app expects, and a model you place yourself is a model the reader sees in pieces.
- **A block without an output port is not a source.** The `Clock`, the definitions and the notes have no outputs and never appear in a link.

## If the MCP does not tell you something

- The catalogue is the sanctioned source of the facts of a class: ports, properties, templates, the entities it declares and uses. Name the class and read the answer; do not guess an element name.
- This guide and the catalogue are the only sanctioned places to learn the format. Reading the engine's Rust source is the last resort and usually the wrong one: the source is not the contract, the tools are.
- Asking the user is cheaper and more accurate than guessing: one question beats twenty probes.
- The tools fail with an error that names the block and its position (`line`, `column`, `node`, `class`). Open the block at that position: the message is about that block, not about the model.
