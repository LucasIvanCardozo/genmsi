# Notas: la forma del listado del GPSS y de la tabla de bloques

Las cinco diferencias del **listado** del programa GPSS y del **reporte** que
quedaron del inventario de la fase 7 (los cambios 2, 3, 12 y 20 de la lista
de `odd/tasks/notas-fase7-lote1.md`, más el que destapó el generador de
bancos). Cada una se cambió con la respuesta del original delante: la
respuesta del `genmsi.exe` original bajo Wine (`scripts/wine-oracle/`), los
listados y los reportes que el propio original dejó en `fixtures/outputs/` y
en `/tmp/bench-out-new/`.

El trabajo es de la **fase 7** (`odd/tasks/genmsi-fase7-revision-clase-por-clase.md`).

## Specs

- **S1** — "EL DESTINO ALTERNATIVO DEL EXAMINE. Nuestro listado no lo escribe
  (el reporte sí, y el original lo exige: sin el, rechaza el modelo con
  'Faltan parámetros')."
- **S2** — "LA LETRA DE LA PUERTA LÓGICA. Escribimos 'GATE L' donde el
  original escribe 'GATE LR' (fixtures/outputs/Problema_1__semaforo-1.gpss.txt,
  línea 71)."
- **S3** — "LA SELECCIÓN POR EL ESTADO DE UN ALMACÉN. storage_state mapea
  Vacio y NoVacio al mismo StorageNotFull, y SelectState no tiene estados para
  'vacío' y 'no vacío'. El original escribe 'SELECT SE param,1,10' y 'SELECT
  SNE param,1,10'."
- **S4** — "EL COUNTER PRODUCE DOS SENTENCIAS. SAVEVALUE + ASSIGN: el original
  escribe las dos y la tabla de bloques tiene una línea más (el encabezado dice
  'BLOCKS 5' donde el original tiene 6). El reporte lista las dos."
- **S5** — "LA LISTA DE CLASES DE ENTIDAD DEL SELECT. Hoy el SELECT sólo
  admite tres clases de las 38 de entitySNAclass; cualquier otra hace fallar
  la compilación con 'unsupported SELECT entity class'. El original las admite
  todas, y escribe el verbo que le toca a cada una ('SELECT MIN param,1,10,,F'
  para la clase F)."
- **S6** — "Un test del core por cada uno de los cinco cambios."
- **S7** — "GPSS: 62 de 62 (o más, si algún banco nuevo queda comparable).
  Métricas: 899 de 958 o más. `cargo test --workspace` y `cargo clippy
  --workspace --all-targets -- -D warnings` en verde, con los tests subiendo."
- **S8** — "Las notas en odd/tasks/notas-gpss-forma.md (nuevo). El commit en
  tu worktree, y me dices la rama."
- **S9** — "NINGÚN commit lleva pie de atribución (nada de 'Co-Authored-By' ni
  similar). El proyecto es del lector y los commits no llevan la firma de
  ninguna herramienta ni de ningún modelo. El mensaje del commit explica qué
  cambió y por qué, y nada más."

## Los cinco cambios

### 1. El destino alternativo del `EXAMINE`

**Lo que dice el original.** La sentencia `EXAMINE` de GPSS lleva tres
operandos: el grupo, el número (vacío en modo transacción) y **el bloque al
que va la transacción cuando no está en el grupo**. El manual del simulador
(`Genmsi_20230711.backup/VISUALSIS.HLP`) lo dice: "Número de bloque. Destino
alternativo para la transacción activa si no es encontrado ningún miembro.
Requerido". Y el macrocode de la clase (`gClasses.xml`, clase `Examine`) lo
toma de su salida de fallo:

```
EXAMINEME	[Grupo:'Grupo de transacciones a examinar'='UnGrupo'],...
 	  	Salida2:'No en [Grupo]'
```

**La evidencia.** El listado del original para el banco del `Examine` (el
generador de bancos conecta su salida de fallo, `odd/tasks/notas-bancos.md`):

```
                GENERATE        60,30
A_Examine       EXAMINE         UnGrupo,,A_Terminate1
A_Terminate     TERMINATE
A_Terminate1    TERMINATE
                GENERATE        3600
                TERMINATE       1
```

