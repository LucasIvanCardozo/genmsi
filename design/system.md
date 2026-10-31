# Sistema de diseño — genmsi-app

Tokens leídos **del código del proyecto**, sin inventar nada. Cada entrada
dice su fuente.

> Esta app no tiene un sistema de diseño escrito: lo tiene **sembrado en el
> código**. Tailwind v4 entra por `@import "tailwindcss"` y el 90 % de los
> tokens son clases utilitarias de la paleta por defecto. Lo que sí son tokens
> del proyecto —y por eso están aquí— son la paleta de las series de los
> gráficos y las reglas de las tarjetas.

## Fuentes

| Token | Valor | Fuente |
|---|---|---|
| `--font-sans` | `"Inter", "Segoe UI", system-ui, sans-serif` | `crates/genmsi-app/src/styles.css:4` |
| `--font-mono` | `"JetBrains Mono", "Cascadia Code", ui-monospace, monospace` | `crates/genmsi-app/src/styles.css:5` |

Los números se escriben siempre en la mono (`font-mono`), para que las
columnas de cifras alineen.

## Color

La paleta base es la de **Tailwind v4 por defecto**, importada en
`crates/genmsi-app/src/styles.css:1`. Los valores hexadecimales no están en el
repo (vienen del paquete), así que aquí se registran por nombre de clase, que
es como el código los usa.

| Rol | Clases en uso | Dónde |
|---|---|---|
| Neutros (texto, bordes, fondos) | `slate-50`, `100`, `200`, `300`, `400`, `500`, `600`, `700`, `800`, `900` | toda la app; `slate-500` (49 usos), `slate-800` (31), `slate-400` (29), `slate-200` (28) |
| Acento (la acción, el foco, "lo vivo") | `indigo-50`, `500`, `600`, `700`, `900` | barra superior, foco, cifras del instante |
| Derivado / atención | `amber-50`, `500`, `600`, `700`, `900` | las cifras derivadas de la sección de gráficos (`MonitorCharts.tsx:125`, `:144`) |
| Error | `rose-50`, `200`, `900` | `ErrorNotice.tsx` |
| Acierto | `emerald-50`, `200`, `900` | avisos de éxito |
| Series de facility / almacén | `teal-600`, `orange-500` | `ResourceRanking.tsx` |
| Selector del panel | `sky-700`, `sky-900` | `ReportView.tsx` |

### La paleta de las series de los gráficos

Token del proyecto. `crates/genmsi-app/src/components/charts/palette.ts:14-25`:

| # | Hex | Uso en el panel |
|---|---|---|
| 1 | `#6366f1` | primera serie (colas) |
| 2 | `#0d9488` | segunda serie (almacenes) |
| 3 | `#ea580c` | tercera serie; también la curva acumulada del histograma |
| 4 | `#7c3aed` | cuarta serie |
| 5 | `#e11d48` | quinta serie |
| 6 | `#0284c7` | sexta serie |
| 7 | `#d97706` | séptima serie |
| 8 | `#059669` | octava serie |
| 9 | `#9333ea` | novena serie |
| 10 | `#db2777` | décima serie |

Las series se eligen por su lugar en la lista (`colorOf`,
`palette.ts:28-30`).

### Tinta de los gráficos

Hexes escritos a mano dentro de los gráficos, con su uso:

| Hex | Uso | Fuente |
|---|---|---|
| `#0f172a` | la línea vertical del reloj | `charts/TimeSeriesChart.tsx:107`, `WaitDistribution.tsx:175` |
| `#f1f5f9` | la rejilla del gráfico | `charts/TimeSeriesChart.tsx:64`, `WaitDistribution.tsx:143` |
| `#6366f1` | serie 1 / barras del histograma | `palette.ts:15`, `WaitDistribution.tsx:182` |
| `#ea580c` | la curva acumulada | `WaitDistribution.tsx:193` |
| `#0d9488` | serie 2 | `palette.ts:16` |

