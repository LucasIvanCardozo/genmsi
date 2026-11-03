# Feature: la semántica que el motor no cuenta — qué le hace cada clase a la simulación

El catálogo de GenMSI dice **qué** es cada clase: su nombre, sus puertos,
sus propiedades, la entidad que declara o usa, y el XML de un bloque de esa
clase. Con eso un agente construye un modelo sin adivinar un solo nombre de
elemento — y con eso construyó 517 bloques en la sesión `chat`.

Lo que el catálogo **no** dice es **qué le hace esa clase a la simulación**:
si la transacción espera, qué recurso toma, qué se libera cuando termina,
qué pasa con la cola, y qué trampa tiene la clase.

El costo fue brutal: **más de la mitad** de la sesión `chat` fue sondeo
empírico (ocho modelos de prueba en `/tmp`) para deducir reglas que un
lector del enunciado debería poder leer en el catálogo:

| Lo que el agente asumía (GPSS) | Lo que hace GenMSI |
|---|---|
| `DEPART` funde al pasajero en el flujo del bus | Sólo descuenta la cola; el pasajero sigue su propio flujo |
| `UNLINK` saca transacciones de una cadena | Exige una etiqueta de puerto que `apply_edit` no puede asignar |
| `LOOP n` itera n veces | salida **ioid 3 = repetir**, **ioid 2 = salir** |
| Terminar una transacción libera lo que tenía | No lo libera: el recurso queda tomado para siempre |
| `SEIZE` de un recurso libre = "quedarse esperando" | No bloquea: el primero que llega se lo lleva y sigue de largo |
| `GATEL` libera de a uno | Libera a todos de una vez |

Cada fila es una regla que el motor **ya tiene escrita en su código** y que
nadie publicó. Esta feature la publica, clase por clase.

## Specs

S1 (la devolución de la sesión `chat`, textual): "1. **`semantics` por clase
en el catálogo**, con cuatro campos obligatorios: *efecto sobre el flujo*,
*qué bloquea y qué no*, *reglas de toma/liberación de recursos* y
*gotchas*. Ej.: `Release: libera el recurso y activa exactamente un
esperante; el recurso queda tomado por ese esperante; la transacción que
termina no libera lo suyo`. Esto solo habría ahorrado la mitad de la
sesión."

S2 (la devolución de la sesión `chat`, textual): "El motor tiene un defecto
propio: Assemble y Gather no liberan el lote pese a lo que dice el
catálogo." (una divergencia con el original, ya medida)

S3 (decisión del usuario, textual): "Las 66 clases completas, pero aqui
usaremos subagentes especializados para no llenarnos de contexto
innecesario".

S4 (D19): "One zone of truth: the engine publishes, the app invents nothing."

S5 (D13): "within one major version a change is additive only".

S6 (D11): "Every tool addition must justify itself against that goal."

S7 (D4 de la fase 0, el oráculo): el motor se mide contra el **original**:
los 25 modelos de referencia y sus salidas grabadas
(`fixtures/outputs/`), y las divergencias que quedan están **fijadas por
tests** (`crates/genmsi-core/tests/diferencias_*.rs`).

### S1 — el contrato de cada clase

El documento `genmsi.class/1` (el que devuelve `catalogue` con una clase)
gana un campo `semantics`: el contrato de la clase, en cuatro partes.

| Parte | Qué dice | Ejemplo |
|---|---|---|
| **flow** | Qué le hace al flujo de la transacción: qué puerto sigue, cuál se pierde, cuál espera. | "La transacción sigue por `Salida1`; la que no encuentra lugar espera en la cola de la instalación." |
| **blocking** | Qué bloquea y qué no: si la transacción espera, en qué, y qué la suelta. | "Espera en la cola mientras la instalación está ocupada; el recurso no se toma hasta que hay lugar." |
| **resources** | Las reglas de tomar y soltar: qué toma, qué suelta, qué pasa cuando la transacción termina. | "Toma el recurso y lo suelta solo con un `Release` explícito: terminar la transacción **no** lo suelta." |
| **gotchas** | Las trampas, con la divergencia con el original cuando la hay. | "El original saca las transacciones de la cadena con el `Unlink`; el motor no libera el lote." |

