# Medición del lado del agente: GenMSI MCP + una persona editando el modelo

**Qué hice:** renombré un bloque (`Generate`, id 1 → caption `LLEGADAS`) y cambié una de sus
propiedades (`Media` 60 → 45) sobre un modelo que la persona tiene abierto **con cambios sin
guardar**, sin tocar su archivo. Dejé el cambio como **propuesta** (`target: "proposal"`).

**Cómo:** todo por el servidor MCP, con el cliente `odd/tasks/medida4/mcp.py`.
Las respuestas crudas están en `odd/tasks/medida4/mcp-calls.log` (20 llamadas de herramienta +
2 consultas `tools/list` = 22 interacciones). El wrapper `odd/tasks/medida4/call.py` es mío:
solo llama a `mcp.py` y agrega la llamada al log.

**Aviso de entorno (importante):** el comando del enunciado
`XDG_DATA_HOME=odd/tasks/medida4/home` **no funciona tal cual**: con una ruta *relativa* el
servidor cae silenciosamente al directorio de datos por defecto (`~/.local/share`) y termina
reportando la app de escritorio **real** que está corriendo en esta máquina (pid 2024,
`/home/lucas/Documentos/Workspace/IA/chat/prueba.gmdl`, `dirty: false`). Con la ruta
*absoluta* sí funciona. Ver la sección 6.

---

## 1. Lista de llamadas, en orden

| # | Herramienta | Argumentos (resumidos) | Resultado |
|---|---|---|---|
| 1 | `app_state` | `{}` | ⚠️ Devolvió la app real (ruta relativa mal resuelta) |
| 2 | `open_model` | `{}` | ⚠️ Ídem (app real, pid 2024) |
| 3 | `open_model` | `{}` (sin `XDG_DATA_HOME`) | Ídem (app real) |
| 4 | `open_model` | `{}` (`XDG_DATA_HOME=/tmp/emptyhome`) | `path: null, pid: null` |
| 5 | `open_model` | `{}` (`XDG_DATA_HOME=<absoluto>` del home falso) | ✅ `path: …/medida4/modelo.gmdl`, `pid: 169026` |
| 6 | `app_state` | `{}` (home falso) | `path: null`, `dirty: false`, `proposal: null` |
| 7 | `app_state` | `{}` (home falso, ya con `live.json`) | ✅ `path: …/modelo.gmdl`, **`dirty: true`** |
| 8 | `explain_model` | `{path: …/modelo.gmdl}` | ✅ Modelo completo (39 módulos, 42 aristas) |
| 9 | `catalogue` | `{classes: ["Generate"]}` | ✅ Puertos, propiedades y plantilla de `Generate` |
| 10 | `lint` | `{path: …/modelo.gmdl}` | ✅ `status: ok`, 52 statements, 0 diagnósticos |
| 11 | `apply_edit` | `{path: …/modelo.gmdl, commands: [setCaption id 1, setProperty id 1 Media=45]}` (target por defecto = `file`) | ❌ **`model_locked`** — no escribió nada |
| 12 | `apply_edit` | `{path: …/modelo.gmdl, target: "proposal", summary: "…", commands: [setCaption id 1, setProperty id 1 Media=45]}` | ✅ **`written: false`**, propuesta creada |
| 13 | `app_state` | `{}` | `dirty: true`, **sin campo `proposal`** (el stub no publica ninguno) |
| 14 | `tools/list` | (consulta, no es tool call) | Esquemas de las 9 herramientas |
| 15 | `tools/list` | (consulta) | Volcado a `/tmp/tools.json` para leerlo |
| 16 | `lint` | `{path: <staging>/1793662304738-….gmdl}` | ✅ `status: ok`, 0 diagnósticos |
| 17 | `simulate` | `{path: <staging>/1793662304738-….gmdl, seed: 42}` | ✅ `Generate` con operandos `45,30,,,` → 86 entradas |
| 18 | `simulate` | `{path: …/modelo.gmdl, seed: 42}` | ✅ línea base: `60,30,,,` → 61 entradas |
| 19 | `app_state` | `{}` (stub del app publica una propuesta inventada) | ✅ devuelve el `proposal` **tal cual** lo publica el app |
| 20 | `app_state` | `{}` (stub publica `state: rejected` + nota) | ✅ vuelve el veredicto con la nota |
| 21 | `app_state` | `{}` (stub publica `state: pending`, estado final) | ✅ `proposal.id = 1793662304738-2f7b5729b3af3d82`, `state: pending` |
| 22 | `apply_edit` | `{commands: [setCaption id 1]}` **sin `path`** (`XDG_DATA_HOME=/tmp/emptyhome`, cwd del servidor en `/tmp`) | ❌ **`no_open_model`** — no escribió nada |

