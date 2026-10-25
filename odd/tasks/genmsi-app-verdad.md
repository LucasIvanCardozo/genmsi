# genmsi-app-verdad — la app dice la verdad

El motor ya está verificado contra el original (el oráculo de las 66 clases
coincide 66 de 66). Lo que queda por truthful es lo que la app **cuenta por su
cuenta**: hoy suma las sentencias de un bloque para mostrar cuántas
instancias pasaron por él, y esa suma da 5 por instancia en un `FacilityUse`,
porque un `FacilityUse` son cinco sentencias.

Esta fase hace que la app no invente cifras: el motor publica lo que cuenta
cada bloque y la app lo muestra tal cual. Y hace que la reproducción no
teletransporte: una transacción que cruza bloques sin espera se ve al menos
un instante en cada bloque.

## Specs

### S1 — La app no inventa cifras

> "los datos que se muestran en la app no deberían ser los reales calculados
> por el motor? no causamos duplicaciones de cálculo que conllevan a que
> pasen estas cosas? no sería mejor que haya una zona de verdad y que todo el
> que quiera mostrar o usar esos datos deba consultarlos de la fuente de
> verdad?"

El principio es del proyecto (D2: las capas GUI/CLI/MCP son finas; el reporte
es el motor's `genmsi.report/1`, no un recálculo; la reproducción es una
proyección verificada contra ese reporte con `checkFidelity`). La violación es
una sola: la app **suma** las sentencias de un bloque para mostrar cuántas
instancias pasaron por él, y esa suma es una cifra que el motor nunca publicó
— el chequeo de fidelidad no puede detectarla, porque no hay con qué
compararla.

El arreglo: el motor publica **qué sentencia cuenta cada bloque** (dato
estable, que sale de la compilación) y la app muestra la cifra de esa
sentencia, sin sumar nada.

### S2 — Un generador no muestra una cifra que no significa nada

El `Generate` cuenta el generador en su `current` y no una transacción (no hay
transacciones dentro de un generador): su fila sale `0 1` y no significa
nada. La app no lo muestra: el generador no tiene "instancias que pasaron por
él".

### S3 — La reproducción no teletransporta

> "Cuando una instancia debe pasar entre varios bloques que no tienen tiempos
> de espera (es decir que la instancia pasa de manera instantánea) la
> velocidad de transición se hace muy rápida y queda raro, como podemos
> mejorar esto?"

Una transacción que cruza bloques sin espera no consume tiempo simulado, así
que en la reproducción el salto es de un cuadro a otro. El arreglo es de la
**presentación**, no del modelo: la fase 4c decidió que el replay es una
proyección de los documentos existentes y no recalcula ninguna métrica del
motor (S11), así que el reloj del modelo sigue siendo el reloj del modelo.

- **A**: cada bloque ocupa un tiempo mínimo en pantalla, aunque el modelo diga
  cero.
- **B**: el vuelo de la pelota entre dos bloques dura un mínimo, aunque el
  modelo diga cero.

### S4 — El reordenamiento no se toca aquí

> "el reordenamiento no es del todo bueno realmente debemos cambiar la forma
> en que se hace esto, pero a lo mejor esto es para una sesión enfocada
> completa a esta mejora"

La composición es del motor (D18) y por lo tanto es una sola; cambiarla es
trabajo de otra sesión, con su propia medida.

## Tasks

| T | Specs | Qué | Ruta | Commit |
|---|---|---|---|---|
| T1 | S1 | El motor publica qué sentencia cuenta cada bloque; la app lo muestra sin sumar | core + app | `c169cad` |
| T2 | S2 | Los generadores no muestran cifra | app | `c169cad` |
| T3 | S3 | El tiempo mínimo por bloque y la duración mínima del vuelo | app | `13d4986` |
| T4 | S1, S2, S3, S4 | Los documentos (la decisión de la zona de verdad, la app y el lazo visual) | docs | pendiente |

## Log

### L1 — El pedido del usuario (verbatim)

> bien, tambien quiero decirte que quiero luego encarar un mejora en la viusual
> y logica del motro (porque encontre unos bugs) y mejoras en cuanto a graficas.
> consideras que primero hagamos lo del mcp o lo otro?

> 1. los datos que se muestran en la app no deberian ser los reales calculado
> por el motor? no causamos duplicaciones de calculo que conllevan a que pasen
> estas cosas? no seria mejor que haya una zona de verdad y que todo el que
> quiera mostrar o usar esos datos deba consultarlos de la fuente de verdad?

> 3. tu recomendacion A + B me parece la indicada

> vamos con fase 7/8 como vos recomendas

### L2 — Las fases 7 y 8 ya estaban cerradas

El usuario pidió continuar con las fases 7 y 8. Al reconciliarlas con el
código y con la evidencia:

