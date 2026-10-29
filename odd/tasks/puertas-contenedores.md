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

## Tasks

| T | Specs | Qué | Ruta | Estado | Commit |
|---|---|---|---|---|---|
| T1 | S1, S2 | Las puertas sin flujo van a la grilla del ámbito, como los bloques sin flujo: se cae la rama `pending`, la superposición y el doble desplazamiento | format | **hecho** | — |
| T2 | S3, S4 | Una lista de colocaciones con identidad discriminado: un aplicador, un deshacer, un comando; y la composición de todo el modelo da lugar a las puertas | format, engine | pendiente | — |
| T3 | S5 | Las cifras del contenedor salen de los cruces de frontera, siempre; se borra el heuristic de la primera puerta | app | pendiente | — |
| T4 | S6 | Los cruces de frontera viajan en el hop: la puerta es un nodo del flujo y el par deja de ser ambiguo | format, core, app | pendiente | — |
| T5 | S7 | La identidad de la puerta en la app es discriminado, y su silueta pasa a la tabla de las clases | app | pendiente | — |

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
