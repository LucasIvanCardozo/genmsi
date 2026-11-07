# Feature: las manos del agente — el verbo y la traza que se pueden usar a escala

El MCP de GenMSI publica nueve herramientas y el motor hace lo que GPSS
hace. Lo que le falta al agente no son herramientas: es **poder usar las que
hay** a la escala de un modelo real.

La medición con agente en frío (`odd/tasks/medida4/`) y la devolución de la
sesión `chat` coinciden en lo mismo: un agente que construye un modelo de
500 bloques tiene que adivinar cosas que el motor ya sabe.

- un lote de 1300 comandos que falla dice "el bloque 9 no tiene el puerto de
  entrada 1": **ids**, donde el agente tenía clases, propiedades y nombres
  lógicos;
- no hay forma de **ensayar** un lote sin escribirlo;
- la traza es una manguera: sin filtros, el agente tuvo que volcar 1000+
  eventos y filtrarlos en el cliente;
- la traza numera **statements compilados**, no bloques del documento: sin un
  mapa, "qué bloque es el 218" es ingeniería inversa;
- el dialecto de expresiones (`@` es módulo, `#` es división entera, `P:`,
  `X:`, `RN2`...) vive en la gramática del motor, en un archivo que el
  agente no lee.

Esta feature da al agente las manos que le faltan, sin agregar **ninguna
herramienta**: el conjunto sigue siendo de nueve.

## Specs

S1 (la devolución de la sesión `chat`, textual): "Un `dry_run` o un error con
\"clase + propiedad + tag\" habría detectado ambos en segundos".

S2 (la devolución de la sesión `chat`, textual): "Faltaría filtrar por
clase/bloque/transacción/ventana de tiempo y un modo \"resumen\" (cuántas
entradas por bloque, primer y último tiempo)".

S3 (la devolución de la sesión `chat`, textual): "No se expone el mapa
**statement ↔ módulo**: la traza numera *statements* compilados, no bloques
del documento, así que no se puede leer \"qué bloque es el 218\" sin
ingeniería inversa del orden de compilación".

S4 (la devolución de la sesión `chat`, textual): "El dialecto numérico no
está documentado en ningún lado. `MOD(...)` no existe (`@` es módulo, `#`/`*`
es multiplicación), las funciones **no aceptan argumento** (la variable
independiente es `Atributo` del bloque `Function`)".

S5 (decisión del usuario, textual): "vamos a seguir con todo eso que quedo en
la cola en esta sesion mejor!!".

S6 (D11): "Every tool addition must justify itself against that goal."

S7 (D13): "within one major version a change is additive only".

S8 (D19): "One zone of truth: the engine publishes, the app invents nothing."

### S1 — el ensayo y el error que nombra

`apply_edit` y `save_model` aceptan un `dry_run` (booleano, por defecto
falso): el lote se aplica, el modelo se compila y **no se escribe nada**. La
respuesta es la misma que la de verdad (`genmsi.edit/1` / `genmsi.lint/1`),
con `written: false` y el `written` del documento diciendo que no se
escribió. El motor ya compila antes de escribir: el ensayo es el mismo
camino sin la escritura.

El error de un comando nombra **lo que el agente escribió**, no el número
interno: la **clase** del bloque, la **propiedad** que faltó o no existe, y
el **nombre lógico** del bloque (el `caption`, que es como el agente lo
nombró; si no lo nombró, el `name` de la clase). El índice del comando y su
`op` siguen ahí: el lote es la fuente de verdad y el agente lo reenvía.

El error se produce en el motor, que es donde se sabe la clase y la
propiedad: la herramienta lo formatea.

### S2 — la traza que se puede filtrar

`get_trace` acepta filtros, y el filtro se aplica **en el motor, al
registrar el evento**, no después: si el límite de registros se contara antes
del filtro, el agente pediría los eventos de una clase y recibiría el
principio del relato. El límite cuenta **los eventos que pasan el filtro**.

Los filtros:

