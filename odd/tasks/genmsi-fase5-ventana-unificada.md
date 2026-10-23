# Fase 5 — la ventana única: grafo, reproducción, gráficos y reporte

Una sola ventana donde el grafo del modelo y la reproducción de la corrida son la
misma picture. El grafo se dibuja una vez, con la capa viva encima, y todo lo que
el motor publica se ve en vivo.

## Specs

### S1 — Una sola ventana

> "Que te parece fusionar la ventana monitor con la de grafo? uniendo la mejor parte de ambas, ya que tranquilamente podrian complementarse, manteniendo la logica bien ordenada, no te parece?"

La app tiene **una** pestaña de trabajo. La pestaña "Grafo" y la pestaña "Monitor"
dejan de existir: el grafo es el lienzo de la reproducción, no un dibujo aparte.
El modelo se sigue editando en ese mismo lienzo.

### S2 — Los sub-paneles

> "que tal si combinamos todo y los reportes los colocamos como ventana como esta dentrod e monitor la ventana de reproduccion y graficos?"

La ventana única tiene tres sub-paneles, con estos nombres exactos:

    Simulación → Reproducción │ Gráficos │ Reporte

- **Reproducción**: el grafo con la capa viva (bloques con actividad, números
  "ahora / total" y las bolitas que viajan entre bloques).
- **Gráficos**: las series temporales y los histogramas que ya existen.
- **Reporte**: el documento `genmsi.report/1` del motor.

### S3 — Los nombres

> "Simulación → Reproducción | Gráficos | Reporte"

La pestaña se llama **Simulación**. Los sub-paneles se llaman **Reproducción**,
**Gráficos** y **Reporte**.

### S4 — El reporte en vivo

> "me gustaria que todo este en vivo, esto le daria mucha mas vista a la aplicacion, lo ves viable?"

El sub-panel Reporte muestra los números **del instante de la reproducción**, no
sólo los de la corrida terminada. Esto alcanza a:

- **Bloques**: entradas, contenido actual y reintentos.
- **Colas**: contenido, entradas, máximo, entradas con espera cero, tiempo total y área.
- **Facilidades**: quién la tiene, si está ocupada, entradas, reintentos e interrupciones.
- **Almacenes**: unidades en uso, mínimo, máximo, entradas y área.
- **Cadenas**: contenido, máximo, entradas, tiempo total y área.

Y **no** alcanza a las pestañas **Tablas**, **QTablas**, **Lógica** y **Valores**:
la traza no publica eventos de tablas, de interruptores ni de valores, así que
esos cuatro siguen mostrando los números finales del motor. No se inventa nada
para llenarlos.

Las cifras vivas se calculan con **las mismas fórmulas del motor**
(`crates/genmsi-core/src/stat.rs`), que son integrales por la derecha sobre los
mismos instantes de evento que la traza publica. Por eso la cifra viva y la del
motor coinciden, y no se parecen.

### S5 — Editar durante una corrida

> "Editar detiene la corrida (lo de hoy)"

Cualquier edición del modelo borra la corrida y su reproducción, como hoy
(`store.ts`, la acción `apply`). La capa viva desaparece con ella.

### S6 — El panel lateral

> "Dos sub-pestañas: Bloque | Estado"

El panel lateral de la Reproducción tiene dos sub-pestañas:

- **Bloque**: las propiedades del bloque seleccionado (lo que hoy está en la
  pestaña Grafo).
- **Estado**: las colas, las facilities y los almacenes de la corrida (lo que
  hoy está en el Monitor).

### S7 — Las bolitas sobre la curva

> "Seguir la curva real de la arista"

La bolita viaja sobre la curva que el grafo dibuja, no en línea recta de centro
a centro. La curva es la misma bezier que React Flow dibuja para la arista.

### S8 — La lógica queda ordenada

> "manteniendo la logica bien ordenada"

- `src/graph/graph.ts`: la geometría del grafo como funciones puras
  (nodos, aristas y los puntos de los puertos).
- `src/monitor/live.ts`: la capa viva como funciones puras (los números de cada
  bloque y el punto de la bolita sobre la curva).
- `src/monitor/stats.ts`: las cifras del motor como funciones puras (las mismas
  fórmulas de `stat.rs`).
- Los componentes sólo proyectan; no calculan.

### S9 — Nada nuevo que instalar

La app no agrega dependencias. Se usan las que ya están: `@xyflow/react` 12.12
(expone `ViewportPortal`, `getBezierPath` y `EdgeLabelRenderer`).

### S10 — La capa viva no puedePaidar el grafo

Los números de cada bloque **no** viajan en `node.data` de React Flow: si
viajan, cada cambio rehace el registro de todos los nodos. Cada nodo se
suscribe a su propio número, y sólo se redibuja el que cambió. La capa viva se
dibuja en un `ViewportPortal` con un SVG, así que el grafo no se entera de que
el tiempo pasó.

## Tasks

