# submodulos-alcance — el contenedor es un bloque, y el alcance manda

Objetivo: que los contenedores y sus bordes de entrada y salida se comporten
como cualquier otro bloque (selección, cifras, animación) y que el
reordenamiento del lienzo rearrange **solamente lo que se está viendo**.
Rama: `feature/submodulos-alcance`. Entrega: `ask-on-risk`.
Pruebas: `cargo test --workspace`, `bun test` (o `npm test`) en
`crates/genmsi-app`, `bunx tsc --noEmit` (o `npm run typecheck`).

## Specs

### S1 — Las entidades se ven entrar y salir del contenedor

> "las entidades tanto como cuando van hacia un contenedor como cuando salen no
> se ven con la animación del circulito. Tambien pasa con el bloque de entrada
> o salida"

Una entidad que viaja de un bloque de fuera hacia un contenedor, y una que
sale de él, tienen que verse cruzar el borde. Lo mismo en el borde: el bloque
`Entrada` y el bloque `Salida` tienen que ver la entidad cruzándolos.

La causa: el motor **cosifica** la frontera del contenedor al compilar
(`crates/genmsi-core/src/program.rs`, `resolve_target` / `follow`), así que el
trace registra el salto entre los bloques de actividad, y la app empareja el
salto con el enlace dibujado por par de bloques
(`crates/genmsi-app/src/components/MonitorView.tsx`, mapa `routes`). En el
alcance del padre el enlace dibujado es `bloque → contenedor`; en el alcance
del contenedor es `Entrada → hijo`. Ninguno de los dos empareja con el par
`bloque → hijo` del trace, y el enlace "visible" que se fabricaba como
plan B queda colgando de un nodo que no existe en ese alcance: la entidad no se
ve.

La verdad de qué bloqueactivity cruza qué enlace la tiene el motor, y por eso
la publica (D1).

### S2 — El contenedor y sus bordes muestran las entidades que pasaron

> "tanto en el contenedor como en la entrada y la salida debe aparecer las
> entidades en ese bloque como las que pasaron en total por ahi"

El contenedor muestra cuántas entidades lo atravesaron; el bloque `Entrada`
cuántas entraron por ese puerto; el bloque `Salida` cuántas salieron por ese
puerto. Hoy el contenedor no muestra nada: los contenedores no se compilan, no
existen en el reporte, y `countsOf` indexa por bloque de actividad
(`crates/genmsi-app/src/monitor/counts.ts`). Y los bordes ni siquiera son
bloques del modelo: su id es `in-<ioid>` / `out-<ioid>`, y `Number("in-1")` es
`NaN`.

### S3 — El contenedor comparte el camino de dibujo de un bloque

> "cuando un bloque normal es seleccionado no se ve igual como cuando se
> selecciona el contenedor, es como que son cosas diferentes, no deberian
> compartir codigo? si realmnente deberian usar la misma logica para casi todo?"

Y la aclaración del usuario sobre qué es un contenedor:

> "no entiendo a que te referis con el marco, osea marco de que? si el
> contenedordeberia ser un bloque como lo es generate por ejemplo"

El contenedor es un bloque como `Generate`: tiene su propia caja, su propia
selección, su propio hover, su propio arrastre. Hoy tiene tres ramas de
dibujo distintas en `BlockNode` (`crates/genmsi-app/src/components/GraphView.tsx`):
la del bloque normal (silueta SVG del módulo `shapes`), la del contenedor (un
`div` con clases sueltas) y la de los bordes (otro `div`). De ahí que el
anillo de selección de un bloque sea un rectángulo de esquinas vivas y el del
contenedor uno redondeado: no es el mismo objeto dibujado, son dos.

### S4 — El reordenamiento rearrange sólo el alcance que se ve

