# Grupo C — bifurcación y grupo: 25 clases (contrato v2)

Clases que dividen el flujo o que operan sobre un conjunto de transacciones: 10 de la familia **Decisión**, 12 de la familia **Grupo** y 3 de la familia **Selección**.

La hoja que se mira está en `grupos/C-grupos.svg`: las 25 clases dibujadas en su huella real, con los puertos en su ancla verdadera, las zonas de rótulo y de cifra punteadas y una cifra de ejemplo por bloque.

Los footprints y las anclas de puerto salen de `odd/tasks/forma-bloques-diseno/puertos.md` (el catálogo del editor original); el color sale de la familia del contrato §5.

**Cómo se lee este archivo**

- `family`: la familia de la clase. `variant`: el nombre de la silueta dentro de la familia (§5.1): la familia es un lenguaje de variantes, no una forma única.
- `path`: el `d` del SVG de la silueta, en coordenadas de la huella (0,0 arriba a la izquierda).
- `details`: los trazos interiores, cada uno un `d` propio, separados por ` ; `. Se dibujan sin relleno, con el color de la familia. `none` = sin trazos. Ningún trazo interior pisa una zona de texto ni una de cifra.
- `label` y `figure`: la zona de rótulo y la zona de cifra, `x,y,ancho,alto`. La zona de cifra se reserva siempre, aunque quede vacía, para que el rótulo no se mueva cuando aparece la cifra.
- `ports`: `nombre@x%,y%` con el ancla normalizada a la huella, tal y como viene en `puertos.md` (los porcentajes redondeados del catálogo). El SVG dibuja el punto en el **píxel exacto** del catálogo original: por ejemplo, `GateF` entra y sale por x=38 (el 33% redondeado de `puertos.md`), y las salidas de `Scan`, `Examine` y `ExamineNumber` están en (20,40) y (60,40).
- Las dos zonas viven **dentro** de la silueta, nunca la invaden.

**Las reglas del contrato v2 que este grupo cumple**

- **§2.1 — ancho útil.** La partición izquierda/derecha se usa cuando el ancho útil de la silueta llega a 60 px. Se apilan (rótulo arriba, cifra abajo, en la misma columna) las siete clases cuya caja de texto no llega a 60 px: `Test`, `Transfer`, `GateF`, `GateL`, `GateM` y `GateS` (caja inscrita del rombo: 32 y 38 px de ancho) y `Loop` (la columna de entrada/salida parte el cuerpo en dos tramos de 33 px). También se apilan las tres de la familia **Selección**: el embudo no da 54+30 px en un mismo lado. Las otras quince clases van con el rótulo a la izquierda y la cifra a la derecha.
- **§2.2 — el subtítulo.** Sólo existe en bloques de 40 px de alto o más. Los bloques de menos de 40 px (`Loop`, `Split`, `Assemble`, `Gather`) llevan **una sola línea** con el nombre de la clase, y los que llevan el nombre en dos líneas (`Multifurcacion`, `MultifurcacionP`, `RemoveNumber`, `RemoveTrans`, `ExamineNumber`) no llevan subtítulo: el nombre ocupa las dos líneas de su zona.
- **§2.3 — el nombre entero.** El nombre de la clase se escribe **entero**: en una línea si cabe en su zona, y en dos si no cabe (la zona necesita 24 px de alto para las dos, y el corte lo pone la app en el punto que equilibra las dos líneas). Ningún nombre de este grupo se corta con puntos suspensivos.
- **§6.3 — la cifra.** Una línea de monoespaciada de 10,5 px. La zona de cifra mide **el ancho de su propio texto más 2 px de aire** (§6.3 v3), así que **ninguna clase usa la forma apilada** y ninguna cifra se trunca. El par de siempre va en una línea (`2/19`).
- **§5.1 — una familia es una familia de variantes.** Cada clase nombra su variante en la línea `note`.
- **§3.1 — la silueta toca el puerto.** Las 40 anclas de este grupo caen en el borde de su silueta, salvo las cinco que el catálogo declara en el interior de una burbuja: la entrada de `JoinMe` y `JoinNumber` en (50,7) y la salida de `RemoveMe`, `RemoveNumber` y `RemoveTrans` en (50,43). Esas cinco son la **excepción única** que el contrato v2 declara: la burbuja se dibuja **centrada en el ancla** y el puerto queda dentro de ella. Las salidas de `Scan`, `Examine` y `ExamineNumber` caen sobre la arista de la cuña de su variante «caja sobre cuña», no dentro del relleno.

