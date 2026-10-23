# Notas: el lote 1 de la fase 7, clase por clase

Inventario de las 24 clases más tocadas del original: en qué se parece y en
qué no nuestro motor. No se arregla nada aquí: el mapa de lo que hay que
arreglar, con su archivo y su prueba, y nada más.

## Specs

- **S1** — "EL LOTE (por este orden, que es el que más se ha tocado): StorageDef, StorageUse, FacilityUse, FacilityInt, GateF, GateL, GateS, GateM, Select, SelectF, SelectS, SelectL, SelectSNA, Match, Assemble, Split, Link, Unlink, JoinMe, JoinNumber, RemoveMe, RemoveTrans, Examine, Mark, Counter, Table."
- **S2** — "Anota por clase: los puertos, las propiedades y sus defaults, la semántica (qué hace y con qué regla) y las cifras. Y el veredicto: verificada / con diferencias / pendiente."
- **S3** — "odd/tasks/notas-fase7-lote1.md: una ficha por clase (los cuatro puntos de arriba, el veredicto y la diferencia concreta), una tabla resumen al principio, y una lista final de los cambios pendientes con su archivo y su prueba."
- **S4** — "Si el cambio va en el core, NO lo hagas: déjalo escrito con el archivo, la función y la prueba que lo fijaría." / "Lo único que haces es el mapa y las notas."
- **S5** — "El commit de las notas en tu worktree, y me dices la rama."
- **S6** — "ESTA SESIÓN ES DE INVENTARIO: NO EDITES NADA DE crates/ NI DE docs/. El core lo tiene otra sesión; tú produces el mapa de lo que hay que arreglar."
- **S7** — "El MÉTODO (por clase): 1. El banco de la clase: scripts/wine-oracle/benches/<Clase>.gmdl. ... 2. La respuesta del original: scripts/wine-oracle/out/<Clase>.gpss.txt y <Clase>.report.txt. ... 3. La nuestra: `cargo run --release -p genmsi-cli -- gpss <banco>` y `cargo run --release -p genmsi-cli -- simulate <banco>`. 4. Compara: verbo, operandos y nombre de cada sentencia (scripts/wine-oracle/compare-gpss.sh) y las cifras del reporte (el ejemplo crates/genmsi-core/examples/compare-benches.rs, con las tolerancias del oráculo: 20% en conteos, 35% en tiempos). 5. Lee el macrocode y el ClassDescrip de la clase en Genmsi_20230711.backup/gClasses.xml: ahí está lo que el original dice que hace, con sus \"Consideraciones\"."
- **S8** — "Lo que ya sabemos y no hay que repetir: - 62 de 62 bancos coinciden en el GPSS y 50 de 55 en las cifras (las cinco restantes son bancos cuyo reporte no trae cabecera). - El ALTER alteraba el parámetro que debía comparar: ya está corregido. - La primera transacción de un generador: ya está zanjada (media u offset). - La capacidad de un almacén sin declarar: lo está arreglando otra sesión."

## Resumen

Las 24 clases del lote, en el orden que pedía el encargo. `GPSS` es la
comparación de `scripts/wine-oracle/compare-gpss.sh` (verbo, operandos y nombre
de cada sentencia) y `Cifras` es la de las métricas del reporte del oráculo
(20% en conteos, 35% en tiempos).

| # | Clase | Verbo | GPSS | Cifras | Veredicto | La diferencia, en una línea |
|---|---|---|---|---|---|---|
| 1 | StorageDef | `STORAGE` | coincide | 2/2 (0%) | verificada | sólo de forma: la columna `LOC` y los decimales enteros del reporte |
| 2 | StorageUse | `QUEUE`/`ENTER`/`DEPART`/`ADVANCE`/`LEAVE` | coincide | sin reporte | pendiente | el original se niega a simular un almacén sin declarar (lo arregla `feature/almacen`) |
| 3 | FacilityUse | `QUEUE`/`SEIZE`/`DEPART`/`ADVANCE`/`RELEASE` | coincide | 7/7 | verificada | ninguna de las que este banco mide |
| 4 | FacilityInt | `QUEUE`/`PREEMPT`/`DEPART`/`ADVANCE`/`RETURN` | coincide | 7/7 | con diferencias | la fila `PREEMPT` del reporte pierde los operandos (`PR`, destino, `Remanente`) |
| 5 | GateF | `GATE NU` | coincide | 3/3 | con diferencias | el `LOC` de la puerta: `A_GateF` contra `GateF` |
| 6 | GateL | `GATE LS` | coincide | 1/1 | con diferencias | el `LOC`, los contadores de la puerta (`57 57 57` contra `0 0 0`) y la sección `LOGICSWITCH` que no imprimimos |
| 7 | GateS | `GATE SNF` | coincide | 0/1 (100%) | con diferencias | nuestra puerta pregunta "¿vacío?" donde el original pregunta "¿lleno?" |
| 8 | GateM | `GATE M` | coincide | 1/2 | con diferencias | el reporte escribe `GATE LS` donde el original escribe `GATE M` |
| 9 | Select | `SELECT MIN` | coincide | 32/32 | verificada | sólo de forma; y el `ClaseSNA` sólo admite 3 de las 38 clases |
| 10 | SelectF | `SELECT NU` | coincide | 4/4 | verificada | el banco no mide los otros cinco operadores del recurso |
| 11 | SelectS | `SELECT SNF` | coincide | 2/2 | con diferencias | la fila del `SELECT` dice `MIN param,1,10,,S` donde el original dice `SNF param,1,10` |
| 12 | SelectL | `SELECT LS` | coincide | 2/2 | con diferencias | lo mismo, y falta la sección `LOGICSWITCH` con las diez llaves |
| 13 | SelectSNA | `SELECT E` | coincide | 5/5 | con diferencias | el reporte escribe `SELECT MIN` sin el modo `E` ni el valor comparado `*1` |
| 14 | Match | `TRANSFER`+`MATCH` | coincide | 1/2 | con diferencias | al reporte le falta la línea del `TRANSFER` (5 líneas de bloques contra 6) |
| 15 | Assemble | `ASSEMBLE` | coincide | 2/2 | con diferencias | el `CURRENT` del bloque no baja al completar el conjunto |
| 16 | Split | `SPLIT` | coincide | 1/1 | con diferencias | el `LOC` dice `A_Split` donde el original dice `Clona` |
| 17 | Link | `LINK` | coincide | 1/2 | con diferencias | el `LINK` no suelta la primera transacción al destino (el "indicador link") |
| 18 | Unlink | `UNLINK` | coincide | 2/2 | con diferencias | el `LOC` y el operando dicen `A_Terminate_SalidaAlt` donde el original dice `A_Terminate1` |
| 19 | JoinMe | `JOIN` | coincide | 2/2 | verificada | falta la sección `XACT_GROUP` del reporte |
| 20 | JoinNumber | `JOIN` | coincide | 2/2 | verificada | falta la sección `NUMERIC_GROUP` del reporte |
| 21 | RemoveMe | `REMOVE` | coincide | 2/2 | con diferencias | la fila lleva de más la letra `E` |
| 22 | RemoveTrans | `REMOVE E` | coincide | 2/2 | verificada | el `CompOp` y el modo "Todos" no llegan al GPSS (no los mide este banco) |
| 23 | Examine | `EXAMINE` | coincide | sin reporte | pendiente | el original rechaza el modelo: "Error en linea 1 -> Faltan parámetros" |
| 24 | Mark | `MARK` | coincide | 2/2 | verificada | la hora marcada no se puede comparar (falta la `FEC_XACT_NUMBER`) |
| 25 | Counter | `SAVEVALUE`+`ASSIGN` | coincide | 2/2 | con diferencias | al reporte le falta la línea del `ASSIGN` y el `RETRY` del savevalue |
| 26 | Table | `TABLE` | coincide | 0/0 | con diferencias | las clases de la tabla: el original termina en un desborde `> a 80` |

**Del total:** 24 clases, **24 de 24** con el GPSS igual al del original;
**17** con todas las cifras dentro de las tolerancias; **15** con diferencias,
**7** verificadas y **2** pendientes (las dos porque el original no dejó un
reporte con el que comparar). De las 15 con diferencias, **cuatro son de
comportamiento** (la condición de la `GateS`, el "indicador link" del `LINK`, el
`CURRENT` del `ASSEMBLE` y el `TRANSFER` que el `Match` no expande) y las otras
once son de la forma del reporte o del nombre de los bloques.

### Sondas

Dos cosas no se pueden ver con los bancos del lote (que consultan un almacén
que nadie declara), así que se midieron con dos sondas contra el original
(`scripts/wine-oracle/run_oracle.sh`, el mismo arnés que `run-benches.sh`):

**Sonda 1 — la `GateS` con el almacén declarado.** El banco `GateS.gmdl` más un
bloque `StorageDef` (`Almacen`, `Tamaño 10`):

```
Almacen         STORAGE         10
                GENERATE        60,30
GateS           GATE    SNF     Almacen
A_Terminate     TERMINATE
```

| | puerta | `A_Terminate` |
|---|---|---|
| original | `ENTRY 61  CURRENT 0  RETRY 0` | **61 entradas** |
| nuestro | `ENTRY 61  CURRENT 61  RETRY 61` | 0 entradas |

El original **abre** la puerta con el almacén vacío y declarado (y con el
almacén inexistente: el banco del lote, donde lo abre con 59 entradas). La
nuestra la deja cerrada. La única regla compatible con las dos medidas es
"abre cuando el almacén **no está lleno**".

**Sonda 2 — la `GateS` con el almacén lleno.** El mismo banco con `Tamaño 1` y
un bloque `Enter` (`Almacen`, `Cantidad 1`) entre el `Generate` y la puerta:

```
Almacen         STORAGE         1
                GENERATE        60,30
A_Enter         ENTER           Almacen,1
GateS           GATE    SNF     Almacen
A_Terminate     TERMINATE
```

El original: la fila de la puerta queda `ENTRY 0  CURRENT 0` y el `TERMINATE`
`ENTRY 0` — la puerta **se cierra** cuando el almacén está lleno (el
`ALMACEN` del reporte sale `CAP. 1  REMAIN. 0  MAX. 1  ENTRIES 1  RETRY 62`).
Nuestro: las transacciones se quedan esperando en la puerta.

Las dos sondas dejan la regla del original escrita: la `GateS` con el operador
`NoLleno` abre cuando al almacén le queda sitio (`SNF`), no cuando está vacío.
Nuestro motor la abre cuando el almacén **no está vacío**. Es el cambio de
fondo del lote, y el que la sección "Cambios pendientes", número 1.

## Fichas

Una por clase, en el orden del lote. Cada ficha dice lo que
el original dice de la clase (su `ClassDescrip`, su macrocode y, cuando
aporta, el manual del simulador `VISUALSIS.HLP`), los puertos, las propiedades
con sus defaults, la semántica, el banco, el GPSS, las cifras y el veredicto.

### StorageDef

**El original.** "Delcara un storage." Es una clase de declaración: nombra un
almacén y su capacidad, no tiene puertos y no produce bloques en tiempo de
ejecución.

- **Puertos.** Ninguno: la sección `graph` de la clase en
  `Genmsi_20230711.backup/gClasses.xml` no tiene partes `Input` ni `Output`, y
  `/tmp/fase7/StorageDef/original.txt` lo dice literal: "entradas: [] salidas: []".
  El banco lo repite en la leyenda del módulo, "StorageDef%20%28clase%20sin%20puertos%3A%20no%20se%20puede%20conectar%29"
  (`/tmp/fase7/StorageDef/bench.gmdl`).
- **Propiedades y defaults.**
  - `Storage` — `EntityDef`, subType `Storage` — default `"Almacen"` (`'Storage a
    Declarar'='Almacen'`, macrocode en `original.txt`) — el nombre del almacén
    que se declara.
  - `Tamaño` — `PosInteger` — default `10` (`'Capacidad del Storage'='10'`) — la
    capacidad. El texto de la clase no dice si son unidades o transacciones;
    otra sesión lo midió contra el original: "el `Tamaño` es la capacidad en
    unidades" (`odd/tasks/notas-almacen.md`, rama `feature/almacen`, sección 1).
    Nuestro catálogo trae los mismos defaults
    (`crates/genmsi-format/src/catalogue.rs`: `("StorageDef", "Storage",
    "Almacen")` y `("StorageDef", "Tamaño", "10")`).
- **Semántica.** La clase compila la sentencia `STORAGE <nombre> <capacidad>` en
  la sección de definiciones del GPSS: `crates/genmsi-core/src/gpss.rs`, fn
  `definitions` (línea 269), escribe `line(out, name, "STORAGE", …)` por cada
  almacén declarado. El motor lee las dos propiedades en
  `crates/genmsi-core/src/program.rs`, fn `Compiler::collect_definitions` (línea
  2024, brazo `"StorageDef"`, línea 2030) y guarda la capacidad en
  `definitions.storages`; `crates/genmsi-core/src/sim.rs`, fn `install_definitions`
  (línea 301), crea la entidad `Storage` con esa capacidad
  (`crates/genmsi-core/src/stat.rs`, struct `Storage`, línea 237). En ejecución no
  hay bloque: `Compiler::expand` devuelve `Ok(None)` para `"StorageDef"`
  (`crates/genmsi-core/src/program.rs`, línea 1025), igual que el original, que
  no genera ninguna sentencia de flujo.

**El banco.** Un `StorageDef` (Storage `Almacen`, Tamaño `10`) sin ninguna
conexión, más un `Clock` de 3600 que compila `GENERATE 3600` + `TERMINATE 1`.
El banco sólo declara el almacén y termina la corrida a las 3600: ninguna
transacción usa el almacén.

**GPSS.** Coincide con el del original. Los dos escriben las mismas tres
sentencias (`Almacen STORAGE 10`, `GENERATE 3600`, `TERMINATE 1`) y el
comparador del oráculo lo confirma: `bash scripts/wine-oracle/compare-gpss.sh
/tmp/fase7/StorageDef/ours.gpss.txt /tmp/fase7/StorageDef/theirs.gpss.txt` →
"el GPSS coincide: 1 bloques y 0 conexiones" (exit 0). Lo único que difiere son
espacios al final de línea y el newline final del archivo nuestro.

**Cifras.** La tabla del oráculo (`/tmp/fase7/StorageDef/numbers.md`: horizon
3600, 3 replications, seed base 7) tiene las dos métricas de la sección
`storage` y las dos coinciden con error relativo 0%: `average_content` de
`ALMACEN` 0 contra 0 (CI [0, 0], "Match: yes") y `utilization` 0 contra 0 (CI
[0, 0], "Match: yes"). Los dos reportes coinciden en el encabezado
(`START_TIME 0`, `END_TIME 3600`, `BLOCKS 2`, `FACILITIES 0`, `STORAGES 1`) y en
la fila del almacén: `CAP. 10`, `REMAIN. 10`, `MIN. 0`, `MAX. 0`, `ENTRIES 0`,
`AVL. 1`, `RETRY 0`, `DELAY 0` (`ours.report.txt` y `theirs.report.txt`).
Quedan dos diferencias de forma, que no mueven ninguna cifra:
- La columna `LOC` de la tabla de bloques: el original numera sus bloques desde
  el 2 (`LOC 2` GENERATE, `LOC 3` TERMINATE) porque la sentencia `STORAGE`
  ocupa la línea 1 del programa; el nuestro numera los bloques desde el 1
  (`crates/genmsi-core/src/report.rs`, fn `render`, contador `numeric_label`,
  líneas 60-68).
- El original escribe `AVE.C. 0,000` y `UTIL. 0,000`; el nuestro escribe `0`,
  porque la fn `number` de `crates/genmsi-core/src/report.rs` (línea 23) saca
  los decimales cuando el valor es entero.

**Veredicto.** verificada

**Diferencia concreta.** Se aparta del original en la columna `LOC` del reporte
(el original descuenta la línea que ocupa la sentencia `STORAGE`) y en el
formato de los decimales enteros de la sección `STORAGE`; el GPSS, las cifras
(0% de error relativo en las dos métricas) y la semántica de la declaración
coinciden. Los dos cambios son de forma y no tocan el core del modelo: el
número de línea vive en `crates/genmsi-core/src/report.rs`, fn `render` (el
contador `numeric_label`), y el formato de los decimales en la misma fn
`number`; lo que los fijaría es un test de `crates/genmsi-core/tests/` que
compare el reporte de este banco con `theirs.report.txt` por fila y por `LOC`.
Aparte, un `StorageDef` sin `Tamaño` vale hoy 0 unidades
(`crates/genmsi-core/src/program.rs`, fn `collect_definitions`, `unwrap_or(0.0)`)
y el original diría "El parámetro B no debe ser nulo"; está anotado como
pendiente en `odd/tasks/notas-almacen.md` (rama `feature/almacen`, sección 5) y
lo fijaría un test de `crates/genmsi-core/tests/` que compile un `StorageDef`
sin `Tamaño`.

### StorageUse

**El original.** "Las transacciones ingresan a un storage por un sierto tiempo y
salen. Si el storage esta lleno, aguardan haciendo cola. El Storage debe estar
declara (esta clase no lo declara)." Consideraciones: "Se debe utilizar otro
módulo que defina la capacidad del Storage", "La propiedad Queue agrega una
cola para recaudar los datos de la espera antes de ingrear el storage. si el
valor dela propiedad es nulo no se crea la cola" y "Media y desvio indican el
tiempo que la transacción permanecerá en el sotrage."
(`/tmp/fase7/StorageUse/original.txt`).

- **Puertos.** Una entrada y una salida: `Entrada1` (rótulo "Ingreso", en el
  borde superior) y `Salida1` (rótulo vacío, en el borde inferior). El paquete
  lo dice literal: "entradas: ['Entrada1'] salidas: ['Salida1']"; el grafo de la
  clase en `Genmsi_20230711.backup/gClasses.xml` lo confirma
  (`part type="Input" value="50,0" ident="Entrada1"`,
  `part type="Output" value="50,40" ident="Salida1"`). El banco los conecta:
  `Generate → StorageUse → Terminate`.
- **Propiedades y defaults.**
  - `Storage` — `EntityUse`, subType `Storage` — default `"Almacen"`
    (`'Storage a utilizar por un tiempo'='Almacen'`) — el almacén que se usa.
    La clase **no** lo declara: lo dice su propia descripción.
  - `Queue` — `EntityUse`, subType `Queue` — default `"qAlmacen"` — la cola que
    "[agrega] una cola para recaudar los datos de la espera antes de ingrear el
    storage"; "si el valor dela propiedad es nulo no se crea la cola", y eso es
    lo que hace el nuestro (`crates/genmsi-core/src/program.rs`, fn
    `optional_entity`, línea 2546: con la propiedad vacía devuelve `None` y el
    motor no crea la cola).
  - `Cantidad` — `TSNA`, subType `SNA` — default vacío (`''`) — "Cantidad de
    ingresos/Egresos del storage": las unidades que toma y devuelve una
    transacción.
  - `Media` — `TSNA` — default `10` (`'Tiempo Medio de uso del storage'='10'`) —
    el tiempo que la transacción permanece en el almacén.
  - `Modificador` — `TSNA` — default `5` (`'Desvio o modificador de la
    media'='5'`) — el desvío de esa media.
  - Nuestro catálogo trae los mismos defaults
    (`crates/genmsi-format/src/catalogue.rs`: `("StorageUse", "Media", "10")`,
    `("StorageUse", "Modificador", "5")`, `("StorageUse", "Queue", "qAlmacen")`,
    `("StorageUse", "Storage", "Almacen")`).
- **Semántica.** La clase compila cinco sentencias: `QUEUE <cola>`,
  `ENTER <almacén>[,<cantidad>]`, `DEPART <cola>`, `ADVANCE <media>,<desvío>` y
  `LEAVE <almacén>[,<cantidad>]`, con la transacción entrando por `Entrada1` y
  saliendo por `Salida1` (macrocode en `original.txt`: `ARRIVE [Queue],[Cantidad]`
  / `ENTER [Storage],[Cantidad]` / `DEPART [Queue],[Cantidad]` / `ADV
  [Media],[Modificador]` / `LEAVE [Storage],[Cantidad]`). El macro `ARRIVE` es
  el que produce la sentencia `QUEUE` del GPSS, y el `DEPART` va justo después
  del `ENTER`: la transacción entra en la cola, entra al almacén y sale de la
  cola en el mismo recorrido. Nuestro motor compila exactamente eso en
  `crates/genmsi-core/src/program.rs`, fn `Compiler::expand`, brazo
  `"StorageUse"` (línea 1375): con `Queue` emite Queue → Enter → Depart →
  Advance → Leave, y sin `Queue` emite Enter → Advance → Leave. El `ENTER` es
  bloqueante: si el almacén está lleno la transacción espera
  (`crates/genmsi-core/src/sim.rs`, fn `step`, brazo `BlockKind::Enter`, línea
  857: `try_enter` falla → la transacción pasa a `Waiting`, se suma un retry al
  bloque y la lista de esperantes del almacén se ordena por prioridad); el
  `LEAVE` (línea 891) suelta las unidades y despierta a quien esperaba.

**El banco.** `Generate` (Media 60, Modificador 30) → `StorageUse` (Storage
`Almacen`, Queue `qAlmacen`, Media 10, Modificador 5, `Cantidad` vacía) →
`Terminate`, más un `Clock` de 3600 que compila `GENERATE 3600` + `TERMINATE 1`.
El banco **no declara el almacén**: no hay ningún `StorageDef` en el modelo
(`/tmp/fase7/StorageUse/bench.gmdl`, y el `lint` lo confirma: "storages 0").

**GPSS.** Coincide con el del original. Los dos escriben las mismas nueve
sentencias (`GENERATE 60,30`; `QUEUE qAlmacen`; `ENTER Almacen`;
`DEPART qAlmacen`; `ADVANCE 10,5`; `LEAVE Almacen`; `TERMINATE`; `GENERATE 3600`;
`TERMINATE 1`) y el comparador del oráculo lo confirma: `bash
scripts/wine-oracle/compare-gpss.sh /tmp/fase7/StorageUse/ours.gpss.txt
/tmp/fase7/StorageUse/theirs.gpss.txt` → "el GPSS coincide: 3 bloques y 0
conexiones" (exit 0); lo único que difiere son los espacios al final de las
líneas del `TERMINATE` y el newline final del archivo nuestro. Ni nuestro GPSS
ni el del original traen una línea `STORAGE`: los dos compilan el `ENTER` de un
almacén que nadie declara.

**Cifras.** No hay cifras que comparar: `/tmp/fase7/StorageUse/numbers.md` está
vacío, porque el original no dejó reporte utilizable. Su salida
(`/tmp/fase7/StorageUse/theirs.report.txt`) es el error en pantalla:

```
Tiempo de simulación: 36,  Transacción activa: 1
Bloque actual    :    2,  StorageUse      QUEUE           qAlmacen
Siguiente bloque :    3,                  ENTER           Almacen
ERROR!!Referencia a una entidad storage inexistente. (Storage: -1).
T. Nº 1
```

