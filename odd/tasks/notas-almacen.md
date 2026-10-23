# Notas: un almacén que se comporta como en el original

Hallazgos de la sesión `feature/almacen` (worktree `../genmsi-almacen`).
Todo lo que sigue está medido: el texto del original sale de `gClasses.xml`
y de corridas del `genmsi.exe` bajo Wine (`scripts/wine-oracle/run_oracle.sh`).

## 1. Lo que dice el original

`Genmsi_20230711/gClasses.xml`, clase `StorageUse`:

- `ClassDescrip`: "Las transacciones ingresan a un storage por un sierto tiempo y
  salen. Si el storage esta lleno, aguardan haciendo cola. **El Storage debe
  estar declarado (esta clase no lo declara)**. ... Se debe utilizar otro módulo
  que defina la capacidad del Storage."
- macrocode: `ARRIVE [Queue='qAlmacen'],[Cantidad]` / `ENTER [Storage],[Cantidad]`
  / `DEPART [Queue],[Cantidad]` / `LEAVE [Storage],[Cantidad]`.

Clase `StorageDef`:

- `ClassDescrip`: "Delcara un storage."
- macrocode: `STORAGE [Storage:'Storage a Declarar'='Almacen'],...` con
  `[Tamaño:'Capacidad del Storage'='10']`.

O sea: **el `Tamaño` es la capacidad en unidades** y **la `Cantidad` son las
unidades que toma (y devuelve) una transacción**. El `Storage` tiene que estar
declarado por un `StorageDef`; el `StorageUse` no lo declara.

## 2. Lo que hace el original (oracle)

### 2.1 Un almacén sin declarar

Banco `Enter.gmdl` (`Generate -> Enter AStorage,1 -> Terminate`, sin
`StorageDef`), corrido bajo Wine:

```
Tiempo de simulación: 34,  Transacción activa: 1
Bloque actual    :    1,                  GENERATE        60,30
Siguiente bloque :    2,  A_Enter         ENTER           AStorage,1
ERROR!!Referencia a una entidad storage inexistente. (Storage: -1).
T. Nº 1
```

El original **abre** el modelo, pero la simulación se detiene en el `ENTER`
con "Referencia a una entidad storage inexistente". Su GPSS generado es

```
                GENERATE        60,30
A_Enter         ENTER           AStorage,1
A_Terminate     TERMINATE
                GENERATE        3600
                TERMINATE       1
```

sin ninguna línea `AStorage STORAGE ...`: la declaración no existe.

Consecuencia para el motor: **no puede inventarse una capacidad**. Nuestro
motor seaba `UNDECLARED_STORAGE_CAPACITY = 1_000_000`, así que todo entraba en
paralelo y el usuario veía una simulación plausible y equivocada.

### 2.2 `Tamaño` y `Cantidad`

Tres modelos con la misma forma (`Generate(30) -> StorageUse -> Terminate`, más
un `Clock` de 3600), corridos en el original:

| `Tamaño` | `Cantidad` | GPSS del original | `STORAGE` en el reporte |
|---|---|---|---|
| 1 | 1 | `almacen STORAGE 1` / `ENTER almacen,1` | `CAP. 1  REMAIN. 0  MIN. 0  MAX. 1  ENTRIES 6  AVE.C. 0,992  UTIL. 0,992  RETRY 114` |
| 2 | 1 | `almacen STORAGE 2` / `ENTER almacen,1` | `CAP. 2 ... MAX. 2  ENTRIES 12  RETRY 108` |
| 1 | 2 | `almacen STORAGE 1` / `ENTER almacen,2` | `CAP. 1 ... MAX. 0  ENTRIES 0  RETRY 120` |

Lecturas:

- `Tamaño` 1 con `Cantidad` 1 → **una instancia dentro a la vez** (`MAX.` 1) y
  las demás esperando (los 114 rechazos quedan en el `RETRY` del *almacén*).
