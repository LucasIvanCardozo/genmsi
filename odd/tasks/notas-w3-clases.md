# Work unit 3 — the three classes the catalogue did not offer

`MFunction`, `MFunctionDist` and `RemoveNumber` are the last three classes of
the original's library that the catalogue did not offer. This work unit
declares them, compiles them and pins what they do.

The original is the specification and its answer is the measure. Everything
below that says "the original" comes from its own listing and its own report,
read from the runs of the bench of the class.

## What the catalogue declares now

The shape of the three classes comes from the original's class library
(`Genmsi_20230711.backup/gClasses.xml`): the ports its graph declares and the
properties its macro code declares, with the default value the macro code
carries.

| Class | Ports | Properties (default) |
|---|---|---|
| `MFunction` | none | `NombresFunc` (`TEntityDefList`, `Function`, `{FuncA;FuncB}`), `Atributo` (`TSNA`, `P:Param`), `Matriz` (`TSNAMATRIX`, `{{11;12;13};{21;22;23}}`) |
| `MFunctionDist` | none | `NombresDist` (`TEntityDefList`, `Function`, `{Func1;Func2}`), `Semilla` (`PosInteger`, `4`), `Resultados` (`FloatPercentMatrix`, `{{20%;30%;50%};{25%;35%;40%}}`) |
| `RemoveNumber` | `Entrada1` in; `Salida1` and `Salida2` ("No existe el Nº") out | `Op` (`Listed`, `op_RelacMaxMin`, `=`), `Grupo` (`EntityUse`, `NumericGroup`, `UnGrupo`), `Numero` (`TSNA`, `0`) |

The value of `Resultados` is the one the macro code carries, with the line
breaks the macro code writes inside it (`{\n{20%;30%;50%};\n{25%;35%;40%}\n}`).
The bench of the class carries the same text, percent-encoded.

The two types the original's own models never carry (`TEntityDefList`,
`TSNAMATRIX`, `FloatPercentMatrix`) are the ones the original itself writes
when it saves a model that carries these modules: a model that declares them
with any other type is "adapted" by the original when it opens it, and the
window that says so stops the oracle from reading the answer
(`scripts/wine-oracle/README.md`).

## `MFunction`

**The original.** "Define múltiples funciones para los mismo puntos X y el
mismo atributo de entrada." (its `ClassDescrip`). Its macro code is
`MULTIFUNCTION [NombresFunc:'lista de nombres de las Funciones'='{FuncA;FuncB}'],
[Atributo:'Variable independiente de todas las funciones'='P:Param'],
[Matriz:'Matriz que define las salidas de las funciones para cada valor de
entrada.'='{{11;12;13};{21;22;23}}']`.

**The rule.** One statement per name of the list, with the row of the matrix
that goes with it: the function takes the value of the attribute and the row
is the value of the function for each input (the first column is the input
`1`, the second the input `2`, ...). The original's own listing of the bench:

```
FuncA           FUNCTION        P$Param,E3
1,11/2,12/3,13
FuncB           FUNCTION        P$Param,E3
1,21/2,22/3,23
```

**What we do.** The compiler reads the list of names and the matrix and
declares one function per name: a function of type `E` (the `Evaluate` of the
engine, the one the `Function` class already uses when its `Tipo` is `Eval`),
with the argument the class declares and the row of the matrix as its points.
The listing is written by the arm that `gpss.rs` already has for the
`Evaluate` function, so **the listing of the class is the original's, word for
word** (`bash scripts/wine-oracle/compare-gpss.sh` on the two files says
"el GPSS coincide: 4 bloques y 0 conexiones").

The test that pins it is `crates/genmsi-core/tests/clases_ausentes.rs`, fn
`the_mfunction_writes_a_function_for_every_name_of_its_list` (the listing) and
fn `the_mfunction_function_gives_the_value_of_the_row_of_its_name` (the
behaviour: a transaction that reads `FuncA` with the input `1` gets `11`, and
one that reads `FuncB` gets `21`).

## `MFunctionDist`

**The original.** The same `ClassDescrip` as the `FunctionDist`, with a macro
code of its own: `MULTIFUNCTIONDIST [NombresDist:'Nombres de las funciones de
distribución'='{Func1;Func2}'], [Semilla:'número de semilla utilizada para
calcular los números aleatorios'='4'], [Resultados='{{20%;30%;50%};{25%;35%;40%}}']`.

**The rule.** One statement per name of the list, with the row of the matrix
of results that goes with it: a distribution function (the `D` of the engine,
the one the `FunctionDist` class already uses) whose seed is the `Semilla` of
the class and whose probabilities are the ones of the row. The original's own
listing of the bench:

```
Func1           FUNCTION        RN4,D3
0.2,1/0.5,2/1,3
Func2           FUNCTION        RN4,D3
0.25,1/0.6,2/1,3
```

**What we do.** The compiler reads the list of names and the matrix of
percentages and declares one distribution function per name, with the running
sum of the probabilities of its row. The listing is written by the arm that
`gpss.rs` already has for the `Distribution` function, so **the listing of the
class is the original's, word for word**.

