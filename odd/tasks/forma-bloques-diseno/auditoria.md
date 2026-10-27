# Auditoría de las reglas geométricas del rediseño de formas de bloque

Auditoría **previa a la implementación**: se revisan las *reglas* (el contrato
que siguen los tres grupos y contra el que se van a medir), no el resultado de
cada grupo. Lo que ya está entregado se usa como **síntoma** de los huecos de
las reglas, no como objeto de revisión.

**Fuentes leídas**

| Fuente | Qué aporta |
|---|---|
| `odd/tasks/forma-bloques-diseno/contrato.md` | las reglas (§2 zonas, §3 silueta, §4 puertos, §5 familias, §6 cifra viva, §7-8 entrega) |
| `odd/tasks/forma-bloques-diseno/puertos.md` y `puertos.json` | la verdad de las anclas (porcentajes redondeados y píxeles exactos) |
| `odd/tasks/genmsi-forma-bloques.md` (hasta `## Log`) | la especificación de la feature: S1-S8, la taxonomía y la tabla «El dato vivo por bloque» |
| `Genmsi_20230711.backup/gClasses.xml` | la geometría que el editor original dibujaba (66 clases, 131 puertos) |
| `crates/genmsi-format/src/catalogue.rs` | las 66 clases, sus puertos y sus propiedades |
| `crates/genmsi-app/src/components/GraphView.tsx`, `src/graph/graph.ts` | cómo se dibuja hoy (el `spread` y el `portSpread`) |
| `crates/genmsi-app/src/monitor/replay.ts`, `stats.ts`, `src/components/charts/entities.ts`, `src/monitor/derived.ts`, `src/format.ts` | de dónde sale (o no sale) cada cifra viva |
| `crates/genmsi-core/src/stat.rs`, `src/sim.rs`, `src/program.rs`, `src/report.rs` | lo que el motor publica y lo que el modelo declara |
| `crates/genmsi-app/src/types/generated/` | los tipos del reporte y de las entidades |
| `odd/tasks/forma-bloques-diseno/grupos/A-lineal.md`, `grupos/C-grupos.md` | dos entregas parciales, usadas como síntoma |

**Método.** Para cada clase se extrajeron del XML original las primitivas
(`Rectangle`, `Ellipse`, `Polygonal`, `RoundRect`) y los puertos (`Input`,
`Output`), y se comprobó geométricamente si cada ancla cae en el **borde** de
la silueta original, en su **interior**, o **fuera** de ella; y si el borde de
la silueta que implica la familia del contrato pasa por esa ancla. Los números
de píxel salen de la huella de la clase normalizada a 0-1, tal como la regla
§4 del contrato exige.

**Convenciones de medida usadas en todo el informe**

- Huella dibujada: la app recorta el nodo a un mínimo de 72×32
  (`crates/genmsi-app/src/graph/graph.ts:108-109`), de modo que la caja real
  es `max(w,72) × max(h,32)`.
- Punto de puerto: el contrato §4 pide 8-10 px con anillo de 2 px → 12 px de
  diámetro → **6 px dentro del borde** en cada lado con puerto.
- Rótulo: 9-10 px → **7 px por carácter**; dos líneas = **22 px** de alto.
- Cifra: monoespaciada de 10 px → **6 px por carácter**; el par por defecto
  `12/40` = 5 caracteres = **30 px** (más 6 px de aire → zona de 36 px).

---

## Bloqueantes

Ordenados por gravedad: primero lo que rompe la medición de todas las clases,
después lo que rompe familias enteras.

### B1 — La cifra viva no tiene formato definido, y 25 de las 49 clases ya entregadas no pueden mostrar la cifra por defecto

**Evidencia.** El contrato §6 asigna a «todo lo demás» el par *dentro /
pasadas*, que es `12/40`: 5 caracteres monoespaciados = 30 px. El contrato no
dice si ese par va en una línea (30 px de ancho) o apilado en dos (12×22 px),
ni cuál es el ancho mínimo de la zona de cifra. Medidas las entregas parciales
(`grupos/A-lineal.md`, `grupos/C-grupos.md`, campos `figure:`):

| Entrega | Zona de cifra que no cabe en ninguna forma |
|---|---|
| `A-lineal.md` | `Terminate` 14×20 (ni 30 px de ancho ni 22 px de alto) |
| `C-grupos.md` | 24 de 25 clases: `Test`/`Transfer` 28×10, `GateF/L/M/S` 22×8, `Loop` 26×11, `Multifurcacion(P)` 24×9, `Split` 24×11, `JoinNumber` 24×11, `Remove*` 24×11, `Assemble` 22×11, `Gather` 22×9, `Match`/`Scan` 24×11, `Examine` 22×10, `ExamineNumber` 17×10, `Alter` 22×11, `Select`/`SelectSNA`/`SelectL` 24×10 |

En el grupo C **la zona de cifra es de 8 a 11 px de alto**: no cabe ni una
línea de 11 px ni dos apiladas de 22 px. El grupo A sí lo resolvió por su cuenta
(«si no cabe en una línea se apila en dos líneas dentro de la misma zona»,
`A-lineal.md` cabecera), pero el grupo C no, y ambos miden contra el mismo
contrato.

**Resolución concreta.** El contrato §6 debe fijar, antes de que los grupos
sigan: (a) el formato del par — dos líneas apiladas `12` sobre `40`, con la
zona de cifra de **12×22 px como mínimo**, o una línea de 30 px; (b) el ancho
mínimo de la zona de cifra (36 px si la cifra es de una línea, 12 px si es
apilada); (c) que la cifra de una línea se **trunca** si no cabe, en vez de
desbordarse. Es una corrección de tres líneas en §6.

### B2 — La familia Decisión no puede ser un único rombo: 8 de sus 10 clases

**Evidencia.** El rombo de la familia tiene sus vértices únicamente en
(50 %, 0 %), (0 %, 50 %), (100 %, 50 %) y (50 %, 100 %). Las anclas reales
(`odd/tasks/forma-bloques-diseno/puertos.md`) y su distancia al borde del
rombo de la huella:

| Clase | Ancla | Distancia al borde del rombo | Qué exige el original |
|---|---|---|---|
| `GateF` 112×48 | Entrada1 (38,0) | **+7,7 px fuera** | rombo 0-76 px + accesorio a la derecha |
| `GateL/M/S` 102×48 | Entrada1 (38,0) | **+6,1 px fuera** | rombo 0-76 px + accesorio |
| `Loop` 96×32 | Entrada1 (57,0) | **+3,0 px fuera** | cuerpo 18-96 px + punta a la izquierda |
| `Multifurcacion(P)` 80×20 | Salida1..Prob5 (0,20)…(80,20) | **+10 px / +5 px fuera** (4 de 5) | triángulo de base plana |
| `Split` 88×32 | Salida1 (0,32), Salida2 (88,32) | **+16 px fuera** | caja con horquilla inferior |
| `Test`, `Transfer` 64×64 | (32,0), (64,32), (0,32) | 0 (vértices) | rombo |

Sólo `Test` y `Transfer` son un rombo. Los otros ocho necesitan una silueta
distinta, y el grupo C ya lo resolvió por su cuenta con **cinco variantes**
dentro de la misma familia (`C-grupos.md`: rombo, rombo+accesorio, cuerpo con
punta izquierda, peine de la multifurcación, horquilla del split).

**Resolución concreta.** La regla «Decisión = rombo» del contrato §5 y de la
tabla de la spec no puede ser la regla de implementación. Sin tocar la
taxonomía (siguen siendo once familias), el contrato debe declarar que **la
familia es una familia de variantes** y que cada clase nombra su variante, y
que el contrato §3 regla 1 se cumple **contra la variante que nombra la
clase**, no contra un rombo ideal.

### B3 — La partición «rótulo a la izquierda, cifra a la derecha» del contrato §2 es imposible en la familia Decisión