El texto es **en inglés**: el catálogo y el glosario ya lo están, y es el
idioma del motor.

El campo es **aditivo** (D13): un documento que no lo lleva sigue siendo
válido, y un lector que no lo mira no cambia.

### S2 — la verdad es la del motor, y el original es el judge

El contrato describe **lo que el motor hace**, no lo que hace GPSS. Cuando
el motor se aparta del original, el contrato lo dice: es una divergencia
medida, fijada por un test (`diferencias_fase8.rs`,
`diferencias_restantes.rs`).

El oráculo ya está corrido: `cargo test -p genmsi-core --test oracle`
compara la salida del motor con la salida grabada del original en los
modelos que corren rápido, y los tests de divergencias fijan lo que queda
fuera. **La adjudicación ya está hecha**: el contrato se escribe leyendo el
motor, y el oráculo dice si lo que dice el motor es lo que hacía el
original.

Lo que **no** se hace: canonizar el sondeo empírico de la sesión `chat`. Un
agente que dedujo "SEIZE no bloquea" mientras el glosario del motor dice
"the transaction waits until it is free" no es una fuente: es una hipótesis.

### S3 — un subagente por familia de clases

Las 66 clases se reparten en **seis familias**, y cada familia la escribe un
subagente que **sólo** lee lo que necesita (el motor, los modelos de
referencia que la usan, las salidas grabadas del original) y **sólo** escribe
el contrato de sus clases. El orquestador no lee los contratos: los revisa,
los unifica y los publica.

| Familia | Clases |
|---|---|
| **F1 · flujo** | Advance, Clock, Depart, Enter, Generate, Leave, Priority, Queue, Split, Terminate, Transfer |
| **F2 · recursos** | Buffer, Examine, FacilityInt, FacilityUse, GateF, GateL, GateS, Release, Seize, StorageDef, StorageUse |
| **F3 · grupos y cadenas** | Assemble, Gather, JoinMe, JoinNumber, Link, Match, RemoveMe, RemoveNumber, RemoveTrans, Return, Unlink |
| **F4 · control** | Alter, Counter, ExamineNumber, GateM, Loop, Mark, Preempt, Scan, Select, SelectF, Test |
| **F5 · selección y datos** | Assign, InitiaSV, InitialLS, Logic, Note, QTable, SaveValue, SelectL, SelectS, SelectSNA, Tabulate |
| **F6 · funciones y variables** | Function, FunctionDist, FunctionExpon, MFunction, MFunctionDist, Multifurcacion, MultifurcacionP, Table, Variable, VariableA, VariableN |

### S4 — el motor es el dueño del texto

El contrato vive en el **núcleo** (`genmsi-core`), junto al glosario y al
catálogo: `genmsi_core::semantics::contract(class)` devuelve el texto de una
clase, y el documento del catálogo lo publica. La app no lo inventa y el MCP
no lo copia: los dos leen el núcleo.

### S5 — el documento

`genmsi.class/1` gana `semantics` (aditivo). `genmsi.classes/1` (el
documento de varias clases) lo gana también, porque viaja con cada clase.

### S6 — el conjunto de herramientas no crece

No hay ninguna herramienta nueva: el contrato viaja en el catálogo, que el
agente ya llama antes de escribir un modelo.

### S7 — cada regla que se puede falsificar, con un test

Un contrato que dice "espera" o "libera" o "toma" es una afirmación sobre el
motor, y cada afirmación que se pueda comprobar lleva un test que la
comprueba (con el patrón de `crates/genmsi-core/tests/`: un modelo mínimo,
una corrida, una aserción). Un contrato sin test es una opinion: se escribe
igual, pero se marca como tal.

## Tasks

