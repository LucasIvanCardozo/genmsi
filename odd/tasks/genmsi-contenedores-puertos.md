# Contenedores y puertos — una sola silueta, una sola ruta

El bloque `Container`, el `PortIn` y el `PortOut` reimplementan, por su cuenta,
la lógica de dibujo que los otros 66 bloques ya tienen: otra silueta, otro
anillo de selección, otro tamaño de marco y otro camino para las bolas que
cruzan su frontera. Esta sesión cierra las dos unidades que más duele:
**una sola forma de dibujar** y **la animación del círculo de vuelta**.

Rama: `feature/genmsi-contenedores-puertos`. Estrategia de entrega:
`ask-on-risk` (RDD está activo: la revisión nativa corre sobre un commit de
unidad de trabajo). Pruebas: `cd crates/genmsi-app && node --test tests/*.test.ts`
y `bunx tsc --noEmit`. Revisión visual: la app en el navegador sobre
`fixtures/models/TP3__aeropuerto.gmdl` y `fixtures/models/TP10__tp10.gmdl`.

## Specs

### S1 — Los tres bloques reimplementan la lógica de los otros 66

> "Tanto el bloque Container, PortIn y PortOut sospecho que no reimplementa la
> logica que si tienen los otros 66 bloques. Me refiero a por ejemplo: todas
> las instancias que entran o salen a estos bloques no se ven con la
> animacion del circulito en la relacion conectada que tengan con otro
> bloque. Ademas otro indicio de esto es que cuando le doy un click, a
> caulquiera de esas tres, se ve diferente a darle click a cualquiera de los
> otros 66 bloques (tanto el Container, portin y portout se le pone mas
> grueso el borde, y a los otros 6 bloques se le hace un cuadrado alrrededor
> del bloque), esto es indicio claro de que no reimplementa el codigo que ya
> usan los bloques, esto es mala practica y reusabilidad."

Los tres tienen su propia rama de dibujo en `src/components/GraphView.tsx`
(`:107` los puertos, `:146` el contenedor, `:~250` los 66 bloques). No
comparten ni la silueta, ni el texto, ni los puertos, ni la selección.

### S2 — El click se ve distinto en los tres

> "(tanto el Container, portin y portout se le pone mas grueso el borde, y a
> los otros 6 bloques se le hace un cuadrado alrrededor del bloque)"

Los 66 bloques ponen el anillo `ring-2 ring-indigo-500` en el **wrapper
externo** (un cuadrado alrededor de la silueta). Los tres lo ponen en el div
**interior que ya tiene `border` y `rounded-md`** (`:116`, `:151`): el anillo
se suma al borde y el borde se ve más grueso. Son dos sistemas de selección.

### S3 — La animación del círculo falta donde entra o sale un contenedor

> "todas las instancias que entran o salen a estos bloques no se ven con la
> animacion del circulito en la relacion conectada que tengan con otro bloque"

La simulación es **plana**: el motor compila los contenedores "inlined"
(`crates/genmsi-core/src/program.rs`, `resolve_target`/`follow`), así que la
traza registra el movimiento entre bloques de actividad internos
(p.ej. `73→1`), nunca a través del contenedor. La app mapea el movimiento a
un enlace del ámbito actual (`src/components/MonitorView.tsx:241-242`): la
búsqueda `${edge.source}->${edge.target}` falla para todo movimiento que
cruza una frontera → cae a `visibleLinkId()` → un enlace transparente entre
dos ids que **no son nodos del ámbito** → React Flow lo descarta en silencio
→ **no hay círculo**.

Medido en `fixtures/models/TP3__aeropuerto.gmdl` (ámbito raíz): se publican
10 bolas, **todas** con id `visible:*`, y en el DOM hay **0 círculos**; las
aristas reales del contenedor (`0:68:1->1:1`, `0:51:1->3:1`, `0:74:1->2:1`…)
están dibujadas pero **sin bola**. **1293 de 3583 movimientos (36%)** del
aeropuerto pierden su bola.

