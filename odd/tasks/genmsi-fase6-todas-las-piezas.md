# Fase 6 — todas las piezas del original: catálogo, bancos y semántica calcada

El proyecto tiene la semántica de GPSS reimplementada a ciegas: los 24 modelos de
referencia coinciden con el original en los números, pero nadie sabe si las clases
que esos modelos no tocan se comportan igual. Esta fase cierra el círculo: el
original publica su propia especificación (el código GPSS que compila, el
`gClasses.xml` con sus clases y sus defaults, y las formas de su editor), y
esas tres fuentes son las que hay que usar.

## Specs

### S1 — El motor emite el GPSS de un modelo

El motor sabe compilar un modelo a un programa; ese programa se escribe ahora con
la misma forma que usa el original en su pestaña "Código GPSS": una sentencia por
línea, con el nombre del bloque, el verbo y los operandos, y las sentencias que
continúan un bloque sin nombre propio.

El texto es el que el original produce para el mismo modelo, así que ambos se
pueden comparar sentencia por sentencia.

### S2 — El catálogo declara todas las clases del original

El original declara **66** clases en su `gClasses.xml`. Nuestro catálogo declara
**51**. Las quince que faltan —`SelectSNA`, `SelectS`, `SelectL`, `RemoveNumber`,
`RemoveMe`, `Mark`, `GateM`, `Examine`, `MFunctionDist`, `VariableN`, `Preempt`,
`Return`, `MultifurcacionP`, `Multifurcacion` y `MFunction`— se catalogan con sus
puertos, sus propiedades y sus defaults, tomados del original.

`Preempt` y `Return` las sabe simular el motor desde hace tiempo; lo que falta
es ofrecerlas en el editor. Esa desalineación se cierra aquí.

### S3 — Cada clase tiene un banco

Cada clase del original tiene un **banco**: un modelo pequeño que la ejercita
(`Generate` → la clase → `Terminate`, con sus puertos reales) que se puede correr
en el original. El banco produce dos cosas: el **código GPSS** del original, que
dice cómo compila esa clase, y su **reporte**, que dice qué hace.

El banco es la prueba: sin él, la semántica de una clase no está verificada por
nada, aunque el código se parezca al del original.

### S4 — Las clases que faltan se comportan como las del original

Cada clase nueva se implementa contra la especificación que da su banco: el
GPSS del original para la forma, y los números de su reporte para el
comportamiento. Donde el banco y nuestra implementación no coincidan, se corrige
la implementación.

### S5 — La comparación es sistemática

Los bancos de las 66 clases se comparan contra el original: el GPSS con
`compare-gpss.sh`, y los números con el oráculo que ya existe. La verificación
deja de depender de que los 24 modelos de referencia casualmente coincidan.

### S6 — Los contratos publicados no se tocan

Los esquemas `genmsi.*/1` que ya se publican siguen igual. El comando nuevo que
expone el GPSS sigue la convención de los demás, con su esquema propio si lo
necesita, y no cambia ninguno de los existentes.

## Tasks

| ID | Specs | Qué | Ruta | Commit |
|----|-------|-----|------|--------|
| T1 | S1, S6 | El motor emite el GPSS de un modelo y la CLI lo expone | inline | — |
| T2 | S2 | El catálogo declara las 15 clases que faltan, con sus puertos y sus defaults | inline | — |
| T3 | S3 | El generador de bancos: un modelo por clase | inline | — |
| T4 | S4 | La semántica de las 15 clases nuevas, contra su banco | inline | — |
| T5 | S5 | Los 66 bancos comparados contra el original (GPSS y números) | inline | — |
| T6 | S6 | La documentación de lo que la fase deja (docs/format-and-engine.md) | inline | — |

## Log

### L1 — "perfecto! te parece avanzar para la creacion de todas las piezas?"

### L2 — "Perfecto con ese B8 vamos a poder generar absolutamente todas las piezas/bloques del genmsi original? calcando su funcionamiento?"

Decisión del padre, con las cifras medidas: B8 da el juicio, no la cobertura. El
original declara 66 clases; el catálogo 51; los 24 modelos de referencia
ejercitan 53. De ahí las tareas T2 a T5.

### L3 — El comparador del GPSS ya está (commit `238c0cb`)

`scripts/wine-oracle/compare-gpss.sh` compara dos archivos de GPSS sentencia por
sentencia e ignora el relleno entre columnas. Verificado contra el GPSS del
original del TP2: 60 sentencias iguales, y detecta un verbo o un operando
distinto. Falta el lado del motor (T1).

### L4 — El motor ya emite su GPSS, y el oráculo ya habla

`genmsi gpss <model>` imprime el programa tal como lo escribe el original
(`genmsi.gpss/1`). Las sentencias coinciden casi una a una con las del TP2 del
original, y las diferencias que quedan son hallazgos, no ruido:

1. El original escribe la probabilidad de un `TRANSFER` en milésimas (850 = 85 %).
2. El orden de los destinos de un `TRANSFER` no es el de nuestros puertos.
3. El original escribe las unidades del `ENTER` y del `LEAVE` cuando el bloque
   las tiene puestas, y las omite en los bloques compuestos.
4. Al `PREEMPT` y al `RETURN` les faltan operandos: el facility, la prioridad y
   el parámetro del remanente.
5. El `TERMINATE` del modelo no lleva operandos en el original; el del Clock sí
   lleva `1`.
6. El Clock sí aparece en el GPSS del original, al final.
7. Nuestro modelo tiene un bloque `Depart` propio y el original no: allí el
   dequeue va dentro del bloque `Queue`.
8. El original respeta el orden del modelo para las definiciones; el nuestro es
   alfabético, porque las definiciones viven en un `HashMap`.

Falta alinear el comparador por etiqueta (ahora compara línea a línea y las
diferencias salen desalineadas) y cerrar los operandos que faltan.
