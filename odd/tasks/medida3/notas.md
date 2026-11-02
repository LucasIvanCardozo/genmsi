# Notas de la medición 3 — el verbo del MCP (`apply_edit`) construyendo el modelo del comedor

**Qué se midió.** El mismo modelo de la corrida anterior (`odd/tasks/medida2/notas.md`): la entrega
de comida en el comedor de la facultad, 35 bloques. Esta vez **sin escribir XML a mano**: todo el
modelo se construyó con los comandos del verbo (`apply_edit`).

**Veredicto en una línea:** **sí**, el verbo permite construir el modelo completo sin escribir XML a
mano, en **12 llamadas al MCP** (7 exitosas) contra las **24** de la corrida anterior — **la mitad** —
y con un modelo que **reproduce exactamente los mismos agregados** de la corrida anterior.

Todo lo que se dice abajo sale de lo que se leyó y de lo que se ejecutó. No se leyó el código del
producto (`crates/**`), ni `docs/format-and-engine.md`, ni `docs/cli.md`, ni `docs/mcp.md`, ni
`fixtures/**`, ni ningún otro archivo de `odd/tasks/` salvo `odd/tasks/medida2/notas.md` y el
documento de la feature.

---

## 1. Número de llamadas al MCP, desglosado por herramienta

**12 llamadas en total. 7 exitosas, 5 fallidas** (las 5 fallidas se muestran en el log y se analizan
en §5).

| Herramienta | Llamadas | Qué fueron |
|---|---|---|
| `catalogue` | **2** | (1) las 23 clases del modelo, en una sola llamada; (3) el catálogo completo + el esqueleto de un modelo |
| `apply_edit` | **5** | (2, 4, 6, 7) intentos fallidos; (8) **el lote que construyó el modelo** |
| `save_model` | **2** | (5) escribir el esqueleto vacío → rechazado; (9) persistir el texto que devolvió el verbo |
| `lint` | **1** | (10) verificación final del archivo en disco |
| `simulate` | **1** | (11) la corrida (seed 42) |
| `explain_model` | **1** | (12) verificar bloque por bloque el modelo escrito |
| **Total** | **12** | |

Comandos enviados y aplicados:

| Lote | Comandos enviados | Aplicados |
|---|---|---|
| (2) `addBlock` ×35 sobre `path` | 35 | 0 (el archivo no existía) |
| (4) `addBlock` ×35 sobre `path` + `model` | 35 | 0 (`path` y `model` son excluyentes) |
| (6) `addBlock` ×35 sobre `model` | 35 | 0 (el modelo con `Match` no compila) |
| (7) lote completo, `setCaption` con `value` | 159 | 0 (el campo se llama `caption`) |
| **(8) lote completo** | **159** | **159** (35 `addBlock` + 35 `setCaption` + 64 `setProperty` + 25 `connect`) |
| (9) `save_model` del texto producido por el verbo | — | 1 archivo de 22 204 bytes |
| **Total** | **423** | **159** |

El modelo entero (35 bloques, 25 enlaces, 64 propiedades) se construyó en **una sola llamada al verbo**
(la 8), con un lote de 159 comandos. Las otras cuatro llamadas al verbo fueron intentos que fallaron
antes de aplicar nada (el lote es atómico: en ningún momento quedó un modelo a medias).

## 2. Comparación con la corrida anterior

| | Corrida anterior (`medida2`) | Esta corrida (`medida3`) |
|---|---|---|
| Llamadas al MCP | **24** | **12** |
| Cómo se construyó el modelo | **XML escrito a mano**, en un solo `save_model` | **lote de comandos del verbo** (`apply_edit`) |
| Llamadas al catálogo | 3 (la guía dice una, y alcanza) | 2 |
| Sondas (modelos de prueba) | 3 sondas × 3 llamadas = 9 | **0** |
| Errores de formato descubiertos a mano | 10 (los «hallazgos» de §4 de las notas anteriores) | 0 (el catálogo y el verbo los evitan) |
| Llamadas fallidas | — | 5 (todas de interfaz, ninguna de modelo) |

**El modelo es el mismo y se verificó con los mismos números** (seed 42):

