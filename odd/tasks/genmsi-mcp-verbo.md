# Feature: el verbo del MCP — `apply_edit`

El MCP de GenMSI expone siete herramientas y cinco leen. Ninguna **añade, edita ni
conecta** bloques: el verbo que construye un modelo no existe en el protocolo,
aunque el motor ya lo tiene escrito, probado y en producción en la app de
escritorio. Esta feature enchufa ese verbo en el MCP.

## Specs

S1 (pedido del usuario, textual): "hoy quiero quie nos enfoquemos en la mejora de
nuestro mcp para que los agentes de ia puedan comunicarse con nuestra app
correctamente".

S2 (pedido del usuario, textual): "osea el mcp no expone una herramienta para
añadir, editar o conectar bloques? porque eso justamente es lo mas importante".

S3 (decisión del usuario, textual): "perfecto! vamos con tu recomendacion A, con la
politica atomica y con tu voto de escribe."

S4 (D7, ROADMAP Phase 3): "`genmsi mcp` runs an rmcp stdio server with tools:
`simulate`, `lint`, `get_trace`, `explain_model`. The tool surface is a thin
adapter over the same API the CLI uses."

S5 (D13): "within one major version a change is additive only [...] and a
breaking change bumps the number."

S6 (D19): "One zone of truth: the engine publishes, the app invents nothing."

S7 (D18): "The engine composes the canvas."

S8 (D11): "Goal is minimum parity with the original, with better code quality —
not a maximally professional system. Every tool addition must justify itself
against that goal."

### S2 — el verbo

El MCP expone una herramienta que **añade, edita y conecta** bloques de un
modelo. El verbo es el del motor (`genmsi_format::EditCommand`), no uno nuevo.

### S3a — una sola gramática (opción A del análisis)

Los comandos que la herramienta recibe son `EditCommand`, el mismo tipo que la app
de escritorio ya habla por cable (`crates/genmsi-app/src/engine/adapter.ts:121`,
`apply(command: EditCommand)`) y que el motor ya serializa con
`#[serde(tag = "op", rename_all = "camelCase")]`. El MCP **no** inventa
`add_block`/`connect`/`set_property`: un segundo dialecto sobre el mismo modelo
rompería D19 y obligaría a traducir.

El esquema JSON de la herramienta se **deriva del tipo del motor**: `EditCommand`
gana `JsonSchema` (detrás de una feature `schemars` de `genmsi-format`, como la
feature `ts` que ya existe). Una sola definición del vocabulario, en un solo
lugar.

### S3b — política atómica

Un lote de comandos es **todo o nada**. Si un comando falla, el modelo queda
exactamente como estaba: los comandos ya aplicados se deshacen (`EditSession::undo`)
y la herramienta devuelve el error diciendo **qué comando** falló (su índice y su
`op`). El agente reintenta con el lote corregido y el estado es el mismo que
antes del intento.

### S3c — escribe

La herramienta **escribe** el modelo en el `path` que recibió. El modelo se
**valida antes de escribir** (igual que `save_model`): un modelo que no compila
**no se escribe** y el error es el del compilador, con línea, columna, nodo y
clase.

El lote es rejugable: los comandos son la fuente de verdad, no el texto. Un
lote que no compila se corrige y se vuelve a enviar completo contra el mismo
`path`, cuyo contenido sigue intacto. El agente nunca pierde trabajo.

### S4 — el verbo es un adaptador delgado

La herramienta no lleva lógica de simulación (D2) ni de formato: llama al
`EditSession` del motor y publica un documento. El documento nuevo
(`genmsi.edit/1`) lo construye el núcleo, como los demás (`genmsi-core`).

### S5 — aditivo

La herramienta nueva es aditiva. La clase de error nueva (`invalid_command`) se
suma al conjunto publicado; ninguna cambia. El documento nuevo
(`genmsi.edit/1`) es propio del MCP, como `genmsi.classlist/1` o
`genmsi.openmodel/1`.

### S6 — la geometría es del motor

El MCP es un cliente, y un cliente decide **dónde cae** un bloque (la app lo
hace en TypeScript: `freeSpotOf`, `crates/genmsi-app/src/state/store.ts:1055`).
La herramienta hace lo mismo con la primitiva hermana del motor
(`genmsi_format::layout::free_spot_of_scope`), que da el mismo lugar que la app.

La herramienta no compone el lienzo por su cuenta: usa la composición que el
motor ya publica cuando el modelo no trae geometría (D18, la misma regla de
`save_model`).

### S7 — la colocación

La herramienta acepta `layout`:

- `"auto"` (por defecto): cada bloque que el lote añade se coloca en el lugar
  libre de su contenedor (como una caída en la app). Las coordenadas del
  comando `addBlock` se ignoran.
