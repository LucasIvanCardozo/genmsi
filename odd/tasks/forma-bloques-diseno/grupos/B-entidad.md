# Grupo B - la familia de la entidad

Siluetas de las 17 clases que operan **sobre una entidad con nombre**: un
recurso (algo que se toma), un deposito (algo que se guarda), una tabla (un
histograma) y una cadena (transacciones enlazadas). La cifra viva de cada
bloque es el estado de esa entidad, asi que la silueta evoca a la entidad y
no a la operacion.

Formato segun el contrato (`odd/tasks/forma-bloques-diseno/contrato.md`,
seccion 7, version 2). El companero `B-entidad.svg` dibuja cada clase en su
huella real, con sus puertos en el ancla verdadera y las zonas de rotulo y de
cifra, para poder **mirar** el resultado.

## Las reglas de la v2 que este grupo cumple

- **§2.2 - el subtitulo solo cabe a partir de 40 px de alto.** `Seize`,
  `Release`, `Return`, `Preempt`, `Enter` y `Leave` miden 32 px (20 px de
  interior utilizable con los puntos de puerto) y `StorageDef` 30: llevan
  **solo el rotulo, en una linea**. Las demas, de 40 px o mas, llevan rotulo y
  subtitulo, dos lineas.
- **§2.1 - la particion izquierda/derecha.** Todas las clases de este grupo
  tienen 76 px de ancho util o mas (las mas estrechas, `Return` y `Preempt`,
  con 88 px de huella), de modo que el rotulo va a la izquierda y la cifra a
  la derecha. Donde la silueta no da la talla -el pie de `Preempt`, el
  accesorio de `FacilityInt`, la barra de `Unlink`- la cifra se ha movido de
  sitio, nunca encima del rotulo.
- **§6.3 - el formato de la cifra.** Una linea de monoespaciada de 10 px en
  una zona reservada de **30 px de ancho minimo**; si no cupiera se truncan
  con `…`. En este grupo ninguna cifra necesita dos lineas: todas las zonas
  reservadas miden 30 px de ancho o mas.
- **§6 - una sola cifra por bloque, y la de su entidad**: `% de ocupacion`
  (`82%`) en los recursos, `unidades/capacidad` (`12/40`) en los depositos,
  `nº de muestras` (`36`) en `Tabulate` y `tamano de la cadena` (`3`) en
  `Link` y `Unlink`.
- **§6.1 - `SelectF` y `SelectS` muestran el par de siempre** (`2/15`): no
  declaran entidad y el motor elige la entidad sin publicar su nombre, asi que
  su cifra por entidad es irresoluble.
- **`StorageDef`, `Table` y `QTable` no llevan cifra**: son declaraciones sin
  puertos y sin estado de corrida. La capacidad declarada de un deposito es
  una propiedad del modelo, no de la corrida, asi que un bloque sin estado no
  muestra cifra.
- **§5.1 - cada clase nombra su variante** dentro de su familia, en la linea
  `variant:`.

## El lenguaje de las cuatro familias

| Familia | Color | Variantes | Cifra viva |
|---|---|---|---|
| Recurso | `#0f766e` | estadio con galon, estadio con galon y accesorio, galon cerrado, galon abierto, galon afilado, embudo con galon | % de ocupacion del recurso (`82%`) |
| Deposito | `#0369a1` | barril, barril con nivel, barril con embudo, barril en placa | unidades / capacidad (`12/40`) |
| Tabla | `#6d28d9` | hoja con cabecera, hoja con cabecera y cola | nº de muestras (`36`), solo en `Tabulate` |
| Cadena | `#c2410c` | capsulas enlazadas (espejo) | tamano de la cadena (`3`) |

Reglas de composicion que valen para las cuatro familias:

- **Zona de texto** a la izquierda: el nombre de la clase (negrita de 9 px, o
  de 8,5 px y de 8 px cuando no cabe en su zona) y, si el bloque tiene 40 px o
  mas de alto, debajo el rotulo del bloque (7,5 px). Se trunca con puntos
  suspensivos antes de salirse de la silueta.
- **Zona de cifra** a la derecha, reservada siempre: monoespaciada de 10 px en
  el color de la familia. Si no hay cifra (una declaracion) no se reserva zona
  de cifra y el rotulo ocupa todo el ancho disponible.
- **Zona de puertos**: en el borde. El texto nunca invade el borde donde hay
  un puerto, y el ancla sale de `puertos.md`, no de un reparto igualitario.
- **Declaraciones** (`StorageDef`, `Table`, `QTable`): misma silueta de su
  familia, pero en placa apagada (borde punteado, relleno tenue) y sin puertos.