### S4 — Una sola silueta, un solo camino de dibujo, un solo anillo

Los 69 bloques del lienzo (66 clases + contenedor + entrada + salida) se
dibujan por el **mismo camino**: la silueta SVG con sus zonas de texto, los
puertos en los anclajes que la forma declara, y el anillo de selección
**siempre en el wrapper externo**. El módulo de formas de clase
(`src/graph/shapes.ts`) es **generado** (`odd/tasks/forma-bloques-diseno/`)
y no se toca: las formas del armazón (el marco del contenedor y las puertas)
viven en un módulo nuevo, `src/graph/frames.ts`, que es donde la forma
estructural se corresponde con la familia de color que ya usan hoy:

| bloque | familia | color de acento | por qué |
|---|---|---|---|
| contenedor | `Decisión` | `#4338ca` | es el marco del modelo, no un paso del flujo |
| puerta de entrada | `Fuente` | `#047857` | de acá sale el flujo hacia adentro |
| puerta de salida | `Destino` | `#be123c` | de acá sale el flujo hacia afuera |

El **contenedor se dibuja al tamaño que el modelo le da** (el motor lo
compone como marco real alrededor de sus hijos:
`crates/genmsi-format/src/layout.rs`, `place_frames`), no a 160×56 fijos:
medido, tras un *Reacomodar* el contenedor 1 de `TP10__tp10.gmdl` tiene marco
**694×596** en el modelo y la app lo dibujaba **160×56**.

### S5 — La ruta visible: todo movimiento tiene una arista que lo lleva

Para cada movimiento `(from → to)` y cada ámbito hay **exactamente una**
arista del modelo que lo lleva y que el ámbito dibuja. La arista se
encuentra por una regla **estructural** (no semántica): el representante de
un bloque en el ámbito es su ancestro más cercano que es hijo directo del
ámbito; y la arista es el enlace que el ámbito posee entre esos
representantes (o el que lleva el flujo hacia/desde el puerto del ámbito).

| caso | representante de `from` | representante de `to` | arista visible |
|---|---|---|---|
| los dos en el ámbito | hijo directo | hijo directo | el enlace que el ámbito posee entre ambos |
| el movimiento **entra** al ámbito | (ninguno) | hijo directo | el enlace que el ámbito posee desde su puerto de entrada |
| el movimiento **sale** del ámbito | hijo directo | (ninguno) | el enlace que el ámbito posee hacia su puerto de salida |
| los dos más adentro | el mismo | el mismo | ninguna: el movimiento no se ve en este ámbito |

Con la regla, el círculo vuelve sin tocar el motor: la arista que lo lleva
**ya está dibujada** en el ámbito (es la del contenedor, en la raíz; es la de
la puerta, dentro del contenedor).

### S6 — Lo que no se rompe

- El motor no cambia: la semántica de la simulación, la traza y el formato
  `.gmdl` quedan como están. La app **proyecta** la ruta; no la decide.
- Los 66 bloques se dibujan **exactamente** como antes (misma silueta, mismo
  texto, mismos puertos, mismo anillo): el cambio es additive para ellos.
- La geometría no entra en la simulación.
- El modelo en memoria del usuario no se toca: nada se escribe en el disco
  sin que el usuario lo pida.

## Tasks

| ID | Specs | Route | Entrega |
|---|---|---|---|
| T1 | S4 | `gentle-ai-worker` (paralelo) | `src/graph/frames.ts`: la forma del marco y la de las puertas, con sus zonas de texto y sus anclajes de puerto |
| T2 | S1, S2, S4 | `gentle-ai-worker` (paralelo) | `src/graph/node.ts` + `src/components/GraphView.tsx`: un solo camino de dibujo para los 69, el anillo en el wrapper externo, el contenedor al tamaño de su marco |
| T3 | S3, S5 | `gentle-ai-worker` (paralelo) | `src/graph/route.ts` + `src/components/MonitorView.tsx`: la ruta visible por ámbito y el círculo de vuelta |
| T4 | S1–S6 | inline (orquestador) | verificación visual en el navegador + diferencial de los 66 bloques + cierre |

