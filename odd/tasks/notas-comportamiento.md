# Las seis diferencias de comportamiento del lote 1

Lo que esta sesión cambió en el core, con la evidencia del original delante.
Todo lo que sigue sale de la respuesta del `genmsi.exe` original bajo Wine
(`scripts/wine-oracle/`), no de una lectura del código.

## Specs

- **S1** — "La puerta sobre un almacén": el original tiene seis operadores
  (`SF:Lleno`, `SNF:NoLleno`, `SE:Vacio`, `SNE:NoVacio`, `SV:Disponible`,
  `SNV:NoDisponible`, en `Genmsi_20230711.backup/gmacroconfig.gmcfg`) y el
  motor tenía un booleano `empty`. La puerta abre cuando el almacén **no está
  lleno**, no cuando está vacío.
- **S2** — "El indicador del `LINK`": la primera transacción que llega a la
  cadena (la que encuentra el indicador apagado) va al destino de la
  sentencia y no entra en la cadena; las demás esperan en ella. El
  `AVE.TIME` de la cadena es el integral de su tamaño sobre sus entradas.
- **S3** — "El `CURRENT` del `ASSEMBLE`": el bloque devuelve las
  transacciones cuando el conjunto se completa.
- **S4** — "El `MATCH` se expande en un bloque": el original escribe dos
  sentencias (`TRANSFER` + `MATCH`), el reporte lista las dos y las entradas
  están en el `TRANSFER`.
- **S5** — "Las filas `SEIZE` y `FACILITY`": cuentan lo que entró, su `RETRY`
  queda en 0 y el que espera se cuenta en el bloque del que vino; la
  facility los guarda en su columna `DELAY`.
- **S6** — "La última transacción de la corrida": el `ENTRY COUNT` de un
  generador cuenta la transacción que ya va en camino, y la corrida cuenta el
  arribo de la hora en la que termina.

## Tasks

| ID | Specs | Qué | Ruta | Commit |
|----|-------|-----|------|--------|
| T1 | S1 | Los seis operadores de la puerta sobre un almacén, medidos con sondas | sondas + core | este commit |
| T2 | S2 | El indicador del `LINK` y el tiempo medio de la cadena | core | este commit |
| T3 | S3 | El `CURRENT` del `ASSEMBLE` | core | este commit |
| T4 | S4 | El `MATCH` como dos sentencias | core | este commit |
| T5 | S5 | Las filas `SEIZE` y `FACILITY` | core | este commit |
| T6 | S6 | La última transacción de la corrida | core | este commit |
| T7 | S1..S6 | Los tests, el oráculo, los 26 bancos y el diferencial | core + sondas | este commit |
| T8 | S1..S6 | Estas notas | `odd/tasks` | este commit |

## Lo que se midió, y cómo

### La puerta sobre un almacén (S1)

La tabla de verdad del original, con tres sondas nuevas
(`scripts/wine-oracle/probes/gates/`): un modelo con un `StorageDef`, un
`Generate`, seis puertas (una por operador) y seis `Terminate`, de manera que
cada operador tiene su propio flujo. La respuesta del original está en
`scripts/wine-oracle/probes/gates/oracle/`.

| Almacén | `SF` | `SNF` | `SE` | `SNE` | `SV` | `SNV` |
|---|---|---|---|---|---|---|
| vacío (0 de 10) | cerrada | **abierta** | **abierta** | cerrada | **abierta** | cerrada |
| una unidad de diez | cerrada | **abierta** | cerrada | **abierta** | **abierta** | cerrada |
| lleno (1 de 1) | **abierta** | cerrada | cerrada | **abierta** | **abierta** | cerrada |

La regla del original es, entonces:

- `SF` (Lleno): el almacén está lleno.
- `SNF` (NoLleno): el almacén **no** está lleno. ← el que usa la clase por
  defecto (`NoLleno`), y el que la sonda 2 de las notas del lote 1 ya
  señalaba.
