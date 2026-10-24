# El `START` del original: de dónde sale, qué lo descuenta y qué termina la corrida

Unidad de **medición** de la spec **S4** de `odd/tasks/genmsi-fase9-app-vivida.md`.
No se tocó el motor, ni la app, ni el catálogo: lo que hay abajo son
medidas contra el original (GenMSI 1.2020.07.28 BETA1 bajo Wine) y un
inventario de lo que habría que cambiar en el motor.

## Las preguntas (textuales)

1. **¿Cuál es el `START` inicial de un modelo?** ¿La cantidad de
   transacciones que el generador va a crear? ¿Un número que declara el
   modelo? ¿El del contenedor raíz?
2. **¿Qué termina la corrida exactamente?** ¿El `START` en cero? ¿El
   `Clock`? ¿Las dos cosas?
3. **¿Qué reporta el original cuando la corrida termina por el `START`?**
   ¿Qué dice el reporte (el `END_TIME`, la sección de eventos pendientes) y
   cómo lo dice nuestro motor?

## El veredicto, en una línea

**El `START` del original es el contador de terminaciones de la corrida, y
lo fija quien la lanza (el campo "Start" del panel Simulador del original,
1 por defecto): el modelo no lo declara en ninguna parte. Lo descuenta el
`Terminate` **con operando A** (su operando A unidades por transacción que
llega) y el `Clock` (una unidad por tic); un `Terminate` **sin** operando A
no descuenta nada. La corrida termina cuando el contador llega a cero **o por
debajo**. Nuestro motor tiene **la regla correcta** pero el valor inicial
**clavado en 1.0**: con el `Start` del original en 1 (su defecto) nuestras
corridas dan el reporte idéntico, y con el `Start` en 5 ours termina
cinco veces antes que el original.**

## Las sondas

Todas las sondas viven **fuera del repositorio**, en
`~/genmsi-scratch/start/probes` (la unidad tiene prohibido escribir en
`/tmp`, que es un tmpfs de RAM). El script que las arma está en el
apéndice; los módulos salen de los bancos que el repositorio ya genera
(`scripts/wine-oracle/benches/*.gmdl`), que el original abre sin quejarse.

| Sonda | Qué es | Qué discrimina |
|---|---|---|
| `pTermina1` | `GENERATE 100,0` → `TERMINATE 1`, **sin** `Clock` | cuántas transacciones terminan antes de que la corrida termine: si el `START` valiese N, la corrida duraría N llegadas. Con `Start`=1 termina con la primera; con `Start`=5, con la quinta |
| `pTermina3` | lo mismo, con `TERMINATE 3` | si el contador se descuenta de a 3 o de a 1, y **qué pasa cuando el contador no cae exactamente en cero** (5 − 3 = 2, 2 − 3 = −1) |
| `pReloj` | un `Clock` de 100 y nada más | si el `Clock` descuenta el `START` (el `Clock` del original compila a `GENERATE 100` + `TERMINATE 1`) y si el `Clock` es lo que termina la corrida |
| `pCantidad` | `GENERATE 100,0,,5` → `TERMINATE` (sin operando) + `Clock` de 1000 | tres cosas a la vez: (a) si el `START` inicial es la `Cantidad` del generador; (b) si un `TERMINATE` **sin** operando A descuenta el contador; (c) qué pasa cuando el generador y el `Clock` compiten |
| `pSecco` | `GENERATE 100,0,,5` → `TERMINATE` (sin operando), **sin** `Clock` | qué hace el original con un modelo que no puede terminar: el `TERMINATE` sin operando A no descuenta y no hay `Clock` |
| `pSecco2` | `GENERATE 100,0,,5` → `TERMINATE` (sin operando) **+ un `TERMINATE 1` al que nada llega** | el otro final de una corrida: no es que el contador se vacíe, es que **no queda nada que hacer** (los eventos se secan) |

La sonda `pSecco2` es la que separa las dos maneras de terminar una corrida:
el contador sigue en 1 y, aun así, la corrida acaba.

**Antes de armar sondas, se miró lo que ya había.** En esta máquina
`/tmp/all-oracle` no existe (es un tmpfs y la máquina se reinició), pero
quedaron los reportes del original de los 25 modelos de referencia, en
`fixtures/outputs/*.report.txt`. El de `TP2__problema1` ya daba la pista: su
`Clock` de 3600 y un `TERMINATE` sin operando; el `END_TIME` es **3600**, el
`TERMINATE` del reloj tiene **1** entrada y el `TERMINATE` sin operando
tiene **59**. O sea: el `TERMINATE` sin operando no toca el contador y el
reloj lo vacía en el primer tic, con el `Start` del panel en su defecto (1).
Eso es lo que las sondas confirman con números.