## Desviaciones declaradas

1. **`Unlink` no lleva subtitulo.** Su barra mide 30 px de alto y los dos
   puertos de entrada y salida la parten en dos: entre los dos puntos de
   puerto (5,5 px de intrusion a cada lado) solo queda una banda de 19 px, y
   en 19 px cabe una linea de rotulo, no dos. La cifra va al extremo derecho
   de la barra, fuera de los tres puertos; el eslabon se dibuja en los dos
   tramos de la union que quedan libres.
2. **`StorageDef` (30 px de alto) no lleva subtitulo**, por la misma regla de
   §2.2 que los bloques de 32 px.
3. **El nombre va a 9 px** y, si no cabe en su zona con 2 px de aire, baja a
   8,5 px (`StorageDef`, `Unlink`) o a 8 px (`StorageUse`); si tampoco cupiera,
   se truncaria con `…`. El subtitulo va a 7,5 px.
4. **`Preempt` afila desde x=80** (y no desde x=74) para que la cifra quepa a
   la izquierda del corte sin pisar el puerto `SalidaAlt` del vertice inferior
   derecho.
5. **`Link` y `Unlink` se redibujan sobre la geometria original**: la barra
   horizontal va de y=10 a y=40 (no de y=0), porque las anclas `Entrada1` y
   `Salida1` caen en esas rectas y la silueta tiene que tocarlas (§3.1).
6. **La boca del barril pasa al extremo derecho** en toda la familia Depósito:
   es donde el editor original dibujaba las marcas de unidades, y el rotulo y
   la cifra tienen que quedar lejos de ahi (§2.1).

## La silueta como funcion (para `src/graph/shapes.ts`)

Cada familia es una funcion que recibe la huella `(w, h)` y devuelve el `d` de
un `<path>` SVG. Los `d` de este documento son esa funcion evaluada en la
huella real de cada clase, para que se puedan leer y dibujar tal cual.

```
recursoCerrado(w, h):        // galon cerrado (Seize, FacilityUse, FacilityInt, SelectF)
  cw = 0.35 * h
  M h/2,0  H w-cw  L w,h/2  L w-cw,h  H h/2  A h/2,h/2 0 0 1 h/2,0 Z
recursoAbierto(w, h):        // galon abierto (Release, Return)
  cw = 0.35 * h ; d = 0.15 * h
  M h/2,0  H w-cw  L w,0.28h  L w-d,h/2  L w,0.72h  L w-cw,h  H h/2  A h/2,h/2 0 0 1 h/2,0 Z
recursoAfilado(w, h):        // galon afilado (Preempt)
  cut = 0.91 * w
  M h/2,0  H cut  L w,h  H h/2  A h/2,h/2 0 0 1 h/2,0 Z
barril(w, h):                // cilindro tumbado (StorageDef, StorageUse, Enter, Leave, SelectS)
  rx = 0.175 * h
  M rx,0  H w-rx  A rx,h/2 0 0 1 w-rx,h  H rx  A rx,h/2 0 0 1 rx,0 Z
hoja(w, h):                  // tabla con cabecera (Table, Tabulate, QTable)
  r = 5 ; cabecera = 0.26 * h (0.32 * h en Tabulate) ; regla de columna en x = 14
  rectangulo redondeado r=5 + banda de cabecera + regla de cabecera y de columna
```

La familia Cadena no se parametriza igual: `Link` y `Unlink` dependen de sus
anclas (la punta de la capsula vertical debe caer en el ancla de `Salida1`, y
la barra tiene que pasar por `Entrada1` y `Salida2`), asi que su geometria se
calcula con las constantes de `puertos.md` y no con proporciones:

- `Link` (98x50): barra horizontal (rectangulo `0..60 x 10..40`, con el
  extremo izquierdo redondeado r=15) + capsula vertical (`60..98 x 0..50`,
  tapas de rx 19 y ry 7). El solape se marca con la linea de eslabon en `x=60`.
- `Unlink` (98x50): capsula vertical izquierda (`0..28 x 0..50`, tapas de
  rx=ry=14, con su punto mas bajo en `14,50`) + barra horizontal
  (`16..98 x 10..40`, extremo derecho redondeado rx 17 ry 15). El solape se
  marca con la linea de eslabon en `x=28`.

Las coordenadas de `path` y `details` estan en el sistema de la huella de cada
clase (0,0 arriba a la izquierda). La funcion que las produce es parametrica:
recibe `w` y `h` y devuelve el `d`, de modo que la misma geometria sirve para
el bloque del lienzo y para la miniatura de la pestana Bloques.

## Las anclas