## Radios

| Token | Valor | Dónde |
|---|---|---|
| `rounded-lg` | 0.5 rem | la tarjeta (`ChartCard.tsx:31`), la StatCard (`StatCard.tsx:25`) |
| `rounded-md` | 0.375 rem | botones (`components/chrome.ts`), pestañas de entidad (`QueueSeries.tsx:107`) |
| `rounded` | 0.25 rem | los badges (`ChartCard.tsx:43`) |

## Espaciado

| Token | Uso |
|---|---|
| `p-4` | el interior de la tarjeta (`ChartCard.tsx:31`) |
| `px-3.5 py-2.5` | la StatCard (`StatCard.tsx:25`) |
| `gap-3` | entre tarjetas de la grilla (`MonitorCharts.tsx:130`) |
| `px-1.5 py-0.5` | el badge (`ChartCard.tsx:43`) |
| `gap-2` | entre la etiqueta y el valor de una StatCard |

## Sombras

El proyecto **no usa sombras** en la sección de gráficos: la profundidad viene
del borde (`border-slate-200`) y del fondo (`bg-white`). La única sombra es el
halo de error (`--alert-ring`, `styles.css:16-31`), y sólo en `alert-flash`.

## Reglas de los componentes

### La tarjeta (`ChartCard`)

`crates/genmsi-app/src/components/charts/ChartCard.tsx:31-54`

- Contenedor: `rounded-lg border border-slate-200 bg-white p-4`.
- Título: `text-sm font-semibold text-slate-800`.
- Subtítulo: `text-[11px] leading-snug text-slate-500` — dice **qué** muestra.
- Badge: `text-[10px] font-semibold px-1.5 py-0.5 rounded` — dice **de dónde**
  sale el número. Es la promesa de procedencia de la sección.
- Pie: `text-[11px] leading-snug text-slate-400` — dice **cómo** se hizo el
  número.

### La StatCard (`StatCard`)

`crates/genmsi-app/src/components/charts/StatCard.tsx:25-36`

- Etiqueta: `text-[10px] font-semibold uppercase tracking-wider text-slate-400`.
- Valor: `font-mono text-lg font-semibold`; en `indigo-700` si es del
  instante, en `text-slate-800` si es publicado.
- Nota: `text-[11px] leading-snug text-slate-500`.

### La tabla de indicadores (`StatTable`)

`crates/genmsi-app/src/components/charts/StatTable.tsx:36-74`

- `text-xs`, cabecera `text-slate-500`, filas con `border-b border-slate-100`.
- El nombre de la entidad en `text-slate-800`; los números en `font-mono`.
- Una columna **derivada** lleva un asterisco `indigo-500` en la cabecera.

### La lista de barras (`BarList`)

`crates/genmsi-app/src/components/charts/BarList.tsx`

- Nombre en `text-slate-700`, número en `font-mono text-slate-700`.
- La barra es una fracción de la más larga, en el color de la serie.

### Las series temporales (`TimeSeriesChart`)

`crates/genmsi-app/src/components/charts/TimeSeriesChart.tsx`

- Rejilla `#f1f5f9`, ejes `fontSize={10}`, línea del reloj `#0f172a` de 1,5 px.
- El eje del tiempo va de 0 al final de la corrida; el del valor, con
  `allowDecimals={false}`.

## Reglas de la sección de gráficos

Las reglas que la sección se impone a sí misma (`docs/app.md:349-383`,
`odd/tasks/genmsi-graficos-estadisticas.md`):

1. **El reloj manda**: el reporte describe la corrida completa, los gráficos
   describen el instante.
2. **Sólo ratios del motor**, marcados con asterisco y con la fórmula escrita.
3. **El eje del tiempo arranca en 0**.
4. **La serie es la historia exacta**, no una muestra.
5. **Cada tarjeta dice de dónde salen sus números y cómo se hicieron**.