## La medida

### 1. El `START` inicial: el campo "Start" del panel Simulador

El original tiene, en la barra del panel **Simulador**, un campo **"Start:"**
que viene con el valor **1** (la captura de `scripts/wine-oracle/out/`
y la de esta unidad lo muestran). Es el `START` de la corrida: el manual del
original (`Genmsi_20230711.backup/VISUALSIS.HLP`) lo dice textualmente:

> "Debemos ordenar que se realice la simulación indicando **cuántas
> transacciones queremos que se contabilicen como terminadas (bloque START del
> lenguaje)**. Para esto debemos hacer click en el botón "Start" de la Barra
> de la Simulación o presionar la tecla F9. [...] En nuestro caso, ingresamos
> el valor 1000 y presionamos la tecla Enter. [...] Esto indica que se han
> logrado simular la cantidad de transacciones indicadas y no se ha producido
> ningún error durante la simulación."

O sea: el `START` es **un parámetro de la corrida**, no una propiedad del
modelo. Y el modelo **no tiene dónde declararlo**:

- el catálogo del original (`Genmsi_20230711.backup/gClasses.xml`) tiene 66
  clases y **ninguna** es un `START` (ni un `Run`, ni un `Control`);
- el módulo raíz (`moduleclass="main"`) **no tiene `props`**: sus atributos
  son `genprefer`, `exehead`, `nonexehead`, `exefoot`, `nonexefoot`;
- el listing de GPSS que el original genera **no lleva ninguna sentencia
  `START`**: el `START` es, en GPSS, una carta de control de la corrida, no
  una sentencia del programa.

Lo que **no** es el `START` inicial, medido:

| Candidato | Por qué queda descartado |
|---|---|
| la `Cantidad` del generador (`GENERATE 100,0,,5`) | con `Start`=1 la corrida de `pCantidad` termina a los **1000** (el primer tic del `Clock`), no a los 500 (la quinta transacción); con `Start`=5 termina a los **5000** (el quinto tic), no a los 500 |
| un número que declara el modelo | no existe: no hay clase `START`, el modelo raíz no tiene propiedades, y el listing de GPSS no lleva `START` |
| el del contenedor raíz | el contenedor raíz es el módulo `main`, que no tiene propiedades |

La prueba directa de que el campo "Start" es el `START` de la corrida: con
`pTermina1` (un `GENERATE 100,0` y un `TERMINATE 1`), el `END_TIME` del
original **es 100 con `Start`=1 y es 500 con `Start`=5**, y el número de
entradas del `TERMINATE` es 1 y 5 respectivamente.

Un dato más del mismo panel: con `Start`=0 el original **no corre** y
dice, en una ventana propia:

> **El Start debe ser mayor a cero.**

### 2. Qué lo descuenta

El manual del original (`VISUALSIS.HLP`, sección `TERMINATE`) lo dice, y las
medidas lo confirman (las comillas del manual están recompuestas: el
`strings` del `.hlp` parte las palabras con acento, "transacci\nn"):

> "El contador de terminación, que está seteado por una sentencia START es
> decrementado en 1."
>
> "Cuando una transacción ingresa al bloque TERMINATE, el operando A es
> evaluado y usado para decrementar el contador de terminación. **Si el
> operando A no es usado, el contador de terminación no es cambiado.**"
>
> "El contador de terminación es seteado por la sentencia START. Cuando el
> contador de terminación llega a 0, la simulación termina, y a menos que sea
> suprimida por la sentencia START, un reporte estándar es escrito."

Y el `Clock` del original (`gClasses.xml`) lo dice de su lado:

> "frecuencia, en unidad de tiempo, con la que se **descuenta 1 unidad del
> START**"

Las medidas:

| Qué llega al `TERMINATE` |_units_ del `START` que descuenta | Cómo se midió |
|---|---|---|
| `TERMINATE 1` | 1 | `pTermina1`: con `Start`=5 terminan 5 transacciones (`END_TIME` 500) |
| `TERMINATE 3` | 3 | `pTermina3`: con `Start`=10 terminan 4 transacciones (`END_TIME` 400): 10 → 7 → 4 → 1 → **−2** |
| `TERMINATE` **sin** operando | **0** | `pCantidad`: las 5 transacciones que llegan al `TERMINATE` sin operando **no** tocan el contador (con `Start`=5 la corrida llega a los 5000, que es el quinto tic del `Clock`, y no a los 500) |
| un tic del `Clock` | 1 | `pReloj`: con `Start`=3 la corrida termina a los **300** (tres tics de 100) y el `TERMINATE 1` del reloj tiene 3 entradas; con `Start`=1, a los 100 |