**Colores de familia**

- **Decisión** (`#4338ca`): el par de siempre (dentro/pasadas).
- **Grupo** (`#be185d`): el par de siempre (dentro/pasadas).
- **Selección** (`#0e7490`): el par de siempre (dentro/pasadas). `SelectL` **no** muestra el valor del interruptor: no declara la entidad que elige (§6.1), así que su cifra por entidad es irresoluble sin una propiedad nueva en el modelo.

**Variantes por familia**

- **Decisión**: `Test`/`Transfer` rombo · `GateF`/`GateL`/`GateM`/`GateS` rombo con accesorio · `Loop` cuerpo con punta a la izquierda · `Multifurcacion`/`MultifurcacionP` peine de la multifurcacion · `Split` horquilla del split.
- **Grupo**: `JoinMe`/`JoinNumber` hexagono con elipse · `RemoveMe`/`RemoveNumber`/`RemoveTrans` cuerpo achaflanado con burbuja · `Assemble` embudo · `Gather` embudo invertido · `Match` cuerpo con mejilla · `Scan`/`Examine`/`ExamineNumber` caja sobre cuña · `Alter` hexagono con elipse lateral.
- **Selección**: `Select`/`SelectSNA`/`SelectL` embudo.

**Lo que no se puede evitar, y por qué**

- `Test` y `Transfer`: la caja inscrita del rombo de 64×64 es de 32×32 px. Caben el rótulo (12 px) y la cifra (11 px), pero **no cabe el subtítulo** (10 px más): harían falta 34 px y hay 32. Es el caso más apretado del grupo y la razón de que el rombo lleve sólo una línea.
- `GateF`, `GateL`, `GateM` y `GateS`: la caja inscrita del rombo es de 38×24 px. El rótulo (12 px) y la cifra (11 px) la llenan, así que **tampoco caben subtítulo**, aunque el bloque mida 48 px de alto; la cifra va en una línea (zona de 34 px) y no apilada, porque la caja inscrita no da para las 22 px de alto de la forma apilada más el rótulo y el subtítulo.
- `Multifurcacion` y `MultifurcacionP`: la clase es de 80×20 px en el catálogo, pero la app dibuja el nodo con un mínimo de 32 px de alto. Con la huella desnuda de 20 px quedarían 8 px útiles entre los puntos de puerto y no cabría ni el rótulo, así que **la entrega usa la huella dibujada de 80×32**; los anclajes no cambian, porque siguen en el borde superior y en el inferior. Es la desviación que anuncia la advertencia A2 de la auditoría.
**Lo que cambia con el nombre entero**

- Cinco clases tienen el nombre **en dos líneas**, porque no caben en una sola de su zona: `Multifurcacion` («Multifu» y «rcacion»), `MultifurcacionP` («Multifu» y «rcacionP»), `RemoveNumber` («Remove» y «Number»), `RemoveTrans` («Remove» y «Trans») y `ExamineNumber` («Examine» y «Number»). Sus zonas de rótulo crecen a 24 px de alto, y las dos del peine estrechan su cifra a los 26 px de su propio texto, sin mover el borde derecho. El nombre se lee entero: ya no llega truncado.
- `Multifurcacion` y `MultifurcacionP`: el nombre va en dos líneas sobre la corona del peine, y la cifra se queda a su derecha. El peine no cambia de silueta. Los dos nombres son **una sola palabra** (el peine no tiene mayúscula que separe dos palabras, y la P final es un sufijo), así que su nombre se parte dentro de la palabra: es el precio de que el peine se lea de un vistazo.
- `Transfer` y `SelectSNA` llevan su rótulo a 9 px (los demás, de 10 a 11) porque son los nombres más largos que aún caben enteros en su zona.

