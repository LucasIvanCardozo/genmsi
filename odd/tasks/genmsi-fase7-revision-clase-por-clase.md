# Fase 7 — Revisión de las clases una por una contra el original

## Specs

### S1 — Cada clase queda verificada con la respuesta del original

Ninguna clase se da por buena porque no ha saltado nada. Para cada clase del
catálogo hay una respuesta del original: el GPSS que compila y el reporte de su
corrida. Esa respuesta se compara con la nuestra, sentencia por sentencia
(verbo, operandos, nombre) y cifra por cifra (con las tolerancias del oráculo:
20% en conteos, 35% en tiempos), y la clase queda en uno de tres estados:
**verificada**, **con diferencias** o **pendiente** (con el motivo).

### S2 — Las diferencias se corrigen una por una, con la evidencia delante

Una diferencia no se corrige porque parezca rara: se corrige porque la respuesta
del original dice qué hace la clase. Cuando la evidencia no alcanza, la clase
queda pendiente y se dice por qué; no se decide de oídas.

### S3 — No hace falta decompilar

El `macrocode` de cada clase en `gClasses.xml` es el código fuente del
original, y su `ClassDescrip` dice, con sus "Consideraciones", qué hace la
clase. Lo que falta no es información, es método: pasar clase por clase
contrastando la respuesta del original con la nuestra.

### S4 — El banco de cada clase es el modelo de la prueba

El banco de una clase la ejercita y nada más (un `Generate`, la clase, un
`Terminate` y un `Clock`). Las clases que no pueden ir solas llevan en su
banco el bloque del que dependen, y el banco lo dice.

### S5 — La cobertura es del catálogo completo

Las 66 clases del original: cada una con su ficha. Un banco que el original se
niega a correr (muestra un diálogo y no abre su ventana) es un hallazgo
sobre el original, no un fallo nuestro: se anota.

## Tasks

| ID | Specs | Qué | Ruta | Commit |
|----|-------|-----|------|--------|
| T1 | S1, S3, S4 | El inventario: la ficha de cada clase del primer lote, con su veredicto | solo lectura | — |
| T2 | S2 | Los arreglos de las clases con diferencias, en tandas | core | — |
| T3 | S1 | Las clases que faltan: `MFunction`, `MFunctionDist`, `RemoveNumber` | core | — |
| T4 | S2, S4 | La fila `GENERATE` del reporte, que cuenta una transacción menos que el original | core | — |
| T5 | S2 | La cola de la `StorageUse`, que no cuenta unidades | core | — |
| T6 | S1 | El segundo lote de clases, con el mismo método | solo lectura | — |

## Log

### L1 — Por qué esta fase

Un lector reportó que tres copias de un generador sacaban sus transacciones
al mismo tiempo, "como si la semilla se copiara entre ellas". No era la
semilla: era que a un `Generate` nuevo le faltaban los defaults del original
(media 60, dispersión 30), y un generador sin dispersión es un reloj, no una
fuente de azar. Corregido (`f905f00`), y con él salieron dos cosas:

- **La familia del bug**: si falta un default importante de una clase, todos
  los bloques de esa clase nacen degenerados. La auditoría de los defaults de
  las 66 clases ya está hecha (`85788ca`), pero el método que la encontró
  —pasar clase por clase— es el que falta sistematizar.
- **La primera transacción**: con dispersión, el original sortea el primer
  intervalo y nosotros no. Está anotado como desviación conocida, no
  resuelta, porque ningún conteo de entradas la separa de la otra lectura.

### L2 — Lo que ya está resuelto (no rehacerlo)

- Las 66 clases están en el catálogo, con sus puertos, sus propiedades y sus
  defaults del original.
- El motor escribe el GPSS igual que el original (`gpss.rs`), y `genmsi gpss`
  lo imprime.
- La semilla es aleatoria por defecto y cada corrida dice cuál usó.
- El `ALTER` ya compara antes de alterar; el `Generate` ya nace con su
  dispersión; el almacén sin declarar ya se para con un error claro, como el
  original; el lint de los tres clientes ya coincide.
- 62 de 66 bancos coinciden en el GPSS con el original y 50 de 55 en las
  cifras.

### L3 — Cómo se hace (el método, en una frase por clase)

1. Se genera el banco: `python3 scripts/wine-oracle/make-benches.py`.
2. Se le pregunta al original: `bash scripts/wine-oracle/run-oracle.sh
   scripts/wine-oracle/benches/<Clase>.gmdl <destino>` (el harness corre el
   original bajo Wine y saca su GPSS y su reporte).
3. Se le pregunta a nuestro motor: `cargo run --release -p genmsi-cli -- gpss
   <banco>` y `... simulate <banco>`.
4. Se comparan: `scripts/wine-oracle/compare-gpss.sh` para el texto, y
   `cargo run --release -p genmsi-core --example compare-benches --` para las
   cifras.
5. Se lee lo que el original dice de la clase (su `macrocode` y su
   `ClassDescrip` en `gClasses.xml`) y se anota la diferencia con su causa.
6. Se corrige, con un test que fije la regla, y se repite el punto 3.

### L4 — El formato de la ficha de una clase

```
Clase: <Nombre>
Puertos: <entradas y salidas, con su ioid>
Propiedades y defaults: <los del catálogo, con su valor>
Semántica: <qué hace, con qué regla, y de dónde sale esa regla>
Cifras: <las del original y las nuestras, con la diferencia>
Veredicto: verificada | con diferencias | pendiente
Si hay diferencias: <el archivo, la función y la prueba que lo fijaría>
```