El `Clock` del original **es** un `Terminate` con operando 1: el listing de
GPSS que genera lo dice (`pReloj`):

```
                GENERATE        100
                TERMINATE       1
```

O sea: el `Clock` del original y nuestro `Clock` son **la misma cosa** —los
dos compilan a un generador + un `TERMINATE 1`— y por eso los dos
descontan una unidad del `START` por tic.

### 3. Qué termina la corrida

La regla del original, escrita con lo medido:

> La corrida termina cuando el **contador de terminaciones** (el `START`)
> **llega a cero o por debajo**. Lo bajan el `Terminate` con operando A (su
> operando A unidades por transacción) y el `Clock` (una unidad por tic).
> Un `Terminate` sin operando A no lo baja.

El "o por debajo" está medido, y es la parte que la palabra "llega a 0" del
manual no dice: con `pTermina3` y `Start`=5 el contador queda en 2 y en
−1, y **la corrida termina ahí** (`END_TIME` 200, dos entradas del
`TERMINATE`). Si la regla fuera "exactamente cero", esa corrida no
terminaría nunca.

Las dos maneras de terminar, entonces, son:

| Final | Cómo se lo reconoce | Medida |
|---|---|---|
| el contador se vació | el `END_TIME` es el instante del evento que lo vació | `pTermina1` (`Start`=5): `END_TIME` 500 |
| no queda nada que hacer | el contador **no** llegó a cero, pero los eventos se secaron | `pSecco2` (`Start`=1): `END_TIME` 500, el `TERMINATE 1` al que nada llega tiene 0 entradas y el contador sigue en 1 |

Y hay un tercer caso, que el original **se niega a correr**: un modelo sin
`TERMINATE` con operando A en el programa de GPSS. `pSecco` es ese modelo
(el `TERMINATE` sin operando no cuenta) y el original responde:

```
Advertencia -> No hay bloques TERMINATE con parámetro A

ERROR!!
T. Nº 5
```

El `Clock` **sí** cuenta, porque compila a un `TERMINATE 1`: `pReloj` (un
`Clock` y nada más) corre sin problema. O sea, la exigencia del original es
sobre el **programa de GPSS**, no sobre el modelo.

### 4. Lo que el original reporta cuando la corrida termina por el `START`

El reporte del original (`Reporte MicroVisualSIS`) **no dice nada del
`START`**: ni la cantidad que se pidió, ni el contador que quedó, ni por qué
terminó la corrida. Lo que dice es lo de siempre:

- la cabecera `START_TIME END_TIME BLOCKS FACILITIES STORAGES`, con el
  `END_TIME` = **el instante del evento que vació el contador** (100, 200,
  300, 400, 500, 1000, 5000 según la sonda y el `Start`);
- la tabla de bloques, con la columna `ENTRY COUNT` (las transacciones que
  llegaron a cada bloque) y el `TERMINATE` con su operando a la vista
  (`TERMINATE       1`, `TERMINATE       3`, `TERMINATE` sin operando);
- la sección `FEC_XACT_NUMBER`, que es la tabla de eventos pendientes, con
  una fila por evento que quedó agendado (`PRIOR`, `BDT`, `CURRENT`, `NEXT`,
  `PARAMETER`, `VALUE`).

Sobre la corrida que termina por el `START`, el original **no dice nada
particular**: no hay un aviso, no hay un renglón, no cambia el formato. La
sección de eventos pendientes es la de siempre: en `pTermina1` con
`Start`=5 queda agendado el siguiente `GENERATE` (una fila, `BDT` 600) y en
`pCantidad` con `Start`=5 queda el siguiente tic del reloj (`BDT` 6000).

Nuestro motor dice lo mismo en el reporte (mismo formato, mismas secciones)
y **además** dice el motivo del final en el documento JSON
(`end_reason: "terminated"`), que el original no tiene: el original no
distingue "se vació el contador" de "no quedaba nada" en el reporte.

### 5. La regla, en una línea

**El `START` es un parámetro de la corrida (el campo "Start" del panel
Simulador del original, 1 por defecto, y tiene que ser mayor que cero); lo
descuenta el `Terminate` con operando A (su operando A unidades por
transacción que llega) y el `Clock` (una unidad por tic); el `Terminate` sin
operando A no lo toca; y la corrida termina cuando el contador llega a cero
o por debajo.**

## La comparación con nuestro motor

Nuestro motor, con el `START` inicial clavado en 1.0
(`crates/genmsi-core/src/program.rs`, `start_counter: 1.0`), sobre **los
mismos modelos**:

| Sonda | `Start` del original | Original: `END_TIME` / entradas del `TERMINATE` | Nuestro motor: `END_TIME` / `end_reason` | ¿Iguales? |
|---|---:|---|---|---|
| `pTermina1` | 1 | 100 / 1 | 100 / `terminated` | **sí** (reporte idéntico byte a byte) |
| `pTermina1` | 5 | **500** / 5 | **100** / `terminated` | **no**: el nuestro termina con la primera transacción |
| `pTermina3` | 1 | 100 / 1 | 100 / `terminated` | **sí** |
| `pTermina3` | 5 | **200** / 2 | **100** / `terminated` | **no** |
| `pTermina3` | 10 | **400** / 4 | **100** / `terminated` | **no** |
| `pReloj` | 1 | 100 / 1 (del reloj) | 100 / `terminated` | **sí** |
| `pReloj` | 3 | **300** / 3 (del reloj) | **100** / `terminated` | **no** |
| `pCantidad` | 1 | 1000 / 1 (del reloj) | 1000 / `terminated` | **sí** |
| `pCantidad` | 5 | **5000** / 5 (del reloj) | **1000** / `terminated` | **no** |
| `pSecco2` | 1 | 500 (fin por "no queda nada") | 500 / `no_events` | **sí** (reporte idéntico byte a byte) |
| `pSecco` | 1 | `ERROR!!` ("No hay bloques TERMINATE con parámetro A") | error: "no TERMINATE block with operand A and no CLOCK block: the simulation would never finish" | los dos se niegan (el texto no) |

Con **`Start`=1**, que es el defecto del original, **los cinco reportes
comparados salen idénticos byte a byte** (`pTermina1`, `pTermina3`,
`pReloj`, `pCantidad`, `pSecco2`): mismos `END_TIME`, mismos conteos,
misma sección de eventos pendientes. La diferencia **no** está en la regla,
está en el valor inicial.

Lo que nuestro motor tiene bien:

- la regla del descuento (el `Terminate` con operando A baja el contador
  por su operando, el `Clock` baja una unidad por tic, el `Terminate` sin
  operando no baja nada) — `crates/genmsi-core/src/sim.rs`, fn `terminate`
  (línea 2293): `self.terminations_left -= spec.units;` y, sólo si
  `spec.units > 0.0`;
- el final de la corrida cuando el contador llega a **cero o por debajo**
  (`if self.terminations_left <= 0.0 { self.finish(EndReason::Terminated) }`),
  que es exactamente lo que se midió en el original (el caso de −1);
- el `Clock` como generador + `Terminate` con una unidad, igual que el
  original (`crates/genmsi-core/src/program.rs`, la rama `"Clock"` del
  compilador);
- la exigencia de que el modelo pueda terminar: nuestro `validate_runnable`
  rechaza el modelo sin `Terminate` con operando A **y** sin `Clock`, que es
  la misma exigencia del original (el `Clock` aporta el `TERMINATE 1`).

Lo que le falta: **el valor inicial no se puede cambiar**.

## Qué cambiaría en el motor

El `START` del original no es una propiedad del modelo: es un **parámetro de
la corrida**, como el horizonte (`until`) que ya existe. El cambio, en
resumen: **`Program.start_counter` deja de ser una constante del programa
compilado y pasa a ser un dato de `SimConfig`**, con el defecto 1.

(Los números de línea son los del árbol en el momento de la medición;
otra unidad estaba cambiando `crates/genmsi-core/src/sim.rs` y `report.rs`
en paralelo, así que pueden haber corrido un par de líneas.)

| Dónde | Qué hay hoy | Qué tendría que haber |
|---|---|---|
| `crates/genmsi-core/src/program.rs:944` | el campo `pub start_counter: f64` de `Program` ("Start counter (units the `START` statement requires)") | o disappears, o queda como **el defecto** (1.0) y la corrida lo pisa |
| `crates/genmsi-core/src/program.rs:1140` | `start_counter: 1.0`, el valor con el que se compila el modelo | ningún valor: el `START` **no** se compila, se elige al correr |
| `crates/genmsi-core/src/sim.rs:428` (`terminations_left`) y `:491` (`terminations_left: program.start_counter`) | el contador se inicializa con el del programa | `terminations_left: config.start` (o `config.start_counter`), con `config.start` = 1.0 por defecto |
| `crates/genmsi-core/src/sim.rs:134` (`SimConfig`) | la configuración de la corrida tiene `seed`, `max_events`, `trace`, `stop_time` (el horizonte), `stop` | un campo más: `start: f64` (o `start_counter`), documentado como "el `START` del original: cuántas unidades de terminación termina la corrida; 1 por defecto" |
| `crates/genmsi-core/src/report.rs:1570` (`program_json`, el documento de `explain_model`) | publica `"start_counter": program.start_counter` | el `START` es de la corrida, no del programa: el documento de la corrida debería publicarlo (o el del programa dejar de publicarlo) |
| `crates/genmsi-cli/src/model.rs:39` (fn `config`) y `crates/genmsi-cli/src/commands/simulate.rs` | el CLI arma el `SimConfig` con la semilla y el `--until` | una opción más, `--start N`, con el mismo estilo que `--until` |
| `crates/genmsi-app/engine/src/lib.rs` (fn `simulate_bytes` y `run_documents_of`) | la fachada arma el `SimConfig` con `seed` y `stop_time` | el `start` entra por los mismos caminos que `until` |
| `crates/genmsi-app/src-tauri/src/commands.rs` (`simulate_model`, `trace_model` y el `RunRequest`) | los comandos Tauri pasan `seed` y `until` | un `start` más |
| `crates/genmsi-mcp/src/engine.rs:92` | arma el `SimConfig` | lo mismo, si el MCP debe poder pedirlo |
| la app (el panel del monitor, junto al campo del horizonte de S3) | no hay ningún campo de `START` | un campo, con el defecto 1, que travel hasta el motor |

