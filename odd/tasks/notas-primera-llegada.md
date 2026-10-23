# Primera llegada de un generador: ¿t=0 o la media?

Rama `feature/primera-llegada`, worktree `../genmsi-primera`, desde el HEAD
`3ca6fb9`. **Esta sesión no tocó el core**: lo que hay aquí es evidencia,
veredicto y propuesta. Los dos scripts que producen la tabla están en
`scripts/wine-oracle/` (se movieron ahí cuando el trabajo llegó a `main`:
leen las bases del original, que es donde viven las herramientas que leen el
original).

## La pregunta (verbatim)

> TAREA: resolver, con la evidencia del original, si la primera transacción de un
> generador nace en t=0 o en su media.
>
> - Nuestro motor la pone EN LA MEDIA (t=60 para un Generate con Media=60). En
>   GPSS lo normal es t=0, y la media es el intervalo entre llegadas.
> - La documentación (docs/format-and-engine.md, sección 5) dice que la regla
>   ("la primera llega en C si viene, si no en A") está verificada con TP10
>   ("GENERATE ,,,1") y TP6 ("GENERATE ,,3600,1"), pero ESOS DOS EJEMPLOS NO LA
>   DISTINGUEN: con la media vacía la primera cae en 0 con las dos lecturas. La
>   regla está sin comprobar.
> - Consecuencia: con varias copias de un generador, sus primeras transacciones
>   salen todas a la vez, y el usuario lo reporta como "resultados iguales".

## Veredicto, en una línea

**La primera transacción de un generador nace en la MEDIA (o en el offset `C`
cuando el modelo lo trae), nunca en t=0. Nuestro motor acierta; lo que está mal
es la afirmación de la documentación, que dice estar verificada con dos
ejemplos que no discriminan nada.**

## Cómo se lee una fila GENERATE del reporte del original

Para comparar conteos hay que saber qué cuenta el original. No cuenta sólo las
llegadas que ya ocurrieron: **cuenta las llegadas que ya ocurrieron más la
transacción que está pendiente en la FEC**. Es decir:

```
entradas(original) = llegadas_ocurridas + 1 (la que espera en la FEC)
entradas(nuestras) = llegadas_ocurridas
```

Esa diferencia de uno está probada de forma determinista, no inferida:

- **TP7_1 y TP7_2**, fila `GENERATE 60,,600`: el original marca **52** entradas
  y el bloque siguiente (`A_Buffer1`) marca **51**. El generador creó 52
  transacciones: 51 ya entraron al buffer y una sigue pendiente. Si el original
  contara sólo las llegadas, marcaría 51, no 52.
- **TP6_1**, fila `GENERATE 90,30,2700`: el original marca **14** entradas y la
  cola `qAfuera` marca **13**. Mismo caso: una transacción va en camino.
- **TP4**, fila `GENERATE FN$MediaClientes,FN$ModificadorClientes`: 259 entradas
  en el generador, 258 en la tabla `TGondolas` y 258 en la cola
  `QPersonasSuper`.

Y el otro extremo de la regla —**un evento que cae exactamente en el fin de la
corrida sí se ejecuta**— también está probado, y sin salir del propio modelo:

- **TP7_1**, la FEC muestra `56 0 3660 4 4`: el generador de la fila 4
  (`GENERATE 60,,600`) tiene su próxima llegada en 3660. Esa fila 3660 sólo
  existe si el tick de las 3600 de ese mismo generador se ejecutó y ya plansó
  la siguiente. O sea: **dentro de TP7_1, un tick de generador que cae
  exactamente en el `END_TIME` se ejecuta**.
- **TP6_1**, la CEC termina en `2 0 3600 12 13`: el último evento de la corrida
  es el del `GENERATE ,,3600,1` del propio modelo, que ocurrió exactamente en
  el `END_TIME` (3600). Lo mismo, en otro modelo.

