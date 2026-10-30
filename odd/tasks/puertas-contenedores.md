# puertas-contenedores — las puertas y el contenedor son ciudadanos de primera clase

Las puertas (`PortIn`/`PortOut`) y el contenedor tienen su propio camino
paralelo en seis capas del sistema, y tres reglas del mismo concepto están
escritas en tres lugares que hoy no coinciden. De ahí los tres síntomas que
reportó el usuario. Esta unidad las cierra: la puerta deja de ser un
`Port` con layout y un nodo de id `"in-3"`, y pasa a ser una cosa colocada
—con posición, con figura, con identidad y con un solo camino de escritura—
al lado de los bloques.

Rama: `feature/puertas-contenedores`. Verificación: `cargo test --workspace`,
`node --test tests/*.test.ts` en `crates/genmsi-app`, `bunx tsc --noEmit`.

## Specs

### S1 — Las puertas no se superponen al reacomodar

> "al reordenar dentro de un Containerlos los bloques de portin y portout se
> ponen uno encima de otro si no tienen conexion con alguien. Esto no pasa con
> los bloques en general, por esto pienso de que estan mal implementados."

Dos puertas de un mismo contenedor **nunca** comparten un lugar, y ninguna se
va al doble de distancia. La razón: las puertas que el flujo no alcanza son,
semánticamente, "bloques que el flujo no alcanza", y esa machinery ya existe
(`grid_of_scope` + `place_free`): la usan los bloques sin flujo, y nunca
supone dos.

### S2 — Reacomodar compone las puertas como compone los bloques

> "Si dentro de un Container coloco cualquier bloque, los bloques de entrada y
> salida se mueven junto con el, y el reordenamiento no esta funcionando con
> ellos, es como si el portin y portout no fueran bloques como cualquier otro
> (literalmente deberian compartir codigo con los bloques normales)"

Reacomodar un ámbito compone **todo** lo que hay en él: los bloques y las
puertas. Una puerta es un bloque de la imagen, y se comporta como tal: el
reacomodar recalcula su lugar como recalcula el de los bloques, y ninguna
puerta queda encima de otra ni encima de un bloque.

El lugar que el lector le dio a una puerta dragging se guarda en el modelo
—como el de un bloque— y lo conserva hasta el próximo reacomodar de ese
ámbito: es lo mismo que pasa con un bloque.

### S3 — El lugar de una puerta se escribe por un solo camino

El modelo tiene **una** lista de cosas colocadas por ámbito, no dos: una
posición de bloque y una posición de puerta son el mismo tipo de dato con
identidades distintas. Un solo aplicador, un solo paso de deshacer, un solo
comando.

### S4 — Toda composición da lugar a las puertas

La composición de **todo** el modelo (la del CLI y la del MCP) también da
lugar a las puertas, como la de un ámbito. Hoy la del modelo entero no las
conoce: una puerta compuesta por el CLI queda sin lugar.

### S5 — El contenedor cuenta todas las entradas

> "Si un Container tiene varias entradas o salidas se buguiean entre si, a
> veces no detectan las instancias que entran etc."

Las entidades que pasaron por un contenedor son **los cruces de su frontera**,
contados desde el trace. La puerta de entrada que se mire no cambia el número,
y el número es el mismo en vivo y en reproducción.

### S6 — La entidad se ve entrar y salir del contenedor

> "a veces la instancia que entra a Container visualmente no hace la animacion,
> a veces no la hace al salir, pero no se como reproducirlo porque parece en
> ocasiones raras."

La entidad que entra por una puerta de un contenedor se ve entrar **por esa
puerta**, y se ve salir por la que le corresponde. Un contenedor con varias
puertas de entrada no confunde las instancias entre sí.

### S7 — La puerta tiene una identidad, no una cadena

La puerta es una cosa con identidad propia en la app: la selección, el panel
de propiedades, el portapapeles y el foco la tratan como lo que es, sin
ramificar el código por `"in-3"`.