**Evidencia.** El rombo de 64×64 tiene una caja inscrita de **32×32** (36×32
dibujado, por el mínimo de 72 px de `graph.ts:108-109`). Con la cifra por
defecto (36 px) y el rótulo (28 px para «Test») no caben lado a lado: 28+36 =
64 px contra 32 px disponibles. Lo mismo en las otras ocho clases de la
familia: la caja inscrita del rombo de `GateF` es 56×24, de `Loop` 48×16, de
`Multifurcacion` 40×16, de `Split` 44×16. El grupo C ya lo incumplió y lo
declaró: en `Test` y `Transfer` la cifra quedó **debajo** del rótulo
(`figure: 18,38,28,10`), y en `Clock` y `FunctionDist` el grupo A hizo lo mismo
(`A-lineal.md`, «Desviaciones declaradas»).

**Resolución concreta.** El contrato §2 necesita una excepción explícita: «en
la familia Decisión (y en las siluetas de ancho útil menor que 60 px) el rótulo
ocupa la banda superior y la cifra la inferior, en la misma columna». Además,
la cifra de `Test`/`Transfer` no puede ser el par en una línea (28 px de zona
< 30 px): o se apila, o la cifra de esa familia es un número suelto.

**Sobre el tamaño de la huella.** No hace falta cambiar la huella para
resolverlo, y cambiar la huella **no** alteraría la geometría guardada: el
modelo guarda `width`/`height` por bloque (`crates/genmsi-format/src/model.rs`,
`Layout`) y el motor no los reescribe (`crates/genmsi-format/src/layout.rs:412`
usa el tamaño guardado). Un bloque nuevo nace hoy con 120×48 fijos
(`crates/genmsi-format/src/edit.rs:24-25,407-408`) y la tarea T11 los fija
según la clase. Por tanto: **cambiar la huella de clase sólo afecta a los
bloques nuevos**; los ya guardados conservan la suya, y el diseño tiene que
funcionar en las dos. La única clase que *necesitaría* una huella mayor para
mostrar rótulo + cifra completa es `Test`/`Transfer` (el rombo necesitaría
≥104 px de ancho, contra los 64 px de la clase) y, si se acepta, **sólo para
los bloques nuevos**; los existentes se degradan (una línea, sin subtítulo).

### B4 — La cifra de `SelectF`, `SelectS` y `SelectL` no se puede resolver: el modelo no declara la entidad

**Evidencia.** El contrato §6 y la spec asignan a estas clases la cifra del
recurso, del depósito y del interruptor «por la propiedad `Facility` /
`Storage` / `LogicSwich`». Pero:

- `SelectF` (`catalogue.rs:1336-1352`) declara `Param`, `maximo`, `minimo`,
  `op`: **ninguna propiedad de recurso**.
- `SelectS` (`catalogue.rs:1372-1388`): `Param`, `maximo`, `minimo`, `op`:
  **ninguna propiedad de depósito**.
- `SelectL` (`catalogue.rs:1354-1370`): `Param`, `maximo`, `minimo`, `op`:
  **ninguna propiedad de interruptor**.
- El motor no puede resolverlo tampoco: estas clases **eligen** una entidad,
  no la nombran (`crates/genmsi-core/src/program.rs:1897-1935`: `SelectF` toma
  la primera facility libre, `SelectS` un depósito según su estado, `SelectL`
  una llave según su estado), y el motor no publica ningún nombre de entidad
  para los bloques `Select` (`crates/genmsi-core/src/sim.rs:2377-2391`).

**Resolución concreta.** El contrato §6 debe sacar a estas tres clases de la
tabla de cifras por entidad y dejarlas en «todo lo demás» (dentro / pasadas),
declarando que la cifra por entidad de una selección **no es resoluble sin
que el modelo declare la entidad** (un cambio del motor y del formato, fuera
del alcance del rediseño). Si el padre quiere la cifra por entidad, hace
falta antes una propiedad nueva en el modelo.

### B5 — El contrato §6 y la tabla «El dato vivo por bloque» de la spec se contradicen en seis filas

**Evidencia.** `contrato.md` §6 frente a `odd/tasks/genmsi-forma-bloques.md`:

| Filas | Contrato §6 | Spec | Efecto en la zona de cifra |
|---|---|---|---|
| `Queue`, `Depart` | «contenido de la cola» | «contenido **y entradas** de la cola» | 6 px (`3`) frente a 30 px (`3/12`) |
| Recurso (7 clases) | «% de ocupación» | «ocupado/libre **y utilización**» | 18 px (`40%`) frente a 30 px (`1/40%`) |
| `Tabulate` | «nº de muestras» | «muestras **y media**» | 12 px (`12`) frente a 36 px (`12/3,5`) |
| `Link`, `Unlink` | «tamaño de la cadena» | «tamaño **y entradas**» | 6 px (`3`) frente a 30 px (`3/12`) |
| `Table`, `QTable` | **no están** | «muestras y media de la tabla» | — |
| `Select`, `SelectSNA` | **no están** | «miembros del grupo y los elegidos» | — |

El contrato se declara «la ley del rediseño» en su introducción, pero difiere de la spec que el usuario aprobó; los
grupos se miden contra el contrato y diseñan contra la spec.

**Resolución concreta.** Una sola fuente: o el contrato adopta las seis filas
de la spec, o la spec se corrige. Son seis líneas; hay que decidirlas **antes**
de que los grupos dibujen, porque cada cifra decidida cambia el ancho de la
zona reservada y, con él, el ancho del rótulo.

### B6 — La cifra «miembros del grupo y los elegidos» de `Select` y `SelectSNA` no la publica el motor

**Evidencia.** Las entidades que publica el motor son `facilities`,
`storages`, `queues`, `chains`, `logic`, `savevalues`, `tables` y `qtables`
(`crates/genmsi-core/src/stat.rs:580-597`; tipos en
`crates/genmsi-app/src/types/generated/Entities.ts`). **No hay grupos de
transacciones**: el motor no cuenta ni publica los miembros de un grupo. El
replay tampoco los lleva (`crates/genmsi-app/src/monitor/stats.ts`, la
estructura `Figures` tiene `queues`, `facilities`, `storages`, `chains`,
`tables`, `logic`, `values`).

**Resolución concreta.** La fila de la spec debe caer (y `Select`/`SelectSNA`
quedan con el par por defecto, que es lo que ya dice el contrato §6), o el
motor tiene que publicar los grupos: eso es un cambio del motor, no del diseño
visual.

### B7 — Cinco anclas caen en el **interior** de la silueta original: la burbuja centrada en el ancla

**Evidencia.** Comprobación punto-en-unión sobre las primitivas del XML: de
los **131** anclas del catálogo, 126 caen en el borde de la silueta original y
**5 caen en su interior**:

| Clase | Ancla | Por qué |
|---|---|---|
| `JoinMe`, `JoinNumber` | Entrada1 (50,7) | es el **centro** de la elipse `43,0,57,14` de 100×50 |
| `RemoveMe`, `RemoveNumber`, `RemoveTrans` | Salida1 (50,43) | es el **centro** de la elipse `43,36,57,50` de 100×50 |

Es decir: en el editor original, esos puertos son un punto **dentro** de una
burbuja, no un punto del borde. El contrato §3 regla 1 («la silueta toca el
borde donde están sus puertos») es **insatisfacible** para esas cinco anclas.
El grupo C lo resolvió dibujando la burbuja **centrada en el ancla**
(`C-grupos.md`, `JoinMe`: «la burbuja se dibuja centrada en el ancla para que
el puerto quede dentro de la figura y no flotando»), lo que es honesto con la
verdad pero incumple la regla del contrato que lo mide.

**Resolución concreta.** El contrato necesita una cláusula de excepción
explícita: «cuando el ancla del original cae en el interior de una burbuja, la
burbuja se dibuja centrada en el ancla y el puerto queda dentro de ella; es la
única excepción a la regla 1». Y la regla 2 del §2 («el texto nunca invade el
borde donde hay un puerto») debe decir qué pasa con un puerto **dentro** de una
burbuja: el texto no puede taparlo.

### B8 — Las salidas de `Scan`, `Examine` y `ExamineNumber` caen en el interior del hexágono de la familia Grupo

**Evidencia.** Las anclas (25 %, 80 %) y (75 %, 80 %) de `Scan` y (25 %, 83 %) y
(75 %, 83 %) de `Examine`/`ExamineNumber` están, en el original, **sobre la
arista de la cuña** (el triángulo `0,30 – 80,30 – 40,48` y el `0,28 – 80,28 –
40,50`). En un hexágono como el de la familia Grupo (`Alter`: `0,25 – 10,0 –
70,0 – 80,25 – 70,50 – 10,50`) esos puntos quedan **dentro** de la silueta: el
punto flotaría en el aire o sobre el relleno.

