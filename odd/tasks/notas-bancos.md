# Notas: los tres cambios del generador de bancos que dejó el inventario del lote 1

Qué cambió en `scripts/wine-oracle/make-benches.py` (y sus pruebas) para que
los bancos le pregunten al original lo mismo que le preguntan el resto: los
operandos, los nombres y las entidades que la clase necesita. Y qué cambió en
las cifras del oráculo por hacerlo.

El trabajo es de la **fase 7** (`odd/tasks/genmsi-fase7-revision-clase-por-clase.md`),
y sus tres pendientes son las filas 26, 27 y 28 de la lista "En los bancos" de
`odd/tasks/notas-fase7-lote1.md`.

## Specs

- **S1** — "el banco tiene que pedirle al original lo mismo que le pide el resto (los operandos, los nombres y las entidades que la clase necesita), y el banco tiene que ser el que el inventario usó para esa clase."
- **S2** — "Los bancos son un artefacto generado: no se commitean (están en .gitignore). Lo que se commitea es el generador y sus pruebas."
- **S3** — "Después de cambiarlos, regenera los bancos y comprueba que el inventario sigue valiendo: los 24 bancos con el GPSS idéntico al del original y 17 de 24 con las cifras dentro de las tolerancias. Si un banco cambia y con él cambia un veredicto, es que el banco nuevo pregunta otra cosa: eso hay que decirlo, no callarlo."
- **S4** — "NINGÚN commit lleva pie de atribución (nada de "Co-Authored-By" ni similar). El mensaje del commit explica qué cambió y por qué, y nada más."
- **S5** — "Son de scripts/wine-oracle/, no del núcleo: no tocan crates/."

## Los tres cambios

### 1. La etiqueta del bloque: la que escribe el macro de su clase

**Lo que había.** El generador ponía `"A_" + nombre` en todos los bancos. Eso
es lo que hacen la mayoría de las clases, pero no lo que hacen todas: el
macrocode de la clase dice qué escribe en la entrada del bloque
(`IN Entrada1:'Ingreso'='moduleName'`), y hay tres formas:

| El token del macro | La etiqueta | Las clases |
|---|---|---|
| `A_moduleName` / `A_ModuleName` | `A_` + el nombre del módulo | 41 clases |
| `moduleName` / `modulename` | el nombre del módulo | las cuatro puertas, las dos multifurcaciones y el macro del almacenamiento |
| un literal | el literal | `Clona` (el `Split`), `Entrada1` (el `Transfer`) |

**La evidencia de que el original no lee la etiqueta del archivo.** El banco
viejo de la `GateF` traía `mylabel="A_GateF"` y el reporte del original escribía
`GateF`; el del `Multifurcacion` traía cinco terminadores con
`mylabel="A_Terminate_SalidaProb2"` … y el original escribía `A_Terminate`,
`A_Terminate1`, `A_Terminate2`, `A_Terminate3`, `A_Terminate4`: el nombre del
módulo con el prefijo del macro, y un número cuando dos bloques se llaman
igual. La etiqueta del archivo no entra en el nombre: el original la recalcula
con el macro de la clase y el nombre del módulo.

**Lo que se hizo.** `parse_class` lee el token del macro de cada puerto de
entrada (`Port.label_token`), y `label_of` lo resuelve contra el nombre del
módulo. Los bloques que terminan una corrida se llaman `Terminate`,
`Terminate1`, … , como los llama el original, y el enlace de un bloque apunta
a la etiqueta del bloque que alimenta.

**Lo que se mide.** La columna `LOC` de la tabla de bloques del reporte, que
sale del `mylabel` del puerto (`crates/genmsi-format/src/model.rs`, fn
`Module::label`): los bancos con las mismas etiquetas que el original pasan
de **46 de 55 a 56 de 58**. Los dos que quedan no son de la etiqueta: el
`Match` y el `GateM` — nuestro motor expande el `Match` en un bloque y el
original escribe dos (un `TRANSFER` y un `MATCH`), que es el pendiente número
11 del core.

