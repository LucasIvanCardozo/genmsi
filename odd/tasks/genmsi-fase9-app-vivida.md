# Fase 9 — la app viva: una corrida que tarda no puede colgar la ventana

Un modelo sencillo —cinco generadores, un almacén de una unidad y un reloj—
deja la aplicación inutilizable: la ventana se congela y hay que matar el
proceso. El modelo está bien. El motor y la app son los que no están a la
altura, y son varios problemas distintos que esta fase cierra.

## Specs

### S1 — El motor no reordena la lista de espereros en cada liberación

Cuando una facilidad se libera, el motor tiene que elegir al siguiente
esperero por la regla del original (prioridad descendente y, a igualdad, orden
de llegada; la regla está medida en `odd/tasks/notas-orden-simultaneos.md`).
Hoy la implementa **ordenando la lista entera cada vez**: saca la lista, la
filtra, la copia y la ordena (`crates/genmsi-core/src/sim.rs`, fn
`release_waiters`, y tres sitios más).

La lista ya está ordenada —en cada liberación se va una transacción y llega
una— así que reordenarla entera es trabajo tirado. Con una cola que crece, el
costo de cada liberación crece con la cola y **el costo total crece de forma
cuadrática**.

Medido con el modelo del usuario (`modelo de test`, cinco generadores con
media 5±3, un `StorageUse` de 10±5 sobre un almacén de tamaño 1 y un reloj de
3600):

| horizonte | 50 | 100 | 200 | 400 | 800 | 1600 | 3600 (completa) |
|---|---|---|---|---|---|---|---|
| tiempo | 5 ms | 4 ms | 16 ms | 88 ms | 547 ms | 4,5 s | **58,4 s** |

La cola llega a 3236 transacciones esperando al final.

**Lo que tiene que quedar**: la lista de espereros se mantiene ordenada y el
que llega se inserta en su lugar; liberar una facilidad no reordena nada. El
costo total tiene que ser lineal en el tamaño de la cola.

**Lo que no puede cambiar**: a quién le toca la facilidad (la regla del
original), los números del reporte y el orden de los eventos. El cambio es de
costo, no de comportamiento: los 66 bancos del oráculo tienen que seguir
coincidiendo en todas sus métricas.

### S2 — La simulación no ocupa el hilo principal de la app

El comando de Tauri que corre la simulación (`simulate_model` y `trace_model`
en `crates/genmsi-app/src-tauri/src/commands.rs`) es **síncrono**, así que
Tauri lo ejecuta en el hilo principal de la aplicación. Mientras corre:

- la ventana no repinta (no se ve el "Ejecutando…");
- la app no se puede cerrar con educación: hay que matarla;
- no hay forma de cancelar.

La simulación tiene que correr en un hilo de trabajo y la ventana tiene que
seguir viva: repintando, mostrando el progreso y pudiendo cancelarse.

**Esto no acelera la simulación**: el bucle de eventos es secuencial por
naturaleza (es un simulador de eventos discretos: cada evento puede cambiar
el estado que ve el siguiente, así que no se puede repartir entre núcleos sin
cambiar el significado del modelo). Lo que se arregla es que la aplicación
deje de parecer colgada.

### S3 — Una corrida que no tiene fin se dice, no se cuelga en silencio

El motor termina una corrida por cuatro vías: una transacción que llega a un
`Terminate` con operando A (el `START`, que hoy vale 1.0 fijo), el tic del
`Clock`, un horizonte explícito (`until`) y el guard de 50 millones de eventos.

Cuando ninguna de las tres primeras puede ocurrir, la app lanza una corrida
infinita sin avisar. La app tiene que:

- poner un **horizonte por defecto** (o el que el usuario elija, que ya
  existe como campo), y
- **avisar** cuando el modelo no tiene forma de terminar, en vez de colgarse
  en silencio.

El motor ya sabe detectarlo: `genmsi_format::validate_runnable` rechaza los
modelos sin `Terminate` con operando A y sin `Clock` con el mensaje "the
simulation would never finish". Ese aviso tiene que llegar a la app, que hoy
sólo lo ve si se pide el lint.

### S4 — El `START` del original: hay que medirlo