- `SE` (Vacio): el almacén está vacío.
- `SNE` (NoVacio): el almacén no está vacío.
- `SV` (Disponible) y `SNV` (NoDisponible): el almacén está disponible o no.
  Con un almacén declarado, `SV` abre siempre y `SNV` nunca; con un almacén
  **no** declarado, el original se para con `Error en linea 2 -> Operador
  lógico inválido` (`oracle/gateS_sinAlmacen_Lleno.report.txt`), así que
  Our motor los trata como "el almacén existe" (`SV`) y "no existe" (`SNV`),
  y sigue como estaba con el `SNF`, el `SE` y el `SNV` de un almacén que
  nadie declara.

Las letras del listado son las del original: `SF`, `SNF`, `SE`, `SNE`, `SV`,
`SNV` (las escriben el original y el nuestro, en
`oracle/gateS_vacio.gpss.txt` y en el nuestro).

**Qué cambió.** `crates/genmsi-core/src/program.rs`: el enum `StorageOp` con
los seis operadores y `GateCondition::Storage { storage, op }` (el booleano
`empty` disappears); la rama `"GateS"` del compilador lee el operador de la
propiedad `op`. `crates/genmsi-core/src/gpss.rs` y
`crates/genmsi-core/src/report.rs`: la letra del operador. En
`crates/genmsi-core/src/sim.rs`, fn `gate_condition`: la prueba.

**La prueba.** `crates/genmsi-core/tests/gates.rs`:
`the_gate_on_a_storage_asks_the_state_the_original_asks` (las seis columnas de
la tabla, con el estado del almacén medido) y
`the_gate_of_the_class_opens_when_the_storage_has_room`.

### El indicador del `LINK` (S2)

El reporte del original del banco `Link`
(`scripts/wine-oracle/benches/Link.gmdl`):

```
    1       57       1     0              1  GENERATE        60,30
    2       56      55     0         A_Link  LINK            Cadena,FIFO,A_Terminate
    3        1       0     0    A_Terminate  TERMINATE
USER_CHAIN       CHAIN_SIZE   RETRY   AVE.CONT   ENTRIES    MAX     AVE.TIME
  CADENA                   55       0     26,466        55     55     1732,291
```

De aquí salen dos reglas: la primera transacción que llega a la cadena (la que
encuentra el indicador apagado) **va al destino de la sentencia** y no entra
en la cadena —la cadena tiene 55 miembros y no 56—, y el `AVE.TIME` de la
cadena es el integral de su tamaño sobre sus entradas (26.466 × 3600 / 55 =
1732,3), no la suma de las esperas de las que ya se fueron (0).

**Qué cambió.** `crates/genmsi-core/src/sim.rs`, rama `BlockKind::Link`: la
cadena tiene un indicador (`crates/genmsi-core/src/stat.rs`, struct `Chain`,
campo `indicator`, que no va al reporte); el que lo encuentra apagado va al
destino. `crates/genmsi-core/src/stat.rs`, fn `Chain::average_time`: el
integral sobre las entradas.

**La prueba.** `crates/genmsi-core/tests/comportamiento.rs`:
`the_link_sends_the_first_transaction_to_the_destination`.

### El `CURRENT` del `ASSEMBLE` (S3)

El reporte del original del banco `Assemble`:

```
    2       58       0     0     A_Assemble  ASSEMBLE        1
```

El `CURRENT` es 0: el bloque devuelve las transacciones cuando el conjunto se
completa. El nuestro las devolvía con `enter_block`, que no baja el contador.

**Qué cambió.** `crates/genmsi-core/src/sim.rs`, rama `BlockKind::Assemble`:
`leave` en vez de `enter_block` (y el contador del bloque baja también cuando
una copia se destruye).

**La prueba.** `crates/genmsi-core/tests/comportamiento.rs`:
`the_assemble_gives_the_transactions_back_when_the_set_is_complete`.

### El `MATCH` en dos sentencias (S4)

El reporte del original del banco `Match`:

```
    2       66      66     0        A_Match  TRANSFER        ,AAA
    3        0      66     0            AAA  MATCH           AAA
```

