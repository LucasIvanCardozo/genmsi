# The granularity of the clock: what the original counts and what we count

Branch `feature/reloj-granularidad`, worktree `../genmsi-w7-reloj`, from `main`
at `0aad38b`. **This unit does not touch the engine**: what is here is
measurements, verdicts and an inventory for the unit that decides.

Everything below is measured with `crates/genmsi-core/examples/reloj.rs`, which
runs the same comparisons against the original's own reports:

```bash
cargo run --release -p genmsi-core --example reloj
```

## The questions (verbatim)

1. With what granularity does the original count the times? Its pending-event
   table (`FEC_XACT_NUMBER`) shows `BDT 3610` where ours shows `3611,344`.
2. Are the facility delays and the delays of the generator whole numbers in
   the original? A facility with a mean of 30.5: does it keep 30, 31 or 30.5?
3. How does the generator behave with dispersion? If the interval it draws is a
   whole number, the first arrival has to fall on a whole number.
4. With dispersion, the original draws the first interval and ours starts at
   the exact mean. Measure the distribution of the first arrival of the
   original with many seeds and say whether the counts can tell "the exact
   mean" from "a drawn interval", and with which test.
5. What would an integer clock change in the engine? Enumerate it, with file
   and function. Change nothing: it is a report for whoever decides.

## The verdict, in one line

**The times of the original are whole units of simulated time; ours are
continuous.** The original's simulated time is an integer (or, which is the
same thing from the outside, all its delays are whole numbers), and the
model of the original cannot even express a fractional delay.

## The probes

The measurements need models the repository does not have, so this unit built
three probes. They are **measurement scaffolding**: they live outside the
repository (in `/tmp/reloj-probes`) and the script that writes them is in the
appendix of this document, so anybody can rebuild them byte for byte. The
modules come from the benches `scripts/wine-oracle/make-benches.py` generates,
which the original opens without complaint; the only module written by hand is
`Mark` (the class that writes the clock into a parameter).

| Probe | What it is | What it measures |
|---|---|---|
| `relojEnteros.gmdl` | `GENERATE 5,2` → `ADVANCE 1000` → `TERMINATE`, plus a `Clock 3600` | the interarrival times: the pending events of the original give the arrival times (the `BDT` minus the advance) |
| `relojFacilidad.gmdl` | `GENERATE 60,30` → `FacilityUse 10,5` → `TERMINATE` | the hold times of a facility: the row of the report carries the average |
| `primeraLlegada.gmdl` | 60 × (`GENERATE 60,30` with one transaction → `MARK Param` → `TABULATE Tabla`), one `Table` of width 1, one `Clock 3600` | the first arrivals: the `MARK` writes the clock into a parameter and the `TABLE` collects the readings, so the histogram of the table is the sample |

The original runs them like any other model:

```bash
bash scripts/wine-oracle/run_oracle.sh /tmp/reloj-probes/relojEnteros.gmdl /tmp/reloj-probes/oracle
```

(the harness needs a display of its own: `XDISPLAY=:93 bash scripts/wine-oracle/run_oracle.sh ...`.
The `Xvfb` of `:99` is shared with other sessions of the machine, and a run
that finds somebody else's window reads somebody else's model).

## 1. The clock of the original: whole units

### 1.1 The original's model cannot say 30.5

The first probe of this unit was a facility with a mean of 30.5, and the
original **refuses to open the model**:

```
El modelo no puede abrirse porque el archivo presenta errores estructurales.
```

The same happens with `30,5` (the comma is the decimal separator of the
original), in the `Media` of a `FacilityUse` and in the `Media` of an
`Advance`. The operands of the original are integers, and its own manual says
so:

> **ADVANCE**, *Operadores*: "El incremento de tiempo medio. Requerido. El
> operando debe ser un nombre, **entero**, SNA, o SNA\*parámetro."
>
> **ADVANCE**, *Acción*: "Si los operandos A y B son presentados, y B no tiene
> especificado una función, ambos A y B son evaluados y un número aleatorio
> entre **A-B y A + B, inclusivamente**, es usado como el incremento de
> tiempo."
>
> **ADVANCE**, *Acción*: "Si B es un SNA de clase FN, llamada función
> modificadora, el resultado de doble precisión de evaluar B es multiplicado
> por el resultado de la evaluación del operando A. **El producto es truncado a
> un entero** y usado como el incremento de tiempo."

