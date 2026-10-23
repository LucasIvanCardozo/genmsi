# wine-oracle

Tools that read the original GenMSI editor (`Genmsi_20230711.backup/genmsi.exe`)
without running it. The original is a Delphi program, so its forms — the
windows and dialogs it shows — are stored inside the executable as resources,
in Delphi's binary form format (`TPF0`). These tools turn those bytes into
text, so the original interface can be read instead of photographed.

Nothing here needs Wine, and nothing here modifies the repository.

(The two tools of the last section, `make-benches.py` and `run-benches.sh`, do
need Wine: they run the original.)

Two of the tools read the original's own baseline reports
(`fixtures/outputs/`) instead of its executable: `first_arrival_evidence.py`
and `compare_generate.py`, the evidence of the first arrival of a generator.

## `dfm.py` — extract the forms

```
python3 scripts/wine-oracle/dfm.py Genmsi_20230711.backup/genmsi.exe /tmp/dfm-out
```

It opens the executable, finds every `TPF0` marker, parses the form that
follows and writes one text file per form into the output directory (a
temporary directory when none is given; the path is printed on stdout).

The executable must exist and must be a PE file: otherwise the tool says so
and exits with a non-zero status. It never writes to the executable.

On the original it finds **55 forms**. One more `TPF0` marker exists in the
file, inside a bitmap resource; it does not hold a form and is ignored (the
count is reported on stdout).

### Output format

One file per form, named after the object (`FormAbout.dfm`), written in the
layout Delphi uses for a text form:

```
object FormAbout: TFormAbout
  Left = 337
  Top = 221
  BorderStyle = bsDialog
  Caption = 'Acerca de Genmsi'
  ClientHeight = 355
  ...
  object Panel1: TPanel
    Left = 0
    Top = 0
    ...
  end
end
```

Values are written the way Delphi writes them: strings between quotes, sets
between brackets, booleans as `True`/`False`, collections as `< item ... end >`
and lists as `( ... )`.

## `dfm-summary.sh` — read the list of forms

```
bash scripts/wine-oracle/dfm-summary.sh Genmsi_20230711.backup/genmsi.exe
```

It runs the extractor in summary mode and prints one line per form: its
class, its name and its caption.

```
TFormAbout                   FormAbout                      Acerca de Genmsi
TFormClassDescription        FormClassDescription           Clase de módulo
TFormReplaceClass            FormReplaceClass               Reemplazo de clase
...
```

## The binary format that is parsed

* `TPF0` marks the start of a form.
* An object is a class name and an object name, both as strings.
* A string is a length and its bytes: one byte of length, or three bytes
  (little endian) when that first byte is 255.
* Properties follow, one after another: the name of the property, then a byte
  with the type of the value, then the value. Type `0` ends the properties of
  the object; a `0` where an object would start ends it.
* The value types are: 1 list, 2 int8, 3 int16, 4 int32, 5 float, 6 string,
  7 identifier, 8 false, 9 true, 10 binary, 11 set, 12 short string, 13
  double, 14 collection, 16 wide string, 17 int64, 18 and 20 strings with a
  four byte length.
* A descendant class is announced by a marker byte: `0xF1` before the class
  name of the form, `0xF1` or `0xF2` before a child object.

The parser is checked against the PE resource table: for every form, the
bytes it reads end exactly where the resource ends.

## `make-benches.py` — one bench per class

```
python3 scripts/wine-oracle/make-benches.py [outdir]
```

The original declares its classes in `Genmsi_20230711.backup/gClasses.xml`:
the graph section gives the ports of every class, the macro code gives its
properties and their default values. This tool reads that file and writes one
`.gmdl` **bench** per class — the smallest model that exercises exactly one
class:

```
Generate  ->  [the class under test]  ->  Terminate
```

The class under test keeps its own ports: the one input is connected to the
`Generate`, the `Salida1` output to the `Terminate`, and the other outputs
stay unconnected (the caption of the block says which ones). A `Clock` closes
the run, because the original only stops when the start counter reaches zero.