**Resolución concreta.** La familia Grupo necesita la variante «caja sobre
cuña» que el grupo C ya dibujó (`Scan`: `M0 6 L6 0 H74 L80 6 V30 L40 50 L0 30
Z`, con las dos rectas pasando por (20,40) y (60,40)). Igual que en B2: la
regla de la familia debe ser «variantes», no «un hexágono».

### B9 — 32 px de alto no alcanzan para rótulo **y** subtítulo con los puntos de puerto que pide el contrato

**Evidencia.** Con el punto de 8-10 px + anillo de 2 px del contrato §4 (12 px
de diámetro, 6 px dentro del borde), un bloque de 32 px de alto tiene
**20 px** de interior utilizable (32 − 6 − 6). El contrato §2 pide dos líneas
de texto (el nombre de la clase «y, debajo, el subtítulo»): a 9-10 px con
interlineado de 11 px son **22 px**. No caben, por 2 px.

Afecta a 15 clases con puerto arriba y abajo: `Advance`, `Assemble`, `Buffer`,
`Depart`, `Enter`, `Gather`, `Leave`, `Logic`, `Loop`, `Preempt`, `Queue`,
`Release`, `Return`, `Seize`, `Split` (y `Multifurcacion`/`MultifurcacionP`,
de 20 px de huella).

**Resolución concreta.** El contrato debe fijar la regla tipográfica, como
hizo el grupo A por su cuenta («10,5 px; si no cabe, 9,5 px; si tampoco, se
trunca con `…`», `A-lineal.md`): con 9,5 px e interlineado de 10 px, dos
líneas son exactamente 20 px y caben; con 10,5 px no caben. Sin esa regla,
cada grupo mide el alto distinto y las 15 clases no son comparables.

### B10 — La familia Fuente y la familia Destino no pueden con una elipse o un círculo centrado

**Evidencia.** El contrato §5 y la spec describen Fuente como «elipse ancha» y
Destino como «círculo con anillo»:

- `Generate` 120×63, `Salida1` (33,63): con una elipse **centrada** en la
  caja, ese punto está **3 px por debajo del arco** → el punto de puerto
  flota fuera de la silueta. En el original la forma es un círculo
  `0,0,63,63` unido a un cuerpo `33,35,120,63`, y (33,63) es la **esquina
  inferior izquierda del cuerpo**.
- `Terminate` 84×64, `Entrada1` (32,0): con un círculo centrado en la caja, el
  punto está **0,9 px por encima del arco**. En el original la forma es un
  círculo `0,0,64,64` con un cuerpo `35,16,84,48`, y (32,0) es el **vértice
  superior del círculo** (el círculo llega hasta x=64, no hasta 84).

**Resolución concreta.** La descripción de las familias tiene que decir
«cabeza redonda + cuerpo» (que es lo que dibujó el grupo A: «círculo de
arranque unido a la caja de emisión»), no «elipse ancha». Sin ese cambio, la
regla medida y la silueta dibujada no pueden coincidir.

### B11 — La silueta que el grupo C dibujó para los «gate» ya no es el rombo de la familia, y eso es lo correcto

No es un bloqueante en sí (es la evidencia de B2), pero conviene que quede
registrado: `GateF` se dibuja como `M0 24 L38 0 L76 24 L38 48 Z M76 24 L94 12
L112 24 L94 36 Z` — un rombo de 76 px de ancho **descentrado** más un
accesorio. Sus anclas (38,0), (38,48) y (0,24) caen exactamente en los
vértices ✓. El accesorio a la derecha es además la **zona de la cifra** (22×8
px). Es una solución buena; lo que falta es que el contrato la permita y la
describa, porque la medida se hará contra el contrato.

---

## Advertencias

