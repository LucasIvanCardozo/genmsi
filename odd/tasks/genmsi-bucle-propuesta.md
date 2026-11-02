# Feature: el bucle de la propuesta — el agente propone, la persona decide

La app y el MCP ya se hablan en un sentido: la app publica lo que está
mostrando (`genmsi.live/1`) y el verbo se niega a escribir sobre un modelo
que la persona tiene abierto con cambios sin guardar (`model_locked`). Pero el
sentido inverso sigue siendo ralo: la **única** forma de que el agente le
muestre su trabajo a la persona es **sobrescribir el archivo**, y esa vía está
cerrada justo cuando la persona está trabajando.

La sesión del 2 de noviembre lo dejó claro: el agente construyó 517 bloques
en `/tmp` durante dos horas, con la app abierta mostrando un modelo de prueba
de 5 bloques. La persona no vio nada, y el modelo terminó sin revisar.

Esta feature abre el otro sentido: el agente **propone** (un modelo que no
toca el disco), la app lo **adopta** en memoria (sucio y deshacible, visible
en el lienzo), la persona **decide** (aceptar o descartar, con una nota) y el
agente **conoce el veredicto**. Y el `path` por defecto del MCP pasa a ser el
modelo que la persona tiene abierto, para que el agente trabaje sobre lo que
la persona está mirando en vez de adivinar un destino.

## Specs

S1 (pedido del usuario, textual): "tenes que resolver el ejercicio en el
archivo que tengo abierto en genmsi como a vos te parezca".

S2 (pedido del usuario, textual): "y pudiendo ver en tiempo real como se va
creando todo lo que hace? porque eso lo considero realmente importante que sea
asi".

S3 (pedido del usuario, textual): "no se puede de alguna manera hacerle saber
al usuario o que el agente le haga saber al usuario que si tiene un modelo
abierto en la app que es el que va a editar el agente pero tiene cambios no
guardados le avise antes de tocar nada?".

S4 (decisión del usuario, textual, hoy): "Frente A: el bucle (path por defecto
+ propuesta)" y "Una feature ODD con varios frentes".

S5 (D19): "One zone of truth: the engine publishes, the app invents nothing."

S6 (D2): "only `genmsi-format` and `genmsi-core` may be dependencies of more
than one crate. CLI, MCP and GUI are thin layers with no simulation logic."

S7 (D13): "within one major version a change is additive only".

S8 (D11): "Every tool addition must justify itself against that goal."

### S1 — el agente trabaja sobre lo que la persona tiene abierto

El MCP resuelve el `path` que falta: si el agente pide un modelo y **no**
nombra un archivo, el servidor usa **el modelo que la app tiene abierto**
(`genmsi_core::read_open_model()`, el documento `genmsi.openmodel/1`). El
agente deja de adivinar un destino, y la acción natural —editar— es editar
lo que la persona está mirando.

Vale para `lint`, `simulate`, `get_trace`, `explain_model` y `apply_edit`
(todas las herramientas que hoy exigen `path` o `model`). Para `save_model`,
`path` deja de ser obligatorio y pasa a significar lo mismo: el modelo que se
va a escribir.