The output is deterministic: the same XML always gives the same bytes. The
benches are generated artefacts and live in `scripts/wine-oracle/benches/`,
which is in `.gitignore`.

Some classes need something the XML does not give, and the tool says so on
stdout:

* `Variable` — the default equation is empty and the original refuses to
  simulate it; the bench uses `0`.
* `Match` — the default label to match (`BBB`) is one no block defines; the
  bench matches the label the class itself defines (`AAA`).
* `GateM` — the gate waits for a label, and only a `Match` block can create
  one; the bench carries a `Match` in front and the gate waits for its label.
* `Transfer` — the default probability is empty and the original refuses to
  run an unconditional transfer without it; the bench uses `100%`.
* `Seize`, `Release`, `Return` and `Preempt` — the original refuses to end a
  transaction that owns a facility, and refuses a release of a facility
  nobody owns, so these benches carry the block that completes the pair
  (`Seize`/`Release`, `Seize`/`Return`, `Preempt`/`Return`).

Three rules decide that the bench asks the original the same question the
rest of the benches ask — the same operands, the same names and the same
entities:

* **The label of a block.** The macro of the original says what it writes on
  the input of the class (`IN Entrada1:'Ingreso'='moduleName'`): the name of
  the module with the `A_` in front of it (`A_moduleName`, what most classes
  do), the name of the module with nothing in front of it (`moduleName`: the
  four gates, the multifurcations and the storage macro), or a literal of its
  own (`Clona` for the split, `Entrada1` for the transfer). The generator
  reads that token and writes it as the label of the block. The blocks that
  end a run are named `Terminate`, `Terminate1`, ... , the way the original
  names them, and the label of the link is the label of the block it feeds.
* **The entities the class uses.** The original stops the run with
  "Referencia a una entidad storage inexistente" when a transaction enters a
  storage that nobody declared, so the bench of the `StorageUse` carries the
  `StorageDef` that declares the storage it uses.
* **The destinations the statement needs.** The `EXAMINE` writes three
  operands: the group, the number (empty in transaction mode) and the block
  the transaction goes to when it is not in the group. The last one is
  required, and the original refuses the model with "Faltan parámetros"
  without it, so the benches of the `Examine` and of the `ExamineNumber`
  connect their failure output to a terminator of its own.

Fifteen classes declare no ports at all (`Clock`, `Note`, the definitions,
the functions). Their bench is the `Generate`/`Terminate` chain with the class
in the model, unconnected.

Two properties the XML does not describe are read from the original itself:
the `Listed` subtypes and their options (`op_Logic_Faci`, `op_Logic_Tran`, ...)
come from the original's own configuration
(`Genmsi_20230711.backup/gmacroconfig.gmcfg`), and the type of a property
comes from what the original writes in the model file.  The types of the
three classes the XML declares but no model of the original carries are
`TEntityDefList` (a list of names), `TSNAMATRIX` (a matrix of numbers) and
`FloatPercentMatrix` (a matrix of percentages): they are the types the
original itself writes when it saves a model that carries those modules, and
they are the ones the generator writes.  A type the original does not know
makes it open the model by "adapting" the module, and the window that says so
stops the oracle from reading the answer.  A `Listed` value that the original
does not accept makes it refuse the model with "el archivo presenta errores
estructurales".

The generator has its own tests, `test_make_benches.py`. They build a little
catalogue of their own, so they run without the backup of the original:

```
python3 -m unittest discover -s scripts/wine-oracle -p "test_*.py"
```

## `run-benches.sh` — run the original over the benches

```
bash scripts/wine-oracle/run-benches.sh [benches-dir] [results-dir]
```

It runs the harness (`run_oracle.sh`) over every bench and leaves the
original's own files next to it:

```
<results>/Advance.report.txt    what the class does
<results>/Advance.gpss.txt       how the class compiles
```

The run does not stop at the first bench that fails: every bench is tried and
the summary at the end says how many ran and how many did not. After every
bench the Wine session is killed: when the harness fails it leaves the
original alive, and its window would be found by the next bench, which would
then drive the wrong window.

