# Contrato de diseño de las formas de bloque (v2)

Este contrato es la ley del rediseño. Los grupos de clases lo respetan al pie
de la letra: sin esto, los grupos no encajarían entre sí y el lienzo quedaría
un mosaico.

> **La v2 corrige la v1** con los diez bloqueantes que encontró la auditoría
> (`auditoria.md`). Lo que cambia está marcado con **[v2]**. La v1 decía
> "Decisión = rombo" y "rótulo a la izquierda, cifra a la derecha" sin
> excepción: ambas cosas son falsas para 8 de 10 clases de Decisión y para
> 15 clases de 32 px de alto, y por eso los grupos se desviaran.
>
> **[v4] La v4 añade §2.3 y §2.4**: el nombre de la clase se escribe **entero**
> (en una línea o en dos, nunca cortado con puntos suspensivos) y las zonas se
> comprueban **dentro de la silueta rellena**, no dentro de su huella. El
> generador deja de escribir el módulo cuando una de las 66 clases no cumple
> la regla.
>
> **[v5] La v5 añade §2.5**: ninguna zona puede **cruzar un trazo interior**
> de su clase. El generador lo comprueba y deja de escribir el módulo cuando
> una zona se pone encima de un trazo: el texto de la zona se dibujaría
> encima del trazo y no se leería.

## 1. Qué se rediseña

La **silueta** de cada clase de bloque (el contorno que la identifica), la
**posición de sus puertos** (dónde entra y por dónde sale), y la
**composición del texto** (dónde va el rótulo, dónde la cifra, para que nunca
se pisen).

Lo que **no** cambia: la huella del bloque (`width`/`height`), la composición
del lienzo, el motor, la simulación y las cifras.

## 2. Las tres zonas de un bloque

Todo bloque se divide en tres zonas, y el diseño las respeta:

```
┌─────────────────────────────────────────────┐
│  ENTRADA          (el puerto de entrada)     │  ← borde superior
│                                             │
│  ┌───────────────────────────────────────┐  │
│  │  ZONA DE TEXTO      │  ZONA DE CIFRA  │  │
│  │  rótulo + título    │  la cifra viva  │  │
│  │  (izquierda)        │  (derecha)      │  │
│  └───────────────────────────────────────┘  │
│                                             │
│  SALIDAS          (los puertos de salida)   │  ← borde inferior / laterales
└─────────────────────────────────────────────┘
```

- **Zona de texto**: a la izquierda. Lleva el nombre de la clase (la
  `caption` del bloque, o el nombre de la clase si no tiene) y, debajo, el
  subtítulo. El nombre se escribe **entero** (§2.3): en una línea si cabe en
  la zona, y en dos si no cabe. Nunca se corta con puntos suspensivos y nunca
  desborda la silueta.
- **Zona de cifra**: a la derecha. Lleva la cifra viva de la familia. Es
  monoespaciada y del color de la familia. Si no hay cifra (la corrida no
  alcanzó el bloque, o la clase es una declaración sin entidad), la zona
  queda **vacía**: el espacio se reserva igual, para que el rótulo no se mueva
  cuando aparece la cifra.
- **Zona de puertos**: en el borde. El texto **nunca** invade el borde donde
  hay un puerto: un puerto es un punto de la silueta por el que entra o sale
  el flujo, y el flujo manda.

### [v2] 2.1 — Cuando la silueta es estrecha, las zonas se apilan

**[v2]** La partición izquierda/derecha **sólo** cabe si el ancho útil de la
silueta es de **60 px o más**. Por debajo de 60 px de ancho útil:

- el **rótulo** ocupa la banda superior;
- la **cifra** ocupa la banda inferior;
- ambas en la **misma columna**, centradas.

Afecta a las siluetas de la familia Decisión (el rombo de 64×64 tiene una caja
inscrita de 32×32), a las de 50–72 px de ancho y a las muy planas
(`Multifurcacion`, 80×20).

