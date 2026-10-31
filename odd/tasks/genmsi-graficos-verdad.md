# Gráficos: que la sección diga la verdad

Implementación de la auditoría de la sección **Gráficos** del monitor. Tres
unidades revisables: primero el bug del reloj, después el rotulado, después
lo que el motor publica y la app no muestra. El diseño se hace con
**gentle-design** (no con Pen).

La auditoría que origina esto está en `odd/tasks/genmsi-auditoria-graficos-informe.md`
(19 hallazgos `[alta]`, 18 confirmados por un verificador independiente).

## Specs

### S1 — Tres unidades, en este orden

> "bien vamos con tu recomendacion! solo que ya no usaremos pen, usaremos gentle-design"

La recomendación del usuario sigue la de la auditoría:

| Unidad | Qué es |
|---|---|
| **U2** | El reloj y las cifras del mismo instante (el bug de exactitud) |
| **U1** | El rotulado: badges, color, leyenda, pies y nombres de columna |
| **U3** | Lo que el motor publica y la app no muestra |

Cada unidad es un commit revisable con sus tests. **U2 va primero**: no cambia
lo que se ve, no depende del diseño y arregla el único hallazgo que hace que
la app muestre cifras que no son del instante que el reloj marca.

### S2 — U2: el reloj y las cifras son del mismo instante

El panel usa el reloj del store vivo (`MonitorCharts.tsx:62`) y las figuras del
prop `state`, que sólo se recalcula cuando cambia el playhead
(`MonitorView.tsx:94-107`) y que **no se mueve durante la reproducción**
(`MonitorView.tsx:385`). `closeFigures` (`monitor/stats.ts:434-464`) cierra el
área con el contenido congelado mientras el reloj avanza: la utilización de una
facility ocupada en ese frame tiende a 100 % y el contenido medio de una cola
vacía decae a 0.

La corrección: un único `Instant` —`{clock, figures, elapsed.live,
elapsed.published}`— armado desde el store vivo (`useFigures`,
`monitor/live.ts:204`, que hoy no lo usa ningún gráfico), y **ninguna métrica
derivada si el reloj y las figuras no son del mismo instante**.

### S3 — U1: una sola leyenda, un solo color, tres estados

Hoy el violeta significa tres cosas: "derivado" (`StatTable.tsx:62`), "del
instante" (`:63`) y "origen: replay" (el badge por defecto,
`ChartCard.tsx:43`). Y el booleano `live` significa "el replay conoce esta
entidad", así que una entidad no alcanzada muestra 0 en slate, que la leyenda
reserva para "publicado" (`MonitorCharts.tsx:239-240`).

La corrección: un color por significado (instante · derivado · publicado), una
leyenda sola que el panel genera, y **tres estados**: del instante, publicado
y todavía no.

### S4 — U1: los pies dicen el divisor real

El divisor real de las derivadas es el reloj del instante
(`clock - start_time`), no "el tiempo de la corrida" (`MonitorCharts.tsx:126`,
`:151`). Cada pie dice su fórmula y el tiempo que divide.

### S5 — U1: las cifras se nombran como lo que son

- La **espera media** de la cola pasa a ser la del motor (`área/entradas`,
  `crates/genmsi-core/src/stat.rs:211-223`), para que la app y el reporte del
  CLI digan lo mismo; la media **de las salidas** queda como segundo número,
  rotulado.
- El **tiempo medio** de la cadena pasa a ser `área/entradas`
  (`stat.rs:439-451`).
- La columna **Reintentos** de las facilities deja de mostrar un 0 que no es
  el del motor (`monitor/replay.ts` nunca emite `facility-retry`): o se emite,
  o la columna se nombra por lo que cuenta.
- Los **Reintentos por bloque** se nombran por lo que cuentan (esperas de la
  traza), no por lo que cuenta el motor (`sim.rs:2730`).
- Las cifras **vivas** que hoy van en slate (el color de "publicado") van en el
  color del instante, en toda la sección.

### S6 — U3: lo que el motor publica y la app no muestra

- `Storage.delay`: cuántas transacciones seguían esperando unidades al
  terminar la corrida (la corrida terminó bloqueada).