Esto importa porque el `END_TIME` de estos modelos lo produce el `Clock`
(`GENERATE 3600` + `TERMINATE 1`): si el tick del modelo no llegara a
ejecutarse, la fila `GENERATE 3600` del original tendría 1 llegada y no 2, y
las dos lecturas darían el mismo número. Con la prueba de arriba, la fila
`GENERATE 3600` de TP7_1 **sí** tiene su llegada de las 3600, y por lo tanto
**no** puede tener además la de t=0.

Con las dos reglas fijas, la comparación es exacta y no hay que interpretar
nada.

## La evidencia, modelo por modelo

Columna «Llegadas (orig − 1)»: las llegadas que el original realmente tuvo,
según su modelo de conteo. Columna «Predice media»: cuántas llegadas habría
si la primera transacción naciera en la media (o en `C` si el modelo lo trae).
Columna «Predice cero»: cuántas si naciera en t=0.

| Modelo | Fila | Operandos | Horizonte | Entradas original | Entradas nuestras | Llegadas (orig − 1) | Predice «media» | Predice «cero» | Veredicto |
|---|---:|---|---:|---:|---:|---:|---:|---:|---|
| Parcial 6-11-23__Veron - Parcial | 18 | `720,FN$Expon,,20` | 36000 | 20 | 20 | 19 | — | — | no discrimina (limitado) |
| Parcial 6-11-23__Veron - Parcial | 108 | `,,,1` | 36000 | 1 | 1 | 0 | — | — | no discrimina (limitado) |
| Parcial 6-11-23__Veron - Parcial | 126 | `,,,1` | 36000 | 1 | 1 | 0 | — | — | no discrimina (limitado) |
| Problema 1__semaforo-1 | 56 | `8,,0,50` | 3600 | 50 | 50 | 49 | — | — | no discrimina (limitado) |
| Problema 1__semaforo-1 | 65 | `3600` | 3600 | 2 | 1 | 1 | 1 | 2 | **MEDIA** |
| Problema 1__semaforo-1 | 67 | `10,FN$Expon,3600` | 3600 | 2 | 1 | 1 | — | — | no discrimina (dispersión) |
| Problema 1__semaforo-1 | 95 | `180,60,3600` | 3600 | 2 | 1 | 1 | — | — | no discrimina (dispersión) |
| TP10__tp10 | 21 | `90,30,,100` | 28800 | 100 | 100 | 99 | — | — | no discrimina (limitado) |
| TP10__tp10 | 52 | `,,,1` | 28800 | 1 | 1 | 0 | — | — | no discrimina (limitado) |
| TP10__tp10 | 57 | `180,60,3600` | 28800 | 143 | 138 | 142 | — | — | no discrimina (dispersión) |
| TP10__tp10 | 83 | `,,,1` | 28800 | 1 | 1 | 0 | — | — | no discrimina (limitado) |
| TP2__problema1 | 6 | `60,30` | 3600 | 60 | 59 | 59 | — | — | no discrimina (dispersión) |
| TP2__problema1 | 52 | `1800,900` | 3600 | 2 | 1 | 1 | — | — | no discrimina (dispersión) |
| TP2__problema1 | 59 | `3600` | 3600 | 2 | 1 | 1 | 1 | 2 | **MEDIA** |
| TP3__aeropuerto | 7 | `10,5,600,30` | 3600 | 30 | 30 | 29 | — | — | no discrimina (limitado) |
| TP3__aeropuerto | 78 | `1800,,2700` | 3600 | 2 | 1 | 1 | 1 | 1 | no discrimina |
| TP3__aeropuerto | 94 | `7,3,600,60` | 3600 | 60 | 60 | 59 | — | — | no discrimina (limitado) |
| TP3__aeropuerto | 100 | `5,2,600,180` | 3600 | 180 | 180 | 179 | — | — | no discrimina (limitado) |
| TP3__aeropuerto | 106 | `3600` | 3600 | 2 | 1 | 1 | 1 | 2 | **MEDIA** |
| TP3__aeropuerto | 108 | `,,,1` | 3600 | 1 | 1 | 0 | — | — | no discrimina (limitado) |
| TP4__supermercado | 29 | `3600` | 3600 | 2 | 1 | 1 | 1 | 2 | **MEDIA** |
| TP4__supermercado | 31 | `FN$MediaClientes,FN$ModificadorClientes` | 3600 | 259 | 246 | 258 | — | — | no discrimina (dispersión) |
| TP4__supermercado | 77 | `2700,900` | 3600 | 2 | 1 | 1 | — | — | no discrimina (dispersión) |
| TP4__supermercado | 87 | `420,240` | 3600 | 9 | 9 | 8 | — | — | no discrimina (dispersión) |
| TP4__supermercado | 94 | `900` | 3600 | 5 | 4 | 4 | 4 | 5 | **MEDIA** |
| TP5__burgerShop | 13 | `90,FN$Expon` | 3600 | 46 | 41 | 45 | — | — | no discrimina (dispersión) |
| TP5__burgerShop | 56 | `3600` | 3600 | 2 | 1 | 1 | 1 | 2 | **MEDIA** |
| TP6__Clase 6_1_PuertaBanco | 1 | `90,30,2700` | 3600 | 14 | 9 | 13 | — | — | no discrimina (dispersión) |
| TP6__Clase 6_1_PuertaBanco | 12 | `,,3600,1` | 3600 | 1 | 1 | 0 | — | — | no discrimina (limitado) |
| TP6__Clase 6_1_PuertaBanco | 17 | `3600` | 3600 | 2 | 1 | 1 | 1 | 2 | **MEDIA** |
| TP6__Clase 6_2_UltimoCliente | 2 | `90,30,2700` | 6184 | 40 | 36 | 39 | — | — | no discrimina (dispersión) |
| TP6__Clase 6_2_UltimoCliente | 14 | `,,3600,1` | 6184 | 1 | 1 | 0 | — | — | no discrimina (limitado) |
| TP6__Clase 6_3_JefeEducado | 1 | `3600` | 3600 | 2 | 1 | 1 | 1 | 2 | **MEDIA** |
| TP6__Clase 6_3_JefeEducado | 3 | `900,300,,,10` | 3600 | 4 | 4 | 3 | — | — | no discrimina (dispersión) |
| TP6__Clase 6_3_JefeEducado | 15 | `120,60` | 3600 | 29 | 28 | 28 | — | — | no discrimina (dispersión) |
| TP7__Clase 7_1_Link_Unlink | 1 | `60,,,3` | 3600 | 3 | 3 | 2 | — | — | no discrimina (limitado) |
| TP7__Clase 7_1_Link_Unlink | 4 | `60,,600` | 3600 | 52 | 51 | 51 | 51 | 51 | no discrimina |
| TP7__Clase 7_1_Link_Unlink | 13 | `3600` | 3600 | 2 | 1 | 1 | 1 | 2 | **MEDIA** |
| TP7__Clase 7_2_ProductorConsumidor | 1 | `60,,,25` | 3600 | 25 | 25 | 24 | — | — | no discrimina (limitado) |
| TP7__Clase 7_2_ProductorConsumidor | 4 | `60,,600` | 3600 | 52 | 51 | 51 | 51 | 51 | no discrimina |
| TP7__Clase 7_2_ProductorConsumidor | 16 | `3600` | 3600 | 2 | 1 | 1 | 1 | 2 | **MEDIA** |
| TP7__Clase 7_3_Cadete | 2 | `,,600,1` | 3600 | 1 | 1 | 0 | — | — | no discrimina (limitado) |
| TP7__Clase 7_3_Cadete | 16 | `300,60,,9` | 3600 | 9 | 9 | 8 | — | — | no discrimina (limitado) |
| TP7__Clase 7_3_Cadete | 19 | `3600` | 3600 | 2 | 1 | 1 | 1 | 2 | **MEDIA** |
| TP8__mecanicos | 18 | `40,15` | 3600 | 91 | 93 | 90 | — | — | no discrimina (dispersión) |
| TP8__mecanicos | 44 | `,,,10` | 3600 | 10 | 10 | 9 | — | — | no discrimina (limitado) |
| TP8__mecanicos | 103 | `300,90` | 3600 | 13 | 11 | 12 | — | — | no discrimina (dispersión) |
| TP8__mecanicos | 107 | `3600` | 3600 | 2 | 1 | 1 | 1 | 2 | **MEDIA** |
| ejercicio_1__ejercicio_1 | 1 | `60,30` | 3600 | 60 | 63 | 59 | — | — | no discrimina (dispersión) |
| ejercicio_1__ejercicio_1 | 10 | `3600` | 3600 | 2 | 1 | 1 | 1 | 2 | **MEDIA** |
| ejercicio_2__ejercicio_2 | 1 | `60,30` | 3600 | 64 | 61 | 63 | — | — | no discrimina (dispersión) |
| ejercicio_2__ejercicio_2 | 17 | `3600` | 3600 | 2 | 1 | 1 | 1 | 2 | **MEDIA** |
| ejercicio_3__ejercicio_3 | 2 | `120,90,,,0` | 1800 | 17 | 14 | 16 | — | — | no discrimina (dispersión) |
| ejercicio_3__ejercicio_3 | 10 | `90,50,,,1` | 1800 | 21 | 20 | 20 | — | — | no discrimina (dispersión) |
| ejercicio_3__ejercicio_3 | 21 | `0,0,7200,500,1` | 1800 | 1 | 0 | 0 | — | — | no discrimina (limitado) |
| ejercicio_3__ejercicio_3 | 25 | `1800` | 1800 | 2 | 1 | 1 | 1 | 2 | **MEDIA** |
| ejercicio_4__ejercicio_4 | 2 | `60,10` | 3600 | 60 | 59 | 59 | — | — | no discrimina (dispersión) |
| ejercicio_4__ejercicio_4 | 29 | `3600` | 3600 | 2 | 1 | 1 | 1 | 2 | **MEDIA** |
| entregaComida__entregaComida | 2 | `3600` | 3600 | 2 | 1 | 1 | 1 | 2 | **MEDIA** |
| entregaComida__entregaComida | 4 | `120,60` | 3600 | 32 | 31 | 31 | — | — | no discrimina (dispersión) |
| entregaComida__entregaComida | 15 | `,,0,1` | 3600 | 1 | 1 | 0 | — | — | no discrimina (limitado) |
| monorriel__monorriel | 10 | `,,,1` | 3600 | 1 | 1 | 0 | — | — | no discrimina (limitado) |
| monorriel__monorriel | 34 | `3600` | 3600 | 2 | 1 | 1 | 1 | 2 | **MEDIA** |
| monorriel__monorriel | 36 | `5,3` | 3600 | 743 | 719 | 742 | — | — | no discrimina (dispersión) |

