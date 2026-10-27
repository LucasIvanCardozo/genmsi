# Grupo A — familia lineal: 24 clases (contrato v2)

Siluetas, anclas de puerto y zonas de texto/cifra de las clases del flujo en línea: lo que nace, lo que espera, lo que opera y lo que muere.

La hoja que se mira está en `grupos/A-lineal.svg`: las 24 clases dibujadas en su huella real, con los puertos en su ancla, las zonas de rótulo y de cifra punteadas y una cifra de ejemplo por bloque.

Los footprints y las anclas de puerto salen de `odd/tasks/forma-bloques-diseno/puertos.md` (el catálogo del editor original); el color sale de la familia del contrato §5 y §5.2.

**Cómo se lee este archivo**

- `family`: la familia de la clase. `variant`: el nombre de la silueta dentro de la familia (§5.1): la familia es un lenguaje de variantes, no una forma única.
- `path`: el `d` del SVG de la silueta, en coordenadas de la huella (0,0 arriba a la izquierda).
- `details`: los trazos interiores, cada uno un `d` propio, separados por ` ; `. Se dibujan sin relleno, con el color de la familia y 1,8 px de grosor. `none` = sin trazos. Ningún trazo interior pisa una zona de texto ni una de cifra.
- `label` y `figure`: la zona de rótulo y la zona de cifra, `x,y,ancho,alto`. La zona de cifra se reserva siempre, aunque quede vacía, para que el rótulo no se mueva cuando aparece la cifra. `figure: none` = la clase no tiene zona de cifra.
- `ports`: `nombre@x%,y%` con el ancla normalizada a la huella. `none` = sin puertos.
- Las dos zonas viven **dentro** de la silueta, nunca la invaden.

**Las tres reglas del contrato v2 que este grupo cumple**

- **§2.1 — ancho útil.** La partición izquierda/derecha se usa cuando el ancho útil (la huella menos 6 px de aire por lado) llega a 60 px. `Clock` (38 px útiles), `FunctionDist` (52 px) y `MFunctionDist` (60 px, pero con la caja inscrita del círculo por debajo) van apilados: rótulo arriba, cifra abajo, en la misma columna. `Terminate` también va apilado, aunque su ancho útil llegue a 72 px, porque rótulo y cifra no caben en el mismo lado (60 + 30 > 72): es la regla general de §2.1, «el texto y la cifra se colocan donde caben».
- **§2.2 — el subtítulo.** Sólo existe en bloques de 40 px de alto o más. Los nueve bloques de menos de 40 px (`Advance`, `Buffer`, `Queue`, `Depart`, `Logic`, `InitialLS`, `Mark`, `Variable`, `VariableA`) llevan **una sola línea** con el nombre de la clase.
- **§6.3 — la cifra.** Una línea de 10-11 px monoespaciada. Las 23 zonas de cifra de este grupo miden **30 px de ancho o más** (el mínimo de §6.3 para una línea), así que **ninguna clase usa la forma apilada** y ninguna cifra se trunca. `Clock` reserva la zona (30 px) pero la deja vacía: es una declaración sin entidad.
- **§3.1 — la silueta toca el puerto.** Las 26 anclas de este grupo caen **en el borde** de su silueta, así que no hace falta la excepción de la burbuja centrada en el ancla.

**Colores de familia**

- **Fuente** (`#047857`): flechas generadas (conteo de entradas del generador).
- **Destino** (`#be123c`): transacciones terminadas (la fila del reporte).
- **Cola** (`#b45309`): contenido de la cola (las figuras vivas de la cola).
- **Operación** (`#475569`): el valor que guarda, el del interruptor o el par de siempre.
- **Anotación** (`#64748b`): sin cifra: no participa del flujo.

**Variantes por familia**

- **Fuente**: `Generate` caja de emisión · `Clock` esfera del reloj.
- **Destino**: `Terminate` circulo limpio.
- **Cola**: `Queue` caja con remate redondeado · `Depart` cajón con galón.
- **Operación**: `Advance` placa base · `Buffer` placa con ranura · `Assign` hoja con pestaña · `Mark` tarjeta con lengüeta · `Counter` paralelogramo · `Priority` cuerpo con cuello · `SaveValue` cajón con lengüeta · `InitiaSV` cajón con lengüeta · `Logic` cuerpo con interruptor · `InitialLS` cuerpo con interruptor · `Variable`/`VariableN`/`VariableA` cinta de datos · `Function`/`FunctionExpon`/`MFunction` tarjeta de gráfica · `FunctionDist` elipse con abanico · `MFunctionDist` elipse con aros.
- **Anotación** (§5.2): `Note` hoja con esquina doblada. Es la doceava familia y sólo contiene `Note`: sale de Operación porque una anotación no es un paso del flujo.