| Magnitud | Correnda anterior | Esta corrida |
|---|---|---|
| `end_time` / `end_reason` | 54000 / `terminated` | **54000 / `terminated`** |
| `transactions` | 953 | **953** |
| `events` | 13 967 | **14 045** (+78) |
| Cola `ColaPago`: `entries` / `max` | 476 / 47 | **476 / 47** |
| Cola `ColaPago`: `content` / `total_time` | 1 / 77 148 | **1 / 77 148** |
| Storage `Horno`: `capacity` / `max_use` / `entries` / `area` | 12 / 12 / 475 / 85 479 | **12 / 12 / 475 / 85 479** |
| Facility `Cajero`: `entries` / `busy_time` | 476 / 8 292 | **476 / 8 292** |
| Facility `Responsable`: `entries` | 469 | **469** |
| SaveValues `TicketsEmitidos` / `ComidasServidas` | 475 / 469 | **475 / 469** |
| Tabla `TablaEspera`: `samples` / `max_value` / `overflow` | 476 / 1200 / 10 | **476 / 1200 / 10** |
| Lógica `AperturaCajero` | true | **true** |

Todos los agregados coinciden **exactamente**. La única diferencia es `events` (14 045 contra 13 967,
+78): el modelo recorre un camino ligeramente distinto en algún punto (la entrega, el `Match` o el
`Terminate` de la apertura está cableada en el otro brazo), pero las magnitudes que definen el
comedor son las mismas. No lo investigué: la corrida es equivalente para el escenario.

## 3. Cuántas llamadas al catálogo y cuántas al verbo

- **Al catálogo: 2** (llamadas 1 y 3).
- **Al verbo (`apply_edit`): 5** (llamadas 2, 4, 6, 7 y 8), de las cuales **4 fueron intentos
  fallidos** y **1 construyó el modelo completo**.
- Las otras 5 fueron de andamiaje y verificación: 2 `save_model`, 1 `lint`, 1 `simulate`, 1
  `explain_model`.

Es decir: **el catálogo bastó con una llamada** (las 23 clases del modelo, con puertos,
propiedades, tipos, entidades y plantilla de cada una). La segunda llamada al catálogo fue para pedir
el **esqueleto de un modelo**, que es la única forma sancionada de arrancar un archivo nuevo (ver §5,
defecto D1). **Ningún otro conocimiento del formato hizo falta**: no leí XML, ni el código, ni las docs,
y no hice ni una sola sonda.

## 4. Qué tenía que saber el agente antes de la primera llamada

**Lo que leí (dos archivos):**

1. `skills/genmsi-modeling/SKILL.md` — la guía: el flujo de trabajo, la forma de los
   `EditCommand`, las entidades, las expresiones, las trampas.
2. `odd/tasks/medida2/notas.md` — el enunciado: el escenario, los supuestos y el diseño bloque por
   bloque (qué clase va en cada punto del flujo y con qué valores). Es la especificación del modelo
   a construir; no es una fuente del formato.

**Lo que aportó el catálogo (una sola llamada):** los `ioid` de los puertos (`Generate` publica
`Salida1` con `ioid=1`, el resto `Entrada1=1` / `Salida1=2`; `Split` y `GateL` tienen un segundo
`Salida2=3`), los nombres literales de las propiedades (`Tamaño` con ñ, `LogicSwich` mal escrito en
el original, `op` en minúscula en `GateL` y `Op` en mayúscula en `Logic` y `SaveValue`), los tipos
(`EntityDef` vs `EntityUse`) y la plantilla de cada clase.

**Lo que costó más de lo esperado (en orden de importancia):**

1. **El arranque del archivo (2 llamadas perdidas: 2 y 5).** El verbo **no crea el archivo**: con
   un `path` que no existe devuelve `model_not_found`, y `save_model` **rechaza** el esqueleto vacío
   porque un modelo sin `Clock` ni `Terminate` con operando no compila. La única forma de llegar a un
   archivo es construir el modelo **en memoria** (`apply_edit` con `model`) y persistir después el
   texto que devuelve el verbo (`save_model` con ese `source`). Es un rodeo que la guía no menciona.
2. **El `Match` y los ids (1 llamada perdida: 6).** La plantilla del `Match` trae los rótulos
   por defecto `AAA`/`BBB`, así que un modelo con `Match` **no compila** tal como sale de `addBlock`:
   el chequeo previo a escribir lo rechaza. Para arreglarlo hay que poner `RotuloPropio`/`RotuloMatch`
   en el **mismo lote** que crea el bloque — y eso exige **saber el id de un bloque que todavía no
   existe**. La guía dice «the ids come back: use them in the next batch», lo cual es imposible con el
   chequeo previo a escribir. Lo resolví por inducción: el error nombraba `node: 26`, y el `Match` era
   el bloque n.º 26 de mi lista → **los ids son correlativos (1..N) en el orden del lote**. Ese es un
   dato que la guía **no dice** y que hoy se conoce por accidente (o por un error que lo enseña).