**Lo que NO cambiaría** (y conviene no tocar):

- la regla del descuento y la del final (`terminations_left <= 0.0`): ya son
  las del original, medidas;
- el `Clock` como `GENERATE` + `TERMINATE 1`: ya es el del original;
- la exigencia de que el modelo pueda terminar (`validate_runnable`): ya es
  la del original;
- el formato del reporte: el original no dice nada del `START` y nosotros
  tampoco deberíamos.

Y una decisión que **no** es de esta unidad: de dónde sale el `START` en la
app. El original lo tiene en el panel Simulador, junto al reloj de la
corrida; nuestra app ya tiene el horizonte (S3) en un lugar análogo, así que
lo natural es un campo al lado. Pero el modelo **no** declara el `START`
(medido), así que **no** se puede guardar en el `.gmdl` sin inventar un
sitio que el original no tiene.

## Lo que esta unidad NO tocó

- Ningún archivo del motor (`crates/genmsi-core/src/**`), de la app, del CLI,
  del MCP, del formato o del catálogo.
- Ningún test.
- `fixtures/`, `scripts/wine-oracle/**` y el `wineprefix` del repositorio: las
  corridas de esta unidad usaron **una copia** del prefijo de Wine en
  `~/genmsi-scratch/start/wineprefix` y su propio display (`:95`), para no
  pisar la sesión de otra unidad.
- No se compiló nada del proyecto: la comparación con nuestro motor se hizo
  con el binario que ya estaba (`target/release/genmsi`).
- No se hizo ningún commit.
- Las sondas y el arnés viven en `~/genmsi-scratch/start/` (fuera del
  repositorio); los scripts están en el apéndice.

## Cómo reproducir

```bash
# 1. las sondas (el script del apéndice)
python3 ~/genmsi-scratch/start/make-probes.py

# 2. el original, con el START que se le pida
#    (el arnés es el del repositorio + el campo "Start" del panel Simulador)
XDISPLAY=:95 bash ~/genmsi-scratch/start/oracle.sh \
    ~/genmsi-scratch/start/probes/pTermina1.gmdl 5 \
    ~/genmsi-scratch/start/oracle/pTermina1-s5

# 3. nuestro motor, sobre el mismo modelo
./target/release/genmsi simulate ~/genmsi-scratch/start/probes/pTermina1.gmdl
```

El `oracle.sh` es el `scripts/wine-oracle/run_oracle.sh` del repositorio con
tres cambios: usa un prefijo de Wine propio (el del repositorio lo comparte
con las otras unidades), pone el valor del campo "Start" antes de correr, y
deja una captura de pantalla del campo puesto (para que se vea qué valor se
mandó). Dos detalles que costaron sangre:

- el campo "Start" está en la barra del panel Simulador, en el punto
  **(316, 156)** de la ventana (el globo que corre la simulación está en el
  **(364, 156)**);
- **`Ctrl+A` es el acelerador de "Abrir"** en el original: usarlo para
  seleccionar el texto del campo abre el diálogo de abrir modelo. El campo
  se vacía con `End` y `BackSpace`.

Los reportes que se compararon quedaron en
`~/genmsi-scratch/start/oracle/<sonda>-s<start>/`.

## Apéndice: los scripts de las sondas

### `make-probes.py` (escribe las sondas)

