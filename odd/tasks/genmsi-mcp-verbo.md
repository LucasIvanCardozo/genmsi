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
| T1 | `genmsi-format`: `EditCommand`, `PlacedId` y `Placement` derivan `JsonSchema` detrás de la feature `schemars` | S3a | inline | `f534b7a` |
| T2 | `genmsi-mcp`: la herramienta `apply_edit` (entrada, colocación `auto`/`explicit`, rollback atómico, check-before-write) y el documento `genmsi.edit/1` | S2, S3, S4, S6, S7 | inline | `f534b7a` (el documento), `228cdaf` (la herramienta) |
| T3 | `genmsi-mcp`: la clase de error `invalid_command` con el índice y el `op` del comando que falló | S3b, S5 | inline | `228cdaf` |
| T4 | tests: unitarios (rollback, colocación, no-escribe, inline) y de protocolo (el agente construye un modelo de punta a punta) | S2, S3 | subagente (worker) | `5344fa5` |
| T5 | `docs/mcp.md`: la herramienta, el documento, la clase de error, la regla de colocación; se saca la sección "Not in this phase" (ya vencida por D15) | S2, S5 | inline | `7ad9493` |
| T6 | `docs/DECISIONS.md`: D21 (el verbo del MCP: un lote, una gramática, atómico, escribe) | S3 | inline | `7ad9493` |
| T7 | close: `cargo test --workspace` verde, `clippy --workspace --all-targets -D warnings` limpio | — | inline | el árbol |
| T8 | medición: agente en frío por MCP, con el criterio de salida de S8 | S8 | subagente (worker) | — |

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

L10 (2026-10-31, verificación del escritor, por spec):

- **S2**: la herramienta existe (`crates/genmsi-mcp/src/server.rs`, `apply_edit`)
  y el servidor publica **ocho** herramientas (el test que lo fija:
  `tests/tools.rs`, `the_server_offers_the_eight_tools`). Comprobado a mano por
  el protocolo: un lote que agrega dos bloques, los conecta y pone una
  propiedad deja el archivo escrito con el enlace dentro.
- **S3a**: el esquema que publica el servidor sale del tipo del motor: el
  `tools/list` trae `$defs/EditCommand` con las doce variantes (cada una con su
  `op` como `const`), y `PlacedId` y `Placement` al lado. No hay una copia del
  vocabulario en el MCP.
- **S3b**: un lote con un `connect` desde un bloque inexistente devuelve
  `invalid_command` con `command: 4` y `op: "connect"`, y el archivo queda byte
  a byte como estaba. El test del worker lo fija y además comprueba que el
  lote siguiente no ve el bloque del lote fallido.
- **S3c**: un lote que deja el modelo sin `Terminate` devuelve el error del
  motor (`invalid_model`, "no TERMINATE block with operand A") y no escribe.
- **S7**: con `auto`, el bloque nuevo cae en el lugar libre de su contenedor
  (80,80 y luego 80,183: la misma columna, más abajo); con `explicit`, cae en
  las coordenadas del comando.

L11 (2026-10-31, cierre técnico): `cargo test --workspace` → **615 tests
verdes, 0 fallos**; `cargo clippy --workspace --all-targets -- -D warnings` →
limpio.

L12 (2026-10-31, defecto preexisting, no de esta feature): el clippy del
workspace no estaba limpio en `main`: `crates/genmsi-core/examples/repro_app_model.rs`
(un `mut` de sobra) y `crates/genmsi-core/examples/repro_bypass.rs` (un import
sin usar), ambos del commit `563db27`. Arreglados aparte (`ea99a3a`), porque
bloqueaban la puerta de verificación.

L13 (2026-10-31, corte de la entrega): la feature se entrega en **cinco
commits** sobre `feature/mcp-apply-edit` (base `main`), cada uno revisable:
`ea99a3a` (los dos lints preexistentes), `f534b7a` (el esquema y el
documento), `228cdaf` (la herramienta), `5344fa5` (los tests), `7ad9493` (la
documentación). El corte de la entrega está en el commit de la
documentación; la medición (T8) cierra la feature.

L14 (2026-10-31, `ToolError` y el tamaño del `Result`): el cuerpo del error
creció (los campos `command` y `op`) y pasó los 128 bytes que clippy tolera en
un `Result`. La respuesta: `Box<ToolError>` en los `Result` del crate
(`errors::Failure`). El **cuerpo publicado no cambia**: el JSON que el agente
lee es el mismo.

L15 (2026-10-31, verificación diferencial): el `ToolError` es código
compartido por las siete herramientas que ya existían, así que se corrió la
comparación contra el código de antes: el mismo servidor, la misma llamada,
la base (`main`, worktree `genmsi-base`) y la rama, **18 casos** (las siete
herramientas con entradas que fallan, entradas ambiguas, clases que no
existen, archivos que no están, un modelo que no compila): **0 diferencias**.
Los cuerpos de error de las herramientas viejas son byte a byte los mismos:
los campos nuevos se omiten cuando no aplican, como los demás.

L16 (2026-10-31, los otros clientes): el tipo TypeScript del documento nuevo
se regeneró (`crates/genmsi-app/scripts/generate-types.sh`: un archivo nuevo,
`EditDocument.ts`, sin ruido en los demás), `tsc --noEmit` limpio y los 381
tests de la app verdes. La CLI no cambió de comportamiento: sus subcomandos
usan el mismo núcleo y el mismo formato.

L17 (2026-10-31, T8 — la medición, `odd/tasks/medida3/`): un agente en frío
construyó el modelo del comedor (35 bloques, 25 enlaces, 64 propiedades)
**íntegramente con el verbo**, sin escribir XML a mano: **12 llamadas al
MCP contra las 24** de la corrida anterior, con **0 sondas** contra 9. El
modelo se verificó con los mismos números (seed 42): 953 transacciones,
`end_time` 54000, `terminated`, y los agregados del comedor idénticos a los
de `medida2` (cola 476/47, horno 12/12/475/85479, savevalues 475/469, tabla
476/1200/10). La única diferencia es el número de eventos (14045 contra
13967): un recorrido levemente distinto en un punto del flujo, sin efecto
sobre las magnitudes.

L18 (2026-10-31, lo que encontró la medición): seis cosas. **D1** era un
defecto del verbo —no había forma sancionada de crear un modelo nuevo: una
ruta que no existe daba `model_not_found` y `save_model` rechaza el
esqueleto vacío— y está **arreglada** (`5473298`): una ruta que no está en
el disco es un modelo que el lote crea, partiendo del esqueleto. **D2** y
**D3** eran la guía: no decía la forma de cada comando (`setCaption` usa
`caption`, no `value`) ni la regla de los ids (los bloques de un lote toman
los ids libres siguientes, en el orden de los comandos); las dos están en la
guía y en la descripción de la herramienta. **D5** (la promesa del lint
sobre `Tamano`/`Tamaño`) quedó **verificada** en esta sesión: el lint lo
dice, con el nombre que probablemente se quiso ("did you mean 'Tamaño'?").

L19 (2026-10-31, T9 — la revisión): el assess con el ítem 1 escalado (la
herramienta escribe en disco) devolvió `risk: high` y un plan de dos
piezas: auto-verificación del escritor (hecha, L10 y L15) y verificador
independiente (en curso al cierre de este corte). La verificación
diferencial contra `main` (18 casos de las siete herramientas viejas) dio
**0 diferencias**.