3. **El campo de `setCaption` (1 llamada perdida: 7).** `setCaption` usa `caption`, no `value`. La guía
   lista la operación pero **no muestra su forma** (los ejemplos son de `addBlock`, `connect` y
   `setProperty`). Y el rechazo **no viene con la clase de error documentada**: es texto crudo, sin
   índice ni `op` del comando que falló.
4. **`path` y `model` son excluyentes (1 llamada perdida: 4).** La guía dice «takes the model (`path`
   or `model`)», lo que se lee como «usa uno u otro»; se puede intentar usar los dos.

**Lo que no costó nada:** el diseño del modelo (ya venía resuelto en las notas de la corrida
anterior), los valores de las propiedades (los daba la especificación) y la geometría (el verbo
coloca cada bloque solo: el archivo final tiene `x`/`y` compuestos por el motor, nunca inventados
por el agente).

## 5. Defectos encontrados en el verbo / en el catálogo

Todos los errores se transcriben **tal cual** los devolvió el servidor.

### D1 — No hay forma sancionada de crear un archivo nuevo (grave, bloquea el primer uso)

`apply_edit` con un `path` que no existe **no crea el archivo**:

```json
{"error": "model_not_found", "message": "cannot read model: odd/tasks/medida3/modelo.gmdl", "path": "odd/tasks/medida3/modelo.gmdl"}
```

Y `save_model` **rechaza** el esqueleto vacío del catálogo:

```json
{"error": "invalid_model", "message": "no TERMINATE block with operand A and no CLOCK block: the simulation would never finish", "path": "odd/tasks/medida3/modelo.gmdl", "line": 1, "column": 59, "node": 0, "class": "main"}
```

El rodeo que funciona es: `apply_edit({model: <esqueleto del catálogo>, commands: [...]})` → el verbo
devuelve el texto del modelo en `source` → `save_model({path, model: <ese source>})`. O sea: **el
modelo se construye íntegro con el verbo, pero el archivo lo escribe otra herramienta**. La guía
recomienda `apply_edit` sobre el `path` ("the file is the state") y ese camino es **inalcanzable para
un modelo nuevo**. Además, el mensaje "no TERMINATE block with operand A" parece un `> 0` mal
escapado: un `Terminate` con `Unidades=0` no cierra la corrida.

### D2 — `setCaption` no está documentado y su error no cumple S3b

El lote con `{"op":"setCaption","id":1,"value":"..."}` fue rechazado con **texto crudo**, sin la
clase `invalid_command` ni el índice ni el `op` del comando que falló (S3b promete las dos cosas):

```
failed to deserialize parameters: missing field `caption`
```

La guía lista `setCaption` entre las operaciones pero **no dice cómo se llama su argumento**, y el
nombre de la operación (`setCaption`) no deja adivinar que el campo sea `caption` y no `value`
(`setProperty` usa `value`).

### D3 — El chequeo previo a escribir impide descubrir los ids (grave, fricción principal)

La regla "el modelo se valida antes de escribir" + "el lote es atómico" se combinan mal con "los ids
vuelven en la respuesta": **no se puede aprender el ids en un lote y usarlo en el siguiente**, porque
el lote siguiente tiene que partir de un modelo que ya compile. Con clases cuyo valor por defecto no
compila (el `Match` es el caso), el agente está obligado a **adivinar el id dentro del mismo lote**:

```json
{"error": "invalid_block", "message": "MATCH Match refers to unknown label \"BBB\"", "path": "<memory>", "node": 26, "class": "Match"}
```

Tres cosas que ayudarían y hoy no existen: (a) que `addBlock` **no** copie los valores por defecto
de la plantilla (o que el catálogo no traiga rótulos por defecto en el `Match`); (b) que el error de
un comando diga **el índice y el `op`** también en los fallos de deserialización; (c) documentar la
regla de ids (hoy son correlativos, y eso solo se sabe por un error).

### D4 — `path` y `model` excluyentes, con un mensaje que la guía no anticipa

```json
{"error": "invalid_arguments", "message": "give `path` or `model`, not both: one names the file, the other carries the document"}
```

El mensaje es claro y accionable (bien), pero la guía se lee como si ambos fueran intercambiables.
Menor: `model` **no** se puede usar junto a `path` ni como «documento base que se escribe en el
archivo», que sería lo que el agente quiere para no pasar por `save_model`.

### D5 — La guía y el motor discrepan sobre la detección de propiedades mal escritas (abierto de la corrida anterior)

