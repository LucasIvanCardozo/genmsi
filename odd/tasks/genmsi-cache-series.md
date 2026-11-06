# Feature: el caché de las series — que pintar los gráficos no cueste la corrida

La sección de gráficos se repinta sesenta veces por segundo mientras la
reproducción corre, y cada repintado reconstruye las series de tiempo desde
el segundo cero de la corrida. Medido con los datos del motor: **2,54 ms por
pintado** de una serie (10 000 registros, 7 colas) — **152 % de un
segundo de CPU** a 60 pintados/s — contra **0,024 ms por frame** de la
simulación (0,0014 del avance del replay, 0,0055 de los contadores, 0,0168
de la imagen del canvas). La simulación es gratis; el pintado es el costo.

Esta feature hace que un repintado cueste lo que cambió desde el repintado
anterior y no la corrida entera: un caché de series que se extiende en vez
de reconstruirse. El ritmo de pintado que ya eligió el lector sigue siendo
la palanca de la otra mitad (el dibujado de los charts), y las dos cosas son
complementarias.

- Rama: `feature/charts-performance` (base: `main`)
- Entrega: `chain_strategy=feature-branch-chain` — un PR por unidad de trabajo
- Tests: `node --test tests/*.test.ts` desde `crates/genmsi-app`
- Pronóstico: ~1800 líneas authored en total; el corte de la primera unidad
  ya pasó las 400 (`b8345df`)

## Specs

S1 (pedido del usuario, textual): "algo que me gustaria aplicar en la parte
de graficas solo si se puede e manera simple es que esa pestaña se recargue
cada x tiempo y que ese tiempo sea configurable, para que? porque como la
simulacion se hace en tiempo real, en pc malas esto puede causar mucho lag,
en cambio si damos esta opcion podemos hacer que ande mejor. Esto se me
ocurrio a mi, decime que pensas y si se te ocurre algo mejor"

S2 (pedido del usuario, textual): "realmente considero implementar la mejora
de \"no calcular la imagen del canvas cuando el canvas no está en pantalla\"
es totalmente logica no crees?"

S3 (pedido del usuario, textual, la mejora que esta feature encara):
"quiero encarar esa mejora! que me aprece increible. lanza un subagente que
se especialice en revisar todo y darte una propuesta clara, luego planteamela
a mi y si me gusta la apruebo y la implementamos" — y la aprobación: "perfecto
me encanta, vamos por ello!"

S4 (decisión del usuario, textual, sobre quién implementa): "usa subagentes
especializados para no ocupar contexto en esta sesion que debe ocuparse de
revision y orquestacion"

S5 (decisión del usuario, textual, sobre la entrega): "Cadena de ramas
(recomendado)" — la feature se entrega como una cadena de PRs, uno por
unidad de trabajo, sobre la rama de la feature.

### S1 — el ritmo del pintado

La sección de gráficos se pinta al ritmo que el lector elige, en un selector
"Refresco" en la barra de la pestaña *Gráficos*: `cada frame`, `cada 100 ms`,
`cada 250 ms`, `cada 500 ms`, `cada 1 s`, `cada 2 s`, `cada 5 s`. El valor por
defecto es `cada frame` (el comportamiento de siempre).

El selector cambia **el ritmo de la pintura**, nunca el de la simulación: la
reproducción corre igual, y la sección muestra el instante de la última
muestra. El panel lateral lo dice cuando el ritmo no es "cada frame", porque
su reloj es el reloj de la última muestra.

Lo que el lector pide se ve al instante: el playhead del header y el fin de
la corrida publican su imagen como *urgente* y saltan el muestreo.

### S2 — la imagen del canvas

La imagen del canvas —las bolas de los links y la figura de cada bloque— sólo
se calcula cuando el canvas está en pantalla. En las otras pestañas ese
trabajo es trabajo tirado: sesenta veces por segundo, para una imagen que
nadie mira. Cuando el canvas vuelve, recibe la imagen del instante, incluso
con la reproducción pausada.

### S3 — el caché de las series

Las filas de una serie son una función pura de `(points, entities, until)`:
sólo el **prefijo** depende del reloj. El caché construye las filas una vez y
las extiende; un repintado cuesta lo que cambió desde el repintado anterior.

- El caché vive en el `ReplayIndex`: su vida es la vida de la corrida. Una
  corrida nueva, un índice nuevo, un caché nuevo. Sin protocolo de
  invalidación, sin singleton, sin estado global.
- Devuelve **el mismo array** cuando nada cambió y uno nuevo cuando el
  contenido cambió: la disciplina de identidad de la casa (`reuseFigures`,
  `reuseCounts`). Nunca un array cuyo contenido cambió con identidad estable
  —recharts no lo re-renderizaría y el área congelada mentiría sobre el
  instante.
