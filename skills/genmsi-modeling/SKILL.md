---
name: genmsi-modeling
description: Read this when an agent has to build, read or change a GenMSI model (a `.gmdl` document) with the GenMSI MCP server.
---

# GenMSI modelling (agent guide)

This file is the procedure and the vocabulary for building a GenMSI model with the GenMSI MCP server.
It is the only sanctioned place to learn the format: the catalogue holds the facts of a class, the tools hold the interface, and this file holds the rest — read it instead of the engine's Rust source.

## The workflow

1. **One catalogue call.** `catalogue({ classes: ["Generate", "Queue", "FacilityUse", "Terminate", "Clock"] })` — every class the model needs, in **one call**, not one call per class. The answer carries, for each class: the title and the description, the input ports (`ioid`, `name`), the output ports (`ioid`, `name`), the properties (`name`, `hint`, `ty`, `subtype`, and the `options` of a listed property), the entity kinds the class `declares` and the ones it `uses`, and the XML template of a minimal valid block of that class.
   - `catalogue()` with no argument returns the list of every class and the skeleton of a model.
   - `catalogue({ class: "Generate" })` returns one class.
2. **Design before writing.** Decide the flow of the transactions, the entities the model needs and the class of every block. The catalogue tells you what a class does; the design is yours.
3. **Draft the document.** Start from the skeleton the catalogue returns, drop one `<module>` per block into its `<modules>`, and give every block its own `moduleid` (the template of a class carries `moduleid="0"`: renumber it, and keep `owner="0"`, the model root holds it). The template already carries the ports and the properties with their defaults.
4. **`lint({ model })`** — the document as text, no file yet. It loads, validates and compiles the model and reports what it found. A model that does not compile is a tool error, not an empty report.
5. **`explain_model({ model })`** — the model as the document writes it: every block with its class, its properties **by name** and its ports, and the `edges` that wire the blocks. It is the cheapest way to see that the model is the model you meant. (It compiles the model first, so read it after `lint`.)
6. **`simulate({ model, seed })`** — the GPSS report. Read its `blocks` (the statements the model compiles to, with their counts) and its `entities` (the queues, facilities and storages it used).
7. **`open_model()`** — the file the desktop app has open (`path` and `pid`; both null when the app is not running).
8. **`save_model({ path, model })`** — the whole document, to the path `open_model` returned. The app watches that file, reloads the model and runs it again.

The order matters: `lint` before `simulate` (a model that does not compile is a tool error, not an empty report), and `simulate` before `save_model` (never write a model you have not run).

`save_model` writes the **whole** document, not a patch. To change a model that is already on disk, read it with `explain_model({ path })`, change the text and hand back the complete document.

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
- **A generator does not arrive at time 0**: the first transaction arrives one interval later. A numeric `Modificador` is a symmetric spread, `uniform(Media-Modificador, Media+Modificador)`; `Modificador=0` means exactly `Media`; a **function** as the modifier multiplies the mean (`Media * f(u)`), the idiom for an exponential service.
- **`FacilityUse` is five statements**: queue, seize, depart, advance, release. The transaction waits in the `Queue` while the facility is busy.
- **`Queue` joins and leaves**: the transaction enters the queue and leaves it as soon as the block lets it continue; the report counts the entries of the queue and the time spent in it.
- **`Match` is a rendezvous**: the transaction waits at the block (the trace shows a `backup` event) for the transaction that carries the conjugate label; the pair then continues together.
- **`Transfer` splits**: the first output takes the share `Prob`, the second takes the rest.
- **A storage needs its capacity**: `StorageDef` declares it; `Enter`/`Leave` (and `StorageUse`) take and give back units, and a transaction that does not fit waits.
- **The gate blocks** (`GateF`, `GateL`, `GateS`, `GateM`): the pass goes to `Salida1`; when `Salida2` is not connected, the transaction waits in the block and every change of the state retries the waiters.
- **`Split` creates copies**: the original continues to its own output and the copies go to `Salida2`.
- One deviation is documented and **not settled**: with a dispersion, the original draws the first interarrival while the engine schedules the first arrival exactly at `Media`. Do not use the first arrival of a generator to tell the two apart; use the counts over many replications.

## Probe before you rely on it

A class whose semantics you have not seen is worth one probe, not twenty.

1. Build a model of two or three blocks: a `Clock`, the class itself and a `Terminate`.
2. `lint({ model })` — it must compile.
3. `simulate({ model, seed: 42 })` and read the `blocks` of the report: the rows are the statements the class compiles to, with their operands and their counts.
4. `get_trace({ model, seed: 42, limit: 40 })` — the events in order, with the time, the block and the detail (`-> 13` is the block the transaction goes to).
5. Change one operand and run again with the same seed: the difference is what the operand does.

Probe the classes whose behaviour you are not sure of, not the ones the catalogue already describes.

## Traps

- **A link that starts at a port the block does not have.** The engine refuses the model (`link starts at missing output port 1`). The `ioid` of a port is the one the catalogue publishes for that class.
- **An entity used and never declared.** `lint` says so as a finding; `simulate` refuses the run. Declare a storage with `StorageDef`, a function with `Function`/`FunctionDist`/`FunctionExpon`, a table with `Table`.
- **The time unit.** The clock is whole units and the delays are truncated: a model whose numbers are in seconds and whose clock is in minutes is a model whose numbers are wrong.
- **`save_model` writes the WHOLE document.** It is not a patch. Read the model with `explain_model` first and hand back the complete text.
- **`start` is a parameter of the run, not of the model.** `simulate({ start: 2 })` runs the model until two units of termination.
- **`%` means two different things.** In an expression `70%` is 700; in `Transfer.Prob` `85%` is 0.85 and a bare `100` is 10%.
- **`#` is multiplication**, not division: `160+(P:Tipo-1)#30` is a price.
- **An entity name may not contain whitespace or `:`**, and the declaration check only fires for a plain name.
- **The report has one row per statement, not per block.** The `FacilityUse` block is five rows and the `Clock` is two: do not read the row count as the block count.
- **A block without an output port is not a source.** The `Clock`, the definitions and the notes have no outputs and never appear in a link.

## If the MCP does not tell you something

- The catalogue is the sanctioned source of the facts of a class: ports, properties, templates, the entities it declares and uses. Name the class and read the answer; do not guess an element name.
- This guide and the catalogue are the only sanctioned places to learn the format. Reading the engine's Rust source is the last resort and usually the wrong one: the source is not the contract, the tools are.
- Asking the user is cheaper and more accurate than guessing: one question beats twenty probes.
- The tools fail with an error that names the block and its position (`line`, `column`, `node`, `class`). Open the block at that position: the message is about that block, not about the model.