**Total: 20 llamadas de herramienta + 2 consultas `tools/list` = 22 interacciones.**
De las 20 llamadas de herramienta: 4 fueron de diagnóstico del entorno (1–4), 1 fue el intento
de escritura bloqueada (11), 1 la propuesta (12), 2 de verificación del modelo (16–18) y el
resto lecturas de estado. **Una sola llamada de escrituraeffective (`apply_edit` #12);** la #11
fue el intento de escritura sobre el archivo, rechazado por el servidor.

---

## 2. Qué respondió el servidor y qué entendí de cada respuesta

Citas textuales de las respuestas (las relevantes) y qué me dijeron:

**#5 `open_model`**
```json
{"schema": "genmsi.openmodel/1",
 "path": "/home/lucas/Documentos/Workspace/Genmsi/odd/tasks/medida4/modelo.gmdl",
 "pid": 169026}
```
→ «Este es el archivo que la persona tiene abierto; no adivine otro». El `pid` (169026) es un
proceso vivo (`sleep 900`): el servidor lo usa como prueba de que «la app está corriendo».

**#6 `app_state` (sin `live.json`)**
```json
{"schema": "genmsi.live/1", "path": null, "pid": null, "dirty": false, "diagnostics": [],
 "model": null, "run": null, "proposal": null}
```
→ «No hay app en vivo: no hay nada que proteger todavía». **Ojo: esto es el mismo documento
vacío que devuelve cuando no hay app, y `dirty: false` es indistinguible de «la persona no
tiene cambios pendientes».**

**#7 `app_state` (con el estado en vivo)**
```json
{"schema": "genmsi.live/1", "path": "…/modelo.gmdl", "pid": 169026, "dirty": true, …}
```
→ «La persona **tiene cambios sin guardar** sobre ese archivo». Esta es la señal que dispara
todo el procedimiento.

**#8 `explain_model`** → el modelo tal cual: 39 módulos (bloques y contenedores), 42 aristas y
el programa GPSS. De acá saqué el `id` del bloque a tocar (id 1 = `Generate`, caption
`Generate`, `Media=60`, `Modificador=30`, `Cantidad` vacío). Entendí que los `id` de este
documento son los que espera `apply_edit`.

**#9 `catalogue {classes:["Generate"]}`** → puertos (`Salida1`, `ioid 1`) y las 5 propiedades
con su nombre exacto (`Cantidad`, `Media`, `Modificador`, `OffSet`, `Prioridad`), más la
plantilla XML. De acá saqué que `Media` es el nombre literal de la propiedad que quiero cambiar.

**#10 `lint {path}`**
```json
{"schema": "genmsi.lint/1", "status": "ok", "title": "Modelo1", "blocks": 52,
 "definitions": {"storages": 4, "qtables": 1, …}, "diagnostics": []}
```
→ «El modelo está sano antes de tocarlo». `diagnostics: []` = nada que arreglar de paso.

**#11 `apply_edit` (target por defecto = `file`)**
```json
{"error": "model_locked",
 "message": "the app has unsaved changes on this model; nothing was written: /…/modelo.gmdl. Ask the reader to save the model and send the edit again.",
 "path": "/…/modelo.gmdl"}
```
→ **La frase que me dijo qué hacer**: *«nothing was written»* y *«Ask the reader to save the
model and send the edit again»*. El servidor me pidió, literalmente, que le pida a la persona
que guarde. Ese mensaje, por sí solo, no menciona la alternativa de la propuesta: la ruta del
`target: "proposal"` está en la **descripción de la herramienta**, no en el error.

**#12 `apply_edit` (target `proposal`)**
```json
{"schema": "genmsi.edit/1", "path": "…/modelo.gmdl", "applied": 2, "written": false,
 "status": "ok", "blocks": 52, "diagnostics": [], "created": [],
 "proposal": {"id": "1793662304738-2f7b5729b3af3d82",
              "model": "…/home/org.gensmi.desktop/staging/1793662304738-2f7b5729b3af3d82.gmdl"}}
```
→ **«`written: false`»** = el disco no fue tocado. **«`applied: 2`»** = los dos comandos se
aplicaron. Y el campo `proposal` me da **la identidad** (id) y **dónde está el modelo**
(staging). Es todo lo que necesito para decirle a la persona qué está esperando.

**#13 `app_state`** → sin campo `proposal` (el stub de la app no publica ninguno). El servidor
**no** sabe nada de la propuesta por su cuenta; el estado del veredicto depende de que la app lo
publique (ver 6).

**#16–#18 `lint`/`simulate` sobre el modelo *de la propuesta*** → `status: ok`, 0 diagnósticos y
un informe que muestra el cambio funcionando:

| | `Generate` operandos | entradas | transacciones | eventos |
|---|---|---|---|---|
| modelo de la persona (`modelo.gmdl`, seed 42) | `60,30,,,` | 61 | 62 | 2009 |
| modelo de la propuesta (mismo seed) | `45,30,,,` | 86 | 87 | 2565 |

→ «La propuesta compila, corre y el cambio hace lo que dice que hace».

**#19–#21 `app_state` con la propuesta publicada** → devuelve el campo `proposal` **verbatim**:
con `id`, `state` (`pending` / `accepted` / `rejected`) y `note`. Probé el caso `rejected` con
la nota *"no tocar la frecuencia de arribos"* y volvió tal cual.

**#22 `apply_edit` sin `path`, sin app abierta**
```json
{"error": "no_open_model",
 "message": "there is no model open in the app: give `path` of a model file, or open a model in the app"}
```
→ «Sin app no hay destino: no escribiré nada».

---

## 3. Cómo supe que la persona estaba editando

**Lo que dijo el servidor:** el campo `dirty` de `app_state()` (documento `genmsi.live/1`):

```json
"dirty": true
```

y, de forma independiente y coincidente, el `apply_edit` con target `file` se negó:

> `"the app has unsaved changes on this model; nothing was written: …/modelo.gmdl. Ask the reader to save the model and send the edit again."`

**Lo que dice la guía** (`skill/genmsi-modeling/SKILL.md`, sección *The person reading*):

> «**Look before you write.** If the model you are about to change is the one the application
> has open, ask `app_state()` first: the reader has **no** changes pending → write it …; the
> reader **has** changes pending → **do not write**. Say so and wait: *"the application has
> unsaved changes on `modelo.gmdl`; shall I wait for you to save it?"*»

> «**`apply_edit` and `save_model` refuse that write for you.** When the application has unsaved
> changes on the model you named, they answer `model_locked` and **write nothing**. That error
> is not a failure of the model: it is the application saying "this one is being edited".»

Y la sección *The verb*:

> «**`target` says where the result goes.** "file" (the default) writes the model to the file
> the batch edited. "proposal" leaves it in the staging area of the app, **without touching the
> disk**, and the reader sees it on their canvas and decides.»

Las tres señales coinciden: `dirty: true` en `app_state()`, el `model_locked` y la guía.
**Conclusión operativa:** con cambios pendientes, `target: "file"` está prohibido; la ruta
correcta es `target: "proposal"`.

---

## 4. Cómo le dejé el trabajo a la persona, y cómo comprobé que su archivo no fue tocado

**Lo que le dejé:** una **propuesta**, no una escritura.

* Identidad: **`1793662304738-2f7b5729b3af3d82`**
* Modelo de la propuesta (el `.gmdl` con el cambio aplicado):
  `odd/tasks/medida4/home/org.gensmi.desktop/staging/1793662304738-2f7b5729b3af3d82.gmdl`
* Ficha de la propuesta (la escribe el servidor al crearla):
  `odd/tasks/medida4/home/org.gensmi.desktop/staging/1793662304738-2f7b5729b3af3d82.json`
  con `{"state": "pending", "note": null, "target": "…/modelo.gmdl", "summary": "…"}`
* `apply_edit` respondió `written: false` y `applied: 2`.
* El cambio, exactamente dos líneas del documento:
  * `caption="Generate"` → `caption="LLEGADAS"` (bloque `Generate`, id 1)
  * `<property name="Media" … value="60" …/>` → `value="45"`

**Cómo comprobé que su archivo no fue tocado.** Antes de la primera llamada copié el archivo a
`/tmp/modelo.before.gmdl` y anoté su `md5`. Al final:

```
md5sum /tmp/modelo.before.gmdl odd/tasks/medida4/modelo.gmdl
27c8052700c0b4749eaac35fe9bd3ea5  /tmp/modelo.before.gmdl
27c8052700c0b4749eaac35fe9bd3ea5  odd/tasks/medida4/modelo.gmdl

cmp /tmp/modelo.before.gmdl odd/tasks/medida4/modelo.gmdl   → sin salida (idénticos byte a byte)
```

Es decir: **mismo MD5 y `cmp` byte a byte sin diferencias** (no solo «el mismo texto»), y la
*mtime* del archivo sigue siendo `2026-11-02 20:29:45`, anterior a mi primera llamada
(`20:30:36`). Además comparé el archivo de la persona contra el modelo de la propuesta
(`diff` normalizando los `><`): las **únicas** dos diferencias son el caption y el `Media`, o
sea que la propuesta es el modelo del archivo + mi cambio, y nada más.

**No toqué nada fuera de `odd/tasks/medida4/**`.** Dentro de ese directorio, lo único que
agregué/alteré: `call.py` (wrapper mío), `mcp-calls.log`, `notas.md` y
`home/org.gensmi.desktop/live.json` (ver 6: el setup no lo proveía y sin él la app «no
tenía» nada sin guardar).

---

## 5. Qué tiene que hacer la persona para ver mi trabajo

Según la guía (*The proposal: the road that stays open*):

> «Leave a **proposal** when the reader is busy … The model does **not** touch the disk. It
> waits in the staging area of the app, and the app puts it on the reader's canvas — dirty,
> and undoable — so they see exactly what you built and decide: *Aceptar* writes it to the
> file, *Descartar* throws it away. The note they leave when they discard it comes back to
> you.»

Y la descripción de `apply_edit` en el propio servidor:

> «`proposal` leaves the model in the staging area of the app, without touching the disk, where
> the person sees it and decides to accept it or to discard it — **the verdict comes back in
> `app_state`**.»

Concretamente, la persona:

1. **No tiene que hacer nada para que su trabajo siga intacto**: su archivo no se tocó. Ya está.
2. En la app, la propuesta aparece **sobre su lienzo, en estado sucio y deshacible** (así lo
   describe la guía): ve el bloque renombrado y la `Media` en 45.
3. Decide: **Aceptar** (se escribe en el archivo) o **Descartar** (se tira; la nota que deje
   me vuelve por `app_state`).
4. Si prefiere **guardar su trabajo primero** y que mi cambio entre por la vía normal: que
   guarde, y yo reenvío el mismo lote. Eso es exactamente lo que pide el error: *«Ask the
   reader to save the model and send the edit again.»*

Y mi lado: **esperar el veredicto**. La guía es explícita:

> «A proposal that is still `pending` means the reader has not decided yet: **wait for it, and
> do not send the same batch again** — the proposal is the batch, replayed on the reader's
> canvas.»

Estado actual: `pending`. El veredicto se lee en `app_state()` → campo `proposal.state`
(`pending` / `accepted` / `rejected`) y `proposal.note`.

---

## 6. Qué faltaba o confundía (la parte que más sirve)

**a) La instrucción del setup es una bomba: `XDG_DATA_HOME` relativo cae al app real, en
silencio.** Con `XDG_DATA_HOME=odd/tasks/medida4/home` (relativo, tal como está escrito en el
enunciado y en el docstring de `mcp.py`) el servidor **no** encuentra ese directorio y **se
va al directorio de datos por defecto**, `~/.local/share`, donde vive la app de escritorio
real de esta máquina. `app_state` respondió:

```json
{"path": "/home/lucas/Documentos/Workspace/IA/chat/prueba.gmdl", "pid": 2024, "dirty": false}
```

Es el **modelo real y abierto del usuario**, con `dirty: false`. Si yo hubiera confiado en ese
estado y hecho `apply_edit` con target `file` (o sin `path`, que es «el modelo que la app tiene
abierto»), **el servidor habría escrito en el archivo real del usuario y no habría dicho nada
raro**: la respuesta habría sido un `genmsi.edit/1` normal con `written: true`. Lo único que
me salvó fue que la ruta del setup estaba en relativo y que yo sospeché. **Esto debería ser un
error ruidoso, no un fallback silencioso.** Con ruta absoluta (`$PWD/odd/tasks/medida4/home`)
todo funciona. El mismo problema conceptual: el servidor no dice *de dónde* sacó el estado.

**b) El setup no trae el archivo que dice «la persona está editando».** Trae
`open-model.json` («qué modelo está abierto»), pero el flag `dirty` vive en **otro** archivo,
`live.json` (documento `genmsi.live/1`), que el setup no provió. Sin él, `app_state` devuelve
`path: null, dirty: false` — **indistinguible de «la app no está corriendo»** — y un agente que
se fiara escribiría sobre el archivo de la persona, destruyendo su trabajo sin guardar. Peor:
la guía **nombra estos archivos en ningún lado**, y el esquema de la herramienta sólo dice
«It returns the `genmsi.live/1` document, the one the app publishes». Descubrí el formato
mirando el directorio de datos de la app real del usuario (`~/.local/share/org.gensmi.desktop/
live.json`), o sea, **fuera de los lugares sancionados** (ni la guía, ni los esquemas, ni el
catálogo). Tuve que escribir ese archivo a mano para poder medir el escenario que el setup
prometía. **Falta en el setup un `live.json` de ejemplo** (y en la guía, una frase que diga
qué archivos existen y dónde).

