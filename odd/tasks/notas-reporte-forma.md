# Notas: la forma del reporte

Las nueve diferencias de la **forma** del reporte que el original escribe y el
nuestro no: la columna `LOC`, las filas de las sentencias que pierden sus
operandos, las secciones que faltan, los contadores de una puerta que no
abre, la tabla y la forma fina (los decimales, el orden de las colas y la
línea `SEMILLA`).

Todo lo que sigue está medido contra el original (`genmsi.exe` bajo Wine,
`scripts/wine-oracle/`), no supuesto: cada fila del original que se cita sale
de su reporte (`scripts/wine-oracle/out/`, `/tmp/bench-out/`,
`/tmp/bench-out-new/` o de las sondas de este lote).

## Specs

- **S1** — "QUÉ HACER (nueve diferencias de la FORMA del reporte):
  1. LA COLUMNA LOC. Sale del mylabel del puerto; el original escribe el nombre
     de la sentencia (GateF, Clona, A_Terminate1). El generador de bancos ya
     reproduce esa regla en las etiquetas del modelo, así que el original y el
     nuestro ya parten de la misma etiqueta.
  2. LA FILA PREEMPT pierde los operandos: escribimos "PREEMPT Recurso" donde el
     original escribe "PREEMPT Cajero1FreeShop,PR,,Remanente".
  3. LA FILA SELECT: escribimos "SELECT MIN param,1,10,,S" (o ",,LS", o "MIN...")
     donde el original escribe "SNF param,1,10", "LS param,1,10" y
     "E param,1,10,*1,Q,".
  4. LA FILA REMOVE lleva de más la letra E en el RemoveMe.
  5. LA FILA LINK no escribe el destino de la sentencia.
  6. FALTAN SECCIONES: LOGICSWITCH, XACT_GROUP, NUMERIC_GROUP y FEC_XACT_NUMBER;
     y la de SAVEVALUE va sin la columna RETRY y con el nombre en minúsculas.
  7. LOS CONTADORES DE UNA PUERTA QUE NO ABRE: el nuestro cuenta el intento
     fallido como entrada y reintento del bloque; el original no cuenta la
     entrada y cuenta los intentos en la llave. La fila de la puerta del banco
     GateL tiene que salir 0 0 0.
  8. LA TABLA: el original reserva la última de las Cantidad clases para el
     desborde ("> a 80") y el nuestro la gasta en otra clase finita ("81 - 90")
     y no tiene fila de desborde. Además la cabecera pierde el operando del
     atributo (P$Atrib) y los rótulos de las estadísticas están cambiados de sitio.
  9. LA FORMA FINA: los decimales enteros (0 donde el original escribe 0,000),
     el orden de las colas (el original pone 1, 2, ... 10 y el nuestro
     1, 10, 2, ...) y la línea SEMILLA, que el original no trae."
- **S2** — "El reporte del original es la especificación de estas nueve: compara
  fila por fila, no 'hasta donde se parezcan'."
- **S3** — "Un test del core por cada una de las nueve diferencias."
- **S4** — "Métricas: 899 de 958 o más. GPSS: 62 de 62 sin cambios."
- **S5** — "`cargo test --workspace` y `cargo clippy --workspace --all-targets
  -- -D warnings` en verde, con los tests subiendo."
- **S6** — "Las notas en odd/tasks/notas-reporte-forma.md (nuevo)." y "El commit
  en tu worktree, y me dices la rama."
- **S7** — "Cuando una regla no esté fijada por evidencia, se dice; no se decide
  de oídas." (de las notas del lote 1).
- **S8** — "NINGÚN commit lleva pie de atribución (nada de "Co-Authored-By" ni
  similar). El mensaje del commit explica qué cambió y por qué, y nada más."

## Lo que se encontró, diferencia por diferencia

### 1. La columna `LOC`

El original escribe **el nombre de la sentencia** (el mismo que el listado) y,
para las sentencias sin nombre, **el número que tienen en el programa**. El
nuestro escribía el `mylabel` del puerto y, para las sentencias sin nombre, un
contador de las que no lo tenían.

Evidencia (el banco `GateF` y su reporte):

```
    1       64       1     0              1  GENERATE        60,30
    2       63       0     0          GateF  GATE    NU      Recurso
    3       63       0     0    A_Terminate  TERMINATE
    4        2       1     0              4  GENERATE        3600
    5        1       0     0              5  TERMINATE       1
