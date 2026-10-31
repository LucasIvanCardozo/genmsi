# Auditoría de la sección Gráficos

Qué dice cada tarjeta del panel **Gráficos** del monitor, si lo dice bien y si
habría una forma mejor de decirlo. Cada tarjeta la juzgó un subagente con
foco exclusivo en ella; los hallazgos se verificaron contra el código antes de
escribirlos acá.

**Alcance**: el panel *Gráficos* del monitor (sus trece tarjetas) y las piezas
que comparten. Quedan fuera el panel *Reporte* y el lienzo de *Reproducción*.

**Las reglas que la sección ya tenía** (de `odd/tasks/genmsi-graficos-estadisticas.md`
y `docs/app.md`), y contra las que se juzga cada tarjeta:

1. **El reloj manda**: el reporte describe la corrida completa, los gráficos
   describen el instante. Cada juego de cifras se divide por *su* tiempo.
2. **Sólo ratios del motor**: lo derivado va marcado con asterisco y la
   tarjeta dice la fórmula.
3. **El eje del tiempo arranca en 0** y termina en el final de la corrida.
4. **La serie es la historia exacta** de la entidad, no una muestra.
5. Cada tarjeta dice de dónde salen sus números y cómo se hicieron.

## Cómo leer los hallazgos

- **Severidad**: *alta* = el lector se lleva una cifra o una conclusión
  equivocada; *media* = la tarjeta engaña o deja información afuera; *baja* =
  ruido, cosmético o robustez.
- **Veredicto**: `KEEP` · `KEEP WITH CHANGES` · `REDUNDANT` · `REMOVE`.
- Toda afirmación lleva evidencia `path:line`. Lo que no se pudo verificar
  desde el código está marcado como tal.

## Resumen

Ninguna tarjeta sobra **como información**: las trece cuentan cosas distintas
y ninguna se puede borrar sin perder algo. Lo que sobra son las **repeticiones**
(tres cifras repetidas entre tarjetas) y lo que sobra es, sobre todo, el
**rotulado**: la sección dice una cosa y muestra otra.

| # | Tarjeta | Veredicto | Lo que hay que arreglar |
|---|---|---|---|
| 1 | La corrida en números | KEEP WITH CHANGES | "Transacciones" y "Población" miden dos cosas distintas según el modo; la cabecera dice "del instante" y dos de las seis son publicadas; "Caudal" sin marcar como derivado |
| 2 | Los recursos que aprietan | KEEP WITH CHANGES | El veredicto "el que más aprieta" compara un % con un conteo y sale siempre del primer panel; el pie dice "tiempo de la corrida" y divide por el reloj; el badge dice "reporte" y son cifras del replay; las barras no tienen escala |
| 3 | Contenido de las colas a lo largo del tiempo | KEEP WITH CHANGES | La media no va marcada como derivada ni da la fórmula; falta la etiqueta de unidades |
| 4 | Distribución de las esperas | KEEP WITH CHANGES | Otra media (la de las salidas) que no es la de la tabla; cifras vivas en gris; el tooltip muestra el centro del intervalo, no el intervalo; el ancho de los bins depende del reloj |
| 5 | Indicadores por cola | KEEP WITH CHANGES | "Espera media" no es el `AVE.TIME` del motor; subtítulo con tres afirmaciones falsas; sin badge ni pie; falta el contenido actual |
| 6 | Ocupación de las facilities | KEEP WITH CHANGES | El % es una cifra viva pintada como publicada; el gris del futuro se lee como "libre"; con preemptions el largo pintado no cuadra con el % |
| 7 | Indicadores por facility | KEEP WITH CHANGES | La columna "Reintentos" es **siempre 0**; el subtítulo atribuye al motor cifras del replay; sin badge ni fórmulas |
| 8 | Unidades en uso — almacén | KEEP WITH CHANGES | "Reintentos" no es el del motor; el subtítulo llama "bloqueado" a lo que es "lleno"; la selección sobrevive a la corrida nueva |
| 9 | Tamaño de la cadena | KEEP WITH CHANGES | "Tiempo medio" no es el `AVE.TIME` del motor; no muestra el tamaño medio que el motor sí publica; rótulo "la cadena más larga" que no es eso |
| 10 | Histograma de la tabla | KEEP WITH CHANGES | Badge "reporte del motor" sobre cifras del instante; la agrupación de tablas con >200 intervalos no existe aunque la doc la promete; media y σ sin marcar |
| 11 | Las tablas del modelo | KEEP WITH CHANGES | Badge "reporte del motor" sobre cifras del replay; asterisco sin pie; `TABLE` y `QTABLE` sin distinguir |
| 12 | Transacciones por bloque | KEEP WITH CHANGES | Badge "reporte del motor" sobre cifras vivas; la cifra publicada se ignora; el generador entra al ranking con las transacciones creadas; dos bloques homónimos indistinguibles |
| 13 | Reintentos por bloque — la contención | KEEP WITH CHANGES | Cuenta esperas, no lo que cuenta el motor; el veredicto compara absolutos sin tráfico |
| 14 | Las piezas compartidas | KEEP WITH CHANGES (familia) | El violeta significa tres cosas; "Sin datos" no es "publicado"; faltan las piezas comunes (`EmptyState`, un `EntityPicker`, `shareOf`) |

### Las prioridades

**P0 — cifras que engañan (arreglar primero)**

1. **Lo del instante se presenta como lo publicado.** Cuatro tarjetas llevan el
   badge "reporte del motor" y muestran cifras del replay
   (`MonitorCharts.tsx:203`, `:211`, `:222`, `:231`); y en las que sí son
   publicadas, la leyenda del pie no lo aclara. El color del violeta tampoco
   ayuda: significa "derivado" y "del instante" a la vez (`StatTable.tsx:62-63`).
2. **El reloj y las cifras no son del mismo instante durante la
   reproducción.** `closeFigures` (`monitor/stats.ts:434-464`) cierra el área con
   el contenido congelado del playhead mientras el reloj del store avanza
   (`MonitorCharts.tsx:57`, `:63-66`): la utilización de una facility ocupada
   en ese frame tiende a 100 % y el contenido medio de una cola vacía decae a 0.
   El store ya publica las figuras frescas (`monitor/live.ts:204`) y ningún
   gráfico las usa.
3. **"Espera media" y "Tiempo medio" no son las del motor.** La app usa
   `totalTime/entradas`; el motor define `AVE.TIME = área/entradas`
   (`crates/genmsi-core/src/stat.rs:211-223`, `:439-451`). Y la tarjeta de
   esperas muestra una tercera cifra: la media de las salidas. Tres números
   distintos con el mismo nombre en tres tarjetas.
4. **Los reintentos no son los del motor.** La columna de facilities siempre
   muestra 0 (nadie emite `facility-retry`), y el ranking por bloque cuenta
   cualquier espera mientras el motor sólo cuenta el arrebato rechazado
   (`crates/genmsi-core/src/sim.rs:2730`).
5. **Los pies dicen el divisor equivocado.** "Tiempo de la corrida" donde el
   divisor es el reloj del instante (`MonitorCharts.tsx:126`, `:151`), y ningún
   pie nombra la fórmula de la utilización de los almacenes.

**P1 — información que el motor ya tiene y la app no muestra**

6. `Storage.delay` (la corrida terminó bloqueada), `overflow_sum` /
   `overflow_sum_squares` (media y σ de los desbordes), `entities.logic` y
   `entities.savevalues` (el estado del modelo: los switches que gobiernan el
   flujo), `seed` y `title`.
7. El contenido actual de las colas: ya se calcula y no se muestra.
8. La agrupación de tablas con más de 200 intervalos: la promete
   `docs/app.md:377-378` y no existe.
9. Falta el tercer estado: "todavía no" se ve como "publicado 0".
10. `pending`, `groups` y `logic_switches` están en el reporte de texto pero no
    en el JSON: requieren campo nuevo en el motor.

**P2 — forma**

11. Una sola leyenda, un solo color y un tipo `Metric` único
    (`valor`, `origen`, `fórmula`) pintado por un solo componente.
12. Un `Instant` único que ate reloj y figuras, y ninguna métrica derivada si
    ambos no son del mismo instante.
13. Piezas comunes: `<EmptyState/>`, un único `EntityPicker`, `shareOf()`,
    `TimeSeriesChart` con dominio Y desde 0 y estado vacío.
14. Fusionar *Las tablas del modelo* con el histograma (una fila elige la
    tabla) para que media, σ y desbordes se digan una sola vez.
15. Tests: hoy ninguna tarjeta tiene test (el runner es `node --test`, sin DOM).
    La ruta que funciona es la de `derived.ts`: mover la lógica de las piezas a
    módulos puros y probarlos ahí.

### Lo que no se toca

Lo ya decidido y que esta auditoría **no** reabre: la línea 0/1 de facilities
(que se cayó, `odd/tasks/genmsi-graficos-estadisticas.md` S5/L4), el alcance
curado en seis grupos (S1), el eje del tiempo desde 0 (S7) y el reloj
sincronizado (S4). El problema de la sección no es qué muestra, sino **cómo lo
dice**.

## Las tarjetas

### La corrida en números

<!-- SECCION:RunSummary -->

### Los recursos que aprietan

**Qué muestra.** Tres paneles de barras: utilización de facilities, contenido
medio de colas y utilización media de almacenes. Los valores salen de las
figuras del **replay** cerradas en el reloj (`closeFigures`), y las métricas
son ratios: `busyTime/elapsed` (`monitor/derived.ts:81-83`),
`area/elapsed` (`monitor/derived.ts:56-58`), `area/(elapsed·capacity)`
(`monitor/derived.ts:109-113`). El reporte sólo aporta los **nombres** y la
**capacidad** del almacén (`charts/entities.ts:15-20`, `:216`); el tiempo es
`elapsed.live = clock - start_time` (`MonitorCharts.tsx:74`).

**1. ¿Tiene sentido?** La idea es buena —el cuello de botella de un vistazo—,
pero dos de los tres paneles repiten columnas que ya existen más abajo:
*Contenido medio* de `QueueTable.tsx:36-43`, *Utilización* de
`FacilityTable.tsx:37-44` y *Utilización media* de `StorageSeries.tsx:103-109`,
con las mismas cifras vivas y la misma derivación. Lo único propio de la
tarjeta es la comparación entre familias… que es justo la parte rota.

**2. ¿Se muestra correctamente?**

- [alta] **La frase "el recurso que más aprieta" es casi siempre falsa.** Cada
  panel normaliza por su propio máximo (`ResourceRanking.tsx:46-53`), así que
  el líder de cada panel tiene `share = 1` y el ganador se decide por el orden
  del array (facilities primero, `ResourceRanking.tsx:96-100`): gana la
  facility con mayor utilización, aunque esté al 3 % y una cola cargue 40
  unidades. Además compara un % con un número de unidades: no hay criterio
  (`MonitorCharts.tsx:123`, `ResourceRanking.tsx:120-126`).
  *Verificado*: `sharesOf` divide por el máximo del panel y `busiest` ordena
  por esa cuota relativa.
- [alta] **El pie dice el denominador equivocado.** "Utilización = tiempo
  ocupado / **tiempo de la corrida**" (`MonitorCharts.tsx:126`), pero el código
  divide por `elapsed.live` (el reloj del instante). Quien rehaga la cuenta
  con la duración de la corrida obtiene otra cifra.
- [alta] **El badge miente:** "derivado del reporte" (`MonitorCharts.tsx:124`)
  cuando las cifras son del replay; el reporte sólo da nombres y capacidad
  (`charts/entities.ts:15-20`). El badge es la promesa de procedencia de la
  sección (`ChartCard.tsx:6-7`).
- [media] **Las barras no tienen escala:** cada panel divide por su máximo
  (`ResourceRanking.tsx:46-53`), sin eje ni referencia al 100 %; un panel cuyo
  máximo es 3 % dibuja una barra llena igual que uno al 98 %.
- [media] **Redondeo a entero** (`ResourceRanking.tsx:78`, `:92`): 99,6 % se
  muestra "100 %", justo lo que la tarjeta afirma; y un valor 0 dibuja una
  astilla de 2 % (`BarList.tsx:46`).
- [media] **No se marcan como cifras del instante:** la leyenda dice que el
  violeta es lo vivo (`MonitorCharts.tsx:237-240`) y las tablas lo tiñen
  (`StatTable.tsx:63`); aquí el número va en slate (`BarList.tsx:41`).
- [media] **Derivado sin asterisco**, contra la regla de la sección
  (`docs/app.md:380-383`, `StatTable.tsx:46`): toda la tarjeta es derivada y
  nada lo dice.
- [baja] **Estado vacío:** `Panel` devuelve `null` si la familia no tiene
  entidades (`ResourceRanking.tsx:28-29`), así que el mensaje de `BarList`
  (`BarList.tsx:28-32`) nunca aparece; sin las tres familias la tarjeta queda
  en blanco.
- [baja] "y N más, con menos carga" (`ResourceRanking.tsx:39`) oculta
  entidades que las tablas sí muestran, y "carga" significa contenido.
- [baja] Título mezcla idiomas: "Utilización de facilities"
  (`ResourceRanking.tsx:105`).
- [baja] Los recursos que el replay aún no tocó salen con 0 y su astilla de
  2 % (`charts/entities.ts:157-160`).

**Lo que el motor publica y la tarjeta ignora:** `busy_time`, `retry` e
`inter` de facilities; `area`, `total_time`, `retry` y `max` de colas; `area`,
`max_use`, `retry` y `delay` de almacenes (`types/generated/Facility.ts`,
`Queue.ts`, `Storage.ts`). Podría mostrar la cifra publicada junto a la del
instante, que es justo lo que la sección promete.

