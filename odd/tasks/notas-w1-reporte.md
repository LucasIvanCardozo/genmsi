# Notas: las filas y las secciones del reporte que faltaban

Las once diferencias que este trabajo cierra, más lo que queda medido y sin
arreglar. Todo lo que sigue está medido contra el original (`genmsi.exe` bajo
Wine, `scripts/wine-oracle/`), no supuesto: cada fila del original que se
cita sale de su reporte (`/tmp/bench-out-new/<Clase>.report.txt` o
`fixtures/outputs/*.report.txt`).

La especificación de cada fila, cuando se pudo, está además en el **código
macro de las clases del original** (`Genmsi_20230711.backup/gClasses.xml`), que
es la fuente de la que salen las sentencias que el original escribe.

## Specs

- **S1** — "Tu trabajo es hacer que las filas y las secciones del reporte se
  escriban como las del original. La lista está medida, con el archivo, la
  función y la prueba; no hay que adivinar nada", con los once puntos:
  1. la fila del `ALTER` pierde los operandos; 2. la del `ENTER` y la del
  `LEAVE` pierden las unidades; 3. la del `TRANSFER` pierde la probabilidad;
  4. las de la `Multifurcacion` y la `MultifurcacionP`; 5. la del `LOOP`
  pierde el segundo operando; 6. la de la `GateM`; 7. la sección `QTABLES`;
  8. las secciones `XACT_GROUP` y `NUMERIC_GROUP`; 9. la fila del `TEST`
  escribe `P:Param` donde el original escribe `P$Param`; 10. la columna `LINE`
  de la tabla de bloques; 11. la columna `BDT` de los eventos pendientes.
- **S2** — "Antes de cambiar nada, mira el reporte del original y el nuestro
  del mismo banco y compáralos fila por fila. El original manda. Si el
  original dice una cosa y el spec de arriba otra, manda el original: dilo."
- **S3** — "Cada punto va con un test en `crates/genmsi-core/tests/report_gpss.rs`
  o en el test nuevo que crees, que fije el texto exacto."
- **S4** — "El oráculo (compare-benches) no puede bajar de 60 de 61."
- **S5** — "`cargo test --workspace` y `cargo clippy --workspace --all-targets
  -- -D warnings` en verde, con los tests subiendo."
- **S6** — "Al terminar, haz UN commit en tu rama, con el estilo del
  repositorio (Conventional Commits, mensaje en inglés, explicando qué cambió
  y por qué). NINGÚN commit lleva pie de atribución."

## Lo que se encontró, punto por punto

### 1. La fila del `ALTER`

El original escribe la sentencia entera: el operador de la comparación, el
grupo, la cantidad de transacciones que altera, el parámetro que altera y su
valor, y el parámetro que compara y su valor.

```
    2       57       0     0        A_Alter  ALTER    E       ungrupo,1,param,0,otro,1
```

El código macro de la clase (`gClasses.xml`) lo confirma: `ALTER
[CompOp:'operador'='='],... [Grupo:...='ungrupo'],... [Cantidad:...='1'],...
[AltParam:...='param'],... [AltValor:...='0'],... [CompParam:...='otro'],...
[CompValue:...='1']`.

El nuestro escribía `ALTER           ungrupo` (sólo el grupo y sin la letra
del operador). Ahora escribe la fila entera.

Un detalle de la forma: el `ALTER` del original ocupa **una columna más** que
las otras sentencias con letra (`ALTER    E`, con el verbo en 9 columnas y no
en 8, como `GATE    M` o `TEST    E`); está medido en los cuatro reportes del
original que tienen una sentencia con letra (`ALTER`, `GATE`, `LOGIC`,
`REMOVE`, `SCAN`, `SELECT`, `TEST`, `UNLINK`).

El destino de la salida de fallo ("No alteró") es el último operando de la
sentencia, y sólo cuando esa salida está conectada: el reporte del original
para la sonda `AlterMiembro` es `ALTER    E
UnGrupo,1,Param,99,Param,0,A_Terminate1` y el del banco, cuya salida de
fallo no está conectada, es `ALTER    E       ungrupo,1,param,0,otro,1`. El
listado ya lo escribía (el trabajo de `notas-w2-listado.md`); el reporte lo
escribe ahora, con la misma regla.

### 2. La fila del `ENTER` y la del `LEAVE`

El original escribe las unidades:

```
    3       10       0     0        A_Enter  ENTER           AStorage,1
    4       63       0     0        A_Leave  LEAVE           AStorage,1
