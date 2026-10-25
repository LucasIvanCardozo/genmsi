# genmsi-reordenamiento — el lienzo se compone bien

Objetivo: rehacer el sistema de reordenamiento del lienzo (la composición de
la geometría) para que un modelo se lea, y medirlo antes y después.
Rama: `feature/genmsi-reordenamiento`. Entrega: `ask-on-risk`.
Pruebas: `cargo test --workspace`, `bun test`, `bunx tsc --noEmit`,
`crates/genmsi-app/scripts/check-visual-loop.sh`.

## Specs

### S1 — El pedido

> "Hola! en esta sesion nos enfocaremos en mejorar totalmente el sistema de
> reordenamiento que actuialmente no esta muy bien hecho te paorece? agarra el
> contexto que creas necesario y dame tu recomendacion"

El reordenamiento es el sistema completo: el algoritmo que compone el lienzo
(`genmsi_format::layout`), su superficie (el botón de la app, `genmsi layout`
de la CLI, el `save_model` del MCP) y su lectura en la app. "Mejor totalmente"
se acepta como un resultado medible, no como una impresión: la fase se cierra
con una medida que compara el antes y el después sobre el corpus de
referencia.

### S2 — El origen del pedido (sesión anterior)

> "el reordenamiento no es del todo bueno realmente debemos cambiar la forma
> en que se hace esto, pero a lo mejor esto es para una sesión enfocada
> completa a esta mejora"

Esa sesión dejó escrito que la composición es del motor (D18) y que cambiar
su forma es trabajo de una sesión propia, con su propia medida. Esta es esa
sesión.

### S3 — Lo que la sesión incluye

> "Motor + app + medición (recomendado)"

Incluye: el algoritmo nuevo en `genmsi-format`, el botón de la app, los
contenedores dibujados como marcos reales y el benchmark con su línea base.
Queda fuera de esta sesión la edición manual (arrastrar, mover un bloque
entre contenedores, guías de alineación): es otra sesión.

### S4 — Lo que el botón hace

> "Recomponer todo (recomendado)"

Un solo botón: **Reacomodar** recompone el lienzo entero con el algoritmo
nuevo. No hay un segundo comando ("Ordenar") que conserve la disposición del
autor: la disposición que el autor dejó se conserva en el archivo (S6), pero
recomponer es recomponer.

### S5 — La medida (el criterio de aceptación)

Sobre los 25 modelos de `fixtures/models` (los 20 que el motor acepta más
5 que no compilan, que también se compondrán, S7), con la geometría quitada
y compuestos por el motor:

| Medida | Antes (línea base) | Después (criterio) |
|---|---|---|
| Pares de bloques que se pisan | 0 | 0 (se mantiene) |
| Bloques fuera de su contenedor | 0 | 0 (se mantiene) |
| Bloques que ningún enlace alcanza | 246 de 1008 | **menos de la mitad** |
| Cruces de aristas | 383 (de cero) / 496 (recompuesto) | **≤ 128** (un tercio de la base) |
| Proporción del lienzo (ancho/alto) | 0.3 – 16.4 | **1.0 – 2.5 en el 80%** de los modelos, y **ninguno por encima de 4:1** |

Y, en todos los modelos:

- la composición es **determinista**: el mismo modelo produce la misma
  geometría, byte a byte;
- la geometría **no toca la semántica**: el programa GPSS y el reporte salen
  idénticos antes y después de componer;
- **un solo algoritmo**: la app, la CLI y el MCP lo piden al motor.

### S6 — Lo que no se rompe

- La geometría es del motor (D18): la app no tiene copia del algoritmo.
- Un modelo ya compuesto conserva la disposición de su autor al guardarlo:
  sólo se compone lo que está incompleto.
- La geometría no entra en la simulación.
- `genmsi layout` sigue sin escribir nada salvo que se lo pidan con
  `--write`, y la escritura es atómica.

### S7 — Lo que se arregla de paso

`genmsi layout` carga **y compila** el modelo (`crates/genmsi-cli/src/model.rs`),
así que un modelo que el motor rechaza no se puede reacomodar, aunque la
geometría no entre en la simulación. La composición es una operación de
geometría: tiene que poder hacerse sobre un modelo roto.

## Tasks

| T | Specs | Qué | Ruta | Estado | Commit |
|---|---|---|---|---|---|
| T1 | S1, S2, S5 | La medida: benchmark determinista sobre el corpus, línea base commiteada e invariantes como pruebas | inline | pendiente | — |
| T2 | S1, S5, S6 | El modelo jerárquico: árbol de contenedores, contenedor como macro-nodo, rango global con los links que cruzan el borde, quiebre de ciclos | inline | pendiente | — |
| T3 | S1, S5 | El orden dentro de la capa: reducción de cruces determinista (sin azar) | inline | pendiente | — |
| T4 | S1, S5 | Las coordenadas: sin solapes, la huella del contenedor reservada, los huérfanos aparte y la proporción del lienzo | inline | pendiente | — |
| T5 | S3, S4, S5 | La app: el botón recompone todo y los contenedores se dibujan como marcos reales | delegada | pendiente | — |
| T6 | S1, S6, S7 | La superficie: `genmsi layout` compone un modelo que no compila; los documentos (D18, `app.md`, `cli.md`) | inline | pendiente | — |
| T7 | S1, S5 | La prueba visual: el lazo visual del antes y del después, con capturas revisadas | inline | pendiente | — |

