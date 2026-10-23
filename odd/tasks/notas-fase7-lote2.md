# Notas: el lote 2 de la fase 7, clase por clase

Inventario de las 20 clases que sostienen el flujo de una simulación —el
generador, el reloj, los bloques que toman y sueltan recursos, los que
miden el tiempo, los que desvían y los que comparan—: en qué se parece y en
qué no nuestro motor al original. Como el lote 1, esto no arregla nada: es
el mapa de lo que hay que arreglar, con su archivo y su prueba, y nada más.

Lo que el lote 1 dejó medido y lo que este lote repite está en
`odd/tasks/notas-fase7-lote1.md`; el método es el de
`odd/tasks/genmsi-fase7-revision-clase-por-clase.md`.

## Specs

- **S1** — "EL LOTE (por este orden, que es el que sostiene el flujo):
  Generate, Clock, Terminate, Advance, Depart, Enter, Leave, Queue, Seize,
  Release, Preempt, Return, Priority, Transfer, Test, Multifurcacion,
  MultifurcacionP, Loop, Gather, Buffer." Son las 20 clases que quedan del
  catálogo del original después del lote 1 (que midió 26) y antes del lote 3
  (que medirá las 20 de los datos: `Note`, `Variable`, `VariableA`,
  `VariableN`, `Assign`, `Alter`, `Logic`, `InitialLS`, `InitiaSV`,
  `SaveValue`, `Scan`, `QTable`, `Tabulate`, `Function`, `FunctionExpon`,
  `FunctionDist`, `MFunction`, `MFunctionDist`, `ExamineNumber` y
  `RemoveNumber`).
- **S2** — "Anota por clase: los puertos, las propiedades y sus defaults, la
  semántica (qué hace y con qué regla) y las cifras. Y el veredicto:
  verificada / con diferencias / pendiente."
- **S3** — "odd/tasks/notas-fase7-lote2.md: una ficha por clase (los cuatro
  puntos de arriba, el veredicto y la diferencia concreta), una tabla resumen
  al principio, y una lista final de los cambios pendientes con su archivo y
  su prueba."
- **S4** — "Si el cambio va en el core, NO lo hagas: déjalo escrito con el
  archivo, la función y la prueba que lo fijaría." Lo único que se hace aquí
  es el mapa y las notas.
- **S5** — "ESTA SESIÓN ES DE INVENTARIO: NO EDITES NADA DE crates/ NI DE
  docs/." El core lo tiene otra sesión.
- **S6** — "El banco tiene que declararle al original todo lo que la clase
  necesita: las entidades que usa (un almacen sin declarar hace que el
  original se niegue a correr) y las salidas de fallo que el macro exige."
  Los cuatro bancos que el original se negaba a correr se arreglan en
  `scripts/wine-oracle/make-benches.py`, con sus pruebas.
- **S7** — "Cuando una regla no quede fijada por los conteos, dilo. Los
  conteos no distinguen la semántica exacta... Si una sonda es necesaria para
  distinguir dos lecturas, hazla y dilo."
- **S8** — "Y una regla del repositorio, para lo que commits: NINGÚN commit
  lleva pie de atribución (nada de "Co-Authored-By" ni similar)."

## Resumen

Las 20 clases del lote, en el orden del encargo. `GPSS` es la comparación de
`scripts/wine-oracle/compare-gpss.sh` (verbo, operandos y nombre de cada
sentencia) y `Cifras` es la de las métricas del reporte del oráculo (20% en
conteos, 35% en tiempos).

| # | Clase | Verbo | GPSS | Cifras | Veredicto | La diferencia, en una línea |
|---|---|---|---|---|---|---|
| 1 | Generate | `GENERATE` | coincide | 1/1 | verificada | sólo de forma: la columna `LOC` de los bloques sin nombre |
| 2 | Clock | `GENERATE`+`TERMINATE` | coincide | 0/0 (sin métrica) | verificada | ninguna |
| 3 | Terminate | `TERMINATE` | coincide | 1/1 | verificada | ninguna |
| 4 | Advance | `ADVANCE` | coincide | 2/2 | verificada | ninguna (el retardo lo mide una sonda) |
| 5 | Depart | `DEPART` | coincide | 6/6 | verificada | ninguna |
| 6 | Enter | `ENTER` | coincide | 4/4 | con diferencias | la fila del reporte pierde las unidades (`ENTER AStorage` contra `ENTER AStorage,1`) |
| 7 | Leave | `LEAVE` | coincide | 5/5 | con diferencias | lo mismo que el `Enter` |
| 8 | Queue | `QUEUE` | coincide | 5/5 | verificada | ninguna: el contenido, el máximo y los promedios de la cola coinciden |
| 9 | Seize | `SEIZE` | coincide | 5/5 | verificada | ninguna |
| 10 | Release | `RELEASE` | coincide | 5/5 | verificada | ninguna |
| 11 | Preempt | `PREEMPT` | coincide | 5/5 | con diferencias | la fila del reporte pierde los operandos (`PREEMPT Recurso` contra `PREEMPT Recurso,PR,,Remanente`) |
| 12 | Return | `RELEASE` | coincide | 5/5 | verificada | ninguna: la clase compila un `RELEASE`, no un `RETURN` |
| 13 | Priority | `PRIORITY` | coincide | 2/2 | verificada | ninguna |
| 14 | Transfer | `TRANSFER` | coincide | 2/2 | con diferencias | la fila del reporte pierde la probabilidad (`TRANSFER ,,A_Terminate` contra `TRANSFER 1000,,A_Terminate`) |
| 15 | Test | `TEST` | coincide | 2/2 | con diferencias | el listado no escribe el destino alternativo del `TEST` (lo dice la sonda) |
| 16 | Multifurcacion | `MULTITRANSFER` | coincide | 6/6 | con diferencias | la fila del reporte escribe los pesos donde el original escribe la función |
| 17 | MultifurcacionP | `MULTITRANSFER` | coincide | 5/5 | con diferencias | la fila del reporte escribe el nombre de la clase; y nuestro catálogo lista cinco propiedades `Prob` que la clase del original no tiene |
| 18 | Loop | `LOOP` | coincide | 2/2 | con diferencias | la fila del reporte pierde la coma del segundo destino (`LOOP Param` contra `LOOP Param,`) |
| 19 | Gather | `GATHER` | coincide | 2/2 | verificada | ninguna |
| 20 | Buffer | `BUFFER` | coincide | 2/2 | verificada | ninguna |

**Del total:** 20 clases, **20 de 20** con el GPSS igual al del original;
**20 de 20** con todas las cifras que el oráculo puede comparar dentro de las
tolerancias (y diez de ellas con menos de tres métricas, que el oráculo no
puede comparar: sus bloques van sin nombre); **12 verificadas**, **8 con
diferencias** y **ninguna pendiente** (las 20 clases dejan reporte del
original). De las ocho con diferencias, **una es del texto del programa** (el
destino alternativo del `TEST`, que el listado no escribe) y las otras siete
son de la forma de una fila del reporte o del catálogo.

**Los cuatro bancos que el original se negaba a correr.** El lote 1 dejó tres
bancos que preguntaban de menos; este lote encontró cuatro más, y el motivo
es siempre el mismo: la clase usa una entidad o una salida que el modelo
tenía que declarar:

| Banco | Lo que el original decía | Lo que le faltaba |
|---|---|---|
| `Depart` | `ERROR!!Contenido de la cola QRECURSO negativo.` | un `Queue` delante: la cola tiene que tener una transacción |
| `Enter` | `ERROR!!Referencia a una entidad storage inexistente. (Storage: -1).` | el `StorageDef` del almacén que entra |
| `Leave` | `ERROR!!Storage: ASTORAGE. Se liberaron más unidades de storage que la capacidad máxima.` | un `Enter` delante (las unidades tienen que estar ahí) y el `StorageDef` |
| `MultifurcacionP` | (nada: el original abre el modelo, dice "uno o mas de los modulos utilizados dentro de este fueron adaptados a las definiciones de clases existentes en este sistema" y el arnés no puede leer la respuesta) | cinco propiedades `Prob` que la clase del original no tiene |

Los cuatro están arreglados en `scripts/wine-oracle/make-benches.py`, con sus
pruebas (`scripts/wine-oracle/test_make_benches.py`), y los cuatro ya tienen
respuesta del original.

## Sondas

Cuatro cosas no se pueden ver con los bancos del lote, así que se midieron
con cuatro sondas contra el original (el mismo arnés que `run-benches.sh`:
`bash scripts/wine-oracle/run_oracle.sh <modelo> <destino>`). Los modelos
están en `scripts/wine-oracle/probes/` y su explicación en
`scripts/wine-oracle/probes/SondasLote2.md`.

### Sonda 1 — el `MultifurcacionP` reparte por el valor del parámetro

El banco del `MultifurcacionP` no dice nada del reparto: sin parámetro, las
transacciones van todas a la primera rama en los dos motores. La sonda le
pone un parámetro antes (`Assign` con `Param = 3`):

```
Generate ──► Assign (Param = 3) ──► MultifurcacionP ──┬─ Salida1     ─► A_Terminate
                                                     ├─ SalidaProb2 ─► A_Terminate1
                                                     ├─ SalidaProb3 ─► A_Terminate2
                                                     ├─ SalidaProb4 ─► A_Terminate3
                                                     └─ SalidaProb5 ─► A_Terminate4
```

| | rama 1 | rama 2 | rama 3 | rama 4 | rama 5 |
|---|---|---|---|---|---|
| original (56 transacciones) | 0 | 0 | **56** | 0 | 0 |
| nuestro (60 transacciones) | 0 | 0 | **60** | 0 | 0 |

La regla queda fijada: **la rama es el valor del parámetro**, y las ramas se
numeran desde el uno (el `Salida1` es el valor 1, el `SalidaProb2` el 2, …),
que es lo que dicen los rótulos de las salidas de la clase
(`'[Parametro]=1'`, `'[Parametro]=2'`, …). Los dos motores la cumplen igual.

### Sonda 2 — el `TEST` con una comparación que falla

El banco del `Test` compara `1` con `1`, así que la comparación siempre se
cumple y la salida de fallo nunca se usa. La sonda la invierte (`1 = 2`) y
conecta las dos salidas:

```
Generate ──► Test (E 1,2) ──┬─ Salida1 ("Verdadero") ─► A_Terminate
                            └─ Salida2 ("Falso")     ─► A_Terminate1
```

| | `A_Terminate` | `A_Terminate1` |
|---|---|---|
| original (64 transacciones) | 0 | **64** |
| nuestro (60 transacciones) | 0 | **60** |

La regla queda fijada: **la transacción que no cumple la comparación va por la
salida de fallo** ("Falso"), y el listado del original escribe esa salida
como tercer operando (`TEST E 1,2,A_Terminate1`). El nuestro la manda
igual, pero **no la escribe en el listado** (y sí en el reporte): esa es la
diferencia del `Test`.

### Sonda 3 — el contenido de una cola sí se acumula

El banco del `Queue` encola y no saca nunca a nadie, así que el contenido de
la cola crece hasta el final de la corrida. La sonda pone un `Advance` de
1000 entre el `Queue` y el `Depart`, para que las transacciones se queden
paradas fuera de la cola antes de salir de ella:

