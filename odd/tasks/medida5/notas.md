# Medición del MCP de GenMSI desde el lado del agente (agente frío)

Fecha: 2025-11-02 · Repo: `/home/lucas/Documentos/Workspace/Genmsi` · Rama de trabajo: la que trae
`crates/genmsi-core/src/app_data.rs`.

**Qué hice.** El modelo `odd/tasks/medida5/modelo.gmdl` (centro de vacunación) está "abierto en la app"
con cambios sin guardar. Sin tocar el archivo, sin leer `crates/**` ni `docs/**`, hice un cambio real
(renombrar un bloque + cambiar una propiedad), lo dejé como **propuesta**, lo provoked a fallar, y leí
la traza, el resumen y el mapa. Todas las llamadas pasaron por `odd/tasks/medida4/mcp.py`.
Las respuestas crudas están en `mcp-calls.log` (35 entradas, con el `=== CALL:` y sus argumentos).

**Veredicto en una línea.** El servidor es unusually bueno: los errores nombran el bloque, la clase,
la propiedad y la posición en el archivo, y el cerrojo `model_locked` protege de verdad las dos
carreteras de escritura. Lo que falla es el *andén*: con la app cerrada, el agente deja la propuesta y
no tiene ninguna forma de saber qué pasó con ella.

---

## 1. Todas las llamadas, en orden

35 intentos registrados; **3 no llegaron al servidor** (fallos míos de shell/JSON,marked "LOCAL-CRASH"),
o sea **32 invocaciones reales de herramienta**: 25 con respuesta satisfactoria y 7 con error.
Además hice **1 llamada de protocolo** (`tools/list`), que no es una llamada de herramienta pero que un
agente real recibe al conectarse: sin ella no hay forma de descubrir `dry_run`, los filtros de la
traza ni el `map` de `explain_model` (el cliente `mcp.py` solo sabe hacer `tools/call`; por eso escribí
`list_tools.py`, que solo imprime los esquemas).

| # | Herramienta | Argumentos (resumidos) | Resultado |
|---|---|---|---|
| 1 | `app_state` | `{}` | OK — el modelo abierto, `dirty: true` |
| 2 | `lint` | `{}` (sin path ⇒ modelo abierto) | OK — `status: ok`, 52 statements, 0 diagnósticos |
| 3 | `explain_model` | `{}` | OK — `program` + `model` + `map` (52 filas) |
| 4 | `catalogue` | `classes:[FacilityUse,Queue,Terminate,Advance,StorageUse]` (una sola llamada) | OK |
| 5 | `apply_edit` | `dry_run:true`, `setCaption`+`setProperty` sobre el bloque 18 | OK — `applied:2, written:false` + `source` |
| 6 | `apply_edit` | mismo lote, `target` por defecto (`file`) | **ERROR `model_locked`** |
| 7 | `apply_edit` | `dry_run:true`, `Media = "no-es-un-numero"` | OK — **aceptado en silencio** (ver §6) |
| 8 | `apply_edit` | `dry_run:true`, `setProperty` sobre el bloque **999** | **ERROR `invalid_command`** |
| 9 | `apply_edit` | `dry_run:true`, `connect` desde el **Clock** (3), que no tiene salida | **ERROR `invalid_command`** |
| 10 | `apply_edit` | `dry_run:true`, `setProperty` con la propiedad **"Tamano"** (sin tilde) | **ERROR `invalid_command`** |
| 11 | `apply_edit` | `target:"proposal"` | LOCAL-CRASH (mi comillas en el shell) |
| 12 | `apply_edit` | `target:"proposal"`, `summary:"Renombra…"` | OK — **propuesta creada** |
| 13 | `app_state` | `{}` | OK — pero **sin el campo `proposal`** (ver §5) |
| 14 | `simulate` | `seed:42` | OK — informe GPSS |
| 15 | `get_trace` | `summary:true, seed:42` | OK — `genmsi.trace_summary/1` |
| 16 | `get_trace` | `limit:8` | OK — primeros eventos |
| 17 | `get_trace` | `classes:["Queue"], limit:5` | OK — filtro por clase |
| 18 | `get_trace` | `nodes:[18], limit:10` | OK — filtro por bloque del documento |
| 19 | `get_trace` | `xact:1` | OK — filtro por transacción (47 eventos) |
| 20 | `get_trace` | `from:500, to:600, limit:8` | OK — filtro por ventana de tiempo |
| 21 | `explain_model` | `path:` el modelo de la **propuesta** | OK — el `caption` ya es el nuevo |
| 22–24 | `get_trace` | `path:` la propuesta (`nodes`, `summary`) | OK — comparación disco/propuesta |
| 25 | `apply_edit` | `path:` **explícito** del archivo, `target:file` | **ERROR `model_locked`** (el path explícito no esquiva el cerrojo) |
| 26 | `save_model` | `path:` + `model:` (documento entero), `target:file` | **ERROR `model_locked`** (escritura nada) |
| 27–29 | `app_state` | 3 llamadas previas al log (ver §6.1) | OK |
| 30 | `lint` | `{}` con un `XDG_DATA_HOME` vacío | **ERROR `no_open_model`** |
| 31–32, 35 | `apply_edit`/`get_trace` | repetir el valor inválido y simularlo | OK — el valor basura **cambia la simulación sin diagnóstico** |
| 33–34 | `get_trace` | dos intentos mal formados por mi parte | LOCAL-CRASH |

