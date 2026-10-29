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
| T3 | S2, S3 | Las puertas son nodos del modelo con posición propia, y el modelo la publica | format, engine, core | **hecho** | `4d53c82` |
| T4 | S2, S3 | La app dibuja cada puerta en su lugar, y su arrastre se guarda | app | **hecho** | `9215021` |
| T5 | S2, S3, S5 | El pulido de las puertas: se pueden borrar, nacen con lugar y el contenedor nuevo nace vacío | format, app | **hecho** | `ffad75f` |

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

Medido en la app corriendo (concurso de pesca, 87 bloques):

| acción | el lienzo se movió |
|---|---|
| colocar un bloque | **no** (`translate(179.371px, -237.003px)` antes y después) |
| elegir un bloque del árbol (80 filas) | **sí** (`translate(104.228px, -183.743px)`) |

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

### L7 — T3 y T4 (hechos), y lo que se midió

**T3 (`4d53c82`)** — La puerta es un nodo del modelo con posición propia:

- el motor la compone **con tamaño** (la caja con la que la app dibuja una
  puerta, 140 x 44), así que dagre la coloca como coloca a cualquier bloque:
  la de entrada abre el flujo y la de salida lo cierra;
- su lugar se escribe en el modelo con el de los bloques, en el **mismo
  comando** (una sola entrada del historial de deshacer);
- el documento del modelo publica el lugar de cada puerto (`ModelPort.layout`);
- una puerta se **arrastra** como cualquier otro bloque (`movePort`).

Una puerta que lleva el flujo a varios bloques es **una** puerta: tiene un
lugar, no uno por enlace.

**T4 (`9215021`)** — La app dibuja cada puerta donde el modelo dice, y una
puerta que todavía no tiene lugar (una nueva) va **debajo de las puertas de su
misma clase** que ya tienen uno: dos puertas nunca comparten lugar.

Medido en la app corriendo (concurso de pesca, 87 bloques):

| regla | medida |
|---|---|
| colocar un bloque no mueve el lienzo | `translate(179.371px, -237.003px)` antes y después |
| elegir un bloque del árbol sí lo mueve | `translate(104.228px, -183.743px)` |
| el recuadro del contenedor no cambia al reacomodar su contenido | `(118, 192, 116 x 34)` antes y después |
| mover un bloque no mueve las puertas | los dos `layout` de las puertas idénticos antes y después |
| arrastrar una puerta la deja donde se suelta | la entrada pasó de `(330, 78)` a `(400, 100)` |
| las puertas no se superponen | 4 puertas (2 de entrada, 2 de salida) en pantalla, 0 solapadas |
| una puerta nueva va bajo las de su clase | `in-1` y `in-3` una debajo de la otra; `out-2` y `out-4` ídem |

El dibujo de los 66 bloques de actividad queda **idéntico**: el diferencial del
DOM (silueta, texto, puertos, caja y anillo de cada nodo) da 66/66 nodos
iguales.

`cargo test --workspace` en verde; la app, 296/296; `bunx tsc --noEmit`
limpio.

### L8 — T5: el pulido de las puertas (hecho)

Lo que el usuario reportó después de probar:

> "cuando entro en un container recien creado, si muevo la entrada uno, la
> salida se mueve (esto deja de pasar luego)"

> "Si elimino un port in o port out, al salir del conteiner y volver a entrar
> estos no se borran"

> "en el arbol quitar el boton de 'Nuevo', ya que esto ya no es necesario que
> este aqui"

**El botón "Nuevo" del árbol** — fuera ✓: la paleta de clases ya trae
"Submódulo", "Entrada" y "Salida" ✓, así que el botón era un camino
redundante.

**Mover una puerta ya no mueve a la otra** ✓: el lugar de una puerta que
todavía no tiene depends del contenido del ámbito, no de las otras puertas
✓ (medido: la entrada pasó de (80, 80) a (500, 300) y la salida siguió en
(80, 154) ✓).

**Borrar una puerta** ✓: la puerta es un bloque de la imagen y se borra como
uno ✓ — el comando `removePort` la quita y se lleva los enlaces que la
tocan ✓, con su deshacer ✓ (medido: la entrada del contenedor 11 borrada
✓, y al salir del contenedor y volver a entrar sigue borrada ✓ ✓).

**El contenedor nuevo nace vacío** ✓: la pregunta del usuario, y la
respuesta es que sí ✓ (ver L10 ✓).

**La composición coloca todas las puertas** ✓, también las que no llevan
flujo todavía ✓: una puerta recién agregada va con las puertas de su misma
clase ✓ (la de entrada con las de entrada ✓, la de salida con las de
salida ✓), una debajo de la otra ✓.

### L9 — La pregunta del contenedor y su respuesta

> "Consideras quitar la opcion de que un Container se cree con un portin y un
> portout? optimizaria y mejoraria el codigo con respecto a simplicidad? porque
> si es asi podemos hacer que un Container se cree sin nada dentro directamente"