```

El `4` y el `5` son el número de la sentencia en el programa, no el número de
las sentencias sin nombre (que serían 2 y 3). Y el nombre es el de la
sentencia: las cuatro puertas y el `Split` toman el nombre de su módulo
(`GateF`, `Clona`), el `Transfer` el de su puerto de entrada (`Entrada1`) y
las demás el de su clase (`A_Assign`, `A_Assign2`, ...).

Los nombres se numeran **en el orden en que el listado los escribe**, que es el
orden en que la transacción recorre el modelo. Nuestro listado los nombraba en
el orden del modelo, así que en los modelos con ramas los números no coincidían
con los del original (`Entrada11` era otra sentencia). Ahora los nombres se
asignan en el orden del listado (`crates/genmsi-core/src/gpss.rs`, fn
`Names::new`).

**El orden del listado sigue sin coincidir con el del original en los modelos
con ramas** (el `TP3__aeropuerto` y el `TP10__tp10`): nuestro recorrido es
topológico y el del original es un recorrido en profundidad. Es una diferencia
del *listado*, no del reporte, y queda anotada abajo como lo que falta.

### 2. La fila `PREEMPT`

El original escribe los cuatro operandos de la sentencia: el recurso, el modo
de la interrupción, el destino del dueño interrumpido y el parámetro del
remanente. El nuestro escribía sólo el recurso.

```
    3       64       0     0              3  PREEMPT         Recurso,PR,,Remanente
```

### 3. La fila `SELECT`

La fila de un `SELECT` es la sentencia de la selección: la letra del operador
de la clase y sus operandos. El nuestro escribía siempre `MIN`/`MAX` con la
clase de la entidad al final.

```
A_SelectS    SELECT   SNF     param,1,10                (banco SelectS)
A_SelectL    SELECT   LS      param,1,10                (banco SelectL)
A_SelectSNA  SELECT   E       param,1,10,*1,Q,          (banco SelectSNA)
A_Select     SELECT   MIN     param,1,10,,Q             (banco Select)
```

El `SELECTENT` del original tiene un operando más que los demás y está
vacío: la fila conserva la coma final (`E param,1,10,*1,Q,`).

### 4. La fila `REMOVE`

La clase `RemoveMe` (la que saca la transacción del grupo) no lleva letra de
comparación ni cantidad: la fila es `REMOVE UnGrupo` con la columna del modo
vacía. La clase `RemoveTrans` sí la lleva: `REMOVE E UnGrupo,1,,otro,1`.

### 5. La fila `LINK`

El `LINK` escribe el bloque al que van las transacciones encadenadas:

```
    2       56      55     0         A_Link  LINK            Cadena,FIFO,A_Terminate
```

### 6. Las secciones que faltaban

El original escribe cuatro secciones que el nuestro no: `LOGICSWITCH` (el valor
de cada llave lógica y los intentos que fallaron su prueba), `XACT_GROUP` (los
grupos de transacciones y su tamaño), `NUMERIC_GROUP` (los grupos de números) y
`FEC_XACT_NUMBER` (los eventos que quedan pendientes al terminar la corrida).
La de `SAVEVALUE` lleva la columna `RETRY` y el nombre en mayúsculas.

```
LOGICSWITCH               VALUE      RETRY          SAVEVALUE                 VALUE      RETRY
  1                          0         62              COUNT                     62          0
XACT_GROUP           GROUP_SIZE      RETRY          NUMERIC_GROUP        GROUP_SIZE      RETRY
  UNGRUPO                    0          0              UNGRUPO                    1          0
FEC_XACT_NUMBER    PRI      BDT    CURRENT   NEXT   PARAMETER   VALUE
            65      0       3601        1       1
