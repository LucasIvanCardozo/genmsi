# `.gmdl` format and simulation semantics

The format of a model (`.gmdl`) and the semantics the engine implements,
with the evidence behind each rule. The decisions that shaped it live in
[`DECISIONS.md`](DECISIONS.md); this file is the working reference for how the
engine behaves.

The **reference application** is the Windows program the harness of
`scripts/wine-oracle/` runs under Wine; every comparison in this file is made
against it.

This is the reference of the `.gmdl` format and of the semantics the engine
implements: what every block means, how an expression is evaluated and what a
run produces. It is here so that whoever writes a model knows what is going to
happen, and so that whoever reads the code knows which rules it obeys. It is
read by the authors of models, by the readers of the engine's source, and by
anyone who asks why the engine behaves the way it does.

- [1. The file format](#1-the-file-format) — the XML of a `.gmdl` document:
  the modules, the properties, the percent-encoding and the canvas the engine
  composes.
  - [Round-trip guarantee](#round-trip-guarantee) — the proof that a rewrite is
    byte-for-byte identical and that the geometry of the model survives it.
  - [The composition of the canvas](#the-composition-of-the-canvas) — how the
    engine lays the blocks out (dagre, network simplex, Brandes–Köpf) and when
    it does it.
- [2. Container topology](#2-container-topology) — how the containers own their
  links, what the entry and exit ports mean and how the flow is resolved
  inside them.
- [3. Block expansion](#3-block-expansion) — the GPSS statements every class
  compiles to, and the modules that compile to none.
- [4. Expression language](#4-expression-language) — the grammar of the
  properties that carry an expression, and the forms that look alike but are
  not.
- [5. Runtime semantics (with the evidence that pinned them)](#5-runtime-semantics-with-the-evidence-that-pinned-them) —
  what each statement does at run time, with the comparison against the
  reference application that pinned it.
  - [Settled: the first interval with a dispersion is drawn](#settled-the-first-interval-with-a-dispersion-is-drawn) —
    the deviation that became a rule, and the measurement that settled it.
- [6. Validation inferred from the five error outputs](#6-validation-inferred-from-the-five-error-outputs) —
  the five refusals of the reference application and the rule the engine
  applies to each one.
- [7. Reports, exports and replications](#7-reports-exports-and-replications) —
  the GPSS report, the JSON and CSV exports, the replications with their
  interval, and the trace.
  - [Deviations from the report of the reference application](#deviations-from-the-report-of-the-reference-application) —
    the places where our report differs on purpose, and why none of them
    changes the simulation.
- [8. Oracle comparison](#8-oracle-comparison) — how the numbers of a run are
  compared with the baseline of the reference application, and the rule that
  decides that they match.
- [9. The GPSS as a second oracle](#9-the-gpss-as-a-second-oracle) — the GPSS
  text of both programs and the tools that compare them statement by
  statement.
  - [What the phase measured](#what-the-phase-measured) — the numbers the phase
    reached, and the benches that cannot be compared.
  - [What the oracle found that no one was looking for](#what-the-oracle-found-that-no-one-was-looking-for) —
    the three defects the comparison uncovered.
  - [9.1 What the reference application writes](#91-what-the-reference-application-writes) —
    the naming, the spelling and the operands of the statements of the
    reference application.

## 1. The file format

A `.gmdl` file is a single line of compact XML:

```
<gmodel versioncode="…"><gclasses>…</gclasses><modules>
  <module moduleid moduleclass owner caption name kind …>…</module>…
</modules></gmodel>
```

- `kind="A"` modules are activity blocks; `kind="C"` modules are containers
  (the model root has `moduleclass="main"` and is also a container).
- Every module may declare `<inputs>`, `<outputs>`, `<links>` and `<props>`; the
  sections appear in different orders, so the parser stores the observed order
  (`SectionKind`) and the writer replays it.
- `<props>` are ordered and schema-defined (`name`, `hint`, `value`, `type`,
  `subType`, `visible`, `unique`). A property with `unique="1"` defines an
  entity.
- Attribute values are sometimes percent-encoded (CP1252) — hints always, and
  in some models entity names too (`tiempo%20demora`, `Q%3Acola`). The raw string
  is preserved for rewriting; `Property::value()` returns the decoded text.
- `<gclasses>` may hold custom module macros (`Multifurcacion` in the
  reference set). The engine does not interpret macro code; the exact source
  slice is preserved so a rewrite keeps it.

### Round-trip guarantee

`genmsi_format::to_string` reproduces every one of the 25 reference models
byte-for-byte (test `rewriting_every_reference_model_is_byte_identical`), and
editing one property changes exactly that attribute
(`rewriting_a_semantically_edited_model_changes_only_the_edited_value`). Canvas
geometry (block and port position/size, link breakpoints) lives in `Layout`
values attached to the semantic model, so positions survive a rewrite without
influencing the simulation.

### The composition of the canvas

The geometry is the engine's business (D18). `genmsi_format::layout` composes
a model that does not carry it, with **dagre** (the Rust port of dagre.js),
the Sugiyama pipeline:

- The **flow** of the model is cut into layers by the **network simplex**
  (`Ranker::NetworkSimplex`), which cuts the rows so that the links cross as
  few layers as possible. When the simplex fails on a compound graph —the
  cycles of the container borders— the composition falls back to the **tight
  tree**, which cuts the same rows by the depth of the flow but crosses more
  links; the composition must never panic, and both answers are the same every
  time, so it stays deterministic. The links that close a cycle are taken out
  first (the flow of a model with a cycle is still a flow, and the block the
  cycle comes back to is its start). The links that cross the border of a
  container are spliced: the flow that enters a container goes to the block its
  entry port names, and the flow that leaves it comes from the block its exit
  port names (`layout::flow`).
- The layers are **ordered** to cross the fewest links (the barycentre of the
  neighbours and the transpose; no luck in it).
- The blocks are **placed** (Brandes–Köpf): the direction is **top to bottom**
  (`TB`), because the input of a block is at its top and its output at its
  bottom, so the flow a reader follows goes down.
- The **containers** are the frames around the blocks they hold: each frame is
  the box of its content, with the margin that separates them.
- The blocks that carry no geometry of their own get the default size
  (`120 × 48`), and the model root — which the editor does not draw — is left
  alone.

The composition is deterministic (the same model always produces the same
geometry), it never touches the semantics, and it is only applied when
something asks for it:

- the app's `Reacomodar` (the facade's `relayout` operation, one undo entry),
- the MCP's `save_model`, before it writes a model whose geometry is
  incomplete,
- the CLI's `genmsi layout`.

A model that is already composed is left exactly as its author placed it: a
model whose blocks all carry their geometry is left alone, and one that does
not is composed entirely (the composition is one picture, not a patch). And
never on its own.

## 2. Container topology

Containers group blocks and own their links. Two port directions exist:

- **Entry**: a link *starts at* the container
  (`(container, ioid) -> (child, ioid)`); the flow continues inside.
- **Exit**: a link *ends at* the container
  (`(child, ioid) -> (container, ioid)`); the flow continues in the container's
  **parent**, where a link starts at `(container, ioid)`.

When an entry port has no explicit internal link (the editor allows it, e.g.
TP4's `EleccionProductos`), the flow starts at the container's first child
(lowest `moduleid`). Resolution follows nested containers until an activity
block is reached; cycles are rejected.

## 3. Block expansion

The compiler generates GPSS statements from the semantic modules, so the report
rows and the block statistics line up with the block table of the reference
application:

| Module | Emitted statements |
|---|---|
| `FacilityUse` | `QUEUE`, `SEIZE`, `DEPART`, `ADVANCE`, `RELEASE` (the queue pair is omitted when `Queue` is empty) |
| `FacilityInt` | `QUEUE`, `PREEMPT`, `DEPART`, `ADVANCE`, `RETURN` |
| `StorageUse` | `QUEUE`, `ENTER`, `DEPART`, `ADVANCE`, `LEAVE` |
| `Clock` | `GENERATE <Frecuencia>` and a synthetic `TERMINATE 1` |
| `Counter` | `SAVEVALUE <SaveValue>+,<Incremento>` and `ASSIGN <Param>,<SaveValue>` |

Definitions (`Function`, `Variable`, `VariableA`, `Table`, `QTable`,
`StorageDef`, `InitiaSV`, `InitialLS`) and `Note` modules produce no statement.

## 4. Expression language

`genmsi-core/src/expr/grammar.pest` is grammar and documentation at once
(D4). Supported forms:

- numbers, `+ - * /`, modulo `@`, parentheses, unary minus;
- `#` is the **multiplication**, not a division: the format has no `*` key — the
  expression editor of the reference application (`FormOIExpresion`, read out
  of the executable) offers `/`, `%`, `#` and `@` — and the models read that way
  ("15#paraLlevar" is a surcharge per delivery, and "160+(P:Tipo-1)#30" is a
  price);
- typed references with `:` or `$`: `P:` parameter, `X:` savevalue, `V:`
  variable, `FN:`/`F:` function, `Q:` queue content, `S:` storage units in use,
  `R:` facility free flag, `CH:` user-chain size, `LS:`/`L:` logic switch,
  `N:` transactions that passed a label, `W:` transactions waiting there;
- indirect names: `*Caja` (the entity named by the value of parameter `Caja`);
  a bare name is also a parameter read (the compiler emits `*cantP` for `cantP`);
- system symbols `AC1` (model clock), `C1` (absolute clock), `TC1` (remaining
  termination count), `RN<k>` (uniform draw from stream `k`);
- `70%` is the editor's per-mille form and evaluates to `700`.

`[name]` placeholders are resolved at compile time against the enclosing
containers: the nearest container with a usable value wins, and a value that is
empty or is the placeholder itself (`media = [media]`, TP4) falls through to the
outer container. Unresolved placeholders are `0` in expressions and keep their
text in entity-name positions.

## 5. Runtime semantics (with the evidence that pinned them)

- **GENERATE A,B,C,D,E**: the first transaction arrives with the increment `C`
  when the model brings it, and otherwise with the increment that comes out of
  `A` and `B` — `A` alone, or the draw of `A±B` when there is a dispersion.
  It never arrives at time 0 by definition. `D` limits the total; an empty `A`
  is a zero interarrival, so with it the first transaction does land at time 0
  (the models use it with `D` to create one transaction at time 0). Verified
  against the 18 models that carry a `GENERATE` without dispersion, without
  offset and without limit: `GENERATE 3600` over a
  horizon of 3600 gives 2 entries (the arrival at 3600 plus the one still
  pending in the FEC), `GENERATE 900` over 3600 gives 5 and `GENERATE 1800`
  over 1800 gives 2; with the first arrival at time 0 those same models would
  give 3, 6 and 3. The table of the eighteen, the counting model of the
  reference application that makes the comparison exact and the two rules it
  rests on are in `odd/tasks/notas-primera-llegada.md`; the table is reproduced
  from the reports of the reference application by
  `scripts/wine-oracle/first_arrival_evidence.py`, and
  `scripts/wine-oracle/compare_generate.py` puts our own counts next to it.
- **ADVANCE A,B / GENERATE with B**: a numeric `B` is a symmetric spread
  (`uniform(A-B, A+B)`); a **function** `B` multiplies the mean
  (`A * f(u)`), which is the idiom for exponential service
  (`GENERATE FN$MediaClientes,FN$ModificadorClientes`). Spread `0` means exactly
  `A`.
- **Function types**: `Discreta` is a step function, `Continua` interpolates
  linearly, `Eval` returns the (evaluated) `y` expression of the matching entry.
  `FunctionExpon` is `-ln(1-u)` on its `Seed` stream; `FunctionDist` maps
  percentages to indices 1..N; `VariableA` is a uniform integer in
  `[Minimo, Maximo]` on its `Seed` stream.
- **SPLIT A creates A copies**; the transaction that arrives continues to its
  own output and the copies go to the `Copias` output. The models prove it:
  `SPLIT P$n` is paired with `ASSEMBLE 1+P$n` (the copies plus the transaction
  that arrived) in TP4/TP5.
- **ASSEMBLE** gathers `Cantidad` transactions of the same assembly set, keeps
  one and destroys the rest; **GATHER** releases all of them.
- **TRANSFER**: literals ending in `%` are fractions (`85%` → 0.85); numeric
  values and expressions are per-mille (`850`, `FN:PValue` → 100 = 10%).
  The first output is the probability branch.
- **SELECT / SELECTF**: `SELECT MIN|MAX` picks the entity number in `[minimo,
  maximo]` with the smallest/largest metric (queue content, storage occupancy);
  `SELECTF` (`op="Libre"`, the `NU` operator) picks the first entity that is
  actually free. Facilities and queues are auto-created by number. The class of
  the entity is the `ClaseSNA` property, and the catalogue of the format has
  38 of them (`Q` queue, `S` storage, `R` storage left, `F` facility, `LS` logic
  switch...): the statement carries the class as its last operand
  (`SELECT MIN param,1,10,,F`).
- **GATE**: pass goes to `Salida1`; a failure goes to `Salida2` when it is
  connected, otherwise the transaction waits in the block and every state change
  retries the waiters (the reference application reports these as RETRY
  counts).
- **PREEMPT/RETURN**: a preempted owner that was delayed in an `ADVANCE` has its
  remaining time stored in the `Remanente` parameter; `RETURN` releases the
  facility and resumes the owner with its remaining time. Interrupt modes are
  accepted but both behave as preempt-and-return.
- **LINK/UNLINK**: `UNLINK` removes up to `Cantidad` matching members (optionally
  filtering by a parameter); the removed transactions continue at the
  `SalidaAlt` target while the active transaction continues at `Salida1`.
- **Groups**: `JOIN` adds the transaction (transaction group) or a number
  (`JoinNumber`); `EXAMINE` tests numeric membership; `REMOVE`, `SCAN` and
  `ALTER` operate on transaction groups.
- **Termination**: the run ends when the start counter reaches zero. A model is
  runnable only if it has a `Clock` or a `TERMINATE` with a positive `A`
  operand — exactly the oracle's refusal rule.
- **`Clock` and the `START`**: the `Clock` spends one unit of the `START` every
  `Frecuencia` time units ("Descuenta 1 unidad del START de la simulación cada
  un cierto tiempo indicado"), so a run with `Frecuencia` 3600 ends at time
  3600 and the report says so in `END_TIME`. The engine compiles the `Clock`
  into `GENERATE Frecuencia` + `TERMINATE 1`.
- **Storages (`ENTER`/`LEAVE`)**: a storage exists only when a `StorageDef`
  declares it, and the declaration gives its capacity in units (`Tamaño`,
  "Capacidad del Storage"). The `Cantidad` of an `ENTER`/`LEAVE` is the units
  one transaction takes and gives back ("Cantidad de ingresos"), so a storage
  of `Tamaño` 1 with a `Cantidad` of 1 holds one transaction at a time and the
  others wait (the reference application reports `MAX` 1 and the refusals in the
  storage's own `RETRY`). A storage the model does not declare has no capacity:
  the reference application stops the run with "Referencia a una entidad
  storage inexistente", and the compiler refuses the model before it runs.

### Settled: the first interval with a dispersion is drawn

With a dispersion (`B` not empty and not zero), the first interarrival is
**drawn** from `A` and `B`, like every other interarrival. The manual of the
reference application says the first increment is "calculated from the operands
A and B"; with `A` alone that is the exact `A`, and with a dispersion it is a
draw.

This used to be a known deviation — the engine scheduled the first arrival at
exactly `A` and started drawing from the second interval on — and it is
**settled**: the engine draws the first one too
(`crates/genmsi-core/src/sim.rs`, `first_arrival` → `apply_modifier`), and
`crates/genmsi-core/tests/primera_llegada.rs` pins the three cases the rule
has (with a spread the first arrival is drawn and not the mean; without a
spread there is nothing to draw and the first arrival is the mean, exactly;
with an offset the offset is the increment of the first transaction and
nothing is drawn). The measurement that settled it is in
`odd/tasks/notas-primera-llegada.md`: sixty first arrivals of the reference
application read with a `MARK` give a mean of 57.133 against the 60 the model
asks for, and the Kolmogorov-Smirnov test against `U(30,90)` does not reject
the uniform (D=0.1167, p=0.3666).

## 6. Validation inferred from the five error outputs

The five models the engine refuses, and the class of error each one fails
with, are published in the CLI's
[reference models](cli.md#reference-models). What this section records is the
rule the engine applies to each class of error, and the message of the
reference application the rule was inferred from.

| Class of error | Message of the reference application | Engine rule |
|---|---|---|
| `invalid block` | `Error en linea 9 -> Bloque inválido` | entity-definition names may not contain whitespace or `:` (the only offender in 25 models is `tiempo demora`) |
| no `TERMINATE` with operand A and no `CLOCK` | `No hay bloques TERMINATE con parámetro A` | `validate_runnable`: a `Clock` or `TERMINATE` with positive `A` is required |
| `UNLINK` without a label | `El parámetro B no debe ser nulo` | `UNLINK` requires a label on its `SalidaAlt` output |
| internal crash of the reference application (`TP9__hack`) | `Se intentó obtener la linea del fuente de un bloque inexistente` | an internal crash of the reference application; **not replicated** — the model has no termination path and the engine reports that modelling error |

All errors carry a `Location` with file, 1-based source line and `moduleid`.

## 7. Reports, exports and replications

- `genmsi_core::render` prints the GPSS report (header counts, block table with
  `ENTRY/CURRENT/RETRY`, facilities, queues, storages, user chains, tables,
  savevalues, diagnostics) with the comma decimals of the format.
- `to_json` returns a typed `serde` document (run, blocks, entities);
  `to_csv` returns `section,entity,metric,value` rows.
- `replicate` runs `N` replications with seeds derived by `mix_u64(base_seed, i)`
  and reports mean, standard deviation and the 95% Student-t interval per metric
  (`t_critical` uses the exact table up to 30 degrees of freedom).
- When the trace is enabled, every event of the run is recorded with time,
  transaction, block, label and detail. The events the trace records are the
  ones the CLI lists in [`trace`](cli.md#trace). The `backup`
  event says that the transaction backs up to the block it is counted in: it
  is recorded in the block that waits (a gate that does not open, a facility
  that is busy, a storage that is full, a match without a partner) and its
  detail names the block the transaction goes back to (`-> 13`). The trace
  also says what the run did to the things the report counts and the events
  cannot say on their own: the name of the entity a block touches, the
  samples of a table, the state of a logic switch, the value of a savevalue
  and the members that leave a chain.

### Deviations from the report of the reference application

These are deliberate and do not change the simulation:

- one report row per module (the reference application expands compound blocks
  into several rows); statistics are identical for the labelled first statement;
- blocked transactions are counted as current in the block they came from, the
  way the reference application counts them (`A_Queue  QUEUE  qAfuera  13 13`
  with the gate shut, and the trace says the same with its `backup` event); the
  exception is the `MATCH`, whose waiting transaction the reference application
  counts in the match as well (`A_Match  TRANSFER  ,PIENSA  43 3 0` and
  `PIENSA  MATCH  ESPERA 40 3 0`), and we count it only in the transfer;
- queue content and facility utilization are integrals over the run, so they do
  not depend on where a blocked transaction is displayed;
- `TABULATE` records one observation per execution (`Peso` is accepted but not
  used as a weight).

## 8. Oracle comparison

`genmsi_core::oracle` parses the report of the reference application; the
release example `crates/genmsi-core/examples/oracle.rs` runs every model to the
baseline's `END_TIME` with replications and writes `docs/oracle-comparison.md`.
A metric matches when its relative error is within 20% (counts) or 35% (times
and occupancies), or when the baseline value falls inside the 95% interval of
our replication mean. Baselines are single samples, so the rule is generous by
design (D6).

## 9. The GPSS as a second oracle

The numbers say whether a model behaves like the reference application; they do
not say whether it behaves like it *for the right reasons*. The reference
application publishes its own answer to that question: it compiles a model
into GPSS and shows it in its **Código GPSS** tab.

`genmsi gpss <model>` writes the same text: one statement per line, the
block's name, the verb and the operands, the definitions first, and the blocks
in the order a transaction walks them. The order matters — a transaction goes
to the next block by itself, so only the connections that break the walk are
written as a `TRANSFER` with no name.

`scripts/wine-oracle/compare-gpss.sh` compares our text with the text of the
reference application for the same model. It compares two things apart from
each other:

- **what each block does**: the verb and the operands of every statement,
  matched by the block's name;
- **where the transaction goes**: the connections, as a set of pairs.

The two are compared apart because the order of the blocks is a matter of the
walk and not of the model: the same connection is a fall-through in one program
and a `TRANSFER` in the other, and that is not a difference of behaviour.

The tools in `scripts/wine-oracle/` read the reference application without
running it:

| Tool | What it does |
|---|---|
| `dfm.py` | Writes the 55 forms of the reference application as text, so its interface can be read instead of photographed |
| `make-benches.py` | Writes a model per class of the class library of the reference application, with the ports and the defaults that library declares |
| `run-benches.sh` | Runs every bench against the reference application and keeps its GPSS and its report |
| `compare-gpss.sh` | Compares our GPSS with the one of the reference application, block by block and connection by connection |
| `first_arrival_evidence.py` | Reads the baseline reports of the reference application and shows when the first transaction of a generator arrives |
| `compare_generate.py` | The same evidence with our engine's counts next to the ones of the reference application |

A bench is the unit of verification: the GPSS of the reference application for
it says how a class compiles, and its report for it says what the class does.
Together they are the specification of that class, and the model under test is
one class with nothing else in it.

### What the phase measured

Against the answer of the reference application, class by class:

| | Before the phase | Now |
|---|---|---|
| The GPSS of the bench is the one of the reference application | 17 of 62 | **62 of 62** |
| The numbers of the bench are the ones of the reference application | — | **50 of 55** |

The four benches the reference application refuses to run are not a failure of
the tool: it shows a dialog ("the modules were adapted") and never shows its
own window, so there is no answer to compare with. The five that cannot be
compared by the numbers are the benches whose report of the reference
application has no header to read — a bench with a single `Depart` measures
nothing.

One command asks both questions:

```
python3 scripts/wine-oracle/make-benches.py
bash scripts/wine-oracle/run-benches.sh scripts/wine-oracle/benches /tmp/bench-out
bash scripts/wine-oracle/compare-benches.sh scripts/wine-oracle/benches /tmp/bench-out
```

### What the oracle found that no one was looking for

- The `ALTER` read `CompParam` and `CompValue` as the pair to **assign**, and
  altered the parameter it was supposed to **compare**, without comparing
  anything. The macro of the reference application says it in its own words, and
  the block has a second output called "No alteró" for the transactions that do
  not match.
- The `#` of an expression is the **multiplication**, not an integer division:
  the format has no `*` key (the expression editor of the reference application
  offers `/`, `%`, `#` and `@`), and the models read that way.
- A transfer's destinations are written in the **reverse** of the port order,
  and the property hint of the reference application says the opposite
  ("probabilidad de continuar por la Salida1"). The text is reversed; the
  engine is not — a probe in `scripts/wine-oracle/probes/` settles it with the
  numbers.

### 9.1 What the reference application writes

The text of the reference application is not what the model says, and reading it
is a matter of rules. These are the ones the benches show.

**The name of a statement.** The reference application does not take the name of
a statement from the model: it gives every block the name of its class
(`A_Advance`, `A_Queue`), and when two blocks of the same class would take
that name it numbers the second one (`A_Terminate`, `A_Terminate1`). The
classes whose statements come from a macro of the reference application write a
name of their own:

| Class | The name it writes |
|---|---|
| `GateF`, `GateL`, `GateS`, `GateM` | the name of the module (`GateF`) |
| `StorageUse` | the name of the module (`StorageUse`, `StorageUse1`) |
| `Transfer` | the name of the input port of its macro (`Entrada1`, `Entrada11`) |
| `Split` | what it does (`Clona`) |
| the definitions | the name of the entity (`Func`, `Tabla`, `vari`, `Almacen`) |

A block that has no input port — a generator, the clock — has no name at all.

**The spelling of an expression.** A typed reference is written with a `$`
where the model writes a `:` (`P:Param` becomes `P$Param`). The operands of
a `TEST`, a `SELECT` or a `GATE` carry the operator of the vocabulary of the
reference application (`E`, `NE`, `L`, `LE`, `G`, `GE`; `MIN`, `MAX`, `NU`;
`LS`, `LR`; `SF`, `SNF`, `SE`, `SNE`, `SV`, `SNV`; `NU`, `U`).

**The operands of a class.** Each class writes the operands its own macro
writes, in the order of that macro: `ALTER E <grupo>,<cantidad>,<parámetro
alterado>,<valor>,<parámetro comparado>,<valor>`; `REMOVE E <grupo>,<cantidad>
,,<comparado>,<valor>`; `SCAN E <grupo>,<comparado>,<valor>,<origen>,<destino>`;
`UNLINK E <cadena>,<bloque que recibe>,<cantidad>,<comparado>,<valor>`;
`SELECT <modo> <parámetro>,<mínimo>,<máximo>,,<clase>`; `LINK <cadena>,<modo>
,<bloque que sigue>`; `LOGIC <S|R> <interruptor>`; `GATE <condición> <entidad>`;
`LOOP <parámetro>,`; `JOIN <grupo>[,<número>]`; `EXAMINE <grupo>,<número>,
<destino>`;
`ASSEMBLE`/`GATHER <cantidad>`; `SPLIT <total>`; `TABULATE <tabla>`;
`TEST E <izquierda>,<derecha>`; `SAVEVALUE <nombre><operador>,<valor>`;
`ASSIGN <parámetro><operador>,<valor>`.

**The counter and the match write two statements.** The counter increments a
savevalue and copies it to a parameter (`SAVEVALUE Count+,1` and
`ASSIGN Nro,X$Count`). The match writes the transfer to its conjugate before
the match itself (`A_Match TRANSFER ,AAA` and `AAA MATCH AAA`), and the report
lists the two statements: the entries are in the transfer and the transactions
that wait are in both. The counter's two statements are two rows of the block
table too, and the header counts the rows: the report of the counter says six
blocks where the model has five.

**The definitions come first.** The storages, the queue tables, the tables,
the functions, the variables and the initial values, and then the blocks. The
function writes the table of its points on a line of its own, and the
exponential function writes the standard table of the exponential
distribution, which its class does not carry.