El motor descuenta el `START` con el operando A del `Terminate` y termina la
corrida cuando llega a cero, pero el valor inicial es **1.0 fijo**
(`crates/genmsi-core/src/program.rs`, `start_counter`). En el original, el
`START` es la cantidad de entidades: una transacción que llega al `Terminate`
con operando A descuenta una unidad, y la corrida termina cuando el `START`
llega a cero.

Con el `START` en 1, una corrida termina con la **primera** transacción que
llega al `Terminate`, que no es lo que hace el original. Hay que medir la
semántica del original (cuál es el `START` inicial y qué lo decrementa) y
corregir el motor.

### S5 — La app no enseña el final de la corrida antes de tiempo

La cabecera de la reproducción muestra cuatro datos que son **del final de la
corrida**: el título del modelo, el rango de tiempos (`0 → 3600`), la cantidad
de registros (`75317`) y la duración (`Dura 1:00:00`). Están en
`crates/genmsi-app/src/components/MonitorView.tsx` y salen de
`monitor.startTime`, `monitor.endTime` y `monitor.records`.

Con la reproducción de a poco, la gracia es **no saber cómo va a terminar**.
Los datos están en otras partes de la app y ocupan una franja entera que no
hace falta.

**Lo que tiene que quedar**: la cabecera desaparece. El alcance del
movimiento se sigue viendo en el control de tiempo de la reproducción (que es
un control, no un dato), y el aviso de traza recortada se queda.

El panel de **Reporte** ya muestra los números vivos durante la reproducción
(`useLive`), así que el único lugar que anticipaba el final es esa cabecera.

### S6 — El aviso de guardado se va solo