---

## Test
- family: Decisión
- variant: rombo
- footprint: 64x64
- ports: Entrada1@50%,0% ; Salida1@100%,50% ; Salida2@0%,50%
- path: M32 0 L64 32 L32 64 L0 32 Z
- details: M32 6 L37 10.5 L32 15 L27 10.5 Z
- label: 8,25,48,12
- figure: 17,38,30,11
- note: Variante «rombo» de la familia Decisión: el único rombo exacto de la familia, el del catálogo original; la entrada baja por el vértice superior (32,0) y las dos salidas salen por los vértices izquierdo y derecho (0,32) y (64,32), que son las tres anclas de la tabla; el rombo anidado de arriba conserva el «doble rombo» del original y, como la caja inscrita del rombo mide 32 px de ancho (menos de 60), el rótulo va en la banda superior y la cifra en la inferior, en la misma columna. La cifra va en una línea y no apilada: la caja inscrita de 32×32 admite de sobra la zona de una línea de 30 px, y la forma apilada de 14×22 tampoco ayudaría, porque con el subtítulo harían falta 44 px de alto y sólo hay 32. El rombo da 48 px de ancho util a la altura de la banda de rotulo, y la cifra baja a la banda de abajo sin perder su sitio.

## Transfer
- family: Decisión
- variant: rombo
- footprint: 64x64
- ports: Entrada1@50%,0% ; Salida1@100%,50% ; Salida2@0%,50%
- path: M32 0 L64 32 L32 64 L0 32 Z
- details: M28 7 L36 11 L28 15
- label: 8,25,48,12
- figure: 17,38,30,11
- note: Variante «rombo» de la familia Decisión, la misma silueta de Test: lo que la distingue es la punta de flecha de la corona, que dice que el flujo se reenvía; las tres anclas son las de la tabla (32,0), (64,32) y (0,32), en los vértices, y la caja inscrita de 32 px de ancho obliga al rótulo arriba y la cifra abajo. La cifra va en una línea (zona de 30 px) y no apilada: la apilada de 14×22 cabría de sobra en la caja inscrita, pero no resuelve el problema del subtítulo, que necesita 44 px de alto de los 32 que hay. El rombo da 48 px de ancho util a la altura de la banda de rotulo, y la cifra baja a la banda de abajo sin perder su sitio.

## GateF
- family: Decisión
- variant: rombo con accesorio
- footprint: 112x48
- ports: Entrada1@33%,0% ; Salida1@33%,100% ; Salida2@0%,50%
- path: M0 24 L38 0 L76 24 L38 48 Z M76 24 L94 12 L112 24 L94 36 Z
- details: none
- label: 18,13,40,12
- figure: 21,26,34,11
- note: Variante «rombo con accesorio» de la familia Decisión: un rombo descentrado (vértices en 0,24 y 76,24) más el accesorio que dibuja la entidad que la compuerta vigila (aquí un rombo, el recurso); el accesorio es la firma de la clase y no lleva texto, porque el rombo mide 76 px y su caja inscrita (38 px de ancho) es menor de 60: el rótulo va en la banda superior y la cifra en la inferior, en la misma columna; las anclas (38,0), (38,48) y (0,24) del catálogo caen en los vértices.