Las 18 filas marcadas **MEDIA** son 18 modelos independientes, y en las 18 el
número del original es **exactamente el que predice la lectura «media»** y
**exactamente uno más** que el que predice la lectura «cero».

### Por qué esas 18 y no otras

Las que discriminan son las que cumplen las tres condiciones:

1. **Sin dispersión** (el modificador está vacío o es 0), para que el intervalo
   sea determinista y el conteo sea exacto y no una distribución.
2. **Sin límite** (`Cantidad` vacía), porque un generador limitado se detiene al
   llegar al límite y las dos lecturas dan el mismo número.
3. **Con una media que no deja el horizonte vacío**, para que la diferencia entre
   las dos lecturas sea de exactamente una llegada y no de muchas.

`GENERATE 3600` sobre un horizonte de 3600 es el caso más limpio:

| Lectura | Llegadas | Entradas del original |
|---|---:|---:|
| primera en t=0 | 0 y 3600 → **2** | 2 + 1 pendiente = **3** |
| primera en la media | 3600 → **1** | 1 + 1 pendiente = **2** ← lo que dice el original |

Y `GENERATE 900` sobre 3600 (TP4): con la primera en 0 serían 5 llegadas
(0, 900, 1800, 2700, 3600) y el original marcaría 6; con la primera en la
media son 4 (900, 1800, 2700, 3600) y el original marca **5**. Otra vez media.

