# Las puertas son bloques y el contenedor es una caja opaca

Los tres problemas del reporte original están cerrados y medidos
(`odd/tasks/submodulos-alcance.md`). El usuario probó la app y reportó cuatro
cosas que siguen mal: el movimiento de la pantalla al colocar un bloque, las
puertas superpuestas, las puertas que no se comportan como bloques, y el
recuadro del contenedor que crece y se mueve. Esta unidad cierra las cuatro.

Rama: `feature/genmsi-contenedores-puertos`. Verificación: `cargo test
--workspace`, `node --test tests/*.test.ts` en `crates/genmsi-app`,
`bunx tsc --noEmit`, y la medición en la app corriendo.

## Specs

### S1 — El movimiento de la pantalla es del clic en el árbol

> "al colocar un bloque la pantalla se mueve visualmente hacia donde esta,
> ese movimiento me gusta y quiero que se realice cuando se le da click a un
> bloque pero desde el arbol. Al colocar un bloque no quiero que lo haga ese
> movimiento"

El movimiento que hoy hace el lienzo al agregar un bloque (el bloque nuevo
arrastra la vista hacia él) se hace **al hacer clic en un bloque del árbol**:
el lienzo sigue al bloque que el lector elige en el árbol. Al colocar un
bloque, el lienzo **no se mueve**: la vista se queda donde estaba.

### S2 — Las puertas no se superponen y se colocan donde corresponde

> "al crear un container, si entro dentro de el, se ven solapados uno encima del
> otro tanto el portin como el portout y si intento agregar mas portout o
> portin se colocan en localizaciones muy extrañas"

Las puertas de un ámbito tienen un lugar propio, uno por puerta, y ninguna
queda encima de otra. Agregar una puerta nueva la coloca en un lugar propio,
no en una posición fija que se pisa con la anterior.

### S3 — Las puertas son bloques como cualquier otro

> "Si dentro de un Container coloco cualquier bloque, los bloques de entrada y
> salida se mueven junto con el, y el reordenamiento no esta funcionando con
> ellos, es como si el portin y portout no fueran bloques como cualquier otro
> (literalmente deberian compartir codigo con los bloques normales)"

Una puerta es un bloque del modelo, con su propia posición:

- el **reordenamiento** del ámbito la compone como compone a cualquier otro
  bloque (participa en el flujo: la puerta de entrada abre el flujo y la de
  salida lo cierra);
- **mover un bloque** del ámbito **no** mueve las puertas;
- la puerta se **arrastra** como cualquier otro bloque y su posición se
  guarda;
- conserva su silueta de puerta (es una puerta) y comparte el camino de
  dibujo con los otros bloques.

### S4 — El recuadro del contenedor no cambia nunca

> "Si coloco bloques dentro de un Container y reordeno, el container aumenta su
> tamaño, esto no deberia pasar nunca y el container no deberia tener esta
> feature de aumentar de tamaño, siempre deberia tener el mismo como cualquier
> otro bloque. Ademas si reordeno o muevo ciertos bloques dentro de el no solo
> aumenta tamaño el bloque Conteiner sino que tambioen cambia de localizacion"

El recuadro de un contenedor es una caja **fija**: su tamaño y su posición no
dependen de lo que hay adentro y el reordenamiento de su contenido no los
toca.

### S5 — El contenedor es una caja opaca

La aclaración del usuario sobre qué es un contenedor:

> "vos pensa que cuando el Container se vea, lo que este dentro no se vera,
> osea realmente lo que tiene dentro no importa en absolutamente nada para el
> Container, me explico? osea el Container no deberia ni tener logica para
> nada de lo que tenga dedans mas que para comunicarse con el port in y out de
> dentro, quedo claro?"

El contenido de un contenedor **no le afecta en nada**: el motor no recalcula
su recuadro, ni su tamaño, ni su lugar. Lo único que el contenedor sabe de su
contenido es la comunicación con las puertas de adentro: el flujo entra por la
puerta de entrada y sale por la de salida.

