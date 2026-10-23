# Sondas: los grupos y las tablas del reporte

Cuatro sondas para medir contra el original (`genmsi.exe` bajo Wine) dos cosas
que los bancos no miden: **qué cuenta el `GROUP_SIZE`** de los grupos y **cómo
escribe el original la sección `TABLAS`**.

Las modelos se generan con `make-reporte-probes.py` (lee la biblioteca de
clases del original, `Genmsi_20230711.backup/gClasses.xml`) y se corren con
`run-probe-reporte.sh` (una copia del arnés `scripts/wine-oracle/run_oracle.sh`
que corre una copia del ejecutable con otro nombre, para que otra sesión de la
máquina no lo mate con su `pkill`).

```
bash scripts/wine-oracle/probes/reporte/run-probe-reporte.sh \
     scripts/wine-oracle/probes/reporte/grupo_numerico.gmdl /tmp/probe-reporte/oficial
```

## Lo que dice el original

### 1. `grupo_numerico.gmdl`: el `GROUP_SIZE` de un grupo de números

Un generador, tres `JoinNumber` que unen el grupo `GNUM` con los números 1, 2
y 3, y un terminador.

```
NUMERIC_GROUP        GROUP_SIZE      RETRY
  GNUM                       3          0
```

Y `grupo_numerico_repetido.gmdl`, el mismo modelo pero con los tres números
iguales (2, 2 y 2):

```
NUMERIC_GROUP        GROUP_SIZE      RETRY
  GNUM                       1          0
```

Las 108 uniones son las mismas en los dos modelos: lo que cambia es el número
de **valores distintos** del grupo. El `GROUP_SIZE` de un grupo de números
cuenta los valores distintos que tiene, no las uniones ni la suma.

### 2. `grupo_transaccion.gmdl`: el `GROUP_SIZE` de un grupo de transacciones

Un generador, un `JoinMe` al grupo `GTRANS`, un `Advance` de 3000 (las
transacciones siguen vivas cuando la corrida termina) y un terminador.

```
XACT_GROUP           GROUP_SIZE      RETRY
  GTRANS                    30          0
```

De 36 transacciones que se unes al grupo, seis llegan al terminador: las 30
que quedan vivas son las que el grupo cuenta.

### 3. `tablas.gmdl`: la sección `TABLAS`

Dos tablas (`TBL1` y `TBL2`, con los mismos límites) y dos flujos: uno asigna
5 y tabula en `TBL1`, el otro asigna 95 y tabula en `TBL2`.

```
TABLAS

TBL2            TABLE           P$Atrib,0,10,10
    MEDIA              : 95,000
    DESVIACION ESTANDAR: 0,000
    CANTIDAD EN RETRY  : 0

    INTERVALO         FRECUENCIA     %     %ACUMULADO     %REMANENTE
    0   -  0                0      0,000       0,000        100,000
    1   -  10               0      0,000       0,000        100,000
    11  -  20               0      0,000       0,000        100,000
    21  -  30               0      0,000       0,000        100,000
    31  -  40               0      0,000       0,000        100,000
    41  -  50               0      0,000       0,000        100,000
    51  -  60               0      0,000       0,000        100,000
    61  -  70               0      0,000       0,000        100,000
    71  -  80               0      0,000       0,000        100,000
    >   a  80              36    100,000     100,000          0,000

    MEDIA OVERFLOW                : 95,000
    DESVIACION ESTANDAR OVERFLOW  : 0,000

    CANTIDAD DE VALORES TABULADOS : 36
    VALOR MAXIMO TABULADO         : 95,000
```

Tres cosas salen de aquí:

- **La última clase es la de los valores que no caben en ninguna otra** (la
  fila `>   a  80`), y tiene sus propias estadísticas (`MEDIA OVERFLOW` y
  `DESVIACION ESTANDAR OVERFLOW`).
- **El original termina la tabla cuando las clases suman el 100%** y escribe
  `Las frecuencias restantes (8) son todas cero.` en vez de las clases vacías
  (es lo que hace la tabla `TBL1` del mismo modelo, que sólo tiene dos clases
  con valores).
- El `CANTIDAD EN RETRY` salió `0` en todas las medidas: cuenta las
  tabulaciones que hubo que **reintentar**, y en estos modelos no hay ninguna.

### 4. Las sondas de los atributos

`tablas_parametro_distinto.gmdl` (la tabla tabula `P:Atrib` y el flujo asigna
`Param`) y `tablas_atributo_inexistente.gmdl` (el atributo nombra un
parámetro que nadie asigna) sirven para medir que **el `Atributo` de la tabla
es el parámetro que la tabla tabula**: con el parámetro que nadie asigna, los
36 valores se tabulan como 0.

`tablas_t1_t2.gmdl` es el modelo con los nombres de una y dos letras (los que
el original rechaza: `El modelo no puede abrirse porque el archivo presenta
errores estructurales`; el nombre de una entidad tiene que tener al menos tres
letras). Se deja por el registro.
