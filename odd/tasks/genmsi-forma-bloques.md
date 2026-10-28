# Forma de los bloques — rediseño visual, catálogo y datos vivos

Rediseño de la forma de cada bloque del lienzo, la pestaña **Bloques** con
mini preview de la forma, y lo que cada bloque muestra mientras la corrida se
reproduce. Primero se diseña en Pen y sólo entonces se implementa: el diseño
es la fuente de la verdad de la implementación.

Rama: `feature/genmsi-forma-bloques`. Estrategia de entrega: `ask-on-risk`.
Pruebas: `cd crates/genmsi-app && node --test tests/*.test.ts` y
`bunx tsc --noEmit`. Revisión visual: `crates/genmsi-app/scripts/check-visual-loop.sh`.

## Specs

### S1 — El alcance es el rediseño visual de cada bloque

> "en esta sesion me encantaria darle un rediseño a cada bloque (como se ve visualmente)"

El alcance es **cómo se ve** el bloque: su silueta, su color y el dato que
muestra. No es un rediseño del motor, del formato `.gmdl` ni de la simulación:
el motor sigue publicando las mismas cifras.

### S2 — La forma identifica el bloque, no sólo el nombre

> "Yo creo que cada bloque debe tener una forma coherente y que lo identifique visualmente."

Cada clase de bloque tiene una **forma** propia y una **familia** de forma
coherente. La coherencia se pide por familia, no clase por clase: el catálogo
tiene 66 clases y 66 siluetas no serían un sistema, serían un ruido. La
forma se lee de un vistazo; el nombre sigue escrito, pero deja de ser la
única vía de reconocimiento.

La referencia de la forma no es inventada: la librería de clases del editor
original (`Genmsi_20230711.backup/gClasses.xml`) ya dibuja una silueta
distinta por clase —elipse para `Generate`, rombo para `Test` y `Transfer`,
cilindro informal para `Enter` y `Leave`, rectángulo con cabecera para
`Table`, elipse con curvas para `FunctionDist`. El rediseño parte de ese
lenguaje y lo ordena en familias.

### S3 — La pestaña Bloques muestra la forma, no sólo el nombre

> "podemos mejorar la pestaña de "bloques" mostrando una mini preview de la forma de ese bloque, esto ayudaria para ver rapidamente la lista e identificar no solo por nombre sino por forma"

La lista del catálogo de la pestaña **Bloques** lleva, por clase, una mini
preview de la silueta y el color de su familia, junto al nombre y al título.
Se reconoce la clase por la forma antes de leer el texto.

### S4 — Cada bloque muestra el dato que le corresponde

> "podemos aprovechar y ver que podemos mostrar en cada uno en la visualizacion del diagrama mientras se simula. por ejemplo ahora todos muestran el dato de cantidade entidades que estan/cantidad de entidades totales que pasaron"

Hoy **todos** los bloques muestran el mismo par: cuántas transacciones hay
dentro y cuántas han pasado. Después del rediseño, cada bloque muestra la
cifra de **la entidad con la que opera**: la cola su contenido, el recurso su
ocupación, el depósito sus unidades, la tabla sus muestras, el interruptor su
valor. El par actual (dentro / pasadas) sigue donde el bloque no opera con
ninguna entidad: las operaciones que sólo reciben y pasan.

La regla de la app se mantiene: **la app no inventa cifras**. Cada dato sale
de lo que el motor publica o de las figuras que el replay reconstruye con la
aritmética del motor; lo derivado se rotula como derivado.

### S5 — Ninguna implementación antes de la aprobación del diseño

> "obviuamente antes de implementar nada tenemos que diseñar bien los modelos en pen como ya veniamos haciendo"

El diseño se hace en Pen (la aplicación abierta) y se muestra al usuario. La
implementación de la app no empieza hasta que el diseño esté aprobado.

### S6 — Una sola fuente de verdad para la forma

La forma de una clase se declara **una vez** en el código de la app y la usan
las dos pantallas que la muestran: el nodo del lienzo y la mini preview del
catálogo. Un bloque del lienzo y su ficha en la pestaña **Bloques** se
dibujan con la misma silueta, o la lista miente.

### S7 — La forma no rompe la geometría del modelo

La silueta se dibuja **dentro** de la huella que el modelo ya trae (`width` y
`height` del módulo), sin mover ni redimensionar bloques: la composición del
lienzo es del motor y no se toca. Un bloque nuevo nace con la huella que la
librería original declara para su clase (el `width`/`height` de
`gClasses.xml`), de modo que un rombo nace cuadrado y un cilindro ancho.

### S8 — Los puertos van donde dice la verdad del editor original

> "Ademas no estas teniendo en cuenta los puntos de input y output que tiene cada componente."

El editor original colocaba cada puerto en un punto concreto de la silueta: la
salida de un `Test` está en el **vértice derecho** del rombo, la de
`Transfer` en el izquierdo, y `Multifurcacion` reparte sus cinco salidas por
el borde inferior. Hoy la app los reparte por igual (`portSpread` en
`src/graph/graph.ts`), y por eso un rombo pierde su salida del lado.

La regla: **la silueta toca el borde donde están sus puertos**. La forma y
el flujo son lo mismo. La verdad de las anclas está en
`odd/tasks/forma-bloques-diseno/puertos.md`, extraída de la librería de
clases del editor original (`gClasses.xml`).

El modelo `.gmdl` no guarda la posición de los puertos de un bloque de
actividad: la guarda el catálogo, que hoy declara el `ioid`, el `name` y el
`caption` de cada puerto pero **no** su ancla. El catálogo tiene que
declararla (T17).

### S9 — El texto no se pisa con nada

> "tanto el texto como la info se ubican de una manera bastante mala"

Todo bloque tiene tres zonas que no se pisan entre sí:

1. la **zona de texto**, a la izquierda: el rótulo y el subtítulo, truncados;
2. la **zona de cifra**, a la derecha: la cifra viva, en monoespaciada y en
   el color de la familia; se reserva aunque no haya cifra, para que el
   rótulo no se mueva cuando aparece;
3. la **zona de puertos**, en el borde: el texto nunca la invade. Un puerto
   es por donde entra o sale el flujo, y el flujo manda.

### S10 — La simulación sigue siendo la misma

El rediseño es de la forma y del rótulo. El motor, la traza, el replay, el
reloj y las cifras no cambian de comportamiento: una simulación da los mismos
números antes y después.

## Tasks