Por lo tanto el interior de un contenedor es **un lienzo propio** (una
composición propia de sus bloques), y el recuadro del contenedor es **la caja
que se ve desde afuera**, que no tiene por qué rodear a ese interior.

## Tasks

| T | Specs | Qué | Ruta | Estado | Commit |
|---|---|---|---|---|---|
| T1 | S1 | El lienzo sigue al bloque que se elige en el árbol; al agregar un bloque no se mueve | app | **hecho** | `7baed15` |
| T2 | S4, S5 | El motor no recalcula el recuadro del ámbito | format | **hecho** | `bf58e30` |
| T3 | S2, S3 | Las puertas son nodos del modelo con posición propia, y el modelo la publica | format, engine, core | en curso | — |
| T4 | S2, S3 | La app dibuja cada puerta en su lugar, y su arrastre se guarda | app | pendiente | — |

## Log

### L1 — El pedido del usuario (verbatim)

> "bien, mucho mejor ahora con respecto a antes pero siguen habiendo cosas que
> mejorar:
>  - al colocar un bloque la pantalla se mueve visualmente hacia donde esta,
> ese movimiento me gusta y quiero que se
>    realice cuando se le da click a un bloque pero desde el arbol. Al colocar un
> bloque no quiero que lo haga ese
>    movimiento
>  - al crear un container, si entro dentro de el, se ven solapados uno encima
> del otro tanto el portin como el portout
>    y si intento agregar mas portout o portin se colocan en localizaciones muy
> extrañas. (aunque si pareciera que se
>    estan colocando porque si me salgo delc ontenedor si se ven sus puntos).
>  - Si dentro de un Container coloco cualquier bloque, los bloques de entrada
> y salida se mueven junto con el, y el
>    reordenamiento no esta funcionando con ellos, es como si el portin y portout
> no fueran bloques como cualquier otro
>    (literalmente deberian compartir codigo con los bloques normales)
>  - Si coloco bloques dentro de un Container y reordeno, el container aumenta
> su tamaño, esto no deberia pasar nunca y
>    el container no deberia tener esta feature de aumentar de tamaño, siempre
> deberia tener el mismo como cualquier otro
>    bloque. Ademas si reordeno o muevo ciertos bloques dentro de el no solo
> aumenta tamaño el bloque Conteiner
>    sino que tambioen cambia de localizacion"

### L2 — La pregunta sobre el recuadro y la respuesta

Se le preguntó al usuario qué debía pasar con el recuadro del contenedor, con
tres opciones (fijo siempre; la esquina fija y el tamaño según el contenido;
fijo y redimensionable a mano). La respuesta, textual:

> "vos pensa que cuando el Container se vea, lo que este dentro no se vera,
> osea realmente lo que tiene dentro no importa en absolutamente nada para el
> Container, me explico? osea el Container no deberia ni tener logica para
> nada de lo que tenga adentro mas que para comunicarse con el port in y out de
> dentro, quedo claro?"

La respuesta descarta las tres opciones y fija el criterio: **el contenedor es
una caja opaca** (S5). De ahí sale T2: el motor deja de recalcular el recuadro
del ámbito, y el interior del contenedor es un lienzo propio que no tiene por
qué caber dentro de la caja.

### L3 — Lo que se encontró en el código

- `relayout_scope` empuja el marco del ámbito (`frame_of_scope`): el recuadro
  del contenedor se recalcula alrededor de sus bloques, y por eso crece y se
  mueve al reacomodar (`crates/genmsi-format/src/layout.rs`).
- Las puertas **no son bloques**: son `Port` dentro del bloque contenedor, con
  un `layout` que el formato ya lee y escribe (`crates/genmsi-format/src/parser.rs`,
  `writer.rs`) pero que nadie usa. La app las dibuja pegadas al bloque que
  unen (`doorPlacement`, `crates/genmsi-app/src/graph/graph.ts`), por eso se
  mueven con él y por eso dos puertas del mismo bloque quedan una encima de la
  otra.