> "el reordenamiento esta teniendo en cuenta a todos los bloques del diagrama
> completo, esto no debe ser asi, debe solo tener en cuenta los bloques que
> estan en la vista actual, ejemplo: [...] el reordenamiento debe ser basado en
> esos 5 bloques y sus conecciones, sin importar lo que este dentro del
> contenedor (no debe inportar nada de lo que este dentro ni reordenarse nada
> de lo que este dentro). Ejemplo 2: [...] esto es dentro del contenedor, al
> reordenar debe solo tener en cuenta esos 3 bloques, y nada de lo que este
> fuera del contenedor como dentro de contenedores que tenga este contenedor si
> los tuiviera"

Al reacomodar en la raíz se rearrangean los bloques de la raíz y sus enlaces;
el contenido del contenedor no se mira ni se mueve. Al reacomodar dentro de un
contenedor se rearrangean los bloques de ese contenedor; nada de afuera se
toca, y un contenedor anidado es una caja atómica: su contenido no se
reordena.

Hoy `relayout` compone el modelo entero de una vez
(`crates/genmsi-format/src/layout.rs`, `compose_with`), y el pedido no lleva
ningún alcance: `SessionRequest::Relayout` es una variante sin carga
(`crates/genmsi-app/engine/src/protocol.rs`).

### S5 — Las decisiones de la sesión

1. **La verdad del flujo vive en el motor** y se publica en el documento del
   modelo. La app no replica las reglas de resolución de puertos: el motor ya
   las tiene (`resolve_target` / `follow`) y la app no debe tener una segunda
   versión.
2. **El contenedor muestra `dentro/pasadas`; los bordes muestran el total que
   cruzaron.** El contenedor es el único bloque que puede tener dos cosas
   adentro a la vez (una entidad que sigue adentro, el total que pasó).

## Tasks

| T | Specs | Qué | Ruta | Estado | Commit |
|---|---|---|---|---|---|
| T0 | — | La navegación por contenedores (el árbol, las migas, el alcance del lienzo) y los bordes que el motor puede editar | inline | **hecho** | `5eeeb6d`, `563db27` |
| T1 | S1, S2 | La verdad del flujo: cada enlace declara los pares de bloques de actividad que lo cruzan, y el documento del modelo lo publica | `gentle-ai-worker` | **hecho** | `39d029d` |
| T2 | S4 | El motor compone un alcance: `relayout_scope(model, scope)` con los contenedores anidados como cajas atómicas | inline | **aplazado** (otra ronda) | — |
| T3 | S4 | El pedido lleva el alcance: protocolo, sesión, facade y el botón de la app | inline | **aplazado** (otra ronda) | — |
| T4 | S1, S2 | La app enruta la entidad por el enlace del alcance y cuenta lo que cruza cada frontera | `gentle-ai-worker` | **hecho** (el enrutado; las cifras quedan para otra ronda) | `8b92324` |
| T5 | S3 | Un solo camino de dibujo: contenedor y bordes con la misma silueta, selección, hover y cifra | `gentle-ai-worker` | **hecho** | `0f56073` |

Esta ronda toma **T1 + T4 + T5** (la unidad 1 y la unidad 2 del plan de la
sesión de investigación): la silueta única con su selección única, y la
animación del círculo de vuelta. **T2 + T3 quedan para otra ronda**: tocan
el algoritmo de composición del motor y piden su propia medición (L6).

### T1 — El contrato (lo que T1 publica)

El documento del modelo (`genmsi.model/1`, `crates/genmsi-core/src/documents.rs`)
publica, **por enlace**, los pares de bloques de actividad que ese enlace
cruza:

```
ModelEdge.hops: Vec<(u32, u32)>   // (bloque de actividad de entrada, de salida)
```

Un enlace puede cruzar varios pares (la salida de un contenedor la alimentan
varios bloques de su interior; un puerto de entrada puede recibir de varios).
Cada par es `(bloque_actividad_origen, bloque_actividad_destino)`, con los
bloques resueltos **a través** de las fronteras: el par de un enlace que entra
a un contenedor es `(bloque de fuera, bloque de adentro)`.

