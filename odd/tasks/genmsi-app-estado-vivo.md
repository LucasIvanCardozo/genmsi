# Feature: el estado vivo de la app — `genmsi.live/1` y el freno del verbo

La app y el MCP ya se habla: el agente escribe el archivo que la app tiene
abierto, la app lo ve en ≤500 ms y recarga. Lo que **no** existe es el otro
sentido: el agente no ve lo que la app está haciendo, y —peor— puede escribir
sobre un modelo que la persona tiene abierto **con cambios sin guardar**:
la app no recarga (protege el trabajo de la persona), el trabajo del agente
queda invisible en el lienzo y, cuando la persona guarda, lo pisa.

Esta feature cierra las dos cosas: la app **publica** su estado vivo, y el
verbo **se niega** a escribir sobre un modelo que la persona tiene abierto sin
guardar.

## Specs

S1 (pedido del usuario, textual): "y pudiendo ver en tiempo real como se va
creando todo lo que hace? porque eso lo considero realmente importante que sea
asi".

S2 (pedido del usuario, textual): "no se puede de alguna manera hacerle saber
al usuario o que el agente le haga saber al usuario que si tiene un modelo
abierto en la app que es el que va a editar el agente pero tiene cambios no
guardados le avise antes de tocar nada?"

S3 (decisión del usuario, textual): "me gusta todo menos la 4, esa la veo
innecesaria \"que la app diga quién cambió el archivo\"".

S4 (D19): "One zone of truth: the engine publishes, the app invents nothing."

S5 (D2): "only `genmsi-format` and `genmsi-core` may be dependencies of more
than one crate. CLI, MCP and GUI are thin layers with no simulation logic."

