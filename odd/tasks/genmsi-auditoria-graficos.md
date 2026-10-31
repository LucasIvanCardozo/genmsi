# Auditoría de la sección de gráficos

Auditoría de las trece tarjetas del panel **Gráficos** del monitor: qué dice
cada una, si lo dice bien, y si habría una forma mejor de decirlo. Cada
tarjeta la juzga un subagente con foco exclusivo en ella; el resultado es un
documento único con una sección por gráfico.

## Specs

### S1 — Un subagente por gráfico

> "me gustaria que para cada grafico le dediquemos un subagente especifico para que este full enfocado en uno"

Cada una de las trece tarjetas de la sección **Gráficos** tiene un subagente
propio, con foco exclusivo en ella. Las piezas compartidas (la tarjeta, las
tablas, las barras, la serie del tiempo) tienen un subagente aparte.

### S2 — Tres preguntas por gráfico

> "analicemos la seccion de graficos analizando cada uno por si tiene sentido, sis e muestra correctamente, o si se podria mostrar la misma info de otra manera"

Cada tarjeta se juzga en tres ejes:

1. **¿Tiene sentido?** — ¿la información aporta algo para entender la corrida?
2. **¿Se muestra correctamente?** — ¿los números que dibuja son los que dice,
   con la fuente y el reloj que dice?
3. **¿Se podría mostrar la misma información de otra manera?** — una forma más
   legible o más útil, con la misma información o con menos.

### S3 — Tarjetas innecesarias y tarjetas que faltan

> "ademas de ver gfraficos que esten y sean innecesarios o graficos posibles a implemnentar que no se muestran actualmente"

Además del juicio por tarjeta, el informe dice:

- qué tarjetas sobran: información redundante, o información que no compensa
  el espacio que ocupa;
- qué gráficos serían posibles con los datos que el motor **ya** publica y que
  la app todavía no muestra.

### S4 — Alcance: el panel "Gráficos" y sus piezas compartidas

> "Panel \"Gráficos\" + piezas compartidas"

El alcance es el panel **Gráficos** del monitor (sus trece tarjetas) más las
piezas que comparten. Quedan fuera el panel *Reporte* y el lienzo de
*Reproducción*.

### S5 — Un solo documento, con una sección por gráfico

> "Un documento único con una sección por gráfico"

El entregable es un solo archivo con una sección por tarjeta, más un resumen
con el veredicto de cada una.

### S6 — La auditoría no toca el código

Esta sesión es de análisis: no se modifica el código de la app. Lo que sale de
aquí son hallazgos con evidencia y propuestas; implementarlos es una decisión
posterior, del usuario.

## Tasks

| ID | Specs | Tarea | Ruta | Commit |
|---|---|---|---|---|
| T1 | S1-S6 | Documento de la feature, inventario de las trece tarjetas y las piezas compartidas | inline | `ec22bf3` |
| T2 | S1, S2 | Un subagente por tarjeta (trece, en tandas) | subagentes | — |
| T3 | S2 | Un subagente para las piezas compartidas | subagente | — |
| T4 | S3 | Un subagente para los gráficos que faltan: lo que el motor publica y la app no muestra | subagente | — |
| T5 | S2, S3, S5 | Síntesis: el informe único con una sección por gráfico y el resumen de los veredictos | inline | `ec22bf3` |
| T6 | S2, S3 | Verificación: cada hallazgo comprobado contra el código; ninguna afirmación sin evidencia | inline + subagente verificador | `d3a4f1c` |

### El corte de la entrega

La auditoría es **un documento**: no hay código, así que el commit es una sola
unidad revisable (`ec22bf3`, 1811 líneas de prosa con evidencia). El corte de
revisión, si el usuario decide implementar, no es este commit sino los tres
bloques de prioridades del informe:

| Unidad | Qué entra | Tamaño esperado |
|---|---|---|
| P0 | El rotulado (badge, color, pies), las medias del motor, los reintentos del motor y el reloj atado a las cifras | una unidad, con su test |
| P1 | Lo que el motor publica y la app no muestra, y el tercer estado ("todavía no") | una unidad, con su test |
| P2 | La familia: `Metric` único, `Instant` único, `EmptyState`, `EntityPicker` único, `shareOf`, `TimeSeriesChart` con dominio Y desde 0, y el HISTOGRAMA fusionado con la lista de tablas | una o dos unidades |

Cada unidad arranca con su test en rojo y termina con la suite entera del
frontend (`node --test tests/*.test.ts`) y el typecheck.

