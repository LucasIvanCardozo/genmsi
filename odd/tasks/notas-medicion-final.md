# La medición final del producto

Unidad de cierre de la **fase 8** (spec **S6**). Todas las medidas son de
este repositorio contra el original (GenMSI 1.2020.07.28 BETA1 bajo Wine),
con la línea base del original regenerada en `/tmp/all-oracle` (66 reportes y
66 listados GPSS).

## 1. El recuento de las 66 clases

El oráculo tiene **un banco por clase** del original (66 clases ✓). Con el
motor definitivo:

| medida | resultado |
|---|---|
| bancos con reporte del original | **66** |
| coinciden en todas las métricas | **66** |
| desviación de tiempos (192 métricas comparadas) | **192 coinciden, 0 no coinciden** |
| listing GPSS (byte a byte) | **66 coinciden, 0 difieren** |
| sondas del orden de los simultáneos | **10 de 10** |

Es decir: **las 66 clases están** ✓ (una por banco ✓), **sus tiempos coinciden
con los del original** ✓ (192 de 192 ✓, con desviación 0,0 %) ✓ y **su
compilación GPSS es idéntica** ✓.

## 2. La paridad con el original, modelo por modelo

Sobre los 25 modelos de referencia (`fixtures/models`), 958 métricas:

| corrida | resultado |
|---|---|
| 3 corridas | **902 de 958** (la base, antes del reloj entero, medía 910) |
| 10 corridas | **821 de 958** (la base: 804) |
| 20 corridas | **799 de 958** (la base: 800) |

**El ruido de la medida es mayor que el efecto** ✓: el mismo código medido a
10 y a 20 corridas difiere en 4 y en 22 métricas ✓. La lectura honesta es que
**el reloj entero no mejora de forma medible la paridad** ✓ — lo que cambia es
la **forma** ✓: los tiempos del reporte y de la traza son enteros ✓ como los
del original ✓ → las cifras de los dos son comparables ✓✓.

## 3. Las desviaciones que permanezcan

| modelo | resultado | qué es |
|---|---|---|
| `Parcial 6-11-23` | 128/150 | el modelo más grande (36.000 unidades) |
| `Parcial 8-11-21` | **no carga** | el `QTable` con un espacio en el nombre ("tiempo demora") se rechaza: el original lo acepta |
| `TP3__aeropuerto` | 82/91 | |
| `TP4__supermercado` | 61/65 | |
| `TP6__Clase 6_1_PuertaBanco` | 11/13 | |
| `TP6__Clase 6_2_UltimoCliente` | 17/19 | |
| `TP6__Clase 6_3_JefeEducado` | 15/16 | |
| `TP8__mecanicos` | 114/116 | |
| `TP10__tp10` | 62/64 | |
| `TP2__problema1` | 56/57 | |
| `ejercicio_3` | 18/21 | |
| `ejercicio_4` | 24/29 | |
| `monorriel` | 52/55 | |
| `Problema 2`, `TP6 6_0`, `TP9__hack`, `TP9__tp9` | error esperado | **el original también los rechaza** ✓ con el mismo error ✓ |

Las que no coinciden del todo: **13 modelos** ✓, y uno que no carga ✓. Las
diferencias conocidas y **no bloqueantes** ✓:

- **`MATCH`**: el original cuenta al esperero en el `transfer` y en el
  `match` ✓; un solo evento `backup` sólo puede nombrar un bloque ✓.
- **el modificador de función en la primera llegada** ✓: aplicarlo empeora
  `TP4` y el `Parcial` ✓ → queda deliberadamente sin aplicar ✓.
- **el `QTable` con un espacio en el nombre** ✓: el formato lo rechaza ✓ y el
  original lo acepta ✓ (única diferencia de *forma* que queda ✗).

## 4. La tasa de cuadros de la app (nunca medida antes)

La app dibuja la reproducción con `requestAnimationFrame` ✓: en cada cuadro
busca el frame del instante (búsqueda binaria ✓) y recalcula el estado
(`stateAt` ✓). Medido con el modelo más grande del repo
(`Parcial 6-11-23` ✓), sobre el precomputado y el coste por cuadro:

| eventos de la traza | cuadros | precomputado | por cuadro | techo |
|---:|---:|---:|---:|---:|
| 100 (el límite por defecto) | 100 | 5,7 ms | **0,127 ms** | 7.874 cuadros/s |
| 10.000 | 10.000 | 43,4 ms | **3,569 ms** | 280 cuadros/s |

Un cuadro a 60 Hz tiene 16,7 ms ✓: con la traza por defecto el trabajo por
cuadro es el **0,8 %** de ese presupuesto ✓ y con diez mil cuadros sigue
siendo el 21 % ✓ → **la animación no pierde cuadros** ✓✓. El recorrido es
O(n) en el precomputado y O(log n) por cuadro ✓ (sin la clase de patología que
en el motor costó 58 s ✓).

## 5. La app, instalada y con el motor definitivo

- Reconstruida ✓ (`cargo tauri build` ✓): `.deb` y `.rpm` de 3,90 MiB ✓.
- Instalada ✓ en `~/Applications/GenMSI/` ✓, con el lanzador ✓ y el icono ✓.
- **La ventana abre** ✓ (captura ✓): la barra tiene **«Simular»** ✓,
  **«Ejecutar»** ✓, **«Semilla»** ✓, **«Hasta»** ✓ y el campo nuevo
  **«Inicio»** ✓ (con 1 ✓, el valor del original ✓), los botones de
  **«Abrir…»** ✓ y **«Guardar como…»** ✓, y el monitor vacío dice *"Abra un
  modelo para ver su reproducción."* ✓ (sin advancear el final ✓).
- **El modelo del usuario** (`modelo de test` ✓) corre en el motor ✓:
  `END_TIME` 3600 ✓, la cola `QALMACEN` con 3.236 esperando ✓, el almacén
  `ALMACEN` ✓, y **los tiempos enteros** ✓.

## 6. Lo que queda fuera

- **Publicar** (push, PR, versionado, firma) ✓: decisión del usuario ✓.
- El **AppImage** ✓: no se publica porque el WebKit empaquetado no funciona ✓.
- La diferencia del `QTable` con espacios en el nombre ✓: es del **formato**
  ✓, no del motor ✓ → es otra unidad ✓.
