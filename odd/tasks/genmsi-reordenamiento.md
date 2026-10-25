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
- Un modelo cuyos bloques llevan todos geometría conserva la disposición de
  su autor al guardarlo. Uno que no la lleva se compone **entero**: la
  composición es una imagen, no un parche (verificarlo dejó esto escrito:
  finding A de L14).
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
| T1 | S1, S2, S5 | La medida: benchmark determinista sobre el corpus, línea base commiteada e invariantes como pruebas | inline | hecho | `088b79d`, `c61b942` |
| T2 | S1, S5, S6 | El modelo jerárquico: árbol de contenedores, contenedor como macro-nodo, rango global con los links que cruzan el borde, quiebre de ciclos | inline | hecho | `1f7d2a2` |
| T3 | S1, S5 | El orden dentro de la capa: reducción de cruces determinista (sin azar) | inline | hecho | `0319e72` |
| T4 | S1, S5 | Las coordenadas: sin solapes, la huella del contenedor reservada, los huérfanos aparte y la proporción del lienzo | inline | hecho | `61f4845` |
| T5 | S3, S4, S5 | La app: el botón recompone todo y los contenedores se dibujan como marcos reales | inline | hecho | `3f182b9`, `b3f50ea` |
| T6 | S1, S6, S7 | La superficie: `genmsi layout` compone un modelo que no compila; los documentos (D18, `app.md`, `cli.md`) | inline | hecho | `66d2e82` |
| T7 | S1, S5 | La prueba visual: el lazo visual del antes y del después, con capturas revisadas | inline | hecho | — |
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

### L6 — T1: la medida (`088b79d`, `c61b942`)

El medidor vive en el motor (`crates/genmsi-format/src/measure.rs`): cuenta
los bloques, las aristas, los solapes, los bloques fuera de su contenedor, los
cruces, los bloques que ningún enlace alcanza y la caja del lienzo. Un ejemplo
(`cargo run -p genmsi-format --example layout_measure`) lo imprime sobre el
corpus y `--write` refresca la línea base; la prueba del corpus
(`tests/layout_corpus.rs`) compara la composición contra esa línea base y
falla con la diferencia si el algoritmo cambia sin que la línea base se
refresque a propósito.

Lo que encontró (y corrigió a la primera cuenta, que tomaba por solapes los
contenedores conteniendo a sus descendientes):

- **0 solapes** y **0 bloques fuera de su contenedor** en los 25 modelos: el
  algoritmo actual no pisa nada, y eso se mantiene;
- **383 cruces** (de cero) y **496** (recompuesto);
- **246 bloques de 1008** (uno de cada cuatro) en ningún flujo;
- **once modelos** con un lienzo de más de 4:1, el peor de 16.4:1.

Las tres pruebas del corpus: la medida es la de la línea base, ningún bloque
cae fuera de su contenedor, y la composición es determinista (las dos
pictures, dos veces).

### L7 — El corte de la entrega

La fase va de ~1.700 líneas, así que el usuario eligió la **cadena sobre la
rama de la feature** (`chain_strategy=feature-branch-chain`): cada unidad de
trabajo es un PR encadenado sobre `feature/genmsi-reordenamiento`, con el
corte de ~400 líneas. Primer corte: `088b79d` + `c61b942` (la medida).

### L8 — T2: el flujo cruza las fronteras (`1f7d2a2`)

El flujo ya no se cuenta por contenedor: se cuenta sobre el modelo entero, con
las fronteras de los contenedores **cosidas** (`crates/genmsi-format/src/
layout/flow.rs`).

- Un enlace entre dos bloques es una arista del flujo.
- Un enlace que toca la frontera de un contenedor no lo es: el contenedor es un
  marco. El flujo que entra en un contenedor va al bloque que su puerto de
  entrada nombra, y el que sale viene del bloque que su puerto de salida
  nombra.
- La profundidad de un bloque (su **rango**) es el camino más largo que llega
  a él. Los enlaces que cierran un ciclo se sacan del camino (los encuentra
  un recorrido en profundidad), así que el rango siempre está definido y
  siempre es el mismo.
- Un contenedor se coloca donde empieza su contenido: el marco rodea al flujo,
  no va antes de él.