## GateL
- family: Decisión
- variant: rombo con accesorio
- footprint: 102x48
- ports: Entrada1@37%,0% ; Salida1@37%,100% ; Salida2@0%,50%
- path: M0 24 L38 0 L76 24 L38 48 Z M76 12 H102 V38 H76 Z
- details: none
- label: 18,13,40,12
- figure: 21,26,34,11
- note: Variante «rombo con accesorio»: el accesorio es una caja (el interruptor que la compuerta consulta) y por eso la compuerta se distingue de GateF, GateM y GateS de un vistazo; el rombo lleva el rótulo arriba y la cifra abajo porque su caja inscrita es de 38 px de ancho; anclas del catálogo en los vértices: (38,0) entrada, (38,48) salida de abajo y (0,24) salida lateral.

## GateM
- family: Decisión
- variant: rombo con accesorio
- footprint: 102x48
- ports: Entrada1@37%,0% ; Salida1@37%,100% ; Salida2@0%,50%
- path: M0 24 L38 0 L76 24 L38 48 Z M76 12 H102 V38 L89 32 L76 38 Z
- details: none
- label: 18,13,40,12
- figure: 21,26,34,11
- note: Variante «rombo con accesorio»: el accesorio es una caja con la muesca que declara el modo de comparación, y la muesca sube por la izquierda como en el original; el rombo lleva el rótulo arriba y la cifra abajo por su caja inscrita de 38 px, y la salida lateral sigue siendo el vértice izquierdo (0,24).

## GateS
- family: Decisión
- variant: rombo con accesorio
- footprint: 102x48
- ports: Entrada1@37%,0% ; Salida1@37%,100% ; Salida2@0%,50%
- path: M0 24 L38 0 L76 24 L38 48 Z M76 25 A13 13 0 1 1 102 25 A13 13 0 1 1 76 25 Z
- details: none
- label: 18,13,40,12
- figure: 21,26,34,11
- note: Variante «rombo con accesorio»: el accesorio es una elipse (el depósito que la compuerta vigila), el mismo lenguaje de burbuja que usan Join y Remove; el accesorio queda vacío de texto y el rombo lleva el rótulo arriba y la cifra abajo; las anclas (38,0), (38,48) y (0,24) son las del catálogo y caen en vértices del rombo.

## Loop
- family: Decisión
- variant: cuerpo con punta a la izquierda
- footprint: 96x32
- ports: Entrada1@59%,0% ; Salida1@59%,100% ; Salida2@0%,50%
- path: M0 16 L18 0 H88 L96 8 V24 L88 32 H18 Z
- details: none
- label: 19,5,32,12
- figure: 19,18,32,11
- note: Variante «cuerpo con punta a la izquierda» de la familia Decisión: la punta ES la salida de retroceso (0,16) del original, y la entrada y la salida de paso cruzan el borde superior e inferior en x=57, que no es el centro del cuerpo (18-96), por eso el cuerpo se desplaza a la derecha; esa columna de entrada/salida parte el cuerpo en dos tramos de 33 px, ninguno de 60, así que el rótulo va arriba y la cifra abajo en el tramo izquierdo.

## Multifurcacion
- family: Decisión
- variant: peine de la multifurcacion
- footprint: 80x40
- ports: Entrada@50%,0% ; Salida1@0%,100% ; SalidaProb2@25%,100% ; SalidaProb3@50%,100% ; SalidaProb4@75%,100% ; SalidaProb5@100%,100%
- path: M2 0 H78 L80 40 L70 30 L60 40 L50 30 L40 40 L30 30 L20 40 L10 30 L0 40 Z
- details: none
- label: 2,6,48,24
- figure: 50,9,26,10
- note: Variante «peine de la multifurcacion» de la familia Decisión: los cinco dientes son exactamente las cinco salidas (0,32), (20,32), (40,32), (60,32) y (80,32) y la entrada sigue en el borde superior (40,0); la clase es de 20 px de alto en el catálogo, pero el peine necesita sitio para sus dos bandas de texto, así que la entrega usa una huella dibujada de 80×40 (la app dibuja el nodo con 32 px de alto mínimo, y aquí hacen falta 40 para que el rótulo y la cifra no se pisen); el texto va en banda propia sobre la corona (rótulo arriba, cifra abajo) porque el nombre completo (64 px) más la cifra (27 px) no caben de lado en 80 px, y las dos bandas caben en la franja superior de 24 px que deja la corona; el nombre ya no llega truncado. «Multifurcacion» no cabe en una linea de la corona, asi que va en dos («Multifu» y «rcacion»): la zona de rotulo ocupa 48 px y 24 px de alto sobre la corona, y la cifra se queda a su derecha con 26 px.

