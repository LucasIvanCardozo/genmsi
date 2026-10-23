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

- numbers, `+ - * /`, integer division `#`, modulo `@`, parentheses, unary minus;
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

- **GENERATE A,B,C,D,E**: the first transaction arrives at `C` when given,
  otherwise at `A`; then interarrivals are drawn. `D` limits the total; an empty
  `A` means a zero interarrival (models use it with `D` to create one transaction
  at time 0). Verified against TP10's `GENERATE ,,,1` (one transaction) and
  TP6's `GENERATE ,,3600,1` (one at 3600).
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