```
Generate ──► Queue ──► Advance (1000) ──► Depart ──► Terminate
```

| | `MAX` | `CONT.` | `ENTRIES` | `AVE.CONT.` | `AVE.TIME` |
|---|---|---|---|---|---|
| original | 20 | 15 | 60 | 14,507 | 870,400 |
| nuestro | 18 | 16 | 60 | 14,589 | 875,369 |

Los dos coinciden: **el contenido de la cola se acumula** mientras la
transacción sigue viva, y las cinco cifras caen dentro de las tolerancias
(35% en tiempos, 20% en conteos). El `Queue` del lote queda verificado.

### Sonda 4 — cuánto retiene un `ADVANCE`

El reporte no trae ninguna cifra del tiempo que una transacción pasa en un
`ADVANCE`, así que el banco del `Advance` (media 10, desvío 5) sólo mide el
tránsito. La sonda mide el retardo por el otro lado: un `ADVANCE` de 1000 sin
desvío, con el reloj de 3600, y se cuentan las transacciones que llegan al
`TERMINATE` antes de que la corrida termine:

| | entran en el `ADVANCE` | quedan dentro al final | llegan al `TERMINATE` |
|---|---|---|---|
| original | 57 | 16 | 41 |
| nuestro | 60 | 16 | 44 |

Los dos retienen lo mismo: **el `ADVANCE` de 1000 deja 16 transacciones
dentro al llegar la hora 3600** en los dos motores, y los que llegan al
terminador son los que entraron antes de la hora 2600. La diferencia entre
41 y 44 es el ruido del generador (60 contra 58 transacciones nacidas), no
la regla del retardo.

## Fichas

Una por clase, en el orden del lote. Cada ficha dice lo que el original dice
de la clase (su `ClassDescrip`, su macrocode y, cuando aporta, el manual del
simulador `VISUALSIS.HLP`), los puertos, las propiedades con sus defaults,
la semántica, el banco, el GPSS, las cifras y el veredicto.

### Generate

**El original.** No describe la clase (`ClassDescrip` vacío en
`Genmsi_20230711.backup/gClasses.xml`), pero su macrocode lo dice todo:

```
GENERATE [Media:'Frecuencia media de arribos'='60'],
         [Modificador:'Desvio o modificador'='30'],
         [OffSet:'Tiempo de espera'],
         [Cantidad:'Cantidad de Transacciones a generar'],
         [Prioridad:'Prioridad de la transaccion generada'] OUT Salida1
```

y el manual (`VISUALSIS.HLP`): "Un bloque GENERATE crea transacciones para
entradas futuras dentro de la simulación", con el operando A como "tiempo de
intergeneración menor" (aquí la media, 60, con su desvío, 30).

- **Puertos.** Ninguna entrada y una salida: `Salida1`. La clase es la única
  del catálogo que produce transactions y no recibe ninguna.
- **Propiedades y defaults.** `Media` (`TSNA`, 60), `Modificador` (`TSNA`,
  30), `OffSet` (`TSNA`, vacío), `Cantidad` (`TSNA`, vacío) y `Prioridad`
  (`TSNA`, vacío). Nuestro catálogo trae las mismas
  (`crates/genmsi-format/src/catalogue.rs`, `("Generate", "Media", "60")` y
  `("Generate", "Modificador", "30")`).
- **Semántica.** La clase compila una sentencia: `GENERATE 60,30`, la
  transacción entrando por la salida. Nuestro motor la compila en
  `crates/genmsi-core/src/program.rs`, fn `Compiler::expand`, brazo
  `"Generate"` (línea 1062), y la ejecuta en `crates/genmsi-core/src/sim.rs`,
  fn `execute` (brazo `BlockKind::Generate`): sortea el intervalo (media más o
  menos desvío), crea la transacción y la manda por `Salida1`. El `Clock` del
  banco es el que cierra la corrida (su propio `GENERATE 3600`).
- **El banco.** Un `Generate` (Media 60, Modificador 30) → `Terminate`, más
  un `Clock` de 3600 que compila `GENERATE 3600` + `TERMINATE 1`.
- **GPSS.** Coincide con el del original (sentencia por sentencia):
  `GENERATE 60,30`, `TERMINATE`, `GENERATE 3600`, `TERMINATE 1`. El
  comparador del oráculo lo confirma: "el GPSS coincide: 2 bloques y 0
  conexiones".
- **Cifras.** La tabla del oráculo (`compare-benches`) da 1 de 1: el bloque
  `A_Terminate` con 57 entradas en el original y 60 en el nuestro (5%, dentro
  del 20%). El `GENERATE` del reloj cuenta 2 entradas en los dos.
- **Veredicto.** verificada
- **Diferencia concreta.** Ninguna de esta clase. Las diferencias de forma
  que valen para todos los bancos del lote están en la sección "Las
  diferencias de forma, comunes a todos los bancos".

### Clock

**El original.** "Descuenta 1 unidad del START de la simulación cada un
cierto tiempo indicado, simulando un reloj. Con este módulo se pueden hacer
simulaciones por un cierto tiempo. Si se utiliza este módulo no debe
utilizarse otro módulo que descuente unidades de la simulación."
(`ClassDescrip` de la clase en `gClasses.xml`). Su macrocode es
`MAKE [Frecuencia:'frecuencia, en unidad de tiempo, con la que se descuenta 1
unidad del START'='3600'] END 1`: no es una sentencia de GPSS sino un bloque
que se compila en dos.

- **Puertos.** Ninguno: la sección `graph` de la clase no declara partes
  `Input` ni `Output`.
- **Propiedades y defaults.** `Frecuencia` (`TSNA`, 3600). Catálogo:
  `("Clock", "Frecuencia", "3600")`.
- **Semántica.** La clase se compila en un `GENERATE 3600` y un `TERMINATE 1`
  (el `END 1` del macrocode es el `TERMINATE` con una unidad), de modo que la
  corrida dura 3600 unidades de tiempo. Nuestro motor lo compila en
  `crates/genmsi-core/src/program.rs`, fn `Compiler::expand`, brazo `"Clock"`
  (línea 1036): crea los dos bloques (`Generate` y `Terminate`, con
  `termination_tick: true`) y los encadena.
- **El banco.** Sólo un `Clock` de 3600: no hay `Generate` ni `Terminate` de
  la clase, así que el banco mide el reloj y nada más.
- **GPSS.** Coincide: `GENERATE 3600` y `TERMINATE 1`.
- **Cifras.** El oráculo no puede comparar este banco: sus dos bloques van
  sin nombre y el oráculo sólo empareja las filas por su etiqueta (0 de 0
  métricas, "no-metrics"). Los dos reportes coinciden en lo que sí se ve: los
  dos cuentan 2 entradas en el `GENERATE 3600` y 1 en el `TERMINATE 1`.
- **Veredicto.** verificada
- **Diferencia concreta.** Ninguna de esta clase.

### Terminate

**El original.** No la describe; su macrocode es
`IN Entrada1:'Ingreso'='A_moduleName' TERMINATE [Unidades:'Cantidad de
unidades a decrementar del START, por cada transacción que llega al módulo']`.
Es la clase que termina las transacciones y descuenta el `START`.

- **Puertos.** Una entrada (`Entrada1`, rótulo "Ingreso") y ninguna salida.
- **Propiedades y defaults.** `Unidades` (`TSNA`, sin default: el modelo la
  deja vacía). Catálogo: sin default para `Terminate`.
- **Semántica.** La sentencia `TERMINATE` (sin operando cuando `Unidades`
  está vacía) destruye la transacción y descuenta del `START` una unidad por
  cada transacción que llega. Nuestro motor la compila en
  `crates/genmsi-core/src/program.rs`, fn `Compiler::expand`, brazo
  `"Terminate"` (línea 1076) y la ejecuta en `crates/genmsi-core/src/sim.rs`,
  fn `terminate` (línea 1675), que quita la transacción y descuenta el
  `START`.
- **El banco.** `Generate` (60,30) → `Terminate`, más el `Clock` de 3600.
- **GPSS.** Coincide: `GENERATE 60,30`, `A_Terminate TERMINATE`,
  `GENERATE 3600`, `TERMINATE 1`.
- **Cifras.** 1 de 1: el bloque `A_Terminate` con 59 entradas en el original
  y 60 en el nuestro (2%).
- **Veredicto.** verificada
- **Diferencia concreta.** Ninguna de esta clase.

### Advance

**El original.** No la describe; su macrocode es
`IN Entrada1:'Ingresa'='A_moduleName' ADV [Media:'Tiempo Medio de espera'='10'],
[Modificador:'Desvio o Modificador'='5'] OUT Salida1`. El manual: "Un bloque
ADVANCE retarda el progreso de una transacción por una cantidad especificada
de tiempo de simulación"; con los dos operandos, "un número aleatorio entre
A-B y A + B, inclusivamente, es usado como el incremento de tiempo".

- **Puertos.** Una entrada (`Entrada1`, rótulo "Ingresa") y una salida
  (`Salida1`).
- **Propiedades y defaults.** `Media` (`TSNA`, 10) y `Modificador` (`TSNA`,
  5). Catálogo: `("Advance", "Media", "10")` y `("Advance", "Modificador",
  "5")`.
- **Semántica.** La sentencia `ADVANCE 10,5`: la transacción espera ese
  tiempo (media más o menos desvío) y sigue por `Salida1`. Nuestro motor la
  compila en `crates/genmsi-core/src/program.rs`, fn `Compiler::expand`,
  brazo `"Advance"` (línea 1087), y la ejecuta en `crates/genmsi-core/src/
  sim.rs` (brazo `BlockKind::Advance`), que sortea el incremento y programa
  la transacción para más tarde.
- **El banco.** `Generate` (60,30) → `Advance` (10,5) → `Terminate`, más el
  `Clock` de 3600.
- **GPSS.** Coincide: `GENERATE 60,30`, `A_Advance ADVANCE 10,5`,
  `A_Terminate TERMINATE`, `GENERATE 3600`, `TERMINATE 1`.
- **Cifras.** 2 de 2 (el `A_Advance` con 56 entradas contra 60, y el
  `A_Terminate` con 56 contra 60; 7% y 7%, dentro del 20%). El reporte no
  trae ninguna cifra del retardo, así que el banco sólo mide el tránsito: el
  retardo lo mide la sonda 4, con un `ADVANCE` de 1000, donde los dos
  motores dejan 16 transacciones dentro al llegar la hora 3600.
- **Veredicto.** verificada
- **Diferencia concreta.** Ninguna de esta clase.

### Depart

**El original.** No la describe; su macrocode es
`IN Entrada1:'Ingreso'='A_ModuleName' DEPART [Queue:'Queue de la cual se
egresa'='Qrecurso'],[Cantidad:'Cantidad de egresos'=''] OUT Salida1`. El
manual: "Un bloque DEPART registra estadísticas que indican una reducción en
el contenido de una entidad queue"; si la cola no existe, "la intención de
decrementar el contenido de la cola causará un error de stop".

- **Puertos.** Una entrada (`Entrada1`, rótulo "Ingreso") y una salida
  (`Salida1`).
- **Propiedades y defaults.** `Queue` (`EntityUse`, subType `Queue`, default
  `Qrecurso`) y `Cantidad` (`TSNA`, default vacío). Catálogo:
  `("Depart", "Queue", "Qrecurso")`.