Con ese dato la app resuelve las dos cosas que faltan, sin replicar ninguna
regla del motor:

- **La entidad que cruza** (S1): una transición del trace entre dos bloques de
  actividad se dibuja sobre el enlace de su alcance que cruza ese par. Un
  enlace pertenece a un alcance (el contenedor que lo posee), así que en cada
  alcance hay exactamente un enlace por par.
- **Las cifras de la frontera** (S2): el número de transiciones que cruzaron
  un enlace es la suma de los conteos de los pares que ese enlace cruza. El
  total de un contenedor es el de sus enlaces de entrada; el de un puerto
  `Entrada`/`Salida`, el del enlace que ese puerto abre o cierra.

La resolución es la que el motor ya hace al compilar
(`crates/genmsi-core/src/program.rs`, `resolve_target` / `follow`): T1 la
revela, no la reescribe.

### T2 — Las coordenadas y el alcance

Cada alcance tiene **su propio espacio de coordenadas** (L4). Un contenedor
ocupa una caja en el espacio de su padre (la de su bloque, como `Generate`) y
sus hijos viven en el espacio del contenedor, que es otro lienzo.

`relayout_scope(model, scope)` compone **sólo** los bloques del alcance:

- los enlaces que se consideran son los que el alcance posee, con las
  fronteras de los contenedores anidados **cosidas** (un contenedor anidado es
  una caja atómica: su contenido no se mira ni se mueve);
- las posiciones que se escriben son las del alcance, y ningún bloque de
  afuera se toca;
- el resultado es determinista y el mismo para el mismo modelo.

El alcance de la raíz (`scope = 0`) es el caso particular de siempre.

La composición de modelo entero (`relayout(model)`, la que piden la CLI y el
MCP) **no cambia**: sigue componiendo el árbol completo y su línea base
commiteada (`crates/genmsi-format/tests/layout-baseline.json`) sigue siendo
la de la medida.

### T5 — El contrato (lo que T5 publica)

El dibujo de los 69 bloques del lienzo pasa por **un solo camino**: la
silueta SVG con sus zonas, los puertos en los anclajes que la forma declara,
y el anillo de selección **siempre en el wrapper externo**.

- `src/graph/frames.ts` (nuevo): las formas **estructurales** —la del
  contenedor y la de las puertas—, que no son clases del catálogo y por eso
  no viven en `src/graph/shapes.ts` (que es generado por
  `odd/tasks/forma-bloques-diseno/generar_shapes.py` y no se toca a mano).
  - El **contenedor** es un bloque más: su caja es la que el modelo le da
    (L2, L3), no un marco que envuelve a sus hijos. Se dibuja como el bloque
    redondeado que dibuja el resto, con el rótulo del contenedor (su
    caption) y la palabra "Contenedor" debajo.
  - Las **puertas** (`Entrada`/`Salida`) tienen su silueta propia, con el
    nombre del puerto dentro.
  - Los colores son los que ya usan hoy, que son los de una familia del
    catálogo: el contenedor, el de `Decisión`; la puerta de entrada, el de
    `Fuente`; la de salida, el de `Destino`.
- `src/graph/node.ts`: `drawingOf` y `nodeSizeOf` dejan de preguntar por la
  **clase** y preguntan por el **tipo de bloque** (`"A"`, `"C"`,
  `"PortIn"`, `"PortOut"`) y por el tamaño que el modelo le da. La clase
  sigue mandando en los 66 bloques; el tipo manda en los otros tres.
- `src/components/GraphView.tsx`: `BlockNode` tiene **una** rama: la
  silueta, el texto, los puertos y el anillo. Las tres ramas que había
  (`:107` las puertas, `:146` el contenedor, la de los 66) se van.
- El **anillo** va en el wrapper externo de los 69: el contenedor y las
  puertas se seleccionan como cualquier otro bloque (S3).