S6 (D13): "`--json` wraps the body in a versioned envelope, `{"schema":
"genmsi.<document>/<major>", ...}`; within one major version a change is
additive only".

S7 (D11): "Goal is minimum parity with the original, with better code quality —
not a maximally professional system. Every tool addition must justify itself
against that goal."

### S1 — el estado vivo

La app publica un documento con lo que está mostrando: el modelo que tiene
abierto, si tiene cambios sin guardar, sus hallazgos y la última corrida que
hizo (el informe y los parámetros con que la corrió). El MCP lo expone.

El documento es `genmsi.live/1` y lo publica **el núcleo** (D19), como
`genmsi.openmodel/1`: el motor publica, la app lo escribe y el MCP lo lee.

El modelo en memoria viaja **sólo cuando la persona tiene cambios sin
guardar**: si no los tiene, el archivo *es* el modelo y el agente lo lee con
las herramientas que ya tiene.

Lo que **no** viaja en esta versión (y por qué):

- **la traza**: es enorme y el agente puede pedirla cuando la quiera;
- **el frame del replay**: lo tiene el frontend, no el motor de la sesión;
- **`stale`** (el disco cambió y la app no recargó): con el freno de S2 el
  agente nunca puede escribir en ese estado, así que el dato no le sirve;
  el aviso al humano ya existe en la app.

### S2 — el freno, y el aviso

**El freno es de la herramienta, no del agente.** `apply_edit` y `save_model`
leen el estado vivo antes de escribir: si la app tiene **cambios sin guardar
sobre ese mismo modelo**, **no escriben** y contestan un error nuevo,
`model_locked`, que dice qué pasó y qué hacer.

La regla no puede depender de que el agente "se acuerde": una regla que
depende de la diligentcia del agente es una regla que un agente se saltea.

**El aviso al humano lo hace el agente**, que es quien está hablando con
ella: el procedimiento está en la guía (`skill/genmsi-modeling/SKILL.md`):
antes de escribir sobre el modelo abierto, preguntar el estado; si hay
cambios sin guardar, avisar y esperar a que la persona guarde; después,
**reenviar el mismo lote** (el lote es rejugable: los comandos son la fuente
de verdad, no el texto).

La app **ya avisa** cuando el disco cambia con cambios sin guardar ("El
modelo del disco cambió y tenés cambios sin guardar: no se recargó"): con el
freno, ese aviso ya no puede ser la consecuencia de que un agente lo pisó.

Sin la app corriendo, el estado vivo es un documento con `path: null`: **una
respuesta, nunca un fallo** (la misma regla que `open_model`).

### S3 — lo que NO se hace

La app **no** dice quién cambió el archivo. No hay `last-touch.json`, no hay
rastro, no hay atribución: el humano sabe que el agente trabaja porque el
agente se lo dice (S2), y la app no necesita un canal para saberlo.

### S4 — una zona de verdad

El documento lo publica el núcleo (`genmsi-core`), lo escribe la app (como
escribe `open-model.json`) y lo lee el MCP (como lee `open-model.json`). Ni la
app ni el MCP inventan la forma del estado: la escriben y la leen.

### S5 — capas delgadas

Ni la app ni el MCP llevan lógica de simulación ni de formato: el documento
es un dato que el núcleo publica, y el freno es una política del verbo (que
es del cliente, no del motor).

### S6 — aditivo

Un documento nuevo (`genmsi.live/1`), una herramienta nueva (`app_state`) y
una clase de error nueva (`model_locked`). Ninguna clase ni ningún documento
que exista cambia: `genmsi.lint/1`, `genmsi.edit/1` y las demás siguen igual.

### S7 — el tamaño

Una app que publica, una herramienta que lee, un freno, un procedimiento. No
hay un canal de push: el MCP es request/response, y el agente pregunta
`app_state` cuando le importa (la persona **sí** ve el trabajo del agente en
vivo, que es lo otro, y ya funciona).

## Tasks

| ID | Task | Specs | Route | Commit |
|---|---|---|---|---|
| T1 | `genmsi-core`: el documento `genmsi.live/1`, su lectura y su escritura atómica (junto a `open_model.rs`) | S1, S4, S6 | inline | 1 |
| T2 | `genmsi-app`: la sesión guarda la última corrida y el shell publica `live.json` (atómico, sólo cuando el estado cambia) | S1, S4 | worker | 2 |
| T3 | `genmsi-mcp`: la herramienta `app_state` | S1, S7 | worker | 3 |
| T4 | `genmsi-mcp`: el freno (`apply_edit` y `save_model` se niegan con `model_locked`) | S2, S6 | worker | 3 |
| T5 | `skill/genmsi-modeling/SKILL.md`: el procedimiento (preguntar, avisar, reenviar el lote) | S2 | inline | 4 |
| T6 | docs: `docs/mcp.md` (la herramienta y el freno), `docs/app.md` (la publicación), `docs/DECISIONS.md` (D22) | S1, S2, S3 | inline | 4 |
| T7 | close: `cargo test --workspace` verde, `clippy --workspace --all-targets -- -D warnings` limpio | — | inline | 4 |

## Log

L1 (2026-10-31, usuario, textual): "una unica consulta sobre el tema del mcp y
nuestra app. El agente por medio del mcp podra editar en tiempo real el archivo
que tenga abierto en la aplicacion tambien? y pudiendo ver en tiempo real como
se va creando todo lo que hace? porque eso lo considero realmente importante que
sea asi"

L2 (2026-10-31, usuario, textual): "me gustaria el nivel 2 ya que es mas
completo no te parece? justificame realmente y aclarame que hariamos"

L3 (2026-10-31, usuario, textual): "perfecto!! me gusta todo menos la 4, esa la
veo innecesaria \"que la app diga quién cambió el archivo\""

L4 (2026-10-31, análisis, evidencia): el ciclo de escritura ya funciona: la app
vigila el archivo que tiene abierto con una huella (mtime en nanosegundos +
tamaño) cada 500 ms, recarga, compila y vuelve a correr; la recarga entra en la
historia de la sesión (el humano la deshace con Ctrl+Z); y si el modelo no
compila, la app no recarga y muestra el error del compilador.

L5 (2026-10-31, análisis, el hueco): el agente no ve el estado de la app (sólo
el archivo), y puede escribir sobre un modelo con cambios sin guardar: la app
no recarga, el trabajo del agente queda invisible y la persona lo pisa al
guardar. El agente no puede ni enterarse, porque no ve el `dirty`.

L6 (2026-10-31, decisión del orquestador): el freno va en la **herramienta**, no
en la guía. Una regla que depende de que el agente se acuerde es una regla que
un agente se saltea; el error `model_locked` la vuelve mecánica.

L7 (2026-10-31, decisión del orquestador): el modelo en memoria viaja sólo
cuando hay cambios sin guardar (si no los hay, el archivo *es* el modelo), y
la traza, el frame del replay y `stale` no viajan en esta versión: el freno
los vuelve innecesarios y el primero es enorme.

L8 (2026-10-31, decisión del usuario): la atribución ("la app dice quién
cambió el archivo") **queda fuera** de la feature: es innecesaria, porque el
humano se entera por el agente.

L9 (2026-10-31, T1 — el núcleo): el documento `genmsi.live/1` vive en
`genmsi-core/src/live.rs` (commit `93de9df`): el modelo abierto, el `dirty`,
los diagnósticos, el modelo en memoria (sólo cuando difiere del disco) y la
última corrida. El error del directorio de datos pasó a `AppDataError`: ya no
es sólo el del modelo abierto. Cinco tests.

L10 (2026-10-31, T3 y T4 — el MCP, worker): la herramienta `app_state` y el
freno `model_locked` (commit `b85242c`). El freno va **antes** de parsear,
compilar o escribir, y sólo aplica a la fuente de archivo (un modelo en
texto no tiene archivo que la app pueda tener abierto). El worker pidió una
decisión sobre el pin de herramientas de `tests/tools.rs` (ocho → nueve): la
tomé yo, porque el pin existe para eso.

L11 (2026-10-31, T5 y T6 — la guía y la documentación, commit `977b5ef` y
`182f988`): el procedimiento en la guía (mirar antes de escribir, avisar,
reenviar el lote), D22 con el porqué, y `docs/mcp.md` y `docs/app.md` con la
herramienta, el documento y el freno.

L12 (2026-10-31, T2 — la app, worker): la app publica `live.json` con el
escritor atómico del núcleo, **sólo cuando el estado cambia** (compara el
texto con el último publicado), y lo borra al cerrar (commit `ff1c187`). La
sesión guarda la última corrida y arma el cuerpo del documento.

L13 (2026-10-31, cierre): `cargo test --workspace` → **637 tests verdes**;
el shell de Tauri (fuera del workspace) → **17 verdes**; `tsc --noEmit`
limpio y los **381** tests del frontend verdes; clippy del workspace limpio.
La verificación diferencial contra `main` (18 casos de las siete herramientas
viejas) dio **0 diferencias**: el documento nuevo, la herramienta nueva y el
freno son aditivos y no cambian el cuerpo que el agente lee de las demás.

Dos límites que quedan, y son límites del diseño, no defectos:

- el `live.json` **no se publica en el arranque**: hasta la primera acción
  del lector no hay petición que lo disponga (y no hay modelo abierto que
  publicar);
- la corrida se recuerda en la sesión que hay cuando termina: si el lector
  abrió otro modelo mientras la corrida estaba en vuelo, la corrida queda
  atribuida al modelo nuevo (el caso límite de una corrida que va fuera del
  hilo de la ventana).