| ID | Specs | Tarea | Ruta | Estado |
|---|---|---|---|---|
| T1 | S1-S10 | Documento de la feature y taxonomía de formas | inline | hecha |
| T2 | S2, S5 | Pen: la biblioteca de formas (once familias, silueta y color) | inline | hecha |
| T3 | S3, S5 | Pen: la pestaña Bloques con mini preview | inline | hecha |
| T4 | S4, S5 | Pen: el bloque del lienzo con su dato vivo por familia | inline | hecha |
| T5 | S2, S6, S7 | Pen: el lenguaje común (colores, tipografía, glifos) | inline | hecha |
| T6 | S1-S10 | Revisión del diseño con el usuario | inline | hecha (aprobado: «Perfecto! ahora si me encanta todo!») |
| T7 | S2, S7, S8 | El contrato de diseño y la verdad de los puertos (`contrato.md`, `puertos.md`) | inline | hecha |
| T8 | S2, S4, S7, S8, S9 | Grupo A (lineal): 24 clases — `Generate`, `Terminate`, `Queue`, `Depart`, las 18 de `Operacion` y `Note` | delegada (`gentle-ai-worker`) | hecha (corregida con el v2) |
| T9 | S2, S4, S7, S8, S9 | Grupo B (entidad): 17 clases — recursos, depósitos, tablas, cadenas | delegada (`gentle-ai-worker`) | hecha (corregida con el v2) |
| T10 | S2, S4, S7, S8, S9 | Grupo C (grupos y decisiones): 25 clases — decisiones, grupos, selecciones | delegada (`gentle-ai-worker`) | hecha (corregida con el v2) |
| T11 | S2, S4, S8, S9 | Auditoría de las reglas: puertos, zonas de texto, cifras, coherencia de familias | delegada (`gentle-ai-verify`) | hecha (10 bloqueantes) |
| T12 | S2, S3, S4, S8, S9 | Integrar las tres propuestas en Pen y revisar la coherencia | inline | hecha |
| T13 | S2, S6, S7, S8, S9 | Código: `src/graph/shapes.ts` (la forma por clase) + tests + el generador que lo produce | delegada (`gentle-ai-worker`) | hecha (`bb8a6e2`) |
| T14 | S2, S8, S9 | Código: el nodo del lienzo dibujado con la forma, las zonas y los puertos en su ancla | delegada (`gentle-ai-worker`) | hecha (`0f52be4`) |
| T15 | S3 | Código: la lista del catálogo a dos columnas, con la miniatura y el título | delegada (`gentle-ai-worker`) | hecha (`e96627b`) |
| T16 | S4 | Código: el dato vivo por bloque (`src/monitor/figures.ts` + el almacén vivo) | delegada (`gentle-ai-worker`) | hecha (`f198658`) |
| T17 | S7 | Motor: la huella de cada clase, y el bloque nuevo que nace con ella | delegada (`gentle-ai-worker`) | hecha (`6da5841`) |
| T18 | S10 | Verificación: la simulación no cambia + la revisión visual | inline | hecha |
| T19 | S1-S10 | Documentación (`docs/app.md`) y el cierre de la feature | inline | hecha (`8cc0198`) |

**Por qué se delega.** La unidadTrigger es el **paralelismo**: las tres
unidades de la primera ronda (la forma, el motor y los datos vivos) tocan
archivos distintos y ninguna depende de las otras dos, asi que se lanzan a
la vez. La segunda ronda (el nodo del lienzo y la lista del catalogo) si
depende de las dos primeras, y por eso va despues.

### El diseño, en Pen

El diseño vive en el lienzo de Pen: `Genmsi.pen`
(`~/.pencil/documents/73873ccb-a24e-4da5-bb93-440c6149b422/Genmsi.pen`), en tres
marcos.

| Marco | Qué muestra |
|---|---|
| **Block Shapes Library** | las once familias: la silueta, el color, las clases y el dato en vivo de cada una. Cada familia aparece en tres huellas (ancha, cuadrada y angosta) para probar que la silueta se adapta. |
| **Bloques Tab Screen** | la pestaña **Bloques** rediseñada: la lista agrupada por familia, con la miniatura de la silueta al lado del nombre, y el campo de búsqueda. |
| **Live Block Screen** | el lienzo con los bloques de cada familia y su cifra, la leyenda de qué dato muestra cada una, y las tres notas de lo que cambia. |

La silueta de cada familia es una geometría paramétrica: se dibuja con la
huella del bloque (ancho y alto), de modo que la misma función sirve para el
bloque del lienzo y para la miniatura de la lista.

### La composicion de un bloque

El bloque tiene tres cosas, y la forma manda sobre las otras dos:

1. **La silueta** de su familia, con el borde en el color de la familia y el
   relleno en ese color al 14 % (14 % en las declaraciones, con el borde
   apagado al 42 % de opacidad).
2. **El rotulo** a la izquierda: la clase y el titulo, en una linea, truncado
   con el espacio de la cifra reservado.
3. **La cifra** a la derecha, en la tipografia monoespaciada y en el color de
   la familia. Es el dato de la entidad con la que opera el bloque.

Un bloque sin dato (aun no lo alcanzo la corrida, o una declaracion) no
muestra cifra: el espacio queda libre y el bloque se lee solo por su forma.

### La lista de la pestaña Bloques

La lista se agrupa por familia, en el orden del flujo (lo que nace, lo que
espera, lo que se toma, lo que se guarda, lo que decide, lo que opera). Cada
fila lleva la miniatura de la silueta —la misma geometria, a escala—, el
nombre de la clase en monoespaciada y el titulo debajo. Un campo de busqueda
filtra por clase y por titulo.

La taxonomía: cada familia tiene una silueta, un color y el dato que
muestra. Las clases del catálogo (`crates/genmsi-format/src/catalogue.rs`)
que no aparecen en la lista son las que se declaran abajo.

| Familia | Silueta | Color | Clases |
|---|---|---|---|
| Fuente | elipse ancha | esmeralda | `Generate`, `Clock` |
| Destino | círculo con anillo | rosa | `Terminate` |
| Decisión | rombo | índigo | `Test`, `Transfer`, `GateF`, `GateL`, `GateM`, `GateS`, `Loop`, `Multifurcacion`, `MultifurcacionP`, `Split` |
| Cola | cajón con remate redondeado y las marcas de quien espera | ámbar | `Queue`, `Depart` |
| Recurso | rectángulo con galón | turquesa | `FacilityUse`, `FacilityInt`, `Seize`, `Release`, `Preempt`, `Return`, `SelectF` |
| Depósito | cilindro | celeste | `StorageDef`, `StorageUse`, `Enter`, `Leave`, `SelectS` |
| Tabla | rectángulo con cabecera | violeta | `Table`, `Tabulate`, `QTable` |
| Cadena | dos cápsulas enlazadas | naranja | `Link`, `Unlink` |
| Grupo | hexágono con elipse | fucsia | `JoinMe`, `JoinNumber`, `RemoveMe`, `RemoveNumber`, `RemoveTrans`, `Assemble`, `Gather`, `Match`, `Scan`, `Alter`, `Examine`, `ExamineNumber` |
| Selección | embudo | cian | `Select`, `SelectSNA`, `SelectL` |
| Operación | rectángulo redondeado | pizarra | `Advance`, `Assign`, `Buffer`, `Mark`, `Counter`, `Priority`, `SaveValue`, `InitiaSV`, `Logic`, `InitialLS`, `Variable`, `VariableN`, `VariableA`, `Function`, `FunctionDist`, `FunctionExpon`, `MFunction`, `MFunctionDist` |
| Anotación | hoja con la esquina doblada | gris | `Note` |

Las **declaraciones** (las quince clases sin puertos: `Clock`, `Function`,
`FunctionDist`, `FunctionExpon`, `MFunction`, `MFunctionDist`, `Variable`,
`VariableN`, `VariableA`, `Table`, `QTable`, `StorageDef`, `InitialLS`,
`InitiaSV`, `Note`) se dibujan con la misma silueta de su familia pero en
**placa apagada** (borde punteado, relleno tenue): no participan del flujo, y
la diferencia se lee de un vistazo.

