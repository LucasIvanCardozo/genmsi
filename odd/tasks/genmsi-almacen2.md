# Almacén 2 — la cola en unidades y la fila `ENTER` con las entradas logradas

Trabajo en el worktree `../genmsi-almacen2`, rama `feature/almacen-detalles`.
Cierra las dos diferencias que el arreglo del almacén dejó fuera
(`odd/tasks/notas-almacen.md`, §5).

## Specs

### S1 — La cola de la `StorageUse` cuenta unidades

> "La cola de la StorageUse no cuenta unidades. El original compila
> "QUEUE qAlmacen,2" con Cantidad 2: su cola cuenta UNIDADES, no
> transacciones. El nuestro cuenta transacciones."

La entidad `Queue` cuenta unidades: su contenido, su máximo y su integral
son unidades, no transacciones. El `Cantidad` de la `StorageUse` (y el de las
clases `Queue` y `Depart`, que en el original también lo tienen) es el número
de unidades que la transacción ocupa en la cola.

### S2 — La fila `ENTER` cuenta las entradas logradas

> "La fila ENTER de la tabla de bloques cuenta los intentos, donde el original
> cuenta las ENTRADAS LOGRADAS."

La fila `ENTER` de la tabla de bloques cuenta las entradas que se lograron, no
los intentos. Las transacciones que esperan no están en la fila `ENTER`: están
en el bloque del que vinieron (en la `StorageUse`, la fila `QUEUE`; en un
`Enter` suelto, el bloque anterior).

### S3 — Lo que dice el original

> "El Storage debe estar declarado (esta clase no lo declara)."

> "La propiedad Queue agrega una cola paraaugustar los datos de la espera antes
> de ingresar al storage. si el valor de la propiedad es nulo no se crea la
> cola."

> "Media y desvío indican el tiempo que la transacción permanecerá en el
> storage."

El `macrocode` de la clase `StorageUse` en `Genmsi_20230711.backup/gClasses.xml`
es la evidencia del texto: `ARRIVE [Queue],[Cantidad]` / `ENTER [Storage],[Cantidad]`
/ `DEPART [Queue],[Cantidad]` / `LEAVE [Storage],[Cantidad]`. La cola del
storage lleva el mismo `Cantidad` que el storage.

### S4 — La regla se fija con la respuesta del original, no de oídas

> "Cuando una regla no esté fijada por evidencia, se dice; no se decide de
> oídas."

Cada arreglo se contrasta con la respuesta del original bajo Wine
(`scripts/wine-oracle/run_oracle.sh`), y lo que la evidencia contradice se dice
en las notas (`odd/tasks/notas-almacen2.md`), con el reporte del original a la
vista.

### S5 — El cierre

> "Un test del core que fije las dos reglas (la cola cuenta unidades; la fila
> ENTER cuenta entradas logradas)."

> "El diferencial limpio: los 25 modelos de fixtures/ con el binario anterior y
> con el nuevo dan el mismo reporte (misma semilla), y los dos casos que
> arreglas coinciden con la respuesta del original."

> "`cargo test --workspace` y `cargo clippy --workspace --all-targets -- -D
> warnings` en verde, con los tests subiendo."

## Tasks

| ID | Specs | Qué | Ruta | Commit |
|----|-------|-----|------|--------|
| T1 | S1 | La entidad `Queue` cuenta unidades: contenido, máximo, integral y `Cantidad` en el GPSS | `crates/genmsi-core` | `feat(core)` |
| T2 | S1 | El tiempo medio de la cola es el integral sobre las entradas, como en el original | `crates/genmsi-core` | `feat(core)` |
| T3 | S2 | La fila `ENTER` cuenta las entradas logradas; las que esperan, en el bloque del que vinieron | `crates/genmsi-core` | `feat(core)` |
| T4 | S2, S3 | La fila del storage: `DELAY` = las transacciones que esperan, `RETRY` = 0 | `crates/genmsi-core` | `feat(core)` |
| T5 | S1 | La proyección de la app cuenta las unidades de la cola | `crates/genmsi-app` | `feat(app)` |
| T6 | S5 | Los tests que fijan las dos reglas | `crates/genmsi-core`, `crates/genmsi-app` | `feat(core)`, `feat(app)` |
| T7 | S4, S5 | El diferencial de los 25 modelos y la comparación con el original | — | — |
| T8 | S4, S5 | Las notas y el commit | `odd/tasks` | `docs(odd)` |

## Log

### L1 — El pedido

> "TAREA: cerrar dos diferencias que el arreglo del almacén dejó fuera, y que
> están descritas en odd/tasks/notas-almacen.md (ya en el repo)."

> "QUÉ SON:
> 1. La cola de la StorageUse no cuenta unidades. El original compila
>    "QUEUE qAlmacen,2" con Cantidad 2: su cola cuenta UNIDADES, no
>    transacciones. El nuestro cuenta transacciones.
> 2. La fila ENTER de la tabla de bloques cuenta los intentos, donde el original
>    cuenta las ENTRADAS LOGRADAS. Nuestro RETRY de la fila ENTER debería ser el
>    número de veces que NO se entró, como en el resto de las filas (lo verificó
>    el diferencial de la sesión anterior: la única diferencia entre el binario
>    anterior y el nuevo en los 25 modelos era justo la columna RETRY de las
>    filas ENTER, y ahora coincide con el reporte del original)."

### L2 — La regla del commit

> "NINGÚN commit lleva pie de atribución (nada de "Co-Authored-By" ni similar).
> El proyecto es del lector y los commits no llevan la firma de ninguna
> herramienta ni de ningún modelo. El mensaje del commit explica qué cambió y
> por qué, y nada más."

### L3 — Lo que encontró el original (y corrigió el encargo)

El `RETRY` de la fila `ENTER` del original es **0** en todos sus reportes: los
rechazos no se cuentan en el bloque, y las transacciones que esperan están en
la fila del bloque del que vinieron (la cola de la `StorageUse`) y en el
`DELAY` de la fila del almacén. El encargo pedía que nuestro `RETRY` contara
las veces que no se entró; se siguió al original y se dijo por qué, con los
reportes a la vista (`odd/tasks/notas-almacen2.md`, §2.3 y §2.4).

### L4 — Lo que quedó fuera

La fila `SEIZE` y la fila de la facility siguen contando los intentos: el
original les aplica la misma regla que al `ENTER` (la sonda
`facilidadContencion` lo muestra), pero es otra clase y es otra tarea. También
queda fuera la última transacción de la corrida, que es la T4 de la fase 7.

### L5 — Los cortes de la entrega

La rama `feature/almacen-detalles` entrega el trabajo en tres cortes, cada
uno con lo suyo:

| Corte | Commit | Qué |
|---|---|---|
| 1 | `d4f4e8f` | El núcleo: la cola en unidades, el tiempo medio de la cola, la fila `ENTER` y la del almacén, con sus tests (394 líneas) |
| 2 | `76c2c05` | La evidencia: los seis modelos y los reportes del original |
| 3 | `9596e1e` | La app: la proyección cuenta las unidades |
| 4 | — | Las notas de esto |

El corte 2 es el que se sale del tamaño de una revisión (los logs de Wine
son largos), pero es la evidencia que sostiene al corte 1: sin él, el
cambio no se puede revisar.
