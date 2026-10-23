# Notas: la cola en unidades y la fila `ENTER` con las entradas logradas

Sesión `feature/almacen-detalles` (worktree `../genmsi-almacen2`).
Cierra las dos diferencias que `odd/tasks/notas-almacen.md` (§5) dejó fuera.

Todo lo que sigue está medido: el texto del original sale de
`Genmsi_20230711.backup/gClasses.xml` y de corridas del `genmsi.exe` bajo Wine
(`scripts/wine-oracle/run_oracle.sh`). Los modelos están en
`scripts/wine-oracle/probes/almacen/` y la respuesta del original, en
`scripts/wine-oracle/probes/almacen/oracle/`.

## 1. Los modelos de prueba

Seis modelos pequeños, con la forma del banco `StorageUse.gmdl` (un
`Generate`, la clase, un `Terminate` y un `Clock` de 3600):

| Modelo | Qué ejercita |
|---|---|
| `almacenCola2.gmdl` | `StorageDef` de 4 unidades + `StorageUse` con `Queue=qAlmacen` y `Cantidad=2` |
| `almacenCola2T1.gmdl` | lo mismo con `Tamaño=1`: dos unidades no entran en un almacén de una |
| `almacenEnter.gmdl` | la clase `Enter` suelta (sin cola) con `StorageDef` de 1 unidad |
| `colaCantidad2.gmdl` | la clase `Queue` con `Cantidad=2` |
| `colaCantidad2Sale.gmdl` | `Queue` con `Cantidad=2` seguida de `Depart` con `Cantidad=2` |
| `facilidadContencion.gmdl` | la clase `FacilityUse` con la facility ocupada (servicio de 600) |

Todos con `Generate 30,0` (determinista) y `Clock 3600`.

## 2. Lo que dice el original

### 2.1 La cola cuenta unidades

El `macrocode` de la clase `StorageUse` en `gClasses.xml`:

```
ARRIVE  [Queue='qAlmacen'],[Cantidad]
ENTER   [Storage='Almacen'],[Cantidad]
DEPART  [Queue],[Cantidad]
ADV     [Media='10'],[Modificador='5']
LEAVE   [Storage],[Cantidad]
```

La cola lleva el mismo `Cantidad` que el storage. Y la cuenta es de
**unidades**, no de transacciones: el GPSS que el original compila es
`QUEUE qAlmacen,2` / `DEPART qAlmacen,2`, y el reporte de `almacenCola2` dice:

```
QUEUE            MAX  CONT. ENTRIES ENTRIES(0) AVE.CONT. AVE.TIME AVE.(-0) RETRY
  QALMACEN       216   216     120         2   105,300   3159,000 3212,542    0
```

120 transacciones de 2 unidades son **216** lugares (el máximo), no 120: la
cola cuenta unidades. `ENTRIES` (120) sí cuenta las transacciones.

Lo mismo en la clase `Queue`, que también tiene `Cantidad`
(`ARRIVE [Queue],[Cantidad]`): `colaCantidad2` da `QUEUE QRecurso,2` y una
cola de 240 lugares para 120 transacciones.

### 2.2 El tiempo medio de la cola es el integral sobre las entradas

El `AVE.TIME` del original es el **integral de la cola dividido por sus
entradas**, no el promedio de las esperas de las que ya se fueron. Se
comprueba en las cuatro filas de cola de `TP3`:

| Cola | `AVE.CONT` | área (= `AVE.CONT` × 3600) | `AVE.TIME` del original | área / entradas |
|---|---|---|---|---|
| `QENTREGAEQUIPAJE` | 118,216 | 425.578 | 1576,207 | 1576,216 |
| `QBUFFET` | 6,835 | 24.606 | 820,233 | 820,200 |
| `QCAJA1FREESHOP` | 0,000 | 0 | 0,000 | 0 |

y en las sondas: `almacenCola2` da 425.578/120 = 3159,000 y
`almacenCola2T1` da 428.400/120 = 3570,000.

Nuestro motor sumaba las esperas de las transacciones que **ya se fueron** de
la cola: en `colaCantidad2` (nadie sale nunca de la cola) daba 0, cuando el
original dice 3570,000.

### 2.3 La fila `ENTER` cuenta las entradas logradas

El reporte del original de `almacenCola2`:

