# Contrato de diseño de las formas de bloque (v2)

Este contrato es la ley del rediseño. Los grupos de clases lo respetan al pie
de la letra: sin esto, los grupos no encajarían entre sí y el lienzo quedaría
un mosaico.

> **La v2 corrige la v1** con los diez bloqueantes que encontró la auditoría
> (`auditoria.md`). Lo que cambia está marcado con **[v2]**. La v1 decía
> "Decisión = rombo" y "rótulo a la izquierda, cifra a la derecha" sin
> excepción: ambas cosas son falsas para 8 de 10 clases de Decisión y para
> 15 clases de 32 px de alto, y por eso los grupos se desviaran.

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
  subtítulo. Se **trunca** con puntos suspensivos; nunca desborda la
  silueta.
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