### 2. El banco del `StorageUse` declara el almacén que usa

**Lo que había.** El banco usaba el almacén `Almacen` sin que nadie lo
declarara, y el original se negaba a correrlo:

```
Tiempo de simulación: 36,  Transacción activa: 1
ERROR!!Referencia a una entidad storage inexistente. (Storage: -1).
```

Es decir: el banco no le preguntaba nada al original sobre esa clase.

**Lo que se hizo.** El banco lleva ahora un bloque `StorageDef` que declara el
almacén que la clase usa (el nombre sale de la propiedad `Storage` de la
clase, no del default de la declaración). El generador lo dice por pantalla:
"el banco lleva un bloque StorageDef: la clase usa una entidad que el original
sólo simula si el modelo la declara".

**Lo que se mide.** El original ahora corre y deja su reporte:

```
    2       60       1     0              1  GENERATE        60,30
    3       59       0     0     StorageUse  QUEUE           qAlmacen
    4       59       0     0              3  ENTER           Almacen
    ...
 QUEUE            MAX  CONT. ENTRIES ENTRIES(0) AVE.CONT. AVE.TIME AVE.(-0) RETRY
  QALMACEN         1     0      59        59     0,000      0,000    0.000    0
 STORAGE           CAP. REMAIN. MIN.  MAX. ENTRIES AVL. AVE.C.  UTIL. RETRY DELAY
  ALMACEN          10     10     0     1      59   1   0,163   0,016    0     0
```

y las siete métricas de la clase caen dentro de las tolerancias del oráculo
(20% en conteos, 35% en tiempos): **7 de 7**. La clase pasa de "pendiente"
a "verificada" en el inventario.

### 3. El banco del `Examine` (y del `ExamineNumber`) conecta su salida de fallo

**Lo que había.** El original rechazaba el modelo:

```
Error en linea     1 -> Faltan parámetros
```

**Por qué.** La sentencia `EXAMINE A,B,C` de GPSS lleva tres operandos: el
grupo, el número (vacío en modo transacción) y **el bloque al que va la
transacción cuando no está en el grupo**. El tercero es obligatorio —lo dice
el manual del simulador (`VISUALSIS.HLP`): "Número de bloque. Destino
alternativo para la transacción activa si no es encontrado ningún miembro.
Requerido"—, y el macro `EXAMINEME` lo toma de la salida de fallo de la
clase. Con el banco viejo esa salida estaba sin conectar, el operando
quedaba vacío y el original no compilaba el modelo.

**Lo que se hizo.** Las dos clases declaran su salida de fallo conectada, con
un terminador propio (`Terminate1`), como ya hacían el `Unlink` y las
multifurcaciones.

**Lo que se mide.** El original ahora corre y deja su reporte:

```
    2       61       0     0      A_Examine  EXAMINE         UnGrupo,,A_Terminate1
    3        0       0     0    A_Terminate  TERMINATE
    4       61       0     0    A_Terminate1  TERMINATE
```

nuestras cifras coinciden con las suyas (**3 de 3** en el `Examine`, **2 de 2**
en el `ExamineNumber`), y nuestro reporte escribe la misma fila que el suyo
(`EXAMINE UnGrupo,,A_Terminate1`).

**El precio, y es un precio del core.** Nuestro *listado* del programa GPSS
escribe `EXAMINE UnGrupo,,` — sin el tercer operando — y el del original
escribe `EXAMINE UnGrupo,,A_Terminate1`. La comparación de GPSS, que compara
sentencia por sentencia, ve la diferencia: los bancos del `Examine` y del
`ExamineNumber` dejan de coincidir con el original en el texto.