| ID | Task | Specs | Route | Commit |
|---|---|---|---|---|
| T1 | `genmsi-core`: el módulo de las semánticas (`genmsi-core/src/semantics.rs`) con el texto de las 66 clases, en las cuatro partes | S1, S2, S4 | worker (6 subagentes, uno por familia) | 1 |
| T2 | `genmsi-core` + `genmsi-mcp`: el catálogo publica el contrato (`genmsi.class/1` y `genmsi.classes/1`) | S1, S5, S6 | inline | 2 |
| T3 | `genmsi-core`: los tests que comprueban las reglas que se pueden falsificar | S7 | worker | 3 |
| T4 | `skill` + `docs/mcp.md`: el catálogo dice más, y la guía lo dice | S1, S6 | inline | 4 |
| T5 | close: `cargo test --workspace`, clippy, y una medición con agente en frío: un agente que **no** sondea deduce las reglas de las clases que usa | S1-S7 | inline | 4 |

## Log

L1 (2026-11-02, usuario, textual): "Las 66 clases completas, pero aqui
usaremos subagentes especializados para no llenarnos de contexto
innecesario".

L2 (2026-11-02, análisis, el oráculo ya corrió): el motor está medido contra
el original. `cargo test -p genmsi-core --test oracle` compara con las
salidas grabadas (`fixtures/outputs/`, 20 reportes válidos) y los tests de
divergencias (`diferencias_fase8.rs`, `diferencias_restantes.rs`) fijan lo
que se aparta: el orden del listado (la Original camina en profundidad), la
fila de la multifurcación, el generador que espera en una puerta y el orden
de las tablas. **La adjudicación no hay que hacer: ya está hecha.** Lo que
queda es escribir lo que el motor hace.

L3 (2026-11-02, análisis, lo que el sondeo de la sesión NO puede canonizar):
la sesión `chat` dedujo "SEIZE de un recurso libre no bloquea" y el glosario
del motor dice "Seizes the facility Facility; the transaction waits until it
is free". Una de las dos está mal y el sondeo no dice cuál. El contrato se
escribe **leyendo el motor** (que es el judge del oráculo), no copiando el
sondeo.

L4 (2026-11-02, análisis, dónde vive el texto): el glosario
(`genmsi-core/src/glossary.rs`) ya lleva 51 frases de una línea por clase
("Qué hace la clase"), y el catálogo las publica en `description`. El
contrato de cuatro partes es el **hermano largo** de esa frase: vive al
lado, en el núcleo, y lo publica el mismo documento.

L5 (2026-11-02, análisis, el hueco que queda): el catálogo publica
`declares` y `uses` (qué entidad declara o usa cada clase), pero no dice
**qué** es esa entidad ni quién la crea. La cadena del `Unlink` es el
ejemplo: la crea el uso (GPSS la crea por uso y así la deja el chequeo de
declaraciones), y `Cadena` es el nombre con el que el agente la nombra.

L10 (2026-11-02, el método de los seis subagentes, T1): el módulo de las
semánticas es `crates/genmsi-core/src/semantics/`: `contract.rs` (la forma
del contrato), `mod.rs` (la tabla que junta las familias y la función
`contract(class)`) y un archivo por familia. Los seis subagentes **no se
pisan**: cada uno escribe el archivo de su familia y nada más.

El método que sigue es el que cada subagente recibe:

1. **Dónde está la verdad, en este orden**: (a) el motor —
   `crates/genmsi-core/src/program.rs` (el `BlockKind` de la clase y el
   `expand` que la compila: una clase que son varias sentencias lo dice) y
   `crates/genmsi-core/src/sim.rs` (la ejecución: qué le pasa a la
   transacción, si espera, qué toma); (b) el original — los
   `fixtures/models/*.gmdl` son modelos de la materia y los
   `fixtures/outputs/*.report.txt` son **las respuestas grabadas del
   original**; el motor está medido contra ellos, así que lo que hace el
   motor es lo que hacía el original, salvo lo que fijan las divergencias;
   (c) las divergencias — `crates/genmsi-core/tests/diferencias_fase8.rs` y
   `diferencias_restantes.rs`: si la clase aparece ahí, el `gotchas` lo dice;
   (d) la guía — la sección *Runtime semantics worth knowing* ya dice
   varias reglas en las palabras del agente: se usan (son ciertas) y se
   agrega lo que falta.