**Sí, simplifica** ✓, por el mismo argumento que hizo única la silueta de
los 69 bloques ✓:

- hoy hay **dos caminos** para tener una puerta: la que fabrica el contenedor
  al nacer ✓ y la que el lector agrega ✓. Con el contenedor vacío hay **un
  solo camino** ✓ (el de las clases "Entrada" y "Salida" ✓) ✓;
- el motor no inventa nada ✓: las puertas son las que el modelo declara ✓;
- el lector puede hacer un contenedor **con una sola entrada** ✓ (un
  sumidero ✓) o **con una sola salida** ✓ (una fuente ✓) — con el
  contenedor lleno de puertas, eso no se podía ✓;
- y el modelo **no miente** ✓: un contenedor sin puertas es un contenedor sin
  puertas ✓, y el validador no se queja ✓ (no hay regla que lo pida ✓).

Lo que cuesta ✓: crear un contenedor son **dos pasos más** (agregar la
entrada y la salida ✓). A cambio, el lector sabe exactamente qué puertas
tiene ✓.

### L10 — T6: la puerta nace donde se la suelta (hecho)

El usuario lo dijo así:

> "los bloques de entrada o salida no utilizan el mismo sistema o codigo que los
> otros bloques, porque? porque cuando los coloco 'spawnean' en cualquier lado y
> no donde suelto el mouse. Ademas hay veces (no se como reproducir este bug)
> que cuando muevo otro bloque diferente, por ejemplo un advance) algun bloque
> de entrada o salida se mueve junto a el"

**El análisis a fondo** (la causa raíz es una sola):

1. **La puerta era el único bloque de la imagen cuyo lugar no vivía en el
   modelo** ✓. El motor la creaba sin lugar (`layout: None` ✓), así que la
   app tenía que inventarle uno ✓ — el "hueco libre" ✓ — y ese hueco depende
   de los bloques del ámbito ✓. Por eso **mover un bloque movía la puerta** ✓
   (medido: la puerta estaba en (344, 462) ✓ y al mover el bloque más bajo
   quedó en (281, 656) ✓).
2. **El motor descartaba la posición del mouse** ✓: la rama de las puertas
   del `add_block` ignoraba `x` e `y` ✓, así que la puerta aparecía donde
   fuera ✗.
3. **La app se diferenciaba de los bloques en tres caminos más** ✓: el
   identificador del nodo ("in-1" ✗, una cadena ✗, contra el número de un
   bloque ✓), el lugar ✗ (el de un puerto ✗, contra el de un bloque ✓) y las
   rutas de la app que trabajan con identificadores de bloque ✓ (la
   selección ✓, las propiedades ✓, el portapapeles ✓) ✗.

**El arreglo** ✓: la puerta nace **con el lugar donde el lector la suelta** ✓,
y ese lugar vive en el modelo como el de cualquier otro bloque ✓ (el motor
escribe el `layout` del puerto al crearlo ✓, con la caja de la puerta ✓).
Con eso, la app **deja de inventar lugares** ✓ y la puerta deja de moverse
cuando se mueve otro bloque ✓.

Medido en la app corriendo:

| regla | medida |
|---|---|
| la puerta nace donde se la suelta | el modelo dice `(420, 300, 140 x 44)` ✓, la pantalla `(389, 326)` ✓ |
| mover otros bloques no mueve la puerta | la puerta siguió en `(389, 326)` ✓ al mover dos bloques ✓ |

**Lo que queda de la reutilización** ✓: el identificador del nodo de la
puerta sigue siendo una cadena ✗ ("in-1" ✓), y por eso las rutas de la app
que trabajan con bloques ✗ (la selección ✗, el panel de propiedades ✗, el
portapapeles ✗) la tratan aparte ✓. Cambiar eso es otra unidad ✓: es
cambiar el tipo de la selección ✗ y el del portapapeles ✗ — y el `.gmdl` no
admite que las puertas sean módulos ✗ (son `<input>` y `<output>` ✓, no
módulos ✓), así que la identidad de la puerta tiene que ser de la app ✓.

### L11 — T7: las cifras del contenedor (hecho)

> "El bloque de contenedor debe mostrar las instancias que estan dentro de el y
> las totales, como lo hace el advance por ejemplo"

Un contenedor es un bloque de la imagen ✓ y muestra sus cifras como cualquier
otro ✓: **dentro/pasadas** ✓.

- **Dentro** ✓ = la suma de las cifras de los bloques que el contenedor
  contiene ✓ (una entidad está en un solo lugar ✓, así que la suma es
  exacta ✓).
- **Pasadas** ✓ = la cuenta de los movimientos que cruzaron la frontera del
  contenedor ✓ (los movimientos del trace ✓ los tiene la app ✓).

El contenedor **no tiene fila en el reporte** ✓ (el motor lo compila
"inline" ✓), así que su cifra se **calcula** ✓. Y hay una trampa ✓: el total
del bloque al que apunta la puerta de entrada **no sirve** ✓ — una cola no
tiene total propio en el replay ✓ — y por eso el contenedor mostraba 0/0 ✓;
los cruces del trace sí lo dicen ✓.