La linea `ports:` de cada clase reproduce los porcentajes de `puertos.md`. El
SVG dibuja los puntos en los **pixeles exactos** de `gClasses.xml`, que
difieren de esos porcentajes en 1 px o menos (la tabla de `puertos.md` redondea
al 1 %; `puertos.json` trae los pixeles). Las dos fuentes coinciden dentro de
esa tolerancia.


## FacilityUse
- family: Recurso
- variant: estadio con galón
- footprint: 100x40
- ports: Entrada1@50%,0% ; Salida1@50%,100%
- path: M 20,0 H 86 L 100,20 L 86,40 H 20 A 20,20 0 0 1 20,0 Z
- details: M 92,14.5 L 98,20 L 92,25.5
- label: 6,8,51,22
- figure: 59,10,30,12
- note: Estadio de extremo redondeado (el RoundRect 0,0,100,40 del editor original) con el galón cerrado a la derecha: la punta dice 'el recurso queda tomado'. Las anclas salen de gClasses.xml (Entrada1 50,0 y Salida1 50,40), sobre las rectas superior e inferior.


## FacilityInt
- family: Recurso
- variant: estadio con galón y accesorio
- footprint: 120x40
- ports: Entrada1@41%,0% ; SalidaAlt@100%,50% ; Salida1@41%,100%
- path: M 20,0 H 96 L 120,20 L 96,40 H 20 A 20,20 0 0 1 20,0 Z
- details: M 108,13 L 104,20 L 108,27 L 112,20 Z
- label: 6,8,49,22
- figure: 58,10,30,12
- note: El galón cerrado hace de salida lateral: su punta cae en (120,20), el ancla de SalidaAlt de gClasses.xml; el rombo interior conserva el diamante que dibujaba el original. La cifra vive en el cuerpo, a la izquierda del galón, porque en el accesorio está el puerto.


## Seize
- family: Recurso
- variant: galón cerrado
- footprint: 98x32
- ports: Entrada1@35%,0% ; Salida1@35%,100%
- path: M 16,0 H 86 L 98,16 L 86,32 H 16 A 16,16 0 0 1 16,0 Z
- details: M 92,11.5 L 96.5,16 L 92,20.5
- label: 6,9,28,12
- figure: 54,10,30,12
- note: Galón cerrado: la punta derecha dice 'el recurso queda tomado'. Es un bloque de 32 px de alto, asi que lleva sólo el rótulo en una línea (contrato v2 §2.2). Las anclas son las de gClasses.xml (35,0 y 35,32).


## Release
- family: Recurso
- variant: galón abierto
- footprint: 98x32
- ports: Entrada1@35%,0% ; Salida1@35%,100%
- path: M 16,0 H 86 L 98,8.96 L 93.2,16 L 98,23.04 L 86,32 H 16 A 16,16 0 0 1 16,0 Z
- details: M 91,11.5 L 87.5,16 L 91,20.5
- label: 6,9,36,12
- figure: 54,10,30,12
- note: Galón abierto: la muesca en V del extremo derecho dice 'el recurso se suelta'. Bloque de 32 px: sólo rótulo, en una línea (§2.2). Las anclas son las de gClasses.xml (35,0 y 35,32).


## Preempt
- family: Recurso
- variant: galón afilado
- footprint: 88x32
- ports: Entrada1@39%,0% ; Salida1@39%,100% ; SalidaAlt@100%,100%
- path: M 16,0 H 80 L 88,32 H 16 A 16,16 0 0 1 16,0 Z
- details: M 81.6,16.6 L 86.4,15.4
- label: 6,9,42,12
- figure: 49,10,30,12
- note: El cuerpo se afila hacia el vértice inferior derecho, que es el ancla de SalidaAlt (100%,100%) de gClasses.xml; la marca de corte sobre la diagonal cuenta la interrupción. El corte arranca en x=80 para que la cifra quepa a su izquierda sin pisar el puerto. Bloque de 32 px: sólo rótulo (§2.2).


## Return
- family: Recurso
- variant: galón abierto
- footprint: 88x32
- ports: Entrada1@39%,0% ; Salida1@39%,100%
- path: M 16,0 H 76 L 88,8.96 L 83.2,16 L 88,23.04 L 76,32 H 16 A 16,16 0 0 1 16,0 Z
- details: M 81,11.5 L 77.5,16 L 81,20.5
- label: 6,9,34,12
- figure: 44,10,30,12
- note: Galón abierto, igual que Release: soltar y devolver el recurso son la misma operación vista desde los dos lados. Bloque de 32 px: sólo rótulo (§2.2). Las anclas son las de gClasses.xml (35,0 y 35,32).