### El dato vivo por bloque, y de dónde sale

El dato no lo decide la familia sino **la entidad con la que opera el
bloque**, que el modelo declara en las propiedades del bloque. El bloque
muestra su cifra propia; si no opera con ninguna entidad, muestra el par de
siempre.

| Bloques | Dato | Origen |
|---|---|---|
| `Generate` | flechas generadas | el conteo de entradas del generador (la fila que cuenta su bloque) |
| `Terminate` | transacciones terminadas | la fila de `Terminate` del reporte |
| `Queue`, `Depart` | contenido de la cola | las figuras vivas de la cola, por la propiedad `Queue` |
| `Seize`, `Release`, `Return`, `Preempt`, `FacilityUse`, `FacilityInt` | % de ocupación del recurso | las figuras vivas del recurso, por la propiedad `Facility` |
| `Enter`, `Leave`, `StorageUse` | unidades en uso y capacidad | las figuras vivas del depósito y la capacidad del reporte, por la propiedad `Storage` |
| `Tabulate` | nº de muestras | las figuras vivas de la tabla, por la propiedad `NombreTabla` |
| `Link`, `Unlink` | tamaño de la cadena | las figuras vivas de la cadena, por la propiedad `Cadena` |
| `Logic`, `InitialLS` | el valor del interruptor | las figuras vivas de `logic` |
| `SaveValue`, `InitiaSV`, `Counter` | el valor que guarda | las figuras vivas de `values` |
| todo lo demás | dentro / pasadas | el par de siempre: el bloque no opera con ninguna entidad |

**Una sola cifra por bloque.** La información que no cabe en el bloque vive
en el Reporte; el bloque lleva la cifra que lo caracteriza.

**Tres clases que no pueden mostrar la cifra de su entidad.** `SelectF`,
`SelectS` y `SelectL` **no declaran la entidad**: `SelectF` declara `Param`,
`maximo`, `minimo` y `op`, y no hay propiedad `Facility` que lo diga
(`crates/genmsi-format/src/catalogue.rs:1336-1388`). El motor tampoco puede
resolverlo: estas clases **eligen** una entidad —la primera facility libre, un
depósito según su estado— y no publican su nombre
(`crates/genmsi-core/src/program.rs:1897-1935`). Su cifra por entidad es
**irresoluble** sin una propiedad nueva en el modelo, que es un cambio del
motor y del formato, fuera del alcance de este rediseño. Las tres quedan en
«todo lo demás».

**No hay entidades de grupo.** El motor publica `facilities`, `storages`,
`queues`, `chains`, `logic`, `savevalues`, `tables` y `qtables`
(`crates/genmsi-core/src/stat.rs:580-597`): nadie cuenta ni publica los
miembros de un grupo. `Select` y `SelectSNA` quedan en «todo lo demás».

## Log

- L1 — "Hola! en esta sesion me encantaria darle un rediseño a cada bloque (como se ve visualmente). Con esto ademas podemos mejorar la pestaña de "bloques" mostrando una mini preview de la forma de ese bloque, esto ayudaria para ver rapidamente la lista e identificar no solo por nombre sino por forma, que te parece mi idea? Yo creo que cada bloque debe tener una forma coherente y que lo identifique visualmente. Ademas podemos aprovechar y ver que podemos mostrar en cada uno en la visualizacion del diagrama mientras se simula. por ejemplo ahora todos muestran el dato de cantidade entidades que estan/cantidad de entidades totales que pasaron. obviuamente antes de implementar nada tenemos que diseñar bien los modelos en pen como ya veniamos haciendo"
- L2 — La idea es del usuario y el alcance queda fijado por L1. Se la acepta con una corrección de forma: la coherencia se pide por **familia**, no por clase (66 clases, 66 siluetas no serían un sistema). La familia se apoya en el lenguaje que el propio editor original ya usaba (`gClasses.xml`): cada clase del original ya tenía su silueta, y esa evidencia manda sobre una invención.
- L3 — Se descubre que el modelo ya guarda la huella de cada bloque (`width`/`height` del módulo) y que la librería original declara la huella por clase. El rediseño no toca la geometría (S7): la silueta se dibuja dentro de la huella, y un bloque nuevo nace con la huella de su clase.
- L4 — El dato vivo por familia sale de lo que el motor ya publica: el reporte (entradas por bloque, capacidad de los depósitos) y las figuras del replay (colas, recursos, depósitos, cadenas, tablas, interruptores, valores). No hace falta tocar el motor para las cifras; sólo para la huella inicial de un bloque nuevo (T11).
- L5 — El diseño se dibujó en Pen (tres marcos: **Block Shapes Library**,
  **Bloques Tab Screen**, **Live Block Screen**). La primera versión dibujaba
  las siluetas con una geometría fija de 120×48, que se deformaba al cambiar
  la huella; la versión final es **paramétrica**: la misma función produce la
  silueta en la huella que tiene el bloque, y por eso sirve también para la
  miniatura de la lista.
- L6 — En el lienzo, el rótulo y la cifra se pisaban en los bloques angostos
  (un `Test` es de 64×64). La regla final: el rótulo ocupa una línea
  truncada con el espacio de la cifra reservado, y la cifra va a la derecha,
  en el color de su familia.
- L7 — Se corrige una imprecisión del diseño: las **declaraciones** son las
  quince clases **sin puertos** (`Clock`, `Function`, `Variable`, `Table`,
  `Note`, ...), no las que "no participan del flujo" en general. `Logic`
  tiene puerto de entrada y salida, así que no es una declaración: se
  dibuja como operación, con su valor de interruptor.
- L8 — Once familias y no diez: `Selección` (`Select`, `SelectSNA`,
  `SelectL`) es una familia propia, con su embudo. En el editor original las
  cinco clases `Select*` compartían silueta, y `SelectSNA`/`Select` (elección
  entre entidades) se distinguen de `SelectL` (elección de interruptor) por
  la entidad con la que operan.
- L9 — El usuario rechaza el diseño en su estado actual: **"hace falta
  pulir bastante el diseño, ya que tanto el texto como la info se ubican de
  una manera bastante mala. Ademas no estas teniendo en cuenta los puntos de
  input y output que tiene cada componente."** Son dos defectos reales y
  verificables:
  - **El texto**: el rótulo y la cifra se pisan en los bloques angostos, y el
    rótulo se sale de la silueta. Faltaba la regla de las tres zonas (S9).
  - **Los puertos**: la app reparte todos los puertos por igual
    (`portSpread`), pero el editor original los colocaba en puntos concretos
    de la silueta. La verdad de esas anclas está en
    `odd/tasks/forma-bloques-diseno/puertos.md` (S8).
- L10 — La revisión de las formas se **delega por grupos de clases
  similares**, a petición del usuario: **"que te parece si en vez de hacer
  todo vos mismo delegas a varios subagenentes por grupos que tengan una
  similtud para la mejora y pensamiento del diseño?"**. Los grupos se cortan
  por estructura, no por familia de color: A (lineal, 24 clases), B
  (entidad, 17), C (grupos y decisiones, 25). Cada grupo entrega la
  especificación de sus clases (silueta paramétrica, anclas de puerto,
  zonas de texto y cifra) y un SVG renderizado que hay que **mirar**. Un
  cuarto subagente audita las reglas (puertos, zonas, cifras, coherencia de
  familias) para que los tres resultados encajen entre sí.
