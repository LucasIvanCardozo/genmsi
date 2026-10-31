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

### S3 — U1: una sola leyenda, un solo color, cuatro estados

Hoy el violeta significa tres cosas: "derivado" (`StatTable.tsx:62`), "del
instante" (`:63`) y "origen: replay" (el badge por defecto,
`ChartCard.tsx:43`). Y el booleano `live` significa "el replay conoce esta
entidad", así que una entidad no alcanzada muestra 0 en slate, que la leyenda
reserva para "publicado" (`MonitorCharts.tsx:239-240`).

La corrección: **un color por significado**, **una leyenda sola** (la genera el
panel, no cada tarjeta) y **cuatro estados**, con estas palabras exactas (las
del arteboard elegido):

| Estado | Se lee | Color | Qué es |
|---|---|---|---|
| **Del instante** | medido en el segundo que marca el reloj | `indigo-700` (`#4338ca`) | la cifra del replay en el instante |
| **Derivada \*** | un ratio del motor; la fórmula está en el pie de su tarjeta | `amber-700` (`#b45309`) + asterisco | un ratio de dos cifras del motor |
| **Publicada** | lo que el motor publicó para la corrida completa | `slate-800` / `slate-500` | la cifra del reporte del motor |
| **Todavía no** | el replay todavía no la alcanzó: no es un cero, es que no se sabe | `slate-300`, con marca de vacío | la entidad que el instante todavía no conoce |

Un estado por cifra, en un solo módulo compartido (`charts/states.ts`), para
que ninguna tarjeta invente el suyo.

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
| T4 | S2 | **U2**: el reloj y las cifras del mismo instante | subagente | `f7e587f` |
| T5 | S3, S4, S5 | **U1**: rotulado, leyenda, cuatro estados, pies y nombres | subagente | `70064e4` |
| T6 | S6 | **U3**: delay, desbordes, estado del modelo, semilla | subagente | pendiente |
| T7 | S8 | Corregir la promesa de la agrupación de 200 intervalos en `docs/app.md` | inline | pendiente |
| T8 | S9 | Verificación: suite del frontend + typecheck + mirada a la app | inline | pendiente |

### El corte de la entrega

Cada unidad es un commit revisable con sus tests. El corte de revisión no es
el commit sino la unidad:

| Unidad | Qué entra | Commit |
|---|---|---|
| La base | La feature, el sistema de diseño (`design/system.md` + `design/system.html`) y la corrección de la promesa del histograma en `docs/app.md` | `84a8a0c`, `bdc6893` |
| El diseño | Los tres artboards de `design/graficos-verdad/` y el lienzo | pendiente |
| **U2** | El reloj y las cifras del mismo instante | `f7e587f` |
| **U1** | El rotulado: el panel lateral del estado de la corrida, la leyenda de los cuatro estados, los pies y los nombres | `70064e4` (panel y leyenda), la verdad de las tarjetas queda en la unidad siguiente |
| **U3** | Lo que el motor publica y la app no muestra | pendiente |

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
- L6 — El sistema de diseño del proyecto se sync-eó de su código (`crates/genmsi-app/src/styles.css`, `charts/palette.ts` y las reglas de las tarjetas): `design/system.md` y `design/system.html`, el tablero que el lienzo muestra antes de cualquier diseño.
- L7 — T7 cerrado: `docs/app.md` ya no promete la agrupación de los intervalos de más de 200 (no existe en el código) y dice la verdad: una barra por intervalo, con el ancho que sigue al número de intervalos.
- L8 — El diseño y la unidad U2 corren en paralelo, con superficies de edición disjuntas: el diseño toca `design/graficos-verdad/` y U2 toca el motor del replay, el panel y sus tests.
- L9 — **El diseño**: `design/graficos-verdad/`, con tres direcciones del mismo contenido y el mismo modelo de ejemplo (hospital: «Mesa 1»/«Mesa 2», «Sala de espera»/«Laboratorio», «Reactivos», «Paciente», «Tiempos de servicio»; reloj 960,0 s de 1.200,0 s; semilla 20260415). Las tres muestran los tres estados, el pie con el divisor real y la información nueva de U3 («esperando al final» 6, media/desviación de los desbordes 18,6 s / 2,1 s, los switches y los valores guardados, la semilla y el título):

  | Arteboard | Dirección | Lo que gana | Lo que paga |
  |---|---|---|---|
  | `01-una-leyenda-arriba.html` | Una leyenda arriba | los tres estados se explican una sola vez; las tarjetas conservan su forma | hay que volver arriba para recordar la leyenda; el pie queda corto |
  | `02-procedencia-por-tarjeta.html` | Cada tarjeta declara su procedencia | cada tarjeta se sostiene sola; panel denso | el metadato se repite en las trece; más alto y más ruidoso |
  | `03-panel-lateral-estado.html` | Un panel lateral con el estado de la corrida | lo que describe la corrida junto, en un panel fijo | la sección depende del ancho: angosta, las columnas se apilan |

  Los tres son **alternativas**: la elección es del usuario, y el handoff de implementación (`gentle_design implement`) necesita el arteboard elegido. El self-check del diseño: sólo las fuentes del sistema (`--font-sans` y `--font-mono`), slate/indigo/amber/rose/emerald y las series `#6366f1`, `#0d9488`, `#ea580c`, `#7c3aed`; sin URLs externas ni CDN.