Ninguna de las dos lecturas se salva con «igual la corrida terminó antes»:
en TP7_1 la FEC demuestra que el tick de las 3600 sí corrió (la llegada
pendiente de las 3660), así que la llegada de las 3600 existe y cuenta.

### Las que no discriminan (y por qué)

- **Media vacía** (`GENERATE ,,,1`): el intervalo es 0 y la primera transacción
  cae en t=0 con las dos lecturas. Es el caso de TP10 y TP6 con el que la
  documentación "verifica" la regla: no verifica nada.
- **Con offset `C`** (`GENERATE 60,,600`): el offset manda en las dos lecturas,
  así que tampoco distinguen. Lo que sí confirman es que **el offset se respeta**:
  51 llegadas desde las 600 hasta las 3600, de 60 en 60, más la pendiente en
  3660, dan las 52 entradas del original.
- **Con dispersión**: el conteo es estocástico y no se puede leer como un
  número exacto (ver más abajo por qué tampoco sirve estadísticamente).
- **Con límite**: el generador se detiene al llegar al límite.

### Salvedad honesta sobre las bases

Las bases del original son **una sola corrida** de un modelo estocástico (lo
dice el propio `docs/oracle-comparison.md`). Un caso se sale del patrón: **TP6_1**,
`GENERATE 90,30,2700` sobre 3600: el original registra 13 llegadas entre las 2700
y las 3600, o sea 12 intervalos de `U(60,120)` dentro de 900 unidades de
tiempo, un promedio de 73 contra una media de 90. Con una simulación de 200.000
muestras, que eso salga así tiene una probabilidad de 0,0009. La misma
generadora en TP6_2 (horizonte 6184) sí encaja: 39 llegadas en 3484 unidades,
un promedio de 89,3. Es una muestra rara del baseline, no una regla distinta;
y no toca el veredicto, que se decide en las 18 filas deterministas.