**Lo que no se puede evitar, y por qué**

- `FunctionDist` y `MFunctionDist`: su nombre de clase no cabe en el ancho útil de un círculo de 64/72 px ni en una línea ni en dos, así que llega truncado («Functio…», «MFuncti…»). Es el precio de la variante circular; el nombre completo está en el rótulo de la ficha del catálogo y en la lista de bloques.
- `Clock`: la tabla «todo lo demás» de la feature le asignaría el par dentro/pasadas, pero el contrato §2 deja la cifra vacía en una declaración sin entidad y un reloj no recibe transacciones: se entrega la zona reservada y vacía.
- `Generate`: la caja de emisión sube de y=35 a y=20 (el catálogo original la dibuja a y=35) para que el rótulo y la cifra compartan banda horizontal; el ancla del puerto no cambia y la silueta sigue siendo la del original unida a un cuerpo más alto.

---

## Generate
- family: Fuente
- variant: caja de emisión
- footprint: 120x63
- ports: Salida1@27%,100%
- path: M 31.5 0 A 31.5 31.5 0 0 1 57.83 20 L 120 20 L 120 63 L 31.5 63 A 31.5 31.5 0 0 1 31.5 0 Z
- details: M 62 20 L 62 63
- label: 10,26,48,24
- figure: 64,32,32,14
- note: cabeza redonda unida a la caja de emisión, la silueta del catálogo original; la caja sube a y=20 para que rótulo y cifra compartan banda, y el único puerto sale por abajo en el 27% del ancho, sobre el borde plano de la caja

## Clock
- family: Fuente
- variant: esfera del reloj
- footprint: 50x50
- ports: none
- path: M 0 25 A 25 25 0 1 0 50 25 A 25 25 0 1 0 0 25 Z
- details: M 2 25 L 6 25 ; M 44 25 L 48 25
- label: 7,12,36,18
- figure: 10,32,30,12
- note: esfera de la familia Fuente en placa apagada, con dos marcas de esfera a los lados; sin puertos no hay ancla, y al medir 50 px de ancho útil (38 px) el rótulo va arriba y la zona de cifra reservada abajo y vacía, porque es una declaración sin entidad

## Terminate
- family: Destino
- variant: circulo limpio
- footprint: 64x64
- ports: Entrada1@50%,0%
- path: M32 0 A32 32 0 1 1 32 64 A32 32 0 1 1 32 0 Z
- details: none
- label: 6,16,52,11
- figure: 14,34,36,11
- note: Circulo limpio, la forma de sumidero que se lee de un vistazo: el flujo entra por arriba y se cierra. El editor original lo dibujaba como una elipse de 64 con un rectangulo pegado al lado; esa pieza es adorno y no se lleva, porque hacia la silueta irregular y el anillo del fondo no significaba nada para quien lee. El puerto de entrada conserva su ancla exacta del original (32,0), que en un circulo es justo el vertice superior; por eso la huella de la clase pasa de 84x64 a 64x64. El rotulo y la cifra van en bandas apiladas porque el ancho util del circulo no da para los dos en el mismo lado.

## Queue
- family: Cola
- variant: caja con remate redondeado
- footprint: 97x32
- ports: Entrada1@36%,0% ; Salida1@36%,100%
- path: M 0 0 H 70 A 27 16 0 0 1 70 32 L 0 32 Z
- details: M 70 3 L 70 29 ; M 76 11 L 76 21 ; M 81 11 L 81 21 ; M 86 11 L 86 21
- label: 5,7,30,12
- figure: 37,7,30,12
- note: caja con remate redondeado a la derecha, la silueta de la familia Cola; las tres marcas del remate son los que esperan, así que rótulo y cifra se quedan en el cuerpo, y la entrada y la salida comparten el 36% del ancho

## Depart
- family: Cola
- variant: cajón con galón
- footprint: 97x32
- ports: Entrada1@36%,0% ; Salida1@36%,100%
- path: M 0 0 H 70 A 27 16 0 0 1 70 32 L 0 32 Z
- details: M 70 3 L 70 29 ; M 76 9 L 85 16 ; M 76 23 L 85 16
- label: 5,7,30,12
- figure: 37,7,30,12
- note: la misma caja de Queue con galón de salida en el remate, la figura del catálogo original; el galón vive en el remate, así que la cifra se queda en el cuerpo, y entrada y salida comparten el 36% del ancho