- `Tamaño` 2 con `Cantidad` 1 → dos instancias dentro a la vez (`MAX.` 2).
- `Tamaño` 1 con `Cantidad` 2 → **nadie entra**: dos unidades no caben en un
  almacén de una unidad (`MAX.` 0, `ENTRIES` 0, los 120 intentos rechazados).

### 2.3 El `RETRY` va en el almacén, no en el bloque

En el reporte original de la primera corrida, la fila del bloque `ENTER` tiene
`RETRY` 0 y el `RETRY` 114 está en la sección `STORAGE`. Nuestro motor lo
sumaba en los dos sitios.

### 2.4 La regla del `START`

El `Frecuencia` **no** es del `StorageDef`: es de la clase `Clock`
("Descuenta 1 unidad del START de la simulación cada un cierto tiempo
indicado, simulando un reloj"). Su macrocode es `MAKE [Frecuencia:...='3600']` /
`END 1`, y el GPSS que genera es `GENERATE 3600` + `TERMINATE 1`.

En las tres corridas de arriba el reporte original termina con
`START_TIME 0` y `END_TIME 3600`: la unidad del `START` se gasta cada
`Frecuencia` unidades de tiempo, y el reporte lo refleja en `END_TIME`.

### 2.5 Consultas sobre un almacén que no existe

- `SelectS.gmdl` (`SELECT SNF param,1,10`, sin `StorageDef`): el original
  **elige** unstorage (`ENTRIES` 59, todas las transacciones siguen al
  `TERMINATE`) y **no crea ninguno** (`STORAGES 0` en el encabezado).
- `GateS.gmdl` (`GATE SNF Almacen`, sin `StorageDef`): la compuerta **pasa**
  (57 entradas, todas al `TERMINATE`) y tampoco crea el almacén.

O sea: un almacén que no existe "no está lleno" y "no tiene contenido"; sólo
el uso (ENTER/LEAVE) es el que el original no tolera.

## 3. Lo que se cambió

1. **`UNDECLARED_STORAGE_CAPACITY` desapareció** (`crates/genmsi-core/src/sim.rs`).
   La capacidad sale siempre de la declaración (`storage_capacity`), y un
   `ENTER`/`LEAVE` de un almacén sin declarar **detiene la corrida** con un
   error que nombra el almacén y la solución, como el original:

   ```
   $ genmsi simulate scripts/wine-oracle/benches/Enter.gmdl
   error: …: the storage 'AStorage' is used but no StorageDef declares it
   ```

2. **El compilador lo dice** (`crates/genmsi-core/src/program.rs`): una
   *observación* (no un error: el original abre el modelo) por cada almacén sin
   declarar. Toma la redacción del formato (`genmsi_format`) y le agrega lo que
   significa para el motor: "The storage has no capacity: the run stops when a
   transaction reaches it.". La observación llega al `lint` de la CLI, al
   informe, al MCP y al panel de la app:

   ```
   $ genmsi lint scripts/wine-oracle/benches/Enter.gmdl
   status: diagnostics
   diagnostics:
     - the model uses the Storage 'AStorage' in the Enter block 2, and no StorageDef block
       declares 'AStorage' (the model declares no Storage at all). Declare 'AStorage' with a
       StorageDef block, or use one of the declared names. The storage has no capacity: the run
       stops when a transaction reaches it.
   ```

3. **La app no repite el aviso**: la capa de formato (`genmsi_format`) ya decía
   lo mismo desde el aviso del editor; el panel de la app muestra la
   observación del motor y descarta el aviso del formato que ya está
   dentro (`crates/genmsi-app/engine/src/session.rs`).
4. **Consultas**: un almacén inexistente se lee como vacío y como "no lleno"
   (`GATE`, `SELECT`), y no se crea la entidad (el original tampoco la crea).
5. **`RETRY`**: los rechazos del `ENTER` se cuentan en el almacén, no en el
   bloque, como en el reporte original.
6. **Unidades**: documentadas en el código (`Tamaño` = capacidad en unidades,
   `Cantidad` = unidades por transacción) y fijadas con tests.

### Lo que cambió en los reportes (comparación con el código anterior)

Corridos los 25 modelos de `fixtures/models` con el binario anterior y con el
nuevo (misma semilla), la **única** diferencia es la columna `RETRY` de las
filas `ENTER` (y nada más: ni las filas de `STORAGE`, ni las colas, ni los
tiempos). El reporte original también lleva `RETRY` 0 en las filas `ENTER`
(`fixtures/outputs/TP3__aeropuerto.report.txt`), así que el cambio acerca el
reporte al original.

## 4. Tests

`crates/genmsi-core/tests/storages.rs` (nuevo, 7 tests):

- `a_storage_of_one_unit_holds_one_transaction_at_a_time`: `Tamaño` 1 y
  `Cantidad` 1 → `max_use == 1` (nunca dos dentro) y rechazos contados.
- `the_units_of_a_transaction_are_the_cantidad_operand`: `Tamaño` 2/`Cantidad` 1
  → `max_use == 2`; `Tamaño` 1/`Cantidad` 2 → nadie entra (`max_use == 0`).
- `a_storage_the_model_does_not_declare_is_reported_and_stops_the_run`: el
  modelo compila con la observación y la corrida se detiene.
- `an_undeclared_storage_found_while_the_model_runs_stops_the_run`: un nombre
  indirecto (`*Param`) que resuelve a un almacén sin declarar.
- `the_clock_spends_one_unit_of_start_per_frequency`: la corrida termina a las
  `Frecuencia` unidades de tiempo y el informe lo dice en `END_TIME`.
- `a_model_that_uses_an_undeclared_storage_says_so`, `a_declared_storage_compiles`.

## 5. Lo que quedó fuera (y por qué)

- **La cola de la `StorageUse` no cuenta unidades.** El original compila
  `StorageUse` con `Cantidad` a `QUEUE`/`DEPART` también (`QUEUE qAlmacen,2`
  con `Cantidad` 2) y su cola cuenta unidades (`MAX` 240 con 120 transacciones
  de 2 unidades). Nuestro motor cuenta transacciones en las colas y usa 1
  unidad en la cola de la `StorageUse`. Arreglarlo toca la entidad `Queue`,
  `QueueSpec` y cada clase que usa una cola: es otra tarea, no parte de
  "el almacén se comporta como en el original".
- **La fila `ENTER` de la tabla de bloques cuenta los intentos.** Nuestro
  `ENTER` marca `ENTRY COUNT` 270 y `CURRENT` 184 en TP3, mientras el original
  marca 86 y 0 (sólo las entradas logradas; los que esperan quedan en la cola,
  y la cola sí coincide: `QENTREGAEQUIPAJE` 270/184 en los dos). Es la tabla de
  bloques, no la del almacén: no se tocó.
- **Un `StorageDef` sin `Tamaño`** vale 0 unidades (el original diría "El
  parámetro B no debe ser nulo"). Hoy el modelo compila y nada entra nunca.
- **El `Parcial 8-11-21`** declara `acreditacon` y usa `acreditacion`: el
  original lo rechaza con "Error en linea 9 -> Bloque inválido", pero esa línea
  es la del `QTable` "tiempo demora" (el nombre con espacio), no la del
  `ENTER`; el modelo igual no compila en nuestro motor por el `QTable`.

## 6. Cómo reproducir las corridas del original

```sh
export WINEPREFIX=$PWD/scripts/wine-oracle/wineprefix DISPLAY=:99
bash scripts/wine-oracle/run_oracle.sh scripts/wine-oracle/benches/Enter.gmdl /tmp/oracle-out
cat /tmp/oracle-out/Enter.report.txt
```

Los tres modelos de la tabla de 2.2 se generaron con el mismo esqueleto del
banco `StorageUse.gmdl`, cambiando `Tamaño` y `Cantidad`.