### S8 — El nombre de una puerta numera las puertas de su clase

> "al generar portin como portout comparten el numero del nombre para sumar,
> esto no deberia ser asi, por ejemplo ahora sui pongo un portin se llama
> entrada1 y si pongo otro portout se llama salida2 cuando deberia llamarse
> salida 1"

Las puertas de una clase se numeran entre sí: las de entrada son `Entrada1`,
`Entrada2`, ... y las de salida `Salida1`, `Salida2`, ... El **id** del puerto
es otra cosa, y sigue siendo único entre todas las puertas del contenedor
(un enlace nombra un puerto por su id). El editor original numera los nombres
así (`Entrada 1` y `Salida 1` en los modelos de referencia).

### S9 — El contenedor cuenta las entradas aunque su flujo sean sus puertas

> "El conteioner no cuenta las entidades que entran a el si solo hay un portin
> y portout conectados entre si."

Un contenedor cuyo flujo es sólo sus puertas (la entrada conectada con la
salida) cuenta las entidades que entraron por él, como cualquier otro.

## Tasks

| T | Specs | Qué | Ruta | Estado | Commit |
|---|---|---|---|---|---|
| T1 | S1, S2 | Las puertas sin flujo van a la grilla del ámbito, como los bloques sin flujo: se cae la rama `pending`, la superposición y el doble desplazamiento | format | **hecho** | — |
| T2 | S3 | Una lista de colocaciones con identidad discriminado: un aplicador, un deshacer, un comando | format, engine | **hecho** | — |
| T3 | S5 | Las cifras del contenedor salen de los cruces de frontera, siempre; se borra el heuristic de la primera puerta | app | **hecho** | — |
| T4 | S6 | La entidad cruza el contenedor por la puerta correcta: la cadena de enlaces del pasaje y el círculo que la recorre | app | **hecho** | — |
| T5 | S7 | La identidad de la puerta en la app es data, no una cadena que se parsea | app | **hecho** | — |
| T6 | S7 | El botón de borrar borra las puertas seleccionadas, como borra los bloques | app | **hecho** | — |
| T7 | S8, S9 | El nombre de una puerta numera las puertas de su clase; y el contenedor cuenta las entradas cuando el flujo es sólo sus puertas | format, app | **hecho** | — |

## Log

### L1 — El pedido del usuario (verbatim)

> "buenas! quiero que hagamos un analicis profundo sobre la implementacion y
> funcionamiento de los bloques de PortOut, PortIn y Container. En cuanto a
> buenas practicas, si respetan la reusabilidad, etc. Porque? porque realmente
> noto bugs o fallos entorno a ellos, por eso siento que de que estan
> reutilizando codigo perteneciente a los bloques. Ejemplos de porque intuyo
> estas cosas:
>  - Si un Container tiene varias entradas o salidas se buguiean entre si, a
> veces no detectan las instancias que entran etc.
>  - a veces la instancia que entra a Container visualmente no hace la
> animacion, a veces no la hace al salir, pero no se como reproducirlo porque
> parece en ocasiones raras.
>  - al reordenar dentro de un Containerlos los bloques de portin y portout se
> ponen uno encima de otro si no tienen conexion con alguien. Esto no pasa con
> los bloques en general, por esto pienso de que estan mal implementados."

### L2 — La pregunta del usuario sobre la reutilización (verbatim)

> "bien y pregunta, no nos combiene incluir tanto al contenedor como a ambas
> puertas a los bloques? para que puedan reutilizar lo que necesiten de ellos?
> si realmente tienen muchas cosas parecidas entre ellos con salvedades?"

### L3 — La decisión del usuario sobre el formato (verbatim)

> "me parece perfecto tu plan! y la respuesta al 7 es un si!"

La respuesta es a la pregunta 7 del análisis: **el `.gmdl` que guarda el
lector tiene que seguir siendo legible por el editor original**. Con eso, las
puertas son una primera clase **en el código**, no en el archivo: `<input>` y
`<output>` siguen siendo lo que son, y la identidad de la puerta vive en la
app, pero con un tipo discriminado en vez de una cadena.

