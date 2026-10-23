# Notas: las diferencias que quedaban (unidad 6)

Las cinco diferencias que el inventario dejó abiertas en
`odd/tasks/notas-reporte-forma.md` (la sección "Lo que queda fuera": A, B, C,
E y G), medidas contra el original bajo Wine
(`scripts/wine-oracle/run_oracle.sh`) y con los bancos y los modelos de
`fixtures/`.

Tres de las cinco quedan arregladas y con su test. Las otras dos quedan
anotadas con lo que se midió y por qué no se pueden decidir.

## Specs

- **S1** — "EL ORDEN DEL LISTADO EN LOS MODELOS CON RAMAS. Nuestro recorrido
  es topológico y el del original es en profundidad (el TP3 y el TP10 lo
  muestran). El original numera las sentencias repetidas en el orden del
  listado, no del modelo. Midelo con los bancos y con los modelos de
  fixtures/, y, si la regla queda clara, aplícala al listado (gpss.rs) y al
  recorrido del programa (program.rs). Si no queda clara, no la inventes:
  anótalo."
- **S2** — "LA FILA DEL MULTITRANSFER. El original escribe otra cosa que la
  que escribimos (mira la ficha del lote 2 y la respuesta del original en
  /tmp/all-oracle/MultifurcacionP.report.txt)."
- **S3** — "EL CURRENT DEL GENERADOR CON LAS TRANSACCIONES QUE ESPERAN EN LA
  PUERTA. El original cuenta de una manera y el nuestro de otra; mide con el
  banco GateL y con el fixture TP6."
- **S4** — "EL ORDEN DE LAS TABLAS. Con dos tablas no se puede distinguir si el
  original las pone en el inverso al de la declaración o en el alfabético
  inverso: haz la sonda que lo distinga y aplica lo que salga. Si no se puede,
  anótalo."
- **S5** — "LA COLUMNA 'CANTIDAD EN RETRY' DE LA TABLA. Salió 0 en todas las
  sondas: sólo se sabe que cuenta las tabulaciones reintentadas. Si una sonda
  la fija, aplícala; si no, anótalo y NO la inventes."
- **S6** — "El GPSS no puede bajar de 66 de 66 y el oráculo no puede bajar de
  66 de 66. `cargo test --workspace` y `cargo clippy --workspace --all-targets
  -- -D warnings` en verde, con los tests subiendo."
- **S7** — "Si al arreglar una cosa se rompe otra, dilo y deja el estado como
  estaba antes de tu cambio, con la explicación."
- **S8** — "NINGÚN commit lleva pie de atribución."

## Resumen

| # | Punto | Qué era | Estado |
|---|---|---|---|
| 1 | El orden del listado | el recorrido era topológico | **arreglado** (el TP10 coincide entero; el TP3 no, y se explica por qué) |
| 2 | La fila del multifurcación | el `LOC` se comía un espacio de más | **arreglado** |
| 3 | El `CURRENT` del generador | ya lo arregló el commit `0aad38b` | **fijado con un test** |
| 4 | El orden de las tablas | alfabético | **arreglado** (sonda de tres tablas) |
| 5 | El `CANTIDAD EN RETRY` de la tabla | siempre `0` | **no se puede decidir** (anotado) |

## 1. El orden del listado: un recorrido en profundidad

### La medida

El original escribe los bloques del programa en el orden en que una
transacción los recorre, y el recorrido es **en profundidad**: la rama del
segundo puerto de un bloque va después de toda la rama del primero.

El modelo `TP10__tp10` lo muestra y es el que se puede medir entero (su
reporte del original está en `fixtures/outputs/TP10__tp10.report.txt`, con la
tabla de bloques completa). El nuestro, antes del cambio, era topológico: las
fuentes primero y las ramas después.

Las diferencias, sentencia por sentencia:

| | el original | el nuestro (antes) |
|---|---|---|
| el `UNLINK` y su rama | `A_Unlink1`, `A_Terminate2`, `A_Advance2` | `A_Unlink1`, `A_Advance2`, …, `A_Terminate2` |
| el `EXAMINE` y su rama | `A_ExamineNumber`, `A_Assign7` | `A_Assign6`, `A_ExamineNumber`, `A_Assign7` |
| el `GENERATE` del grupo `EsperaHastaAbrir` | `GENERATE ,,,1`, `A_Logic`, `A_Advance` | `A_Logic`, `A_Advance`, …, `GENERATE ,,,1` |

### La regla

La regla que sale de la medida del `TP10` (y que el listado de los 66 bancos
no contradice, porque los bancos son lineales) es:

1. El recorrido arranca en los bloques a los que **nada lleva** (las fuentes
   del flujo), **en el orden del modelo**.
2. De cada bloque se siguen sus salidas **en el orden de los puertos**, y la
   rama de un puerto se recorre **entera** antes de la del siguiente.
3. Los bloques a los que el recorrido no llega (un ciclo, o una rama que
   nada alimenta) van al final, en el orden del modelo.

Con esa regla, el listado del `TP10` coincide **sentencia por sentencia**
con la tabla de bloques del reporte del original (los 60 nombres de las
sentencias, en el mismo orden). Antes del cambio no coincidía ninguno.

### El código

`crates/genmsi-core/src/gpss.rs` · fn `flow_order`: el recorrido topológico
(los módulos, con un ordenamiento por predecesores) se cambia por el
recorrido en profundidad sobre los bloques: se cuentan las entradas de cada
bloque, las fuentes son las raíces y la pila del recorrido se llena con las
sucesoras al revés, para que la rama del primer puerto sea la primera.

### Lo que no queda claro: el `TP3`

El `TP3__aeropuerto` tiene dos cadenas que el recorrido en profundidad
coloca en un sitio que el original no:

- la cadena del grupo `FirstClassGenerate` (`GENERATE 10,5,600,30`,
  `A_Assign`, …, `A_Assign43`), que el original escribe **primera**, y
- la cadena del grupo `Supervisor` (`GENERATE 1800,,2700`, `A_FacilityInt`,
  `A_FacilityInt1`, `A_Terminate2`), que el original escribe **en medio**
  (después de `A_Release1` y antes de `A_Advance5`), y que ninguna rama del
  modelo alimenta.

Se midió lo que se pudo para explicar el orden del original y **no se
encontró la regla**:

- El recorrido en profundidad con las raíces en el orden del modelo da la
  cadena de `FirstClassGenerate` en su sitio (la primera) ✓, pero la de
  `Supervisor` sale primera de todo, y no en medio.
- La raíz del recorrido del `TP3` no es el primer módulo del modelo (el
  `Advance1`, que es el primero), sino el `Generate1` del grupo
  `FirstClassGenerate`, y el orden de las raíces del original
  (`FirstClassGenerate`, `Supervisor`, `Advance3`, `BussinesClassGenerat`,
  `EconomyClassGenerate`, `Clock`, `Cambio de q`) no es el del modelo
  (los ids son 55, 40, 6, 62, 68, 80, 81), ni el inverso, ni el de la
  posición en el lienzo, ni el de la longitud de las cadenas.
- Se probaron el recorrido en profundidad con las raíces en el orden del
  modelo, con las raíces invertidas, con las fuentes solamente, y el
  recorrido "plano" (sin pila): ninguno da el orden del original.

Lo que **sí** queda fijado es la parte del recorrido que se puede medir
entera (el `TP10`): la profundidad, el orden de los puertos y el hecho de que
una rama se recorre entera antes de la siguiente. La posición de una cadena
que ninguna rama alimenta (el `Supervisor` del `TP3`) queda **sin decidir**:
no se inventa.

### La prueba

`crates/genmsi-core/tests/diferencias_restantes.rs` ·
`the_listing_walks_the_flow_in_depth`: el listado del `TP10__tp10` se compara,
sentencia por sentencia, con la columna `LOC` de la tabla de bloques del
reporte del original (`fixtures/outputs/TP10__tp10.report.txt`).