## Log

### L1 — El pedido del usuario (verbatim)

> Hola! en esta sesion nos enfocaremos en mejorar totalmente el sistema de
> reordenamiento que actuialmente no esta muy bien hecho te paorece? agarra el
> contexto que creas necesario y dame tu recomendacion

### L2 — Las dos decisiones de la sesión

El usuario eligió, de entre las opciones que la recomendación ofrecía:

> "Motor + app + medición (recomendado)"

> "Recomponer todo (recomendado)"

Quedan fuera: el comando "Ordenar" que conservara la disposición del autor, y
la edición manual (arrastrar, mover entre contenedores, guías).

### L3 — La línea base medida

La primera cuenta (un guion de Python sobre `genmsi layout --json`) estaba
mal, y la medida en Rust la corrigió: los "322 pares que se pisan" eran
contenedores conteniendo a sus propios descendientes, que es el diseño y no
un defecto. La medida vive ahora en el motor (`genmsi_format::measure`) y la
prueba del corpus la compara contra la línea base commiteada
(`crates/genmsi-format/tests/layout-baseline.json`).

La línea base real, sobre los 25 modelos de `fixtures/models`:

| model | bloques | cruces | aislados | lienzo | ratio |
|---|---|---|---|---|---|
| Parcial 6-11-23 | 107 | 118 | 22 | 11316×690 | 16.4 |
| Parcial 8-11-21 | 81 | 32 | 19 | 8604×1050 | 8.2 |
| TP4 supermercado | 81 | 37 | 25 | 8436×1206 | 7.0 |
| TP8 mecánicos | 82 | 58 | 13 | 7212×846 | 8.5 |
| TP3 aeropuerto | 88 | 26 | 9 | 7164×630 | 11.4 |
| **total (25 modelos)** | **1008** | **383** | **246** | | **0.3 – 16.4** |

El total: **0 solapes**, **0 bloques fuera de su contenedor**, **383 cruces**
(de cero), **496** (recompuesto), **246 bloques que ningún enlace alcanza**
(uno de cada cuatro), y once modelos con un lienzo de más de 4:1 (el peor,
16.4:1 — una cinta de 11316×690 px para 107 bloques).

Cinco modelos no compilan (falta un `TERMINATE`, un `UNLINK` sin etiqueta,
un nombre de entidad con espacio): es semántica, no geometría, y la
composición va a poder llegar a ellos (S7).

### L4 — El diagnóstico

Seis defectos, con su sitio:

1. **El orden dentro de la capa es el del id** (`layout.rs:147`, `layout.rs:284`):
   la dirección del flujo no se respeta y no hay reducción de cruces.
2. **Las coordenadas no se equilibran** (`layout.rs:227`): todas las capas
   arrancan en la misma `x` y se apilan; no hay alineación con el predecesor
   ni objetivo de proporción. De ahí la cinta de 16:1.
3. **Un cuarto de los bloques está en ningún flujo**: los 246 bloques que
   ningún enlace alcanza caen todos en la primera capa, apilados en una
   columna por id, sin relación con lo que los rodea.
4. **La jerarquía no participa en el rango**: los links que cruzan el borde de
   un contenedor no lo rankean en su padre, y sus hijos se ordenan como si el
   contenedor no existiera.
5. **"Reacomodar" borra la disposición del autor** siempre
   (`crates/genmsi-app/engine/src/session.rs:216`), sin alternativa.
6. **`genmsi layout` no compone un modelo que no compila**
   (`crates/genmsi-cli/src/model.rs:38`): la geometría no entra en la
   simulación y no debería estar atada a la compilación.

Lo que **no** es un defecto (la primera cuenta lo tomaba por uno): los
contenedores contienen a sus descendientes y ningún bloque se pisa con otro
(0 solapes, 0 fuera de su contenedor, en los 25 modelos).

### L5 — El plan

Siete unidades, cada una con su commit:

- **T1** la medida primero: el benchmark y la línea base, para que el
  resultado se pueda comparar y no se pueda "ver mejor" sin evidencia.
- **T2** el modelo jerárquico: el contenedor como macro-nodo y el rango global
  con los links que cruzan el borde.
- **T3** el orden dentro de la capa: reducción de cruces determinista.
- **T4** las coordenadas: sin solapes, la huella reservada, los huérfanos
  aparte, la proporción del lienzo.
- **T5** la app: el botón y los contenedores como marcos reales (delegada a
  `gentle-ai-worker`: superficie propia, la app).
- **T6** la superficie: la CLI compone modelos que no compilan y los
  documentos se actualizan.
- **T7** la prueba visual: el lazo con capturas del antes y del después.

Los riesgos, por unidad:

- **T1**: ninguno. Es cálculo y un guion nuevo.
- **T2, T3, T4**: **ítem 3** (cambia un resultado observable que la app, la CLI
  y el MCP consumen: la composición). Cubierto por la prueba de determinismo,
  la de que el GPSS sale byte a byte igual y los invariantes del corpus.
- **T5**: ninguno. Es el render de la app.
- **T6**: **ítem 1** (`genmsi layout --write` reescribe un modelo guardado).
  Cubierto por el verificador independiente y por las pruebas de escritura
  atómica e idempotente.
- **T7**: ninguno. Es una revisión con capturas.