### L4 — Lo que encontró el análisis (evidencia)

Tres síntomas, tres causas raíz, todas reproducidas antes de tocar código:

1. **La superposición** (`crates/genmsi-format/src/layout.rs:735-747`): las
   puertas sin flujo caen en el hueco libre con dos contadores **por clase**
   sobre la **misma** base → la primera entrada y la primera salida quedan en
   el mismo píxel. Reproducido: 1 entrada + 1 salida sin enlaces → las dos en
   `(80, 80)`; 2 + 2 → dos superposiciones. Y el hueco libre viene en
   coordenadas absolutas (`layout.rs:784-800`) mientras la composición corre un
   shift del ámbito por las puertas (`layout.rs:762-773`) → con un hijo en
   `(300, 400)` las puertas caen en `(600, 880)`, el doble. Y
   `layout.rs` **nunca lee `Port.layout`**: cada reacomodar recalcula las
   puertas desde cero y borra el lugar que el lector les dio.

2. **Las cifras** (`crates/genmsi-app/src/monitor/counts.ts:278-292`): el
   "pasadas" de un contenedor sale de la **primera** puerta de entrada que
   tenga enlace. Con varias puertas, las instancias que entraron por las
   demás no se cuentan. Y la ruta live del monitor llama `countsWithContainers`
   **sin** el mapa de cruces (`crates/genmsi-app/src/components/MonitorView.tsx:347`),
   mientras que la de reproducción sí lo pasa (`MonitorView.tsx:123-128`): en
   vivo y en reproducción el contenedor muestra dos números distintos.

3. **La animación** (`crates/genmsi-core/src/borders.rs:293-311`): los hops
   publican el par de **extremos** del flujo, replicado en todos los enlaces
   del pasaje → el par es ambiguo y la app mapea par→enlace con "el primero
   gana" (`crates/genmsi-app/src/graph/graph.ts:540-547`). Con un contenedor
   cuyo flujo es sólo sus puertas, el trace registra **un** movimiento
   (fuera→fuera) y la bola entra a la caja del contenedor y nunca sale.
   Además, una puerta de entrada **sin enlace interno** no la lleva ningún
   enlace → dentro del contenedor no hay nada que dibujar, y dos puertas que
   llegan al mismo bloque interno desde el mismo bloque externo publican el
   mismo par → la entidad que entró por la puerta 2 se dibuja por la puerta 1.

### L5 — La decisión de diseño

1. **Las puertas no son módulos del archivo.** `<input>`/`<output>` siguen
   siendo lo que son: los 25 modelos de referencia y el oráculo del editor
   original mandan, y el resolver de bordes (`core/borders.rs`) indexa por
   `(ámbito, módulo, ioid)`.
2. **Una puerta es una "cosa colocada"**, como un bloque: tiene posición,
   tamaño, ámbito, figura, y una identidad. Lo que la distingue de un bloque
   son **datos**, no caminos de código: no simula, no es módulo, tiene un
   solo puerto, no es dueña de sus enlaces.
3. **Las tres reglas duplicadas se cierran una por una**: el hueco libre de
   las puertas (una sola), el flujo de un ámbito (formato y motor, la misma),
   y la puerta de entrada del contenedor (los cruces, una sola).

### L6 — Las reproducciones (guardadas en `/tmp/genmsi-repros/`)

- `scratch_doors.rs` — la superposición y el doble desplazamiento de las
  puertas (`genmsi-format`).
- `scratch_only_doors.rs` — un contenedor cuyo flujo es sólo sus puertas: el
  trace registra un solo movimiento y los hops son ambiguos (`genmsi-core`).
- `scratch_container_trace.rs` — una puerta de entrada sin enlace interno: los
  hops del enlace interno salen vacíos (`genmsi-core`).
- `scratch_routes.test.ts` — dos puertas al mismo destino: el par es el mismo
  y la app elige siempre el primer enlace (`genmsi-app`).

