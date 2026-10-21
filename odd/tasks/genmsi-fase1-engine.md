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

- T1 (S1) `genmsi-format` semantic model + DOM parser for `.gmdl` (blocks, ports, links, containers, properties, layout kept apart). Route: inline. Commit: feat(format).
- T2 (S1) `.gmdl` writer with byte-identical round trip and structural validation with `path:line` context. Route: inline. Commit: feat(format).
- T3 (S2, D4) pest expression grammar + AST + evaluator for SNAs (P:, FN:, F:, Q:, S:, R:, CH:, V:, X:, BV:, N:, W:, C1/AC1, RNj, arithmetic). Route: inline. Commit: feat(core).
- T4 (S2, D5) Seeded RNG streams, scheduler (CEC/FEC), transaction state machine, entity statistics. Route: inline. Commit: feat(core).
- T5 (S2) Top-19 blocks. Route: inline. Commit: feat(core).
- T6 (S2) Medium-frequency blocks and short tail required by the 20 valid models. Route: inline. Commit: feat(core).
- T7 (S3) Load/run validation replicating the three original error classes (invalid block, TERMINATE without A, null B operand) with line/node context. Route: inline. Commit: feat(format/core).
- T8 (S4) GPSS-style report, structured event trace, JSON and CSV export. Route: inline. Commit: feat(core).
- T9 (S5) Replications with derived seeds and 95% confidence intervals. Route: inline. Commit: feat(core).
- T10 (S6) Oracle comparison harness for the 20 valid models + `docs/oracle-comparison.md`. Route: inline. Commit: test(oracle) / docs.
- T11 (S6, S8) insta snapshots, proptest invariants, final `cargo test` verification. Route: inline. Commit: test.

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