Eso no es un defecto del banco: es una diferencia entre nuestro motor y el
original que el banco viejo **tapaba** (el original no compilaba el modelo, así
que no había nada que comparar). Y está localizada: el reporte ya escribe el
destino (`crates/genmsi-core/src/report.rs`, fn `operand_parts`, rama
`BlockKind::Examine`, línea 629, con `port_target_label(program, block,
"Salida2")`), y el listado no (`crates/genmsi-core/src/gpss.rs`, fn
`block_statements`, rama `BlockKind::Examine`, línea 532, que escribe
`format!("{},,", spec.group)`). El arreglo del core es el mismo que el de la
sentencia `TRANSFER` (línea 666): añadir el nombre del destino de la segunda
salida. Lo fijaría un test de `crates/genmsi-core/tests/report_gpss.rs` que
compruebe que el listado del banco `Examine` sale
`EXAMINE UnGrupo,,A_Terminate1`.

Si se prefieriera no tocar ese punto del core, el arreglo del banco es una
línea: quitar `"Examine"` y `"ExamineNumber"` de `EXTRA_OUTPUTS` en
`scripts/wine-oracle/make-benches.py`. Pero entonces el original vuelve a
rechazar el modelo y la clase vuelve a quedar sin comparar.

## Las cifras, antes y después

La medición es la del oráculo: `bash scripts/wine-oracle/compare-benches.sh
scripts/wine-oracle/benches <las respuestas del original>`, con las respuestas
del original en `/tmp/bench-out` (las de antes) y en `/tmp/bench-out-new` (las
de los doce bancos que cambiaron, vueltas a correr contra el original).

| | Antes | Después |
|---|---|---|
| GPSS: bancos que coinciden con el original | **62 de 62** | **60 de 62** |
| Cifras: bancos con reporte del original | 55 | 58 |
| Cifras: bancos que coinciden en todas sus métricas | 51 | 52 |
| Cifras: bancos sin comparación posible | 11 | 8 |
| Etiquetas `LOC` iguales a las del original | 46 de 55 | 56 de 58 |

**Lo que baja.** Los dos bancos de GPSS que difieren son el `Examine` y el
`ExamineNumber`, y difieren en un solo operando: el destino alternativo del
`EXAMINE`, que nuestro listado no escribe (el punto del core de arriba). No
es una diferencia de comportamiento —el motor sends las transacciones por la
salida de fallo igual que el original, y las cifras coinciden— sino del texto
del listado.

**Lo que sube.** Tres bancos que antes no se comparaban ahora sí:

| Banco | Antes | Después |
|---|---|---|
| `StorageUse` | sin reporte (el original lo rechazaba) | 7 de 7, coinciden |
| `Examine` | sin reporte (el original lo rechazaba) | 3 de 3, coinciden |
| `ExamineNumber` | sin reporte (el original lo rechazaba) | 2 de 2, coinciden |

Y las etiquetas de las puertas, que antes no se comparaban porque no coincidían
(el oráculo empareja las filas de bloque por su `LOC`), ahora sí, y eso
descubre dos diferencias del core que estaban tapadas:

| Banco | Antes | Después | La diferencia que aparece |
|---|---|---|---|
| `GateF` | 3 de 3 | 4 de 4, coinciden | — |
| `GateL` | 1 de 1 | 1 de 2 | la puerta que no abre: nuestro motor cuenta el intento fallido como entrada (pendiente 14 del core) |
| `GateS` | 0 de 1 | 1 de 2 | la condición de la puerta: el original abre cuando al almacén le queda sitio (SNF), el nuestro cuando no está vacío (pendiente 1 del core) |
| `GateM` | 1 de 2 | 2 de 3 | — |

Lo mismo con las filas de los terminadores que el original escribe con
`A_Terminate1`, `A_Terminate2`, … y que antes no se comparaban:

| Banco | Antes | Después |
|---|---|---|
| `Unlink` | 2 de 2 | 3 de 3, coinciden |
| `Multifurcacion` | 1 de 1 | 5 de 6 |
| `Split` | 1 de 1 | 2 de 2, coinciden |
| `Transfer` | 1 de 1 | 2 de 2, coinciden |