Las unidades T1+T2 (el dibujo) y T3 (la ruta) son **independientes**: no
comparten archivos y se lanzan a la vez.

## Log

### L1 — El pedido original

> "Hola! hoy quiero que encaremos varias cosas que quedarios mal implementadas
> realmente en la aplicacion:
> - Tanto el bloque Container, PortIn y PortOut sospecho que no reimplementa la
> logica que si tienen los otros 66 bloques. Me refiero a por ejemplo: todas las
> instancias que entran o salen a estos bloques no se ven con la animacion del
> circulito en la relacion conectada que tengan con otro bloque. Ademas otro
> indicio de esto es que cuando le doy un click, a caulquiera de esas tres, se
> ve diferente a darle click a cualquiera de los otros 66 bloques (tanto el
> Container, portin y portout se le pone mas grueso el borde, y a los otros 6
> bloques se le hace un cuadrado alrrededor del bloque), esto es indicio claro
> de que no reimplementa el codigo que ya usan los bloques, esto es mala
> practica y reusabilidad.
> - Al reordenar el grafico tiene en cuenta todos los bloques, cuando
> realmente deberia tener en cuenta todos los bloques que son hermanos entre
> si en el arbol. Ejemplo: si entro a un contenedor y le doy a reordenar, se
> reordena tambien todos los otros bloques que estan fuera de el, ademas de
> que al tenerlos en cuenta tambien se reordenan mal los mismos que estan
> dentrod el Container.
> - Cuando entro a un contenedor ocurren cosas raras al introducir nuevos
> bloques, al moverlos o al establecer conexiones entre ellos, es como que se
> reorganizan o mueven cuando no deberian hacerlo"

### L2 — La sesión de investigación

Se investigó el problema leyendo el código y **corriendo la app real** (dev
server + navegador, `agent-browser`) sobre `TP3__aeropuerto.gmdl` y
`TP10__tp10.gmdl`, midiendo el DOM, las bolas publicadas y el efecto de cada
comando. Hallazgos medidos:

- **P1** las tres ramas de dibujo, el anillo doble, el `160×56` fijo del
  contenedor, los puertos inventados en `(120 + i*180, 30)` y
  `(120 + i*180, 420)`.
- **P1.6** 1293 de 3583 movimientos (36%) del aeropuerto pierden su bola; en
  el DOM hay 0 círculos donde deberían.
- **P2** el *Reacomodar* dentro de un contenedor cambió **86 de 86** bloques;
  la composición es un único grafo dagre global (`layout.rs:183
  compose_with`) con las fronteras cosidas (`layout/flow.rs:24`).
- **P3** al agregar un bloque dentro de un contenedor cae **fuera de la
  pantalla** (canvas de 742 px, bloque nuevo en `y=899`); el marco del
  contenedor no se recalcula al cambiar su contenido; `containerAt()` se
  borró del store y ya no se puede soltar un bloque dentro de un contenedor.

### L3 — La decisión de esta sesión

> "vamos con la unidad 1 + 2"

Se toman las unidades 1 y 2 del plan propuesto (la silueta única con su
selección única, y la ruta visible que devuelve el círculo). Las unidades 3
(recomposición por hermanos) y 4 (el marco del ámbito y las puertas en el
borde) quedan para después: la 3 toca el algoritmo del motor y pide su
propia medición; la 4 es la que hace que el marco del contenedor sea la
puerta de entrada al ámbito.

### L4 — Nota de riesgo

Riesgo: **item 3** (se cambia la firma de `drawingOf`/`nodeSizeOf`, que son
helpers compartidos por el nodo y sus pruebas) y **item 6** parcial (el
círculo se ve en la pantalla: la prueba es visual). Mitigación: la prueba
diferencial de T4 (los 66 bloques se dibujan igual) y la verificación visual
en el navegador.