- **Semántica.** La sentencia `DEPART Qrecurso` saca una unidad (o las que
  diga `Cantidad`) de la cola y actualiza sus estadísticas. Nuestro motor la
  compila en `crates/genmsi-core/src/program.rs`, fn `Compiler::expand`,
  brazo `"Depart"` (línea 1167) y la ejecuta en `crates/genmsi-core/src/
  sim.rs` (brazo `BlockKind::Depart`, línea 779): `Queue::dequeue` baja el
  contenido y devuelve el tiempo de espera.
- **El banco.** `Generate` (60,30) → `Queue` → `Depart` → `Terminate`, más el
  `Clock` de 3600. **El `Queue` es obligatorio**: con el banco viejo (sin la
  cola delante) el original no corría y decía `ERROR!!Contenido de la cola
  QRECURSO negativo.` El banco nuevo le da al `Depart` la cola que él nombra
  (`Qrecurso`, no el `QRecurso` del default de la clase `Queue`).
- **GPSS.** Coincide: `GENERATE 60,30`, `A_Queue QUEUE Qrecurso`,
  `A_Depart DEPART Qrecurso`, `A_Terminate TERMINATE`, `GENERATE 3600`,
  `TERMINATE 1`.
- **Cifras.** 6 de 6, todas dentro de las tolerancias: la cola `QRECURSO` con
  `MAX 1`, `CONT. 0`, `ENTRIES 61` contra 60, `AVE.CONT. 0,000` contra 0,
  `AVE.TIME 0,000` contra 0; los bloques `A_Queue` (61 contra 60),
  `A_Depart` (61 contra 60) y `A_Terminate` (61 contra 60).
- **Veredicto.** verificada
- **Diferencia concreta.** Ninguna de esta clase. Lo que había que arreglar
  estaba en el banco, y está en "Los cuatro bancos que el original se negaba
  a correr".

### Enter

**El original.** No la describe; su macrocode es
`IN Entrada1:'Ingreso'='A_ModuleName' ENTER [Storage:'Storage al cual ingresa
la transacción'='AStorage'],[Cantidad:'Cantidad de ingresos'='1'] OUT
Salida1`. El manual: un `ENTER` toma unidades de un almacén; si el almacén
está lleno, la transacción espera en la cadena de demora.

- **Puertos.** Una entrada (`Entrada1`, rótulo "Ingreso") y una salida
  (`Salida1`).
- **Propiedades y defaults.** `Storage` (`EntityUse`, subType `Storage`,
  default `AStorage`) y `Cantidad` (`TSNA`, default 1). Catálogo:
  `("Enter", "Storage", "AStorage")` y `("Enter", "Cantidad", "1")`.
- **Semántica.** La sentencia `ENTER AStorage,1`: la transacción toma una
  unidad del almacén y sigue; si no hay unidades, queda esperando. Nuestro
  motor la compila en `crates/genmsi-core/src/program.rs`, fn
  `Compiler::expand`, brazo `"Enter"` (línea 1467) y la ejecuta en
  `crates/genmsi-core/src/sim.rs` (brazo `BlockKind::Enter`, línea 888): si
  el almacén no tiene sitio, la transacción espera y el almacén suma un
  reintento.
- **El banco.** `StorageDef` (el almacén `AStorage`, tamaño 10) →
  `Generate` (60,30) → `Enter` → `Terminate`, más el `Clock` de 3600. **El
  `StorageDef` es obligatorio**: sin él el original no corría y decía
  `ERROR!!Referencia a una entidad storage inexistente. (Storage: -1).`