```python
#!/usr/bin/env python3
"""Build the probes of the START-semantics work unit (S4).

The probes are measurement scaffolding: they live outside the repository, in
``~/genmsi-scratch/start/probes``, and this script is the recipe, so anybody
can rebuild them byte for byte.

The modules come from the benches the repository already generates
(``scripts/wine-oracle/benches/*.gmdl``), which the original opens without
complaint.

Each probe answers one question about the ``START`` of the original:

* ``pTermina1``  -- a ``Terminate`` whose operand A is 1, no ``Clock``.
* ``pTermina3``  -- a ``Terminate`` whose operand A is 3, no ``Clock``.
* ``pReloj``     -- a ``Clock`` of 100 and nothing else.
* ``pCantidad``  -- a ``Generate`` that declares ``Cantidad`` 5, a plain
  ``Terminate`` (no operand A) and a ``Clock`` of 1000.
* ``pSecco``     -- a plain ``Terminate`` and no ``Clock``: a model the
  original refuses to run.
* ``pSecco2``    -- the same, plus a ``TERMINATE 1`` that nothing reaches:
  the run ends because the events run dry.
"""

from __future__ import annotations

import copy
import xml.etree.ElementTree as ET
from pathlib import Path

REPO = Path("/home/lucas/Documentos/Workspace/Genmsi")
BENCHES = REPO / "scripts/wine-oracle/benches"
OUT = Path.home() / "genmsi-scratch/start/probes"


def module_of(tree: ET.ElementTree, module_id: str) -> ET.Element:
    for module in tree.getroot().iter("module"):
        if module.get("moduleid") == module_id:
            return module
    raise KeyError(module_id)


def set_prop(module: ET.Element, name: str, value: str) -> None:
    for element in module.findall("props/property"):
        if element.get("name") == name:
            element.set("value", value)
            return
    raise KeyError(name)


def drop_prop(module: ET.Element, name: str) -> None:
    props = module.find("props")
    for element in list(props):
        if element.get("name") == name:
            props.remove(element)


def link(main: ET.Element, src: str, src_io: str, dst: str, dst_io: str) -> None:
    element = ET.SubElement(main.find("links"), "link")
    element.set("frommoduleid", src)
    element.set("fromioid", src_io)
    element.set("tomoduleid", dst)
    element.set("toioid", dst_io)


def unlink_all(main: ET.Element) -> None:
    links = main.find("links")
    for element in list(links):
        links.remove(element)


def write(tree: ET.ElementTree, name: str) -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    tree.write(
        OUT / f"{name}.gmdl", encoding="latin-1", xml_declaration=False, short_empty_elements=True
    )
    print(name, (OUT / f"{name}.gmdl").stat().st_size, "bytes")


def probe_terminate(unidades: str, name: str) -> None:
    """`GENERATE 100` -> `TERMINATE <unidades>`, without a Clock.

    The generator has no dispersion, so the arrivals are at 100, 200, 300...
    and the number of arrivals that reach the `Terminate` is the number of
    units the `START` had left.
    """
    tree = ET.parse(BENCHES / "Terminate.gmdl")
    root = tree.getroot()
    modules = root.find("modules")
    main = module_of(tree, "0")
    generate = module_of(tree, "1")
    terminate = module_of(tree, "2")
    clock = module_of(tree, "3")

    set_prop(generate, "Media", "100")
    set_prop(generate, "Modificador", "0")
    set_prop(terminate, "Unidades", unidades)
    modules.remove(clock)

    unlink_all(main)
    link(main, "1", "1", "2", "1")
    write(tree, name)


def probe_reloj() -> None:
    """A `Clock` of 100 and nothing else.

    The Clock of the original is a generator of period 100 whose transactions
    terminate with one unit of the `START`, so the run ends on the tick that
    empties the `START`.
    """
    tree = ET.parse(BENCHES / "Clock.gmdl")
    root = tree.getroot()
    set_prop(module_of(tree, "1"), "Frecuencia", "100")
    write(tree, "pReloj")


def probe_cantidad() -> None:
    """`GENERATE 100` with `Cantidad` 5 -> plain `TERMINATE`, plus `Clock 1000`.

    Three readings are possible and this probe tells them apart:
    the `START` could be the `Cantidad` of the generator (the run would end at
    500, when the fifth transaction terminates), the `START` could be the one
    of the simulator (the run would end at 1000, when the clock ticks), or
    the `Cantidad` could be the number of transactions of the generator and
    the run would end when the events run dry (1000 as well, but with a
    different pending-event table).
    """
    tree = ET.parse(BENCHES / "Terminate.gmdl")
    root = tree.getroot()
    modules = root.find("modules")
    main = module_of(tree, "0")
    generate = module_of(tree, "1")
    terminate = module_of(tree, "2")
    clock = module_of(tree, "3")

    set_prop(generate, "Media", "100")
    set_prop(generate, "Modificador", "0")
    set_prop(generate, "Cantidad", "5")
    set_prop(terminate, "Unidades", "")
    set_prop(clock, "Frecuencia", "1000")

    unlink_all(main)
    link(main, "1", "1", "2", "1")
    write(tree, "pCantidad")


def probe_seco() -> None:
    """`GENERATE 100` with `Cantidad` 5 -> plain `TERMINATE`, without a Clock.

    Nothing decrements the `START` here, so the run can only end when the
    events run dry: the five transactions are generated, they terminate, and
    nothing is left to do.  It is the case that tells the "the `START` ran
    out" end apart from the "there is nothing left to do" end.
    """
    tree = ET.parse(BENCHES / "Terminate.gmdl")
    root = tree.getroot()
    modules = root.find("modules")
    main = module_of(tree, "0")
    generate = module_of(tree, "1")
    terminate = module_of(tree, "2")
    clock = module_of(tree, "3")

    set_prop(generate, "Media", "100")
    set_prop(generate, "Modificador", "0")
    set_prop(generate, "Cantidad", "5")
    set_prop(terminate, "Unidades", "")
    modules.remove(clock)

    unlink_all(main)
    link(main, "1", "1", "2", "1")
    write(tree, "pSecco")


def probe_seco2() -> None:
    """`GENERATE 100` with `Cantidad` 5 -> plain `TERMINATE`, and a `TERMINATE 1`
    that nothing reaches (its input is not connected).

    The model is runnable (the original demands a TERMINATE with operand A),
    but the counter of terminations never moves: the five transactions
    terminate and nothing is left to do.  It is the case in which the run
    ends because the events run dry, not because the counter ran out.
    """
    tree = ET.parse(BENCHES / "Terminate.gmdl")
    root = tree.getroot()
    modules = root.find("modules")
    main = module_of(tree, "0")
    generate = module_of(tree, "1")
    terminate = module_of(tree, "2")
    clock = module_of(tree, "3")

    set_prop(generate, "Media", "100")
    set_prop(generate, "Modificador", "0")
    set_prop(generate, "Cantidad", "5")
    set_prop(terminate, "Unidades", "")
    modules.remove(clock)

    # a TERMINATE 1 that nothing reaches: its input is not connected
    orphan = copy.deepcopy(terminate)
    orphan.set("moduleid", "900")
    orphan.set("name", "Terminate1")
    orphan.set("x", "420")
    orphan.set("y", "140")
    orphan.find("inputs/input").set("mylabel", "A_Terminate1")
    set_prop(orphan, "Unidades", "1")
    modules.append(orphan)

    unlink_all(main)
    link(main, "1", "1", "2", "1")
    write(tree, "pSecco2")


def main() -> None:
    probe_terminate("1", "pTermina1")
    probe_terminate("3", "pTermina3")
    probe_reloj()
    probe_cantidad()
    probe_seco()
    probe_seco2()


if __name__ == "__main__":
    main()
```

