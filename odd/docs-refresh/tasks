# Feature: la documentación de GenMSI — al día, legible y sin jerga

La documentación de `docs/` (13 documentos, ~5.400 líneas) más los seis
documentos que están fuera de ella (la guía del agente, los README de
`scripts/`, `fixtures/`, el crate de la app, el `LEEME.txt` y el registro de
skills)tense更多的是 como el **cuaderno de notas de una fase**: hablan de
"Phase 2/3/4", de "the third vertical slice", de "the original" sin definirlo
jamás, y en **veintiséis puntos concretos afirman algo que el código ya no
cumple**. El README se reescribió con el estilo nuevo (qué es → para qué →
detalle, todo verificable); los docs todavía no.

Este trabajo lleva el mismo estilo a los 19 documentos, y de paso **arregla lo
que miente**: el punto de partida es la verdad, no la prosa.

- Rama: `feature/readme-genmsi` (continuación: el README es el modelo, los
  docs son su continuación)
- Entrega: un commit por lote, en el orden de los lotes
- Verificación: `cargo test --workspace`, `bun run test` en
  `crates/genmsi-app`, el test nuevo doc-vs-código, y la comprobación de
  enlaces

## Specs

### S1 — Primero el análisis, después la mano

> "vamos a revisar todas las docs en busca de mejoras de ese estilo. hace uin
> analisis completo y edcime todos los cambios que reocmendas. Sin mi
> autorizacion no haras nada"

El análisis se entrega completo (patrones, hallazgos con `ruta:línea`, y las
decisiones que el padre no puede tomar). **Ninguna** edición antes de la
autorización explícita del usuario.

### S2 — El estilo al que se apunta

El del README nuevo:

- **qué es y para qué** antes que el detalle;
- cada afirmación es **verificable** por el lector, o el doc dice cómo
  verificarla;
- **sin jerga sin definir**: ni fases, ni "slice", ni "D19" suelto, ni
  "the original" sin definir;
- los nombres de archivo, comandos y rutas van en `código`;
- los documentos largos llevan **índice** y se leen por capas;
- lo que el lector no necesita, no está.

### S3 — Los docs tienen que decir la verdad

> "los cambios que reocmendas"

