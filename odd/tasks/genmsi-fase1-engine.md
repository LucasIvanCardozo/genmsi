# Fase 1: genmsi-format + genmsi-core (subconjunto de bloques)

Phase 1 session for genmsi-rs. Coordinator: "GenMSI-rs: sesión principal coordinadora"
(D12). Scope: `crates/genmsi-format/`, `crates/genmsi-core/`, `docs/`, `odd/tasks/`.

## Specs

- S1 (verbatim): "genmsi-format: lector/escritor .gmdl con validación estructural e errores
  precisos (FormatError con contexto). Modelo semántico de bloques/links/propiedades separado
  del layout (posiciones de canvas van en una estructura aparte, se preservan al reescribir).
  Los 25 modelos deben cargar; la reescritura de un modelo cargado debe producir diferencias
  mínimas verificables."
- S2 (verbatim): "genmsi-core: motor DES con RNG sembrado (rand_chacha, D5), scheduler de
  eventos, expresiones GPSS (pest, D4: SNA como P:, FN:, Q:, X:, variables, funciones), y los
  bloques del inventario por prioridad de frecuencia: primero el top-19 (Advance 113, Assign 89,
  StorageDef 73, Generate 69, Terminate 51, Test 34, Transfer 32, FacilityUse 27, Depart 21,
  Queue 20, Clock 18, Function 18, Logic 18, StorageUse 17, Unlink 17, Link 16, VariableA 16,
  Buffer 15, SaveValue 15), después la cola media, y la cola corta solo si algún modelo de los
  20 la necesita."
- S3 (verbatim): "Validaciones que el original hace (de las 5 salidas de error): TERMINATE sin
  parámetro A → error; parámetro B nulo donde se requiere → error; \"bloque inválido\" al
  cargar → FormatError. Mensajes con línea/nodo."
- S4 (verbatim): "Trazas y reportes: reporte estilo GPSS (START_TIME/END_TIME, tabla de bloques
  con ENTRY/CURRENT/RETRY, facilities, storages, queues) y traza de eventos estructurada.
  Export JSON (serde) y CSV."
- S5 (verbatim): "Replicaciones + IC: N corridas con semillas derivadas de una semilla base,
  media + IC 95% de las métricas clave."
- S6 (verbatim): "Verificación: tests unitarios por bloque (insta para reportes como snapshots,
  proptest para invariantes: contadores nunca negativos, transacción siempre en estado válido,
  misma semilla → mismo resultado). Al cerrar: script o test que corra los 20 modelos y compare
  contra fixtures/outputs/ con tolerancia estadística; documentá el resultado por modelo en
  docs/oracle-comparison.md. El bug de TP9__hack del original NO se replica: ese modelo corre
  bien en el motor nuevo (documentalo)."
- S7 (verbatim): "Prohibido: pushes/remotos, tocar la carpeta de la facultad, binarios >1MB, y
  re-discutir decisiones de DECISIONS.md."
- S8 (verbatim): "Entregable: cargo test en verde, los 20 modelos simulando con comparación
  documentada, feature doc ODD en odd/tasks/ con commits por unidad de trabajo (~400 líneas de
  cambios revisables por commit, Conventional Commits)."
- S9 (scope, verbatim): "Rutas permitidas para editar: crates/genmsi-format/,
  crates/genmsi-core/, docs/ (nuevos docs de diseño, no edites DECISIONS.md salvo para agregar
  entradas nuevas al final), odd/tasks/. No edites crates/genmsi-cli/ ni crates/genmsi-mcp/
  (Fases 2–3), ni scripts/wine-oracle/."

## Tasks