```
 LINE    ENTRY CURRENT RETRY            LOC  BLOCK_TYPE
    3      120     108     0     StorageUse  QUEUE           qAlmacen,2
    4       12       0     0              3  ENTER           almacen,2
    5       12       0     0              4  DEPART          qAlmacen,2
```

- La fila `ENTER` cuenta **12**: las entradas que se lograron (el almacén
  dice `ENTRIES 12`).
- Su `CURRENT` es **0** y su `RETRY` es **0**: ni los intentos ni los
  rechazos.
- Las 108 transacciones que esperan están en la fila **anterior**, la del
  `QUEUE` (`CURRENT 108`).

En `TP3` lo mismo: `27 270 184 0 StorageUse2 QUEUE qEntregaEquipaje` y
`28 86 0 0 22 ENTER entregaEquipaje` (86 = el `ENTRIES` del almacén).

Y con un `Enter` suelto, sin cola (`almacenEnter`), las que esperan están en
la fila de donde vinieron, que es el `GENERATE`:

```
    2      121     120     0              1  GENERATE        30,0
    3        1       0     0        A_Enter  ENTER           almacen,1
```

### 2.4 La fila del almacén: `RETRY` 0 y `DELAY` con las que esperan

```
STORAGE           CAP. REMAIN. MIN.  MAX. ENTRIES AVL. AVE.C.  UTIL. RETRY DELAY
  ALMACEN           4      0     0     4      12   1   3,950   0,988    0   108
```

`RETRY` es 0 (el original **no** cuenta los rechazos en ningún lado de la
tabla de bloques ni en la del almacén) y `DELAY` son las 108 transacciones
que esperan. En `TP3`: `ENTREGAEQUIPAJE 12 0 0 12 86 1 9,089 0,757 0 184`.

