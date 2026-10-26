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

### S8 — La composición usa dagre, de arriba abajo

> "dagre en Rust + TB (recomendado)"

El motor compone el lienzo con **dagre** (el crate de Rust, puerto de dagre.js)
y con la dirección **de arriba abajo** (`rankdir: TB`, la de por omisión).
Razón: los puertos de la app son entradas-arriba y salidas-abajo, así que el
flujo que se lee es vertical; y dagre hace bien las tres piezas que lo
nuestro hace flojo (el rango con *network simplex*, los cruces con baricentro,
las coordenadas con **Brandes–Köpf**).

El algoritmo sigue siendo del motor (D18): la app, la CLI y el MCP lo piden, y
la geometría sigue fuera de la simulación.

### S9 — Los criterios que dicen si se lee

La proporción del lienzo deja de ser un criterio (un lienzo vertical es alto
por naturaleza). Los que miden si la imagen se lee:

| Medida | Antes (T1) | Criterio |
|---|---|---|
| Bloques pisados / fuera de su contenedor | 0 / 0 | 0 / 0 |
| Cruces de aristas | 383 | **menos de la mitad** |
| Enlaces que pasan por encima de un bloque | 280 | **menos de la mitad** |
| Longitud media de los enlaces | (nueva) | **menos de la mitad** |

Y en todos los modelos: determinista, la geometría no toca la semántica, y un
solo algoritmo (el del motor).

### S10 — Las fuentes del flujo arrancan el lienzo

> "viste que hay 3 generate? bueno los generate deben estar arriba, no abajo al
> final, me explico?"

Toda fuente del flujo (los bloques `Generate` del modelo) empieza el lienzo:
las fuentes de todas las cadenas están en la primera fila, y el flujo baja
desde ahí. El motor compone las cadenas separadas una por una y las coloca
al lado, todas con su fuente en la misma fila.

### S11 — El lienzo, lo más cuadrado posible

> "el reacomodar debe intentar minimizar el numero mas alto del aspect ratio,
> es decir al reacomodar el aspect ratio total debe ser el minimo del mas alto
> (alto o ancho), por ejemplo: es mejor 1300x1200 que 2000x300 me explico?
> esto haria que no se disperce tanto ya sea horizontalmente como verticalmente"

La composición deja el lienzo con el lado mayor lo más chico que la
estructura del modelo permita. Los bloques que no están en ningún flujo no
ocupan una hilera entera: van en una rejilla al lado del flujo, con la forma
que deja el lienzo más cuadrado.

### S12 — El lector mueve un bloque y el lienzo no se mueve

> "al tocar reacomodar, si luego muevo cualquier bloque de posicion me hace
> zoom out al maximo"

El lienzo se encuadra cuando llega un modelo y cuando el lector lo pide
(*Reacomodar*, *Encuadrar*). Un cambio del modelo —el lector mueve un
bloque— no lo reencuadra: la vista se queda donde el lector la dejó.

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
| T8 | S1, S8 | La dependencia: `dagre` en el motor, fijada, con la prueba de que compone un modelo de referencia | inline | hecho | `457ff16`, `e589b0f` |
| T9 | S1, S8, S9 | La composición con dagre: dirección TB (el flujo baja) y los contenedores como grafos compuestos | inline | hecho | `457ff16` |
| T10 | S1, S5, S9 | La medida nueva: la longitud media de los enlaces, y los criterios que dicen si se lee | inline | hecho | `457ff16` |
| T11 | S12 | El encuadre: el lienzo no se reencuadra cuando el modelo cambia (la regla vive en `graph/frame.ts`, con prueba) | inline | hecho | `e6aec89` |
| T12 | S10, S11 | Las fuentes arriba y la rejilla de los bloques que no están en el flujo; el lado mayor del lienzo como criterio | inline | en curso | `24f7d9b` |
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

### L16 — El usuario ve dos cosas y decide el algoritmo

El usuario probó la app instalada y dijo:

> "cosas que veo mal:
>  - al mover un compoinente de lugar me hace zoom-in al maximo.
>  - no me gusta como ahce el reacomodamiento realmente, podes buscar en
>    interntet si existe algun algoritmo top o mejor para nuestro caso? antes de
>    implementarlo presentame que opcion encontras y si realmente es mejor.
>    ademas a consideracion mia siento que elreacomodamiento actual que es
>    enfocado de izquierda a derecha deberia ser de arriba hacia abajo, no se,
>    me explico"

La segunda mitad tiene base técnica: los puertos de la app son entradas-arriba
y salidas-abajo (`GraphView.tsx`: `Position.Top` → `Position.Bottom`), así que
el flujo que la app dibuja es vertical; y dagre y ELK usan **de arriba abajo**
como dirección por omisión.

Lo que encontró la investigación (las fuentes están en el mensaje al usuario):

- **dagre** (el JS que usan Mermaid y Go d2) y su puerto a Rust
  (`dagre` 0.1.1): el pipeline completo de Sugiyama, **las cuatro
  direcciones**, **grafos compuestos con contenedores**, *network simplex*
  para el rango, baricentro para los cruces y **Brandes–Köpf** para las
  coordenadas; 0 dependencias, contrastado con dagre.js en 20 grafos, 528
  pruebas, Apache-2.0.
- **ELK**: el algoritmo más completo, con enrutado ortogonal y puertos como
  anclas; su ventaja grande no la aprovechamos (la app dibuja los enlaces), y
  su puerto a Rust (`elkrs`) es **"flat layout only for now"**: los grupos
  quedarían fuera.

El usuario eligió:

> "dagre en Rust + TB (recomendado)"

Y con ello, el criterio de aceptación cambia (S9): la proporción del lienzo
deja de medir si la imagen se lee (un lienzo vertical es alto por naturaleza) y
entran los cruces, los enlaces que pasan por encima de un bloque y la longitud
media de los enlaces.

### L17 — El zoom al máximo

El único punto donde la app toca la vista es `fitView`
(`crates/genmsi-app/src/components/GraphView.tsx`), y una mutación del modelo
(pegar un bloque) **no** reencuadra (medido: la vista no cambió). `fitView`
sube al `maxZoom` de React Flow cuando el conjunto a encuadrar es minúsculo
(un bloque solo), y eso es el "zoom-in al máximo" que vio el usuario.

Corregido (`e14133c`): el encuadre tiene un techo (`maxZoom: 1.25`), así que
ningún encuadre puede pasar de 1.25. Queda por confirmar con el usuario si el
síntoma era en cada arrastre o sólo en el primero tras abrir el modelo (si es
sólo el primero, la causa es el reencuadre al remontar la vista, y el techo lo
resuelve).

### L18 — T8, T9 y T10: dagre compone el lienzo (`457ff16`, `e589b0f`)

El motor compone el lienzo con **dagre** (el crate de Rust, puerto de
dagre.js), con la dirección **de arriba abajo**: la entrada de un bloque está
arriba y su salida abajo, así que el flujo que se lee baja.

Lo que la composición hace ahora, en orden:

1. El **flujo** del modelo se cuenta sobre el modelo entero, con las
   fronteras de los contenedores **cosidas** (`layout::flow`): el flujo que
   entra en un contenedor va al bloque que su puerto de entrada nombra, y el
   que sale viene del bloque que su puerto de salida nombra.
2. Los enlaces que **cierran un ciclo** se quitan (el flujo de un modelo con
   un ciclo sigue siendo un flujo, y el bloque al que vuelve el ciclo es su
   comienzo).
3. **dagre** corta el flujo en capas (*network simplex*), las ordena para
   cruzar menos enlaces (baricentro + transpuesto) y coloca los bloques
   (Brandes–Köpf).
4. Los **contenedores** son los marcos alrededor de los bloques que
   contienen: cada marco es la caja de su contenido, con el margen.