- L10 — **Arteboard elegido**: `design/graficos-verdad/03-panel-lateral-estado.html` («Un panel lateral con el estado de la corrida»); los otros dos quedan como alternativas. El handoff quedó registrado en `design/graficos-verdad/handoffs.json` y el lienzo lo muestra como el arteboard en curso. U1 y U3 se implementan con este arteboard como especificación visual: las tarjetas a la izquierda, y a la derecha un panel fijo con el título, la semilla, el reloj, el fin, los switches, los valores guardados, «esperando al final» y la leyenda de los estados.
- L11 — El artefacto del handoff define la leyenda de los cuatro estados (S3): **del instante** (indigo), **derivada \*** (ámbar, con asterisco), **publicada** (pizarra) y **todavía no** (pizarra clara, con marca de vacío). Las palabras son las del arteboard, textual.
- L12 — Los datos de U3 ya existen y no hay que inventar nada: `report.title`, `report.seed` y `report.end_reason` (`types/generated/JsonReport.ts`), el `delay` de los almacenes (`types/generated/Storage.ts:48`, publicado), y los switches y los valores guardados, que el replay ya calcula vivos (`monitor/replay.ts:739-747` → `Figures.logic` y `Figures.values`, `monitor/stats.ts:126-128`) y el motor publica (`types/generated/Entities.ts:31,35`).
- L13 — **U2 cerrada** (`f7e587f`): la capa viva publica el reloj, las cifras, la población, los counts y los retries en una sola `picture` atómica; el seam `instantOf` (`monitor/stats.ts`) sólo usa cifras del instante —las de la capa viva si la capa dice que son de ese frame, el respaldo del playhead sólo si su tiempo es el del reloj— y `live = 0` implica cifras nulas, así que no se deriva ninguna métrica sin instante conocido. La tarjeta *Población* deja de leer el prop congelado: dice "Sin datos." cuando el instante no se conoce. Tests nuevos: `instant.test.ts` (oráculo a mano), `population.test.ts`, `blocks.test.ts` y la regla de `known` en `live.test.ts`. Suite: 340 en verde; typecheck limpio.
- L14 — La población se calcula como la suma por línea de programa, no como la suma del `now` de los counts por bloque: sólo la fila que cuenta el bloque lleva las transacciones (un uso de facility son cinco filas y una sola cuenta), y la suma por línea es la que coincide con la rama publicada al final de la corrida.
- L15 — **W1 cerrada** (`70064e4`): el panel lateral *Estado de la corrida* (título, semilla, reloj, fin, motivo del fin, transacciones, «esperando al final» con el `delay` de los almacenes, los switches y los valores guardados, con «todavía no» para lo que el replay no alcanzó), la leyenda de los cuatro estados en su propio componente, el vocabulario compartido en `charts/states.ts` y el layout de dos columnas (el panel en un `<aside>` fuera de la grilla de tarjetas, fijo a la derecha en pantalla ancha, primero en la pila cuando es angosta). La tarjeta *La corrida en números* queda como la del arteboard y el pie de la sección con la leyenda vieja —que meanings distintos tenía— se va. Suite: 359 en verde; typecheck limpio.
- L16 — Dos cosas que W2 tiene que recoger: el `StatCard` y el `StatTable` todavía no conocen los cuatro estados (W1 dejó un `Stat` local en `RunSummary` que hay que borrar), y la caja «esperando al final» conserva la alarma ámbar del arteboard, pero en esta sección el ámbar significa «derivado»: la alarma tiene que usar la familia del error de la app.
