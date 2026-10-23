# Fase 5b — pulido de la ventana: la consola en el header y el aside con las clases

El pulido de la ventana única: las herramientas de la corrida suben al header, el
cuerpo queda sólo con la picture, y el panel lateral gana la lista de las clases
de bloque que se pueden crear.

## Specs

### S1 — La velocidad por defecto es x1

La reproducción arranca a **x1**: un segundo de modelo por segundo de pantalla.
Hoy arranca a la velocidad que mete la corrida en doce segundos, y esa cuenta
(`fittingSpeed`) desaparece: el que quiere ver la corrida más rápido la pide.

### S2 — El header es la consola de la corrida

El header lleva, de izquierda a derecha y sin salir de su fila:

- el nombre de la app;
- el selector de modelo, la semilla y el horizonte (los parámetros de la
  corrida);
- **Simular** (el modelo, y con él el reporte) y **Ejecutar** (el modelo y la
  traza, con lo que la reproducción tiene de dónde salir);
- **pausar o reproducir**, la **velocidad** y el **reloj**;
- el **deslizador** del reloj, angosto.

Las herramientas aparecen **siempre**, no sólo después de una corrida: sin
corrida están deshabilitadas y el reloj marca cero.

El body **pierde la barra de controles** entera: la consola está arriba.

El deslizador baja a un ancho de 160 px (`w-40`): el cuerpo es la picture, y el
deslizador es un control, no el protagonista.

El título del modelo deja el header (la cabecera de la corrida ya lo dice).

### S3 — El reloj y el deslizador comparten el estado

El reloj vive en el almacén de la capa viva, así que el header lo puede leer sin
que la ventana se redibuje. El deslizador del header **mueve la corrida**: pone
el reloj en el instante pedido y lleva la reproducción a ese registro, y la
picture se redibuja con él aunque la reproducción esté pausada.

### S4 — Fuera la barra de las cifras del modelo

La barra que dice cuántos bloques, funciones, variables, almacenes y tablas tiene
el modelo desaparece: el grafo ya muestra los bloques y el aviso de
diagnóstico ya dice si el modelo compila.

### S5 — Las clases de bloque son una pestaña del panel lateral

El panel lateral tiene tres pestañas: **Bloque** (las propiedades del bloque
seleccionado), **Estado** (las colas, las facilities y los almacenes de la
corrida) y **Bloques** (las clases que se pueden crear).

El botón «Añadir bloque» de la barra desaparece: las clases se crean desde la
pestaña.

La lista es **vistosa y detallada**: cada clase muestra su nombre, **qué hace**
(el título del catálogo) y sus puertos y propiedades, para que el lector sepa
qué está creando antes de crearlo. Crear una clase es un clic (el bloque cae
en el primer sitio libre) o un arrastre hasta el lienzo.

### S6 — El aviso de diagnóstico al pie de la ventana

El aviso que dice si el modelo compila baja al **pie de la ventana**, debajo de
la picture: es una línea discreta en el margen inferior, y no una franja que
roba altura al grafo.

## Tasks

| ID | Qué | Specs | Ruta | Commit |
|---|---|---|---|---|
| T1 | La consola del header: Simular, Ejecutar, pausar, velocidad, reloj y deslizador angosto; la velocidad por defecto x1 | S1, S2, S3 | writer A | `feat(app): the run's console is in the header` |
| T2 | El cuerpo sin barra de controles; el aside con la pestaña de las clases; fuera la barra de cifras; el diagnóstico al pie | S2, S4, S5, S6 | writer B | `feat(app): the body is the picture, and the aside lists the classes` |

## Log

### L1 — 2026-10-22

El usuario revisó la ventana y pidió:

> "- la velocidad por defecto de la simulacion debe ser x1"

> "- al tocar el boton de Ejecutar y reproducir aparecen los botones para pausar o
> barra de deslizamiento, me gustaria que eso ya aparezca por defecto, no que deba
> darle al boton de Ejecutar y reproducir. Ademas me gustaria que estas
> herramientas aparezcan en el header de arriba del todo reemplazando el boton de
> Simular y que en este header tenga las herramientas escenciales para la
> simulacion. Ademas esto conlleva a que se minimice la vizualizacion por lo menos
> del deslizados en ancho."

> "- /home/lucas/.tmp/gprCzRig/pi-clipboard-8d792f33-1983-489b-a7d6-4cf382e88296.png
> esto lo veo totalmente innecesario, te parece eliminarlo?"

> "- me gustaria que los bloques que se pueden añadir se coloquen en una pestaña
> nuieva en el aside, donde esta la pestaña bloque y Estado, quitando el boton de
> añadir bloque. esto permitira tener un listado mas vistozo ademas de detallado
> por si en un futuro queremos mostrar algo mas que el nombre del bloque a
> colocar."

> "- /home/lucas/.tmp/gprCzRig/pi-clipboard-9fdc5bc0-abfc-4a13-a343-92a8ed6aac31.png
> esto me gustaria que se cambie de lugar y que aparezca abajo del todo de la
> ventana"

### L2 — 2026-10-22 — Decisiones del usuario

- El header lleva **dos** botones: «Simular» (el modelo y el reporte) y
  «Ejecutar» (el modelo y la traza, con la reproducción).
- El deslizador vive en el **header**, angosto.
- Las capturas señaladas son la barra de las cifras del modelo (S4) y el aviso
  de diagnóstico (S6).

### L3 — 2026-10-22 — El contrato entre las dos unidades

- El almacén de la capa viva (`src/monitor/live.ts`) gana una acción
  `setClock(clock)`: poner el reloj sin tocar los números ni las bolitas. La
  escribe T1.
- La ventana (`src/components/MonitorView.tsx`) publica la picture cuando
  cambia el estado de la reproducción, para que el deslizador del header la
  mueva aunque la reproducción esté pausada. Lo escribe T2.
- Las dos unidades no se tocan: T1 escribe `App.tsx`, `RunConsole.tsx`,
  `live.ts`, `store.ts` y `replay.ts`; T2 escribe `MonitorView.tsx`,
  `ClassList.tsx` y borra `BlockPalette.tsx` y `ModelSummary.tsx`.
