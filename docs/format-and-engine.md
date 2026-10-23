# `.gmdl` format and simulation semantics

Design notes for Phase 1 (`genmsi-format` + `genmsi-core`). They record the
semantics the engine implements and the evidence behind each rule. Decisions
D1–D12 of `docs/DECISIONS.md` are not re-opened here; this file is the working
reference for how the subset behaves.

## 1. The file format as the original writes it

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

The original generates GPSS statements from the semantic modules. The compiler
does the same, so report rows and block statistics line up with the original's
block table:

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
- `#` is the **multiplication**, not a division: the original's expression
  editor (`FormOIExpresion`, read out of the executable) has no `*` key — it
  offers `/`, `%`, `#` and `@` — and its models read that way ("15#paraLlevar"
  is a surcharge per delivery, and "160+(P:Tipo-1)#30" is a price);
- typed references with `:` or `$`: `P:` parameter, `X:` savevalue, `V:`
  variable, `FN:`/`F:` function, `Q:` queue content, `S:` storage units in use,
  `R:` facility free flag, `CH:` user-chain size, `LS:`/`L:` logic switch,
  `N:` transactions that passed a label, `W:` transactions waiting there;
- indirect names: `*Caja` (the entity named by the value of parameter `Caja`);
  a bare name is also a parameter read (the original emits `*cantP` for `cantP`);
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
  (the original does the same: the models use it with `D` to create one
  transaction at time 0). Verified against the 18 models that carry a `GENERATE`
  without dispersion, without offset and without limit: `GENERATE 3600` over a
  horizon of 3600 gives 2 entries (the arrival at 3600 plus the one still
  pending in the FEC), `GENERATE 900` over 3600 gives 5 and `GENERATE 1800`
  over 1800 gives 2; with the first arrival at time 0 those same models would
  give 3, 6 and 3. The table of the eighteen, the counting model of the
  original that makes the comparison exact and the two rules it rests on are
  in `odd/tasks/notas-primera-llegada.md`; the table is reproduced from the
  original's reports by `scripts/wine-oracle/first_arrival_evidence.py`, and
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
- **SPLIT A creates A copies**; the original continues to its own output and the
  copies go to the `Copias` output. The models prove it: `SPLIT P$n` is paired
  with `ASSEMBLE 1+P$n` (the copies plus the original) in TP4/TP5.
- **ASSEMBLE** gathers `Cantidad` transactions of the same assembly set, keeps
  one and destroys the rest; **GATHER** releases all of them.
- **TRANSFER**: literals ending in `%` are fractions (`85%` → 0.85); numeric
  values and expressions are per-mille (`850`, `FN:PValue` → 100 = 10%).
  The first output is the probability branch.
- **SELECT / SELECTF**: `SELECT MIN|MAX` picks the entity number in `[minimo,
  maximo]` with the smallest/largest metric (queue content, storage occupancy);
  `SELECTF` (`op="Libre"`, GPSS `NU`) picks the first entity that is actually
  free. Facilities and queues are auto-created by number, as GPSS does.
- **GATE**: pass goes to `Salida1`; a failure goes to `Salida2` when it is
  connected, otherwise the transaction waits in the block and every state change
  retries the waiters (the original reports these as RETRY counts).
- **PREEMPT/RETURN**: a preempted owner that was delayed in an `ADVANCE` has its
  remaining time stored in the `Remanente` parameter; `RETURN` releases the
  facility and resumes the owner with its remaining time. Interrupt modes are
  accepted but both behave as preempt-and-return.
- **LINK/UNLINK**: `UNLINK` removes up to `Cantidad` matching members (optionally
  filtering by a parameter); the removed transactions continue at the
  `SalidaAlt` target while the active transaction continues at `Salida1`.
- **Groups**: `JOIN` adds the transaction (transaction group) or a number
  (`JoinNumber`); `EXAMINE` tests numeric membership; `REMOVE`, `SCAN` and
  `ALTER` operate on transaction groups as in GPSS.
- **Termination**: the run ends when the start counter reaches zero. A model is
  runnable only if it has a `Clock` or a `TERMINATE` with a positive `A`
  operand — exactly the oracle's refusal rule.