| Filtro | Qué selecciona |
|---|---|
| `class` | Los eventos de una clase de bloque (`"Counter"`, `"Seize"`...). |
| `node` | Los eventos de un bloque del modelo (por su `moduleid`). |
| `xact` | Los eventos de una transacción. |
| `from` / `until` | La ventana de tiempo (el `until` ya existe como horizonte de la corrida: el filtro es otra cosa). |

Y un **resumen**: `get_trace` con `summary: true` devuelve, por bloque, cuántas
entradas tuvo, el primer y el último tiempo y las transacciones que lo
tocaron — sin los eventos. Es la pregunta que el agente se hace el 90% de
las veces ("¿cuántas veces pasó por acá?").

El documento de la traza sigue siendo `genmsi.trace/1` y gana los campos que
dicen qué se filtró (aditivos).

### S3 — el mapa statement ↔ bloque

Hoy la traza numera **statements compilados** (`TraceEvent.block` es el índice
en `program.blocks`) y el reporte numera lo mismo con base 1 (`JsonBlock.line`
= índice + 1). El módulo del documento se llama `node` en el reporte y en el
explain, y **no aparece en la traza**.

El mapa que se publica:

- el **evento de la traza** gana `node`: el `moduleid` del bloque del
  documento del que sale el statement;
- el **documento del explain** publica, por cada statement del programa, su
  índice, su `node` y la **posición en el archivo** (línea y columna del
  bloque en el `.gmdl`), que hoy se detiene en la capa del formato.

Con eso la pregunta "qué bloque es el 218" tiene una respuesta de una línea:
el evento trae el `node`, el explain trae la línea y la columna.

La posición viaja desde el formato: el compilador la copia al statement
(`ExecBlock`), y de ahí a los documentos. Es aditivo.

### S4 — el dialecto de expresiones, publicado

El dialecto ya está escrito: es la gramática del motor
(`crates/genmsi-core/src/expr/grammar.pest`), que nombra los prefijos
(`P:`, `X:`, `Q:`, `S:`, `CH:`, `FN:`, `V:`, `RN2`, `AC1`, `[param]`), los
operadores (`+ - * /`, `#` división entera, `@` módulo) y las reglas (un
porcentaje es un valor por mil, un nombre pelado es un parámetro).

Se publica donde el agente lo lee:

- la **guía** (`skill/genmsi-modeling/SKILL.md`) lleva la tabla corta con los
  equivalentes exactos: es la pieza que el agente lee siempre;
- el **catálogo** publica el dialecto junto al esqueleto del modelo: el
  `catalogue` sin argumentos ya lleva el `skeleton`, y ahí cabe una tabla de
  los prefijos.

Ni una herramienta nueva: los dos textos viajan en documentos que ya
existen.

### S5 — el canal de la cadena: lo que la medición Sly dice

La devolución de la sesión `chat` decía: "El `Unlink` exige una etiqueta de
puerto que `apply_edit` no puede asignar → clase inutilizable desde el MCP".

Eso es **un diagnóstico equivocado**, y la feature lo corrige con un hecho:
la cadena no es una etiqueta de puerto sino el valor de la propiedad
`Cadena`, y el agente ya puede ponerlo con `setProperty`. El modelo de
referencia `TP7__Clase 7_1_Link_Unlink.gmdl` nombra su cadena `Cadena`, y
los dos bloques (`Link` y `Unlink`) la nombran igual.

Lo que faltaba no era el comando: era **decirlo**. El catálogo publica, por
cada clase, qué entidad nombra cada propiedad (`declares`/`uses` ya lo
hacen, pero no dicen que `Cadena` nombra una cadena ni que la cadena la crea
el uso). Y la guía lo dice.

Antes de escribir nada, la feature **comprueba** el hecho con el motor: un
modelo con un `Link` y un `Unlink` que nombran la misma cadena corre.

### S6 — la cola de la sesión, sin crecer

Nada de esto agrega una herramienta: el conjunto sigue siendo de nueve
(`catalogue`, `lint`, `simulate`, `get_trace`, `explain_model`, `save_model`,
`apply_edit`, `open_model`, `app_state`). Los filtros de la traza, el
`dry_run` y el `summary` son **campos opcionales** de herramientas que ya
existen.

### S7 — documentos versionados

