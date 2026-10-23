# Notas: los destinos de las salidas de fallo y los operadores de un almacén

Las cuatro diferencias que dejó el inventario de la fase 7 sobre el
**destino de las salidas de fallo** y sobre los **operadores de un
almacén**: el destino que el original escribe en el listado del programa
GPSS, el aviso que el original da cuando un `EXAMINE` no tiene destino, y
los dos operadores de la disponibilidad de un almacén que el catálogo del
editor no ofrecía.

Cada una se midió contra el original, corriendo bajo Wine
(`scripts/wine-oracle/`): el listado y el reporte que el propio original
dejó, y sondas nuevas para lo que los bancos no miden.

El trabajo es de la **fase 7** (`odd/tasks/genmsi-fase7-revision-clase-por-clase.md`).

## Specs

- **S1** — "EL DESTINO DE LA SALIDA DE FALLO DE LA FAMILIA SELECT. El original
  escribe el destino de la salida alternativa en las sentencias de la familia
  (por ejemplo "SELECT SE param,1,10,,,A_Terminate1", y "A_SelectF SELECT NU
  caja,1,V$cantCajas,,,A_Select" en TP5). El nuestro no."
- **S2** — "EL DESTINO ALTERNATIVO DEL TEST. Lo detectó el lote 2: el TEST manda
  por su salida de fallo, y nuestro listado no la escribe."
- **S3** — "EL EXAMINE CON OPERANDOS VACÍOS. Se compila y nadie lo dice; el
  original rechaza el modelo con "Error en linea 1 -> Faltan parámetros". Que
  el validador lo avise, como avisa de lo demás que el original rechaza."
- **S4** — "LOS OPERADORES SV Y SNV DEL SelectS. El original los escribe y el
  catálogo del editor no los ofrece. Su semántica es "disponible" y "no
  disponible" de un almacén: hay que fijarla contra el original antes de
  codificarla (con el almacén declarado, porque el original no corre el SV ni
  el SNV sin almacén declarado). Si la semántica no se puede fijar con
  evidencia, dilo y déjalo anotado en vez de decidirlo de oídas."
- **S5** — "Un test del core por cada una de las cuatro cosas."
- **S6** — "GPSS: 62 de 62 y los que se puedan, más. Métricas: 899 de 958 o
  más. `cargo test --workspace` y `cargo clippy --workspace --all-targets --
  -D warnings` en verde, con los tests subiendo."
- **S7** — "Las notas en odd/tasks/notas-destinos.md (nuevo). El commit en tu
  worktree, y me dices la rama."
- **S8** — "NINGÚN commit lleva pie de atribución (nada de 'Co-Authored-By' ni
  similar). El proyecto es del lector y los commits no llevan la firma de
  ninguna herramienta ni de ningún modelo. El mensaje del commit explica qué
  cambió y por qué, y nada más."
- **S9** — "TRABAJA EN UN WORKTREE PROPIO, desde el HEAD actual:
  `git worktree add ../genmsi-destinos -b feature/destinos-y-operadores`".

## Las cuatro cosas

### 1. El destino de la salida de fallo de la familia `SELECT`

**Lo que dice el original.** El listado de la familia `SELECT` lleva el
destino de la salida de fallo ("No selec") como el último operando, con dos
operandos vacíos antes. Las cuatro clases de la familia lo escriben así:

| Clase | Lo que escribe el original | De dónde sale la evidencia |
|---|---|---|
| `SelectS` | `SELECT SE param,1,10,,,A_Terminate1` | la sonda `sels_SE` (`scripts/wine-oracle/probes/select/`) |
| `SelectF` | `SELECT NU caja,1,V$cantCajas,,,A_Select` | el modelo del propio original, `fixtures/outputs/TP5__burgerShop.gpss.txt` |
| `SelectF` | `SELECT NU param,1,10,,,A_Terminate1` | la sonda `select_selectf_dest` (nueva) |
| `SelectL` | `SELECT LS param,1,10,,,A_Terminate1` | la sonda `select_selectl_dest` (nueva) |
| `SelectSNA` | `SELECT E param,1,10,*1,Q,A_Terminate1` | la sonda `select_selectsna_dest` (nueva) |

