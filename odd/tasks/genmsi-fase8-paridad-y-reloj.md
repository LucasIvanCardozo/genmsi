# Fase 8 — la paridad que queda: la forma, el reloj y la medición final

La fase 7 cerró el inventario clase por clase contra la respuesta real del
original. Quedaron cuatro diferencias de forma medidas, un brazo del listado
sin escribir, dos decisiones de fondo sobre el reloj y la primera llegada, y
la medición final del producto. Esta fase las cierra.

El empaquetado de la app (instalarla en esta PC o en cualquier otra) queda
fuera de esta fase: seMidió que falta lo de fondo (la carpeta de modelos está
fijada en tiempo de compilación) y se decidió seguir con la fase 7.

## Specs

### S1 — El listado del `RemoveNumber` es el del original

El catálogo ofrece la clase desde `c59adab` y el motor la simula, pero el
listado no tiene un brazo propio: el GPSS que escribimos no es el que escribe
el original. El parche exacto está escrito en `odd/tasks/notas-w3-clases.md`
(`crates/genmsi-core/src/program.rs`, `crates/genmsi-core/src/gpss.rs`,
`crates/genmsi-core/src/report.rs`, `crates/genmsi-core/src/sim.rs`).

La forma que el original escribe es `A_RemoveNumber  REMOVE   E       UnGrupo,,0`
— el grupo, un operando vacío, el número, y el destino de la salida de fallo
como séptimo operando cuando está conectada.

### S2 — Las cuatro diferencias de forma que la unidad 6 midió y no arregló

Medidas contra el original, con su evidencia y su archivo, en
`odd/tasks/notas-w6-resto.md` (sección "Lo que queda fuera"):

- **A** — La tabla de bloques del informe no tiene las filas de las
  conexiones. El original escribe una fila por sentencia del listado, y las
  conexiones son filas más (`TP10`: 69 filas el original, 66 las nuestras;
  `TP3`: 108 y 103). En `crates/genmsi-core/src/report.rs`, fn `block_table`.
- **B** — El orden de los almacenes en el informe: el original los escribe en
  el orden de la declaración (`MIGRACION`, `ENTREGAEQUIPAJE`, `SALIDA` en el
  `TP3`) y nosotros en el alfabético. En `crates/genmsi-core/src/sim.rs` y
  `crates/genmsi-core/src/report.rs`.
- **C** — El rótulo `TABLAS` se repite antes de cada tabla; el original lo
  escribe una vez, al principio de la sección. En `crates/genmsi-core/src/report.rs`,
  fn `render`.
- **D** — El `GENERATE` con la media vacía: el original escribe
  `GENERATE ,,,1` y nosotros `GENERATE 0,,,1`. En
  `crates/genmsi-core/src/gpss.rs`, fn `block_statements`.

Cada una con su prueba, y sin romper el GPSS de 66 de 66 ni las métricas de
66 de 66.

### S3 — Con dispersión, la primera llegada se sortea

> "Sí, que sortee el primer intervalo (recomendada)"

El original sortea el primer intervalo: 60 muestras leídas del reloj con un
`MARK` y recogidas en una `TABLA` (de 30 a 90) dan media 57,133 y desviación
17,728, y el Kolmogorov-Smirnov contra U(30,90) da D=0,1167 con p=0,3666 (no
se rechaza la uniforme). El nuestro es la media exacta en todas las semillas:
600 primeras llegadas, todas 60,000, y el mismo test las rechaza con D=0,5.

Ningún conteo de entradas separa las dos lecturas: esto cambia los **tiempos**,
no las cifras. La evidencia y el inventario están en `odd/tasks/notas-reloj.md`.

Queda sin explicar (y anotado como tal): el generador del original sortea
bajo — los intervalos de la sonda promedian 4,562 contra 5,0, con chi2 26,48
y 4 grados de libertad.

### S4 — El orden de los eventos del mismo instante: primero se mide

> "Medir el orden de los simultáneos primero (recomendada)"

El reloj del original cuenta unidades enteras y el nuestro fracciones. Con
tiempos enteros, muchos eventos comparten instante y el orden dentro del
instante decide quién toma una facilidad, quién sale preemptado y el orden
de la cola. Ese orden **no está medido**.

Hay que medirlo con una sonda propia (del mismo clase que las tres de
`odd/tasks/notas-reloj.md`, con su script en la nota) y decir cuál es la
regla del original: qué gana un `SEIZE` cuando dos transacciones llegan en el
mismo instante, a quién preempta, y el FIFO de la cola.

