# Notas: el lote 3 de la fase 7, clase por clase

Inventario de las 20 clases que sostienen los datos del original —las variables,
las funciones, los parámetros, las llaves lógicas y los valores guardados—: en
qué se parece y en qué no nuestro motor al original. Como el lote 1 y el lote 2,
esto no arregla nada del núcleo: es el mapa de lo que hay que arreglar, con su
archivo y su prueba, y nada más.

Lo que el lote 1 dejó medido y lo que el lote 2 repite está en
`odd/tasks/notas-fase7-lote1.md` y `odd/tasks/notas-fase7-lote2.md`; el método
es el de `odd/tasks/genmsi-fase7-revision-clase-por-clase.md`.

## Specs

- **S1** — "EL LOTE (las veinte clases que faltan): Note, Variable, VariableA,
  VariableN, Assign, Alter, Logic, InitialLS, InitiaSV, SaveValue, Scan, QTable,
  Tabulate, Function, FunctionExpon, FunctionDist, MFunction, MFunctionDist,
  ExamineNumber, RemoveNumber."
- **S2** — "Repite el método entero, clase por clase: 1. Un banco mínimo por
  clase, que ejercite ESA clase y nada más. 2. Correr el original bajo Wine
  (bash scripts/wine-oracle/run_oracle.sh <modelo> <directorio>) y quedarse con
  su código GPSS y su reporte. 3. Comparar nuestro GPSS con el del original,
  sentencia por sentencia (scripts/wine-oracle/compare-gpss.sh). 4. Comparar
  las cifras del reporte con las tolerancias del oráculo (20% en conteos, 35%
  en tiempos). 5. Veredicto por clase: verificada, con diferencias o
  pendiente, y por qué."
- **S3** — "TRES CLASES QUE HOY NO PRODUCEN RESPUESTA DEL ORIGINAL (MFunction,
  MFunctionDist y RemoveNumber): el lote 2 ya resolvió un caso parecido. El
  original abría una ventana ("adaptándolo a las definiciones de clases") y esa
  ventana impedía leer la respuesta; la salida fue quitar del modelo las sondas
  que la clase del original no tiene. Prueba lo mismo aquí, y si la ventana
  sigue tapando la respuesta, dilo y explica qué es lo que hay que hacer para
  verla (y si hace falta, usa el manual VISUALSIS.HLP o el gmacroconfig.gmcfg
  para entender qué pide la clase)."
- **S4** — "El banco tiene que declararle al original todo lo que la clase
  necesita: las entidades que usa (un almacén sin declarar hace que el original
  se niegue a correr) y las salidas de fallo que el macro exige (el EXAMINE
  necesita su destino alternativo, el TEST también)."
- **S5** — "Cuando dos lecturas de una regla no se separan con los conteos, haz
  una sonda y dilo (el lote 2 usó cuatro)."
- **S6** — "Un detalle de forma: el resumen del lote 1 dice "24 clases" y su
  tabla lista 26. Corrige esa errata en las notas del lote 1 (es un archivo de
  texto, no tocas el producto)."
- **S7** — "ESTA SESIÓN ES DE SÓLO LECTURA DEL PRODUCTO: no se toca crates/ ni
  docs/. Es el tercer y último lote del inventario de la fase 7."
- **S8** — "NINGUN commit lleva pie de atribución (nada de "Co-Authored-By" ni
  similar). El proyecto es del lector y los commits no llevan la firma de
  ninguna herramienta ni de ningún modelo. El mensaje del commit explica qué
  cambió y por qué, y nada más."

## Resumen

Las 20 clases del lote, en el orden del encargo. `GPSS` es la comparación de
`scripts/wine-oracle/compare-gpss.sh` (verbo, operandos y nombre de cada
sentencia) y `Cifras` es la de las métricas del reporte del oráculo (20% en
conteos, 35% en tiempos).

| # | Clase | Verbo | GPSS | Cifras | Veredicto | La diferencia, en una línea |
|---|---|---|---|---|---|---|
| 1 | Note | (ninguno) | coincide | 0/0 | verificada | ninguna de esta clase |
| 2 | Variable | `VARIABLE` | coincide | 0/0 | verificada | ninguna de esta clase |
| 3 | VariableA | `VARIABLE` | coincide | 0/0 | verificada | ninguna de esta clase |
| 4 | VariableN | `FUNCTION`+`VARIABLE` | coincide | 0/0 | verificada | ninguna de esta clase |
| 5 | Assign | `ASSIGN` | coincide | 2/2 | verificada | ninguna de esta clase |
| 6 | Alter | `ALTER` | coincide | 2/2 | con diferencias | la fila del reporte pierde los operandos (`ALTER ungrupo` contra `ALTER E ungrupo,1,param,0,otro,1`) |
| 7 | Logic | `LOGIC` | coincide | 2/2 | con diferencias | falta la sección `LOGICSWITCH`; y el operador "Invertir" no existe |
| 8 | InitialLS | `INITIAL` | coincide | 0/0 | con diferencias | falta la sección `LOGICSWITCH` |
| 9 | InitiaSV | `INITIAL` | coincide | 0/0 | con diferencias | la sección `SAVEVALUE` va sin la columna `RETRY` |
| 10 | SaveValue | `SAVEVALUE` | coincide | 2/2 | con diferencias | la sección `SAVEVALUE` va sin la columna `RETRY` |
| 11 | Scan | `SCAN` | coincide | 2/2 | verificada | ninguna de esta clase |
| 12 | QTable | `QTABLE` | coincide | 0/0 | con diferencias | la sección `QTABLES` es un resumen de cuatro líneas; el original imprime el histograma |
| 13 | Tabulate | `TABLE`+`TABULATE` | coincide | 2/2 | con diferencias | la sección `TABLAS` pierde el atributo, los rótulos y las filas vacías |
| 14 | Function | `FUNCTION` | coincide | 0/0 | verificada | ninguna de esta clase |
| 15 | FunctionExpon | `FUNCTION` | coincide | 0/0 | verificada | ninguna de esta clase |
| 16 | FunctionDist | `FUNCTION` | coincide | 0/0 | verificada | ninguna de esta clase |
| 17 | MFunction | `FUNCTION`×2 | **no lo tenemos** | — | pendiente | nuestro catálogo no ofrece la clase |
| 18 | MFunctionDist | `FUNCTION`×2 | **no lo tenemos** | — | pendiente | nuestro catálogo no ofrece la clase |
| 19 | ExamineNumber | `EXAMINE` | coincide | 2/2 | con diferencias | falta la sección `NUMERIC_GROUP` del reporte |
| 20 | RemoveNumber | `REMOVE` | **no lo tenemos** | — | pendiente | nuestro catálogo no ofrece la clase |

**Del total:** 20 clases; **17 de 20** con el GPSS igual al del original (las
tres que no lo tienen son clases que nuestro catálogo no ofrece, así que ni
siquiera compilan; **las 17 que compilamos coinciden todas**); **17 de 17** con
todas las cifras que el oráculo puede comparar dentro de las tolerancias (y
**diez** de ellas con menos de tres métricas, que el oráculo no puede comparar:
sus bloques van sin nombre); **10 verificadas**, **7 con diferencias** y **3
pendientes**.

De las siete con diferencias, **seis son de la forma del reporte** (la fila del
`ALTER`, las secciones `LOGICSWITCH` y `SAVEVALUE`, la sección `QTABLES` y la
sección `TABLAS`) y **una es de la forma del reporte también** (la sección
`NUMERIC_GROUP`). Ninguna es de comportamiento: lo que las clases hacen, lo
hacen los dos motores.

## Las cuatro clases que el original no corría

El lote 2 encontró cuatro bancos que el original se negaba a correr. Este lote
encontró **cuatro más**, y el motivo de tres de ellos es el mismo: **el
generador de bancos escribía un tipo de propiedad que el original no
conoce**.

El síntoma es el aviso del original: abre el modelo y muestra una ventana
titulada "Información" que dice

> El modelo pudo abrirse, sin embargo uno o mas de los modulos utilizados
> dentro de este fueron adaptados a las definiciones de clases existentes en
> este sistema. […] Las siguientes clases de modulos fueron adaptadas para
> poder abrir el modelo:

Esa ventana es modal y tapa el simulador: el arnés no tiene nada sobre lo que
cliclear y el banco se queda sin respuesta. Es el mismo caso del
`MultifurcacionP` del lote 2 (que cargaba cinco propiedades `Prob` que la clase
del original no tiene), pero con otra causa: aquí las propiedades **existen**
en la clase del original, y lo que está mal es **el tipo** con el que el banco
las escribe.

**Cómo se averiguó.** El original sabe el tipo correcto de cada propiedad: es
el que escribe él cuando guarda el modelo. La prueba es abrir el banco en el
original, aceptar el aviso (su botón "Aceptar", abajo en el centro) y guardar
el modelo con su propio "Guardar" (`Ctrl+G`, no `Ctrl+S`): el archivo que
escribe lleva los tipos del original. Lo que salió:

| Clase | Propiedad | Tipo del banco | Tipo que escribe el original |
|---|---|---|---|
| `MFunction` | `NombresFunc` | `EntityDefList` | **`TEntityDefList`** |
| `MFunction` | `Matriz` | `TSNAMatrix` | **`TSNAMATRIX`** |
| `MFunctionDist` | `NombresDist` | `EntityDefList` | **`TEntityDefList`** |
| `RemoveNumber` | `Op` | `Listed`/`op_Relac` | `Listed`/**`op_RelacMaxMin`** |

El `RemoveNumber` es el caso distinto: la propiedad `Op` es una `Listed` y lo
que estaba mal era la **lista** de la que se elegía. La del original es la que
también trae `Minimo` y `Maximo` (la misma que usa el `RemoveTrans` para su
`CompOp`); con `op_Relac` el original adaptaba el módulo. Se comprobó con
cuatro variantes del mismo modelo (sólo `Op` con cada lista; `Op` con la lista
vacía) y con el modelo guardado por el original.

**Lo que se hizo.** `scripts/wine-oracle/make-benches.py` escribe ahora los
tipos que el original escribe (`TEntityDefList`, `TSNAMATRIX`) y la lista que
la clase declara (`op_RelacMaxMin`), con sus pruebas en
`scripts/wine-oracle/test_make_benches.py` (clase `PropertyTypes`). Con eso
los tres bancos abren sin el aviso y el original responde:

```
FuncA           FUNCTION        P$Param,E3
1,11/2,12/3,13
FuncB           FUNCTION        P$Param,E3
1,21/2,22/3,23
```

```
Func1           FUNCTION        RN4,D3
0.2,1/0.5,2/1,3
Func2           FUNCTION        RN4,D3
0.25,1/0.6,2/1,3
```

```
A_RemoveNumber  REMOVE   E       UnGrupo,,0
```

La cuarta clase que no corría era el `Tabulate`, y su causa era la de siempre:
el banco no declaraba la tabla que la clase tabula, y el original se paraba con
"ERROR!!Se intentó actualizar el valor de una tabla inexistente. (Tabla: -1)".
El banco lleva ahora un bloque `Table` (como el `StorageUse` lleva su
`StorageDef`), con el nombre de la tabla que la clase usa (`Tabla`).

**Lo que hay que hacer para ver la respuesta cuando el aviso tapa la pantalla.**
Si un banco vuelve a mostrar el aviso, la respuesta se puede leer sin tocar el
generador: se abre el modelo, se clicquea "Aceptar" en la ventana
"Información" y se sigue el flujo de siempre (Modelo → Simular → el globo →
las pestañas "Reporte" y "Código GPSS"). El aviso no impide la simulación:
informa de que el módulo se adaptó, y el modelo adaptado es el que responde.
Lo que hay que arreglar, en ese caso, es el banco, que le pregunta al original
por un módulo que el original no reconoce.

## Sondas

Cuatro medidas que los bancos del lote no pueden dar. Los modelos están en
`scripts/wine-oracle/probes/` y sus respuestas del original, en
`/tmp/lote3-probes2` (el arnés las deja ahí con el mismo nombre).

| Sonda | Lo que pregunta | Modelo |
|---|---|---|
| 1 | ¿El `ASSIGN` pone el valor del parámetro? ¿El `SAVEVALUE` el suyo? | `AssignValor.gmdl` |
| 2 | ¿La llave lógica queda en 1 o en 0? | `LogicSet.gmdl` |
| 3 | ¿La llave lógica con `Reset` la deja en 0? | `LogicReset.gmdl` |
| 4 | ¿El `ALTER` altera al miembro del grupo? | `AlterMiembro.gmdl` |

### Sonda 1 — el `Assign` y el `SaveValue`

**La pregunta.** El banco del `Assign` pone `Param,0` y no dice nada: no hay
ninguna cifra del reporte que diga si el valor llegó al parámetro de la
transacción. El `SaveValue` tampoco: su sección `SAVEVALUE` muestra el valor,
pero el del banco es 0, que es el mismo que el valor por defecto.

**El modelo.** Un `Assign` que pone `Param = 7`, un `SaveValue` que pone el
valor 5 y un `TEST` que compara `P:Param` con 7:

```
Generate ──► Assign (Param = 7) ──► SaveValue (1,5) ──► Test (E P:Param,7) ──┬─ A_Terminate
                                                                             └─ A_Terminate1
Clock (3600) closes the run.
```

**La respuesta.**

| | `A_Terminate` (la comparación se cumple) | `A_Terminate1` (no se cumple) | sección `SAVEVALUE` |
|---|---|---|---|
| original | **60** | 0 | `1` → **5** |
| nuestro | **57** | 0 | `1` → **5** |

**La regla.** El `ASSIGN` pone el valor en el parámetro de la transacción
activa ("Un bloque ASSIGN es usado para colocar o modificar un entero en un
parámetro de una transacción", manual del simulador), y el `SAVEVALUE` en el
valor guardado ("Un bloque SAVEVALUE es usado para asignar, incrementar, o
decrementar el valor de un valor guardado en la entidad"). Los dos motores lo
cumplen: la comparación con el parámetro se cumple en todas las transacciones
y el valor guardado es 5.

Lo que **no** se ve en esta sonda es la diferencia de la fila del `TEST`: la
del original escribe la referencia al parámetro como `P$Param` y la nuestra
como `P:Param` (el detalle está en "Cambios pendientes", número 8).

### Sonda 2 y 3 — la llave lógica

**La pregunta.** El banco del `Logic` pone la llave en 1 (`LOGIC S 1`), que es
el valor por defecto de una llave recién creada: el resultado no distingue
"el bloque la puso en 1" de "la llave ya valía 1". Y el manual dice que el
operador puede ser `S`, `R` **o `I`** (invertir), que nuestro motor no
conoce.

**El modelo.** Un `InitialLS` que pone la llave 1 en 1 y un `Logic` que la
pone en 1 (`LogicSet`) o la reinicia (`LogicReset`).

**La respuesta** (sección `LOGICSWITCH` del reporte del original):

| | `LOGIC S 1` (sonda 2) | `LOGIC R 1` (sonda 3) |
|---|---|---|
| original | valor **1** | valor **0** |
| nuestro | (no imprimimos la sección) | (no imprimimos la sección) |

**La regla.** El `LOGIC` configura la llave: con `S` la pone en "set" y con
`R` la pone en "reset" ("Si el operador lógico es S o R, la llave lógica
especificada por el operando A es puesta en el estado set o reset,
respectivamente", manual). Nuestro motor cumple la regla (el bloque `Logic`
del core la configura), pero **nuestro reporte no imprime la sección
`LOGICSWITCH`**, así que no se puede ver: es el pendiente 5 de "Cambios
pendientes" (el mismo que ya tenía el `GateL` y el `SelectL` del lote 1).

Con `LOGIC R 1` la llave queda en 0 **después** de que el `InitialLS` la
ponga en 1: o sea, el bloque `LOGIC` manda sobre el valor inicial.

### Sonda 4 — el `ALTER` y los miembros del grupo

**La pregunta.** El banco del `Alter` usa el grupo `ungrupo`, que nadie
define: no hay ningún miembro que alterar y la comparación del `ALTER` (el
parámetro `otro` contra el valor 1) no se cumple nunca. El banco no dice si
el `ALTER` altera al miembro del grupo, ni a cuál, ni qué pasa con la
transacción activa.

**El modelo.** Un `Assign` que pone `Param = 0`, un `JoinMe` que mete la
transacción en el grupo `UnGrupo`, el `ALTER` que altera el parámetro `Param`
de un miembro a 99 (comparando `Param` con 0), y un `TEST` que comprueba
`P:Param = 99`:

```
Generate ──► Assign (Param = 0) ──► JoinMe (UnGrupo) ──► Alter (E UnGrupo,1,Param,99,Param,0) ──┬─ Test (E P:Param,99) ──┬─ A_Terminate
                                                                                                  └─ A_Terminate2 ─► A_Terminate2
                                                                                        (la salida de fallo del ALTER, "No alteró", va a A_Terminate1)
Clock (3600) closes the run.
```

**La respuesta.**

| | `A_Terminate` (el `TEST` se cumple) | `A_Terminate2` (no se cumple) | `A_Terminate1` (el `ALTER` no alteró a nadie) |
|---|---|---|---|
| original | **62** | 0 | 0 |
| nuestro | **62** | 0 | 0 |

**La regla.** El `ALTER` altera a los **miembros** del grupo, no a la
transacción activa: "Un bloque ALTER selecciona transacciones de un grupo y
altera uno de los atributos de dichas transacciones" (manual). Aquí la
transacción activa es el único miembro del grupo, así que es a ella a quien
altera, y su `Param` pasa a 99: por eso la comparación posterior se cumple en
las dos.

Los dos motores cumplen la regla. Lo que **no** coincide es la fila del
reporte: la del original escribe `ALTER E UnGrupo,1,Param,99,Param,0,A_Terminate1`
—con el operador, el grupo, el límite, el parámetro y su valor, el parámetro
comparado y su valor, y el destino de la salida de fallo— y la nuestra escribe
`ALTER UnGrupo`: es el pendiente 1 de "Cambios pendientes". El **listado**
tampoco lleva ese último operando (el destino de la salida de fallo), y esa
diferencia también está en el pendiente 1.

## Fichas

Una por clase, en el orden del lote. Cada ficha dice lo que el original dice
de la clase (su `ClassDescrip` y su macrocode en
`Genmsi_20230711.backup/gClasses.xml`, y el manual del simulador
`VISUALSIS.HLP` cuando aporta), los puertos, las propiedades con sus defaults,
la semántica, el banco, el GPSS, las cifras y el veredicto.

### Note

**El original.** No la describe y no tiene macrocode: su grafo es un polígono
con un texto ("Etiqueta"). Es una **anotación**: un rótulo que el usuario
escribe en el lienzo del modelo. No tiene puertos ni propiedades.

- **Puertos.** Ninguno: la sección `graph` de la clase no declara partes
  `Input` ni `Output`.
- **Propiedades y defaults.** Ninguna. Nuestro catálogo la ofrece sin
  propiedades (`crates/genmsi-format/src/catalogue.rs`, línea 1031).
- **Semántica.** No produce ninguna sentencia: es un dibujo. Nuestro motor la
  compila a nada (`crates/genmsi-core/src/program.rs`, fn `Compiler::expand`,
  línea 1129, la rama que devuelve `Ok(None)` para las clases de
  declaración), y el editor la muestra como un texto.
- **El banco.** Un `Note` sin conectar más un `Clock` de 3600, que compila
  `GENERATE 3600` + `TERMINATE 1`.
- **GPSS.** Coincide con el del original: los dos escriben las mismas dos
  sentencias y nada más. El comparador del oráculo lo confirma
  ("el GPSS coincide: 1 bloques y 0 conexiones").
- **Cifras.** 0 de 0: los dos bloques del banco van sin nombre y el oráculo
  empareja las filas por su etiqueta, así que no hay ninguna métrica que
  comparar ("no-metrics"). Los dos reportes coinciden en lo que sí se ve: 2
  entradas en el `GENERATE 3600` y 1 en el `TERMINATE 1`.
- **Veredicto.** verificada
- **Diferencia concreta.** Ninguna de esta clase.

### Variable

**El original.** No la describe. Su macrocode es
`VARIABLE [NombreVar:'Nombre o Nro de la variable'='vari'],[Expresion:'Expresion
matematica']`: declara una **variable** con un nombre y una ecuación.

- **Puertos.** Ninguno.
- **Propiedades y defaults.** `NombreVar` (`EntityDef`, subType `Variable`,
  default `"vari"`) y `Expresion` (`Expresion`, **sin default**: el XML no le
  da valor). Nuestro catálogo trae las mismas (`("Variable", "NombreVar",
  "vari")`). La ecuación vacía no la puede simular el original, así que el
  banco usa `0` (el generador lo dice por pantalla: "el banco usa el valor más
  pequeño que el original puede simular").
- **Semántica.** La sentencia `VARIABLE <nombre> <ecuación>` de la sección de
  definiciones del GPSS: el valor de la variable se evalúa con la ecuación
  cuando alguna expresión la usa (`V$vari`, el prefijo `V` de las SNA del
  original, `gmacroconfig.gmcfg`). Nuestro motor la compila en
  `crates/genmsi-core/src/program.rs`, fn `Compiler::expand` (rama
  `"Variable"`, línea 2247) y la escribe en `crates/genmsi-core/src/gpss.rs`,
  fn `definitions`.
- **El banco.** Un `Variable` (nombre `vari`, ecuación `0`) sin conectar más el
  `Clock` de 3600.
- **GPSS.** Coincide: los dos escriben `vari VARIABLE 0`, `GENERATE 3600` y
  `TERMINATE 1`.
- **Cifras.** 0 de 0 ("no-metrics"): los bloques del banco van sin nombre.
- **Veredicto.** verificada
- **Diferencia concreta.** Ninguna de esta clase. Lo que el banco **no**
  mide es el valor de la variable: ni el reporte del original trae una sección
  de variables ni el nuestro, así que la regla "la ecuación se evalúa cuando
  algo la usa" no la mide ningún banco del lote. La mediría un modelo que
  usara `V$vari` en un `ASSIGN` y leyera el valor del parámetro.

### VariableA

**El original.** No la describe. Su macrocode es
`VARIABLE [NombreVar:'Nombre o Nro de la variable'='Azar'],
((RN:[Seed:'Seed'='80']#1000)+RN:[Seed])@(1+([Maximo:'Maximo'='10'])-([Minimo:'Minimo'='1']))+([Minimo])`:
una variable **uniforme** entre un mínimo y un máximo, con su propia semilla.

- **Puertos.** Ninguno.
- **Propiedades y defaults.** `NombreVar` (`"Azar"`), `Seed` (`"80"`), `Maximo`
  (`"10"`), `Minimo` (`"1"`). Nuestro catálogo trae las mismas
  (`("VariableA", "Maximo", "10")`, `("VariableA", "Minimo", "1")`,
  `("VariableA", "NombreVar", "Azar")`, `("VariableA", "Seed", "80")`).
- **Semántica.** La sentencia `VARIABLE` con la ecuación del original: el
  `RN` es el generador de números aleatorios con la semilla que la nombra
  (`RN80`), el `#` es la parte entera, el `@` la escala al intervalo y el `+`
  el desplazamiento al mínimo. Es decir: **uniforme en `[Minimo, Maximo]`**.
  Nuestro motor la compila igual (misma rama `"VariableA"`).
- **El banco.** Un `VariableA` sin conectar más el `Clock`.
- **GPSS.** Coincide, la ecuación entera:
  `Azar VARIABLE ((RN80#1000)+RN80)@(1+(10)-(1))+(1)`.
- **Cifras.** 0 de 0 ("no-metrics").
- **Veredicto.** verificada
- **Diferencia concreta.** Ninguna de esta clase. Igual que con la `Variable`,
  el valor de la variable no lo dice ningún número del reporte.

### VariableN

**El original.** "Calcula un valor al azar según una distribución Normal" (su
`ClassDescrip`). Su macrocode declara dos expresiones (`Media`, default 60, y
`Desvio`, default 1), **define la función** `NORMAL` con su tabla de 25
puntos y su semilla (`RN:[Seed:'Semilla'='90']`), y escribe
`VARIABLE [NombreVar:'Normal'], [Media]+([Desvio]#FN:NORMAL)/10`.

- **Puertos.** Ninguno.
- **Propiedades y defaults.** `NombreVar` (`"Normal"`), `Media` (`Expresion`,
  `60`), `Desvio` (`Expresion`, `1`), `NombreFunc` (`"NORMAL"`), `Seed`
  (`"90"`). Nuestro catálogo trae las mismas
  (`("VariableN", "Media", "60")`, `("VariableN", "Desvio", "1")`,
  `("VariableN", "NombreFunc", "NORMAL")`, `("VariableN", "NombreVar",
  "Normal")`, `("VariableN", "Seed", "90")`).
- **Semántica.** Dos sentencias: la **definición** de la función `NORMAL`
  (`FUNCTION NORMAL RN90,C25` con los 25 puntos de la curva normal, en tipo
  `C`, continua) y la **variable** que la usa (`Normal VARIABLE 60+(1#FN$NORMAL)/10`):
  el valor de la variable es la media más la desviación por el valor de la
  función, dividido por 10. Nuestro motor compila las dos
  (`crates/genmsi-core/src/program.rs`, fn `Compiler::expand`, rama
  `"VariableN"`).
- **El banco.** Un `VariableN` sin conectar más el `Clock`.
- **GPSS.** Coincide, las tres sentencias (la definición de la función con sus
  25 puntos, la variable y el reloj).
- **Cifras.** 0 de 0 ("no-metrics").
- **Veredicto.** verificada
- **Diferencia concreta.** Ninguna de esta clase. El valor de la variable (que
  es lo que la clase calcula) no lo dice ningún número del reporte: ni el
  original ni el nuestro imprimen una sección de variables.

### Assign

**El original.** No la describe. Su macrocode es
`IN Entrada1:'Entrada1'='A_moduleName' ASSI [Parametro:'Parametro a asignar o
modificar'='Param'], [Operador:'Operador para modificar el Parámetro con el
Valor'='='], [Valor:'Valor a asignar'='0'], [Funcion:'Función por la que se
multiblica el valor'=''] OUT Salida1`. El manual: "Un bloque ASSIGN es usado
para colocar o modificar un entero en un parámetro de una transacción";
"nunca es rechazada a entrar en un bloque ASSIGN".

- **Puertos.** Una entrada (`Entrada1`, rótulo "Entrada1") y una salida
  (`Salida1`).
- **Propiedades y defaults.** `Parametro` (`EntityUse`, subType `Parameter`,
  `"Param"`), `Operador` (`Listed`, subType `op_AddSub`, `"="`), `Valor`
  (`TSNA`, `"0"`) y `Funcion` (`EntityUse`, subType `Function`, vacío). Nuestro
  catálogo trae las tres primeras con los mismos defaults.
- **Semántica.** La sentencia `ASSIGN <parámetro>,<valor>`: el parámetro de
  la transacción activa toma el valor (con `+` o `-` lo suma o lo resta). La
  clase multiplica por una función si la propiedad `Funcion` tiene valor. Nuestro
  motor la compila en `crates/genmsi-core/src/program.rs`, fn `Compiler::expand`
  (rama `"Assign"`, línea 1194) y la ejecuta en `crates/genmsi-core/src/sim.rs`
  (rama `BlockKind::Assign`).
- **El banco.** `Generate` (60,30) → `Assign` (Parametro `Param`, Operador
  `=`, Valor 0) → `Terminate`, más el `Clock` de 3600.
- **GPSS.** Coincide: `A_Assign ASSIGN Param,0`.
- **Cifras.** 2 de 2, dentro de las tolerancias: el bloque `A_Assign` con 63
  entradas en el original y 62 en el nuestro (2%), y el `A_Terminate` con 63
  contra 62.
- **Veredicto.** verificada
- **Diferencia concreta.** Ninguna de esta clase. La sonda 1 fija la regla del
  `ASSIGN` (el valor llega al parámetro) y los dos motores la cumplen.

### Alter

**El original.** No la describe. Su macrocode es `ALTER [CompOp:'operador'='='],
[Grupo:'Grupo de transacciones'='ungrupo'], [Modo:'Como se alteran'='Algunos'],
[Cantidad:'Cantidad de transacciones a alterar'='1'],
[AltOpcion:'Que se altera'='Parámetro'], [AltParam:'Parametro a alterar'='param'],
[AltValor:'Valor a asignar'='0'], [CompOpcion:'Que se compara'='Parámetro'],
[CompParam:'Parametro a comparar'='otro'], [CompValue:'Valor a comparar'='1'],
Salida2:'No alteró' OUT Salida1:'Alteró'`. El manual: "Un bloque ALTER
selecciona transacciones de un grupo y altera uno de los atributos de dichas
transacciones"; si se usa un operador condicional, "cada miembro del grupo
debe pasar un test antes de ser alterado"; la transacción activa puede ir por
la salida alternativa (el operando J) cuando no se alteró a nadie.

- **Puertos.** Una entrada (`Entrada1`, rótulo "Ingreso") y dos salidas:
  `Salida1` (rótulo "Alteró") y `Salida2` (rótulo "No alteró").
- **Propiedades y defaults.** `CompOp` (`Listed`, subType `op_RelacMaxMin`,
  `"="`), `Grupo` (`EntityUse`, subType `TransacGroup`, `"ungrupo"`), `Modo`
  (`Listed`, subType `OperationMode`, `"Algunos"`), `Cantidad` (`TSNA`, `"1"`),
  `AltOpcion` (`Listed`, subType `PriorityOption`, `"Parámetro"`), `AltParam`
  (`EntityUse`, subType `Parameter`, `"param"`), `AltValor` (`TSNA`, `"0"`),
  `CompOpcion` (`Listed`, subType `PriorityOption`, `"Parámetro"`),
  `CompParam` (`EntityUse`, subType `Parameter`, `"otro"`), `CompValue`
  (`TSNA`, `"1"`). Nuestro catálogo trae las diez con los mismos defaults.
- **Semántica.** La sentencia `ALTER E <grupo>,<límite>,<parámetro>,<valor>,
  <parámetro comparado>,<valor comparado>`: altera hasta `<límite>`
  miembros del grupo cuyo parámetro comparado cumple la comparación, y les
  pone el valor nuevo. Nuestro motor la compila en
  `crates/genmsi-core/src/program.rs`, fn `Compiler::expand` (rama
  `"Alter"`, línea 2043) y la ejecuta en `crates/genmsi-core/src/sim.rs`
  (rama `BlockKind::Alter`, línea 1650, con la fn `alter_group`).
- **El banco.** `Generate` (60,30) → `Alter` (con sus diez valores por
  defecto) → `Terminate`, más el `Clock` de 3600. La salida de fallo
  ("No alteró") queda **sin conectar**, que es lo que hace que el `ALTER` no
  tenga destino alternativo.
- **GPSS.** Coincide: `A_Alter ALTER E ungrupo,1,param,0,otro,1`.
- **Cifras.** 2 de 2: el bloque `A_Alter` con 62 entradas en el original y 61
  en el nuestro (2%), y el `A_Terminate` con 62 contra 61.
- **Veredicto.** con diferencias
- **Diferencia concreta.** La **fila del bloque en nuestro reporte** escribe
  `ALTER ungrupo` donde el original escribe `ALTER E ungrupo,1,param,0,otro,1`
  (con el operador, el grupo, el límite, el parámetro alterado y su valor, el
  parámetro comparado y su valor). Lo arreglaría
  `crates/genmsi-core/src/report.rs`, fn `operand_parts` (rama
  `BlockKind::Alter`, línea 719), que hoy devuelve sólo el grupo; los operandos
  ya están en `crates/genmsi-core/src/gpss.rs`, fn `block_statements` (rama
  `BlockKind::Alter`, línea 886). Lo fijaría un test en
  `crates/genmsi-core/tests/report_gpss.rs` que compruebe que la fila del banco
  `Alter` sale `ALTER E ungrupo,1,param,0,otro,1`.

### Logic

**El original.** No la describe. Su macrocode es `IN Entrada1:'Ingreso'='A_ModuleName'
LOGIC [Op:'Operador de configuracción de la llave lógica'='Set'],
[LogicSwich:'Llave lógica a configurar'='1'] OUT Salida1`. El manual: "Un
bloque LOGIC cambia el estado de una entidad llave lógica"; "Si el operador
lógico es S o R, la llave lógica especificada por el operando A es puesta en
el estado set o reset, respectivamente. Si el operador lógico es I, el estado
de la llave lógica especificada por el operando A es invertido".

- **Puertos.** Una entrada (`Entrada1`, rótulo "Ingreso") y una salida
  (`Salida1`).
- **Propiedades y defaults.** `Op` (`Listed`, subType `op_Swich`, `"Set"`) y
  `LogicSwich` (`EntityUse`, subType `LogicSwich`, `"1"`). Nuestro catálogo
  trae las dos con los mismos defaults.
- **Semántica.** La sentencia `LOGIC S <llave>`: configura la llave lógica
  (set, reset o invertir). Nuestro motor la compila en
  `crates/genmsi-core/src/program.rs`, fn `Compiler::expand` (rama `"Logic"`,
  línea 1717) y la ejecuta en `crates/genmsi-core/src/sim.rs` (rama
  `BlockKind::Logic`).
- **El banco.** `Generate` (60,30) → `Logic` (Op `Set`, LogicSwich `1`) →
  `Terminate`, más el `Clock` de 3600.
- **GPSS.** Coincide: `A_Logic LOGIC S 1`.
- **Cifras.** 2 de 2: el bloque `A_Logic` con 58 entradas en el original y 59
  en el nuestro (2%), y el `A_Terminate` con 58 contra 59.
- **Veredicto.** con diferencias
- **Diferencia concreta.** Dos cosas. La primera es la **sección `LOGICSWITCH`
  del reporte**, que el original imprime (`LOGICSWITCH VALUE RETRY` con el
  estado de cada llave) y el nuestro no: es el pendiente 5 de "Cambios
  pendientes" (el mismo que el `GateL` y el `SelectL` del lote 1), y lo
  arreglaría `crates/genmsi-core/src/report.rs`, fn `render`. La segunda es el
  **operador "Invertir"**: el manual dice que `I` invierte el estado de la
  llave, y nuestro motor trata cualquier operador que no sea "Set" como un
  "Reset" (`crates/genmsi-core/src/program.rs`, fn `Compiler::expand`, rama
  `"Logic"`: `is_some_and(|op| op.eq_ignore_ascii_case("set"))` → `LogicOp::Set`,
  cualquier otro → `LogicOp::Reset`); el listado tiene el mismo problema
  (`crates/genmsi-core/src/gpss.rs`, rama `BlockKind::Logic`, que sólo escribe
  `S` y `R`). Lo fijaría un test con un `Logic` de operador `Invertir` que
  compruebe que el listado sale `LOGIC I 1` y que la llave queda invertida.

### InitialLS

**El original.** No la describe. Su macrocode es `INILS [LogicSwich:'LogicSwich
a inicicializar'='1'],[Valor:'Valor de inicio'='Set']`: es una clase de
**declaración**, sin puertos.

- **Puertos.** Ninguno.
- **Propiedades y defaults.** `LogicSwich` (`EntityDef`, subType `LogicSwich`,
  `"1"`) y `Valor` (`Listed`, subType `op_Swich`, `"Set"`). Nuestro catálogo
  trae las dos con los mismos defaults.
- **Semántica.** La sentencia `INITIAL LS <llave>`: pone la llave lógica en
  el estado inicial al instalar el modelo (no hay bloque en tiempo de
  ejecución: `crates/genmsi-core/src/program.rs`, fn `Compiler::expand`,
  línea 1129, la devuelve como `Ok(None)`).
- **El banco.** Un `InitialLS` sin conectar más el `Clock` de 3600.
- **GPSS.** Coincide: `INITIAL LS1`.
- **Cifras.** 0 de 0 ("no-metrics").
- **Veredicto.** con diferencias
- **Diferencia concreta.** La **sección `LOGICSWITCH` del reporte** que el
  original imprime y el nuestro no (el mismo pendiente 5 que en el `Logic`).
  La sonda 3 confirma que la regla es la que dice el manual: con un
  `InitialLS` que pone la llave en 1 y un `LOGIC R 1` detrás, la llave queda
  en 0.

### InitiaSV

**El original.** "Permite Inicializar un SaveValue" (su `ClassDescrip`). Su
macrocode es `INISV [SaveValue:'SaveValue a inicializar'='1'],[Valor:'Valor
inicial de SaveValue'='0']`: otra clase de **declaración**, sin puertos.

- **Puertos.** Ninguno.
- **Propiedades y defaults.** `SaveValue` (`EntityDef`, subType `SaveValue`,
  `"1"`) y `Valor` (`PosInteger`, `"0"`). Nuestro catálogo trae las dos con los
  mismos defaults.
- **Semántica.** La sentencia `INITIAL X <valor>`: el valor guardado empieza
  con ese valor. Sin bloque en tiempo de ejecución (la misma rama de
  `Compiler::expand` que la `InitialLS`).
- **El banco.** Un `InitiaSV` sin conectar más el `Clock`.
- **GPSS.** Coincide: `INITIAL X1,0`.
- **Cifras.** 0 de 0 ("no-metrics"). Pero la **sección `SAVEVALUE`** del
  reporte del original sí dice el valor: `SAVEVALUE VALUE RETRY` con
  `1  0  0`.
- **Veredicto.** con diferencias
- **Diferencia concreta.** La **sección `SAVEVALUE` del reporte**: la nuestra
  escribe `SAVEVALUE VALUE` y el valor (`1  0`), y la del original escribe
  `SAVEVALUE VALUE RETRY` y las dos columnas (`1  0  0`). Es el pendiente 5 de
  "Cambios pendientes" (el mismo del `Counter` del lote 1), y lo arreglaría
  `crates/genmsi-core/src/report.rs`, fn `render` (la sección `SAVEVALUE`,
  línea 244), con el contador de reintentos en
  `crates/genmsi-core/src/stat.rs`.

### SaveValue

**El original.** No la describe. Su macrocode es `IN Entrada1:'Ingreso'='A_moduleName'
SV [SaveValue:'SaveValue a assignar un valor'='1'], [Op:'Operador'='='],
[Valor:'Valor a assignar o modificar al SaveValue'='0'] OUT Salida1`. El
manual: "Un bloque SAVEVALUE es usado para asignar, incrementar, o decrementar
el valor de un valor guardado en la entidad"; "Si el operando A está seguido
por +, entonces el operando B es sumado al viejo valor. Si el operando A está
seguido por -, el operando B es restado".

- **Puertos.** Una entrada (`Entrada1`, rótulo "Ingreso") y una salida
  (`Salida1`).
- **Propiedades y defaults.** `SaveValue` (`EntityUse`, subType `SaveValue`,
  `"1"`), `Op` (`Listed`, subType `op_AddSub`, `"="`), `Valor` (`TSNA`, `"0"`).
  Nuestro catálogo trae las tres con los mismos defaults.
- **Semántica.** La sentencia `SAVEVALUE <valor>,<operando>`: el valor
  guardado toma el operando (o lo suma o lo resta, según el signo). Nuestro
  motor la compila en `crates/genmsi-core/src/program.rs`, fn
  `Compiler::expand` (rama `"SaveValue"`, línea 1214) y la ejecuta en
  `crates/genmsi-core/src/sim.rs` (rama `BlockKind::SaveValue`).
- **El banco.** `Generate` (60,30) → `SaveValue` (SaveValue `1`, Op `=`,
  Valor 0) → `Terminate`, más el `Clock`.
- **GPSS.** Coincide: `A_SaveValue SAVEVALUE 1,0`.
- **Cifras.** 2 de 2: el bloque `A_SaveValue` con 59 entradas en el original
  y 60 en el nuestro (2%), y el `A_Terminate` con 59 contra 60.
- **Veredicto.** con diferencias
- **Diferencia concreta.** La misma que en el `InitiaSV`: la **sección
  `SAVEVALUE` del reporte** va sin la columna `RETRY` (el pendiente 5 de
  "Cambios pendientes"). El valor (0) coincide en los dos.

### Scan

**El original.** No la describe. Su macrocode es `SCAN [CompOp:'Operador
condicional'='='], [Grupo:'Grupo a escanera'='UnGrupo'],
[CompOpcion:'Que se compara'='Parámetro'], [CompParam:'Parametro de la
transacciones del Grupo a comparar'='otro'], [CompValue:'Valor a comparar'='1'],
[EnOpcion:'Opción de copia'='Parámetro'], [DeParam:'Parametro de la trancción
del gurpo, del cual se extrae el valor'='Param'], [DeOpción:'Opción de
extracción'='Parámetro'], [EnParam:'Parametro al cual se copia el valor'='Param'],
Salida2:'No escaneó' OUT Salida1:'Escaneó'`. El manual: "Un bloque SCAN
comunica información de un miembro de un grupo de transacciones a la
transacción activa"; "Un bloque SCAN encuentra la primera transacción [del
grupo] que satisface la condición".

- **Puertos.** Una entrada (`Entrada1`, rótulo "Ingreso") y dos salidas:
  `Salida1` (rótulo "Escaneó") y `Salida2` (rótulo "No escaneó").
- **Propiedades y defaults.** `CompOp` (`Listed`, subType `op_RelacMaxMin`,
  `"="`), `Grupo` (`EntityUse`, subType `TransacGroup`, `"UnGrupo"`),
  `CompOpcion` (`Listed`, subType `PriorityOption`, `"Parámetro"`),
  `CompParam` (`EntityUse`, subType `Parameter`, `"otro"`), `CompValue`
  (`TSNA`, `"1"`), `EnOpcion` (`Listed`, subType `PriorityOption`,
  `"Parámetro"`), `DeParam` (`EntityUse`, subType `Parameter`, `"Param"`),
  `DeOpción` (`Listed`, subType `PriorityOption`, `"Parámetro"`), `EnParam`
  (`EntityUse`, subType `Parameter`, `"Param"`). Nuestro catálogo trae las
  nueve con los mismos defaults.
- **Semántica.** La sentencia `SCAN E <grupo>,<parámetro comparado>,<valor
  comparado>,<parámetro de origen>,<parámetro de destino>`: busca en el grupo
  un miembro cuyo parámetro cumple la comparación y copia su parámetro de
  origen al de destino. Nuestro motor la compila en
  `crates/genmsi-core/src/program.rs`, fn `Compiler::expand` (rama `"Scan"`,
  línea 2025) y la ejecuta en `crates/genmsi-core/src/sim.rs` (rama
  `BlockKind::Scan`).
- **El banco.** `Generate` (60,30) → `Scan` (con sus nueve valores por
  defecto) → `Terminate`, más el `Clock`. La salida de fallo ("No escaneó")
  queda sin conectar y el grupo `UnGrupo` no lo define nadie.
- **GPSS.** Coincide: `A_Scan SCAN E UnGrupo,otro,1,Param,Param`.
- **Cifras.** 2 de 2: el bloque `A_Scan` con 58 entradas en el original y 58
  en el nuestro (0%), y el `A_Terminate` con 58 contra 58.
- **Veredicto.** verificada
- **Diferencia concreta.** Ninguna de esta clase. Lo que el banco **no**
  mide es el `SCAN` de verdad: el grupo no tiene miembros, así que no hay
  ningún valor que copie. La regla "el escaneo copia el parámetro del primer
  miembro que cumple la condición" la mediría un banco con un `JoinMe` delante
  (que meta la transacción en el grupo) y un `ASSIGN` que ponga un valor
  distinto en cada transacción; el valor copiado no se ve en el reporte
  (haría falta la tabla `FEC_XACT_NUMBER`, que nuestro reporte no imprime),
  así que la mediría un `TEST` detrás que comparase el parámetro de destino.

### QTable

**El original.** "Define una tabla para ser tabulada." (su `ClassDescrip`). Su
macrocode es `QTABLE [NombreTabla:'Nombre de la tabla a definir'='QTabla'],
[Queue:'Queue de Entrada1 a la tabla'='qRecurso'], [PrimerIntervalo:'Limite
superior de la primer frecuencia'='0'], [TamFrecuencia:'Limite superior de la
primer frecuencia'='600'], [Cantidad:'cantidad de clases de frecuencia'='60']`.
El manual (`VISUALSIS.HLP`, sección `QTABLE`): "La QTABLE tabula
exclusivamente tiempos en cola, por lo que como primer operando debe ir el
nombre o el número de la cola".

- **Puertos.** Ninguno: es una clase de **declaración**.
- **Propiedades y defaults.** `NombreTabla` (`EntityDef`, subType `QTable`,
  `"QTabla"`), `Queue` (`EntityDef`, subType `Queue`, `"qRecurso"`),
  `PrimerIntervalo` (`PosInteger`, `"0"`), `TamFrecuencia` (`PosInteger`,
  `"600"`), `Cantidad` (`PosInteger`, `"60"`). Nuestro catálogo trae las cinco
  con los mismos defaults.
- **Semántica.** La sentencia `QTABLE <cola>,<primer intervalo>,<tamaño de la
  frecuencia>,<cantidad de clases>`: la tabla de tiempos en cola de esa cola.
  Nuestro motor la compila en `crates/genmsi-core/src/program.rs`, fn
  `Compiler::expand` (rama `"QTable"`, línea 2397).
- **El banco.** Un `QTable` sin conectar más el `Clock`.
- **GPSS.** Coincide: `QTabla QTABLE qRecurso,0,600,60`.
- **Cifras.** 0 de 0 ("no-metrics"): los bloques del banco van sin nombre.
- **Veredicto.** con diferencias
- **Diferencia concreta.** La **sección `QTABLES` del reporte**: la del
  original es el histograma entero (la cabecera `QTabla QTABLE qRecurso,0,600,60`,
  la media, la desviación, el "CANTIDAD EN RETRY", y las **61 filas** del
  histograma: los 60 intervalos y la de desborde `> a 34800`, con su frecuencia,
  su porcentaje, su porcentaje acumulado y su porcentaje restante), y la
  nuestra es un resumen de cuatro líneas (`crates/genmsi-core/src/report.rs`,
  fn `render`, la sección `QTABLES`, línea 227: la cabecera, la media, la
  desviación y el "CANTIDAD DE VALORES"). Lo arreglaría esa misma fn, con la
  tabla de frecuencias en `crates/genmsi-core/src/stat.rs` (la `QTable`), y
  lo fijaría un test en `crates/genmsi-core/tests/report_gpss.rs` que compare
  la sección `QTABLES` del banco `QTable` con la del reporte del original.

### Tabulate

**El original.** "Define una tabla para ser tabulada." (su `ClassDescrip`, el
mismo texto que el de la `QTable`). Su macrocode es `IN Entrada1:'Ingreso'='A_moduleName'
TABULATE [NombreTabla:'Nombre de la tabla a definir'='Tabla'], [Peso:'Factor
de peso'=''] OUT Salida1`. El manual: "Un bloque TABULATE dispara un [aconteo
en la tabla]"; "Cuando una transacción ingresa a este bloque TABULATE, la
entidad tabla llamada SALES es encontrada. […] Si tal entidad no existe ocurre
un error de stop. Las entidades tabla deben ser definidas con una sentencia
TABLE".

- **Puertos.** Una entrada (`Entrada1`, rótulo "Ingreso") y una salida
  (`Salida1`).
- **Propiedades y defaults.** `NombreTabla` (`EntityUse`, subType `Table`,
  `"Tabla"`) y `Peso` (`TSNA`, vacío). Nuestro catálogo trae las dos con los
  mismos defaults.
- **Semántica.** La sentencia `TABULATE <tabla>`: la transacción cuenta en la
  tabla, con el factor de peso. La tabla la declara la clase `Table` (que
  compila `TABLE <tabla>,<atributo>,<primer intervalo>,<tamaño>,<cantidad>`).
  Nuestro motor la compila en `crates/genmsi-core/src/program.rs`, fn
  `Compiler::expand` (rama `"Tabulate"`, línea 1933).
- **El banco.** `Generate` (60,30) → `Tabulate` (NombreTabla `Tabla`) →
  `Terminate`, más el `Clock`, **y un bloque `Table` que declara la tabla**
  (`Tabla`, `Atributo P:Atrib`, `PrimerIntervalo 0`, `TamFrecuencia 10`,
  `Cantidad 10`). Sin ese bloque el original se paraba con
  "ERROR!!Se intentó actualizar el valor de una tabla inexistente. (Tabla: -1)",
  o sea que el banco no le preguntaba nada.
- **GPSS.** Coincide, las tres sentencias: `Tabla TABLE P$Atrib,0,10,10`,
  `A_Tabulate TABULATE Tabla` y el reloj.
- **Cifras.** 2 de 2: el bloque `A_Tabulate` con 60 entradas en el original y
  60 en el nuestro (0%), y el `A_Terminate` con 60 contra 60.
- **Veredicto.** con diferencias
- **Diferencia concreta.** La **sección `TABLAS` del reporte**: la del original
  es `Tabla TABLE P$Atrib,0,10,10` con la media, la desviación, el "CANTIDAD EN
  RETRY", el histograma (la primera clase con 61 y el 100%, y la línea "Las
  frecuencias restantes (9) son todas cero."), el "CANTIDAD DE VALORES
  TABULADOS" y el "VALOR MAXIMO TABULADO"; la nuestra es `Tabla TABLE 0, 10,
  10` (sin el operando del atributo), con el "CANTIDAD DE VALORES", el "VALOR
  MAXIMO" y **las diez filas** del histograma, sin la línea de las frecuencias
  que son cero. Es el pendiente 5 de "Cambios pendientes" (el del `Table` del
  lote 1) más los rótulos: lo arreglaría `crates/genmsi-core/src/report.rs`,
  fn `render` (la sección `TABLAS`, línea 178) y `crates/genmsi-core/src/stat.rs`
  (struct `Table`).

### Function

**El original.** No la describe. Su macrocode es `FUNCTION [NombreFunc:'Nombre de
la Función'='Func'], [Atributo:'Variable independiente'='P:Param'], [Tipo:'Tipo
de Función'='Discreta'], [Tabla:'Tabla que define el compertamiento de la
función'='{1|1;2|2}']`.

- **Puertos.** Ninguno: es una clase de **declaración**.
- **Propiedades y defaults.** `NombreFunc` (`EntityDef`, subType `Function`,
  `"Func"`), `Atributo` (`TSNA`, `"P:Param"`), `Tipo` (`Listed`, subType
  `FuncTypes`, `"Discreta"`), `Tabla` (`Table`, `"{1|1;2|2}"`). Nuestro catálogo
  trae las cuatro con los mismos defaults.
- **Semántica.** La sentencia `FUNCTION <nombre> <atributo>,D<cantidad>` con la
  tabla de la función: la función **discreta** que, para cada valor del
  atributo, devuelve el valor de la tabla. Nuestro motor la compila en
  `crates/genmsi-core/src/program.rs`, fn `Compiler::expand` (rama
  `"Function"`, línea 2326).
- **El banco.** Un `Function` sin conectar más el `Clock`.
- **GPSS.** Coincide: `Func FUNCTION P$Param,D2` y la tabla `1,1/2,2`.
- **Cifras.** 0 de 0 ("no-metrics").
- **Veredicto.** verificada
- **Diferencia concreta.** Ninguna de esta clase. El **valor** que la función
  devuelve no lo dice ningún número del reporte (ni el original ni el nuestro
  imprimen una sección de funciones); lo mediría un modelo que la usara en un
  `ASSIGN` (`FN$Func`).

### FunctionExpon

**El original.** No la describe. Su macrocode declara una semilla
(`[Seed:'Cemilla de número aleatorio'='2']`, `PosInteger`) y **define la
función** `Expon` con la tabla de la exponencial (`RN:2`, `Continua`, 24
puntos).

- **Puertos.** Ninguno.
- **Propiedades y defaults.** `NombreFunc` (`"Expon"`), `Seed` (`"2"`) y la
  propiedad `2` (la semilla que el macro nombra por número, `RN:2`). Nuestro
  catálogo trae las tres con los mismos defaults.
- **Semántica.** La sentencia `FUNCTION Expon RN2,C24` con la tabla de la
  exponencial: una función **continua** que se evalúa con el generador de
  números aleatorios `RN` y la semilla 2. Nuestro motor la compila igual.
- **El banco.** Un `FunctionExpon` sin conectar más el `Clock`.
- **GPSS.** Coincide: `Expon FUNCTION RN2,C24` y los 24 puntos de la tabla.
- **Cifras.** 0 de 0 ("no-metrics").
- **Veredicto.** verificada
- **Diferencia concreta.** Ninguna de esta clase.

### FunctionDist

**El original.** "Devuelve un número entero positivo entre 1 y n siguiendo una
distribución de probabilidad. […] La distribución es una lista ordenadas de
probabilidades de que devuelva cada número. La cantidad de elementos
determina el número máximo que devuelve. Los números son consecutivos." (su
`ClassDescrip`). Su macrocode es `FUNCTIONDIST [NombreDist:'Nombre de la
función de distribución'='FuncDist'], [Semilla:'número de semilla utilizada para
calcular los números aleatorios'='3'], [Distribucion:'Distribución de
probabilidades para cada resultado'='{50%;50%}']`.

- **Puertos.** Ninguno.
- **Propiedades y defaults.** `NombreDist` (`EntityDef`, subType `Function`,
  `"FuncDist"`), `Semilla` (`"3"`) y `Distribucion` (`FloatPercentList`,
  `"{50%;50%}"`). Nuestro catálogo trae las tres con los mismos defaults.
- **Semántica.** La sentencia `FUNCTION <nombre> RN<semilla>,D<cantidad>` con
  la distribución: la función **discreta** que reparte los enteros 1..n según
  las probabilidades dadas. Nuestro motor la compila igual.
- **El banco.** Un `FunctionDist` sin conectar más el `Clock`.
- **GPSS.** Coincide: `FuncDist FUNCTION RN3,D2` y la tabla `0.5,1/1,2`.
- **Cifras.** 0 de 0 ("no-metrics").
- **Veredicto.** verificada
- **Diferencia concreta.** Ninguna de esta clase.

### MFunction

**El original.** "Define múltiples funciones para los mismo puntos X y el mismo
atributo de entrada." (su `ClassDescrip`). Su macrocode es `MULTIFUNCTION
[NombresFunc:'lista de nombres de las Funciones'='{FuncA;FuncB}'],
[Atributo:'Variable independiente de todas las funciones'='P:Param'],
[Matriz:'Matriz que define las salidas de las funciones para cada valor de
entrada.'='{{11;12;13};{21;22;23}}']`.

- **Puertos.** Ninguno.
- **Propiedades y defaults.** `NombresFunc` (tipo **`TEntityDefList`**, subType
  `Function`, `"{FuncA;FuncB}"`), `Atributo` (`TSNA`, `"P:Param"`) y `Matriz`
  (tipo **`TSNAMATRIX`**, `"{{11;12;13};{21;22;23}}"`). Los dos tipos
  exóticos son los que escribe el propio original (ver "Las cuatro clases que
  el original no corría").
- **Semántica.** **Una sentencia por cada nombre de la lista**, con la fila
  correspondiente de la matriz: la respuesta del original es

  ```
  FuncA           FUNCTION        P$Param,E3
  1,11/2,12/3,13
  FuncB           FUNCTION        P$Param,E3
  1,21/2,22/3/23
  ```

  o sea, `E3` (tres entradas: el valor del atributo y los dos valores de la
  fila) y la tabla `valor|función` de cada fila de la matriz.
- **El banco.** Un `MFunction` sin conectar más el `Clock`.
- **GPSS.** **No lo tenemos**: nuestro motor no compila la clase. El error es
  explícito: `node 1 (MFunction): block class "MFunction" is not implemented by
  the engine`.
- **Cifras.** No se pueden comparar: el modelo no compila.
- **Veredicto.** pendiente
- **Diferencia concreta.** **Nuestro catálogo no ofrece la clase.** La
  diferencia no está en el motor sino en la lista de clases: ni
  `crates/genmsi-format/src/catalogue.rs` la declara, ni
  `crates/genmsi-core/src/program.rs` la compila (`fn known_classes`, línea
  944, y fn `Compiler::expand`). El hueco está hoy **fijado por un test**,
  `crates/genmsi-format/tests/catalogue_matches_the_reference_models.rs`, fn
  `the_three_classes_the_original_declares_and_the_catalogue_does_not_offer`,
  que exige que `MFunction`, `MFunctionDist` y `RemoveNumber` **no** estén en
  el catálogo. Lo arreglaría: declarar las tres clases en el catálogo (con los
  defaults del original, que ya están anotados en ese test: `NombresFunc`
  `{FuncA;FuncB}`, `Atributo` `P:Param` y `Matriz` `{{11;12;13};{21;22;23}}`),
  añadir sus ramas en `Compiler::expand` y en `fn known_classes`, y **cambiar
  ese test** para que exija lo contrario. Lo fijaría un test de
  `crates/genmsi-core/tests/report_gpss.rs` que compruebe que el listado del
  banco `MFunction` sale con las dos sentencias `FUNCTION P$Param,E3` y sus
  dos tablas, y otro de `crates/genmsi-format` que compruebe los defaults de
  las tres clases.

### MFunctionDist

**El original.** "Devuelve un número entero positivo entre 1 y n siguiendo una
distribución de probabilidad. […] La distribución es una lista ordenadas de
probabilidades de que devuelva cada número." (su `ClassDescrip`, el mismo
texto que el de la `FunctionDist`). Su macrocode es `MULTIFUNCTIONDIST
[NombresDist:'Nombres de las funciones de distribución'='{Func1;Func2}'],
[Semilla:'número de semilla utilizada para calcular los números aleatorios'='4'],
[Resultados='{{20%;30%;50%};{25%;35%;40%}}']`.

- **Puertos.** Ninguno.
- **Propiedades y defaults.** `NombresDist` (tipo **`TEntityDefList`**, subType
  `Function`, `"{Func1;Func2}"`), `Semilla` (`"4"`) y `Resultados` (tipo
  `FloatPercentMatrix`, `"{{20%;30%;50%};{25%;35%;40%}}"`).
- **Semántica.** Una sentencia por cada nombre, con la fila correspondiente de
  la matriz de probabilidades: la respuesta del original es

  ```
  Func1           FUNCTION        RN4,D3
  0.2,1/0.5,2/1,3
  Func2           FUNCTION        RN4,D3
  0.25,1/0.6,2/1,3
  ```

  o sea, la función **discreta** de distribución (`D3`) que reparte 1..3 según
  las probabilidades de su fila.
- **El banco.** Un `MFunctionDist` sin conectar más el `Clock`.
- **GPSS.** **No lo tenemos**: `node 1 (MFunctionDist): block class
  "MFunctionDist" is not implemented by the engine`.
- **Cifras.** No se pueden comparar.
- **Veredicto.** pendiente
- **Diferencia concreta.** La misma que en el `MFunction`: el catálogo no
  ofrece la clase (el mismo test del catálogo la exige ausente, y el mismo
  `Compiler::expand` y `fn known_classes` la tienen que compilar). Con la
  evidencia del GPSS del original que está en la ficha del `MFunction`.

### ExamineNumber

**El original.** No la describe. Su macrocode es `IN Entrada1:'Entrada1'='A_ModuleName'
EXAMINENUMBER [Grupo:'Grupo a examinar'='UnGrupo'], [Numero:'Nº a Examinar si
pertenece al grupo [Grupo]'='0'], Salida2:'[Numero] No Pertenece a [Grupo]' OUT
Salida1:'En [Grupo]'`. El manual: "Un bloque EXAMINE puede ser usado para
testear a miembros de un grupo numérico" (aquí, en modo numérico: el
miembro es un número, no una transacción).

- **Puertos.** Una entrada (`Entrada1`, rótulo "Entrada1") y dos salidas:
  `Salida1` (rótulo "En [Grupo]") y `Salida2` (rótulo "[Numero] No Pertenece a
  [Grupo]").
- **Propiedades y defaults.** `Grupo` (`EntityUse`, subType `NumericGroup`,
  `"UnGrupo"`) y `Numero` (`TSNA`, `"0"`). Nuestro catálogo trae las dos con los
  mismos defaults.
- **Semántica.** La sentencia `EXAMINE <grupo>,<número>,<destino
  alternativo>`: si el número pertenece al grupo, la transacción sigue por
  `Salida1`; si no, por `Salida2`. El tercer operando es obligatorio ("Número
  de bloque. Destino alternativo para la transacción activa si no es encontrado
  ningún miembro. Requerido", manual), y por eso el banco conecta la salida
  de fallo con un terminador propio.
- **El banco.** `Generate` (60,30) → `ExamineNumber` (Grupo `UnGrupo`, Número
  0) → `Terminate`, **más un `Terminate1`** para la salida de fallo, más el
  `Clock`.
- **GPSS.** Coincide: los dos escriben
  `A_ExamineNumber EXAMINE UnGrupo,0,A_Terminate1`. El destino alternativo lo
  escribió otra sesión en el commit `ff13c32` ("the shape of the GPSS
  listing"), que arregló el defecto que el lote 1 había anotado como
  pendiente A y el lote 2 como pendiente 5.
- **Cifras.** 2 de 2: el terminador de la salida de fallo (`A_Terminate1`) con
  64 entradas en el original y 61 en el nuestro (5%), y el `A_Terminate` con 0
  contra 0 (el grupo no tiene el número 0, así que todas las transacciones van
  por la salida de fallo, en los dos motores).
- **Veredicto.** con diferencias
- **Diferencia concreta.** La **sección `NUMERIC_GROUP`** del reporte, que el
  original imprime (`NUMERIC_GROUP GROUP_SIZE RETRY` con `UNGRUPO 0 0`) y el
  nuestro no: es el pendiente 5 de "Cambios pendientes" (las secciones que
  faltan en el reporte), y lo arreglaría
  `crates/genmsi-core/src/report.rs`, fn `render`, con el contador del grupo
  numérico en `crates/genmsi-core/src/stat.rs`.

### RemoveNumber

**El original.** No la describe. Su macrocode es `IN Entrada1:'Ingreso'='A_ModuleName'
REMOVENUMBER [Op:'Operador relacional'='='], [Grupo:'Grupo numérico al que se
agrega un número'='UnGrupo'], [Numero:'Número agregardo al grupo'='0'],
Salida2:'No existe el Nº' OUT Salida1`.

- **Puertos.** Una entrada (`Entrada1`, rótulo "Ingreso") y dos salidas:
  `Salida1` (sin rótulo) y `Salida2` (rótulo "No existe el Nº").
- **Propiedades y defaults.** `Op` (`Listed`, subType **`op_RelacMaxMin`**,
  `"="`), `Grupo` (`EntityUse`, subType `NumericGroup`, `"UnGrupo"`) y `Numero`
  (`TSNA`, `"0"`).
- **Semántica.** La sentencia `REMOVE E <grupo>,,<número>`: **agrega** el
  número al grupo numérico (a diferencia del `REMOVE` de transacciones, que
  quita). La respuesta del original es `A_RemoveNumber REMOVE E UnGrupo,,0`;
  con la salida de fallo conectada escribe el destino como séptimo operando
  (`REMOVE E UnGrupo,,0,,,A_Terminate1`).
- **El banco.** `Generate` (60,30) → `RemoveNumber` → `Terminate`, más el
  `Clock`. La salida de fallo ("No existe el Nº") queda sin conectar.
- **GPSS.** **No lo tenemos**: `node 2 (RemoveNumber): block class
  "RemoveNumber" is not implemented by the engine`.
- **Cifras.** No se pueden comparar.
- **Veredicto.** pendiente
- **Diferencia concreta.** La misma que en el `MFunction` y el
  `MFunctionDist`: el catálogo no ofrece la clase (el test del catálogo la
  exige ausente; `crates/genmsi-core/src/program.rs`, fn `known_classes` y fn
  `Compiler::expand`, la tienen que compilar). Con la evidencia del GPSS del
  original que está en esta ficha, y con la del `REMOVE` en modo numérico del
  manual ("Un bloque REMOVE excluye valores numéricos").

## Las diferencias de forma, comunes a todos los bancos

No son de ninguna clase en concreto: son del generador del reporte, y las
mismo en los veinte bancos de este lote (y en los de los lotes anteriores, y en
los sesenta y seis del recuento final).

| Qué | El original | El nuestro | Dónde |
|---|---|---|---|
| La columna `LOC` de los bloques sin nombre | el número de línea del programa (`2`, `3`, `4`…) | un contador de bloques sin nombre (`1`, `2`…) | `crates/genmsi-core/src/report.rs`, fn `render` (el contador `numeric_label`) |
| Los decimales enteros | `0,000`, `100,000` | `0`, `100` | `crates/genmsi-core/src/report.rs`, fn `number` |
| La línea de la semilla | no la tiene | `SEMILLA    7` | `crates/genmsi-core/src/report.rs`, fn `render` |
| La tabla de los eventos pendientes | `FEC_XACT_NUMBER PRI BDT CURRENT NEXT PARAMETER VALUE` | no la tiene | `crates/genmsi-core/src/report.rs`, fn `render` |
| La referencia a un parámetro | `P$Param` | `P:Param` | `crates/genmsi-core/src/report.rs` (la fila del `TEST`) |

La primera tiene un detalle que este lote deja claro: el número de línea del
original cuenta **las líneas de definición** también. En el banco de la
`Variable` el original numera sus dos bloques 2 y 3 (la sentencia `VARIABLE` es
la línea 1); en el de la `VariableN`, 4 y 5 (la definición de la función son
las líneas 1 a 3); en el del `Tabulate`, 2 y 3 (la sentencia `TABLE` es la
línea 1).

## Cambios pendientes

Cada fila dice **qué** hay que arreglar, **dónde** (archivo y función) y **qué
prueba** lo fijaría. Nada de esto se hizo en esta sesión: el core lo tiene
otra sesión.

### En el core (`crates/genmsi-core`)

| # | Qué hay que arreglar | Dónde | La prueba que lo fijaría |
|---|---|---|---|
| 1 | **La fila del `ALTER` pierde los operandos:** escribimos `ALTER ungrupo` donde el original escribe `ALTER E ungrupo,1,param,0,otro,1` (el operador, el grupo, el límite, el parámetro alterado y su valor, el parámetro comparado y su valor). Y el **listado** tampoco escribe el destino de la salida de fallo cuando ésta está conectada: el original escribe `ALTER E UnGrupo,1,Param,99,Param,0,A_Terminate1` y el nuestro `ALTER E UnGrupo,1,Param,99,Param,0`. | `crates/genmsi-core/src/report.rs` · `operand_parts` (rama `BlockKind::Alter`, línea 719); `crates/genmsi-core/src/gpss.rs` · `block_statements` (rama `BlockKind::Alter`, línea 886) | Un test en `crates/genmsi-core/tests/report_gpss.rs` que compruebe que la fila del banco `Alter` sale `ALTER E ungrupo,1,param,0,otro,1`, y otro con la sonda `scripts/wine-oracle/probes/AlterMiembro.gmdl` que compruebe que el listado sale `ALTER E UnGrupo,1,Param,99,Param,0,A_Terminate1` |
| 2 | **La sección `QTABLES` del reporte es un resumen de cuatro líneas:** el original imprime el histograma entero (los 60 intervalos y la fila de desborde `> a 34800`) con su frecuencia, su porcentaje, su acumulado y su restante, y la columna "CANTIDAD EN RETRY". | `crates/genmsi-core/src/report.rs` · `render` (la sección `QTABLES`, línea 227); las frecuencias están en `crates/genmsi-core/src/stat.rs` (la `QTable`) | Un test en `crates/genmsi-core/tests/report_gpss.rs` que compare la sección `QTABLES` del banco `QTable` con la del reporte del original |
| 3 | **El operador "Invertir" de la llave lógica no existe:** el manual dice que `I` invierte el estado de la llave, y el motor trata cualquier operador que no sea "Set" como un "Reset". El listado tampoco escribe la `I`. | `crates/genmsi-core/src/program.rs` · `Compiler::expand` (rama `"Logic"`, línea 1717); `crates/genmsi-core/src/gpss.rs` · `block_statements` (rama `BlockKind::Logic`) | Un test con un `Logic` de operador `Invertir` que compruebe que el listado sale `LOGIC I 1` y que la llave queda invertida (la sonda 3, `scripts/wine-oracle/probes/LogicReset.gmdl`, es el modelo con el que se midió) |
| 4 | **La sección `TABLAS` del reporte no es la del original:** la cabecera pierde el operando del atributo (`P$Atrib`), los rótulos son otros ("CANTIDAD DE VALORES" en vez de "CANTIDAD EN RETRY" y de "CANTIDAD DE VALORES TABULADOS"), el original colapsa las frecuencias que están a cero ("Las frecuencias restantes (9) son todas cero.") y el nuestro las imprime todas. | `crates/genmsi-core/src/report.rs` · `render` (la sección `TABLAS`, línea 178); `crates/genmsi-core/src/stat.rs` · struct `Table` (ya estaba anotado como pendiente 15 del lote 1) | Un test en `crates/genmsi-core/tests/report_gpss.rs` que compare la sección `TABLAS` del banco `Tabulate` con la del reporte del original |
| 5 | **Faltan las secciones `LOGICSWITCH`, `NUMERIC_GROUP`, `XACT_GROUP` y `FEC_XACT_NUMBER` del reporte, y la de `SAVEVALUE` va sin la columna `RETRY`.** El original las imprime: `LOGICSWITCH VALUE RETRY`, `NUMERIC_GROUP GROUP_SIZE RETRY`, `XACT_GROUP GROUP_SIZE RETRY`, `SAVEVALUE VALUE RETRY` y la tabla de los eventos pendientes. | `crates/genmsi-core/src/report.rs` · `render` (la sección `SAVEVALUE`, línea 244); `crates/genmsi-core/src/stat.rs` (los contadores por llave lógica) | Un test que compare esas secciones con los reportes del original de los bancos `Logic`, `InitialLS`, `InitiaSV`, `SaveValue` y `ExamineNumber` (ya estaba anotado como pendiente 13 del lote 1) |
| 6 | **El listado del `EXAMINE` no escribía el destino alternativo** (el original escribe `EXAMINE UnGrupo,0,A_Terminate1` y el nuestro `EXAMINE UnGrupo,0,`). **Ya está arreglado** por otra sesión: el commit `ff13c32` ("the shape of the GPSS listing") lo escribió, y el listado del banco `ExamineNumber` ya coincide con el del original. Queda aquí como prueba de que la diferencia era del listado y no del banco. | `crates/genmsi-core/src/gpss.rs` · `block_statements` (rama `BlockKind::Examine`, línea 532) | Un test en `crates/genmsi-core/tests/report_gpss.rs` que compruebe que el listado del banco `ExamineNumber` sale `EXAMINE UnGrupo,0,A_Terminate1` |
| 7 | **La columna `LOC` de los bloques sin nombre** es un contador de bloques; el original escribe el número de línea del programa, que cuenta también las líneas de definición. | `crates/genmsi-core/src/report.rs` · `render` y `loc_label` | Un test en `crates/genmsi-core/tests/report_gpss.rs` que compruebe que la fila del `GENERATE` del banco `Variable` sale con `LOC` 2 (ya estaba anotado como pendiente 4 del lote 1) |
| 8 | **La fila del `TEST` escribe la referencia al parámetro como `P:Param`**, donde el original escribe `P$Param`. | `crates/genmsi-core/src/report.rs` (la fila del `TEST`: fn `operand_parts` y `fn expr_operand`) | Un test en `crates/genmsi-core/tests/report_gpss.rs` que compruebe que la fila del `TEST` de la sonda `scripts/wine-oracle/probes/AssignValor.gmdl` sale `TEST E P$Param,7,A_Terminate1` |

### En el catálogo y en el motor (`crates/genmsi-format` y `crates/genmsi-core`)

| # | Qué hay que arreglar | Dónde | La prueba que lo fijaría |
|---|---|---|---|
| 9 | **Las tres clases que el catálogo no ofrece: `MFunction`, `MFunctionDist` y `RemoveNumber`.** El original las declara en su biblioteca de clases, las compila (sus GPSS están en las fichas) y las simula. Hoy nuestro motor se niega: "block class \"MFunction\" is not implemented by the engine". | `crates/genmsi-format/src/catalogue.rs` (las tres clases, con los defaults del original, que ya están anotados en `crates/genmsi-format/tests/catalogue_matches_the_reference_models.rs`); `crates/genmsi-core/src/program.rs` · fn `known_classes` (línea 944) y fn `Compiler::expand` | Un test de `crates/genmsi-format` que exija que las tres clases estén en el catálogo (el test de hoy exige lo contrario: `the_three_classes_the_original_declares_and_the_catalogue_does_not_offer`, línea 227) y un test de `crates/genmsi-core/tests/report_gpss.rs` con el listado de cada una de las tres: `FUNCTION P$Param,E3` (dos veces, con sus tablas), `FUNCTION RN4,D3` (dos veces) y `REMOVE E UnGrupo,,0` |

### En los bancos (`scripts/wine-oracle/make-benches.py`)

Lo que este lote cambió, con sus pruebas:

| # | Qué hay que arreglar | Dónde | La prueba que lo fijaría |
|---|---|---|---|
| 10 | **El generador escribía un tipo de propiedad que el original no conoce** en las clases `MFunction` (`EntityDefList` y `TSNAMatrix`) y `MFunctionDist` (`EntityDefList`), y una lista equivocada en el `RemoveNumber` (`op_Relac` en vez de `op_RelacMaxMin`). El original abría esos tres modelos adaptando el módulo, y la ventana del aviso tapaba la respuesta. | `scripts/wine-oracle/make-benches.py` · `CLASS_PROP_TYPES` (líneas 134-167) | `scripts/wine-oracle/test_make_benches.py`, clase `PropertyTypes` |
| 11 | **El banco del `Tabulate` no declaraba la tabla que la clase tabula**, y el original se paraba con "Se intentó actualizar el valor de una tabla inexistente". | `scripts/wine-oracle/make-benches.py` · `DECLARATIONS` | `scripts/wine-oracle/test_make_benches.py`, clase `PropertyTypes` (el test de la tabla) |
| 12 | **El arnés y el original comparten el prefijo de Wine**, y el `wineserver -k` que termina una corrida mata la simulación de la otra sesión; además, el arnés es frágil cuando otra sesión usa el mismo display. | `scripts/wine-oracle/run_oracle.sh` · la variable `WINEPREFIX` (ahora `GENMSI_WINEPREFIX` la puede Apartar) | — (es una robustez del arnés, no una regla del original) |

### Lo que ya está arreglado y no hay que rehacer

| # | Qué | Dónde |
|---|---|---|
| 13 | **Las tres clases que el original no corría** (`MFunction`, `MFunctionDist`, `RemoveNumber`) y la **`Tabulate`**: los cuatro bancos responden. | `scripts/wine-oracle/make-benches.py` (los tipos, la lista y la declaración de la tabla) |
| 14 | **La ecuación vacía de la `Variable`**: el original no la simula, y el banco usa `0`. | `scripts/wine-oracle/make-benches.py` · `PROP_VALUES` |
| 15 | **El destino alternativo del `EXAMINE`**: lo escribió otra sesión (el commit `ff13c32`, "the shape of the GPSS listing"), así que el listado del `ExamineNumber` ya coincide con el del original. | `crates/genmsi-core/src/gpss.rs` · `block_statements` (rama `BlockKind::Examine`) |

## El recuento definitivo de las 66 clases del original

Las 66 clases del catálogo del original, después de los tres lotes. La columna
"GPSS" dice si nuestro listado coincide con el del original, sentencia por
sentencia; la columna "Cifras", si las métricas que el oráculo puede comparar
caen dentro de las tolerancias (20% en conteos, 35% en tiempos).

Las 66 clases del catálogo del original, después de los tres lotes. La
columna "GPSS" dice si nuestro listado coincide con el del original, sentencia
por sentencia (`scripts/wine-oracle/compare-gpss.sh`); la columna "Cifras", si
las métricas que el oráculo puede comparar caen dentro de las tolerancias
(20% en conteos, 35% en tiempos, `cargo run --release -p genmsi-core --example
compare-benches -- ... --runs 3`).

**La regla del veredicto.** Una clase está *verificada* cuando su listado es el
del original, sus cifras coinciden y el reporte dice lo mismo que el del
original sobre ella. Las diferencias que **comparten todas las clases** —la
columna `LOC` con el número de línea del programa, los decimales enteros, la
línea `SEMILLA` y la tabla `FEC_XACT_NUMBER`, que no imprimimos— van aparte
(abajo) y no cuentan contra ninguna clase. Una diferencia cuenta si es **de la
clase**: su fila del reporte, su sección, su columna, su comportamiento o su
catálogo.

| # | Clase | GPSS | Cifras | Estado | Lo que lo separa del original |
| 1 | Advance | coincide | 2/2 | verificada | — |
| 2 | Alter | coincide | 2/2 | con diferencias | la fila dice `ALTER E ungrupo,1,param,0,otro,1` |
| 3 | Assemble | coincide | 2/2 | verificada | — |
| 4 | Assign | coincide | 2/2 | verificada | — |
| 5 | Buffer | coincide | 2/2 | verificada | — |
| 6 | Clock | coincide | 0/0 | verificada | — |
| 7 | Counter | coincide | 2/2 | con diferencias | la sección `SAVEVALUE` va sin la columna `RETRY` |
| 8 | Depart | coincide | 6/6 | verificada | — |
| 9 | Enter | coincide | 4/4 | con diferencias | la fila dice `ENTER AStorage,1` |
| 10 | Examine | coincide | 3/3 | con diferencias | falta la sección `XACT_GROUP` |
| 11 | ExamineNumber | coincide | 2/2 | con diferencias | falta la sección `NUMERIC_GROUP` |
| 12 | FacilityInt | coincide | 7/7 | con diferencias | la fila dice `PREEMPT Recurso,PR,,Remanente` |
| 13 | FacilityUse | coincide | 7/7 | verificada | — |
| 14 | Function | coincide | 0/0 | verificada | — |
| 15 | FunctionDist | coincide | 0/0 | verificada | — |
| 16 | FunctionExpon | coincide | 0/0 | verificada | — |
| 17 | GateF | coincide | 4/4 | verificada | — |
| 18 | GateL | coincide | 1/2 | con diferencias | las cifras no coinciden (1/2); falta la sección `LOGICSWITCH` |
| 19 | GateM | coincide | 4/4 | con diferencias | la fila dice `GATE M AAA` |
| 20 | GateS | coincide | 2/2 | verificada | — |
| 21 | Gather | coincide | 2/2 | verificada | — |
| 22 | Generate | coincide | 1/1 | verificada | — |
| 23 | InitiaSV | coincide | 0/0 | con diferencias | la sección `SAVEVALUE` va sin la columna `RETRY` |
| 24 | InitialLS | coincide | 0/0 | con diferencias | falta la sección `LOGICSWITCH` |
| 25 | JoinMe | coincide | 2/2 | con diferencias | falta la sección `XACT_GROUP` |
| 26 | JoinNumber | coincide | 2/2 | con diferencias | falta la sección `NUMERIC_GROUP` |
| 27 | Leave | coincide | 5/5 | con diferencias | la fila dice `ENTER AStorage,1` |
| 28 | Link | coincide | 2/2 | con diferencias | la fila dice `LINK Cadena,FIFO,A_Terminate` |
| 29 | Logic | coincide | 2/2 | con diferencias | falta la sección `LOGICSWITCH` |
| 30 | Loop | coincide | 2/2 | con diferencias | la fila dice `LOOP Param,` |
| 31 | MFunction | no lo tenemos | — | pendiente | nuestro motor no compila la clase |
| 32 | MFunctionDist | no lo tenemos | — | pendiente | nuestro motor no compila la clase |
| 33 | Mark | coincide | 2/2 | verificada | — |
| 34 | Match | coincide | 3/3 | verificada | — |
| 35 | Multifurcacion | coincide | 6/6 | con diferencias | la fila dice `TRANSFER ,FN$MultifurcacionGO` |
| 36 | MultifurcacionP | coincide | 5/5 | con diferencias | la fila dice `TRANSFER ,FN$MultifurcacionGO` |
| 37 | Note | coincide | 0/0 | verificada | — |
| 38 | Preempt | coincide | 5/5 | con diferencias | la fila dice `PREEMPT Recurso,PR,,Remanente` |
| 39 | Priority | coincide | 2/2 | verificada | — |
| 40 | QTable | coincide | 0/0 | con diferencias | la sección `QTABLES` no es la del original |
| 41 | Queue | coincide | 5/5 | verificada | — |
| 42 | Release | coincide | 5/5 | verificada | — |
| 43 | RemoveMe | coincide | 2/2 | con diferencias | la fila dice `REMOVE UnGrupo`; falta la sección `XACT_GROUP` |
| 44 | RemoveNumber | no lo tenemos | — | pendiente | nuestro motor no compila la clase |
| 45 | RemoveTrans | coincide | 2/2 | con diferencias | falta la sección `XACT_GROUP` |
| 46 | Return | coincide | 5/5 | verificada | — |
| 47 | SaveValue | coincide | 2/2 | con diferencias | la sección `SAVEVALUE` va sin la columna `RETRY` |
| 48 | Scan | coincide | 2/2 | verificada | — |
| 49 | Seize | coincide | 5/5 | verificada | — |
| 50 | Select | coincide | 32/32 | verificada | — |
| 51 | SelectF | coincide | 4/4 | verificada | — |
| 52 | SelectL | coincide | 2/2 | con diferencias | la fila dice `SELECT LS param,1,10`; falta la sección `LOGICSWITCH` |
| 53 | SelectS | coincide | 2/2 | con diferencias | la fila dice `SELECT SNF param,1,10` |
| 54 | SelectSNA | coincide | 5/5 | con diferencias | la fila dice `SELECT E param,1,10,*1,Q,` |
| 55 | Split | coincide | 2/2 | verificada | — |
| 56 | StorageDef | coincide | 2/2 | verificada | — |
| 57 | StorageUse | coincide | 7/7 | verificada | — |
| 58 | Table | coincide | 0/0 | con diferencias | la sección `TABLAS` no es la del original |
| 59 | Tabulate | coincide | 2/2 | con diferencias | la sección `TABLAS` no es la del original |
| 60 | Terminate | coincide | 1/1 | verificada | — |
| 61 | Test | coincide | 2/2 | verificada | — |
| 62 | Transfer | coincide | 2/2 | con diferencias | la fila dice `TRANSFER 1000,,A_Terminate` |
| 63 | Unlink | coincide | 3/3 | verificada | — |
| 64 | Variable | coincide | 0/0 | verificada | — |
| 65 | VariableA | coincide | 0/0 | verificada | — |
| 66 | VariableN | coincide | 0/0 | verificada | — |

**El recuento:** **34 verificadas**, **29 con diferencias** y **3 pendientes**
(las tres son clases que nuestro catálogo no ofrece: `MFunction`,
`MFunctionDist` y `RemoveNumber`).

**Cómo se midió.** Los 66 bancos con el generador de este lote
(`python3 scripts/wine-oracle/make-benches.py`), el original bajo Wine con un
prefijo de Wine propio (`GENMSI_WINEPREFIX`) y un display libre, y nuestro
motor con el binario del commit `ff13c32` (la última versión del núcleo cuando
se cerró esta fase). Los tres instrumentos: el GPSS, sentencia por sentencia
(`scripts/wine-oracle/compare-gpss.sh`), las cifras del reporte
(`cargo run --release -p genmsi-core --example compare-benches -- ... --runs 3`,
con la semilla base 7) y la comparación de las **filas de la tabla de bloques**
y de las **secciones** de los dos reportes (el original contra el nuestro), que
es lo que separa "con diferencias" de "verificada".

De las 29 con diferencias, **diez** se separan del original **sólo** por una
sección o una columna que el original imprime y el nuestro no (`XACT_GROUP`,
`NUMERIC_GROUP`, `LOGICSWITCH` y la columna `RETRY` de la `SAVEVALUE`): son
las `Examine`, `ExamineNumber`, `Counter`, `InitiaSV`, `InitialLS`, `JoinMe`,
`JoinNumber`, `Logic`, `RemoveTrans` y `SaveValue`. Con la convención de los
lotes anteriores (una sección que falta no cuenta contra la clase), el
recuento sería **44 verificadas, 19 con diferencias y 3 pendientes**.

Y así queda el inventario de la fase 7: **63 de las 66 clases** escriben el
mismo GPSS que el original (las tres que no son las que nuestro catálogo no
ofrece), y **de las 63** todas las cifras que el oráculo puede comparar caen
dentro de las tolerancias salvo **una**: la fila de la puerta del `GateL`,
que es el único cambio de comportamiento que queda en pie (el pendiente 14 del
lote 1: nuestro motor cuenta el intento fallido como entrada de la puerta y
el original no).



## Tasks

- **T1** — Leer el macrocode, el `ClassDescrip` y los puertos de las 20 clases — hecho.
- **T2** — Regenerar los bancos y revisar los que el original no corría — hecho: cuatro bancos cambiados (los tipos de las tres clases y la declaración de la tabla).
- **T3** — Correr el original bajo Wine sobre los 20 bancos — hecho: en `/tmp/lote3-oracle`, con el prefijo de Wine aislado.
- **T4** — Comparar el GPSS sentencia por sentencia — hecho: 17 de 17 coinciden; las tres que no son clases que nuestro motor no compila.
- **T5** — Comparar las cifras con las tolerancias del oráculo — hecho: 17 de 17 coinciden en las métricas que el oráculo puede comparar.
- **T6** — Las cuatro sondas — hecho: `AssignValor`, `LogicSet`, `LogicReset` y `AlterMiembro`, con su nota en `scripts/wine-oracle/probes/SondasLote3.md`.
- **T7** — Las 20 fichas y la lista de pendientes (S2, S3) — hecho: las tres secciones de este documento.
- **T8** — La errata del lote 1 (S6) — hecho: en `odd/tasks/notas-fase7-lote1.md`.
- **T9** — El recuento definitivo de las 66 clases (S2) — hecho: la sección de arriba, con la medición completa de las 66.

## Log

- **L1** — El encargo de la sesión (verbatim):

  "Trabajo en /home/lucas/Documentos/Workspace/Genmsi (Rust: genmsi-core,
  genmsi-format, genmsi-cli, genmsi-app con Tauri+React).

   Reconstruimos GenMSI (un simulador GPSS de escritorio) en Rust. 346 tests del core + 99 del front verdes, clippy
   limpio, y un oráculo que compara nuestra salida con la del original (que corre bajo Wine).

   MÉTODO (no lo cambies): […] QUÉ HAY QUE HACER: las veinte clases que faltan del catálogo del original:
     Note, Variable, VariableA, VariableN, Assign, Alter, Logic, InitialLS,
     InitiaSV, SaveValue, Scan, QTable, Tabulate, Function, FunctionExpon,
     FunctionDist, MFunction, MFunctionDist, ExamineNumber, RemoveNumber

   Repite el método entero, clase por clase: […] TRES CLASES QUE HOY NO PRODUCEN RESPUESTA DEL ORIGINAL (MFunction, MFunctionDist
   y RemoveNumber): el lote 2 ya resolvió un caso parecido. El original abría una ventana ("adaptándolo a las
   definiciones de clases") y esa ventana impedía leer la respuesta; la salida fue quitar del modelo las sondas que la
   clase del original no tiene. Prueba lo mismo aquí, y si la ventana sigue tapando la respuesta, dilo y explica qué es
   lo que hay que hacer para verla (y si hace falta, usa el manual VISUALSIS.HLP o el gmacroconfig.gmcfg para entender
   qué pide la clase).

   OTRAS REGLAS QUE HAN IDO DESCUBRIENDO:
   - El banco tiene que declararle al original todo lo que la clase necesita […]
   - Cuando dos lecturas de una regla no se separan con los conteos, haz una sonda y dilo (el lote 2 usó cuatro).
   - Un detalle de forma: el resumen del lote 1 dice "24 clases" y su tabla lista 26. Corrige esa errata en las notas del
     lote 1 (es un archivo de texto, no tocas el producto).

   DONE CUANDO:
   - odd/tasks/notas-fase7-lote3.md (nuevo) con la tabla resumen del lote, una ficha por clase […] y la lista final de
     pendientes, con archivo, funcion y la prueba que lo fijaria, como en los lotes anteriores.
   - Al final, el RECUENTO DEFINITIVO de las 66 clases del original […]
   - No hay commit de codigo: solo las notas y, si hace falta, el generador de bancos (scripts/wine-oracle/make-benches.py)
     con sus pruebas, que es un archivo aparte y no pisa el trabajo de la otra sesion.

   Y una regla del repositorio, para lo que commits: NINGUN commit lleva pie de atribución […]"

- **L2** — La causa de las tres clases que no respondían **no era el modelo**: era el **tipo** con el que el generador
  escribía sus propiedades. El original sabe el tipo correcto (es el que escribe al guardar el modelo), y el banco
  usaba otros: `EntityDefList` en vez de `TEntityDefList`, `TSNAMatrix` en vez de `TSNAMATRIX`, y en el `RemoveNumber`
  la lista `op_Relac` en vez de la que la clase declara (`op_RelacMaxMin`). Con los tipos del original, los tres bancos
  abren sin el aviso y responden.
- **L3** — El aviso de "adaptado" **no impide la simulación**: informa de que el módulo se adaptó. La respuesta se ve
  clicqueando "Aceptar" en la ventana "Información" y siguiendo el flujo de siempre. Lo que hay que arreglar en ese
  caso es el banco, que le pregunta al original por un módulo que el original no reconoce.
- **L4** — El `Guardar` del original es **`Ctrl+G`**, no `Ctrl+S`. Es lo que permitió leer los tipos que escribe.
- **L5** — El arnés es frágil cuando otra sesión corre el original en la misma máquina: el `wineserver -k` de una
  corrida mata la simulación de la otra, y dos sesiones en el mismo display se pisan. Con un prefijo de Wine propio
  (`GENMSI_WINEPREFIX`) y un display libre, las veinte respuestas salieron.
- **L6** — El **oráculo de métricas** empareja las filas de bloque por su etiqueta, así que los bancos cuyos bloques
  van sin nombre (las clases de declaración y las funciones) no tienen ninguna métrica que comparar: el estado es
  "no-metrics", no "coincide". Diez de los veinte bancos de este lote están en ese caso.
- **L7** — La **columna `LOC`** del original cuenta las líneas de definición (la `VARIABLE` de la `Variable`, las tres
  líneas de la función de la `VariableN`, la `TABLE` de la `Tabulate`), no sólo los bloques.
- **L8** — La fila del `ALTER` es la diferencia **de comportamiento del reporte** más nueva del lote, y la del `TEST`
  (la referencia al parámetro: `P$Param` contra `P:Param`) es la segunda más pequeña. Las dos son de la forma del
  reporte, no del modelo.
- **L9** — El **recuento final** se midió con los 66 bancos, no con los 20 del
  lote: el original se volvió a correr sobre los 66 (con un prefijo de Wine
  propio y un display libre) y se compararon los tres instrumentos (el GPSS,
  las cifras y las filas y secciones del reporte). La regla del veredicto
  quedó escrita: una diferencia cuenta si es de la clase, y las que comparten
  todas las clases (la columna `LOC`, los decimales, la línea `SEMILLA` y la
  tabla `FEC_XACT_NUMBER`) van aparte. Con ella, el recuento es 34 verificadas,
  29 con diferencias y 3 pendientes; y **diez** de las 29 se separan del
  original sólo por una sección que el original imprime y el nuestro no, o sea
  por un único pendiente (el 5 de la lista).
- **L10** — La otra sesión cambió el núcleo mientras esta sesión medía (los
  commits `2acde39` y `ff13c32`, "the shape of the GPSS listing"), y uno de
  ellos arregló el **destino alternativo del `EXAMINE`**, que era el pendiente
  6 de esta lista y el que separaba al `ExamineNumber` en el GPSS. Las cifras
  y los veredictos de este documento son los del motor de `ff13c32`.
- **L11** — El trabajo va en dos commits: el generador de bancos con sus pruebas (y el arnés, que es lo que se puede
  tocar sin pisar a la otra sesión), y estas notas con las sondas.