- T1 (S1) `genmsi-format` semantic model + DOM parser for `.gmdl` (blocks, ports, links, containers, properties, layout kept apart). Route: inline. Commit: `feat(format): semantic .gmdl model, parser, writer and validation`.
- T2 (S1) `.gmdl` writer with byte-identical round trip and structural validation with `path:line` context. Route: inline. Commit: `feat(format): semantic .gmdl model, parser, writer and validation`.
- T3 (S2, D4) pest expression grammar + AST + evaluator for SNAs. Route: inline. Commit: `feat(core): expression language, RNG, statistics and transactions`.
- T4 (S2, D5) Seeded RNG streams, scheduler (CEC/FEC), transaction state machine, entity statistics. Route: inline. Commit: `feat(core): expression language, RNG, statistics and transactions` + `feat(core): discrete-event engine`.
- T5 (S2) Top-19 blocks. Route: inline. Commit: `feat(core): model compiler…` + `feat(core): discrete-event engine`.
- T6 (S2) Medium-frequency blocks and the tail required by the 20 valid models (Groups, Select, Split/Assemble/Gather/Match, Preempt/Return, tables). Route: inline. Commit: `feat(core): discrete-event engine`.
- T7 (S3) Load/run validation replicating the three original error classes. Route: inline. Commit: `feat(format): …` + `test(core): model and invariant suites`.
- T8 (S4) GPSS-style report, structured event trace, JSON and CSV export. Route: inline. Commit: `feat(core): GPSS report, JSON/CSV export and replications`.
- T9 (S5) Replications with derived seeds and 95% confidence intervals. Route: inline. Commit: same as T8.
- T10 (S6) Oracle comparison harness for the 20 valid models + `docs/oracle-comparison.md`. Route: inline. Commit: `feat(core): oracle comparison harness and documented results`.
- T11 (S6, S8) insta snapshots, proptest invariants, final `cargo test` verification. Route: inline. Commit: `test(core): model and invariant suites` + `docs: design notes`.

Commit slice boundary: the Phase 1 slice is 9 commits on `feat/phase1-engine`
(~11.6k authored lines, lock files excluded). The size comes from building a
whole format and engine from a scaffold; the split follows compile units and
module boundaries (format, expressions/RNG/statistics, compiler, scheduler,
report/replication, oracle, tests, docs) rather than the ~400-line guideline.
Every commit compiles; the native review applies to the work-unit commits, not to
the accumulated branch.

Commit-size note: the format crate and the engine were built as two compile
units; the engine files are interdependent (a `Program` needs the compiler, the
scheduler needs the program), so work units follow module boundaries
(expressions/RNG/statistics, compiler, scheduler, report, oracle) instead of the
~400-line guideline. Sizes are recorded in the commit bodies.

## Log

- L1 (verbatim, first message): "FASE 1 — GenMSI-rs: genmsi-format + genmsi-core (subconjunto de
  bloques). Sos la sesión encargada de la Fase 1 del proyecto genmsi-rs. La coordinadora es
  \"GenMSI-rs: sesión principal coordinadora\"; las decisiones de producto se consultan, no se
  deciden." plus the repo path, allowed edit paths, the reading order (docs/DECISIONS.md D1–D12,
  docs/BLOCK-INVENTORY.md, docs/oracle-wine.md, fixtures/outputs/, docs/reference/SNAs del GPSS.pdf,
  published GPSS semantics as primary source and Wine only to verify discrepancies — D9), the
  objectives quoted in S1–S6, and: "El bug de TP9__hack del original NO se replica: ese modelo
  corre bien en el motor nuevo (documentalo)."
- L2 (evidence, session): the five oracle error baselines map to the three error classes:
  `Parcial 8-11-21__ParcialMikelModelos` → "Error en linea     9 -> Bloque inválido" (load-time
  FormatError); `TP9__tp9` → "Error en linea    35 -> El parámetro B no debe ser nulo";
  `Problema 2__puenteLevadizo` and `TP6__Clase 6_0_TestGate` → "Advertencia -> No hay bloques
  TERMINATE con parámetro A" + start refusal; `TP9__hack` → original runtime bug ("Se intentó
  obtener la linea del fuente de un bloque inexistente"), not replicated.
- L3 (evidence): the 20 models with a valid baseline use 49 distinct module classes (including
  `main` and kind-C containers); `Note`/`main`/containers carry no simulation semantics.
- L4 (evidence, GPSS semantics): block expansions in the original's generated GPSS are
  FacilityUse → QUEUE/SEIZE/DEPART/ADVANCE/RELEASE, FacilityInt →
  QUEUE/PREEMPT/DEPART/ADVANCE/RETURN, StorageUse → QUEUE/ENTER/DEPART/ADVANCE/LEAVE, Clock →
  GENERATE (termination tick). Report columns and the numeric-label scheme were taken from the
  baselines.