`genmsi.trace/1` gana los campos del filtro (aditivos). `genmsi.class/1`
gana el dialecto (aditivo). El documento del explain gana el mapa (aditivo).
La traza gana `node` (aditivo).

### S8 — un solo dueño del formato

El filtro de la traza se aplica en el motor (que es donde se registra cada
evento) y la herramienta sólo pasa los filtros. El dialecto vive en el
núcleo (que es donde está la gramática) y las dos publicaciones lo leen de
ahí. El mapa lo publica el núcleo.

## Tasks

| ID | Task | Specs | Route | Commit |
|---|---|---|---|---|
| T1 | `genmsi-mcp`: `dry_run` en `apply_edit` y `save_model` | S1 | inline | 1 |
| T2 | `genmsi-mcp` + `genmsi-format`: el error de un comando nombra la clase, la propiedad y el nombre lógico | S1 | inline | 1 |
| T3 | `genmsi-core` + `genmsi-mcp`: los filtros de la traza (en el motor) y el modo resumen | S2 | worker | 2 |
| T4 | `genmsi-core` + `genmsi-mcp`: el mapa statement↔bloque (el `node` en la traza, el índice y la posición en el explain) | S3 | worker | 3 |
| T5 | `genmsi-core` + `skill`: el dialecto publicado (el catálogo y la guía) | S4 | inline | 4 |
| T6 | la comprobación de la cadena: un modelo con `Link` y `Unlink` que nombran la misma cadena, y el catálogo que lo dice | S5 | inline | 4 |
| T7 | close: `cargo test --workspace`, clippy, los tests del frente, y una medición con agente en frío del lote de un modelo real | S1-S7 | inline | 5 |

## Log

L1 (2026-11-02, usuario, textual): "vamos a seguir con todo eso que quedo en
la cola en esta sesion mejor!!".

L2 (2026-11-02, decisión del usuario, textual): "Las 66 clases completas,
pero aqui usaremos subagentes especializados para no llenarnos de contexto
innecesario" — es para la feature de la semántica
(`odd/tasks/genmsi-semantica.md`).

L3 (2026-11-02, análisis, el mapa): el dialecto de expresiones **ya está
escrito** en `crates/genmsi-core/src/expr/grammar.pest` (los prefijos, los
operadores `#` y `@`, el porcentaje por mil, el nombre pelado). La devolución
de la sesión decía que "no está documentado en ningún lado": está escrito,
pero en un archivo que el agente no lee. Publicarlo es un trabajo de dos
lugares, no de investigación.