**c) `app_state` es un passthrough puro del documento que publica la app: el MCP no sabe nada
de su propia propuesta.** Pruebas: (i) después de crear la propuesta (#12), `app_state` **no
traía campo `proposal`**; (ii) metí a mano en `live.json` un `proposal` con id inventado
(`TEST-ID-FROM-APP`) y `app_state` lo devolvió **verbatim**. O sea: el `state`, el `note` y el
`id` que ve el agente **los publica la app, no el MCP**. Consecuencias: (1) con una app de
mentira (o sin app) **el agente nunca puede enterarse del veredicto por `app_state`**, y tiene
que ir a leer el `.json` del staging, que la guía no menciona; (2) el MCP y la app pueden
**desacordarse**: mi `.json` del staging seguía en `pending` mientras la app publicaba
`rejected`, y el servidor no reconcilia ni limpia el staging.

**d) El mensaje de `model_locked` no dice que existe la otra salida.** Dice «ask the reader to
save the model and send the edit again» — que es el consejo correcto, pero para un agente que
quiere dejar el trabajo *sin molestar* a la persona, la respuesta útil («usá `target:
"proposal"`») está **solo** en la descripción de la herramienta y en la guía. Con el error solo,
un agente puede quedarse esperando a que la persona guarde, que es justo lo contrario de lo que
pedía el escenario.

