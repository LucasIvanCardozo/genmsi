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
| T1 | S2, S3, S4, S8, S9 | La composición en el motor: `crates/genmsi-format/src/layout.rs`, con sus pruebas | format | pendiente |
| T2 | S5 | El botón "Reacomodar" pide la composición al motor; la copia del frontend se borra | format + app | pendiente |
| T3 | S6, S7 | `save_model` compone antes de escribir y la CLI compone un modelo | mcp + cli | pendiente |
| T4 | S1, S3 | Los documentos (la decisión, el formato, el MCP, la CLI, la app) y el lazo visual | docs | pendiente |

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