La regla general que lo resume: **el texto y la cifra se colocan donde caben,
en el orden texto-arriba / cifra-abajo cuando no caben en el mismo lado.** Lo
que nunca se permite es que se pisen entre sí o que tapen un puerto.

### [v2] 2.2 — El subtítulo sólo cabe en bloques de 40 px de alto o más

**[v2]** Un bloque de 32 px de alto tiene **20 px** de interior utilizable
(32 − 6 de los puntos de puerto). El rótulo y el subtítulo juntos necesitan
22 px. Por tanto:

- en bloques de **40 px o más** de alto: rótulo y subtítulo, dos líneas;
- en bloques de **menos de 40 px** de alto: **sólo el rótulo**, en una línea.

Afecta a 15 clases con puerto arriba y abajo (`Advance`, `Buffer`, `Queue`,
`Depart`, `Seize`, `Release`, `Return`, `Enter`, `Leave`, `Logic`, `Loop`,
`Preempt`, `Gather`, `Assemble`, `Split`) y a las dos `Multifurcacion` de
20 px.

### 2.3 — El nombre de la clase se escribe entero

El nombre de la clase es lo que identifica al bloque, y **no se corta nunca**
con puntos suspensivos: un bloque que dice «Genera…» no dice qué es. El
nombre se escribe **entero** dentro de su propia silueta:

- en **una línea**, si el ancho del nombre cabe en la zona de texto; o
- en **dos líneas**, si no cabe.

La medida es la del diseño, y es la misma que mide la app
(`src/graph/catalogue.ts`): el nombre va en la **monoespaciada de 9 px**, que
mide **0,6 em por carácter**.

| Medida del nombre | Valor |
|---|---|
| Tipografía | monoespaciada (0,6 em por carácter) |
| Tamaño | 9 px |
| Ancho de un nombre de `n` caracteres | `n × 0,6 × 9` px |
| Aire que el texto deja dentro de su zona | 4 px (2 a cada lado) |
| Alto de una línea | 11 px |
| Aire entre dos líneas | 2 px |
| Alto de una zona con dos líneas | 24 px (11 + 2 + 11) |

**La regla que el generador hace cumplir.** Para cada una de las 66 clases:

- el ancho del nombre es `≤ ancho de la zona de texto − 4` (el nombre cabe en
  una línea), **o**
- el nombre se puede partir en dos líneas, cada una de `≤ ancho de la zona −
  4`, y la zona de texto mide **24 px de alto o más**.

El generador **no escribe** el módulo si una clase no cumple la regla, y
entrega el nombre de la clase, el ancho que el nombre necesita y el ancho que
la zona tiene.

**Dónde cae el corte.** El corte lo pone la app, y es el mismo en las 66
clases:

1. en un **espacio**, si el nombre tiene uno y las dos mitades caben en la
   zona (el espacio se queda con la línea a la que pertenece);
2. si no, donde una **mayúscula empieza una palabra**: los nombres de las
   clases van en camel case, y la mayúscula es donde el nombre dice que
   acaba una palabra. «RemoveNumber» se parte en «Remove» y «Number», y no
   en «Remov» y «eNumber»;
3. si el nombre no tiene ni espacios ni mayúsculas, se parte **dentro de la
   palabra**, en el punto que equilibra mejor las dos líneas: es lo que pasa
   con `Multifurcacion` y `MultifurcacionP`, que son una sola palabra.