**El andamiaje de las sondas vive fuera del repositorio** (en `/tmp`), con el
script que las escribe en la nota, como las anteriores.

Por qué el orden importa tanto (medido en el modelo grande): 79.904 eventos
de traza sobre 36.000 unidades de tiempo, 15,9 eventos por unidad entera de
media y **506 eventos compartiendo un mismo instante**. Los tiempos del
original son enteros, así que esos 506 eventos están en el mismo instante y
los ordena su disciplina; los nuestros los separa una fracción, que es un
orden arbitrario.

### S5 — El reloj entero, con la regla de S4 en la mano

> "realmente nuestra iumplementacion es mejor que la original verdad? o seria
> mejor que fuera entero?"

La respuesta es que la nuestra **no es mejor**: es más expresiva (puede
expresar un retardo de 30,5, que el original no puede ni abrir) y menos
fiel. El objetivo del proyecto es la compatibilidad con el original, y un
modelo que el original no puede abrir está fuera del objetivo.

El inventario de lo que cambiaría, archivo por archivo, está en
`odd/tasks/notas-reloj.md` §3: los retardos (`apply_modifier`,
`evaluate_delay`), las llegadas (`install_definitions`, `tick`), el `Advance`,
la preemption, el orden de la cola de eventos, la traza, el reporte
(`event_time`), las estadísticas (`stat.rs`) y el reloj de la app.

**Criterio de aceptación**: los 66 bancos del oráculo siguen coincidiendo en
todas sus métricas y la tolerancia de tiempos (35%) no empeora; si se puede,
se estrecha. Los contratos publicados (`genmsi.report/1`, `genmsi.trace/1`)
no cambian: mismo campo, mismo tipo, valor entero en vez de fraccional.

Esta tarea **depende de S4**: sin la regla del orden de los simultáneos, el
reloj entero cambia el orden de eventos que hasta ahora casi nunca empataban,
y el oráculo lo marcaría.

### S6 — La medición final del producto

- El recuento recalculado de las 66 clases con el núcleo de hoy: cuántas
  verificadas, cuántas con diferencias, cuántas pendientes, y la desviación
  que queda en cada una.
- Las desviaciones que permanezcan, con su evidencia.
- La tasa de cuadros de la app, que nunca se midió: el motor va de 79 a 119
  ms por corrida, y la traza ocupa 183 bytes por evento (15 MB con 82.795
  eventos).

## Tasks

| ID | Specs | Qué | Ruta | Commit |
|---|---|---|---|---|
| T1 | S1, S2 | El brazo del `RemoveNumber` en el listado y las cuatro diferencias de forma (A, B, C, D), con sus pruebas | format + core | uno |
| T2 | S3 | La primera llegada con dispersión se sortea como en el original, con su prueba | core | uno |
| T3 | S4 | La sonda del orden de los eventos del mismo instante y la nota con la regla | core (ejemplo) + docs | uno |
| T4 | S5 | El reloj entero, con la regla de T3 en la mano, y el oráculo como criterio de aceptación | core + app | uno |
| T5 | S6 | La medición final: el recuento de las 66 clases, las desviaciones que quedan y la tasa de cuadros de la app | docs | uno |

El orden importa: **T4 depende de T3**. T1, T2 y T3 no se pisan (T1 y T2
tocan `sim.rs`, así que van seguidos).

## Log

- **L1** — "Ahora no, seguir con la fase 7" (la pregunta era si se empaqueta la
  app; el empaquetado queda para otro día).
- **L2** — "Sí, que sortee el primer intervalo (recomendada)" — la primera
  llegada con dispersión se sortea.
- **L3** — "Medir el orden de los simultáneos primero (recomendada)" — el reloj
  entero se decide después de medir la regla del orden.
- **L4** — "realmente nuestra iumplementacion es mejor que la original verdad?
  o seria mejor que fuera entero?" — la respuesta, con los números: no es
  mejor, es más expresiva y menos fiel; en el modelo grande 506 eventos
  comparten un mismo instante, y hoy los ordenamos con una fracción que es un
  orden arbitrario.
- **L5** — "ahora pregunto la app ya la puedo generar para instalarla en la pc
  o en cualquier pc? o como es eso?" — medido: falta el fondo (la carpeta de
  modelos está fijada en tiempo de compilación, con `env!("CARGO_MANIFEST_DIR")`,
  y no hay diálogo de archivos), más el CLI de Tauri, que no está instalado.
  Queda anotado para otra unidad.