Every bench runs.  Three of them used not to: `MFunction`, `MFunctionDist` and
`RemoveNumber`.  The original opened those models, showed its "Información"
dialog ("uno o mas de los modulos ... fueron adaptados a las definiciones de
clases existentes en este sistema") and never showed its own window, so the
harness had nothing to click.  The models were accepted by the original, but
adapted: the properties of those modules carried a type the original does not
know (`EntityDefList` instead of `TEntityDefList`, `TSNAMatrix` instead of
`TSNAMATRIX`, and a `Listed` subtype the class does not declare).  With the
types the original itself writes, the notice does not appear and the three
benches answer like the rest.

Two things can still keep the harness from reading an answer: a window of the
adaptation notice (the original opens the model anyway, but the notice has to
be accepted with its "Aceptar" button before the simulator can be driven), and
a second session running the original under the same Wine prefix (the
`wineserver -k` that ends a run kills the other session's simulation too).
`GENMSI_WINEPREFIX` points the harness at a prefix of its own.

## `first_arrival_evidence.py` — when the first transaction of a generator arrives

```
python3 scripts/wine-oracle/first_arrival_evidence.py [model-fragment ...]
```

It reads the original's baseline reports (`fixtures/outputs/*.report.txt`) and
prints, for every `GENERATE` row, the horizon, the original's entry count and
what each reading of the first arrival predicts.

The original counts the transaction that is already on its way: the `ENTRY
COUNT` of a generator row is the arrivals that already happened plus the one
still pending in the FEC. The script applies that counting model (proved in
`odd/tasks/notas-primera-llegada.md`), which is what makes the comparison with
the two readings exact instead of approximate.

The two readings are:

- `media`: the first increment is `A` (the mean), so the first arrival is at
  `t = A`;
- `cero`: the first arrival is at `t = 0`.

A row prints `MEDIA` when the original's count is exactly what the first
reading predicts, `CERO` when it is exactly what the second predicts, and
`no discrimina` when both predict the same number. Only a `GENERATE` without
dispersion, without offset and without limit can tell the two readings apart:
the eighteen models that do are the evidence that the first arrival happens at
the mean.

With an argument, only the models whose name contains it are printed.

## `compare_generate.py` — our counts next to the original's

```
python3 scripts/wine-oracle/compare_generate.py [seed]
```

The same table with our own engine on the side: for every model of
`fixtures/models` that has a baseline report, it reads the original's report,
runs our engine with the horizon the original declares and the given seed,
matches the `GENERATE` rows by their operands and prints both counts next to
the two readings.

It needs the release binary (`cargo build --release -p genmsi-cli`).

## `probes/` — experiments that settle a question

A bench shows what one class does. A **probe** is an experiment: a model built
so that the question cannot be answered by reading the text, run in the
original and in our engine, and the numbers of the two reports compared.

```
bash scripts/wine-oracle/probes/run-probe.sh
```

Each probe directory holds the models, what the original produced
(`oracle/`) and what our engine produced (`ours/`), plus a note with the
question, the numbers and the verdict.

### `TransferProbOrder` — which order does the original's engine follow for a `Transfer`?

The original writes the destinations of a `TRANSFER` in the reverse of the
port order, and in GPSS that order is the meaning of the probability, so the
text of the original contradicts the property hint of its own block
("Probabilidad de continuar por la Salida1"). The probe gives `Salida1` a long
`Advance` (1000) and `Salida2` a short one (10), with `Prob` at 85% and at
15%, and compares the entry counts of the two advances.

The verdict: the original's **engine follows the port order** — the probability
goes to `Salida1` (53 of 360 transactions at `Prob` 15%, 300 of 360 at 85%) —
and only the **text** it writes is reversed. Our engine already behaves that
way; our GPSS rendering is the part that has to follow the original's order.

The details and the numbers are in
[`probes/TransferProbOrder.md`](probes/TransferProbOrder.md).