## MultifurcacionP
- family: Decisión
- variant: peine de la multifurcacion
- footprint: 80x40
- ports: Entrada@50%,0% ; Salida1@0%,100% ; SalidaProb2@25%,100% ; SalidaProb3@50%,100% ; SalidaProb4@75%,100% ; SalidaProb5@100%,100%
- path: M2 0 H78 L80 40 L70 30 L60 40 L50 30 L40 40 L30 30 L20 40 L10 30 L0 40 Z
- details: M56 4.5 H72 ; M64.5 4.5 A2.5 2.5 0 1 1 59.5 4.5 A2.5 2.5 0 1 1 64.5 4.5 Z
- label: 2,6,48,24
- figure: 50,9,26,10
- note: Variante «peine de la multifurcacion», la misma silueta de la otra multifurcación: lo que la distingue es el raíl y la perilla del parámetro sobre la corona, y no un nombre distinto; las cinco salidas y la entrada tienen las mismas anclas del catálogo, y también se entrega sobre la huella dibujada de 80×40 (la clase es de 20 px), con el texto en banda propia sobre la corona por lo mismo que en la otra multifurcación, y el raíl y la perilla aparte, en la esquina superior izquierda, sobre la banda, para que no se pisen con el rótulo. «MultifurcacionP» va en dos lineas («Multifu» y «rcacionP»): la zona de rotulo ocupa 48 px y 24 px de alto sobre la corona, la cifra se queda a su derecha con 26 px, y el raíl y la perilla del parametro quedan en la esquina, sobre la banda.

## Split
- family: Decisión
- variant: horquilla del split
- footprint: 88x32
- ports: Entrada1@50%,0% ; Salida1@0%,100% ; Salida2@100%,100%
- path: M0 0 H88 V32 H52 L44 22 L36 32 H0 Z
- details: none
- label: 6,7,38,12
- figure: 48,8,30,11
- note: Variante «horquilla del split» de la familia Decisión: el borde inferior se abre en una horquilla y las dos esquinas de abajo son las dos salidas (0,32) y (88,32), con la entrada en el borde superior (44,0); la horquilla se hunde sólo hasta y=22 para que la banda de texto (76 px, más de 60) quepa entera, porque en las punteras no cabe una zona de cifra de 30 px.

## JoinMe
- family: Grupo
- variant: hexagono con elipse
- footprint: 100x50
- ports: Entrada1@50%,14% ; Salida1@50%,100%
- path: M0 14 H100 V40 L90 50 H10 L0 40 Z M43 7 A7 7 0 1 1 57 7 A7 7 0 1 1 43 7 Z
- details: none
- label: 4,16,62,21
- figure: 68,20,30,11
- note: Variante «hexagono con elipse» de la familia Grupo: cuerpo hexagonal y burbuja de los miembros arriba; la entrada (50,7) cae en el INTERIOR de esa burbuja y no en su borde, que es lo que declara el catálogo original, así que —única excepción a §3.1— la burbuja se dibuja CENTRADA en el ancla para que el puerto quede dentro de la figura y no flotando sobre el relleno; la salida sale por el borde inferior (50,50).