Medido en la app corriendo (concurso de pesca, al final del replay ✓):

| bloque | lo que muestra |
|---|---|
| Organizador (el contenedor 11) | **37/41** ✓ |
| Generacion de pescadores (el contenedor 1, sin puerta de entrada) | 3/0 ✓ |
| Espera hasta abrir (un bloque normal) | 0/41 ✓ |

**Lo que falta** ✓: las **puertas** todavía no muestran su total ✓ (la spec
S2 lo pide ✓: "tanto en el contenedor como en la entrada y la salida debe
aparecer las entidades en ese bloque como las que pasaron en total por ahi" ✓).
La cuenta de los cruces ya está ✓ — la puerta de entrada muestra las
entidades que entraron por ella ✓ y la de salida las que salieron ✓ — pero
falta llevarla a la puerta ✓, que tiene su propia forma ✓ y su propia zona
de figura ✓.

### L12 — Los tres problemas que el usuario reportó (análisis)

> "el reordenamiento dentro de un container sigue sin tener en cuenta a los
> bloques de entrada y salida con sus relaciones como lo hace con cualquier
> otro bloque"

> "las instancias que entran a un container no cuentan en su suma de
> instancias (en el bloque de container), es decir, se queda siempore en 0/0"

> "las instancias que salen de un container no se ven animadas con el
> circulito"

El usuario pasó dos capturas del mismo módulo (un submódulo cuyo flujo es sólo
sus puertas: Entrada2→Salida3 y Entrada4→Salida1). Antes de la reorganization
las puertas están donde el autor las puso; después quedan apiladas en una
columna y los enlaces las atraviesan.

**P1 — el reordenamiento no compone las puertas.** La causa está en el motor:
el enlace que va de una puerta a otra puerta del mismo contenedor **vive en
los enlaces del padre** (la convención del formato: un enlace vive en el
ámbito de su ancestro común más bajo ✓), y el flujo del ámbito leía sólo los
enlaces del propio ámbito ✓. El motor nunca vio esos enlaces ✓ → las puertas
no entraron en la composición ✓ → no tenían lugar ✓ → la app las apiló en el
hueco libre ✓, justo en medio de los enlaces que las unen ✓.

**P2 y P3 — los movimientos hacia las puertas se pierden.** La causa está en
la app: `buildTransitions` mapea el bloque del programa al bloque del
modelo ✓, y **un puerto no es un bloque del modelo** ✓ → el destino del
movimiento no se encuentra ✓ → el movimiento no existe ✓. Medido: 287
movimientos en el trace y **ninguno** sale del contenedor 11 ✓. De ahí sale
que la bola no se vea al salir ✓ y que el contenedor cuente 0/0 ✓ (su total de
"pasadas" son los cruces de su frontera ✓, que se cuentan con esos mismos
movimientos ✓).

Lo que **no** era la causa ✓: la app publica su imagen viva cuando cambia el
playhead ✓ (`MonitorView.tsx` ✓, el efecto que publica con el reloj ✓) — las
cifras no están viejas ✓; lo que no hay son los movimientos ✓.

### L13 — T8: el motor compone las puertas que se hablan entre sí (hecho)

**T8 (`ae662ca`)** — el flujo de un ámbito alcanza ahora los enlaces que van
de una puerta a otra del mismo contenedor ✓, y las puertas que esos enlaces
unen se componen como cualquier otro bloque ✓: la puerta de entrada queda
encima de la de salida que alimenta ✓, con el aire del flujo entre ellas ✓.

Los tests del motor (`crates/genmsi-format/tests/layout_scope.rs` ✓):
la puerta de entrada va **encima** de la de salida ✓, no se pisan ✓, y las
puertas de un ámbito **no** están en el hueco libre ✓ (las compone la
composición ✓). El primero falla sin el arreglo ✓ (RED comprobado ✓).

Las puertas que todavía no llevan flujo se apilan **por su clase** ✓ (las de
entrada bajo las de entrada ✓, las de salida bajo las de salida ✓): antes el
conteo mezclaba las dos clases ✗.

### L14 — Lo que queda de esta unidad

- **S2** (que el contenedor y sus puertas muestren cuántas entidades los
  atravesaron) sigue sin hacer: es otra unidad, con `src/monitor/counts.ts` y
  un lugar donde escribirlo.
- **El interior de un contenedor no tiene por qué caber en la caja del
  contenedor**: la caja es la que se ve desde afuera (S5) y el interior es un
  lienzo propio. Cuando el contenido crece, la caja no crece con él: el
  lector ve el contenido fuera de la caja, y es lo que pidió.
- **La puerta que el lector arrastra** se guarda en el modelo ✓, y la
  puerta que acaba de agregar recibe su lugar en el próximo *Reacomodar* ✓
  (o en el momento de agregarla, si el lector quiere ✓: es un detalle
  menor ✓).