Guardar deja un aviso verde en la parte de arriba de la app ("Se guardó
modelo de test (7245 bytes)") que **no se va nunca**: `savedMessage` sólo se
limpia cuando se abre o se guarda otra cosa (`crates/genmsi-app/src/state/store.ts`
y `crates/genmsi-app/src/App.tsx`).

Un aviso tiene que ser un aviso: se muestra unos segundos y desaparece, sin
dejar el lugar ocupado.

### S7 — «Ejecutar» simula una sola vez

Hoy «Ejecutar» hace **dos corridas completas del motor**: una para el informe
(`simulateCurrent`) y otra para la traza (`traceCurrent`). La segunda repite
todo el trabajo de la primera, y el motor es determinista, así que la traza
pertenece al informe de la primera.

**Lo que tiene que quedar**: una sola corrida que produce **el informe y la
traza**. El motor puede: una corrida con la traza prendida da el resultado
(`RunResult`, de donde sale el informe) y los eventos (de donde sale la
traza). La fachada expone una operación que devuelve los dos documentos, y la
app la usa.

Efecto: la app hace la mitad del trabajo del motor, y «Simular» y «Ejecutar»
siguen siendo una corrida cada uno.

### S8 — El modelo se abre con el diálogo del sistema, y el selector desaparece

En la barra de la app hay un desplegable "MODELO — Selecciona un modelo…"
(`crates/genmsi-app/src/components/ModelPicker.tsx`) que lista la carpeta de
ejemplos del repositorio. Es una afordancia de prueba, no una manera de
trabajar: **no hay forma de abrir un modelo de otra parte** (ni diálogo, ni
arrastrar y soltar, ni escribir una ruta), y el motor no tiene ese límite
(`openModel(path)` abre cualquier ruta; lo que restringe el modelo es sólo la
UI).

La app final tiene que:

- **Abrir…**: el diálogo del sistema (el plugin de diálogo de Tauri,
  `tauri-plugin-dialog`, que ya está en el caché de cargo);
- **Guardar como…**: el diálogo del sistema (el `save(path?)` también necesita
  una ruta que el usuario elija);
- **recientes**: una lista corta de los últimos modelos abiertos, en un menú,
  **persistida**;
- el nombre del modelo abierto en la barra (estado, no un selector);
- **el desplegable desaparece**.

El modo navegador (el `devBridge` con su ruta `models`) puede conservar su
lista de ejemplos: es una herramienta de desarrollo, no la app final.

**Orden**: el diálogo entra **con** la quita del desplegable, en el mismo
cambio. Quitar el desplegable sin el diálogo deja la app sin forma de abrir
nada.

## Tasks

| ID | Specs | Qué | Ruta | Commit |
|---|---|---|---|---|
| T1 | S1 | La lista de espereros se mantiene ordenada: liberar no reordena | core | uno |
| T5 | S7 | Una sola corrida: el informe y la traza salen del mismo paso del motor | app | uno |
| T6 | S5, S6 | La cabecera del monitor desaparece y el aviso de guardado se va solo | app | uno |
| T7 | S8 | El diálogo del sistema para abrir y guardar, los recientes, y el selector de modelos fuera | app | uno |
| T2 | S2 | La simulación corre en un hilo de trabajo: la ventana sigue viva, con progreso y cancelación | app | uno |
| T3 | S3 | Horizonte por defecto y aviso cuando el modelo no tiene forma de terminar | app + core | uno |
| T4 | S4 | Medir el `START` del original y corregir el motor | core + docs | uno |

El orden importa: **T1 va primero** (es la causa del costo). T5, T6 y T7 no
tocan el motor y pueden ir en paralelo. T2 y T3 pueden venir después. T4 es
una medición contra el original y no depende de los otros.

## Log

- **L1** — "use clock default, 5 generate con media 5+-3 conectados a un
  storageuse de 10+-5 con storagedef de tamaño1 y este conectado a un
  terminate" — el modelo con el que la app se cuelga.
- **L2** — "si yo abro la app y armo el modelo que no me funcionaba vos podes
  verlo? porque por ahi habia algo mal configurado no? o no necesitas que lo
  haga?" — el modelo **no** estaba mal configurado: se midió con su archivo
  (`modelo de test`) y termina bien (END_TIME 3600). El problema es el costo y
  el hilo.
- **L3** — "listo, lo guiarde como 'modelo de test'" — el modelo, guardado
  desde la app. Sus conexiones: los cinco `Generate` al `StorageUse`, el
  `StorageUse` al `Terminate` (sin operando A), el `Clock` de 3600 suelto y el
  `StorageDef` de tamaño 1. Nota: el archivo quedó sin extensión en la raíz
  del repositorio; hay que moverlo (es material del usuario, no del repo).
- **L4** — "no entiendo que es eso del esperero... algo que se me ocurre, no
  se puede hacer que se usen mas de un hilo en vez de uno solo?" — el esperero
  es una transacción que espera en una facilidad. Los hilos: el cálculo de la
  simulación **no** se puede paralelizar (es secuencial por naturaleza), pero
  que la app se cuelgue **sí** es un problema de hilos (S2), y el reordenamiento
  no se arregla con más hilos sino dejando de ordenar (S1).
- **L5** — "esta parte la veo totalmente innecesaria, estos datos en todo caso
  se encuentran en otros lados de la app deberiamos quitarla, a parte que ocupa
  lugar al dope" — la cabecera de la reproducción (el título, los tiempos, los
  registros y la duración) desaparece (S5).
- **L6** — "al guardar quedo eso ahi, minimo en x tiempo deberia desaparecer,
  quedo ocupando lugar ahi de mas" — el aviso de guardado se va solo (S6).
- **L7** — "si no entendi mal al darle a 'ejecutar' la simulacion se ejecuta
  dos veces y esto no esta mal que pase? deberia ejecutar solo la simulacion
  del trace no? y realmente no deberia aparecer en ningun lado datos de como
  va a terminar la simulacion porque esa es la gracia del trace que se simule
  de a poco, en todo caso el dato final aparecera al final de la simulacion.
  de esta manera solo se simula una sola vez tanto para el boton de simular
  como el de ejecutar" — una sola corrida (S7) y ningún dato del final antes
  de tiempo (S5).
- **L8** — "quiero que lo quitemos ya que dijimos que agregariamos el boton
  de abrir te acordas?" — el desplegable de modelos se quita **ya**, junto con
  el botón "Abrir…" del diálogo del sistema (S8). El orden que se había
  fijado (el diálogo antes de quitar el desplegable) se cumple en el mismo
  cambio.