## Advance
- family: Operación
- variant: placa base
- footprint: 80x32
- ports: Entrada1@50%,0% ; Salida1@50%,100%
- path: M 7 0 H 73 A 7 7 0 0 1 80 7 V 25 A 7 7 0 0 1 73 32 H 7 A 7 7 0 0 1 0 25 V 7 A 7 7 0 0 1 7 0 Z
- details: none
- label: 3,7,44,12
- figure: 47,7,30,12
- note: la placa base de la familia Operación, sin trazos interiores: es la operación neutra, la que sólo recibe y pasa; con 32 px de alto no cabe subtítulo, y el puerto entra y sale al 50% del ancho, sobre el borde plano de la placa

## Assign
- family: Operación
- variant: hoja con pestaña
- footprint: 100x45
- ports: Entrada1@50%,0% ; Salida1@50%,100%
- path: M 16 0 H 84 V 17 H 92 A 8 8 0 0 1 100 25 V 37 A 8 8 0 0 1 92 45 H 8 A 8 8 0 0 1 0 37 V 25 A 8 8 0 0 1 8 17 H 16 Z
- details: M 38 8 L 62 8
- label: 10,18,50,22
- figure: 64,22,30,14
- note: hoja con pestaña superior, la silueta del catálogo original; la pestaña lleva la línea del título y el rótulo vive en el cuerpo, con la entrada y la salida al 50% del ancho, sobre la pestaña

## Buffer
- family: Operación
- variant: placa con ranura
- footprint: 88x32
- ports: Entrada1@50%,0% ; Salida1@50%,100%
- path: M 7 0 H 81 A 7 7 0 0 1 88 7 V 25 A 7 7 0 0 1 81 32 H 7 A 7 7 0 0 1 0 25 V 7 A 7 7 0 0 1 7 0 Z
- details: M 4 26 L 84 26
- label: 3,7,50,12
- figure: 55,7,30,12
- note: placa con la ranura del búfer atravesada en el tercio inferior, la línea del catálogo original; con 32 px no cabe subtítulo y el texto se queda arriba de la ranura, y los puertos cruzan el 50% del ancho

## Mark
- family: Operación
- variant: tarjeta con lengüeta
- footprint: 96x36
- ports: Entrada1@39%,0% ; Salida1@39%,100%
- path: M 8 0 H 68 A 8 8 0 0 1 76 8 V 9 H 88 A 8 8 0 0 1 96 17 V 21 A 8 8 0 0 1 88 29 H 76 V 28 A 8 8 0 0 1 68 36 H 8 A 8 8 0 0 1 0 28 V 8 A 8 8 0 0 1 8 0 Z
- details: M 82 19 A 4 4 0 1 0 90 19 A 4 4 0 1 0 82 19 Z
- label: 5,7,32,12
- figure: 39,7,30,12
- note: tarjeta con lengüeta a la derecha, la silueta del catálogo original; el agujero del remate la firma como marca y la cifra se queda en la tarjeta para no pisarlo, y la entrada y la salida bajan al 39% del ancho

## Counter
- family: Operación
- variant: paralelogramo
- footprint: 100x40
- ports: Entrada1@50%,0% ; Salida1@50%,100%
- path: M 6 0 H 74 L 94 34 A 6 6 0 0 1 88 40 H 26 A 6 6 0 0 1 20 34 L 0 6 A 6 6 0 0 1 6 0 Z
- details: M 30 31 L 30 37 ; M 35 31 L 35 37 ; M 40 31 L 40 37
- label: 20,8,34,24
- figure: 55,22,30,12
- note: placa inclinada, el paralelogramo del catálogo original: el flujo entra por arriba y sale por abajo, y las tres rayas del pie son el contador; la pendiente manda el rótulo a la derecha y la cifra a la banda baja, y los puertos van al 50% del ancho

## Priority
- family: Operación
- variant: cuerpo con cuello
- footprint: 88x52
- ports: Entrada1@50%,0% ; Salida1@50%,100%
- path: M 32 0 H 56 A 6 6 0 0 1 62 6 V 20 H 82 A 6 6 0 0 1 88 26 V 46 A 6 6 0 0 1 82 52 H 6 A 6 6 0 0 1 0 46 V 26 A 6 6 0 0 1 6 20 H 26 V 6 A 6 6 0 0 1 32 0 Z
- details: M 34 12 L 34 18 ; M 42 8 L 42 18 ; M 50 4 L 50 18
- label: 8,24,40,22
- figure: 54,28,30,14
- note: cuerpo con cuello elevado, la silueta del catálogo original; las tres barras del cuello son los niveles de prioridad y el rótulo vive en el cuerpo, con los puertos al 50% del ancho, sobre el cuello