## JoinNumber
- family: Grupo
- variant: hexagono con elipse
- footprint: 100x50
- ports: Entrada1@50%,14% ; Salida1@50%,100%
- path: M0 14 H100 V40 L90 50 H10 L0 40 Z M43 7 A7 7 0 1 1 57 7 A7 7 0 1 1 43 7 Z
- details: M82 33 V39 M88 33 V39 M94 33 V39
- label: 4,16,62,21
- figure: 68,20,30,11
- note: Variante «hexagono con elipse»: la misma burbuja de JoinMe, con las tres rayas del contador de «N» en la esquina baja derecha; la entrada sigue en el centro de la burbuja (50,7) y por eso la burbuja se dibuja centrada en el ancla (excepción de §3.1), y la salida en el borde inferior (50,50).

## RemoveMe
- family: Grupo
- variant: cuerpo achaflanado con burbuja
- footprint: 100x50
- ports: Entrada1@50%,0% ; Salida1@50%,86% ; Salida2@100%,20%
- path: M0 10 L10 0 H90 L100 10 V36 H0 Z M43 43 A7 7 0 1 1 57 43 A7 7 0 1 1 43 43 Z
- details: M8 31 H92
- label: 6,8,58,21
- figure: 67,16,30,11
- note: Variante «cuerpo achaflanado con burbuja» de la familia Grupo: cuerpo con las esquinas superiores achaflanadas, la línea de la lista de miembros y la burbuja colgando abajo; la salida Salida1 (50,43) cae en el centro de esa burbuja, igual que la entrada de Join, así que la burbuja se dibuja CENTRADA en el ancla para que el puerto quede dentro de ella (única excepción a §3.1); la entrada sube por (50,0) y la segunda salida sale por el borde derecho (100,10).

## RemoveNumber
- family: Grupo
- variant: cuerpo achaflanado con burbuja
- footprint: 100x50
- ports: Entrada1@50%,0% ; Salida1@50%,86% ; Salida2@100%,20%
- path: M0 10 L10 0 H90 L100 10 V36 H0 Z M43 43 A7 7 0 1 1 57 43 A7 7 0 1 1 43 43 Z
- details: M8 31 H92 ; M12 33 V36 M18 33 V36 M24 33 V36
- label: 8,6,40,24
- figure: 67,16,30,11
- note: Variante «cuerpo achaflanado con burbuja»: igual que RemoveMe, con las tres rayas del contador bajo la línea de la lista, que dicen «N en lugar de uno»; la salida (50,43) sigue en el centro de la burbuja, que por eso se dibuja centrada en el ancla (excepción de §3.1), y las anclas de la entrada (50,0) y de la salida lateral (100,10) son las del catálogo. «RemoveNumber» no cabe en una linea del cuerpo, asi que va en dos («Remove» y «Number»), la zona de rotulo sube a 24 px de alto por encima de la linea de la lista, y la cifra conserva su sitio y su ancho.

## RemoveTrans
- family: Grupo
- variant: cuerpo achaflanado con burbuja
- footprint: 100x50
- ports: Entrada1@50%,0% ; Salida1@50%,86% ; Salida2@100%,20%
- path: M0 10 L10 0 H90 L100 10 V36 H0 Z M43 43 A7 7 0 1 1 57 43 A7 7 0 1 1 43 43 Z
- details: M8 31 H92 ; M14 35 L26 32
- label: 8,6,40,24
- figure: 67,16,30,11
- note: Variante «cuerpo achaflanado con burbuja»: el mismo cuerpo de RemoveMe con una sola raya diagonal bajo la línea, la transacción concreta que sale del grupo; la entrada sube por el borde superior (50,0), la salida lateral por el borde derecho (100,10) y la otra salida queda en el centro de la burbuja, dibujada centrada en el ancla (excepción de §3.1). «RemoveTrans» no cabe en una linea del cuerpo, asi que va en dos («Remove» y «Trans»), la zona de rotulo sube a 24 px de alto por encima de la linea de la lista, y la cifra conserva su sitio y su ancho.