## Log

### L1 — El pedido del usuario (verbatim)

> Bien ya lo instale y probe, cosas a arreglar:
>  - las entidades tanto como cuando van haciua un contenedor como cuando salen
>    no se ven con la animacion del circulito. Tambien pasa con el bloque de
>    entrada o salida
>  - tanto en el contenedor como en la entrada y la salida debe aparecer las
>    entidades en ese bloque como las que pasaron en total por ahi.
>  - ademas todavia siugo pensando de porque cuando un bloque normal es
>    seleccionado no se ve igual como cuando se selecciona el contenedor, es
>    como que son cosas diferentes, no deberian compartir codigo? si realmnente
>    deberian usar la misma logica para casi todo?
>  - el reordenamiento esta teniendo en cuenta a todos los bloques del
>    diagrama completo, esto no debe ser asi, debe solo tener en cuenta los
>    bloques que estan en la vista actual, ejemplo: en este caso [imagen de la
>    raíz con Generate, Clock, Advance, Modulo 4 y Terminate] el reordenamiento
>    debe ser basado en esos 5 bloques y sus conecciones, sin importar lo que
>    este dentro del contenedor (no debe inportar nada de lo que este dentro ni
>    reordenarse nada de lo que este dentro). Ejemplo 2: [imagen del interior
>    de Modulo 4 con Entrada1, Advance y Salida1] esto es dentro del
>    contenedor, al reordenar debe solo tener en cuenta esos 3 bloques, y nada
>    de lo que este fuera del contenedor como dentro de contenedores que tenga
>    este contenedor si los tuiviera
>
> analicemos todo esto y pensemos bien por favoraaaaaa, antes de implementar
> detallame que problema encontraste y como se solucionaria

### L2 — La aclaración sobre qué es un contenedor

> "no entiendo a que te referis con el marco, osea marco de que? si el
> contenedordeberia ser un bloque como lo es generate por ejemplo"

El contenedor es un bloque: tiene su caja, y en la composición de su alcance
es una caja como cualquier otra. Los bloques que contiene se reacomodan
cuando se reacomoda **su** alcance, y no cuando se reacomoda el del padre.

### L3 — Lo que se midió antes de escribir esto

El dibujo de los contenedores en los modelos de referencia confirma la
lectura del usuario: el contenedor **no** es un marco que envuelve a sus
hijos. En `fixtures/models/TP8__mecanicos.gmdl` el contenedor 18 tiene su caja
en `(327, 215, 153×39)` y sus cuatro hijos en `(330, 132)`, `(494, 220)`,
`(191, 220)` y `(344, 319)`: **ninguno** de los cuatro cae dentro de la caja
del padre. Lo mismo en los otros 24 modelos de referencia (el conteo de
"hijos dentro de la caja del contenedor" da 0 o 1 en todos los contenedores
salvo dos). La caja del contenedor es su propio bloque en el lienzo, como
`Generate`.

Los bordes de un contenedor tampoco son bloques del modelo: en `TP8` el
contenedor 18 declara `inputs: [1]` y `outputs: [2]`, y son esos puertos los
que los enlaces tocan (`18:1 → 19:1` adentro, `82:2 → 18:1` afuera).

### L4 — Cada alcance es su propio lienzo

Medido en los 25 modelos de referencia: los hijos de un contenedor **no**
comparten espacio con los de la raíz. En `Parcial 6-11-23` hay 68 pares de
bloques que se pisan si se leen todos en el mismo espacio, y 1 si cada
contenedor se lee en el suyo. La lectura correcta es la segunda: **cada
contenedor es un lienzo propio**, y su caja en el lienzo del padre es la de
un bloque más (153×39 en `TP8`, 130×65 en el Parcial), no la de un marco.