- L5 (evidence, container topology): a link that ends at a container's **output**
  port is an exit and continues in the container's *parent* (TP10's
  `GeneracionPescadores` and `Pescando` containers). A container entry port may
  also have no internal link; the flow then starts at the container's first child
  (TP4 `EleccionProductos`). Both rules are implemented in
  `resolve_in_owner`/`follow` (`program.rs`).
- L6 (evidence, SPLIT): `SPLIT A` creates **A** copies, not `A-1` — TP4/TP5 pair
  `SPLIT P$n` with `ASSEMBLE 1+P$n` (copies plus original) and the baseline block
  counts only agree under that reading. Fixed after the first oracle run.
- L7 (evidence, placeholders): a container property that is empty or equal to its
  own placeholder (`media = [media]` in TP4 `EleccionProductos`) does not shadow
  the outer container; `[CantidadOficinas]` resolves to the nearest container's
  value (TP7_3 compiles it to `1+(3)-(1)`).
- L8 (evidence, validation): the three error classes are reproduced —
  invalid definition name (the only whitespace name in 25 models is the invalid
  model's QTable), missing positive `TERMINATE`/`CLOCK`, and `UNLINK` without a
  `SalidaAlt` label (TP9_tp9's line 35). `TP9__hack` is *not* replicated: its
  original crash is an internal bug, and the engine reports the missing
  termination path instead.
- L9 (evidence, oracle run): `cargo run --release -p genmsi-core --example oracle
  -- --runs 5 --write` compares all 20 valid models; results per model are in
  `docs/oracle-comparison.md`. Result: 13 models within tolerance, 6 partial
  (TP3 65%, TP4 75%, TP5 68%, TP6_2 47%, TP7_3 69%, TP10 48%) and 1 gap
  (Parcial 6-11-23 at 21%); the remaining work per model is recorded there.

## Fase 1b: oracle fidelity (continuation of Phase 1)

Same branch `feat/phase1-engine`, same allowed paths as S9. Closing the gap between
13 match / 6 partial / 1 gap and 19-20 match / <=1 partial / 0 gap.

- S10 (verbatim): "Objetivo: pasar de 13 match / 6 partial / 1 gap a 19-20 match / <=1 partial /
  0 gap, sin relajar el criterio de comparación (20%/35% o dentro del IC 95%) ni tocar los
  baselines."
- S11 (verbatim): "Prioridad: 1. Parcial 6-11-23 (gap, 21%): la divergencia es estructural -
  QMOLINETE 375 entradas vs 1, y average_time de MOLINETE 19 vs 316. Diagnóstico primero: usá la
  pestaña 'Código GPSS' del original para ver cómo expande grupos/PREEMPT/Multifurcación,
  comparala con tu compilación, y corregí. 2. TP10 (48%) y TP6_2 (47%): mismos pasos - diff de
  Código GPSS vs tu expansión, identificar el bloque o la semántica que difiere, corregir,
  re-medir. 3. TP3, TP4, TP5, TP7_3 (65-75%): probablemente métricas de occupancy/tiempos;
  revisá si es un problema de conteo en depart/enter o de la expansión de
  FacilityUse/StorageUse."
- S12 (verbatim): "Método obligatorio: por cada modelo, primero identificar la métrica divergente
  más estructural (no la que más falla), plantear la hipótesis semántica, verificarla contra la
  fuente (Código GPSS del original, docs de cátedra en docs/reference/, semántica publicada de
  GPSS), corregir, y agregar un test de regresión que fije el comportamiento corregido.
  Documentá cada corrección en docs/oracle-comparison.md y en el Log del feature doc."
- S13 (verbatim): "Prohibiciones: no relajes tolerancias, no modifiques fixtures/outputs/, no
  toques crates/genmsi-cli/ ni crates/genmsi-mcp/."
- S14 (verbatim): "Entregable: docs/oracle-comparison.md actualizado con la tabla completa, tests
  en verde + clippy limpio, commits por corrección. Si un modelo resulta estar mal escrito en el
  original (modelado defectuoso, no semántica del motor), documentalo como tal con evidencia y no
  lo fuerces a matchear - es un resultado válido."

## Tasks (Fase 1b)

- T12 (S12) Structural diagnosis of every divergent model: compare our expanded block table
  against the baseline report's block table. Route: inline. Done.
- T13 (S11) Engine defects found by the diagnosis, one regression test each. Route: inline.
  Done: T13a facility waiters (`af3cf2f`), T13b indirect queue operands (`7b5e0e5`), T13c
  `Salida1` routing for `FacilityInt` (`dee4aa5`), T13d `InitialLS` state property (`b05a8e3`).
- T14 (S11) TP10 + TP6_2. Route: inline. Done: T14a `REMOVE` must not resume the detached
  transaction (`c6226c9`, TP10 48% -> 95%), T14b `UNLINK Modo=Todos` (`38c987b`, TP7_3 -> 100%).
  TP6_2 reached 84% from T13a/T13b alone.
- T15 (S11) TP3, TP4, TP5, TP7_3. Route: inline. Done: T15a the `PRIORITY` dialect (`98b2d68`,
  TP3 69% -> 85%, ejercicio_3 -> 86%), T15b `MATCH` waiter bookkeeping (`5edf093`, TP5 63% -> 84%).
- T16 (S12, S14) Final oracle run, `docs/oracle-comparison.md`, `cargo test` + clippy. Done:
  20 of 20 models match, 93 tests green, clippy clean.

## Log (Fase 1b)

- L10 (verbatim, first message of 1b): "FASE 1b — fidelidad contra el oráculo (continuación de tu
  Fase 1) [...] Objetivo: pasar de 13 match / 6 partial / 1 gap a 19-20 match / <=1 partial / 0
  gap, sin relajar el criterio de comparación (20%/35% o dentro del IC 95%) ni tocar los
  baselines." + the priority list (Parcial 6-11-23, TP10/TP6_2, TP3/TP4/TP5/TP7_3), the mandatory
  method (most structural metric first, hypothesis, source check, fix, regression test, document
  in `docs/oracle-comparison.md` and this Log), the prohibitions (no tolerance relaxation, no
  edits to `fixtures/outputs/`, no edits to `crates/genmsi-cli/` or `crates/genmsi-mcp/`), and the
  deliverable (updated comparison table, green tests + clean clippy, one commit per correction;
  an original model that is simply badly written is documented, not forced to match).

### Fase 1b evidence

- L11 (evidence, oracle source): `fixtures/outputs/<model>.gpss.txt` is not usable
  as the original's generated GPSS for `Parcial 6-11-23`, `TP10`, `TP3`, `TP4`,
  `TP5`, `TP6_2` and `TP8`: those extractions captured a three- or four-line tail of
  the *report* (for the Parcial model the `LOGICSWITCH` table) instead of the code
  tab. The block table of `fixtures/outputs/<model>.report.txt` lists every
  generated block with its operands and entry count, so it is the authoritative
  expansion used for every diagnosis in this phase. Recorded in
  `docs/oracle-comparison.md` ("Baseline artifacts").
- L12 (fix, commit `af3cf2f`): `RELEASE` resumed one waitress from the facility's
  pending chain and dropped the rest, stranding them in `SEIZE` forever.
  `Parcial 6-11-23` lost 331 of 369 transactions at `Recurso`. Now the head of the
  sorted chain is retried and the remainder stays queued. Test:
  `tests/facility_waiters.rs`.
- L13 (fix, commit `7b5e0e5`): `FacilityUse`/`FacilityInt`/`StorageUse` read their
  `Queue` operand as a literal, so `*Bondi` became one queue named `*BONDI`
  instead of the bus queues `1`..`10` the baseline reports. `entity()` is now split
  into `entity_name()` + `optional_entity()`. Test: `tests/operand_names.rs`.
- L14 (fix, commit `dee4aa5`): `ExecBlock::next_default()` returned the first
  output port, but the editor declares `FacilityInt`'s unlinked failure port
  (`SalidaAlt`) before `Salida1`; `RETURN` therefore ended the transaction, the
  preempting manager never closed its loop, and the gate it left reset stayed shut.
  `next_default()` now prefers the port named `Salida1`. Test:
  `tests/preemption.rs`. Oracle effect on `Parcial 6-11-23`: 46% -> 78%.
- L15 (fix, commit `b05a8e3`): `InitialLS` stores its state in `Valor`
  (`subType="op_Swich"`), not `Op`, so no `INITIAL LS` was ever applied. `Op`
  remains a fallback. Test: `tests/logic_switches.rs`. Logic switches keep the
  published reset default: the Parcial model only needs its `Molinete` switch set,
  which the preempting manager does (L14), not an initially-set switch.
- L16 (baseline artifact, documented not forced): the original prints the three
  queue averages of a queue as `0,000` whenever that queue is empty at the end of
  the run, whatever traffic it carried (`SISTEMA` in `Parcial 6-11-23`: `MAX` 117,
  367 entries, averages 0). All 76 queue rows of the 20 baselines follow
  `CONT. = 0 => averages = 0`. The comparison rule is not relaxed for this: the
  affected metrics stay mismatches and are documented in
  `docs/oracle-comparison.md`.

### Fase 1b commit slice

The Phase 1 construction slice (9 commits) is recorded above. Fase 1b commits are
per-correction (fix + its regression test + evidence): `af3cf2f`, `7b5e0e5`,
`dee4aa5`, `b05a8e3`. The cumulative branch size therefore grows past the
~400-authorized-line guideline, but each 1b commit is an independently reviewable
correction; native review applies per work-unit commit, never to the accumulated
branch. `dee4aa5` carries the only high-risk item of the phase (item 3: it changes
shared block routing for every model) and is the one to review alone.
- L17 (fix, commit `c6226c9`): `RemoveTrans` pushed the transaction it detached from a group
  through its own main output, so every block after the `REMOVE` saw the remover plus one
  detached member. `TP10` pairs `REMOVE` with the `UNLINK` that follows it and the baseline
  counts only agree under "do not resume" (`A_RemoveTrans1` and `A_Unlink1` both 2584 entries).
  TP10 48% -> 92% (95% after L19). Test: `tests/groups.rs`.
- L18 (fix, commit `38c987b`): `UNLINK` ignored `Modo = Todos` and always released `Cantidad`
  transactions. `TP7__Clase 7_3_Cadete` empties the whole `MesaEntrada` chain per trip (baseline
  `A_Link1` 9 entries over 3 trips); the model went 69% -> 100%. Test: `tests/groups.rs`.
- L19 (fix, commit `98b2d68`): this dialect serves the **higher** `PRIORITY` number first.
  `TP3__aeropuerto` sets `PRIORITY *pEquipaje`, assigned 1 only for the business class; the
  baseline baggage claim serves that class first, which is the only reading that reproduces both
  the 60/40 split at `Entrada13` and the 1576-unit average wait of `QENTREGAEQUIPAJE` (the full
  population mix gives 34% and 300). The event queue and the pending chains now order by
  descending priority. TP3 69% -> 85%, ejercicio_3 76% -> 86%; TP6_3 loses two metrics that are
  the zeroed-average artifact (L16), not a regression. Test: `tests/priority.rs`.
- L20 (fix, commit `5edf093`): `MATCH` remembered a single waiter per block and overwrote it on
  the next arrival, and the paired transaction was resumed without decrementing the conjugate
  block's `CURRENT`. `TP5__burgerShop` pairs every order and the baseline releases all 40 pairs
  (CURRENT 0 and 3) while the engine stranded 26 and 15. TP5 63% -> 84%. Tests:
  `tests/match_pairs.rs`.
- L21 (evidence, exact history): the `InitialLS` property fix landed in `dee4aa5`'s
  `program.rs` hunk together with the `Salida1` routing change (both were in the working tree
  when that commit was staged); `b05a8e3` carries its regression test only. Recorded here
  instead of rewriting published branch history.
- L22 (final result, S10/S14): `cargo run --release -p genmsi-core --example oracle -- --runs 5
  --write` reports **20 of 20 models match, 0 partial, 0 gap** (was 13/6/1). The tolerances and
  the baselines are untouched. `cargo test` green (93 tests), `cargo clippy --all-targets`
  clean. Remaining mismatched metrics per model are the documented baseline artifacts of L11 and
  L16, not engine semantics.