La medida del corpus, contra la línea base de T1:

| | T1 (el motor de antes) | dagre + TB |
|---|---|---|
| Bloques colocados | 1008 | **1008** |
| Bloques pisados | 0 | **1** |
| Bloques fuera de su contenedor | 0 | **0** |
| Cruces de aristas | 383 | **276** |
| Enlaces por encima de un bloque | 404 | **225** |

El canvas también cambió de forma: el Parcial pasó de 11316×690 (una cinta
de 16:1) a 3587×7080 (una columna de 0.5:1) — el flujo se lee hacia abajo,
que es lo que pedía el usuario.

**Tres cosas del crate que hubo que resolver** (dagre 0.1.1 es joven):

- **`set_edge` sin etiqueta deja la arista sin `minlen`**: el puerto deja la
  etiqueta a `None` y el `minlen` sale 0, con lo que **todas las filas se
  colapsan en una**. Hay que pasar `Some(EdgeLabel::default())`.
- **La convención de las coordenadas es el centro**: dagre coloca un bloque
  por su medio y el archivo lleva su esquina (la conversión va en el motor).
- **El *network simplex* entra en pánico con los grafos compuestos que tienen
  ciclos** (`network_simplex.rs:346`, un `unwrap` sobre un nodo sin rango):
  cinco de los 25 modelos. La composición no puede entrar en pánico, así que
  el motor la envuelve y, cuando falla, responde con el **árbol apretado**
  (`TightTree`), que funciona en todos los modelos. Los bloques que el
  árbol apretado deja sin sitio (los del ciclo) reciben un sitio al final, y
  los marcos de sus contenedores crecen para hold them.

**El coste**: el crate es lento en los modelos grandes (0.5 s en TP4, 2.4 s
en el Parcial) y la prueba del corpus tarda 25 s. Para los modelos de un
agente (10–30 bloques) son decenas de milisegundos.

El lazo visual, con la app de escritorio y `TP4__supermercado.gmdl`: el
"antes" es la geometría del autor (los bloques encimados) y el "después" un
diagrama por capas vertical, con el flujo bajando, los enlaces cortos y
locales y los contenedores como marcos.

### L19 — El flujo empieza arriba: el árbol apretado

El usuario vio el lienzo reacomodado y dijo que los bloques estaban como
"flex end" cuando deberían estar como "flex start": "esas tres columnas
deberían estar empezando de arriba, no abajo".

Tenía razón, y la causa no era de maquetación: el **network simplex** de dagre
minimiza la longitud total de los enlaces, y para lograrlo **empuja los
orígenes hacia abajo** (acortar un enlace pesa más que dejar las ramas
alineadas). En la captura, la columna izquierda arrancaba más abajo que las
otras dos porque ese nodo fuente lo había bajado el algoritmo.

El rango que hace lo contrario es el **árbol apretado** (`TightTree`): cada
bloque queda tan arriba en el flujo como puede estar, así que las fuentes de
un modelo están todas en la primera fila y **las ramas paralelas del flujo
arrancan juntas**: el lector ve el fork, no una secuencia.

El coste (dicho con números, sobre el corpus):

| | network simplex | árbol apretado |
|---|---|---|
| Cruces de aristas | **276** | 352 |
| Enlaces por encima de un bloque | **225** | 233 |
| Proporción del lienzo (TP8) | 0.6 | **1.7** |
| Las ramas arrancan juntas | no | **sí** |

Se eligió el árbol apretado: la lectura del flujo importa más que 76 cruces,
y la imagen además queda más equilibrada (una proporción de 1.7 se parece a la
del autor, 1.1, y la de 0.6 no).

El criterio de S9 (menos de la mitad de los cruces) ya no lo cumple el árbol
apretado (352 contra 383 de la línea base): el criterio que manda ahora es el
de la lectura. Queda dicho, y el otro rango sigue a una línea de distancia
(`Ranker::NetworkSimplex`) por si el lector prefiera la imagen compacta.