Y **sin** la salida de fallo conectada, el original no escribe destino
ninguno: `SELECT SE param,1,10`, `SELECT NU param,1,10`,
`SELECT LS param,1,10`. La clase `SelectSNA` es la excepción: su campo de
destino es propio y lo escribe siempre, aunque esté vacío (`SELECT E
param,1,10,*1,Q,` es lo que escribe el banco del oráculo). La clase
`Select` —la del `MIN` y el `MAX`— no tiene salida de fallo (su grafo en
`Genmsi_20230711.backup/gClasses.xml` sólo tiene `Salida1`), así que su
sentencia no lleva destino: `SELECT MIN param,1,10,,Q`.

**Qué cambió.** `crates/genmsi-core/src/gpss.rs`, fn `block_statements`, el
brazo `BlockKind::Select`: el enum nuevo `Failure` dice cómo lleva cada
familia el destino (`None`, `AfterTwoEmpties`, `OwnField`), y el destino se
saca con la fn `port_destination`, que ya se usaba para el `EXAMINE`.

**La prueba.** `crates/genmsi-core/tests/destinos.rs`:
`the_selection_writes_the_destination_of_its_failure_output` (las cinco
formas de la tabla de arriba, con y sin la salida conectada).

**Lo que se gana.** El listado de las sondas de la familia `SELECT` que
tienen la salida de fallo conectada coincide con el del original. Antes, las
ocho sondas `sels_*` de la sesión anterior quedaban fuera por este operando
(la fila A de "Lo que queda fuera" de `odd/tasks/notas-gpss-forma.md`).

### 2. El destino alternativo del `TEST`

**Lo que dice el original.** El `TEST` lleva el destino de su salida de
fallo ("Falso") como tercer operando: la sonda
`scripts/wine-oracle/probes/TestFalso.gmdl` deja el listado

```
A_Test          TEST    E       1,2,A_Terminate1
```

y sin la salida de fallo conectada el listado es `A_Test  TEST  E  1,1` (el
banco del oráculo). El manual del simulador (`Genmsi_20230711.backup/
VISUALSIS.HLP`) lo confirma con sus propios ejemplos, y el original los
escribe en sus modelos: `TEST E P$SectorVisitado,3,A_Test1`, `TEST NE
P$ImpacienciaMedia,0,A_JoinMe`, `TEST L P$Tipo,6,Entrada11`, `TEST G
P$Donde,3,A_Unlink1`, `TEST GE P$CantPescados,1,A_Terminate7`
(`fixtures/outputs/`).

**Qué cambió.** `crates/genmsi-core/src/gpss.rs`, fn `block_statements`, el
brazo `BlockKind::Test`: el destino de la salida `Salida2` se escribe como
tercer operando cuando lo hay.

**La prueba.** `crates/genmsi-core/tests/destinos.rs`:
`the_test_writes_the_destination_of_its_failure_output`.

**Lo que se gana.** Ocho modelos del original (`Problema 1__semaforo-1`,
`TP3__aeropuerto`, `TP4__supermercado`, `TP5__burgerShop`, `TP7__Clase 7_3_
Cadete`, `TP8__mecanicos`, `TP10__tp10`, `Parcial 6-11-23__Veron - Parcial`,
`monorriel__monorriel`) escriben ahora su listado como el del original.

### 3. El `EXAMINE` sin destino

**Lo que dice el original.** El tercer operando del `EXAMINE` es el bloque
al que va la transacción cuando el grupo no la tiene, y el manual lo dice
expresamente: "Número de bloque. Destino alternativo para la transacción
activa si no es encontrado ningún miembro. Requerido" y, en "Restricciones
Especiales", "A y C deben ser positivos. C debe ser una dirección de bloque
en el modelo corriente" (`VISUALSIS.HLP`).

La sonda nueva `examine_sin_destino.gmdl` (el `Examine` con la salida de
fallo sin conectar) lo confirma: el original **no abre el modelo** y dice