| # | Hallazgo | Evidencia | Qué conviene |
|---|---|---|---|
| A1 | La cifra más ancha que el motor puede imprimir es de **7 caracteres**, no de 5: `number()` devuelve el entero tal cual hasta 1e12 (`crates/genmsi-app/src/format.ts:12-17`), así que un conteo de 1 234 567 sale `1234567` = 42 px. Y el `1.842` del enunciado no existe en la convención de la app: la app escribe `1,842` (coma decimal). | `format.ts:12-17` | Fijar en el contrato el formato de la cifra (abreviar: `1,2 M`) y el ancho máximo de la zona (48 px si el conteo no se abrevia). |
| A2 | La huella **dibujada** no es la huella de la clase: la app recorta el nodo a 72×32 (`graph.ts:108-109`). Siete clases se dibujan más grandes que su clase: `Clock` (50×50→72×50), `FunctionDist` (64×64→72×64), `Test`/`Transfer` (64×64→72×64), `Multifurcacion(P)` (80×20→80×32), `StorageDef` (100×30→100×32). | `graph.ts:108-109` | Quitar el recorte o declarar el tamaño dibujado en la entrega; si no, el SVG de la entrega no coincide con el lienzo y S6 («una sola fuente de verdad») se rompe. |
| A3 | El mapa que resuelve la entidad de cada bloque (`ENTITY_PROPERTY`, `crates/genmsi-app/src/monitor/replay.ts:312-324`) cubre **10 clases**; el contrato §6 asigna cifra por entidad a 13 más (`FacilityUse`, `FacilityInt`, `StorageUse`, `Tabulate`, `Logic`, `SaveValue`, `InitiaSV`, `Counter`, `Table`, `QTable` y las tres `Select*`). Además, `counts_block` (una sola fila por bloque) no dice **qué fila** lleva la entidad de un bloque que se compila en varias (`FacilityUse` → Queue+Seize+Depart+Advance+Release). | `replay.ts:312-324`, `counts.ts:33-46`, `program.rs:1319,1442,1530` | Definir la regla de resolución (propiedad del modelo + fila del programa) antes de implementar T10. |
| A4 | No existe formateador de porcentaje ni de booleano: `decimals()` (`format.ts:22-27`) no tiene `%`, y la llave lógica es un booleano (`Entities.ts`: `logic: { [key in string]: boolean }`). «% de ocupación» puede ser `40%` (3) o `40,3%` (5) — 5 caracteres son 30 px. | `format.ts`, `Entities.ts` | Fijar el formato exacto de cada cifra en el contrato §6 (es la fuente del ancho de la zona). |
| A5 | `InitiaSV` e `InitialLS` son declaraciones: no se ejecutan nunca (`program.rs:1176` las compila a `None`), así que su cifra no puede ser «viva»: las figuras vivas de `values`/`logic` están vacías. Su valor es el **final** del reporte (`Entities.ts`: `savevalues`, `logic`) o la propiedad `Valor` del modelo (`catalogue.rs:913-922`, `catalogue.rs:902-912`). | `program.rs:1176`, `catalogue.rs:902-922` | Corregir el origen que dice el contrato §6 para estas dos clases: valor inicial del modelo o valor final del reporte, no «figuras vivas». |
| A6 | La figura del generador contradice la regla vigente de la app: hoy los generadores **no tienen figura** (`counts.ts:38-42`, `MonitorView.tsx:109-115`). El dato sí existe: el evento `generate` cuenta una entrada por transacción creada (`sim.rs:1002`) y la fila del generador del reporte trae esa cuenta. | `counts.ts:38-42`, `sim.rs:1002` | Es un cambio de comportamiento consciente (T10 lo cubre): «el generador no tiene transacciones dentro, pero sí flechas generadas». |
| A7 | La cifra de `Terminate` no puede ser «la fila del reporte» en vivo: la fila trae el valor **final**, y en vivo el número sale del replay (`totals` del bloque, alimentado por el evento `enter`; `terminate()` sólo decrementa el `current`, `sim.rs:2356-2365`). Mostrar la fila del reporte mientras el reloj va por la mitad muestra el futuro. | `sim.rs:1158-1168`, `sim.rs:2356-2365` | Que el contrato diga «en vivo, del replay; al final, del reporte». |
| A8 | La familia **Grupo** (12 clases) tiene **7 siluetas originales distintas**: hexágono + burbuja a la derecha (`Alter`), cuerpo hexagonal + burbuja arriba (`JoinMe`, `JoinNumber`), cuerpo con esquinas achaflanadas + burbuja abajo (`RemoveMe`, `RemoveNumber`, `RemoveTrans`), rectángulo con horquilla (`Assemble`), rectángulo con X (`Gather`), rectángulo con cuña (`Scan`, `Examine`, `ExamineNumber`) y rectángulo con mejilla (`Match`). «Hexágono con elipse» sólo describe `Alter`. | XML original, filas `JoinMe`…`ExamineNumber` | Declarar las variantes de la familia (o su excepción), como en B2. |
| A9 | La familia **Operación** (19 clases) tiene **12 siluetas originales distintas**: rectángulo (`Advance`), rectángulo con línea (`Buffer`), placa con pestaña (`Assign`), dos rectángulos redondeados (`Mark`), paralelogramo (`Counter`), bandera en T (`Priority`), cajón con lengüeta (`SaveValue`, `InitiaSV`), caja con accesorio (`Logic`, `InitialLS`), bandera (`Variable`, `VariableA`, `VariableN`), cuadrado con curva (`Function`, `FunctionExpon`, `MFunction`), elipse con curvas (`FunctionDist`, `MFunctionDist`) y hoja con esquina doblada (`Note`). «Rectángulo redondeado» sólo describe a `Advance`. Y contradice la propia S2, que nombra «elipse con curvas para FunctionDist» como lenguaje que el rediseño **ordena**, no borra. | XML original; `odd/tasks/genmsi-forma-bloques.md` S2 | O la familia admite variantes, o S2 (la forma viene del original) queda sin sostén en 9 clases. |
| A10 | La familia **Recurso** (7 clases) tiene 7 siluetas originales: galón a la derecha (`Seize`), galón a la izquierda (`Release`), galón en diagonal (`Return`), galón con barra (`Preempt`), rectángulo redondeado (`FacilityUse`), rectángulo redondeado + flecha (`FacilityInt`) y embudo (`SelectF`). «Rectángulo con galón» describe 4 de 7. | XML original | Igual que A8/A9: variantes. |
| A11 | La familia **Depósito** (5 clases) tiene 4 siluetas originales: cilindro (`Enter`, `Leave`), rectángulos superpuestos (`StorageDef`, `StorageUse`) y **embudo** (`SelectS`). «Cilindro» describe 2 de 5. Y `SelectS` es un embudo como `Select`/`SelectSNA`/`SelectL`, pero en otra familia: la forma ya no dice «selección». | XML original | Vigilar la coherencia de `SelectS` frente a `SelectL`/`SelectSNA` (L8 de la spec). |
| A12 | La familia **Cadena** es coherente, pero `Link` y `Unlink` son **espejos** (en `Link` el rectángulo 20-80 % está a la izquierda y la cápsula a la derecha; en `Unlink` al revés) y dos anclas de `Unlink` no tocan el borde: `Salida1` (14 %, 98 %) queda 0,9 px sobre el borde inferior y `SalidaAlt` (99 %, 52 %) 0,9 px dentro del borde derecho. | XML original, filas `Link`/`Unlink` | Declarar el espejo como variante y llevar las anclas exactas en píxeles (ver A14). |
| A13 | `StorageUse` 100×42: `Salida1` está al **95 %** (40 px de 42) porque en el original la forma no llena la huella (el rectángulo acaba en y=40); con un cilindro que sí la llena, el puerto queda 1,8 px por encima del borde. | XML original, `StorageUse` | El ancla va al 100 %, o la familia debe conservar el margen de 2 px de abajo. |
| A14 | Los porcentajes de `puertos.md` están redondeados: 11 clases se desvían hasta **1,2 px** del valor exacto (`Seize` 35 % en vez de 35,7 %; `FacilityInt` 41 % en vez de 41,7 %; `GateF` 33 % en vez de 33,9 %; `Generate` 27 % en vez de 27,5 %; `Unlink` `SalidaAlt` 98 % en vez de 99 %). | `puertos.md` vs. XML | La app debe consumir `puertos.json`, que trae los píxeles exactos, no los porcentajes redondeados. |
| A15 | El contrato §2 dice «el nombre de la clase (la caption del bloque, o el nombre de la clase si no tiene) y, debajo, el subtítulo», pero la app modela al revés: `title` = clase y `subtitle` = caption (`graph.ts:57-70`), y el nombre de la clase es lo que se trunca primero. | `graph.ts:57-70`, `contrato.md` §2 | Aclarar qué línea va arriba; de eso depende cuántos caracteres entran (la clase: 4-15; la caption: la que ponga el usuario, sin límite). |
| A16 | El punto de puerto de 8-10 px del contrato §4 no puede cumplirse en la miniatura de la pestaña Bloques (S3): a escala de miniatura un punto de 10 px se convierte en 2-3 px. El tamaño de la miniatura todavía no existe (T9 pendiente). | `contrato.md` §4; S3 de la spec | **Pregunta abierta**: la miniatura necesita su propia regla de puerto (o ningún puerto), y el contrato no la tiene. |
| A17 | La zona de cifra se reserva **siempre** (§2: «el espacio se reserva igual»), pero 11 de las 15 declaraciones nunca tendrán cifra (`Clock`, `Function`, `FunctionDist`, `FunctionExpon`, `MFunction`, `MFunctionDist`, `Variable`, `VariableN`, `VariableA`, `StorageDef`, `Note`). En `Clock` la reserva de 36 px deja el rótulo en 2 caracteres (`Clock` → «Cl…»), donde cabrían 8. | `contrato.md` §2; el grupo A lo resolvió con «Clock: bloque redondo de 50 px, mismo criterio» | Liberar la reserva en las declaraciones, o dejarlo escrito como excepción. |
| A18 | El grupo A propone **mover `Note` a una familia «Anotación»** nueva. El contrato §5 lo permite («si el diseño de un grupo propone mover una clase de familia, lo dice y lo justifica») y está justificado, pero la taxonomía está decidida y `Note` está en `Operación`. | `A-lineal.md`, «Desviaciones declaradas»; `contrato.md` §5 | El padre decide: aceptar la familia o mantener `Note` en `Operación`. No es un defecto de las reglas. |

### Coherencia de las familias (resumen)

| Familia | Clases | Siluetas originales distintas | ¿Coherente? |
|---|---|---|---|
| Cola | 2 | 1 | sí |
| Tabla | 3 | 2 (`QTable` lleva una elipse pegada) | casi |
| Cadena | 2 | 2 (espejo) | casi |
| Destino | 1 | 1 | sí (pero ver B10) |
| Selección | 3 | 1 | sí |
| Fuente | 2 | 2 | no (B10) |
| Depósito | 5 | 4 | no (A11) |
| Recurso | 7 | 7 | no (A10) |
| Decisión | 10 | 5 | no (B2, B3) |
| Grupo | 12 | 7 | no (A8, B7, B8) |
| Operación | 19 | 12 | no (A9) |

Seis de las once familias no son coherentes geométricamente: el nombre de la
familia describe una forma y la familia contiene varias. El patrón que usan
las entregas que sí resuelven las anclas (grupos A y C) es el mismo: **la
familia es un lenguaje, no una forma única**, y cada clase nombra su
variante. El contrato debería decirlo así, con la lista de variantes por
familia y el ancla que las distingue.

---

## Verificacion

Comandos ejecutados y salida observada. Todos son de lectura: no se modificó
ningún archivo del repositorio fuera de este entregable.

**1. Las 66 clases del catálogo (comando de verificación del enunciado)**

```
$ cd /home/lucas/Documentos/Workspace/Genmsi && python3 -c "import re;s=open('crates/genmsi-format/src/catalogue.rs').read();print(len(set(re.findall(r'class: \"([^\"]+)\"', s))))"
66
```

**2. Fidelidad de `puertos.md` contra `gClasses.xml`** (script en
`/tmp/genmsi-audit-cV3J/`, sobre el XML original)

```
2. clases en puertos.md: 66 | puertos: 131 | huellas identicas: True | clases del xml: 66
```