2. **El texto**: en inglés (el catálogo y el glosario lo están), una o dos
   frases por parte, en el orden del catálogo. `flow` (qué le hace al flujo),
   `blocking` (qué bloquea y qué no), `resources` (las reglas de tomar y
   soltar) y `gotchas` (las trampas y la divergencia con el original; vacío
   si no hay trampa).

3. **Las reglas**: se dice **lo que hace el motor**, no lo que hace GPSS; no
   se adivina (una parte que no se puede determinar queda vacía y se dice en
   el informe); no se especula sobre el original (sus respuestas están
   grabadas); los nombres son los del motor (`StorageDef.Tamaño`,
   `Advance.Media`), para que el agente los pueda buscar en el catálogo.

4. **El reparto**: F1 flujo (Advance, Clock, Depart, Enter, Generate, Leave,
   Priority, Queue, Split, Terminate, Transfer); F2 recursos (Buffer,
   Examine, FacilityInt, FacilityUse, GateF, GateL, GateS, Release, Seize,
   StorageDef, StorageUse); F3 grupos y cadenas (Assemble, Gather, JoinMe,
   JoinNumber, Link, Match, RemoveMe, RemoveNumber, RemoveTrans, Return,
   Unlink); F4 control (Alter, Counter, ExamineNumber, GateM, Loop, Mark,
   Preempt, Scan, Select, SelectF, Test); F5 selección y datos (Assign,
   InitiaSV, InitialLS, Logic, Note, QTable, SaveValue, SelectL, SelectS,
   SelectSNA, Tabulate); F6 funciones y variables (Function, FunctionDist,
   FunctionExpon, MFunction, MFunctionDist, Multifurcacion, MultifurcacionP,
   Table, Variable, VariableA, VariableN).

L11 (2026-11-02, T1, las seis familias): cada familia la escribió un
subagente que leyó el motor (el `expand` que compila la clase y su
ejecución) y la contrastó con las respuestas grabadas del original. Lo que
quedó resuelto, y era lo que la sesión `chat` había adivinado por sondeo:
- `Seize` de un recurso ocupado **sí** bloquea (la transacción espera en la
  cadena de retraso del recurso);
- **terminar una transacción no libera lo que tenía**: el recurso queda
  tomado hasta un `Release` explícito;
- el `Loop` repite por su **puerto 3** (`Salida2`, el epígrafe `loop`) y sale
  por el puerto 2 (`Salida1`), y con el puerto de repetición sin conectar el
  bucle corre **una vez, sin error**;
- una función **no recibe argumento**: la variable independiente es el
  `Atributo` del propio bloque;
- un `Select` con `ClaseSNA=SV` **recorre los almacenamientos**, y el
  prefijo que lee un save value es `X:`;
- la cadena la crea el uso y la nombra la propiedad `Cadena`.

Dos descripciones de una línea del propio motor **minten** y quedaron
corregidas: el `Counter` no cuenta las llegadas de un grupo (suma
`Incremento` a un save value y copia la cuenta a un parámetro) y el
`ExamineNumber` no espera (comprueba y bifurca).

Y una afirmación del worker resultó falsa: el catálogo **sí** declara
`Variable.Expresion` y `MFunctionDist.Resultados` (las propiedades se declaran
en la especificación de la clase, no en la tabla de valores por defecto).

L12 (2026-11-02, T2, la publicación): el documento de una clase lleva ahora
el contrato (`semantics`), y el catálogo lo publica. El campo es aditivo: un
lector que no lo mira no cambia, y una clase sin contrato lo dice en vez de
inventar uno.

L13 (2026-11-02, T3, lo que falta): las reglas **falsificables** de los
contratos necesitan un test que las compruebe. Con lo que hay hoy, el
oráculo mide el motor contra el original en los modelos de referencia, y los
tests de divergencias fijan lo que se aparta; lo que falta es un test por
regla del contrato que se pueda Falsear. Queda como la primera tarea de la
siguiente sesión.