- **L6** — "a que te referis con los modelos? los ejemplos de los tps? si es
  eso, literalmente no me intereza" — aclaración del requisito: los ejemplos
  del repo (`fixtures/models/`: los ejercicios de referencia del original y
  los bancos de las sondas) **no son el producto**. El requisito de la unidad
  de empaquetado es que la app trabaje con **los modelos del usuario, desde
  donde estén**:
  - hoy el selector es un desplegable que se llena con `list_models()`, que
    lee `fixtures/models`: **no hay forma de abrir un `.gmdl` de otra parte**
    (ni diálogo, ni arrastrar y soltar, ni escribir una ruta). El modelo
    `.gmdl` del usuario es el que la app tiene que abrir;
  - la carpeta de modelos no puede estar fijada en tiempo de compilación: la
    lista tiene que ser configurable, o la app tiene que abrir el modelo que
    el usuario elija;
  - los ejemplos pueden quedar como semilla o como material de prueba, pero no
    como la lista que la app ofrece.
- **L7** — "bien, ademas no quiero que este ese selector de modelos (porque
  esto realmente es para probar cosass, pero para la aplicacion final esto no
  tiene sentido me explico?)" — el **selector de modelos se quita** de la app
  final. Es una afordancia de prueba (una lista de una carpeta fija), no una
  manera de trabajar:
  - hoy el selector es un `<select>` (`crates/genmsi-app/src/components/ModelPicker.tsx`)
    alimentado por `list_models()`, que lee `fixtures/models`;
  - el motor no tiene ese límite: `openModel(path)` abre cualquier ruta. Lo
    que restringe el modelo es sólo la UI;
  - la app final necesita **el diálogo del sistema** para abrir, y otro para
    "guardar como", porque `save(path?)` también necesita una ruta que el
    usuario elija;
  - **orden de las cosas**: el diálogo tiene que entrar **antes** de quitar el
    desplegable, o la app se queda sin forma de abrir nada (ni siquiera para
    probar);
  - el modo navegador (el `devBridge` con su ruta `models`) puede seguir
    listando los ejemplos para el desarrollo: es una herramienta de desarrollo,
    no la app final.
- **L8** — "Diálogo del sistema + recientes (recomendada)" — así se abre un
  modelo en la app final:
  - **Abrir…**: el diálogo del sistema (plugin de diálogo de Tauri);
  - **Guardar como…**: el diálogo del sistema (el `save(path?)` también
    necesita una ruta que el usuario elija);
  - **recientes**: una lista corta de los últimos modelos abiertos, en un
    menú, **persistida** en el directorio de datos de la app;
  - el nombre del modelo abierto se ve en la barra (el estado, no un
    selector);
  - el desplegable de `ModelPicker.tsx` desaparece de la app final; el modo
    navegador puede conservar su lista para el desarrollo.
- **L9** — "de nuevo paso! reinicie la pc por si era eso" — la unidad del reloj
  entero se interrumpió **tres veces** y la sesión con ella. La causa no era el
  código sino la memoria de la máquina (15 GiB), medida:
  - una sola invocación de cargo compilaba **49 binarios de test** con **12
    `rustc` en paralelo** (el limitador de trabajos no existía en el repo);
  - el worker copiaba el repositorio y su `target` (de 518 MB a 1,4 GB) a
    `/tmp`, que es un **tmpfs**: un archivo que vive en la memoria RAM;
  - `/tmp` además arrastraba el andamiaje de las unidades anteriores (7,5 GB);
  - la sesión en sí ocupa ~600 MB con el contexto completo.
  Decisiones:
  - `.cargo/config.toml` en el repo: cargo con **4 trabajos** (`edc58aa`);
  - el andamiaje de las corridas va a **`~/genmsi-scratch`** (disco), nunca a
    `/tmp` (`e393028`), y para comparar el "antes" se usa `git worktree` y no
    una copia del repositorio;
  - un watcher vigila la memoria libre, el uso de `/tmp` y el proceso más
    grande, para frenar la carrera **antes** de que el kernel cierre servicios.
- **L10** — "es que vos no estabas haciendo nada! nose porque decis que estabas
  corriendo algo en paralelo" — **corrección**: las verificaciones del padre sí
  corrían en segundo plano, pero **no en paralelo** con la compilación del
  worker: cargo toma un cerrojo sobre el directorio `target` y una espera a la
  otra. La causa real del agotamiento de memoria eran los **12 `rustc` de una
  sola invocación** más el scratch en el tmpfs, no dos compilaciones al mismo
  tiempo.
