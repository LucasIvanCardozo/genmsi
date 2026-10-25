# genmsi-layout-motor — el motor compone el modelo

La geometría del lienzo deja de ser algo que sólo el archivo puede llevar: el
motor la calcula, la aplica y la guarda. Un modelo sin `x`, `y`, `width` ni
`height` deja de ser un lienzo vacío y pasa a ser un modelo que la app puede
dibujar.

Es la primera unidad del orden que el usuario aceptó: el layout primero, después
las fases 7 y 8 (la verdad del motor), después la 9 (la app viva y lo visual),
y al final el resto del MCP (la superficie de autoría).

## Specs

### S1 — "dale, vamos con tu recomendacion"

El orden de trabajo es el siguiente: el layout al motor (esta fase), después el
cierre de la fase 7 y la fase 8 (la paridad y el reloj), después la fase 9 (la
app viva y las gráficas), y al final la superficie de autoría del MCP
(`add_block`, `connect`, `set_property`), el lint que junta los hallazgos y la
guía del lazo en vivo.

### S2 — "la posible causa de que el MCP no proporcione una posición en pantalla para cada componente"

Un modelo que el agente escribe tiene que tener una posición en pantalla para
cada uno de sus componentes. Hoy no la tiene: la plantilla del catálogo no
trae geometría y `save_model` escribe el modelo tal cual, así que la app dibuja
un lienzo vacío (`crates/genmsi-app/src/graph/graph.ts:95` deja fuera todo
bloque sin `layout`).

### S3 — El motor compone el modelo

La geometría es del motor, no del archivo. Hoy el motor la recibe y la
conserva, pero no la compone: `crates/genmsi-core/src/documents.rs:728` lo
dice literal, *"The geometry comes from the file, not from a layout engine"*.
Esta fase cambia esa frase: el motor compone el modelo cuando su geometría
está incompleta, y el archivo lo guarda como cualquier otra cosa.

### S4 — La composición coloca los bloques que no tienen geometría

El algoritmo de la app (`crates/genmsi-app/src/edit/layout.ts:84`) deja
intacto el bloque que no tiene geometría: por eso el botón "Reacomodar"
tampoco puede arreglar un modelo sin colocar. La composición del motor
coloca **todos** los bloques: el que no tiene geometría recibe la que le
corresponde, con el tamaño por omisión.

### S5 — Un solo algoritmo

El botón "Reacomodar" de la app pide la composición al motor y borra su copia
del algoritmo. No hay dos algoritmos que puedan discrepar: la app y el MCP
ponen el modelo en el lienzo igual.

### S6 — `save_model` compone antes de escribir

El modelo que el MCP escribe sale del `save_model` con la geometría puesta,
así que la app lo dibuja sin que nadie tenga que tocar el archivo a mano. La
composición sólo se aplica cuando la geometría está incompleta: un modelo ya
colocado conserva la disposición que su autor dejó.

La composición **no es un hallazgo**: el documento de lint de un modelo que
compila es `ok` y no tiene hallazgos, componga el motor el modelo o no. Un
cliente que lea `diagnostics` como "el modelo tiene problemas" no puede ser
engañado por la geometría; la descripción de la herramienta dice que la
composición pasa, y `genmsi layout` dice qué compuso.

### S7 — La CLI compone un modelo

`genmsi layout` compone un modelo y dice qué compuso. Sin escribir nada por
omisión: la escritura es explícita (`--write`).

### S8 — La composición es determinista y no toca la semántica

El mismo modelo produce siempre las mismas posiciones. Un modelo compuesto
compila igual y corre igual que el mismo modelo sin componer: la geometría no
entra en la simulación.

### S9 — Un modelo ya colocado no se mueve al guardarlo

Un modelo en el que todos los bloques tienen geometría conserva su
disposición al guardarlo. Sólo se compone lo que está incompleto.

## Tasks

| T | Specs | Qué | Ruta | Commit |
|---|---|---|---|---|
| T1 | S2, S3, S4, S8, S9 | La composición en el motor: `crates/genmsi-format/src/layout.rs`, con sus pruebas | format | `ebe554c` |
| T2 | S5 | El botón "Reacomodar" pide la composición al motor; la copia del frontend se borra | format + app | `f4432d4` |
| T3 | S6, S7 | `save_model` compone antes de escribir y la CLI compone un modelo | mcp + cli | `b5a1b58` |
| T4 | S1, S3 | Los documentos (la decisión, el formato, el MCP, la CLI, la app) y el lazo visual | docs | `dbb3c3b`, `f49ac03` |

