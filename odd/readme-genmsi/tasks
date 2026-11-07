# Feature: el README de GenMSI — corto, en inglés y con lo que se ve

El README actual describe la **arquitectura** (13,6 KB de prosa, una tabla de
crates al principio y seis bloques JSON de configuración de clientes) y no el
**producto**. No tiene ni una sola pieza visual; el gancho —el servidor MCP
para agentes y la paridad semántica con el Delphi original— aparece una vez
en el primer párrafo y se pierde; y la prueba más fuerte del proyecto (20
modelos comparados contra el original, 83–100 % de las métricas) vive en
`docs/oracle-comparison.md`, a un click de la portada que nadie clickea.

El README nuevo es **corto**, en **inglés**, ordenado por capas —qué es → lo
pruebo en vivo → por qué creerno → cómo está hecho → detalle → docs— y **sin
binarios**: en GitHub, sin imágenes, lo único que se ve de verdad son los
badges, los diagramas mermaid, las tablas, los `<details>` y el ritmo del
texto. Eso define el trabajo de diseño: **jerarquía, aire y consistencia**, no
decoración.

- Rama: `feature/readme-genmsi` (base: `main`)
- Entrega: commits de unidad de trabajo en la rama; el push y el PR son del usuario
- Diseño: `design/readme-genmsi/` (ocho artboards, tema claro y oscuro)

## Specs

### S1 — Primero pensarlo, después diseñar

> "Antes de implementar nada ni diseñar nada quiero que analices y pensemos
> juntos que debemos mostrar en el readme y cuando finalmente estemos de
> acuerdo en todo recien ahi pasaremos al diseño"

El contenido del README se acuerda **antes** de escribir una línea de markdown
y antes de diseñar. El orden es: análisis → acuerdo → diseño → implementación.

### S2 — No sólo información: también cómo se ve

> "hoy quiero que mejoremos y pulamos mucho mejor el readme, no solo de info
> sino para como visualmente se ve. Podemos usar gentle design para esto"

El README se diseña, no sólo se escribe. El trabajo visual se hace con
**gentle design** (artboards bajo `design/readme-genmsi/`), y lo que sale de
ahí es la maquetación final.

### S3 — readership, idioma, material y extensión

Cuatro decisiones del usuario:

- **Audiencia**: "Generalista y equilibrado" — ninguna domina; el orden es por
  capas (qué es → demo en vivo → detalle técnico).
- **Idioma**: "English, como hoy".
- **Material visual**: "Nada binario por ahora" — texto y tablas bien
  maquetados primero; los visuales se evalúan después.
- **Extensión**: "Corto + docs como detalle" — el README es de una pantalla;
  el resto vive en `docs/`.

### S4 — El esqueleto aprobado

Nueve secciones, en este orden:

1. **Hero** — nombre, una línea de posicionamiento, badges (CI, MIT, v0.5.1).
2. **What it is** — motor y formato / servidor MCP / app de escritorio.
3. **Try it in 60 seconds** — requisitos y un bloque bash.
4. **The demo** — el flujo MCP de cuatro llamadas **y** el reporte GPSS real.
5. **Why the numbers hold** — la paridad con el original, con la tabla.
6. **One engine, four clients** — el diagrama de la arquitectura.
7. **The MCP server** — las ocho herramientas, la guía del agente, los avisos.
8. **Documentation** — el mapa de los documentos.
9. **Status & license** — versión, fases, MIT, el Delphi original.

### S5 — El diagrama, en mermaid

Decisión del usuario: "Mermaid, con clases de la paleta". El diagrama se
renderiza nativamente en GitHub y sigue el tema claro/oscuro del lector. Sus
colores salen de `design/system.md` (`#6366f1` indigo, `#0d9488` teal,
`#ea580c` naranja, `#0f172a` slate, `#f1f5f9` rejilla), para que el README se
vea de la misma familia que la app.

### S6 — Las dos demos

Decisión del usuario: "Las dos (flujo MCP + reporte)".

- **El flujo del agente**: `catalogue` → `apply_edit` → `lint` → `simulate`,
  más el dato de que un agente frío construyó 35 bloques en 12 llamadas MCP.
- **El reporte real**: la salida verdadera de
  `genmsi simulate fixtures/models/ejercicio_1__ejercicio_1.gmdl`, de 29
  líneas, con las tablas `FACILITY` y `QUEUE` del reporte MicroVisualSIS. La
  salida se **copia de una corrida real**, no se inventa.

### S7 — Lo que sale del README