So the question "a facility with a mean of 30.5 keeps 30, 31 or 30.5?" has no
answer in the original: **a facility with a mean of 30.5 does not exist**. The
original's model has whole numbers only, and the arithmetic that produces a
delay truncates to an integer (the same rule for `VARIABLE`: "El resultado
final y los resultados intermedios son también truncados").

### 1.2 The totals the reports imply: 50 of 50 whole

A *difference* of two times is a whole number whether the clock is whole or
not (the original prints its times without decimals, so the difference of two
printed times is whole either way). What tells the two apart is a **sum**: the
busy time of a facility is the sum of its hold times, and the report gives it
as `AVE_TIME x ENTRIES`.

Read over the 91 reports of the original this repository has (the 66 benches
of `/tmp/all-oracle` and the 25 reference models of `fixtures/outputs`),
counting only the facilities with uses and a busy time of its own:

| | facilities with uses | whose total busy time is a whole number |
|---|---:|---:|
| the original | 50 | **50 (100.00 %)** |
| ours | 49 | **0 (0.00 %)** |

The tolerance is the precision of the report: the average is printed with
three decimals, so the total it implies is known to within `0.0005 x ENTRIES`
units. A continuous clock would land inside that window by chance in about
6 % of the cases, so 50 of 50 has a probability of about `0.06^50 = 10^-61`
under a continuous clock.

The same test on the tables: the sum of the values of a table (`MEDIA x
CANTIDAD`) is a whole number in 5 of 5 tables of the original. (A table whose
values are counts sums to a whole number in both engines, so this test only
tells when the values are times.)

And the sum of the 60 clock readings of the `primeraLlegada` probe is
**3428** exactly.

### 1.3 The interarrival times: 217 of 217 whole, and the caveat

The `relojEnteros` probe leaves 218 transactions waiting in the `ADVANCE` when
the run ends, so the report gives 218 arrival times and 217 gaps between them:

| | gaps | whole gaps | the gaps go from |
|---|---:|---:|---|
| the original | 217 | **217 (100.00 %)** | 3 to 7 |
| ours | 203 | **0 (0.00 %)** | 3.039 to 6.941 |

The gaps of the original are the whole numbers 3, 4, 5, 6 and 7: the
inclusive range `[A-B, A+B] = [3,7]` of the manual, and the endpoints 3 and 7
both appear (53 and 22 times), which is what "inclusivamente" says and what a
truncation of a continuous draw would not do (a truncation would almost never
give the upper end).

**The caveat**: this test alone does not prove that the original's clock is
whole, because the original prints its times without decimals: the difference
of two *rounded* times is whole either way. The test in 1.2 (the sums) is the
one that sees the clock itself.

### 1.4 An anomaly that this unit does not explain

The gaps of the `relojEnteros` probe are not uniform: 53 threes, 65 fours, 45
fives, 32 sixes and 22 sevens, a chi-square of 26.48 with 4 degrees of
freedom (`p = 2.5e-5`), and a mean of **4.562** against the 5.0 the model
asks for. The repository's own `AdvanceDelay` probe (a `GENERATE 60,30`
behind an `ADVANCE 1000`) shows the same thing: 19 gaps with a mean of
**50.47** against the 60.0 the model asks for.

So the original's generator produced shorter intervals than a uniform over
`[A-B, A+B]` in these two runs. That is a question about the **random
generator of the original**, not about the clock: the totals of 1.2 are whole
whatever the distribution of the gaps is, and the verdict of this document
does not depend on it. It is left here for the unit that owns the random
streams, because it would change the numbers of every model.

## 2. The first arrival with dispersion: the original draws it

The `primeraLlegada` probe holds 60 generators with one transaction each, a
mean of 60 and a dispersion of 30. Every generator draws its first interval,
and the `MARK` of its transaction writes the clock into a parameter that a
`TABLE` of width 1 collects, so the histogram of the table *is* the sample of
the first arrivals.

The sample of the original (read back from the histogram of its report):