```
Error en linea     1 -> Faltan parámetros
```

Con la salida conectada (`examine_probe.gmdl`) el modelo corre y el listado
es `A_Examine EXAMINE UnGrupo,,A_Terminate1`.

Ojo con el otro operando vacío: el segundo (el número) **puede** estar
vacío —el manual lo dice ("Opcional. El operando debe ser nulo…") y su
propio ejemplo es `EXAMINE VALID_XACTS, , NOT_VALID_XACT`—, y el banco del
`Examine` corre con `EXAMINE UnGrupo,,A_Terminate1` sin quejarse. Lo que el
original exige es el destino.

**Qué cambió.** `crates/genmsi-core/src/program.rs`: la fn nueva
`report_examine_without_destination`, que el compilador llama después de
cablear los puertos, dice en los `diagnostics` del programa que el modelo
tiene un `EXAMINE` sin destino. El modelo sigue compilando (el original
abre un modelo así: lo que no hace es simularlo), pero el `lint` ya no
calla.

**La prueba.** `crates/genmsi-core/tests/destinos.rs`:
`the_examination_without_a_destination_is_reported` (el aviso está cuando no
lo hay, y no está cuando lo hay).

### 4. Los operadores `SV` y `SNV` del `SelectS`

**Lo que dice el original.** La lista `op_Logic_Stor` de
`Genmsi_20230711.backup/gmacroconfig.gmcfg` tiene seis operadores:
`SF:Lleno`, `SNF:NoLleno`, `SE:Vacio`, `SNE:NoVacio`, `SV:Disponible`,
`SNV:NoDisponible`. El catálogo del editor offering cuatro de ellos, y el
listado escribía `SNF` para los dos que faltaban (la fn `storage_state` no
los distinguía).