**e) La guía promete un error que no vi.** Dice: «With no app running, or with no model open,
the tools answer `no_open_model`». Lo que vi: `open_model` y `app_state` devuelven **documentos
normales** con `path: null` (que es, como dice el propio esquema, «an answer, not a failure»),
y recién `apply_edit` sin `path` contesta `no_open_model` (verificado, #22). La frase de la
guía induce a pensar que cualquier herramienta falla con ese error.

**f) `explain_model` devuelve una pared de texto.** Un modelo de 39 módulos produjo **3358
líneas** de JSON (bloques + `layout` de cada puerto + aristas + `program` + glosario). No hay
filtro, paginación ni modo «solo lo que me importa». En un agente con contexto acotado, **una
sola llamada de diagnóstico puede comerse la ventana**: me costó bastante parsear el bloque útil.
Un `?classes=` o un `outline` barato (id → clase → caption → props) would've servido.

**g) El MCP no tiene herramienta para el veredicto ni para «esperar».** No hay
`list_proposals`, `get_proposal`, `accept_proposal` ni `wait_proposal`: el agente depende
enteramente de `app_state` (que es un passthrough) y de leer archivos del staging a mano.
Tampoco hay forma de **cancelar** una propuesta que quedó `pending`, ni de saber si el
`staging/` se limpia solo.

