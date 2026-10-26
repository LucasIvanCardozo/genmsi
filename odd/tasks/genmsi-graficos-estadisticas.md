# Gráficos — estadísticas útiles y rediseño de la sección

Rediseño de la sección **Gráficos** de la app: primero se decide qué
estadísticas valen la pena, después se diseñan en Pen y sólo entonces se
implementa. El diseño es la fuente de la verdad de la implementación.

## Specs

### S1 — El alcance es curado: seis grupos, cerca de diez visuales

El conjunto de estadísticas que se diseñan y se implementan es el curado, no
el exhaustivo:

1. **Resumen** — tarjetas de la corrida y ranking de los recursos críticos.
2. **Colas** — contenido en el tiempo, distribución de las esperas e
   indicadores por cola.
3. **Facilities** — línea de tiempo de la ocupación e indicadores por
   facility.
4. **Almacenes y cadenas** — unidades en uso contra la capacidad, y tamaño de
   la cadena.
5. **Tablas** — histograma con la media, la desviación y los desbordes.
6. **Bloques** — transacciones por bloque y reintentos por bloque.

### S2 — Las métricas derivadas se derivan en la app y se rotulan

> "Derivarlas en la app, rotuladas como derivadas (recomendado)"

Son razones de números que el motor ya publica: `área/tiempo` (contenido
medio), `tiempo total/entradas` (espera media), `tiempo ocupado/tiempo`
(utilización), `área/(tiempo·capacidad)` (utilización de un almacén). La app
no recalcula nada que el motor no diga, y cada cifra derivada lo dice en su
rótulo.

### S3 — La sección es un solo scroll

> "todo en un scroll, pero acordate que primero haremos el diseño en la
> aplicacion pen (que esta ahora abierta). hasta que a mi no me guste el seño
> no se empezara ninguna implementacion"

Todo el contenido de la sección vive en una columna que se desplaza. No hay
pestañas por familia de entidad.

### S4 — Los gráficos están sincronizados con el reloj de la reproducción

> "Sí, sincronizados con el reloj (recomendado)"

Una línea vertical marca el instante del playhead en cada serie temporal, y
los indicadores muestran el valor de ese instante, como ya hace el Reporte.

### S5 — Los tres gráficos que ya existen también se juzgan

> "Los graficos o estadisticas que se nencuentran en la aplicacion actualmente
> se pondran tambien en tela de juicio."

- **Contenido de las colas en el tiempo**: se queda. Hoy todas las series
  comparten el mismo color, no hay cursor del reloj, no hay línea de media y no
  hay etiqueta de unidades.
- **Facilities 0/1 en el tiempo**: se cae. Una línea binaria por facility es
  ilegible y no dice lo que dice la utilización.
- **Histograma de las tablas**: se queda. El motor publica `suma` y `suma de
  cuadrados`, así que la media y la desviación se pueden marcar y hoy no se
  ven; los desbordes tampoco tienen clase propia en el gráfico.

### S7 — El eje del tiempo siempre empieza en 0

> "los graficos que tienenel tiempo como coordenada x siempre deben empezar de
> 0, no de cuando aparece el primer dato como pasa actualmente, solo eso."

Todo gráfico cuya coordenada *x* sea el tiempo de la corrida tiene su dominio
en **[0, fin de la corrida]**, aunque el primer dato llegue más tarde.

Hoy la serie del gráfico de colas empieza en el primer cambio de la entidad
(`mergeSeries` arma las filas con los instantes en que algo se mueve), así que
el eje arranca donde llega el primer dato. Con S7, la serie empieza en 0 con el
valor inicial de la entidad —el contenido o el estado con que la corrida
empieza— y el dominio del eje es fijo, no el de los datos.

El diseño ya lo cumple: en Pen, todas las series temporales arrancan en 0.

### S6 — Ninguna implementación antes de la aprobación del diseño

El diseño se hace en Pen (la aplicación abierta). La implementación de la app
no empieza hasta que el diseño esté aprobado.

## Tasks