```

(los bancos `Enter` y `Leave`; también `A_Enter  ENTER  Organizador,1` en el
`TP10` y `A_CargarTanda  ENTER  Comidas,5` en el `entregaComida`).

El código macro dice por qué: la clase `Enter` es
`ENTER [Storage],[Cantidad:'Cantidad de ingresos'='1']` — su `Cantidad` tiene
un **valor por defecto de uno**, así que la sentencia siempre lleva las
unidades. La clase `StorageUse`, en cambio, es `ENTER
[Storage,...],[Cantidad:'Cantidad de ingresos/Egresos del storage'='']`: su
`Cantidad` no tiene valor por defecto y el original **no** escribe las
unidades cuando el modelo no las da.

```
    4       59       0     0              3  ENTER           Almacen
```

(banco `StorageUse`).

El programa compilado no guarda el texto de las propiedades: `Cantidad = 1` y
`Cantidad` vacío llegan los dos como el número uno. La regla que sí se puede
implementar es la de las clases: las clases `Enter` y `Leave` escriben siempre
las unidades, y las sentencias que escribe una clase que se expande en varias
(`StorageUse`) las escriben sólo cuando el modelo las da. Eso es lo que hace
el reporte ahora, con el número de sentencias del módulo como distinguishable.

### 3. La fila del `TRANSFER`

El original escribe la probabilidad en miles:

```
    2       60       0     0       Entrada1  TRANSFER        1000,,A_Terminate
```

(banco `Transfer`, cuyo modelo tiene `Prob = 100%`).

El nuestro escribía `TRANSFER        ,,A_Terminate`: la probabilidad se
perdía porque el compilador dobla el `100%` a `1.0` y el reporte trataba ese
`1.0` como "sin probabilidad" (para el `TRANSFER` que escribe la clase
`Match`, que sí va sin probabilidad: `TRANSFER ,AAA`).

Ahora la regla es la del listado (`gpss.rs`): si la probabilidad es un
número, se escribe en miles (`100%` es `1000`); si es una expresión, se
escribe su texto en la notación del original (`X$qValue`, `FN$PValue`,
`P$pValue`); y si no hay probabilidad (el `TRANSFER` de la clase `Match`), la
primera columna queda vacía.

### 4. Las filas de la `Multifurcacion` y la `MultifurcacionP`

El original escribe la transferencia a la función que mapea la rama a su
destino:

```
    6       60       0     0 Multifurcacion  TRANSFER        ,FN$MultifurcacionGO
```

El nuestro escribía los pesos del modelo (`TRANSFER 0.2,0.2,0.2,0.2,0.2`) y,
en la `MultifurcacionP`, el nombre de la clase (`MULTIFURCACIONP`).

El código macro de las dos clases lo dice: `MULTITRANSFER
MultifurcacionGO,FN:MultifurcacionCH,{...}` y `MULTITRANSFER
MultifurcacionGO,P:[Parametro],{...}`. Las dos escriben la misma sentencia
GPSS, y el `BLOCK_TYPE` de las dos es `TRANSFER`.

**Lo que no se pudo medir**: el original **no escribió reporte** para el banco
`MultifurcacionP` (`/tmp/bench-out-new/MultifurcacionP.report.txt` está
vacío), así que esa fila sale del código macro de la clase y no de una
medida.

### 5. La fila del `LOOP`

El original escribe los dos operandos del `LOOP`, y el segundo (el destino
de la iteración) queda vacío cuando el modelo no lo conecta:

```
    2       59       0     0         A_Loop  LOOP            Param,
```

El nuestro escribía `LOOP            Param` (se comía la coma final). Ahora
la coma se queda, como en el `SELECTENT` (`E param,1,10,*1,Q,`).

### 6. La fila de la `GateM`

El original escribe `M` (la transacción tiene que coincidir con la etiqueta)
y no el `LS` de una llave lógica:

```
    4        0       0     0          GateM  GATE    M       AAA
```

El código macro de la clase es `GATEM [op:...='Maching'],... [Rotulo:...]`,
y el listado (`gpss.rs`) ya escribía `GATE M AAA`. El reporte no.

### 7. La sección `QTABLES`

El original escribe la `QTABLE` como escribe la `TABLE`: la cabecera con los
operandos de la sentencia, las tres líneas de las estadísticas, el histograma
entero y las dos líneas que lo cierran.

```
QTABLES
QTabla          QTABLE          qRecurso,0,600,60
    MEDIA              : 0,000
    DESVIACION ESTANDAR: 0,000
    CANTIDAD EN RETRY  : 0

    INTERVALO         FRECUENCIA     %     %ACUMULADO     %REMANENTE
    0   -  0                0      0,000       0,000        100,000
    ...
    >   a  34800              0      0,000       0,000        100,000

    CANTIDAD DE VALORES TABULADOS : 0
    VALOR MAXIMO TABULADO         : 0,000
