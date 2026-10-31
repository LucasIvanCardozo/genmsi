# Encargo común: auditoría de una tarjeta de la sección Gráficos

Sos un subagente de la auditoría de la sección **Gráficos** del monitor de
GenMSI (app de escritorio Tauri + React + TypeScript). Tu tarea es **una sola**
tarjeta, con foco exclusivo en ella. No audites otras tarjetas: cada una tiene
su propio subagente.

Todo es **sólo lectura**: no escribas ni edites archivos, no corras comandos que
cambien el repo. Devolvé tu sección como texto en la respuesta final.

## Dónde está cada cosa

- Raíz del repo: `/home/lucas/Documentos/Workspace/Genmsi` (contenido idéntico
  al worktree `/home/lucas/Documentos/Workspace/Genmsi-worktrees/auditoria-graficos`).
- Frontend de la app: `crates/genmsi-app/src/`.
- El panel que ensambla las tarjetas: `crates/genmsi-app/src/components/MonitorCharts.tsx`.
- Una tarjeta = un componente en `crates/genmsi-app/src/components/charts/`.
- El motor (Rust) y el documento del reporte: `crates/genmsi-core/src/report.rs`;
  su forma en la app: `crates/genmsi-app/src/types/generated/JsonReport.ts`.
- El replay: `crates/genmsi-app/src/monitor/replay.ts` (la traza proyectada),
  `crates/genmsi-app/src/monitor/stats.ts`, `crates/genmsi-app/src/monitor/derived.ts`
  (las métricas derivadas), `crates/genmsi-app/src/monitor/series.ts` (series,
  tramos e histogramas).
- El diseño y las reglas de la sección: `odd/tasks/genmsi-graficos-estadisticas.md`
  (la feature anterior, con las decisiones del usuario) y `docs/app.md`, sección
  *"The charts read the documents"*.

## Las reglas que la sección ya tiene (son el contrato)

1. **El reloj manda.** El reporte describe la corrida completa; los gráficos
   describen el **instante** que marca el reloj de la reproducción. Un valor
   "vivo" (del instante) nunca puede presentarse como publicado, ni al revés.
   Cada juego de cifras se divide por **su** tiempo: el del reloj para las
   cifras del instante, la duración de la corrida para las publicadas.
2. **Sólo ratios del motor.** La app no recalcula nada que el motor no diga.
   Lo derivado (área/tiempo, tiempo total/entradas, tiempo ocupado/tiempo) va
   marcado con asterisco y la tarjeta dice la fórmula.
3. **El eje del tiempo arranca en 0** y termina en el final de la corrida,
   aunque el primer dato llegue más tarde; la línea vertical marca el reloj.
4. **La serie es la historia exacta** de la entidad: los escalones son los
   instantes en que su valor cambia, no una muestra de la traza.
5. Cada tarjeta dice de dónde salen sus números (el badge) y cómo se hicieron
   (el pie).

## Las tres preguntas

1. **¿Tiene sentido?** — ¿la información aporta para entender una corrida de
   simulación por eventos discretos? ¿Es redundante con otra tarjeta?
2. **¿Se muestra correctamente?** — comprobá el cálculo contra los campos que
   publica el motor y contra el código del replay. Buscá: denominadores
   equivocados, mezcla de cifras vivas y publicadas, valores que no se cortan
   en el reloj, estados vacíos que no se contemplan, ejes que engañan,
   rótulos que no dicen lo que muestran, métricas derivadas sin marcar,
   números recalculados en vez de leídos, claves de React que se rompen con
   nombres repetidos, atributos que chocan con los del SVG.
3. **¿Se podría mostrar la misma información de otra manera?** — una forma más
   legible o más útil, con la misma información o con menos.

## Lo que hay que mirar también

- Los tests del frontend: `crates/genmsi-app/tests/` (`entities.test.ts`,
  `series.test.ts`, `replay.test.ts`). Decí si tu tarjeta está cubierta y cómo.
- Lo que la app **no** muestra del reporte: el motor publica más campos de los
  que la sección usa. Si tu tarjeta podría enseñar algo que el motor ya
  publica y la app ignora, decilo.

## Formato de la respuesta

Markdown, en **español** (el informe lo lee una persona hispanohablante; los
identificadores, los nombres de archivo y los términos técnicos van en inglés).
Máximo ~700 palabras. Estructura:

```markdown
## <Título de la tarjeta>

**Qué muestra.** <qué datos, de qué fuente, qué campos>  (fuente: `path:line`)

**1. ¿Tiene sentido?** <juicio + por qué>

**2. ¿Se muestra correctamente.** <juicio + los hallazgos>
- [alta|media|baja] <hallazgo> — `path:line`

**3. ¿Otra forma?** <alternativa concreta, con una línea de justificación>

**Veredicto.** KEEP | KEEP WITH CHANGES | REDUNDANT | REMOVE
**Riesgo de equivocarse.** <qué se rompe si este gráfico miente>
```

Reglas: **toda** afirmación lleva evidencia `path:line`. Si no pudiste
verificar algo, decilo explícitamente como "no verificado". No inventes
comportamiento: si no lo leíste en el código, no lo afirmes.