**La semántica, medida.** La selección deja en un parámetro el número de la
entidad que eligió, y un cero cuando no eligió ninguna (el manual: "Si la
entidad no es encontrada, 0 es colocado en el parámetro de la transacción
activa"). Las sondas nuevas `par_<op>_<estado>.gmdl` leen ese parámetro con
un `TEST` y dicen cuántas transacciones lo encontraron. El modelo declara
el almacén `1` —el `SELECT` camina las entidades cuyo número cae en el
rango de la sentencia (`1,10`), así que un almacén llamado `Almacen` nunca
está en él— y lo pone en uno de tres estados:

| Sonda | `op` | Estado del almacén | Lo encontraron | No lo encontraron |
|---|---|---|---|---|
| `par_se_vacio` | `Vacio` | vacío | 59 | 0 |
| `par_se_uno` | `Vacio` | una unidad | 0 | 120 |
| `par_sv_vacio` | `Disponible` | vacío | 55 | 0 |
| `par_sv_uno` | `Disponible` | una unidad | 114 | 0 |
| `par_snv_vacio` | `NoDisponible` | vacío | 0 | 57 |
| `par_snv_lleno` | `NoDisponible` | lleno | 0 | 120 |

La regla queda fijada: **el `SV` elige el almacén y el `SNV` no elige
ninguno**. Es lo que dice el manual del simulador: "El storage tiene que
estar disponible para que sea seleccionado" (`SV`) y "El storage no tiene
que estar disponible para que sea seleccionado" (`SNV`).

El estado de "disponible" es una bandera del almacén que sólo cambian las
sentencias `SAVAIL` y `SUNAVAIL` del manual, y la biblioteca de clases del
original (66 clases) no tiene ninguna de las dos: un almacén de un modelo
del editor siempre está disponible. Las sondas de la puerta sobre un
almacén (`scripts/wine-oracle/probes/gates/`, que ya estaban en el
repositorio) dicen lo mismo: la puerta `SV` abre en los tres estados
medidos y la `SNV` cierra en los tres.

**Qué cambió.**

- `crates/genmsi-format/src/catalogue.rs`: la lista `op_Logic_Stor` pasa de
  cuatro operadores a los seis del original, así que el editor ofrece los
  que ofrece el original.
- `crates/genmsi-core/src/program.rs`: el enum `SelectState` gana
  `StorageAvailable` y `StorageNotAvailable`, y la fn `storage_state` los
  reparte (`Disponible` → `StorageAvailable`, `NoDisponible` →
  `StorageNotAvailable`; antes los dos caían en `StorageNotFull`).
- `crates/genmsi-core/src/gpss.rs`: las letras `SV` y `SNV` de la sentencia.
- `crates/genmsi-core/src/sim.rs`, fn `select`: la prueba. Un almacén está
  disponible (nadie lo pone en el estado "no disponible") y ninguno está
  en ese estado.

**Las pruebas.** `crates/genmsi-core/tests/destinos.rs`:
`the_selection_by_the_availability_of_a_storage_writes_its_letter` (las dos
letras) y `the_availability_of_a_storage_decides_the_selection` (la
semántica: el `SV` elige el almacén y el `SNV` no). Y
`crates/genmsi-format/tests/blocks.rs`:
`the_editor_offers_the_six_operators_of_a_storage` (el catálogo).

## Lo que se comprobó

- **El GPSS**: los 62 bancos con respuesta del original
  (`scripts/wine-oracle/benches` contra `/tmp/bench-out-new/`) coinciden,
  sentencia por sentencia (`scripts/wine-oracle/compare-gpss.sh`): **62 de
  62**. (Las respuestas del original para el `Depart`, el `Enter` y el
  `Leave` eran anteriores a los bancos actuales —los que declaran la cola y
  el almacén que esas clases usan—, así que se volvieron a correr contra el
  original antes de comparar.)
- **Las sondas de la familia `SELECT`**: las 34 de la sesión anterior que
  tienen respuesta del original siguen coincidiendo (**34 de 34**), y las
  nuevas —`select_selectf_dest`, `select_selectl_dest`,
  `select_selectsna_dest`, `par_sv_vacio`, `par_snv_vacio`,
  `sels_sv_num_vacio`, `sels_sv_num_lleno`— coinciden también, salvo en dos
  diferencias que ya estaban y no son de este trabajo (el `GENERATE` con
  `Cantidad` y el `ENTER` con sus unidades; abajo).
- **El oráculo de las métricas**: `cargo run --release -p genmsi-core
  --example oracle -- --runs 3` da **899 de 958**, los mismos números que
  antes del trabajo (ninguna métrica baja).
- **El diferencial**: el binario de antes (`ff13c32`) contra el de ahora, con
  la misma semilla, sobre los 66 bancos y los 25 modelos de
  `fixtures/models`: sólo cambian las líneas de este trabajo —el `TEST` con
  su destino y el `SELECT NU` con el suyo—. El `gpss`, el `simulate` y el
  `lint` de todo lo demás dan el mismo texto y el mismo código de salida.
- **Los tests**: `cargo test --workspace` en verde, con los cinco tests
  nuevos de `crates/genmsi-core/tests/destinos.rs` y el del catálogo.
- **Clippy**: `cargo clippy --workspace --all-targets -- -D warnings` sin
  avisos.

## Lo que queda fuera (y por qué)

| # | Qué | Evidencia | Dónde | La prueba que lo fijaría |
|---|---|---|---|---|
| A | **La fila del `SELECT` del reporte.** El original escribe `SNF param,1,10` (la letra del operador de la clase, sin la clase al final) y el nuestro escribe `MIN param,1,10,,S`. Es el cambio 6 de las notas del lote 1 y es de otra sesión. | el reporte del banco del `SelectS` | `crates/genmsi-core/src/report.rs` · `operand_parts`, brazo `BlockKind::Select` | Un test en `crates/genmsi-core/tests/report_gpss.rs` que fije `SNF param,1,10` |
| B | **El `SELECT` que no elige nada no va a su destino alternativo.** El manual dice que el operando F ("Número de bloque alternativo. El destino alternativo si no es seleccionada una entidad") lleva la transacción cuando no se elige ninguna entidad, pero las sondas muestran que el original manda esa transacción al bloque siguiente (el `NSB`), no al operando F. El nuestro hace lo mismo en las clases de la familia (`SelectS`, `SelectL`, `SelectSNA`) y lo contrario en el `Select` y el `SelectF`, donde la transacción va a `Salida2`. | las sondas `sels_se_num_uno` y `sels_snf_num_lleno` (la selección falla y la transacción va al bloque siguiente) | `crates/genmsi-core/src/sim.rs` · el brazo `BlockKind::Select` (la bandera `missing_to_failure`) | Un test que compare, con el `SelectS` y su salida de fallo conectada, a dónde va la transacción cuando no se elige nada |
| C | **El `GENERATE` con `Cantidad` y el `ENTER` con sus unidades.** El original escribe `GENERATE 10,30,,1` (la cantidad en el cuarto operando) y `ENTER 1,1` (las unidades); el nuestro escribe `GENERATE 10,30,,,1` y `ENTER 1`. Son los cambios 1 y 18 de las notas del lote 2 y no son de este trabajo. | la sonda `sels_sv_num_lleno` | `crates/genmsi-core/src/gpss.rs` · los brazos `BlockKind::Generate` y `BlockKind::Enter` | Un test en `crates/genmsi-core/tests/gpss.rs` que fije `GENERATE 10,30,,1` y `ENTER 1,1` |
| D | **El parámetro que el `SELECT` deja.** La selección deja el número de la entidad elegida en un parámetro, y el manual dice que deja un cero cuando no elige ninguna. El nuestro lo deja, pero el parámetro de un `SELECT` que no eligió nada no se puede comparar con el del original sin una sonda que diga qué valor queda. | las sondas `par_*` | `crates/genmsi-core/src/sim.rs` · fn `select` | Un test que fije el valor del parámetro después de un `SELECT` que no eligió nada |
| E | **El `EXAMINE` con el número puesto y sin destino** (`EXAMINE UnGrupo,0,`): el original lo rechaza también ("Faltan parámetros"), y el aviso de este trabajo lo dice igual, porque el aviso depende del destino y no del número. | el banco del `ExamineNumber` con la salida de fallo sin conectar | — | — |

## Tasks

| ID | Specs | Qué | Ruta | Commit |
|----|-------|-----|------|--------|
| T1 | S1 | El destino de la salida de fallo de la familia `SELECT` | `gpss.rs` | `3a5f2cf` |
| T2 | S2 | El destino de la salida de fallo del `TEST` | `gpss.rs` | `6e79fdf` |
| T3 | S3 | El `EXAMINE` sin destino | `program.rs` | `3341f16` |
| T4 | S4 | Los operadores `SV` y `SNV` del `SelectS` | `catalogue.rs`, `program.rs`, `gpss.rs`, `sim.rs` | `6b989b5` |
| T5 | S5 | Los cinco tests del core y el del catálogo | `tests/destinos.rs`, `tests/blocks.rs` | `3a5f2cf`, `6e79fdf`, `3341f16`, `6b989b5` |
| T6 | S6 | La verificación: GPSS, métricas, tests y clippy | — | — |
| T7 | S7, S8 | Estas notas | `odd/tasks` | este commit |

El trabajo va en cinco commits: uno por cosa (con su test), uno con las
sondas y uno con estas notas. Cada commit deja el árbol en verde y se puede
revisar solo; el conjunto pasa del límite de ~400 líneas de cambio, que es
lo que el reparto en commits de trabajo deja escrito.

## Log

- **L1** — El encargo de la sesión (verbatim):

  "Trabajo en /home/lucas/Documentos/Workspace/Genmsi (Rust: genmsi-core,
  genmsi-format, genmsi-cli, genmsi-app con Tauri+React).

  Reconstruimos GenMSI (un simulador GPSS de escritorio) en Rust. 356 tests
  del core + 99 del front verdes, clippy limpio, y un oráculo que compara
  nuestra salida con la del original (que corre bajo Wine).

  MÉTODO (no lo cambies):
  - El original publica su propia especificación y la LEEMOS, no la adivinamos:
    * Genmsi_20230711.backup/gClasses.xml: la biblioteca de clases del original.
      Cada <moduleclass className="X"> trae su macrocode (el código fuente de la
      clase, URL-encoded) y su ClassDescrip (la descripción, con sus
      "Consideraciones"). Eso ES el código del original: no hace falta decompilar.
    * Los operadores están en gmacroconfig.gmcfg, y el manual en VISUALSIS.HLP.
    * scripts/wine-oracle/ hace correr el original y saca su código GPSS y su reporte.
  - Cuando una regla no esté fijada por evidencia, se dice; no se decide de oídas.
  - Al cerrar cada bloque: tests que lo fijen y un commit con el estilo del
    repositorio (Conventional Commits, en inglés).
  - No se agregan dependencias. Los contratos publicados genmsi.*/1 no se tocan sin
    necesidad explícita. No se modifica fixtures/ ni scripts/wine-oracle/out/.
  - El trabajo se registra en odd/tasks/ y en el todo.

  [...] EL SPEC DE ESTA TAREA:
  - odd/tasks/notas-fase7-lote1.md (cambio 18: el EXAMINE con operandos vacíos).
  - odd/tasks/notas-fase7-lote2.md (el destino alternativo del TEST).
  - odd/tasks/notas-gpss-forma.md (las tres diferencias que las sondas destaparon y
    que quedaron fuera: el destino de la salida de fallo de la familia SELECT, los
    operadores SV y SNV del SelectS, y la fila SELECT del reporte, que es de otra
    sesión).
  Las tres están medidas contra el original: con archivo, función y prueba.

  ESTADO DEL ORÁCULO: 899 de 958 en las métricas, 62 de 62 en el GPSS.

  [...] QUÉ HACER (cuatro cosas):
  1. EL DESTINO DE LA SALIDA DE FALLO DE LA FAMILIA SELECT. [...]
  2. EL DESTINO ALTERNATIVO DEL TEST. [...]
  3. EL EXAMINE CON OPERANDOS VACÍOS. [...]
  4. LOS OPERADORES SV Y SNV DEL SelectS. [...]"