---

## 2. Qué respondió el servidor y qué entendí

Citas textuales de las respuestas (el log trae el JSON completo).

**a) "El lector está editando" — `app_state` (llamada 1).**
```json
{ "schema": "genmsi.live/1",
  "path": "/…/odd/tasks/medida5/modelo.gmdl",
  "pid": 382987, "dirty": true, "diagnostics": [], "model": null, "run": null }
```
`dirty: true` + `path` = "el lector tiene cambios sin guardar sobre este archivo". `model: null` significa
que la app no publica el documento en memoria: el MCP trabaja sobre el **disco**. Eso importa: el modelo
que yo veo por MCP **no contiene** los cambios sin guardar del lector; una propuesta se construye sobre
la versión del disco y el lector tendrá que reconciliar las dos cosas.

**b) Los modelos que no dependen de un path.** `lint {}`, `explain_model {}`, `simulate {}` y
`get_trace {}` (sin `path` ni `model`) trabajaron sobre el modelo abierto y respondieron con el `path`
del disco en el campo `path`. Confirmado también el caso contrario (llamada 30, home vacío):
```
ISERROR { "error": "no_open_model",
  "message": "there is no model open in the app: give `path` of a model file, or open a model in the app" }
```
Mensaje perfecto: dice exactamente qué hacer.

**c) El cerrojo (llamada 6, y repetido en 25 y 26).**
```
ISERROR { "error": "model_locked",
  "message": "the app has unsaved changes on this model; nothing was written: /…/modelo.gmdl.
              Ask the reader to save the model and send the edit again, or leave a proposal instead
              (target: \"proposal\") and let the reader decide.",
  "path": "/…/modelo.gmdl" }
```
Esto es lo que **me dijo qué hacer a continuación**, sin que yo lo supiera de antemano: la propia
respuesta ofrece las dos salidas ("que el lector guarde y reenvíes el mismo lote" o "deja una
propuesta"). Probé además a **nombrar el `path` explícito** (25) y a usar la otra calle, `save_model`
(26): los dos siguen dando `model_locked` y no escriben nada. El md5 de `modelo.gmdl` no cambió ni una
vez en toda la medición.

**d) Probar el lote sin escribir (llamada 5).** El parámetro `dry_run` no está en la guía; está en el
esquema publicado:
> "`dry_run` tries the batch WITHOUT writing it: the commands are applied and the model is compiled,
> nothing is written, and the answer carries the model it would have written."