**Tests.** Cubierta la parte numérica: `tests/entities.test.ts:117-124`
verifica `meanContent = área/tiempo del instante` y `tests/derived.test.ts:47-52, 66-71`
las tres razones. **Sin cobertura** para `ResourceRanking.tsx` ni `BarList.tsx`:
ningún test los importa, así que `sharesOf`, el corte de top-5, la frase del
"más aprieta" y el estado vacío no se verifican.

**3. ¿Otra forma?** Mantener los tres paneles pero con **escala absoluta**
(barra = % de 100 %, o contenido sobre el máximo del modelo, con la cifra
escrita al lado), eliminar la frase "el que más aprieta" o reemplazarla por un
criterio comparable (p. ej. % de capacidad consumida) y marcar lo vivo en
violeta. Alternativa más útil: **una sola tabla "recurso / carga / de qué /
ahora / corrida"**, ordenada por una presión comparable, que además muestre
la cifra publicada que el motor ya da.

**Veredicto.** KEEP WITH CHANGES
**Riesgo de equivocarse.** El lector lleva el nombre de un recurso que no
aprieta a rediseñar el modelo; las barras llenas le hacen creer que un recurso
al 3 % está al límite; y el pie, al decir "tiempo de la corrida", lo lleva a
calcular una utilización distinta de la mostrada.

### Indicadores por cola

**Qué muestra.** Una tabla con una fila por cola y cinco columnas: *Contenido
medio\**, *Espera media\**, *Máximo*, *Entradas*, *Espera cero\**
(`charts/QueueTable.tsx:28-60`). Las filas las arma `queuesOf`
(`charts/entities.ts:148-176`) a partir de las figuras del replay cerradas en
el reloj (`MonitorCharts.tsx:66-70`); del reporte sólo se toman los **nombres**
(`entities.ts:136-146`). Los valores del motor que existen y no se usan:
`content`, `total_time`, `area`, `retry` (`types/generated/Queue.ts:18-42`).

**1. ¿Tiene sentido?** Sí: la serie da el contenido a lo largo del tiempo y el
histograma la forma de las esperas, pero ninguna de las dos dice por cola
cuántas entradas tuvo, cuál fue su máximo ni qué share esperó cero. La única
redundancia real es *Contenido medio*, ya mostrado como ranking en *Los
recursos que aprietan* (`ResourceRanking.tsx:110-112`).

**2. ¿Se muestra correctamente.** El corte en el reloj sí está bien:
`closeFigures(state.figures, clock)` cierra el área en el reloj y el divisor es
`clock - report.start_time` (`MonitorCharts.tsx:66-76`), con `start_time = 0`
(`crates/genmsi-core/src/sim.rs:860`) y el reloj en segundos de corrida
(`monitor/live.ts:50`); *Contenido medio* coincide con el `AVE.CONT` del motor
al final de la corrida (`crates/genmsi-core/src/stat.rs:204-210`,
`crates/genmsi-core/src/report.rs:199`). Los problemas:

- [alta] ***Espera media* no es la espera media que publica el motor.** El
  motor define `AVE.TIME = área / entradas` (`crates/genmsi-core/src/stat.rs:211-223`,
  `report.rs:205`) —la integral del contenido, que incluye a las que siguen
  esperando—; la app calcula `totalTime / entradas`
  (`monitor/derived.ts:59`), la suma de las esperas reales de las que
  salieron. No son el mismo número ni la misma unidad, y el propio
  `derived.ts:17-18` afirma que la fórmula es la del motor.
  *Verificado contra `crates/genmsi-core/src/stat.rs:211-223`.*
- [alta] La tarjeta no tiene badge ni pie (`MonitorCharts.tsx:149-154`;
  `ChartCard.tsx:44-48` sólo los pinta si se los pasan), contra la regla 5 de la
  sección. Y el subtítulo dice tres cosas falsas: "se derivan del reporte del
  motor" (los datos son del replay), "divididos por el tiempo de la corrida"
  (el divisor es el tiempo del instante) y "el área y el tiempo total,
  divididos por el tiempo" (sólo *Contenido medio* se divide por el tiempo;
  *Espera media* y *Espera cero* se dividen por entradas,
  `monitor/derived.ts:58-60`).
- [media] La leyenda de la sección miente aquí: "las demás son las que el
  motor publicó" (`MonitorCharts.tsx:238-240`), pero en esta tabla lo no
  violeta es una cola que el replay todavía no tocó y muestra ceros
  (`entities.ts:155-174`): un 0 que no es el 0 publicado.
- [media] Falta el **contenido actual**, que ya está calculado
  (`entities.ts:165`) y publicado (`Queue.ts:18`): es el número vivo más útil
  de una cola; en su lugar la tabla muestra el máximo, que es histórico.
- [media] El motor publica el `retry` de la cola (`Queue.ts:42`,
  `report.rs:212`) y la app no lo muestra; el replay no tiene cambio de
  reintento de cola (`monitor/stats.ts:107-175`), así que sólo se puede leer
  del reporte.
- [baja] Unidades mezcladas sin rótulo: *Máximo* es en unidades, *Entradas*
  son transacciones, *Espera media* en segundos.
- [baja] *Espera cero* redondea al entero (`QueueTable.tsx:14-16`) mientras el
  resto usa 3 decimales (`format.ts:12-18`).
- [baja] Las filas se ordenan con `localeCompare` (`entities.ts:142-145`) y no
  con el orden numérico del motor (`report.rs:189`, `:770`); cosmético.
- [baja] La tarjeta se dibuja aunque no haya colas (`MonitorCharts.tsx:149-154`,
  frente a las condicionales de facilities/almacenes/cadenas en `:156`, `:175`,
  `:187`) y muestra "Sin datos." (`StatTable.tsx:33`).
- [baja] Las filas se keyan por índice (`StatTable.tsx:53`): hoy no rompe nada
  (los nombres son únicos, `entities.ts:141`); no verificado en runtime.

**Cobertura de tests.** Hay cobertura de datos, no del componente:
`tests/entities.test.ts:99-132` (contenido/entradas/zeroShare/meanWait y "el
tiempo del instante"), `tests/derived.test.ts:17-44` (las tres fórmulas y el
caso sin tiempo), `tests/stats.test.ts:7-58` (área, esperas nulas, unidades) y
`tests/replay.test.ts:121-197` (contenido y esperas de la traza). **No hay
ningún test que compare *Espera media* con el `AVE.TIME` del motor**, ni tests
de render (no hay `.tsx` en `tests/`).

**3. ¿Otra forma?** Una fila por cola con *contenido actual* (unidades) +
*entradas* + *máximo* + *espera media* + *% de espera nula*, ordenada por
entradas, y una minigráfica de contenido tomada de la serie que ya existe
(`QueueSeries`), con las unidades en el rótulo y la fórmula en el pie.