## Log

### L1 — El pedido del usuario (verbatim)

> bien, tambien quiero decirte que quiero luego encarar un mejora en la viusual y
> logica del motro (porque encontre unos bugs) y mejoras en cuanto a graficas.
> consideras que primero hagamos lo del mcp o lo otro?

> dale, vamos con tu recomendacion

### L2 — Lo que la sesión del usuario probó

La sesión que el usuario corrió con el MCP nuevo (58 llamadas a `codemode`, 17
`lint` con 7 recusadas) dejó tres cosas claras:

1. El agente **se fue del MCP para escribir el modelo**: `/tmp/build_gmdl.py` y
   `/tmp/patch_hints.py`, dos scripts de Python que generan y parchean el
   `.gmdl` a mano. Las siete recusadas son errores de autoría, no de simulación.
2. El lienzo quedó vacío: el modelo que el agente escribió no tenía geometría
   y la app no dibuja los bloques que no la tienen. El agente lo tapó a mano
   (44 bloques con `x`, `y`, `width` y `height`).
3. El agente guardó en `simulacion/modelocomedor.gmdl` mientras la app tenía
   abierto `simulacion/modelo.gmdl`: la app vigila el archivo que tiene
   abierto, así que nunca vio el trabajo.

### L3 — La corrección de la fase 3b

La fase 3b decidió que las herramientas de mutación no eran el siguiente paso.
La sesión la contradice: la medición no lo mostró porque el agente medido
escribía el XML en el argumento de la herramienta, donde la guía alcanza; el
agente que tuvo que trabajar solo escribió un generador. Las herramientas de
mutación son el siguiente paso, pero van **después** de las fases 7, 8 y 9.

### L4 — Lo que la app hace hoy

- El lazo en vivo existe: la app sondea el archivo que tiene abierto cada
  500 ms (`crates/genmsi-app/src/state/store.ts:73`) y, cuando cambia, lo
  recarga, lo compila y lo corre (`store.ts:763`).
- Al recargar, un modelo que no compila se rechaza: la app conserva el modelo
  anterior y muestra el error (`store.ts:755`).
- El botón "Reacomodar" es del frontend y escribe en memoria; al guardar, la
  geometría llega al archivo.

### L5 — El orden y su porque

El motor primero: la superficie del MCP es delgada y puede esperar, pero
codifica la semántica del motor en su documentación, y codificarla antes de
cerrar la paridad es escribir un contrato sobre un motor que todavía tiene
diferencias medidas. La medición final del producto (fase 8) es el lugar
natural para medir de nuevo al agente: con el motor verificado, un número se
atribuye al MCP y no a la semántica.

### L6 — T1: la composición en el motor (`ebe554c`)

`crates/genmsi-format/src/layout.rs`: los bloques de cada contenedor se
colocan en capas, los bloques sin geometría reciben la que les corresponde
(con el tamaño por omisión de 120 × 48), los contenedores crecen para
contener lo que poseen y la raíz del modelo se deja como está (el editor no
la dibuja).

Las seis pruebas de `crates/genmsi-format/tests/layout.rs`: un modelo sin
geometría queda compuesto; los bloques de una capa se apilan y las capas
avanzan; la composición es determinista; un modelo ya compuesto no se mueve;
un bloque sin geometría recibe la suya; un contenedor crece hasta contener
los bloques que posee.

### L7 — T2: el botón "Reacomodar" pide la composición al motor (`f4432d4`)

La fachada de la app gana la operación `relayout` (el protocolo, la concha de
Tauri, el adaptador y sus dos backedends), el store la llama y la copia del
algoritmo que el frontend tenía (`src/edit/layout.ts` y su prueba) desaparece.

**El lazo visual llevaba 101 commits roto**: la app quitó el desplegable de
modelos (`14e9f1c`, "one run, the system dialog, and no spoilers") y el lazo
seguía seleccionando desde él, así que moría en su primera espera y ninguna
interfaz se había revisado con un navegador desde entonces. Ahora abre los
modelos por los recientes (la única vía que el navegador tiene), espera el
reloj en vez del texto que el monitor ya no lleva, y revisa también la
composición.

El lazo corre entero: 16 capturas, la última de ellas la del monitor con sus
gráficas.

### L8 — T3: `save_model` compone y la CLI compone (`b5a1b58`)