**La zona tiene que dar para ese corte.** El generador comprueba, además de
que el nombre cabe, que **el corte en la mayúscula quepa en la zona**: un
nombre partido dentro de una palabra («Remov» y «eNumber») es un nombre
cortado, y el nombre de la clase se lee entero. Los nombres que son una sola
palabra («Multifurcacion`) no tienen mayúscula, y en los que la tienen al
final («MultifurcacionP») es un sufijo: en los dos casos no hay nada que
exigir, y el nombre se parte dentro de la palabra.

Un nombre que ocupa dos líneas **no deja sitio para el subtítulo**: el nombre
de la clase importa más que su subtítulo.

### 2.4 — Las zonas viven dentro de la silueta

La zona de texto y la zona de cifra están **dentro de la silueta rellena**,
no dentro de su huella: el texto se dibuja ahí, y el texto no puede salirse
de la figura ni pisar un puerto. El generador lo comprueba sobre el `path` de
cada clase, no sobre su caja.

### [v5] 2.5 — Ninguna zona cruza un trazo interior

**[v5]** Los detalles interiores de una clase (la linea de la cabecera de una
tabla, la linea de nivel de un deposito, el rail del parametro de una
multifurcacion, la elipse de candidatos de un `SelectF`) son **trazos** que
se dibujan dentro de la silueta. El texto de una zona se dibuja **encima** de
la zona, asi que una zona que cruza un trazo pone el texto encima del trazo:
un subtitulo sobre la linea de la tabla, un rotulo sobre el rail, un nombre
sobre la elipse de los candidatos.

La regla que el generador hace cumplir, para cada una de las 66 clases y para
**las dos** zonas (la de texto y la de cifra):

- el **rectangulo** de la zona no toca ningun trazo interior de su clase;
- y deja **2 px de aire** entre su borde y el trazo mas cercano.

El generador **no escribe** el modulo si una clase no cumple la regla, y
entrega la clase, la zona y el trazo que la zona cruza.

**El aire de 2 px es el del texto, no el de una medida decorativa**: el trazo
se dibuja con 1,8 px de grosor, y dos pixeles de aire son los que hacen que
el texto y el trazo se lean los dos.

**La unica excepcion declarada es `Unlink`.** Su capsula mide 38 px de ancho
y el nombre entero («Unlink», 32,4 px) necesita una zona de 36,4 px: con los
2 px de aire que el eslabon (x=38) exige, la zona mas ancha que cabe mide
36 px, y el nombre se partiria dentro de la palabra («Unl» / «ink»). Se
conserva el **nombre entero** —el texto acaba a 2,6 px del eslabon— y la
zona **no lo cruza**. Es el unico caso de las 66 en que la zona llega al
eslabon sin tocarlo, y esta declarado en el generador (`EXCEPCIONES_TRAZO`)
y en la prueba de la app.

**Lo que la regla movio.** Doce clases tenian una zona pegada o encima de un
trazo interior; la zona se movio, y **la silueta, los puertos y los colores
no se tocaron**:

| Clase | El trazo | Lo que se movio |
|---|---|---|
| `Table`, `QTable` | la linea de la cabecera (y=20) | la linea baja a y=32 y la banda de cabecera crece |
| `SelectF` | la elipse de los candidatos (x=1..16) | la zona de texto arranca en x=18 y la de cifra baja a la banda de abajo |
| `Clock` | las dos marcas de esfera | la zona de texto se estrecha a 34 px |
| `Counter` | las tres rayas del pie | la zona de texto sube a y=4 |
| `MFunction` | la curva mas alta de la grafica | su zona de cifra desaparece (§2.7) |
| `MFunctionDist` | los dos aros concentricos | su zona de cifra desaparece (§2.7) |
| `Note` | el pliegue de la esquina | la zona de texto se estrecha a 45 px |
| `Preempt` | la marca de corte | la zona de cifra termina en x=74 |
| `StorageUse` | la linea de nivel | la zona de texto sube a y=6 |
| `RemoveNumber`, `RemoveTrans` | la linea de la lista | la zona de texto sube a y=4 |

Y una clase mas se movio por el nombre entero (§2.6): los dos peinees, cuyo
rail y perilla del parametro pasan a la corona libre que queda entre el
rotulo y la cifra.

### [v5] 2.6 — El peine crece, y el nombre entero cabe en una linea

**[v5]** El lector miro las pictures de verdad y pidio que el nombre de las
dos multifurcaciones **cabe en una linea** de la corona del peine. El nombre
mide 75,6 px (`Multifurcacion`) y 81 px (`MultifurcacionP`), asi que la zona
de texto necesita 80 px y 86 px.

La salida es la del propio diseno: **el peine crece** y su huella pasa de
80x40 a **144x40**. Los cinco dientes se separan 36 px en vez de 20 (y siguen
siendo los cinco puntos donde el flujo sale: 0, 36, 72, 108 y 144), la
corona crece con ellos, el rotulo va a la izquierda (86 px) y la cifra a la
derecha (30 px). Los dos nombres caben enteros en una linea, y el motor
publica la huella nueva: un bloque nuevo de esas clases nace con ella.

### [v5] 2.7 — Una clase que no puede tener cifra no reserva zona de cifra

**[v5]** El lector pregunto que significaba el «...» de algunos bloques (el
titulo del bloque, cortado) y acepto este criterio:

> **Mostrar la cifra cuando el motor la publica; nunca inventar una; y no
> dejar un hueco reservado que parezca un error.**

La tercera parte del criterio es una regla del diseno. Una **zona de cifra
reservada que nunca se puede llenar** es un hueco vacio en el bloque: parece
un fallo, y ademas roba ancho al rotulo (que es justo lo que se estaba
cortando).

**Que clases pueden tener cifra.** La cifra sale del motor: la fila del
reporte de un bloque, o el valor que publica una entidad. Un bloque que el
motor **nunca recorre** no tiene fila ni valor: no hay nada que mostrar.

**La regla que el generador hace cumplir.** Una clase de la lista de las
**nueve que no pueden tener cifra** declara `figure: null`:

| Clase | Por que no puede tener cifra |
|---|---|
| `Function`, `FunctionExpon`, `MFunction`, `FunctionDist`, `MFunctionDist` | son declaraciones que el motor nunca recorre: no hay fila del reporte para ellas y no hay valor que publicar |
| `Note`, `StorageDef`, `Table`, `QTable` | lo mismo: son declaraciones, y ya declaraban `figure: null` |

**Las tres variables NO estan en la lista.** `Variable`, `VariableA` y
`VariableN` son declaraciones, pero el motor publica el valor de una variable
cuando la corrida se lo asigna (la tabla `SAVEVALUE` del reporte) y la app lo
muestra: su zona de cifra se queda. Cuando la corrida nunca se lo asigna, la
app dibuja un guion tenue (otra unidad de trabajo, en la app), y no un hueco.

El generador **no escribe** el modulo si una de las nueve declara zona de
cifra, y entrega la clase y el motivo:

```
Function: declara zona de cifra (34,32,32,14) y es una clase que no puede
tener cifra: el motor nunca la recorre y no publica ningun valor para ella,
asi que la zona quedaria siempre vacia
```

**La lista vive en el modulo.** El generador la escribe en
`crates/genmsi-app/src/graph/shapes.ts` (`CLASSES_WITHOUT_FIGURE`), y la
prueba de la app la lee de ahi en vez de repetirla: hay una sola fuente de la
verdad. La prueba comprueba las dos direcciones: las nueve declaran `figure:
null` y **todas las demas** declaran zona de cifra.

**Lo que la regla movio.** Las cinco funciones pierden su zona de cifra
(`Function`, `FunctionExpon`, `MFunction`, `FunctionDist`, `MFunctionDist`):
sus zonas de texto no cambian, porque ya ocupaban todo el ancho que les
daba la silueta (84 px en la tarjeta de 100, la banda central del circulo
en las otras dos).

## 3. La silueta

La silueta se declara como una **geometría paramétrica**: una función que,
dada la huella (`w` × `h`), devuelve el `d` de un `<path>` SVG. La misma
función dibuja el bloque del lienzo y la miniatura de la lista, así que nunca
pueden discrepar.

Reglas:

1. **[v2]** **La silueta toca el borde donde están sus puertos.** Si un puerto
   está en el vértice derecho de un rombo, el rombo tiene que tener un
   vértice derecho en ese punto. La forma y el flujo son lo mismo.
   - **[v2] Excepción única**: cuando el ancla del editor original cae en el
     **interior** de una burbuja (`JoinMe` y `JoinNumber` con su entrada en
     (50,7); `RemoveMe`, `RemoveNumber` y `RemoveTrans` con su salida en
     (50,43)), la burbuja se dibuja **centrada en el ancla** y el puerto
     queda dentro de ella. Es la verdad del original, y la única excepción a
     esta regla.
   - **[v2]** La regla se evalúa **contra la variante que nombra la clase**,
     no contra una silueta ideal de la familia (ver 5.1).
2. **[v2]** **La silueta se adapta a la huella.** Un rombo en 64×64 es un
   rombo; la misma variante en 120×40 se estira en horizontal sin deformarse
   (usa coordenadas relativas, no absolutas).
3. **Nada de texto dentro de la silueta**: la silueta es el fondo; el texto va
   encima, en las zonas de la regla 2.
4. Los detalles interiores (las rayas de una tabla, las marcas de una cola,
   la elipse de un grupo) son trazos **sin relleno**, del color de la familia,
   con 1,5–2 px.

## 4. Los puertos

Cada puerto se coloca en el punto que dice la verdad del editor original
(`puertos.md`), no repartido por igual. Esa tabla es la autoridad; si una
clase no está en ella, se deduce de su silueta y se dice así en la entrega.

- El ancla se normaliza a la huella (0–1) para que sirva en cualquier
  tamaño.
- **[v2]** Un puerto de salida lateral (a la izquierda o a la derecha) exige
  una silueta que tenga ese borde: un rombo con salida a la derecha tiene su
  salida en el vértice derecho, y la figura se dibuja a la derecha de esa
  línea, no en el centro.
- Los puertos **deben verse**: un punto de 8–10 px del color de la familia,
  con un anillo blanco de 2 px para que se lea sobre el fondo del bloque.
  El punto ocupa 12 px con su anillo, y por eso el interior del bloque
  pierde 6 px por cada lado con puerto.

## 5. El color y las variantes

El color lo da la **familia**, y la familia la da lo que el bloque hace:

| Familia | Color | Clases |
|---|---|---|
| Fuente | `#047857` | `Generate`, `Clock` |
| Destino | `#be123c` | `Terminate` |
| Decisión | `#4338ca` | `Test`, `Transfer`, `GateF`, `GateL`, `GateM`, `GateS`, `Loop`, `Multifurcacion`, `MultifurcacionP`, `Split` |
| Cola | `#b45309` | `Queue`, `Depart` |
| Recurso | `#0f766e` | `FacilityUse`, `FacilityInt`, `Seize`, `Release`, `Preempt`, `Return`, `SelectF` |
| Depósito | `#0369a1` | `StorageDef`, `StorageUse`, `Enter`, `Leave`, `SelectS` |
| Tabla | `#6d28d9` | `Table`, `Tabulate`, `QTable` |
| Cadena | `#c2410c` | `Link`, `Unlink` |
| Grupo | `#be185d` | `JoinMe`, `JoinNumber`, `RemoveMe`, `RemoveNumber`, `RemoveTrans`, `Assemble`, `Gather`, `Match`, `Scan`, `Alter`, `Examine`, `ExamineNumber` |
| Selección | `#0e7490` | `Select`, `SelectSNA`, `SelectL` |
| Operación | `#475569` | `Advance`, `Assign`, `Buffer`, `Mark`, `Counter`, `Priority`, `SaveValue`, `InitiaSV`, `Logic`, `InitialLS`, `Variable`, `VariableN`, `VariableA`, `Function`, `FunctionDist`, `FunctionExpon`, `MFunction`, `MFunctionDist` |
| Anotación | `#64748b` | `Note` |

### [v2] 5.1 — Una familia es una familia de variantes

**[v2]** La familia no es **una** silueta: es un **lenguaje** de siluetas.
Cada clase nombra su variante, y la regla 3.1 se evalúa contra esa variante.

El caso que lo fuerza: **la familia Decisión no puede ser un rombo único.**
Sólo `Test` y `Transfer` son un rombo exacto. Las anclas reales de las otras
ocho caen fuera de un rombo centrado:

| Clase | Ancla | Fuera del rombo |
|---|---|---|
| `GateF` (112×48) | entrada y salida en x=38 | +7,7 px |
| `GateL`/`GateM`/`GateS` (102×48) | entrada y salida en x=38 | +6,1 px |
| `Loop` (96×32) | entrada y salida en x=57 | +3,0 px |
| `Multifurcacion(P)` (80×20) | 4 de sus 5 salidas | hasta +10 px |
| `Split` (88×32) | sus dos salidas, en x=0 y x=88 | +16 px |

Y en la familia **Grupo**, las salidas de `Scan`, `Examine` y `ExamineNumber`
caen sobre la arista de la cuña original, dentro de un hexágono de familia.

Por eso el grupo C ya 제안 cinco variantes dentro de Decisión (rombo, rombo
con accesorio, cuerpo con punta a la izquierda, peine de la multifurcación,
horquilla del split) y una variante "caja sobre cuña" en Grupo. **Esas
variantes son la respuesta correcta**: la familia se conserva (el color y el
lenguaje siguen siendo los mismos) y cada clase nombra la suya.

**[v2]** La entrega nombra la variante en la línea `note` de cada clase.

### [v2] 5.2 — Una clase que cambia de familia

**[v2]** El grupo A propone sacar `Note` de la familia Operación a una
familia «Anotación» propia (gris), porque una anotación no es un paso del
flujo. La propuesta **se acepta**: `Note` pasa a la familia «Anotación»,
gris `#64748b`, con su silueta de hoja con la esquina doblada y sin puerto
alguno. La familia «Anotación» es la doceava, y sólo contiene `Note`.

## 6. La cifra viva

**[v2]** La cifra sale de la **entidad con la que opera el bloque**, que el
modelo declara en sus propiedades. **Una sola cifra por bloque**, no dos:
la información que no cabe, vive en el Reporte.

| Bloques | Cifra | Formato |
|---|---|---|
| `Generate` | flechas generadas | `1.842` |
| `Terminate` | transacciones terminadas | `1.842` |
| `Queue`, `Depart` | contenido de la cola | `7` |
| `Seize`, `Release`, `Return`, `Preempt`, `FacilityUse`, `FacilityInt` | % de ocupación del recurso | `82%` |
| `Enter`, `Leave`, `StorageUse` | unidades / capacidad | `12/40` |
| `Tabulate` | nº de muestras | `36` |
| `Link`, `Unlink` | tamaño de la cadena | `3` |
| `Logic`, `InitialLS` | el valor del interruptor | `on` |
| `SaveValue`, `InitiaSV`, `Counter` | el valor que guarda | `12` |
| todo lo demás | dentro / pasadas | `2/15` |

### [v2] 6.1 — Tres clases que no pueden mostrar la cifra de su entidad

**[v2]** `SelectF`, `SelectS` y `SelectL` **no declaran la entidad**:
`SelectF` declara `Param`, `maximo`, `minimo` y `op`; no hay propiedad
`Facility`. El motor no puede resolverlo tampoco: estas clases **eligen** una
entidad (la primera facility libre, un depósito según su estado), y no
publican su nombre. Sin una propiedad nueva en el modelo, su cifra por
entidad es **irresoluble**.

Por tanto las tres quedan en **«todo lo demás»**: muestran el par de
siempre. (La cifra por entidad de una selección es un cambio de motor y de
formato, fuera del alcance de este rediseño.)

### [v2] 6.2 — «Miembros del grupo» no existe

**[v2]** El motor publica `facilities`, `storages`, `queues`, `chains`,
`logic`, `savevalues`, `tables` y `qtables`. **No hay grupos de
transacciones**: nadie cuenta ni publica los miembros de un grupo. La fila
que decía «miembros del grupo y los elegidos» para `Select` y `SelectSNA`
**cae**, y esas dos clases quedan en «todo lo demás».

### [v2] 6.3 — El formato de la cifra, y su ancho mínimo

**[v2]** La cifra es **una línea** de monoespaciada de 10–11 px. Su zona
reservada:

- si la cifra **no cabe** en el ancho disponible, **se trunca** con `…` en
  vez de desbordarse. **Nunca** desborda la silueta;
- si la cifra va **apilada en dos líneas** (el par `2` sobre `15`), ocupa
  **22 px** de alto.

**[v3] El ancho mínimo no es fijo: depende de lo que la clase muestra.** El
ancho de la zona es el del **texto de su cifra** más 2 px de aire (la
monoespaciada del app mide 0,6 em por carácter):

| Cifra | Caracteres | Ancho de la zona |
|---|---|---|
| `12/40` (depósitos) | 5 | 32 px |
| `1.842` (fuente, destino) | 5 | 32 px |
| `2/15` (el par de siempre) | 5 | 32 px, o 14 px apilada en dos líneas |
| `3` (cadena), `7` (cola), `82%` (recurso), `36` (tabla) | 1–3 | 10–20 px |

El mínimo fijo de 30 px de la v2 era una regla deهلك para todas las clases:
obligaba a `Link` y `Unlink` (cuya cifra es un solo número) a una zona de 30
px dentro de una cápsula de 28, que no cabe. La regla de la v3 es la misma
con la cifra correcta: la zona guarda **su** texto.

El par de siempre (`dentro/pasadas`) se escribe **apilado en dos líneas**
cuando la zona tiene menos de 32 px de ancho, y en una línea (`2/15`) cuando
tiene 32 px o más.

## 7. Qué se entrega

Cada grupo entrega **un archivo de texto** con la especificación de sus
clases, en este formato exacto:

```
## <ClassName>
- family: <familia>
- variant: <el nombre de la variante dentro de la familia>
- footprint: <w>x<h>
- ports: <nombre>@<x%>,<y%> ; <nombre>@<x%>,<y%>
- path: <el `d` del SVG, en coordenadas de la huella>
- details: <los trazos interiores, separados por `;`>
- label: <x>,<y>,<ancho>,<alto>   (la zona de texto)
- figure: <x>,<y>,<ancho>,<alto>  (la zona de la cifra; `none` si no hay)
- note: <una línea: por qué esta silueta, y de dónde sale el ancla de los puertos>
```

Y además **un SVG renderizable** con la familia entera: cada clase dibujada
en su huella real, con sus puertos, su rótulo y su cifra de ejemplo, para que
se pueda **mirar** el resultado y no sólo leerlo.

## 8. Reglas de la entrega

- El SVG se renderiza y se **mira**. Una silueta que sólo se lee en el
  código no está revisada.
- Ningún texto se sale de su silueta, y ningún texto se pisa con un puerto.
- La cifra nunca se pisa con el rótulo, ni al revés.
- La silueta es coherente con su familia: dos clases de la misma familia se
  parecen de lejos, aunque cada una nombre su variante.
- Si una clase no encaja en su familia, se propone el cambio de familia y se
  justifica.
- Se responde en español, como el resto del diseño.

## 9. Lo que se lee antes de escribir

1. Este contrato.
2. `odd/tasks/forma-bloques-diseno/puertos.md` — la verdad de los puertos.
3. `odd/tasks/genmsi-forma-bloques.md`, hasta `## Log` — la especificación
   de la feature.