### El reparto de los subagentes
| # | Tarjeta | Componente | Datos |
|---|---|---|---|
| 1 | La corrida en números | `charts/RunSummary.tsx` | reporte + reloj |
| 2 | Los recursos que aprietan | `charts/ResourceRanking.tsx` | reporte + derivados |
| 3 | Contenido de las colas a lo largo del tiempo | `charts/QueueSeries.tsx` | replay |
| 4 | Distribución de las esperas | `charts/WaitDistribution.tsx` | traza |
| 5 | Indicadores por cola | `charts/QueueTable.tsx` | reporte + derivados |
| 6 | Ocupación de las facilities a lo largo del tiempo | `charts/FacilityTimeline.tsx` | replay |
| 7 | Indicadores por facility | `charts/FacilityTable.tsx` | reporte + derivados |
| 8 | Unidades en uso — almacén | `charts/StorageSeries.tsx` | replay + reporte |
| 9 | Tamaño de la cadena | `charts/ChainSeries.tsx` | replay |
| 10 | Histograma de la tabla | `charts/TableHistogram.tsx` | reporte |
| 11 | Las tablas del modelo | `charts/TablesList.tsx` | reporte |
| 12 | Transacciones por bloque | `charts/BlockRanking.tsx` | reporte + estado |
| 13 | Reintentos por bloque — la contención | `charts/RetryRanking.tsx` | reporte + estado |
| 14 | Las piezas compartidas | `charts/{ChartCard,StatTable,StatCard,BarList,TimeSeriesChart,EntityPicker}.tsx`, `charts/{palette,blocks,entities}.ts`, `monitor/{series,derived,stats}.ts` | — |
| 15 | Los gráficos que faltan | `crates/genmsi-core/src/report.rs`, `types/generated/JsonReport.ts` | — |

El encargo común de los quince está en `odd/genmsi-auditoria-graficos/AGENT-BRIEF.md`:
cada subagente lee ese archivo y hace su parte.

## Log

- L1 — "buenas!! en esta sesion me guistaria que, mediante un worktree qye que estoy trabajando en main, analicemos la seccion de graficos analizando cada uno por si tiene sentido, sis e muestra correctamente, o si se podria mostrar la misma info de otra manera, te parece? me gustaria que para cada grafico le dediquemos un subagente especifico para que este full enfocado en uno. ademas de ver gfraficos que esten y sean innecesarios o graficos posibles a implemnentar que no se muestran actualmente, te parece?"
- L2 — Decisiones del usuario sobre el alcance y el entregable: alcance **"Panel \"Gráficos\" + piezas compartidas"**; entregable **"Un documento único con una sección por gráfico"**.
- L3 — El trabajo ocurre en un worktree propio (`/home/lucas/Documentos/Workspace/Genmsi-worktrees/auditoria-graficos`, rama `work/auditoria-graficos`) para no ensuciar `main`, que es la rama de trabajo del usuario.
- L4 — El reparto: quince subagentes (catorce de lectura con `gentle-ai-explore` y uno de verificación con `gentle-ai-verify`), todos con el encargo común de `odd/genmsi-auditoria-graficos/AGENT-BRIEF.md`. El de *Distribución de las esperas* falló sin informe la primera vez y se relanzó.
- L5 — La verificación del padre: once hallazgos `[alta]` contrastados uno por uno contra el código (el denominador del pie, el veredicto "el que más aprieta", `AVE.TIME` de colas y cadenas, el productor inexistente de `facility-retry`, la divergencia del contador de reintentos, el badge mentiroso de las tablas, la agrupación de 200 intervalos que la documentación promete y el código no hace, la ausencia de productor de `facility-retry`, el `countsOf` sin el conjunto de generadores, el reloj y las cifras desalineados en la reproducción). Una re-verificación independiente quedó delegada a `gentle-ai-verify` (T6).
- L6 — El informe: `odd/tasks/genmsi-auditoria-graficos-informe.md`. Veredicto de las trece tarjetas: **KEEP WITH CHANGES** en todas; ninguna se propone borrar. El problema de la sección no es qué muestra, sino **cómo lo dice**: el rotulado de la procedencia (badge, color, pies), las medias que no son las del motor, los reintentos que no son los del motor y el reloj desalineado de las cifras durante la reproducción.
- L7 — Un hallazgo del padre que ningún subagente vio del todo: `closeFigures` (`monitor/stats.ts:434-464`) cierra el área con el contenido **congelado** del frame del playhead mientras el reloj del store vivo avanza. Consecuencia medida en el código: la utilización de una facility ocupada en ese frame tiende a 100 % (`busyTime = busyTime0 + (clock − seizedAt)`, dividido por `clock`) y el contenido medio de una cola vacía en ese frame decae hacia 0. Las series temporales no lo sufren (salen de `monitor.*Points` y se cortan en el reloj); el problema es de las cifras de tablas y tarjetas.
- L8 — La verificación independiente cerró: de los 19 hallazgos `[alta]`, **18 confirmados y ninguno incorrecto**; uno quedó como no verificable desde el código (el posible "+1" de las cadenas con `LINK` y destino, que necesita un modelo que el repo no tiene). Los `[media]` con afirmación factual, todos confirmados. Cuatro citas `path:line` no eran exactas y se corrigieron — deriva menor, el contenido era correcto.

  El verificador dejó además dos regresiones anotadas (no escritas, porque la auditoría no toca código): `audit-clock-figures.test.ts` (la utilización con el área congelada tiende a 100 %; la columna de reintentos de facility es 0) y `audit-engine-metrics.test.ts` (la "espera media" de la app no es el `AVE.TIME` del motor). Y avisó de que `tests/series.test.ts:138` hay que cambiarlo si se corrige el rótulo de la primera clase del histograma: hoy fija el rótulo defectuoso "0 - 10".