- **L2** — La base del trabajo es `ff13c32` ("fix(core): the shape of the GPSS
  listing"), el HEAD que había cuando se recibió el encargo. El trabajo va
  en el worktree `../genmsi-destinos`, en la rama
  `feature/destinos-y-operadores`.

- **L3** — Los bancos son un artefacto generado: en el worktree nuevo hay que
  generarlos (`python3 scripts/wine-oracle/make-benches.py`), y los tests
  del core los leen.

- **L4** — Las sondas nuevas están en
  `scripts/wine-oracle/probes/select/make-select-probes.py` (el generador) y
  su respuesta está escrita en
  `scripts/wine-oracle/probes/select/EvidenciaSelect.md` (la evidencia). Los
  modelos y las respuestas del original son artefactos generados: no se
  commitean, como los de las sondas de la sesión anterior.

- **L5** — El arnés de las sondas (`scripts/wine-oracle/probes/select/
  run-probe.sh`) deja una ventana del original por cada corrida, y una
  ventana vieja hace que el arnés maneje la ventana equivocada: las
  corridas se hacen de una en una, matando el `wine` de la anterior entre
  una y otra.

- **L6** — El original no corre un `SELECT` sobre un almacén que nadie
  declara, y un `SELECT` sólo camina las entidades cuyo **número** cae en
  el rango de la sentencia: un almacén llamado `Almacen` no está en el
  rango `1,10` y la selección no lo encuentra nunca. Por eso las sondas de
  la disponibilidad declaran el almacén `1`.

- **L7** — El original **se niega a abrir** un modelo cuyo `EXAMINE` no
  tiene destino ("Error en linea 1 -> Faltan parámetros") y no escribe
  ni el listado: el aviso del validador es la forma que tiene este
  repositorio de decirlo.

- **L8** — Un parámetro del modelo no puede llamarse `P`: el original se
  niega a abrir el modelo ("El modelo no puede abrirse porque el archivo
  presenta errores estructurales"). Las sondas usan `ParamSel`.