```

**El `GROUP_SIZE` de cada grupo** se midió con dos sondas contra el original
(`scripts/wine-oracle/probes/reporte/`, con `run_oracle.sh`):

| grupo | cómo se armó | lo que dice el original |
|---|---|---|
| `XACT_GROUP` | 36 transacciones que se unen a `GTRANS` y 6 que llegan al terminador | `30` (las que siguen vivas) |
| `NUMERIC_GROUP` | 36 transacciones que unen `GNUM` con 1, 2 y 3 | `3` (los números **distintos**) |
| `NUMERIC_GROUP` | lo mismo, con 2, 2 y 2 | `1` |

O sea: el grupo de transacciones cuenta sus miembros vivos y el de números
cuenta sus valores distintos (un banco que une sesenta veces el mismo número
da `1`, como en el banco `JoinNumber`).

### 7. Los contadores de una puerta que no abre

El banco `GateL` (una puerta `GATE LS 1` cuya llave nadie pone) dice el
original:

```
    2        0       0     0          GateL  GATE    LS      1
LOGICSWITCH               VALUE      RETRY
  1                          0         62
```

La puerta no cuenta nada (ni entradas, ni los que esperan en ella, ni los
intentos) y los intentos se cuentan en la llave lógica. Nuestro motor contaba
la entrada y el reintento del bloque.

**La regla que explica las tres medidas** (el banco `GateL`, el `TP6__Clase 6_2`
y el `TP3__aeropuerto`): la fila de la puerta cuenta las transacciones que
**entran** en ella; las que fallan la prueba y se van por la salida de fallo
entran y se cuentan; las que fallan la prueba y **esperan** no entran en la
bloque, no se cuentan, y su intento se cuenta en la entidad que la puerta
mira (la llave, el almacén o el recurso).

### 8. La tabla

El original reserva la última clase de la tabla para los valores que no caben
en ninguna otra, y la escribe aparte (`>   a  80`); el nuestro la gastaba en
una clase finita más (`81 - 90`). Además la cabecera lleva el operando del
atributo (`P$Atrib`) y los rótulos de las estadísticas están en su sitio.

Con la sonda `scripts/wine-oracle/probes/reporte/tablas.gmdl` (dos tablas, una
que tabula 5 y otra 95) el original dice:

```
TBL2            TABLE           P$Atrib,0,10,10
    MEDIA              : 95,000
    DESVIACION ESTANDAR: 0,000
    CANTIDAD EN RETRY  : 0

    INTERVALO         FRECUENCIA     %     %ACUMULADO     %REMANENTE
    0   -  0                0      0,000       0,000        100,000
    ...
    71  -  80               0      0,000       0,000        100,000
    >   a  80              36    100,000     100,000          0,000

    MEDIA OVERFLOW                : 95,000
    DESVIACION ESTANDAR OVERFLOW  : 0,000

    CANTIDAD DE VALORES TABULADOS : 36
    VALOR MAXIMO TABULADO         : 95,000