Cuando no hay app corriendo, o la app no tiene ningún modelo abierto, la
herramienta **no adivina**: falla con la clase de error nueva
`no_open_model`, que dice qué hacer ("give `path`, or open a model in the
app"). Es una respuesta, no un fallo mudo.

`path` y `model` juntos siguen siendo un error (`invalid_arguments`): el
documento en texto y el archivo son dos fuentes, y el agente que da las dos
tiene que decir cuál manda.

### S2 — el trabajo del agente llega a la vista de la persona

El agente propone un modelo **sin tocar el disco**. La propuesta viaja por un
directorio de destino propio de la app — `<data>/staging/` — con un documento
por propuesta:

- `<id>.gmdl`: el modelo propuesto;
- `<id>.json`: el documento `genmsi.proposal/1`.

`genmsi.proposal/1` lleva:

| Campo | Tipo | Qué es |
|---|---|---|
| `id` | string | La identidad de la propuesta. |
| `model` | string | La ruta absoluta del modelo propuesto en el directorio de destino. |
| `target` | string o null | El modelo que la propuesta propone cambiar (el que la app tiene abierto). |
| `summary` | string o null | Lo que el agente dice que hizo, en una línea. |
| `state` | string | `pending`, `accepted` o `rejected`. |
| `note` | string o null | Lo que la persona responde al descartar. |

La app mira el directorio de destino y **adopta** la propuesta pendiente más
antigua: carga el modelo en memoria y lo deja **sucio** (difiere del disco) y
**deshacible** (un paso de deshacer devuelve el modelo anterior). La persona
lo ve en el lienzo, con el monitor corriendo, sin que se haya escrito nada.

Adoptar no es escribir: el archivo del disco sigue con el modelo anterior
hasta que la persona acepte.

### S3 — la persona decide, y el agente se entera

En el pie de la ventana (la banda de `StatusLine`, el mismo lugar donde vive
el aviso de "Guardar / Cancelar" de un modelo con errores) aparece el aviso
de la propuesta: qué propuso el agente, y dos botones, **Aceptar** y
**Descartar**.

- **Aceptar** es el guardado normal de la app: el modelo en memoria se
  escribe en el archivo que la app tiene abierto, y la propuesta queda
  `accepted`.
- **Descartar** es el deshacer normal de la app: el modelo anterior vuelve al
  lienzo, y la propuesta queda `rejected` con la nota que la persona escriba
  (opcional).

El veredicto viaja en el documento vivo: `genmsi.live/1` gana el campo
`proposal` (`id`, `state`, `summary`, `note`), de modo que `app_state` le
dice al agente qué pasó con su propuesta sin que el agente tenga que
preguntar otra vez.

Si la persona tiene **cambios sin guardar**, la app **no** adopta la
propuesta: la deja `pending` y el aviso dice que hay una propuesta esperando.
La regla es la misma del freno: el trabajo de la persona no se pisa. Cuando
la persona guarda, la propuesta se adopta.

### S4 — el freno sigue siendo la ley

Adoptar la propuesta deja la sesión sucia, así que el freno `model_locked`
sigue mirando lo que mira: un modelo con cambios sin guardar no se escribe.
La propuesta es un camino **alternativo** al de escribir el archivo, no un
 bypassing del freno: el motor del freno no cambia.

### S5 — un solo dueño del formato

El documento `genmsi.proposal/1` y el directorio de destino viven en
`genmsi-core` (`crates/genmsi-core/src/proposal.rs`), junto a
`open_model.rs` y `live.rs`. El MCP y la app llaman a las mismas funciones
del núcleo: dos clientes, un formato, un escritor. La app y el MCP son
escritores de **estados distintos** del mismo documento (el MCP lo crea
`pending`; la app lo pasa a `accepted` o `rejected`), y ninguno reescribe el
documento entero por su cuenta.

### S6 — el MCP sigue siendo un adaptador fino

La propuesta no mete lógica de simulación en el MCP: el MCP sigue siendo un
adaptador sobre el motor y sobre las mismas funciones que la CLI.

### S7 — documentos versionados, aditivos dentro de un major

- `genmsi.proposal/1` es un documento nuevo (aditivo).
- `genmsi.live/1` gana el campo `proposal` (aditivo: un campo nuevo puede
  aparecer).
- `genmsi.edit/1` gana el campo `proposal` (aditivo).
- `genmsi.openmodel/1` no cambia.

**El MCP no gana ninguna herramienta nueva.** El conjunto de las nueve
herramientas queda como está, y el test que lo fija (`crates/genmsi-mcp/tests/tools.rs`)
no se toca. Lo que se agrega son **campos opcionales**: `apply_edit` y
`save_model` aceptan `target`, y `save_model` acepta `path` ausente.

### S8 — cada campo nuevo se justifica

| Cambio | Justificación |
|---|---|
| `path` por defecto | S1: el agente deja de adivinar el destino. |
| clase `no_open_model` | S1: sin app no hay destino, y el agente tiene que saberlo. |
| `target: "proposal"` | S2/S3: la única forma de que la persona vea el trabajo del agente sin que se escriba nada. |
| `genmsi.edit/1` gana `proposal` | S3: el agente necesita el id de lo que propuso. |
| `genmsi.live/1` gana `proposal` | S3: el veredicto tiene que volver al agente. |

## Tasks

| ID | Task | Specs | Route | Commit |
|---|---|---|---|---|
| T1 | `genmsi-core`: el documento `genmsi.proposal/1`, el directorio de destino y su lectura/escritura atómica; `LiveBody` gana `proposal`; `genmsi-format`: `EditSession::adopted` (un modelo que entra sucio y deshacible) | S2, S3, S5, S7 | inline | 1 |
| T2 | `genmsi-mcp`: `path` por defecto (el modelo abierto) con la clase `no_open_model`; `target` en `apply_edit` y `save_model`; `genmsi.edit/1` gana `proposal` | S1, S2, S4, S6, S7, S8 | inline | 2 |
| T3 | `genmsi-app` (motor y shell): la sesión adopta la propuesta (sucia, deshacible), el shell la lee del directorio de destino y resuelve el veredicto, `live.json` lo publica | S2, S3, S4, S5 | worker | 3 |
| T4 | `genmsi-app` (frente): el aviso de la propuesta con Aceptar/Descartar y la nota, y el store que vigila el directorio de destino | S2, S3 | worker | 4 |
| T5 | `skill/genmsi-modeling/SKILL.md` (el procedimiento), `docs/mcp.md`, `docs/app.md`, `docs/DECISIONS.md` (D23) | S1, S2, S3, S8 | inline | 5 |
| T6 | close: `cargo test --workspace` verde, `clippy --workspace --all-targets -- -D warnings` limpio, los tests del frente verdes, y la medición con un agente en frío (propone, la persona ve, el agente lee el veredicto) | — | inline | 5 |

## Log

L1 (2026-11-02, usuario, textual): "Bien mira, tengo una sesion activa
actualmente creo llamada chat, en la cual podras ver todo el flujo que realizo
para hacer un trabajo con genmsi y una devolucion final sobre nuestro mcp.
leela y decime que pensas".

L2 (2026-11-02, usuario, textual): "decime si te acordas asi te digo que cositas
pasaron para que podamos pensar como mejorar el flujo entre el agente y la app
mediante nuestro mcp".

L3 (2026-11-02, decisión del usuario, textual): "Frente A: el bucle (path por
defecto + propuesta)" y "Una feature ODD con varios frentes".