- `"explicit"`: las coordenadas del comando se usan tal cual.

### S8 — el tamaño

Una herramienta, un documento, una clase de error. La medición con agente frío
es el criterio de salida: un modelo de 9 bloques construido **por el MCP**, sin
leer el repositorio y sin escribir XML a mano.

## Tasks

| ID | Task | Specs | Route | Commit |
|---|---|---|---|---|
| T1 | `genmsi-format`: `EditCommand`, `PlacedId` y `Placement` derivan `JsonSchema` detrás de la feature `schemars` | S3a | inline | 1 |
| T2 | `genmsi-mcp`: la herramienta `apply_edit` (entrada, colocación `auto`/`explicit`, rollback atómico, check-before-write) y el documento `genmsi.edit/1` | S2, S3, S4, S6, S7 | inline | 1 |
| T3 | `genmsi-mcp`: la clase de error `invalid_command` con el índice y el `op` del comando que falló | S3b, S5 | inline | 1 |
| T4 | tests: unitarios (rollback, colocación, no-escribe, inline) y de protocolo (el agente construye un modelo de punta a punta) | S2, S3 | inline | 1 y 2 |
| T5 | `docs/mcp.md`: la herramienta, el documento, la clase de error, la regla de colocación; se saca la sección "Not in this phase" (ya vencida por D15) | S2, S5 | inline | 2 |
| T6 | `docs/DECISIONS.md`: D21 (el verbo del MCP: un lote, una gramática, atómico, escribe) | S3 | inline | 2 |
| T7 | close: `cargo test --workspace` verde, `clippy --workspace --all-targets -D warnings` limpio | — | inline | 2 |
| T8 | medición: agente en frío por MCP, con el criterio de salida de S8 | S8 | subagente (background) | 3 |

## Log

L1 (2026-10-31, usuario, textual): "buenas! en estos ultimos dias avanzamos mucho en
temas de optimizaciones y mejoras de codigo de este proyecto. hoy quiero quie nos
enfoquemos en la mejora de nuestro mcp para que los agentes de ia puedan
comunicarse con nuestra app correctamente. Quiero que analices nuestra situacion
actual con respecto al mcp y analicemos si merece la pena un refactor o mejora y
con justificacion"

L2 (2026-10-31, usuario, textual): "osea el mcp no expone una herramienta para
añadir, editar o conectar bloques? porque eso justamente es lo mas importante"

L3 (2026-10-31, usuario, textual): "perfecto! vamos con tu recomendacion A, con la
politica atomica y con tu voto de escribe."

L4 (2026-10-31, análisis, decisión del orquestador): el refactor interno del crate
MCP **no** se hace (1.161 líneas, protocolo cubierto por tests de cliente real,
ratio test/src sano, sin lógica de dominio). Lo que se hace es cerrar el verbo.
El análisis no se reescribe: se documenta en `docs/mcp.md` y en D21.

L5 (2026-10-31, análisis, evidencia): el motor ya tiene el verbo
(`crates/genmsi-format/src/edit.rs`, `EditCommand`, 12 variantes, 60 tests), la
app ya lo usa en producción (`crates/genmsi-app/engine/src/session.rs:200`) y ya
lo habla por cable (`crates/genmsi-app/src/engine/adapter.ts:121`). El MCP es el
único cliente que no puede mutar.

L6 (2026-10-31, análisis, hueco confirmado): ninguna herramienta devuelve el texto
crudo del `.gmdl`; `explain_model` publica el documento **estructurado**
(`genmsi.model/1`) y `save_model` exige el documento **completo** como texto. El
ciclo "leer → modificar → guardar" está roto para un agente que sólo tiene MCP.
`apply_edit` lo cierra: con un `path`, el archivo es el estado; con un modelo
inline, la respuesta lleva el texto.

L7 (2026-10-31, análisis, decisión del orquestador): la colocación por defecto es
`"auto"` (el lugar libre del contenedor, la misma regla que la app), porque un
agente no tiene el lienzo en la cabeza y `addBlock` con coordenadas inventadas
 produce un lienzo apilado e ilegible. `"explicit"` existe para cuando el agente
sí quiere un lugar.

L8 (2026-10-31, análisis, decisión del orquestador): el modelo que no compila
**no** se escribe (S3c, el voto del usuario), pero el lote es rejugable: el agente
corrige y reenvía. Por eso el error de compilación no necesita llevar el texto
del modelo.

L9 (2026-10-31, análisis, decisión del orquestador): la feature es **aditiva**.
El conjunto de clases de error publicadas crece con `invalid_command`; ninguna
cambia. `model_unreadable` (nunca emitida) se deja como está: retirarla sería un
cambio de contrato y no es el trabajo de esta feature.