Es decir: las 66 clases están, las 131 anclas están con el nombre correcto y
las huellas coinciden; sólo hay redondeo de porcentajes (advertencia A14).

**3. Clasificación de las 131 anclas contra la silueta original** (punto en la
unión de primitivas, tolerancia 1,5 px)

```
RemoveTrans     Salida1     Output (  50.0, 43.0)  50.0%, 86.0%  INTERIOR-UNION(Ellipse)
RemoveNumber    Salida1     Output (  50.0, 43.0)  50.0%, 86.0%  INTERIOR-UNION(Ellipse)
RemoveMe        Salida1     Output (  50.0, 43.0)  50.0%, 86.0%  INTERIOR-UNION(Ellipse)
JoinNumber      Entrada1    Input  (  50.0,  7.0)  50.0%, 14.0%  INTERIOR-UNION(Ellipse)
JoinMe          Entrada1    Input  (  50.0,  7.0)  50.0%, 14.0%  INTERIOR-UNION(Ellipse)
```

126 anclas en el borde, 5 en el interior (B7). Ninguna fuera del original.

**4. El rombo de la familia Decisión frente a las anclas** (distancia con signo
al borde; 0 = en el vértice)

```
A. rombo de la familia Decision (distancia con signo al borde; 0 = en el vertice):
   Test           64x64  Entrada1(32,0):+0.0  Salida1(64,32):+0.0  Salida2(0,32):+0.0
   Transfer       64x64  Entrada1(32,0):+0.0  Salida1(64,32):+0.0  Salida2(0,32):+0.0
   GateF          112x48  Entrada1(38,0):+7.7  Salida1(38,48):+7.7  Salida2(0,24):+0.0
   GateL          102x48  Entrada1(38,0):+6.1  Salida1(38,48):+6.1  Salida2(0,24):+0.0
   GateM          102x48  Entrada1(38,0):+6.1  Salida1(38,48):+6.1  Salida2(0,24):+0.0
   GateS          102x48  Entrada1(38,0):+6.1  Salida1(38,48):+6.1  Salida2(0,24):+0.0
   Loop           96x32  Entrada1(57,0):+3.0  Salida1(57,32):+3.0  Salida2(0,16):+0.0
   Multifurcacion 80x20  Entrada(40,0):+0.0  Salida1(0,20):+10.0  SalidaProb2(20,20):+5.0  SalidaProb3(40,20):+0.0  SalidaProb4(60,20):+5.0  SalidaProb5(80,20):+10.0
   MultifurcacionP 80x20  Entrada(40,0):+0.0  Salida1(0,20):+10.0  SalidaProb2(20,20):+5.0  SalidaProb3(40,20):+0.0  SalidaProb4(60,20):+5.0  SalidaProb5(80,20):+10.0
   Split          88x32  Entrada1(44,0):+0.0  Salida1(0,32):+16.0  Salida2(88,32):+16.0

C. elipses de la familia: elipse centrada en la caja
   Generate  120x63  Salida1(33,63): +3.0 px del arco
   Terminate  84x64  Entrada1(32,0): +0.9 px del arco

D. cajas inscritas (rombo) y presupuesto de rotulo:
   Test           dibujado 72x64 -> caja inscrita 36x32
   Transfer       dibujado 72x64 -> caja inscrita 36x32
   GateF          dibujado 112x48 -> caja inscrita 56x24
   Multifurcacion dibujado 80x32 -> caja inscrita 40x16
   Split          dibujado 88x32 -> caja inscrita 44x16

B. clases cuya huella es menor que el minimo que dibuja la app (graph.ts:108-109 -> 72x32):
   Clock          huella 50x50 -> se dibuja 72x50
   FunctionDist   huella 64x64 -> se dibuja 72x64
   Multifurcacion huella 80x20 -> se dibuja 80x32
   MultifurcacionP huella 80x20 -> se dibuja 80x32
   StorageDef     huella 100x30 -> se dibuja 100x32
   Test           huella 64x64 -> se dibuja 72x64
   Transfer       huella 64x64 -> se dibuja 72x64
```

**5. Presupuesto de texto** (ancho útil = huella − 6 px por banda de puerto −
6 px de aire; zona de cifra 36 px; 7 px por carácter del rótulo)

```
6. 80x32 -> ancho util 74, caja de rotulo 26 -> 3 caracteres (cifra 36 px)
6. 97x32 -> ancho util 91, caja de rotulo 43 -> 6 caracteres (cifra 36 px)
6. 80x20 -> ancho util 74, caja de rotulo 68 -> 9 caracteres (cifra 0 px)
6. 64x64 -> ancho util 58, caja de rotulo 52 -> 7 caracteres (cifra 0 px)
6. 80x48 -> ancho util 74, caja de rotulo 26 -> 3 caracteres (cifra 36 px)
6. 120x63 -> ancho util 114, caja de rotulo 66 -> 9 caracteres (cifra 36 px)
```

(Cálculo sobre la huella cruda, sin el recorte de 72×32 y sin el recorte del
rombo: para la familia Decisión el límite real es la caja inscrita del rombo
—32×32 en `Test`— y para `Multifurcacion` la huella dibujada es 80×32, no
80×20. Los dos anexos de abajo ya aplican esas correcciones.)

**6. Taxonomía: el contrato y la spec dicen lo mismo**

```
contrato: {'Fuente': 2, 'Destino': 1, 'Decisión': 10, 'Cola': 2, 'Recurso': 7, 'Depósito': 5, 'Tabla': 3, 'Cadena': 2, 'Grupo': 12, 'Selección': 3, 'Operación': 19} 66
total contrato 66 faltan [] sobran []
Cadena 2 2 IGUAL / Cola 2 2 IGUAL / Decisión 10 10 IGUAL / Depósito 5 5 IGUAL / Destino 1 1 IGUAL /
Fuente 2 2 IGUAL / Grupo 12 12 IGUAL / Operación 19 19 IGUAL / Recurso 7 7 IGUAL / Selección 3 3 IGUAL / Tabla 3 3 IGUAL
```

**7. Zonas de cifra entregadas: cuántas no pueden mostrar el par por defecto**

```
== A-lineal.md 24 clases
   -> sin una linea: 14  sin ninguna forma: 1
== C-grupos.md 25 clases
   -> sin una linea: 0  sin ninguna forma: 24
```

(«una línea» = 30 px de ancho; «apilada» = 12×22 px.)

**8. Lo que publica el motor, frente a lo que el contrato pide**

```
7. entidades publicadas por el motor: ['facilities', 'storages', 'queues', 'chains', 'logic', 'savevalues', 'tables', 'qtables']
7. mapa ENTITY_PROPERTY del replay: ['Queue', 'Depart', 'Seize', 'Release', 'Preempt', 'Return', 'Enter', 'Leave', 'Link', 'Unlink']
```

**9. Estado del repositorio** (antes y después; no se creó ni modificó ningún
archivo salvo este entregable)

```
$ git status --porcelain
?? "modelo de test"
?? odd/tasks/forma-bloques-diseno/
?? odd/tasks/genmsi-forma-bloques.md
```

Las tres entradas ya estaban antes de la auditoría (`odd/tasks/` no está
versionado).

**Directorio temporal usado** (los grupos lo pueden borrar):
`/tmp/genmsi-audit-cV3J` (contiene `verify.py`, `verify2.py`, `tablas.py`,
`ensamblar.py`, `gen_tables.py`, `pc.py`, `tablaA.md`, `tablaB.md`; ninguno
escribe fuera de ahí salvo este entregable).

---

## Anexo A — Las 66 clases: anclas de puerto

Criterio de la columna «¿el borde de la silueta de familia pasa por el ancla?»:
la silueta que la familia del contrato §5 implica (elipse ancha, rombo, cajón
con remate, rectángulo con galón, cilindro, rectángulo con cabecera, dos
cápsulas, hexágono con elipse, embudo, rectángulo redondeado, círculo con
anillo) *centrada en la huella de la clase*.