El original escribe dos sentencias (`TRANSFER ,AAA` y `MATCH AAA`), el reporte
lista las dos filas, y las entradas están en el `TRANSFER`; las que esperan
están en las dos.

Lo mismo se ve en un modelo de verdad, el `TP5__burgerShop`
(`fixtures/outputs/TP5__burgerShop.report.txt`):

```
   51       43       3     0        A_Match  TRANSFER        ,PIENSA
   52       40       3     0            PIENSA  MATCH           ESPERA
```

43 transacciones entraron al `TRANSFER`, 40 completaron el encuentro (las
entradas del `MATCH`) y 3 esperan (el `CURRENT` de las dos filas).

**Qué cambió.** `crates/genmsi-core/src/program.rs`, rama `"Match"`: la clase
se expande en dos sentencias (un `TRANSFER` incondicional y el `MATCH`), como
el original. `crates/genmsi-core/src/gpss.rs`: la sentencia `MATCH` se
escribe sola (la del `TRANSFER` la escribe el bloque del transfer) y la
sentencia del `MATCH` se llama con la etiqueta que la clase define. En
`crates/genmsi-core/src/report.rs`: el `BLOCK_TYPE` del transfer y los
operandos de un transfer con un solo destino.

**La prueba.** `crates/genmsi-core/tests/comportamiento.rs`:
`the_match_is_two_statements_and_the_entries_are_in_the_transfer`.

### Las filas `SEIZE` y `FACILITY` (S5)

El reporte del original de la sonda `facilidadContencion`
(`scripts/wine-oracle/probes/almacen/`):

```
    2      120     114     0  A_FacilityUse  QUEUE           Cola
    3        6       0     0              3  SEIZE           Recurso
    4        6       0     0              4  DEPART          Cola
FACILITY       ENTRIES  UTIL.  AVE_TIME  AVAILABLE  OWNER PEND INTER RETRY DELAY
  RECURSO           6   0,992  595,000       1         7    0     0     0   114
```

La fila `SEIZE` cuenta las **seizures** (6), no los intentos (120), su
`RETRY` es 0 y el `CURRENT` es 0: las 114 que esperan están en la fila del
bloque del que vinieron (la `QUEUE`). En la facility, los que esperan están en
la columna `DELAY` (114) y el `PEND` es 0.

**Qué cambió.** `crates/genmsi-core/src/sim.rs`, rama `BlockKind::Seize` (y
el conteo genérico de `execute`): la fila cuenta lo que entró, sin reintentos,
y el que espera se cuenta en el bloque del que vino.
`crates/genmsi-core/src/report.rs`: la fila de la facility pone el `PEND` en 0
y los que esperan en el `DELAY`.

**La prueba.** `crates/genmsi-core/tests/comportamiento.rs`:
`the_seize_row_counts_the_seizures_and_the_waiters_wait_where_they_came_from`.

### La última transacción de la corrida (S6)

El reporte del original de la sonda `almacenCola2`:

```
    2      121       1     0              1  GENERATE        30,0
    3      120     108     0     StorageUse  QUEUE           qAlmacen,2
QUEUE            MAX  CONT. ENTRIES ENTRIES(0) AVE.CONT. AVE.TIME AVE.(-0) RETRY
  QALMACEN       216   216     120         2   105,300   3159,000 3212,542    0
```

Dos cosas: el `ENTRY COUNT` del generador (121) cuenta la transacción que ya
va en camino (120 hechas + 1 agendada), y la cola cuenta 120 entradas: el
arribo de la hora en la que la corrida termina (3600) está contado.

El orden con el que el original procesa los eventos de esa hora es el que
decide si ese arribo se cuenta o no. En la sonda `almacenCola2` el arribo
(agendado a las 3570) se cuenta; en el `TP6__Clase 6_1_PuertaBanco` la
transacción que pone la llave lógica (agendada al instalar el modelo, junto
con el reloj) **no** se cuenta: el original se para antes. La única regla que
explica las dos cosas es que el original procesa los eventos de la misma hora
en el orden inverso al de su agenda: el último agendado va primero. Con esa
regla, el reloj (agendado al final) se procesa antes que la transacción de la
llave en el `TP6__Clase_6_1`, y el arribo (agendado a las 3570) se procesa
antes que el reloj en el `almacenCola2`.