**h) Cosas menores que costaron tiempo.** `apply_edit` acepta `path` **o** el modelo abierto:
no avisa si el `path` que pasás y el modelo que la app tiene abierto son distintos (yo pasé
`path` explícito por seguridad, y funcionó, pero el MCP no chequea coherencia). El `id` de
bloque que usa `apply_edit` es el `moduleid` del documento, y eso **no está en el catálogo**:
hay que cruzarlo con `explain_model` (o leer el `.gmdl`, que la guía desaconseja). Y el `id` de
propuesta (`1793662304738-2f7b5729b3af3d82`) es opaco (timestamp + hash): sirve, pero no es
legible ni ordenable a simple vista.

**i) La guía y la descripción de `open_model` se contradicen en un detalle:** la descripción de
la herramienta dice «Write the model to that path (with `save_model`)», mientras la guía
dice «Prefer `apply_edit`». Menor, pero es la clase de detalle que hace que un agente freshly
cold elija mal la herramienta.

---

## 7. Estado final

* **El cambio quedó como propuesta, no escrito en el archivo.**
* **Identidad de la propuesta:** `1793662304738-2f7b5729b3af3d82`
* **Modelo de la propuesta:**
  `/home/lucas/Documentos/Workspace/Genmsi/odd/tasks/medida4/home/org.gensmi.desktop/staging/1793662304738-2f7b5729b3af3d82.gmdl`
* **Ficha de la propuesta (estado):**
  `/home/lucas/Documentos/Workspace/Genmsi/odd/tasks/medida4/home/org.gensmi.desktop/staging/1793662304738-2f7b5729b3af3d82.json`
  → `{"state": "pending", "note": null, "target": "…/medida4/modelo.gmdl"}`
* **El archivo de la persona** (`…/medida4/modelo.gmdl`) está **intacto**: mismo MD5 que antes
  de empezar, `cmp` byte a byte sin diferencias, mtime anterior a mi primera llamada.
* **El cambio dentro de la propuesta:** bloque `Generate` (id 1) renombrado a `LLEGADAS` y su
  `Media` de `60` a `45`. Compila (`lint` ok, 0 diagnósticos) y corre (`simulate` seed 42:
  86 entradas del generador frente a 61 del original).
* **Estado publicado:** `app_state()` → `dirty: true`, `proposal.id =
  1793662304738-2f7b5729b3af3d82`, `proposal.state = pending`.
* **Pendiente de la persona:** *Aceptar* o *Descartar* la propuesta desde la app. Si la
  descarta con una nota, la nota vuelve por `app_state()`.