| ID | Specs | Tarea | Ruta | Commit |
|---|---|---|---|---|
| T1 | S1-S6 | Documento de la feature y propuesta de estadísticas | inline | sin commit (S6) |
| T2 | S2-S4 | Tokens y componentes reutilizables en Pen | inline | sin commit (S6) |
| T3 | S1 | Pen: Resumen (tarjetas + ranking de recursos) | inline | sin commit (S6) |
| T4 | S1 | Pen: Colas (serie, distribución, indicadores) | inline | sin commit (S6) |
| T5 | S1 | Pen: Facilities (ocupación + indicadores) | inline | sin commit (S6) |
| T6 | S1 | Pen: Almacenes y cadenas | inline | sin commit (S6) |
| T7 | S1, S5 | Pen: Tablas (histograma con media, σ y desbordes) | inline | sin commit (S6) |
| T8 | S1 | Pen: Bloques (transacciones y reintentos) | inline | sin commit (S6) |
| T9 | S3 | Pen: la sección completa, un solo scroll | inline | sin commit (S6) |
| T10 | S1-S6 | Implementación en la app (tras la aprobación del diseño) | a definir | pendiente |

### Las unidades de trabajo de la implementación

La implementación va por grupos, cada uno una unidad revisable. Cada grupo
arranca sus tests antes del código (RED) y termina con la suite entera, el
typecheck y una mirada a la app.

| Unidad | Specs | Qué entra | Estado |
|---|---|---|---|
| W1 | S2, S7 | El cimiento: `src/monitor/derived.ts` (las métricas derivadas), `src/monitor/series.ts` (las series, los tramos y los histogramas), las esperas por cola, los puntos de almacenes y cadenas, y la suma de las tablas en las figuras vivas. | hecha |
| W2 | S1, S2 | El grupo **Resumen**: la corrida en números y los recursos que aprietan. | hecha |
| W3 | S1, S7 | El grupo **Colas**: la serie con un color por cola, la distribución de las esperas y los indicadores. | hecha |
| W4 | S1 | El grupo **Facilities**: la línea de tiempo de la ocupación y los indicadores (sustituye el gráfico 0/1). | hecha |
| W5 | S1 | El grupo **Almacenes y cadenas**. | hecha |
| W6 | S1, S5 | El grupo **Tablas**: el histograma con la media, la desviación y los desbordes, y la lista de las tablas. | hecha |
| W7 | S1 | El grupo **Bloques**: las transacciones y los reintentos. | hecha |
| W8 | S1 | La documentación (`docs/app.md`) y la mirada final. | hecha |

### Lo que la implementación encontró

- **Las cifras publicadas divididas por el reloj del instante.** La primera
  versión de *Los recursos que aprietan* dividía el área publicada (la de
  toda la corrida) por el reloj de la reproducción, y una cola de cuatro
  unidades aparecía con un contenido medio de 330. Ahora cada juego de
  cifras se divide por **su** tiempo: el de la reproducción para las cifras
  del instante, el de la corrida para las publicadas. El tiempo viaja con
  las cifras (`Elapsed` en `src/components/charts/entities.ts`).
- **La serie ahora empieza en 0** (S7): `seriesFrom` mete la fila del
  instante cero con el valor inicial de la entidad, y los ejes de los
  gráficos fijan su dominio en `[0, fin de la corrida]`.
- **La traza no lleva todos los eventos del motor** (400 registros para 447
  eventos en el modelo de prueba): el conteo de transacciones en vivo cuenta
  las que la traza lleva, que es lo que la reproducción puede saber.
- **Un atributo SVG y un nombre de dato pueden chocar.** La fila del
  histograma llevaba un campo `overflow` (el booleano que dice si el
  intervalo es el de los desbordes) y Recharts da los campos de la fila a la
  forma que dibuja: el SVG recibía `overflow={false}` y React avisaba. El
  campo se llama ahora `overflowClass`.
- **Dos bloques del modelo pueden llamarse igual.** La lista de barras usaba
  el nombre del bloque como clave de React, y dos bloques con el mismo
  nombre la rompían. La clave es el nombre más su lugar.
- **Un histograma de una sola barra es un muro.** Cuando todas las salidas
  esperaron cero, el gráfico era una barra de 600 px: ahora la tarjeta dice
  que nadie esperó y no dibuja el gráfico.

### Lo que se diseñó

El diseño está en el lienzo de Pen: `Genmsi.pen`
(`~/.pencil/documents/73873ccb-a24e-4da5-bb93-440c6149b422/Genmsi.pen`), en el
marco **Graficos Screen**, con seis filas de dos tarjetas.