```

El nuestro escribía cuatro líneas de resumen con los operandos con
decimales (`QTabla           QTABLE          0,000, 600,000, 60`) y sin el
histograma.

Ahora el cuerpo de las dos secciones (`TABLAS` y `QTABLES`) es el mismo
función (`write_table_body`), y el reporte del banco `QTable` sale **igual
línea por línea** al del original.

Al comparar las dos tablas salió además una diferencia de la sección `TABLAS`
que tampoco se había medido: la columna del límite inferior del intervalo es
de **tres** columnas y el separador lleva un espacio delante (`1201 -  1800`),
no cuatro (`1201-  1800`). Con los números cortos de la `TABLAS` del banco
`Table` no se nota; con los de la `QTABLE` sí.

### 8. Las secciones `XACT_GROUP` y `NUMERIC_GROUP`

El original escribe la fila del grupo aunque nadie pertenezca a él:

```
XACT_GROUP           GROUP_SIZE      RETRY
  UNGRUPO                    0          0
```

(bancos `Examine` y `RemoveTrans`; el `ExamineNumber` escribe la misma fila
en `NUMERIC_GROUP`).

El nuestro sólo listaba los grupos que el motor conoce (los que alguien se
unió), así que las secciones no salían. Ahora el reporte añade los grupos que
nombran las sentencias del programa (`EXAMINE`, `EXAMINE NUMBER` y `REMOVE`),
con su tamaño en cero.

### 9. La fila del `TEST` (y de las otras que llevan una expresión)

El original escribe las expresiones en la notación de GPSS, con un dólar:

```
   81        0      0     0         A_Test  TEST    E       P$Cuadra,50,A_Test1
```

El nuestro escribía `P:Cuadra` (la notación del modelo). La causa es que el
listado (`gpss.rs`) ya traducía la `:` a `$` y el reporte no.

Ahora `expr_operand` usa la misma traducción (`gpss::sna`), así que el
reporte y el listado dicen lo mismo. **Esto toca más filas que la del
`TEST`**: el `JOIN`, el `EXAMINE`, el `REMOVE`, el `UNLINK`, el `SPLIT`, el
`SCAN`, el `ASSIGN` y el `TRANSFER` también llevan expresiones, y el original
las escribe con el dólar en todas (`GPesos,P$PesoPescado`,
`GPescando,1,,IdPescador,P$NroPescador`, `P$CantTuristas,A_Queue1`,
`X$qValue,Entrada13,A_Queue2`, `FN$ProbCliente`).

Los tests de `crates/genmsi-core/tests/report_gpss.rs` que fijaban la forma
anterior (`P:Cuadra`, `X:qValue`, `FN:ProbCliente`, …) se actualizaron a la
forma del original, que es la que ya citaban en sus comentarios.

### 10. La columna `LINE`

La columna `LINE` de la tabla de bloques **ya** escribía el número de línea
del programa (y no un contador de las sentencias del modelo): el listado de
`gpss.rs` cuenta las líneas de las definiciones y las de las conexiones. Se
verificó con el banco `Enter`, cuyo modelo declara un `STORAGE` (la primera
línea del programa) antes del generador:

```
    2       62      52     0              1  GENERATE        60,30
    3       10       0     0        A_Enter  ENTER           AStorage,1
```

No hubo que cambiar nada; lo que sí se fijó es un test que lo comprueba.

### 11. La columna `BDT` de los eventos pendientes

El original escribe la hora del evento como un entero:

```
FEC_XACT_NUMBER    PRI      BDT    CURRENT   NEXT   PARAMETER   VALUE
            59      0       3624        1       1