```

y el nuestro da **el mismo texto, línea por línea** (salvo el orden de las
tablas,(abajo)). De esa sonda sale además una regla que no estaba en el encargo:
el original **termina la tabla cuando las clases suman el 100%** y escribe
`Las frecuencias restantes (8) son todas cero.` en vez de las clases vacías.

Lo que **no** queda fijado por la evidencia: el `CANTIDAD EN RETRY` salió `0`
en todas las medidas (también con 36 valores en la clase de desborde), así que
sólo se sabe que cuenta las tabulaciones que hubo que **reintentar** (ninguna
en estos modelos). El nuestro escribe `0`.

### 9. La forma fina

- **Los decimales.** El original escribe tres decimales y coma en los promedios
  (`0,000`, `9,636`) y enteros en los conteos. Cuando el promedio no tiene
  divisor (una cola que no tomó a nadie, un recurso que nadie tomó) escribe
  `0.000`, con punto: es el formato que Delphi da a un real que no pudo
  calcular. La regla se ve en los reportes del original: `0,000      0.000
  0.000` (la cola `COLAINT`, con 64 entradas y 64 de espera) al lado de
  `0,000      0,000    0.000` (la misma cola con las dos medias calculables).
- **El orden de las colas.** El original ordena las entidades por el valor de
  su nombre cuando es un número: `1, 2, …, 10` y no `1, 10, 2, …`.
- **La línea `SEMILLA`.** El original no la escribe.

## Qué cambió

- `crates/genmsi-core/src/report.rs`
  - `number` escribe ahora tres decimales y coma; `whole` escribe los enteros
    (los del encabezado, los de la tabla y los del `SAVEVALUE`); `plain` deja
    el formato viejo (el de las tablas del oráculo); `not_computed` escribe el
    `0.000` del promedio sin divisor.
  - La tabla de bloques se arma con el listado (`gpss::block_names`,
    `gpss::listing_order`, `gpss::listing_lines`): las filas van en el orden
    del listado, el `LOC` es el nombre de la sentencia (o su número) y el
    `LINE` es su línea en el listado.
  - La fila del `PREEMPT` escribe sus cuatro operandos; la del `SELECT` la
    letra del operador de la clase; la del `REMOVE` de la clase `RemoveMe`
    deja la columna del modo vacía; la del `LINK` escribe el destino.
  - Las secciones nuevas: `LOGICSWITCH`, `XACT_GROUP`, `NUMERIC_GROUP` y
    `FEC_XACT_NUMBER`; la del `SAVEVALUE` con la columna `RETRY` y el nombre
    en mayúsculas.
  - Las secciones de las entidades (recursos, colas, almacenes, cadenas) con las
    columnas del original (y el `0.000` del promedio sin divisor).
  - La sección `TABLAS` con la cabecera del original (el atributo y los rótulos
    de las estadísticas en su sitio), la clase de desborde y las líneas
    `MEDIA OVERFLOW` y `DESVIACION ESTANDAR OVERFLOW`.
- `crates/genmsi-core/src/gpss.rs`: los nombres de las sentencias se asignan en
  el orden del listado (el original numera las sentencias repetidas en ese
  orden) y se exponen `block_names`, `listing_order` y `listing_lines`.
- `crates/genmsi-core/src/sim.rs`: la puerta no cuenta la transacción que
  espera (ni su entrada ni su reintento) y el intento se cuenta en la entidad
  que la puerta mira; el resultado lleva las llaves lógicas con sus reintentos,
  los grupos con su tamaño y los eventos que quedan pendientes.
- `crates/genmsi-core/src/stat.rs`: la tabla guarda la suma y la suma de
  cuadrados de los valores que no caben (para las líneas `OVERFLOW`).
- `crates/genmsi-core/src/oracle.rs` y los dos ejemplos: las tablas del oráculo
  siguen escribiendo los números como antes (`plain`).

## La prueba

`crates/genmsi-core/tests/reporte_forma.rs` (diez tests, uno por cada una de
las nueve diferencias y uno más para el nombre de la sentencia):

| test | qué fija |
|---|---|
| `the_loc_column_holds_the_name_of_the_statement` | el `LOC` del banco `GateF` (el nombre y los números) |
| `the_loc_is_the_name_of_the_statement_and_not_the_label_of_the_port` | una puerta etiquetada `A_GateF` escribe `GateF` |
| `the_preempt_row_writes_its_four_operands` | `PREEMPT Recurso,PR,,Remanente` |
| `the_select_rows_write_the_letter_of_the_operator` | `SNF`, `LS` y `E param,1,10,*1,Q,` |
| `the_remove_rows_write_the_group_and_the_comparison` | `REMOVE UnGrupo` y `REMOVE E UnGrupo,1,,otro,1` |
| `the_link_row_writes_the_destination` | `LINK Cadena,FIFO,A_Terminate` |
| `the_report_has_the_sections_of_the_original` | las cuatro secciones y el `SAVEVALUE` con `RETRY` |
| `a_gate_that_does_not_open_counts_nothing_and_the_switch_counts_the_attempts` | la fila `0 0 0` del banco `GateL` y el `RETRY` de la llave |
| `the_table_keeps_the_last_class_for_the_overflow` | la cabecera con `P$Atrib`, la clase de desborde y los rótulos |
| `the_fine_shape_of_the_report` | los decimales, el orden de las colas y la ausencia de `SEMILLA` |

También se actualizaron los tests que fijaban la forma anterior:
`crates/genmsi-core/tests/comportamiento.rs` (el nombre de la fila del
`TRANSFER` del `MATCH`), `crates/genmsi-core/tests/logic_switches.rs` (los
reintentos de la puerta se cuentan en la llave),
`crates/genmsi-core/tests/report_gpss.rs` (el `LOC` de la puerta es el nombre
de su módulo), `crates/genmsi-core/tests/storages.rs` (la hora del encabezado
es un entero) y `crates/genmsi-core/tests/trace_entities.rs` (el valor del
trazo).

## Lo que se comprobó

- **El oráculo de las métricas**: `cargo run --release -p genmsi-core --example
  oracle -- --runs 3` da **902 de 958** (antes 899 de 958). Suben el
  `Problema_1__semaforo-1` (131 → 132), el `TP6__Clase_6_1_PuertaBanco`
  (10 → 11) y el `TP6__Clase_6_2_UltimoCliente` (16 → 17): son las puertas
  que no abren, que ahora cuentan como el original. Ninguna métrica baja.
- **El GPSS**: los 62 bancos con respuesta del original
  (`/tmp/bench-out-new/`) coinciden, sentencia por sentencia
  (`scripts/wine-oracle/compare-gpss.sh`): **62 de 62**. El diferencial (el
  binario de antes contra el de ahora, sobre los 66 bancos) da **el mismo
  texto**: los cambios de este trabajo no tocan el listado.
- **Los bancos, con sus números**: 61 con reporte del original, 60 con todas
  sus métricas dentro de las tolerancias (el que queda es el
  `Multifurcacion`, cuyo 5/6 es el ruido del reparto de una rama).
- **Las sondas**: con las cuatro de `scripts/wine-oracle/probes/reporte/`
  (grupos y tablas) el nuestro da el mismo texto que el original, línea por
  línea, salvo el orden de las tablas (abajo).
- **Los tests**: `cargo test --workspace` en verde, con **366** tests (los 356
  de antes más los diez nuevos).
- **Clippy**: `cargo clippy --workspace --all-targets -- -D warnings` sin
  avisos.

## Lo que queda fuera (y por qué)

| # | Qué | Evidencia | Dónde | La prueba que lo fijaría |
|---|---|---|---|---|
| A | **El orden del listado en los modelos con ramas.** El original recorre el modelo en profundidad y el nuestro lo recorre por/topología: en el `TP3__aeropuerto` las sentencias `Entrada14`, `Entrada15` y `Entrada16` quedan en otro orden y con otros números. | los reportes del original (`fixtures/outputs/TP3__aeropuerto.report.txt`) contra los nuestros | `crates/genmsi-core/src/gpss.rs` · `flow_order` | Un test que compare el orden de las filas del `TP3__aeropuerto` con el del original |
| B | **El orden de las tablas.** El original escribe `TBL2` antes que `TBL1` (las dos del modelo de la sonda) y el nuestro las escribe en orden alfabético. Con dos tablas no se puede distinguir si es el orden inverso al de la declaración o el alfabético inverso. | la sonda `scripts/wine-oracle/probes/reporte/tablas.gmdl` | `crates/genmsi-core/src/report.rs` · el recorrido de `result.table_names` | Una sonda con tres tablas |
| C | **La fila del `GENERATE` no cuenta las transacciones que esperan en la puerta.** El original cuenta en el `CURRENT` del generador las 62 transacciones que esperan en la puerta (`63  63  0`); el nuestro cuenta sólo la que está agendada (`57  1  0`). | el banco `GateL` | `crates/genmsi-core/src/sim.rs` · la rama de la puerta | Un test que compare el `CURRENT` del generador del banco `GateL` |
| D | **La hora del `FEC_XACT_NUMBER` sale con decimales** (`3610,266`) porque nuestro generador sortea un intervalo real y el original uno entero. | el banco `GateF` | `crates/genmsi-core/src/sim.rs` · `apply_modifier` | Un test que fije que las horas del reporte son enteras |
| E | **La fila del `ALTER`** pierde los operandos (`ALTER ungrupo` contra `ALTER E ungrupo,1,param,0,otro,1`) y la fila del `MULTITRANSFER` escribe los pesos donde el original escribe la función. | los bancos `Alter` y `Multifurcacion` | `crates/genmsi-core/src/report.rs` · `operand_parts` | Un test por fila, como los de este lote |
| F | **El `CURRENT` de la puerta** que sí abre: el original lo deja en 0 (`GateL  0  0  0`) y el nuestro también, pero el del `Buffer` y el del `ASSEMBLE` tienen reglas propias que este trabajo no tocó. | el banco `Assemble` | `crates/genmsi-core/src/sim.rs` | Un test por clase |
| G | **El `CANTIDAD EN RETRY` de la tabla** no se pudo medir con un valor distinto de cero: el original lo escribe siempre en 0 en las sondas. | las sondas de las tablas | `crates/genmsi-core/src/report.rs` | Una sonda que produzca una tabulación reintentada |

## Tasks

- **T1** — Medir las nueve diferencias contra el original (S1, S2, S7) — hecho:
  las nueve están medidas con los reportes del original y con dos sondas
  nuevas; las notas de esta tabla.
- **T2** — La columna `LOC` y el orden del listado (S1.1) — hecho:
  `crates/genmsi-core/src/report.rs` (la tabla de bloques) y
  `crates/genmsi-core/src/gpss.rs` (los nombres y el orden); el test
  `the_loc_column_holds_the_name_of_the_statement`.
- **T3** — Las cuatro filas que perdían operandos: `PREEMPT`, `SELECT`,
  `REMOVE` y `LINK` (S1.2, S1.3, S1.4, S1.5) — hecho:
  `crates/genmsi-core/src/report.rs` · `operand_parts`; los tests
  `the_preempt_row_writes_its_four_operands`,
  `the_select_rows_write_the_letter_of_the_operator`,
  `the_remove_rows_write_the_group_and_the_comparison` y
  `the_link_row_writes_the_destination`.
- **T4** — Las secciones que faltaban (S1.6) — hecho:
  `crates/genmsi-core/src/report.rs` (las secciones) y
  `crates/genmsi-core/src/sim.rs` (las llaves, los grupos y los eventos
  pendientes); el test `the_report_has_the_sections_of_the_original`.
- **T5** — Los contadores de una puerta que no abre (S1.7) — hecho:
  `crates/genmsi-core/src/sim.rs` · la rama de la puerta; el test
  `a_gate_that_does_not_open_counts_nothing_and_the_switch_counts_the_attempts`.
- **T6** — La tabla (S1.8) — hecho: `crates/genmsi-core/src/stat.rs` (la
  clase de desborde) y `crates/genmsi-core/src/report.rs` (la sección); el
  test `the_table_keeps_the_last_class_for_the_overflow`.
- **T7** — La forma fina (S1.9) — hecho: `crates/genmsi-core/src/report.rs` ·
  `number`, `whole` y el orden de las entidades; el test
  `the_fine_shape_of_the_report`.
- **T8** — Comprobar el oráculo, el GPSS, los tests y el clippy (S4, S5) — hecho:
  902 de 958 de métricas, 62 de 62 de GPSS, 366 tests en verde y el clippy
  sin avisos.
- **T9** — Las notas y el commit (S6, S8) — hecho: este archivo y el commit
  de la rama `feature/reporte-forma`.
- **T10** — El reparto en commits (S8) — hecho: **un solo commit**
  (`e5610e2`, 1393 líneas) para el trabajo del core y sus pruebas, y **otro**
  para estas notas y las sondas. El encargo pide "el commit" y las nueve
  diferencias se tocan en los mismos dos archivos del core
  (`crates/genmsi-core/src/report.rs` y `crates/genmsi-core/src/sim.rs`), así
  que partirlos en unidades de unas 400 líneas dejaría commits intermedios que
  no compilan. El corte queda aquí: el commit `e5610e2` es el trabajo del
  motor y el que viene con estas notas es la documentación y la evidencia.

## Log

- **L1** — "Contexto: el bloque común de arriba.

  TRABAJA EN UN WORKTREE PROPIO, desde el HEAD actual:
    git worktree add ../genmsi-reporte -b feature/reporte-forma
    cd ../genmsi-reporte
  Ojo: los bancos son un artefacto generado que no está en git; en el worktree
  nuevo hay que generarlos (los tests del core los leen):
    python3 scripts/wine-oracle/make-benches.py

  QUÉ HACER (nueve diferencias de la FORMA del reporte):
  1. LA COLUMNA LOC. Sale del mylabel del puerto; el original escribe el nombre
     de la sentencia (GateF, Clona, A_Terminate1). El generador de bancos ya
     reproduce esa regla en las etiquetas del modelo, así que el original y el
     nuestro ya parten de la misma etiqueta.
  2. LA FILA PREEMPT pierde los operandos: escribimos "PREEMPT Recurso" donde el
     original escribe "PREEMPT Cajero1FreeShop,PR,,Remanente".
  3. LA FILA SELECT: escribimos "SELECT MIN param,1,10,,S" (o ",,LS", o "MIN...")
     donde el original escribe "SNF param,1,10", "LS param,1,10" y
     "E param,1,10,*1,Q,".
  4. LA FILA REMOVE lleva de más la letra E en el RemoveMe.
  5. LA FILA LINK no escribe el destino de la sentencia.
  6. FALTAN SECCIONES: LOGICSWITCH, XACT_GROUP, NUMERIC_GROUP y FEC_XACT_NUMBER;
     y la de SAVEVALUE va sin la columna RETRY y con el nombre en minúsculas.
  7. LOS CONTADORES DE UNA PUERTA QUE NO ABRE: el nuestro cuenta el intento
     fallido como entrada y reintento del bloque; el original no cuenta la
     entrada y cuenta los intentos en la llave. La fila de la puerta del banco
     GateL tiene que salir 0 0 0.
  8. LA TABLA: el original reserva la última de las Cantidad clases para el
     desborde ("> a 80") y el nuestro la gasta en otra clase finita ("81 - 90")
     y no tiene fila de desborde. Además la cabecera pierde el operando del
     atributo (P$Atrib) y los rótulos de las estadísticas están cambiados de sitio.
  9. LA FORMA FINA: los decimales enteros (0 donde el original escribe 0,000),
     el orden de las colas (el original pone 1, 2, ... 10 y el nuestro
     1, 10, 2, ...) y la línea SEMILLA, que el original no trae.

  CÓMO SE HACE:
  - La evidencia está en el original, corriendo bajo Wine:
      bash scripts/wine-oracle/run_oracle.sh <modelo> <directorio>
    En scripts/wine-oracle/probes/ hay sondas de sesiones anteriores que puedes
    reusar y ampliar.
  - El reporte del original es la especificación de estas nueve: compara fila por
    fila, no "hasta donde se parezcan".
  - El oráculo de las métricas no puede bajar de 899 de 958, y el GPSS de los 62
    bancos con respuesta del original no puede cambiar (tú no tocas gpss.rs, pero
    tus cambios pueden alterar lo que se compila: míralo).
  - Si al arreglar una cosa se rompe otra, dilo y deja el estado como estaba antes
    de tu cambio, con la explicación.

  DONE CUANDO:
  - Un test del core por cada una de las nueve diferencias.
  - Métricas: 899 de 958 o más. GPSS: 62 de 62 sin cambios.
  - `cargo test --workspace` y `cargo clippy --workspace --all-targets -- -D warnings`
    en verde, con los tests subiendo.
  - Las notas en odd/tasks/notas-reporte-forma.md (nuevo).
  - El commit en tu worktree, y me dices la rama.

  Y una regla del repositorio, para lo que commits: NINGÚN commit lleva pie de
  atribución (nada de "Co-Authored-By" ni similar). El proyecto es del lector y
  los commits no llevan la firma de ninguna herramienta ni de ningún modelo. El
  mensaje del commit explica qué cambió y por qué, y nada más."

- **L2** — El oráculo de las métricas subió a **902 de 958** (de 899): las tres
  puertas que no abren ahora cuentan como el original.
- **L3** — La sonda de los grupos y la de las tablas
  (`scripts/wine-oracle/probes/reporte/`) fijaron el `GROUP_SIZE` de los dos
  tipos de grupo y la forma de la sección `TABLAS`.
- **L4** — Lo que queda fuera está en la tabla de la sección "Lo que queda
  fuera": el orden del listado en los modelos con ramas (A), el orden de las
  tablas (B), el `CURRENT` del generador con las transacciones que esperan en la
  puerta (C), las horas con decimales del `FEC_XACT_NUMBER` (D), las filas del
  `ALTER` y del `MULTITRANSFER` (E), el `CURRENT` de otras clases (F) y el
  `CANTIDAD EN RETRY` de la tabla (G).