| Clase | Familia | Huella | Anclas (x%, y%) | ¿El borde de la silueta de familia pasa por el ancla? | Ancla en el interior de la silueta original | Veredicto |
|---|---|---|---|---|---|---|
| `Advance` | Operación | 80x32 | Entrada1@50%,0%, Salida1@50%,100% | Entrada1: borde; Salida1: borde | — | ok |
| `Alter` | Grupo | 100x50 | Entrada1@40%,0%, Salida1@10%,100%, Salida2@70%,100% | Entrada1: borde; Salida1: borde; Salida2: borde | — | ok |
| `Assemble` | Grupo | 88x32 | Entrada1@50%,0%, Salida1@50%,100% | Entrada1: borde; Salida1: borde | — | ok |
| `Assign` | Operación | 100x45 | Entrada1@50%,0%, Salida1@50%,100% | Entrada1: borde; Salida1: borde | — | ok |
| `Buffer` | Operación | 88x32 | Entrada1@50%,0%, Salida1@50%,100% | Entrada1: borde; Salida1: borde | — | ok |
| `Clock` | Fuente | 50x50 | — | — | — | ok |
| `Counter` | Operación | 100x40 | Entrada1@50%,0%, Salida1@50%,100% | Entrada1: borde; Salida1: borde | — | ok |
| `Depart` | Cola | 97x32 | Entrada1@36%,0%, Salida1@36%,100% | Entrada1: borde; Salida1: borde | — | ok |
| `Enter` | Depósito | 96x32 | Entrada1@36%,0%, Salida1@36%,100% | Entrada1: borde; Salida1: borde | — | ok |
| `Examine` | Grupo | 80x48 | Entrada1@50%,0%, Salida1@25%,83%, Salida2@75%,83% | Entrada1: borde; Salida1: INTERIOR del hexágono (cae en la arista de la cuña del original); Salida2: INTERIOR del hexágono (cae en la arista de la cuña del original) | — | **bloqueante**: INTERIOR del hexágono (cae en la arista de la cuña del original) |


| `ExamineNumber` | Grupo | 80x48 | Entrada1@50%,0%, Salida1@25%,83%, Salida2@75%,83% | Entrada1: borde; Salida1: INTERIOR del hexágono (cae en la arista de la cuña del original); Salida2: INTERIOR del hexágono (cae en la arista de la cuña del original) | — | **bloqueante**: INTERIOR del hexágono (cae en la arista de la cuña del original) |


| `FacilityInt` | Recurso | 120x40 | Entrada1@42%,0%, SalidaAlt@100%,50%, Salida1@42%,100% | Entrada1: borde; SalidaAlt: borde; Salida1: borde | — | ok |
| `FacilityUse` | Recurso | 100x40 | Entrada1@50%,0%, Salida1@50%,100% | Entrada1: borde; Salida1: borde | — | ok |
| `Function` | Operación | 100x100 | — | — | — | ok |
| `FunctionDist` | Operación | 64x64 | — | — | — | ok |
| `FunctionExpon` | Operación | 100x100 | — | — | — | ok |
| `GateF` | Decisión | 112x48 | Entrada1@34%,0%, Salida1@34%,100%, Salida2@0%,50% | Entrada1: FUERA del rombo (7,7 px sobre la arista); Salida1: FUERA del rombo (el rombo no tiene base plana); Salida2: vértice lateral | — | **bloqueante**: FUERA del rombo (7,7 px sobre la arista); FUERA del rombo (el rombo no tiene base plana) |


| `GateL` | Decisión | 102x48 | Entrada1@37%,0%, Salida1@37%,100%, Salida2@0%,50% | Entrada1: FUERA del rombo (6,1 px sobre la arista); Salida1: FUERA del rombo (el rombo no tiene base plana); Salida2: vértice lateral | — | **bloqueante**: FUERA del rombo (6,1 px sobre la arista); FUERA del rombo (el rombo no tiene base plana) |


| `GateM` | Decisión | 102x48 | Entrada1@37%,0%, Salida1@37%,100%, Salida2@0%,50% | Entrada1: FUERA del rombo (6,1 px sobre la arista); Salida1: FUERA del rombo (el rombo no tiene base plana); Salida2: vértice lateral | — | **bloqueante**: FUERA del rombo (6,1 px sobre la arista); FUERA del rombo (el rombo no tiene base plana) |


| `GateS` | Decisión | 102x48 | Entrada1@37%,0%, Salida1@37%,100%, Salida2@0%,50% | Entrada1: FUERA del rombo (6,1 px sobre la arista); Salida1: FUERA del rombo (el rombo no tiene base plana); Salida2: vértice lateral | — | **bloqueante**: FUERA del rombo (6,1 px sobre la arista); FUERA del rombo (el rombo no tiene base plana) |


| `Gather` | Grupo | 88x32 | Entrada1@50%,0%, Salida1@50%,100% | Entrada1: borde; Salida1: borde | — | ok |
| `Generate` | Fuente | 120x63 | Salida1@28%,100% | Salida1: FUERA de la elipse (3 px bajo el arco) | — | **bloqueante**: FUERA de la elipse (3 px bajo el arco) |


| `InitiaSV` | Operación | 100x45 | — | — | — | ok |
| `InitialLS` | Operación | 96x32 | — | — | — | ok |
| `JoinMe` | Grupo | 100x50 | Entrada1@50%,14%, Salida1@50%,100% | Entrada1: INTERIOR (centro de la burbuja); Salida1: borde | Entrada1 (50,7) = centro de la elipse de 100x50 | **bloqueante**: INTERIOR (centro de la burbuja) |


| `JoinNumber` | Grupo | 100x50 | Entrada1@50%,14%, Salida1@50%,100% | Entrada1: INTERIOR (centro de la burbuja); Salida1: borde | Entrada1 (50,7) = centro de la elipse de 100x50 | **bloqueante**: INTERIOR (centro de la burbuja) |


| `Leave` | Depósito | 96x32 | Entrada1@36%,0%, Salida1@36%,100% | Entrada1: borde; Salida1: borde | — | ok |
| `Link` | Cadena | 98x50 | Entrada1@36%,20%, Salida1@36%,80% | Entrada1: borde; Salida1: borde | — | ok |
| `Logic` | Operación | 96x32 | Entrada1@40%,0%, Salida1@40%,100% | Entrada1: borde; Salida1: borde | — | ok |
| `Loop` | Decisión | 96x32 | Entrada1@59%,0%, Salida1@59%,100%, Salida2@0%,50% | Entrada1: FUERA del rombo (3,0 px sobre la arista); Salida1: FUERA del rombo (el rombo no tiene base plana); Salida2: vértice lateral | — | **bloqueante**: FUERA del rombo (3,0 px sobre la arista); FUERA del rombo (el rombo no tiene base plana) |


| `MFunction` | Operación | 100x100 | — | — | — | ok |
| `MFunctionDist` | Operación | 72x72 | — | — | — | ok |
| `Mark` | Operación | 96x36 | Entrada1@40%,0%, Salida1@40%,100% | Entrada1: borde; Salida1: borde | — | ok |
| `Match` | Grupo | 80x48 | Entrada1@50%,0%, Salida1@50%,100% | Entrada1: borde; Salida1: borde | — | ok |
| `Multifurcacion` | Decisión | 80x20 | Entrada@50%,0%, Salida1@0%,100%, SalidaProb2@25%,100%, SalidaProb3@50%,100%, SalidaProb4@75%,100%, SalidaProb5@100%,100% | Entrada: vértice superior; Salida1: FUERA del rombo (el rombo no tiene base plana); SalidaProb2: FUERA del rombo (el rombo no tiene base plana); SalidaProb3: vértice inferior; SalidaProb4: FUERA del rombo (el rombo no tiene base plana); SalidaProb5: FUERA del rombo (el rombo no tiene base plana) | Entrada en el vértice (40,0) del triángulo original | **bloqueante**: FUERA del rombo (el rombo no tiene base plana) |


| `MultifurcacionP` | Decisión | 80x20 | Entrada@50%,0%, Salida1@0%,100%, SalidaProb2@25%,100%, SalidaProb3@50%,100%, SalidaProb4@75%,100%, SalidaProb5@100%,100% | Entrada: vértice superior; Salida1: FUERA del rombo (el rombo no tiene base plana); SalidaProb2: FUERA del rombo (el rombo no tiene base plana); SalidaProb3: vértice inferior; SalidaProb4: FUERA del rombo (el rombo no tiene base plana); SalidaProb5: FUERA del rombo (el rombo no tiene base plana) | Entrada en el vértice (40,0) del triángulo original | **bloqueante**: FUERA del rombo (el rombo no tiene base plana) |