El MCP compone el modelo antes de escribirlo (después de validarlo y compilarlo, como antes): un modelo sin geometría llega al archivo con la geometría puesta; un modelo que ya la tiene se escribe byte a byte como llegó. La composición no es un hallazgo: el documento de lint de un modelo que compila es `ok`, sin hallazgos (lo corrigió la verificación independiente, que señaló que un cliente podía leer `diagnostics` como "el modelo tiene problemas"). La CLI gana `genmsi layout`, que dice dónde quedó cada bloque y no escribe nada, salvo que se lo pidan con `--write` (la escritura es atómica).

Las pruebas: el archivo escrito lleva la geometría de cada bloque y la raíz no, y el modelo compuesto simula igual que el que el agente mandó; un modelo que ya tiene geometría se escribe sin cambios; un modelo que no compila no llega al disco; la CLI compone sin escribir, escribe con `--write` y falla como las demás órdenes.

### L9 — T4: los documentos

La decisión D18 ("el motor compone el lienzo"), la sección de la composición
en `docs/format-and-engine.md`, el botón de la app en `docs/app.md`, los
pasos del lazo en `docs/app-visual-loop.md`, la orden nueva en `docs/cli.md` y
la composición en `docs/mcp.md`. La guía del agente (`skills/`) dice que la
geometría es del motor y que no hay que colocar los bloques a mano.

### L10 — La verificación

- `cargo test --workspace`: 78 suites, 532 pruebas, 0 fallos.
- `cargo clippy --workspace --all-targets -- -D warnings`: limpio.
- El lazo visual: 16 capturas, entero.
- La prueba de extremo a extremo: un modelo de referencia al que se le quitó
  la geometría, compuesto por el motor, se abre en la app con sus 18 bloques
  en el lienzo (antes: un lienzo vacío).
- `genmsi layout --write` dos veces sobre el mismo archivo: el segundo write
  no cambia un byte (el modelo ya está compuesto).
- El plan de `assess` con la escalación del ítem 1 (el `--write` reescribe un
  archivo guardado): el verificador independiente corre sobre la unidad de
  escritura.

### L11 — El riesgo

- T1 (la composición): ninguno. Es cálculo puro y un módulo nuevo.
- T2 (el botón y el lazo): ninguno. La fachada crece con una operación y el
  frontend pierde su copia del algoritmo.
- T3 (la escritura): **ítem 1** (efectos sobre datos: el `--write` reescribe
  un modelo guardado y el `save_model` ahora escribe un modelo compuesto).
  Cubierto por el verificador independiente, por las pruebas de la escritura
  (byte a byte, atómica, idempotente) y por la prueba de que la geometría no
  cambia la semántica.

### L12 — La verificación independiente de la unidad de escritura

El plan de `assess` con la escalación del ítem 1 pidió un verificador independiente sobre la unidad de escritura (`b5a1b58`). Verificó las seis cosas: nada se pierde (el archivo escrito sólo difiere del original en la geometría: 19 módulos, 27 propiedades, 12 links, 23 puertos y 23 etiquetas sobreviven), un modelo que no compila no llega al disco, la escritura es atómica (240.043 lecturas concurrentes, ninguna parcial), `--write` es idempotente, la geometría no cambia la semántica (el GPSS y los reportes salen byte a byte iguales) y ningún bloque se superpone con otro del mismo contenedor.

Sus tres avisos se corrigieron:

- **A1**: una escritura que falla a mitad de camino dejaba el archivo temporal
  (`.modelo.gmdl.tmp`) en el directorio. Ahora las dos escrituras (la del MCP
  y la de la CLI) lo borran cuando la escritura falla.
- **A2**: `--write` reescribía el archivo entero (con la forma canónica del
  motor) aunque no hubiera compuesto nada. Ahora un modelo que no necesita
  composición **no se escribe**: la orden no toca un archivo que no necesita
  toque.
- **A3**: la nota de la composición convertía el documento de lint en
  `diagnostics`, y un cliente podía leerlo como "el modelo tiene
  problemas". La nota se fue: la composición no es un hallazgo.

### L13 — Lo que la fase no hace

- No publica una geometría en el catálogo: la composición es del motor y no
  de la clase.
- No toca la semántica: el motor compone, el simulador no se entera.
- El lazo visual abre los modelos por los recientes, que es la única vía que
  el navegador tiene; la app de escritorio sigue usando el diálogo del
  sistema.