### El lote 1, clase por clase

Las 26 clases del lote del inventario, con el GPSS y las cifras de antes y de
después (`/` separa lo de antes de lo de después):

| Clase | GPSS | Cifras | Veredicto |
|---|---|---|---|
| StorageDef | coincide | 2/2 | verificada |
| StorageUse | coincide | sin reporte → **7/7** | **verificada** (antes: pendiente) |
| FacilityUse | coincide | 7/7 | verificada |
| FacilityInt | coincide | 7/7 | verificada |
| GateF | coincide | 3/3 → **4/4** | verificada |
| GateL | coincide | 1/1 → **1/2** | con diferencias (la puerta que no abre) |
| GateS | coincide | 0/1 → **1/2** | con diferencias (la condición de la puerta) |
| GateM | coincide | 1/2 → **2/3** | con diferencias |
| Select | coincide | 32/32 | verificada |
| SelectF | coincide | 4/4 | verificada |
| SelectS | coincide | 2/2 | verificada |
| SelectL | coincide | 2/2 | verificada |
| SelectSNA | coincide | 5/5 | verificada |
| Match | coincide | 1/2 | con diferencias (sin cambios) |
| Assemble | coincide | 2/2 | verificada |
| Split | coincide | 1/1 → **2/2** | verificada |
| Link | coincide | 1/2 | con diferencias (sin cambios) |
| Unlink | coincide | 2/2 → **3/3** | verificada |
| JoinMe | coincide | 2/2 | verificada |
| JoinNumber | coincide | 2/2 | verificada |
| RemoveMe | coincide | 2/2 | verificada |
| RemoveTrans | coincide | 2/2 | verificada |
| Examine | **difiere** | sin reporte → **3/3** | **con diferencias**: el listado del GPSS no escribe el destino alternativo |
| Mark | coincide | 2/2 | verificada |
| Counter | coincide | 2/2 | verificada |
| Table | coincide | 0/0 | con diferencias (sin cambios) |

**Lo que cambia de veredicto, y por qué.**

- `StorageUse` pasa de "pendiente" a "verificada": el banco ahora le pregunta
  al original lo mismo que le preguntaba al resto (con el almacén declarado),
  y el original responde.
- `Examine` pasa de "pendiente" a "con diferencias": el banco pregunta por
  fin, y la respuesta muestra que nuestro listado no escribe el destino
  alternativo del `EXAMINE`. El banco nuevo pregunta otra cosa, y por eso el
  veredicto cambia; no se esconde.
- `GateL` y `GateS` pierden una métrica que antes no se comparaba: la fila de
  la puerta, que ahora se empareja porque la etiqueta coincide. Las dos
  diferencias son del core y ya estaban anotadas (los pendientes 1 y 14).

## Lo que queda para el core

| # | Qué | Dónde | La prueba que lo fijaría |
|---|---|---|---|
| A | **El listado del `EXAMINE` no escribe el destino alternativo.** El original escribe `EXAMINE UnGrupo,,A_Terminate1` y el nuestro `EXAMINE UnGrupo,,`. El reporte ya lo escribe bien. | `crates/genmsi-core/src/gpss.rs` · fn `block_statements`, rama `BlockKind::Examine` (línea 532), como hace la rama `BlockKind::Transfer` (línea 666) | Un test de `crates/genmsi-core/tests/report_gpss.rs` que compruebe que el listado del banco `Examine` sale `EXAMINE UnGrupo,,A_Terminate1` |
| B | **La fila de la puerta que no abre.** Nuestro motor cuenta el intento fallido como entrada y reintento del bloque; el original no cuenta la entrada y cuenta los intentos en la llave. La fila de la `GateL` sale `60 60 60` (con la semilla de esta corrida) donde el original escribe `0 0 0`. (Ya estaba anotado como el pendiente 14 del core; aquí sale a la luz porque la etiqueta de la puerta por fin coincide.) | `crates/genmsi-core/src/sim.rs` · rama `BlockKind::Gate` | La fila de la puerta del banco `GateL` tiene que salir `0 0 0` |
| C | **La condición de la puerta sobre un almacén.** El original abre cuando al almacén le queda sitio (`SNF`); el nuestro abre cuando no está vacío. (Ya estaba anotado como el pendiente 1 del core; aquí sale a la luz por lo mismo.) | `crates/genmsi-core/src/program.rs` · `Compiler::expand`, rama `"GateS"`; `crates/genmsi-core/src/sim.rs` · `gate_condition` | La sonda de la sección "Sondas" de `odd/tasks/notas-fase7-lote1.md` |