The tests are fn `the_mfunction_dist_writes_a_distribution_for_every_name_of_its_list`
(the listing) and fn `the_mfunction_dist_function_follows_the_probabilities_of_its_row`
(the behaviour: a thousand transactions read the function and the three
results come out 20%, 30% and 50% of the time).

## `RemoveNumber`

**The original.** It has no `ClassDescrip`, but its macro code says what it
does: `REMOVENUMBER [Op:'Operador relacional'='='], [Grupo:'Grupo numérico al
que se agrega un número'='UnGrupo'], [Numero:'Número agregardo al grupo'='0'],
Salida2:'No existe el Nº' OUT Salida1`. The manual of the original
(`Genmsi_20230711.backup/VISUALSIS.HLP`, the `REMOVE` entry) says what the
statement means:

> Un bloque REMOVE opera en modo numérico si el operando C es usado. En modo
> numérico, los operandos A y C son evaluados y el número especificado por C
> es testeado con respecto a números de miembros en el grupo numérico
> especificado por A. Si este número es un miembro del grupo numérico, este es
> removido del grupo. Si el valor numérico no es un miembro del grupo numérico
> y el operando F es usado, la transacción de entrada procede al bloque
> especificado por F. En otro caso, la transacción de entrada procede al NSB.
> Únicamente A, C, y F pueden ser usados en modo numérico.

The class writes that statement: `REMOVE E <grupo>,,<número>` (A the group, B
empty, C the number), and with the failure output connected the destination as
the seventh operand (`REMOVE E <grupo>,,<número>,,,<destino>`). The original's
own listing of the bench is `A_RemoveNumber  REMOVE   E       UnGrupo,,0`.

**What we do.** The class compiles to a `REMOVE` whose number is the `Numero`
of the class and whose group is the `Grupo`, and the transaction goes to the
failure output ("No existe el Nº") when the number is not in the group, as the
manual says.

### The ficha of the third lot says "agrega"; the original's report says it does not