### L7 — S2 cambió: el reacomodar compone las puertas como compone los bloques

El plan de la sesión decía que el motor iba a **respetar el lugar** que el
lector le dio a una puerta (`Port.layout`), para que "reacomodar" no fuera
destructivo. Al implementarlo se vio que eso contradice lo que el usuario
pidió: las puertas deben comportarse **como los bloques**, y el reacomodar ya
recompone los bloques (el lugar que el lector les dio tampoco sobrevive).

Conservar el lugar de las puertas y no el de los bloques sería una asimetría
nueva: la puerta sería lo único del ámbito que no se recompone, y volvería a
ser "una cosa aparte". Por eso **S2 se reescribió**: el reacomodar compone
las puertas como compone los bloques, y lo que garantiza es que **nada
queda encima de nada**.

Lo que sí se conserva: el lugar que el lector le dio a una puerta dragging se
guarda en el modelo (como el de un bloque) y lo tiene hasta el próximo
reacomodar de ese ámbito.

### L8 — T1 (hecho): las puertas sin flujo van a la grilla

La rama `pending` de `compose_scope` (el "hueco libre" con un contador por
clase) desaparece. Las puertas que el flujo no alcanza son, semánticamente,
**bloques que el flujo no alcanza**, y esa machinery ya existía:
`grid_of_scope` + `place_free`. Ahora las dos cosas van a **la misma grilla**.

De paso caen:

- **la superposición**: la grilla no superpone dos cosas nunca (es lo que ya
  garantizaban los bloques);
- **el doble desplazamiento**: `free_spot_of_scope` devolvía coordenadas
  absolutas y la composición corría el shift del ámbito por encima; la grilla
  trabaja en coordenadas del ámbito, como los bloques;
- **el `free_spot_of_scope`**: la composición ya no lo usa. Queda como helper
  público del crate (la app tiene el suyo en TypeScript: T2 lo unifica).

`place_free` pasó a ser una función pura: recibe las cosas de la grilla y la
caja del flujo, y devuelve el lugar de cada una. La usa las dos composiciones
(la del modelo entero y la de un ámbito).

Dos tests nuevos, los dos en RED antes del arreglo:

- `the_doors_that_carry_no_flow_do_not_share_a_place` — dos puertas sin flujo
  no comparten lugar.
- `the_composition_of_a_scope_translates_with_the_model` — el ámbito se
  compone en el mismo sistema de coordenadas que el modelo: si el modelo se
  mueve, la imagen se mueve lo mismo. Éste es el que caza el doble
  desplazamiento.

Dos tests existentes se ajustaron a su intención: el flujo del.scope y sus
bloques son dos cosas distintas (los bloques que el flujo no alcanza van a la
grilla, que está al nivel de las puertas), así que `the_ports_of_the_scope_are_the_ends_of_the_flow`
mide ahora los bloques **del flujo**, y `every_door_of_the_scope_has_a_place`
compara la puerta con el bloque **por el que entra el flujo**, en vez de con
el mínimo de todos los bloques.

`cargo test --workspace`: 86 suites en verde. `node --test tests/*.test.ts` en
`crates/genmsi-app`: 304 en verde.

### L9 — T3 (hecho): las cifras del contenedor salen de los cruces

El "pasadas" de un contenedor sale ahora de **una sola** regla: las entidades
que cruzaron su frontera, contadas desde los movimientos del trace. Se borra
`entryTargetOf`, que tomaba la **primera** puerta de entrada que tuviera
enlace: con varias puertas, las instancias que entraron por las demás no se
contaban, y el contenedor se quedaba en 0/0.

La ruta live del monitor (`MonitorView.tsx`) ahora pasa los cruces como la de
recepción: antes la recepción los pasaba y la live no, y por eso el mismo
contenedor mostraba dos números distintos según si el lector miraba la corrida
o la repetición.