## Las pruebas del generador

`scripts/wine-oracle/test_make_benches.py` (nuevo) fija las tres reglas:

```
python3 -m unittest discover -s scripts/wine-oracle -p "test_*.py"
```

- `LabelRule` — la etiqueta sale del token del macro (con prefijo, sin
  prefijo y literal), y el enlace apunta a la etiqueta del bloque que alimenta.
- `Declarations` — el banco del `StorageUse` declara el almacén que la clase
  usa, con el nombre que la clase usa.
- `FailureOutput` — la salida de fallo del `Examine` apunta a un terminador,
  y el terminador se llama como el original lo llama (`Terminate1` →
  `A_Terminate1`).
- `TheRealCatalogue` — con el catálogo real: las cuatro puertas se nombran
  como el módulo, el `Split` como `Clona`, el `Transfer` como `Entrada1`, el
  `StorageUse` como `StorageUse`, los demás con el prefijo `A_`; los 66 bancos
  se escriben; y el mismo catálogo da los mismos bytes.

Los tests que leen el catálogo real se saltan si el backup del original no
está (`Genmsi_20230711.backup/`); los otros tres corren siempre.

## Cómo repetir la medición

```
# 1. los bancos
python3 scripts/wine-oracle/make-benches.py

# 2. las respuestas del original (tarda unos 25 segundos por banco)
bash scripts/wine-oracle/run-benches.sh scripts/wine-oracle/benches /tmp/bench-out-new

# 3. la comparación
bash scripts/wine-oracle/compare-benches.sh scripts/wine-oracle/benches /tmp/bench-out-new

# 4. las pruebas del generador
python3 -m unittest discover -s scripts/wine-oracle -p "test_*.py"

# 5. las del motor
cargo test --workspace
```

## Tasks

- **T1** — La etiqueta del bloque, según el token del macro de su clase — hecho: `label_of` en `scripts/wine-oracle/make-benches.py`, con sus pruebas.
- **T2** — El banco del `StorageUse` declara su almacén — hecho: `DECLARATIONS` en el generador, con sus pruebas.
- **T3** — El banco del `Examine` (y del `ExamineNumber`) conecta su salida de fallo — hecho: `EXTRA_OUTPUTS` en el generador, con sus pruebas.
- **T4** — Regenerar los bancos y volver a correr el original sobre los que cambiaron — hecho: doce bancos, en `/tmp/bench-out-new`.
- **T5** — Medir: GPSS, cifras y etiquetas, antes y después — hecho: las tres tablas de arriba.
- **T6** — Las pruebas del motor — hecho: `cargo test --workspace` en verde.
- **T7** — El commit, sin pie de atribución — hecho: rama `feature/bancos-lote1`.

## Log