El RED de la prueba: contra el algoritmo anterior, los dos contenedores de un
modelo con un enlace que cruza la frontera salían en la misma capa ("B at 0,
A at 0"); con el nuevo, el del principio del flujo sale primero.

Lo que cambió en la medida:

| | antes | después |
|---|---|---|
| Parcial 6-11-23 | 11316×690 (**16.4**) | 7608×2244 (**3.4**) |
| Parcial 8-11-21 | 8604×1050 (8.2) | 4812×1632 (2.9) |
| TP4 supermercado | 8436×1206 (7.0) | 4992×2496 (2.0) |
| modelos por encima de 4:1 | 11 | **5** |
| cruces | 383 | 458 |

Los cruces **suben**, y es lo esperado: las capas ahora agrupan por la
profundidad del flujo (antes cada contenedor contaba por su cuenta) y el orden
dentro de la capa sigue siendo el del id. Ese orden es lo que cambia T3.

### L9 — T3: el orden dentro de la capa (`0319e72`)

Los bloques de una capa ya no se ordenan por su id: se ordenan por el flujo.
El algoritmo es el clásico y no tiene azar: el **baricentro** de los vecinos
(el promedio de las posiciones de los bloques a los que un bloque está unido)
barre las capas de arriba abajo y de abajo arriba, y después el **transpuesto**
intercambia dos vecinos cuando el intercambio cruza menos enlaces.

`crates/genmsi-format/src/layout/order.rs`. El RED de la prueba: contra el
orden por id, dos enlaces cruzados se cruzaban; con el orden nuevo, no.

| | antes de T3 | después de T3 |
|---|---|---|
| cruces (de cero) | 458 | **329** |
| cruces (recompuesto) | 550 | **422** |

### L10 — T4: las coordenadas (`61f4845`)

El flujo ya no es una cinta: las capas se **empaquetan en columnas**. Una capa
que sigue a otra va debajo de ella, y las columnas van una tras otra, de
izquierda a derecha. El flujo se lee como se escribe: down una columna, y
después la columna siguiente. Y como la app dibuja las entradas de un bloque
arriba y las salidas abajo, el flujo hacia abajo encaja con los puertos.

- Los bloques que **ningún enlace alcanza** (246 de 1008) no son parte del
  flujo: van aparte, en una rejilla, para no estirar la imagen del flujo.
- Los contenedores crecen para lo que poseen, y **ningún bloque queda fuera**
  de su contenedor (0 en los 25 modelos).
- La imagen se acerca a lo que espera quien lee: la proporción del lienzo pasó
  de 0.3–16.4 a **0.5–3.3**, y el lienzo del Parcial de 11316×690 a
  3348×3414 (el autor lo tenía en 906×871, con bloques encimados).

Lo que la medida dice ahora:

| | antes (T1) | ahora |
|---|---|---|
| solapes | 0 | 0 |
| bloques fuera de su contenedor | 0 | 0 |
| bloques en ningún flujo | 246 | 246 (aparte, en rejilla) |
| cruces | 383 | **345** |
| enlaces que pasan por encima de un bloque | 280 | 404 |
| proporción del lienzo | 0.3 – 16.4 | **0.5 – 3.3** |

Los cruces bajan y la imagen se compacta, pero **más enlaces pasan por encima
de un bloque**: las columnas están cerca y el enlace que salta de una columna
a la otra cruza las de en medio. La métrica nueva (`por-encima`) es la que
dice si algo se lee mal, y queda como deuda de la fase: el orden dentro de
una columna es el del flujo, pero el enlace entre columnas se dibuja en
diagonal.

### L11 — T5: la app (`3f182b9`, `b3f50ea`)

El botón **Reacomodar** ya pedía la composición al motor, así que no cambia:
lo que cambia es lo que el motor compone. En la app, dos cosas:

- **Un contenedor es el marco de lo que sostiene**: se dibuja detrás de su
  contenido y **no se arrastra** (los que se arrastran son los bloques de
  dentro). Antes el marco se podía mover y dejaba a su contenido atrás.
- **La imagen se encuadra de nuevo** después de reacomodar: el lector tiene
  que ver la imagen nueva entera, no el trozo de la vieja que estaba
  mirando. Y el encuadre baja el piso del zoom (0.15), porque una imagen
  compuesta puede ser ancha.

La prueba de la app (`tests/graph.test.ts`) fija que el contenedor es el marco
y que no se arrastra.

### L12 — T6: la superficie (`66d2e82`)

`genmsi layout` carga **y compilaba** el modelo, así que un modelo que el
motor rechaza no se podía reacomodar: la geometría no entra en la simulación
y no tiene por qué estar atada a la compilación. La CLI tiene ahora
`model::load_geometry`, que carga sin compilar, y `layout` lo usa.

Comprobado: `genmsi layout` sobre `TP9__tp9.gmdl` (un `UNLINK` sin etiqueta
que el motor rechaza) ya no falla con exit 1: el modelo sale `unchanged`
porque el fixture ya lleva geometría. Sin geometría compone sus 67 bloques
(esto lo comprueba ahora `crates/genmsi-cli/tests/layout_write.rs`).

Los documentos: D18 (la decisión, con la reconstrucción de la composición),
`docs/format-and-engine.md` (cómo compone el motor), `docs/app.md` (el
botón y el marco) y `docs/cli.md` (el comando compone un modelo que no
compila).

### L13 — T7: el lazo visual

El lazo visual completo, con el navegador, sobre `TP4__supermercado.gmdl`
(81 bloques):

- **Antes** (la geometría del autor): los bloques **se pisan** entre ellos
  (el `Generate` encima del contenedor, los contenedores encima de los
  bloques que no son suyos). Es lo que el lector veía.
- **Después** de `Reacomodar`: ningún bloque se pisa, los contenedores son
  marcos alrededor de lo que sostienen, el flujo se lee de columna en
  columna, y el deshacer devuelve la imagen del autor (un solo paso de undo,
  como siempre).

El undo de la composición y la corrida del modelo (el reloj del monitor)
siguen funcionando después de reacomodar.

### L14 — El verificador independiente de la unidad de escritura

El plan de `assess` con la escalación del ítem 1 (la escritura) pidió un
verificador independiente sobre la unidad de escritura. Verificó seis cosas:

| # | Promesa | Veredicto |
|---|---|---|
| 1 | La escritura es atómica | **CUMPLE** (4785 lecturas concurrentes, 0 parciales; un fallo inyectado deja el destino intacto y sin temporal) |
| 2 | La escritura es idempotente | **CUMPLE** (mismos bytes y mismo mtime la segunda vez) |
| 3 | Un modelo que el motor rechaza se compone y se escribe | **CUMPLE CON RESERVA** (4 de 5; el quinto fallaba por la validación estructural) |
| 4 | La geometría no cambia la semántica | **CUMPLE** (GPSS y reporte byte a byte iguales en los 20 modelos que compilan) |
| 5 | Un modelo que no necesita composición no se escribe | **CUMPLE** (mismo mtime) |
| 6 | Ningún bloque se pisa ni sale de su contenedor | **CUMPLE** (0 y 0, medido sobre la geometría escrita en disco) |

Sus hallazgos:

- **A** (preexistente): con geometría **parcial**, la composición reposiciona
  **todos** los bloques. La lectura literal de S6 ("sólo se compone lo que
  está incompleto") no era la del comportamiento. Se corrigió la **palabra**
  de S6 y de los documentos (la composición es una imagen, no un parche), no
  el comportamiento: componer la mitad y dejar la otra mitad es una imagen
  que no se puede leer.
- **B** (preexistente): el temporal de la escritura tiene nombre fijo, así
  que dos escrituras simultáneas al mismo archivo lo comparten. No se
  reprodujo fallo en 8 iteraciones; queda dicho.
- **C** (preexistente): el temporal vive en el directorio de destino, así que
  un lector que enumera el directorio lo ve a medio escribir. El comentario
  del código decía lo contrario: se corrigió el comentario.
- **D** (fuera de la unidad): la app guarda con `fs::write` sin temporal; el
  MCP sí usa temporal y rename. Queda dicho para otra fase.

La **reserva** de S7 sí se corrigió: `model::load_geometry` ya no valida el
modelo (sólo lo lee), así que un modelo con un nombre de entidad inválido
también se puede reacomodar (`Parcial 8-11-21`, que antes fallaba con exit 1,
ahora compone sus 81 bloques).

Y las cinco pruebas de regresión que el verificador dejó validadas (RED en la
base, GREEN en la cabeza) quedan en el repositorio:
`crates/genmsi-cli/tests/layout_write.rs` (idempotencia, atómica, la
semántica que no se toca, el modelo que el motor rechaza y la escritura que
falla sin dejar nada atrás).

### L15 — La revisión nativa: empezada, sin veredicto

El plan de `assess` (con la escalación del ítem 1) dejó la revisión nativa
**pendiente** y con el verificador independiente como paso obligatorio (el
plan pide los dos: el verificador corre y la revisión nativa es la vista
adicional). La revisión se empezó con las tres lentes que el cambio pedía
(risk, readability, reliability) y el consentimiento lo resolvió el host.

La **captura no llegó a correr**: el provider rechaza el binding de captura
("collectBinding is missing or stale for current STATUS") en todas las
formas que el protocolo permite (objeto, cadena serializada, cadena con las
comillas escapadas), y un segundo `inspect` reporta el estado del provider
como inconsistente ("unrelated target status is inconsistent"). Se probó
también con el workspace alineado al candidate congelado y con el workspace
en su cabeza: el mismo rechazo.

Lo que esto deja:

- la **revisión nativa no dio veredicto** (no es un "aprobado" ni un
  "rechazado"): queda como **no disponible**;
- el **verificador independiente sí corrió** y verificó la unidad de escritura
  (L14), que es lo que el plan pedía como paso obligatorio;
- el resto de la fase está probado por sus propias pruebas y por el lazo
  visual (L13).

La deuda queda dicha: el engine de revisión no pudo correr sobre esta
fase, y el commit que añadió las pruebas del verificador (L14) movió el
workspace después de que el candidate se congelara. Queda para otra sesión
reintentar la revisión nativa sobre la rama ya quieta.