De paso, los cruces son **eventos** y no un conteo por frame: se cuentan una
sola vez cuando se conoce la corrida (`crossingsEvents`) y después se cuentan
hasta la hora que alcanzó la repetición (`crossingsUntil`, con búsqueda
binaria). Antes se caminaban todos los movimientos en cada frame.

Test nuevo: `a container with several entry doors counts every entity that
entered it`. El test viejo que pineaba el comportamiento anterior (el total
del bloque de la primera puerta) se reescribió a la regla nueva.

`node --test tests/*.test.ts`: 305 en verde. `bunx tsc --noEmit`: limpio.
`cargo test --workspace`: 86 suites en verde.

### L10 — T2 (hecho): un camino de escritura para el lugar de las cosas

El modelo recibía el lugar de las cosas del ámbito en **dos** listas: una de
bloques y otra de puertas, con dos aplicadores y dos caminos de deshacer. Ahora
el lugar viaja en **una** lista de `Placement`, con la identidad que dice si la cosa
es un bloque o una puerta (`PlacedId`), y el aplicador escribe cada lugar donde
corresponde.

- `EditCommand::SetLayouts` lleva **una** lista: `placements`.
- `ScopeComposition::placements()` es la lista que la composición entrega.
- Un solo aplicador (`set_layouts`) y un solo paso de deshacer.

El archivo `.gmdl` no cambia: un `Placement` es una cosa del lienzo, no un
módulo. `PlacedId::Door` dice de qué contenedor y de qué puerta se trata, y el
aplicador lo escribe en el `layout` del puerto.

**Lo que NO se hizo (y por qué)**: la composición de **todo** el modelo (la
del CLI y la del MCP) no da lugar a las puertas. No es un descuido: la
composición de todo el modelo es **otra imagen** —la del modelo entero, con
los bloques de cada contenedor dentro de su recuadro— y las puertas viven en
la imagen de su ámbito. Ponerlas en la imagen del modelo entero las pondría
en un sistema de coordenadas que no es el suyo. El CLI compone el modelo
entero y las puertas quedan sin lugar: la app las coloca con su regla (que
es la misma que la del motor). Queda anotado como deuda.

**El corte de la entrega**: los commits de esta unidad son tres (T1, T3, T2) y
suman más de las ~400 líneas del corte de revisión. El corte queda acá:
después de T2, la unidad sigue con **T4** (el protocolo del hop) y **T5** (la
identidad de la puerta en la app), que van en su propio commit cada uno.

`cargo test --workspace`: 86 suites en verde. `node --test tests/*.test.ts`:
305 en verde. `bunx tsc --noEmit`: limpio.

El "pasadas" de un contenedor sale ahora de **una sola** regla: las entidades
que cruzaron su frontera, contadas desde los movimientos del trace. Se borra
`entryTargetOf`, que tomaba la **primera** puerta de entrada que tuviera
enlace: con varias puertas, las instancias que entraron por las demás no se
contaban, y el contenedor se quedaba en 0/0.

La ruta live del monitor (`MonitorView.tsx`) ahora pasa los cruces como la de
recepción: antes la reception los pasaba y la live no, y por eso el mismo
contenedor mostraba dos números distintos según si el lector miraba la
corrida o la repetición.

De paso, los cruces son **eventos** y no un conteo por frame: se cuentan una
sola vez cuando se conoce la corrida (`crossingsEvents`) y después se cuentan
hasta la hora que alcanzó la repetición (`crossingsUntil`, con búsqueda
binaria). Antes se caminaban todos los movimientos en cada frame.

Test nuevo: `a container with several entry doors counts every entity that
entered it`. El test viejo que pineaba el comportamiento anterior (el total
del bloque de la primera puerta) se reescribió a la regla nueva.

`node --test tests/*.test.ts`: 305 en verde. `bunx tsc --noEmit`: limpio.
`cargo test --workspace`: 86 suites en verde.

### L11 — T4 (hecho): la entidad cruza el contenedor por la puerta correcta