## SaveValue
- family: Operación
- variant: cajón con lengüeta
- footprint: 100x45
- ports: Entrada1@50%,0% ; Salida1@50%,100%
- path: M 8 0 H 92 A 8 8 0 0 1 100 8 V 20 A 8 8 0 0 1 92 28 H 84 V 45 H 16 V 28 H 8 A 8 8 0 0 1 0 20 V 8 A 8 8 0 0 1 8 0 Z
- details: M 24 36 L 76 36
- label: 8,3,52,22
- figure: 62,10,30,14
- note: cajón con lengüeta abajo, el espejo de Assign: asignar empuja hacia fuera y guardar deja el valor en el cajón; la cifra es el valor guardado y la lengüeta lleva la ranura, con la entrada y la salida al 50% del ancho

## InitiaSV
- family: Operación
- variant: cajón con lengüeta
- footprint: 100x45
- ports: none
- path: M 8 0 H 92 A 8 8 0 0 1 100 8 V 20 A 8 8 0 0 1 92 28 H 84 V 45 H 16 V 28 H 8 A 8 8 0 0 1 0 20 V 8 A 8 8 0 0 1 8 0 Z
- details: M 24 36 L 76 36
- label: 8,3,52,22
- figure: 62,10,30,14
- note: el mismo cajón de SaveValue en placa apagada, porque esta clase es la que inicializa ese valor; no tiene puertos y por eso no hay ancla, y la cifra es el valor inicial del modelo

## Logic
- family: Operación
- variant: cuerpo con interruptor
- footprint: 96x32
- ports: Entrada1@39%,0% ; Salida1@39%,100%
- path: M 7 0 H 63 A 7 7 0 0 1 70 7 V 8 H 88 A 8 8 0 0 1 96 16 V 24 H 70 V 25 A 7 7 0 0 1 63 32 H 7 A 7 7 0 0 1 0 25 V 7 A 7 7 0 0 1 7 0 Z
- details: M 76 20 L 88 12
- label: 5,7,32,12
- figure: 37,7,30,12
- note: cuerpo con la caja del interruptor a la derecha, la silueta del catálogo original; la palanca dibujada en la caja dice la posición, así que la cifra se queda en el cuerpo, y los puertos pasan al 39% del ancho

## InitialLS
- family: Operación
- variant: cuerpo con interruptor
- footprint: 96x32
- ports: none
- path: M 7 0 H 63 A 7 7 0 0 1 70 7 V 8 H 88 A 8 8 0 0 1 96 16 V 24 H 70 V 25 A 7 7 0 0 1 63 32 H 7 A 7 7 0 0 1 0 25 V 7 A 7 7 0 0 1 7 0 Z
- details: M 76 20 L 88 12
- label: 5,7,32,12
- figure: 37,7,30,12
- note: el mismo interruptor de Logic en placa apagada, porque fija el valor de partida; sin puertos no hay ancla que repartir, y la cifra es el valor inicial del interruptor

## Variable
- family: Operación
- variant: cinta de datos
- footprint: 120x32
- ports: none
- path: M 0 0 H 104 L 120 16 L 104 32 H 0 Z
- details: M 108 10 L 108 22
- label: 6,10,60,12
- figure: 70,10,30,12
- note: cinta de datos con la punta cortada, la silueta del catálogo original; es una declaración sin puertos en placa apagada, y con 32 px de alto el rótulo va en una sola línea

## VariableN
- family: Operación
- variant: cinta de datos
- footprint: 120x40
- ports: none
- path: M 0 0 H 104 L 120 20 L 104 40 H 0 Z
- details: M 108 14 L 108 26
- label: 8,6,52,24
- figure: 66,10,30,14
- note: la misma cinta de Variable, más alta: la silueta comparte familia y la diferencia de altura dice el tipo; sin puertos no hay ancla, y con 40 px ya caben rótulo y subtítulo