## SelectF
- family: Recurso
- variant: embudo con galón
- footprint: 100x50
- ports: Entrada1@50%,0% ; Salida1@50%,100% ; Salida2@100%,50%
- path: M 25,0 H 82 L 100,25 L 82,50 H 25 A 25,25 0 0 1 25,0 Z
- details: M 6,25 A 5,10 0 1 0 16,25 A 5,10 0 1 0 6,25 Z ; M 91,20 L 97,25 L 91,30
- label: 18,13,36,22
- figure: 56,13,30,12
- note: La punta del galón cerrado coincide con la salida lateral Salida2 (100%,50%) de gClasses.xml y la elipse de la izquierda conserva la reserva de candidatos del original. SelectF no declara entidad (Param, maximo, minimo, op), asi que su cifra es el par de siempre (contrato v2 §6.1).


## StorageDef
- family: Depósito
- variant: barril en placa (declaración)
- footprint: 100x30
- ports: ninguno (declaración)
- path: M 5.25,0 H 94.75 A 5.25,15 0 0 1 94.75,30 H 5.25 A 5.25,15 0 0 1 5.25,0 Z
- details: M 14,5 H 86 ; M 94.75,0 A 3,15 0 0 0 94.75,30
- label: 8,9,50,12
- figure: none
- note: Placa apagada de la familia Depósito: el barril declara el depósito y no tiene puertos. No lleva cifra: la capacidad es una propiedad del modelo, no del estado de la corrida (contrato v2, corrección 7). Al medir 30 px de alto sigue la regla de §2.2 y muestra sólo el rótulo.


## StorageUse
- family: Depósito
- variant: barril con nivel
- footprint: 100x42
- ports: Entrada1@50%,0% ; Salida1@50%,95%
- path: M 7,0 H 93 A 7,20 0 0 1 93,40 H 7 A 7,20 0 0 1 7,0 Z
- details: M 16,33 H 82 ; M 93,0 A 4.5,20 0 0 0 93,40
- label: 6,9,46,22
- figure: 54,10,32,12
- note: El cilindro se dibuja de 0 a 40 de alto dentro de la huella de 42, porque el ancla de Salida1 (50%,95%) cae en y=40 y debe quedar sobre el borde inferior, como en el editor original (Rectangle 0,0,96,36 + Rectangle 4,4,100,40).


## Enter
- family: Depósito
- variant: barril
- footprint: 96x32
- ports: Entrada1@36%,0% ; Salida1@36%,100%
- path: M 5.6,0 H 90.4 A 5.6,16 0 0 1 90.4,32 H 5.6 A 5.6,16 0 0 1 5.6,0 Z
- details: M 90.4,0 A 3.5,16 0 0 0 90.4,32
- label: 8,9,32,12
- figure: 44,10,32,12
- note: Cilindro tumbado: el original ya dibujaba aquí un extremo cilíndrico, con la boca y las marcas de unidades a la derecha. La cifra va en el centro del barril, lejos de la boca curva (§6.3), y el bloque, de 32 px, lleva sólo el rótulo (§2.2). Anclas 35,0 y 35,32 de gClasses.xml.


## Leave
- family: Depósito
- variant: barril
- footprint: 96x32
- ports: Entrada1@36%,0% ; Salida1@36%,100%
- path: M 5.6,0 H 90.4 A 5.6,16 0 0 1 90.4,32 H 5.6 A 5.6,16 0 0 1 5.6,0 Z
- details: M 90.4,0 A 3.5,16 0 0 0 90.4,32
- label: 8,9,32,12
- figure: 44,10,32,12
- note: Misma silueta que Enter: entrar y egresar del depósito son la misma operación vista desde los dos lados. Cifra en el centro del barril, rótulo sólo (32 px de alto) y anclas 35,0 y 35,32 de gClasses.xml.


## SelectS
- family: Depósito
- variant: barril con embudo
- footprint: 100x50
- ports: Entrada1@50%,0% ; Salida1@50%,100% ; Salida2@100%,50%
- path: M 8.75,0 H 91.25 A 8.75,25 0 0 1 91.25,50 H 8.75 A 8.75,25 0 0 1 8.75,0 Z
- details: M 91.25,0 A 5,25 0 0 0 91.25,50
- label: 12,13,39,22
- figure: 53,13,30,12
- note: El punto más a la derecha del barril es (100,25), el ancla de Salida2 (100%,50%) de gClasses.xml; el resto de anclas, 50,0 y 50,50, caen sobre las rectas. SelectS no declara entidad, asi que su cifra es el par de siempre (contrato v2 §6.1).