## Assemble
- family: Grupo
- variant: embudo
- footprint: 88x32
- ports: Entrada1@50%,0% ; Salida1@50%,100%
- path: M0 4 L4 0 H84 L88 4 L72 32 H16 Z
- details: none
- label: 12,13,48,12
- figure: 52,1,26,11
- note: Variante «embudo» de la familia Grupo: muchas entradas entran por el borde superior (44,0) y el lote sale por el borde inferior (44,32), y el cuerpo se estrecha hacia abajo como en la cuña del editor original; con 32 px de alto no cabe subtítulo, así que va sólo el rótulo, y el ancho útil (76 px) da para la partición izquierda/derecha. El embudo da 48 px de ancho util en su banda media, y la cifra sube a la banda alta, a la derecha, con 26 px (los de su texto), para no pisar el puerto de salida.

## Gather
- family: Grupo
- variant: embudo invertido
- footprint: 88x32
- ports: Entrada1@50%,0% ; Salida1@50%,100%
- path: M14 0 H74 L88 28 L84 32 H4 L0 28 Z
- details: none
- label: 11,9,38,12
- figure: 49,13,30,11
- note: Variante «embudo invertido» de la familia Grupo, el gemelo de Assemble: el grupo se reúne en la base ancha y la entrada (44,0) queda en el borde superior corto; con 32 px de alto sólo cabe el rótulo, y el texto y la cifra se reparten en la banda ancha del cuerpo, por encima de la salida de abajo. El rotulo se ensancha a 38 px para que «Gather» (32,4 px) quepa entero, y la zona de cifra conserva su sitio y su ancho.

## Match
- family: Grupo
- variant: cuerpo con mejilla
- footprint: 80x48
- ports: Entrada1@50%,0% ; Salida1@50%,100%
- path: M0 8 L8 0 H72 L80 24 L72 48 H8 L0 40 Z
- details: none
- label: 4,8,40,22
- figure: 46,18,30,11
- note: Variante «cuerpo con mejilla» de la familia Grupo: el cuerpo hexagonal con la mejilla derecha en punta, que es el triángulo que el original dibujaba sobre el borde derecho para el cotejo de rótulos; la entrada (40,0) y la salida (40,48) caen en los bordes superior e inferior, y el ancho útil (68 px) da para la partición izquierda/derecha.

## Scan
- family: Grupo
- variant: caja sobre cuña
- footprint: 80x50
- ports: Entrada1@50%,0% ; Salida1@25%,80% ; Salida2@75%,80%
- path: M0 6 L6 0 H74 L80 6 V30 L40 50 L0 30 Z
- details: none
- label: 6,7,38,20
- figure: 48,12,30,11
- note: Variante «caja sobre cuña» de la familia Grupo: la caja con las esquinas superiores achaflanadas sobre la cuña del original, y las dos rectas de la cuña pasan exactamente por las anclas de salida (20,40) y (60,40), de modo que los puntos de puerto caen SOBRE la arista y no flotan dentro del relleno (en un hexágono de familia quedarían dentro); la entrada baja por (40,0).

## Examine
- family: Grupo
- variant: caja sobre cuña
- footprint: 80x48
- ports: Entrada1@50%,0% ; Salida1@25%,83% ; Salida2@75%,83%
- path: M8 0 H72 L78 6 V32 H80 L40 48 L0 32 H2 V6 Z
- details: none
- label: 3,8,42,22
- figure: 46,10,30,11
- note: Variante «caja sobre cuña»: la misma horquilla de Scan, con la caja más ancha y sin detalle en la cuña; la cuña se dimensiona para que sus dos rectas pasen justo por (20,40) y (60,40), las anclas de salida del catálogo, así que los puertos quedan sobre la arista y no dentro del relleno. El rotulo se ensancha a 42 px para que «Examine» (37,8 px) quepa entero en una linea.