Esto es lo que hace que el pedido del usuario sea coherente: el contenedor es
un bloque como `Generate` —caja propia, selección propia, arrastre propio— y
sus hijos se reacomodan cuando se reacomoda **su** alcance, no el del padre.

Y por eso el motor tiene dos composiciones y las dos son del motor (D18):
`relayout(model)` compone el árbol entero (la que piden la CLI y el MCP, con
su línea base commiteada) y `relayout_scope(model, scope)` compone un
alcance. La app usa la segunda, porque el lector ve un alcance.

### L5 — El rótulo "T" de los enlaces

Lo que el usuario vio ("ahora solo pone 'T' en vez de 'A_Advance'") viene de
que los puertos de un contenedor nacían con la etiqueta fija `"T"`
(`crates/genmsi-format/src/edit.rs`), y el motor de enlaces reutiliza la
etiqueta que ya está: el "T" se propagaba a todos los enlaces nuevos. Con los
puertos naciendo vacíos (L5 de la fase anterior) el motor acuña la etiqueta
canónica `A_<Clase>` sola, y el enlace que entra a un contenedor lleva la
del contenedor (`A_Modulo4`).

### L6 — La sesión de investigación (lo que se midió con la app corriendo)

La pregunta del usuario ("vamos con la unidad 1 + 2") se responde después de
investigar el problema **corriendo la app real** (dev server + navegador,
`agent-browser`) sobre `fixtures/models/TP3__aeropuerto.gmdl` y
`fixtures/models/TP10__tp10.gmdl`, midiendo el DOM, las bolas publicadas y
el efecto de cada comando. Lo medido:

- **Las bolas que cruzan una frontera no existen.** En el alcance raíz del
  aeropuerto se publican 10 bolas, **todas** con id `visible:*`, y en el DOM
  hay **0 círculos**; las aristas reales del contenedor (`0:68:1->1:1`,
  `0:51:1->3:1`, `0:74:1->2:1`…) están dibujadas pero **sin bola**. En 20 s
  de polling: ids `visible:*` y `0:*`, con máximo 2 círculos. **1293 de 3583
  movimientos (36%)** del aeropuerto pierden su bola. La causa es la que
  anticipa S1: el trace registra el salto entre los bloques de actividad
  (`73→1`) y ningún enlace del alcance empareja con ese par.
- **El contenedor se dibuja con un tamaño inventado.** `src/graph/graph.ts`
  lo fuerza a `160×56` y tira el tamaño que el modelo le da. Medido: tras un
  *Reacomodar*, el contenedor 1 de `TP10` tiene **694×596** en el modelo y
  la app lo dibuja **160×56**.
- **El reordenamiento mueve el modelo entero.** Entrando al contenedor 1 y
  apretando *Reacomodar*, cambiaron **86 de 86** bloques: los que están
  afuera, incluidos. La composición es un único grafo dagre global
  (`crates/genmsi-format/src/layout.rs`, `compose_with`) con las fronteras
  cosidas (`layout/flow.rs`, `flow_edges_for`): los hijos de un contenedor se
  rankean por el flujo **global**, así que su orden interno depende de lo
  que hay afuera.
- **Dentro de un contenedor pasan cosas raras.** Al agregar un bloque, cae
  **fuera de la pantalla** (canvas de 742 px, bloque nuevo en `y=899`): el
  hueco libre se calcula abajo del hijo más bajo, en coordenadas absolutas,
  y la vista no se reencuadra. Las puertas están en posiciones fijas
  (`120 + i·180, 30` y `120 + i·180, 420`): la `Entrada2` apareció en la
  esquina superior izquierda y la `Salida1` en el medio del diagrama, y se
  mueven cada vez que se agrega otra puerta.

### L7 — La decisión de esta sesión

> "vamos con la unidad 1 + 2"

Se toman **T1 + T4 + T5**: la verdad del flujo en el motor (T1), el
enrutado de la entidad por el enlace del alcance (T4) y el camino de dibujo
único (T5). **T2 + T3 quedan para otra ronda** (L6): tocan el algoritmo de
composición, que es del motor y tiene su línea base commiteada
(`crates/genmsi-format/tests/layout-baseline.json`).