`odd/tasks/notas-fase7-lote3.md` (the `RemoveNumber` fiche) says that the
statement **adds** the number to the numeric group ("a diferencia del `REMOVE`
de transacciones, que quita"). The report of the original says otherwise:

```
XACT_GROUP           GROUP_SIZE      RETRY
  UNGRUPO                    0          0
```

The group is a **transaction** group (`XACT_GROUP`), it is empty, and there is
no `NUMERIC_GROUP` row at all — the number `0` was never added to a numeric
group (the bench of the `JoinNumber` class, whose statement *is* a `JOIN`,
prints `NUMERIC_GROUP UNGRUPO 1 0`, and the one of the `ExamineNumber` class
prints `NUMERIC_GROUP UNGRUPO 0 0`). The hint of the property ("al que se
agrega un número") is a copy of the hint of the `JOIN` class: the `RemoveMe`
class carries the very same hint for its group, and that class takes the
transaction *out* of its group. The rule we follow is the manual's.

### What is not done, and the exact patches

`crates/genmsi-core/src/gpss.rs`, `crates/genmsi-core/src/report.rs` and
`crates/genmsi-core/src/sim.rs` are the work of another unit and were not
touched. The listing of the `RemoveNumber` class therefore cannot be pinned
yet, and the removal of the number cannot be done on a numeric group.

**1. `crates/genmsi-core/src/program.rs`** — the `RemoveSpec` needs a field
for the number of the numeric mode (this is the change the two patches below
assume):

```rust
pub struct RemoveSpec {
    /// Transaction group name.
    pub group: String,
    /// Maximum number removed.
    pub count: ProgramExpr,
    /// Remove all matches.
    pub all: bool,
    /// Parameter compared.
    pub parameter: Option<String>,
    /// Value compared.
    pub value: Option<ProgramExpr>,
    /// Whether the transaction itself is the one removed.
    pub me: bool,
    /// The number of the numeric mode (`REMOVE E <grupo>,,<número>`).
    pub number: Option<ProgramExpr>,
}
```

and the arm of `Compiler::expand` writes `number: Some(number)` and
`value: None` (today it writes `value: Some(number)`).

**2. `crates/genmsi-core/src/gpss.rs`** — fn `block_statements`, the arm of
`BlockKind::Remove`:

```rust
        BlockKind::Remove(spec) => {
            if spec.me {
                statement(out, "REMOVE", &spec.group);
            } else if let Some(number) = &spec.number {
                // The `REMOVE` in numeric mode: the group, an empty limit,
                // the number, and the destination of the failure output as
                // the seventh operand (`REMOVE E UnGrupo,,0` and
                // `REMOVE E UnGrupo,,0,,,A_Terminate1`).
                let mut operands = format!("E {},,{}", spec.group, sna(number));
                let destination = port_destination(block, names, "Salida2");
                if !destination.is_empty() {
                    operands.push_str(",,,");
                    operands.push_str(&destination);
                }
                statement(out, "REMOVE", &operands);
            } else {
                let operands = format!(
                    "E {},{},{},{},{}",
                    spec.group,
                    sna(&spec.count),
                    "",
                    spec.parameter.clone().unwrap_or_default(),
                    spec.value.as_ref().map(sna).unwrap_or_default(),
                );
                statement(out, "REMOVE", &operands);
            }
        }
```

**3. `crates/genmsi-core/src/report.rs`** — the row of the block, fn
`operands`:

```rust
        BlockKind::Remove(spec) => {
            if spec.me {
                return (Some(""), vec![spec.group.clone()]);
            }
            if let Some(number) = &spec.number {
                // `A_RemoveNumber  REMOVE  E  UnGrupo,,0`: the group, an
                // empty limit, the number, and the destination of the
                // failure output as the seventh operand.
                let mut refs = vec![spec.group.clone(), String::new(), expr_operand(number)];
                if let Some(label) = port_target_label(names, block, "Salida2") {
                    refs.push(String::new());
                    refs.push(String::new());
                    refs.push(label);
                }
                mode = Some("E");
                return (mode, refs);
            }
            ... (the rest of the arm, as it is today)
        }
```

**4. `crates/genmsi-core/src/sim.rs`** — fn `execute`, the arm of
`BlockKind::Remove`: the number is taken out of the **numeric** group, and
the transaction goes to the failure output when the number is not a member of
it:

```rust
                if let Some(number) = &spec.number {
                    let value = {
                        let mut ctx = CtxView { sim: self, xact, depth: 0 };
                        number.eval(&mut ctx)
                    };
                    let removed = self
                        .numeric_groups
                        .get_mut(&spec.group)
                        .and_then(|members| {
                            members
                                .iter()
                                .position(|member| (member - value).abs() < 1e-9)
                                .map(|position| members.remove(position))
                        })
                        .is_some();
                    let target = if removed {
                        block.next_named("Salida1")
                    } else {
                        block.next_named("Salida2")
                    };
                    self.leave(xact, block_index, target);
                    return Ok(());
                }
```

Until that patch lands, the engine compiles the class and routes the
transaction as the manual says (to the failure output when the number is not
in the group), but the removal itself acts on a **transaction** group, which
is what the `REMOVE` of the engine does today.

**5. `crates/genmsi-format/tests/blocks.rs`** — the table
`CLASS_LIBRARY_DEFAULTS` is the transcription of the original's class library
that the test `the_catalogue_gives_no_default_the_class_library_does_not_declare`
checks the catalogue against. The three classes are not in it yet, so that one
test fails until the nine rows below are added:

```rust
    ("MFunction", "Atributo", "P%3AParam"),
    ("MFunction", "Matriz", "%7B%7B11%3B12%3B13%7D%3B%7B21%3B22%3B23%7D%7D"),
    ("MFunction", "NombresFunc", "%7BFuncA%3BFuncB%7D"),
    ("MFunctionDist", "NombresDist", "%7BFunc1%3BFunc2%7D"),
    ("MFunctionDist", "Resultados", "%7B%0A%7B20%25%3B30%25%3B50%25%7D%3B%0A%7B25%25%3B35%25%3B40%25%7D%0A%7D"),
    ("MFunctionDist", "Semilla", "4"),
    ("RemoveNumber", "Grupo", "UnGrupo"),
    ("RemoveNumber", "Numero", "0"),
    ("RemoveNumber", "Op", "%3D"),
```

## What the measurements say

**The listing.** The two functions are the original's, word for word:

```
$ bash scripts/wine-oracle/compare-gpss.sh ours-MFunction.gpss.txt /tmp/all-oracle/MFunction.gpss.txt
el GPSS coincide: 4 bloques y 0 conexiones
$ bash scripts/wine-oracle/compare-gpss.sh ours-MFunctionDist.gpss.txt /tmp/all-oracle/MFunctionDist.gpss.txt
el GPSS coincide: 4 bloques y 0 conexiones
```

The `RemoveNumber` differs in the shape of the operands (the pending patch of
`gpss.rs`):

```
  < A_RemoveNumber	REMOVE E UnGrupo,,,,0
  ---
  > A_RemoveNumber	REMOVE E UnGrupo,,0
```

**The numbers.** With the original's own report of every bench
(`/tmp/all-oracle`), the three classes give:

```
MFunction                       3600    0/  0     0%  no-metrics
MFunctionDist                   3600    0/  0     0%  no-metrics
RemoveNumber                    3600    2/  2   100%  no-metrics
```

**The report of the `RemoveNumber` bench** against the original's:

| | The original | Ours |
|---|---|---|
| The group | `XACT_GROUP UNGRUPO 0 0` | `XACT_GROUP UNGRUPO 0 0` |
| The row of the block | `REMOVE E UnGrupo,,0` | `REMOVE E UnGrupo,,,0` (the pending patch) |
| The entries of the block | 62 | 60 (the first arrival, the known difference) |

**The whole oracle** (`bash scripts/wine-oracle/compare-benches.sh
scripts/wine-oracle/benches /tmp/bench-out-new`): 62 of 62 listings and 61 of
62 sets of metrics, the same as before this work unit.