## VariableA
- family: Operación
- variant: cinta de datos
- footprint: 120x32
- ports: none
- path: M 0 0 H 104 L 120 16 L 104 32 H 0 Z
- details: M 108 10 L 108 22
- label: 6,10,60,12
- figure: 70,10,30,12
- note: la misma cinta de Variable: el catálogo original les da la misma silueta y sólo cambia el tipo de dato, así que la diferencia se deja en el rótulo; sin puertos no hay ancla

## Function
- family: Operación
- variant: tarjeta de gráfica
- footprint: 100x100
- ports: none
- path: M 8 0 H 92 A 8 8 0 0 1 100 8 V 92 A 8 8 0 0 1 92 100 H 8 A 8 8 0 0 1 0 92 V 8 A 8 8 0 0 1 8 0 Z
- details: M 16 92 L 88 92 ; M 16 92 L 16 54 ; M 20 86 L 44 60 L 62 74 L 86 56
- label: 8,6,84,22
- figure: 34,32,32,14
- note: tarjeta con ejes y curva, la silueta del catálogo original; la gráfica es la firma de la función y baja a media tarjeta para dejar sitio al rótulo y a la cifra, y sin puertos no hay ancla

## FunctionExpon
- family: Operación
- variant: tarjeta de gráfica
- footprint: 100x100
- ports: none
- path: M 8 0 H 92 A 8 8 0 0 1 100 8 V 92 A 8 8 0 0 1 92 100 H 8 A 8 8 0 0 1 0 92 V 8 A 8 8 0 0 1 8 0 Z
- details: M 16 92 L 88 92 ; M 16 92 L 16 54 ; M 20 58 C 40 60 58 82 88 90
- label: 8,6,84,22
- figure: 34,32,32,14
- note: la misma tarjeta de Function con la curva exponencial en lugar de la quebrada; el rótulo coge los 84 px de la banda alta para que el nombre no se corte, y la gráfica baja a media tarjeta

## MFunction
- family: Operación
- variant: tarjeta de gráfica
- footprint: 100x100
- ports: none
- path: M 8 0 H 92 A 8 8 0 0 1 100 8 V 92 A 8 8 0 0 1 92 100 H 8 A 8 8 0 0 1 0 92 V 8 A 8 8 0 0 1 8 0 Z
- details: M 16 92 L 88 92 ; M 16 92 L 16 54 ; M 20 86 L 44 60 L 62 74 L 86 56 ; M 20 72 L 44 44 L 62 58 L 86 44
- label: 8,6,84,22
- figure: 34,32,32,14
- note: la tarjeta de Function con dos curvas, que es lo que la distingue de la simple en el catálogo original; declaración en placa apagada, y sin puertos no hay anclas

## FunctionDist
- family: Operación
- variant: elipse con abanico
- footprint: 64x64
- ports: none
- path: M 0 32 A 32 32 0 1 0 64 32 A 32 32 0 1 0 0 32 Z
- details: M 28 32 L 32 35 L 36 32 ; M 30 30.5 L 32 35 L 34 30.5 ; M 31 29.5 L 32 35 L 33 29.5
- label: 12,8,40,22
- figure: 16,36,32,22
- note: elipse con el abanico de la distribución, la silueta del catálogo original; con 64 px de ancho el rótulo va arriba y la cifra abajo, en la misma columna, y el nombre llega truncado porque no cabe de otro modo en un círculo de 64 px

## MFunctionDist
- family: Operación
- variant: elipse con aros
- footprint: 72x72
- ports: none
- path: M 0 36 A 36 36 0 1 0 72 36 A 36 36 0 1 0 0 36 Z
- details: M 30.5 37 A 5.5 5.5 0 1 0 41.5 37 A 5.5 5.5 0 1 0 30.5 37 Z ; M 33.5 37 A 2.5 2.5 0 1 0 38.5 37 A 2.5 2.5 0 1 0 33.5 37 Z
- label: 15,9,42,22
- figure: 18,44,36,22
- note: elipse con aros concéntricos en la banda media, la silueta del catálogo original; en un círculo de 72 px el rótulo va arriba y la cifra abajo, y el nombre llega truncado por lo mismo que en FunctionDist

## Note
- family: Anotación
- variant: hoja con esquina doblada
- footprint: 80x80
- ports: none
- path: M 0 0 H 58 L 80 22 V 80 H 0 Z
- details: M 58 0 V 22 H 80
- label: 10,10,52,26
- figure: none
- note: hoja con la esquina doblada, la silueta del catálogo original; cambia de familia: pasa de Operación a la familia propia «Anotación» (gris #64748b) porque una anotación no es un paso del flujo, y no tiene puertos ni cifra