- **`Clock` and the `START`**: the `Clock` spends one unit of the `START` every
  `Frecuencia` time units ("Descuenta 1 unidad del START de la simulación cada
  un cierto tiempo indicado"), so a run with `Frecuencia` 3600 ends at time
  3600 and the report says so in `END_TIME`. The original compiles the `Clock`
  into `GENERATE Frecuencia` + `TERMINATE 1`, and the engine does the same.
- **Storages (`ENTER`/`LEAVE`)**: a storage exists only when a `StorageDef`
  declares it, and the declaration gives its capacity in units (`Tamaño`,
  "Capacidad del Storage"). The `Cantidad` of an `ENTER`/`LEAVE` is the units
  one transaction takes and gives back ("Cantidad de ingresos"), so a storage
  of `Tamaño` 1 with a `Cantidad` of 1 holds one transaction at a time and the
  others wait (the original reports `MAX` 1 and the refusals in the storage's
  own `RETRY`). A storage the model does not declare has no capacity: the
  original stops the run with "Referencia a una entidad storage inexistente",
  and the compiler refuses the model before it runs.

### Known deviation, not settled: the first interval with a dispersion

With a dispersion (`B` not empty and not zero), the original **draws** the
first interarrival from `A` and `B`, like every other interarrival, and our
engine does not: it schedules the first arrival at exactly `A` and starts
drawing from the second interval on (`crates/genmsi-core/src/sim.rs`,
`Simulation::init`). The original's manual says the first increment is
"calculated from the operands A and B"; with `A` alone that is the exact `A`
our engine uses, but with a dispersion it is a draw.

The consequence shows with several copies of the same generator: ours release
their first transaction all at the same instant, the original staggers them.

This is **not settled**. No count of entries can separate the two readings,
because the expected number of arrivals is the same either way — the measured
difference is 0.01 arrivals over 200,000 samples. The evidence, the models it
cannot settle and the proposed change are in
`odd/tasks/notas-primera-llegada.md`.

## 6. Validation inferred from the five error outputs

| Baseline | Original message | Engine rule |
|---|---|---|
| `Parcial 8-11-21__ParcialMikelModelos` | `Error en linea 9 -> Bloque inválido` | entity-definition names may not contain whitespace or `:` (the only offender in 25 models is `tiempo demora`) |
| `Problema 2__puenteLevadizo`, `TP6__Clase 6_0_TestGate` | `No hay bloques TERMINATE con parámetro A` | `validate_runnable`: a `Clock` or `TERMINATE` with positive `A` is required |
| `TP9__tp9` | `El parámetro B no debe ser nulo` | `UNLINK` requires a label on its `SalidaAlt` output |
| `TP9__hack` | `Se intentó obtener la linea del fuente de un bloque inexistente` | an internal crash of the original; **not replicated** — the model has no termination path and the engine reports that modelling error |

All errors carry a `Location` with file, 1-based source line and `moduleid`.

## 7. Reports, exports and replications

- `genmsi_core::render` prints the GPSS report (header counts, block table with
  `ENTRY/CURRENT/RETRY`, facilities, queues, storages, user chains, tables,
  savevalues, diagnostics) with the original's comma decimals.
- `to_json` returns a typed `serde` document (run, blocks, entities);
  `to_csv` returns `section,entity,metric,value` rows.
- `replicate` runs `N` replications with seeds derived by `mix_u64(base_seed, i)`
  and reports mean, standard deviation and the 95% Student-t interval per metric
  (`t_critical` uses the exact table up to 30 degrees of freedom).
- When the trace is enabled, every entry, leave, wait and termination is
  recorded with time, transaction, block, label and detail. The trace also
  says what the run did to the things the report counts and the events
  cannot say on their own: the name of the entity a block touches, the
  samples of a table, the state of a logic switch, the value of a savevalue
  and the members that leave a chain.

### Deviations from the original's report

These are deliberate and do not change the simulation:

- one report row per module (the original expands compound blocks into several
  rows); statistics are identical for the labelled first statement;
- blocked transactions are counted as current in the blocking block (the
  original leaves them in the previous block);
- queue content and facility utilization are integrals over the run, so they do
  not depend on where a blocked transaction is displayed;
- `TABULATE` records one observation per execution (`Peso` is accepted but not
  used as a weight).

## 8. Oracle comparison

`genmsi_core::oracle` parses the original's report; the release example
`crates/genmsi-core/examples/oracle.rs` runs every model to the baseline's
`END_TIME` with replications and writes `docs/oracle-comparison.md`. A metric
matches when its relative error is within 20% (counts) or 35% (times and
occupancies), or when the baseline value falls inside the 95% interval of our
replication mean. Baselines are single samples, so the rule is generous by
design (D6).

## 9. The GPSS as a second oracle

The numbers say whether a model behaves like the original; they do not say
whether it behaves like the original *for the right reasons*. The original
publishes its own answer to that question: it compiles a model into GPSS and
shows it in its **Código GPSS** tab.

`genmsi gpss <model>` writes the same text: one statement per line, the
block's name, the verb and the operands, the definitions first, and the blocks
in the order a transaction walks them. The order matters — a transaction goes
to the next block by itself, so only the connections that break the walk are
written as a `TRANSFER` with no name.

`scripts/wine-oracle/compare-gpss.sh` compares our text with the original's
for the same model. It compares two things apart from each other:

- **what each block does**: the verb and the operands of every statement,
  matched by the block's name;
- **where the transaction goes**: the connections, as a set of pairs.

The two are compared apart because the order of the blocks is a matter of the
walk and not of the model: the same connection is a fall-through in one program
and a `TRANSFER` in the other, and that is not a difference of behaviour.

The tools in `scripts/wine-oracle/` read the original without running it:

| Tool | What it does |
|---|---|
| `dfm.py` | Writes the original's 55 forms as text, so its interface can be read instead of photographed |
| `make-benches.py` | Writes a model per class of the original's class library, with the ports and the defaults that library declares |
| `run-benches.sh` | Runs every bench against the original and keeps its GPSS and its report |
| `compare-gpss.sh` | Compares our GPSS with the original's, block by block and connection by connection |
| `first_arrival_evidence.py` | Reads the original's own baseline reports and shows when the first transaction of a generator arrives |
| `compare_generate.py` | The same evidence with our engine's counts next to the original's |

A bench is the unit of verification: the original's GPSS for it says how a
class compiles, and the original's report for it says what the class does.
Together they are the specification of that class, and the model under test is
one class with nothing else in it.

### What the phase measured

Against the original's own answer, class by class:

| | Before the phase | Now |
|---|---|---|
| The GPSS of the bench is the original's | 17 of 62 | **62 of 62** |
| The numbers of the bench are the original's | — | **50 of 55** |

The four benches the original refuses to run are not a failure of the tool: it
shows a dialog ("the modules were adapted") and never shows its own window, so
there is no answer to compare with. The five that cannot be compared by the
numbers are the benches whose original report has no header to read — a bench
with a single `Depart` measures nothing.

One command asks both questions:

```
python3 scripts/wine-oracle/make-benches.py
bash scripts/wine-oracle/run-benches.sh scripts/wine-oracle/benches /tmp/bench-out
bash scripts/wine-oracle/compare-benches.sh scripts/wine-oracle/benches /tmp/bench-out
```

### What the oracle found that no one was looking for

- The `ALTER` read `CompParam` and `CompValue` as the pair to **assign**, and
  altered the parameter it was supposed to **compare**, without comparing
  anything. The macro of the original says it in its own words, and the block
  has a second output called "No alteró" for the transactions that do not
  match.
- The `#` of an expression is the **multiplication**, not an integer division:
  the original's expression editor has no `*` key (it offers `/`, `%`, `#` and
  `@`), and its models read that way.