## 2. La fila del multifurcación: el `LOC` que no cabe

### La medida

El original (`/tmp/all-oracle/MultifurcacionP.report.txt`, línea 9):

```
    4       60       0     0MultifurcacionP  TRANSFER        ,FN$MultifurcacionGO
```

El nuestro, antes:

```
    4       62       0     0 MultifurcacionP  TRANSFER        ,FN$MultifurcacionGO
```

La fila del bloque es la misma (el `TRANSFER` con la función de la clase ✓,
eso ya estaba); lo que difiere es **la columna `LOC`**: la del original es un
campo de quince columnas alineado a la derecha, y un nombre que no cabe se
come el espacio que lo separa de la columna `RETRY`.

La misma regla en el reporte del original de la `entregaComida`, cuyo `LOC`
`HornoConComidas` (dieciséis letras) se come ese espacio y empuja la celda
del `BLOCK_TYPE` una columna más:

```
   11       30       0     0HornoConComidas  GATE    SNE     Comidas
```

### El código

`crates/genmsi-core/src/report.rs` · fn `render` (la tabla de bloques): el
`LOC` se escribe como un campo de quince columnas alineado a la derecha que
empieza en la columna veintiocho (justo detrás del `RETRY`), y no como un
campo de catorce columnas con un espacio de por medio.

### La prueba

`crates/genmsi-core/tests/diferencias_restantes.rs` ·
`the_row_of_the_multifurcation_eats_the_space_of_its_column`: la fila del
`MultifurcacionP` sale `0MultifurcacionP  TRANSFER ,FN$MultifurcacionGO`.

Los dos tests que leen las filas de la tabla de bloques siguen el cambio (el
`LOC` ahora empieza en la columna veintiocho): `crates/genmsi-core/tests/
report_gpss.rs` (la constante `BLOCK_TABLE`) y `crates/genmsi-core/tests/
reporte_pendientes.rs` (la constante `LOC_COLUMN`).

## 3. El `CURRENT` del generador con las transacciones que esperan en la puerta

### La medida

El reporte del original del banco `GateL` (`/tmp/all-oracle/GateL.report.txt`):

```
    1       58      58     0              1  GENERATE        60,30
    2        0       0     0          GateL  GATE    LS      1
```

La fila de la puerta es `0 0 0` ✓ (eso ya lo arregló el commit `e5610e2`) y
la del generador es `58 58`: **las transacciones que esperan en la puerta
siguen contando en el bloque del que vinieron**. El nuestro, antes del
`0aad38b`, decía `60 1`.

El commit `0aad38b` ("the transaction that waits at a shut gate") ya lo
arregló en el motor, y con él los snapshots de la CLI. Lo que faltaba era
**fijarlo con un test**, y eso es lo que hace esta unidad.

El fixture `TP6__Clase 6_1_PuertaBanco` lo dice también: la cola de la puerta
cerrada lee `13 13` en el original.

### La prueba

`crates/genmsi-core/tests/diferencias_restantes.rs` ·
`the_generator_counts_the_transactions_that_wait_at_the_gate`: con un
generador, una puerta que nunca abre y un terminador, todas las
transacciones que esperan en la puerta siguen en el generador (su `CURRENT`
es su `ENTRY`) y la fila del reporte lo dice igual.

## 4. El orden de las tablas: el inverso al de la declaración

### La sonda

La sonda de las notas (`scripts/wine-oracle/probes/reporte/tablas.gmdl`) tiene
dos tablas, `TBL1` y `TBL2`, y el original las escribe en el informe al revés
de como las declara, lo que no distingue el "inverso al de la declaración"
del "alfabético inverso". La sonda nueva tiene **tres** tablas, declaradas
`TBL2`, `TBL1` y `TBL3`, y un solo flujo que las tabula en el orden `TBL1`,
`TBL3`, `TBL2` (para que también se distinguiese del "inverso al de uso").