## Table
- family: Tabla
- variant: hoja con cabecera (declaración)
- footprint: 100x70
- ports: ninguno (declaración)
- path: M 5,0 H 95 A 5,5 0 0 1 100,5 V 65 A 5,5 0 0 1 95,70 H 5 A 5,5 0 0 1 0,65 V 5 A 5,5 0 0 1 5,0 Z
- details: M 0,20 H 100 ; M 14,0 V 70
- label: 18,3,46,26
- figure: none
- note: Placa apagada de la familia Tabla: hoja con cabecera y columna de índice, como la dibujaba gClasses.xml (Line 0,16,100,16 y Line 10,0,10,65). El nombre va en la banda de cabecera y el rótulo debajo, como en el original. Sin puertos y sin cifra: es una declaración.


## Tabulate
- family: Tabla
- variant: hoja con cabecera
- footprint: 100x50
- ports: Entrada1@50%,0% ; Salida1@50%,100%
- path: M 5,0 H 95 A 5,5 0 0 1 100,5 V 45 A 5,5 0 0 1 95,50 H 5 A 5,5 0 0 1 0,45 V 5 A 5,5 0 0 1 5,0 Z
- details: M 0,16 H 100 ; M 14,0 V 50
- label: 16,18,44,22
- figure: 62,18,30,12
- note: El rótulo va en el cuerpo y no en la cabecera porque Entrada1 está en (50,0), en el borde superior de la banda: el texto nunca invade el borde con puerto. La cifra es el número de muestras (§6). Anclas 50,0 y 50,50 de gClasses.xml.


## QTable
- family: Tabla
- variant: hoja con cabecera y cola (declaración)
- footprint: 130x70
- ports: ninguno (declaración)
- path: M 5,0 H 125 A 5,5 0 0 1 130,5 V 65 A 5,5 0 0 1 125,70 H 5 A 5,5 0 0 1 0,65 V 5 A 5,5 0 0 1 5,0 Z
- details: M 0,20 H 130 ; M 14,0 V 70 ; M 116,7 L 122,10 L 116,13
- label: 18,3,46,26
- figure: none
- note: Placa apagada de la familia Tabla; la flecha de la cabecera recoge el lápiz de cola que el editor original dibujaba a la derecha de QTable. No tiene puertos y, por ser una declaración, no lleva cifra.


## Link
- family: Cadena
- variant: cápsulas enlazadas (cápsula a la derecha)
- footprint: 98x50
- ports: Entrada1@35%,20% ; Salida1@35%,80%
- path: M 15,10 H 60 V 7 A 19,7 0 0 1 98,7 V 43 A 19,7 0 0 1 60,43 V 40 H 15 A 15,15 0 0 1 0,25 A 15,15 0 0 1 15,10 Z
- details: M 60,10 V 40
- label: 62,11,35,22
- figure: 4,17,30,12
- note: Cápsulas enlazadas: la barra horizontal y la cápsula vertical que el original ya superponía (Rectangle 0,10,71,40 + RoundRect 70,0,98,50). El texto va en la cápsula vertical y la cifra en el extremo libre de la barra, porque la barra está partida por los dos puertos. Entrada1 (35,10) y Salida1 (35,40) caen sobre las rectas de la barra; la línea de eslabón marca el solape. Unlink es su calco al espejo.


## Unlink
- family: Cadena
- variant: cápsulas enlazadas (cápsula a la izquierda)
- footprint: 98x50
- ports: Entrada1@56%,20% ; Salida1@14%,98% ; SalidaAlt@98%,52% ; Salida2@56%,80%
- path: M 83,10 H 38 V 7 A 19,7 0 0 0 0,7 V 43 A 19,7 0 0 0 38,43 V 40 H 83 A 15,15 0 0 0 98,25 A 15,15 0 0 0 83,10 Z
- details: M 38,10 V 40
- label: 1,11,35,22
- figure: 60,17,30,12
- note: Calco al espejo de Link: la misma figura reflejada en horizontal (la comprobación es objetiva: la silueta de Link, reflejada, coincide con esta en el 99,8 % de los píxeles). La barra va a la derecha y la cápsula a la izquierda, y el texto se refleja con ella: el rótulo en la cápsula (1,11) y la cifra en el extremo libre de la barra (60,17). La cifra va 4 px antes del espejo exacto (64) porque su salida alternativa, en el borde derecho, se queda sin sitio si la cifra ocupa la esquina. Sus cuatro anclas —Entrada1 (55,10), Salida2 (55,40), Salida1 (14,49) y SalidaAlt (97,26)— caen cada una en el borde de la silueta.