- L11 — Los subagentes no tienen el MCP de Pen, así que el reparto es: ellos
  **piensan el diseño** (geometría y colocación exactas, con render
  verificable) y el padre lo **integra en Pen** y en la app. El contrato
  (`odd/tasks/forma-bloques-diseno/contrato.md`) es la ley común: sin él,
  los tres grupos no encajarían.
- L12 — La auditoría de las reglas (`odd/tasks/forma-bloques-diseno/auditoria.md`)
  encontró **diez bloqueantes**, y el padre verificó los load-bearing en el
  código antes de aceptarlos:
  - **La cifra viva no tenía formato definido**: el par por defecto `12/40`
    ocupa 30 px, y 24 de las 25 clases del grupo C entregaron zonas de
    8–11 px de alto, que no caben en ningún formato. El contrato v2 §6.3 fija
    el formato (una línea de 30 px mínimo, o dos apiladas de 14×22) y el
    truncado con `…` antes que desbordar.
  - **La familia Decisión no puede ser un rombo**: sólo `Test` y `Transfer`
    lo son. Las anclas reales de las otras ocho caen **fuera** de un rombo
    centrado (hasta 16 px en `Split`), y las de `GateF/L/M/S` caen 6–8 px
    fuera. La familia pasa a ser una **familia de variantes** (contrato v2
    §5.1), con la variante nombrada por clase.
  - **La partición rótulo-izquierda/cifra-derecha no cabe** en el rombo de
    64×64 (caja inscrita de 32×32). El contrato v2 §2.1 lo resuelve: con
    menos de 60 px de ancho útil, el rótulo va arriba y la cifra abajo, en la
    misma columna.
  - **32 px de alto no dan para rótulo y subtítulo** (20 px útiles contra
    22 que hacen falta). El contrato v2 §2.2: por debajo de 40 px de alto,
    sólo rótulo.
  - **`SelectF`, `SelectS` y `SelectL` no tienen cifra resoluble** (no
    declaran entidad; el motor la elige y no la publica). **Mi spec estaba
    equivocada** en esas tres filas: se corrigen a «todo lo demás».
  - **«Miembros del grupo» no lo publica el motor**: no hay entidades de
    grupo. Esa fila cae.
  - **Cinco anclas caen en el interior** de la silueta original (la burbuja
    de `JoinMe`/`JoinNumber` y de `Remove*`): la regla «la silueta toca el
    borde donde está el puerto» necesita **una excepción explícita**, y el
    grupo C ya la había resuelto por su cuenta dibujando la burbuja centrada
    en el ancla.
  - **La app recorta todo nodo a 72×32 como mínimo**
    (`crates/genmsi-app/src/graph/graph.ts:108-109`), así que siete clases
    se dibujan más grandes que su huella de clase: el diseño tiene que
    funcionar en las dos.
  - El contrato y la spec se contradecían en seis filas de cifras: ahora
    hay **una sola fuente**, el contrato v2 §6, y la spec se corrige para
    coincidir.
- L13 — Se acepta la propuesta del grupo A: **`Note` sale de la familia
  Operación a una familia «Anotación»** propia (gris `#64748b`), porque una
  anotación no es un paso del flujo. La familia «Anotación» es la duodécima
  y sólo contiene `Note`.
- L14 — La sesión se interrumpió por falta de memoria: las correcciones de
  los grupos B y C se perdieron a mitad de camino (sus entregables seguían
  con las reglas v1). El grupo A sí quedó corregido y verificado (24 clases,
  las ocho reglas del contrato v2, ninguna zona fuera de su huella). B y C
  se relanzaron con el mismo encargo; no se cambió ninguna regla por el
  incidente, sino que se repitió el trabajo perdido.
- L15 — El padre escribe el integrador (`odd/tasks/forma-bloques-diseno/render.py`),
  que lee los tres entregables y produce **dos** salidas de la misma lectura:
  la hoja de contacto en SVG (para mirar el resultado sin tocar la app) y el
  fragmento de JavaScript para el lienzo de Pen. Así el lienzo y la hoja no
  pueden discrepar: no hay dos fuentes de verdad, y el padre no copia las
  siluetas a mano.
- L16 — Los tres grupos quedan corregidos contra el contrato v2 y el padre
  los verifica con su propio verificador (`render.py`, que comprueba las
  reglas del v2 sobre las 66 clases de golpe: la zona de texto dentro de la
  huella, la zona de cifra de 30 px o 14×22, el texto y la cifra sin
  solaparse, la variante nombrada). **Las 66 pasan.** La hoja de contacto y
  el lienzo de Pen salen de la misma lectura, así que no pueden discrepar.
- L17 — Cuatro marcos en Pen:
  - **Zones and Ports Screen**: las tres zonas y los puertos en su sitio (el
    `Test` con sus tres vértices, el `Queue` con su puerto descentrado, el
    `Multifurcacion` con sus cinco salidas).
  - **Shapes Screen**: las 66 formas en su huella real, con el puerto en su
    ancla verdadera, la zona de rótulo y la de cifra. Cero solapes.
  - **Bloques Tab Screen**: la pestaña **Bloques** con la miniatura de la
    silueta de cada clase, agrupada por familia.
  - **Live Canvas Screen**: el bloque del lienzo con la cifra de su entidad
    (la cola su contenido, el recurso su ocupación, el depósito sus
    unidades, la tabla sus muestras).
- L18 — El grupo C entregó la corrección del v2 y con ella **desaparecen
  las 25 zonas de cifra de 8–11 px**: las 25 son ahora de una línea, de 30 a
  34 px de ancho. Se apilan (rótulo arriba, cifra abajo) las ocho clases
  cuya silueta no da 60 px de ancho útil (`Test`, `Transfer`, las cuatro
  `Gate*`, `Loop` y las tres de Selección); las otras 17 van rótulo a la
  izquierda y cifra a la derecha. `SelectL` ya muestra el par de siempre.
- L19 — **La `Multifurcacion` destapa un defecto del motor.** La clase es de
  80×20, pero la app recorta todo nodo a 72×32 como mínimo
  (`crates/genmsi-app/src/graph/graph.ts:108-109`): el motor compone con los
  20 px guardados y la app dibuja 32, con lo que el bloque se solapa con el
  de abajo. El diseño lo entrega a 80×32, que es **justo lo que se ve en el
  lienzo**, y la corrección es que el motor componga con el mismo tamaño
  que la app dibuja (T17). El original, además, no rotulaba ese bloque: es
  un triángulo plano con sus cinco salidas.
- L20 — El documento de Pen estaba **mal organizado y superpuesto**: las
  cuatro pantallas nuevas se habían creado todas en `x:0, y:0`, encima de las
  del usuario. Se reorganiza el documento entero con un **empaquetado en
  cuatro columnas** (`COLS=4`, `SEP=160`), con las cuatro pantallas del
  rediseño en la primera fila para poder revisarlas de inmediato. **Cero
  solapes** entre los doce marcos, verificado por comparación de cajas.