(`/tmp/bench-out-new/Examine.gpss.txt`.) Y el modelo de verdad del propio
original (`fixtures/outputs/TP10__tp10.gpss.txt`): `A_ExamineNumber  EXAMINE
GPesos,P$PesoPescado,A_Assign7`.

**Qué cambió.** `crates/genmsi-core/src/gpss.rs`, fn `block_statements`,
brazo `BlockKind::Examine`: el tercer operando es el nombre de la sentencia
del bloque al que va la salida de fallo. La fn nueva `port_destination`
(busca el puerto por su nombre y devuelve el nombre de la sentencia de su
destino) hace lo mismo que la rama de `TRANSFER`, que ya lo escribía.

**La prueba.** `crates/genmsi-core/tests/gpss_forma.rs`:
`the_examine_writes_the_alternative_destination`. El test anterior de
`crates/genmsi-core/tests/gpss.rs` (`the_blocks_write_the_operands_of_their_class`)
fijaba el texto viejo (`EXAMINE UnGrupo,0,`); ahora fija el del original.

**Lo que se gana.** Los dos bancos del `Examine` que quedaban fuera (el
listado difería en un solo operando) vuelven a coincidir: el GPSS pasa de 60
de 62 a **62 de 62**.

### 2. La letra de la puerta lógica

**Lo que dice el original.** La lista `op_Logic_Swich` de
`gmacroconfig.gmcfg` tiene dos operadores: `LS:Set` y `LR:Reset`. El listado
del original los escribe tal cual (`fixtures/outputs/Problema_1__semaforo-1.gpss.txt`,
línea 71):

```
GateL1          GATE    LR      *Cuadra
```

y el mismo modelo, más abajo, la puerta que sí está puesta:

```
GateL           GATE    LS      *Cuadra
```

**Qué cambió.** `crates/genmsi-core/src/gpss.rs`, fn `block_statements`,
brazo `BlockKind::Gate`, rama `GateCondition::Logic`: la letra del estado
deshecho es `LR`, no `L`. (El reporte ya escribía `LR`: `crates/genmsi-core/src/report.rs`,
brazo `BlockKind::Gate`.)

**La prueba.** `crates/genmsi-core/tests/gpss_forma.rs`:
`the_gate_on_a_logic_switch_writes_the_reset_letter` (los dos operadores:
`Set` → `LS`, `Reset` → `LR`).

### 3. La selección por el estado de un almacén

**Lo que dice el original.** La lista `op_Logic_Stor` de
`gmacroconfig.gmcfg` tiene seis operadores —`SF:Lleno`, `SNF:NoLleno`,
`SE:Vacio`, `SNE:NoVacio`, `SV:Disponible`, `SNV:NoDisponible`— y la clase
`SelectS` de `gClasses.xml` toma su `op` de esa lista:

```
SELECTS [op:'Operador Logico de Storage.'='NoLleno'],...
	[Param:'parámetro donde se guarda el valor seleccionado'='param'],...
	[minimo:...='1'],...
	[maximo:...='10'],...
	Salida2:'No selec'
```

**La evidencia.** El listado del original para un `SelectS` con cada
operador (sonda `scripts/wine-oracle/probes/select/`, con el mismo arnés que
`run_oracle.sh`; la tabla completa está en
`scripts/wine-oracle/probes/select/EvidenciaSelect.md`):

| `op` | Lo que escribe el original |
|---|---|
| `Lleno` | `SELECT SF param,1,10` |
| `NoLleno` | `SELECT SNF param,1,10` |
| `Vacio` | `SELECT SE param,1,10` |
| `NoVacio` | `SELECT SNE param,1,10` |
| `Disponible` | `SELECT SV param,1,10` |
| `NoDisponible` | `SELECT SNV param,1,10` |

**Qué cambió.**

- `crates/genmsi-core/src/program.rs`: el enum `SelectState` gana
  `StorageEmpty` y `StorageNotEmpty`, y la fn `storage_state` los reparte:
  `Vacio` → `StorageEmpty`, `NoVacio` → `StorageNotEmpty` (antes `Vacio` caía
  en `StorageNotFull`, el mismo estado que `NoLleno`), `Lleno` →
  `StorageFull`, y el resto → `StorageNotFull`.