L4 (2026-11-02, análisis, el mapa de la traza): `TraceEvent` lleva `time`,
`xact`, `block` (el índice del statement compilado), `class`, `label`, `event`
y `detail`. El documento lleva `records`, `limit` y `truncated`. **No hay
ningún filtro**: el CLI lo dice en su propio comando ("The trace has no
filters (decision of the coordinator: minimum surface)"). El límite se
aplica en el motor, al registrar (`sim.rs:1113`).

L5 (2026-11-02, análisis, el mapa del mapa): la relación existe y no se
publica con el mismo nombre: `report.blocks[].line - 1 == trace.events[].block`
(las dos son el índice en `program.blocks`), y `program.blocks[i].node` es el
`moduleid` del módulo. La **posición** (línea y columna del `.gmdl`) se
detiene en la capa del formato: `ExecBlock` no la copia, así que ni el
reporte ni la traza la tienen. El mapa se publica uniendo las tres piezas.

L6 (2026-11-02, corrección del orquestador, la cadena): el diagnóstico de la
sesión `chat` sobre el `Unlink` era equivocado. La cadena **no** es una
etiqueta de puerto (la etiqueta `A_Queue` es la etiqueta de *conexión*, que
es el `LOC` del statement): es el valor de la propiedad `Cadena`, y el
modelo de referencia `TP7__Clase 7_1_Link_Unlink.gmdl` la nombra `Cadena` en
los dos bloques. El agente puede ponerla con `setProperty`. La cadena la
crea el uso: el catálogo de declaraciones deja los tipos que GPSS crea por
uso (la cadena, la cola, el recurso) fuera del chequeo. La feature no
agrega ningún comando: **dice** lo que ya se puede hacer (T6).

L7 (2026-11-02, corrección del orquestador, el ensayo): el `dry_run` es
**sólo** de `apply_edit`. El ensayo de un documento que el agente escribió
**ya existe**: es `lint` con el documento como texto, que lo compila y no
escribe nada. Poner un `dry_run` en `save_model` obligaría al documento del
lint a decir si se escribió — un campo que no le corresponde — para volver a
decir lo mismo.

L8 (2026-11-02, T1 y T2, inline): `dry_run` en `apply_edit` (el destino
`Trial` del motor: el lote se aplica y compila, y no se escribe nada; la
respuesta lleva el modelo en `source`, porque ya no hay archivo que lo
guarde) y el error que nombra: el cuerpo de la falla gana `name` (el nombre
que el agente le dio al bloque) y `property` (la propiedad del comando), y
el mensaje los nombra también. El motor sabe el nombre del bloque y su
clase; el comando sabe la propiedad. Los tres tests nuevos: el ensayo no
escribe, el ensayo de un lote que falla falla, y la falla nombra lo que el
agente escribió.

L9 (2026-11-02, análisis, el dialecto: TERCER diagnóstico equivocado de la
sesión `chat`): la devolución decía "El dialecto numérico no está
documentado en ningún lado". **La guía lo documenta**, con una tabla
completa (los prefijos, los operadores, el porcentaje por mil, el nombre
pelado) en su sección *Expressions*. El agente la leyó (la leyó primero, en
la sesión) y de todos modos loDeclaró ausente: el problema no era que
faltaba, sino que nadie lo decía en el lugar donde se lo busca.

Y encima la guía y la gramática **se contradecían**: el comentario de la
gramática decía que `#` es "división entera" y el parser dice
`"#" => BinOp::Mul` (multiplicación) — la guía tenía razón y la gramática
mintiendo. El comentario de la gramática quedó corregido.

El arreglo que de verdad importa no es otra tabla: es que **la tabla la
publica el motor** y que un test la compare con la gramática, para que no
puedan discrepar (T5).

L10 (2026-11-02, T3 y T4, el núcleo): el filtro se aplica **al registrar el
evento** (`record_trace`), así que el límite cuenta lo que pasa el filtro: el
agente que pide los eventos de una clase recibe esos eventos, no el principio
del relato. El resumen (las entradas de cada bloque, su primer y último
tiempo) se acumula **fuera de la traza**: no lo corta el límite y se calcula
con la traza apagada. El evento de la traza lleva el `node` (el `moduleid`
del bloque del documento) y el statement lleva la posición de su bloque en el
`.gmdl`: la pregunta "qué bloque es el 218" tiene respuesta de una línea.

L11 (2026-11-02, T3 y T4, el MCP): `get_trace` toma los filtros (`classes`,
`nodes`, `xact`, `from`, `to`) y el `summary`; el documento de la traza
publica el filtro que usó; y el documento del explain lleva el **mapa**: una
fila por statement, con el `node` del bloque del documento, el **nombre que
le dio el agente** y su línea y columna. El `until` es el horizonte de la
corrida y el `from`/`to` son la ventana del filtro: son dos cosas y la
descripción lo dice.

L12 (2026-11-02, el defecto que encontró la familia F1): un `Depart` de una
cola en la que la transacción nunca entró **hacía caer al motor**: la salida
cuenta una espera cero y la cola no tiene ninguna entrada de la que restarla,
así que la resta se iba por debajo de cero y el motor entraba en pánico al
escribir el reporte. La resta es saturada ahora, y dos tests lo fijan. Un
modelo que una persona puede escribir no puede tirar el motor.

L13 (2026-11-02, la guía estaba atrasada y la medición lo dijo): la guía no
mencionaba el ensayo, ni los filtros de la traza, ni el resumen, ni el mapa;
su tabla de comandos listaba 8 de las 12 operaciones; y decía que un nombre
de propiedad mal escrito es silencioso, lo cual es cierto **por la vía de la
escritura a mano** y falso por la vía de la edición (`setProperty` falla y
nombra la propiedad y el bloque). Todo corregido, y la guía dice ahora que la
semilla hay que pasarla explícita (la que se dibuja al azar hace que dos
corridas "iguales" no sean la misma).

L14 (2026-11-02, T5 y T6, la guía y la cadena): la cadena se nombra con la
propiedad `Cadena` (el modelo de referencia `TP7__Clase 7_1_Link_Unlink` la
nombra `Cadena`), la crea el uso, y **no** es la etiqueta de conexión
(`A_Queue`): lo comprobé de punta a punta — renombré la cadena del modelo de
referencia con `setProperty` y la simulación dio los mismos números (51, 3,
3, 52). Y el comentario de la gramática decía que `#` es "división entera"
cuando el parser dice `"#" => BinOp::Mul`: la guía tenía razón y la
gramática mintiendo; el comentario quedó corregido.

L15 (2026-11-02, la medición en frío, `odd/tasks/medida5/`): 32 llamadas
reales contra la superficie nueva. El agente dejó el cambio como propuesta
sin tocar el archivo del lector (el md5 del archivo es el mismo antes y
después), provocó tres fallos a propósito y los tres lo nombraron sin tocar
el código, usó los filtros, el resumen y el mapa. Lo que encontró:
- **el motor acepta un operando numérico que no es un número en silencio**
  (`Media="no-es-un-numero"` compila y la simulación cambia): un `dry_run`
  que dice "ok" sobre eso da una falsa confianza. Es el hueco más serio que
  quedó;
- **sin la app corriendo ninguna herramienta devuelve el veredicto**: el
  estado de la propuesta vive en el documento que va al lado del modelo, y
  ahora la respuesta dice dónde está (`proposal.document`);
- el resumen no trae el nombre del bloque: hay que cruzarlo con el mapa, y
  el cruce (por el número de statement) ahora está documentado;
- un `XDG_DATA_HOME` **relativo** se descarta en silencio y el MCP pasa a
  hablar con la app real de la máquina: documentado en `docs/mcp.md`.

L16 (2026-11-02, T7, la verificación diferencial, contra el commit base
`b11d219`): el MCP de la base y el de la rama, compilados los dos y
comparados con las mismas llamadas. **Las mismas respuestas** en `lint` con
un archivo, `simulate` con un archivo, `lint` y `simulate` con un
documento, y `lint` con las dos cosas. Y **diferencias sólo donde las
pedimos**: el documento de la traza lleva el filtro y el `node`; el del
explain lleva el `index`, la línea y la columna; la llamada que no nombra
ni archivo ni documento pasó de `invalid_arguments` a `no_open_model`; la
falla de un comando nombra el bloque; el catálogo publica los contratos y
tres descripciones que mentían; y la herramienta `open_model` ya no está.
El script vivió en un `mktemp -d`, fuera del repositorio: es la prueba de
un cambio, no una prueba del producto.

L17 (2026-11-02, T7, el cierre): `cargo test --workspace` verde, clippy del
workspace limpio, el shell (26) y el frente (398) verdes, el typecheck
limpio, y la medición en frío de `odd/tasks/medida5/`.

**Lo que quedó para la siguiente sesión**, y es lo más valioso que salió
de todo esto:
1. **el motor acepta un operando numérico que no es un número en
   silencio** (`Media="no-es-un-numero"` compila y la simulación cambia):
   un `dry_run` que dice "ok" sobre eso da una falsa confianza. Es una
   validación de operando que falta en el motor;
2. **los tests de las reglas falsificables** de los contratos (T3 de la
   feature de la semántica);
3. la **semántica** de las 66 clases, ya escrita y publicada: lo que falta
   es la medición con un agente que **no** sondea (la prueba de que
   sirve);
4. el **puente de desarrollo** (el modo del navegador) no publica el
   estado vivo: el agente sigue sin ver la app en ese modo;
5. la propuesta **sin la app corriendo**: el veredicto vive en el documento
   al lado del modelo y la respuesta dice dónde, pero ninguna herramienta lo
   devuelve: un `proposal_state` sería la respuesta natural.
