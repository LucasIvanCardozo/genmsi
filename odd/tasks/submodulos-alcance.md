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
| T1 | S1, S2 | La verdad del flujo: cada enlace declara los pares de bloques de actividad que lo cruzan, y el documento del modelo lo publica | inline | — | — |
| T2 | S4 | El motor compone un alcance: `relayout_scope(model, scope)` con los contenedores anidados como cajas atómicas | inline | — | — |
| T3 | S4 | El pedido lleva el alcance: protocolo, sesión, facade y el botón de la app | inline | — | — |
| T4 | S1, S2 | La app enruta la entidad por el enlace del alcance y cuenta lo que cruza cada frontera | inline | — | — |
| T5 | S3 | Un solo camino de dibujo: contenedor y bordes con la misma silueta, selección, hover y cifra | inline | — | — |

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
