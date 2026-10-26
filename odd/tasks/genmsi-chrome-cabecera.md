# La cabecera y la barra — rediseño del chrome de la app

Rediseño de las dos filas que hacen de marco de la app: la **cabecera** (la
identidad del modelo y la consola de la corrida) y la **barra** (los
comandos). El diseño se hizo primero en Pen —como se hizo la sección de
gráficos— y la implementación siguió al diseño.

## Specs

### S1 — La identidad del trabajo encabeza la app

> "vamos a rediseñar toda esta parte"

La cabecera dice primero **qué se está editando** y después **qué está
haciendo la corrida**: el icono, la marca, el nombre del modelo, el punto
de "cambios sin guardar" y el título del modelo; a la derecha, la consola.
El nombre del modelo deja de ser el último elemento de la fila.

El modelo es el sujeto del trabajo, así que encabeza; la consola es el
instrumento, así que cierra la fila, donde la mano ya está.

### S2 — La barra son cuatro grupos con nombre

> "Reorganizar y limpiar (recomendado)"

La tira plana de quince controles pasa a ser una fila con cuatro grupos que
dicen lo que hacen: **archivo** (nuevo, abrir, recientes, guardar, guardar
como), **edición** (deshacer, rehacer, borrar), **vista** (encuadrar,
reacomodar) y **corrida** (semilla, hasta, inicio), al otro extremo.

### S3 — Densidad de herramienta profesional

> "Densificar: aspecto de herramienta profesional"

La cabecera mide 44 px y la barra 38: el marco le da el espacio al grafo.
Los botones que se entienden con su icono quedan sólo con el icono; los
campos de número pierden las flechas del navegador, que se comían el ancho
de la semilla.

### S4 — El transporte de la reproducción es su propio grupo

El botón que reproduce la traza y el botón que la graba son cosas distintas:
la consola separa ambos con una regla. "Simular" lleva el icono del motor
y "Ejecutar" el del.play, para que dos triángulos no significen lo mismo.

### S5 — La confirmación de un guardado no compilable sale de la barra

El aviso «El motor no compila este modelo. ¿Guardarlo de todos modos?» es un
mensaje, no un comando: pasa a la banda de avisos de la app, con la barra
libre para los comandos.

## Tasks

| ID | Spec | Ruta | Commit |
| --- | --- | --- | --- |
| T1 | S1 | `src/components/AppHeader.tsx`, `src/components/ModelIdentity.tsx` | — |
| T2 | S2, S3 | `src/components/Toolbar.tsx`, `src/components/ModelBar.tsx`, `src/components/chrome.ts` | — |
| T3 | S4 | `src/components/RunConsole.tsx` | — |
| T4 | S5 | `src/App.tsx` | — |

## Log

- L1 — El usuario manda una captura de la cabecera y la barra de la app
  instalada y pide: **"Ahora vamos a rediseñar toda esta parte
  /home/lucas/.tmp/pi-clipboard-5928b4b7-2c21-425b-bad9-74308be6b1ab.png te
  parece?"**
- L2 — El diagnóstico que se le da: la jerarquía está al revés (lo que pesa
  es la consola y el modelo queda último y en pequeño), la barra mezcla
  clases de comandos (archivo, edición y vista en la misma tira) y el corte
  que hace al no caber separa a "Encuadrar" de "Reacomodar", que son
  compañeros; y el espacio está mal repartido (el campo de la semilla es el
  más ancho y el reloj es texto suelto sin unidad).
- L3 — El usuario elige el alcance: **"Reorganizar y limpiar"** y la
  densidad: **"Densificar: aspecto de herramienta profesional"**.