- L21 — Dentro de los marcos se corrigió lo que se salía: los rótulos
  largos de la pantalla de zonas, la lista del panel **Bloques** (que ahora
  muestra las 66 clases enteras, agrupadas por familia, en vez de cortarse),
  las etiquetas del eje de la pantalla de Gráficos y la última fila de la
  grilla del lienzo. Lo que Pen sigue reportando como "recortado" son los
  **puertos**, que sobresalen 4 px del bloque **a propósito**: el puerto se
  dibuja sobre el borde, que es donde tiene que estar.
- L22 — **El usuario aprueba el diseño con tres correcciones** (verbatim):
  > "perfecto, me gustan todos menos 3:
  >  - el terminate: la forma irregular no me gusta sinceramente, ademas ese
  >    ovalo con forma de pildora no entiendo a que va.
  >  - el multifurcacion: y multifurcacionP el texto se soplapa copn la
  >    forma. la forma me encanta, pero tiene ese problema"
  Se corrigen las dos formas señaladas (la tercera era el mismo defecto de
  `MultifurcacionP`):
  - **`Terminate` pasa a círculo limpio.** La forma anterior (un «cuenco con
    anillo») era irregular y su anillo no significaba nada para quien lee. El
    círculo dice de inmediato «aquí se cierra el flujo», y el puerto de
    entrada conserva su ancla exacta del original (32,0), que en un círculo
    es justo el vértice superior; por eso la huella de la clase pasa de
    84×64 a 64×64.
  - **`Multifurcacion` y `MultifurcacionP` conservan el peine** (que al
    usuario le encanta) y ganan **dos bandas propias de texto** sobre la
    corona, una para el rótulo y otra para la cifra, para que el texto no
    pise ni la forma ni los dientes. El peine crece a 80×40 dibujados: la
    clase es de 80×20 en el catálogo y no cabe texto en 20 px.
  - El fallo de fondo era del **generador**, no del diseño: el rótulo no se
    recortaba, así que «Multifurcacion» (65 px) desbordaba su zona de 36 px
    y pisaba la cifra. El generador ahora **mide el texto y lo recorta con
    puntos suspensivos**, igual que hace la app con `truncate`: la hoja y el
    lienzo de Pen no pueden mentir sobre lo que se verá en la app.
- L23 — Al corregir `Terminate` (64×64) y el peine (80×40) aparecen dos
  huellas dibujadas que **no son las de la clase** en el catálogo: el nodo
  se dibuja más alto de lo que el modelo guarda. Es el mismo caso que la
  `Multifurcacion` de 80×20 (L19): el motor compone con la huella guardada
  y la app dibuja con el mínimo, así que el bloque se solapa con el de
  abajo. La corrección de raíz es una sola, en T17: **el motor compone con
  la huella que la app dibuja**, y el catálogo declara esa huella por clase.
- L24 — **Tercera corrección del usuario** (verbatim):
  > "uno mas que quiero cambiar, el unlink, deberia ser igual al link pero
  > modo espejo, y actualmente no es igual, solo eso"

  Tenia razon, y el editor original lo confirma: `Link` y `Unlink` son la
  **misma figura reflejada** (`gClasses.xml`: `Link` = Rectangle 0,10,71,40 +
  RoundRect 70,0,98,50; `Unlink` = Rectangle 16,10,98,40 + RoundRect
  0,0,28,50). El grupo B los habia dibujado con siluetas distintas.
- L25 — **La primera corrección salió al revés**, y el usuario lo detectó
  (verbatim):
  > "no, asi no, tenias que copiar el diseño que tenia el link y calcarlo
  > modo espejo al unlink. lo hiciste al reves"

  El padre habia **rediseñado `Link`** para que `Unlink` fuera su espejo, en
  vez de **calcar `Link` sobre `Unlink`**: cambio la figura que el usuario
  no habia tocado. La consigna era conservar `Link` tal cual y reflejarlo.
  Se corrige: `Link` vuelve **exactamente** al diseño que tenia, y `Unlink`
  es su calco al espejo (ruta reflejada en `x -> 98 - x`, con el barrido de
  los arcos invertido), con el texto reflejado con la figura.

  La comprobación es **objetiva y rasterizada**: se dibujan ambas siluetas a
  8x y se compara `Link` con el reflejo horizontal de `Unlink` — coinciden
  en el **99,7 % de los pixeles**. Además, las cuatro anclas de `Unlink`
  (`Entrada1` 55,10 · `Salida1` 14,49 · `SalidaAlt` 97,26 · `Salida2` 55,40)
  caen **cada una en el borde** de la silueta calcada, y ninguna zona de
  texto pisa un punto de puerto.

  Queda una desviación declarada: la cifra de `Unlink` va 4 px antes del
  espejo exacto (60 en vez de 64), porque su salida alternativa, en el borde
  derecho, se queda sin sitio si la cifra ocupa la esquina. El `Link` no
  cambia.
- L26 — La correccion destapo un defecto de la regla: el ancho minimo de la
  zona de cifra estaba fijado en 30 px para todas las clases, pero la cifra
  de la cadena es un solo numero («3»). Una regla que obliga a una zona de
  30 px dentro de una capsula de 28 px es una regla que no se puede cumplir.
  El contrato v3 §6.3 fija el ancho minimo **segun el texto de su propia
  cifra** (el ancho del texto mas 2 px de aire): 32 px para el par de cinco
  caracteres, 10 px para un numero suelto. El verificador del padre lo
  comprueba clase por clase.
- L27 — Los **SVG de los tres grupos** se regeneran desde los `.md` con el
  mismo generador que la hoja de contacto (`render.py --grupo`): asi ninguna
  hoja del repositorio puede contradecir a otra, y las correcciones del
  usuario (el círculo del `Terminate`, las bandas del peine, el espejo de
  `Link`/`Unlink` y las zonas de cifra de los depósitos) quedan en las tres
  hojas y no sólo en el lienzo de Pen.
- L28 — **Los colores se suavizan** (verbatim del usuario):
  > "los colores van a ser parte tambien no en la app? si es asi me
  > gustaria que sean colores sean los mismos tonos porque me gustan pero mas
  > claritos. porque? porque quiero que visualmente se note el color pero que
  > sea muy sutil."

  Sí: los colores van en la app — la silueta del bloque lleva el color de su
  familia y la miniatura de la pestaña **Bloques** usa el mismo, asi que el
  suavizado se ve en las dos pantallas.

  El color de cada familia pasa a **tres niveles del mismo tono**, derivado
  mezclando el tono con el blanco en espacio lineal (que conserva el matiz
  al aclarar, donde HSL lo desplaza):

  | Nivel | Para que sirve | Mezcla | Contraste con blanco |
  |---|---|---|---|
  | `acento` | la cifra viva, el titulo de la familia, el punto del puerto | el tono puro | 4,8 a 7,9 |
  | `borde` | el contorno de la silueta | 45 % hacia el blanco | ~1,8 |
  | `relleno` | el fondo del bloque | 90 % hacia el blanco | ~1,1 |

  El borde a 1,8 de contraste **se nota al mirar la forma pero no grita**
  sobre el lienzo, que es lo que se pedia; el relleno es apenas un velo. Lo
  que **no** se suaviza es lo que tiene que leerse: la cifra viva y el punto
  del puerto, que son texto y son el ancla del flujo, y siguen con el tono
  fuerte.

  Los once tonos (y el gris de Anotacion) quedan en el generador, con su
  acento, su borde y su relleno: una sola fuente para la hoja de contacto, el
  lienzo de Pen y, mas adelante, el codigo de la app.