| `Note` | Operación | 80x80 | — | — | — | ok |
| `Preempt` | Recurso | 88x32 | Entrada1@40%,0%, Salida1@40%,100%, SalidaAlt@100%,100% | Entrada1: borde; Salida1: borde; SalidaAlt: borde | — | ok |
| `Priority` | Operación | 88x52 | Entrada1@50%,0%, Salida1@50%,100% | Entrada1: borde; Salida1: borde | — | ok |
| `QTable` | Tabla | 130x70 | — | — | — | ok |
| `Queue` | Cola | 97x32 | Entrada1@36%,0%, Salida1@36%,100% | Entrada1: borde; Salida1: borde | — | ok |
| `Release` | Recurso | 98x32 | Entrada1@36%,0%, Salida1@36%,100% | Entrada1: borde; Salida1: borde | — | ok |
| `RemoveMe` | Grupo | 100x50 | Entrada1@50%,0%, Salida1@50%,86%, Salida2@100%,20% | Entrada1: borde; Salida1: INTERIOR (centro de la burbuja); Salida2: borde | Salida1 (50,43) = centro de la elipse de 100x50 | **bloqueante**: INTERIOR (centro de la burbuja) |


| `RemoveNumber` | Grupo | 100x50 | Entrada1@50%,0%, Salida1@50%,86%, Salida2@100%,20% | Entrada1: borde; Salida1: INTERIOR (centro de la burbuja); Salida2: borde | Salida1 (50,43) = centro de la elipse de 100x50 | **bloqueante**: INTERIOR (centro de la burbuja) |


| `RemoveTrans` | Grupo | 100x50 | Entrada1@50%,0%, Salida1@50%,86%, Salida2@100%,20% | Entrada1: borde; Salida1: INTERIOR (centro de la burbuja); Salida2: borde | Salida1 (50,43) = centro de la elipse de 100x50 | **bloqueante**: INTERIOR (centro de la burbuja) |


| `Return` | Recurso | 88x32 | Entrada1@40%,0%, Salida1@40%,100% | Entrada1: borde; Salida1: borde | — | ok |
| `SaveValue` | Operación | 100x45 | Entrada1@50%,0%, Salida1@50%,100% | Entrada1: borde; Salida1: borde | — | ok |
| `Scan` | Grupo | 80x50 | Entrada1@50%,0%, Salida1@25%,80%, Salida2@75%,80% | Entrada1: borde; Salida1: INTERIOR del hexágono (cae en la arista de la cuña del original); Salida2: INTERIOR del hexágono (cae en la arista de la cuña del original) | — | **bloqueante**: INTERIOR del hexágono (cae en la arista de la cuña del original) |


| `Seize` | Recurso | 98x32 | Entrada1@36%,0%, Salida1@36%,100% | Entrada1: borde; Salida1: borde | — | ok |
| `Select` | Selección | 100x50 | Entrada1@50%,0%, Salida1@50%,100% | Entrada1: borde; Salida1: borde | — | ok |
| `SelectF` | Recurso | 100x50 | Entrada1@50%,0%, Salida1@50%,100%, Salida2@100%,50% | Entrada1: borde; Salida1: borde; Salida2: borde | — | ok |
| `SelectL` | Selección | 100x50 | Entrada1@50%,0%, Salida1@50%,100%, Salida2@100%,50% | Entrada1: borde; Salida1: borde; Salida2: borde | — | ok |
| `SelectS` | Depósito | 100x50 | Entrada1@50%,0%, Salida1@50%,100%, Salida2@100%,50% | Entrada1: borde; Salida1: borde; Salida2: borde | — | ok |
| `SelectSNA` | Selección | 100x50 | Entrada1@50%,0%, Salida1@50%,100%, Salida2@100%,50% | Entrada1: borde; Salida1: borde; Salida2: borde | — | ok |
| `Split` | Decisión | 88x32 | Entrada1@50%,0%, Salida1@0%,100%, Salida2@100%,100% | Entrada1: vértice superior; Salida1: FUERA del rombo (el rombo no tiene base plana); Salida2: FUERA del rombo (el rombo no tiene base plana) | — | **bloqueante**: FUERA del rombo (el rombo no tiene base plana) |


| `StorageDef` | Depósito | 100x30 | — | — | — | ok |
| `StorageUse` | Depósito | 100x42 | Entrada1@50%,0%, Salida1@50%,95% | Entrada1: borde; Salida1: 1,8 px sobre el borde inferior | — | ok |
| `Table` | Tabla | 100x70 | — | — | — | ok |
| `Tabulate` | Tabla | 100x50 | Entrada1@50%,0%, Salida1@50%,100% | Entrada1: borde; Salida1: borde | — | ok |
| `Terminate` | Destino | 84x64 | Entrada1@38%,0% | Entrada1: FUERA del círculo centrado (0,9 px sobre el arco) | — | **bloqueante**: FUERA del círculo centrado (0,9 px sobre el arco) |


| `Test` | Decisión | 64x64 | Entrada1@50%,0%, Salida1@100%,50%, Salida2@0%,50% | Entrada1: vértice superior; Salida1: vértice lateral; Salida2: vértice lateral | — | el rombo tiene los tres vértices: las tres anclas caen en vértice |
| `Transfer` | Decisión | 64x64 | Entrada1@50%,0%, Salida1@100%,50%, Salida2@0%,50% | Entrada1: vértice superior; Salida1: vértice lateral; Salida2: vértice lateral | — | el rombo tiene los tres vértices: las tres anclas caen en vértice |
| `Unlink` | Cadena | 98x50 | Entrada1@56%,20%, Salida1@14%,98%, SalidaAlt@99%,52%, Salida2@56%,80% | Entrada1: borde; Salida1: 0,9 px sobre el borde inferior; SalidaAlt: 0,9 px dentro del borde derecho; Salida2: borde | — | ok |
| `Variable` | Operación | 120x32 | — | — | — | ok |
| `VariableA` | Operación | 120x32 | — | — | — | ok |
| `VariableN` | Operación | 120x40 | — | — | — | ok |

---

## Anexo B — Las 66 clases: zonas de texto y de cifra

Columnas: «Interior útil» es la caja de la huella menos 6 px por cada banda de
borde con puerto y menos 6 px de aire; «Caja de texto» es esa caja para las
familias normales y la **caja inscrita** del rombo para la familia Decisión;
«Zona de cifra» son los 36 px que exige el par por defecto; «Rótulo» es el
número de caracteres que entran en la caja de texto con la cifra reservada;
«Líneas que caben» es cuántas líneas de 11 px caben en el alto útil.