### `oracle.sh` (corre el original con el `START` que se le pida)

```bash
#!/usr/bin/env bash
# The oracle of the START-semantics work unit (S4).
#
# It is the harness of the repository (scripts/wine-oracle/run_oracle.sh) plus
# the one thing this unit needs: the value of the "Start" field of the
# Simulador panel of the original, which is the START count of the run.
#
# usage: oracle.sh <model.gmdl> <start-value> <outdir> [--discover]
#
# The scaffolding lives outside the repository, in ~/genmsi-scratch/start.

set -uo pipefail

REPO="/home/lucas/Documentos/Workspace/Genmsi"
EXE="$REPO/Genmsi_20230711.backup/genmsi.exe"
export GENMSI_WINEPREFIX="$HOME/genmsi-scratch/start/wineprefix"
export WINEPREFIX="$GENMSI_WINEPREFIX"
MODEL="$(realpath "$1")"
NAME="$(basename "$MODEL")"
NAME="${NAME%.*}"
SAFE_NAME="$(echo "$NAME" | tr ' ' '_')"
START_VALUE="$2"
OUT="$3"
DISCOVER="${4:-}"
mkdir -p "$OUT"
ZPATH="Z:$(echo "$MODEL" | sed 's|/|\\\\|g')"
export DISPLAY="${XDISPLAY:-:95}"

pkill -f "[g]enmsi\.exe" 2>/dev/null || true
sleep 2
wineserver -k 2>/dev/null || true
sleep 1

if ! xdpyinfo -display "$DISPLAY" >/dev/null 2>&1; then
  nohup Xvfb "$DISPLAY" -screen 0 1280x800x24 -nolisten tcp > "$OUT/xvfb.log" 2>&1 &
  for _ in $(seq 1 20); do xdpyinfo -display "$DISPLAY" >/dev/null 2>&1 && break; sleep 0.5; done
fi

WID=""

app_window() {
  # the main window of the original: the one whose name starts with "Genmsi - "
  local w
  for w in $(xdotool search --name "." 2>/dev/null); do
    case "$(xdotool getwindowname "$w" 2>/dev/null)" in
      "Genmsi - "*) echo "$w"; return 0;;
    esac
  done
  return 1
}

click_in() {
  local wid
  wid="$(app_window)" || wid="$WID"
  WID="$wid"
  eval "$(xdotool getwindowgeometry --shell "$wid")"
  xdotool mousemove $((X + $1)) $((Y + $2)) click 1
  sleep "${3:-0.7}"
}
keys_in() {
  local wid
  wid="$(app_window)" || wid="$WID"
  WID="$wid"
  eval "$(xdotool getwindowgeometry --shell "$wid")"
  xdotool mousemove $((X + $1)) $((Y + $2)) click 1
  sleep 0.4
  shift 2
  for k in "$@"; do xdotool key --clearmodifiers "$k"; sleep 0.3; done
  sleep 0.5
}
extract_pane() {
  keys_in 456 294 ctrl+Home ctrl+shift+End ctrl+c
  python3 "$REPO/scripts/wine-oracle/read_clipboard.py" > "$1" 2>/dev/null
  if [ ! -s "$1" ]; then
    echo "error: clipboard copy produced an empty $1" >&2
    return 1
  fi
  echo "extracted $(wc -l < "$1") lines -> $1"
}

nohup wine "$EXE" "$ZPATH" > "$OUT/$SAFE_NAME.wine.log" 2>&1 &
for _ in $(seq 1 60); do
  WID="$(app_window || true)"
  [ -n "$WID" ] && break
  sleep 1
done
if [ -z "$WID" ]; then echo "error: no GenMSI window on $DISPLAY" >&2; exit 1; fi
echo "GenMSI window: $WID on $DISPLAY ($(xdotool getwindowname "$WID"))"
sleep 5

click_in 157 14    # "Modelo"
click_in 183 60    # "Simular"
sleep 1

# --- the Start field --------------------------------------------------------
# The field is the one that holds the START count of the run (it shows 1).
# Its position was found with the discovery mode of this script: the field is
# at (316, 156), the globe that runs the simulation at (364, 156).
# Careful: Ctrl+A is the accelerator of "Abrir" in the original, so the
# content of the field is erased with End and BackSpace.
click_in 316 156 0.5            # the "Start" field
xdotool key --clearmodifiers End
for _ in 1 2 3 4 5 6 7 8; do xdotool key --clearmodifiers BackSpace; done
sleep 0.3
xdotool type --delay 60 "$START_VALUE"
sleep 0.5
import -display "$DISPLAY" -window root "$OUT/$SAFE_NAME.startfield.png" 2>/dev/null || true

if [ "$DISCOVER" = "--discover" ]; then
  echo "discovery: screenshot $OUT/$SAFE_NAME.startfield.png"
  pkill -f "[g]enmsi\.exe" 2>/dev/null || true
  wineserver -k 2>/dev/null || true
  exit 0
fi

# --- run --------------------------------------------------------------------
click_in 364 156 1.5            # the globe: run
sleep 2

extract_pane "$OUT/$SAFE_NAME.report.txt"
click_in 256 181                # the "Código GPSS" tab
extract_pane "$OUT/$SAFE_NAME.gpss.txt"

wineserver -k 2>/dev/null || true
echo "done: $OUT/$SAFE_NAME.{report,gpss}.txt"
```