El movimiento del trace es de bloque a bloque (`fuera -> dentro`), y el par no
decía **por qué puerta** pasó la entidad: con dos puertas de entrada al mismo
bloque, el par era el mismo y la app elegía siempre el primer enlace
(`byPair`, "el primero gana"). Y el pasaje entero era **un** movimiento: el
círculo entraba a la caja del contenedor y nunca salía.

La solución no cambia el motor: **los enlaces son el flujo**. Cada enlace del
modelo une dos pasos (el puerto de un bloque donde el flujo está), y los
enlaces que un movimiento recorre son los que se encadenan. La app camina esos
enlaces —no resuelve las fronteras: los enlaces **son** las fronteras, ya
escritas por el editor— y camina con la regla del motor: de los enlaces que
arrancan en un paso, el primero es el que recorre el flujo.

- `chainsOf` indexa los enlaces por el paso del que arrancan y por el bloque
  del que salen.
- `chainFor` camina la cadena de un movimiento: del bloque del que salió la
  entidad al bloque al que llegó.
- `legsOf` dice de la cadena **qué enlaces son del ámbito que se está
  mirando**: la entidad los recorre uno después del otro.
- El monitor reparte el vuelo del movimiento entre los enlaces de la cadena:
  el círculo entra al contenedor y sale de él.

Con esto:

- la entidad que entró por la **segunda** puerta se ve entrar por la segunda
  puerta (antes se veía por la primera);
- una entidad que atraviesa un contenedor **sin bloques dentro** se ve entrar y
  salir, y no quedarse dentro de la caja;
- el movimiento que un ámbito no tiene sigue sin dibujarse (nada que dibujar).

**Lo que NO se arregla (queda anotado)**: una puerta de entrada **sin enlace
interno** (el motor entra igual por el primer hijo del contenedor) no tiene
cadena: el movimiento no es una cadena de enlaces y cae en el respaldo de
siempre, el enlace que lleva el par. Y el caso de un mismo bloque que alimenta
**dos** puertas del mismo contenedor sigue sin distinguirse: el trace dice de
bloque a bloque y no dice el puerto. Los dos necesitan que el trace diga el
puerto del que salió la entidad: es la unidad siguiente.

Tests nuevos en `route.test.ts`:

- `the move that crosses the border travels over the links of the scope`
- `each entity travels over the link of ITS door`
- `a move the scope has no link for travels over nothing`

`node --test tests/*.test.ts`: 308 en verde. `bunx tsc --noEmit`: limpio.
`cargo test --workspace`: 86 suites en verde.

### L12 — T5 (hecho): la identidad de la puerta es data, no una cadena

La puerta era un nodo con el id `"in-3"` / `"out-3"`, y **cinco** lugares de la
app lo parseaban con `startsWith` y con una expresión regular para saber qué
era: la selección, el arrastre, el borrado, la conexión y el filtro de los
enlaces. Ese es exactamente el "no comparten código con los bloques" que el
usuario intuyó.

Ahora la identidad de la puerta es **data**: `BlockData.door` dice de qué
contenedor es, qué puerto es y de qué lado está. Y el id del nodo la nombra
completa: `door:<contenedor>:<puerto>:in|out`. Antes el id no decía de qué
contenedor era la puerta: dos contenedores con una puerta del mismo puerto
tenían el mismo nodo.

De paso se cayó un bug latente: el handle del origen o del destino de un
enlace era a veces el **id del nodo** (`in-7`) y no el id del puerto (7). React
Flow resuelve un enlace por handle, y un handle que no existe hace que el
enlace **desaparezca sin decir nada**.

Los cinco lugares que parseaban la cadena ahora preguntan al nodo qué es.

**Lo que queda para la unidad siguiente**: la puerta todavía no es
seleccionable, ni aparece en el árbol, ni se copia con el portapapeles: el
corte son las tres cosas que tocan el tipo de la selección, que hoy es
`number[]`. Y la silueta del contenedor y la de las puertas siguen por un
`switch` (`shapeOfKind`) en vez de la tabla de las 66 clases: la diferencia
entre un bloque y una puerta es de datos (de dónde sale el footprint), no de
código, y la tabla puede guardarlo.