```

El nuestro la escribía con decimales (`3603,150`). Ahora la escribe entera
(`event_time`).

**Lo que no se pudo decidir**: el original **no redondea**, sus horas son
enteras porque su generador sortea un intervalo entero (no está en el código
macro, que sólo dice `[Modificador:'Desvio o modificador'='30']`, sino en el
comportamiento medido: todas las horas de todos los reportes del original son
enteras). El nuestro sortea un intervalo real, así que la hora que se escribe
es la más cercana al entero. Si el trabajo paralelo del motor (`sim.rs`)
hace que el generador sortee un intervalo entero, la diferencia desaparece y
esta redondeo no se nota.

## Lo que queda fuera (y por qué)

| # | Qué | Evidencia | Dónde | La prueba que lo fijaría |
|---|---|---|---|---|
| A | **La fila del `TERMINATE`**: el original escribe `TERMINATE 0` cuando el modelo guarda `Unidades = 0` y `TERMINATE` cuando la propiedad está vacía; el nuestro escribe `TERMINATE` en los dos casos. Es la misma pérdida de información que el punto 2 (el programa compilado no guarda el texto de la propiedad) y no estaba en el encargo. | los reportes del original (`ejercicio_3`, `entregaComida`) contra los nuestros | `crates/genmsi-core/src/program.rs` (el texto de la propiedad) | Un test que fije la fila del `TERMINATE` con `Unidades = 0` |
| B | **La fila del `GENERATE`**: el original escribe `GENERATE ,,,1` cuando el modelo no da la `Media` y el nuestro escribe `GENERATE 0,,,1` (el cero es el valor por defecto del compilador). Misma causa que A. | el `monorriel` | `crates/genmsi-core/src/program.rs` | Un test que fije la fila del `GENERATE` sin `Media` |
| C | **La fila de la `QUEUE` y la de la `DEPART`**: el original escribe las unidades cuando el modelo las da (`QUEUE qAlmacen,2`) y no las escribe cuando no (`QUEUE Cola`); como no se puede saber, el reporte las escribe cuando el número **no** es uno. El caso "el modelo da exactamente 1" queda sin decidir. | los reportes del original | `crates/genmsi-core/src/program.rs` | Un banco con `Cantidad = 1` en una `Queue` |
| D | **Las filas de la `TRANSFER` de la conexión**: la tabla de bloques del original incluye las `TRANSFER` que el listado escribe entre las sentencias que rompen el flujo (`   68       12       0     0            48  TRANSFER        ,A_Assign6`); el nuestro no las tiene. Es la diferencia de forma que ya estaba anotada en `notas-reporte-forma.md` (el orden del listado) y depende de `gpss.rs`, que es de otro trabajo. | el `TP10__tp10` | `crates/genmsi-core/src/gpss.rs` · `flow_order` | Un test que compare las filas del `TP10` con las del original |
| E | **Las columnas `PARAMETER` y `VALUE` de los eventos pendientes**: el original escribe los parámetros de la transacción que espera (`  42      0      0     0 7      7   SEMAFORO       39`); el nuestro las deja vacías. No estaba en el encargo. | los reportes del original | `crates/genmsi-core/src/sim.rs` | Un test que fije las columnas de un evento pendiente |
| F | **La sección `DIAGNOSTICOS`**: el compilador nuestro avisa de cosas que el original no dice (el banco `GateS` usa un almacén que ningún `StorageDef` declara) y el original no tiene esa sección. | el banco `GateS` | `crates/genmsi-core/src/program.rs` (los diagnósticos) | — |
| G | **La `MultifurcacionP`**: el original no escribió reporte para ese banco, así que su fila sale del código macro de la clase y no de una medida. | `/tmp/bench-out-new/MultifurcacionP.report.txt` (vacío) | — | Una corrida del original sobre ese banco |

## Qué cambió

- `crates/genmsi-core/src/report.rs`
  - `expr_operand` escribe los operandos en la notación del original (el
    dólar en vez de los dos puntos).
  - `transfer_probability` escribe la probabilidad en miles cuando es un
    número y su texto cuando es una expresión.
  - La fila del `ALTER` escribe los seis operandos, la letra del operador y
    el destino de su salida de fallo cuando está conectada.
  - La fila del `ENTER` y la del `LEAVE` escriben las unidades.
  - La fila de la `MULTITRANSFER` escribe la función de destino
    (`FN$MultifurcacionGO`) y su `BLOCK_TYPE` es `TRANSFER` en las dos clases.
  - La fila del `LOOP` conserva la coma final.
  - La puerta de una etiqueta (`GateM`) escribe `M`/`NM`.
  - La fila de la `LOGIC` escribe el tercer operador de la llave, el que
    invierte su estado (`LOGIC   I       1`). El trabajo del listado
    (`notas-w2-listado.md`) dejó este punto escrito para el dueño de
    `report.rs`, que es este trabajo.
  - La sección `QTABLES` es la de `TABLAS`: la cabecera con los operandos de la
    sentencia y el histograma entero (`write_table_body`).
  - Las secciones de los grupos escriben los grupos que nombran las
    sentencias, aunque no tengan miembros.
  - La hora de los eventos pendientes se escribe entera (`event_time`).
  - La cabecera de la tabla de bloques lleva el espacio que le falta, y la
    columna del límite inferior del intervalo de las tablas es de tres
    columnas con el separador separado por un espacio.
- `crates/genmsi-core/tests/reporte_pendientes.rs` (nuevo): trece tests, uno
  por punto (y dos más: el destino de la salida de fallo del `ALTER` y el
  tercer operador de la llave), con el texto entero de la fila.
- `crates/genmsi-core/tests/report_gpss.rs`: los tests que fijaban la
  notación del modelo (`P:Cuadra`, `X:qValue`, `FN:ProbCliente`, …) se
  actualizaron a la del original.
- `crates/genmsi-cli/tests/snapshots/cli__simulate_report.snap` y
  `crates/genmsi-core/src/snapshots/genmsi_core__report__tests__report_mini.snap`:
  las fotos del reporte cambian con la hora entera y con el espacio de la
  cabecera. (Estos dos ficheros están fuera de las superficies de edición
  del encargo: son las fotos de `insta` de los tests que cambian.)
- La rama incluye un `merge` del trabajo del listado y de los destinos
  (`de74fae`), que llegó a `main` mientras este trabajo corría: el reporte
  tiene que escribir el `ALTER` con su destino de fallo y la `LOGIC` con su
  tercer operador, como hace el listado.

## Lo que se comprobó

- **El oráculo de los bancos**: `cargo run --release -p genmsi-core --example
  compare-benches -- scripts/wine-oracle/benches /tmp/bench-out-new --runs 3`
  da **61 de 62** bancos con todas las métricas dentro de las tolerancias, y
  la salida es **idéntica** a la de `main` (comparada archivo por archivo).
- **El GPSS**: los 62 bancos con respuesta del original
  (`scripts/wine-oracle/compare-gpss.sh`) coinciden: **62 de 62**.
- **Los reportes, fila por fila**: el diferencial contra el reporte del
  original de los 66 bancos (`/tmp/bench-out-new/`) deja **una sola**
  diferencia: la sección `DIAGNOSTICOS` del banco `GateS` (el punto F de la
  tabla de arriba) y el banco `MultifurcacionP`, del que el original no
  escribió reporte. Los reportes de `QTable` y de `Table` salen **idénticos**
  al del original, línea por línea.
- **Los tests**: `cargo test --workspace` en verde, con **389** tests (los 376
  de antes más los trece nuevos).
- **Clippy**: `cargo clippy --workspace --all-targets -- -D warnings` sin
  avisos.

## Tasks

- **T1** — Medir los once puntos contra el original (S1, S2) — hecho: las
  once diferencias están medidas con los reportes del original y con el
  código macro de las clases; las notas de esta tabla.
- **T2** — Las filas que perdían operandos: `ALTER`, `ENTER`/`LEAVE`,
  `TRANSFER`, `MULTITRANSFER`, `LOOP`, `GATE` y `TEST` (S1.1 a S1.6, S1.9) —
  hecho: `crates/genmsi-core/src/report.rs` · `operand_parts`; los tests
  `the_alter_row_writes_the_operator_the_group_the_count_and_the_two_parameters`,
  `the_alter_row_writes_the_destination_of_its_failure_output`,
  `the_enter_and_the_leave_rows_write_the_units_of_the_storage`,
  `the_transfer_row_writes_the_probability_in_thousandths`,
  `the_multifurcation_rows_write_the_function_the_class_creates`,
  `the_loop_row_writes_the_empty_destination_of_the_iteration`,
  `the_gate_of_a_label_writes_the_matching_letter`,
  `the_logic_row_writes_the_third_operator_of_the_switch` y
  `the_test_row_writes_the_parameter_with_the_dollar_of_gpss`.
- **T3** — La sección `QTABLES` y las de los grupos (S1.7, S1.8) — hecho:
  `crates/genmsi-core/src/report.rs` · `render` y `write_table_body`; los
  tests `the_qtables_section_writes_the_histogram_of_the_queue_table` y
  `the_group_sections_write_the_groups_that_the_statements_name`.
- **T4** — La columna `LINE` y la columna `BDT` (S1.10, S1.11) — hecho:
  `crates/genmsi-core/src/report.rs` · `render` y `event_time`; los tests
  `the_line_column_is_the_line_of_the_program` y
  `the_pending_events_write_their_time_as_a_whole_number`.
- **T5** — Comprobar el oráculo, el GPSS, los tests y el clippy (S4, S5) —
  hecho: 61 de 62 bancos, 62 de 62 de GPSS, 383 tests en verde y el clippy
  sin avisos.
- **T6** — Las notas y el commit (S6) — hecho: este archivo y el commit de la
  rama `feature/reporte-pendientes`.