| | n | mean | deviation | min | max | distinct values |
|---|---:|---:|---:|---:|---:|---:|
| the original | 60 | **57.133** | **17.728** | 30 | 90 | 37 |
| ours (10 seeds x 60 generators) | 600 | **60.000** | **0.000** | 60 | 60 | 1 |

The report of the original agrees: `MEDIA 57,133`, `DESVIACION ESTANDAR
17,728`, `VALOR MAXIMO TABULADO 90,000`, 60 values.

**The test.** A Kolmogorov-Smirnov test of the sample against the uniform
`U(30,90)` (the distribution our engine draws its intervals from), with the
asymptotic p-value and the Stephens correction, which the example computes:

| sample | D | p | reading |
|---|---:|---:|---|
| the original | **0.1167** | **0.3666** | not rejected: the first arrivals are consistent with a draw over `[30,90]` |
| ours | **0.5000** | **0.0000** | rejected: our first arrivals are all the exact mean |

**The counts tell the two readings apart without any test**: the reading "the
exact mean" gives 60 copies of 60, a deviation of 0 and a single class in the
histogram; the original's sample has 37 distinct values, from 30 to 90, with
a deviation of 17.728 against the 17.32 that `U(30,90)` predicts. The
Kolmogorov-Smirnov test says the same about the shape of the distribution: the
original's first arrival is **a drawn interval**, not the exact mean.

(Our engine's own numbers confirm the other half of the story: with a
dispersion of 30 our first arrival is exactly 60.000 in every one of the ten
seeds, and the gaps of the following arrivals are 0 whole out of 203.)

### What this test does not say

- The original cannot be seeded from the model, so the 60 samples are 60
  successive draws of one run, not 60 independent runs. If the original's
  draws are correlated, the effective sample is smaller than 60.
- The test says "consistent with a draw over `[30,90]`", not "the same
  distribution as the other intervals": the anomaly of 1.4 (the original's
  generator seems to draw low) is not settled by a sample of 60.
- The original's manual says the same thing, and it is worth quoting:
  "Cuando una simulación es comenzada, cualquier bloque GENERATE que no ha
  sido 'primed' es llamado para planificar su primera transacción. [...] El
  operando C puede ser usado para especificar un incremento de tiempo positivo
  para la primera transacción. **En otro caso, el primer tiempo de incremento
  es calculado desde los operandos A y B.**"

## 3. What an integer clock would change in the engine

An inventory, with file and function. **Nothing was changed.**

### 3.1 The delays of the blocks

| Where | What | What an integer clock would mean |
|---|---|---|
| `crates/genmsi-core/src/sim.rs`, `apply_modifier` (line 2660) | the symmetric spread is a continuous uniform: `mean - spread + 2*spread*draw` | the draw would have to be a **whole number** over `[A-B, A+B]` (inclusive), and the product with a function modifier would have to be **truncated** to an integer, as the manual of the original says |
| `crates/genmsi-core/src/sim.rs`, `evaluate_delay` (line 2636) | the mean and the modifier are evaluated in `f64` | the result would have to be truncated to an integer (the rule the original applies to `VARIABLE` and to the product of a modifier function) |
| `crates/genmsi-core/src/sim.rs`, `Simulation::install_definitions` (line 377) | the first arrival of a generator: the offset, or the exact mean | the first arrival would be a whole number, and (a separate decision) the *drawn* first interval of the original (section 2) would have to be drawn as a whole number too |
| `crates/genmsi-core/src/sim.rs`, `Simulation::tick` (line 690) | the next arrival of a generator: `self.now + delay` | the arrival would be a whole number |
| `crates/genmsi-core/src/sim.rs`, the `Advance` block (line 899) | `self.now + delay` and `transaction.scheduled_at` | the scheduled time would be a whole number |
| `crates/genmsi-core/src/sim.rs`, the preemption (lines 2161-2190) | `remaining = (scheduled - self.now).max(0.0)` | the time the preempted owner had left would be whole, and the `ADVANCE` it returns to would resume at a whole time |

### 3.2 The order of the events of the priority queue