## El veredicto, con la evidencia que lo sostiene

**La primera transacción de un generador nace en la media (o en el offset `C`),
no en t=0.** Nuestro motor está bien en esto. La documentación dice la verdad
sobre la regla y miente sobre la verificación.

1. La regla del original está escrita en su propio manual
   (`Genmsi_20230711/VISUALSIS.HLP`, sección GENERATE, "Acción"):

   > "Cuando una simulación es comenzada, cualquier bloque GENERATE que no ha
   > sido 'primed' es llamado para planificar su primer transacción. [...] El
   > operando C puede ser usado para especificar un incremento de tiempo
   > positivo para la primera transacción. En otro caso, el primer tiempo de
   > incremento es calculado desde los operandos A y B."

   Es decir: el incremento de la primera transacción es `C` si el modelo lo
   trae, y si no, se calcula con la misma fórmula que cualquier otro
   intervalo (`A` solo, o `A±B` si hay dispersión). Nunca 0 por definición.

2. Las 18 filas deterministas de la tabla lo confirman contra los reportes
   reales del original, con el conteo exacto y con dos reglas probadas
   aparte (la del pendiente y la del evento en el fin de la corrida).

3. La lectura «cero» queda refutada en las 18: en todas, el número del
   original es uno menos de lo que predice «cero».

**Lo que hay que corregir es la documentación**, no el motor: `docs/format-and-engine.md`
sección 5 dice «Verified against TP10's `GENERATE ,,,1` (one transaction) and
TP6's `GENERATE ,,3600,1` (one at 3600)». Ninguno de los dos ejemplos
discrimina: con la media vacía la primera cae en 0 con las dos lecturas, y con
el offset `3600` el offset manda en las dos lecturas. La afirmación de
verificación es falsa tal como está.

Texto propuesto para la sección 5 (lo escribe la sesión del core; aquí no se
toca `docs/`):