- A transfer's destinations are written in the **reverse** of the port order,
  and the original's own property hint says the opposite ("probabilidad de
  continuar por la Salida1"). The text is reversed; the engine is not — a
  probe in `scripts/wine-oracle/probes/` settles it with the numbers.

### 9.1 What the original writes

The text of the original is not what the model says, and reading it is a
matter of rules. These are the ones the benches show.

**The name of a statement.** The original does not take the name of a
statement from the model: it gives every block the name of its class
(`A_Advance`, `A_Queue`), and when two blocks of the same class would take
that name it numbers the second one (`A_Terminate`, `A_Terminate1`). The
classes whose statements come from a macro of the original write a name of
their own:

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
a `TEST`, a `SELECT` or a `GATE` carry the operator of the original's own
vocabulary (`E`, `NE`, `L`, `LE`, `G`, `GE`; `MIN`, `MAX`, `NU`; `LS`, `LR`;
`SF`, `SNF`, `SE`, `SNE`, `SV`, `SNV`; `NU`, `U`).

**The operands of a class.** Each class writes the operands its own macro
writes, in the order of that macro: `ALTER E <grupo>,<cantidad>,<parámetro
alterado>,<valor>,<parámetro comparado>,<valor>`; `REMOVE E <grupo>,<cantidad>
,,<comparado>,<valor>`; `SCAN E <grupo>,<comparado>,<valor>,<origen>,<destino>`;
`UNLINK E <cadena>,<bloque que recibe>,<cantidad>,<comparado>,<valor>`;
`SELECT <modo> <parámetro>,<mínimo>,<máximo>,,<clase>`; `LINK <cadena>,<modo>
,<bloque que sigue>`; `LOGIC <S|R> <interruptor>`; `GATE <condición> <entidad>`;
`LOOP <parámetro>,`; `JOIN <grupo>[,<número>]`; `EXAMINE <grupo>,<número>,`;
`ASSEMBLE`/`GATHER <cantidad>`; `SPLIT <total>`; `TABULATE <tabla>`;
`TEST E <izquierda>,<derecha>`; `SAVEVALUE <nombre><operador>,<valor>`;
`ASSIGN <parámetro><operador>,<valor>`.

**The counter and the match write two statements.** The counter increments a
savevalue and copies it to a parameter (`SAVEVALUE Count+,1` and
`ASSIGN Nro,X$Count`). The match writes the transfer to its conjugate before
the match itself (`A_Match TRANSFER ,AAA` and `AAA MATCH AAA`), and the report
lists the two statements: the entries are in the transfer and the transactions
that wait are in both.

**The definitions come first.** The storages, the queue tables, the tables,
the functions, the variables and the initial values, and then the blocks. The
function writes the table of its points on a line of its own, and the
exponential function writes the standard table of the exponential
distribution, which its class does not carry.