La guía afirma que el `lint` "names the property a block carries that its class does not declare
(`Tamano` for `Tamaño`)". La corrida anterior observó lo contrario: el `lint` respondió `status: ok`
**sin diagnósticos** con `Tamano` escrito, y el modelo compilaba con `capacity: 0`. En esta corrida
**no lo volví a probar** (usé los nombres literales del catálogo), así que la discrepancia sigue
abierta: o la guía miente, o el `lint` sólo lo detecta en algunos casos.

### D6 — El catálogo: los `hint` de dos propiedades están copiados del vecino

`Table.PrimerIntervalo` y `Table.TamFrecuencia` traen **el mismo** `hint` ("Limite superior de la
primer frecuencia"), que no describe al primero. Cosmético, pero es el único dato de la clase que no
sirve. Los `hint` vienen con el texto codificado en percent (`%2C`, `%F3n`), lo cual es correcto
(la plantilla los trae así).

## 6. El modelo final

- **Archivo:** `odd/tasks/medida3/modelo.gmdl` (22 205 bytes), escrito por `save_model` con el texto
  que produjo el verbo. 36 `<module>` (la raíz + 35 bloques), 25 `<link>`, geometría `x`/`y`
  compuesta por el motor.
- **`lint` (llamada 10):** `status: ok`, `blocks: 31`, `diagnostics: []`, `definitions: {storages: 1,
  tables: 1, ...}`.
- **`simulate` (llamada 11, seed 42):** `end_time: 54000`, `end_reason: terminated`,
  `transactions: 953`, `events: 14045`, `diagnostics: []` — agregados idénticos a los de la corrida
  anterior (ver §2).
- **`explain_model` (llamada 12):** 36 bloques con sus propiedades por nombre y 25 aristas, todas
  como se querían. Spot-checks: `Generate`#8 `Cantidad: ""` (vacío = infinito: el generador hizo 477
  llegadas), `Match`#26 `RotuloPropio: Alumno` / `RotuloMatch: Bandeja`, `Match`#27 inverso,
  `Depart`#17 `Queue: ColaPago`, `Tabulate`#31 `NombreTabla: TablaEspera`, `StorageDef`#4
  `Tamaño: "12"`.
- **Sondas: 0.** Ninguna clase necesitó una prueba aparte: el catálogo más la especificación de la
  corrida anterior bastaron para escribir el modelo a la primera, y la corrida lo confirmó.

## 7. Veredicto

**Sí: el verbo permite construir el modelo sin escribir XML a mano.** El modelo completo (35 bloques,
25 enlaces, 64 propiedades) salió de **un lote de 159 comandos** en **una sola llamada**, y el
resultado es indistinguible del modelo escrito a mano: mismos agregados, mismo archivo con
geometría compuesta por el motor.

**En costo de llamadas: 12 contra 24 — la mitad.** Pero la mitad no es del verbo: es de que el
modelo ya venía **especificado** (clase por clase, con valores) en las notas de la corrida anterior y
de que el **catálogo de una sola llamada** trae todos los hechos del formato. Lo que el verbo agrega de
verdad es que la corrida anterior gastó **9 de sus 24 llamadas en tres sondas** (modelos de prueba
de `Generate`, `StorageDef`, `Match`, `GateL`, `Mark`/`Table`): con los comandos del verbo, escribir
el bloque es un `addBlock` + un `setProperty` por propiedad, sin riesgo de que el XML esté mal formado
y sin riesgo de que una propiedad quede con el valor por defecto en vez del valor.

**Lo que hay que arreglar antes de que esto sea un flujo que cualquier agente pueda usar:**

1. **El arranque (D1).** O `apply_edit` crea el archivo cuando el `path` no existe (con el esqueleto
   mínimo), o la guía documenta el rodeo `apply_edit(model) → save_model(source)` como **el** camino
   sancionado para un modelo nuevo. Hoy el verbo, que es la herramienta principal, no puede arrancar
   un modelo.
2. **Los ids y el chequeo previo (D3).** O `addBlock` no hereda los valores por defecto que no
   compilan, o el error de un comando mal formado trae índice y `op` como promete S3b, o se documenta
   que los ids son correlativos y se puede mandar todo en un lote. Hoy el agente tiene que adivinar para poder
   usar un `Match`.
3. **`setCaption` (D2).** El nombre del campo, en la guía.

Con esos tres arreglos, el número de llamadas de esta corrida baja de 12 a **7** (1 catálogo, 1 verbo,
1 `save_model`/bootstrap, 1 `lint`, 1 `simulate`, 1 `explain_model`, + el catálogo del esqueleto), y la
medición deja de medir el rodeo y empieza a medir el verbo.