`node --test tests/*.test.ts`: 308 en verde. `bunx tsc --noEmit`: limpio.
`cargo test --workspace`: 86 suites en verde.

### L13 — T6 (hecho): el borrado es el mismo para los bloques y para las puertas

Había **dos caminos de borrado** con comportamientos distintos: la tecla
Delete de React Flow borra la puerta (`onNodesDelete` → `removePort`) y el
botón de la barra de herramientas borra los bloques (`deleteSelection` →
`removeBlock`). El lector que seleccionaba una puerta y apretaba el botón no
pasaba nada.

Ahora la app tiene **dos selecciones**: la de los bloques (`selectedBlockIds`)
y la de las puertas (`selectedDoors`), porque una puerta no es un módulo y no
puede entrar en una lista de ids de módulo. El botón de borrar borra las dos
cosas, y las puertas **primero**: una puerta es de su contenedor, y un
contenedor que se borra se lleva sus puertas.

El canvas dice qué está seleccionado en las dos listas, y el estado sólo
cambia cuando la selección cambia de verdad (el canvas la reporta en cada
render, y un array nuevo empezaría un bucle).

**Lo que queda anotado**: borrar un bloque y una puerta a la vez son **dos**
entradas de deshacer (una por comando). El motor no tiene un comando de lote
para las puertas, y agregarlo es otra unidad.

Test nuevo: `crates/genmsi-app/tests/selection.test.ts`
(`the button that deletes what is selected deletes a door too`,
`the selection of the doors is the selection of the reader`), con el store
cargado por el servidor de Vite, como lo carga `horizon.test.ts`.

`node --test tests/*.test.ts`: 310 en verde. `bunx tsc --noEmit`: limpio.

### L14 — T7 (hecho): el nombre de la puerta y la cuenta del contenedor

**El nombre** (`crates/genmsi-format/src/edit.rs`): el nombre de una puerta
numeraba las puertas **de su clase** con el id del puerto, que es único entre
todas las puertas del contenedor. Por eso la primera puerta de entrada se
llamaba `Entrada1` y la primera de salida `Salida2`.

Ahora el nombre se numera entre las puertas de su clase (el número es el
mayor que ya hay de esa clase, así una puerta borrada no le cede su número a
otra) y el **id** sigue siendo el que era: único entre todas las puertas del
contenedor, porque un enlace nombra un puerto por su id.

**La cuenta** (`crates/genmsi-app/src/monitor/counts.ts`): los cruces de
frontera se contaban por los **extremos** del movimiento (los bloques de
actividad de origen y de destino). En un contenedor cuyo flujo es sólo sus
puertas, el movimiento va de fuera a fuera: los extremos están fuera, y el
contenedor no contaba nada (0/0).

Ahora los cruces se cuentan por la **cadena** del movimiento —la misma que
usa la animación (L11)—: los enlaces de la cadena que terminan en una puerta
de **entrada** son las entradas a un contenedor. Los enlaces que **salen** de
un contenedor no son una entrada: una entidad que entra y sale pasó una vez.

Test nuevo: `the container whose flow is only its doors counts the entities
that entered it` (en RED antes del arreglo ✓). El test del motor que pineaba
el nombre viejo (`Salida2`) se actualizó a la regla nueva.

**Observación (no se cambió)**: el editor original escribe los nombres con un
espacio (`Entrada 1`, `Salida 1`) y la app sin (`Entrada1`). Los modelos que
vienen del editor original y los que nacen en la app mezclan las dos formas
en un mismo contenedor. Unificarlo es una decisión de forma, no de fondo: si
se unifica hacia el original, los modelos que la app ya escribió quedan
mezclados; si se unifica hacia la app, los de referencia quedan mezclados.

`cargo test --workspace`: 86 suites en verde. `node --test tests/*.test.ts`:
311 en verde. `bunx tsc --noEmit`: limpio.