La respuesta: `{"applied": 2, "written": false, "status": "ok", "created": [], "source": "<…el modelo tal
como quedaría…>"}`. En `source` comprobé (con un diff mental) que lo único que cambia respecto del
disco es `caption="Mesa de recepcion"` en el módulo 18 y `Media` de `25` a `30`. Es exactamente el
"ver antes de escribir" que quería, y devuelve el documento completo para que uno pueda ver el efecto.

**e) El catálogo (llamada 4).** Una sola llamada con las cinco clases, como manda la guía. Cada clase
trae `class`, `title`, `description`, `inputs`/`outputs` con su `ioid`, las `properties` con
`name/hint/ty/subtype/options`, `declares`/`uses` y la plantilla XML. Con eso confirmé que `Media` y
`Modificador` son de la clase `FacilityUse` y que `Entrada1`=1 / `Salida1`=2.

**f) `explain_model` (llamada 3).** Devuelve tres cosas: `program` (el GPSS compilado, una fila por
*sentencia*), `model` (el documento con cada bloque, sus propiedades por nombre y las `edges` que los
cablean) y `map` (una fila por sentencia: `statement`, `node`, `class`, `caption`, `line`, `column`).
En `glossary` además viene la explicación de cada clase en el modelo.

**g) `simulate` (llamada 14).** Informe GPSS: `end_time: 3600`, `end_reason: "terminated"`,
`transactions: 62`, `events: 2009`, y las filas por sentencia con `entries`/`current`/`retries`.
El `Terminate` del modelo no tiene `Unidades`, así que la corrida la cierra el `Clock` (3600).

---

## 3. El fallo que provoqué a propósito

Probé **tres** formas de romper un lote; las tres dan `invalid_command` con el mismo formato:
índice del comando, `op`, bloque, clase, nombre, propiedad y posición en el archivo.

**(a) Propiedad que la clase no declara** (llamada 10) — esta es la que responde a la pregunta:
```
ISERROR { "error": "invalid_command",
  "message": "command 0 (setProperty) failed on the block 18 (\"FacilityUse\"), property \"Tamano\":
              el bloque 18 (FacilityUse) no declara la propiedad Tamano",
  "path": "/…/modelo.gmdl",
  "line": 1, "column": 12403, "node": 18, "class": "FacilityUse",
  "command": 0, "op": "setProperty", "name": "FacilityUse", "property": "Tamano" }
```
**Sí: sin mirar el código fuente.** El mensaje trae el bloque (18), su clase (`FacilityUse`), la
propiedad que se intentaba poner (`Tamano`) y dónde está en el archivo (línea 1, columna 12403). Con el
`map` de `explain_model` (fila `{"statement": 16, "node": 18, …, "column": 12403}`) sé que ese 12403
es el desplazamiento del `<module moduleid="18">` dentro del `.gmdl`. Y como el mensaje va en el mismo
idioma que el resto del dominio ("no declara la propiedad"), no hace falta traducir nada.

**(b) Bloque que no existe** (llamada 8):
```
"message": "command 0 (setProperty) failed on the block 999, property \"Media\": no existe el bloque 999"
"node": 999, "command": 0, "op": "setProperty", "property": "Media"
```
Nombra la propiedad y el id, pero **no** trae `class` ni `line`/`column` (obvio: el bloque no existe).

**(c) `connect` desde un bloque sin puerto de salida** (llamada 9):
```
"message": "command 0 (connect) failed on the block 3 (\"Clock\"): el bloque 3 no tiene el puerto de salida 1"
"line": 1, "column": 1872, "node": 3, "class": "Clock", "name": "Clock"
```
Nombra el bloque, su clase, su nombre y su lugar en el archivo.