Una corrección al plan de la sesión de investigación: la ruta visible **no**
se resuelve en la app con una regla estructural propia (el representante del
bloque en el alcance). La verdad del flujo vive en el motor (D1, S5) y se
publica; la app la proyecta. La razón es concreta: un puerto de entrada sin
enlace interno (que el editor permite, p.ej. `EleccionProductos` de TP4) no
tiene arista en el alcance, y la regla de la app perdería la bola ahí.

### L8 — El borde de la unidad

Riesgo: **item 3** (cambian las firmas de `drawingOf`/`nodeSizeOf`, que son
helpers compartidos por el nodo y sus pruebas) y **item 6** parcial (el
círculo se ve en la pantalla: la prueba es visual). Mitigación: la prueba
diferencial de los 66 bloques (se dibujan igual que antes) y la verificación
visual en el navegador sobre los dos modelos de referencia.

### L9 — Lo que se entregó y lo que se midió

- **T5** (`0f56073`): un solo camino de dibujo para los 69 bloques. El
  contenedor y las puertas tienen su silueta (`src/graph/frames.ts`), el
  anillo de la selección está en el wrapper externo de los 69, y el
  contenedor se dibuja al tamaño que el modelo le da (medido: 694×596 en el
  modelo, 160×56 antes).
- **T1** (`39d029d`): la resolución de las fronteras tiene **una**
  implementación, compartida con el compilador
  (`genmsi_core::program::borders`), y el documento publica por enlace los
  pares de bloques de actividad que cruza (`ModelEdge.hops`). El programa,
  la traza y el reporte de los 24 modelos de referencia salen byte a byte
  iguales. Costo: 52 µs en release para el modelo más grande del corpus (el
  documento entero, 286 µs; compilarlo, 342 µs).
- **T4** (`8b92324`): las rutas de un ámbito salen de los pares publicados; el
  círculo cruza el borde del contenedor en la raíz y el puerto dentro del
  contenedor.

Medido en el navegador, con la app corriendo:

- **El círculo cruza la frontera.** En la raíz del aeropuerto las entidades
  se publican sobre `0:68:1->1:1` (la salida del contenedor 68) y sobre
  `0:74:1->2:1`, y el DOM tiene círculos (máximo 3 simultáneos). Antes: 10
  bolas publicadas **todas** con id `visible:*` y **0** círculos en el DOM.
- **Dentro del contenedor**, el círculo cruza el puerto: sobre
  `68:73:2->68:1` (el bloque 73 hacia la `Salida1` del contenedor 68).
- **Los 66 bloques no cambiaron**: el diferencial del DOM (silueta, texto,
  puertos, caja y anillo de cada nodo, antes y después) da 66/66 nodos
  idénticos.
- El contenedor y las puertas se seleccionan igual que cualquier otro
  bloque: el anillo es el mismo objeto.

### L10 — La revisión nativa del tramo de T4

Con el RDD activo, `assess` sobre el tramo `d85c4e1..HEAD` dio riesgo medio y
`reviewDue: false` (bajo presupuesto). La revisión nativa se pidió igual, a
pedido del proveedor, y **el candidato completo no entra en el presupuesto de
contexto**: `lens_context_budget_exceeded`, sin autoridad creada. Como los
commits ya están partidos por unidad, la revisión se encadenó por tramo: el de
T4 (`39d029d..HEAD`, 5 archivos, 448 líneas) sí entró y creó la autoridad
`review-07593c1d666d238e` (riesgo medio, lentes `review-readability` y
`review-reliability`, presupuesto de correcciones 200).

