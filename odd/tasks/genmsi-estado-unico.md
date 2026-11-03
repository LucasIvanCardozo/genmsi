# Feature: el estado de la app en un archivo

El estado de la app de escritorio vive hoy en **dos archivos** de su
directorio de datos, y el agente lee cada mitad de uno:

| Archivo | Lo que lleva | Quién lo lee |
|---|---|---|
| `open-model.json` | El modelo que la app tiene abierto y el pid | La herramienta `open_model` y el `path` por defecto de las herramientas |
| `live.json` | El modelo abierto, el pid, el `dirty`, los diagnósticos, el modelo en memoria, la última corrida, la propuesta | La herramienta `app_state` |

El resultado es un agente que lee el `path` de un archivo y el `dirty` de
otro: dos verdades que no pueden discrepar, pero que sí pueden — y la
medición con agente en frío lo mostró: el `path` venía de
`open-model.json` y el `dirty` de `live.json`, y un escenario con la
persona "editando" parecía una app que no editaba nada.

Peor: `open_model` es un **subconjunto** de `app_state`. Las dos
herramientas responden lo mismo sobre el modelo abierto; una lo hace mejor.

Esta feature deja **un archivo** y **una herramienta** para el estado.

## Specs

S1 (decisión del usuario, textual): "Un archivo, y se va open_model".

S2 (decisión del usuario, textual): "Un archivo, y se va open_model" — con
la consequence: "open_model y open-model.json desaparecen (rompe: hay que
subir la versión a 0.5.0). Una sola fuente de verdad: el dirty y el path no
pueden discrepar".

S3 (D13): "within one major version a change is additive only [...] and a
breaking change bumps the number".

S4 (D19): "One zone of truth: the engine publishes, the app invents nothing."

S5 (D11): "Every tool addition must justify itself against that goal."

### S1 — un archivo

La app publica **un** documento: `live.json`, con el `genmsi.live/1` que ya
lleva el modelo abierto, el pid, el `dirty`, los diagnósticos, el modelo en
memoria, la última corrida y la propuesta.

`open-model.json` **desaparece**: no lo publica la app y nadie lo lee. El
documento `genmsi.openmodel/1` desaparece con él.

### S2 — una herramienta

La herramienta `open_model` desaparece. Lo que decía —el modelo abierto y su
pid— está en `app_state`, que además dice si hay cambios sin guardar, qué
encontró la app, cuál fue la última corrida y qué propuesta hay sobre la
mesa.

El conjunto de herramientas pasa de **nueve a ocho**.

El `path` por defecto de las herramientas (S1 de
`odd/tasks/genmsi-manos-del-agente.md`) se resuelve con el mismo documento:
el `path` y el `pid` de `live.json`, con la misma regla de siempre (un pid
muerto no cuenta).

### S3 — el cambio rompe, y por eso sube la versión

Desaparecer `open_model` y `open-model.json` es un cambio que rompe: el
documento `genmsi.openmodel/1` y el archivo que lo acompaña se van, y una
herramienta del conjunto se va con ellos. La versión del producto sube
(0.4.0 → 0.5.0) y el release lo dice en su nota.

Es el primer cambio que rompe desde que el proyecto se versiona, y la regla
(D13) es explícita: un cambio que rompe sube el número.

### S4 — el motor es el dueño

El documento vivo lo construye el núcleo (`genmsi-core/src/live.rs`) y lo
escriben los dos clientes con el mismo escritor atómico. El MCP lo lee con
el lector del núcleo. La app publica con la función que publica.

### S5 — el conjunto de herramientas se achica

Ocho herramientas, cada una con una razón (D11). Ninguna se agrega: una
desaparece.

## Tasks

| ID | Task | Specs | Route | Commit |
|---|---|---|---|---|
| T1 | `genmsi-core`: el documento vivo es la única fuente del modelo abierto (el lector del `path` y el `pid`); `genmsi.openmodel/1` y su archivo se van | S1, S4 | inline | 1 |
| T2 | `genmsi-mcp`: `open_model` desaparece; el `path` por defecto se resuelve con el documento vivo | S2, S5 | inline | 1 |
| T3 | `genmsi-app`: la app deja de publicar `open-model.json` | S1 | worker | 2 |
| T4 | `genmsi-cli` + `skill` + `docs`: la herramienta que se va, el documento que se va, la versión y lo que el agente tiene que hacer en su lugar | S2, S3 | inline | 3 |
| T5 | close: `cargo test --workspace`, clippy, los tests del frente, y la comprobación de que el `path` por defecto y el `dirty` salen del mismo documento | S1-S5 | inline | 3 |

## Log

L1 (2026-11-02, usuario, textual): "Un archivo, y se va open_model".

L2 (2026-11-02, análisis, el mapa): `open-model.json` lo leen **sólo** el
MCP (la herramienta `open_model` y el `path` por defecto) y los tests del
núcleo. La CLI no lo lee. La app lo escribe cuando cambia el modelo abierto.
No hay otro consumidor: un archivo que nadie más usa.

L3 (2026-11-02, análisis, el mapa): el documento vivo **ya lleva** el modelo
abierto y el pid (`LiveBody.path` y `LiveBody.pid`), y la app lo publica
después de **cada** petición de sesión — o sea, siempre que hay algo que
contar, y también al abrir un modelo. El archivo del modelo abierto es
información que el documento vivo ya lleva.

L4 (2026-11-02, análisis, la asimetría): `open_model` responde un subconjunto
de lo que responde `app_state`: el modelo abierto y el pid, sin el `dirty`,
sin los diagnósticos, sin la corrida y sin la propuesta. La herramienta que
sobra es la que responde menos.

L5 (2026-11-02, T1 y T2, el núcleo y el MCP): el documento vivo es la
única fuente del modelo abierto: `genmsi_core::live::open_model_from_live`
lo lee de ahí (con la regla de siempre: un pid muerto no cuenta), y la
herramienta `open_model` **desaparece**. El conjunto de herramientas pasa de
nueve a ocho, y `crates/genmsi-mcp/tests/tools.rs` lo fija.

El módulo del núcleo que alojaba el documento del modelo abierto se quedó
con lo que sigue siendo cierto —dónde viven los datos de la app y si un
proceso vive— y se renombró `app_data`: el nombre viejo mentía.

L6 (2026-11-02, T3, la app): la app publica el estado vivo después de
**cada** petición, incluso de las que terminan en un error: con un solo
archivo como fuente, "siempre publicar" es la invariante que evita que el
`dirty` y el archivo del disco cuenten historias distintas. El shell ya lo
hacía; se cerró un agujero (una petición que terminaba en error se saltaba
la publicación).

L7 (2026-11-02, T4, la versión): el cambio **rompe** —una herramienta del
conjunto y un documento publicado se van— y la regla (D13) es que un cambio
que rompe sube el número: la versión va a **0.5.0**, escrita en las siete
copias por `scripts/bump-version.sh`.

L8 (2026-11-02, lo que queda): el **puente de desarrollo** (el modo del
navegador) nunca publica el estado vivo, así que en ese modo el agente
sigue sin ver el estado de la app. Y `live.json` no se publica hasta que
hay una petición: una app recién abierta, sin Modelo y sin petición, no
tiene estado publicado.