La respuesta del original (corrida con `run_oracle.sh`; el modelo está en
`/tmp/w6/probes/tablas_tres.gmdl` y la respuesta en
`/tmp/w6/oracle/tablas_tres/`):

```
TABLAS

TBL3            TABLE           P$Atrib,0,10,10
    ...
TBL1            TABLE           P$Atrib,0,10,10
    ...
TBL2            TABLE           P$Atrib,0,10,10
```

O sea:

| regla | lo que daría |
|---|---|
| el inverso al de la declaración (`TBL2`, `TBL1`, `TBL3`) | `TBL3`, `TBL1`, `TBL2` ✓ |
| el alfabético inverso | `TBL3`, `TBL2`, `TBL1` |
| el inverso al de uso (`TBL1`, `TBL3`, `TBL2`) | `TBL2`, `TBL3`, `TBL1` |

**La regla es el inverso al de la declaración** ✓.

La misma corrida muestra otra cosa: el **listado** del original escribe las
tablas en el orden de la declaración (`TBL2`, `TBL1`, `TBL3`), y el informe
las escribe al revés.

### El código

- `crates/genmsi-core/src/program.rs` · `Definitions`: un campo nuevo,
  `table_order`, con el orden en que el modelo declara las tablas
  (`collect_definitions` lo llena).
- `crates/genmsi-core/src/sim.rs` · fn `run`: `table_names` se arma con ese
  orden **invertido** (antes se ordenaban alfabéticamente).
- `crates/genmsi-core/src/gpss.rs` · fn `definitions`: el listado escribe las
  tablas en el orden de la declaración (antes alfabéticas).

### La prueba

`crates/genmsi-core/tests/diferencias_restantes.rs` ·
`the_tables_come_the_other_way_round_than_the_declaration`: un modelo con
tres tablas (`TBL2`, `TBL1`, `TBL3`): el listado las escribe en el orden de
la declaración y el informe al revés.

## 5. El `CANTIDAD EN RETRY` de la tabla: no se puede decidir

Lo que dice la ficha del lote 3 es que el `CANTIDAD EN RETRY` salió `0` en
todas las medidas y que sólo se sabe que cuenta las tabulaciones que hubo
que **reintentar**. Se buscaron sondas que lo fijaran y ninguna lo
consiguió:

| sonda | qué busca | qué dice el original |
|---|---|---|
| `tabla_inexistente` | tabular en una tabla que nadie declara | el original **se para**: "ERROR!!Se intentó actualizar el valor de una tabla inexistente. (Tabla: -1)." — no hay informe |
| `tabla_muchos` | cincuenta transacciones que tabulan la misma tabla en el mismo instante | `CANTIDAD EN RETRY  : 0` |
| `tabla_peso_cero` | una tabulación con peso cero | `CANTIDAD EN RETRY  : 0` |
| `tabla_negativa` | tabular un valor negativo (debajo de la primera clase) | `CANTIDAD EN RETRY  : 0` |
| `tabla_debajo` | tabular un valor por debajo de la primera clase (la primera clase empieza en 10) | `CANTIDAD EN RETRY  : 0` |

El manual (`VISUALSIS.HLP`) dice que "nunca se le niega la entrada a un bloque
TABULATE", y el ejecutable del original sólo tiene el mensaje de error de
"vincular una retry de una tabla inexistente" (una entidad inexistente, no un
reintento). **No hay forma de que una tabulación se reintente** con lo que
se sabe del original, así que la columna se queda en `0` —que es lo que
escribimos— y **no se inventa** una regla.

## Lo que queda fuera (y por qué)

Estas cuatro diferencias se midieron con el original mientras se hacía esta
unidad. **No se arreglan aquí**: no son ninguno de los cinco puntos del
encargo y su arreglo toca más archivos de los que la unidad puede tocar.