## ExamineNumber
- family: Grupo
- variant: caja sobre cuña
- footprint: 80x48
- ports: Entrada1@50%,0% ; Salida1@25%,83% ; Salida2@75%,83%
- path: M8 0 H72 L78 6 V32 H80 L40 48 L0 32 H2 V6 Z
- details: M36 40 V46 M40 40 V46 M44 40 V46
- label: 3,8,43,24
- figure: 46,10,30,11
- note: Variante «caja sobre cuña», igual que Examine, con las tres rayas de conteo dentro de la cuña (el número que se examina, no la transacción); los anclajes (20,40) y (60,40) siguen sobre las rectas de la cuña, que es lo que exige §3.1. «ExamineNumber» no cabe en una linea de la caja, asi que va en dos («Examine» y «Number») y la zona de rotulo sube a 24 px de alto, por encima de la cuña.

## Alter
- family: Grupo
- variant: hexagono con elipse lateral
- footprint: 100x50
- ports: Entrada1@40%,0% ; Salida1@10%,100% ; Salida2@70%,100%
- path: M0 25 L10 0 H70 L80 25 L70 50 H10 Z M80 25 A10 10 0 1 1 100 25 A10 10 0 1 1 80 25 Z
- details: none
- label: 8,10,34,22
- figure: 46,18,30,11
- note: Variante «hexagono con elipse lateral» de la familia Grupo: el hexágono del original con la burbuja del valor nuevo a la derecha; la entrada baja por el borde superior (40,0) y las dos salidas salen por los dos vértices inferiores (10,50) y (70,50); la elipse lateral es la firma de la clase y no lleva texto, así que el rótulo va a la izquierda y la cifra a la derecha dentro del hexágono, y la cifra baja porque a la altura del rótulo el hexágono se estrecha y no da 60 px de ancho.

## Select
- family: Selección
- variant: embudo
- footprint: 100x50
- ports: Entrada1@50%,0% ; Salida1@50%,100%
- path: M24 0 H72 L100 25 L72 50 H24 V34 A24 9 0 0 1 24 16 Z
- details: none
- label: 26,9,54,21
- figure: 39,33,30,11
- note: Variante «embudo» de la familia Selección: la boca elíptica izquierda es el depósito de miembros y la punta derecha es lo que sale elegido; la entrada baja por el borde superior (50,0) y la salida sale por el inferior (50,50); el rótulo y la cifra no caben en el mismo lado del embudo, así que van apilados en la misma columna (rótulo arriba, cifra abajo), como manda §2.1.

## SelectSNA
- family: Selección
- variant: embudo
- footprint: 100x50
- ports: Entrada1@50%,0% ; Salida1@50%,100% ; Salida2@100%,50%
- path: M24 0 H72 L100 25 L72 50 H24 V34 A24 9 0 0 1 24 16 Z
- details: M18 25 A6 6 0 1 1 6 25 A6 6 0 1 1 18 25 Z
- label: 26,9,54,21
- figure: 39,33,30,11
- note: Variante «embudo»: el mismo embudo con un círculo en la boca, el elemento vivo, la clase de entidad que se elige; la punta derecha (100,25) es la segunda salida que declara el catálogo; el rótulo y la cifra van apilados en la misma columna y la cifra es el par de siempre (dentro/pasadas), no un número de entidad.

## SelectL
- family: Selección
- variant: embudo
- footprint: 100x50
- ports: Entrada1@50%,0% ; Salida1@50%,100% ; Salida2@100%,50%
- path: M24 0 H72 L100 25 L72 50 H24 V34 A24 9 0 0 1 24 16 Z
- details: M18 25 A6 6 0 1 1 6 25 A6 6 0 1 1 18 25 Z ; M12 25 L16 20
- label: 26,9,54,21
- figure: 39,33,30,11
- note: Variante «embudo»: el embudo con el círculo y la palanca del interruptor en la boca; la punta derecha (100,25) es la salida alternativa. Su cifra es el par de siempre (dentro/pasadas) y NO el valor del interruptor: SelectL no declara la entidad que elige (§6.1), así que su cifra por entidad es irresoluble sin una propiedad nueva en el modelo, fuera de este rediseño.