- **L1** — "Trabajo en /home/lucas/Documentos/Workspace/Genmsi.

   [el bloque común de contexto, igual que en las sesiones anteriores]

   LEE PRIMERO:
   - odd/tasks/genmsi-fase7-revision-clase-por-clase.md
   - odd/tasks/notas-fase7-lote1.md, sobre todo la sección "Cambios pendientes":
     al final de la lista están los tres cambios que NO son del núcleo, sino del
     generador de bancos.

   TRABAJA EN UN WORKTREE PROPIO, desde el HEAD actual:
     git worktree add ../genmsi-bancos -b feature/bancos-lote1
     cd ../genmsi-bancos

   TAREA: los tres cambios del generador de bancos que dejó el inventario del
   lote 1. Son de scripts/wine-oracle/, no del núcleo: no tocan crates/.

   QUÉ HACER:
   - Arregla esos tres, con el criterio de siempre: el banco tiene que pedirle
     al original lo mismo que le pide el resto (los operandos, los nombres y las
     entidades que la clase necesita), y el banco tiene que ser el que el
     inventario usó para esa clase.
   - Los bancos son un artefacto generado: no se commitean (están en .gitignore).
     Lo que se commitea es el generador y sus pruebas.
   - Después de cambiarlos, regenera los bancos y comprueba que el inventario
     sigue valiendo: los 24 bancos con el GPSS idéntico al del original y 17
     de 24 con las cifras dentro de las tolerancias. Si un banco cambia y con
     él cambia un veredicto, es que el banco nuevo pregunta otra cosa: eso hay
     que decirlo, no callarlo.

   DONE CUANDO:
   - `python3 scripts/wine-oracle/make-benches.py` genera los 66 bancos, y
     `bash scripts/wine-oracle/compare-benches.sh scripts/wine-oracle/benches
     <directorio con las respuestas del original>` da los mismos números que antes
     (62/62 en el GPSS, 50/55 en las cifras), o mejores.
   - `cargo test --workspace` en verde.
   - Las notas de lo que cambiaste en odd/tasks/notas-bancos.md (nuevo).
   - El commit en tu worktree, y me dices la rama.

   Y una regla del repositorio, para lo que commits: NINGÚN commit lleva pie de
   atribución (nada de "Co-Authored-By" ni similar). El mensaje del commit
   explica qué cambió y por qué, y nada más."

- **L2** — El inventario cuenta 26 clases en su tabla resumen y dice "24" en
  el texto; la medición de aquí usa las 26 de la tabla (las del lote del
  encargo) y el mismo instrumento de siempre, `compare-benches.sh`.
- **L3** — La etiqueta del bloque no sale del `mylabel` del archivo: el
  original la recalcula con el macro de la clase y el nombre del módulo. La
  prueba son los cinco terminadores del banco del `Multifurcacion`, que el
  original nombra `A_Terminate` … `A_Terminate4` aunque en el archivo sus
  etiquetas sean `A_Terminate_SalidaProb2` … .
- **L4** — La fila `LOC` de los bloques sin nombre sigue siendo distinta: el
  original escribe el número de línea del programa y el nuestro un contador
  (pendiente 19 del core). No lo mide esta sesión.
- **L5** — La diferencia del `EXAMINE` no es del banco: es del listado de
  nuestro motor, que no escribe el tercer operando (el destino de la salida
  de fallo). El reporte sí lo escribe. Queda anotado como pendiente A del
  core, con el archivo y la función.
- **L6** — La corrida del original sobre los doce bancos que cambiaron tuvo
  que repetirse: la primera vez chocó con otra sesión que estaba usando el
  mismo prefijo de Wine y el mismo display. Con un display propio
  (`XDISPLAY=:97`) y esperando a que no hubiera ningún `genmsi.exe` corriendo,
  los doce responden.
- **L7** — Las respuestas del original son aleatorias (cada corrida sortea sus
  transacciones), así que los conteos cambian de una corrida a otra; lo que
  no cambia son las etiquetas, los verbos y los operandos, que es lo que
  comparan las dos columnas.
- **L8** — El trabajo va en dos commits: el generador con sus pruebas y el
  README (505 líneas, el corte de ~400 se pasa en poco), y estas notas. El
  corte queda aquí: la mitad del generador (la etiqueta y las declaraciones)
  es un trabajo unitario y las notas son la documentación de la medición, que
  depende de las dos mitades para estar completa.