| Where | What | What an integer clock would mean |
|---|---|---|
| `crates/genmsi-core/src/sim.rs`, `struct Event` (line 254) and `impl Ord for Event` (line 281) | the order is `(time, priority, sequence)` | with whole times, **many events share the same instant** and the tie-break (the priority, then the sequence) decides the order of the simultaneous events. Today the ties are rare, so that order is barely exercised |
| `crates/genmsi-core/src/sim.rs`, `Simulation::push_event` (line 466) | the sequence number, negated for the events of the hour in which the run ends (the original processes them the other way round) | with whole times, the rule would apply to many more events, and the order of the original for the events of the same instant would have to be **measured** (a probe of its own): FIFO of the queue, who wins a `SEIZE`, who is preempted |

### 3.3 The trace

| Where | What | What an integer clock would mean |
|---|---|---|
| `crates/genmsi-core/src/sim.rs`, `TraceEvent::time` (line 119) | the time of the event, an `f64` | the times of the trace would be whole numbers |
| `crates/genmsi-cli/src/commands/trace.rs`, `render` (line 20) | the trace prints the time with `{:>9.3}` | the same format would print `60.000`; the decimals would be zeros. The format itself does not have to change |
| `crates/genmsi-core/src/documents.rs`, the trace document | the times of the `genmsi.trace/1` document | the numbers would be whole. **The published contract does not change**: the same field, the same type, a whole value |

### 3.4 The report

| Where | What | What an integer clock would mean |
|---|---|---|
| `crates/genmsi-core/src/report.rs`, `event_time` (line 76) | the `BDT` of the `FEC_XACT_NUMBER` section is printed with `value.round()` | today the round **hides** the fraction of our time (the `3611,344` of the question is the raw time of ours; the report prints `3611`). With whole times the round would be exact and would hide nothing |
| `crates/genmsi-core/src/report.rs`, `number` (line 27) and `whole` (line 40) | the averages with three decimals and a comma, the counts as whole numbers | the averages would be averages of whole numbers, so their decimals would come only from the division. The original prints decimals in its averages too (`1732,291` in a chain), so **the format does not change** |
| `crates/genmsi-core/src/stat.rs`, `Facility::stop_busy` (line 115) | `busy_time += (now - seized_at)` | the busy time would be a whole number, and `AVE_TIME` (line 99) and `UTIL.` (line 90) would be the ones of the original |
| `crates/genmsi-core/src/stat.rs`, `Queue::enqueue` (line 180) and `dequeue` (line 190) | the area (`content x time`) and the waiting time | the area and the waiting time would be whole numbers |
| `crates/genmsi-core/src/stat.rs`, the chains | the area of a chain, integrated over the time | the area would be a whole number |
| `crates/genmsi-core/src/stat.rs`, the tables | the tabulated values and their mean and deviation | the tabulated values would be whole numbers (the `MARK` of the original writes a whole clock) |
| `crates/genmsi-core/src/xact.rs`, `Transaction` (lines 43-67) | `created`, `mark`, `entered_at`, `scheduled_at` | the times of a transaction would be whole numbers |

### 3.5 The clock of the app

| Where | What | What an integer clock would mean |
|---|---|---|
| `crates/genmsi-app/src/monitor/replay.ts`, the type of the event (line 232) and `advanceClock` (line 1065) | the simulated time of the events and the clock of the replay, a `number` that advances with the wall clock | the *simulated* times the app replays would be whole numbers; the clock of the animation is a wall clock and does not change |
| `crates/genmsi-app/src/monitor/live.ts` (line 42) | the `clock` of the picture | the same |
| `crates/genmsi-app/src/components/MonitorView.tsx` (lines 133-202) | the clock of the replay that the reader sees | the same |
| `crates/genmsi-app/src/components/MonitorCharts.tsx` (lines 132-150) | the times of the series of the queues and the facilities | the points of the series would fall on whole times |

### 3.6 What the change would cost, and what it would not

- The **published contracts** (`genmsi.report/1`, `genmsi.trace/1`) do not
  change: the same fields, the same types, whole values instead of fractional
  ones. The `replication.rs` metrics would carry whole times too.