Un documento que afirma algo que el código contradice es peor que un
documento que no existe: el lector (o el agente) lo toma por cierto. Cada
afirmación se contrasta con el código antes de escribirla, y las que no se
pueden contrastar se dicen como tales ("esto se comprobó en la corrida del
<fecha>").

### S4 — Un contrato, una fuente

El mismo contrato copiado en tres o cinco lugares divergió tres veces en este
repo (el corte de las capas, el contrato del CLI, los eventos del trace, los
cinco modelos rechazados, la historia de `model_locked`). Queda **una** fuente
de verdad y los demás lugares **enlazan**.

### S5 — Lo que no debe quedar visible

Los nombres del coursework (con apellidos) repetidos ~50 veces, una ruta
absoluta de una máquina (`Z:\home\lucas\...`) y el registro de skills con rutas
de mi equipo. El README ya no expone ningún coursework; los docs tampoco.

### S6 — Los documentos generados se editan en el generador

`docs/oracle-comparison.md` lo escribe entero
`crates/genmsi-core/examples/oracle.rs` (prosa incluida) y
`docs/BLOCK-INVENTORY.md` lo escribe `scripts/inventory_blocks.py`. Una mejora
de estilo aplicada al `.md` se pierde en la próxima corrida: se hace en el
generador y se regenera.

### S7 — Que la deriva no vuelva a ser silenciosa

No hay ningún test que compare un documento con el código, y por eso la
deriva pasó inadvertida. Un test que lea `docs/mcp.md` y afirme la lista de
herramientas y los campos de cada tabla vuelve la deriva un error de CI.

## Tasks

| ID | Lote | Qué hace | Specs |
|---|---|---|---|
| T1 | **La verdad** | Los 26 hallazgos P0: afirmaciones que el código contradice, enlaces rotos, comandos que no funcionan | S3 |
| T2 | **La capa** | En los 8 documentos largos: "qué es" + índice + "Start here"; sacar la jerga de fases | S2 |
| T3 | **Una fuente** | Los 9 contratos duplicados: queda una fuente y el resto enlaza | S4 |
| T4 | **Lo que no se ve** | coursework, rutas absolutas, y el grupo del oráculo enlazado desde donde corresponde | S5 |
| T5 | **El pulido** | Tablas incompletas, anclas, nombres de botones, glosarios, el `LEEME.txt` | S2, S3 |
| T6 | **Las trampas** | El test doc-vs-código y la marca "generado, no editar a mano" en los dos documentos generados | S6, S7 |

Cada lote es un commit. El orden importa: la verdad primero (que el resto se
apoye sobre ella), la capa y la desduplicación después (que reescriban sobre
texto cierto), y las trampas al final (que protejan el resultado).

## Decisiones

| # | Decisión | Resolución |
|---|---|---|
| D1 | Los nombres del coursework en los docs internos | **Se quedan los nombres reales.** Los docs internos los nombran como hoy; sólo la portada pública no los expone, como quedó el README. |
| D2 | El vocabulario: ¿"the original" / "GPSS" se nombran o se eliminan? | **Eliminarlo del todo.** Toda comparación se reescribe en positivo ("el motor hace X"), sin nombrar otro proyecto. Se conservan los nombres de los artefactos del producto (el programa GPSS que el motor compila, `genmsi gpss`, `genmsi.gpss/1`), porque esos son nuestros. |
| D3 | `docs/ROADMAP.md`: reescribirlo como estado, o borrarlo | **Borrarlo.** Las fases 0 a 4 están entregadas y el log de tareas (`odd/tasks/`) queda como fuente. Se sacan las referencias que lo nombraban (el README, `docs/cli.md`, `docs/mcp.md`, D13). |
| D4 | `docs/mcp-transcript.md`: regenerarlo contra el servidor actual | **Regenerarlo.** Se capturó una sesión real contra `genmsi mcp` (initialize, `tools/list`, `catalogue`, `apply_edit`, `lint`, `simulate`, `get_trace` y un error con su arreglo) y el documento se genera desde esa captura. |
| D5 | `crates/genmsi-app/LEEME.txt`: sigue en español | Se mantiene (es el documento del usuario final descargado). |
| D6 | La interfaz en español y los docs en inglés | Fuera de alcance: es un cambio de producto, no de documentación. Queda anotado como fricción conocida. |

## Log

- L1 — "perfecto! me encanta"
- L2 — "buenisimo! vamos a revisar todas las docs en busca de mejoras de ese estilo. hace uin analicis completo y edcime todos los cambios que reocmendas. Sin mi autorizacion no haras nada"
- L3 — Análisis completo: seis exploradores leyeron los 19 documentos y verificaron cada hallazgo contra el código. ~70 hallazgos; los patrones: documentos congelados en el pasado, 26 afirmaciones falsas, 9 contratos duplicados (4 ya divergentes), 8 documentos largos sin índice, ~100 menciones de "the original" sin definir, coursework con apellidos y una ruta de máquina visibles, y 2 documentos generados que se leen como editables.
- L4 — "dale autorizo!" — autorización explícita para aplicar los cambios.
- L5 — Rama `feature/readme-genmsi`: el trabajo continúa en la misma rama, porque el README es el modelo del estilo que se está aplicando y separar las dos mitades obligaría a rebasar la referencia.
- L6 — Las seis decisiones del usuario: D1 los coursework se quedan en los docs internos; D2 "the original" y "GPSS" **se eliminan** y toda comparación se reescribe en positivo (los nombres de nuestros propios artefactos —el programa GPSS, `genmsi gpss`, `genmsi.gpss/1`— se conservan); D3 `docs/ROADMAP.md` **se borra** y se sacan sus referencias; D4 la transcripción del MCP **se regenera**; D5 el `LEEME.txt` sigue en español; D6 el idioma de la interfaz queda fuera de alcance.
- L7 — El padre, en paralelo a los cuatro lotes de correcciones: `docs/format-and-engine.md` (el algoritmo de composición: el doc decía "tight tree" y el código usa el network simplex con el tight tree de respaldo), `docs/DECISIONS.md` (D2, D3, D8, D13, D23), el borrado del ROADMAP y su efecto en el README, y la transcripción del MCP regenerada desde una captura real (la sesión completa: el modelo se construye con `apply_edit`, sin escribir XML, y el agente se equivoca con un almacén sin declarar y lo arregla con un `StorageDef`).
- L8 — Un hallazgo del padre durante la captura: **los mensajes de error del motor están en español** ("no existe el contenedor 1", "el bloque 1 no tiene el puerto de salida 7") mientras que el protocolo y los documentos van en inglés. Es una fricción de idioma del producto, no de la documentación; queda anotada para el usuario.