| Fila | Tarjetas |
|---|---|
| Resumen | *La corrida en números* (6 tarjetas) · *Los recursos que aprietan* (3 paneles de barras) |
| Colas | *Contenido de las colas a lo largo del tiempo* · *Distribución de las esperas* · *Indicadores por cola* |
| Facilities | *Ocupación de las facilities a lo largo del tiempo* · *Indicadores por facility* |
| Almacenes | *Unidades en uso — Almacén Reactivos* · *Tamaño de la cadena* |
| Tablas | *Histograma — Tiempos de servicio* · *Las tablas del modelo* |
| Bloques | *Transacciones por bloque* · *Reintentos por bloque* |

Los números del diseño salen de un modelo de ejemplo coherente (hospital con
dos facilities, dos colas, un almacén, una cadena y cuatro tablas): el mismo
número aparece en el gráfico y en la tabla de al lado.

El archivo vive fuera del repositorio (el lienzo de Pen), así que este
documento es la única traza en el repo hasta que la implementación empiece.

## Log

- L1 — "Hola! hoy quiero que nos enfoquemos en mejorar notablemente la seccion de graficos! Quiero que analices todos los componentes que tenemos y en base a todos proponerque estadisticas son realmente utiles de todos. Una vez que tengamos que estadisticas podemos sacar de cada uno vamos a diseñar como se veria cada una mediante pen (ya tengo abiuerta la aplicacion), te parece? Los graficos o estadisticas que se nencuentran en la aplicacion actualmente se pondran tambien en tela de juicio."
- L2 — Decisiones del usuario: alcance **curado** (seis grupos, ~10 visuales); métricas derivadas **derivadas en la app y rotuladas**; estructura **un solo scroll**; **sincronía con el reloj** sí.
- L3 — El usuario escribe "seño" donde el contexto pide "diseño": se conserva la cita tal cual.
- L4 — Juicio sobre los tres gráficos que ya había, con la decisión que cada uno lleva:
  - *Contenido de las colas en el tiempo*: **se queda** (serie exacta, media, cursor del reloj, un color por cola).
  - *Facilities 0/1 en el tiempo*: **se cae** — lo reemplaza la línea de tiempo de ocupación (tramos ocupado/libre) y el ranking de utilización.
  - *Histograma de las tablas*: **se queda** (frecuencia + acumulada + banda de la desviación + clase de desbordes).
- L5 — Las cifras del diseño son coherentes entre sí: los tramos de ocupación suman la utilización que muestra la tabla, la media de la cola sale de su propia serie y el histograma de esperas tiene la misma media que la columna "espera media".
- L7 — La implementación termina: los seis grupos, las doce tarjetas, los
  tests nuevos (`tests/derived.test.ts`, `tests/series.test.ts` y las
  pruebas nuevas del replay y de las figuras), `docs/app.md` al día y la
  sección mirada en la app con dos modelos (`entregaComida` y `TP8__mecanicos`).
- L8 — El usuario ve la app instalada y pide que la **ocupación de las
  facilities** avance con el tiempo: mostraba la corrida entera de una. Era
  un bug: `segmentsOf` usaba el instante final sólo para el último tramo, y
  los tramos posteriores al reloj se dibujaban. Ahora los tramos se cortan en
  el reloj (`segmentsOf` filtra los cambios posteriores) y hay un test que lo
  fija. La app se recompiló y reinstaló.
- L9 — Dos cosas más que el usuario vio en la app instalada:
  - **La línea del reloj estaba desplazada** de los tramos que marcaba: su
    posición estaba calculada a mano (`calc(6rem + (100% - 9rem) * …)`) y no
    coincidía con la pista. Ahora la línea se dibuja con la misma estructura
    que las filas (los mismos anchos y el mismo hueco), así que cae donde
    tiene que caer sin aritmética.
  - **Al principio de la corrida los gráficos ya mostraban el final**: las
    cifras del instante caían al reporte del motor cuando el replay todavía
    no conocía la entidad. Ahora las cifras son **sólo las del instante** (lo
    que el replay sabe hasta el reloj): una entidad que aún no ocurrió vale
    cero. El reporte sólo aporta los nombres de las entidades y la
    capacidad de los almacenes. Hay un test nuevo
    (`tests/entities.test.ts`) que lo fija.

  Con esto la regla de la sección queda: **el reloj manda**. El reporte
  describe la corrida completa; los gráficos describen el instante.
- L6 — El usuario aprueba el diseño entero con una sola corrección: "los graficos que tienenel tiempo como coordinada x siempre deben empezar de 0, no de cuando aparece el primer dato como pasa actualmente, solo eso. Todo lo otro me encanta!" Queda como S7; el diseño de Pen ya lo cumplía, así que la regla es para la implementación.