- `crates/genmsi-core/src/gpss.rs`: las letras `SE` y `SNE` de la sentencia.
- `crates/genmsi-core/src/sim.rs`, fn `select`: la prueba. `StorageEmpty` es
  un almacén sin unidades (y un almacén que nadie declara está vacío), y
  `StorageNotEmpty` uno con unidades.

**La prueba.** `crates/genmsi-core/tests/gpss_forma.rs`:
`the_selection_by_the_state_of_a_storage_writes_its_letter` (los cuatro
operadores del catálogo).

### 4. El `COUNTER` produce dos sentencias

**Lo que dice el original.** El macrocode de la clase `Counter`
(`gClasses.xml`) es un `SV` y un `ASSI`:

```
SV	[SaveValue:'SaveValueUtilizado'='Count'],+,[Incremento:'Valor incremental de conteo'='1']
ASSI	[Param:'Parmetro de conteo'='Nro'],...
 	=,...
        X:[SaveValue]
```

**La evidencia.** El reporte del original del banco del `Counter`
(`/tmp/bench-out-new/Counter.report.txt`):

```
    2       62       0     0      A_Counter  SAVEVALUE       Count+,1
    3       62       0     0              3  ASSIGN          Nro,X$Count
```

El encabezado del mismo reporte dice `BLOCKS 6` donde el modelo tiene cinco
bloques: el contador son dos.

**Qué cambió.** `crates/genmsi-core/src/report.rs`: la fn nueva
`block_rows` arma las filas de la tabla de bloques, y el contador toma dos
filas —la del `SAVEVALUE` (con el nombre del bloque) y la del `ASSIGN` (sin
nombre: la del `ASSIGN` que el original escribe)— con los mismos números. La
columna `LINE` cuenta las filas y el encabezado `BLOCKS` las cuenta también.

**La prueba.** `crates/genmsi-core/tests/gpss_forma.rs`:
`the_counter_takes_two_rows_in_the_block_table`.

**Lo que se gana.** El reporte del banco del `Counter` tiene las seis filas
del original y su encabezado dice `BLOCKS 6`. Los modelos de verdad que
tienen un contador (`Problema 1__semaforo-1`, `TP8__mecanicos`, `TP10__tp10`)
también: su tabla de bloques tiene una fila más, como la del original.

### 5. La lista de clases de entidad del `SELECT`

**Lo que dice el original.** La lista `entitySNAclass` de
`gmacroconfig.gmcfg` tiene 38 clases, y cada una nombra un atributo de una
entidad: `Q` (ocupación actual de una cola), `S` (ocupación actual de un
almacén), `R` (ocupación remanente de un almacén), `F` (ocupación actual de
un recurso), `LS`/`LR` (la llave lógica), `X` (valor de un savevalue), y así
las 38. La clase `Select` de `gClasses.xml` toma de esa lista su propiedad
`ClaseSNA`:

```
SELECT  [op:'Operador Máximo o Minimo.'='Minimo'],...
	[Param:...='param'],...
	[minimo:...='1'],...
	[maximo:...='10'],...
	[ClaseSNA:'Clase de SNA de entidad por el cual se realiza la seleccion'='Q']
```

**La evidencia.** El listado del original para un `Select` de cada clase
(sonda `scripts/wine-oracle/probes/select/`, con la tabla de las 38 en
`scripts/wine-oracle/probes/select/EvidenciaSelect.md`): el verbo es el que
dice la propiedad `op` del módulo (`MIN` por defecto) y la clase es el último
operando, en las 38:

```
A_Select1       SELECT   MIN     param,1,10,,F
```

Nuestro listado coincide con el del original en las 34 sondas de las que el
original dejó su listado (comparador de `scripts/wine-oracle/compare-gpss.sh`):
**34 de 34**.

**Qué cambió.**

- `crates/genmsi-format/src/catalogue.rs`: la lista `entitySNAclass` del
  catálogo pasa de tres clases (`Q`, `S`, `R`) a las 38 del original, así
  que el editor ofrece las que ofrece el original.