- `seriesFrom` **no se reimplementa**: queda como implementación de
  referencia, y los tests pinan que el caché y ella coinciden.

Cubre las tres familias que se reconstruyen hoy: las series (`rows`), los
segmentos de una facility (`segments`) y la distribución de las esperas
(`waits`).

### S4 — lo que el app no puede romper

Los contratos de honestidad de la sección, que el caché no puede tocar:

- la serie **empieza en cero**: el eje del tiempo es el de la corrida entera;
- la **última fila es el instante que el lector está**, con los valores del
  último cambio anterior a él — es la fila que hace que el área llegue al
  cursor;
- **no hay filas futuras**: lo que la reproducción todavía no alcanzó no es
  un cero;
- el **ancho de los bins de las esperas se elige una vez por cola**, para que
  dos instantes de la misma cola sean comparables;
- los derivados se dividen por el **reloj del instante**, no por la duración
  de la corrida.

## Tasks

| ID | Specs | Tarea | Ruta | Commit |
|---|---|---|---|---|
| T0 | S1, S2 | El ritmo del pintado y la imagen del canvas: el sampler, el selector "Refresco", la vía urgente y el panel que dice el ritmo | inline | `b8345df` |
| T1 | S3 | El caché puro `SeriesCache` (`rows`, `segments`, `waits`) y sus tests de equivalencia con las implementaciones de referencia | subagente | `18822ce` |
| T2 | S3 | El script de medición del costo del pintado, antes y después, con los datos del motor | subagente | `9666000` |
| T3 | S3 | El índice de la reproducción lleva el caché | inline | `18822ce` |
| T4 | S3 | Las tres series (colas, almacenes, cadenas) leen el caché | subagente | `9d2f796` |
| T5 | S3 | Los segmentos de las facilities y la distribución de las esperas leen el caché | subagente | `9d2f796` |
| T6 | S3 | `ResourceRanking` recibe lo que `MonitorCharts` ya calculó | inline | `9d2f796` |
| T7 | S3 | El cierre: suite completa, medición antes/después y verificación visual | inline | — |

### El corte de la entrega

La unidad base (`b8345df`, 479 líneas) ya pasó las ~400: a partir de ahí cada
unidad de trabajo es un PR encadenado sobre la rama de la feature, y el
documento va registrando el corte de cada uno.

## Log

L1 (2026-11-06, usuario, textual): "quiero encarar esa mejora! que me
aprece increible. lanza un subagente que se especialice en revisar todo y
darte una propuesta clara, luego planteamela a mi y si me gusta la apruebo y
la implementamos"

L2 (2026-11-06, usuario, textual): "perfecto me encanta, vamos por ello!"

L3 (2026-11-06, usuario, textual): "usa subagentes especializados para no
ocupar contexto en esta sesion que debe ocuparse de revision y orquestacion"

L4 (2026-11-06, usuario, textual, la entrega): "Cadena de ramas (recomendado)"

L5 (2026-11-06, análisis, la evidencia): el subagente de exploración midió
el camino del pintado y corrigió dos cosas que el orquestador tenía mal: los
gráficos usan **recharts** (no SVG a mano), y el costo dominante es la
reconstrucción de la serie (2,54 ms por pintado de una cola), no
`closeFigures` ni los derivados (O(entidades), ~10–20 µs). La propuesta que
el usuario aprobó deja `seriesFrom` como implementación de referencia y
pide que el test nuevo **pine la equivalencia** con ella.

L6 (2026-11-06, T0, inline): el ritmo del pintado y la imagen del canvas
quedaron en `b8345df` (479 líneas). Verificado en la app real con
`agent-browser`: con "cada frame" el reloj visible de la sección se movía 21
veces en 2 s; con "cada 500 ms", 4 veces en 2 s; el seek de una reproducción
pausada se ve igual de rápido con un ritmo de 2 s que con "cada frame"
(157–256 ms contra 238–314 ms), y el contador de urgentes del store
incrementa en cada seek. `tsc --noEmit` limpio, `npm run build` OK, **404/404
tests** verdes.

L7 (2026-11-06, análisis, el límite): el script `scripts/check-visual-loop.sh`
falla en la parte de edición (un arrastre con `agent-browser`) **en el
baseline, sin estos cambios**: es preexistente y no lo causa esta feature.
Además, `tests/horizon.test.ts`, `tests/live_model.test.ts` y
`tests/proposal_loop.test.ts` levantan un servidor Vite cada uno y se cuelgan
mutuamente por el caché de `optimizeDeps` cuando corren en paralelo: también
preexistente (reproducido en el baseline), y quedan fuera del portón.