> Esto contradice el encargo ("Nuestro RETRY de la fila ENTER debería ser el
> número de veces que NO se entró"). En todos los reportes del original que
> tenemos a la vista —`TP3` y las seis sondas, con hasta 120 rechazos— el
> `RETRY` de la fila `ENTER` es **0**. Se siguió al original: el `RETRY` de
> la fila `ENTER` queda en 0 y los rechazos se ven en el `DELAY` del
> almacén.

## 3. Lo que se cambió

1. **La entidad `Queue` cuenta unidades** (`crates/genmsi-core/src/stat.rs`):
   `content` y `max` son unidades; `enqueue`/`dequeue` reciben las unidades;
   el integral es unidades × tiempo. `entries` sigue contando las
   transacciones que entraron.
2. **El `Cantidad` llega a la cola** (`program.rs`): `QueueSpec.units` pasó a
   ser una expresión, y la `StorageUse` (y las clases `Queue` y `Depart`, que
   en el original también lo tienen) compila su `Cantidad` a la cola y al
   `DEPART`. El GPSS escribe `QUEUE qAlmacen,2` / `DEPART qAlmacen,2`, como el
   original.
3. **El tiempo medio de la cola** es el integral sobre las entradas
   (`average_time`, `average_time_nonzero`): la fórmula del original. El campo
   `total_time` del JSON sigue siendo la suma de las esperas de las que se
   fueron (el contrato publicado no se toca).
4. **La fila `ENTER` cuenta las entradas logradas** (`sim.rs`): el bloque ya
   no cuenta los intentos en su `ENTRY COUNT`, y la transacción que espera se
   cuenta en el bloque del que vino (el `QUEUE` de la `StorageUse`, el bloque
   anterior en un `Enter` suelto). El `RETRY` de la fila queda en 0.
5. **La fila del almacén**: `RETRY` en 0 y `DELAY` con las transacciones que
   esperan.
6. **El reporte** escribe las unidades de la cola y del storage solo cuando
   el modelo las da (`QUEUE qAlmacen,2`, `ENTER almacen,2`, `ENTER
   Migracion`), que es como los escribe el original.
7. **La app** cuenta las unidades de la cola también
   (`crates/genmsi-app/src/monitor/`).

## 4. El diferencial

Los 25 modelos de `fixtures/models`, con el binario anterior y con el nuevo
(misma semilla), cambian **solo** en lo que el arreglo manda:

- las filas `ENTER` (las entradas logradas) y las filas `QUEUE` (las que
  esperan);
- el `AVE.TIME` y el `AVE.(-0)` de las colas;
- el `RETRY` y el `DELAY` de los almacenes;
- los operandos de las filas `ENTER`/`LEAVE` (`ENTER Migracion,1` →
  `ENTER Migracion`, como los escribe el original).

Nada más: ni los tiempos, ni los generadores, ni las facilities, ni el resto
de las filas. Los nueve modelos sin almacenes ni colas (o que no compilan) no
cambian ni un número.

Contra los reportes del original que están en `fixtures/outputs`, las
métricas que compara el oráculo suben de 876 de 956 a 887 de 956:

| Modelo | Antes | Ahora |
|---|---|---|
| `TP2__problema1` | 56/57 | 57/57 |
| `TP3__aeropuerto` | 83/91 | 85/91 |
| `TP4__supermercado` | 57/65 | 59/65 |
| `TP6__Clase_6_1_PuertaBanco` | 8/13 | 9/13 |
| `TP8__mecanicos` | 108/116 | 109/116 |
| `monorriel__monorriel` | 47/55 | 52/55 |
| `ejercicio_4__ejercicio_4` | 25/29 | 24/29 |

(`cargo run --release -p genmsi-core --example oracle -- --runs 3`.)

## 5. Lo que quedó fuera (y por qué)

- **La última transacción de la corrida.** El original procesa los eventos de
  la hora en que termina la simulación; nosotros perdemos el último
  arriveso, y por eso nuestras cuentas salen en 119 donde las del original
  salen en 120 (las diferencias de `AVE.TIME` vienen de ahí: el área es la
  misma, `105,300` en los dos, y lo que cambia es el divisor). Es la T4 de
  `odd/tasks/genmsi-fase7-revision-clase-por-clase.md`, ya conocida.
- **La fila `GENERATE`**: el original cuenta en su `ENTRY COUNT` las
  transacciones creadas *más* la que está agendada (121 donde el nuestro dice
  120), y su `CURRENT` suma las que esperan a la creación agendada. También
  es la T4 de la fase 7.
- **La fila `SEIZE` y la fila de la facility**: el original les aplica la
  misma regla que al `ENTER` (la sonda `facilidadContencion` da
  `3 6 0 0 SEIZE Recurso` con 114 transactions esperando, y la facility
  `RECURSO 6 0,992 595,000 1 7 0 0 0 114`: el `PEND` en 0 y las que esperan
  en el `DELAY`). Nuestro motor sigue contando los intentos en la fila
  `SEIZE` y las que esperan en el `PEND` de la facility. No se tocó: es otra
  clase y otra tarea.
- **Un `StorageDef` sin `Tamaño`** vale 0 unidades (el original diría "El
  parámetro B no debe ser nulo"). Hoy el modelo compila y nada entra nunca.
- **El `Parcial 8-11-21`** declara `acreditacon` y usa `acreditacion`: el
  original lo rechaza con "Error en linea 9 -> Bloque inválido", pero esa línea
  es la del `QTable` "tiempo demora" (el nombre con espacio), no la del
  `ENTER`; el modelo igual no compila en nuestro motor por el `QTable`.

## 6. Cómo reproducir las corridas del original

```sh
export WINEPREFIX=$PWD/scripts/wine-oracle/wineprefix DISPLAY=:99
bash scripts/wine-oracle/run_oracle.sh \
  scripts/wine-oracle/probes/almacen/almacenCola2.gmdl \
  scripts/wine-oracle/probes/almacen/oracle
cat scripts/wine-oracle/probes/almacen/oracle/almacenCola2.report.txt
```

Y las nuestras, con la misma semilla:

```sh
cargo run --release -p genmsi-cli -- simulate --seed 7 \
  scripts/wine-oracle/probes/almacen/almacenCola2.gmdl
```

## 7. Los tests

`crates/genmsi-core/tests/storages.rs`:

- `the_queue_of_a_storage_counts_units`: el contenido de la cola son dos
  lugares por transacción que espera.
- `the_enter_row_counts_the_entries_that_got_in`: la fila `ENTER` cuenta las
  entradas del almacén, su `CURRENT` es 0, su `RETRY` es 0, y las que esperan
  están en la fila de la cola.
- `a_lone_enter_counts_its_waiters_where_they_came_from`: un `Enter` sin cola
  cuenta las que esperan en el bloque anterior.

`crates/genmsi-core/src/stat.rs`:

- `a_queue_counts_units_and_not_transactions`
- `the_average_wait_is_the_integral_over_the_entries`

`crates/genmsi-app/tests/stats.test.ts`:

- `a queue counts the units its transactions carry`
