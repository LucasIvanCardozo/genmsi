# Probe — which order does the original's engine follow for a `Transfer`?

**Question (odd/tasks/genmsi-fase6-todas-las-piezas.md, T4/S4, L5 point 1).**
The original writes the destinations of a `TRANSFER` in the reverse of the port
order. In GPSS the order of the destinations *is* the meaning of the
probability, so the text of the original says the opposite of what its own
property hint says ("Probabilidad de continuar por la Salida1"). Which of the
two does the original's **engine** do?

**Answer.** The original's engine follows the **port order**: the probability
goes to `Salida1`, the port the property hint names. Only the **text** is
reversed. Our engine already behaves that way; our **GPSS rendering** is the
part that has to change (it must write the destinations in the original's
reversed order).

## The probe

Three models, all in this directory, all the same shape:

```
Generate ──► Transfer ──┬─ Salida1 ─► Advance 1000 ─► Terminate   (largo)
 Prob = 85% / 15%       └─ Salida2 ─► Advance   10 ─► Terminate   (corto)
Clock (3600) closes the run.
```

* `TransferProbOrder.gmdl` — `Prob` = 85%, `Generate` 60,30 (≈60 transactions).
* `TransferProbOrder85.gmdl` — `Prob` = 85%, `Generate` 10,0 (≈360 transactions).
* `TransferProbOrder15.gmdl` — `Prob` = 15%, `Generate` 10,0 (≈360 transactions).

The two advances hold the transactions for very different times, so the report
of each run says plainly which branch took them: the entry count of
`A_AdvanceLargo` (the target of `Salida1`) against the entry count of
`A_AdvanceCorto` (the target of `Salida2`).

`Prob` is written in the model as `85%` / `15%` (the original's own format);
the original writes it in its GPSS as `850` / `150` (thousandths).

## The numbers

`A_AdvanceLargo` is the target of `Salida1`; `A_AdvanceCorto` is the target of
`Salida2`.

| model | `Prob` | through the `TRANSFER` | original: largo / corto | ours: largo / corto |
|---|---|---|---|---|
| `TransferProbOrder` | 85% | 60 | **47 / 13** | 51 / 8 (59 through) |
| `TransferProbOrder85` | 85% | 360 | **300 / 60** | 309 / 50 (359 through) |
| `TransferProbOrder15` | 15% | 360 | **53 / 307** | 52 / 307 (359 through) |

The 15% run is the decisive one: the original sends **53 of 360** (14.7%) to
the branch of `Salida1`, so the probability went to `Salida1` — the *last*
destination it writes, not the first. If the original's engine followed the
order it writes, that branch would have taken ~306 and the other ~53.

The 85% runs agree with the same reading: 300 of 360 (83%) and 47 of 60 (78%)
go to `Salida1`'s branch, against ~60 and ~9 for the other reading.

Our engine's numbers sit on the same side of the same split (309/50, 51/8,
52/307).

## The text, for the record

The original's GPSS for the probe (from `oracle/*.gpss.txt`):

```
Entrada1        TRANSFER        850,A_AdvanceCorto,A_AdvanceLargo
```

The first destination written is the target of `Salida2` — the reversed port
order — while the engine gives the probability to `Salida1`. The text and the
engine of the original disagree with each other; the engine agrees with the
model's own property hint.

Our `genmsi gpss` prints the destinations in the port order
(`850,A_AdvanceLargo,A_AdvanceCorto`), so the renderer has to write them
reversed to reproduce the original's text. Our *report* renderer already writes
them reversed (`850,A_AdvanceCorto,A_AdvanceLargo`), which is another way of
saying the same thing: the two renderers do not agree with each other today.

## How to run it

```
bash scripts/wine-oracle/probes/run-probe.sh
```

It runs the original over the three models (the same harness
`run-benches.sh` uses, killing the Wine session between models), runs our
engine over the same models, and prints the two entry counts side by side.

By hand:

```
WINEPREFIX="$PWD/scripts/wine-oracle/wineprefix" \
  bash scripts/wine-oracle/run_oracle.sh \
    scripts/wine-oracle/probes/TransferProbOrder85.gmdl /tmp/genmsi-probe-out
wineserver -k

cargo run -p genmsi-cli -- simulate scripts/wine-oracle/probes/TransferProbOrder85.gmdl
cargo run -p genmsi-cli -- gpss     scripts/wine-oracle/probes/TransferProbOrder85.gmdl
```

## Files

* `TransferProbOrder*.gmdl` — the three models.
* `oracle/` — what the original produced (report and GPSS of each model).
* `ours/` — what our engine produced for the same models.