- L29 — El usuario no podia ver como se veria la pestaña **Bloques**
  (verbatim): "en la bloques tab screen no puedo ver como se verian". Al
  buscar el motivo aparecieron **dos defectos del generador**, no del diseño:
  - **Las miniaturas salian vacias**: el generador de la pestaña pedia los
    colores `rl` (relleno) y `bd` (borde) a un diccionario de datos que
    **no los llevaba**, asi que el path salia sin relleno ni borde: un marco
    gris vacio. El generador de la hoja de contacto si los llevaba, y por eso
    ahi las formas se veian.
  - **Faltaba el titulo de cada clase**: la lista mostraba solo el nombre
    (`Generate`). El titulo es lo que dice la clase en palabras («Generar»),
    y el catalogo del motor ya lo declara para las 66
    (`crates/genmsi-format/src/catalogue.rs`), asi que la lista lo toma de ahi
    en vez de inventarlo.
  Corregidos los dos, la lista muestra la miniatura con el color de su
  familia y el titulo debajo del nombre. Verificado: 66 filas, 66 titulos,
  **0 miniaturas sin color**.
- L30 — **La miniatura de la lista se dibuja solo con su contorno**
  (verbatim del usuario): "bien me gustan todos los diseños solo una cosa es
  que el fondo debe ser trannsparente, con la silueta de la figura era
  suficiente (me refiero a la lista de "nñpqies tab screen")".

  En la lista, la miniatura es **solo la silueta**: sin marco de fondo y sin
  relleno, únicamente el contorno en el tono claro de la familia. El bloque
  del lienzo **no** cambia: sigue con su velo de relleno, porque ahi el
  relleno es lo que separa una silueta de otra.

  Al aplicarlo aparecio un detalle de Pen que conviene saber: `fill:'none'`
  **no es un valor valido** (el esquema pide un hex o `enabled:false`), asi
  que el relleno se queda como estaba y la miniatura seguia pintada. La
  forma correcta de dejarlo transparente es **omitir la propiedad `fill`**.
  Verificado sobre el documento: **0 marcos con fondo opaco, 0 miniaturas con
  relleno opaco, 0 miniaturas sin trazo**, y la prueba visual sobre un fondo
  ambar confirma que la silueta no tiene fondo.
- L31 — **La lista pasa a dos columnas** (verbatim del usuario):
  > "actualmente la lista tiene una sola columna, que te parece si hacemos
  > que tenga dos columnas asi uno puede ver mas componentes en menos lugar,
  > te parece buena idea?"

  La idea es buena y se midio antes de aplicarla. El panel de la app es de
  **320 px** (`w-80` en `MonitorView.tsx:318`): en dos columnas quedan ~150 px
  por celda, y tras la miniatura quedan ~110 px de texto.

  | Texto | Ancho máximo | Entra? |
  |---|---|---|
  | nombre de clase (`MultifurcacionP`) | 99 px | si, sin truncar |
  | titulo (`Definir multiples funciones de distribucion`) | 236 px | no |

  Los **66 nombres entran enteros**; solo **3 de los 66 titulos** se
  recortan (`FunctionDist`, `MFunctionDist`, `MultifurcacionP`), y siguen
  distinguiendo las clases. El titulo no es lo que identifica la clase: eso
  lo hace la forma, que es el motivo del rediseño.

  El diseno aplica las dos columnas con el panel a **400 px** (el lienzo
  conserva holgura de sobra) y el texto recortado al ancho real de la celda.
  En la app, esto significa pasar el panel de `w-80` a 400 px.

  Al aplicarlo en Pen aparecio un fallo del generador: el encabezado de
  familia salia **invisible** (usaba `c.color`, y en ese fragmento la clave
  del acento es `c.c`), asi que los grupos de clase se veian como filas
  sueltas. Corregido.
- L32 — **La lista no lleva buscador** (verbatim del usuario): "me gusta, pero
  el buscador no quiero que este (ese "Buscar clase")".

  El buscador era un añadido del padre, no una fila de la app: el catalogo
  de verdad (`ClassList.tsx`) no tiene busqueda. Ademas va en linea con la
  lista a dos columnas: la disposicion es justamente lo que hace que se
  pueda recorrer sin buscar. La lista arranca justo bajo las pestañas.

  En la app, la lista del catalogo conserva su agrupacion por familia y
  suma las dos columnas; no se anade ningun campo de busqueda.
- L33 — El arnés de revisión visual estaba **desactualizado** y se
  destapó al preparar la línea base: `check-visual-loop.sh` buscaba un boton
  llamado «Recientes», pero ese boton es un **icono** con
  `aria-label="Los modelos abiertos últimamente"`, y ademas **queda
  deshabilitado mientras la lista de recientes esta vacia** (los recientes se
  leen del almacenamiento del navegador en el arranque, asi que hay que
  escribirlos **antes** de recargar). Corregido el script: el boton se
  localiza por su etiqueta y la lista se abre desde ahi.

  Con eso el ciclo de revision vuelve a abrir un modelo: la linea base
  (`/tmp/genmsi-baseline/00-lienzo-actual.png`) muestra los **20 nodos** del
  modelo de desarrollo, todos el mismo rectangulo redondeado. Es la foto
  «antes» contra la que se mirara el «despues».
- L34 — **La sesión se interrumpió otra vez por falta de memoria** y los tres
  subagentes de la primera ronda de implementacion fueron **matados por el
  OOM** (no WriteError, no un fallo del trabajo: el sistema se quedó sin
  memoria). Lo que habian escrito en disco estaba completo, asi que se
  verifico y se conservo:

  - `shapes.ts` (T13): 11 tests en verde, suite completa 176/176.
  - `figures.ts` (T16): 24 tests en verde.
  - Motor (T17): `cargo test -p genmsi-format` (63+17+6+5+2) y
    `cargo test -p genmsi-core` (61+3+2) en verde.

  La unidad de la lista del catalogo (T15) no habia escrito nada, asi que se
  relanzo. Dos medidas para que no se repita: los subagentes corren **de uno
  en uno** en las pruebas pesadas, y el padre **no** levanta el servidor ni el
  navegador mientras corren.
- L35 — **La implementación aterriza por partes, verificada por el padre**:
  - T13 (la forma): `shapes.ts` **generado** por
    `odd/tasks/forma-bloques-diseno/generar_shapes.py`, que lee el diseño
    aprobado y **se niega a escribir** si el diseño viola una regla del
    contrato. 11 tests.
  - T16 (el dato vivo): `src/monitor/figures.ts` con `figureOf(bloque,
    fuente)`, que devuelve o la figura de la entidad o el par de siempre, y
    nunca un numero que el motor no publique. 24 tests.
  - T17 (el motor): la huella de cada clase, publicada por el catalogo, y
    el bloque nuevo que nace con ella. `cargo test` en verde.
  - T15 (la lista): dos columnas agrupadas por familia, con la miniatura de
    la silueta (`fill="none"`, el trazo del tono claro) y el titulo. 12
    tests, 212 en total.

  Quedo un resto del worker muerto por el OOM: `live.ts` declaraba las
  figuras **opcionales** en `LivePicture` y **obligatorias** en `LiveState`, y
  el typecheck lo reclamaba. Corregido en el padre: el estado vivo construye
  sus valores en vez de extender el picture, y `tsc --noEmit` vuelve a estar
  en verde.