> **GENERATE A,B,C,D,E**: la primera transacción llega con el incremento `C`
> cuando el modelo lo trae, y si no con el incremento que sale de `A` y `B`
> (es decir, `A` solo, o el sorteo de `A±B` si hay dispersión); nunca en t=0
> por definición. `D` limita el total; una `A` vacía es un intervalo 0 (los
> modelos la usan con `D` para crear una transacción en t=0).
> Verificado contra los 18 modelos con un GENERATE sin dispersión, sin offset
> y sin límite: `GENERATE 3600` sobre un horizonte de 3600 da 2 entradas (la
> llegada de las 3600 más la que queda pendiente en la FEC), `GENERATE 900`
> sobre 3600 da 5, y `GENERATE 1800` sobre 1800 da 2. Con la primera en t=0
> esos mismos modelos darían 3, 6 y 3.

Y una prueba que lo fijaría (para la sesión del core), sobre la interfaz
pública del motor:

- `GENERATE 3600` con horizonte 3600 → el bloque reporta 2 entradas y el
  bloque siguiente 1.
- `GENERATE 900` con horizonte 3600 → 5 entradas, 4 en el bloque siguiente.
- `GENERATE 60,,600` con horizonte 3600 → 52 entradas, 51 en el bloque
  siguiente.

## Hallazgos secundarios (para la sesión del core; aquí no se toca nada)

### 1. Nuestra fila GENERATE del reporte está un número abajo del original

En los 18 casos deterministas, **nuestro `ENTRY COUNT` es el del original
menos uno** (1 contra 2, 4 contra 5, 51 contra 52). No es un error de
simulación: es que el original cuenta en la fila del generador la transacción
que ya va en camino (la que todavía no entró al bloque siguiente) y nosotros
sólo contamos las que ya cayeron. La simulación es la misma; el número que se
imprime no.

- Fichero: `crates/genmsi-core/src/sim.rs`, función `tick` (línea 501:
  `self.block_stats[block_index].entries += 1;`).
- Cambio posible: contar la entrada cuando la transacción se crea (al planear
  la siguiente) y no cuando dispara el tick, para que la fila coincida con la
  del original. Conviene decidirlo junto con la columna `CURRENT` (el original
  pone 1 en casi todos los generadores con una llegada pendiente; nosotros
  también, salvo en TP6_3, donde el original pone 0 con una llegada pendiente
  en 3896 y nosotros ponemos 1).
- Prueba que lo fijaría: `GENERATE 3600` sobre 3600 → la fila del bloque tiene
  que decir 2 entradas.

### 2. Con dispersión, el primer intervalo del original se sortea y el nuestro no

Esto **no** es lo que preguntaste, pero es lo más cerca del síntoma «resultados
iguales» que se ha reportado, así que va aquí.

El manual del original dice que el primer incremento «es calculado desde los
operandos A y B». Con `A` solo, «calculado desde A y B» es `A` exacto (nuestro
comportamiento ✓). **Con dispersión, el primer intervalo del original también
se sortea** (`A±B`, o `A·f(u)` si el modificador es una función). El nuestro
no: `crates/genmsi-core/src/sim.rs`, `Simulation::init` (líneas 324-354), usa
`spec.mean.eval(&mut ctx)` para la primera llegada, y el sorteo sólo empieza en
el segundo intervalo.

La consecuencia es exactamente la que se reporta: **varias copias de un
generador con dispersión sacan su primera transacción todas en el mismo
instante** (`t = A` exacto para todas), mientras que en el original cada copia
sortea su primer intervalo del flujo de números aleatorios (el manual dice que
el bloque GENERATE usa el generador número 1) y salen escalonadas. Con un
generador **sin** dispersión el síntoma es inherente al original también (su
primer incremento es `A` exacto), así que si el modelo del usuario tiene
dispersión, ahí está la diferencia.

Lo que la evidencia **no** puede decidir: los conteos de entrada no distinguen
las dos lecturas cuando la dispersión es simétrica, porque el número esperado
de llegadas es el mismo. Medido por simulación (200.000 muestras por caso):