- El **oráculo de las 66 clases coincide 66 de 66** hoy (`compare-benches.sh`
  contra las respuestas del original del scratch).
- Los trece "cambios pendientes" de `notas-fase7-lote3.md` (las once filas
  del reporte, las cuatro secciones, el `ALTER`, el operador `Invertir`, la
  puerta del `GateL`, la columna `LINE`, el `BDT` entero) **están hechos**:
  `is_one_unit`, `transfer_probability`, el `P$Param`, el `MultifurcacionGO`
  y la puerta que no abre están en el código.
- La **primera llegada con dispersión se sortea** (`sim.rs`, `fn
  first_arrival`: la primera llegada toma el primer número del propio flujo
  del generador).
- El **reloj es entero** (fase 8, commits `27f42fb` y `934a5fb`) y la
  **medición final** está hecha (`odd/tasks/notas-medicion-final.md`).

Lo que quedaba abierto en las tablas de tareas de la 7 y la 8 era
contabilidad vieja: el trabajo lo hicieron las sesiones siguientes.

### L3 — Lo que encontró el usuario

Los cuatro puntos, con su diagnóstico:

1. **El `FacilityUse` cuenta 5 instancias por instancia** (4 al entrar, 1 al
   salir). **No es del motor**: el reporte publica una fila por sentencia
   (como el original) y sus cifras coinciden con las del original. Es la app,
   `MonitorView.tsx`, que suma las sentencias del bloque.
2. **El `Generate` sale `0 1`**: el motor cuenta el generador en su `current`
   porque no hay transacciones dentro de un generador → la cifra no significa
   nada.
3. **Las transiciones instantáneas son demasiado rápidas**: es la
   presentación de la reproducción (S3).
4. **El reordenamiento no convence**: sesión propia (S4).

### L4 — El diagnóstico del punto 1, con la evidencia

La duplicación es real, y es una: la app **inventa** una cifra que el motor no
publica (la suma de las sentencias de un bloque). Todo lo demás ya tiene una
zona de verdad:

- el reporte es el documento del motor, byte a byte (`tests/parity.test.ts`),
- la reproducción es una proyección del motor y hay un chequeo que la compara
  con el reporte al final de la corrida (`checkFidelity`),
- lo que la proyección **no** puede comprobar es una cifra que el motor nunca
  publicó: la suma.

Por eso el arreglo no es "que la app calcule mejor": es que el motor publique
lo que cuenta cada bloque y la app lo muestre.

### L5 — T1 y T2: el motor publica y la app no inventa (`c169cad`)

El reporte del motor marca, en cada fila, si esa fila cuenta su bloque (la
primera de cada bloque: las transacciones que entraron a él). El campo es
aditivo en el JSON del reporte (`counts_block`), como los que la clase ganó
en la fase 3b.

La app deja de sumar: `src/monitor/counts.ts` (una función pura, con sus
pruebas) muestra la cifra de la fila que cuenta el bloque, y un generador no
muestra cifra.

Las pruebas: dos en el motor (`crates/genmsi-core/tests/cuenta_del_bloque.rs`:
la primera fila de un bloque es la que lo cuenta, y cada bloque del modelo
tiene la suya) y tres en la app (`crates/genmsi-app/tests/counts.test.ts`: la
cifra es la de la fila que cuenta, un generador no tiene cifra, y un reporte
que no lo dice deja al bloque sin cifra).

En la pantalla: un `FacilityUse` que sirvió 59 transacciones muestra `0/59`
(antes `0/295`), y el `Generate` no muestra nada.

### L6 — T3: el cruce siempre se ve (`13d4986`)

El vuelo de la pelota tiene un suelo en el reloj del lector (la velocidad de
la reproducción por el vuelo más corto) y el reloj se detiene en el próximo
cruce en vez de saltarlo. El reloj de la corrida sigue siendo el de la
corrida: el suelo está en el tiempo del lector, no en el del modelo.

Las pruebas: dos en las bolas (el vuelo es el suelo; el vuelo es el tiempo
que la transacción pasó en el bloque) y dos en el reloj (no se salta un cruce;
una corrida sin cruces avanza a la velocidad pedida).

### L7 — T4: los documentos

La decisión D19 ("una zona de verdad: el motor publica, la app no inventa"),
la sección del monitor en `docs/app.md` y esta fase.

### L8 — La decisión

El motor publica qué sentencia cuenta cada bloque (el dato sale de la
compilación: un `FacilityUse` son cinco sentencias, y la primera es la que
cuenta las entradas). La app deja de sumar y muestra esa cifra. Los
generadores no muestran cifra (S2).

Es un campo **aditivo** en el documento del modelo (`genmsi.model/1`), la
misma práctica que la fase 3b con el documento de la clase (`declares` y
`uses` sin cambiar la versión).