**Veredicto.** KEEP WITH CHANGES
**Riesgo de equivocarse.** Si el lector compara *Espera media* con el
`AVE.TIME` del reporte del CLI (o se apoya en el subtítulo "derivado del
reporte"), concluye que la app calcula mal la espera; y si lee los ceros de una
cola que el replay aún no tocó como "el motor publicó 0", diagnostica una cola
que no existe.

### Contenido de las colas a lo largo del tiempo

**Qué muestra.** El contenido de hasta tres colas elegidas por el lector,
instante a instante, más una línea punteada horizontal por cola con su
**contenido medio**. La curva sale de la proyección de la traza:
`monitor.queuePoints` (los instantes en que el contenido cambia,
`monitor/replay.ts:1290-1300`), las filas las arma `seriesFrom`
(`monitor/series.ts:40-77`) y las dibuja `TimeSeriesChart`
(`charts/TimeSeriesChart.tsx:63-118`). La línea punteada usa
`QueueNumbers.meanContent` (`charts/entities.ts:147-176`), un ratio *vivo*:
área cerrada en el reloj ÷ `elapsed.live` (`MonitorCharts.tsx:66-80`,
`monitor/stats.ts:433-441`). Badge: "replay" (`MonitorCharts.tsx:135`).

**1. ¿Tiene sentido?** Sí, y es la única tarjeta que cuenta la *historia* de la
cola (cuándo se acumula y cuándo se vacía); la tabla da los números
(`charts/QueueTable.tsx`) y el histograma da las esperas
(`charts/WaitDistribution.tsx`), ninguna cuenta el cuándo. No es redundante.
El usuario la confirmó explícitamente
(`odd/tasks/genmsi-graficos-estadisticas.md:55-57`, `:172`).

**2. ¿Se muestra correctamente.** En lo estructural, sí: el eje arranca en 0 y
termina en el fin de la corrida (`TimeSeriesChart.tsx:66-71`,
`endTime = monitor.endTime`), la serie se corta en el reloj
(`series.ts:50-55`) y la vertical marca el instante (`TimeSeriesChart.tsx:107`);
la media divide el área *viva* por el tiempo *vivo*, no mezcla cifras
publicadas (`entities.ts:161` + `MonitorCharts.tsx:74`); al final de la corrida
esa media coincide con la del motor (ambos integran desde la primera
referencia de la cola: `monitor/stats.ts:179-187` vs
`crates/genmsi-core/src/stat.rs:615-618`); el reloj nunca pasa de `endTime`
(`components/MonitorView.tsx:350-352`), así que la vertical nunca sale del
eje; no hay colisión de claves (los nombres son únicos,
`TimeSeriesChart.tsx:85`, `:109`). Los hallazgos:

- [media] **La media no está marcada como derivada ni da la fórmula.** La nota
  dice "derivado del área que cubrió" (`QueueSeries.tsx:73-81`) pero no
  "área / tiempo", y no lleva asterisco, como sí lo hacen el resto de la
  sección (`docs/app.md:382-384`; cf. `charts/StorageSeries.tsx:105` "derivada:
  área / tiempo / capacidad" y el pie de la tarjeta de recursos,
  `MonitorCharts.tsx:120`). El badge "replay" tampoco lo dice. Además la
  convención de color de la sección ("violeta = cifras del instante",
  `MonitorCharts.tsx:239-240`) no aplica a esta línea, dibujada en el color de
  la serie: el lector puede leerla como una cifra publicada de la corrida
  completa.
- [media] **Falta la etiqueta de unidades**, que la decisión del usuario
  listaba explícitamente (`odd/tasks/genmsi-graficos-estadisticas.md:55-57`).
  El `YAxis` no tiene rótulo (`TimeSeriesChart.tsx:73-77`) y el subtítulo sólo
  explica los escalones (`MonitorCharts.tsx:134`); el contenido es en
  *lugares* (unidades), no en transacciones, cuando el modelo declara
  `Cantidad` > 1 (`monitor/replay.ts:382-395`,
  `crates/genmsi-core/src/stat.rs:132-134`).
- [baja] **Estado vacío mal redactado**: en una corrida sin colas el picker no
  renderiza nada (`charts/EntityPicker.tsx:18-20`) y la tarjeta dice "Elige una
  cola para verla" (`QueueSeries.tsx:67-70`), pero la tarjeta se monta siempre
  (`MonitorCharts.tsx:132-138`), al contrario que facilities/almacenes/cadenas
  (`:148`, `:168`, `:178`).
- [baja] **El nombre de la cola es la clave del dato**: una cola llamada
  `time` pisaría la clave `time` de la fila (`series.ts:28`, `:61`, `:70`) y
  rompería el eje X; un nombre con punto lo interpretaría Recharts como ruta
  anidada (no verificado: no hay `node_modules/recharts` en el repo).
- [baja] **El color depende del orden de la selección**
  (`QueueSeries.tsx:46-51`, `charts/palette.ts:25-27`): al quitar una cola, las
  otras cambian de color.
- [baja] La selección por defecto son las tres primeras colas que menciona la
  corrida, no las más contendidas (`QueueSeries.tsx:36-38`).

**Tests.** Cubierta sólo de forma indirecta: `tests/series.test.ts:11-60` (la
serie arranca en 0, se detiene en el reloj, unión de instantes),
`tests/replay.test.ts:159-191` (`queuePoints` = los instantes del cambio y las
filas con su valor), `tests/entities.test.ts:120-128` (`meanContent` =
área/tiempo del instante), `tests/derived.test.ts:17-22`. **No hay** prueba que
renderice la tarjeta (el runner es `node --test tests/*.test.ts`,
`crates/genmsi-app/package.json:11`, sin DOM): no se cubren la selección por
defecto, el estado vacío, ni que la línea punteada tome la media viva y no la
publicada.

**Lo que el motor publica y la tarjeta ignora:** el **máximo** de cada cola
(`entities.ts:51`, publicado en `types/generated/Queue.ts`) y sus
**reintentos** (`retry`: las entradas que fallaron; ninguna tarjeta de la
sección los muestra por cola). El `retry` es lo que explica por qué la cola se
acumula.

**3. ¿Otra forma.** Una sola cola a la vez (como `StorageSeries`,
`charts/StorageSeries.tsx:37-56`) con un selector, el **máximo** como segunda
línea punteada y el `retry` en la nota: misma información, menos curvas
superpuestas y la causa de la acumulación visible. Manteniendo las tres curvas,
el cambio mínimo es etiquetar el eje ("unidades en la cola"), poner la
fórmula en el pie y marcar la media como derivada.

**Veredicto.** KEEP WITH CHANGES
**Riesgo de equivocarse.** Si la media se tomara del área publicada en vez de
la viva, mostraría el promedio de toda la corrida mientras la curva sólo llega
al reloj —el error que ya mordió una vez
(`odd/tasks/genmsi-graficos-estadisticas.md:119-125`): una cola de cuatro
unidades aparecería con contenido medio 330. Sin etiqueta de unidades, una
cola de *lugares* se lee como una cola de transacciones.

### Indicadores por facility

**Qué muestra.** Una tabla de 7 columnas (`Facility`, `Ahora`, `Utilización`,
`Servicio medio`, `Entradas`, `Interrupciones`, `Reintentos`) construida sobre
`FacilityNumbers` (`charts/FacilityTable.tsx:25-58`). Los valores salen de
`facilitiesOf()` (`charts/entities.ts:180-202`), que combina los **nombres**
publicados por el motor (`entities.ts:141-143`) con las **figuras del replay**
cerradas en el reloj (`MonitorCharts.tsx:66-76`). La columna `Ahora` es el
dueño de la facility en el instante (`FacilityTable.tsx:28-30`). Todo es cifra
viva: las dos derivadas se dividen por `elapsed.live = clock - report.start_time`
(`MonitorCharts.tsx:73-74`, `monitor/derived.ts:81-84`), la misma aritmética
del motor (`crates/genmsi-core/src/stat.rs:89-105`). La tarjeta se monta en
`MonitorCharts.tsx:170-177`.

**1. ¿Tiene sentido?** Sí: es el resumen por recurso de la sección, y la
columna `Ahora` (quién tiene la facility en este instante) no la dice ninguna
otra tarjeta; el `FacilityTimeline` (`MonitorCharts.tsx:156-168`) muestra
*cuándo* estuvo ocupada, no *quién* la tiene. Parcialmente redundante, en
cambio, con el panel "Utilización de facilities" de *Los recursos que
aprietan* (`ResourceRanking.tsx:70-77`), que recalcula el mismo `utilization`
con el mismo `facilitiesOf` para el top-5.

**2. ¿Se muestra correctamente.** El reloj y los denominadores están bien; el
origen declarado y una columna, no.

- [alta] **La columna `Reintentos` es siempre 0**: el replay nunca emite el
  cambio `facility-retry` (el `case "Seize"` sólo genera `facility-seize`,
  `monitor/replay.ts:582-599`; el único que suma el contador es
  `monitor/stats.ts:327-330`, sin productor), así que `retry` sale siempre de
  `figure?.retry ?? 0` (`entities.ts:198`). El motor sí publica `retry`
  (`crates/genmsi-core/src/stat.rs:44-46`) y la app lo ignora: ni la tabla de
  esta tarjeta ni la pestaña "Facilidades" del reporte lo muestran
  (`components/ReportView.tsx:236-259`). El propio test lo delata: el reporte
  publica `retry: 1` y la figura viva `retry: 0`
  (`crates/genmsi-app/tests/entities.test.ts:43` vs `:88`). Y el chequeo de
  fidelidad no lo compara (`monitor/replay.ts:1435-1441` compara `busy`,
  `entries`, `busyTime`, `inter`), o sea que nadie lo atrapa.
  *Verificado*: `facility-retry` no tiene productor en `monitor/replay.ts`;
  el único `kind: "...-retry"` emitido es el de storage (`:664`).
- [media] **El subtítulo atribuye al motor cifras que salen del replay**:
  "se derivan del reporte del motor; las interrupciones y los reintentos los
  cuenta el motor" (`MonitorCharts.tsx:173`), pero todo lo que se ve son
  figuras del instante (`entities.ts:180-202`). La tarjeta se contradice con
  su propio texto.
- [media] **No dice las fórmulas**: el asterisco marca las derivadas
  (`StatTable.tsx:46`) y la tarjeta no da ni el denominador de la utilización
  ni el del servicio medio, como sí hace la tarjeta de recursos
  (`MonitorCharts.tsx:126`). Además **no tiene badge**
  (`MonitorCharts.tsx:170-175`; `ChartCard.tsx:36-45`), que es la regla 5 de
  la sección.
- [media] **`Servicio medio` de una facility nunca tomada muestra "0,0 s"**
  (`FacilityTable.tsx:42`), donde el motor escribe "no computado" para el caso
  sin divisor (`crates/genmsi-core/src/report.rs:64-71` y `:162-168`).
- [baja] `libre` no distingue "libre ahora" de "el replay todavía no la
  conoce" (`FacilityTable.tsx:30` vs `entities.ts:201`).
- [baja] La clave de fila es el índice (`StatTable.tsx:53`); hoy no rompe nada
  porque los nombres son únicos por construcción (`entities.ts:142`), pero por
  nombre sería más robusto.
- [baja] El motor publica también los **waiters** de cada facility (columna
  `DELAY`, `crates/genmsi-core/src/report.rs:176`), que no llegan al JSON
  (`stat.rs:36-39`, `serde(skip)`) y que la app no muestra; el replay sí tiene
  el dato (`monitor/replay.ts:783-787`).

**Cobertura de tests.** La aritmética sí (`tests/derived.test.ts:46-61`), el
armado de filas también (`tests/entities.test.ts:108-116`), pero **el
componente no se renderiza en ningún test** (nada importa `FacilityTable`) y
`entities.test.ts` no afirma `utilization`, `meanService`, `inter` ni `retry`
de la facility.

**3. ¿Otra forma.** Una fila por facility con: chip de estado (`libre` /
`tx N` / `esperando N`), una barra fina de utilización (en vez del porcentaje
redondeado) y las cifras de contención (`interrupciones`, `esperando`),
ordenadas por utilización; el `Reintentos` sale de la fila publicada o se
elimina. Justificación: se lee de un vistazo y reemplaza una columna muerta por
la métrica que sí muestra el motor (`DELAY`).

**Veredicto.** KEEP WITH CHANGES
**Riesgo de equivocarse.** Alto: una columna que dice "0" cuando el reporte
dice "1" hace creer que no hubo contención; y si el pie de la sección no se
lee, el lector entiende estas cifras como las publicadas de la corrida
completa y las compara con el reporte, que no coinciden hasta el final (y con
`Reintentos` nunca coinciden).

### Ocupación de las facilities a lo largo del tiempo

**Qué muestra.** Una pista por facility: cada tramo de color es un intervalo en
que estuvo tomada; el fondo gris es el resto; la línea vertical marca el reloj.
Los tramos salen de `segmentsOf(monitor.facilityPoints.get(name), clock)`
(`charts/FacilityTimeline.tsx:36-39`, `monitor/series.ts:98-118`) y los puntos
vienen del replay: cada cambio de dueño (`monitor/replay.ts:1266-1272`). La
escala es `span = monitor.endTime` = `trace.end_time`
(`FacilityTimeline.tsx:44`, `monitor/replay.ts:824`), y `report.start_time` es
siempre 0.0 (`crates/genmsi-core/src/sim.rs:860`), así que el eje 0→fin
coincide con el pie de la sección (`MonitorCharts.tsx:241-242`). A la derecha
de cada pista se imprime la utilización (`FacilityTimeline.tsx:71-73`):
`busyTime / reloj` (`charts/entities.ts:187-199`, `monitor/derived.ts:81-85`).

**1. ¿Tiene sentido.** Sí: es la única tarjeta que dice *cuándo* estuvo
ocupada cada facility (la tabla dice *cuánto*), y es el reemplazo acordado del
gráfico 0/1 que se cayó (`odd/tasks/genmsi-graficos-estadisticas.md:58-59`).
No es redundante con el resto: el reloj la hace viva y la tabla no. Lo único
redundante es el porcentaje que reimprime.

**2. ¿Se muestra correctamente.** Más o menos: el corte en el reloj está bien,
pero el contrato de la sección se incumple en tres puntos.

- [media] **El % es una cifra viva mostrada como si fuera publicada.** Es
  `busyTime/reloj` (`monitor/derived.ts:83`), o sea del instante, pero se
  pinta en `text-slate-600` (`FacilityTimeline.tsx:71`), sin violeta, y el pie
  dice que lo no violeta es lo publicado (`MonitorCharts.tsx:238-240`). Tampoco
  lleva asterisco ni fórmula: el badge es "replay" y no hay `footnote`
  (`MonitorCharts.tsx:157-168`), al contrario de la tabla, que marca esa misma
  columna como derivada y viva (`charts/FacilityTable.tsx:30-36`,
  `StatTable.tsx:49-51`, `:66-73`).
- [media] **El gris después del reloj afirma que la facility seguirá libre.**
  La pista tiene fondo `bg-slate-100` en todo el ancho
  (`FacilityTimeline.tsx:56`) y el subtítulo dice "el gris la libre"
  (`MonitorCharts.tsx:159`): el futuro, que el replay todavía no conoce, se
  presenta como libre conocido. Los tramos sí se cortan bien
  (`series.ts:98-118`, test `crates/genmsi-app/tests/series.test.ts:116-133`,
  decisión L8 en `odd/tasks/genmsi-graficos-estadisticas.md:180-185`), pero
  nada distingue "libre hasta ahora" de "todavía no se sabe". La serie de
  colas sí deja el futuro en blanco (`charts/TimeSeriesChart.tsx:60-77`).
- [media] **Con preemptions, los tramos no suman la utilización que muestra la
  tabla.** El `preempt` reinicia `seizedAt` sin acumular `busyTime`
  (`monitor/stats.ts:314-325`) mientras el dueño nunca pasa por `null`
  (`monitor/replay.ts:611-632`, `:1266-1272`), así que la pista lo pinta
  ocupado. El motor hace lo mismo (`crates/genmsi-core/src/sim.rs:1411-1414`,
  no re-verificado), de modo que el % coincide con el publicado, pero el
  dibujo no: rompe la coherencia que la sección promete ("los tramos de
  ocupación suman la utilización que muestra la tabla",
  `odd/tasks/genmsi-graficos-estadisticas.md:159-161`).
  *Verificado leyendo el código del replay y de las figuras; sin ejecutar un
  modelo con `PREEMPT`.*
- [baja] El % es la utilización *hasta el reloj* (denominador = reloj,
  `MonitorCharts.tsx:73-74`) pero está pegado a una pista que abarca la
  corrida entera (`FacilityTimeline.tsx:44`): el lector puede compararlo con
  el ancho completo y equivocarse; la tarjeta no dice "hasta el reloj".
- [baja] La línea del reloj se dibuja sobre las etiquetas del eje (el overlay
  es `inset-0` de toda la tarjeta, `FacilityTimeline.tsx:78-90`, y las marcas
  están debajo, `:91-104`); las marcas del tiempo repetidas dan claves de
  React duplicadas si la corrida dura menos de 4 s (`Math.round(span*share)`
  en `:45`, `key={tick}` en `:96`).
- [baja] La tarjeta muestra todas las facilities sin selector, al revés que
  las otras series de la sección (`charts/QueueSeries.tsx:33-40`).

**Cobertura.** El helper está probado (`tests/series.test.ts:65-84`, `:116-133`),
los puntos del replay también (`tests/replay.test.ts:159-171`) y las cifras
del instante (`tests/entities.test.ts:108-115`), pero **no hay ningún test que
una la longitud pintada con `busyTime`/utilización**, ni test de render del
componente (en `crates/genmsi-app/tests/` no hay pruebas de componente).

**Lo que el motor publica y esta tarjeta ignora:** las preempciones por
facility (`types/generated/Facility.ts:33-35`) y `available`
(`Facility.ts:25-27`) — la utilización **no** está en el JSON (sólo en el CSV,
`crates/genmsi-core/src/report.rs:1522-1528`), así que derivarla y marcarla
con `*` es lo correcto.

**3. ¿Otra forma.** Mantener la pista y: pintar la región a la derecha del
reloj con un patrón "todavía no se sabe" en vez del gris "libre"; sacar el %
de la fila (o ponerlo junto a la línea del reloj, rotulado "utilización hasta
el reloj *", con la fórmula en el pie); marcar las preempciones sobre la
pista; y añadir un selector como las otras series. Con la misma información,
otra pregunta sería más útil: **cuántas facilities están ocupadas a la vez**
(un área apilada desde los mismos `facilityPoints`), un dato que hoy no
muestra ninguna tarjeta.

**Veredicto.** KEEP WITH CHANGES
**Riesgo de equivocarse.** Si el gris se lee como "libre", el lector concluye
que el recurso no se vuelve a necesitar; si la pista pintada no cuadra con el
% (preempciones), la utilización que el lector calcula a ojo contradice la de
la tabla de al lado y la sección pierde su garantía de coherencia.

### Tamaño de la cadena

**Qué muestra.** El tamaño de una *cadena de usuario* GPSS (la que se llena con
`LINK` y se vacía con `UNLINK`), no la cadena del flujo del diagrama ni el
ensamblaje: el motor sólo crea entidades de cadena en las ramas `Link`/`Unlink`
(`crates/genmsi-core/src/sim.rs:1711`, `:2770`) y en la expresión `CH:`
(`sim.rs:3294`). La curva es la serie exacta del tamaño, reconstruida desde la
traza (`monitor/replay.ts:1284-1292`, `monitor/series.ts:41-77`); las cuatro
cifras salen de las figuras vivas de la cadena (`charts/entities.ts:236-254`,
`monitor/stats.ts:362-390`). Sólo se monta si hay cadenas
(`MonitorCharts.tsx:189`), así que en un modelo sin cadenas la tarjeta no
aparece.

**1. ¿Tiene sentido?** Sí, y no es redundante: es la única vista de las
cadenas, y en los modelos de referencia con `LINK` describe el backlog de un
conjunto de transacciones encadenadas, que ni las colas ni las facilities
muestran (p. ej. `MONORRIEL` con 504 entradas y 215 en la cadena al final:
`fixtures/outputs/monorriel__monorriel.report.txt:126-130`; también TP7, TP8,
TP10). El riesgo de confusión de vocabulario es real: `chainsOf` también
existe en `graph/graph.ts:583` y ahí "cadena" es otra cosa.

**2. ¿Se muestra correctamente.** Bien en lo esencial (eje de 0 al final, línea
del reloj, serie exacta, cifras del instante), pero:

- [media] "Tiempo medio" no es el `AVE.TIME` del motor: la tarjeta calcula
  `totalTime / entradas` (`charts/ChainSeries.tsx:44`), mientras el motor
  define el tiempo medio de la cadena como `área / entradas`
  (`crates/genmsi-core/src/stat.rs:439-451`) y lo imprime como `AVE.TIME`
  (`crates/genmsi-core/src/report.rs:254`, `:272`). Además `total_time` sólo
  suma los miembros que ya salieron (`monitor/stats.ts:389`), así que la cifra
  subestima el tiempo en cadena mientras la corrida avanza.
  *Verificado contra `crates/genmsi-core/src/stat.rs:439-451`.*
- [media] La app ya calcula el tamaño medio (`area / elapsed`,
  `charts/entities.ts:243,251`, `monitor/derived.ts:131-132`) y el motor lo
  publica (`AVE.CONT`, `report.rs:264`), pero la tarjeta no lo muestra: es la
  métrica que acompaña naturalmente a la curva.
- [media] Posible "+1" fantasma: el motor tiene el indicador de `LINK`; la
  primera transacción que llega a una cadena con el indicador apagado pasa al
  destino y **no** entra en la cadena, y las entradas sólo se incrementan para
  las que entran (`sim.rs:1705-1722`, `:1730-1735`). El replay cuenta una
  entrada en cada `enter` de un bloque `Link` (`monitor/replay.ts:683-690`), y
  la traza registra el `enter` antes de decidir (`sim.rs:1166-1169`). Si el
  bloque `Link` tiene salida conectada
  (`crates/genmsi-core/src/program.rs:815-821`), contenido, máximo y entradas
  quedarían +1 desde ese instante. **No verificado en ejecución**; ningún
  modelo de las fixtures tiene `LINK` con destino
  (`fixtures/outputs/TP7__Clase_7_1_Link_Unlink.gpss.txt:3`), por lo que el
  test de fidelidad nunca lo ejercita.
- [baja] El rótulo "Máximo / la cadena más larga" (`ChainSeries.tsx:82-84`) no
  describe lo que muestra: es el tamaño máximo de *esta* cadena
  (`monitor/stats.ts:367`).
- [baja] "Entradas / enlaces hechos en la corrida" (`ChainSeries.tsx:88-90`) es
  acumulado hasta el reloj, no de la corrida completa; el pie debería decirlo.
- [baja] El color de las cuatro cifras depende de `live` = "el replay ya conoce
  esta cadena" (`entities.ts:252`). La cadena existe en el reporte desde el
  inicio (`entities.ts:126-133`), así que antes del primer `LINK` la tarjeta
  muestra ceros en pizarra (el color reservado para lo publicado) cuando en
  realidad son ceros del instante.
- [baja] El estado vacío "El modelo no tiene cadenas" (`ChainSeries.tsx:36-42`)
  es inalcanzable: la tarjeta sólo se monta con `chains.length > 0`
  (`MonitorCharts.tsx:189`).
- [baja] La tarjeta muestra una sola cadena por vez con color fijo
  (`ChainSeries.tsx:69`) y duplica el marcado de los botones
  (`ChainSeries.tsx:50-68`) en vez de usar `EntityPicker` y `palette.colorOf`
  como hace `QueueSeries.tsx:46-51`; con cuatro cadenas hay que pulsarlas una
  por una.

**Tests.** La aritmética de las cadenas sí está cubierta:
`tests/stats.test.ts:139-174` (entrar/salir, salida múltiple),
`tests/derived.test.ts:75-78` (tamaño medio), `tests/replay.test.ts:238-252`
(`chainPoints`) y `tests/replay.test.ts:398-425` (los miembros que salen se
leen de la traza). No están cubiertos `chainsOf` (`tests/entities.test.ts:10-15`
sólo importa colas, facilities y almacenes) ni el componente. Y aunque la
fidelidad compara las cadenas con el reporte (`monitor/replay.ts:1462-1479`),
el modelo del test no tiene cadenas (`tests/replay.test.ts:31`).

**3. ¿Otra forma.** Multi-serie como *Colas*: `EntityPicker` + `colorOf` y
todas las cadenas superpuestas con su línea de tamaño medio punteada; y
sustituir "Tiempo medio" por "Tamaño medio" (`area/tiempo`), que sí es una
métrica del motor, dejando el tiempo total como cifra leída.

**Veredicto.** KEEP WITH CHANGES
**Riesgo de equivocarse.** El lector cree que hay una transacción más
encadenada de la que hay, compara un "tiempo medio" que no es el `AVE.TIME` del
reporte con el número del reporte, y toma el máximo de una cadena por el de la
más larga.

### Unidades en uso — almacén

**Qué muestra.** La curva escalonada (`stepAfter`) de unidades en uso del
almacén elegido, leída del **replay** (`monitor.storagePoints`) y **cortada en
el reloj** (`seriesFrom([name], monitor.storagePoints, clock)`), con el eje del
tiempo de 0 al fin de la corrida (`endTime={monitor.endTime}`, dominio
`[0, endTime]`) y la línea vertical del reloj
(`charts/TimeSeriesChart.tsx:68,107,111`). Encima, una línea punteada de
**capacidad** (propiedad del modelo, leída del reporte:
`charts/entities.ts:216`) y cuatro cifras del instante: *En uso ahora*,
*Máximo*, *Utilización media* (derivada, `area/(elapsed·capacity)`,
`monitor/derived.ts:109-113`) y *Reintentos* (`StorageSeries.tsx:88-115`).
Fuentes: `StorageSeries.tsx:33,77-86`; `entities.ts:207-231`;
`MonitorCharts.tsx:74,179-185`.

**1. ¿Tiene sentido.** Sí: es el único lugar donde se ve la *magnitud* de un
recurso de unidades y su distancia al límite; no hay otra serie de almacenes.
Pero hay solapamiento parcial: la "Utilización media de almacenes" de *Los
recursos que aprietan* es **el mismo número** que esta tarjeta (ambas salen de
`storagesOf` → `meanUtilization`, `ResourceRanking.tsx:73,96-104` vs
`entities.ts:217-219,229`). Y la app ya trae el dato sin usarlo:
`StorageNumbers.entries` se calcula (`entities.ts:226`) y esta tarjeta nunca lo
muestra.

**2. ¿Se muestra correctamente.** En general sí: el reloj manda (datos cortados
en el reloj, cifras vivas cerradas en el reloj), la serie es la historia exacta
(`monitor/replay.ts:1274-1281`, `monitor/series.ts:40-72`), la curva nunca
excede la capacidad porque el motor respeta `try_enter`
(`crates/genmsi-core/src/stat.rs:319-330`), y las claves de React son nombres
únicos (`StorageSeries.tsx:54`). Los hallazgos:

- [media] **"Reintentos" no es el número que publica el motor.** La app cuenta
  los `wait` de la traza en un bloque Enter (`monitor/replay.ts:660-666`), o sea
  intentos de ENTER que no entraron; el motor sólo incrementa `storage.retry`
  cuando un **GATE** bloquea por el almacén
  (`crates/genmsi-core/src/sim.rs:2449-2459`) y `try_enter` no lo toca al
  fallar (`crates/genmsi-core/src/stat.rs:319-330`). El reporte publica
  `retry` (`stat.rs:274`) y `delay` (`stat.rs:280,364-372`); la fidelidad
  compara `in_use/min_use/max_use/entries/area`, **no** `retry`
  (`monitor/replay.ts:1450-1458`), así que la divergencia pasa inadvertida.
  Mismo nombre, otro número. *Verificado en los tres puntos.*
- [media] **Un almacén que el replay todavía no tocó muestra ceros en gris**,
  es decir "publicado" (`live=false` → `text-slate-800`, `StatCard.tsx:26-32`),
  cuando el motor publicará otra cosa al final y el pie de la sección define
  gris = publicado (`MonitorCharts.tsx:238-240`); `entities.ts:230` marca
  `live: false`.
- [media] **El subtítulo afirma más de lo que la tarjeta muestra**: "cuando la
  curva toca el límite, el modelo está bloqueado" (`MonitorCharts.tsx:182`).
  Lleno no es bloqueado: el bloqueo es que haya alguien esperando, y eso el
  motor lo publica como `delay` (`stat.rs:280`), que la app ignora.
- [media] **La selección sobrevive a la corrida nueva**: `selected` no se
  reinicia (al revés que `QueueSeries.tsx:39-42`) y `MonitorCharts` se monta
  sin `key` (`MonitorView.tsx:592`); si el almacén elegido no existe en la
  corrida nueva, `storage` queda `null` y la tarjeta dice "El modelo no tiene
  almacenes" (`StorageSeries.tsx:30,37-44`) aunque la corrida nueva tenga
  almacenes.
- [baja] **Sin pie de tarjeta**: la fórmula aparece sólo en la nota de una
  cifra ("derivada: área / tiempo / capacidad", `StorageSeries.tsx:107`) y no
  aclara que el tiempo es el del reloj, no el de la corrida.
- [baja] **Badge incompleto**: dice "replay" (`MonitorCharts.tsx:183`) pero la
  capacidad viene del reporte (`entities.ts:213-216`).
- [baja] **La línea de capacidad queda pegada al borde superior** porque el
  dominio Y es `[0, max(capacidad,1)]` (`StorageSeries.tsx:85`): "tocar el
  límite" es indistinguible de "tocar el tope del gráfico".
- [baja] Colores escritos a mano fuera de `palette.ts`
  (`StorageSeries.tsx:77,82`); el `#0d9488` es también el color de la segunda
  serie de `SERIES_COLORS` (`palette.ts:12`).
- [baja] Rama "El modelo no tiene almacenes" muerta: la tarjeta sólo se monta
  si `storages.length > 0` (`MonitorCharts.tsx:179`).
- [baja] Cobertura: la lógica sí está probada (`tests/entities.test.ts:117-138`,
  `tests/derived.test.ts:63-73`, `tests/replay.test.ts:213-236`,
  `tests/stats.test.ts:117-137`), pero **ningún test renderiza el componente**:
  no hay cobertura de yDomain, referencia, colores ni del texto "El modelo no
  tiene almacenes".

**3. ¿Otra forma.** Una banda de ocupación por franjas (como
`FacilityTimeline`) con la capacidad como fondo y el color bajo el límite
cuando hay esperadores, más el contador de *entradas* y *esperando ahora*
leídos del reporte (`entries`, `delay`): muestra lo mismo —nivel, límite,
bloqueos— y hace visible el "bloqueado" que hoy se afirma sin mostrarse, sin
ambigüedad del eje superior.

**Veredicto.** KEEP WITH CHANGES
**Riesgo de equivocarse.** Que un lector equipare "Reintentos" con la columna
`RETRY` del reporte (o con `delay`), o que lea los ceros grises de un almacén
intacto como su valor final; y que tome "lleno" por "bloqueado".

### Las tablas del modelo

**Qué muestra.** Una fila por tabla del modelo (`TABLE` y `QTABLE`), con cinco
columnas: Muestras, Media, Desviación, Desbordes y Máximo
(`charts/TablesList.tsx:68-101`). Las cifras salen **de la traza**:
`live?.tables[table.name]` del replay (`TablesList.tsx:43-49`), y
Media/Desviación se derivan con `tableDerived` (`TablesList.tsx:49`,
`monitor/derived.ts:165-176`). La lista de tablas y su orden salen del reporte
(`TablesList.tsx:60-65`); el motor publica además `first/width/intervals`,
`sum`, `sum_squares`, `max_value`, `overflow`, `overflow_sum`,
`overflow_sum_squares` y `frequencies`
(`types/generated/Table.ts:9-45`, `crates/genmsi-core/src/stat.rs:463-495`).

**1. ¿Tiene sentido?** Sí, la información es pertinente para una simulación por
eventos discretos: el conteo de desbordes dice si el histograma declarado por el
modelo es demasiado angosto, y la media y el máximo dicen qué se tabuló. Pero
la tarjeta es redundante en dos frentes: repite las cuatro cifras que el
histograma ya pone en su pie para la tabla seleccionada
(`charts/TableHistogram.tsx:109-113`) y repite muestras/máximo/desbordes de la
pestaña "Tablas" del reporte (`components/ReportView.tsx:376-392`); además no
aporta lo que hace falta para interpretar los números: la definición de la
tabla (primer valor, ancho, intervalos).

**2. ¿Se muestra correctamente.**

- [alta] El rótulo miente sobre el origen: badge "reporte del motor" y
  subtítulo "con lo que el motor publica de ella" (`MonitorCharts.tsx:211-213`),
  pero **ningún** número sale del reporte: todos salen de las figuras vivas del
  replay (`TablesList.tsx:43-49`). El propio docstring del componente afirma
  lo contrario de lo que hace ("The row says what the engine published of the
  table", `TablesList.tsx:4`). *Verificado en el propio componente.*
- [alta] El color no codifica lo que el pie del panel dice: "Las cifras en
  violeta son las del instante… las demás son las que el motor publicó"
  (`MonitorCharts.tsx:239-240`), pero en `StatTable` la columna derivada pinta
  violeta siempre (`charts/StatTable.tsx:62`) y la viva también (`:63`): en
  *Media* y *Desviación* el violeta significa "derivada", no "del instante".
  Peor: una tabla que el replay aún no muestreó tiene `live=false`
  (`TablesList.tsx:57`) y sus ceros salen en gris, color que el pie reserva
  para "publicado por el motor".
- [media] Asterisco sin pie: `StatTable` pone el "*" (`StatTable.tsx:46`) y su
  docstring promete que "the footnote of the card says what the derivation is"
  (`StatTable.tsx:3-5`), pero la tarjeta no pasa `footnote`
  (`MonitorCharts.tsx:209-216`; el pie es opcional, `ChartCard.tsx:24,47-51`).
  El lector nunca ve la fórmula (media = suma/muestras;
  desviación = √(E[x²]−E[x]²)), sí el "Recursos que aprietan"
  (`MonitorCharts.tsx:126`).
- [media] Sin estado vacío ni tipo: una tabla que el modelo nunca tabuló
  aparece como una fila de ceros (no hay "Sin datos": `StatTable.tsx:29-31`
  sólo se dispara con cero filas), y `TABLE` y `QTABLE` se mezclan sin columna
  de tipo (`TablesList.tsx:60-65`) aunque el motor los separa
  (`crates/genmsi-core/src/report.rs:255-290`); una qtable tabula esperas de
  cola, no un valor del modelo, y dos homónimos quedarían indistinguibles.
- [media] Fidelidad parcial: el chequeo de fidelidad compara `samples`,
  `maxValue` y `frequencies` con el reporte (`monitor/replay.ts:1483-1512`),
  pero no `sum`, `sum_squares` ni `overflow`, que son los que alimentan Media,
  Desviación y Desbordes (aunque salen de la misma aritmética:
  `crates/genmsi-core/src/stat.rs:517-535` vs `monitor/replay.ts:719-735` y
  `monitor/stats.ts:394-412`).
- [baja] La app deriva `overflowRate` (`monitor/derived.ts:154,174`) y no la
  muestra en ninguna tarjeta: la tasa de desbordes, la señal útil de
  "histograma angosto", se queda sin usar.
- [baja] El orden de las filas es el del diccionario del reporte (alfabético,
  por el `BTreeMap` del motor), no el del programa, que es como el reporte las
  escribe (`report.rs:255-262`).
- [baja] Precisión inconsistente: Media y Desviación con 1 decimal
  (`TablesList.tsx:81,87` vía `format.ts:26-33`), contra 2 decimales en el
  histograma (`TableHistogram.tsx:110-111`) y 3 en el reporte del motor
  (`report.rs:32-38`).

**Pruebas.** No hay prueba de componente: `npm test` corre
`node --test tests/*.test.ts` (`crates/genmsi-app/package.json:13`) y ningún
archivo menciona `TablesList` ni `StatTable`. La cobertura es indirecta y de
las capas de abajo: `tests/derived.test.ts:80-100` (las fórmulas),
`tests/stats.test.ts:176-203` (la acumulación de la tabla viva) y
`tests/replay.test.ts:362-393` (las muestras salen de la traza). Nadie prueba
el mapeo reporte→filas (tabla sin muestras, qtable en la lista).

**3. ¿Otra forma.** Unir lista e histograma en una sola tarjeta: el selector de
tabla (que ya existe en `TableHistogram.tsx:90-105`) elige la fila, y cada fila
añade lo que el histograma no dice —definición (primer valor, ancho,
intervalos), TABLE/QTABLE y tasa de desbordes— mientras la media, la
desviación, las muestras y el máximo salen **una sola vez**. Alternativa
mínima si se mantiene la lista: leer el publicado con el patrón `cellAt` de
`ReportView.tsx:47-53` (vivo en violeta, publicado en gris) y anotar el pie con
las fórmulas.

**Veredicto.** KEEP WITH CHANGES
**Riesgo de equivocarse.** Con el reloj en 0 (o con una tabla todavía no
tabularada), la tarjeta rotulada "reporte del motor" muestra "0 muestras, 0
media, 0 desbordes, máximo 0" en el color que el pie reserva para lo publicado:
el lector concluirá que el modelo no tabula nada, cuando el motor publicó
cientos de muestras. Y con el violeta significando dos cosas a la vez, no se
puede saber si una cifra es del instante o derivada.

### Transacciones por bloque

**Qué muestra.** Ranking de los bloques del modelo por número de transacciones
que pasaron por ellos. La cifra sale de los totales acumulados por el replay
hasta el reloj (`charts/blocks.ts:66-69` → `monitor/counts.ts:43-56` →
`state.totals[line-1]`), tomados **sólo de la fila que cuenta el bloque**
(`counts_block`, `monitor/counts.ts:50`; el motor lo publica en
`crates/genmsi-core/src/report.rs:1404,1455`). Se pinta con
`charts/BarList.tsx` (nombre + número + barra) y se ensambla en
`MonitorCharts.tsx:219-226` con badge "reporte del motor" y el pie que explica
la regla de "la fila que cuenta el bloque".

**1. ¿Tiene sentido.** Sí en el fondo: es la única vista que ordena por
volumen de trabajo y muestra dónde se acumula. Pero hoy el ranking está
dominado por una entrada que no aporta información —el bloque generador— y el
resto de la información ya está en el panel de reporte
(`components/ReportView.tsx:206-215`, columna `Entry`) y en las figuras del
lienzo. No es redundante en el formato (un ranking ordena; una tabla no
ordena), pero sí en el contenido.

**2. ¿Se muestra correctamente.** Parcialmente: la aritmética de "no contar
cinco veces" es correcta, pero el origen y el momento de las cifras están mal
declarados.

- [alta] **El badge miente sobre la fuente y las cifras no están marcadas como
  vivas.** El badge dice "reporte del motor" (`MonitorCharts.tsx:222`), pero la
  cifra viene del replay (viva al reloj): `charts/blocks.ts:66-69` y `:92-93`.
  La convención de la app es explícita —"el número del instante va en indigo,
  el publicado en slate" (`charts/StatCard.tsx:5-9,26-30`,
  `StatTable.tsx:44-45`)— y aquí el número se escribe en `text-slate-800`
  (`BarList.tsx:41`, color de publicado) y sólo la barra es violeta
  (`BlockRanking.tsx:32`). El lector ve una cifra gris de publicada y, por el
  pie de la sección ("Las cifras en violeta son las del instante…",
  `MonitorCharts.tsx:236-240`), puede leer al revés.
- [alta] **La cifra publicada del motor se ignora.** `JsonBlock.entries`
  (`types/generated/JsonBlock.ts`, `report.rs:1462`) no se lee nunca para el
  ranking: `blocks.ts:93` usa sólo el total del replay. La app podría mostrar
  "hasta el reloj / publicado" (el motor lo publica) y no lo hace.
- [media] **Los contenedores no existen en la tarjeta.** Un contenedor no tiene
  fila de reporte (`monitor/counts.ts:16-18`) y `blocks.ts:83-88` recorre sólo
  `report.blocks`, así que ningún contenedor aparece, aunque el motor ya da lo
  necesario (`crossingsEvents`/`crossingsUntil`, `monitor/counts.ts:194,267`) y
  el lienzo sí los cuenta (`MonitorView.tsx:132-139`).
- [media] **El bloque generador recibe una figura que no es "transacciones que
  pasaron".** El propio módulo del app dice que "un generador no tiene figura"
  (`monitor/counts.ts:11-13`) y `countsOf` acepta un conjunto `generators` para
  excluirlo (`counts.ts:46,50`) que la app nunca calcula (`charts/blocks.ts:69`,
  igual `MonitorView.tsx:370-374`). El replay cuenta una entrada por
  transacción creada (`monitor/replay.ts:760-764`), de modo que el generador
  entra al ranking con el total de transacciones creadas —el número más alto—
  bajo el rótulo "transacciones que pasaron por cada bloque"
  (`MonitorCharts.tsx:221`). **No verificado contra el reporte**:
  `checkFidelity` salta las filas `Generate` (`monitor/replay.ts:1382-1388`).
  *Verificado en el código*: `blocks.ts:66-69` llama `countsOf` sin el
  conjunto de generadores.
- [media] **Dos bloques con el mismo nombre son indistinguibles.** El rótulo
  sale del *caption* del modelo (`blocks.ts:38-45`), sin clase ni id; la clase
  ya se calcula (`blocks.ts:97`, `BlockRow.class`) y no se usa. La clave de
  React es nombre+índice (`BarList.tsx:38`), así que la app no se rompe
  (`odd/tasks/genmsi-graficos-estadisticas.md:137-139`), pero el lector ve dos
  barras idénticas y no sabe cuál es cuál. El motor publica la etiqueta GPSS,
  única por bloque (`report.rs:1450-1454`, `JsonBlock.label`), y no se usa.
- [baja] La barra es una parte de la más larga, no del total
  (`BlockRanking.tsx:25`, `BarList.tsx:4-5`), y el pie no dice la fórmula:
  regla 2 incumplida ("sólo ratios del motor… con asterisco y la tarjeta dice
  la fórmula").
- [baja] Estado vacío confuso: "Ningún bloque tuvo transacciones."
  (`BlockRanking.tsx:33`) aparece también al comienzo de la reproducción,
  cuando todavía no pasó nada (todos los totales en 0). No verificado
  ejecutando la app.
- [baja] Código muerto en el helper: `counting` se arma y no se usa
  (`blocks.ts:73-81`); `current` y `class` de `BlockRow` no los usa ninguna
  tarjeta.

**Cobertura de tests.** La tarjeta **no está cubierta**: no hay test de
`charts/blocks.ts` ni de `BlockRanking` (búsqueda en `crates/genmsi-app/tests/`).
Lo que sí está probado es la regla que la sostiene:
`tests/counts.test.ts:44-54` (un facility use cuenta 59, no 295), `:56-71` (el
generador), `:73-77` (un reporte sin `counts_block` deja el bloque sin figura)
y `tests/replay.test.ts:499-513` (los totales del replay igualan las `entries`
publicadas al final de la corrida).

**Lo que el motor publica y la app no usa:** `entries` y `current` publicados
por fila, `label` (GPSS) y `operands` (`report.rs:1456-1466`).

**3. ¿Otra forma.** Mantener el ranking pero con una fila por bloque que diga
*caption + clase* (o etiqueta GPSS) y dos columnas: "hasta el reloj" en indigo
y "publicado" en gris, más un rótulo propio para el generador ("transacciones
creadas"). Justificación: una línea por bloque con la clase ya calculada
resuelve de paso los nombres repetidos, y las dos columnas respetan la regla
del reloj sin ambigüedad de color; el ranking se ordena por la cifra viva, que
es la que el lector está viendo.

**Veredicto.** KEEP WITH CHANGES
**Riesgo de equivocarse.** Un lector que tome las barras como cifras del
reporte, y (cuando el reloj está al final) como definitivas, atribuirá al
modelo un volumen de trabajo que el motor no publica, y creerá que el bloque
más cargado es donde se acumula el trabajo cuando puede ser el generador.

### Histograma de la tabla

**Qué muestra.** El histograma de **una** tabla o qtable del modelo, elegible
con botones (`charts/TableHistogram.tsx:47-60, 90-107`): las barras son las
frecuencias de los intervalos, la curva naranja es el acumulado en %, la banda
violeta es media ± desviación (`TableHistogram.tsx:139-142, 160-162, 166-171`).
Los datos salen de las **figuras vivas del replay**
(`TableHistogram.tsx:68-74` → `monitor/stats.ts:107-110, 394-412`), y la
geometría de los intervalos sale del reporte (`first`, `width`, `intervals`:
`crates/genmsi-core/src/stat.rs:463-495`,
`crates/genmsi-app/src/types/generated/Table.ts:12-22`). La media y la
desviación las calcula la app con `tableDerived`
(`monitor/derived.ts:160-176`), con la misma fórmula del motor
(`crates/genmsi-core/src/stat.rs:558-573`).

**1. ¿Tiene sentido?** Sí. La forma de la distribución de una magnitud tabulada
(tiempos de servicio, esperas) es información que las tablas de indicadores no
dan; no es redundante con *Las tablas del modelo*, que sólo publica escalares
(`charts/TablesList.tsx:60-100`). Sí repite, en cambio, media, desviación y
desbordes de esa tabla (`TableHistogram.tsx:110-112` vs `TablesList.tsx:74-90`).

**2. ¿Se muestra correctamente.** El cálculo es correcto (frecuencias, clase de
desborde y fórmulas coinciden con el motor: `monitor/replay.ts:721-736`,
`monitor/stats.ts:394-412`, `crates/genmsi-core/src/stat.rs:517-535`), pero las
etiquetas y el rotulado no.

- [alta] El badge dice "reporte del motor" y el propio comentario del
  componente dice que las barras son "the **published frequencies**"
  (`MonitorCharts.tsx:203`, `TableHistogram.tsx:4-6`), pero las barras son las
  cifras **del instante** (`TableHistogram.tsx:68-69`); contradice la regla "el
  reloj manda" y la leyenda de la sección ("Las cifras en violeta son las del
  instante… las demás son las que el motor publicó", `MonitorCharts.tsx:239`).
  El lector compara un histograma parcial con el reporte de la corrida
  completa y cree que difieren.
- [alta] **La agrupación de las tablas con más de 200 intervalos ya no
  existe**, aunque es una decisión del usuario (S9: "Agrupar con aviso
  visible", `odd/tasks/genmsi-fase4c-monitor.md:37-39, 98, 173`) y la
  documentación la promete (`docs/app.md:377-378`). No hay código de agrupación
  en `crates/genmsi-app/src`: se dibuja una barra por intervalo
  (`TableHistogram.tsx:75-76`) con `barSize = max(4, round(620/n)-8)`, o sea
  4 px por barra con eje numérico (`TableHistogram.tsx:118-124, 149`): con 400
  clases las barras se pisan y el histograma es un muro. Ningún modelo del repo
  llega a 200 intervalos (`docs/BLOCK-INVENTORY.md:340, 442`), así que el caso
  está sin ejercitar. *Verificado: la búsqueda de "agrup"/"cubre cada" en
  `crates/genmsi-app/src` no encuentra la agrupación.*
- [media] La **media y la desviación no están marcadas como derivadas** (sin
  asterisco ni fórmula: `TableHistogram.tsx:110-112`), al revés que la tabla
  gemela, que sí marca `derived: true` (`TablesList.tsx:74-79` →
  `StatTable.tsx:5-6, 46`) y que el pie de la tarjeta explica
  (`MonitorCharts.tsx:149-151`). El motor publica MEDIA y DESVIACION ESTANDAR
  en el reporte de texto (`crates/genmsi-core/src/report.rs:456-462`); el JSON
  no las trae, así que la app las recalcula y debe decirlo.
- [media] La **primera clase está mal rotulada** cuando `first > 0`: la app
  escribe "0 – first" (`monitor/series.ts:216-217`, fijado por
  `tests/series.test.ts:138`), pero la clase 0 del motor es "todo valor ≤
  `first`" (`crates/genmsi-core/src/stat.rs:522-524`) y el reporte la imprime
  como "first – first" (`crates/genmsi-core/src/report.rs:486-491`). La barra
  se ancla en `x = first/2` (`series.ts:220`), que sí cae dentro de la clase 0,
  pero el rótulo promete un下限 que la clase no tiene.
- [baja] La clase de desborde: la app la detecta por ser la última fila
  (`monitor/series.ts:219`) y la pinta de naranja
  (`TableHistogram.tsx:153-156`) ✓, pero el número de "desbordes"
  (`TableHistogram.tsx:73`) es **la misma cifra** que la barra naranja
  (`monitor/stats.ts:405-410`); y no se dice que esa clase no tiene límite
  superior, como sí lo dice el reporte ("> a 80", `report.rs:477-492`).
- [baja] Estado vacío no contemplado: antes del primer valor, `frequencies` es
  `[]` (`TableHistogram.tsx:69`) y se dibuja un gráfico vacío con "0 valores ·
  media 0.00 · desviación 0.00" (`TableHistogram.tsx:80-86, 110-112`); el
  tooltip muestra el punto medio de la clase como "valor X"
  (`TableHistogram.tsx:137`) y descarta el rango que la fila ya trae
  (`monitor/series.ts:221`).
- [baja] La fidelidad **no verifica** de dónde salen la media, la σ y los
  desbordes: el control compara `samples`, `max_value` y `frequencies`, pero no
  `sum`, `sum_squares` ni `overflow`
  (`monitor/replay.ts:1491-1494, 1507-1510`).

**Campos del motor que la app ignora:** `overflow_sum` y `overflow_sum_squares`
(`types/generated/Table.ts:53, 57`), que el reporte publica como MEDIA OVERFLOW
/ DESVIACION ESTANDAR OVERFLOW (`crates/genmsi-core/src/report.rs:525-540`); y
el atajo del propio motor para cortar la cola vacía ("Las frecuencias
restantes (N) son todas cero", `report.rs:512-518`).

**3. ¿Otra forma.** Dos ajustes, sin perder información: (a) aplicar la regla
del reporte al histograma —recortar las clases vacías del final y, si son
muchas, agrupar con el aviso "cada barra cubre N intervalos" (S9)—, y (b) usar
el `label` de la fila en el tooltip (`monitor/series.ts:221`) y marcar media/σ
con asterisco y pie "media = suma/muestras · σ = √(suma²/n − media²)". Si
además se fundiera el histograma dentro de *Las tablas del modelo* (una fila
selecciona la tabla), se eliminaría la duplicación de media/σ/desbordes y los
botones.

**Veredicto.** KEEP WITH CHANGES
**Riesgo de equivocarse.** El badge hace pasar cifras del instante por
publicadas: el lector cree que el histograma del reloj es el de la corrida y
desconfía del reporte (o al revés). Y con una tabla de muchas clases, las
barras superpuestas dan una forma que el modelo no tiene.

### Reintentos por bloque — la contención

**Qué muestra.** Ranking de los bloques del modelo (hasta 8,
`charts/RetryRanking.tsx:13`) ordenados por reintentos, con barra proporcional
al mayor (`RetryRanking.tsx:21-29`) y el número en `BarList.tsx:41`. La fuente
son las filas del reporte del motor (`JsonReport.blocks[].retries`,
`crates/genmsi-core/src/report.rs:1392-1394`, `:1464`), agregadas por nodo de
bloque —el bloque suma las filas que lo componen—
(`charts/blocks.ts:74-82`, `:100`). El ensamblado está en
`MonitorCharts.tsx:228-234`.

**1. ¿Tiene sentido?** Sí: es la única tarjeta que señala *dónde* se bloquea el
modelo, y no es redundante con *Transacciones por bloque* (mismo `rows`, pero
entradas) ni con *Indicadores por facility* (entidades, no bloques). El
problema es que el número que muestra no es el que dice ser.

**2. ¿Se muestra correctamente.** Parcialmente: la agregación por bloque es
correcta (no cuenta cinco veces una transacción, al revés que las entradas:
`blocks.ts:71-82`), pero el origen, la semántica y la comparación fallan.

- [alta] **Muestra cifras del reloj con el badge de cifras publicadas.** En el
  panel de gráficos `state` nunca es `null` (se anula sólo si `monitor === null`:
  `MonitorView.tsx:94-97`, y el panel exige `monitor !== null`:
  `MonitorView.tsx:586-592`), así que `blocks.ts:78` siempre toma la rama viva
  (`state.retries[line]`), que viene del replay
  (`monitor/replay.ts:783-786`, `:867-869`), nunca del reporte. El badge dice
  "reporte del motor" (`MonitorCharts.tsx:231`) y el número no va en violeta,
  que es como la sección marca las cifras del instante
  (`MonitorCharts.tsx:239-241`; `BarList.tsx:41` usa slate). Contradice la
  regla 1 y la convención de la sección (`charts/entities.ts:4-13`).
- [alta] **Las dos fuentes no dicen lo mismo, y el motor no cuenta lo que el
  subtítulo promete.** El único incremento de `block_stats[..].retries` es el
  del arrebato rechazado por prioridad (`crates/genmsi-core/src/sim.rs:2730`,
  llamado en `sim.rs:1379-1383`); la fila SEIZE con facility ocupada no cuenta
  reintento (`sim.rs:1327-1339`), la ENTER sin unidades tampoco
  (`sim.rs:1466-1484`) y el GATE cuenta el intento en la entidad, no en la fila
  (`sim.rs:1643-1648`, `:2449-2464`; confirmado en
  `crates/genmsi-core/tests/logic_switches.rs:69-73` y `storages.rs:191-195`).
  El replay, en cambio, cuenta *toda* espera (`monitor/replay.ts:783-786`). El
  comentario de `replay.ts:784-785` ("el motor lo cuenta como reintento del
  bloque") es falso salvo en el caso del preempt.
  *Verificado*: `retries +=` aparece una sola vez en `sim.rs` (`:2730`, en el
  camino del preempt rechazado); el replay cuenta cualquier `wait`.
- [media] **El ranking no es comparable entre bloques.** Son acumulados desde
  el arranque (`monitor/replay.ts:210-211`), sin denominador, aunque la fila ya
  trae `entries` (`blocks.ts:26`, `:98`): un bloque con más tráfico acumula más
  rechazos sin ser más contendido. Y el mismo conflicto aparece en sitios
  distintos según la clase (una puerta cuenta en la entidad, un facility en la
  fila).
- [media] **Falta el pie.** La regla no obvia —"el bloque suma los reintentos
  de sus filas"— no está escrita, aunque la tarjeta vecina sí explica su regla
  (`MonitorCharts.tsx:223`).
- [media] **La traza puede estar truncada** (`TRACE_LIMIT = 200_000`,
  `src/state/store.ts:66`; aviso global en `MonitorView.tsx:438-442`): el
  ranking del instante puede estar incompleto y la tarjeta no lo dice.
- [baja] Badge ámbar —el color de "derivado" (`MonitorCharts.tsx:125`, `:144`)—
  con texto "reporte del motor"; el color por defecto del badge es indigo
  (`ChartCard.tsx:41-43`).
- [baja] Estado vacío y cola (`RetryRanking.tsx:36`, `:40`) hablan del modelo
  entero ("nada del modelo se rechazó") cuando el dato es del instante.
- [baja] Código muerto: `counting` (`blocks.ts:73`, `:79-81`) nunca se usa;
  `class` (`blocks.ts:97`) no lo usa esta tarjeta; dos bloques con el mismo
  nombre son indistinguibles (`blocks.ts:36-46`).

**Tests.** Sin cobertura: ningún test importa `blockRows` ni `RetryRanking`
(`crates/genmsi-app/tests/`), `tests/counts.test.ts` cubre `countsOf`
(entradas), no la agregación de reintentos, y `tests/replay.test.ts` construye
eventos `wait` pero no afirma `state.retries`. La fidelidad
(`monitor/replay.ts:1379-1399`) compara `current` y `entries` de bloque, nunca
los reintentos —justo la familia que difiere del reporte.

**Lo que el motor publica y la app ignora:** `blocks[].retries` del reporte
sólo se usa en la rama muerta (`blocks.ts:77`); `storage.delay`/`storage.retry`
—el "no había unidades" que promete el subtítulo—
(`src/types/generated/Storage.ts:36-47`) y el `retry` de colas y switches
lógicos (`crates/genmsi-core/src/sim.rs:820-824`).

**3. ¿Otra forma.** Mantener la tarjeta pero: (a) badge "replay" y cifras en
violeta, con pie que diga "reintentos acumulados hasta el reloj"; (b) barra =
reintentos/entradas del bloque (ratio de cifras del motor, marcado con `*` y
la fórmula), que sí es comparable entre clases; o, si se quiere leer sólo al
motor, ranking de `blocks[].retries` publicado —que casi sólo tiene filas de
preempt— y eso no alcanza para "la contención".

**Veredicto.** KEEP WITH CHANGES
**Riesgo de equivocarse.** Alto: el lector ve "reporte del motor" sobre cifras
del instante, y una contención de facility/almacén que el motor nunca cuenta
como reintento aparece en la tarjeta como si el motor la hubiera medido; además
el ranking compara absolutos sin tráfico y puede señalar el bloque más
transitado como el cuello de botella.

### La corrida en números

**Qué muestra.** Seis tarjetas (`StatCard`) con las cifras globales de la
corrida: **Transacciones** (en vivo, `generateTimes` contados hasta el reloj; en
publicado, `report.transactions`), **Eventos** (`report.events`), **Reloj**
(reloj del replay en vivo / `end_time - start_time` en publicado), **Fin**
(`end_reason` traducido), **Caudal** (ratio transacciones/segundo) y
**Población** (suma de `state.blocks` en vivo / suma de `block.current` en
publicado) — `charts/RunSummary.tsx:66-107`, armado en
`MonitorCharts.tsx:106-119` con `generated={countAt(monitor.generateTimes, clock)}`
(`MonitorCharts.tsx:117`). Fuentes del motor: `transactions` = `next_xact`
(`crates/genmsi-core/src/sim.rs:863`), `events` = contador de eventos del motor
(`sim.rs:757`), publicado en `crates/genmsi-core/src/report.rs:1469-1480` /
`crates/genmsi-app/src/types/generated/JsonReport.ts:17-43`. Las cifras vivas
van en violeta y las publicadas en pizarra (`charts/StatCard.tsx:33-35`), y el
pie de la sección lo explica (`MonitorCharts.tsx:234-239`).

**1. ¿Tiene sentido.** Sí: es la orientación mínima de la corrida (cuánto pasó,
cuánto duró, cómo terminó, cuánto quedó vivo) y no repite ninguna otra
tarjeta de la sección. Es parcialmente redundante con otras pantallas: "Fin",
"Transacciones" y "Eventos" ya están en la cabecera del panel Reporte
(`components/ReportView.tsx:167-175`) y el reloj ya está en la consola del
replay (`components/RunConsole.tsx:215`).

**2. ¿Se muestra correctamente.** Hay tres desvíos y dos detalles de forma.

- [media] **"Transacciones" mide dos cosas distintas según el modo.** La cifra
  viva cuenta eventos `generate` de la traza
  (`monitor/replay.ts:753-765`), o sea solo nacimientos de generadores; la
  publicada es `next_xact`, el total de identificadores de transacción creados,
  que **incluye las copias de `SPLIT`** (`crates/genmsi-core/src/sim.rs:1071-1073`,
  `:1817-1819`), y las copias no dejan evento `generate` (sólo `enter`). En un
  modelo con `SPLIT` (p. ej. `fixtures/outputs/TP5__burgerShop.report.txt:14`)
  el panel de Reporte y esta tarjeta dan dos totales distintos para la misma
  corrida, con la misma etiqueta. **No verificado en ejecución: deducido del
  código.**
- [media] **"Población" publicada y viva no coinciden ni al final de la
  corrida.** La publicada suma `block.current` de todas las filas, y el motor
  da `current = 1` al generador mientras tenga una llegada pendiente
  (`crates/genmsi-core/src/sim.rs:625`, y sólo lo pone en 0 al agotar su
  límite, `sim.rs:993`, `:1037`; evidencia en los reportes:
  `fixtures/outputs/Problema_1__semaforo-1.report.txt:17,19,47` frente a `:8`).
  El replay, en cambio, nunca cuenta el generador como bloque con transacción
  (`monitor/replay.ts:753-765`) y su propia verificación de fidelidad lo
  excluye explícitamente (`monitor/replay.ts:1383-1388`). La población viva
  queda así un número menor por cada generador vivo.
- [media] **La cabecera de la sección miente sobre dos de las seis tarjetas.**
  Dice "Los números del instante, con el reloj de la reproducción"
  (`MonitorCharts.tsx:108-111`), pero **Eventos** y **Fin** son siempre cifras
  publicadas (`RunSummary.tsx:72-87`) y **Fin** es el final de una corrida que
  todavía no terminó en el reloj.
- [media] **"Caudal" es una métrica derivada sin marcar.** Es un ratio
  calculado por la app (`RunSummary.tsx:88-101`), pero no lleva asterisco ni
  dice la fórmula, como sí hacen las demás de la sección
  (`MonitorCharts.tsx:127` footnote de "Utilización = …",
  `charts/QueueTable.tsx:36-56`). La regla 2 del contrato pide exactamente esa
  marca.
- [media] **La rama "publicada" de la tarjeta es código muerto.** `state` es
  `null` sólo cuando `monitor` es `null` (`MonitorView.tsx:94-97`) y
  `MonitorCharts` sólo se monta con `monitor !== null`
  (`MonitorView.tsx:581-592`), luego `live` es siempre `true` en la app: las
  cinco variantes publicadas (incluida `publishedPopulation`,
  `RunSummary.tsx:37-42`) nunca se ven. El mismo contrato se respeta bien en el
  reparto del tiempo: el caudal vivo divide por `clock - start_time` y el
  publicado por `end_time - start_time` (`RunSummary.tsx:91-97`,
  `monitor/entities.ts:28-42`).
- [baja] La tarjeta no tiene badge ni pie propios (no es un `ChartCard`), así
  que no dice de dónde sale cada número ni que dos de ellos son publicados.
- [baja] `RunSummary.tsx:16-22` redefine un `decimals` propio con 2 decimales,
  duplicando `format.ts:29-34` (que usa 1 decimal y por defecto
  `places = 1`).
- [baja] "Reloj" en vivo muestra el reloj del replay con fracciones
  (`format.ts:12-21` escribe 3 decimales con coma), mientras la consola lo
  muestra como `h:mm:ss`: dos lecturas del mismo número.

**Cobertura de tests.** La tarjeta **no está cubierta**: no hay ningún test que
la importe (`crates/genmsi-app/tests/` no la menciona). Sí están cubiertos sus
insumos: `countAt` (`tests/series.test.ts:111-113`), `generateTimes`
(`tests/replay.test.ts:200-211`) y `entities.ts` (`tests/entities.test.ts`). Las
dos funciones de población son privadas dentro del `.tsx`, así que ni siquiera
son alcanzables desde un test.

**Lo que el motor publica y la sección ignora.** `report.diagnostics`
(`crates/genmsi-core/src/report.rs:1434`, `JsonReport.ts:48`) lleva el detalle
del corte por el guard de eventos con los bloques más calientes
(`sim.rs:945-973`); el panel Reporte tampoco lo muestra, y en una corrida
cortada por `event_limit` el "Fin" de esta tarjeta
(`engine/reasons.ts:89-91`) no dice ni dónde se cortó.

**3. ¿Otra forma.** Mantener las seis cifras pero en una sola tarjeta por
métrica con **el número vivo y, en la nota, el total publicado** ("1.234 de
2.345 en la corrida"), de modo que las dos cifras sean comparables por
construcción —en vez de dos tarjetas que no se pueden contrastar—; y añadir a
"Caudal" el asterisco con su fórmula, como el resto de la sección. Añadir una
línea "generadores pendientes" o quitar el `current` del generador de la
población publicada.

**Veredicto.** KEEP WITH CHANGES
**Riesgo de equivocarse.** Que el lector tome la tarjeta por el resumen de la
corrida y la compare con el panel Reporte: verá dos "Transacciones" distintas
(nacimientos vs. identificadores creados) y dos "Poblaciones" distintas (con y
sin el generador), y creerá que una de las dos está mal.

## Las piezas compartidas

La familia sobre la que se construye cada tarjeta: `ChartCard.tsx`
(envoltorio), `StatTable.tsx`/`StatCard.tsx`/`BarList.tsx` (cifras),
`TimeSeriesChart.tsx`/`EntityPicker.tsx`/`palette.ts` (dibujo),
`blocks.ts`/`entities.ts` (adaptadores de datos) y el motor
`monitor/series.ts`, `derived.ts`, `stats.ts`, `live.ts`.

| Pieza | Veredicto |
|---|---|
| `ChartCard.tsx` | KEEP WITH CHANGES (el badge es texto libre: 5 significados distintos) |
| `StatTable.tsx` | KEEP WITH CHANGES (el violeta significa dos cosas) |
| `StatCard.tsx` | KEEP WITH CHANGES (mismo violeta, otro significado) |
| `BarList.tsx` | KEEP WITH CHANGES (sin eje de instante; barra falsa en 0) |
| `TimeSeriesChart.tsx` | KEEP WITH CHANGES (eje Y libre, sin estado vacío) |
| `EntityPicker.tsx` | KEEP (pero huérfano: 4 tarjetas reescriben el mismo botón) |
| `palette.ts` | KEEP WITH CHANGES (no es la paleta de la familia) |
| `blocks.ts` | KEEP WITH CHANGES (calcula `current`, `class` y `live` que nadie muestra) |
| `entities.ts` | KEEP WITH CHANGES (recibe reloj y cifras por separado) |
| `monitor/series.ts` | KEEP (dos convenciones de intervalo en el mismo módulo) |
| `monitor/derived.ts` | KEEP (correcto y puro) |
| `monitor/stats.ts` | KEEP (la aritmética del motor, correcta) |
| `monitor/live.ts` | KEEP WITH CHANGES (publica lo que las tarjetas no leen) |

**Hallazgos de la familia**

- [alta] — **La familia no ata el reloj a sus cifras.** `entities.ts` recibe
  `live: Figures` y `elapsed.live` por separado
  (`charts/entities.ts:154`, `:161`, `:189`, `:219`, `:243`), y el panel los
  toma de fuentes distintas: el reloj del store vivo (`MonitorCharts.tsx:57`) y
  las figuras del prop `state` (`MonitorCharts.tsx:63-66`). `state` sólo se
  recalcula cuando cambia el playhead (`MonitorView.tsx:94-107`) y durante la
  reproducción el playhead no se actualiza hasta el final
  (`MonitorView.tsx:385`). El store sí publica las figuras frescas
  (`monitor/live.ts:100-104`, `useFigures` en `live.ts:204`) y **ninguna
  tarjeta las usa** (`useFigures` no tiene un solo uso en la app). Consecuencia:
  un área congelada dividida por un reloj que crece.
  *Verificado y precisado por el padre*: `closeFigures`
  (`monitor/stats.ts:434-464`) cierra el área con el contenido **congelado** del
  frame del playhead y el reloj que avanza, de modo que (a) la utilización de
  una facility que estaba ocupada en ese frame tiende a **100 %**
  (`busyTime = busyTime0 + (clock − seizedAt)`, dividido por `clock`), y (b) el
  contenido medio de una cola vacía en ese frame decae hacia 0. Las **series**
  temporales no sufferen el problema: salen de `monitor.*Points` y se cortan en
  el reloj (`monitor/series.ts:40-77`); el problema es de las cifras de las
  tablas y tarjetas.
- [alta] — **El violeta significa tres cosas**: "derivado" (`StatTable.tsx:62`),
  "del instante" (`StatTable.tsx:63` y `StatCard.tsx:32`) y "origen: replay"
  (badge por defecto, `ChartCard.tsx:43`); el "derivado" se marca además en
  ámbar (`MonitorCharts.tsx:125`, `:144`, `:232`). La leyenda del pie sólo
  explica el violeta (`MonitorCharts.tsx:239-240`). Una cifra derivada
  mostrada en `StatCard`/`BarList` no tiene ninguna marca de color.
- [alta] — **"Sin datos" no es "publicado"**. El booleano `live` significa "el
  replay conoce esta entidad" (`entities.ts:174`, `:201`, `:230`, `:252`); cuando
  es `false` la celda queda en slate (`StatTable.tsx:62-63`), que la leyenda
  declara "publicado por el motor" (`MonitorCharts.tsx:239-240`). Una entidad
  no alcanzada muestra 0 como si fuera un 0 publicado. Falta el tercer estado.
- [alta] — **El pie dice el divisor equivocado**: "divididos por el tiempo de
  la corrida" (`MonitorCharts.tsx:151`) y "tiempo ocupado / tiempo de la
  corrida" (`MonitorCharts.tsx:126`), pero el divisor real es
  `clock - start_time` (`MonitorCharts.tsx:73-75` → `entities.ts:161`). Sólo
  coinciden al final; contradice `docs/app.md:381-383`.
- [media] — Sin estado vacío ni eje de valores comunes: `TimeSeriesChart` deja
  el dominio Y a la librería (`TimeSeriesChart.tsx:76`) y no tiene estado vacío,
  mientras el resto sí lo tiene (`StatTable.tsx:33`, `BarList.tsx:30`); hay 7
  mensajes vacíos distintos, 6 con `py-6` y uno con `py-4` (`BarList.tsx:30`).
- [media] — `EntityPicker` sólo lo usa `QueueSeries.tsx:56`;
  `StorageSeries.tsx:41-52`, `ChainSeries.tsx:41-52`,
  `TableHistogram.tsx:88-100` y `WaitDistribution.tsx:113-125` copian el mismo
  marcado. Y "las entidades del modelo" no es el mismo conjunto: el picker usa
  `monitor.queues` (replay) y las tablas usan reporte ∪ replay
  (`entities.ts:137-144`): una cola no tocada aparece con ceros en la tabla
  pero no se puede graficar.
- [media] — Dos sistemas de color: `BarList` recibe clases Tailwind
  (`BarList.tsx:44`, `BlockRanking.tsx:32`) y `palette.ts:10-21` da hex;
  `StorageSeries.tsx:78` y `ChainSeries.tsx:69` hardcodean hex fuera de la
  paleta.
- [media] — `BarList.tsx:46` fuerza `Math.max(2, share*100)`: un recurso con 0
  muestra barra visible (en `ResourceRanking` todas al 2 % si el mayor es 0).
- [media] — "Media" significa dos cosas: "espera media" = `totalTime/entries`
  (`entities.ts:167`) vs. "media de las salidas" (`WaitDistribution.tsx:79`);
  el diseño (`odd/tasks/genmsi-graficos-estadisticas.md` L5) pedía que
  coincidieran.
- [baja] — `Elapsed.published` no lo usa nadie (`MonitorCharts.tsx:75` lo
  calcula, `entities.ts:38-41` lo declara): la mitad "publicado" de la regla
  del reloj no existe, aunque el motor publica
  `area/total_time/entries/zero_entries/max/retry` por entidad
  (`monitor/replay.ts:100-120`) que la app ignora salvo nombres y capacidad
  (`entities.ts:205-210`).
- [baja] — Datos muertos: `BlockRow.current/class/live` (`blocks.ts:23-33`) no
  los muestra ninguna tarjeta; `useFigures()` (`monitor/live.ts:204`) y
  `useRunHasPublished()` (`live.ts:225`, que sí usa el canvas en
  `GraphView.tsx:125`) no los usa ningún gráfico.
- [baja] — Claves: `BarList.tsx:38` (`name+index`) y `StatTable.tsx:53`
  (`index`) se rompen al reordenar en vivo (`blocks.ts:112` ordena por
  entradas).
- [baja] — `series.ts` tiene dos convenciones de intervalo: `histogram()` usa
  `[low, high)` (`series.ts:140-156`) y `histogramRows()` la del motor
  (`series.ts:210-231`): dos histogramas rotulan al revés.
- [baja] — `WaitDistribution.tsx:155-159` y `:168-172` dibujan la línea de la
  media dos veces, con el mismo `#0f172a` que en `TimeSeriesChart.tsx:107`
  significa "el reloj".

**Propuesta para la familia**

1. **Un `Instant` único** (`{clock, figures, elapsed.live, elapsed.published,
   known}`) que arme `entities.ts`; leer las figuras del store vivo
   (`useFigures`) y el reloj del mismo `publish`, y **no derivar ratios si
   `figures` y `clock` no son del mismo instante**.
2. **Un tipo `Metric` único** (`{value, kind: engine|derived, formula}`)
   pintado por un solo componente con una sola semántica de color (violeta =
   instante, ámbar + `*` = derivado, slate = publicado) y **una sola leyenda**
   que el panel genere, no que cada tarjeta escriba.
3. **Un estado de tres valores** (instante / publicado / todavía no) en vez del
   booleano `live`.
4. **Piezas nuevas compartidas**: `<EmptyState/>`, un único `EntityPicker`
   (multiselección y uno-a-uno) usado por las 5 tarjetas con la lista
   reporte ∪ replay, `shareOf()` en la familia (hoy triplicado en
   `ResourceRanking.tsx:48`, `BlockRanking.tsx:27`, `RetryRanking.tsx:28`), y
   `TimeSeriesChart` con dominio Y desde 0 y estado vacío.
5. **El pie con la fórmula lo genera la familia**, no la tarjeta: `derived.ts`
   sabe que el divisor es `elapsed.live` y el pie debe decir "tiempo hasta el
   reloj", no "tiempo de la corrida".

**Tests de la familia**

- Cubiertos: `tests/entities.test.ts` (el cero de una entidad no alcanzada y
  el `live=false`, `:104-118`; las derivadas divididas por el tiempo del
  instante, `:120-128`); `tests/series.test.ts` (serie desde 0, corte en el
  reloj, tramos, ambos histogramas, `:135-148`); `tests/derived.test.ts` (las 6
  familias de ratio y el divisor 0); `tests/stats.test.ts` (la aritmética del
  motor); `tests/live.test.ts` (identidad, `setClock`);
  `tests/replay.test.ts:175-185` (serie y reloj).
- **Sin cubrir**: `charts/blocks.ts` (nada importa `blockRows`), `palette.ts` y
  **todos los componentes React** (`ChartCard`, `StatTable`, `StatCard`,
  `BarList`, `TimeSeriesChart`, `EntityPicker`). El runner es
  `node --test tests/*.test.ts` (`crates/genmsi-app/package.json:10`), sin DOM:
  por eso las piezas no se prueban. **Propuesta**: mover la lógica de las
  piezas a módulos puros (`shares`, estados de valor, etiqueta de fórmula) y
  probarlos como se prueba `derived.ts`.

## Los gráficos que faltan

El panel **no consume las estadísticas publicadas** del reporte: del documento
JSON toma sólo los *nombres* de las entidades y la *capacidad* de los
almacenes; todo número que se ve sale del replay
(`charts/entities.ts:18-25`, `monitor/stats.ts:1-20`). Por eso "lo que no
muestra" = campos publicados **sin equivalente vivo** en ninguna tarjeta.

| Dato publicado | Motor | ¿Sirve para entender la corrida? | Tarjeta que lo hospeda |
|---|---|---|---|
| `Storage.delay` — transacciones **aún esperando unidades al terminar** | `crates/genmsi-core/src/stat.rs:280` (se fija en `stat.rs:344`) | Sí, mucho: dice que la corrida acabó **bloqueada**. Hoy sólo se ve en el texto del reporte | "Unidades en uso — almacén" (`charts/StorageSeries.tsx:88-118`): quinta StatCard "Esperando al final" (marcada *publicada*) |
| `Table.overflow_sum` / `overflow_sum_squares` → media y desviación de los desbordes | `stat.rs:492-494` (el motor ya las usa en `report.rs:551-565`) | Sí, cuando `overflow > 0`: dice de qué tamaño son los valores que no entraron en su clase | "Las tablas del modelo" (`charts/TablesList.tsx:73-99`): columna "Media de los desbordes" (derivada, con asterisco) |
| `entities.logic` y `entities.savevalues` | `stat.rs:590-592` | Sí: son las variables que gobiernan el flujo (un GATE sobre un switch). El replay ya las calcula vivas (`monitor/stats.ts:415,421`) | Tarjeta nueva compacta "El estado del modelo" (switches on/off + savevalues), o StatCards en "La corrida en números" |
| `Storage.min_use` | `stat.rs:262` | Poco: casi siempre 0 | "Unidades en uso — almacén"; columna "Mínimo" junto a "Máximo" (`StorageSeries.tsx:96-100`) |
| `Facility.available` | `stat.rs:27` | Marginal: sólo en modelos con facility deshabilitada | "Ocupación de las facilities": un tercer estado (libre / ocupada / no disponible) en `FacilityTimeline.tsx:60-85` |
| `report.seed` y `report.title` | `crates/genmsi-core/src/report.rs:1413,1416` | Sí (reproducibilidad: la corrida es reproducible sólo con la semilla) | "La corrida en números" (`MonitorCharts.tsx:112-121`): título y semilla en el encabezado, como en `ReportView.tsx:158` |
| `Queue.retry` | `stat.rs:152` | **No**: el motor nunca lo incrementa (sólo facility `sim.rs:2463` y storage `sim.rs:2458`) | — dato muerto: que siga oculto |
| `JsonBlock.current` por bloque, `operands`, `label`, `class` | `report.rs:1391,1383-1385` | El `current` por bloque ya se ve en el lienzo; el texto del programa, en la pestaña Reporte | — que sigan ocultos |
| `report.diagnostics` | `report.rs:1434` | No es una cifra | — que siga oculto (status line) |

**Lo que ni siquiera se publica** (imposible hoy): `result.pending` (tabla
`FEC_XACT_NUMBER`, `crates/genmsi-core/src/report.rs:418`), `result.groups`
(`XACT_GROUP`/`NUMERIC_GROUP`, `report.rs:382`) y `result.logic_switches`
(`report.rs:368`) están en el reporte de texto pero **no** en `JsonReport`
(`report.rs:1411-1435`). El más valioso sería *pending* (qué va a hacer cada
transacción programada). Requiere campo nuevo en el motor.

## Las tarjetas que sobran (o que se solapan)

- **"Los recursos que aprietan"** (`ResourceRanking.tsx:78-101`) repite tres
  cifras ya mostradas: utilización de facilities (`FacilityTable.tsx:33`),
  contenido medio de colas (`QueueTable.tsx:34`) y utilización media de
  almacenes (`StorageSeries.tsx:100`). Lo único nuevo es el top-5 y el veredicto
  final, que **compara un % con un conteo** (`ResourceRanking.tsx:126-132`):
  mixing unidades, no significa nada. → mantener sólo el veredicto con unidades
  comparables, o la tarjeta.
- **Cabecera del "Histograma de la tabla"** (`TableHistogram.tsx:143-148`)
  repite tres columnas de "Las tablas del modelo" (`TablesList.tsx:73-95`):
  misma información, igual de legible.
- **Utilización por facility al borde de la pista** (`FacilityTimeline.tsx:87-89`)
  repite la columna "Utilización" (`FacilityTable.tsx:33`).
- **No** es redundancia, pero sí trampa: "Espera media" (`QueueTable.tsx:40`,
  área/entradas) y "media de las salidas" (`WaitDistribution.tsx:99-108`, media
  de muestras) usan denominadores distintos con el mismo nombre.

## Las estadísticas que la app calcula y tira

- `Figures.logic` y `Figures.values` vivos (`monitor/stats.ts:415,421`),
  verificados por el chequeo de fidelidad (`monitor/replay.ts:1516-1523`):
  nunca se dibujan.
- `StorageFigure.minUse` (`monitor/stats.ts:353`) se calcula y se pierde:
  `StorageNumbers` no lo lleva (`charts/entities.ts:205-227`).
- `StorageNumbers.entries` y `QueueNumbers.content` (`entities.ts:120,214`) se
  calculan y no se renderizan: la tabla de colas **no tiene columna "Ahora"**,
  que sí tienen facility, almacén y cadena (`FacilityTable.tsx:28`,
  `StorageSeries.tsx:93`, `ChainSeries.tsx:78`).
- `chainDerived().meanSize` (`monitor/derived.ts:132`, `entities.ts:251`) y
  `tableDerived().overflowRate` (`monitor/derived.ts:174`): calculados, nunca
  renderizados (sólo tests).
- `Elapsed.published` (`entities.ts:37-41`, `MonitorCharts.tsx:80`): se calcula
  y nunca se lee — todas las derivadas dividen por `elapsed.live`. La mitad
  "publicada" de la regla *el reloj manda* no tiene consumidor: la sección no
  puede hoy contrastar el valor vivo con el publicado.
- `BlockRow.class` (`charts/blocks.ts:52-64`): calculado, nunca leído.
- **No reabrir**: la línea 0/1 de facilities (decidido en
  `odd/tasks/genmsi-graficos-estadisticas.md`, S5/L4) ni el alcance curado en
  seis grupos (S1); si acaso, la espera por cola muestra una cola a la vez, y
  eso es diseño, no olvido.
- Cobertura: los tests fijan `derived/series/entities/replay`, pero ninguna
  tarjeta tiene test propio.

### Distribución de las esperas

**Qué muestra.** El histograma de las esperas de **una cola**, con una barra por
intervalo y la curva acumulada; el selector de pestañas elige la cola. Las
muestras salen de la **traza**: el replay recuerda la entrada de cada
transacción en la cola (`monitor/replay.ts:548`) y anota una muestra por salida,
con la espera = instante de salida − instante de entrada
(`monitor/replay.ts:561-574`); la tarjeta las filtra por el reloj
(`charts/WaitDistribution.tsx:67`), las mete en intervalos con `histogram()`
(`monitor/series.ts:134-152`) y calcula la acumulada con `cumulative()`
(`series.ts:159-175`). La media y el ancho de los intervalos los calcula la app
(`WaitDistribution.tsx:76-79`). Badge: "derivado de la traza"
(`MonitorCharts.tsx:140-147`).

**1. ¿Tiene sentido?** Sí, y no es redundante: es la única tarjeta que muestra
la **forma** de las esperas; *Indicadores por cola* sólo da media, máximo y
proporción de espera cero (`QueueTable.tsx:33-59`). Excepción: si el modelo
declara un `QTable` de esa cola, el motor ya tabula sus tiempos de espera
(`crates/genmsi-core/src/sim.rs:3080-3100`, `report.rs:315-340`) y *Histograma
de la tabla* lo dibuja con las frecuencias publicadas
(`TableHistogram.tsx:47-49`) → la misma distribución aparece dos veces, con
anchos de intervalo distintos.

**2. ¿Se muestra correctamente.** Parcialmente: el origen de los datos es
correcto y el corte por el reloj se respeta, pero las cifras y los rótulos no.

- [media] La **media que muestra no es la del motor** y no va marcada: es la
  media de las *salidas* (`WaitDistribution.tsx:79`), sin asterisco ni fórmula
  en el pie (la tarjeta no pasa `footnote`, `MonitorCharts.tsx:140-147`),
  mientras la tabla hermana muestra "Espera media \*" = `totalTime/entradas`
  (`QueueTable.tsx:40-43`, `monitor/derived.ts:59`). Son dos medias distintas
  para la misma cola, y contradice el diseño
  (`odd/tasks/genmsi-graficos-estadisticas.md:175`).
- [media] Las cifras **vivas van en gris** (`WaitDistribution.tsx:123,131-133`,
  `text-slate-500`), sin el violeta que la sección reserva para "el instante que
  marca el reloj" (`MonitorCharts.tsx:238-241`, `StatTable.tsx:62-63`): el
  lector puede leerlas como publicadas.
- [media] El **tooltip muestra el centro del intervalo**, no el intervalo: usa
  el `x` = `low + width/2` (`WaitDistribution.tsx:82,171`), y el campo `label`
  con el intervalo real (`:83`) no se usa en ninguna parte → "espera 0,05 s" por
  un intervalo [0; 0,1).
- [media] El **ancho de los intervalos depende del reloj**: `widthOf()` elige el
  paso según el mayor valor observado hasta ese instante
  (`WaitDistribution.tsx:39-47,75-76`), así que el histograma se re-binea y el
  eje se estira mientras corre (`:147`). Dos instantes no son comparables y el
  ancho no es una cifra del motor.
- [media] La **truncatura de la traza no se refleja**: si la traza se cortó,
  faltan las salidas posteriores y "N salidas" (`WaitDistribution.tsx:131`)
  subestima la corrida; hay aviso global (`MonitorView.tsx:438-442`) y el
  límite es 200.000 registros (`src/state/store.ts:66`), pero la tarjeta no lo
  dice.
- [baja] La línea de la media está **duplicada**
  (`WaitDistribution.tsx:173-177` y `:186-190`) y sin rótulo que diga qué es.
- [baja] La acumulada con `stepAfter` sobre los centros de los intervalos
  (`:191-199`) salta a mitad de intervalo: la CDF queda corrida medio intervalo.
- [baja] `barSize` se calcula sobre 620 px fijos (`:80`) dentro de un
  `ResponsiveContainer` al 100 % en una grilla de dos columnas
  (`MonitorCharts.tsx:131`): el ancho de barra no sigue al ancho real (no
  verificado en pantalla).
- [baja] `Math.max(...samples)` (`:75`) con hasta 200.000 registros de traza.
- [baja] Las pestañas salen de toda la traza (`:58`): desde el segundo cero ya
  listan colas que no han salido, en orden de primera salida y no alfabético
  (`charts/entities.ts:129-134` ordena); una cola que nunca tuvo salidas no
  aparece.

**Cobertura.** **Nada** de la tarjeta se prueba. Hay soporte indirecto:
`histogram`/`cumulative` (`tests/series.test.ts:86-108`) y el mapa de esperas
(`tests/replay.test.ts:193-198`, un solo ejemplo). `widthOf` no se exporta y no
se prueba; tampoco el filtro por reloj, el caso vacío ni la truncatura.

**3. ¿Otra forma.** Mantener el histograma, pero: (a) ancho de intervalo
**estable** (elegido una vez por cola, no por reloj) o los intervalos del
`QTable` del motor cuando exista; (b) partir el primer bin con la proporción de
**espera cero**, que el motor ya cuenta
(`crates/genmsi-core/src/stat.rs:197-199`) y la app tiene vivo
(`monitor/stats.ts:287-288`); (c) un solo cuadro de texto en violeta con
`salidas N · espera media (motor) = tiempo total/entradas *` y pie con la
fórmula; (d) rótulo del intervalo en el tooltip. Todo con los mismos datos.

**Veredicto.** KEEP WITH CHANGES
**Riesgo de equivocarse.** Que el lector tome la distribución por la de la
corrida completa (es sólo lo que salió hasta el reloj, y puede estar recortada
por la traza) y que confunda la "media de las salidas" con la "espera media" de
la cola, que divide por las entradas e incluye a las que siguen esperando.