- La composición del ámbito ya usa las puertas como los **extremos** del
  flujo (`flow.entries` / `flow.exits`), pero como nodos **sin tamaño y sin
  posición**: no se escriben (`crates/genmsi-format/src/layout/flow.rs`).
- El documento del modelo no publica la posición de un puerto
  (`ModelPort` en `crates/genmsi-core/src/documents.rs`).
- El foco de la vista lo pone `addBlock` (`focusBlockId` en
  `crates/genmsi-app/src/state/store.ts`); el árbol ya avisa el clic de un
  bloque (`onSelectBlock` en `crates/genmsi-app/src/components/ModelTree.tsx`).

### L4 — La decisión de esta sesión

1. La puerta es un bloque del modelo con posición propia: el motor la compone
   como cualquier otro (con su tamaño), le escribe la posición, y el modelo la
   publica. La app la dibuja donde el modelo dice.
2. El recuadro del contenedor es fijo: el motor no lo recalcula nunca, ni al
   reacomodar ni al agregar contenido. Su lugar en el lienzo del padre lo
   compone el padre, como cualquier otro bloque.
3. El interior de un contenedor es un lienzo propio: la composición de sus
   bloques no tiene por qué caber dentro del recuadro del contenedor, y no se
   mueve por culpa del recuadro.
4. El movimiento de la vista es del clic en el árbol, no del alta de un bloque.

### L5 — T1 y T2 (hechos)

**T1 (`7baed15`)** — El lienzo sigue al bloque que el lector elige en el árbol y
no se mueve al colocar un bloque. El pedido de seguimiento lleva un token, así
que el lienzo sigue al bloque otra vez cuando el lector lo pide dos veces.
Los tests de la app: 297 en verde.

**T2 (`bf58e30`)** — El motor no recalcula el recuadro del ámbito: la
composición de un ámbito escribe las posiciones de sus bloques y nada más.
Medido sobre los modelos de referencia (25 modelos, 120 ámbitos): la caja del
contenedor no cambia de tamaño ni de lugar, y ningún bloque de un ámbito queda
encima de otro.

**Lo que se encontró al hacerlo (L3, actualizado)**: el recuadro del
contenedor en el `.gmdl` **no** es un marco alrededor de su contenido: el
módulo 11 del concurso de pesca mide 116 x 34 en (118, 192), y su contenido va
de (63, 75) a (521, 540). El recuadro que la app dibujaba era una invención
nuestra (el `frame_of_scope` del motor). Y las puertas de un contenedor **sí**
tienen posición en el `.gmdl`: la del módulo 11 está en (330, 78) con 35 x 35,
en el mismo espacio absoluto que sus bloques — no es relativa a la caja del
contenedor. Esa es la posición que el modelo ya sabía y que la app no usaba.

### L6 — T3: la puerta, un bloque del modelo

La puerta de un contenedor es un `Port` del modelo, y el modelo ya tiene el
lugar para guardarlo (`Port.layout`, que el `.gmdl` lee y escribe). Faltaban
tres cosas:

1. **La composición la place**: hoy los nodos de entrada y de salida del
   grafo de dagre no tienen tamaño (sólo marcan los extremos del flujo) y su
   lugar no se escribe. Ahora llevan la caja con la que la app dibuja una
   puerta (140 x 44) y su lugar vuelve en la composición, como el de cualquier
   otro bloque.
2. **El modelo la publique**: `ModelPort` gana su `layout`, y el documento del
   modelo lo lleva.
3. **Se pueda mover**: una puerta arrastrada se guarda como se guarda un
   bloque movido (el comando `movePort`).

Las coordenadas de una puerta son absolutas: en el canvas del modelo, como
las de los bloques. El `layout` de un puerto de un bloque normal sí es
relativo a ese bloque (es lo que dice el `.gmdl`), pero las puertas de un
contenedor se guardan en absoluto en los modelos de referencia.