| ID | Qué | Specs | Ruta | Commit |
|---|---|---|---|---|
| T1 | `src/graph/graph.ts`: `graphOf`, `portPoint`, `edgePath` como funciones puras, con tests | S7, S8 | writer A | `feat(app): the geometry of the graph, as pure functions` |
| T2 | `src/monitor/stats.ts`: las fórmulas de `stat.rs` como funciones puras, con tests | S4, S8 | writer B | `feat(app): the engine's statistics, as pure functions` |
| T3 | `replay.ts`: la traza resuelve la espera de cada dequeue y la reproducción lleva las cifras vivas; `checkFidelity` las compara con el motor | S4, S8 | writer C | `feat(app): the live figures, checked against the engine` |
| T4 | `GraphView` acepta la capa viva; los números van en el nodo por suscripción y las bolitas en un `ViewportPortal` | S1, S7, S10 | parent | `feat(app): the graph carries the live layer` |
| T5 | La ventana única: la pestaña Simulación, los sub-paneles, el panel lateral Bloque\|Estado, el Reporte vivo | S1, S2, S3, S4, S5, S6 | parent | `feat(app): one window for the model, the run and the report` |
| T6 | `check-visual-loop.sh` actualizado y la verificación visual completa | S2, S3 | parent | `test(app): the visual loop follows the new window` |

## Log

### L1 — 2026-10-22

> "perfecto! tengo una propuesta para hacerte sobre la aplicacion:
>  - Que te parece fusionar la ventana monitor con la de grafo? uniendo la mejor parte de ambas, ya que tranquilamente podrian complementarse, manteniendo la logica bien ordenada, no te parece? ya que realmente la parte de grafos podria implementarse en la parte de reproduccion de monitor y seria casi lo mismo no? como esto es un cambio grande no pido hacer ninguna featuire mas asi pensamos e implementamos bien esto, no te quedes con lo que te acuerdes para responderme, analiza bien el codigfo actual, podrias usar codegraph si te es de utilidad"

### L2 — 2026-10-22

Decisiones del usuario sobre la estructura:

> "que tal si combinamos todo y los reportes los colocamos como ventana como esta dentrod e monitor la ventana de reproduccion y graficos? y que todo se actualice en tiempo real, tanto los graficos como los reportes etc y combinamos lo mejor de todos los mundos?"

> "Editar detiene la corrida (lo de hoy)"

> "Dos sub-pestañas: Bloque | Estado"

> "Seguir la curva real de la arista"

> "Simulación → Reproducción | Gráficos | Reporte"

### L3 — 2026-10-22

> "me gustaria que todo este en vivo, esto le daria mucha mas vista a la aplicacion, lo ves viable?"

### L4 — 2026-10-22 — Evidencia del análisis

El análisis del código actual que sostiene las specs:

- `GraphView.tsx:145` (`graphOf`) y `MonitorCanvas.tsx:39` (`picture`) dibujan
  **la misma picture desde la misma fuente**: `model.blocks` + `block.layout` +
  `model.edges`. Hay dos renderizadores del mismo modelo y el del Monitor es el
  más pobre (pierde los puertos, las etiquetas, el zoom y el minimapa).
- `GraphView.tsx:389` (`fitFlow`) y `MonitorCanvas.tsx:44` encuadran igual:
  ambos descartan los contenedores.
- `MonitorView.tsx:56` y `:117`: el reloj es estado local y se publica en cada
  `requestAnimationFrame`. Con React Flow, ese re-render sería de todo el árbol
  de nodos 60 veces por segundo (S10).
- `store.ts` (`apply`): toda edición borra `report`, `trace`, `monitor` y
  `transitions` (S5).
- `crates/genmsi-core/src/stat.rs:177-192`: el motor calcula el área como
  `content * (now - last_change)` y el tiempo total como `now - enqueued_at`,
  es decir integrales por la derecha sobre los mismos instantes que la traza
  publica (S4).
- `crates/genmsi-core/src/sim.rs`: la traza publica sólo `generate`, `enter`,
  `wait`, `leave` y `terminate`. **No** publica eventos de tablas, interruptores
  ni valores: por eso esas cuatro pestañas del reporte no pueden ser vivas
  (S4).
- `crates/genmsi-app/src/components/MonitorCharts.tsx:54`: el histograma de las
  tablas sale del **reporte**, no de la traza.
- `replay.ts:828` (`checkFidelity`): la reconstrucción desde la traza ya
  coincide exactamente con `report.entries` y `report.current` de cada bloque.
  Ese mecanismo es el que se extiende para verificar las cifras vivas (S4).
- `scripts/check-visual-loop.sh:250`: el script visual hace clic en la pestaña
  "Monitor" por nombre; hay que actualizarlo (T6).

### L5 — 2026-10-22 — Decisiones del usuario sobre la motor

El usuario eligió que **el motor sea la fuente de verdad** y que publique en
la traza lo que el reporte cuenta y la traza no llevaba:

- el nombre de la entidad en el evento `enter` (S4, T-1);
- las muestras de las tablas, el estado de los interruptores y el valor de los
  savevalues (S4, T-2);
- los miembros que salen de una cadena (S4, T-3).

El usuario: "en mi ultima respuesta me parecio la mejor para que todo
tenga coherencia y siga la misma filosofia".

### L6 — 2026-10-22 — Lo que se midió

- Las cifras vivas coinciden **exactamente** con el reporte del motor en 17 de
  los 22 modelos que compilan, después de que el motor publicara el nombre de
  la entidad. Los que no coinciden son los que vacían una cadena con un
  criterio: la traza no decía qué miembro sale.
- La distancia de la bolita a la curva de su arista es de **0.00** (medida en
  el navegador contra el `path` que React Flow dibuja).
- 60 fps con 81 nodos, 64 aristas y la capa viva activa (mediana 16,7 ms,
  p95 16,8 ms).
- La bola sale 2 s de modelo antes de la llegada y llega exactamente con la
  instancia: 38% de los cuadros con alguna bola en el modelo más simple, 79%
  en el supermercado.