| Clase | Familia | Dibujo | Interior útil (An×Al) | Caja de texto (An×Al) | Zona de cifra (36 px) | Rótulo (caracteres) | Líneas que caben | ¿Cabe la cifra por defecto? | Veredicto |
|---|---|---|---|---|---|---|---|---|---|
| `Advance` | Operación | 80x32 | 74×20 | 74×20 | 36 px | 3 | 1 | sí | rótulo de 3 caracteres; una sola línea |
| `Alter` | Grupo | 100x50 | 94×38 | 94×38 | 36 px | 6 | 2 | sí | ok |
| `Assemble` | Grupo | 88x32 | 82×20 | 82×20 | 36 px | 4 | 1 | sí | rótulo de 4 caracteres; una sola línea |
| `Assign` | Operación | 100x45 | 94×33 | 94×33 | 36 px | 6 | 2 | sí | ok |
| `Buffer` | Operación | 88x32 | 82×20 | 82×20 | 36 px | 4 | 1 | sí | rótulo de 4 caracteres; una sola línea |
| `Clock` | Fuente | 72x50 | 66×50 | 66×50 | 36 px | 2 | 2 | sí | rótulo de 2 caracteres |
| `Counter` | Operación | 100x40 | 94×28 | 94×28 | 36 px | 6 | 2 | sí | ok |
| `Depart` | Cola | 97x32 | 91×20 | 91×20 | 36 px | 6 | 1 | sí | una sola línea |
| `Enter` | Depósito | 96x32 | 90×20 | 90×20 | 36 px | 6 | 1 | sí | una sola línea |
| `Examine` | Grupo | 80x48 | 74×42 | 74×42 | 36 px | 3 | 2 | sí | rótulo de 3 caracteres |
| `ExamineNumber` | Grupo | 80x48 | 74×42 | 74×42 | 36 px | 3 | 2 | sí | rótulo de 3 caracteres |
| `FacilityInt` | Recurso | 120x40 | 108×28 | 108×28 | 36 px | 8 | 2 | sí | ok |
| `FacilityUse` | Recurso | 100x40 | 94×28 | 94×28 | 36 px | 6 | 2 | sí | ok |
| `Function` | Operación | 100x100 | 94×100 | 94×100 | 36 px | 6 | 2 | sí | ok |
| `FunctionDist` | Operación | 72x64 | 66×64 | 66×64 | 36 px | 2 | 2 | sí | rótulo de 2 caracteres |
| `FunctionExpon` | Operación | 100x100 | 94×100 | 94×100 | 36 px | 6 | 2 | sí | ok |
| `GateF` | Decisión | 112x48 | 100×36 | 56×24 | no cabe | 7 | 2 | sí | la cifra no cabe en el rombo |
| `GateL` | Decisión | 102x48 | 90×36 | 51×24 | no cabe | 6 | 2 | sí | la cifra no cabe en el rombo |
| `GateM` | Decisión | 102x48 | 90×36 | 51×24 | no cabe | 6 | 2 | sí | la cifra no cabe en el rombo |
| `GateS` | Decisión | 102x48 | 90×36 | 51×24 | no cabe | 6 | 2 | sí | la cifra no cabe en el rombo |
| `Gather` | Grupo | 88x32 | 82×20 | 82×20 | 36 px | 4 | 1 | sí | rótulo de 4 caracteres; una sola línea |
| `Generate` | Fuente | 120x63 | 114×57 | 114×57 | 36 px | 9 | 2 | sí | ok |
| `InitiaSV` | Operación | 100x45 | 94×45 | 94×45 | 36 px | 6 | 2 | sí | ok |
| `InitialLS` | Operación | 96x32 | 90×32 | 90×32 | 36 px | 6 | 2 | sí | ok |
| `JoinMe` | Grupo | 100x50 | 94×44 | 94×44 | 36 px | 6 | 2 | sí | ok |
| `JoinNumber` | Grupo | 100x50 | 94×44 | 94×44 | 36 px | 6 | 2 | sí | ok |
| `Leave` | Depósito | 96x32 | 90×20 | 90×20 | 36 px | 6 | 1 | sí | una sola línea |
| `Link` | Cadena | 98x50 | 92×50 | 92×50 | 36 px | 6 | 2 | sí | ok |
| `Logic` | Operación | 96x32 | 90×20 | 90×20 | 36 px | 6 | 1 | sí | una sola línea |
| `Loop` | Decisión | 96x32 | 84×20 | 48×16 | no cabe | 6 | 1 | sí | la cifra no cabe en el rombo; una sola línea |
| `MFunction` | Operación | 100x100 | 94×100 | 94×100 | 36 px | 6 | 2 | sí | ok |
| `MFunctionDist` | Operación | 72x72 | 66×72 | 66×72 | 36 px | 2 | 2 | sí | rótulo de 2 caracteres |
| `Mark` | Operación | 96x36 | 90×24 | 90×24 | 36 px | 6 | 2 | sí | ok |
| `Match` | Grupo | 80x48 | 74×36 | 74×36 | 36 px | 3 | 2 | sí | rótulo de 3 caracteres |
| `Multifurcacion` | Decisión | 80x32 | 62×20 | 40×16 | no cabe | 4 | 1 | sí | la cifra no cabe en el rombo; rótulo de 4 caracteres; una sola línea |
| `MultifurcacionP` | Decisión | 80x32 | 62×20 | 40×16 | no cabe | 4 | 1 | sí | la cifra no cabe en el rombo; rótulo de 4 caracteres; una sola línea |
| `Note` | Operación | 80x80 | 74×80 | 74×80 | 36 px | 3 | 2 | sí | rótulo de 3 caracteres |
| `Preempt` | Recurso | 88x32 | 76×20 | 76×20 | 36 px | 4 | 1 | sí | rótulo de 4 caracteres; una sola línea |
| `Priority` | Operación | 88x52 | 82×40 | 82×40 | 36 px | 4 | 2 | sí | rótulo de 4 caracteres |
| `QTable` | Tabla | 130x70 | 124×70 | 124×70 | 36 px | 10 | 2 | sí | ok |
| `Queue` | Cola | 97x32 | 91×20 | 91×20 | 36 px | 6 | 1 | sí | una sola línea |
| `Release` | Recurso | 98x32 | 92×20 | 92×20 | 36 px | 6 | 1 | sí | una sola línea |
| `RemoveMe` | Grupo | 100x50 | 88×44 | 88×44 | 36 px | 5 | 2 | sí | ok |
| `RemoveNumber` | Grupo | 100x50 | 88×44 | 88×44 | 36 px | 5 | 2 | sí | ok |
| `RemoveTrans` | Grupo | 100x50 | 88×44 | 88×44 | 36 px | 5 | 2 | sí | ok |
| `Return` | Recurso | 88x32 | 82×20 | 82×20 | 36 px | 4 | 1 | sí | rótulo de 4 caracteres; una sola línea |
| `SaveValue` | Operación | 100x45 | 94×33 | 94×33 | 36 px | 6 | 2 | sí | ok |
| `Scan` | Grupo | 80x50 | 74×44 | 74×44 | 36 px | 3 | 2 | sí | rótulo de 3 caracteres |
| `Seize` | Recurso | 98x32 | 92×20 | 92×20 | 36 px | 6 | 1 | sí | una sola línea |
| `Select` | Selección | 100x50 | 94×38 | 94×38 | 36 px | 6 | 2 | sí | ok |
| `SelectF` | Recurso | 100x50 | 88×38 | 88×38 | 36 px | 5 | 2 | sí | ok |
| `SelectL` | Selección | 100x50 | 88×38 | 88×38 | 36 px | 5 | 2 | sí | ok |
| `SelectS` | Depósito | 100x50 | 88×38 | 88×38 | 36 px | 5 | 2 | sí | ok |
| `SelectSNA` | Selección | 100x50 | 88×38 | 88×38 | 36 px | 5 | 2 | sí | ok |
| `Split` | Decisión | 88x32 | 70×20 | 44×16 | no cabe | 5 | 1 | sí | la cifra no cabe en el rombo; una sola línea |
| `StorageDef` | Depósito | 100x32 | 94×32 | 94×32 | 36 px | 6 | 2 | sí | ok |
| `StorageUse` | Depósito | 100x42 | 94×36 | 94×36 | 36 px | 6 | 2 | sí | ok |
| `Table` | Tabla | 100x70 | 94×70 | 94×70 | 36 px | 6 | 2 | sí | ok |
| `Tabulate` | Tabla | 100x50 | 94×38 | 94×38 | 36 px | 6 | 2 | sí | ok |
| `Terminate` | Destino | 84x64 | 78×58 | 78×58 | 36 px | 4 | 2 | sí | rótulo de 4 caracteres |
| `Test` | Decisión | 72x64 | 54×58 | 36×32 | no cabe | 4 | 2 | sí | la cifra no cabe en el rombo; rótulo de 4 caracteres |
| `Transfer` | Decisión | 72x64 | 54×58 | 36×32 | no cabe | 4 | 2 | sí | la cifra no cabe en el rombo; rótulo de 4 caracteres |
| `Unlink` | Cadena | 98x50 | 92×50 | 92×50 | 36 px | 6 | 2 | sí | ok |
| `Variable` | Operación | 120x32 | 114×32 | 114×32 | 36 px | 9 | 2 | sí | ok |
| `VariableA` | Operación | 120x32 | 114×32 | 114×32 | 36 px | 9 | 2 | sí | ok |
| `VariableN` | Operación | 120x40 | 114×40 | 114×40 | 36 px | 9 | 2 | sí | ok |