- `crates/genmsi-core/src/program.rs`: el compilador ya no se niega ("unsupported
  SELECT entity class") con ninguna de ellas. El enum nuevo `SelectEntity` dice
  qué entidad camina cada familia (colas, almacenes, recursos y llaves
  lógicas; el resto —las cadenas, los grupos, las etiquetas, los parámetros,
  los savevalues, las variables, las funciones y las semillas— las nombra y las
  escribe, pero el motor todavía no las camina: la selección no encuentra
  nada y la transacción sigue por la salida que corresponde).
- `crates/genmsi-core/src/gpss.rs` y `crates/genmsi-core/src/report.rs`: la
  clase es el último operando de la sentencia (`SELECT MIN param,1,10,,F`).

**La prueba.** `crates/genmsi-core/tests/gpss_forma.rs`:
`the_select_writes_the_class_of_the_entity`: la clase `F` y la clase `Q`, y
las 38 clases una por una (todas compilan y todas escriben su letra).

## Lo que se comprobó

- **El GPSS**: los 62 bancos con respuesta del original
  (`/tmp/bench-out-new/`) coinciden, sentencia por sentencia
  (`scripts/wine-oracle/compare-gpss.sh`): **62 de 62**. Antes de este trabajo
  eran 60 de 62 (los dos del `Examine`).
- **El diferencial**: el binario de antes (`47d0b8f`) contra el de ahora, con
  la misma semilla, sobre los 66 bancos y los 25 modelos de `fixtures/models`:
  sólo cambian las líneas de este trabajo —el `EXAMINE` con su destino, el
  `GATE LR` y la fila del `ASSIGN` del contador—. Todo lo demás da el mismo
  texto.
- **El oráculo de las métricas**: `cargo run --release -p genmsi-core --example
  oracle -- --runs 3` da **899 de 958**, los mismos números que antes del
  trabajo (ninguna métrica baja). Los bancos, con el comparador del
  repositorio (`scripts/wine-oracle/compare-benches.sh`): 58 con reporte del
  original, 56 con todas sus métricas dentro de las tolerancias.
- **Las sondas**: con los 34 modelos de `scripts/wine-oracle/probes/select/`
  de los que el original dejó su listado, el nuestro coincide sentencia por
  sentencia: **34 de 34** (`scripts/wine-oracle/compare-gpss.sh`). Los ocho
  que quedan (los `sels_*` con la salida de fallo conectada) difieren en el
  destino: es lo que queda anotado abajo, en la fila A.
- **Los tests**: `cargo test --workspace` en verde, con los tests del core
  subiendo (los cinco nuevos de `crates/genmsi-core/tests/gpss_forma.rs`).
- **Clippy**: `cargo clippy --workspace --all-targets -- -D warnings` sin
  avisos.

## Lo que queda fuera (y por qué)

Las tres diferencias siguientes se midieron con las mismas sondas, pero no son
de las cinco de este encargo: quedan escritas con su evidencia, su archivo y
la prueba que las fijaría, como las de las notas del lote 1.

| # | Qué | Evidencia | Dónde | La prueba que lo fijaría |
|---|---|---|---|---|
| A | **El destino de la salida de fallo en las sentencias de la familia `SELECT`.** El original lo escribe como último operando, con dos operandos vacíos antes: `SELECT SE param,1,10,,,A_Terminate1` (la sonda `sels_SE`) y `A_SelectF  SELECT  NU  caja,1,V$cantCajas,,,A_Select` (el modelo del propio original, `fixtures/outputs/TP5__burgerShop.gpss.txt`). El nuestro no lo escribe. | sondas `sels_*` y el modelo `TP5` | `crates/genmsi-core/src/gpss.rs` · `block_statements`, brazo `BlockKind::Select` | Un test con un `SelectS` (o un `SelectF`) cuya salida de fallo esté conectada, que fije la sentencia completa. |
| B | **Los operadores `SV` y `SNV` de la lista `op_Logic_Stor`.** El original los escribe en el `SELECT` (`SELECT SV param,1,10`, `SELECT SNV param,1,10`); el nuestro escribe `SNF`, porque `storage_state` no los distingue. (El catálogo del editor sólo ofrece los cuatro operadores medidos en el cambio 3, así que un modelo tiene que llevar el `op` a mano para llegar a `SV`.) | sondas `sels_SV` y `sels_SNV` | `crates/genmsi-core/src/program.rs` · `storage_state` y el enum `SelectState`; `crates/genmsi-core/src/gpss.rs` (las letras); `crates/genmsi-core/src/sim.rs` · `select` (qué es "disponible") | Dos tests de GPSS: un `SelectS` con `op` `Disponible` → `SELECT SV param,1,10`, y otro con `NoDisponible` → `SELECT SNV param,1,10`. |
| C | **La fila del `SELECT` del reporte.** El original escribe `SNF param,1,10` (la letra del operador de la clase, sin la clase al final) y el nuestro escribe `MIN param,1,10,,S`. | el reporte del banco del `SelectS` (`/tmp/bench-out-new/SelectS.report.txt`) | `crates/genmsi-core/src/report.rs` · `operand_parts`, brazo `BlockKind::Select` | Es el cambio 6 de las notas del lote 1: un test en `crates/genmsi-core/tests/report_gpss.rs` que fije `SNF param,1,10`. |

- **La fila del `SELECT` del reporte** es, además, el cambio 6 de las notas
  del lote 1: no es de esta tanda porque es la forma del reporte, no del
  listado.
- **La columna `LOC` del reporte.** El original numera las filas sin nombre
  con su número de línea y el nuestro con un contador de filas sin nombre
  (las diferencias son de siempre: el cambio 4 y el 19 de las notas del lote
  1). Lo que cambia con este trabajo es que el contador consume un número
  para la fila del `ASSIGN` del contador.
- **El motor no camina todas las familias de la lista de clases.** Como se
  dice en el cambio 5, el motor camina las colas, los almacenes, los recursos
  y las llaves lógicas; para las demás familias la selección no encuentra
  nada. La regla de qué métrica escoge el original para cada clase (el
  contenido de la cola, el promedio de entradas, el tiempo de uso...) no está
  fijada por la evidencia de esta tanda.
- **Los operadores `SV` y `SNV` de la puerta sobre un almacén.** El
  `SelectS` los nombra en su lista, pero el catálogo del editor sólo ofrece
  los cuatro que se midieron (`Vacio`, `NoVacio`, `Lleno`, `NoLleno`).

## Tasks

| ID | Specs | Qué | Ruta | Commit |
|----|-------|-----|------|--------|
| T1 | S1 | El destino alternativo del `EXAMINE` en el listado | `gpss.rs` | `2acde39` |
| T2 | S2 | La letra `LR` de la puerta lógica | `gpss.rs` | `2acde39` |
| T3 | S3 | Los estados "vacío" y "no vacío" de un almacén | `program.rs`, `gpss.rs`, `sim.rs` | `2acde39` |
| T4 | S4 | El contador toma dos filas en la tabla de bloques | `report.rs` | `8705d07` |
| T5 | S5 | Las 38 clases de entidad del `SELECT` | `catalogue.rs`, `program.rs`, `gpss.rs`, `report.rs`, `sim.rs` | `2acde39` |
| T6 | S6 | Los cinco tests del core | `tests/gpss_forma.rs` | `2acde39`, `8705d07` |
| T7 | S7 | La verificación: GPSS, métricas, tests y clippy | — | — |
| T8 | S8, S9 | Estas notas | `odd/tasks` | este commit |

El trabajo va en tres commits: el del listado (las cuatro diferencias del
listado y sus tests), el del reporte (las dos filas del contador) y el de las
sondas, que son la evidencia de los dos últimos.

## Log

- **L1** — El encargo de la sesión (verbatim):

  "Trabajo en /home/lucas/Documentos/Workspace/Genmsi (Rust: genmsi-core,
  genmsi-format, genmsi-cli, genmsi-app con Tauri+React).

  Reconstruimos GenMSI (un simulador GPSS de escritorio) en Rust. 346 tests del
  core + 99 del front verdes, clippy limpio, y un oráculo que compara nuestra
  salida con la del original (que corre bajo Wine).

  MÉTODO (no lo cambies):
  - El original publica su propia especificación y la LEEMOS, no la adivinamos:
    * Genmsi_20230711.backup/gClasses.xml: la biblioteca de clases del original.
      Cada <moduleclass className="X"> trae su macrocode (el código fuente de la
      clase, URL-encoded) y su ClassDescrip (la descripción, con sus
      "Consideraciones"). Eso ES el código del original: no hace falta decompilar.
    * Los operadores están en gmacroconfig.gmcfg.
    * scripts/wine-oracle/ hace correr el original y saca su código GPSS y su reporte.
    * scripts/wine-oracle/compare-gpss.sh compara nuestro GPSS con el del original.
  - Cuando una regla no esté fijada por evidencia, se dice; no se decide de oídas.
  - Al cerrar cada bloque: tests que lo fijen y un commit con el estilo del
    repositorio (Conventional Commits, en inglés).
  - No se agregan dependencias. Los contratos publicados genmsi.*/1 no se tocan sin
    necesidad explícita. No se modifica fixtures/ ni scripts/wine-oracle/out/.
  - El trabajo se registra en odd/tasks/ y en el todo.

  [...] TRABAJA EN UN WORKTREE PROPIO, desde el HEAD actual:
     git worktree add ../genmsi-gpss -b feature/gpss-forma
     cd ../genmsi-gpss
  [...] QUÉ HACER (cinco diferencias del LISTADO del GPSS y del programa):
  1. EL DESTINO ALTERNATIVO DEL EXAMINE. [...]
  2. LA LETRA DE LA PUERTA LÓGICA. [...]
  3. LA SELECCIÓN POR EL ESTADO DE UN ALMACÉN. [...]
  4. EL COUNTER PRODUCE DOS SENTENCIAS. [...]
  5. LA LISTA DE CLASES DE ENTIDAD DEL SELECT. [...]"