Las seis configuraciones de clientes (Pi, Claude Desktop, Cursor, VS Code,
Windsurf, Zed) y los dos avisos que muerden (el directorio de trabajo y el
guía que es un archivo) pasan a `docs/mcp.md`, que hoy no los tiene. Las reglas
de la casa quedan en `docs/DECISIONS.md`; la tabla de crates se reduce a una
línea; la sección "The original" se reduce a dos líneas dentro de "Why the
numbers hold".

### S8 — Los hechos, correctos

El README afirma ocho herramientas del MCP y las nombra: `catalogue`,
`apply_edit`, `lint`, `simulate`, `get_trace`, `explain_model`, `save_model`
y **`app_state`** (la que devuelve `genmsi.live/1`: qué modelo tiene abierta
la app). El README actual la llama `open_model`, que no existe. Toda cifra
publicada —20 modelos, 83–100 %, 66 clases de bloque, 25 modelos de ejemplo,
v0.5.1— se verifica contra el repo antes de escribirse.

### S9 — Sin binarios, con lo que GitHub sí dibuja

El README no compromete imágenes. Lo visual se consigue con lo que GitHub
renderiza nativamente: badges, mermaid, tablas de pipe, `<details>` para lo
secundario y el ritmo del texto. Cuando se evalúen los visuales más adelante,
el de mayor retorno es uno: **la captura de la app**.

## Tasks

| ID | Qué hace | Specs | Ruta | Commit |
|---|---|---|---|---|
| T1 | Los ocho artboards del README (tema claro y oscuro) en `design/readme-genmsi/`, con el `design.json` | S2, S3, S4, S5, S9 | padre (canvas) | `docs(design)` |
| T2 | El README nuevo, con las nueve secciones | S1, S3, S4, S5, S6, S8, S9 | padre | `docs(readme)` |
| T3 | `docs/mcp.md`: las configuraciones de los seis clientes y los dos avisos | S7 | padre | `docs(mcp)` |
| T4 | El estado del proyecto y la licencia: versión, fases, MIT, el original | S4, S8 | padre | `docs(readme)` |
| T5 | Verificación: enlaces, cifras contra el repo, el reporte contra una corrida real, el markdown renderizado | S8 | padre | `docs(readme)` |

## Log

- L1 — "Buenas! hoy quiero que mejoremos y pulamos mucho mejor el readme, no solo de info sino para como visualmente se ve. Podemos usar gentle design para esto. Antes de implementar nada ni diseñar nada quiero que analices y pensemos juntos que debemos mostrar en el readme y cuando finalmente estemos de acuerdo en todo recien ahi pasaremos al diseño"
- L2 — Análisis del README actual y del repo: 13,6 KB de prosa, cero piezas visuales, el gancho enterrado, la prueba del oráculo ausente, seis JSON de clientes, la instalación mal ubicada dentro de la sección del MCP.
- L3 — Cuatro decisiones del usuario: audiencia "Generalista y equilibrado", idioma "English, como hoy", material visual "Nada binario por ahora", extensión "Corto + docs como detalle".
- L4 — Esqueleto aprobado "Sí, tal cual"; diagrama "Mermaid, con clases de la paleta"; demo "Las dos (flujo MCP + reporte)".
- L5 — Hallazgos de la verificación previa, que corrigen el README actual: la herramienta `open_model` no existe (es `app_state`, que devuelve `genmsi.live/1`); las seis configuraciones de clientes y los dos avisos no están en `docs/mcp.md`, que es adonde van; `docs/mcp.md` ya describe las ocho herramientas.
- L6 — Rama `feature/readme-genmsi` creada desde `main`.
- L7 — Diseño: `design/readme-genmsi/`, con ocho artboards (01-hero, 02-que-es, 03-demo, 04-por-que-los-numeros, 05-arquitectura, 06-mcp, 07-docs-estado, 08-readme-completo) y el generador `design/readme-genmsi/render.mjs`, que los renderiza **desde `README.md`** con los tokens de `design/system.md`: la maqueta y el markdown no pueden divergir. Cada artboard lleva su conmutador de tema (claro/oscuro) y sus notas de diseño. El artboard elegido es el **08** (el README completo), que es el que se implementa; los otros son sus secciones.
- L8 — La corrida del reporte va con **semilla fija** (`--seed 42`): sin ella el reporte cambia entre corridas, porque el modelo es estocástico, y el README mostraría cifras que el lector no puede reproducir. Con la semilla, la salida del README es exactamente la que el lector obtiene.
- L9 — Correcciones de hecho sobre el README anterior, encontradas al verificar: la herramienta `open_model` no existe (es `app_state`); las seis configuraciones de clientes y los dos avisos vivían en el README y no en `docs/mcp.md` (que es adonde van, T3); el README decía «The five configurations» y son seis.