| Generador | A | B | Horizonte | Esperado con la primera en A | Esperado con la primera sorteada |
|---|---:|---:|---:|---:|---:|
| TP8 `300,90` | 300 | 90 | 3600 | 11,52 | 11,51 |
| TP8 `40,15` | 40 | 15 | 3600 | 89,52 | 89,52 |
| TP6_3 `120,60` | 120 | 60 | 3600 | 29,53 | 29,54 |
| ejercicio_4 `60,10` | 60 | 10 | 3600 | 59,51 | 59,50 |
| ejercicio_3 `120,90` | 120 | 90 | 1800 | 14,60 | 14,59 |

La diferencia esperada es de 0,01 llegadas: ningún conteo de entradas puede
separar las dos lecturas. Sólo el manual del original las separa, y por eso
esto queda como **propuesta a decidir**, no como veredicto.

Dibujo del cambio, si la sesión del core lo decide:

- Fichero: `crates/genmsi-core/src/sim.rs`.
- Función: `Simulation::init`, en el bloque que planifica la primera llegada de
  cada generador (líneas 324-354). Hoy:

  ```rust
  match &spec.offset {
      Some(offset) => offset.eval(&mut ctx),
      None => spec.mean.eval(&mut ctx),
  }
  ```

  Con dispersión, la rama `None` debería hacer el mismo sorteo que hace el
  `tick` para los intervalos siguientes, es decir pasar por `evaluate_delay` y
  `apply_modifier` (las dos funciones ya existen, líneas 2104 y 2128), con el
  `stream` del bloque:

  ```rust
  match &spec.offset {
      Some(offset) => offset.eval(&mut ctx),
      None => {
          let (mean, modifier) = evaluate_delay(&spec.mean, spec.modifier.as_ref(), &mut ctx);
          apply_modifier(mean, modifier, &mut self.streams, block.stream)
      }
  }
  ```

  (el bucle actual recorre `program.blocks` y se queda con el `spec`; para el
  `stream` del bloque hay que conservar el `ExecBlock` o su índice).
- Prueba que lo fijaría: un modelo con **dos** generadores de la misma media y
  dispersión, y una traza; los dos primeros `generate` de la traza tienen que
  caer en instantes **distintos**. Y el caso sin dispersión tiene que seguir
  cayendo en `t = A` exacto (para no romper los 18 modelos deterministas de la
  tabla).
- Riesgo: item 3 (cambia el comportamiento observable de un contrato que ya
  consumen los reportes y las comparaciones con el original) y item 6 (el
  cambio de resultados de muchos modelos necesita la prueba nueva que lo
  fije). Que lo decida la sesión del core.

### 3. Menos: nuestro GPSS escribe `GENERATE 0,,3600,1` donde el original escribe `,,3600,1`

El original deja el operando vacío cuando vale 0; nosotros lo escribimos como
`0`. No cambia la simulación (los dos significan lo mismo) pero ensucia la
comparación de GPSS con el original. Está en `crates/genmsi-core/src/report.rs`,
función `operand_parts`, rama `BlockKind::Generate`.

## Cómo se reproduce

Desde la raíz del repo:

```bash
cargo build --release -p genmsi-cli

# la tabla de la evidencia, modelo por modelo, con nuestro conteo al lado
python3 scripts/wine-oracle/compare_generate.py 1

# sólo la evidencia del original (sin correr nuestro motor)
python3 scripts/wine-oracle/first_arrival_evidence.py
```

`scripts/wine-oracle/compare_generate.py` corre nuestro motor con el
horizonte que declara el reporte del original, empareja las filas GENERATE por
sus operandos y aplica el modelo de conteo (entradas = llegadas + 1
pendiente).

## Lo que esta sesión NO tocó

- Ningún fichero del core (`crates/genmsi-core/**`), ni `docs/format-and-engine.md`.
- El worktree principal: no se editó ningún fichero suyo ni se cambió su
  estado de git (sólo se leyó).
- Sólo se añadieron este documento y los dos scripts (que después se movieron
  a `scripts/wine-oracle/`).