| # | Qué | Evidencia | Dónde |
|---|---|---|---|
| A | **La tabla de bloques del informe no tiene las filas de las conexiones.** El original escribe una fila por cada sentencia del listado, y las conexiones (los `TRANSFER` que el listado escribe cuando el flujo se rompe) son filas más, con el número de la sentencia en la columna `LOC`. En el `TP10` el original tiene 69 filas y nosotros 66; en el `TP3`, 108 y 103. | `fixtures/outputs/TP10__tp10.report.txt` (filas `48 TRANSFER ,A_Assign6`, `59 TRANSFER ,A_Advance2`, `61 TRANSFER ,A_Assign5`) contra el informe nuestro | `crates/genmsi-core/src/report.rs` · fn `block_table` (las filas se arman con `gpss::listing_order`, que son bloques, y no con las sentencias del listado) |
| B | **El orden de los almacenes en el informe.** El original los escribe en el orden de la declaración (`MIGRACION`, `ENTREGAEQUIPAJE`, `SALIDA` en el `TP3`) y nosotros en el alfabético (`MIGRACION`, `SALIDA`, `ENTREGAEQUIPAJE`). | `fixtures/outputs/TP3__aeropuerto.report.txt` | `crates/genmsi-core/src/sim.rs` (los almacenes) y `crates/genmsi-core/src/report.rs` |
| C | **El rótulo `TABLAS` se repite antes de cada tabla.** El original lo escribe una vez, al principio de la sección; con dos o más tablas, nosotros lo escribimos otra vez antes de cada una. | la sonda `tablas_tres` | `crates/genmsi-core/src/report.rs` · fn `render` (el bucle de `result.table_names`) |
| D | **El `GENERATE` con la media vacía.** El original escribe `GENERATE ,,,1` y nosotros `GENERATE 0,,,1` (la media vacía se escribe como `0`). | `fixtures/outputs/TP10__tp10.report.txt` (fila 52) contra el informe nuestro | `crates/genmsi-core/src/gpss.rs` · fn `block_statements` (la rama `BlockKind::Generate`) |

## Las cifras

- **El oráculo**: 66 de 66 bancos coinciden en todas sus métricas
  (`cargo run --release -p genmsi-core --example compare-benches --
  scripts/wine-oracle/benches /tmp/all-oracle --runs 3`).
- **El GPSS**: 66 de 66 con la respuesta fresca del original
  (`scripts/wine-oracle/compare-gpss.sh` banco por banco). Con la respuesta
  que hay en `/tmp/all-oracle` son 65 de 66: la del `GateS` es **vieja** (se
  tomó antes del commit `61b4d87`, "the bench of the gate on a storage
  declares the storage", que le añadió la declaración del almacén al banco;
  corrida de nuevo, el original escribe `Almacen STORAGE 10` y coincide con
  el nuestro). No es una regresión de esta unidad: con el código de antes
  pasa lo mismo.
- **Los tests**: `cargo test --workspace` en verde (54 ejecutables, 0
  fallos), con los cuatro tests nuevos de
  `crates/genmsi-core/tests/diferencias_restantes.rs`.
- **El clippy**: `cargo clippy --workspace --all-targets -- -D warnings` sin
  avisos.

## Log

- **L1** — Las cuatro diferencias de la lista del encargo quedan con su
  cambio y su test; la quinta (el `CANTIDAD EN RETRY` de la tabla) queda
  anotada con las cinco sondas que se hicieron y con el motivo por el que no
  se puede decidir.
- **L2** — El recorrido del listado pasa a ser en profundidad. El `TP10`
  coincide entero con el original; del `TP3` se explica lo que no se pudo
  decidir (la posición de una cadena que ninguna rama alimenta).
- **L3** — Se toca un archivo fuera de la superficie de la unidad:
  `crates/genmsi-core/tests/reporte_pendientes.rs` (una constante, la
  columna donde empieza el campo `LOC`). Es el mismo cambio que el de
  `crates/genmsi-core/tests/report_gpss.rs`, que sí estaba en la superficie:
  los dos tests leen las filas de la tabla de bloques por columnas, y la
  columna `LOC` del informe es ahora la del original (empieza en la columna
  veintiocho).