- L36 — **Las huellas del motor, reconciliadas con el diseño.** El motor
  publicaba la huella **original** de cada clase; el diseno aprobado cambia
  tres. Se compararon las 66 clases de las dos fuentes y solo difieren esas
  tres, asi que se corrigieron en el motor:

  | Clase | Original | Diseno | Motivo |
  |---|---|---|---|
  | `Terminate` | 84x64 | **64x64** | el circulo limpio |
  | `Multifurcacion` | 80x20 | **80x40** | el peine y sus cinco salidas |
  | `MultifurcacionP` | 80x20 | **80x40** | el peine y sus cinco salidas |

  Sin esto, un bloque nuevo nacia con el tamano viejo y la forma nueva se
  estiraba al estirar la caja. Los tres tests del motor que fijaban las
  huellas originales se actualizaron a la regla aprobada, y se **anadio** el
  test que faltaba y que habria detectado el fallo: **el motor da a cada
  clase la huella que su forma dibuja** (`tests/shapes.test.ts`).
- L37 — La revision visual gano un paso: la captura de la **pestana
  Bloques**, que hasta ahora nadie miraba. El guion
  `check-visual-loop.sh` es la fuente de la verdad de
  `docs/app-visual-loop.md`, asi que el doc lleva el mismo paso.
- L38 — **La revision visual, y lo que encontro.** El guion
  `check-visual-loop.sh` es la prueba de la entrega, y estaba **desactualizado
  en tres puntos** (no era el diseno lo que fallaba):

  1. buscaba el boton de recientes por la palabra `ultimamente`, pero el
     boton dice `ultimamente` **con tilde**: nunca lo encontraba y el modelo
     no se abria;
  2. buscaba un boton `Reacomodar` que hoy se llama `Reordenar el grafo en
     capas`;
  3. no miraba la pestana **Bloques**, que es media entrega: se le anadio
     el paso, con su captura, y el doc lleva el mismo paso.

  Y el paso nuevo destapo un **defecto del diseño en el motor**: `/tmp` es un
  tmpfs de 7,8 GB y un typecheck abandonado de un subagente lo llenaba
  entero. Se limpio.

  Con el guion andando, la primera captura del grafo muestra **las formas
  correctas** (el `Terminate` es un circulo limpio, el `Test` un rombo, el
  `Facility` un hexagono, las colores por familia) y **el texto mal**:
  la etiqueta se sale de la silueta a la izquierda, el nombre se corta de mas
  (`Genera...` en vez de `Generate`), la forma se estira a la caja del bloque
  viejo (el `Terminate` sale elipse) y hay un puerto que se dibuja como un
  cuadrado grande. La captura de la **pestana Bloques sale exactamente como
  el diseno**: dos columnas, agrupadas por familia con su color, la
  silueta real de cada clase como miniatura sin relleno, el nombre y el
  titulo.

## Segunda ronda — lo que el lector vio

El lector instaló la app y dijo tres cosas: las figuras se ven **cortadas**
y con el **borde de distinto grosor**, hay clases cuyo **texto se corta**, y
las miniaturas de la lista estan **muy chicas**. La medicion dio la causa:

- La figura se dibujaba **encogida para caber en la caja legacy** que el
  modelo le guardaba. De ahi salia todo: el borde de 2 px en unas figuras y
  de 0,8 en otras (el trazo escalaba con la figura), el texto encogido con
  la figura (el `Note` de un modelo viejo era ilegible) y las figuras que
  parecian cortadas.
- **30 de las 66 clases** tienen el nombre mas ancho que su zona de texto:
  `MultifurcacionP` (81 px de nombre en 72 de zona), `ExamineNumber`,
  `FacilityUse`, `InitialLS`, `Depart`... El nombre se cortaba con elipsis.
- Las miniaturas eran de 40x20.

La regla que queda: **la forma es la verdad**. Se dibuja a su tamano, uno a
uno, sin escalarla ni recortarla; el texto va en una linea o en dos, nunca
cortado; y el generador del diseno **rechaza** cualquier clase cuyo nombre
no quepa en su zona.

El **banco de pruebas** es la idea del lector: un modelo con las 66 clases,
sin conectar, en rejilla, para verlas todas con la misma vista. Lo genera
`crates/genmsi-app/scripts/make-catalogue-model.py` con la sesion del propio
motor, asi que es un modelo valido por construccion (no compila: un GPSS
sin enlaces no es un modelo, y la app lo dice en la linea de estado mientras
dibuja los 66 bloques).

| ID | Specs | Tarea | Ruta | Estado |
|---|---|---|---|---|
| T20 | S2, S8, S9 | La maqueta del texto dentro de la forma | delegada | hecha |
| T21 | S2, S8, S9 | La forma se dibuja a su tamano: ni encogida ni recortada | delegada | hecha (`9919eda`) |
| T22 | S2, S8 | El diseno: la zona de texto de cada clase abriga su nombre | delegada | hecha (`64536bb`) |
| T23 | S3 | Las miniaturas de la lista, de 40x20 a 56x28 | delegada | hecha (`d2dc248`) |
| T24 | S10 | El banco de pruebas: el modelo con las 66 clases | inline | hecha (`c05587b`) |
| T25 | S2, S3 | El aire de la caja: el borde completo, en el lienzo y en la miniatura | inline | hecha (`4bbc3de`) |
| T26 | S3 | La galeria: cinco modelos que corren y cubren las 66 clases | delegada | hecha (`f9aa663`) |
| T27 | S10 | El guion de fotos: una foto real de cada clase | inline | hecha (`134fc96`) |
| T28 | S2, S3, S8 | La ronda de pulido con el lector, componente por componente | inline | hecha |
| T31 | S2, S7, S8 | Diseno: el peine mas ancho, la linea de la tabla, la zona del SelectF y la regla "ninguna zona sobre un trazo" | delegada | hecha (`44cc471`) |
| T32 | S8 | App: el rotulo se achica antes de cortarse y la cifra centrada en su zona | delegada | hecha (`282014b`) |
| T35 | S4 | Diseno: las nueve clases sin cifra posible pierden su zona reservada | delegada | hecha (`44cc471`) |
| T36 | S4 | App: el guion tenue de una zona reservada sin cifra | delegada | hecha (`282014b`) |

- L39 — **La entrega: ocho commits, y la feature se cierra.** Cada commit
  compila y pasa sus tests por si solo, y la tabla de la entrega dice el
  orden de la revision. La **simulacion no cambio**: el diff del motor,
  fuera de los tests, son los dos campos nuevos de la huella y las tres
  cifras del diseno aprobado; la huella solo se lee al crear un bloque y
  al publicar el documento del catalogo, nunca en la simulacion. Las
  cifras de la corrida siguen viniendo del motor, y la app no inventa
  ninguna.

  La **revision visual** pasa entera (17 capturas, ningun paso fallo) y
  las ultimas correcciones fueron de maquetacion, no de diseno: la
  etiqueta se escribia en la caja del bloque en vez de en la caja de la
  forma (por eso se salia de la silueta y por eso el nombre se cortaba
  de mas), y la forma se estiraba a la caja del bloque viejo (por eso el
  `Terminate` salia elipse). Ahora la forma manda: se dibuja en su
  propia huella, con su proporcion, y el texto y los puertos viajan con
  esa misma transformada.