**Qué cambió.** `crates/genmsi-core/src/sim.rs`: el `ENTRY COUNT` de un
generador cuenta también el arribo agendado (y lo suelta cuando el generador
se agota); la hora en la que termina la corrida se calcula al instalar las
definiciones (la del reloj) y los eventos de esa hora se numeran al revés, para
que se procesen en el orden del original.

**La prueba.** `crates/genmsi-core/tests/comportamiento.rs`:
`the_run_counts_the_arrival_of_the_hour_in_which_it_ends`.

## Lo que se comprobó

- **El oráculo de métricas**: `cargo run --release -p genmsi-core --example
  oracle -- --runs 3` pasa de **887 de 956** a **899 de 958**. Los modelos que
  suben: `Problema_1__semaforo-1` (126 → 131), `TP3__aeropuerto` (85 → 87),
  `TP4__supermercado` (59 → 61), `TP5__burgerShop` (62 → 64) y
  `TP6__Clase_6_1_PuertaBanco` (9 → 10); ninguno baja. Con las cinco
  réplicas del documento (`docs/oracle-comparison.md`, regenerado) pasa de 823
  de 956 a 858 de 958: el total sube en dos porque el `MATCH` de `TP5` son
  ahora dos filas.
- **El GPSS**: los 26 bancos con respuesta del original (`/tmp/fase7/*/
  theirs.gpss.txt`) siguen dando el mismo texto, sentencia por sentencia
  (`scripts/wine-oracle/compare-gpss.sh`): 26 coinciden, 0 difieren.
- **El diferencial**: los 25 modelos de `fixtures/models`, con la misma
  semilla y el binario de antes contra el de ahora, cambian solo en las filas
  arregladas: el `ENTRY COUNT` de los generadores (+1), las filas `SEIZE` (el
  `RETRY` a 0 y el `CURRENT` a 0), las filas `QUEUE` de delante (los que
  esperan), las filas `FACILITY` (el `PEND` a 0 y el `DELAY` con los que
  esperan), las filas `ASSEMBLE`/`GATHER` (el `CURRENT` a 0), el `AVE.TIME`
  de las cadenas y las filas del final de la corrida (las del `MATCH`, que
  ahora son dos).
- **Los tests**: `cargo test --workspace` en verde, con 346 tests del core
  (los 344 de antes y los dos nuevos de la puerta) más los cinco de
  `crates/genmsi-core/tests/comportamiento.rs`. La app sigue con sus 64
  tests en verde y los tres que fallaban antes (los que necesitan el motor
  en WASM) siguen fallando igual.
- **Clippy**: `cargo clippy --workspace --all-targets -- -D warnings` sin
  avisos.

## Lo que queda fuera (y por qué)

- **La puerta que no abre no cuenta la entrada ni el reintento** (el cambio 14
  de las notas del lote 1): el original cuenta los intentos de la puerta en la
  llave (en el `RETRY` del almacén) y no en la fila de la puerta. Es otra
  clase y otra tarea.
- **La fila `PREEMPT`**: pierde los operandos (`PR`, destino, `Remanente`) y
  cuenta un reintento donde el original no cuenta ninguno. También es el
  cambio 5 de las notas del lote 1, de la forma del reporte.
- **El `SV`/`SNV` con un almacén que nadie declara**: el original se para
  (`Error en linea 2 -> Operador lógico inválido`) y el nuestro abre o cierra
  según haya o no almacén. No hay forma de medirlo: el original no corre el
  modelo.
- **El multiplicador de la `GateS`**: en la sonda `gateS_uno` (el almacén con
  una unidad de diez) el generador de la puerta crea más transacciones de
  las que le tocan (121 en vez de 60). La regla no está fijada por la
  evidencia y no se ha tocado.