**Un hallazgo colateral de esta sección:** la guía dice que *un error de nombre de propiedad es
silencioso en el motor y sólo aparece en el `lint`* ("a typo is silent in the engine, so the lint is
where it shows up"). Por la vía de edición **no es cierto**: `apply_edit` rechaza `Tamano` con
`invalid_command`. La afirmación de la guía vale para un archivo escrito a mano, no para el lote de
edición. (Ojo al revés: el `lint` que corre *dentro* de `apply_edit` no lo detecta, porque el motor ya
falló antes.)

---

## 4. La traza

**Los cuatro filtros (llamadas 17–20) funcionan, y el `limit` cuenta lo que pasa el filtro**, no los
eventos de la corrida. Lo comprobé: `classes:["Queue"], limit:5` devolvió 5 eventos, todos de clase
`Queue`, y el primero es de la transacción 1 en el t=42 (no el evento 1 de la corrida, que fue un
`generate`). La respuesta además **repite el filtro aplicado**, que es una cortesía muy útil:
```json
"filter": { "classes": ["Queue"], "nodes": [], "xact": null, "from": null, "to": 600.0 },
"records": 5, "truncated": true
```
- Por **clase**: `classes:["Queue"]` → los `enter`/`leave` de las colas, con el nombre de la entidad en
  `detail` (`"centro"`, `"Cola_recepcion"`).
- Por **bloque del documento**: `nodes:[18]` → los 5 eventos/entradas del `FacilityUse` (Queue, Seize,
  Depart, Advance, Release), con `node: 18` en cada uno.
- Por **transacción**: `xact:1` → los 47 eventos de la primera transacción, de punta a punta
  (generada → cola → advance → facility → transfer → … → terminate).
- Por **ventana de tiempo**: `from:500,to:600` → solo los eventos entre 500 y 600 (cerrada por los dos
  lados, como avisa la descripción).

**¿El resumen respondió "cuántas veces pasó el flujo por aquí"? Sí, pero con una trampa de vocabulario.**
`get_trace({summary:true})` devuelve `genmsi.trace_summary/1`: por fila `block` (el índice de la
*sentencia*), `node` (el `moduleid`), `class`, `label`, `entries`, `first`, `last`. O sea, **por
sentencia, no por bloque del documento**: el `FacilityUse` (nodo 18) aparece en cinco filas —
```json
{"block":16,"node":18,"class":"Queue","entries":60,"first":53.0,"last":3577.0}
{"block":17,"node":18,"class":"Seize","entries":60,…}
{"block":18,"node":18,"class":"Depart","entries":60,…}
{"block":19,"node":18,"class":"Advance","entries":60,…}
{"block":20,"node":18,"class":"Release","entries":59,"first":79.0,"last":3545.0}
```
La respuesta a la pregunta es **60 veces** por el `FacilityUse` (59 salidas: la última transacción
seguía dentro cuando terminó la corrida), pero hay que **agrupar por `node`** para no contar cinco
veces. El nombre del campo (`block`) y el texto de la herramienta ("for every block") invitan a leer
"bloque del documento" cuando es "sentencia compilada".

**De un evento al bloque del modelo y a su lugar en el archivo: sí, y es una línea.** El evento trae
`block` (sentencia) y `node` (moduleid); el `map` de `explain_model` une las dos cosas:
```json
{"statement": 16, "node": 18, "class": "Queue", "caption": "FacilityUse", "line": 1, "column": 12403}
```
→ el evento de las 53.0 en el nodo 18 es el `FacilityUse` que el lector ve en el lienzo como
"FacilityUse", y vive en la línea 1, columna 12403 del `.gmdl`. Y al revés también: el `detail`
`"-> 17"` de un evento es *la siguiente sentencia*, con lo que la traza y el `map` forman un camino
completo evento→sentencia→bloque→posición. Lo comprobé sobre la propuesta (llamada 21): el mismo
`statement 16` sale ahora con `"caption": "Mesa de recepcion"`.

**Y el efecto de mi cambio, medido.** Con `seed:42` sobre el disco (Media=25) y sobre la propuesta
(Media=30), con el mismo `summary`:
```json
sentencia 19 (Advance del nodo 18)   disco:      entra 53.0, sale 79.0
                                        propuesta:  entra 53.0, sale 84.0
sentencia 20 (Release)                disco:      first 79.0  last 3545.0
                                        propuesta:  first 84.0  last 3550.0
```
El primer servicio pasa de 26 a 31 unidades y todo lo de abajo se corre 5 unidades. Es la prueba de
que el cambio es real y de que la propuesta se puede correr **antes** de que el lector la acepte (la
propuesta es un `.gmdl` en disco y se le pasa como `path` a `get_trace`/`explain_model`).

---

## 5. Qué tiene que hacer la persona

Caducado lo anterior, esto es lo que el servidor me dijo, literal:

> "Ask the reader to save the model and send the edit again, **or leave a proposal instead
> (target: \"proposal\") and let the reader decide.**"

O sea, dos caminos y ninguno los pisa:

1. **El lector guarda** su trabajo sin guardar y yo reenvío el mismo lote (`target:"file"`): el
   archivo se escribe y la app lo recoge sola.
2. **Dejo la propuesta** (`target:"proposal"`): el modelo **no toca el disco**; queda en el área de
   preparación de la app y el lector la ve en su lienzo, sucia y deshacible, y decide *Aceptar*
   (se escribe) o *Descartar* (se pierde, y la nota del descarte me vuelve).

Cómo lo supe, exactamente: el mensaje de `model_locked` (arriba) y la descripción de `apply_edit`
("`target` says where the result goes: … `\"proposal\"` leaves the model in the staging area of the app,
without touching the disk, where the person sees it and decides to accept it or to discard it — the
verdict comes back in `app_state`"). El `summary` que mandé viaja con la propuesta, así que el lector
lee "Renombra el bloque FacilityUse (18) a 'Mesa de recepcion' y sube su propiedad Media de 25 a 30"
**antes** de decidir.

**El único punto ciego de este camino:** la guía dice que "el veredicto vuelve en `app_state()`", pero
`app_state` **no sabe nada** de la propuesta mientras la app no corra. La llamada 13 (después de dejar
la propuesta) respondió:
```json
{ "schema": "genmsi.live/1", "path": "/…/modelo.gmdl", "dirty": true, "pid": 412916,
  "diagnostics": [], "model": null, "run": null }        ← ni una palabra de "proposal"
```
Y lo comprobé en un home de prueba: si el campo `proposal` está en el `live.json` que publica la app,
`app_state` lo devuelve tal cual (`{"id": …, "state": "pending", "note": null}`); si no, no devuelve
nada. Como la app no está corriendo, el único sitio donde el estado de la propuesta se ve es **el
archivo de al lado**, `staging/<id>.json`:
```json
{ "schema": "genmsi.proposal/1", "id": "1793667085547-f375446fe41f5376",
  "model": "/…/staging/1793667085547-f375446fe41f5376.gmdl",
  "target": "/…/modelo.gmdl", "state": "pending", "note": null,
  "summary": "Renombra el bloque FacilityUse (18) a 'Mesa de recepcion' y sube su propiedad Media de 25 a 30" }
```
Es decir: **el estado de la propuesta vive en un archivo que ninguna herramienta lee**. Un agente con la
app cerrada puede dejar la propuesta y leer el `.json` a mano, pero no tiene ninguna herramienta que le
diga "aceptada" o "descartada".

---

## 6. Lo que faltaba o confundía (lo más valioso)

### 6.1 El `XDG_DATA_HOME` relativo es una trampa, y el fixture venía roto
La primera llamada, con `XDG_DATA_HOME=odd/tasks/medida5/home` (relativo), **ignoró la variable** y leyó
el `live.json` del usuario real: la app de escritorio del usuario estaba de verdad corriendo (pid 2024,
abriendo `/home/lucas/Documentos/Workspace/IA/chat/prueba.gmdl`). Un `XDG_DATA_HOME` relativo se
descarta en silencio y el MCP pasa a hablar con la app real. Con ruta absoluta, el fixture preparación
por la tarea tampoco funcionaba: su `live.json` traía `pid: 376777`, **muerto**, y el servidor contesta
`path: null` — o sea "nadie tiene un modelo abierto" — sin una palabra de que el pid es lo que decide.
Descubrí la regla experimentally (copié el `live.json` a un home de prueba y le puse un pid vivo) y
**tuve que corregir el fixture**: `odd/tasks/medida5/home/org.gensmi.desktop/live.json` ahora apunta a
un `sleep` de seis horas (pid en `odd/tasks/medida5/app.pid`). Dos avisos: (a) el servidor debería
avisar "el pid del live.json no está vivo" en vez de hacerse el muerto; (b) **cualquier agente que
corra con un `XDG_DATA_HOME` relativo puede tocar el trabajo del usuario real sin enterarse**.

### 6.2 La guía va atrasada respecto de las herramientas
La guía (`skill/genmsi-modeling/SKILL.md`) es buena, pero el servidor grew:
- **No menciona `dry_run`**, ni los filtros de `get_trace` (`classes`, `nodes`, `xact`, `from`, `to`),
  ni `summary:true`, ni el `map` de `explain_model`. Es justo lo que la tarea pedía medir y está
  **sólo en las descripciones de las herramientas** (que son excellentes, pero hay que ir a buscarlas).
- Su tabla de comandos lista 8 operaciones; el esquema publicado tiene **12**: faltan `setLayouts`,
  `movePort`, `removePort` y `paste`.
- Dice que un nombre de propiedad mal escrito es silencioso en el motor y sólo lo ve el `lint`; por la
  vía de edición es falso (§3).
- Dice que "el veredicto vuelve en `app_state()`"; con la app cerrada, `app_state` no lo dice (§5).
- Nunca menciona que el servidor necesita un **pid vivo** en el `live.json` para considerar que hay una
  app abierta, ni que `XDG_DATA_HOME` debe ser absoluto.

### 6.3 El motor no valida los operandos numéricos (el hueco más serio)
El lote `setProperty {id:18, property:"Media", value:"no-es-un-numero"}` **se acepta en silencio**:
`applied: 1, written: false, status: "ok", diagnostics: []`, y el modelo "compila". Peor: la simulación
**cambia de comportamiento sin que nadie diga nada** — con ese valor el servicio del `FacilityUse` dura
~5 unidades en vez de 26. Un `dry_run` que dice "ok" sobre un operando que no es un número da una
falsa confianza: el agente no tiene forma de saber que dejó basura en el modelo salvo que mire la
traza y compare. Faltaría una validación de operando (o al menos un diagnóstico) en `setProperty`.

### 6.4 Nombres que no se corresponden con lo que significan
- `get_trace`/`get_trace(summary)`: el campo `block` es el índice de la **sentencia compilada**, no el
  bloque del documento (que es `node`). Un `FacilityUse` son 5 filas; leerlas como 5 bloques lleva a
  responder "60 entradas" cinco veces.
- El resumen **no trae el `caption`** (el nombre que ve el lector) ni la posición: hay que cruzarlo
  con el `map` de `explain_model`. El cruce es fácil (statement↔statement, node↔node) pero no está
  documentado en ninguna parte.
- `explain_model` da `line: 1` siempre, porque estos `.gmdl` son de una sola línea: el sitio real en
  el archivo es `column`. No es un error, pero "linea 1, columna 12403" suena raro en un archivo de 28 KB.
- En `catalogue`, la clase viene bajo la clave `class`, no `name` (las propiedades sí usan `name`).
  Un agente que busque `name` en la clase no la encuentra.
- `simulate` y `get_trace` tienen **semillas distintas por defecto** (`3982041859211083264` y
  `15598816602025196018`): dos corridas "iguales" con los mismos argumentos no son la misma corrida.
  Siempre hay que pasar `seed` explícito para comparar dos modelos.

### 6.5 Mezcla de idiomas
Las descripciones de las herramientas y los `error` (`model_locked`, `no_open_model`,
`invalid_command`) están en inglés; el texto que los explica es en español ("no existe el bloque 999",
"el bloque 3 no tiene el puerto de salida 1", "el bloque 18 (FacilityUse) no declara la propiedad
Tamano"). No es grave, pero un agente (o una persona) tiene que leer los dos idiomas para no perder
matices.

### 6.6 Lo que el agente tiene que saber y no se le dice
- Que una propuesta se puede **correr y explicar antes** de que la app la entere (es un `.gmdl` en
  `staging/`), y que por eso `path:` a la propuesta funciona con `get_trace`/`explain_model`. Lo
  descubrí por el campo `model` de la respuesta, no por la documentación.
- Que el `summary` de la propuesta viaja en un `.json` al lado, y que ese archivo es la única fuente
  del estado de la propuesta con la app cerrada.
- Que `target:"proposal"` funciona **aunque el lector tenga cambios sin guardar** (la propuesta
  espera), mientras que `target:"file"` se niega. Eso es lo que hace que la propuesta sea la única
  salida con el lector ocupado.

### 6.7 Cosas menores
- `apply_edit` con `path` explícito **no** esquiva el cerrojo (bien), pero tampoco avisa de que el
  `path` es el del lector: el error dice `model_locked` y ya está.
- `explain_model` sobre un modelo de 39 módulos devuelve ~88 KB de JSON en un solo objeto; el
  `source` de `apply_edit` mete el documento entero como **string escapado** dentro del JSON (28 KB en
  una línea), que es incómodo de leer. Un `dry_run` que devuelve el diff (o al menos los bloques
  tocados) sería mucho más usable que el documento entero.
- El cliente `mcp.py` levanta un proceso del servidor **por llamada**: 32 llamadas = 32 arranques. Para
  un agente real que hace 10 llamadas, es un coste real de latencia. No es culpa del servidor, pero
  es parte de la experiencia.

---

## 7. Estado final

- **El archivo del lector quedó intacto.** `odd/tasks/medida5/modelo.gmdl` sigue con
  `caption="FacilityUse"` y `Media="25"` en el módulo 18. md5 `27c8052700c0b4749eaac35fe9bd3ea5`
  antes y después de las 32 llamadas. El `model_locked` fue effective en las tres vías de escritura
  (`apply_edit` sin path, `apply_edit` con path, `save_model`).
- **La propuesta quedó viva** (no escrita en el archivo):
  - id: **`1793667085547-f375446fe41f5376`**
  - modelo de la propuesta: **`/home/lucas/Documentos/Workspace/Genmsi/odd/tasks/medida5/home/org.gensmi.desktop/staging/1793667085547-f375446fe41f5376.gmdl`**
  - estado: **`pending`** (en `staging/1793667085547-f375446fe41f5376.json`)
  - destino (`target`): `/home/lucas/Documentos/Workspace/Genmsi/odd/tasks/medida5/modelo.gmdl`
  - resumen: *"Renombra el bloque FacilityUse (18) a 'Mesa de recepcion' y sube su propiedad Media de 25 a 30"*
  - contenido: `caption` del módulo 18 = `Mesa de recepcion`, `Media` = `30` (el resto, byte a byte, igual)
- **La app sigue "ocupada"**: `live.json` con `dirty: true` y un pid vivo (placeholder, `odd/tasks/medida5/app.pid`).
  El lector (o el harness) tiene que *aceptar* la propuesta desde la app para que el archivo cambie; el
  MCP no puede hacerlo por su cuenta.

### Archivos que dejé dentro de `odd/tasks/medida5/`
- `notas.md` — este informe.
- `mcp-calls.log` — las 35 entradas con el `=== CALL:` y la respuesta cruda del servidor.
- `call.sh`, `last.py`, `list_tools.py` — los ayudantes que usé (el log se escribe con `call.sh`).
- `tools-schema.txt` — los esquemas que publica el servidor (salida de `list_tools.py`).
- `probe1/`, `probe2/` — homes de datos de prueba (el pid vivo y el caso "no hay app").
- `app.pid` — el pid del proceso placeholder que hace que el servidor considere que hay una app abierta.
- `home/org.gensmi.desktop/staging/` — la propuesta y su `.json`.