- L40 — **La segunda ronda, medida.** El lector instalo la app y dijo tres
  cosas; la medicion dio la causa de las tres:

  | Queja | Causa medida |
  |---|---|
  | las figuras se ven cortadas | la figura se encogia para caber en la caja legacy del modelo |
  | el borde no tiene el mismo grosor | el trazo escalaba con la figura: 2 px en unas, 0,8 en las encogidas |
  | hay texto que se corta | **30 de 66** clases tienen el nombre mas ancho que su zona |

  La regla que queda: **la forma es la verdad**. Se dibuja a su tamano, uno
  a uno, sin escalarla ni recortarla; el nombre va entero, en una linea o en
  dos, y el corte cae en la mayuscula que empieza una palabra. El generador
  del diseno **rechaza** cualquier clase que no cumpla la regla, y ahora
  tambien cualquier trazo interior que se salga de la silueta (el boton del
  parametro de la multifunclacion asomaba por el borde del peine, y el rayo
  del preempt salia de su galon).

  57 clases caben en una linea y 9 en dos: `InitialLS`, `FunctionDist`,
  `MFunctionDist`, `StorageUse`, `RemoveNumber`, `RemoveTrans` y
  `ExamineNumber` se parten en su mayuscula; los dos peinees
  (`Multifurcacion` y `MultifurcacionP`) se parten dentro de la palabra,
  porque su nombre es una sola palabra y no cabe en un peine de 80 px.

  El **banco** de las 66 clases es la idea del lector y resulto ser el
  instrumento del diagnostico: sin el, las 66 siluetas se veian mezcladas
  con la geometria vieja de los modelos y no se podia judging nada. Con el,
  la causa salio a la vista en una captura.

- L41 — **El borde completo.** El lector vio que las figuras se veian
  cortadas por los cuatro costados, en el lienzo y en la lista. La causa
  era del dibujo, no del diseno: el contorno se dibuja **centrado sobre la
  linea** de la forma, asi que la mitad exterior cae fuera de la caja y la
  caja la recorta. Toda figura cuyo contorno tocara el borde de su huella
  salia con los lados planos, como un trozo de algo.

  El arreglo: la caja del bloque y la de la miniatura dejan **2 px de aire**
  para el borde, y la figura se dibuja uno a uno dentro. El nodo crece 4 px
  (2 por lado) y los puertos se desplazan con el, porque la posicion de un
  puerto es su ancla dentro de la huella.

- L42 — **La ronda de pulido, con las fotos de la app real.** El lector
  recibio una foto de cada clase, tal como la dibuja la app durante una
  corrida, y respondio componente por componente: 52 "perfecto" y siete
  con cambios.

  | Queja | Que se hizo |
  |---|---|
  | el peine no entra en una linea | el peine pasa de 80 a **144 px** y su nombre entra entero |
  | el subtitulo de la tabla se sobrepone con su linea | la linea del encabezado baja (y=20 -> 32) |
  | el texto del SelectF toca su ovalo | la zona se mueve a la derecha |
  | el numero del Terminate no esta centrado | la cifra va **centrada** en su zona |
  | que son los "..." de los bloques | el **rotulo** (el nombre que le puso el usuario) se cortaba porque la zona se habia dimensionado para el nombre de la clase |

  El ultimo caso es el de fondo: la zona de texto la dimensionaba el
  nombre de la clase (corto y fijo) y ahi iba tambien el rotulo del
  bloque, que es del usuario y puede ser largo. Ahora el rotulo **se
  achica antes de cortarse** (hasta 7 px) y solo se corta con elipsis si
  no entra ni asi.

  Y con el criterio que el lector aprobo ("mostrar la cifra cuando el
  motor la publica, nunca inventar una, y no dejar un hueco que parezca
  un error"):

  - las **nueve clases que nunca pueden tener cifra** (las cinco
    funciones, la Note, el StorageDef, la Table y la QTable) **pierden la
    zona reservada**: un hueco que nunca se llena no es un hueco, es un
    fallo con apariencia de fallo. El generador lo impide.
  - una zona reservada **sin cifra muestra un guion tenue**, y solo
    mientras la corrida corre: antes de correr, no hay nada que decir.

  El generador creció otra regla: **ninguna zona de texto cruza un
  trazo interior**. La encontro en trece clases (las tres senaladas y
  otras doce), y las trece se arreglaron moviendo la zona, sin tocar
  silueta, puerto ni color.

  La **galeria** (cinco modelos que corren y cubren las 66 clases) y el
  **banco** (el modelo con las 66 clases sin conectar) son el banco de
  pruebas del diseno; el guion `shoot-classes.sh` toma la foto de cada
  clase desde la app.

## Entrega — las unidades de revisión

La rama `feature/genmsi-forma-bloques` nace de `feature/genmsi-reordenamiento`,
así que **arrastra los 38 commits de la feature anterior**; el trabajo de
esta feature empieza en `73ecbb6` (el límite de la rebanada). Cada commit
compila y pasa sus tests por sí solo.

| Commit | Unidad |
| --- | --- |
| `73ecbb6` | la especificación de la feature |
| `4a7469a` | el límite de la rebanada de esta feature |
| `6da5841` | el motor: la huella de cada clase (`crates/genmsi-format`, `crates/genmsi-core`) |
| `f198658` | la figura viva de cada bloque (`src/monitor/figures.ts`, el almacén vivo) |
| `bb8a6e2` | la forma de cada clase (el diseño, el generador y `src/graph/shapes.ts`) |
| `0f52be4` | el nodo del lienzo dibujado con la forma (`src/graph/node.ts`, `GraphView.tsx`) |
| `e96627b` | la lista del catálogo a dos columnas (`src/graph/catalogue.ts`, `ClassList.tsx`) |
| `8cc0198` | el guion de la revisión visual y la documentación |
| `c05587b` | el banco de pruebas: el modelo con las 66 clases |
| `64536bb` | el diseño: el nombre de cada clase, escrito entero |
| `9919eda` | la app: la figura se dibuja a su tamaño |
| `d2dc248` | las miniaturas de la lista, de 40x20 a 56x28 |
| `4bbc3de` | el aire de la caja, para que el borde no se recorte |
| `f9aa663` | la galería: cinco modelos que corren y cubren las 66 clases |
| `134fc96` | el guion de fotos: una foto real de cada clase |
| `44cc471` | el diseño: las zonas fuera de los trazos y sin huecos muertos |
| `282014b` | la app: el texto que no se corta antes de tiempo y la cifra centrada |

**El orden de la revisión.** El motor va antes que la app porque la app
dibuja con la huella que el motor publica; la forma va antes que el nodo
porque el nodo la dibuja; y el guion de la revisión visual va al final
porque es el que demuestra todo lo demás.