### `run-batch.sh` (la tanda de corridas)

```bash
#!/usr/bin/env bash
# The batch of the START unit: the same probes with different START counts.
# usage: run-batch.sh
set -uo pipefail
HERE="$HOME/genmsi-scratch/start"
OUT="$HERE/oracle"
mkdir -p "$OUT"

run() {  # run <model> <start>
  local model="$1" start="$2"
  local tag="$(basename "$model" .gmdl)-s$start"
  echo "=================== $tag"
  XDISPLAY=:95 timeout 240 bash "$HERE/oracle.sh" "$HERE/probes/$model" "$start" "$OUT/$tag" 2>&1 | grep -v "^  " | tail -3
  if [ -f "$OUT/$tag/$(basename "$model" .gmdl).report.txt" ]; then
    sed -n '1,12p' "$OUT/$tag/$(basename "$model" .gmdl).report.txt"
  else
    echo "(no report)"
  fi
}

run pTermina1.gmdl 1
run pTermina1.gmdl 5
run pTermina3.gmdl 1
run pTermina3.gmdl 5
run pTermina3.gmdl 10
run pReloj.gmdl 1
run pReloj.gmdl 3
run pCantidad.gmdl 1
run pCantidad.gmdl 5
run pSecco.gmdl 1
run pSecco2.gmdl 1
```