- **La `LOC` de las filas** (el cambio 4 de las notas del lote 1): la puerta
  se sigue nombrando con la etiqueta del puerto (`A_GateS`) y no con el
  nombre de la sentencia (`GateS`).

## El reparto en commits

La entrega va en tres commits, y el corte de ~400 líneas se anota aquí
(`docs/format-and-engine.md` y el core en el primero, las sondas en el
segundo, el documento del oráculo en el tercero). El primero son 813 líneas
de código y de tests: las seis diferencias de comportamiento de la fase 7,
que se tocan unas a otras en `sim.rs` (la fila de la puerta, la del `SEIZE`,
la del `ASSEMBLE`, la del `LINK` y la del `MATCH` son el mismo bloque de
conteo) y que, por eso, no se pueden partir sin partir el motor en trozos
que no se pueden probar por separado.

## Log

- **L1** — El encargo de la sesión (verbatim):

  "Trabajo en /home/lucas/Documentos/Workspace/Genmsi (Rust: genmsi-core,
  genmsi-format, genmsi-cli, genmsi-app con Tauri+React).

  Reconstruimos GenMSI (un simulador GPSS de escritorio) en Rust. 344 tests
  Rust + 99 del front verdes, clippy limpio, y un oráculo que compara nuestra
  salida con la del original (que corre bajo Wine).

  MÉTODO (no lo cambies):
  - El original publica su propia especificación y la LEEMOS, no la adivinamos:
    * Genmsi_20230711.backup/gClasses.xml: la biblioteca de clases del original.
      Cada <moduleclass className="X"> trae su macrocode (el código fuente de la
      clase, URL-encoded) y su ClassDescrip (la descripción, con sus
      "Consideraciones"). Eso ES el código del original: no hace falta decompilar.
    * Los operadores de las clases están en gmacroconfig.gmcfg.
    * scripts/wine-oracle/ hace correr el original y saca su código GPSS y su reporte.
    * scripts/wine-oracle/make-benches.py genera un banco por clase y
      run-benches.sh lo corre contra el original.
    * scripts/wine-oracle/compare-gpss.sh compara nuestro GPSS con el del original.
  - Cuando una regla no esté fijada por evidencia, se dice; no se decide de oídas.
  - Al cerrar cada bloque: tests que lo fijen y un commit con el estilo del
    repositorio (Conventional Commits, en inglés).
  - No se agregan dependencias. Los contratos publicados genmsi.*/1 no se tocan sin
    necesidad explícita. No se modifica fixtures/ ni scripts/wine-oracle/out/.
  - El trabajo se registra en odd/tasks/ y en el todo."

  Y las seis diferencias que había que arreglar: la puerta sobre un almacén
  (notas, cambio 1), el "indicador link" (cambio 10), el current del assemble
  (cambio 9), el match que se expande en un bloque (cambio 11), las filas seize
  y facility (notas del almacén) y la última transacción de la corrida (notas
  del almacén).

- **L2** — La puerta: la verdad de los seis operadores no estaba en las notas
  del lote 1 (que sólo tenían dos sondas, del `SNF`). Se midió con tres
  sondas nuevas, una por estado del almacén, y con la letra de cada operador
  en el listado del original.

- **L3** — El `MATCH`: el original escribe dos sentencias y el reporte lista
  las dos. Se comprobó en el banco del lote y en un modelo de verdad
  (`TP5__burgerShop`), donde el `MATCH` tiene 40 entradas y 3 en espera: las
  dos filas los cuentan.

- **L4** — La última transacción: el orden de los eventos de la hora en la
  que la corrida termina es el que decide si el arribo de esa hora se cuenta.
  Se probaron tres reglas (procesar los eventos de esa hora, el orden inverso
  en toda la corrida, y el orden inverso sólo en esa hora) contra las dos
  sondas que lo fijan: la tercera es la que las explica a las dos.

- **L5** — El `ENTRY COUNT` de un generador cuenta el arribo agendado. Eso
  cambia el JSON publicado (`blocks[].entries`): los valores cambian, el
  contrato no.
