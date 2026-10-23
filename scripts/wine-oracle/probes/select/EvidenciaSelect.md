# Evidencia: cómo el original compila el `SELECT` y el `EXAMINE`

Las sondas de este directorio le preguntan al original (`genmsi.exe`, bajo
Wine) tres cosas que el motor no sabía:

1. qué escribe el listado del programa cuando el `SELECT` escoge una entidad
   de cada una de las 38 clases de la lista `entitySNAclass`;
2. qué letra escribe cuando el `SELECT` escoge por el **estado** de un
   almacén (los operadores de `op_Logic_Stor`);
3. qué escribe el `EXAMINE` cuando su salida de fallo está conectada.

Los modelos y las respuestas del original son artefactos generados: no se
commitean (como los bancos). Lo que se commitea es el generador, el arnés y
esta tabla, que es la evidencia.

## Cómo se hicieron

Los modelos se generan con el catálogo del propio original
(`Genmsi_20230711.backup/gClasses.xml`), que es lo que evita el diálogo de
"adaptación" que cuelga el arnés:

```
python3 scripts/wine-oracle/probes/select/make-select-probes.py
```

Cada sonda se corre con el arnés del repositorio:

```
XDISPLAY=:96 bash scripts/wine-oracle/run_oracle.sh \
    scripts/wine-oracle/probes/select/models/<modelo>.gmdl <directorio>
```

(`scripts/wine-oracle/probes/select/run-probe.sh` es el mismo arnés con
capturas de pantalla, para los modelos que no terminan.)

## Las 38 clases de `entitySNAclass`

El verbo es el que dice la propiedad `op` del módulo (`Minimo` → `MIN`) y la
clase es el último operando. Las 38:

| Clase | Lo que escribe el original | Modelo |
|---|---|---|
| `BV` | `SELECT MIN param,1,10,,BV` | `select_BV` |
| `CA` | `SELECT MIN param,1,10,,CA` | `select_CA` |
| `CC` | `SELECT MIN param,1,10,,CC` | `select_CC` |
| `CH` | `SELECT MIN param,1,10,,CH` | `select_CH` |
| `CM` | `SELECT MIN param,1,10,,CM` | `select_CM` |
| `CT` | `SELECT MIN param,1,10,,CT` | `select_CT` |
| `F` | `SELECT MIN param,1,10,,F` | `select_F` |
| `FC` | `SELECT MIN param,1,10,,FC` | `select_FC_alone` |
| `FN` | `SELECT MIN param,1,10,,FN` | `select_FN` |
| `FR` | `SELECT MIN param,1,10,,FR` | `select_g1` |
| `FT` | `SELECT MIN param,1,10,,FT` | `select_g4` |
| `FV` | `SELECT MIN param,1,10,,FV` | `select_FV` |
| `GN` | `SELECT MIN param,1,10,,GN` | `select_GN` |
| `GT` | `SELECT MIN param,1,10,,GT` | `select_GT` |
| `LR` | `SELECT MIN param,1,10,,LR` | `select_LR` |
| `LS` | `SELECT MIN param,1,10,,LS` | `select_LS` |
| `MP` | `SELECT MIN param,1,10,,MP` | `select_MP` |
| `N` | `SELECT MIN param,1,10,,N` | `select_N_alone` |
| `P` | `SELECT MIN param,1,10,,P` | `select_P` |
| `Q` | `SELECT MIN param,1,10,,Q` | `bench_Select` |
| `QA` | `SELECT MIN param,1,10,,QA` | `select_g3` |
| `QC` | `SELECT MIN param,1,10,,QC` | `select_g2` |
| `QM` | `SELECT MIN param,1,10,,QM` | `select_g2` |
| `QT` | `SELECT MIN param,1,10,,QT` | `select_QT_alone` |
| `QX` | `SELECT MIN param,1,10,,QX` | `select_QX_alone` |
| `QZ` | `SELECT MIN param,1,10,,QZ` | `select_QZ_alone` |
| `R` | `SELECT MIN param,1,10,,R` | `select_R` |
| `RN` | `SELECT MIN param,1,10,,RN` | `select_RN` |
| `S` | `SELECT MIN param,1,10,,S` | `select_S` |
| `SA` | `SELECT MIN param,1,10,,SA` | `select_SA` |
| `SC` | `SELECT MIN param,1,10,,SC` | `select_SC` |
| `SM` | `SELECT MIN param,1,10,,SM` | `select_SM` |
| `SR` | `SELECT MIN param,1,10,,SR` | `select_SR` |
| `ST` | `SELECT MIN param,1,10,,ST` | `select_ST` |
| `SV` | `SELECT MIN param,1,10,,SV` | `select_g4` |
| `V` | `SELECT MIN param,1,10,,V` | `select_g4` |
| `W` | `SELECT MIN param,1,10,,W` | `select_g1` |
| `X` | `SELECT MIN param,1,10,,X` | `select_X` |

## La selección por el estado de un almacén

Los seis operadores de `op_Logic_Stor`. La columna de la izquierda es la
propiedad `op` del módulo `SelectS`; las dos del medio son el listado del
original con la salida de fallo (`Salida2`, "No selec") **sin** conectar y
**con** conectar:

| `op` | Sin salida de fallo | Con la salida conectada |
|---|---|---|
| `Lleno` | `SELECT SF param,1,10` | `SELECT SF param,1,10,,,A_Terminate1` |
| `NoLleno` | `SELECT SNF param,1,10` | `SELECT SNF param,1,10,,,A_Terminate1` |
| `Vacio` | `SELECT SE param,1,10` | `SELECT SE param,1,10,,,A_Terminate1` |
| `NoVacio` | `SELECT SNE param,1,10` | `SELECT SNE param,1,10,,,A_Terminate1` |
| `Disponible` | `SELECT SV param,1,10` | `SELECT SV param,1,10,,,A_Terminate1` |
| `NoDisponible` | `SELECT SNV param,1,10` | `SELECT SNV param,1,10,,,A_Terminate1` |

Los modelos son `sels_<op>.gmdl` (con la salida conectada) y
`sels_<op>_libre.gmdl` (sin ella).

## El destino de la salida de fallo de las otras clases de la familia

Las sondas `select_<clase>_dest.gmdl` (una por clase de la familia, con la
salida de fallo conectada) miden lo que el original escribe en el listado
cuando tiene un destino que escribir. Las tres:

| Clase | Lo que escribe el original |
|---|---|
| `SelectF` | `SELECT NU param,1,10,,,A_Terminate1` |
| `SelectL` | `SELECT LS param,1,10,,,A_Terminate1` |
| `SelectSNA` | `SELECT E param,1,10,*1,Q,A_Terminate1` |

La clase `SelectSNA` es la excepción: su campo de destino es propio y lo
escribe siempre, aunque esté vacío (`SELECT E param,1,10,*1,Q,` es lo que
escribe el banco del oráculo cuando la salida de fallo no está conectada).
La clase `Select` (la del `MIN` y el `MAX`) no tiene salida de fallo: su
sentencia es `SELECT MIN param,1,10,,Q` y no lleva destino.

## El `EXAMINE` sin destino

La sonda `examine_sin_destino.gmdl` (el `Examine` con la salida de fallo sin
conectar) no deja listing: el original se niega a abrir el modelo y dice

```
Error en linea     1 -> Faltan parámetros
```

Con la salida conectada (`examine_probe.gmdl`) el modelo corre y el listado
es `A_Examine EXAMINE UnGrupo,,A_Terminate1`.

## La disponibilidad de un almacén (los operadores `SV` y `SNV`)

La selección deja en un parámetro el número de la entidad que eligió (y un
cero cuando no eligió ninguna, según el manual: "Si la entidad no es
encontrada, 0 es colocado en el parámetro de la transacción activa"). Las
sondas `par_<op>_<estado>.gmdl` leen ese parámetro con un `TEST` y dicen
cuántas transacciones lo encontraron.

El modelo declara el almacén `1` (el `SELECT` camina las entidades cuyo
número cae en el rango de la sentencia, `1,10`, así que un almacén llamado
`Almacen` nunca está en él) y lo pone en uno de tres estados: vacío, con una
unidad dentro, o lleno.

| Sonda | `op` | Estado del almacén | Lo encontró | No lo encontró |
|---|---|---|---|---|
| `par_se_vacio` | `Vacio` | vacío | 59 | 0 |
| `par_se_uno` | `Vacio` | una unidad | 0 | 120 |
| `par_sv_vacio` | `Disponible` | vacío | 55 | 0 |
| `par_sv_uno` | `Disponible` | una unidad | 114 | 0 |
| `par_snv_vacio` | `NoDisponible` | vacío | 0 | 57 |
| `par_snv_lleno` | `NoDisponible` | lleno | 0 | 120 |

La regla que queda: el `SV` elige el almacén y el `SNV` no elige ninguno.
Es la que dice el manual del simulador (`VISUALSIS.HLP`): "El storage tiene
que estar disponible para que sea seleccionado" y "El storage no tiene que
estar disponible para que sea seleccionado". El estado de "disponible" es
una bandera del almacén que sólo cambian las sentencias `SAVAIL` y
`SUNAVAIL` (la biblioteca de clases del original no tiene ninguna de las
dos), así que un almacén de un modelo del editor siempre está disponible.

## El `EXAMINE`

Con la salida de fallo conectada a un terminador, el original escribe:

```
A_Examine       EXAMINE         UnGrupo,,A_Terminate1
```

y sin ella no compila el modelo ("Error en linea 1 -> Faltan parámetros").
Es lo mismo que dice el manual del simulador (`VISUALSIS.HLP`): "Número de
bloque. Destino alternativo para la transacción activa si no es encontrado
ningún miembro. Requerido".

## Lo que dice nuestro motor

Con los 34 modelos de estas sondas de los que el original dejó el listado,
nuestro listado coincide con el suyo, sentencia por sentencia
(`scripts/wine-oracle/compare-gpss.sh`): **34 de 34**.

Lo que **no** coincide, y queda anotado en `odd/tasks/notas-gpss-forma.md`
("Lo que queda fuera"):

- el `SELECTS` con la salida de fallo conectada: el original escribe el
  destino como último operando (`SE param,1,10,,,A_Terminate1`) y el nuestro
  no;
- los operadores `SV` y `SNV` de la lista `op_Logic_Stor`: el original los
  escribe y el catálogo del editor no los ofrece.

Las dos cosas están arregladas en `odd/tasks/notas-destinos.md` (rama
`feature/destinos-y-operadores`): el listado escribe el destino de la salida
de fallo de las cuatro clases de la familia y los seis operadores de
`op_Logic_Stor` están en el catálogo del editor.