**La captura de los resultados quedó bloqueada**: el `STATUS` devuelve los dos
bindings de captura textualmente y `gentle_review_capture` (y el de grupo) los
rechazan con `capture-binding-rejected` ("collectBinding is missing or stale
for current STATUS"). Se probaron **ocho** formas: slot por slot y grupo
completo, el binding como objeto y como cadena, con y sin
`reviewerRunAcknowledged`. Un `STATUS` fresco, tomado después del último
commit, devuelve los mismos bindings **byte a byte** (mismo target, mismo
árbol candidato): no es que estén rancios, es que la validación del
`capture` no admite el binding del propio proveedor.

Los veredictos no se escriben a mano: los admite el motor, y acá no hay
veredicto. La autoridad queda **viva** (estado `reviewing`, generación 1), sin
quemar: el mantenimiento la puede terminar desde el CLI, que es donde vive la
validación del binding:

    gentle-ai review status --cwd <repo> --contract gentle-ai.review-integration/v2 --next-transition
    gentle-ai review capture-result --help

La rama completa se reintentó después del último commit y volvió a dar
`lens_context_budget_exceeded` (terminal para ese candidato): la revisión
nativa de esta rama **tiene que ir encadenada por unidad** (T4 → T5 → T1),
como está previsto en el diseño de commits de trabajo.

**El bloqueo es del camino de captura, no del candidato.** Se probó con una
autoridad **nueva** sobre el tramo de T5 (`review-462d8dcc01c99a15`, 7
archivos, 1.395 líneas, otra lineage, otro target, otros hashes de sujeto):
el `STATUS` devuelve sus bindings y la captura los rechaza igual. Con eso
queda descartado que sea un binding rancio o una autoridad trabada: la
validación de `capture` no admite ningún binding del proveedor.

Quedan dos autoridades **vivas** (estado `reviewing`, sin quemar), que el
mantenimiento puede terminar desde el CLI:

| autoridad | tramo | líneas | lentes |
|---|---|---|---|
| `review-07593c1d666d238e` | T4 (`39d029d..HEAD`) | 448 | readability, reliability |
| `review-462d8dcc01c99a15` | T5 (`d85c4e1..0f56073`) | 1.395 | readability, reliability |

Y falta encadenar el tramo de T1 (el motor, `39d029d`), que entra por
debajo del presupuesto igual que los otros dos.

**La cadena quedó completa.** Con una tercera autoridad (T1) el mismo
`capture` la rechaza igual, con el mismo motivo: el bloqueo no es del
candidato, ni de los bindings, ni de una autoridad trabada. Las tres
autoridades de la ronda quedan **vivas** (estado `reviewing`, sin quemar), y
cada una cubre una unidad de trabajo:

| autoridad | unidad | tramo | líneas | lentes |
|---|---|---|---|---|
| `review-0360430e8b8b91f4` | T1 — el motor | `0f56073..39d029d` | 1.228 | risk, resilience, readability |
| `review-462d8dcc01c99a15` | T5 — el dibujo | `d85c4e1..0f56073` | 1.395 | readability, reliability |
| `review-07593c1d666d238e` | T4 — el enrutado | `39d029d..HEAD` | 448 | readability, reliability |

Lo que sí está probado, sin la revisión nativa: `cargo test --workspace`
(578) y el suite de la app (279) en verde, `tsc` limpio, el diferencial del
DOM de los 66 bloques idéntico, y la medición en el navegador del círculo
cruzando la frontera.

### L11 — Lo que queda de esta unidad

- **S2** (las cifras del contenedor y de sus puertas) queda sin hacer: el
  dato ya está publicado (los pares de cada enlace) pero contarlo y
  publicarlo es otra unidad, con `src/monitor/counts.ts` y un lugar donde
  escribirlo.
- **T2 + T3** (la recomposición por alcance) quedan para otra ronda: tocan
  el algoritmo de composición del motor y piden su propia medición.
- La posición de las puertas (hoy fijas en el código) y el marco del ámbito
  siguen siendo de otra unidad.