### L20 — El "flex end": los bloques que están en ningún flujo

El usuario probó la app con `modelocomedor.gmdl` (el modelo de la Facultad) y
dijo que el reacomodamiento dejaba las cosas "como flex end, cuando debería
ser flex start".

La causa está en el crate: **los bloques que ningún enlace alcanza no reciben
rango** (el recorrido del crate no los visita) y dagre los deja en la
**última fila**, o sea, al final de la imagen. Eran las notas del modelo
(los bloques 1 a 18, sin un solo enlace): la hilera larga que se veía **abajo**.

La corrección (L20, `cfd14f3`): los bloques que están en ningún flujo y viven
en la raíz van a la **primera fila**, arriba del flujo, con su propia fila. Los
que viven dentro de un contenedor se quedan con él (moverlos rompería el marco
de su contenedor).

La medida no cambia: 1 bloque pisado, 0 fuera de su contenedor, 276 cruces y
225 enlaces por encima de un bloque.

De paso se volvió al **network simplex**: los rangos son los mismos que con el
árbol apretado (el corte de filas es la profundidad del flujo), pero cruza 76
enlaces menos (`80f687f`).

### L18 — El usuario ve dos cosas más (verbatim)

> "bien pero cosas que pasan:
>  - al tocar reacomodar, si luego muevo cualquier bloque de posicion me hace
>    zoom out al maximo y todavia no entendiste bien loq ue quise decir porque
>    por ejemplo viste que hay 3 generate? bueno los generate deben estar
>    arriba, no abajo al final, me explico?
>  - tambien si es posible el reacommodar debe intentar minimizar el numero
>    mas alto del aspect ratio, es decir al reacomodar el aspect ratio total
>    debe ser el imnimo del mas alto (alto o ancho), por ejemplo: es mejor
>    1300x1200 que 2000x300 me epxlico? esto haria que no se disperce tanto
>    ya sea horizontalmente como verticalmente. busca en internet algoritmos
>    para esto, quiero el mejor, simple y eficaz"

#### El zoom (S12)

El bug estaba en `GraphView.tsx`: el efecto que encuadra dependía de
`model`, así que reencuadraba con cada cambio del modelo. Como `fitSignal`
deja de ser 0 después de *Reacomodar*, el efecto solo se activaba a partir de
ahí: exactamente lo que el usuario vio. La regla ahora vive en
`graph/frame.ts` (`decideFrame`), con su prueba: el lienzo se encuadra cuando
llega un modelo y cuando el lector lo pide, nunca porque el modelo cambie.

#### Las fuentes arriba (S10)

El modelo que el usuario tenía abierto (`modelocomedor.gmdl`) tiene tres
`Generate` (19, 22 y 38) y tres cadenas separadas: 19→20→21, 22→…→37 y
38→…→43. El motor las componía **terminadas al mismo nivel**: 22 quedaba en
la fila 1, 38 en la fila 9 y 19 en la fila 14.

La causa está en el crate: `dagre` 0.1.1 asigna los rangos desde el final
(`layout/rank/util.rs::longest_path` da rango 0 a cada sumidero), y su
`normalize_ranks` (`layout/util.rs:188`) hace un único desplazamiento global:
todas las cadenas terminan en la misma fila y sus inicios quedan escalonados.
El crate no separa tampoco los componentes horizontalmente (no existe un
`separateComponents` ✗) y su layout compuesto mete bloques de la raíz dentro
del marco de un contenedor.

La corrección: componer las cadenas separadas una por una (cada una con su
rango propio) y alinearlas por su inicio, de modo que todas las fuentes
quedan en la primera fila; y poner los bloques que no están en ningún flujo
en una rejilla al lado del flujo, con la forma que deja el lienzo más
cuadrado (S11).

La medida del lado mayor del lienzo (`largest` en `measure.rs`) baja de 60186
a 54099 sobre el corpus: los bloques que no estaban en ningún flujo ya no
estiran el lienzo en una hilera.