- **L11** — "pero como 12 gb de compilacion, tanto necesita?? es una locura" —
  no era una compilación de 12 GB: eran doce procesos a la vez, cada uno con la
  información de depuración del crate más grande del proyecto (800 MB a más de
  1 GB cada uno), más el enlazado. Con 4 trabajos, el pico es de unos 4 GB.
  Queda pendiente decidir si el limitador vive en el repo (afecta a quien clonee)
  o en `~/.cargo/config.toml` (sólo esta máquina).
- **L12** — El trunc del instante destapa un caso que con tiempos fraccionales no
  existía: un retardo que trunca a **0** hace que el evento dispare en el
  mismo instante, y si el bloque se reprograma a sí mismo **el reloj nunca
  avanza** (el horizonte, que se comprueba al avanzar el reloj, nunca se
  dispara): la corrida no termina y la cola de eventos crece sin límite (el test
  `cancelacion` llegó a 6,8 GB de RSS). La regla **no se inventa**: hay que
  medir qué hace el original con un retardo de 0 y copying esa semántica.
- **L13** — La instalación de la app para Linux pasa a ser un comando:
  `scripts/install-linux.sh` (`6a746c7`) compila con `-j 2` e instala el
  binario, el lanzador y el icono en las carpetas del usuario, con
  `crates/genmsi-app/LEEME.txt` como texto de la carpeta instalada
  (`989f421`). La instalación anterior era a mano y quedó una copia vieja del
  motor: los números que mostraba la app no eran los de la última versión.
- **L15** — El reloj entero (worker `mvmhb5vi-1-i0mx`, commits `27f42fb` y
  `934a5fb`): el reloj de la corrida es **entero**, como el del original ✓. El
  reporte, la traza, las transacciones, las estadísticas y los eventos
  pendientes salen enteros **por construcción** (todos leen el mismo reloj ✓),
  y los contratos publicados conservan sus campos y sus tipos ✓. Los retardos
  siguen el manual del original: el sorteo de la dispersión es un entero sobre
  `[A-B, A+B]` inclusive ✓ y el producto media × función se trunca ✓.
  - **El bucle infinito del retardo 0**, medido contra el original:
    - el original **rechaza** un modelo con media fraccionaria ("el modelo no
      puede abrirse porque el archivo presenta errores estructurales") ✓: no
      hay original contra el que copiar ✓;
    - con media **entera 0** el original **abre el modelo, lo corre y nunca
      termina**: el reloj se queda en `00:00:00` y el simulador dice
      "Simulando..." para siempre ✓ (su memoria crece ✓). O sea que un retardo
      de cero **no** avanza el reloj del original tampoco ✓ → truncar a 0 es
      fiel ✓;
    - nuestro motor hacía lo mismo pero con un GROWTH de la cola de eventos
      (3,5 GB de RSS en 41 s ✗). La diferencia con el original: el original
      rechaza el modelo de media fraccionaria ✓ y el nuestro lo corre ✓.
  - **La desviación que el workerxrime y dejé aceptada**: la media que no es
    entera **conserva su fracción por dentro** (el reloj visible sigue entero
    ✓). Está **acotada a los modelos que el original rechaza** ✓: para todo
    modelo que el original puede abrir los dos relojes son el mismo ✓✓. La
    razón de no truncar la media: truncar a 0 congelaría el reloj (el bucle
    infinito ✓) y la app se comería la memoria de la máquina ✗ — el mismo
    problema que nos costó tres interrupciones ✓.
  - **Verificado por el padre**: suite verde ✓ (21 binarios, 0 fallos ✓),
    clippy verde ✓, oráculo **66 de 66** ✓ con las mismas líneas por banco ✓,
    GPSS **66 de 66** ✓, las diez sondas del orden **10 de 10** ✓, y la paridad
    con los 25 modelos en **902 de 958** ✓ (la base mide 910 ✓, pero la medida
    se mueve ±20 métricas entre corridas ✓: con 10 corridas es 821 contra 804 ✓
    y con 20 es 799 contra 800 ✓).
  - **La conclusión honesta**: el reloj entero **no mejora de forma medible** la
    paridad con el original ✓ (el ruido de la medida es mayor que el efecto ✓).
    Lo que sí cambia es la **forma**: los tiempos del reporte y de la traza
    son enteros ✓, como los del original ✓ → las cifras de los dos son
    comparables ✓✓.