O sea, el original abre el modelo y detiene la simulación en el primer `ENTER`
de un almacén que nadie declara. El nuestro, en cambio, corre y da
(`/tmp/fase7/StorageUse/ours.report.txt`): 62 entradas en el `GENERATE`, la cola
`QALMACEN` con 62 entradas y `MAX` 1, y la sección `STORAGE` con `CAP. 1000000`
— la capacidad inventada. La fila `ENTER` marca 62 entradas y 0 reintentos, y la
de `LEAVE` 61: nuestro `ENTER` cuenta los intentos, el original cuenta las
entradas que entraron (esto lo anota `odd/tasks/notas-almacen.md`, rama
`feature/almacen`, sección 5: "La fila `ENTER` de la tabla de bloques cuenta los
intentos").

Lo que sí se midió de esta clase, en otro banco, es la regla de las unidades
(`odd/tasks/notas-almacen.md`, rama `feature/almacen`, sección 2.2, con tres
modelos de la misma forma que este banco y con `Clock` de 3600): `Tamaño` 1 y
`Cantidad` 1 dan `MAX. 1` y las demás transacciones esperando (`RETRY` 114 en el
almacén); `Tamaño` 2 y `Cantidad` 1 dan `MAX. 2`; `Tamaño` 1 y `Cantidad` 2 no
deja entrar a nadie (`MAX. 0`, `ENTRIES 0`). Esa rama lo fija con
`crates/genmsi-core/tests/storages.rs`, que no está en este worktree.

**Veredicto.** pendiente

**Diferencia concreta.** No se puede verificar con este banco: el original
dejó un error y no un reporte, y la causa de la diferencia está determinada y
corregida en otra rama: el original se niega a simular el `ENTER` de un almacén
que nadie declara ("Referencia a una entidad storage inexistente") y nuestro
motor lo simula con una capacidad inventada de 1.000.000
(`UNDECLARED_STORAGE_CAPACITY`, `crates/genmsi-core/src/sim.rs`, línea 26, y los
usos en las líneas 862, 872, 880 y 895 del mismo archivo). **Lo arregla otra
sesión**: la rama `feature/almacen`, con los commits `0a4383f` ("fix(core): a
storage only exists when the model declares it") y `5f1490c` ("feat(app): the
editor says once what a storage nobody declares means"), que no están en este
worktree (aquí HEAD es `82f6515`). Ese commit borra la capacidad inventada, hace
que un `ENTER`/`LEAVE` de un almacén sin declarar detenga la corrida, y deja el
`RETRY` de los rechazos en el almacén y no en el bloque.

Quedan dos diferencias que ese arreglo **no** toca y que este banco tampoco
puede medir (su `Cantidad` está vacía):
- La cola de la `StorageUse` no cuenta unidades. El macrocode del original
  pasa `Cantidad` al `QUEUE` y al `DEPART` (`DEPART [Queue],[Cantidad]`), y con
  `Cantidad` 2 su GPSS sería `QUEUE qAlmacen,2` / `DEPART qAlmacen,2`; el
  nuestro fija 1 (`crates/genmsi-core/src/program.rs`, fn `Compiler::expand`,
  brazo `"StorageUse"`: `QueueSpec { units: 1.0 }` y
  `Depart(QueueSpec { units: 1.0 })`), y ni el GPSS ni el motor lo miran
  (`crates/genmsi-core/src/gpss.rs`, fn `block_statements`, líneas 593-598: la
  cola se escribe sin unidades; `crates/genmsi-core/src/sim.rs`, fn `step`,
  brazo `BlockKind::Queue`, línea 745: encola si `units > 0` y
  `crates/genmsi-core/src/stat.rs`, fn `Queue::enqueue`, línea 176: suma 1 por
  transacción). Está anotado como pendiente en `odd/tasks/notas-almacen.md`
  (rama `feature/almacen`, sección 5: "La cola de la `StorageUse` no cuenta
  unidades"); el arreglo toca la entidad `Queue` de
  `crates/genmsi-core/src/stat.rs`, el `QueueSpec` de
  `crates/genmsi-core/src/program.rs` y cada clase que usa una cola, y lo
  fijaría un test de `crates/genmsi-core/tests/` con una `StorageUse` de
  `Cantidad` 2 que compare `MAX` y `ENTRIES` de la cola con los del original
  (240 y 120 con 120 transacciones de 2 unidades, según esas notas).
- El `lint` no dice nada: `genmsi lint` sobre este banco responde "status: ok …
  diagnostics: none" (`/tmp/fase7/StorageUse/lint.txt`), pero el modelo usa el
  almacén `Almacen` sin que ningún `StorageDef` lo declare. La regla ya existe
  en la capa de formato (`crates/genmsi-format/src/declaration.rs`, fn
  `undeclared_entities`: `Storage` con `Policy::Required` y `StorageDef` como
  declarante) y la app la usa (`crates/genmsi-app/engine/src/session.rs`,
  línea 234), pero el `lint` de la CLI no: `crates/genmsi-cli/src/commands/lint.rs`,
  fn `run` (línea 54), se apoya en `genmsi_core::documents::lint_document`, que
  sólo devuelve `program.diagnostics`, los del compilador del core. Lo
  arreglaría que `lint_document` sumara `genmsi_format::declaration_warnings`
  (o que el core emitiera la observación, que es lo que hace `feature/almacen`),
  y lo fijaría un test de `crates/genmsi-cli/src/commands/lint.rs` que fije que
  el `lint` de este banco da "status: diagnostics" y no "none".

### FacilityUse

**El original.** "Las transacciones intenta tomar una facility, si no pueden
esperan ordenadas por prioridad. Cuando las transacciones toman la facility, la
utilizan por un tiempo indicado según media y modificador. Luego la liberan para
dar lugar a que otra transacción tome la facility. Las transacciones cuando
liberan la facility continúan por la salida1. Puede declararse una fila de
espera (Queue) utilizando la propiedad Queue."
(`/tmp/fase7/FacilityUse/original.txt`, el `ClassDescrip` de la clase en
`Genmsi_20230711.backup/gClasses.xml`, `className="FacilityUse"`).

- **Puertos.** Una entrada y una salida: `Entrada1` (rótulo "Ingreso al recurso")
  y `Salida1` (rótulo vacío). El paquete lo dice literal: "entradas:
  ['Entrada1'] salidas: ['Salida1']"; el grafo de la clase lo confirma
  (`part type="Input" value="50,0" ident="Entrada1"`,
  `part type="Output" value="50,40" ident="Salida1"`). El banco los conecta:
  `Generate → FacilityUse → Terminate`.
- **Propiedades y defaults.**
  - `Facility` — `EntityUse`, subType `Facility` — default `"Recurso"`
    (`'[Facility:'Recurso utilizado'='Recurso']'`) — el recurso que la
    transacción toma.
  - `Queue` — `EntityUse`, subType `Queue` — default `"Cola"`
    (`'[Queue:'cola del recurso utilizado'='Cola']'`) — la fila de espera: "Puede
    declararse una fila de espera (Queue) utilizando la propiedad Queue". Es
    opcional: nuestro motor la crea sólo si la propiedad tiene valor
    (`crates/genmsi-core/src/program.rs`, fn `Compiler::expand`, brazo
    `"FacilityUse"`, línea 1164: `self.optional_entity(module, "Queue")?`).
  - `Media` — `TSNA`, subType `SNA` — default `10`
    (`'[Media:'Tiempo Medio de uso del recurso'='10']'`) — el tiempo que la
    transacción usa el recurso.
  - `Modificador` — `TSNA`, subType `SNA` — default `5`
    (`'[Modificador:'Desvio o Modificador de la media'='5']'`) — el desvío de esa
    media.
  - Nuestro catálogo trae los mismos defaults
    (`crates/genmsi-format/src/catalogue.rs`, `FACILITYUSE`, líneas 682-695, y la
    lista de defaults, líneas 141-144).
- **Semántica.** La clase compila cinco sentencias: `QUEUE <cola>`,
  `SEIZE <recurso>`, `DEPART <cola>`, `ADVANCE <media>,<desvío>` y
  `RELEASE <recurso>` (macrocode: `ARRIVE [Queue]` / `SEIZE [Facility]` /
  `DEPART [Queue]` / `ADV [Media],[Modificador]` / `RECOUT [Facility]` /
  `OUT Salida1`; el `RECOUT` del macro es nuestro `RELEASE`). Nuestro motor
  compila exactamente esa secuencia en `crates/genmsi-core/src/program.rs`, fn
  `Compiler::expand` (línea 996), brazo `"FacilityUse"` (línea 1164): con `Queue`
  emite Queue → Seize → Depart → Advance → Release; sin `Queue`, Seize →
  Advance → Release. En `crates/genmsi-core/src/sim.rs`, fn `step`: el `SEIZE`
  (línea 758) es bloqueante y sus esperantes se ordenan por prioridad
  (`waiters.sort_by_key(|waiter| (std::cmp::Reverse(self.priority_of(*waiter)),
  *waiter))`, línea 779) — el "esperan ordenadas por prioridad" de la descripción
  del original; el `RELEASE` (línea 790) suelta el recurso y despierta al
  primero de la lista.

**El banco.** `Generate` (Media 60, Modificador 30) → `FacilityUse` (Facility
`Recurso`, Queue `Cola`, Media 10, Modificador 5) → `Terminate`, más un `Clock`
de 3600 que compila `GENERATE 3600` + `TERMINATE 1`
(`/tmp/fase7/FacilityUse/bench.gmdl`). El `lint` del banco no dice nada:
"status: ok … diagnostics: none" (`/tmp/fase7/FacilityUse/lint.txt`).

**GPSS.** Coincide con el del original. Los dos escriben las mismas nueve
sentencias:

```
                GENERATE        60,30
A_FacilityUse   QUEUE           Cola
                SEIZE           Recurso
                DEPART          Cola
                ADVANCE         10,5
                RELEASE         Recurso
A_Terminate     TERMINATE
                GENERATE        3600
                TERMINATE       1
```

(`/tmp/fase7/FacilityUse/ours.gpss.txt` contra
`/tmp/fase7/FacilityUse/theirs.gpss.txt`; el comparador del oráculo lo
confirma: `bash scripts/wine-oracle/compare-gpss.sh
/tmp/fase7/FacilityUse/ours.gpss.txt /tmp/fase7/FacilityUse/theirs.gpss.txt` →
"el GPSS coincide: 3 bloques y 0 conexiones", exit 0). Lo único que difiere es
el relleno de espacios y el newline final.

**Cifras.** Las siete métricas de la tabla del oráculo
(`/tmp/fase7/FacilityUse/numbers.md`) coinciden, con las tolerancias del oráculo
(20% en conteos, 35% en tiempos y ocupaciones):

- facility `RECURSO` utilization: 0,147 del original contra 0,168 nuestro, 14%
  de error (dentro del 35%), intervalo [0,131; 0,205].
- facility `RECURSO` average_time: 9,636 contra 9,886, 3%.
- queue `COLA` entries: 55 contra 61, 11% (dentro del 20%); average_content
  0 contra 0; average_time 0 contra 0.
- block `A_FacilityUse` entries: 55 contra 61, 11%; block `A_Terminate`
  entries: 55 contra 61, 11%.

Los promedios de la cola son cero en los dos porque la cola nunca tiene
contenido al final de la corrida (`CONT.` 0 en las dos tablas): es el defecto
documentado del original en `docs/oracle-comparison.md` ("El original imprime
`AVE.CONT.`, `AVE.TIME` y `AVE.(-0)` como exactamente `0,000` para cada cola
cuyo `CONT.` es cero al final de la corrida"). La cola llega a `MAX 1` y su
contenido medio es 0: **este banco no llega a ejercitar la espera por prioridad**
—la regla que la descripción del original enuncia— porque el recurso está libre
el 85% del tiempo (utilization 0,147) y nunca hay dos transacciones
compitiendo por él.

Diferencias de formato del reporte que no son de esta clase (valen para los
cuatro bancos de estas fichas): nuestro reporte imprime una línea
`SEMILLA` que el original no imprime; el original cierra con la tabla
`FEC_XACT_NUMBER` (la lista de eventos pendientes) que el nuestro no imprime; la
columna `LOC` de los bloques sin nombre no es el número de línea del original
sino un contador de bloques sin nombre (el nuestro pone `2,3,4,5,6,7` donde el
original pone `3,4,5,6,8,9`), y esas etiquetas numéricas el oráculo no las
compara (`crates/genmsi-core/src/oracle.rs`, fn `compare_metrics`, línea 471:
"Numeric labels are assigned by each implementation in its own emission order, so
only named labels are comparable"); y el original escribe los ceros de la fila de
cola como `0,000` donde el nuestro escribe `0`
(`crates/genmsi-core/src/report.rs`, fn `number`, línea 23).

**Veredicto.** verificada

**Diferencia concreta.** Nada de lo que este banco mide se aparta del original:
el GPSS coincide sentencia por sentencia, las siete cifras de la tabla caen
dentro de las tolerancias y el comportamiento es el que la clase describe (toma
el recurso, espera en la cola si no puede, lo usa `Media`±`Modificador` y lo
suelta por `Salida1`). Queda sin verificar con este banco la espera por
prioridad —la cola nunca pasa de una transacción— y el formato de los ceros de
la tabla de colas.

### FacilityInt

**El original.** La clase **no** tiene descripción propia: su `ClassDescrip` es
una copia del de `FacilityUse` ("Las transacciones intenta tomar una facility,
si no pueden esperan ordenadas por prioridad. Cuando las transacciones toman la
facility, la utilizan por un tiempo indicado según media y modificador. Luego la
liberan para dar lugar a que otra transacción tome la facility. Las
transacciones cuando liberan la facility continúan por la salida1. Puede
declararse una fila de espera (Queue) utilizando la propiedad Queue."
— `/tmp/fase7/FacilityInt/original.txt`). Lo que la distingue de `FacilityUse` no
está en la descripción sino en el macrocode: toma el recurso con un `PREEMPT`
—que puede interrumpir al dueño— y lo devuelve con un `RETURN`, y tiene una
salida extra para el dueño interrumpido.

- **Puertos.** Una entrada y dos salidas: `Entrada1` (rótulo "Ingreso al
  recurso"), `SalidaAlt` (rótulo "Propietaria") y `Salida1` (rótulo vacío). El
  paquete lo dice literal: "entradas: ['Entrada1'] salidas: ['SalidaAlt',
  'Salida1']"; el grafo de la clase lo confirma (`part type="Input"
  value="50,0" ident="Entrada1"`, `part type="Output" value="120,20"
  ident="SalidaAlt"`, `part type="Output" value="50,40" ident="Salida1"`).
  `SalidaAlt` es por donde sale "Propietaria", la transacción a la que se le
  interrumpe el recurso.
- **Propiedades y defaults.**
  - `Facility` — `EntityUse`, subType `Facility` — default `"Recurso"`
    (`'[Facility:'Facility a interrumpir por la transacción activa.'='Recurso']'`)
    — el recurso que se toma y se interrumpe.
  - `Media` — `TSNA`, subType `SNA` — default `10`
    (`'[Media:'Tiempo Medio de uso del recurso'='10']'`) — el tiempo de uso.
  - `Modificador` — `TSNA`, subType `SNA` — default `5`
    (`'[Modificador:'Desvio o Modificador de la media'='5']'`) — el desvío.
  - `ModoInterrupcion` — `Listed`, subType `PriorityMode` — default `"Prioridad"`
    (`'[ModoInterrupcion:'Modo de interrupción o por prioridad'='Prioridad']'`)
    — cómo se interrumpe: por prioridad.
  - `ModoArrebato` — `Listed`, subType `RemoveMode` — default `"NoRemover"`
    (`'[ModoArrebato:'Modo de arrebato temporal o permanete.'='NoRemover']'`) —
    si el dueño interrumpido se remueve o no.
  - `Queue` — `EntityUse`, subType `Queue` — default `"ColaInt"`
    (`'[Queue:'cola del recurso interrumpido'='ColaInt']'`) — la cola de espera
    del recurso interrumpido.
  - `Remanente` — `EntityUse`, subType `Parameter` — default `"Remanente"`
    (`'[Remanente:'Parámetro de la transacción propietaria donde se guarda el
    remanente'='Remanente']'`) — el parámetro donde se guarda el tiempo que le
    quedaba al dueño interrumpido.
  - Nuestro catálogo trae los mismos defaults
    (`crates/genmsi-format/src/catalogue.rs`, `FACILITYINT`, líneas 661-676, y la
    lista de defaults, líneas 133-139).
- **Semántica.** La clase compila seis sentencias: `QUEUE <cola>`,
  `PREEMPT <recurso>,PR,,<Remanente>`, `DEPART <cola>`,
  `ADVANCE <media>,<desvío>`, `RETURN <recurso>` y `OUT Salida1` (macrocode:
  `ARRIVE [Queue]` / `PREEMPT [Facility],[ModoInterrupcion],SalidaAlt,[Remanente],
  [ModoArrebato]` / `DEPART [Queue]` / `ADV [Media],[Modificador]` /
  `RETURN [Facility]` / `OUT Salida1`). El `PR` es el modo "por prioridad": la
  transacción que llega sólo toma el recurso si su prioridad es mayor que la
  del dueño; si no, espera como en un `SEIZE` (`crates/genmsi-core/src/sim.rs`,
  fn `step`, brazo `BlockKind::Preempt`, línea 795: la regla está en las líneas
  805-809, `self.priority_of(previous) >= self.priority_of(xact)` →
  `self.wait_for_facility(...)`). Cuando sí interrumpe, guarda el tiempo
  restante del dueño en el parámetro `Remanente` y lo manda por `SalidaAlt`
  (`interrupt_owner` y `route_owner`, líneas 814-828). Nuestro motor compila esa
  secuencia en `crates/genmsi-core/src/program.rs`, fn `Compiler::expand`,
  brazo `"FacilityInt"` (línea 1287).

**El banco.** `Generate` (Media 60, Modificador 30) → `FacilityInt` (Facility
`Recurso`, Queue `ColaInt`, Media 10, Modificador 5, `ModoInterrupcion`
`Prioridad`, `ModoArrebato` `NoRemover`, `Remanente` `Remanente`) →
`Terminate`, más un `Clock` de 3600 (`/tmp/fase7/FacilityInt/bench.gmdl`). La
salida `SalidaAlt` queda **sin conectar** (el `caption` del módulo lo dice:
"FacilityInt (salidas sin conectar: SalidaAlt)"), así que el dueño interrumpido
no tendría adónde ir; el `lint` no dice nada ("status: ok … diagnostics: none",
`/tmp/fase7/FacilityInt/lint.txt`).

**GPSS.** Coincide con el del original. Los dos escriben las mismas nueve
sentencias, incluido el `PREEMPT` con sus cuatro operandos:

```
                GENERATE        60,30
A_FacilityInt   QUEUE           ColaInt
                PREEMPT         Recurso,PR,,Remanente
                DEPART          ColaInt
                ADVANCE         10,5
                RETURN          Recurso
A_Terminate     TERMINATE
                GENERATE        3600
                TERMINATE       1
```

(`/tmp/fase7/FacilityInt/ours.gpss.txt` contra
`/tmp/fase7/FacilityInt/theirs.gpss.txt`; `bash
scripts/wine-oracle/compare-gpss.sh /tmp/fase7/FacilityInt/ours.gpss.txt
/tmp/fase7/FacilityInt/theirs.gpss.txt` → "el GPSS coincide: 3 bloques y 0
conexiones", exit 0). El `PREEMPT` sale bien porque el listado del programa lo
escribe `crates/genmsi-core/src/gpss.rs`, fn `block_statements` (línea 494), brazo
`BlockKind::Preempt` (líneas 606-621): recurso, `PR`, el destino de
interrupción (vacío cuando `SalidaAlt` no está conectado) y el parámetro
`Remanente`.

**Cifras.** Las siete métricas de la tabla del oráculo
(`/tmp/fase7/FacilityInt/numbers.md`) coinciden:

- facility `RECURSO` utilization: 0,172 del original contra 0,168 nuestro, 3% de
  error; average_time: 9,656 contra 9,886, 2%.
- queue `COLAINT` entries: 64 contra 61, 5%; average_content 0 contra 0;
  average_time 0 contra 0.
- block `A_FacilityInt` entries: 64 contra 61, 5%; block `A_Terminate` entries:
  64 contra 61, 5%.

Con estas cifras el banco **no** ejercita la interrupción: el recurso está libre
casi siempre (utilization 0,172), la cola no llega a tener dos transacciones
(`MAX 1`, contenido 0) y todas las transacciones nacen con la misma prioridad
(el `Generate` del banco no tiene `Prioridad`), de modo que el `PR` nunca
interrumpe a nadie: se mide el camino sin interrupción y nada más.

Sí hay una cifra del reporte que difiere: la columna `OWNER` de la fila
`FACILITY RECURSO`. La nuestra dice 57 y la del original dice 0
(`/tmp/fase7/FacilityInt/ours.report.txt` contra
`/tmp/fase7/FacilityInt/theirs.report.txt`). No es una regla distinta: es el
estado final de la corrida. En nuestra corrida había todavía una transacción
dentro del `ADVANCE` al llegar la hora 3600 (la fila del `ADVANCE` marca
`CURRENT 1`), y por eso el recurso seguía tenedor de esa transacción —la
columna `OWNER` es el identificador de la transacción que tiene el recurso, no
un contador—; en la del original el `ADVANCE` marca `CURRENT 0` y el recurso ya
estaba devuelto. Con otras semillas el final es el del original: con
`./target/release/genmsi simulate --seed 1 /tmp/fase7/FacilityInt/bench.gmdl` (y
las semillas 2 a 6) salen `OWNER 0` y `ADVANCE CURRENT 0`. Que la columna sea el
identificador del dueño y no un contador lo confirman los reportes del propio
original, donde el valor es mayor que `ENTRIES`
(`fixtures/outputs/TP3__aeropuerto.report.txt`, línea 118: "CAJERO1FREESHOP 11
0,231 75,636 1 126 0 0 0 0"), y nuestro motor da la misma forma
(`crates/genmsi-core/src/report.rs`, fn `render`, línea 98:
`facility.owner.unwrap_or(0)`).

**Veredicto.** con diferencias

**Diferencia concreta.** La fila `PREEMPT` de la tabla de bloques del reporte:
la nuestra escribe `PREEMPT Recurso` y la del original
`PREEMPT Recurso,PR,,Remanente` —con el `PR`, el destino de interrupción y el
`Remanente`
(`/tmp/fase7/FacilityInt/theirs.report.txt`; también
`fixtures/outputs/TP3__aeropuerto.report.txt`, líneas 54, 73, 81 y 86:
"PREEMPT Cajero1FreeShop,PR,,Remanente"). El listado del programa sí lo escribe
bien; el que lo pierde es el generador del reporte: `crates/genmsi-core/src/
report.rs`, fn `type_and_operands` (línea 677), brazo `BlockKind::Preempt`
(línea 451), que devuelve sólo el recurso. Lo arreglaría esa función, que
debería escribir los mismos cuatro operandos que `crates/genmsi-core/src/gpss.rs`
(líneas 606-621): recurso, `PR` (o vacío), el destino de interrupción (vacío
cuando `SalidaAlt` no está conectado) y el parámetro `Remanente`. Lo fijaría un
test en `crates/genmsi-core/tests/report_gpss.rs` —el archivo que ya fija las
filas `GATE`, `TEST`, `SELECT` y `REMOVE` contra los reportes del original— que
comprobara que la fila `PREEMPT` de `TP3__aeropuerto` sale
"Cajero1FreeShop,PR,,Remanente".

### GateF

**El original.** La clase **no** tiene descripción: su `ClassDescrip` está
vacío (`/tmp/fase7/GateF/original.txt`: "(vacio: el original no describe esta
clase)"). Lo que dice de ella es su macrocode: una puerta que se abre cuando
un recurso está libre, con una salida de fallo.

- **Puertos.** Una entrada y dos salidas: `Entrada1` (rótulo "Ingreso"),
  `Salida1` (rótulo vacío) y `Salida2` (rótulo "fallo"). El paquete lo dice
  literal: "entradas: ['Entrada1'] salidas: ['Salida1', 'Salida2']"; el grafo
  de la clase lo confirma (`part type="Input" value="38,0" ident="Entrada1"`,
  `part type="Output" value="38,48" ident="Salida1"`, `part type="Output"
  value="0,24" ident="Salida2"` — la salida de fallo sale por el lado
  izquierdo). El macrocode empieza con `DECLARE Salida1,Output //Se declara
  primero por compatibilidad.`
- **Propiedades y defaults.**
  - `Facility` — `EntityUse`, subType `Facility` — default `"Recurso"`
    (`'[Facility:'Facility a evaluar.'='Recurso']'`) — el recurso que se
    evalúa.
  - `op` — `Listed`, subType `op_Logic_Faci` — default `"Libre"`
    (`'[op:'Operador logico de facility.'='Libre']'`) — el operador lógico del
    recurso: si está libre la puerta se abre.
  - Nuestro catálogo trae los mismos defaults
    (`crates/genmsi-format/src/catalogue.rs`, `GATEF`, líneas 736-750, y la
    lista de defaults, líneas 161-162).
- **Semántica.** La clase compila una sentencia: `GATE NU <recurso>` (el
  `NU` es el modo "recurso no usado"), y la transacción sale por `Salida1` si
  la prueba se cumple y por `Salida2` ("fallo") si no (macrocode: `GATEF [op],
  [Facility], Salida2:'fallo'` / `OUT Salida1`). Nuestro motor lo compila en
  `crates/genmsi-core/src/program.rs`, fn `Compiler::expand`, brazo `"GateF"`
  (línea 1589): `op` vacío o "Libre" → `GateCondition::Facility { available:
  true }`; cualquier otro valor → `available: false`. La prueba está en
  `crates/genmsi-core/src/sim.rs`, fn `gate_condition` (línea 1574): el recurso
  tiene que estar `available` y no `busy`. Si la prueba falla y la salida de
  fallo está conectada, la transacción se va por `Salida2`; si no está
  conectada, la transacción **espera en la puerta** y se reintenta cuando el
  recurso cambia (brazo `BlockKind::Gate`, línea 983: `retries += 1` y la
  transacción a `gate_waiters`, líneas 1001-1002; `fn retry_gates`, línea 1599).

**El banco.** `Generate` (Media 60, Modificador 30) → `GateF` (Facility
`Recurso`, op `Libre`) → `Terminate`, más un `Clock` de 3600
(`/tmp/fase7/GateF/bench.gmdl`). La salida `Salida2` queda **sin conectar**
(el `caption` del módulo lo dice: "GateF (salidas sin conectar: Salida2)"), así
que la puerta no tiene salida de fallo. El `lint` no dice nada ("status: ok …
diagnostics: none", `/tmp/fase7/GateF/lint.txt`).

**GPSS.** Coincide con el del original. Los dos escriben las mismas cinco
sentencias:

```
                GENERATE        60,30
GateF           GATE            NU Recurso
A_Terminate     TERMINATE
                GENERATE        3600
                TERMINATE       1
```

(`/tmp/fase7/GateF/ours.gpss.txt` contra `/tmp/fase7/GateF/theirs.gpss.txt`;
`bash scripts/wine-oracle/compare-gpss.sh /tmp/fase7/GateF/ours.gpss.txt
/tmp/fase7/GateF/theirs.gpss.txt` → "el GPSS coincide: 3 bloques y 0
conexiones", exit 0; lo único que difiere es el reparto de espacios).

**Cifras.** Las tres métricas de la tabla del oráculo
(`/tmp/fase7/GateF/numbers.md`) coinciden: facility `RECURSO` utilization 0
contra 0 y average_time 0 contra 0 (el recurso no lo usa nadie: la puerta se
abre siempre), y block `A_Terminate` entries 63 contra 61, 3%. La tabla del
original y la nuestra coinciden en la fila de la puerta: 63 entradas, 0 current,
0 retry (la nuestra 59, 0, 0) — o sea, todas las transacciones pasan de largo.

La fila de la puerta **no** está entre las métricas que el oráculo compara, y
no por las cifras sino por la etiqueta: el oráculo empareja las filas de bloque
por su `LOC` (`crates/genmsi-core/src/oracle.rs`, fn `compare_metrics`, línea
471: "Block entry counts, matched by GPSS label"), y la del original se llama
`GateF` mientras la nuestra se llama `A_GateF` (ver "Diferencia concreta"). Si se
comparara, daría 63 contra 59, un 6%, dentro de la tolerancia del 20%.

**Veredicto.** con diferencias

**Diferencia concreta.** La columna `LOC` de la fila de la puerta: la del
original dice `GateF` y la nuestra `A_GateF` (`/tmp/fase7/GateF/theirs.report.txt`
contra `/tmp/fase7/GateF/ours.report.txt`). La etiqueta del original sale del
propio macrocode de la clase, que para las cuatro puertas es
`IN Entrada1:'Ingreso'='moduleName'` —el nombre del módulo, sin el `A_` que
ponen las demás clases (`Genmsi_20230711.backup/gClasses.xml`: las puertas
`GateS`, `GateM`, `GateF` y `GateL` escriben `='moduleName'` y el resto
`='A_moduleName'`); el modelo del propio original lo confirma
(`fixtures/models/TP6__Clase 6_3_JefeEducado.gmdl`: el módulo `GateF` tiene la
entrada con `mylabel="GateF"`). La nuestra sale del `mylabel` del puerto
(`crates/genmsi-format/src/model.rs`, fn `Module::label`, línea 254 →
`crates/genmsi-core/src/program.rs`, `label: module.label()`, línea 1003 →
`crates/genmsi-core/src/report.rs`, fn `render`, línea 63), y en este banco ese
`mylabel` es `A_GateF` porque el generador de bancos pone `"A_" + nombre` a
todas las clases sin mirar el token de su macro
(`scripts/wine-oracle/make-benches.py`, línea 535). O sea: el banco se aparta de
lo que haría el editor del original, y de paso esconde del oráculo la fila de la
puerta. Lo arreglaría una de estas dos cosas: (a) el generador de bancos, que
debería poner el `mylabel` que el token de la clase impone (`moduleName` → el
nombre del módulo, `A_moduleName` → `"A_"` + el nombre), o (b) el motor, que
debería escribir en la columna `LOC` el nombre de la sentencia —la regla que ya
existe en `crates/genmsi-core/src/gpss.rs`, fn `base_name`, línea 119— en vez del
`mylabel` del puerto. Lo fijaría un test en
`crates/genmsi-core/tests/report_gpss.rs` que renderice un `GateF` con la
entrada etiquetada `A_GateF` y compruebe que la fila sale con `LOC` `GateF`; el
archivo ya fija el caso `GateF` para un modelo cuyo `mylabel` es `GateF`
(`fn gate_facility_free_row_keeps_the_else_target`), que es exactamente el
banco original y por eso no lo ve.

Además, con este banco no se puede verificar el caso del operador distinto de
`Libre`: la letra que el original escribiría para un recurso ocupado ("U" en
nuestro listado, `crates/genmsi-core/src/gpss.rs`, línea 705) no aparece en
ningún reporte del original, y el banco usa el default `Libre`.

### GateL

**El original.** La clase **no** tiene descripción: su `ClassDescrip` está
vacío (`/tmp/fase7/GateL/original.txt`: "(vacio: el original no describe esta
clase)"). Lo que dice de ella es su macrocode: una puerta que se abre cuando
una llave lógica está en un estado dado, con una salida de fallo.

- **Puertos.** Una entrada y dos salidas: `Entrada1` (rótulo "Ingreso"),
  `Salida1` (rótulo vacío) y `Salida2` (rótulo "fallo"). El paquete lo dice
  literal: "entradas: ['Entrada1'] salidas: ['Salida1', 'Salida2']"; el grafo
  de la clase lo confirma (`part type="Input" value="38,0" ident="Entrada1"`,
  `part type="Output" value="38,48" ident="Salida1"`, `part type="Output"
  value="0,24" ident="Salida2"`). El macrocode empieza con `DECLARE Salida1,
  Output //Se declara primero por compatibilidad.`
- **Propiedades y defaults.**
  - `LogicSwich` — `EntityUse`, subType `LogicSwich` — default `"1"`
    (`'[LogicSwich:'Logic Swich a evaluar.'='1']'`) — la llave lógica que se
    evalúa.
  - `op` — `Listed`, subType `op_Logic_Swich` — default `"Set"`
    (`'[op:'Operador logico del logic swich.'='Set']'`) — el operador: con
    `Set` la puerta se abre cuando la llave está puesta; con `Reset`, cuando
    está deshecha.
  - Nuestro catálogo trae los mismos defaults
    (`crates/genmsi-format/src/catalogue.rs`, `GATEL`, líneas 752-766, y la
    lista de defaults, líneas 164-165).
- **Semántica.** La clase compila una sentencia: `GATE LS <llave>` (o `LR`,
  con el operador `Reset` — el listado de nuestro motor escribe `L` en ese
  caso, ver "Diferencia concreta"), y la transacción sale por `Salida1` si la
  prueba se
  cumple y por `Salida2` ("fallo") si no (macrocode: `GATEL [op],[LogicSwich],
  Salida2:'fallo'` / `OUT Salida1`). Nuestro motor lo compila en
  `crates/genmsi-core/src/program.rs`, fn `Compiler::expand`, brazo `"GateL"`
  (línea 1563): `op` = "Set" → la llave tiene que estar puesta; cualquier otro
  valor → tiene que estar deshecha. La prueba está en
  `crates/genmsi-core/src/sim.rs`, fn `gate_condition` (línea 1574): el valor de
  la llave (`self.entities.logic(&name)`, y `crates/genmsi-core/src/stat.rs`,
  línea 558). Si la prueba falla y la salida de fallo está conectada, la
  transacción se va por `Salida2`; si no está conectada, la transacción
  **espera en la puerta** y se reintenta cuando la llave cambia (brazo
  `BlockKind::Gate`, línea 983: `retries += 1` y la transacción a
  `gate_waiters`, líneas 1001-1002; `fn retry_gates`, línea 1599).

**El banco.** `Generate` (Media 60, Modificador 30) → `GateL` (LogicSwich `1`,
op `Set`) → `Terminate`, más un `Clock` de 3600
(`/tmp/fase7/GateL/bench.gmdl`). La salida `Salida2` queda **sin conectar** (el
`caption` del módulo lo dice: "GateL (salidas sin conectar: Salida2)") y nada
en el modelo pone la llave `1`: la puerta no se abre nunca y todas las
transacciones se quedan esperando. El `lint` no dice nada ("status: ok …
diagnostics: none", `/tmp/fase7/GateL/lint.txt`).

**GPSS.** Coincide con el del original. Los dos escriben las mismas cinco
sentencias:

```
                GENERATE        60,30
GateL           GATE            LS 1
A_Terminate     TERMINATE
                GENERATE        3600
                TERMINATE       1
```

(`/tmp/fase7/GateL/ours.gpss.txt` contra `/tmp/fase7/GateL/theirs.gpss.txt`;
`bash scripts/wine-oracle/compare-gpss.sh /tmp/fase7/GateL/ours.gpss.txt
/tmp/fase7/GateL/theirs.gpss.txt` → "el GPSS coincide: 3 bloques y 0
conexiones", exit 0; lo único que difiere es el reparto de espacios).

**Cifras.** La tabla del oráculo (`/tmp/fase7/GateL/numbers.md`) trae **una
sola** métrica: block `A_Terminate` entries, 0 contra 0, 0% de error. Con una
métrica así el oráculo no puede decir nada del comportamiento de la puerta
(`crates/genmsi-core/src/oracle.rs`, fn `ModelComparison::status`, línea 316:
`"no-metrics"` cuando hay menos de tres métricas). Las dos corridas coinciden
en lo esencial: ninguna transacción llega al `TERMINATE` (0 entradas en las
dos).

Pero los contadores de la tabla de bloques sí se apartan:

- La fila de la puerta: la del original dice "0 0 0" (entradas, current,
  retry) y la nuestra "57 57 57". O sea, el original **no cuenta** como entrada
  del bloque `GATE` la transacción que falla la prueba, y la nuestra sí.
- La fila del `GENERATE`: la del original dice "63 63 0" (63 entradas, 63
  current) y la nuestra "57 1 0": en el original las transacciones que fallan
  la puerta siguen contando en el bloque anterior; en el nuestro no.
- El original imprime además una sección de llaves lógicas que el nuestro no
  tiene:

```
LOGICSWITCH               VALUE      RETRY
  1                          0         62
```

  (`/tmp/fase7/GateL/theirs.report.txt`; el original la imprime también en
  `fixtures/outputs/TP6__Clase 6_1_PuertaBanco.report.txt`, línea 30: "PUERTA 0
  13"). Los 62 son los intentos fallidos que el original contó para esa llave
  (el `GENERATE` del original marca 63 transacciones generadas).

**Veredicto.** con diferencias

**Diferencia concreta.** Tres cosas se apartan del original:

1. La columna `LOC` de la fila de la puerta: la del original dice `GateL` y la
   nuestra `A_GateL` (`/tmp/fase7/GateL/theirs.report.txt` contra
   `/tmp/fase7/GateL/ours.report.txt`). La etiqueta del original sale del
   macrocode de la clase, que para las cuatro puertas es `IN Entrada1:'Ingreso'
   ='moduleName'` —el nombre del módulo, sin el `A_` que ponen las demás clases
   (`Genmsi_20230711.backup/gClasses.xml`)— y del modelo del propio original
   (`fixtures/models/TP6__Clase 6_1_PuertaBanco.gmdl`: el módulo `GateL` tiene
   la entrada con `mylabel="GateL"`). La nuestra sale del `mylabel` del puerto
   (`crates/genmsi-format/src/model.rs`, fn `Module::label`, línea 254 →
   `crates/genmsi-core/src/program.rs`, línea 1003 → `crates/genmsi-core/src/
   report.rs`, fn `render`, línea 63), y en este banco ese `mylabel` es
   `A_GateL` porque el generador de bancos pone `"A_" + nombre` a todas las
   clases sin mirar el token de su macro (`scripts/wine-oracle/make-benches.py`,
   línea 535). Lo arreglaría el generador de bancos (que ponga el `mylabel`
   que el token de la clase impone) o el motor (que escriba en la columna `LOC`
   el nombre de la sentencia, la regla que ya está en
   `crates/genmsi-core/src/gpss.rs`, fn `base_name`, línea 119). Lo fijaría un
   test en `crates/genmsi-core/tests/report_gpss.rs` que renderice un `GateL` con
   la entrada etiquetada `A_GateL` y compruebe que la fila sale con `LOC`
   `GateL`.
2. Los contadores de la puerta: la del original "0 0 0" y la nuestra
   "57 57 57", y de rebote la fila del `GENERATE` (63 current en el original,
   1 en el nuestro). Nuestro motor cuenta el intento fallido como entrada y
   retry del bloque y deja a la transacción esperando en él
   (`crates/genmsi-core/src/sim.rs`, fn `step`, brazo `BlockKind::Gate`, línea
   983: `self.block_stats[block_index].retries += 1` y
   `self.gate_waiters.entry(block_index)`, líneas 1001-1002), mientras que el
   original no cuenta la entrada y cuenta los intentos en la llave lógica. Y
   nuestro reporte no tiene sección de llaves lógicas: `crates/genmsi-core/src/
   report.rs`, fn `render` (línea 35) no tiene ninguna, y la entidad sólo
   guarda el valor de la llave (`crates/genmsi-core/src/stat.rs`, línea 519,
   `pub logic: BTreeMap<String, bool>`), sin un contador de intentos. Lo
   arreglaría el core: un contador por llave en `crates/genmsi-core/src/stat.rs`,
   que el `Gate` de `crates/genmsi-core/src/sim.rs` sumara en cada intento
   fallido, y una sección `LOGICSWITCH` en `crates/genmsi-core/src/report.rs`;
   más la regla de si el intento fallido cuenta como entrada del bloque `GATE`,
   que es la que el original no aplica. Lo fijaría un test en
   `crates/genmsi-core/tests/report_gpss.rs` (o uno nuevo de la puerta) con
   este banco: la fila de la puerta tiene que salir "0 0 0" y la sección
   `LOGICSWITCH` tiene que imprimir el valor de la llave y los intentos
   fallidos.
3. La letra del modo en el listado del GPSS cuando la llave tiene que estar
   deshecha: el nuestro escribe `GATE L <llave>` y el original `GATE LR
   <llave>` (`fixtures/outputs/Problema_1__semaforo-1.gpss.txt`, línea 71:
   "GateL1  GATE  LR  *Cuadra"; y `fixtures/outputs/TP10__tp10.report.txt`,
   línea 31: "GateL  GATE  LR  Fin,A_Counter1"). El error está en
   `crates/genmsi-core/src/gpss.rs`, fn `block_statements` (línea 494), brazo
   `BlockKind::Gate` (línea 696): `if *state { "LS" } else { "L" }`; el generador
   del reporte sí escribe bien "LR" (`crates/genmsi-core/src/report.rs`, línea
   480). El banco no lo ve porque usa el operador `Set` (y por eso el GPSS
   coincide en los 62 bancos). Lo fijaría un test de listado del programa con un
   `GateL` de operador `Reset`. El mismo brazo afecta a `GateS` (nuestro
   "S"/"SNF" donde el original escribe "SE"/"SNE",
   `fixtures/outputs/entregaComida__entregaComida.gpss.txt`, líneas 11 y 16) y a
   `GateM`, que son de otro lote.

Queda sin verificar con este banco la regla de la puerta cuando la llave sí
cambia de estado (nada en el modelo la pone) y el caso del operador `Reset` de
la puerta, salvo por el listado del programa de los reportes del original.

### GateS

**El original.** El original no describe esta clase (`/tmp/fase7/GateS/original.txt`:
"ClassDescrip (vacio: el original no describe esta clase)"); lo que dice de ella es
su macrocode: "GATES [op:'Operador logico del storage.'='NoLleno'], [Storage:'Storage a
evaluar.'='Almacen'], Salida2:'fallo'" — una puerta que deja pasar la transacción
cuando el almacén que nombra está en el estado que dice el operador, y que la manda a
la salida de fallo cuando no lo está.

- **Puertos.** Entrada1, con el rótulo "Ingreso"; Salida1, sin rótulo; Salida2, con el
  rótulo "fallo". La sección `graph` de la clase en `Genmsi_20230711.backup/gClasses.xml`
  da "entradas: ['Entrada1'] salidas: ['Salida1', 'Salida2']"
  (`/tmp/fase7/GateS/original.txt`), y nuestro catálogo trae los mismos
  (`crates/genmsi-format/src/catalogue.rs`, static `GATES`, línea 784).
- **Propiedades y defaults.**
  - `Storage` — `EntityUse`, subType `Storage` — default `"Almacen"`
    (`'Storage a evaluar.'='Almacen'`) — el almacén que la puerta evalúa. Catálogo:
    `("GateS", "Storage", "Almacen")` (`crates/genmsi-format/src/catalogue.rs`, línea 170).
  - `op` — `Listed`, subType `op_Logic_Stor` — default `"NoLleno"`
    (`'Operador logico del storage.'='NoLleno'`) — la condición de la puerta. Catálogo:
    `("GateS", "op", "NoLleno")` (línea 171). La lista del operador en el original
    (`Genmsi_20230711.backup/gmacroconfig.gmcfg`, `listedname="op_Logic_Stor"`) tiene
    seis: "SF:Lleno | SNF:NoLleno | SE:Vacio | SNE:NoVacio | SV:Disponible |
    SNV:NoDisponible". Nuestro motor sólo entiende dos de las seis.
- **Semántica.** La puerta se compila como `GATE <letra> <almacén>` y la transacción
  sigue por `Salida1` cuando se cumple, por `Salida2` ("fallo") cuando no, y espera en
  el bloque si esa salida no está conectada. Nuestro motor: `crates/genmsi-core/src/program.rs`,
  fn `Compiler::expand`, brazo `"GateS"` (línea 1576), lee las dos propiedades y arma
  `BlockKind::Gate(GateSpec { condition: GateCondition::Storage { storage, empty } })`,
  donde el operador queda aplastado en un booleano `empty` que sólo es verdadero para
  "vacio"/"empty"; `crates/genmsi-core/src/gpss.rs`, fn `block_statements` (línea 494),
  brazo `BlockKind::Gate` (línea 687), escribe "S" cuando `empty` y "SNF" en cualquier
  otro caso; `crates/genmsi-core/src/sim.rs`, fn `gate_condition` (línea 1574) abre la
  puerta cuando `storage.is_empty() == empty`; y el brazo `BlockKind::Gate` del despacho
  (línea 983) manda a `Salida2` si hay destino de fallo y, si no, deja la transacción
  esperando con un reintento.

**El banco.** Un `Generate` (Media 60, Modificador 30) → `GateS` (Storage "Almacen", op
"NoLleno") → `Terminate`, más un `Clock` (Frecuencia 3600) que compila `GENERATE 3600` +
`TERMINATE 1`. El almacén "Almacen" no lo declara nadie: el banco no tiene ningún
`StorageDef`.

**GPSS.** Coincide con el del original: los dos escriben `GateS GATE SNF Almacen`
(`/tmp/fase7/GateS/ours.gpss.txt` contra `/tmp/fase7/GateS/theirs.gpss.txt`), y el
comparador del oráculo lo confirma: `bash scripts/wine-oracle/compare-gpss.sh
/tmp/fase7/GateS/ours.gpss.txt /tmp/fase7/GateS/theirs.gpss.txt` → "el GPSS coincide: 3
bloques y 0 conexiones" (exit 0). La letra coincide sólo por suerte con este operador:
con la línea base del original `fixtures/outputs/entregaComida__entregaComida.gpss.txt`
el original escribe "HornoConComidas GATE    SNE     Comidas" (op "NoVacio") y "HornoVacio
GATE    SE      Comidas" (op "Vacio"), y el nuestro
(`./target/release/genmsi gpss fixtures/models/entregaComida__entregaComida.gmdl`)
escribe "HornoConComidas GATE            SNF Comidas" y "HornoVacio      GATE            S
Comidas".

**Cifras.** La tabla del oráculo (`/tmp/fase7/GateS/numbers.md`) tiene una sola métrica y
no coincide: bloque `A_Terminate`, entradas: línea base 59, nuestro 0, intervalo [0, 0],
error relativo 100%, "Match: no". Los reportes lo dicen con más detalle: el original
(`/tmp/fase7/GateS/theirs.report.txt`) tiene la fila de la puerta con `ENTRY COUNT 59`,
`CURRENT COUNT 0` y `RETRY 0` — la puerta abre y las 59 transacciones llegan al
`Terminate` —; el nuestro (`/tmp/fase7/GateS/ours.report.txt`) tiene `ENTRY COUNT 59`,
`CURRENT COUNT 59` y `RETRY 59`: las 59 transacciones quedan esperando en la puerta. Los
dos encabezados difieren también en el número de almacenes: el original dice `STORAGES 0`
y no tiene sección de almacenes; el nuestro dice `STORAGES 1` y agrega la fila
`ALMACEN 1000000 1000000 0 0 0 1 0 0 0 0`, el almacén que nadie declaró. La fila de la
puerta no se puede comparar con la del original porque el nombre de la columna `LOC` no
coincide: el nuestro pone "A_GateS" y el original "GateS", así que el comparador de cifras
no las empareja.

**Veredicto.** con diferencias

**Diferencia concreta.** Nuestra puerta pregunta "¿el almacén no está vacío?" (abre
cuando `is_empty()` es falso) donde el original pregunta "¿el almacén no está lleno?"
(`NoLleno` es `SNF`): con el almacén vacío —o inexistente— el original abre la puerta y
la transacción termina (59 entradas contra nuestras 0), y además el reporte imprime la
letra "SNE" donde el
original imprime "SNF" (el original imprime la misma letra que su GPSS) y crea un
almacén que nadie declaró. Lo arreglaría el core, en
`crates/genmsi-core/src/program.rs` (fn `Compiler::expand`, brazo `"GateS"`, línea 1576:
guardar el operador del original y no sólo el booleano `empty`), en
`crates/genmsi-core/src/sim.rs` (fn `gate_condition`, línea 1574: evaluar "no lleno"
como `remaining() > 0`, "vacío" como `in_use == 0` y "no vacío" como `in_use > 0`) y en
`crates/genmsi-core/src/gpss.rs` (fn `block_statements`, brazo `BlockKind::Gate`, línea
687: escribir "SE", "SNE", "SF", "SNF" según el operador), con el reporte siguiendo la
misma letra en `crates/genmsi-core/src/report.rs` (fn `operand_parts`, brazo
`BlockKind::Gate`, línea 480); lo fijaría un test de `crates/genmsi-core/tests/` que
compile el banco de `GateS` con `op` "NoLleno" y compruebe que la puerta abre (las
entradas del `Terminate` igualan a las del `GateS`) y que la fila del reporte dice
"GATE SNF Almacen", más otro con `op` "NoVacio" que fije "GATE SNE" (el que hoy falla
con la línea base de `entregaComida`). El almacén inventado lo quita otra sesión: la
rama `feature/almacen` (commit 0a4383f "a storage only exists when the model declares it")
hace que un almacén sin `StorageDef` no cree entidad y se lea como vacío; con ese
arreglo desaparece la sección `STORAGE` del reporte y las cifras no cambian, pero la
puerta sigue cerrada porque lo que falla es la condición del operador, no el almacén.

### GateM

**El original.** El original no describe esta clase (`/tmp/fase7/GateM/original.txt`:
"ClassDescrip (vacio: el original no describe esta clase)"); lo que dice de ella es su
macrocode: "GATEM [op:'Operador logico de transaccion.'='Maching'], [Rotulo:'Rotulo a
evaluar.'='Rotulo'], Salida2:'fallo'" — una puerta que compara la etiqueta de la
transacción con la que nombra la clase y la deja pasar cuando coinciden (o cuando no,
con el otro operador).

- **Puertos.** Entrada1, con el rótulo "Ingreso"; Salida1, sin rótulo; Salida2, con el
  rótulo "fallo". La sección `graph` de la clase en `Genmsi_20230711.backup/gClasses.xml`
  da "entradas: ['Entrada1'] salidas: ['Salida1', 'Salida2']"
  (`/tmp/fase7/GateM/original.txt`), y nuestro catálogo trae los mismos
  (`crates/genmsi-format/src/catalogue.rs`, static `GATEM`, línea 768).
- **Propiedades y defaults.**
  - `Rotulo` — `EntityUse`, subType `Label` — default `"Rotulo"`
    (`'Rotulo a evaluar.'='Rotulo'`) — la etiqueta contra la que se compara. Catálogo:
    `("GateM", "Rotulo", "Rotulo")` (`crates/genmsi-format/src/catalogue.rs`, línea 167).
  - `op` — `Listed`, subType `op_Logic_Tran` — default `"Maching"`
    (`'Operador logico de transaccion.'='Maching'`) — la condición. Catálogo:
    `("GateM", "op", "Maching")` (línea 168). La lista del operador en el original
    (`Genmsi_20230711.backup/gmacroconfig.gmcfg`, `listedname="op_Logic_Tran"`) tiene dos:
    "M:Maching | NM:NotMaching".
- **Semántica.** La puerta se compila como `GATE M <etiqueta>` (o `GATE NM <etiqueta>`) y
  la transacción sigue por `Salida1` cuando su etiqueta coincide con la nombrada, por
  `Salida2` ("fallo") cuando no, y espera en el bloque si esa salida no está conectada.
  Nuestro motor: `crates/genmsi-core/src/program.rs`, fn `Compiler::expand`, brazo
  `"GateM"` (línea 1542), lee el rótulo y arma
  `BlockKind::Gate(GateSpec { condition: GateCondition::Logic { switch, state } })`, y
  guarda la clase del bloque como "Gate" (no "GateM") para distinguirla de `GateL`;
  `crates/genmsi-core/src/gpss.rs`, fn `block_statements` (línea 494), brazo
  `BlockKind::Gate` (línea 687), escribe "M" o "NM" cuando `block.class == "Gate"`;
  `crates/genmsi-core/src/sim.rs`, fn `gate_condition` (línea 1574) y el brazo
  `BlockKind::Gate` del despacho (línea 983) comparan la etiqueta de la transacción con
  el nombre resuelto.

**El banco.** Un `Generate` (Media 60, Modificador 30) → un `Match` auxiliar (RotuloMatch
"AAA", RotuloPropio "AAA") → `GateM` (Rotulo "AAA", op "Maching") → `Terminate`, más un
`Clock` (Frecuencia 3600) que compila `GENERATE 3600` + `TERMINATE 1`. La leyenda del
módulo lo dice: "GateM%20%28salidas%20sin%20conectar%3A%20Salida2%3B%20con%20bloque%20auxiliar%3A%20Match%29"
(`/tmp/fase7/GateM/bench.gmdl`): la salida de fallo no está conectada y hace falta un
`Match` para que las transacciones lleguen a la puerta con etiqueta.

**GPSS.** Coincide con el del original: los dos escriben `GateM GATE M AAA`
(`/tmp/fase7/GateM/ours.gpss.txt` contra `/tmp/fase7/GateM/theirs.gpss.txt`), y el
comparador del oráculo lo confirma: `bash scripts/wine-oracle/compare-gpss.sh
/tmp/fase7/GateM/ours.gpss.txt /tmp/fase7/GateM/theirs.gpss.txt` → "el GPSS coincide: 4
bloques y 1 conexiones" (exit 0).

**Cifras.** La tabla del oráculo (`/tmp/fase7/GateM/numbers.md`) tiene dos métricas y una
no coincide: bloque `AAA` (el `Match` auxiliar), entradas: línea base 0, nuestro 61,
intervalo [49.615, 72.385], error relativo "n/a" (la línea base es 0), "Match: no"; y
bloque `A_Terminate`, entradas: línea base 0, nuestro 0, intervalo [0, 0], error
relativo 0%, "Match: yes". La diferencia de la primera métrica no es de la puerta: el
original (`/tmp/fase7/GateM/theirs.report.txt`) cuenta la llegada a la sentencia
`TRANSFER ,AAA` del macro del `Match` (fila "A_Match TRANSFER ,AAA" con `ENTRY COUNT 58`)
y deja la fila del `MATCH` con `ENTRY COUNT 0` y `CURRENT COUNT 58`; el nuestro
(`/tmp/fase7/GateM/ours.report.txt`) tiene una sola fila, "AAA MATCH AAA", con `ENTRY
COUNT 59` y `CURRENT COUNT 59`. La puerta sí se comporta igual en los dos: la fila de la
puerta queda con `ENTRY COUNT 0` y `CURRENT COUNT 0` en el original y en el nuestro
(nuestro la llama "A_GateM"), porque en los dos casos las transacciones quedan atrapadas
en el `Match` y nunca llegan a ella; por eso este banco no llega a ejercitar la
comparación de etiquetas. Dos diferencias más de forma: el original cuenta 7 bloques
(`BLOCKS 7`) porque la sentencia `TRANSFER` del macro del `Match` tiene fila propia, y
nuestro 6; y la columna `LOC` de la puerta dice "A_GateM" donde el original dice "GateM",
por lo que la fila de la puerta no entra en la comparación de cifras.

**Veredicto.** con diferencias

**Diferencia concreta.** El GPSS coincide y la puerta no deja pasar ninguna transacción en
los dos motores, pero nuestro reporte escribe la fila de la puerta como "A_GateM GATE LS
AAA" donde el original escribe "GateM GATE M AAA": la letra "LS" es la de un
`GateL` (el original imprime "M", la de un `GateM`) y el nombre de la columna `LOC` es la
etiqueta del modelo y no el nombre de la sentencia. Lo arreglaría el core, en
`crates/genmsi-core/src/report.rs` (fn `operand_parts`, brazo `BlockKind::Gate`, línea
480: imprimir "M"/"NM" cuando `block.class == "Gate"`, como ya hace
`crates/genmsi-core/src/gpss.rs`, fn `block_statements`, línea 687) y en
`crates/genmsi-core/src/report.rs` (fn `loc_label`, línea 319: usar el nombre que la
sentencia lleva en el
programa —los nombres de `crates/genmsi-core/src/gpss.rs`, fn `Names::new`, línea 65, y
`base_name`, línea 119, que hoy son privados— en vez de la etiqueta del modelo); lo
fijaría un test de `crates/genmsi-core/tests/report_gpss.rs` que compruebe que la fila
del banco de `GateM` dice "GATE M AAA" y que su `LOC` es "GateM". El conteo de entradas
del `Match` auxiliar (0 contra 61) no se arregla aquí: es de la clase `Match`, que tiene
ficha propia en este lote.

### Select

**El original.** El original no describe esta clase (`/tmp/fase7/Select/original.txt`:
"ClassDescrip (vacio: el original no describe esta clase)"); lo que dice de ella es su
macrocode: "SELECT [op:'Operador Máximo o Minimo.'='Minimo'], [Param:'parámetro donde se
guarda el valor seleccionado'='param'], [minimo:'Minimo valor de la selección (Cualquier
SNA).'='1'], [maximo:'Maximo valor de la selección (Cualquier SNA).'='10'],
[ClaseSNA:'Clase de SNA de entidad por el cual se realiza la seleccion'='Q']" — una
selección que elige, de un rango de entidades de una clase, la que tiene el valor más
chico (o más grande) y deja el número de esa entidad en un parámetro.

- **Puertos.** Entrada1, con el rótulo "Ingreso"; Salida1, sin rótulo. La sección `graph`
  de la clase en `Genmsi_20230711.backup/gClasses.xml` da "entradas: ['Entrada1'] salidas:
  ['Salida1']" (`/tmp/fase7/Select/original.txt`): esta clase no tiene salida de fallo, y
  nuestro catálogo trae lo mismo (`crates/genmsi-format/src/catalogue.rs`, static
  `SELECT`, línea 1205).
- **Propiedades y defaults.**
  - `ClaseSNA` — `Listed`, subType `entitySNAclass` — default `"Q"`
    (`'Clase de SNA de entidad por el cual se realiza la seleccion'='Q'`) — la clase de
    entidad que se escanea. Catálogo: `("Select", "ClaseSNA", "Q")`
    (`crates/genmsi-format/src/catalogue.rs`, línea 276).
  - `Param` — `EntityUse`, subType `Parameter` — default `"param"`
    (`'parámetro donde se guarda el valor seleccionado'='param'`) — dónde se guarda el
    valor seleccionado. Catálogo: `("Select", "Param", "param")` (línea 277).
  - `maximo` — `TSNA`, subType `SNA` — default `"10"`
    (`'Maximo valor de la selección (Cualquier SNA).'='10'`) — el último número del rango
    que se escanea. Catálogo: `("Select", "maximo", "10")` (línea 278).
  - `minimo` — `TSNA`, subType `SNA` — default `"1"`
    (`'Minimo valor de la selección (Cualquier SNA).'='1'`) — el primer número del rango.
    Catálogo: `("Select", "minimo", "1")` (línea 279).
  - `op` — `Listed`, subType `op_MaxMin` — default `"Minimo"`
    (`'Operador Máximo o Minimo.'='Minimo'`) — si se busca el mínimo o el máximo.
    Catálogo: `("Select", "op", "Minimo")` (línea 280). La lista del operador en el
    original (`Genmsi_20230711.backup/gmacroconfig.gmcfg`, `listedname="op_MaxMin"`) tiene
    dos: "MAX:Maximo | MIN:Minimo".
- **Semántica.** La sentencia `SELECT MIN|MAX <parámetro>,<mín>,<máx>,,<clase>` recorre
  las entidades numeradas de `<mín>` a `<máx>` de la clase que dice `ClaseSNA`, se queda
  con la de menor (o mayor) valor y escribe su número en el parámetro; si no hay ninguna,
  la transacción sigue por `Salida1` (esta clase no tiene salida de fallo). Nuestro motor:
  `crates/genmsi-core/src/program.rs`, fn `Compiler::expand`, brazo de las clases `Select`
  (línea 1719), arma `BlockKind::Select(SelectSpec { kind, parameter, low, high, mode,
  state, compare_value, missing_to_failure })`, con el modo `Max` sólo cuando `op` es
  "Maximo" y el tipo de entidad según `ClaseSNA` ("Q" cola, "S" almacén, "R" recurso);
  `crates/genmsi-core/src/gpss.rs`, fn `block_statements` (línea 494), brazo
  `BlockKind::Select` (línea 756), escribe `MIN <param>,<mín>,<máx>,,<clase>` (o `MAX`);
  `crates/genmsi-core/src/sim.rs`, fn `select` (línea 1857) recorre el rango con
  `crate::error::numeric_name` y el brazo `BlockKind::Select` del despacho (línea 1260)
  deja el número elegido en el parámetro y manda la transacción por `Salida1`.

**El banco.** Un `Generate` (Media 60, Modificador 30) → `Select` (ClaseSNA "Q", op
"Minimo", Param "param", minimo 1, maximo 10) → `Terminate`, más un `Clock` (Frecuencia
3600) que compila `GENERATE 3600` + `TERMINATE 1`. Ninguna cola está declarada: las diez
colas que la selección escanea (1 a 10) las crean las dos simulaciones al mirarlas, y por
eso los dos reportes las listan.

**GPSS.** Coincide con el del original: los dos escriben `A_Select SELECT MIN param,1,10,,Q`
(`/tmp/fase7/Select/ours.gpss.txt` contra `/tmp/fase7/Select/theirs.gpss.txt`), y el
comparador del oráculo lo confirma: `bash scripts/wine-oracle/compare-gpss.sh
/tmp/fase7/Select/ours.gpss.txt /tmp/fase7/Select/theirs.gpss.txt` → "el GPSS coincide: 3
bloques y 0 conexiones" (exit 0).

**Cifras.** La tabla del oráculo (`/tmp/fase7/Select/numbers.md`) coincide en todo: las
treinta métricas de cola (diez colas por tres métricas: entradas, contenido medio y
tiempo medio) dan línea base 0 contra nuestro 0, error relativo 0%, "Match: yes"; y los
bloques dan `A_Select` 58 contra 61 (error 5%, "Match: yes") y `A_Terminate` 58 contra 61
(error 5%, "Match: yes"), dentro del 20% de tolerancia en conteos. Los dos reportes
traen la sección de colas con las diez colas y todos los números en cero; lo que difiere
es la forma: nuestro reporte las ordena por texto ("1, 10, 2, 3, ..." donde el original las
ordena de 1 a 10), escribe "0" donde el original escribe "0,000" y "0.000"
(`crates/genmsi-core/src/report.rs`, fn `number`, línea 23), y agrega una línea "SEMILLA"
que el original no tiene (el original cierra con una tabla "FEC_XACT_NUMBER").

**Veredicto.** verificada

**Diferencia concreta.** El GPSS coincide palabra por palabra y todas las cifras del banco
están dentro de las tolerancias del oráculo; lo que queda son diferencias de forma en el
reporte (el orden de las colas, los decimales enteros y la línea "SEMILLA") y dos cosas
que este banco no mide: el valor que el original deja en el parámetro (su reporte no lo
dice: la tabla "FEC_XACT_NUMBER" del final no trae el nombre del parámetro) y las
clases de entidad que el motor no soporta. La lista `entitySNAclass` del original
(`Genmsi_20230711.backup/gmacroconfig.gmcfg`) tiene 38 clases ("Q", "S", "R", "F", "P",
"FN", ...) y nuestro motor sólo acepta "Q", "S" y "R": cualquier otra hace fallar la
compilación con "unsupported SELECT entity class" (`crates/genmsi-core/src/program.rs`, fn
`Compiler::expand`, brazo de las clases `Select`, línea 1719, el `match` de `ClaseSNA`).
Lo arreglaría esa misma fn, con un test de `crates/genmsi-core/tests/` que compile un
`Select` con `ClaseSNA` "F" y compruebe que el GPSS sale `SELECT MIN param,1,10,,F`; con
este banco no se puede medir.

### SelectF

**El original.** El original no describe esta clase (`/tmp/fase7/SelectF/original.txt`:
"ClassDescrip (vacio: el original no describe esta clase)"); lo que dice de ella es su
macrocode: "SELECTF [op:'Operador Logico de Facility.'='Libre'], [Param:'parámetro donde se
guarda el valor seleccionado'='param'], [minimo:'Minimo valor de la selección (Cualquier
SNA).'='1'], [maximo:'Maximo valor de la selección (Cualquier SNA).'='10'],
Salida2:'No selec'" — una selección que elige el primer recurso del rango que esté en el
estado que dice el operador y deja su número en un parámetro.

- **Puertos.** Entrada1, con el rótulo "Ingreso"; Salida1, sin rótulo; Salida2, con el
  rótulo "No selec". La sección `graph` de la clase en `Genmsi_20230711.backup/gClasses.xml`
  da "entradas: ['Entrada1'] salidas: ['Salida1', 'Salida2']"
  (`/tmp/fase7/SelectF/original.txt`), y nuestro catálogo trae los mismos
  (`crates/genmsi-format/src/catalogue.rs`, static `SELECTF`, línea 1223).
- **Propiedades y defaults.**
  - `Param` — `EntityUse`, subType `Parameter` — default `"param"`
    (`'parámetro donde se guarda el valor seleccionado'='param'`) — dónde se guarda el
    valor seleccionado. Catálogo: `("SelectF", "Param", "param")`
    (`crates/genmsi-format/src/catalogue.rs`, línea 282).
  - `maximo` — `TSNA`, subType `SNA` — default `"10"`
    (`'Maximo valor de la selección (Cualquier SNA).'='10'`) — el último número del rango
    que se escanea. Catálogo: `("SelectF", "maximo", "10")` (línea 283).
  - `minimo` — `TSNA`, subType `SNA` — default `"1"`
    (`'Minimo valor de la selección (Cualquier SNA).'='1'`) — el primer número del rango.
    Catálogo: `("SelectF", "minimo", "1")` (línea 284).
  - `op` — `Listed`, subType `op_Logic_Faci` — default `"Libre"`
    (`'Operador Logico de Facility.'='Libre'`) — la condición del recurso. Catálogo:
    `("SelectF", "op", "Libre")` (línea 285). La lista del operador en el original
    (`Genmsi_20230711.backup/gmacroconfig.gmcfg`, `listedname="op_Logic_Faci"`) tiene seis:
    "NU:Libre | U:Usada | I:Interrumpida | NI:NoInterrumpida | FNV:NoDisponible |
    FV:Disponible"; nuestro motor sólo implementa el primero.
- **Semántica.** La sentencia `SELECT <letra> <parámetro>,<mín>,<máx>` recorre los
  recursos numerados de `<mín>` a `<máx>`, se queda con el primero que esté en el estado
  que dice el operador, deja su número en el parámetro y sigue por `Salida1`; si no hay
  ninguno, la transacción sale por `Salida2` ("No selec"). Nuestro motor:
  `crates/genmsi-core/src/program.rs`, fn `Compiler::expand`, brazo de las clases `Select`
  (línea 1719), para "SelectF" pone `mode = SelectMode::FirstFree` y `kind =
  RefKind::Facility`, sin mirar el valor de `op`, y marca `missing_to_failure`;
  `crates/genmsi-core/src/gpss.rs`, fn `block_statements` (línea 494), brazo
  `BlockKind::Select` (línea 756), escribe `NU <parámetro>,<mín>,<máx>` para el modo
  `FirstFree` (la letra dice sola la clase, y por eso no lleva el operando de clase);
  `crates/genmsi-core/src/sim.rs`, fn `select` (línea 1857), en el modo `FirstFree` se
  queda con el primer recurso `available && !busy`, y el brazo `BlockKind::Select` del
  despacho (línea 1260) manda por `Salida2` cuando no hay ninguno.

**El banco.** Un `Generate` (Media 60, Modificador 30) → `SelectF` (Param "param", minimo
1, maximo 10, op "Libre") → `Terminate`, más un `Clock` (Frecuencia 3600) que compila
`GENERATE 3600` + `TERMINATE 1`. La salida de fallo no está conectada ("SelectF%20%28salidas%20sin%20conectar%3A%20Salida2%29",
`/tmp/fase7/SelectF/bench.gmdl`) y ningún recurso está declarado: el que la selección
mira (el número 1) lo crean las dos simulaciones al mirarlo.

**GPSS.** Coincide con el del original: los dos escriben `A_SelectF SELECT NU param,1,10`
(`/tmp/fase7/SelectF/ours.gpss.txt` contra `/tmp/fase7/SelectF/theirs.gpss.txt`), y el
comparador del oráculo lo confirma: `bash scripts/wine-oracle/compare-gpss.sh
/tmp/fase7/SelectF/ours.gpss.txt /tmp/fase7/SelectF/theirs.gpss.txt` → "el GPSS coincide:
3 bloques y 0 conexiones" (exit 0).

**Cifras.** La tabla del oráculo (`/tmp/fase7/SelectF/numbers.md`) coincide en las cuatro
métricas: recurso 1, utilización 0 contra 0 (error 0%, "Match: yes") y tiempo medio 0
contra 0 (error 0%, "Match: yes"); bloque `A_SelectF`, entradas 65 contra 61 (error 6%,
"Match: yes"); bloque `A_Terminate`, entradas 65 contra 61 (error 6%, "Match: yes"),
dentro del 20% de tolerancia en conteos. Los dos reportes coinciden en la fila del
recurso y en la fila de la sentencia: nuestro dice "SELECT   NU      param,1,10" y el
original "SELECT   NU      param,1,10"; sólo difieren los decimales de la sección de
recursos ("0" donde el original escribe "0,000" y "0.000",
`crates/genmsi-core/src/report.rs`, fn `number`, línea 23).

**Veredicto.** verificada

**Diferencia concreta.** El GPSS coincide palabra por palabra, la fila del reporte es
idéntica y todas las cifras del banco están dentro de las tolerancias; lo que este banco
no mide es que nuestro motor sólo implementa el operador por defecto: los otros cinco de
la lista `op_Logic_Faci` del original ("U:Usada", "I:Interrumpida", "NI:NoInterrumpida",
"FNV:NoDisponible", "FV:Disponible") se perderían, porque el operador no se lee
(`crates/genmsi-core/src/program.rs`, fn `Compiler::expand`, brazo de las clases `Select`,
línea 1719, donde "SelectF" pone `SelectMode::FirstFree` sin mirar `op`) y el GPSS escribe
siempre "NU" (`crates/genmsi-core/src/gpss.rs`, fn `block_statements`, línea 756). Lo
arreglaría el core con un operador por estado del recurso, como se hizo con
`storage_state` para los almacenes (`crates/genmsi-core/src/program.rs`, línea 2674), y lo
fijaría un test de `crates/genmsi-core/tests/gpss.rs` que compile un `SelectF` con `op`
"Usada" y compruebe que el GPSS sale `A_SelectF SELECT U param,1,10`.

### SelectS

**El original.** El original no describe esta clase (`/tmp/fase7/SelectS/original.txt`:
"ClassDescrip (vacio: el original no describe esta clase)"); lo que dice de ella es su
macrocode: "SELECTS [op:'Operador Logico de Storage.'='NoLleno'], [Param:'parámetro donde
se guarda el valor seleccionado'='param'], [minimo:'Minimo valor de la selección
(Cualquier SNA).'='1'], [maximo:'Maximo valor de la selección (Cualquier SNA).'='10'],
Salida2:'No selec'" — una selección que elige el primer almacén del rango que esté en el
estado que dice el operador y deja su número en un parámetro.

- **Puertos.** Entrada1, con el rótulo "Ingreso"; Salida1, sin rótulo; Salida2, con el
  rótulo "No selec". La sección `graph` de la clase en `Genmsi_20230711.backup/gClasses.xml`
  da "entradas: ['Entrada1'] salidas: ['Salida1', 'Salida2']"
  (`/tmp/fase7/SelectS/original.txt`), y nuestro catálogo trae los mismos
  (`crates/genmsi-format/src/catalogue.rs`, static `SELECTS`, línea 1259).
- **Propiedades y defaults.**
  - `Param` — `EntityUse`, subType `Parameter` — default `"param"`
    (`'parámetro donde se guarda el valor seleccionado'='param'`) — dónde se guarda el
    valor seleccionado. Catálogo: `("SelectS", "Param", "param")`
    (`crates/genmsi-format/src/catalogue.rs`, línea 292).
  - `maximo` — `TSNA`, subType `SNA` — default `"10"`
    (`'Maximo valor de la selección (Cualquier SNA).'='10'`) — el último número del rango
    que se escanea. Catálogo: `("SelectS", "maximo", "10")` (línea 293).
  - `minimo` — `TSNA`, subType `SNA` — default `"1"`
    (`'Minimo valor de la selección (Cualquier SNA).'='1'`) — el primer número del rango.
    Catálogo: `("SelectS", "minimo", "1")` (línea 294).
  - `op` — `Listed`, subType `op_Logic_Stor` — default `"NoLleno"`
    (`'Operador Logico de Storage.'='NoLleno'`) — el estado del almacén. Catálogo:
    `("SelectS", "op", "NoLleno")` (línea 295). La lista del operador en el original
    (`Genmsi_20230711.backup/gmacroconfig.gmcfg`, `listedname="op_Logic_Stor"`) tiene seis:
    "SF:Lleno | SNF:NoLleno | SE:Vacio | SNE:NoVacio | SV:Disponible | SNV:NoDisponible";
    nuestro motor sólo distingue dos.
- **Semántica.** La sentencia `SELECT <letra> <parámetro>,<mín>,<máx>` recorre los
  almacenes numerados de `<mín>` a `<máx>`, se queda con el primero que esté en el estado
  que dice el operador, deja su número en el parámetro y sigue por `Salida1`; si no hay
  ninguno, la transacción sale por `Salida2` ("No selec"). Nuestro motor:
  `crates/genmsi-core/src/program.rs`, fn `Compiler::expand`, brazo de las clases `Select`
  (línea 1719), para "SelectS" pone `kind = RefKind::Storage` y
  `state = Some(storage_state(&op))`, donde `storage_state` (línea 2674) sólo conoce dos
  estados: "Lleno" es lleno y cualquier otro operador es "no lleno";
  `crates/genmsi-core/src/gpss.rs`, fn `block_statements` (línea 494), brazo
  `BlockKind::Select` (línea 756), escribe "SF <parámetro>,<mín>,<máx>" o
  "SNF <parámetro>,<mín>,<máx>" según ese estado, sin operando de clase (la letra ya la
  dice); `crates/genmsi-core/src/sim.rs`, fn `select` (línea 1857), prueba
  `remaining() <= 0` para el lleno y `remaining() > 0` para el que no lo está, y el brazo
  `BlockKind::Select` del despacho (línea 1260) manda por `Salida2` cuando no encuentra
  ninguno.

**El banco.** Un `Generate` (Media 60, Modificador 30) → `SelectS` (Param "param", minimo
1, maximo 10, op "NoLleno") → `Terminate`, más un `Clock` (Frecuencia 3600) que compila
`GENERATE 3600` + `TERMINATE 1`. La salida de fallo no está conectada ("SelectS%20%28salidas%20sin%20conectar%3A%20Salida2%29",
`/tmp/fase7/SelectS/bench.gmdl`) y ningún almacén está declarado: los diez almacenes que la
selección escanea (1 a 10) no los declara nadie.

**GPSS.** Coincide con el del original: los dos escriben `A_SelectS SELECT SNF param,1,10`
(`/tmp/fase7/SelectS/ours.gpss.txt` contra `/tmp/fase7/SelectS/theirs.gpss.txt`), y el
comparador del oráculo lo confirma: `bash scripts/wine-oracle/compare-gpss.sh
/tmp/fase7/SelectS/ours.gpss.txt /tmp/fase7/SelectS/theirs.gpss.txt` → "el GPSS coincide:
3 bloques y 0 conexiones" (exit 0).

**Cifras.** La tabla del oráculo (`/tmp/fase7/SelectS/numbers.md`) tiene dos métricas y las
dos coinciden: bloque `A_SelectS`, entradas 58 contra 61 (error 5%, "Match: yes") y bloque
`A_Terminate`, entradas 58 contra 61 (error 5%, "Match: yes"), dentro del 20% de
tolerancia en conteos. Pero el texto del reporte no coincide: nuestro
(`/tmp/fase7/SelectS/ours.report.txt`) escribe la fila de la sentencia como
"SELECT   MIN     param,1,10,,S" donde el original (`/tmp/fase7/SelectS/theirs.report.txt`)
escribe "SELECT   SNF     param,1,10" — ni la letra ni los operandos; y nuestro reporte
agrega una sección de almacenes que el original no tiene: el original dice "STORAGES 0"
en el encabezado y no lista ninguno, el nuestro dice "STORAGES 1" y lista
"1  1000000 1000000 0 0 0 1 0 0 0 0", el almacén que la selección se inventó al mirar un
almacén que nadie declaró.

**Veredicto.** con diferencias

**Diferencia concreta.** El GPSS coincide y las cifras del banco están dentro de las
tolerancias, pero el reporte no dice lo mismo que el original: nuestra fila de la
sentencia dice "MIN param,1,10,,S" donde el original dice "SNF param,1,10" (el reporte
imprime la letra del modo máximo/mínimo y el operando de clase, que para una selección
por estado no existen), y nuestro motor crea el almacén que nadie declaró. Lo
arreglaría el core, en `crates/genmsi-core/src/report.rs` (fn `operand_parts`, brazo
`BlockKind::Select`, línea 559: tomar la letra y los operandos de la misma fuente que
`crates/genmsi-core/src/gpss.rs`, fn `block_statements`, línea 756 — "LS"/"LR"/"SF"/"SNF"
y sin operando de clase) y en `crates/genmsi-core/src/program.rs` (fn `storage_state`,
línea 2674: los seis operadores del original, para que "Vacio" sea "vacío" (`SE`) y
"NoVacio" sea "no vacío" (`SNE`), y no "no lleno"); lo fijaría un test de
`crates/genmsi-core/tests/report_gpss.rs` que compruebe que la fila del banco de
`SelectS` dice "SNF param,1,10", más uno de `crates/genmsi-core/tests/gpss.rs` que
compile un `SelectS` con `op` "Vacio" y compruebe que el GPSS sale
"A_SelectS SELECT SE param,1,10". El almacén inventado lo quita otra sesión: la rama
`feature/almacen` (commit 0a4383f "a storage only exists when the model declares it")
hace que un almacén sin `StorageDef` no cree entidad y se lea como vacío y no lleno
(`crates/genmsi-core/src/sim.rs`, fn `select`, línea 1857, y la nueva fn
`storage_capacity`); con ese arreglo desaparece la sección `STORAGE` del reporte, la
selección sigue eligiendo el almacén 1 y las cifras no cambian, pero la fila de la
sentencia sigue diciendo "MIN param,1,10,,S" mientras el original dice "SNF param,1,10".

### SelectL

**El original.** El original no describe esta clase (`/tmp/fase7/SelectL/original.txt`:
"ClassDescrip (vacio: el original no describe esta clase)"); lo que dice de ella es su
macrocode: "SELECTL [op:'Operador Logico de LogicSwich.'='Set'], [Param:'parámetro donde se
guarda el valor seleccionado'='param'], [minimo:'Minimo valor de la selección (Cualquier
SNA).'='1'], [maximo:'Maximo valor de la selección (Cualquier SNA).'='10'],
Salida2:'No selec'" — una selección que elige la primera llave lógica del rango que esté
en el estado que dice el operador y deja su número en un parámetro.

- **Puertos.** Entrada1, con el rótulo "Ingreso"; Salida1, sin rótulo; Salida2, con el
  rótulo "No selec". La sección `graph` de la clase en `Genmsi_20230711.backup/gClasses.xml`
  da "entradas: ['Entrada1'] salidas: ['Salida1', 'Salida2']"
  (`/tmp/fase7/SelectL/original.txt`), y nuestro catálogo trae los mismos
  (`crates/genmsi-format/src/catalogue.rs`, static `SELECTL`, línea 1241).
- **Propiedades y defaults.**
  - `Param` — `EntityUse`, subType `Parameter` — default `"param"`
    (`'parámetro donde se guarda el valor seleccionado'='param'`) — dónde se guarda el
    valor seleccionado. Catálogo: `("SelectL", "Param", "param")`
    (`crates/genmsi-format/src/catalogue.rs`, línea 288).
  - `maximo` — `TSNA`, subType `SNA` — default `"10"`
    (`'Maximo valor de la selección (Cualquier SNA).'='10'`) — el último número del rango
    que se escanea. Catálogo: `("SelectL", "maximo", "10")` (línea 289).
  - `minimo` — `TSNA`, subType `SNA` — default `"1"`
    (`'Minimo valor de la selección (Cualquier SNA).'='1'`) — el primer número del rango.
    Catálogo: `("SelectL", "minimo", "1")` (línea 290).
  - `op` — `Listed`, subType `op_Logic_Swich` — default `"Set"`
    (`'Operador Logico de LogicSwich.'='Set'`) — el estado de la llave. Catálogo:
    `("SelectL", "op", "Set")` (línea 291). La lista del operador en el original
    (`Genmsi_20230711.backup/gmacroconfig.gmcfg`, `listedname="op_Logic_Swich"`) tiene dos:
    "LS:Set | LR:Reset".
- **Semántica.** La sentencia `SELECT LS|LR <parámetro>,<mín>,<máx>` recorre las llaves
  lógicas numeradas de `<mín>` a `<máx>`, se queda con la primera que esté en el estado
  que dice el operador, deja su número en el parámetro y sigue por `Salida1`; si no hay
  ninguna, la transacción sale por `Salida2` ("No selec"). Nuestro motor:
  `crates/genmsi-core/src/program.rs`, fn `Compiler::expand`, brazo de las clases `Select`
  (línea 1719), para "SelectL" pone `kind = RefKind::LogicSwitch` y
  `state = Some(SelectState::LogicSet)`, salvo que `op` sea "reset", en cuyo caso es
  `LogicReset`; `crates/genmsi-core/src/gpss.rs`, fn `block_statements` (línea 494), brazo
  `BlockKind::Select` (línea 756), escribe "LS <parámetro>,<mín>,<máx>" o
  "LR <parámetro>,<mín>,<máx>" según ese estado, sin operando de clase (la letra ya la
  dice); `crates/genmsi-core/src/sim.rs`, fn `select` (línea 1857), prueba el valor de la
  llave con `entities.logic(&name)`, y el brazo `BlockKind::Select` del despacho (línea
  1260) manda por `Salida2` cuando no encuentra ninguna.

**El banco.** Un `Generate` (Media 60, Modificador 30) → `SelectL` (Param "param", minimo
1, maximo 10, op "Set") → `Terminate`, más un `Clock` (Frecuencia 3600) que compila
`GENERATE 3600` + `TERMINATE 1`. La salida de fallo no está conectada ("SelectL%20%28salidas%20sin%20conectar%3A%20Salida2%29",
`/tmp/fase7/SelectL/bench.gmdl`) y ninguna llave está declarada: las diez llaves que la
selección escanea (1 a 10) las crean las dos simulaciones al mirarlas.

**GPSS.** Coincide con el del original: los dos escriben `A_SelectL SELECT LS param,1,10`
(`/tmp/fase7/SelectL/ours.gpss.txt` contra `/tmp/fase7/SelectL/theirs.gpss.txt`), y el
comparador del oráculo lo confirma: `bash scripts/wine-oracle/compare-gpss.sh
/tmp/fase7/SelectL/ours.gpss.txt /tmp/fase7/SelectL/theirs.gpss.txt` → "el GPSS coincide:
3 bloques y 0 conexiones" (exit 0). El original escribe en el GPSS una sección
"LOGICSWITCH VALUE RETRY" con las llaves que el modelo declara
(`fixtures/outputs/Parcial_6-11-23__Veron_-_Parcial.gpss.txt`, línea 2) —nuestro motor no
la escribe tampoco, pero es de otra clase—, y en este banco no se declara ninguna llave,
así que los dos GPSS salen iguales.

**Cifras.** La tabla del oráculo (`/tmp/fase7/SelectL/numbers.md`) tiene dos métricas y las
dos coinciden: bloque `A_SelectL`, entradas 60 contra 61 (error 2%, "Match: yes") y bloque
`A_Terminate`, entradas 60 contra 61 (error 2%, "Match: yes"), dentro del 20% de tolerancia
en conteos. El texto del reporte, en cambio, no coincide: nuestro
(`/tmp/fase7/SelectL/ours.report.txt`) escribe la fila de la sentencia como
"SELECT   MIN     param,1,10,,LS" donde el original (`/tmp/fase7/SelectL/theirs.report.txt`)
escribe "SELECT   LS      param,1,10"; y el original cierra el reporte con una sección
"LOGICSWITCH VALUE RETRY" que lista las diez llaves que la selección creó (todas con
valor 0 y reintento 0), que nuestro reporte no tiene.

**Veredicto.** con diferencias

**Diferencia concreta.** El GPSS coincide y las cifras del banco están dentro de las
tolerancias, pero el reporte no dice lo mismo que el original: nuestra fila de la
sentencia dice "MIN param,1,10,,LS" donde el original dice "LS param,1,10", y a nuestro
reporte le falta la sección "LOGICSWITCH VALUE RETRY" con las diez llaves que la
selección creó (nuestro motor las crea igual: `crates/genmsi-core/src/stat.rs`, fn
`Entities::logic`, línea 558, inserta la llave al primer uso). Lo arreglaría el core, en
`crates/genmsi-core/src/report.rs` (fn `operand_parts`, brazo `BlockKind::Select`, línea
559: tomar la letra y los operandos de la misma fuente que
`crates/genmsi-core/src/gpss.rs`, fn `block_statements`, línea 756 — "LS"/"LR" y sin
operando de clase) y en `crates/genmsi-core/src/report.rs` (fn `render`, línea 35: la
sección de las llaves lógicas, con el mismo formato que el original: "LOGICSWITCH VALUE
RETRY" y una fila por llave); lo fijaría un test de
`crates/genmsi-core/tests/report_gpss.rs` que compruebe que la fila del banco de
`SelectL` dice "LS param,1,10" y que el reporte tiene la sección con las diez llaves.

### SelectSNA

**El original.** La clase no tiene `ClassDescrip` en `Genmsi_20230711.backup/gClasses.xml`
(`/tmp/fase7/SelectSNA/original.txt`): lo que dice de ella es su macrocode,
`SELECTENT [op],[Param],[minimo],[maximo],[valorComp],[ClaseSNA], Salida2:'No selec' OUT Salida1`,
y el manual del simulador que acompaña al original (`Genmsi_20230711.backup/VISUALSIS.HLP`):
"Un bloque SELECT elige una entidad y coloca su número de entidad en un parámetro de la transacción".

- **Puertos.** Entrada `Entrada1` (rótulo "Ingreso"); salidas `Salida1` (sin rótulo) y
  `Salida2` (rótulo "No selec").
- **Propiedades y defaults.** `ClaseSNA` (tipo `Listed`, sub-tipo `entitySNAclass`; default del
  macrocode `'Q'`): la clase de SNA de la entidad por la que se selecciona. `Param` (tipo
  `EntityUse`, sub-tipo `Parameter`; default `'param'`): el parámetro donde se guarda el número de
  la entidad elegida. `minimo` y `maximo` (tipo `TSNA`, sub-tipo `SNA`; defaults `'1'` y `'10'`):
  el rango de números de entidad que se recorren. `op` (tipo `Listed`, sub-tipo `op_Relac`;
  default `'='`): el operador relacional. `valorComp` (tipo `TSNA`, sub-tipo `SNA`; default `'*1'`):
  el valor con el que se compara el atributo de la entidad.
- **Semántica.** El macrocode compila a `SELECT E <param>,<minimo>,<maximo>,<valorComp>,<ClaseSNA>,`:
  el operador condicional `E` de la sentencia SELECT (el manual: "Cuando un operador condicional es
  usado, los operandos A, B, C, E y usualmente D son usados... el operador condicional especifica
  la relación entre el operando D y E que debe influenciar por la entidad a seleccionar"), o sea que
  se recorren las entidades `minimo..maximo` y se elige la primera cuyo atributo de la clase de SNA
  sea igual a `valorComp`; el número de la entidad elegida va al parámetro `Param`. Cuando no se
  elige ninguna, la transacción sigue por la primera salida (`Salida1`). Nuestro motor compila la
  clase en `crates/genmsi-core/src/program.rs` (rama `"Select" | "SelectF" | "SelectL" | "SelectS" |
  "SelectSNA"`): deja `state = SelectState::EntityEqual`, `compare_value = valorComp` y
  `missing_to_failure = false` (la transacción sigue por `Salida1` si no hay nada que elegir), y
  `crates/genmsi-core/src/sim.rs` (`fn select`) compara el atributo de la entidad con el valor
  comparado.

**El banco.** `Generate` (Media 60, Modificador 30) → `SelectSNA` (ClaseSNA `Q`, Param `param`,
minimo 1, maximo 10, op `=`, valorComp `*1`) → `Terminate`; más un `Clock` (Frecuencia 3600) y su
`TERMINATE 1`. La salida `Salida2` de la clase está sin conectar (el rótulo del módulo lo dice:
"SelectSNA (salidas sin conectar: Salida2)"). El validador dice `status: ok`, `blocks: 5`,
`diagnostics: none` (`/tmp/fase7/SelectSNA/lint.txt`).

**GPSS.** El original produce

```
                GENERATE        60,30
A_SelectSNA     SELECT   E       param,1,10,*1,Q,
A_Terminate     TERMINATE
                GENERATE        3600
                TERMINATE       1
```

y el nuestro las mismas sentencias con otra repartición de los espacios
(`/tmp/fase7/SelectSNA/ours.gpss.txt`): `scripts/wine-oracle/compare-gpss.sh` dice "el GPSS coincide:
3 bloques y 0 conexiones".

**Cifras.** La tabla del oráculo (`/tmp/fase7/SelectSNA/numbers.md`) coincide en todo lo que mide:
`queue 1` (entries, average_content, average_time) 0 contra 0; bloque `A_SelectSNA` 61 contra 61
(error 0%, dentro del intervalo [49.615, 72.385]); bloque `A_Terminate` 61 contra 61 (error 0%). El
reporte del original y el nuestro coinciden en las cifras de la tabla de bloques y de la cola
(`/tmp/fase7/SelectSNA/theirs.report.txt`, `ours.report.txt`), salvo en la forma de la línea del
SELECT: el original escribe `SELECT   E       param,1,10,*1,Q,` y el nuestro
`SELECT   MIN     param,1,10,,Q`. El banco no puede distinguir el comportamiento de la comparación
(las 61 transacciones van por la primera salida en los dos casos, tanto si se elige la entidad 1
como si no se elige ninguna), así que la regla de comparación queda sin verificar.

**Veredicto.** con diferencias

**Diferencia concreta.** El GPSS y las cifras coinciden, pero el reporte de nuestro motor imprime la
línea del SELECTSNA como un `SELECT MIN` sin el modo `E` ni el valor comparado `*1`, donde el
original escribe `SELECT E param,1,10,*1,Q,`. El arreglo va en
`crates/genmsi-core/src/report.rs`, en `fn operand_parts` (rama `BlockKind::Select`): tiene que
mirar `spec.state == Some(SelectState::EntityEqual)` y escribir el modo `E` y los operandos
`param,minimo,maximo,valorComp,clase` como hace `crates/genmsi-core/src/gpss.rs` (rama
`BlockKind::Select`), dejando el último operando vacío (el original lo escribe: la coma final).
La prueba que lo fijaría: una fila del reporte del banco `SelectSNA` con
`(SELECT, E, param,1,10,*1,Q,)` al lado de las filas de `crates/genmsi-core/tests/report_gpss.rs`
(las de `SELECT` de `TP4__supermercado`).

### Match

**El original.** La clase no tiene `ClassDescrip` en `Genmsi_20230711.backup/gClasses.xml`
(`/tmp/fase7/Match/original.txt`): lo que dice de ella es su macrocode,
`MATCH [RotuloPropio:'Rótulo propio.'='AAA'], [RotuloMatch:'Rótulo con el cual sincronizar'='BBB'] OUT Salida1`,
y el manual del simulador (`Genmsi_20230711.backup/VISUALSIS.HLP`): "Un bloque MATCH causa a
transacciones esperar por otras transacciones"; el conjugado es "Número de bloque. Conjugar un
bloque MATCH para ser testeado por transacciones iguales (mismo conjunto de reunión)".

- **Puertos.** Entrada `Entrada1` (rótulo "Entrada1"); salida `Salida1` (sin rótulo).
- **Propiedades y defaults.** `RotuloPropio` (tipo `EntityDef`, sub-tipo `Label`; default del
  macrocode `'AAA'`): el rótulo propio de la clase, el nombre que el macrocode le pone a la
  sentencia `MATCH` y el destino del `TRANSFER` que lo precede. `RotuloMatch` (tipo `EntityUse`,
  sub-tipo `Label`; default `'BBB'`): el rótulo del bloque `MATCH` conjugado con el que se
  sincroniza. El banco usa `AAA` en las dos (el valor por defecto del conjugado es `BBB`).
- **Semántica.** El macrocode escribe dos sentencias: `A_Match TRANSFER ,<RotuloPropio>` y
  `<RotuloPropio> MATCH <RotuloMatch>`: la transacción entra por el `TRANSFER` y llega al bloque
  `MATCH`, donde espera en su cadena de match. El manual: "Si el bloque conjugado MATCH contiene una
  transacción (en su cadena match) del mismo conjunto de reunión activa, la transacción relacionada es
  removida de la cadena de match"; si no encuentra ninguna, "esta descansa en la cadena match del
  bloque MATCH". Nuestro motor compila la clase en un solo bloque (`crates/genmsi-core/src/program.rs`,
  rama `"Match"`: `BlockKind::Match(MatchSpec { conjugate })` con la etiqueta `RotuloPropio`) y
  `crates/genmsi-core/src/gpss.rs` (rama `BlockKind::Match`) escribe las dos sentencias desde ese
  bloque; el emparejamiento está en `crates/genmsi-core/src/sim.rs` (rama `BlockKind::Match`), que
  toma el que espera más antiguo en el bloque conjugado sin mirar el conjunto de reunión.

**El banco.** `Generate` (Media 60, Modificador 30) → `Match` (RotuloPropio `AAA`, RotuloMatch
`AAA`: el conjugado es el propio bloque) → `Terminate`; más un `Clock` (Frecuencia 3600) y su
`TERMINATE 1`. El validador dice `status: ok`, `blocks: 5`, `diagnostics: none`
(`/tmp/fase7/Match/lint.txt`).

**GPSS.** El original produce

```
                GENERATE        60,30
A_Match         TRANSFER        ,AAA
AAA             MATCH           AAA
A_Terminate     TERMINATE
                GENERATE        3600
                TERMINATE       1
```

y el nuestro lo mismo (`/tmp/fase7/Match/ours.gpss.txt`):
`scripts/wine-oracle/compare-gpss.sh` dice "el GPSS coincide: 3 bloques y 1 conexiones" (el
`TRANSFER` cuenta como conexión en las dos partes).

**Cifras.** La tabla del oráculo (`/tmp/fase7/Match/numbers.md`) tiene una métrica que no coincide:
bloque `AAA` entries, base 0 contra 61 nuestros (error no calculable, `Match = no`). La otra
coincide: `A_Terminate` 0 contra 0. La causa está en la tabla de bloques: el reporte del original
tiene seis líneas y el nuestro cinco, porque el original cuenta el `TRANSFER` del macrocode como un
bloque más (`A_Match TRANSFER ,AAA`: 66 entradas, 66 pendientes) y las entradas que le tocarían
al `MATCH` no le tocan: su línea dice `MATCH  AAA` con 0 entradas y 66 pendientes
(`/tmp/fase7/Match/theirs.report.txt`). En el nuestro no hay línea de `TRANSFER` y el `MATCH` se
queda con las 55 entradas y las 55 pendientes (`ours.report.txt`): son las mismas transacciones
contadas en otra línea. El original añade además al final la tabla `FEC_XACT_NUMBER` de
transacciones y eventos, que nuestro reporte no produce.

**Veredicto.** con diferencias

**Diferencia concreta.** Nuestro reporte no tiene la línea del `TRANSFER` que el macrocode de la
clase escribe delante del `MATCH`, así que la tabla de bloques tiene 5 líneas donde el original tiene
6 y las entradas se atribuyen al `MATCH` (61) en vez de al `TRANSFER` (66): de ahí el
`Match = no` de la tabla del oráculo. El arreglo va en `crates/genmsi-core/src/program.rs` (rama
`"Match"`): la clase tendría que expandirse en dos bloques, un `Transfer` (al rótulo
`RotuloPropio`) y un `Match` (al `RotuloMatch`), como hace el original, y
`crates/genmsi-core/src/gpss.rs` (rama `BlockKind::Match`) dejaría de escribir el `TRANSFER` a mano.
La prueba que lo fijaría: el número de líneas de la tabla de bloques del banco `Match` (6) y las
entradas de la línea `A_Match`; el bloque `Transfer` que se cree tendría que tomar el rótulo del
puerto de entrada de la clase (`A_Match`), que es el que el original le da a esa línea, para que el
oráculo (`crates/genmsi-core/src/oracle.rs`) pueda emparejarla con la base. Aparte, la regla del emparejamiento no es la del original: el
original exige "mismo conjunto de reunión" y el nuestro empareja el que espera más antiguo en el
bloque conjugado (`crates/genmsi-core/src/sim.rs`, rama `BlockKind::Match`); este banco no lo
mide porque su conjugado es el propio bloque y cada transacción generada es un conjunto de reunión
distinto, y el comportamiento actual está fijado por
`crates/genmsi-core/tests/match_pairs.rs` (`every_transaction_is_paired`), que empareja
transacciones de generadores distintos.

### Assemble

**El original.** La clase no tiene `ClassDescrip` en `Genmsi_20230711.backup/gClasses.xml`
(`/tmp/fase7/Assemble/original.txt`): lo que dice de ella es su macrocode,
`COLLECT [Cantidad:'Cantidad de transacciones, del mismo grupo de reunión, a combinar'='1'] OUT Salida1`,
y el manual del simulador (`Genmsi_20230711.backup/VISUALSIS.HLP`): "Un bloque ASSEMBLE es usado
para sincronización. Las transacciones relacionadas son destruidas".

- **Puertos.** Entrada `Entrada1` (rótulo "Convina"); salida `Salida1` (sin rótulo).
- **Propiedades y defaults.** `Cantidad` (tipo `TSNA`, sub-tipo `SNA`; default del macrocode `'1'`):
  la cantidad de transacciones del mismo grupo de reunión que hay que combinar. El original no
  declara ningún otro operando.
- **Semántica.** El macrocode compila a `ASSEMBLE <Cantidad>`: la primera transacción del conjunto
  de reunión espera; "Cuando otra transacción del mismo conjunto de reunión ingresa al bloque, [esta]
  es destruida y la transacción que esperaba es permitida para que continúe"; al llegar la cuenta a
  cero la que esperaba sale por el bloque siguiente. Nuestro motor lo compila en
  `crates/genmsi-core/src/program.rs` (rama `"Assemble" | "Gather"`, `BlockKind::Assemble`) y lo
  ejecuta en `crates/genmsi-core/src/sim.rs` (rama `BlockKind::Assemble`): reúne `Cantidad`
  transacciones con la misma clave de conjunto (la clave de reunión de la transacción) y, si no es
  un `GATHER`, destruye todas menos la primera, que sigue por `Salida1`.

**El banco.** `Generate` (Media 60, Modificador 30) → `Assemble` (Cantidad 1) → `Terminate`; más un
`Clock` (Frecuencia 3600) y su `TERMINATE 1`. El validador dice `status: ok`, `blocks: 5`,
`diagnostics: none` (`/tmp/fase7/Assemble/lint.txt`).

**GPSS.** El original produce

```
                GENERATE        60,30
A_Assemble      ASSEMBLE        1
A_Terminate     TERMINATE
                GENERATE        3600
                TERMINATE       1
```

y el nuestro las mismas sentencias (`/tmp/fase7/Assemble/ours.gpss.txt`):
`scripts/wine-oracle/compare-gpss.sh` dice "el GPSS coincide: 3 bloques y 0 conexiones".

**Cifras.** La tabla del oráculo (`/tmp/fase7/Assemble/numbers.md`) coincide en las dos métricas que
mide: bloque `A_Assemble` entries 58 contra 61 (error 5%, dentro del intervalo [49.615, 72.385]) y
bloque `A_Terminate` entries 58 contra 61 (error 5%). Lo que no coincide es una columna que el
oráculo no mide: el `CURRENT COUNT` del bloque `A_Assemble`, que en el reporte del original es 0
(`/tmp/fase7/Assemble/theirs.report.txt`) y en el nuestro 59 (`ours.report.txt`), igual al número
de entradas: es decir, nuestro ASSEMBLE nunca descuenta las transacciones que ya se fueron. El
flujo de las transacciones sí es el mismo (las 59 llegan al `Terminate`).

**Veredicto.** con diferencias

**Diferencia concreta.** El `CURRENT COUNT` del bloque `ASSEMBLE` de nuestro reporte es el número de
entradas (59) donde el original pone 0: al completar el conjunto, nuestro motor pasa las
transacciones al siguiente bloque con `enter_block` y no con `leave`, así que el contador de
presentes del bloque no baja (`crates/genmsi-core/src/sim.rs`, rama `BlockKind::Assemble`, la que
llama a `self.enter_block(id, target)`). El arreglo va en esa misma rama: usar `self.leave(id,
block_index, target)` como hacen las demás, para que `leave` descuente el `current` del bloque. La
prueba que lo fijaría: un `ASSEMBLE` de `Cantidad` 2 con dos flujos, comprobando que el bloque
termina con `current` 0 (el oráculo, `crates/genmsi-core/src/oracle.rs`, sólo compara `entries`, así
que la prueba tiene que ser del core; hoy no hay ninguna prueba de ejecución del `ASSEMBLE`, la
única mención está en la emisión del GPSS, `crates/genmsi-core/tests/gpss.rs`).

### Split

**El original.** La clase no tiene `ClassDescrip` en `Genmsi_20230711.backup/gClasses.xml`
(`/tmp/fase7/Split/original.txt`): lo que dice de ella es su macrocode,
`DECLARE Salida1,Output //Se declara primero por compatibilidad. IN Entrada1:'AClonar'='Clona' CLONE [Cantidad:'Cantidad de Copias'='1'], Salida2:'Copias', [Param:'Parámetro Donde se numeran las transacciones'=''] OUT Salida1:'Original'`,
y el manual del simulador (`Genmsi_20230711.backup/VISUALSIS.HLP`): "Un bloque SPLIT crea
transacciones del mismo conjunto de reunión (assembly set) que la transacción [padre]"; "Una copia de
la transacción padre es creada la cual debería seguir a la transacción padre al próximo bloque".

- **Puertos.** Entrada `Entrada1` (rótulo "AClonar"); salidas `Salida1` (rótulo "Original") y
  `Salida2` (rótulo "Copias").
- **Propiedades y defaults.** `Cantidad` (tipo `TSNA`, sub-tipo `SNA`; default del macrocode `'1'`):
  la cantidad de copias que se crean. `Param` (tipo `EntityUse`, sub-tipo `Parameter`; default `''`,
  vacío): el parámetro donde se numeran las transacciones (el manual: "El operando opcional C es
  especifica el número serial de la transacción padre y la transacción nueva creada recibe un número
  en serie. La numeración comienza con el valor correspondiente del parámetro original de la
  transacción padre, más 1"); el banco no lo usa.
- **Semántica.** El macrocode compila a `SPLIT <Cantidad>` y da a la sentencia el nombre `Clona`:
  se crean `Cantidad` copias de la transacción, del mismo conjunto de reunión, con su misma prioridad
  y sus mismos parámetros; las copias van al destino de `Salida2` ("Copias") y, cuando ese destino no
  está, "deberían seguir a la transacción padre al próximo bloque"; el padre sigue por `Salida1`
  ("Original"). Nuestro motor compila la clase en `crates/genmsi-core/src/program.rs` (rama
  `"Split"`, `BlockKind::Split(SplitSpec { total })`) y la ejecuta en `crates/genmsi-core/src/sim.rs`
  (rama `BlockKind::Split`): crea `Cantidad` copias con la misma clave de conjunto de reunión, las
  manda al destino de `Salida2` y, si no lo hay, al primer destino del bloque; el padre sigue por
  `Salida1`.

**El banco.** `Generate` (Media 60, Modificador 30) → `Split` (Cantidad 1, `Param` vacío) →
`Terminate`; más un `Clock` (Frecuencia 3600) y su `TERMINATE 1`. La salida `Salida2` ("Copias")
está sin conectar. El validador dice `status: ok`, `blocks: 5`, `diagnostics: none`
(`/tmp/fase7/Split/lint.txt`).

**GPSS.** El original produce

```
                GENERATE        60,30
Clona           SPLIT           1
A_Terminate     TERMINATE
                GENERATE        3600
                TERMINATE       1
```

y el nuestro las mismas sentencias (`/tmp/fase7/Split/ours.gpss.txt`):
`scripts/wine-oracle/compare-gpss.sh` dice "el GPSS coincide: 3 bloques y 0 conexiones".

**Cifras.** La tabla del oráculo (`/tmp/fase7/Split/numbers.md`) tiene una sola métrica: bloque
`A_Terminate` entries 116 contra 122 nuestros (error 5%, dentro del intervalo [99.231, 144.769]),
`Match = yes`. Coincide el comportamiento: las 58 transacciones que entran al `SPLIT` dejan dos
transacciones cada una en el `Terminate` (116 en el original, 110 en el nuestro, `ours.report.txt`).
La métrica del propio bloque `SPLIT` no está en la tabla porque el oráculo empareja los bloques por
su etiqueta (`crates/genmsi-core/src/oracle.rs`) y las etiquetas no coinciden: la del original es
`Clona` y la nuestra `A_Split`. Los dos reportes tienen entradas en ese bloque (58 en el original,
55 en el nuestro, dentro del intervalo del oráculo), pero el oráculo no las compara.

**Veredicto.** con diferencias

**Diferencia concreta.** Lo único que se aparta es el nombre que da el reporte al bloque: la columna
`LOC` de nuestro reporte dice `A_Split` (el `mylabel` del puerto de entrada, que es lo que
`crates/genmsi-core/src/program.rs` guarda en `ExecBlock::label`) donde el original dice `Clona` (el
nombre que el macrocode le da a la sentencia, el mismo que escribe nuestro GPSS); por eso el
oráculo no pudo comparar las entradas de ese bloque. El arreglo va en
`crates/genmsi-core/src/report.rs`, en `fn loc_label` (y en `fn port_target_label`, que usa la misma
etiqueta para los destinos): el reporte tendría que usar el mismo nombre que
`crates/genmsi-core/src/gpss.rs` (`struct Names`, que numera los bloques por el nombre del módulo y
el nombre que da su macro). La prueba que lo fijaría: la fila del banco `Split` con `LOC = Clona`,
al lado de las de `crates/genmsi-core/tests/report_gpss.rs` (`split_rows_render_the_copy_target`, que
hoy busca la fila por `Clona` y por eso pasa con los modelos de referencia). Aparte, y sin que este
banco lo mida: la propiedad `Param` (la numeración en serie de las copias) la declara la clase y
`crates/genmsi-core/src/program.rs` (rama `"Split"`) no la lee; y las copias, cuando `Salida2` está
sin conectar, van al destino de `Salida1` y no al bloque siguiente de la secuencia, que es lo que
hace el original.

### Link

**El original.** La clase no tiene `ClassDescrip` en `Genmsi_20230711.backup/gClasses.xml`
(`/tmp/fase7/Link/original.txt`): lo que dice de ella es su macrocode,
`IN Entrada1:'Ingreso'='A_ModuleName' LINK [Cadena:'Cadena a la cual se enlazará la transacción activa'='Cadena'], [Modo:'Modo en que se ordenaran las transacciones'='FIFO'], [Parametro:'Parametro a ordenar, en caso de que el modo sea por Parámetro'=''], Salida1`,
y el manual del simulador (`Genmsi_20230711.backup/VISUALSIS.HLP`): "Un bloque LINK controla la
colocación [de la transacción activa] en una Cadena de Usuario"; el manual explica además el
"indicador link" de cada cadena: "Un interruptor llamado un 'indicador link' es parte de cada entidad
userchain... Cuando el indicador link de una entidad userchain está en off, si el operando C es
usado, el bloque LINK no deberá colocar la transacción en la cadena de usuario. En cambio, la
transacción [debe] proceder al bloque especificado por C, y luego el indicador link deberá ser set.
Las siguientes transacciones entrantes al bloque LINK deberán ir hacia la cadena de usuario"; el
indicador "es apagado (reset) cuando un bloque UNLINK encuentra la cadena de usuario vacía".

- **Puertos.** Entrada `Entrada1` (rótulo "Ingreso"); salida `Salida1` (sin rótulo). El macrocode
  escribe el destino de `Salida1` como el tercer operando de la sentencia cuando la salida está
  conectada.
- **Propiedades y defaults.** `Cadena` (tipo `EntityUse`, sub-tipo `UserChain`; default del
  macrocode `'Cadena'`): la cadena de usuario donde se encola la transacción. `Modo` (tipo `Listed`,
  sub-tipo `LinkMode`; default `'FIFO'`): el orden en que se ordenan las transacciones de la cadena
  (el manual admite también LIFO o el nombre de un parámetro, SNA o SNA*par). `Parametro` (tipo
  `EntityUse`, sub-tipo `Parameter`; default `''`): el parámetro por el que se ordenan cuando el
  modo no es FIFO ni LIFO; el banco no lo usa.
- **Semántica.** El macrocode compila a `LINK <Cadena>,<Modo>,<destino>`: la transacción va a la
  cadena de usuario por el orden que marca `Modo`; con el tercer operando puesto (el destino de
  `Salida1`), la primera transacción que llega a una cadena con el "indicador link" apagado no entra
  en la cadena sino que "procede al bloque especificado por C" y deja el indicador seteado, y las
  siguientes sí entran en la cadena. Nuestro motor compila la clase en
  `crates/genmsi-core/src/program.rs` (rama `"Link"`, `BlockKind::Link(LinkSpec { chain, fifo })`) y
  la ejecuta en `crates/genmsi-core/src/sim.rs` (rama `BlockKind::Link`): mete la transacción en la
  cadena y la deja esperando; no hay indicador link y el destino de la sentencia no se usa en
  tiempo de ejecución.

**El banco.** `Generate` (Media 60, Modificador 30) → `Link` (Cadena `Cadena`, Modo `FIFO`, con su
`Salida1` conectada al `Terminate`) → `Terminate`; más un `Clock` (Frecuencia 3600) y su
`TERMINATE 1`. El validador dice `status: ok`, `blocks: 5`, `diagnostics: none`
(`/tmp/fase7/Link/lint.txt`).

**GPSS.** El original produce

```
                GENERATE        60,30
A_Link          LINK            Cadena,FIFO,A_Terminate
A_Terminate     TERMINATE
                GENERATE        3600
                TERMINATE       1
```

y el nuestro las mismas sentencias (`/tmp/fase7/Link/ours.gpss.txt`):
`scripts/wine-oracle/compare-gpss.sh` dice "el GPSS coincide: 3 bloques y 0 conexiones".

**Cifras.** La tabla del oráculo (`/tmp/fase7/Link/numbers.md`) tiene una métrica que no coincide:
bloque `A_Terminate` entries, base 1 contra 0 nuestros (error 100%, `Match = no`); la otra coincide:
bloque `A_Link` entries 56 contra 61 (error 9%, dentro del intervalo [49.615, 72.385]). La diferencia
de `A_Terminate` es la regla del "indicador link": en el original la primera transacción que llega a
la cadena vacía fue al destino `A_Terminate` (1 entrada, y la cadena se quedó con las otras 55), y en
el nuestro ninguna llegó (`ours.report.txt`: `A_Link` 59 entradas y 59 pendientes,
`A_Terminate` 0). La tabla de cadenas del reporte también difiere en una columna que el oráculo no
mide: el original da `CADENA  55  0  26,466  55  55  1732,291` y el nuestro
`CADENA  59  0  29,732  59  59  0`: el `AVE.TIME` de la cadena es 0 en el nuestro porque sólo se
suma el tiempo de las transacciones que se desenlazan (`crates/genmsi-core/src/sim.rs`, `fn unlink`),
y el original cuenta también las que siguen en la cadena al acabar la corrida.

**Veredicto.** con diferencias

**Diferencia concreta.** Nuestro `LINK` no suelta ninguna transacción al destino que la sentencia
nombra (el original soltó exactamente una, la primera que encontró la cadena con el indicador
apagado: `A_Terminate` 1 entrada contra 0) y el `AVE.TIME` de la cadena sale 0 (el original, 1732,291).
El arreglo va en `crates/genmsi-core/src/sim.rs`, rama `BlockKind::Link`: hacer que la cadena tenga
un indicador (un campo nuevo en `Chain`, `crates/genmsi-core/src/stat.rs`), que la transacción que
llega con el indicador apagado vaya al destino de `Salida1` en vez de entrar en la cadena, y que
`fn unlink` lo apague cuando encuentre la cadena vacía; y en `crates/genmsi-core/src/stat.rs`,
`fn Chain::finish` (la cierra `Entities::finish`), sumar el tiempo de los miembros que quedan en la
cadena para que `average_time` dé el del original. La prueba que lo fijaría: un banco con un `LINK`
que tiene destino y una cadena vacía, comprobando que la primera transacción va al destino, que la
segunda entra en la cadena y que el `AVE.TIME` de la cadena no es 0 (el oráculo
`crates/genmsi-core/src/oracle.rs` no compara las cadenas). Aparte, y sin que este banco lo mida: la
propiedad `Parametro` y los modos de ordenación que no sean FIFO ni LIFO la declara la clase y
`crates/genmsi-core/src/program.rs` (rama `"Link"`) no los lee; y el reporte no escribe el destino de
la sentencia: nuestro `LINK` sale como `LINK  Cadena,FIFO` donde el original escribe
`LINK  Cadena,FIFO,A_Terminate` (arreglo en `crates/genmsi-core/src/report.rs`, `fn operand_parts`,
rama `BlockKind::Link`, como la fila de `crates/genmsi-core/tests/report_gpss.rs`
`link_rows_render_the_fifo_mode`, que hoy sólo cubre el caso sin destino de `TP10__tp10`).

### Unlink

**El original.** La clase no tiene `ClassDescrip` en `Genmsi_20230711.backup/gClasses.xml`
(`/tmp/fase7/Unlink/original.txt`): lo que dice de ella es su macrocode,
`IN Entrada1:'Ingreso'='A_ModuleName' UNLINK [CompOp:'Operador de comparación'='='], [Cadena:'Cadena de la cual se desencadenaran transacciones'='Cadena'], SalidaAlt:'Desencadenadas', [Modo:'Como se alteran'='Algunos'], [Cantidad:'Cantidad de transacciones a desencadenar'='1'], [CompOpcion:'Que se atributo comparo de la transacción encadenada'='Parámetro'], [CompParam:'Parametro a comparar'='Otro'], [CompValue:'Valor a comparar'='0'], Salida2:'No desencadenó' OUT Salida1:'Desencadenó'`,
y el manual del simulador (`Genmsi_20230711.backup/VISUALSIS.HLP`): "Un bloque UNLINK remueve
transacciones de una cadena de usuario"; "El operando F es usado para especificar el destino
alternativo a tomar por la transacción de entrada cuando el contador límite (operando C) no puede ser
alcanzado, o cuando no se puede remover transacciones de la cadena de usuario. Si el operando F no es
usado, la transacción de entrada siempre va hacia el NSB".

- **Puertos.** Entrada `Entrada1` (rótulo "Ingreso"); salidas `Salida1` (rótulo "Desencadenó"),
  `SalidaAlt` (rótulo "Desencadenadas") y `Salida2` (rótulo "No desencadenó").
- **Propiedades y defaults.** `CompOp` (tipo `Listed`, sub-tipo `op_RelacMaxMin`; default del
  macrocode `'='`): el operador de comparación. `Cadena` (tipo `EntityUse`, sub-tipo `UserChain`;
  default `'Cadena'`): la cadena de la que se desencadenan las transacciones. `Modo` (tipo `Listed`,
  sub-tipo `OperationMode`; default `'Algunos'`): si se desencadenan algunas o todas. `Cantidad`
  (tipo `TSNA`, sub-tipo `SNA`; default `'1'`): cuántas transacciones se desencadenan. `CompOpcion`
  (tipo `Listed`, sub-tipo `PriorityOption`; default `'Parámetro'`): qué atributo de la transacción
  encadenada se compara. `CompParam` (tipo `EntityUse`, sub-tipo `Parameter`; default `'Otro'`): el
  parámetro comparado. `CompValue` (tipo `TSNA`, sub-tipo `SNA`; default `'0'`): el valor
  comparado.
- **Semántica.** El macrocode compila a `UNLINK E <Cadena>,<destino de SalidaAlt>,<Cantidad>,<CompParam>,<CompValue>`
  (el `E` es el modo de igualdad, el que el manual llama el valor por defecto del operador
  relacional): se removen de la cadena hasta `Cantidad` transacciones cuyo atributo cumpla la
  comparación, y esas transacciones removidas van al destino de `SalidaAlt` ("Desencadenadas"); la
  transacción que entra al bloque sigue por su propia salida `Salida1` ("Desencadenó"), o por el
  destino alternativo `Salida2` ("No desencadenó") cuando no se pudo remover ninguna y ese destino
  está usado. Nuestro motor compila la clase en `crates/genmsi-core/src/program.rs` (rama
  `"Unlink"`, `BlockKind::Unlink`) y la ejecuta en `crates/genmsi-core/src/sim.rs` (rama
  `BlockKind::Unlink`, con `fn unlink`): el `Modo = Todos` libera todos los miembros, la comparación
  es siempre por parámetro y con igualdad, y la transacción que entra va a `Salida2` cuando el bloque
  tiene una segunda salida conectada y a `Salida1` si no la hay o si `Salida2` no está conectada.

**El banco.** `Generate` (Media 60, Modificador 30) → `Unlink` (Cadena `Cadena`, Modo `Algunos`,
Cantidad 1, CompOp `=`, CompOpcion `Parámetro`, CompParam `Otro`, CompValue 0) con `Salida1` al
`Terminate` y `SalidaAlt` a un segundo `Terminate`; más un `Clock` (Frecuencia 3600) y su
`TERMINATE 1`. La salida `Salida2` está sin conectar y la cadena `Cadena` no la llena nadie: el banco
mide el `UNLINK` con la cadena vacía. El validador dice `status: ok`, `blocks: 6`,
`diagnostics: none` (`/tmp/fase7/Unlink/lint.txt`).

**GPSS.** El original produce

```
                GENERATE        60,30
A_Unlink        UNLINK   E       Cadena,A_Terminate1,1,Otro,0
A_Terminate     TERMINATE
A_Terminate1    TERMINATE
                GENERATE        3600
                TERMINATE       1
```

y el nuestro las mismas sentencias (`/tmp/fase7/Unlink/ours.gpss.txt`):
`scripts/wine-oracle/compare-gpss.sh` dice "el GPSS coincide: 4 bloques y 0 conexiones".

**Cifras.** La tabla del oráculo (`/tmp/fase7/Unlink/numbers.md`) coincide en las dos métricas que
mide: bloque `A_Unlink` entries 59 contra 61 (error 3%, dentro del intervalo [49.615, 72.385]) y
bloque `A_Terminate` entries 59 contra 61 (error 3%). La tabla de cadenas sale vacía en los dos
(cadena `CADENA` con 0 presentes, 0 entradas y `AVE.TIME` 0,000 en el original; 0, 0 y 0 en el
nuestro), y el segundo `Terminate` (el destino de `SalidaAlt`) tiene 0 entradas en los
dos. Lo que difiere es el nombre: el original escribe `A_Terminate1` en la columna `LOC` y en el
segundo operando del `UNLINK`, y el nuestro escribe `A_Terminate_SalidaAlt`
(`/tmp/fase7/Unlink/theirs.report.txt` contra `ours.report.txt`); por ese nombre el oráculo no pudo
comparar las entradas del segundo `Terminate`.

**Veredicto.** con diferencias

**Diferencia concreta.** Lo único que se aparta es el nombre que da el reporte a los bloques y a los
destinos: nuestro `LOC` dice `A_Terminate_SalidaAlt` (el `mylabel` del puerto, que es lo que
`crates/genmsi-core/src/program.rs` guarda en `ExecBlock::label`) donde el original dice
`A_Terminate1` (el nombre del módulo con el sufijo que el original da al segundo bloque de la misma
clase), y el segundo operando de nuestro `UNLINK` dice lo mismo; el GPSS que emitimos, en cambio,
sí dice `A_Terminate1`. El arreglo va en `crates/genmsi-core/src/report.rs`, en `fn loc_label` y en
`fn port_target_label`: el reporte tiene que usar el mismo nombre que
`crates/genmsi-core/src/gpss.rs` (`struct Names`). La prueba que lo fijaría: la fila del banco
`Unlink` con `LOC = A_Terminate1` y el operando `UNLINK E Cadena,A_Terminate1,1,Otro,0`, al lado de
las de `crates/genmsi-core/tests/report_gpss.rs` (`unlink_rows_render_the_engine_branches_in_baseline_order`).
Aparte, y sin que este banco lo mida: la clase declara `CompOp` y `CompOpcion` y
`crates/genmsi-core/src/program.rs` (rama `"Unlink"`) no las lee (la comparación es siempre `E` por
parámetro), y la regla del "indicador link" que el manual del original describe para el `LINK` y el
`UNLINK` (apagado cuando el `UNLINK` encuentra la cadena vacía) no está en nuestro motor.

### JoinMe

**El original.** El original no describe esta clase (`/tmp/fase7/JoinMe/original.txt`:
"ClassDescrip (vacio: el original no describe esta clase)"); lo que dice de ella es su
macrocode: "IN Entrada1:'Ingreso'='A_ModuleName' JOINME
[Grupo:'Grupo numérico al que se agrega un número'='UnGrupo'] OUT Salida1" — la transacción
se une a un grupo de transacciones y sigue de largo.

- **Puertos.** Una entrada y una salida: `Entrada1` (rótulo "Ingreso") y `Salida1` (sin
  rótulo). El paquete lo dice literal: "entradas: ['Entrada1'] salidas: ['Salida1']"; nuestro
  catálogo trae lo mismo (`crates/genmsi-format/src/catalogue.rs`, static `JOINME`, línea
  852: `PortSpec { ioid: 1, name: "Entrada1", caption: "Ingreso" }` y
  `PortSpec { ioid: 2, name: "Salida1", caption: "" }`).
- **Propiedades y defaults.**
  - `Grupo` — `EntityUse`, subType `TransacGroup` — default `"UnGrupo"`
    (`'Grupo numérico al que se agrega un número'='UnGrupo'`, macrocode en `original.txt`) —
    el grupo de transacciones al que se une la transacción. Catálogo:
    `("JoinMe", "Grupo", "UnGrupo")` (`crates/genmsi-format/src/catalogue.rs`, línea 187).
  - La clase no declara ninguna otra propiedad: el macrocode sólo menciona `[Grupo]`, y el
    banco tampoco puede declarar más (`/tmp/fase7/JoinMe/bench.gmdl`: un solo `property`).
- **Semántica.** La clase compila una sentencia: `JOIN <grupo>`, con la transacción
  entrando por `Entrada1` y saliendo por `Salida1` (macrocode `JOINME [Grupo]`). Nuestro
  motor: `crates/genmsi-core/src/program.rs`, fn `Compiler::expand` (línea 996), brazo
  `"JoinNumber" | "JoinMe"` (línea 1813), arma
  `BlockKind::Join(JoinSpec { group, number: self.sna(module, "Numero")? })`; como el
  `JoinMe` no tiene propiedad `Numero`, el número queda en `None`. En ejecución,
  `crates/genmsi-core/src/sim.rs`, fn `execute` (línea 623), brazo `BlockKind::Join`
  (línea 1336): con número va al grupo numérico (`numeric_groups`) y sin número al grupo de
  transacciones (`transaction_groups`); la transacción sigue por `Salida1`. La sentencia la
  escribe `crates/genmsi-core/src/gpss.rs`, fn `block_statements`, brazo `BlockKind::Join`
  (línea 833): `JOIN <grupo>` a secas cuando no hay número.

**El banco.** `Generate` (Media 60, Modificador 30) → `JoinMe` (Grupo `UnGrupo`) →
`Terminate`, más un `Clock` de 3600 que compila `GENERATE 3600` + `TERMINATE 1`. El grupo
`UnGrupo` recibe las uniones pero nadie lo consulta después: el banco no mide la pertenencia
al grupo, sólo el tránsito. El validador: "status: ok … blocks: 5 … diagnostics: none"
(`/tmp/fase7/JoinMe/lint.txt`).

**GPSS.** Coincide con el del original: los dos escriben `A_JoinMe JOIN UnGrupo`
(`/tmp/fase7/JoinMe/ours.gpss.txt` contra `/tmp/fase7/JoinMe/theirs.gpss.txt`), y el
comparador del oráculo lo confirma: `bash scripts/wine-oracle/compare-gpss.sh
/tmp/fase7/JoinMe/ours.gpss.txt /tmp/fase7/JoinMe/theirs.gpss.txt` → "el GPSS coincide: 3
bloques y 0 conexiones" (exit 0).

**Cifras.** La tabla del oráculo (`/tmp/fase7/JoinMe/numbers.md`) coincide en las dos
métricas: bloque `A_JoinMe` entries 58 contra 61 nuestros (error relativo 5%, "Match: yes") y
bloque `A_Terminate` entries 58 contra 61 (5%, "Match: yes"), dentro del 20% de tolerancia
en conteos. Los dos reportes coinciden en el encabezado (`START_TIME 0`, `END_TIME 3600`,
`BLOCKS 5`, `FACILITIES 0`, `STORAGES 0`) y en las filas de los tres bloques. Lo que difiere
está en las secciones que el original añade y el nuestro no:
- El original cierra con la sección `XACT_GROUP` (`UNGRUPO`, `GROUP_SIZE 0`, `RETRY 0`) y con
  la tabla `FEC_XACT_NUMBER`; nuestro reporte no imprime ninguna de las dos
  (`crates/genmsi-core/src/report.rs`, fn `render`, línea 35: sólo escribe facilities, queues,
  storages, chains, tables, qtables, savevalues y diagnostics).
- El `GENERATE 3600` del reloj cuenta 2 entradas en el original y 1 en el nuestro: es el
  asunto ya zanjado de la primera transacción del generador
  (`odd/tasks/notas-primera-llegada.md`: "entradas(original) = llegadas_ocurridas + 1 (la que
  espera en la FEC)"), no algo nuevo de esta clase.

**Veredicto.** verificada

**Diferencia concreta.** El GPSS coincide palabra por palabra, las dos métricas del banco
están dentro de la tolerancia y la semántica de la clase (unirse al grupo y seguir) coincide
con lo que dice el macrocode; lo que se aparta es el reporte: nuestro no tiene la sección
`XACT_GROUP` (tamaño y retry del grupo) ni la `FEC_XACT_NUMBER`, así que la pertenencia al
grupo no se puede comparar con el original, y la regla del `GROUP_SIZE` no queda determinada
con este banco (el original deja el tamaño en 0 después de 58 uniones). El arreglo es del
reporte, no del modelo: `crates/genmsi-core/src/report.rs`, fn `render` (línea 35), escribiendo
una sección de grupos con lo que el motor ya guarda en `crates/genmsi-core/src/sim.rs` (los
mapas `transaction_groups` y `numeric_groups` del fn `execute`); lo que lo fijaría es un test
de `crates/genmsi-core/tests/report_gpss.rs` que compare la sección de grupos de este banco
con `/tmp/fase7/JoinMe/theirs.report.txt`. Y para medir si el `JOIN` del original mete de
verdad a la transacción en el grupo (su `GROUP_SIZE` en 0 no lo dice) haría falta un banco
`JoinMe → RemoveMe` o `JoinMe → JoinNumber`, con el `XACT_GROUP` del original como medida.

### JoinNumber

**El original.** El original no describe esta clase (`/tmp/fase7/JoinNumber/original.txt`:
"ClassDescrip (vacio: el original no describe esta clase)"); lo que dice de ella es su
macrocode: "IN Entrada1:'Ingreso'='A_ModuleName' JOINNUMBER
[Grupo:'Grupo numérico al que se agrega un número'='UnGrupo'],
[Numero:'Número agregardo al grupo'='0'] OUT Salida1" — al grupo se le agrega un número.

- **Puertos.** Una entrada y una salida: `Entrada1` (rótulo "Ingreso") y `Salida1` (sin
  rótulo). El paquete: "entradas: ['Entrada1'] salidas: ['Salida1']"; nuestro catálogo trae lo
  mismo (`crates/genmsi-format/src/catalogue.rs`, static `JOINNUMBER`, línea 866).
- **Propiedades y defaults.**
  - `Grupo` — `EntityUse`, subType `NumericGroup` — default `"UnGrupo"`
    (`'Grupo numérico al que se agrega un número'='UnGrupo'`) — el grupo al que se agrega el
    número. Catálogo: `("JoinNumber", "Grupo", "UnGrupo")` (línea 189).
  - `Numero` — `TSNA`, subType `SNA` — default `0` (`'Número agregardo al grupo'='0'`) — el
    número que se agrega al grupo. Catálogo: `("JoinNumber", "Numero", "0")` (línea 190).
  - La diferencia con el `JoinMe` está en el sub-tipo: aquí el grupo es `NumericGroup` y en el
    `JoinMe` es `TransacGroup` (`/tmp/fase7/JoinNumber/bench.gmdl`: `subType="NumericGroup"`).
- **Semántica.** La clase compila una sentencia: `JOIN <grupo>,<número>`, con la transacción
  entrando por `Entrada1` y saliendo por `Salida1`. Nuestro motor:
  `crates/genmsi-core/src/program.rs`, fn `Compiler::expand` (línea 996), brazo
  `"JoinNumber" | "JoinMe"` (línea 1813), arma
  `BlockKind::Join(JoinSpec { group, number: self.sna(module, "Numero")? })`; aquí el número
  sí está (`0`), y en ejecución `crates/genmsi-core/src/sim.rs`, fn `execute` (línea 623),
  brazo `BlockKind::Join` (línea 1336), lo mete en el grupo **numérico**
  (`numeric_groups`), no en el de transacciones: ésa es exactamente la distinción que hace el
  original, que lista el `JoinMe` en la sección `XACT_GROUP` y el `JoinNumber` en la
  `NUMERIC_GROUP`. La sentencia la escribe `crates/genmsi-core/src/gpss.rs`, fn
  `block_statements`, brazo `BlockKind::Join` (línea 833): `JOIN <grupo>,<número>`.

**El banco.** `Generate` (Media 60, Modificador 30) → `JoinNumber` (Grupo `UnGrupo`, Número
`0`) → `Terminate`, más un `Clock` de 3600 que compila `GENERATE 3600` + `TERMINATE 1`. Nadie
consulta el grupo después. El validador: "status: ok … blocks: 5 … diagnostics: none"
(`/tmp/fase7/JoinNumber/lint.txt`).

**GPSS.** Coincide con el del original: los dos escriben `A_JoinNumber JOIN UnGrupo,0`
(`/tmp/fase7/JoinNumber/ours.gpss.txt` contra `/tmp/fase7/JoinNumber/theirs.gpss.txt`), y el
comparador del oráculo lo confirma: `bash scripts/wine-oracle/compare-gpss.sh
/tmp/fase7/JoinNumber/ours.gpss.txt /tmp/fase7/JoinNumber/theirs.gpss.txt` → "el GPSS coincide:
3 bloques y 0 conexiones" (exit 0).

**Cifras.** La tabla del oráculo (`/tmp/fase7/JoinNumber/numbers.md`) coincide en las dos
métricas: bloque `A_JoinNumber` entries 62 contra 61 nuestros (error relativo 2%, "Match:
yes") y bloque `A_Terminate` entries 62 contra 61 (2%, "Match: yes"). Los dos reportes
coinciden en el encabezado (`BLOCKS 5`) y en las filas de los tres bloques. Las diferencias
son de secciones, como en el `JoinMe`:
- El original cierra con la sección `NUMERIC_GROUP` (`UNGRUPO`, `GROUP_SIZE 1`, `RETRY 0`) y
  con la tabla `FEC_XACT_NUMBER`; nuestro reporte no imprime ninguna de las dos
  (`crates/genmsi-core/src/report.rs`, fn `render`, línea 35).
- El `GENERATE 3600` del reloj cuenta 2 entradas en el original y 1 en el nuestro: es el
  asunto ya zanjado de la primera transacción del generador
  (`odd/tasks/notas-primera-llegada.md`: "entradas(original) = llegadas_ocurridas + 1 (la que
  espera en la FEC)"), no algo nuevo de esta clase.

**Veredicto.** verificada

**Diferencia concreta.** El GPSS coincide, las cifras del banco están dentro de la
tolerancia y la semántica (agregar un número al grupo numérico, distinto del grupo de
transacciones del `JoinMe`) coincide con lo que dice el macrocode; lo que se aparta es que
nuestro reporte no imprime la sección `NUMERIC_GROUP` (ni la `FEC_XACT_NUMBER`), de modo que
el `GROUP_SIZE` del original —1 después de 62 uniones con el número 0— no se puede comparar
ni se sabe qué regla lo produce. El arreglo es del reporte: `crates/genmsi-core/src/report.rs`,
fn `render` (línea 35), una sección de grupos que lea los mapas `numeric_groups` y
`transaction_groups` de `crates/genmsi-core/src/sim.rs`; lo fijaría un test de
`crates/genmsi-core/tests/report_gpss.rs` que compare esa sección con
`/tmp/fase7/JoinNumber/theirs.report.txt`. Para medir la regla del tamaño del grupo haría
falta un banco que uniera con números distintos (aquí se une 62 veces con el mismo 0), porque
con un solo valor no se distingue "miembros" de "valores distintos".

### RemoveMe

**El original.** El original no describe esta clase (`/tmp/fase7/RemoveMe/original.txt`:
"ClassDescrip (vacio: el original no describe esta clase)"); lo que dice de ella es su
macrocode: "DECLARE Salida1,Output //Se declara primero por compatibilidad. IN
Entrada1:'Ingreso'='A_ModuleName' REMOVEME [Grupo:'Grupo numérico al que se agrega un
número'='UnGrupo'], Salida2:'No en [Grupo]' OUT Salida1" — la transacción se saca a sí misma
del grupo.

- **Puertos.** Una entrada y dos salidas: `Entrada1` (rótulo "Ingreso"), `Salida1` (sin
  rótulo, por donde sigue la transacción) y `Salida2` (rótulo "No en [Grupo]", la que no
  está en el grupo). El paquete: "entradas: ['Entrada1'] salidas: ['Salida1', 'Salida2']";
  nuestro catálogo trae lo mismo (`crates/genmsi-format/src/catalogue.rs`, static `REMOVEME`,
  línea 1102). En el banco `Salida2` queda sin conectar: la leyenda del módulo lo dice
  ("RemoveMe%20%28salidas%20sin%20conectar%3A%20Salida2%29",
  `/tmp/fase7/RemoveMe/bench.gmdl`).
- **Propiedades y defaults.**
  - `Grupo` — `EntityUse`, subType `TransacGroup` — default `"UnGrupo"`
    (`'Grupo numérico al que se agrega un número'='UnGrupo'`) — el grupo del que la
    transacción se saca. Catálogo: `("RemoveMe", "Grupo", "UnGrupo")`
    (`crates/genmsi-format/src/catalogue.rs`, línea 248).
  - La clase no declara más propiedades: el macrocode sólo menciona `[Grupo]`.
- **Semántica.** La clase compila una sentencia: `REMOVE <grupo>` — sin la comparación que
  escribe el `RemoveTrans` — con la transacción entrando por `Entrada1` y saliendo por
  `Salida1`. Nuestro motor: `crates/genmsi-core/src/program.rs`, fn `Compiler::expand`
  (línea 996), brazo `"RemoveMe"` (línea 1854), arma
  `BlockKind::Remove(RemoveSpec { group, count: self.empty_expr(), all: false, parameter:
  None, value: None, me: true })` con la clase del bloque puesta en `"Remove"`; en ejecución,
  `crates/genmsi-core/src/sim.rs`, fn `execute` (línea 623), brazo `BlockKind::Remove`
  (línea 1402): con `me` quita la transacción del grupo de transacciones
  (`transaction_groups`) y la deja seguir por `Salida1`. La sentencia la escribe
  `crates/genmsi-core/src/gpss.rs`, fn `block_statements`, brazo `BlockKind::Remove`
  (línea 542): `REMOVE <grupo>` a secas cuando es el `RemoveMe`.

**El banco.** `Generate` (Media 60, Modificador 30) → `RemoveMe` (Grupo `UnGrupo`) →
`Terminate`, más un `Clock` de 3600 que compila `GENERATE 3600` + `TERMINATE 1`. El banco no
une a nadie a `UnGrupo` antes, así que el grupo está vacío y la transacción no está en él:
lo que se mide es el tránsito, no el efecto sobre el grupo. El validador: "status: ok …
blocks: 5 … diagnostics: none" (`/tmp/fase7/RemoveMe/lint.txt`).

**GPSS.** Coincide con el del original: los dos escriben `A_RemoveMe REMOVE UnGrupo`
(`/tmp/fase7/RemoveMe/ours.gpss.txt` contra `/tmp/fase7/RemoveMe/theirs.gpss.txt`), y el
comparador del oráculo lo confirma: `bash scripts/wine-oracle/compare-gpss.sh
/tmp/fase7/RemoveMe/ours.gpss.txt /tmp/fase7/RemoveMe/theirs.gpss.txt` → "el GPSS coincide: 3
bloques y 0 conexiones" (exit 0).

**Cifras.** La tabla del oráculo (`/tmp/fase7/RemoveMe/numbers.md`) coincide en las dos
métricas: bloque `A_RemoveMe` entries 60 contra 61 nuestros (error relativo 2%, "Match:
yes") y bloque `A_Terminate` entries 60 contra 61 (2%, "Match: yes"). Los dos reportes
coinciden en el encabezado (`BLOCKS 5`) y en las tres filas de bloques, con una excepción de
forma en la fila del `RemoveMe`: el original escribe

```
    2       60       0     0  A_RemoveMe  REMOVE           UnGrupo
```

y el nuestro

```
    2       63       0     0  A_RemoveMe  REMOVE   E       UnGrupo
```

es decir, de más la letra de modo `E` (la de la comparación del `RemoveTrans`). Además, el
original cierra con la sección `XACT_GROUP` (`UNGRUPO`, `GROUP_SIZE 0`, `RETRY 0`) y con la
tabla `FEC_XACT_NUMBER`, que nuestro reporte no imprime
(`crates/genmsi-core/src/report.rs`, fn `render`, línea 35). El `GENERATE 3600` del reloj
cuenta 2 entradas en el original y 1 en el nuestro: es el asunto ya zanjado de la primera
transacción del generador (`odd/tasks/notas-primera-llegada.md`: "entradas(original) =
llegadas_ocurridas + 1 (la que espera en la FEC)").

**Veredicto.** con diferencias

**Diferencia concreta.** La fila del bloque `RemoveMe` en nuestro reporte lleva la letra de
modo `E` que el original no pone: la forma del `REMOVE` con comparación es del `RemoveTrans`, y
el `REMOVE` del `RemoveMe` no la trae. El arreglo va en `crates/genmsi-core/src/report.rs`,
fn `operand_parts` (línea 410), brazo `BlockKind::Remove` (línea 615), donde hoy se hace
`mode = Some("E")` (línea 632) para los dos casos: tendría que ser sólo cuando la clase no
es el `RemoveMe` (`spec.me` es `false`). Lo que lo fijaría: un test de
`crates/genmsi-core/tests/report_gpss.rs` que compruebe que la fila del `RemoveMe` del banco
`RemoveMe` sale `REMOVE           UnGrupo` y la del `RemoveTrans` sale
`REMOVE   E       UnGrupo,1,,otro,1`. Y sigue sin medirse con este banco lo que la clase
de verdad hace sobre el grupo (que la transacción se saque de él): haría falta un banco
`JoinMe → RemoveMe` con la sección `XACT_GROUP` del original como medida.

### RemoveTrans

**El original.** El original no describe esta clase (`/tmp/fase7/RemoveTrans/original.txt`:
"ClassDescrip (vacio: el original no describe esta clase)"); lo que dice de ella es su
macrocode: "DECLARE Salida1,Output … IN Entrada1:'Entrada1'='A_ModuleName' REMOVETRANS
[CompOp:'Operador relacional: CompParam OP CompValue'='='],
[Grupo:'Grupo numérico al que se agrega un número'='UnGrupo'],
[Modo:'Cuantas se remueven'='Algunos'],
[Cantidad:'Cantidad de transacciones a remover'='1'],
[CompOpcion:'Que se compara'='Parámetro'],
[CompParam:'Parametro a comparar'='otro'],
[CompValue:'Valor a comparar'='1'], Salida2:'No rmv' OUT Salida1" — saca del grupo las
transacciones que cumplen una comparación.

- **Puertos.** Una entrada y dos salidas: `Entrada1` (rótulo "Entrada1"), `Salida1` (sin
  rótulo) y `Salida2` (rótulo "No rmv", la que no se removió nada). El paquete: "entradas:
  ['Entrada1'] salidas: ['Salida1', 'Salida2']"; nuestro catálogo trae lo mismo
  (`crates/genmsi-format/src/catalogue.rs`, static `REMOVETRANS`, línea 1117). En el banco
  `Salida2` queda sin conectar ("RemoveTrans%20%28salidas%20sin%20conectar%3A%20Salida2%29",
  `/tmp/fase7/RemoveTrans/bench.gmdl`).
- **Propiedades y defaults.**
  - `CompOp` — `Listed`, subType `op_RelacMaxMin` — default `"="` (`'Operador relacional:
    CompParam OP CompValue'='='`) — el operador de la comparación. La lista del original
    (`Genmsi_20230711.backup/gmacroconfig.gmcfg`, `listedname="op_RelacMaxMin"`) tiene ocho:
    "E:=|NE:!=|G:>|L:<|GE:>=|LE:<=|MAX:Maximo|MIN:Minimo". Catálogo:
    `("RemoveTrans", "CompOp", "%3D")` (línea 251; el default viene codificado en la URL, como
    en el banco, donde la propiedad vale `%3D`).
  - `Grupo` — `EntityUse`, subType `TransacGroup` — default `"UnGrupo"` — el grupo del que se
    sacan las transacciones. Catálogo: línea 255.
  - `Modo` — `Listed`, subType `OperationMode` — default `"Algunos"` (`'Cuantas se remueven'`) —
    cuántos se remueven. La lista del original (`gmacroconfig.gmcfg`,
    `listedname="OperationMode"`) tiene dos: "ALL:Todos | :Algunos". Catálogo: línea 256.
  - `Cantidad` — `TSNA`, subType `SNA` — default `1` (`'Cantidad de transacciones a remover'`)
    — cuántas se remueven cuando el modo es "Algunos". Catálogo: línea 250.
  - `CompOpcion` — `Listed`, subType `PriorityOption` — default `"Parámetro"` (`'Que se
    compara'`) — qué se compara. La lista del original (`gmacroconfig.gmcfg`,
    `listedname="PriorityOption"`) tiene dos: "PR:Prioridad | :Parámetro". Catálogo: línea 252.
  - `CompParam` — `EntityUse`, subType `Parameter` — default `"otro"` (`'Parametro a
    comparar'`) — el parámetro que se compara. Catálogo: línea 253.
  - `CompValue` — `TSNA`, subType `SNA` — default `1` (`'Valor a comparar'`) — el valor con el
    que se compara. Catálogo: línea 254.
- **Semántica.** La clase compila una sentencia: `REMOVE E <grupo>,<cantidad>,,<parámetro>,
  <valor>` (macrocode `REMOVETRANS`), con la transacción entrando por `Entrada1` y saliendo
  por `Salida1`; la letra de modo sale de `CompOp` ("=" → "E"). Nuestro motor:
  `crates/genmsi-core/src/program.rs`, fn `Compiler::expand` (línea 996), brazo
  `"RemoveTrans"` (línea 1873), arma
  `BlockKind::Remove(RemoveSpec { group, count, all, parameter, value, me: false })` con
  `all` cuando `Modo` es "Todos" y el parámetro y el valor de `CompParam`/`CompValue`; en
  ejecución, `crates/genmsi-core/src/sim.rs`, fn `execute` (línea 623), brazo
  `BlockKind::Remove` (línea 1402), llama a `remove_from_group` (línea 1960), que saca hasta
  `count` miembros cuyo parámetro vale `wanted` y los deja esperando donde están (no los
  reanuda), y manda la transacción por `Salida2` si no quitó a nadie y por `Salida1` si quitó
  a alguno. La sentencia la escribe `crates/genmsi-core/src/gpss.rs`, fn `block_statements`,
  brazo `BlockKind::Remove` (línea 542): `E <grupo>,<cantidad>,,<parámetro>,<valor>`.

**El banco.** `Generate` (Media 60, Modificador 30) → `RemoveTrans` (CompOp "=", Grupo
`UnGrupo`, Modo "Algunos", Cantidad 1, CompOpcion "Parámetro", CompParam "otro", CompValue 1)
→ `Terminate`, más un `Clock` de 3600 que compila `GENERATE 3600` + `TERMINATE 1`. Nadie se
une a `UnGrupo`, así que el grupo está vacío y la comparación no llega a hacerse: el banco
mide el tránsito y la forma de la sentencia, no el efecto sobre el grupo. El validador:
"status: ok … blocks: 5 … diagnostics: none" (`/tmp/fase7/RemoveTrans/lint.txt`).

**GPSS.** Coincide con el del original: los dos escriben `A_RemoveTrans REMOVE E
UnGrupo,1,,otro,1` (`/tmp/fase7/RemoveTrans/ours.gpss.txt` contra
`/tmp/fase7/RemoveTrans/theirs.gpss.txt`), y el comparador del oráculo lo confirma: `bash
scripts/wine-oracle/compare-gpss.sh /tmp/fase7/RemoveTrans/ours.gpss.txt
/tmp/fase7/RemoveTrans/theirs.gpss.txt` → "el GPSS coincide: 3 bloques y 0 conexiones"
(exit 0). El hueco del tercer operando (`,,`) también está en el original: es el operando de
`CompOpcion`, que con "Parámetro" queda vacío.

**Cifras.** La tabla del oráculo (`/tmp/fase7/RemoveTrans/numbers.md`) coincide en las dos
métricas, y con error cero: bloque `A_RemoveTrans` entries 61 contra 61 nuestros (error
relativo 0%, "Match: yes") y bloque `A_Terminate` entries 61 contra 61 (0%, "Match: yes").
Los dos reportes coinciden en el encabezado (`BLOCKS 5`) y en las filas de los tres bloques,
incluida la letra `E` de la fila del `RemoveTrans`. Las diferencias son de secciones: el
original cierra con la sección `XACT_GROUP` (`UNGRUPO`, `GROUP_SIZE 0`, `RETRY 0`) y con la
tabla `FEC_XACT_NUMBER`, que nuestro reporte no imprime
(`crates/genmsi-core/src/report.rs`, fn `render`, línea 35). El `GENERATE 3600` del reloj
cuenta 2 entradas en el original y 1 en el nuestro: es el asunto ya zanjado de la primera
transacción del generador (`odd/tasks/notas-primera-llegada.md`: "entradas(original) =
llegadas_ocurridas + 1 (la que espera en la FEC)").

**Veredicto.** verificada

**Diferencia concreta.** Con este banco la clase coincide: la sentencia es la misma
operando por operando, las cifras dan 0% de error relativo y el tránsito es el mismo (el
grupo vacío manda la transacción por `Salida1`, que es adonde va el original). Quedan dos
cosas que este banco no mide y que el motor no resuelve:
- El operador de la comparación: `crates/genmsi-core/src/gpss.rs`, fn `block_statements`
  (línea 542) escribe la letra `E` fija, sin mirar `CompOp`; con `CompOp` "Minimo" o
  "Maximo" (que el original escribe "MIN"/"MAX") el GPSS nuestro no lo diría. Lo arreglaría
  esa misma fn (el `RemoveSpec` tendría que llevar el operador, como lo lleva el `Scan`), y
  lo fijaría un test de `crates/genmsi-core/tests/gpss.rs` que compile un `RemoveTrans` con
  `CompOp` "Minimo" y compruebe que sale `REMOVE MIN UnGrupo,1,,otro,1`.
- El modo "Todos": nuestro `RemoveSpec.all` sólo cambia el *reporte* (escribe `ALL` en el
  operando de la cuenta, `crates/genmsi-core/src/report.rs`, fn `operand_parts`, línea 615) y
  el motor borra a todos los miembros, pero el GPSS sigue escribiendo la cuenta numérica
  (`crates/genmsi-core/src/gpss.rs`, línea 542), donde el original escribiría `ALL`. Lo
  fijaría un test de `crates/genmsi-core/tests/gpss.rs` con un `RemoveTrans` de `Modo`
  "Todos". Y el efecto real sobre el grupo (que se saquen las que cumplen la comparación)
  tampoco lo mide este banco: haría falta un `JoinMe → RemoveTrans` con la sección
  `XACT_GROUP` del original como medida.

### Examine

**El original.** El original no describe esta clase (`/tmp/fase7/Examine/original.txt`:
"ClassDescrip (vacio: el original no describe esta clase)"); lo que dice de ella es su
macrocode: "DECLARE Salida1,Output //Se declara primero por compatibilidad. IN
Entrada1:'Entrada1'='A_ModuleName' EXAMINEME [Grupo:'Grupo de transacciones a examinar'=
'UnGrupo'], Salida2:'No en [Grupo]' OUT Salida1:'En [Grupo]'" — se pregunta si la transacción
está en un grupo de transacciones.

- **Puertos.** Una entrada y dos salidas: `Entrada1` (rótulo "Entrada1"), `Salida1` (rótulo
  "En [Grupo]") y `Salida2` (rótulo "No en [Grupo]"). El paquete: "entradas: ['Entrada1']
  salidas: ['Salida1', 'Salida2']"; nuestro catálogo trae lo mismo
  (`crates/genmsi-format/src/catalogue.rs`, static `EXAMINE`, línea 630). En el banco `Salida2`
  queda sin conectar ("Examine%20%28salidas%20sin%20conectar%3A%20Salida2%29",
  `/tmp/fase7/Examine/bench.gmdl`).
- **Propiedades y defaults.**
  - `Grupo` — `EntityUse`, subType `TransacGroup` — default `"UnGrupo"` (`'Grupo de
    transacciones a examinar'='UnGrupo'`) — el grupo que se examina. Catálogo:
    `("Examine", "Grupo", "UnGrupo")` (`crates/genmsi-format/src/catalogue.rs`, línea 128).
  - La clase no declara ninguna otra propiedad: el macrocode sólo menciona `[Grupo]`, y el
    catálogo (`static EXAMINE`, línea 630) también lista una sola. La clase hermana
    `ExamineNumber` es la que tiene el `Numero` (`'Nº a Examinar si pertenece al grupo
    [Grupo]'='0'`, macrocode en `Genmsi_20230711.backup/gClasses.xml`).
- **Semántica.** La clase compila una sentencia: `EXAMINE <grupo>,,` — el grupo y dos
  operandos vacíos — con la transacción entrando por `Entrada1` y saliendo por `Salida1` si
  está en el grupo o por `Salida2` si no. Nuestro motor:
  `crates/genmsi-core/src/program.rs`, fn `Compiler::expand` (línea 996), brazo `"Examine"`
  (línea 1839), arma `BlockKind::Examine(ExamineSpec { group, number: self.empty_expr(),
  transaction: true })`; en ejecución, `crates/genmsi-core/src/sim.rs`, fn `execute` (línea
  623), brazo `BlockKind::Examine` (línea 1361), busca la transacción en
  `transaction_groups` y la manda por `Salida1` si la encuentra y por `Salida2` si no. La
  sentencia la escribe `crates/genmsi-core/src/gpss.rs`, fn `block_statements`, brazo
  `BlockKind::Examine` (línea 532): `format!("{},,", spec.group)` para la clase `Examine` y
  `format!("{},{},", group, número)` para la `ExamineNumber`.

**El banco.** `Generate` (Media 60, Modificador 30) → `Examine` (Grupo `UnGrupo`) →
`Terminate`, más un `Clock` de 3600 que compila `GENERATE 3600` + `TERMINATE 1`. El banco no
une a nadie a `UnGrupo` antes, así que la respuesta del `Examine` sería "no está en el grupo"
y la transacción saldría por `Salida2`, que está sin conectar (y con
`crates/genmsi-core/src/program.rs`, fn `next_named` (línea 661), una salida sin conectar
cae al primer destino conectado: el `Terminate`). El validador no dice nada: "status: ok …
blocks: 5 … diagnostics: none" (`/tmp/fase7/Examine/lint.txt`).

**GPSS.** Coincide con el del original, y eso es justamente el problema: los dos escriben
`A_Examine EXAMINE UnGrupo,,` (`/tmp/fase7/Examine/ours.gpss.txt` contra
`/tmp/fase7/Examine/theirs.gpss.txt`), y el comparador del oráculo lo confirma: `bash
scripts/wine-oracle/compare-gpss.sh /tmp/fase7/Examine/ours.gpss.txt
/tmp/fase7/Examine/theirs.gpss.txt` → "el GPSS coincide: 3 bloques y 0 conexiones" (exit 0).

**Cifras.** No hay cifras que comparar: `/tmp/fase7/Examine/numbers.md` está vacío, porque
el original no dejó reporte. Lo que dejó fue un error en pantalla
(`/tmp/fase7/Examine/theirs.report.txt`, dos líneas, tal como lo recogió el script del
oráculo, `/tmp/bench-out/Examine.run.log`: "extracted 2 lines"):

```
Error en linea     1 -> Faltan parámetros
```

Nuestro motor, en cambio, corre: 60 entradas en el `GENERATE`, 60 en el `EXAMINE` y 60 en el
`TERMINATE` (`/tmp/fase7/Examine/ours.report.txt`), con la fila del `EXAMINE` escrita
`EXAMINE         UnGrupo` (el reporte no repite los operandos vacíos).

Qué son esos dos operandos vacíos y por qué los deja el original: no vienen de propiedades
vacías del banco. La clase declara una sola propiedad, `Grupo`, y el banco le pone su valor
por defecto `"UnGrupo"` (`/tmp/fase7/Examine/bench.gmdl`); los vacíos los produce el propio
macrocode de la clase, `EXAMINEME [Grupo],...` seguido de `Salida2`, que el editor compila
en `EXAMINE <grupo>,,`. La prueba es la clase hermana: el banco `ExamineNumber`
(`/tmp/bench-out/ExamineNumber.gpss.txt`) compila `EXAMINE UnGrupo,0,` — con el `Numero` ya
puesto y un hueco menos — y el original tampoco lo corre
(`/tmp/bench-out/ExamineNumber.report.txt`: "Error en linea     1 -> Faltan parámetros"). El
mensaje es del motor del original, no del editor: está en `Genmsi_20230711.backup/VisualSIS.exe`
(cadena "Error en linea " seguida de " -> Faltan par" / "metros."), o sea que el interpretador
de GPSS del original rechaza la sentencia al cargarla.

**Veredicto.** pendiente

**Diferencia concreta.** No se puede verificar con este banco: el original no dejó un reporte
sino el error "Error en linea     1 -> Faltan parámetros", y la causa está determinada (la
sentencia `EXAMINE <grupo>,,` que la propia clase del original produce tiene los operandos
vacíos y el motor del original no la carga) pero no verificada para el comportamiento. En
cuanto a quién le toca arreglarlo: **no es el generador de bancos**
(`scripts/wine-oracle/make-benches.py`), porque la clase no tiene más propiedades que
llenar —la lista de propiedades sale del macrocode y del catálogo (`static EXAMINE`,
`crates/genmsi-format/src/catalogue.rs`, línea 630), y los dos tienen una sola—; es del lado
del original: la clase `Examine` de `Genmsi_20230711.backup/gClasses.xml` (o su macrocode
`EXAMINEME`) tendría que rellenar los operandos que la sentencia necesita, como hace la
`ExamineNumber` con su `Numero`. De nuestro lado lo que queda es una diferencia de
comportamiento: nosotros aceptamos y corremos una sentencia que el original rechaza, y
nadie lo dice: el validador responde "status: ok … diagnostics: none". Si se quiere ser
fiel al original, el sitio es `crates/genmsi-core/src/program.rs`, fn `Compiler::expand`
(brazo `"Examine"`, línea 1839) o el validador (`crates/genmsi-cli/src/commands/lint.rs`, fn
`run`, línea 54, que se apoya en `genmsi_core::documents::lint_document`,
`crates/genmsi-core/src/documents.rs`, línea 97, y sólo recoge `program.diagnostics`), que
deberían avisar de que el `EXAMINE` se compila con operandos vacíos; lo fijaría un test de
`crates/genmsi-core/tests/` que compile el banco `Examine` y compruebe el aviso. Para poder
comparar cifras con el original haría falta un banco que el original pueda correr (una
`ExamineNumber` con todos sus operandos, o un `EXAMINE` escrito a mano en el GPSS), y su
reporte daría el `XACT_GROUP` con el tamaño del grupo, que es justo lo que este banco no
puede medir.

### Mark

**El original.** El original no describe esta clase (`/tmp/fase7/Mark/original.txt`:
"ClassDescrip (vacio: el original no describe esta clase)"); lo que dice de ella es su
macrocode: "IN Entrada1:'Ingreso'='A_moduleName' MARK [Parametro:'Parametro donde se guarda
la hora'='Param'] OUT Salida1" — la transacción marca la hora en un parámetro.

- **Puertos.** Una entrada y una salida: `Entrada1` (rótulo "Ingreso") y `Salida1` (sin
  rótulo). El paquete: "entradas: ['Entrada1'] salidas: ['Salida1']"; nuestro catálogo trae lo
  mismo (`crates/genmsi-format/src/catalogue.rs`, static `MARK`, línea 942).
- **Propiedades y defaults.**
  - `Parametro` — `EntityUse`, subType `Parameter` — default `"Param"` (`'Parametro donde se
    guarda la hora'='Param'`) — el parámetro donde se guarda la hora. Catálogo:
    `("Mark", "Parametro", "Param")` (`crates/genmsi-format/src/catalogue.rs`, línea 203).
  - La clase no declara más propiedades: el macrocode sólo menciona `[Parametro]`.
- **Semántica.** La clase compila una sentencia: `MARK <parámetro>`, con la transacción
  entrando por `Entrada1` y saliendo por `Salida1`. Nuestro motor compila la clase como una
  asignación del reloj: `crates/genmsi-core/src/program.rs`, fn `Compiler::expand` (línea
  996), brazo `"Mark"` (línea 1678), arma
  `BlockKind::Assign(AssignSpec { parameter, op: AssignOp::Set, value: self.clock_expr() })`,
  con `clock_expr` (línea 2489) siendo el símbolo de sistema `Clock`; en ejecución,
  `crates/genmsi-core/src/sim.rs`, fn `execute` (línea 623), el brazo `BlockKind::Assign`
  escribe el valor del reloj en el parámetro de la transacción y la deja seguir por
  `Salida1`. La sentencia GPSS la escribe `crates/genmsi-core/src/gpss.rs`, fn
  `block_statements`, brazo `BlockKind::Assign` (línea 561), que para los bloques de clase
  `"Mark"` escribe `MARK <parámetro>` en vez de `ASSIGN`; el reporte también lo muestra como
  `MARK` (`crates/genmsi-core/src/report.rs`, fn `block_type`, línea 288, que pone el nombre
  de la clase en mayúsculas).

**El banco.** `Generate` (Media 60, Modificador 30) → `Mark` (Parámetro `Param`) →
`Terminate`, más un `Clock` de 3600 que compila `GENERATE 3600` + `TERMINATE 1`. El parámetro
`Param` no se vuelve a leer en el banco, así que la hora marcada no aparece en ninguna cifra
del reporte. El validador: "status: ok … blocks: 5 … diagnostics: none"
(`/tmp/fase7/Mark/lint.txt`).

**GPSS.** Coincide con el del original: los dos escriben `A_Mark MARK Param`
(`/tmp/fase7/Mark/ours.gpss.txt` contra `/tmp/fase7/Mark/theirs.gpss.txt`), y el comparador
del oráculo lo confirma: `bash scripts/wine-oracle/compare-gpss.sh
/tmp/fase7/Mark/ours.gpss.txt /tmp/fase7/Mark/theirs.gpss.txt` → "el GPSS coincide: 3 bloques
y 0 conexiones" (exit 0).

**Cifras.** La tabla del oráculo (`/tmp/fase7/Mark/numbers.md`) coincide en las dos
métricas: bloque `A_Mark` entries 55 contra 61 nuestros (error relativo 11%, "Match: yes") y
bloque `A_Terminate` entries 55 contra 61 (11%, "Match: yes"), dentro del 20% de tolerancia
en conteos. Los dos reportes coinciden en el encabezado (`BLOCKS 5`) y en las filas de los
tres bloques. Las diferencias son de secciones: el original cierra con la tabla
`FEC_XACT_NUMBER` (que es donde se vería el valor del parámetro, pero en este banco su
columna `PARAMETER` viene vacía) y nuestro reporte no imprime esa tabla
(`crates/genmsi-core/src/report.rs`, fn `render`, línea 35). El `GENERATE 3600` del reloj
cuenta 2 entradas en el original y 1 en el nuestro: es el asunto ya zanjado de la primera
transacción del generador (`odd/tasks/notas-primera-llegada.md`: "entradas(original) =
llegadas_ocurridas + 1 (la que espera en la FEC)").

**Veredicto.** verificada

**Diferencia concreta.** El GPSS coincide, las dos métricas del banco están dentro de la
tolerancia y lo que la clase hace —escribir la hora en un parámetro— coincide con lo que dice
el macrocode; lo que se aparta es que nuestro reporte no imprime la tabla `FEC_XACT_NUMBER`,
de modo que la hora marcada no se puede comparar con el original, y en este banco tampoco
se vería (su columna `PARAMETER` viene vacía). Para poder medirlo haría falta un banco que
vuelva a leer el parámetro marcado (por ejemplo un `Assign` que lo copie a otro sitio o un
`Tabulate` de él) y una forma de ver el valor: el `FEC_XACT_NUMBER` del reporte original
—que nuestro `render` (`crates/genmsi-core/src/report.rs`, línea 35) no produce— o el
`trace` del motor, que en el brazo `BlockKind::Assign` (`crates/genmsi-core/src/sim.rs`, fn
`execute`, línea 623) no anota el valor que escribe. Con este banco la diferencia medible es
la que se anota en las cifras: 11% de error relativo en los conteos de los bloques, sin
consecuencias.

### Counter

**El original.** "Numerador de transacciones. Usa una entidad saveValue." (el `ClassDescrip`
de la clase en `Genmsi_20230711.backup/gClasses.xml`, en
`/tmp/fase7/Counter/original.txt`). Su macrocode: "IN Entrada1:'Ingreso al módulo'=
'A_moduleName' SV [SaveValue:'SaveValueUtilizado'='Count'],+,[Incremento:'Valor incremental de
conteo'='1'] ASSI [Param:'Parmetro de conteo'='Nro'],=,X:[SaveValue] OUT Salida1" — suma uno
a un savevalue y copia el resultado a un parámetro.

- **Puertos.** Una entrada y una salida: `Entrada1` (rótulo "Ingreso al módulo") y `Salida1`
  (sin rótulo). El paquete: "entradas: ['Entrada1'] salidas: ['Salida1']"; nuestro catálogo
  trae lo mismo (`crates/genmsi-format/src/catalogue.rs`, static `COUNTER`, línea 584).
- **Propiedades y defaults.**
  - `SaveValue` — `EntityUse`, subType `SaveValue` — default `"Count"` (`'SaveValueUtilizado'=
    'Count'`) — el savevalue que lleva la cuenta. Catálogo: `("Counter", "SaveValue", "Count")`
    (línea 121).
  - `Incremento` — `TSNA`, subType `SNA` — default `1` (`'Valor incremental de conteo'='1'`) —
    cuánto se suma en cada paso. Catálogo: línea 119.
  - `Param` — `EntityUse`, subType `Parameter` — default `"Nro"` (`'Parmetro de conteo'='Nro'`) —
    el parámetro donde se copia la cuenta. Catálogo: línea 120.
- **Semántica.** La clase compila dos sentencias: `SAVEVALUE <savevalue>+,<incremento>` y
  `ASSIGN <parámetro>,X$<savevalue>` (macrocode `SV … ASSI …`), con la transacción entrando
  por `Entrada1` y saliendo por `Salida1`. Nuestro motor: `crates/genmsi-core/src/program.rs`,
  fn `Compiler::expand` (línea 996), brazo `"Counter"` (línea 1130), arma
  `BlockKind::Counter(CounterSpec { savevalue, parameter, increment })`; en ejecución,
  `crates/genmsi-core/src/sim.rs`, fn `execute` (línea 623), brazo `BlockKind::Counter`
  (línea 722): suma el incremento al savevalue (`*cell += increment`) y copia el valor nuevo
  al parámetro de la transacción. Las dos sentencias GPSS las escribe
  `crates/genmsi-core/src/gpss.rs`, fn `block_statements`, brazo `BlockKind::Counter`
  (línea 585): `SAVEVALUE <savevalue>+,<incremento>` y `ASSIGN <parámetro>,X$<savevalue>`.

**El banco.** `Generate` (Media 60, Modificador 30) → `Counter` (SaveValue `Count`,
Incremento 1, Parámetro `Nro`) → `Terminate`, más un `Clock` de 3600 que compila `GENERATE 3600`
+ `TERMINATE 1`. El savevalue `Count` no se declara en ninguna otra parte: lo crea el primer
`SAVEVALUE` que lo usa. El validador: "status: ok … blocks: 5 … diagnostics: none"
(`/tmp/fase7/Counter/lint.txt`).

**GPSS.** Coincide con el del original: los dos escriben las mismas cinco sentencias
(`A_Counter SAVEVALUE Count+,1`, `ASSIGN Nro,X$Count`, `A_Terminate TERMINATE`, `GENERATE
3600`, `TERMINATE 1`) (`/tmp/fase7/Counter/ours.gpss.txt` contra
`/tmp/fase7/Counter/theirs.gpss.txt`), y el comparador del oráculo lo confirma: `bash
scripts/wine-oracle/compare-gpss.sh /tmp/fase7/Counter/ours.gpss.txt
/tmp/fase7/Counter/theirs.gpss.txt` → "el GPSS coincide: 3 bloques y 0 conexiones" (exit 0).

**Cifras.** La tabla del oráculo (`/tmp/fase7/Counter/numbers.md`) coincide en las dos
métricas: bloque `A_Counter` entries 62 contra 61 nuestros (error relativo 2%, "Match: yes")
y bloque `A_Terminate` entries 62 contra 61 (2%, "Match: yes"). El valor del savevalue
también es el que corresponde a cada corrida: el original "SAVEVALUE … COUNT 62" (62
transacciones contadas) y el nuestro "SAVEVALUE … Count 59" (59 en nuestra corrida), que es
la misma regla con otra semilla. Lo que difiere es la forma del reporte:
- La tabla de bloques: el original tiene seis líneas y el nuestro cinco. El original cuenta
  el `ASSIGN` como un bloque más (su línea 3 es `ASSIGN          Nro,X$Count`, con 62
  entradas) y por eso su encabezado dice `BLOCKS 6`; nosotros compilamos la clase en un solo
  bloque (`BlockKind::Counter`), así que su fila dice `SAVEVALUE       Count+,1` y el `ASSIGN`
  no aparece (`/tmp/fase7/Counter/ours.report.txt` contra `/tmp/fase7/Counter/theirs.report.txt`).
- La sección `SAVEVALUE`: el original la titula "SAVEVALUE                 VALUE      RETRY"
  y escribe el nombre en mayúsculas ("COUNT", con 62 y retry 0); el nuestro escribe
  "SAVEVALUE                 VALUE" y el nombre tal como se declaró ("Count", con 59)
  (`crates/genmsi-core/src/report.rs`, fn `render`, líneas 250-253: ahí está el `name` sin
  `to_uppercase()` que sí usan las facilities, las colas, los almacenes y las cadenas).
- El original cierra con la tabla `FEC_XACT_NUMBER`, que nuestro reporte no imprime
  (`crates/genmsi-core/src/report.rs`, fn `render`, línea 35). Y el `GENERATE 3600` del reloj
  cuenta 2 entradas en el original y 1 en el nuestro: es el asunto ya zanjado de la primera
  transacción del generador (`odd/tasks/notas-primera-llegada.md`: "entradas(original) =
  llegadas_ocurridas + 1 (la que espera en la FEC)").

**Veredicto.** con diferencias

**Diferencia concreta.** La clase cuenta lo mismo que el original (la sentencia, el valor del
savevalue y los conteos de los bloques coinciden dentro de la tolerancia), pero nuestro
reporte no refleja las dos sentencias que la clase produce: falta la línea del `ASSIGN`
—la tabla de bloques tiene 5 líneas donde el original tiene 6 y el encabezado dice `BLOCKS 5`
en vez de `BLOCKS 6`— y la sección `SAVEVALUE` no lleva la columna `RETRY` ni el nombre en
mayúsculas. El arreglo va en `crates/genmsi-core/src/report.rs`: fn `render` (línea 35), que
tiene que sacar el `ASSIGN` del `Counter` como línea propia de la tabla de bloques (o el
core tiene que exponer la segunda sentencia del bloque: hoy `BlockKind::Counter` es un
bloque sólo, `crates/genmsi-core/src/program.rs`, fn `Compiler::expand`, brazo `"Counter"`,
línea 1130, y `crates/genmsi-core/src/gpss.rs`, fn `block_statements`, línea 585, escribe las
dos sentencias desde ese bloque), y la sección `SAVEVALUE` de las líneas 250-253, que
tendría que llevar el `RETRY` y el nombre en mayúsculas. Lo que lo fijaría: un test de
`crates/genmsi-core/tests/report_gpss.rs` que compare la tabla de bloques y la sección
`SAVEVALUE` del banco `Counter` con `/tmp/fase7/Counter/theirs.report.txt`.

### Table

**El original.** "Define una tabla para ser tabulada." (el `ClassDescrip` de la clase en
`Genmsi_20230711.backup/gClasses.xml`, en `/tmp/fase7/Table/original.txt`). Su macrocode:
"TABLE [NombreTabla:'Nombre de la tabla a definir'='Tabla'],
[Atributo:'Atributo de Entrada1 a la tabla'='P:Atrib'],
[PrimerIntervalo:'Limite superior de la primer frecuencia'='0'],
[TamFrecuencia:'Limite superior de la primer frecuencia'='10'],
[Cantidad:'cantidad de clases de frecuencia'='10']" — una clase de definición: nombra la
tabla, el atributo que se tabula y las clases de frecuencia.

- **Puertos.** Ninguno: el paquete lo dice literal ("entradas: [] salidas: []") y la leyenda
  del módulo en el banco lo repite ("Table%20%28clase%20sin%20puertos%3A%20no%20se%20puede
  conectar%29", `/tmp/fase7/Table/bench.gmdl`). Nuestro catálogo también: `static TABLE`
  (`crates/genmsi-format/src/catalogue.rs`, línea 1342) no declara ni `inputs` ni `outputs`.
- **Propiedades y defaults.**
  - `NombreTabla` — `EntityDef`, subType `Table` — default `"Tabla"` (`'Nombre de la tabla a
    definir'='Tabla'`) — el nombre de la tabla que se define. Catálogo:
    `("Table", "NombreTabla", "Tabla")` (línea 317).
  - `Atributo` — `TSNA`, subType `SNA` — default `"P:Atrib"` (`'Atributo de Entrada1 a la
    tabla'`) — el atributo que se tabula (aquí el parámetro `Atrib`; el original lo escribe
    `P$Atrib` en el GPSS). Catálogo: `("Table", "Atributo", "P%3AAtrib")` (línea 315).
  - `PrimerIntervalo` — `PosInteger` — default `0` (`'Limite superior de la primer frecuencia'`)
    — el límite superior de la primera clase. Catálogo: línea 318.
  - `TamFrecuencia` — `PosInteger` — default `10` (mismo texto de pista) — el ancho de cada
    clase. Catálogo: línea 319.
  - `Cantidad` — `PosInteger` — default `10` (`'cantidad de clases de frecuencia'`) — cuántas
    clases de frecuencia tiene la tabla. Catálogo: línea 316.
- **Semántica.** La clase compila una sentencia de definición: `TABLE <atributo>,<primer
  intervalo>,<tamaño>,<cantidad>`, con el nombre de la tabla como rótulo del bloque (macrocode
  `TABLE …`). Nuestro motor: `crates/genmsi-core/src/program.rs`, fn
  `Compiler::collect_definitions` (línea 2024), brazo `"Table"` (línea 2180), guarda el
  `TableDef { attribute, first, width, intervals }` (`crates/genmsi-core/src/program.rs`, línea
  694); la entidad se crea en `crates/genmsi-core/src/stat.rs`, struct `Table` (línea 428) y
  `fn new` (línea 457), con `intervals` clases de frecuencia; el reparto de un valor en clase
  está en `fn tabulate` (línea 473): el primer intervalo es `[0, first]`, los siguientes son
  `(first + (n-1)*ancho, first + n*ancho]` y lo que pasa del último se guarda en la última
  clase. La tabla se imprime en la sección `TABLAS` del reporte
  (`crates/genmsi-core/src/report.rs`, fn `render`, líneas 184-230).

**El banco.** Sólo un `Table` (NombreTabla `Tabla`, Atributo `P:Atrib`, PrimerIntervalo 0,
TamFrecuencia 10, Cantidad 10) y un `Clock` de 3600 que compila `GENERATE 3600` + `TERMINATE
1`: no hay `Generate` ni `Terminate`, y ningún `Tabulate` tabula nada, de modo que la tabla
nunca recibe un valor. El validador: "status: ok … blocks: 2 … definitions: … tables: 1 …
diagnostics: none" (`/tmp/fase7/Table/lint.txt`).

**GPSS.** Coincide con el del original: los dos escriben `Tabla TABLE P$Atrib,0,10,10`
(`/tmp/fase7/Table/ours.gpss.txt` contra `/tmp/fase7/Table/theirs.gpss.txt`), y el comparador
del oráculo lo confirma: `bash scripts/wine-oracle/compare-gpss.sh
/tmp/fase7/Table/ours.gpss.txt /tmp/fase7/Table/theirs.gpss.txt` → "el GPSS coincide: 1
bloques y 0 conexiones" (exit 0).

**Cifras.** No hay cifras que comparar: la tabla del oráculo (`/tmp/fase7/Table/numbers.md`)
tiene el encabezado y ninguna fila, porque el banco no tabula nada y sus dos bloques (el
reloj) no llevan rótulo. La comparación es de la sección `TABLAS` de los dos reportes, y ahí
se ven cuatro diferencias:
- **La cabecera de la tabla.** El original escribe `Tabla           TABLE
  P$Atrib,0,10,10` y el nuestro `Tabla           TABLE           0, 10, 10`: a nosotros nos
  falta el operando del atributo (`crates/genmsi-core/src/report.rs`, fn `render`, línea 188:
  escribe `"{:<16} TABLE           {}, {}, {}"` con `table.first`, `table.width` y
  `table.intervals`, y el atributo que sí está guardado en la definición
  (`TableDef.attribute`, `crates/genmsi-core/src/program.rs`, línea 694) no sale).
- **Las clases de frecuencia.** El original imprime diez filas, la última de ellas el
  desborde:

  ```
      0   -  0                0      0,000       0,000        100,000
      1   -  10               0      0,000       0,000        100,000
      ...
      71  -  80               0      0,000       0,000        100,000
      >   a  80               0      0,000       0,000        100,000
  ```

  y el nuestro imprime las mismas nueve primeras y, en vez del desborde, una décima clase
  más:

  ```
      0     - 0              0          0          0          100
      1     - 10             0          0          0          100
      ...
      71    - 80             0          0          0          100
      81    - 90             0          0          0          100
  ```

  Con `Cantidad` 10, `PrimerIntervalo` 0 y `TamFrecuencia` 10, el original hace nueve clases
  finitas (de `0 - 0` a `71 - 80`) más la clase de desborde (`> a 80`), y nosotros hacemos
  diez clases finitas (de `0 - 0` a `81 - 90`) y ninguna de desborde: lo que pasa de 80 se
  guarda en la última clase (`crates/genmsi-core/src/stat.rs`, fn `tabulate`, línea 473) en
  lugar de tener su propia fila.
- **Los rótulos de las statistics.** El original pone antes de las filas
  "MEDIA : 0,000", "DESVIACION ESTANDAR: 0,000" y "CANTIDAD EN RETRY : 0", y después
  "CANTIDAD DE VALORES TABULADOS : 0" y "VALOR MAXIMO TABULADO : 0,000"; el nuestro pone
  las cuatro líneas antes de las filas y con otros rótulos: "MEDIA : 0", "DESVIACION
  ESTANDAR: 0", "CANTIDAD DE VALORES: 0" y "VALOR MAXIMO : 0"
  (`crates/genmsi-core/src/report.rs`, fn `render`, líneas 194-198). O sea: nuestro
  "CANTIDAD DE VALORES" imprime el número de valores tabulados en el sitio donde el
  original imprime el retry, y el "CANTIDAD DE VALORES TABULADOS" del original (que va al
  final) no aparece en el nuestro.
- **El formato de los números.** El original escribe `0,000` y `100,000`; el nuestro `0` y
  `100`, porque `fn number` (`crates/genmsi-core/src/report.rs`, línea 23) saca los decimales
  cuando el valor es entero. Y la numeración de líneas de la tabla de bloques: el original
  empieza en 2 (la sentencia `TABLE` ocupa la línea 1 del programa) y el nuestro en 1.

Aparte, el `GENERATE 3600` del reloj cuenta 2 entradas en el original y 1 en el nuestro: es
el asunto ya zanjado de la primera transacción del generador
(`odd/tasks/notas-primera-llegada.md`: "entradas(original) = llegadas_ocurridas + 1 (la que
espera en la FEC)"), no algo de esta clase.

**Veredicto.** con diferencias

**Diferencia concreta.** La tabla se define igual (el GPSS coincide) pero se imprime distinta:
nuestro reporte pierde el operando del atributo en la cabecera, y —que es la diferencia de
fondo— el original reserva la última de las `Cantidad` clases para el desborde (`> a 80` con
`Cantidad` 10) mientras nosotros la gastamos en otra clase finita (`81 - 90`) y no tenemos
fila de desborde; además los rótulos de las statistics y su sitio están cambiados. El
arreglo va en dos sitios del core: `crates/genmsi-core/src/stat.rs`, struct `Table` (línea
428), `fn new` (línea 457) y `fn tabulate` (línea 473), que tienen que repartir `intervals - 1`
clases finitas más una de desborde, y `crates/genmsi-core/src/report.rs`, fn `render` (línea
35), que tiene que imprimir la fila `> a <último límite>` y el atributo de la cabecera (está
en `TableDef.attribute`, `crates/genmsi-core/src/program.rs`, línea 694) y mover
"CANTIDAD DE VALORES TABULADOS" y "VALOR MAXIMO TABULADO" debajo de las filas. Lo que lo
fijaría: un test de `crates/genmsi-core/tests/` que compare la sección `TABLAS` del banco
`Table` con `/tmp/fase7/Table/theirs.report.txt` fila por fila, y otro con un `Tabulate`
que meta un valor por encima del último límite para ver en qué clase lo cuenta cada uno.
Como el banco no tabula nada, la diferencia de las frecuencias (todas en cero) no se puede
medir aquí: haría falta un banco con un `Tabulate` de un atributo que varíe.

## Cambios pendientes

Cada fila dice **qué** hay que arreglar, **dónde** (archivo y función) y **qué
prueba** lo fijaría. Nada de esto se hizo en esta sesión: el core lo tiene otra
sesión.

### En el core (`crates/genmsi-core`)

| # | Qué hay que arreglar | Dónde | La prueba que lo fijaría |
|---|---|---|---|
| 1 | **La puerta sobre un almacén.** El original tiene seis operadores (`SF:Lleno`, `SNF:NoLleno`, `SE:Vacio`, `SNE:NoVacio`, `SV:Disponible`, `SNV:NoDisponible`, en `Genmsi_20230711.backup/gmacroconfig.gmcfg`) y el motor tiene uno: un booleano `empty`. Hoy nuestra `GateS` abre cuando el almacén **no está vacío**; el original abre cuando **no está lleno**. | `crates/genmsi-core/src/program.rs` · `Compiler::expand`, brazo `"GateS"`; `crates/genmsi-core/src/sim.rs` · `gate_condition`, brazo `GateCondition::Storage`; `crates/genmsi-core/src/gpss.rs` · `block_statements`, brazo `BlockKind::Gate`; `crates/genmsi-core/src/report.rs` · `operand_parts`, brazo `BlockKind::Gate` | Un test del banco `GateS` (op `NoLleno`) que compruebe que la puerta abre y que la fila del reporte dice `GATE SNF Almacen`; otro con op `NoVacio` que fije `GATE SNE`; y la sonda de la sección "Sondas" (el almacén declarado y lleno tiene que cerrar la puerta). |
| 2 | **La selección por estado de un almacén.** `storage_state` mapea `Vacio` y `NoVacio` al mismo `StorageNotFull`, y `SelectState` no tiene estados para "vacío" y "no vacío". | `crates/genmsi-core/src/program.rs` · `storage_state` y el enum `SelectState` | Dos tests de `crates/genmsi-core/tests/gpss.rs`: un `SelectS` con op `Vacio` → `SELECT SE param,1,10`, y otro con `NoVacio` → `SELECT SNE param,1,10`. |
| 3 | **La letra de la puerta en el listado del GPSS.** Escribimos `GATE L` donde el original escribe `GATE LR` (`fixtures/outputs/Problema_1__semaforo-1.gpss.txt`, línea 71). | `crates/genmsi-core/src/gpss.rs` · `block_statements`, brazo `BlockKind::Gate` (`GateCondition::Logic`) | Un test de listado del programa con un `GateL` de operador `Reset` que fije `GATE LR`. |
| 4 | **La columna `LOC` del reporte.** Sale del `mylabel` del puerto; el original escribe el nombre de la sentencia (`GateF`, `Clona`, `A_Terminate1`). | `crates/genmsi-core/src/report.rs` · `render` y `loc_label` | Un test en `crates/genmsi-core/tests/report_gpss.rs` que renderice un `GateF` etiquetado `A_GateF` y compruebe que la fila sale con `LOC` `GateF`. |
| 5 | **La fila `PREEMPT` del reporte** pierde los operandos: escribimos `PREEMPT Recurso` donde el original escribe `PREEMPT Recurso,PR,,Remanente`. | `crates/genmsi-core/src/report.rs` · `operand_parts`, brazo `BlockKind::Preempt` | Un test en `crates/genmsi-core/tests/report_gpss.rs` que compruebe la fila `PREEMPT Cajero1FreeShop,PR,,Remanente` de `TP3__aeropuerto`. |
| 6 | **La fila `SELECT` del reporte.** Escribimos `SELECT MIN param,1,10,,S` / `,,LS` / `MIN ...` donde el original escribe `SNF param,1,10`, `LS param,1,10` y `E param,1,10,*1,Q,`. | `crates/genmsi-core/src/report.rs` · `operand_parts`, brazo `BlockKind::Select` | Tres casos en `crates/genmsi-core/tests/report_gpss.rs`: `SelectS` → `SNF param,1,10`, `SelectL` → `LS param,1,10`, `SelectSNA` → `E param,1,10,*1,Q,`. |
| 7 | **La fila `REMOVE` del reporte** lleva de más la letra `E` en el `RemoveMe`, donde el original no la pone. | `crates/genmsi-core/src/report.rs` · `operand_parts`, brazo `BlockKind::Remove` | Un test que compruebe `REMOVE UnGrupo` en el banco `RemoveMe` y `REMOVE E UnGrupo,1,,otro,1` en el `RemoveTrans`. |
| 8 | **La fila `LINK` del reporte** no escribe el destino de la sentencia. | `crates/genmsi-core/src/report.rs` · `operand_parts`, brazo `BlockKind::Link` | El caso `link_rows_render_the_fifo_mode` de `crates/genmsi-core/tests/report_gpss.rs`, extendido al banco `Link` (que sí tiene destino). |
| 9 | **El `CURRENT` del bloque `ASSEMBLE`** no baja al completar el conjunto: usamos `enter_block` donde las demás ramas usan `leave`. | `crates/genmsi-core/src/sim.rs` · rama `BlockKind::Assemble` | Un test del core con un `ASSEMBLE` de `Cantidad` 2 y dos flujos, comprobando que el bloque termina con `current` 0. |
| 10 | **El "indicador link".** El `LINK` original suelta hacia el destino de la sentencia la primera transacción que encuentra la cadena con el indicador apagado; el nuestro la mete en la cadena siempre. Y el `AVE.TIME` de la cadena sale 0 (el original, 1732,291). | `crates/genmsi-core/src/sim.rs` · rama `BlockKind::Link` y `fn unlink`; `crates/genmsi-core/src/stat.rs` · `Chain` (el indicador) y `Chain::finish` | Un banco con un `LINK` que tiene destino y una cadena vacía: la primera transacción va al destino, la segunda entra en la cadena, y el `AVE.TIME` de la cadena no es 0 (el oráculo no compara las cadenas). |
| 11 | **El `Match` se expande en un bloque**; el original escribe dos (`TRANSFER` + `MATCH`) y el reporte lista los dos, con las entradas en el `TRANSFER`. | `crates/genmsi-core/src/program.rs` · rama `"Match"`; `crates/genmsi-core/src/gpss.rs` · brazo `BlockKind::Match` | El número de líneas de la tabla de bloques del banco `Match` (6) y las entradas de la línea `A_Match`. |
| 12 | **El `Counter` también produce dos sentencias** (`SAVEVALUE` + `ASSIGN`) y el reporte lista una: la tabla tiene 5 líneas y el encabezado dice `BLOCKS 5` donde el original tiene 6. | `crates/genmsi-core/src/report.rs` · `render`; `crates/genmsi-core/src/program.rs` · rama `"Counter"` | Un test que compare la tabla de bloques del banco `Counter` con el reporte del original. |
| 13 | **Faltan secciones en el reporte**: `LOGICSWITCH`, `XACT_GROUP`, `NUMERIC_GROUP` y `FEC_XACT_NUMBER`; y la de `SAVEVALUE` va sin la columna `RETRY` y con el nombre en minúsculas. | `crates/genmsi-core/src/report.rs` · `render`; `crates/genmsi-core/src/stat.rs` (un contador por llave lógica) | Un test que compare esas secciones con los reportes del original de los bancos `GateL`, `SelectL`, `JoinMe`, `JoinNumber` y `Counter`. |
| 14 | **Los contadores de una puerta que no abre**: el nuestro cuenta el intento fallido como entrada y reintento del bloque; el original no cuenta la entrada y cuenta los intentos en la llave. | `crates/genmsi-core/src/sim.rs` · rama `BlockKind::Gate` | La fila de la puerta del banco `GateL` tiene que salir `0 0 0`. |
| 15 | **La tabla.** El original reserva la última de las `Cantidad` clases para el desborde (`> a 80`); el nuestro la gasta en otra clase finita (`81 - 90`) y no tiene fila de desborde. Además la cabecera pierde el operando del atributo (`P$Atrib`) y los rótulos de las estadísticas están cambiados de sitio. | `crates/genmsi-core/src/stat.rs` · `Table::new` y `Table::tabulate`; `crates/genmsi-core/src/report.rs` · `render` (sección `TABLAS`) | Un test que compare la sección `TABLAS` del banco `Table` fila por fila con el reporte del original, y otro con un `Tabulate` que pase del último límite. |
| 16 | **La cola de la `StorageUse` no cuenta unidades**: el macrocode del original pasa `Cantidad` al `QUEUE` y al `DEPART`; el nuestro fija 1. | `crates/genmsi-core/src/program.rs` · `Compiler::expand`, brazo `"StorageUse"` (`QueueSpec { units: 1.0 }`); `crates/genmsi-core/src/stat.rs` · `Queue::enqueue`; `crates/genmsi-core/src/gpss.rs` · `block_statements` | Un banco con `Cantidad` 2 que compare `MAX` y `ENTRIES` de la cola con los del original (240 y 120 con 120 transacciones de 2 unidades). |
| 17 | **El `lint` de la CLI no ve lo que la capa de formato sí ve**: el modelo usa el almacén `Almacen` sin que ningún `StorageDef` lo declare y `genmsi lint` responde `diagnostics: none`. | `crates/genmsi-cli/src/commands/lint.rs` · `run`; `crates/genmsi-core/src/documents.rs` · `lint_document` | Un test del `lint` del banco `StorageUse` que fije `status: diagnostics`. |
| 18 | **El `EXAMINE` se compila con operandos vacíos y nadie lo dice**: el original rechaza el modelo con "Faltan parámetros". | `crates/genmsi-core/src/program.rs` · `Compiler::expand`, brazo `"Examine"`; o el validador | Un test que compile el banco `Examine` y compruebe el aviso. |
| 19 | **La forma del reporte**: los decimales enteros (`0` donde el original escribe `0,000`), el orden de las colas (alfabético contra numérico: el original pone `1, 2, …, 10` y el nuestro `1, 10, 2, …`) y la línea `SEMILLA`, que el original no trae. | `crates/genmsi-core/src/report.rs` · `number` y el recorrido de las colas (`entities.queues` es un `BTreeMap<String, Queue>`) | Un test que compare el reporte del banco `Select` con el del original. |
| 20 | **El `SELECT` sólo admite tres clases de entidad** de las 38 de la lista `entitySNAclass`; cualquier otra hace fallar la compilación con "unsupported SELECT entity class". | `crates/genmsi-core/src/program.rs` · `Compiler::expand`, el `match` de `ClaseSNA` | Un test que compile un `Select` con `ClaseSNA` "F" y compruebe `SELECT MIN param,1,10,,F`. |
| 21 | **La `SelectF` sólo implementa el operador por defecto**: los otros cinco de `op_Logic_Faci` (`U:Usada`, `I:Interrumpida`, `NI:NoInterrumpida`, `FNV:NoDisponible`, `FV:Disponible`) no existen, y el GPSS escribe siempre `NU`. | `crates/genmsi-core/src/program.rs` · `Compiler::expand` (rama `SelectF`); `crates/genmsi-core/src/gpss.rs` · `block_statements` | Un test que compile un `SelectF` con op `Usada` y compruebe `SELECT U param,1,10`. |
| 22 | **El `RemoveTrans` no lee `CompOp`** (la letra `E` es fija, donde el original escribe `MIN`/`MAX`) **ni el modo "Todos"** (no escribe `ALL`). | `crates/genmsi-core/src/gpss.rs` · `block_statements`, brazo `RemoveSpec` | Dos tests de GPSS: un `RemoveTrans` con `CompOp` `Minimo` → `REMOVE MIN UnGrupo,1,,otro,1`, y uno con `Modo` "Todos" → `REMOVE ALL ...`. |
| 23 | **El `Split` no lee la propiedad `Param`** y manda las copias a `Salida1` cuando `Salida2` está sin conectar, donde el original las manda al bloque siguiente de la secuencia. | `crates/genmsi-core/src/program.rs` · rama `"Split"` | Un banco con un `Split` de `Salida2` sin conectar que compare el número de entradas de cada bloque con el del original. |
| 24 | **El `Unlink` no lee `CompOp` ni `CompOpcion`**: la comparación es siempre `E` por parámetro. | `crates/genmsi-core/src/program.rs` · rama `"Unlink"` | Un banco con un `Unlink` de `CompOp` `Maximo` que compare la fila del reporte con la del original. |
| 25 | **El `Link` no lee `Parametro`** ni los modos de ordenación que no sean FIFO ni LIFO. | `crates/genmsi-core/src/program.rs` · rama `"Link"` | Un banco con un `Link` de modo `LIFO` y `Parametro` que compare el orden de la cadena con el del original. |

### En los bancos (`scripts/wine-oracle/make-benches.py`)

| # | Qué hay que arreglar | Dónde | La prueba que lo fijaría |
|---|---|---|---|
| 26 | **La etiqueta que el generador pone** (`"A_" + nombre`) no es la que pone el editor del original: para las cuatro puertas el macrocode dice `='moduleName'` (el nombre del módulo, sin `A_`), y el modelo del propio original lo confirma (`fixtures/models/TP6__Clase 6_3_JefeEducado.gmdl`: el módulo `GateF` tiene `mylabel="GateF"`). Con la etiqueta equivocada la fila de la puerta ni entra en la comparación de cifras del oráculo. | `scripts/wine-oracle/make-benches.py`, donde arma el `mylabel` de la entrada | El banco `GateF` con la etiqueta `GateF`, como en el modelo de referencia del original. |
| 27 | **El banco de `StorageUse` no declara el almacén**, y el original se niega a correrlo ("Referencia a una entidad storage inexistente"): sin reporte no hay cifras contra las que comparar. | `scripts/wine-oracle/make-benches.py` (un `StorageDef` en el banco) | El banco `StorageUse` con el `StorageDef` declarado: el original deja un reporte y se puede comparar. |
| 28 | **El banco de `Examine` no se puede comparar**: el original rechaza el modelo con "Error en linea 1 -> Faltan parámetros". No lo arregla el generador (la clase sólo declara `Grupo`, y los dos operandos vacíos los produce el propio macrocode `EXAMINEME` del original): es del lado del original. Para medirlo haría falta un banco `ExamineNumber` con todos sus operandos. | `scripts/wine-oracle/make-benches.py` no; hace falta un banco nuevo | Un banco con un `EXAMINE` que el original acepte, para tener su reporte. |

### Lo que ya está arreglando otra sesión

| # | Qué | Dónde | Qué deja de estar pendiente |
|---|---|---|---|
| 29 | **El almacén sin declarar.** Hoy el motor se inventa una capacidad de 1.000.000 y simula un `ENTER`/`LEAVE` de un almacén que nadie declara. | Rama `feature/almacen`, commits `0a4383f` ("fix(core): a storage only exists when the model declares it") y `5f1490c`; no está en este worktree (aquí `HEAD` es `82f6515`) | `StorageUse` deja de ser "pendiente" por esa causa (el original y el nuestro paran igual: error en el `ENTER`), `GateS` y `SelectS` dejan de crear la entidad de almacén y el `STORAGE` del reporte. **Lo demás de esas dos clases no lo arregla**: la condición del operador de la puerta y la letra de la fila del `SELECT` siguen en pie, porque lo que falla es la semántica, no el almacén. |

## Tasks

- **T1** — Fichas de StorageDef, StorageUse, FacilityUse, FacilityInt, GateF, GateL (S1, S2, S4) — hecho: las seis fichas están en "Fichas"; commit: `06ea66b`
- **T2** — Fichas de GateS, GateM, Select, SelectF, SelectS, SelectL (S1, S2, S4) — hecho: las seis fichas están en "Fichas"; commit: `06ea66b`
- **T3** — Fichas de SelectSNA, Match, Assemble, Split, Link, Unlink (S1, S2, S4) — hecho: las seis fichas están en "Fichas"; commit: `06ea66b`
- **T4** — Fichas de JoinMe, JoinNumber, RemoveMe, RemoveTrans, Examine, Mark, Counter, Table (S1, S2, S4) — hecho: las ocho fichas están en "Fichas"; commit: `06ea66b`
- **T5** — Comprobar el alcance de la rama `feature/almacen` sobre las diferencias medidas (S4, S8) — hecho: se compiló esa rama aparte (worktree `../genmsi-fase7-almacen`, `5f1490c`) y se corrieron los bancos afectados; el almacén deja de ser inventado y `StorageUse` para con error como el original, pero la condición de la `GateS` y la letra de la fila del `SELECT` siguen en pie; commit: `06ea66b`
- **T6** — Ensamblar el documento: tabla resumen, las 24 fichas y la lista final de cambios pendientes con archivo y prueba (S3) — hecho: las tres secciones están en este archivo; commit: `06ea66b`
- **T7** — Commit de las notas y entrega de la rama (S5) — hecho: rama `feature/fase7-inventario`, worktree `../genmsi-fase7`; commit: `06ea66b`

## Log

- **L1** — "Contexto: el bloque común de arriba.

   TRABAJA EN UN WORKTREE PROPIO, desde el HEAD actual:
     git worktree add ../genmsi-fase7 -b feature/fase7-inventario
     cd ../genmsi-fase7
   ESTA SESIÓN ES DE INVENTARIO: NO EDITES NADA DE crates/ NI DE docs/. El core
   lo tiene otra sesión; tú produces el mapa de lo que hay que arreglar.

   TAREA: el primer lote de la fase 7 — pasar clase por clase y decir, con
   números, en qué se parece y en qué no our motor y el original.

   EL MÉTODO (por clase):
   1. El banco de la clase: scripts/wine-oracle/benches/<Clase>.gmdl. Se genera
      con `python3 scripts/wine-oracle/make-benches.py` (artefacto ignorado por
      git, hay que generarlo).
   2. La respuesta del original: scripts/wine-oracle/out/<Clase>.gpss.txt y
      <Clase>.report.txt. Ya están en /tmp/bench-out (62 clases); si no están,
      se regeneran con `bash scripts/wine-oracle/run-benches.sh scripts/wine-oracle/benches
      /tmp/bench-out` (tarda cerca de una hora).
   3. La nuestra: `cargo run --release -p genmsi-cli -- gpss <banco>` y
      `cargo run --release -p genmsi-cli -- simulate <banco>`.
   4. Compara: verbo, operandos y nombre de cada sentencia
      (scripts/wine-oracle/compare-gpss.sh) y las cifras del reporte (el ejemplo
      crates/genmsi-core/examples/compare-benches.rs, con las tolerancias del
      oráculo: 20% en conteos, 35% en tiempos).
   5. Lee el macrocode y el ClassDescrip de la clase en
      Genmsi_20230711.backup/gClasses.xml: ahí está lo que el original dice que
      hace, con sus "Consideraciones".
   6. Anota por clase: los puertos, las propiedades y sus defaults, la
      semántica (qué hace y con qué regla) y las cifras. Y el veredicto:
      verificada / con diferencias / pendiente.

   EL LOTE (por este orden, que es el que más se ha tocado):
   StorageDef, StorageUse, FacilityUse, FacilityInt, GateF, GateL, GateS, GateM,
   Select, SelectF, SelectS, SelectL, SelectSNA, Match, Assemble, Split, Link,
   Unlink, JoinMe, JoinNumber, RemoveMe, RemoveTrans, Examine, Mark, Counter, Table.

   Lo que ya sabemos y no hay que repetir:
   - 62 de 62 bancos coinciden en el GPSS y 50 de 55 en las cifras (las cinco
     restantes son bancos cuyo reporte no trae cabecera).
   - El ALTER alteraba el parámetro que debía comparar: ya está corregido.
   - La primera transacción de un generador: ya está zanjada (media u offset).
   - La capacidad de un almacén sin declarar: lo está arreglando otra sesión.

   DUEÑO:
   - Si el cambio va en el core, NO lo hagas: déjalo escrito con el archivo, la
     función y la prueba que lo fijaría.
   - Si el cambio va en el catálogo o en el validador, tampoco: déjalo escrito.
   - Lo único que haces es el mapa y las notas.

   DONE CUANDO:
   - odd/tasks/notas-fase7-lote1.md: una ficha por clase (los cuatro puntos de
     arriba, el veredicto y la diferencia concreta), una tabla resumen al
     principio, y una lista final de los cambios pendientes con su arquivo y su
     prueba.
   - El commit de las notas en tu worktree, y me dices la rama."

- **L2** — Decisión de trabajo: el worktree sale de `main` en 82f6515, que no
  trae el arreglo del almacén sin declarar (está en `feature/almacen`, 5f1490c).
  Las diferencias que dependan de ese arreglo se anotan como "lo arregla otra
  sesión" y se nombran, para que T5 pueda medirlas aparte.
- **L3** — El binario del CLI se llama `genmsi`, no `genmsi-cli`: el comando del
  enunciado (`cargo run --release -p genmsi-cli -- gpss`) deja el ejecutable en
  `target/release/genmsi`.
- **L4** — Medición del lote: 24 de 24 bancos con el GPSS igual al del original
  (`scripts/wine-oracle/compare-gpss.sh`, "el GPSS coincide" en los 24) y 17 de
  24 con todas las cifras dentro de las tolerancias del oráculo (20% en
  conteos, 35% en tiempos). Veredictos: 7 verificadas, 15 con diferencias, 2
  pendientes. Las dos pendientes son las que el original no dejó comparar:
  `StorageUse` (el original se niega a simular un almacén sin declarar) y
  `Examine` (el original rechaza el modelo con "Faltan parámetros").
- **L5** — Dos sondas contra el original, porque los bancos del lote consultan
  un almacén que nadie declara y no distinguen la regla de la puerta: con el
  almacén declarado y vacío la `GateS` del original abre (61 entradas al
  `TERMINATE`) y la nuestra no; con el almacén declarado y lleno la del
  original se cierra. La regla del original es "abre cuando al almacén le queda
  sitio" (`SNF`), y el motor pregunta "¿está vacío?".
- **L6** — Las 24 fichas se escribieron primero como un archivo por clase bajo
  `odd/tasks/fase7-lote1/` (para que cuatro trabajadores escribieran en
  paralelo sin pisarse) y después se incorporaron a este documento, que es el
  entregable; el directorio se borró.
- **L7** — Los objetos de la medición se reparte así: el banco de cada clase
  (`scripts/wine-oracle/benches/<Clase>.gmdl`, generado por
  `make-benches.py`), el GPSS y el reporte del original (`/tmp/bench-out`),
  el nuestro (`target/release/genmsi gpss|simulate`) y las cifras del oráculo
  (el ejemplo `crates/genmsi-core/examples/compare-benches.rs`, con las
  tolerancias del oráculo).
- **L8** — Las notas viven en un solo commit (`06ea66b`), que es lo que pedía
  el encargo: un archivo con la tabla resumen, las 24 fichas y la lista final
  de cambios pendientes. El corte de ~400 líneas no aplica aquí: partir el
  documento en varios commits dejaría el entregable a medias, que es justo lo
  que el encargo no quiere.