- L4 — El diseño se maqueta en Pen (documento `Genmsi.pen`, marcos "App
  Header Before" y "App Header Screen") y de ahí sale la implementación.
- L5 — Implementado: cabecera nueva, barra por grupos, consola compacta,
  aviso de guardado fuera de la barra. Typecheck limpio, 161 tests en verde,
  build correcto, revisado con Playwright (con y sin modelo) y app
  recompilada e instalada.
- L6 — El usuario aprueba el resultado y fija una **regla inbloqueable**:
  todo diseño dibujado en Pen es una propuesta hasta que **él** lo aprueba,
  y esa aprobación no la sustituye nadie más. Queda registrada en
  `docs/DECISIONS.md` (D20) y en la memoria del proyecto. Este rediseño es
  la única excepción: se aprobó después de implementado ("por ahora déjalo
  porque me gustó") y queda cerrado.
- L7 — Dos ajustes pedidos por el usuario sobre la barra, ya implementados:
  - **El cerrojo de la semilla**: un checkbox al lado del campo. Desmarcado
    (el estado por defecto) cada corrida sortea su propia semilla, como
    hasta ahora; marcado, la corrida se repite con la misma semilla. Escribir
    una semilla marca la casilla, porque una semilla elegida es una semilla
    fijada.
  - **Fuera la palabra "CORRIDA"**: el rótulo del grupo no decía nada y
    ocupa el lugar de los rótulos que sí. Los tres números quedan sueltos
    al otro extremo, con el cerrojo al lado de la semilla.
- L8 — El usuario ve que al simular o ejecutar un modelo que el motor
  rechaza aparece una banda de aviso **arriba**, y la quiere fuera: "el
  error de compilación lo dice en tiempo real en la parte inferior". Lo que
  pide es que el cuadro del pie **titile** para que el usuario se dé
  cuenta.
  - La banda de arriba (el error rojo y el aviso azul de la corrida) **se
    fue**: el pie de la ventana es ahora el único lugar donde la app dice
    que algo salió mal.
  - Cuando una corrida falla, el cuadro del pie dice la falla y **su halo
    late tres veces** (0,9 s cada una), con el color de la caja. El aviso
    queda donde termina el grafo y el halo dice "mirá acá".
  - Los cuatro estados del pie (todo bien, falla de la corrida, el modelo no
    compila, compila con observaciones) están maquetados en Pen, en el
    marco "Status Line Screen".
- L9 — El usuario precisa: **todas** las notificaciones de arriba bajan, no
  sólo la del error ("al guardar la notificación también me sale arriba.
  yo quería que todas las notificaciones de arriba salgan abajo, al igual
  que como hiciste con el error").
  - Arriba **no queda ninguna banda de mensajes**. Abajo, en el pie, vive
    la única banda de la app: el estado de la compilación, la falla de la
    corrida, la confirmación del guardado y la pregunta del motor («el
    motor no compila este modelo, ¿guardarlo de todos modos?»).
  - El componente se llama `StatusLine` (antes `DiagnosticsPanel`): ya no
    dice sólo qué compila el modelo, dice todo lo que la app tiene que
    decir.
  - El halo late cuando la banda necesita el ojo: la falla de la corrida y
    la pregunta del motor. La confirmación del guardado no late: es una
    buena noticia de algo que el lector acaba de hacer.
  - Los seis estados del pie están maquetados en Pen, en el marco
    "Status Line Screen".
- L10 — El usuario pregunta si la pestaña **Estado** del panel lateral
  «literalmente tiene sentido» y dice que está pensando en eliminarla. La
  respuesta con la evidencia: sus tres listas (colas, facilities y
  almacenes) ya están en el Reporte (en vivo) y en los Gráficos (con más
  detalle), y el panel lateral es **del modelo**, no de la corrida. Lo que
  el usuario aprueba (maquetado en Pen, marcos "Aside Screen" y "Facility
  Indicators Screen"):
  - **La pestaña "Estado" se elimina**, y con ella el componente
    `StatePanel`. El panel lateral queda con dos pestañas: *Bloque* (la de
    por defecto) y *Bloques*.
  - **Lo único que se perdía no se pierde**: la columna **«Ahora»** de los
    indicadores por facility dice quién la tiene en el instante (*libre* /
    *tx 3*), en el color de las cifras vivas.

## Entrega — las unidades de revisión

La rama `feature/genmsi-reordenamiento` lleva el trabajo de estas dos
features en diez commits, y cada commit compila y pasa sus tests por sí
solo (comprobado commit por commit):

| Commit | Unidad |
| --- | --- |
| `ddb068e` | las métricas derivadas de la corrida |
| `d32c7fe` | las series de la corrida |
| `ef941ef` | las figuras del instante |
| `2d9b4cf` | las cartas de los gráficos |
| `d9805f7` | las series de los gráficos |
| `f903eae` | las tablas y las esperas de los gráficos |
| `d251e1e` | la sección de los gráficos |
| `dcdb9c2` | el marco de la app (cabecera, barra, pie y panel lateral) |
| `b8a51f5` | la documentación |
| `8e89058` | la versión 0.2.0 |

**La frontera de la entrega**: las unidades 1 a 7 son la feature de los
gráficos y la unidad 8 es la del chrome; la 9 es la documentación de las
dos y la 10 es la versión. Un pull request con el todo son casi diez mil
líneas: si se abre, va apilado —un PR por unidad, del monitor hacia la
app— para que quien revisa no tenga que leerlo todo de una vez.