- **GPSS.** Coincide, con una diferencia que el comparador del oráculo
  declara equivalente por su propia regla ("las unidades de un ENTER o un
  LEAVE son las mismas las escriba el original o no"): el original escribe
  `ENTER AStorage,1` y el nuestro `ENTER AStorage`.
- **Cifras.** 4 de 4, dentro de las tolerancias: el bloque `A_Enter` con 10
  entradas en los dos; el almacén `ASTORAGE` con `CAP. 10`, `REMAIN. 0`,
  `MIN. 0`, `MAX. 10`, `ENTRIES 10` en los dos, `AVL. 1`, `AVE.C. 9,027`
  contra 9,003 (0,3%), `UTIL. 0,903` contra 0,900, `RETRY 48` contra 49; el
  `GENERATE` con 59 entradas y 49 pendientes en el original y 50 en el
  nuestro (las transacciones que esperan sitio en el almacén).
- **Veredicto.** con diferencias
- **Diferencia concreta.** La fila del bloque en nuestro reporte escribe
  `ENTER AStorage` donde el original escribe `ENTER AStorage,1`: las
  unidades no salen. Lo arreglaría `crates/genmsi-core/src/report.rs`, fn
  `operand_parts` (rama `BlockKind::Enter`/`BlockKind::Leave`, línea 468),
  que hoy las escribe sólo cuando no son una (`fn is_one_unit`, línea 375):
  para las clases `Enter` y `Leave` el operando es el `Cantidad` de la
  clase, y su default es 1, que el original escribe siempre. Lo mismo en el
  listado: `crates/genmsi-core/src/gpss.rs`, fn `block_statements` (ramas
  `BlockKind::Enter` y `BlockKind::Leave`, líneas 638 y 646) omiten el
  `,1`. Lo fijaría un test en `crates/genmsi-core/tests/report_gpss.rs` que
  compruebe que la fila del banco `Enter` sale `ENTER AStorage,1` y que el
  listado sale `ENTER AStorage,1`.

### Leave

**El original.** No la describe; su macrocode es
`IN Entrada1:'Entrada1'='A_ModuleName' LEAVE [Storage:'Storage del cual
egresa la transacción'='AStorage'],[Cantidad:'Cantidad de egresos'='1'] OUT
Salida1`. El manual: "Un bloque LEAVE hace a las unidades de storage
disponibles para otras transacciones"; si se liberan más unidades de las
definidas, "ocurre un error de stop".

- **Puertos.** Una entrada (`Entrada1`, rótulo "Entrada1") y una salida
  (`Salida1`).
- **Propiedades y defaults.** `Storage` (`EntityUse`, subType `Storage`,
  default `AStorage`) y `Cantidad` (`TSNA`, default 1). Catálogo:
  `("Leave", "Storage", "AStorage")` y `("Leave", "Cantidad", "1")`.
- **Semántica.** La sentencia `LEAVE AStorage,1`: la transacción devuelve
  una unidad del almacén y sigue; si el almacén no tiene esa unidad, el
  original se para. Nuestro motor la compila en
  `crates/genmsi-core/src/program.rs`, fn `Compiler::expand`, brazo
  `"Leave"` (línea 1478) y la ejecuta en `crates/genmsi-core/src/sim.rs`
  (brazo `BlockKind::Leave`, línea 947), que sube la disponibilidad y
  despierta a quien esperaba.
- **El banco.** `StorageDef` (el almacén `AStorage`, tamaño 10) →
  `Generate` (60,30) → `Enter` → `Leave` → `Terminate`, más el `Clock` de
  3600. **El `Enter` delante es obligatorio**: sin él el original se paraba
  con `ERROR!!Storage: ASTORAGE. Se liberaron más unidades de storage que la
  capacidad máxima.` (nadie había entrado en el almacén, así que no había
  unidades que liberar).
- **GPSS.** Coincide (con la misma regla de equivalencia del `ENTER`):
  el original escribe `LEAVE AStorage,1` y el nuestro `LEAVE AStorage`.
- **Cifras.** 5 de 5, dentro de las tolerancias: los bloques `A_Enter` (61
  contra 59), `A_Leave` (61 contra 59) y `A_Terminate` (61 contra 59); el
  almacén `ASTORAGE` con `CAP. 10`, `REMAIN. 10`, `MIN. 0`, `MAX. 1`,
  `ENTRIES 61` contra 59 en los dos.
- **Veredicto.** con diferencias
- **Diferencia concreta.** La misma que el `Enter`: la fila del reporte
  escribe `LEAVE AStorage` donde el original escribe `LEAVE AStorage,1`.
  El arreglo es el mismo (`crates/genmsi-core/src/report.rs`, fn
  `operand_parts`, rama `BlockKind::Enter`/`BlockKind::Leave`, línea 468, y
  `crates/genmsi-core/src/gpss.rs`, fn `block_statements`, líneas 638 y 646),
  y la prueba la fijaría un test en `crates/genmsi-core/tests/report_gpss.rs`
  con el banco `Leave`.

### Queue

**El original.** No la describe; su macrocode es
`IN Entrada1:'Ingreso'='A_moduleName' ARRIVE [Queue:'Queue en la que ingresa
la transacción'='QRecurso'], [Cantidad:'Cantidad de ingresos en la cola'='']
OUT Salida1`. El manual: "Un bloque QUEUE actualiza las estadísticas de la
entidad queue para reflejar un incremento en el contenido"; la entidad cola
"es creada si es necesario".

- **Puertos.** Una entrada (`Entrada1`, rótulo "Ingreso") y una salida
  (`Salida1`).
- **Propiedades y defaults.** `Queue` (`EntityUse`, subType `Queue`, default
  `QRecurso`) y `Cantidad` (`TSNA`, default vacío). Catálogo:
  `("Queue", "Queue", "QRecurso")`.
- **Semántica.** La sentencia `QUEUE QRecurso`: la transacción entra en la
  cola y el contenido de la cola sube (en 1, o en lo que diga `Cantidad`).
  Nuestro motor la compila en `crates/genmsi-core/src/program.rs`, fn
  `Compiler::expand`, brazo `"Queue"` (línea 1156) y la ejecuta en
  `crates/genmsi-core/src/sim.rs` (brazo `BlockKind::Queue`, línea 769):
  `Queue::enqueue` sube el contenido, guarda el máximo y el momento de la
  entrada.
- **El banco.** `Generate` (60,30) → `Queue` (QRecurso) → `Terminate`, más el
  `Clock` de 3600. La cola no la saca nadie: el contenido crece hasta el
  final de la corrida, que es lo que hace el banco interesante.
- **GPSS.** Coincide: `GENERATE 60,30`, `A_Queue QUEUE QRecurso`,
  `A_Terminate TERMINATE`, `GENERATE 3600`, `TERMINATE 1`.
- **Cifras.** 5 de 5, dentro de las tolerancias: los bloques `A_Queue` (61
  contra 60) y `A_Terminate` (61 contra 60); y la cola `QRECURSO` con
  `MAX 61` contra 60, `CONT. 61` contra 60, `ENTRIES 61` contra 60,
  `ENTRIES(0) 0` en los dos, `AVE.CONT. 30,049` contra 30,659 (2%) y
  `AVE.TIME 1773,393` contra 1839,544 (3,7%).
- **Veredicto.** verificada
- **Diferencia concreta.** Ninguna de esta clase. La sonda 3 lo confirma en
  el caso en que la transacción sale de la cola más tarde (la `Queue` con un
  `Advance` de 1000 delante del `Depart`): `MAX` 20 contra 18, `CONT.` 15
  contra 16, `AVE.CONT.` 14,507 contra 14,589, `AVE.TIME` 870,400 contra
  875,369.

### Seize

**El original.** No la describe; su macrocode es
`IN Entrada1:'Ingreso'='A_moduleName' SEIZE [Facility:'Facility a tomar'=
'Recurso'] OUT Salida1`. El manual: "Un bloque SEIZE habilita a una
transacción a adquirir la posesión de una facility"; si la facility está
apropiada, "la transacción descansa, última sin su prioridad, en la Cadena de
Demora de la facility y no entra al bloque SEIZE".

- **Puertos.** Una entrada (`Entrada1`, rótulo "Ingreso") y una salida
  (`Salida1`).
- **Propiedades y defaults.** `Facility` (`EntityUse`, subType `Facility`,
  default `Recurso`). Catálogo: `("Seize", "Facility", "Recurso")`.
- **Semántica.** La sentencia `SEIZE Recurso`: la transacción toma el
  recurso si está libre; si no, espera en su cola de demora, ordenada por
  prioridad. Nuestro motor la compila en `crates/genmsi-core/src/program.rs`,
  fn `Compiler::expand`, brazo `"Seize"` (línea 1251) y la ejecuta en
  `crates/genmsi-core/src/sim.rs` (brazo `BlockKind::Seize`, línea 789): si
  el recurso está ocupado, la transacción espera y el recurso suma un
  reintento.
- **El banco.** `Generate` (60,30) → `Seize` (Recurso) → `Release` →
  `Terminate`, más el `Clock` de 3600. El `Release` detrás es obligatorio: el
  original no deja terminar una transacción que posea un recurso.
- **GPSS.** Coincide: `GENERATE 60,30`, `A_Seize SEIZE Recurso`,
  `A_Release RELEASE Recurso`, `A_Terminate TERMINATE`, `GENERATE 3600`,
  `TERMINATE 1`.
- **Cifras.** 5 de 5: los bloques `A_Seize` (59 contra 60), `A_Release` (59
  contra 60) y `A_Terminate` (59 contra 60); el recurso `RECURSO` con
  `ENTRIES 59` contra 60, `UTIL. 0,000` en los dos, `AVE_TIME 0,000` en los
  dos, `AVAILABLE 1`, `OWNER 0`, `PEND 0`, `INTER 0`, `RETRY 0`, `DELAY 0`.
- **Veredicto.** verificada
- **Diferencia concreta.** Ninguna de esta clase. Lo que este banco no
  mide es la espera por prioridad (el recurso está libre casi siempre: la
  `UTIL.` es 0 y la cola de demora nunca tiene dos transacciones).

### Release

**El original.** No la describe; su macrocode es
`IN Entrada1:'Ingreso'='A_moduleName' RELEASE [Facility:'Facility a liberar'=
'Recurso'] OUT Salida1`. El manual: "Un bloque RELEASE libera la propiedad de
una facility"; la transacción de entrada "es removida de la propiedad o de la
contención de la facility", y "si la propiedad de la facility está por ser
decidida, el próximo propietario es tomado de la Cadena de Pendientes, la
Cadena de Interrupción, y finalmente la Cadena de Demora".

- **Puertos.** Una entrada (`Entrada1`, rótulo "Ingreso") y una salida
  (`Salida1`).
- **Propiedades y defaults.** `Facility` (`EntityUse`, subType `Facility`,
  default `Recurso`). Catálogo: `("Release", "Facility", "Recurso")`.
- **Semántica.** La sentencia `RELEASE Recurso`: la transacción suelta el
  recurso y sigue; si el recurso está ocupado por alguien, se lo queda el
  siguiente de la lista de espera. Nuestro motor la compila en
  `crates/genmsi-core/src/program.rs`, fn `Compiler::expand`, brazo
  `"Release"` (línea 1259) y la ejecuta en `crates/genmsi-core/src/sim.rs`
  (brazo `BlockKind::Release`): suelta el recurso y despierta al primero de
  la lista de demora.
- **El banco.** `Generate` (60,30) → `Seize` (Recurso) → `Release` →
  `Terminate`, más el `Clock` de 3600. El `Seize` delante es obligatorio: el
  original no suelta un recurso que nadie posee.
- **GPSS.** Coincide: `GENERATE 60,30`, `A_Seize SEIZE Recurso`,
  `A_Release RELEASE Recurso`, `A_Terminate TERMINATE`, `GENERATE 3600`,
  `TERMINATE 1`.
- **Cifras.** 5 de 5: los bloques `A_Seize` (59 contra 60), `A_Release` (59
  contra 60) y `A_Terminate` (59 contra 60); el recurso `RECURSO` con
  `ENTRIES 59` contra 60 y el resto de las columnas en cero en los dos.
- **Veredicto.** verificada
- **Diferencia concreta.** Ninguna de esta clase. El banco mide el camino
  sin espera (el recurso está libre el 100% del tiempo): la regla de "el
  siguiente de la lista de demora recibe el recurso" no la mide.

### Preempt

**El original.** No la describe; su macrocode es
`DECLARE Salida1,Output ... IN Entrada1:'Ingreso'='A_moduleName' PREEMPT
[Facility:'Facility a interrumpir por la transacción activa.'='Recurso'],
[ModoInterrupcion:'Modo de interrupción o por prioridad'='Prioridad'],
SalidaAlt:'Propietaria',
[Remanente:'Parámetro de la transacción propietaria donde se guarda el
remanete'='Remanente'],
[ModoArrebato:'Modo de arrebato temporal o permanete.'='NoRemover'] OUT
Salida1`. El manual: "Un bloque PREEMPT destituye a una transacción de la
propiedad de una facility"; en modo prioridad, "únicamente la transacción con
mayor prioridad puede destituir a la transacción que posee la facility".

- **Puertos.** Una entrada (`Entrada1`, rótulo "Ingreso") y dos salidas:
  `Salida1` (la que sigue la transacción que preempta) y `SalidaAlt` (rótulo
  "Propietaria", por donde sale la transacción a la que se le quitó el
  recurso). En el banco `SalidaAlt` queda **sin conectar**.
- **Propiedades y defaults.** `Facility` (`EntityUse`, `Recurso`),
  `ModoInterrupcion` (`Listed`, subType `PriorityMode`, `Prioridad`),
  `Remanente` (`EntityUse`, subType `Parameter`, `Remanente`) y
  `ModoArrebato` (`Listed`, subType `RemoveMode`, `NoRemover`). Catálogo:
  líneas 231-234 de `crates/genmsi-format/src/catalogue.rs`, las mismas.
- **Semántica.** La sentencia `PREEMPT Recurso,PR,,Remanente`: la
  transacción toma el recurso; si alguien lo tenía, lo interrumpe (en modo
  `PR` sólo si su prioridad es mayor), le guarda el tiempo que le quedaba en
  el parámetro `Remanente` y lo manda por `SalidaAlt`. Nuestro motor la
  compila en `crates/genmsi-core/src/program.rs`, fn `Compiler::expand`,
  brazo `"Preempt"` (línea 1269) y la ejecuta en `crates/genmsi-core/src/
  sim.rs` (brazo `BlockKind::Preempt`, línea 826).
- **El banco.** `Generate` (60,30) → `Preempt` (Recurso, `Prioridad`,
  `Remanente`, `NoRemover`) → `Return` → `Terminate`, más el `Clock` de 3600.
  El `Return` detrás es obligatorio (nadie puede terminar una transacción que
  posea un recurso) y la `SalidaAlt` sin conectar es lo que hace que el
  operando del destino de interrupción salga vacío.
- **GPSS.** Coincide: `GENERATE 60,30`, `A_Preempt PREEMPT
  Recurso,PR,,Remanente`, `A_Return RELEASE Recurso`, `A_Terminate
  TERMINATE`, `GENERATE 3600`, `TERMINATE 1`.
- **Cifras.** 5 de 5: los bloques `A_Preempt` (64 contra 60), `A_Return` (64
  contra 60) y `A_Terminate` (64 contra 60); el recurso `RECURSO` con
  `ENTRIES 64` contra 60, `UTIL. 0,000`, `AVAILABLE 1`, `OWNER 0`, `PEND 0`,
  `INTER 0`, `RETRY 0`, `DELAY 0`. El banco **no** mide la interrupción: el
  recurso está libre casi siempre (utilización 0) y todas las transacciones
  nacen con la misma prioridad, así que el `PR` nunca destituye a nadie.
- **Veredicto.** con diferencias
- **Diferencia concreta.** La fila del bloque en nuestro reporte escribe
  `PREEMPT Recurso` donde el original escribe `PREEMPT Recurso,PR,,Remanente`
  (con el modo, el destino de interrupción y el parámetro del remanente).
  Es el mismo defecto que el `FacilityInt` del lote 1 (pendiente 5 de esa
  lista): lo arreglaría `crates/genmsi-core/src/report.rs`, fn
  `operand_parts` (rama `BlockKind::Preempt`, línea 467), que devuelve sólo
  el recurso y tendría que escribir los cuatro operandos como lo hace
  `crates/genmsi-core/src/gpss.rs` (rama `BlockKind::Preempt`, línea 617).
  Lo fijaría un test en `crates/genmsi-core/tests/report_gpss.rs` que
  compruebe que la fila del banco `Preempt` sale
  `PREEMPT Recurso,PR,,Remanente`.

### Return

**El original.** No la describe; su macrocode es
`IN Entrada1:'Ingreso'='A_moduleName' RELEASE [Facility:'Facility a liberar'=
'Recurso'] OUT Salida1`: **la clase `Return` compila un `RELEASE`**, no un
`RETURN` de GPSS. (El `RETURN` de GPSS es otra cosa: lo usa el macro de la
clase `FacilityInt`.) El manual describe el bloque `RETURN` ("libera la
propiedad de una facility") con la misma acción.

- **Puertos.** Una entrada (`Entrada1`, rótulo "Ingreso") y una salida
  (`Salida1`).
- **Propiedades y defaults.** `Facility` (`EntityUse`, `Recurso`). Catálogo:
  `("Return", "Facility", "Recurso")`.
- **Semántica.** La sentencia `RELEASE Recurso`, como la clase `Release`.
  Nuestro motor lo compila en `crates/genmsi-core/src/program.rs`, fn
  `Compiler::expand`, brazo `"Return"` (línea 1293), que arma el mismo
  `BlockKind::Release` que la clase `Release` (y con la clase del bloque
  `"Release"`, que es lo que hace que el reporte lo muestre como
  `RELEASE`).
- **El banco.** `Generate` (60,30) → `Seize` (Recurso) → `Return` →
  `Terminate`, más el `Clock` de 3600.
- **GPSS.** Coincide: `GENERATE 60,30`, `A_Seize SEIZE Recurso`,
  `A_Return RELEASE Recurso`, `A_Terminate TERMINATE`, `GENERATE 3600`,
  `TERMINATE 1`.
- **Cifras.** 5 de 5: los bloques `A_Seize` (55 contra 60), `A_Return` (55
  contra 60) y `A_Terminate` (55 contra 60); el recurso `RECURSO` con
  `ENTRIES 55` contra 60.
- **Veredicto.** verificada
- **Diferencia concreta.** Ninguna de esta clase.

### Priority

**El original.** No la describe; su macrocode es
`IN Entrada1:'Ingreso'='A_moduleName' PRIORITY [Prioridad:'Nº prioridad de
la transacción. Cuanto mas grande sea el nº, mayor es la prioridad.'='0'] OUT
Salida1`. El manual: "Un bloque PRIORITY setea la prioridad de la
transacción activa"; "la transacción es planificada hacia el NSB y colocada en
la CEC de acuerdo a su nueva prioridad".

- **Puertos.** Una entrada (`Entrada1`, rótulo "Ingreso") y una salida
  (`Salida1`).
- **Propiedades y defaults.** `Prioridad` (`TSNA`, 0). Catálogo:
  `("Priority", "Prioridad", "0")`.
- **Semántica.** La sentencia `PRIORITY 0`: la transacción toma esa
  prioridad y sigue. Nuestro motor la compila en
  `crates/genmsi-core/src/program.rs`, fn `Compiler::expand`, y la ejecuta
  en `crates/genmsi-core/src/sim.rs` (cambia la prioridad de la transacción
  y la reprograma).
- **El banco.** `Generate` (60,30) → `Priority` (0) → `Terminate`, más el
  `Clock` de 3600.
- **GPSS.** Coincide: `GENERATE 60,30`, `A_Priority PRIORITY 0`,
  `A_Terminate TERMINATE`, `GENERATE 3600`, `TERMINATE 1`.
- **Cifras.** 2 de 2: los bloques `A_Priority` (57 contra 60) y
  `A_Terminate` (57 contra 60).
- **Veredicto.** verificada
- **Diferencia concreta.** Ninguna de esta clase. El banco no mide lo único
  que la clase hace (el orden de la CEC según la prioridad): el reporte no
  trae ninguna cifra del orden de las transacciones, y el `FEC_XACT_NUMBER`
  del original (la tabla de los eventos pendientes, que sí lo diría) es una
  de las secciones que nuestro reporte no imprime.

### Transfer

**El original.** No la describe; su macrocode es
`DECLARE Salida1,Output ... IN Entrada1:'Ingreso'='Entrada1' TRANSFER
[Prob:'Probabilidad de continuar por la Salida1 1'], Salida2:'resto',
Salida1:'[prob]'`. El manual: "Un bloque TRANSFER causa que la transacción
activa salte a una nueva dirección"; en modo fraccional, "la transacción
activa salta a la dirección especificada por el operando C con una
probabilidad dada por el operando A" y "el destino alternativo está
especificado por el operando B, o el NSB si el operando B es omitido".

- **Puertos.** Una entrada (`Entrada1`, rótulo "Ingreso") y dos salidas:
  `Salida1` (rótulo "[prob]") y `Salida2` (rótulo "resto"). El rótulo de la
  entrada es el nombre del módulo sin prefijo (`'Entrada1'='Entrada1'`), así
  que el bloque se llama `Entrada1` en el reporte del original.
- **Propiedades y defaults.** `Prob` (`TSNA`, sin default: el modelo lo
  deja). El banco le pone `100%`, que es el valor que el manual usa como
  ejemplo ("`TRANSFER .75, , NEW_PLACE`").
- **Semántica.** La sentencia `TRANSFER 1000,,A_Terminate`: la probabilidad
  en milésimas, y después los destinos (en el orden en que los escribe el
  original, que es el inverso al de los puertos). Nuestro motor la compila
  en `crates/genmsi-core/src/program.rs`, fn `Compiler::expand`, brazo
  `"Transfer"` (línea 1507), y la ejecuta en `crates/genmsi-core/src/sim.rs`
  (brazo `BlockKind::Transfer`, línea 978): sortea la probabilidad y manda
  la transacción por `Salida1` o por `Salida2`. El reparto (la probabilidad
  va a `Salida1`, el puerto que nombra la pista de la propiedad) ya está
  fijado por la sonda de la fase 6
  (`scripts/wine-oracle/probes/TransferProbOrder.md`).
- **El banco.** `Generate` (60,30) → `Transfer` (Prob 100%) → `Terminate`,
  más el `Clock` de 3600. El `Salida2` ("resto") queda sin conectar, que es
  lo que hace que el segundo operando salga vacío.
- **GPSS.** Coincide: `GENERATE 60,30`, `Entrada1 TRANSFER 1000,,A_Terminate`,
  `A_Terminate TERMINATE`, `GENERATE 3600`, `TERMINATE 1`.
- **Cifras.** 2 de 2: los bloques `Entrada1` (56 contra 60) y `A_Terminate`
  (56 contra 60).
- **Veredicto.** con diferencias
- **Diferencia concreta.** La fila del bloque en nuestro reporte escribe
  `TRANSFER ,,A_Terminate` donde el original escribe `TRANSFER 1000,,A_Terminate`:
  la probabilidad no sale. Lo arreglaría
  `crates/genmsi-core/src/report.rs`, fn `transfer_probability` (línea 358) y
  `fn operand_parts` (rama `BlockKind::Transfer`, línea 487): hoy la
  probabilidad se borra cuando vale 1 (el caso de una `Prob` vacía, que el
  original escribe como `TRANSFER ,A_Enter`), pero cuando el modelo dice
  `100%` el original escribe `1000` (las milésimas del porcentaje). El
  `TransferSpec` tendría que llevar el texto de origen de la probabilidad, o
  el flag de que vino de un porcentaje. Lo fijaría un test en
  `crates/genmsi-core/tests/report_gpss.rs` que compruebe que la fila del
  banco `Transfer` sale `TRANSFER 1000,,A_Terminate`.

### Test

**El original.** No la describe; su macrocode es
`DECLARE Salida1,Output ... IN Entrada1:'Entrada1'='A_ModuleName' TEST
[1_Valor:'Primer Valor'='1'], [2_OP:'Operador relacional'='='],
[3_Valor:'Valor a comparar con el primero'='1'], Salida2:'Falso' OUT
Salida1:'Verdadero'`. El manual: "Un bloque TEST compara valores, normalmente
SNAs, y controla el destino de la transacción activa"; si el operando C (el
destino alternativo) no se usa, "el bloque TEST opera en modo de rechazo" y
la transacción que falla la prueba espera hasta que la prueba se cumpla.

- **Puertos.** Una entrada (`Entrada1`, rótulo "Entrada1") y dos salidas:
  `Salida1` (rótulo "Verdadero") y `Salida2` (rótulo "Falso").
- **Propiedades y defaults.** `1_Valor` (`TSNA`, 1), `2_OP` (`Listed`,
  subType `op_Relac`, `=`) y `3_Valor` (`TSNA`, 1). Catálogo: `("Test",
  "1_Valor", "1")`, `("Test", "2_OP", "%3D")` y `("Test", "3_Valor", "1")`.
  La lista `op_Relac` del original tiene seis operadores (`E:=`, `NE:!=`,
  `G:>`, `L:<`, `GE:>=`, `LE:<=`); nuestro motor los entiende todos en la
  forma en que los guarda el editor (`=`, `!=`, `>`, `<`, `>=`, `<=`, que es
  lo que traen los modelos de referencia del original).
- **Semántica.** La sentencia `TEST E 1,1`: la transacción sigue por
  `Salida1` si la comparación se cumple y por `Salida2` si no. Nuestro motor
  la compila en `crates/genmsi-core/src/program.rs`, fn `Compiler::expand`,
  brazo `"Test"` (línea 1489), y la ejecuta en `crates/genmsi-core/src/sim.rs`
  (brazo `BlockKind::Test`, línea 962): `Salida1` si `spec.op.test(left,
  right)`, `Salida2` si no.
- **El banco.** `Generate` (60,30) → `Test` (1 `=` 1) → `Terminate`, más el
  `Clock` de 3600. El `Salida2` ("Falso") queda sin conectar, y la
  comparación siempre se cumple, así que el banco mide sólo el camino
  verdadero: el camino falso lo mide la sonda 2.
- **GPSS.** Coincide: `GENERATE 60,30`, `A_Test TEST E 1,1`, `A_Terminate
  TERMINATE`, `GENERATE 3600`, `TERMINATE 1`.
- **Cifras.** 2 de 2: los bloques `A_Test` (62 contra 60) y `A_Terminate` (62
  contra 60).
- **Veredicto.** con diferencias
- **Diferencia concreta.** El listado del programa **no escribe el destino
  alternativo** del `TEST`: con la salida de fallo conectada (la sonda 2) el
  original escribe `TEST E 1,2,A_Terminate1` y el nuestro `TEST E 1,2`. Es
  el mismo defecto que el `EXAMINE` (pendiente A de
  `odd/tasks/notas-bancos.md`): lo arreglaría
  `crates/genmsi-core/src/gpss.rs`, fn `block_statements`, rama
  `BlockKind::Test` (línea 654), que escribe sólo `E <valor>,<valor>`, y
  tendría que escribir el nombre del destino de la segunda salida, como
  hacen las ramas `BlockKind::Transfer` (línea 666) y `BlockKind::Examine`
  (línea 532). El reporte ya lo escribe bien
  (`crates/genmsi-core/src/report.rs`, fn `operand_parts`, rama
  `BlockKind::Test`, línea 477). Lo fijaría un test en
  `crates/genmsi-core/tests/report_gpss.rs` que compruebe que el listado de
  la sonda `TestFalso` sale `TEST E 1,2,A_Terminate1`.

### Multifurcacion

**El original.** "Multi Transfer" (el `ClassDescrip` de la clase en
`gClasses.xml`). Su macrocode es
`FUNCTIONDIST MultifurcacionCH,2,{[Prob1];[Prob2];[Prob3];[Prob4];[Prob5]}
IN Entrada:'Multifurcacion'='moduleName' MULTITRANSFER MultifurcacionGO,
FN:MultifurcacionCH,{Salida1:'[Prob1]';SalidaProb2:'[Prob2]';SalidaProb3:'[Prob3]';SalidaProb4:'[Prob4]';SalidaProb5:'[Prob5]'}`:
la clase reparte el tráfico con las probabilidades que guarda y manda cada
rama a su salida.

- **Puertos.** Una entrada (`Entrada`, rótulo "Multifurcacion") y cinco
  salidas: `Salida1`, `SalidaProb2`, `SalidaProb3`, `SalidaProb4` y
  `SalidaProb5`. El rótulo de la entrada es el nombre del módulo sin
  prefijo, así que el bloque se llama `Multifurcacion` en el reporte.
- **Propiedades y defaults.** `MultifurcacionCH` (`EntityDef`, subType
  `Function`), `MultifurcacionGO` (`EntityDef`, subType `Function`) y las
  cinco probabilidades `Prob1`…`Prob5` (`FloatPercent`). Las probabilidades
  no las declara el macrocode: las crea el editor al dibujar el bloque (el
  modelo de referencia del original lo confirma,
  `fixtures/models/Parcial 6-11-23__Veron - Parcial.gmdl`: el módulo
  `Multifurcacion` tiene `Prob1` 25%, `Prob2` 60% y `Prob3` 15%). El banco
  les pone 20% a todas.
- **Semántica.** La clase se compila en dos sentencias de definición y una
  de flujo:
  `MultifurcacionCH FUNCTION RN2,D5 0.2,1/0.4,2/0.6,3/0.8,4/1,5` (la
  distribución de las probabilidades), `MultifurcacionGO FUNCTION
  FN$MultifurcacionCH,D5 1,A_Terminate/2,A_Terminate1/…` (la tabla que
  asocia cada rama con su destino) y `Multifurcacion TRANSFER
  ,FN$MultifurcacionGO` (la sentencia que hace el reparto). Nuestro motor
  compila lo mismo en `crates/genmsi-core/src/program.rs`, fn
  `Compiler::expand`, rama `"Multifurcacion" | "MultifurcacionP"` (línea
  1523), y lo ejecuta en `crates/genmsi-core/src/sim.rs` (brazo
  `BlockKind::MultiTransfer`, línea 999): sortea el peso y elige la rama.
- **El banco.** `Generate` (60,30) → `Multifurcacion` (cinco `Prob` al 20%)
  → cinco `Terminate`, más el `Clock` de 3600. Las cuatro salidas de
  probabilidad están conectadas a su terminador, que el original necesita
  para escribir la sentencia.
- **GPSS.** Coincide, las nueve sentencias y las dos definiciones de
  función: `bash scripts/wine-oracle/compare-gpss.sh` dice "el GPSS coincide:
  9 bloques y 1 conexiones".
- **Cifras.** 6 de 6, dentro de las tolerancias del oráculo (que compara
  contra el intervalo de nuestras réplicas, no contra el punto): el bloque
  `Multifurcacion` con 59 entradas contra 60, y las cinco ramas con 13/7/14/14/11
  en el original y 11/11/13/16/9 en el nuestro. Las dos ejecuciones Reparten
  el mismo tráfico (un quinto para cada rama); la diferencia de la segunda
  rama (7 contra 11) es el ruido de un solo sorteo, no una regla distinta.
- **Veredicto.** con diferencias
- **Diferencia concreta.** La fila del bloque en nuestro reporte escribe
  `TRANSFER 0.2,0.2,0.2,0.2,0.2` donde el original escribe
  `TRANSFER ,FN$MultifurcacionGO`: la fila enseña los pesos del modelo donde
  el original enseña la función de destinos. Lo arreglaría
  `crates/genmsi-core/src/report.rs`, fn `operand_parts` (rama
  `BlockKind::MultiTransfer`, línea 496), que hoy devuelve los pesos (su
  propio comentario lo dice: "we keep the branch weights the model actually
  stores") y tendría que escribir `,FN$<MultifurcacionGO>` como lo hace
  `crates/genmsi-core/src/gpss.rs` (rama `BlockKind::MultiTransfer`, línea
  681). Lo fijaría un test en `crates/genmsi-core/tests/report_gpss.rs` que
  compruebe que la fila del banco `Multifurcacion` sale
  `TRANSFER ,FN$MultifurcacionGO`.

### MultifurcacionP

**El original.** "Multi Transfer según el valor de un parámetro" (el
`ClassDescrip` de la clase en `gClasses.xml`). Su macrocode es
`IN Entrada:'Multifurcacion'='moduleName' MULTITRANSFER MultifurcacionGO,
P:[Parametro],{Salida1:'[Parametro]=1';SalidaProb2:'[Parametro]=2';SalidaProb3:'[Parametro]=3';SalidaProb4:'[Parametro]=4';SalidaProb5:'[Parametro]=5]'}`:
la rama es el valor del parámetro de la transacción.

- **Puertos.** Una entrada (`Entrada`, rótulo "Multifurcacion") y cinco
  salidas: `Salida1`, `SalidaProb2`…`SalidaProb5`.
- **Propiedades y defaults.** `MultifurcacionGO` (`EntityDef`, subType
  `Function`) y `Parametro` (`EntityUse`, subType `Parameter`, default
  `Param`). **La clase no tiene las propiedades `Prob`**: el banco con ellas
  lo abría el original diciendo que lo adaptó a las definiciones de clases
  existentes, y esa ventana le impedía al arnés leer la respuesta (ver "Los
  cuatro bancos que el original se negaba a correr"). Catálogo: las mismas
  dos, más cinco `Prob` que sobran
  (`crates/genmsi-format/src/catalogue.rs`, líneas 223-227).
- **Semántica.** La sentencia `MultifurcacionP TRANSFER ,FN$MultifurcacionGO`
  con la función `MultifurcacionGO FUNCTION P$Param,D5
  1,A_Terminate/2,A_Terminate1/…`: la rama se elige por el valor del
  parámetro. Nuestro motor compila lo mismo en
  `crates/genmsi-core/src/program.rs`, fn `Compiler::expand` (rama
  `"Multifurcacion" | "MultifurcacionP"`, línea 1579) y lo ejecuta en
  `crates/genmsi-core/src/sim.rs` (rama `MultiTransferSelection::Parameter`,
  línea 1101): el número de la rama es el valor del parámetro (el uno es la
  primera rama) y un valor fuera de las ramas toma la primera. La sonda 1 lo
  confirma contra el original.
- **El banco.** `Generate` (60,30) → `MultifurcacionP` (Parametro `Param`) →
  cinco `Terminate`, más el `Clock` de 3600. Sin un `Assign` delante, el
  parámetro no existe y todas las transacciones van a la primera rama (en los
  dos motores).
- **GPSS.** Coincide, las siete sentencias: las dos definiciones de función
  y `MultifurcacionP TRANSFER ,FN$MultifurcacionGO`.
- **Cifras.** 5 de 5: el bloque `MultifurcacionP` con 58 entradas contra 60
  y la primera rama con 58 contra 60; las otras cuatro ramas con cero en los
  dos.
- **Veredicto.** con diferencias
- **Diferencia concreta.** Dos cosas. La primera es la fila del bloque en
  nuestro reporte: escribe `MULTIFURCACIONP` (el nombre de la clase en
  mayúsculas) donde el original escribe `TRANSFER ,FN$MultifurcacionGO`. Lo
  arreglaría `crates/genmsi-core/src/report.rs`, fn `operand_parts` (rama
  `BlockKind::MultiTransfer`, línea 496), igual que para la clase
  `Multifurcacion`: la fila tiene que decir `TRANSFER` y llevar la función.
  La segunda es el catálogo: `crates/genmsi-format/src/catalogue.rs` (líneas
  223-227) lista para la clase `MultifurcacionP` las cinco propiedades
  `Prob1`…`Prob5` con el default `20%25`, y la clase del original **no las
  tiene** (la evidencia: el modelo del banco con esas propiedades lo abre el
  original adaptándolo, y sin ellas lo abre sin quejarse; y el modelo de
  referencia del original, `fixtures/models/Parcial 6-11-23__Veron -
  Parcial.gmdl`, sólo lleva `Prob` en la clase `Multifurcacion`). Lo
  arreglaría el catálogo (quitar esas cinco filas) y el motor, que hoy las
  lee para el reparto por pesos (no las usa en el reparto por parámetro).
  Lo fijaría un test de `crates/genmsi-format` que compruebe que la clase
  `MultifurcacionP` no declara ninguna propiedad `Prob`, y otro de
  `crates/genmsi-core/tests/report_gpss.rs` con la fila del banco
  `MultifurcacionP`.

### Loop

**El original.** No la describe; su macrocode es
`DECLARE Salida1,Output ... IN Entrada1:'Ingreso'='A_ModuleName' LOOP
[Parametro:'Parametro a decrementar con cada iteracion'='Param'], Salida2:'loop'
OUT Salida1`. El manual: "Un bloque LOOP modifica un parámetro y controla el
destino de la transacción activa basado en el resultado"; el parámetro "es
decrementado por 1" y, "si el nuevo valor del parámetro no es cero y el
operando B es especificado, la transacción es planificada hacia la dirección
especificada en el operando B".

- **Puertos.** Una entrada (`Entrada1`, rótulo "Ingreso") y dos salidas:
  `Salida1` (la que sigue cuando la cuenta llega a cero) y `Salida2` (rótulo
  "loop", la de la iteración).
- **Propiedades y defaults.** `Parametro` (`EntityUse`, subType `Parameter`,
  default `Param`). Catálogo: `("Loop", "Parametro", "Param")`.
- **Semántica.** La sentencia `LOOP Param,`: la transacción decrece el
  parámetro y, si no llega a cero, vuelve por `Salida2`; si llega a cero (o
  si el `Salida2` no está conectado), sigue por `Salida1`. Nuestro motor la
  compila en `crates/genmsi-core/src/program.rs`, fn `Compiler::expand` (rama
  `"Loop"`), y la ejecuta en `crates/genmsi-core/src/sim.rs` (brazo
  `BlockKind::Loop`).
- **El banco.** `Generate` (60,30) → `Loop` (Parametro `Param`) →
  `Terminate`, más el `Clock` de 3600. El `Salida2` ("loop") queda sin
  conectar y **nadie pone el parámetro**: es lo que hace que el banco mida
  el caso degenerado.
- **GPSS.** Coincide: `GENERATE 60,30`, `A_Loop LOOP Param,`, `A_Terminate
  TERMINATE`, `GENERATE 3600`, `TERMINATE 1`.
- **Cifras.** 2 de 2: los bloques `A_Loop` (63 contra 60) y `A_Terminate`
  (63 contra 60). Con el parámetro inexistente, el original **no** se para
  (el manual dice que un parámetro que no existe es un error de stop: el
  original no lo trata así) y las 63 transacciones siguen de largo, como las
  60 nuestras.
- **Veredicto.** con diferencias
- **Diferencia concreta.** La fila del bloque en nuestro reporte escribe
  `LOOP Param` donde el original escribe `LOOP Param,`: el segundo operando
  (el destino de la iteración) no sale, y sale vacío porque el `Salida2` no
  está conectado. Lo arreglaría `crates/genmsi-core/src/report.rs`, fn
  `operand_parts` (rama `BlockKind::Loop`), que tiene que escribir el
  destino de la segunda salida aunque esté vacío, como hace con el `TEST`
  (rama `BlockKind::Test`, línea 477). Lo fijaría un test en
  `crates/genmsi-core/tests/report_gpss.rs` que compruebe que la fila del
  banco `Loop` sale `LOOP Param,`. Aparte, y sin que este banco lo mida: la
  regla del `LOOP` con el parámetro existente (decrementar, y volver por
  `Salida2` mientras no llegue a cero) no la mide ningún banco del lote,
  porque el parámetro del banco no existe; la mediría un banco con un
  `Assign` delante que le pusiera un valor, con la `Salida2` conectada.

### Gather

**El original.** No la describe; su macrocode es
`IN Entrada1:'Juntar'='A_moduleName' MEET [Cantidad:'Cantidad de
transacciones a sincronizar por grupo de reunion'='1'] OUT Salida1`. El
manual: "Un bloque GATHER sincroniza transacciones"; "la primera transacción
de un conjunto de reunión es espera cuando esta ingresa al bloque GATHER";
"los bloques GATHER difieren de los bloques ASSEMBLE en que las transacciones
después las primeras no son destruidas".

- **Puertos.** Una entrada (`Entrada1`, rótulo "Juntar") y una salida
  (`Salida1`).
- **Propiedades y defaults.** `Cantidad` (`TSNA`, 1). Catálogo:
  `("Gather", "Cantidad", "1")`.
- **Semántica.** La sentencia `GATHER 1`: las transacciones del mismo grupo
  de reunión se juntan y todas siguen (a diferencia del `ASSEMBLE`, que
  destruye a todas menos la primera). Nuestro motor la compila en
  `crates/genmsi-core/src/program.rs`, fn `Compiler::expand`, rama
  `"Assemble" | "Gather"` (línea 1679), que arma el mismo `BlockKind::
  Assemble` con la marca `gather`, y la ejecuta en `crates/genmsi-core/src/
  sim.rs` (rama `BlockKind::Assemble`, línea 1198).
- **El banco.** `Generate` (60,30) → `Gather` (Cantidad 1) → `Terminate`,
  más el `Clock` de 3600. Con `Cantidad` 1 el banco mide el camino de una
  transacción (no hay grupo que juntar).
- **GPSS.** Coincide: `GENERATE 60,30`, `A_Gather GATHER 1`, `A_Terminate
  TERMINATE`, `GENERATE 3600`, `TERMINATE 1`.
- **Cifras.** 2 de 2: los bloques `A_Gather` (60 contra 60) y `A_Terminate`
  (60 contra 60), y el `CURRENT COUNT` del bloque `A_Gather` es **0** en los
  dos (las 60 transacciones que pasaron por el bloque ya no cuentan como
  presentes).
- **Veredicto.** verificada
- **Diferencia concreta.** Ninguna de esta clase. El `CURRENT` del bloque
  bajaba con `enter_block` en vez de con `leave`, y ya no: lo arregló la
  sesión del núcleo en el commit `47d0b8f` ("the assemble did not take its
  CURRENT down when it completed the set"), que arregla las dos clases
  porque el motor las corre en la misma rama (`BlockKind::Assemble`, con la
  marca `gather`).

### Buffer

**El original.** No la describe; su macrocode es
`IN Entrada1:'Ingreso'='A_moduleName' BUFFER OUT Salida1`. El manual: "Un
bloque BUFFER ubica a la transacción activa en la CEC detrás de sus pares de
igual prioridad"; "una transacción nunca es rechazada a entrar en un bloque
BUFFER".

- **Puertos.** Una entrada (`Entrada1`, rótulo "Ingreso") y una salida
  (`Salida1`).
- **Propiedades y defaults.** Ninguna: la clase no declara propiedades.
  Catálogo: sin propiedades para `Buffer`.
- **Semántica.** La sentencia `BUFFER`: la transacción vuelve al final de la
  cola de eventos, detrás de sus pares de igual prioridad, y sigue por
  `Salida1`. Nuestro motor la compila en `crates/genmsi-core/src/program.rs`,
  fn `Compiler::expand` (rama `"Buffer"`), y la ejecuta en
  `crates/genmsi-core/src/sim.rs` (reprograma la transacción al final).
- **El banco.** `Generate` (60,30) → `Buffer` → `Terminate`, más el `Clock`
  de 3600.
- **GPSS.** Coincide: `GENERATE 60,30`, `A_Buffer BUFFER`, `A_Terminate
  TERMINATE`, `GENERATE 3600`, `TERMINATE 1`.
- **Cifras.** 2 de 2: los bloques `A_Buffer` (57 contra 60) y `A_Terminate`
  (57 contra 60).
- **Veredicto.** verificada
- **Diferencia concreta.** Ninguna de esta clase. Lo que la clase hace (el
  orden dentro de la cola de eventos) no lo dice ningún número del reporte:
  el `FEC_XACT_NUMBER` del original (la tabla de los eventos pendientes) es
  una de las secciones que nuestro reporte no imprime.

## Las diferencias de forma, comunes a todos los bancos

No son de ninguna clase en concreto: son del generador del reporte, y las
mismo en los veinte bancos de este lote (y en los del lote 1).

| Qué | El original | El nuestro | Dónde |
|---|---|---|---|
| La columna `LOC` de los bloques sin nombre | el número de línea del programa (`1`, `3`, `4`…) | un contador de bloques sin nombre (`2`, `3`…) | `crates/genmsi-core/src/report.rs`, fn `render` (el contador `numeric_label`) |
| Los decimales enteros | `0,000`, `100,000` | `0`, `100` | `crates/genmsi-core/src/report.rs`, fn `number` (línea 23) |
| La línea de la semilla | no la tiene | `SEMILLA    7` | `crates/genmsi-core/src/report.rs`, fn `render` |
| La tabla de los eventos pendientes | `FEC_XACT_NUMBER PRI BDT CURRENT NEXT PARAMETER VALUE` | no la tiene | `crates/genmsi-core/src/report.rs`, fn `render` (línea 35) |
| El ancho del encabezado | `START_TIME    END_TIME     BLOCKS    FACILITIES    STORAGES` | `START_TIME    END_TIME     BLOCKS    FACILITIES    STORAGES` con otras columnas | `crates/genmsi-core/src/report.rs`, fn `render` |

## Cambios pendientes

Cada fila dice **qué** hay que arreglar, **dónde** (archivo y función) y **qué
prueba** lo fijaría. Nada de esto se hizo en esta sesión: el core lo tiene
otra sesión.

### En el core (`crates/genmsi-core`)

| # | Qué hay que arreglar | Dónde | La prueba que lo fijaría |
|---|---|---|---|
| 1 | **La fila del `ENTER` y del `LEAVE` pierde las unidades.** El original escribe `ENTER AStorage,1` (el `Cantidad` de la clase, cuyo default es 1, siempre escrito); el nuestro escribe `ENTER AStorage`. | `crates/genmsi-core/src/report.rs` · `operand_parts` (ramas `BlockKind::Enter`/`BlockKind::Leave`, línea 468) y `fn is_one_unit` (línea 375); `crates/genmsi-core/src/gpss.rs` · `block_statements` (ramas `BlockKind::Enter` y `BlockKind::Leave`, líneas 638 y 646) | Un test en `crates/genmsi-core/tests/report_gpss.rs` que compruebe que la fila del banco `Enter` sale `ENTER AStorage,1` y la del `Leave` `LEAVE AStorage,1`, y que el listado sale con el `,1` |
| 2 | **La fila del `PREEMPT` pierde los operandos:** escribimos `PREEMPT Recurso` donde el original escribe `PREEMPT Recurso,PR,,Remanente`. (Ya estaba anotado como pendiente 5 del lote 1, con la clase `FacilityInt`.) | `crates/genmsi-core/src/report.rs` · `operand_parts` (rama `BlockKind::Preempt`, línea 467) | Un test en `crates/genmsi-core/tests/report_gpss.rs` que compruebe que la fila del banco `Preempt` sale `PREEMPT Recurso,PR,,Remanente` |
| 3 | **La fila del `TRANSFER` pierde la probabilidad:** escribimos `TRANSFER ,,A_Terminate` donde el original escribe `TRANSFER 1000,,A_Terminate`. La regla actual borra la probabilidad cuando vale 1, que es el caso de una `Prob` vacía, pero no el de una `Prob` de `100%` (que el original escribe en milésimas). | `crates/genmsi-core/src/report.rs` · `fn transfer_probability` (línea 358) y `operand_parts` (rama `BlockKind::Transfer`, línea 487); el `TransferSpec` (`crates/genmsi-core/src/program.rs`, fn `Compiler::expand`, rama `"Transfer"`, línea 1507) tendría que llevar el origen de la probabilidad | Un test en `crates/genmsi-core/tests/report_gpss.rs` que compruebe que la fila del banco `Transfer` sale `TRANSFER 1000,,A_Terminate` |
| 4 | **La fila del reparto (la `Multifurcacion` y la `MultifurcacionP`) no dice lo que dice la del original:** escribimos los pesos (`TRANSFER 0.2,0.2,0.2,0.2,0.2`) o el nombre de la clase (`MULTIFURCACIONP`) donde el original escribe `TRANSFER ,FN$MultifurcacionGO`. | `crates/genmsi-core/src/report.rs` · `operand_parts` (rama `BlockKind::MultiTransfer`, línea 496) | Un test en `crates/genmsi-core/tests/report_gpss.rs` que compruebe las dos filas: `TRANSFER ,FN$MultifurcacionGO` |
| 5 | **El listado del `TEST` no escribe el destino alternativo:** con la salida de fallo conectada, el original escribe `TEST E 1,2,A_Terminate1` y el nuestro `TEST E 1,2`. (Es el mismo asunto que el `EXAMINE`, pendiente A de `odd/tasks/notas-bancos.md`.) | `crates/genmsi-core/src/gpss.rs` · `block_statements` (rama `BlockKind::Test`, línea 654) | Un test en `crates/genmsi-core/tests/report_gpss.rs` con el modelo de la sonda `scripts/wine-oracle/probes/TestFalso.gmdl` |
| 6 | **La fila del `LOOP` pierde el segundo operando:** escribimos `LOOP Param` donde el original escribe `LOOP Param,` (el destino de la iteración, vacío cuando el `Salida2` no está conectado). | `crates/genmsi-core/src/report.rs` · `operand_parts` (rama `BlockKind::Loop`) | Un test en `crates/genmsi-core/tests/report_gpss.rs` que compruebe que la fila del banco `Loop` sale `LOOP Param,` |
### En el catálogo (`crates/genmsi-format`)

| # | Qué hay que arreglar | Dónde | La prueba que lo fijaría |
|---|---|---|---|
| 7 | **La clase `MultifurcacionP` no tiene las propiedades `Prob` en el original**, y nuestro catálogo le pone cinco (`Prob1`…`Prob5`, default `20%25`). La evidencia: el modelo del banco con esas propiedades lo abre el original adaptándolo ("uno o mas de los modulos utilizados dentro de este fueron adaptados a las definiciones de clases existentes en este sistema") y sin ellas lo abre sin quejarse; y el modelo de referencia del original sólo lleva `Prob` en la clase `Multifurcacion`. | `crates/genmsi-format/src/catalogue.rs` (líneas 223-227) | Un test de `crates/genmsi-format` que compruebe que la clase `MultifurcacionP` no declara ninguna propiedad `Prob` |

### En los bancos (`scripts/wine-oracle/make-benches.py`)

Lo que este lote cambió, con sus pruebas:

| # | Qué hay que arreglar | Dónde | La prueba que lo fijaría |
|---|---|---|---|
| 8 | **El banco del `Depart` no llenaba la cola:** el original se paraba con `ERROR!!Contenido de la cola QRECURSO negativo.` El banco lleva ahora un `Queue` delante, con el nombre de la cola que el `Depart` nombra (`Qrecurso`, no el `QRecurso` del default de la clase `Queue`). | `scripts/wine-oracle/make-benches.py` · `COMPANIONS_BEFORE` y `fn copy_with_values` | `scripts/wine-oracle/test_make_benches.py`, clase `Companions` |
| 9 | **El banco del `Enter` no declaraba el almacén:** el original se paraba con `ERROR!!Referencia a una entidad storage inexistente.` El banco lleva ahora el `StorageDef` del almacén que la clase usa. | `scripts/wine-oracle/make-benches.py` · `DECLARATIONS` | `scripts/wine-oracle/test_make_benches.py`, clase `Companions` |
| 10 | **El banco del `Leave` ni declaraba el almacén ni lo llenaba:** el original se paraba con `ERROR!!Storage: ASTORAGE. Se liberaron más unidades de storage que la capacidad máxima.` El banco lleva ahora el `StorageDef` y un `Enter` delante. | `scripts/wine-oracle/make-benches.py` · `DECLARATIONS` y `COMPANIONS_BEFORE` | `scripts/wine-oracle/test_make_benches.py`, clase `Companions` |
| 11 | **El banco del `MultifurcacionP` llevaba cinco propiedades `Prob` que la clase del original no tiene:** el original abría el modelo adaptándolo, y la ventana que lo dice le impedía al arnés leer la respuesta. El banco ya no las lleva. | `scripts/wine-oracle/make-benches.py` · `EXTRA_PROPS` | `scripts/wine-oracle/test_make_benches.py`, clase `TheRealCatalogue` |

### Lo que ya está arreglado y no hay que rehacer

| # | Qué | Dónde |
|---|---|---|
| 12 | **La primera transacción de un generador**: ya no difiere (el `GENERATE` del reloj cuenta 2 entradas en los dos). | `odd/tasks/notas-primera-llegada.md` y el commit `47d0b8f` ("la última transacción de la corrida") |
| 13 | **El reparto de un `TRANSFER` fraccional**: la probabilidad va a `Salida1` (el puerto que nombra la pista de la propiedad), aunque el texto la escriba al revés. | `scripts/wine-oracle/probes/TransferProbOrder.md` (fase 6) |

## Lo que queda del catálogo

Las 66 clases del original, después de este lote:

| Lote | Clases | Verificadas | Con diferencias | Pendientes |
|---|---|---|---|---|
| Lote 1 | 26 | 15 | 11 | 0 (las dos que quedaban pendientes —`StorageUse` y `Examine`— las resolvió el lote de bancos: la primera pasó a verificada y la segunda a con diferencias) |
| Lote 2 (este) | 20 | 12 | 8 | 0 |
| **Total medido** | **46** | **27** | **19** | **0** |

**Quedan 20 clases por revisar**, que son las del lote 3 (los datos y las
funciones): `Note`, `Variable`, `VariableA`, `VariableN`, `Assign`, `Alter`,
`Logic`, `InitialLS`, `InitiaSV`, `SaveValue`, `Scan`, `QTable`, `Tabulate`,
`Function`, `FunctionExpon`, `FunctionDist`, `MFunction`, `MFunctionDist`,
`ExamineNumber` y `RemoveNumber`.

De las 46 clases medidas hasta ahora, **19** tienen una diferencia escrita
con su archivo, su función y su prueba: las once del lote 1 (`FacilityInt`,
`GateL`, `GateM`, `SelectS`, `SelectL`, `SelectSNA`, `Link`, `RemoveMe`,
`Counter`, `Examine` y `Table`) y las ocho de este lote (`Enter`, `Leave`,
`Preempt`, `Transfer`, `Test`, `Multifurcacion`, `MultifurcacionP` y
`Loop`).

Las cifras del lote 1 se han vuelto a medir con el motor que dejó la sesión
del núcleo en el commit `47d0b8f` ("the six behaviour differences of the
first lot of phase seven"): de las 26 clases de aquel lote, 25 coinciden
ahora en todas las cifras que el oráculo compara (la que no es la `GateL`, la
fila de la puerta que no abre) y las 26 escriben el mismo GPSS salvo el
`Examine`, cuyo listado no escribe el destino alternativo. Los ocho cambios
que aquel commit arregló (la puerta sobre un almacén, el indicador del
`LINK`, el `CURRENT` del `ASSEMBLE` —y del `GATHER`—, el `MATCH` en dos
sentencias, las filas del `SEIZE` y de la `FACILITY`, y la última transacción
de la corrida) eran diferencias de comportamiento: por eso las fichas de este
lote no las cuentan.

## Tasks

- **T1** — Elegir el lote y volcar lo que el original dice de cada clase (S1, S2) — hecho: las 20 clases del lote y su `ClassDescrip`, su macrocode y sus puertos.
- **T2** — Correr el original sobre los 20 bancos (S7) — hecho: en `/tmp/lote2-oracle`, con un banco por clase.
- **T3** — Arreglar los cuatro bancos que el original se negaba a correr (S6) — hecho: `Depart`, `Enter`, `Leave` y `MultifurcacionP`, con sus pruebas en `scripts/wine-oracle/test_make_benches.py`.
- **T4** — Comparar el GPSS sentencia por sentencia (S2) — hecho: 20 de 20 coinciden.
- **T5** — Comparar las cifras con las tolerancias del oráculo (S2) — hecho: 20 de 20 coinciden en las métricas que el oráculo puede comparar.
- **T6** — Las cuatro sondas (S7) — hecho: `MultifurcacionPParam`, `TestFalso`, `QueueContent` y `AdvanceDelay`, con su nota en `scripts/wine-oracle/probes/SondasLote2.md`.
- **T7** — Las 20 fichas y la lista de pendientes (S3, S4) — hecho: las tres secciones de este documento.
- **T8** — El recuento de lo que queda (S3) — hecho: la sección "Lo que queda del catálogo".
- **T9** — Volver a medir con el motor que dejó la sesión del núcleo (S2) — hecho: el commit `47d0b8f` llegó a mitad de la medición; con él, el `CURRENT` del `GATHER` ya baja y la primera transacción del generador ya no difiere. Las cifras y los veredictos de este documento son los del motor de ese commit.

## Log

- **L1** — "Trabajo en /home/lucas/Documentos/Workspace/Genmsi.

   [el bloque común de contexto, igual que en las sesiones anteriores]

   LEE PRIMERO:
   - odd/tasks/genmsi-fase7-revision-clase-por-clase.md (la fase y su método)
   - odd/tasks/notas-fase7-lote1.md (el primer lote: el modelo a seguir, la
     tabla resumen, las 26 fichas y la lista de cambios pendientes)

   QUÉ HAY QUE HACER (el mismo metodo, clase por clase):
   - Toma el siguiente lote de clases del catálogo del original que todavía no
     estén en las notas del lote 1, yRepite el metodo entero por cada una:
     1. Un banco minimo por clase, que ejercite ESA clase y nada mas.
     2. Correr el original bajo Wine (bash scripts/wine-oracle/run_oracle.sh
        <modelo> <directorio>) y quedarse con su codigo GPSS y su reporte.
     3. Comparar nuestro GPSS con el del original, sentencia por sentencia
        (scripts/wine-oracle/compare-gpss.sh).
     4. Comparar las cifras del reporte con las tolerancias del oraculo (20% en
        conteos, 35% en tiempos): scripts/wine-oracle/compare-benches.sh y
        cargo run --release -p genmsi-core --example compare-benches -- <bancos> <salida> --runs 3.
     5. Veredicto por clase: verificada, con diferencias o pendiente, y por que.
   - El banco tiene que declararle al original todo lo que la clase necesita: las
     entidades que usa (un almacen sin declarar hace que el original se niegue a
     correr) y las salidas de fallo que el macro exige (el EXAMINE, por ejemplo,
     necesita su destino alternativo: sin el, el original rechaza el modelo con
     "Faltan parametros"). El lote 1 dejo tres bancos que preguntaban de menos;
     no repitas eso.
   - Cuando una regla no quede fijada por los conteos, dilo. Los conteos no
     distinguen la semantica exacta: por ejemplo, la primera transaccion de un
     generador nace en la media o en el offset C, y ningun conteo separa ambas
     lecturas.
   - Si una sonda es necesaria para distinguir dos lecturas, hazla y dilo: el
     lote 1 uso dos sondas para fijar la condicion de la puerta del almacen.

   DONE CUANDO:
   - odd/tasks/notas-fase7-lote2.md (nuevo) con la tabla resumen del lote, una
     ficha por clase (lo que se midio, lo que coincidio, lo que no, y la evidencia:
     el reporte del original) y la lista final de pendientes, con archivo,
     funcion y la prueba que lo fijaria, como en el lote 1.
   - Al final, un recuento: cuantas clases del original quedan por revisar, con el
     estado de cada una (verificada, con diferencias o pendiente).
   - No hay commit de codigo: solo las notas y, si hace falta, el generador de
     bancos (scripts/wine-oracle/make-benches.py) con sus pruebas, que es un
     archivo aparte y no pisa el trabajo de la otra sesion.

   Y una regla del repositorio, para lo que commits: NINGUN commit lleva pie de
   atribucion (nada de "Co-Authored-By" ni similar). El proyecto es del lector y
   los commits no llevan la firma de ninguna herramienta ni de ningun modelo. El
   mensaje del commit explica que cambio y por que, y nada mas."

- **L2** — El lote son las 20 clases del flujo: el generador, el reloj, el
  terminador, el retardo, la cola y el almacén, el recurso, el desvío y la
  comparación. Quedan 20 clases para el lote 3 (los datos y las funciones).
- **L3** — La descripción de la clase está en el **atributo** `ClassDescrip`
  del `<moduleclass>`, no en un elemento: cuatro de las veinte clases la
  tienen ("Multi Transfer", "Multi Transfer según el valor de un parámetro" y
  las dos del `Clock`), y las otras dieciséis la tienen vacía.
- **L4** — Los bancos que el original se negaba a correr: cuatro, y por cuatro
  motivos distintos. El `Depart` (la cola vacía), el `Enter` (el almacén sin
  declarar), el `Leave` (las unidades que nadie puso) y el `MultifurcacionP`
  (las cinco propiedades `Prob` que la clase del original no tiene: el
  original abría el modelo adaptándolo, y la ventana del aviso le impedía al
  arnés leer la respuesta).
- **L5** — La regla del compañero: el bloque que acompaña a la clase trabaja
  sobre la misma entidad que la clase (la cola del `Depart` es la que el
  `Depart` nombra), y el generador lo hace con `fn copy_with_values`, que
  copia los puertos también: sin los puertos, el banco tenía un enlace que
  terminaba en un puerto que no existía.
- **L6** — Cuatro sondas: el reparto por parámetro de la `MultifurcacionP`, el
  camino falso del `TEST`, el contenido de una cola con la transacción
  detenida después de encolarse, y el retardo de un `ADVANCE` largo. Las dos
  primeras fijan reglas que el banco no podía ver; las dos últimas confirman
  reglas que el banco ya apuntaba.
- **L7** — El `RETRY` del almacén (48 contra 49) y el `CURRENT` del `GENERATE`
  (49 contra 50) son el ruido de dos corridas distintas: el original sortea
  sus transacciones en cada corrida y la nuestra también.
- **L8** — El trabajo va en dos commits: el generador de bancos con sus
  pruebas y las sondas (lo que se puede tocar sin pisar a la otra sesión), y
  estas notas. El corte de ~400 líneas no aplica al segundo: partir las notas
  dejaría el entregable a medias, que es lo que el encargo no quiere.
- **L9** — A mitad de la medición llegó a `main` el commit `47d0b8f` de la
  sesión del núcleo ("the six behaviour differences of the first lot of phase
  seven"), que arregla ocho diferencias de las que el lote 1 había medido. Dos
  de ellas tocan a este lote: el `CURRENT` del `ASSEMBLE` —que es el mismo
  bloque que el `GATHER`, porque el motor los corre en la misma rama— y la
  última transacción de la corrida. Las cifras y los veredictos de este
  documento se volvieron a medir con ese motor: el `GATHER` pasa a verificada
  y el `Clock` deja de tener la diferencia de la primera transacción. Las
  26 clases del lote 1 también se volvieron a medir, y su recuento actual
  está en la sección "Lo que queda del catálogo".