L8 (2026-11-06, T1, subagente): el caché puro (`src/monitor/seriesCache.ts`,
493 líneas) y sus 20 tests (`tests/series_cache.test.ts`, 278 líneas).
RED: el módulo no existía (`ERR_MODULE_NOT_FOUND`). GREEN: 20/20, y un fuzz
diferencial fuera del worktree de 300 rondas (entidades intercaladas, puntos
desordenados, relojes barajados = scrub, varios anchos de bin) con `fuzz ok`.
El padre revisó el módulo: la fila final se arma con los valores del último
cambio, el prefijo sale de `slice`, y `segments`/`waits` reconstruyen hacia
atrás sin aliasear los arrays que ya pertenecían a otro estado.
Commit `18822ce` (con T3: el índice lleva el caché).

L9 (2026-11-06, T2, subagente, la corrección): el script de medición
(`scripts/measure-chart-cost.ts`) medió el costo **real** de un pintado y
corrigió la cifra que el orquestador venía repitiendo: los **2,54 ms** eran
una medición **en frío** (con compilación JIT); el costo en regime es
**0,258 ms** por pintado de la sección entera (1,5 % de un segundo de CPU a
60 pintados/s), y con el caché **0,014 ms** — **18× más barato**. El script
comprueba además que los dos caminos dan las mismas filas, los mismos
segmentos y las mismas figuras de esperas en los 90 relojes del barrido, y
sale con código 1 si discrepan.

L10 (2026-11-06, T4, subagente): las tres series (colas, almacenes, cadenas)
leen el caché. El diff son tres call sites y tres imports que sobran; los
`useMemo` y sus dependencias no se tocan. 424/424 tests verdes y un
diferencial propio sobre tres formas de llamada y trece relojes.

L11 (2026-11-06, T5, subagente): los intervalos de las facilities y la
distribución de las esperas leen el caché. El subagente escribió sus
propias pruebas de equivalencia **fuera** del worktree: 423 relojes para el
histograma (filas, rótulos, media, máxima, estado vacío y contador de
salidas) y 109 para los intervalos, `deep-equal` contra las fórmulas
viejas. Un hallazgo que vale la pena: la suma de la media de las esperas
sale del caché en **orden de tiempo** y la vieja salía en el orden de la
lista; con una traza en orden de tiempo —como son las reales— son el
mismo número, y sólo diferirían en el último ULP.

L12 (2026-11-06, T6, orquestador): la tarjeta de los recursos deja de
calcular un segundo juego de las cifras de las entidades: recibe las que
la sección ya calculó. De paso las cifras conservan la identidad que les
corresponde, así que una tarjeta que no cambió no se repinta.

L13 (2026-11-06, T7, orquestador, la prueba visual): la app real, con el
modelo del monitor, la corrida en el medio y la pestaña de gráficos:
**17 paths dibujados, 19 643 puntos**, las tres tarjetas que leen el caché
presentes y con cifras, y el selector de refresco funcionando igual con
el caché (la nota del panel sigue diciendo el ritmo). La equivalencia de
lo dibujado no se prueba con una captura —la semilla es distinta en cada
corrida— sino con las pruebas del caché y con el script, que comparan los
dos caminos reloj por reloj.

L14 (2026-11-06, orquestador, el número real): la medición con el índice
ya cableado da, con la máquina cargada, **0,60–0,89 ms** por pintado para
la reconstrucción y **0,035–0,064 ms** con el caché (**9–18×**); sobre una
máquina tranquila el script del repo mide **0,258 ms → 0,014 ms**. El
primer pintado, que construye la serie entera, cuesta ~0,57 ms y se paga
una vez por sesión.

**Lo que esto NO arregla, y hay que decirlo claro**: la reconstrucción de
la serie era el 1,5–4,5 % de un núcleo a 60 pintados/s, no el costo
dominante del pintado. Lo que sigue pesando es el **dibujado** —recharts
trazando un path con un punto por cambio real—, que nadie midió todavía y
que la decimación rompería (el contrato de la tarjeta: "cada escalón es un
cambio real, no una muestra"). Para esa mitad, el selector "Refresco" que
ya existe es la palanca.

L15 (2026-11-06, cierre): la suite entera **424/424** — los 407 tests que
no levantan un servidor (23 s) y los tres que sí lo levantan, de a uno
(8 + 4 + 5) —, `tsc --noEmit` limpio y `npm run build` OK. La prueba
visual en la app real quedó registrada en L13. Los commits de la rama, en
el orden de la cadena de entrega:

| commit | unidad | PR |
|---|---|---|
| `b8345df` | el ritmo del pintado y la imagen del canvas | 1 |
| `18822ce` | el caché de las series y el índice que lo lleva | 2 |
| `9d2f796` | los gráficos leen el caché | 3 |
| `9666000` | el script que mide el costo de un pintado | 4 |
| `e6b239a` | este documento | — |

La entrega es una cadena de PRs sobre la rama de la feature
(`chain_strategy=feature-branch-chain`), uno por unidad de trabajo. Ni el
push ni la apertura de los PRs están decididos: eso es del usuario.