- **Every model's numbers would change**: the arrival times, the waiting
  times, the hold times, and every count that depends on the order of two
  events that now share an instant. The comparison with the original would get
  closer (the original's times are whole), but it would not close by itself:
  the **order of the simultaneous events** is a separate question that this
  unit has not measured.
- The reports of the original print their times as whole numbers, so our
  reports would print the same digits in the `FEC_XACT_NUMBER` section; the
  decimals of our averages would remain (the original has them too).

## What this unit did NOT touch

- No file of the engine (`crates/genmsi-core/src/**`), no catalogue, no test
  of the engine, no contract (`genmsi.*/1`).
- No file of `fixtures/`, of `scripts/wine-oracle/out/` or of the catalogue of
  the original.
- No dependency was added.
- The probes live outside the repository (`/tmp/reloj-probes`), because the
  allowed surfaces of this unit were the notes and the example. The script
  that writes them is in the appendix, so they can be rebuilt anywhere.
- The main worktree was only read.

## How to reproduce

```bash
# the measures (the original's reports are read from /tmp/all-oracle and
# fixtures/outputs; the probes from /tmp/reloj-probes)
cargo run --release -p genmsi-core --example reloj

# the probes, and the original on them
python3 /tmp/reloj-probes/make-probes.py            # the appendix of this note
XDISPLAY=:93 bash scripts/wine-oracle/run_oracle.sh \
    /tmp/reloj-probes/relojEnteros.gmdl /tmp/reloj-probes/oracle
XDISPLAY=:93 bash scripts/wine-oracle/run_oracle.sh \
    /tmp/reloj-probes/relojFacilidad.gmdl /tmp/reloj-probes/oracle
XDISPLAY=:93 bash scripts/wine-oracle/run_oracle.sh \
    /tmp/reloj-probes/primeraLlegada.gmdl /tmp/reloj-probes/oracle
```

The example takes the directories as options when they are not where it
expects them: `--oracle`, `--fixtures`, `--models`, `--probes`, `--benches`,
`--seed` and `--seeds`.

## Appendix: the script that builds the probes

```python
#!/usr/bin/env python3
"""Build the probes of the clock-granularity work unit.

The probes are measurement scaffolding: they live outside the repository and
this script is the recipe, so anybody can rebuild them byte for byte.

The modules come from the benches the repository already generates
(`scripts/wine-oracle/benches/*.gmdl`), which the original opens without
complaint; the only module written by hand is `Mark`, whose shape is the one
of the `Assign` module of a reference model.
"""

from __future__ import annotations

import copy
import xml.etree.ElementTree as ET
from pathlib import Path

REPO = Path("/home/lucas/Documentos/Workspace/genmsi-w7-reloj")
BENCHES = REPO / "scripts/wine-oracle/benches"
OUT = Path("/tmp/reloj-probes")


def module_of(tree: ET.ElementTree, module_id: str) -> ET.Element:
    for module in tree.getroot().iter("module"):
        if module.get("moduleid") == module_id:
            return module
    raise KeyError(module_id)


def set_prop(module: ET.Element, name: str, value: str) -> None:
    for element in module.findall("props/property"):
        if element.get("name") == name:
            element.set("value", value)
            return
    raise KeyError(name)


def link(main: ET.Element, src: str, src_io: str, dst: str, dst_io: str) -> None:
    element = ET.SubElement(main.find("links"), "link")
    element.set("frommoduleid", src)
    element.set("fromioid", src_io)
    element.set("tomoduleid", dst)
    element.set("toioid", dst_io)


def mark_module(module_id: str, name: str, y: int) -> ET.Element:
    module = ET.Element(
        "module",
        {
            "moduleid": module_id,
            "moduleclass": "Mark",
            "owner": "0",
            "caption": "Mark",
            "name": name,
            "kind": "A",
            "width": "96",
            "height": "36",
            "x": "120",
            "y": str(y),
        },
    )
    inputs = ET.SubElement(module, "inputs")
    ET.SubElement(
        inputs,
        "input",
        {"ioid": "1", "name": "Entrada1", "caption": "", "mylabel": f"A_{name}"},
    )
    outputs = ET.SubElement(module, "outputs")
    ET.SubElement(
        outputs,
        "output",
        {"ioid": "2", "name": "Salida1", "caption": "", "mylabel": "A_Terminate"},
    )
    props = ET.SubElement(module, "props")
    ET.SubElement(
        props,
        "property",
        {
            "name": "Parametro",
            "hint": "Parametro%20donde%20se%20guarda%20la%20hora",
            "value": "Param",
            "type": "EntityUse",
            "subType": "Parameter",
            "visible": "1",
            "unique": "0",
        },
    )
    return module


def write(tree: ET.ElementTree, name: str) -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    tree.write(
        OUT / f"{name}.gmdl", encoding="latin-1", xml_declaration=False, short_empty_elements=True
    )
    print(name, (OUT / f"{name}.gmdl").stat().st_size, "bytes")


def probe_enteros() -> None:
    """A fast generator with dispersion behind an ADVANCE of 1000.

    At the end of the run the transactions that arrived in the last 1000 units
    are still waiting: the FEC of the original shows their arrival time (the
    BDT minus the 1000 units of the advance).
    """
    tree = ET.parse(BENCHES / "Advance.gmdl")
    root = tree.getroot()
    modules = root.find("modules")
    main = module_of(tree, "0")
    generate = module_of(tree, "1")
    advance = module_of(tree, "2")

    set_prop(generate, "Media", "5")
    set_prop(generate, "Modificador", "2")
    set_prop(advance, "Media", "1000")
    set_prop(advance, "Modificador", "0")

    for element in list(main.find("links")):
        main.find("links").remove(element)
    link(main, "1", "1", "2", "1")
    link(main, "2", "2", "3", "1")
    write(tree, "relojEnteros")


def probe_facilidad() -> None:
    """A generator with dispersion in front of a facility with dispersion."""
    tree = ET.parse(BENCHES / "FacilityUse.gmdl")
    root = tree.getroot()
    modules = root.find("modules")
    main = module_of(tree, "0")
    generate = module_of(tree, "1")
    facility = module_of(tree, "2")

    set_prop(generate, "Media", "60")
    set_prop(generate, "Modificador", "30")
    set_prop(facility, "Media", "10")
    set_prop(facility, "Modificador", "5")

    for element in list(main.find("links")):
        main.find("links").remove(element)
    link(main, "1", "1", "2", "1")
    link(main, "2", "2", "3", "1")
    write(tree, "relojFacilidad")


def probe_primera_llegada(count: int = 60) -> None:
    """`count` generators, one transaction each, all tabulating the clock."""
    tree = ET.parse(BENCHES / "Tabulate.gmdl")
    root = tree.getroot()
    modules = root.find("modules")
    main = module_of(tree, "0")
    table = module_of(tree, "1")
    generate = module_of(tree, "2")
    tabulate = module_of(tree, "3")
    terminate = module_of(tree, "4")
    clock = module_of(tree, "5")

    # The table reads the parameter the MARK writes, with classes of one unit.
    set_prop(table, "Atributo", "P%3AParam")
    set_prop(table, "TamFrecuencia", "1")
    set_prop(table, "Cantidad", "100")
    set_prop(generate, "Cantidad", "1")
    set_prop(generate, "Media", "60")
    set_prop(generate, "Modificador", "30")
    set_prop(tabulate, "Peso", "1")
    table.set("moduleid", "900")
    terminate.set("moduleid", "999")
    clock.set("moduleid", "1000")

    first_generate = copy.deepcopy(generate)
    first_tabulate = copy.deepcopy(tabulate)
    for module in (generate, tabulate, table, terminate, clock):
        modules.remove(module)

    new_modules = [table]
    links = []
    next_id = 1
    for index in range(count):
        gen = copy.deepcopy(first_generate)
        gen.set("moduleid", str(next_id))
        gen.set("name", f"Generate{index}")
        gen.set("y", str(40 + 120 * index))
        gen.find("outputs/output").set("mylabel", f"A_Mark{index}")
        mark = mark_module(str(next_id + 1), f"Mark{index}", 160 + 120 * index)
        tab = copy.deepcopy(first_tabulate)
        tab.set("moduleid", str(next_id + 2))
        tab.set("name", f"Tabulate{index}")
        tab.set("y", str(280 + 120 * index))
        tab.find("inputs/input").set("mylabel", f"A_Tabulate{index}")
        tab.find("outputs/output").set("mylabel", "A_Terminate")
        new_modules.extend([gen, mark, tab])
        links.append((str(next_id), "1", str(next_id + 1), "1"))
        links.append((str(next_id + 1), "2", str(next_id + 2), "1"))
        links.append((str(next_id + 2), "2", "999", "1"))
        next_id += 3

    new_modules.extend([terminate, clock])
    for module in new_modules:
        modules.append(module)

    for element in list(main.find("links")):
        main.find("links").remove(element)
    for src, src_io, dst, dst_io in links:
        link(main, src, src_io, dst, dst_io)

    write(tree, "primeraLlegada")


def main() -> None:
    probe_enteros()
    probe_facilidad()
    probe_primera_llegada()


if __name__ == "__main__":
    main()
```

## Epílogo: la anomalía del generador que "sorteaba bajo" (refutada)

La sección 1.4 dejó anotado que el generador del original sortea bajo (los
huecos de dos corridas promediaban 4,562 contra 5,0, con chi2 26,48 y 4
grados de libertad). **Eso no se sostiene**, y la prueba es de conteo, no de
densidad:

- La medición de 1.4 mira **los últimos 1000 unidades de una sola corrida**.
  Medida la corrida **entera**, el generador de la sonda tiene 733 entradas
  (732 llegadas más la pendiente de la FEC) contra las **719,63 ± 7,59** que
  predice la regla de la manual (uniforme entera sobre `[A-B, A+B]`): **z =
  +1,63**. La media de los huecos de la corrida entera es 4,912.
- La segunda sonda (`AdvanceDelay`, 60,30): 61 llegadas contra 59,55 ± 2,30
  esperadas → **z = +0,63**.
- **La prueba decisiva**: 51 corridas del original con un generador `60,30`
  sobre 3600 dan **59,29 ± 0,32** llegadas contra las **59,55 ± 2,30**
  esperadas → **z = −0,80**, y el histograma de los conteos da **chi2 4,75 con
  4 grados de libertad (p ≈ 0,32)** contra la distribución simulada de la
  regla. Una media baja (57) pediría 63,1 llegadas: **11,6 sigma** lejos. El
  conteo es una estadística de la corrida entera, insensible a dónde se mire.

O sea: **el generador del original no sortea bajo**.

Lo que sí queda sin explicar es otra cosa: la ventana final de esas dos
corridas es **más densa** que el modelo de renovación (218 llegadas contra
200,03 ± 3,99, **z = +4,5**; y 20 contra 16,67 ± 1,28, **z = +2,6**), y las
dos ventanas de la *misma* corrida difieren **4,8 sigma** (los primeros 514
huecos promedian 5,06 y los últimos 217, 4,56). El ACF de la serie de huecos
no muestra periodicidad (|ACF| ≤ 0,14 en los retardos 1 a 5). Es una
pregunta sobre el **flujo aleatorio** del original (o sobre la ventana
elegida), no sobre la distribución del generador. Queda anotada, con los
guiones reproducibles en `/tmp/reloj-analisis/` (fuera del repositorio) sobre
los reportes del original que ya estaban en `/tmp/reloj-probes/oracle/` y
`/tmp/all-oracle/`.

## Epílogo: la primera llegada, ya sorteada

La unidad del core aplicó lo que esta nota midió (sección 2): la primera
llegada de un generador **con dispersión** es un intervalo sorteado con la
misma regla que los de adelante (`apply_modifier`, del flujo propio del
bloque), y sin dispersión sigue siendo la media exacta, **sin consumir
números** del flujo. Con `offset` (operando C) manda el offset, como dice el
manual.

**Una desviación que queda anotada**: cuando el modificador es una **función**
(`GENERATE 60,FN$Expon`), la primera llegada sigue siendo la media. El
manual dice que el primer incremento "es calculado desde los operandos A y
B", o sea que la función debería entrar; pero aplicarla consume un número
del flujo de esa función y mueve todas sus evaluaciones siguientes, y la
medición no da evidencia a favor: el TP4 queda en 56 de 65 (contra 62 de 65
de la base) y el Parcial sube a 138 de 150 (contra 134 de 150). Queda como
decisión de quien coordina; es una línea en `Simulation::first_arrival`.