L4 (2026-11-02, evidencia, la sesión `chat`): el agente pidió "crear mi modelo
en genmsi, actualmente tengo una rchivo abierto" y "resolver el ejercicio en
el archivo que tengo abierto en genmsi como a vos te parezca". Eligió trabajar
en `/tmp/tp9_problema1.gmdl` (517 bloques) durante dos horas, con la app
abierta mostrando un modelo de prueba de 5 bloques: el bucle no se entregó.

L5 (2026-11-02, análisis, el hueco): el canal app→agente es rico (`live.json`:
modelo abierto, `dirty`, diagnósticos, modelo en memoria, última corrida) y el
canal agente→app es pobre: la única forma de que la persona vea el trabajo
del agente es sobrescribir el archivo, vía que el freno `model_locked` corta
justo cuando la persona está trabajando. No hay forma de **proponer**.

L6 (2026-11-02, análisis, la evidencia del código): el `EditCommand` tiene
doce variantes y el MCP publica el esquema del tipo del motor, así que
`movePort`, `removePort`, `setLayouts` y `paste` ya están disponibles; lo que
faltaba era que la **descripción** de la herramienta sólo mencionara ocho. Y
las etiquetas de cadena que el agente necesitaba para el `Unlink` sí se
publican en `explain_model` (`ModelPort.label`, `ModelEdge.label`). Son
huecos de descubribilidad, no de capacidad: la feature los anota en el Log
como trabajo futuro, no los mete en esta feature.

L7 (2026-11-02, decisión del orquestador): el MCP **no** gana ninguna
herramienta nueva. Un campo opcional (`target`) hace todo el trabajo y deja
intacto el conjunto de nueve herramientas que el test `tests/tools.rs` fija
(S8: cada adición se justifica).

L8 (2026-11-02, decisión del orquestador): adoptar una propuesta es
**cargar** un modelo en memoria, no escribir un archivo. Por eso el modelo
adoptado entra **sucio** (difiere del disco) y **deshacible** (un paso de
deshacer): hace falta `EditSession::adopted(model, checkpoint)`, porque
`EditSession::new` deja el checkpoint en el modelo mismo y la sesión
quedaría limpia.

L9 (2026-11-02, decisión del orquestador): la app **no** adopta una propuesta
si la persona tiene cambios sin guardar: la deja `pending` y lo dice. Es la
misma regla del freno, del otro lado.

L10 (2026-11-02, decisión del orquestador): el aviso de la propuesta vive en
la banda de `StatusLine`, que es donde la app ya pone el aviso de "Guardar /
Cancelar" y donde la propia app dice que debe vivir su banda ("the status line
is the only band of the app").

L11 (2026-11-02, decisión del orquestador): no hay comprobación de "obsoleta"
en esta versión. Si el archivo cambió después de que la propuesta se hizo, la
persona lo ve en el lienzo antes de aceptar y es quien juzga. La huella se
deja para una versión posterior.

L12 (2026-11-02, análisis, el mapa del lado app): el vigilante del archivo
está en el **frente** (`crates/genmsi-app/src/state/store.ts:752-753`, un
intervalo de 500 ms que llama `checkModelFile`), no en Rust; el aviso se
muestra por `StatusLine`; las operaciones de la sesión viajan por un solo
comando Tauri (`session`) con un protocolo de ops (`engine/src/protocol.rs`);
y `Session::reload` cambia la ruta del modelo, así que adoptar una
propuesta necesita un método nuevo que **no** la cambie.