- `Table.overflow_sum` / `overflow_sum_squares`: media y desviación de los
  desbordes (`stat.rs:492-494`).
- `entities.logic` y `entities.savevalues`: el estado del modelo (los switches
  que gobiernan el flujo), que el replay ya calcula vivo
  (`monitor/stats.ts:415,421`).
- `seed` y `title` de la corrida.

### S7 — El diseño se hace con gentle-design

> "solo que ya no usaremos pen, usaremos gentle-design"

El diseño de lo visible (U1 y U3) se hace con la skill **gentle-design**: los
artboards van en `design/graficos-verdad/` y se abren en el lienzo. U2 no
cambia lo que se ve y no necesita diseño.

### S8 — Lo que NO se implementa

- La familia de piezas de P2 (`Metric` único, `EmptyState`, `EntityPicker`
  único, `shareOf`, `TimeSeriesChart` con dominio Y desde 0, fusionar las
  tablas con el histograma): es plomería para que las tarjetas se vean
  iguales. Se hace **sólo lo que U1, U2 y U3 fuercen**.
- La agrupación de tablas con más de 200 intervalos: ningún modelo del repo
  llega a 200 (`docs/BLOCK-INVENTORY.md:340, 442`). En vez de implementarla,
  se corrige la promesa de `docs/app.md:377-378`.
- `pending`, `groups` y `logic_switches` en el JSON del reporte: requieren
  tocar el motor y son otra feature.

### S9 — Cada unidad arranca con su test en rojo

El runner del frontend es `node --test tests/*.test.ts`
(`crates/genmsi-app/package.json`), sin DOM: la lógica de cada pieza va a
módulos puros para poder probarla, como ya se hace con `derived.ts`.

> Ojo: `tests/series.test.ts:138` fija hoy el rótulo defectuoso "0 - 10" de la
> primera clase del histograma. Si U1 lo corrige, ese test cambia.

## Tasks

| ID | Specs | Tarea | Ruta | Commit |
|---|---|---|---|---|
| T1 | S1 | Documento de la feature, rama `work/graficos-verdad` | inline | pendiente |
| T2 | S7 | Sync del sistema de diseño (`design/system.md` + `design/system.html`) | inline | pendiente |
| T3 | S7 | Diseño de U1 y U3 con gentle-design (`design/graficos-verdad/`) | subagente | pendiente |
| T4 | S2 | **U2**: el reloj y las cifras del mismo instante | subagente | pendiente |
| T5 | S3, S4, S5 | **U1**: rotulado, leyenda, tres estados, pies y nombres | subagente | pendiente |
| T6 | S6 | **U3**: delay, desbordes, estado del modelo, semilla | subagente | pendiente |
| T7 | S8 | Corregir la promesa de la agrupación de 200 intervalos en `docs/app.md` | inline | pendiente |
| T8 | S9 | Verificación: suite del frontend + typecheck + mirada a la app | inline | pendiente |

### El orden y por qué

T4 (U2) va primero y no espera al diseño: es el bug. T3 (el diseño) va en
paralelo porque U1 y U3 dependen de que el usuario apruebe el lienzo. T5 y T6
esperan al diseño aprobado.

## Log

- L1 — "bien vamos con tu recomendacion! solo que ya no usaremos pen, usaremos gentle-design"
- L2 — La auditoría previa: `odd/tasks/genmsi-auditoria-graficos-informe.md`, con 19 hallazgos `[alta]` (18 confirmados de forma independiente) y el veredicto de las trece tarjetas: todas KEEP WITH CHANGES.
- L3 — Decisiones que el usuario acepta al decir "vamos con tu recomendación": tres unidades (U2, U1, U3), no implementar la familia de P2, no implementar la agrupación de 200 intervalos, y que la cifra titular de la espera sea la del motor (`área/entradas`) para que la app y el reporte del CLI digan lo mismo.
- L4 — El diseño pasa de Pen a **gentle-design**: los artboards van en `design/graficos-verdad/` y se revisan en el lienzo.
- L5 — La rama de la implementación es `work/graficos-verdad`, nacida de `work/auditoria-graficos` (que ya tiene la auditoría), en el mismo worktree `/home/lucas/Documentos/Workspace/Genmsi-worktrees/auditoria-graficos`.