- **L2** — La base del trabajo es `47d0b8f` ("fix(core): the six behaviour
  differences of the first lot of phase seven"), el HEAD que había cuando se
  recibió el encargo. Los bancos son un artefacto generado: en el worktree
  nuevo hay que generarlos (`python3 scripts/wine-oracle/make-benches.py`), y
  las respuestas del original son las de `/tmp/bench-out-new/`.

- **L3** — La evidencia del `EXAMINE` y la del `COUNTER` no hizo falta
  sonda: están en las respuestas del original que el generador de bancos ya
  había dejado (`/tmp/bench-out-new/Examine.gpss.txt` y
  `/tmp/bench-out-new/Counter.report.txt`). La de la letra `LR` está en el
  propio modelo del original (`fixtures/outputs/Problema_1__semaforo-1.gpss.txt`).

- **L4** — La lista de 38 clases sale de `gmacroconfig.gmcfg`
  (`listedname="entitySNAclass"`), y el significado de cada una, de la
  sección `gpsssnas` del mismo archivo (que dice, por ejemplo, `F` =
  "Ocupación actual de una Facility" y `R` = "Ocupación remanente de un
  Sotrage").

- **L5** — Las sondas de la familia `SELECT` y del `EXAMINE` están en
  `scripts/wine-oracle/probes/select/`: el generador de los modelos
  (`make-select-probes.py`, que usa el catálogo del propio original), el
  arnés con capturas (`run-probe.sh`) y la evidencia escrita
  (`EvidenciaSelect.md`). Los modelos se generan y no se commitean (como los
  bancos); lo que se commitea es el generador y las respuestas del original.

- **L6** — Un hallazgo del propio original, que salió con las sondas: para
  algunas clases de la lista el `genmsi.exe` **se cae** (un acceso inválido)
  en cuanto una transacción llega al `SELECT`. Por eso el generador hace dos
  modelos por clase: uno con el `SELECT` en el flujo y otro con él fuera del
  flujo (el listado se escribe antes de la simulación, así que la misma
  sentencia se mide igual).

- **L7** — El `SV` y el `SNV` del `SelectS` no se arreglaron: el encargo pide
  los cinco cambios de la lista y son los que están. Quedan medidos y
  apuntados arriba (fila B), con el archivo y la prueba.
