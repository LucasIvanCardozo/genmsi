# El orden de los eventos del mismo instante: qué gana un `SEIZE`, quién sale preemptado y el FIFO de la cola

Rama `main`. **Esta unidad midió: no tocó el motor.** Lo que hay aquí son
sondas, medidas contra el original y el veredicto con la regla (o con el
motivo de por qué no se puede decidir). El ejemplo
`crates/genmsi-core/examples/simultaneos.rs` reproduce la comparación.

El andamiaje de las sondas vive **fuera del repositorio** (`/tmp/simultaneos-probes`),
como el de `odd/tasks/notas-reloj.md`, y el script que las escribe está en el
apéndice de esta nota, para que cualquiera las reconstruya byte a byte.

## Las preguntas (textuales)

1. **Quién gana un `SEIZE`**: dos transacciones que llegan en el mismo
   instante a una facilidad ocupada — ¿la que estaba esperando, la que llega
   nueva, o la de mayor prioridad? Y con dos que llegan nuevas a la vez,
   ¿cuál entra primero?
2. **A quién preempta**: dos transacciones de distinta prioridad que compiten
   por el mismo recurso en el mismo instante — ¿a cuál se le quita?
3. **El FIFO de la cola**: el orden en que salen las transacciones de una cola
   cuando se vacía en un mismo instante.

Y, de paso, lo que la medida aparezca y no esté en esta lista (no se acota:
el trabajo es decir cuál es la regla).

## Veredicto, en una línea

**El original procesa los eventos de un mismo instante por prioridad
descendente y, entre los de la misma prioridad, al revés (el último
programado se procesa primero); la cadena de demora de una facilidad se
sirve por prioridad descendente y, a igualdad de prioridad, por orden de
llegada (FIFO).**

## Las sondas

Cada sonda es un modelo que el original abre y corre. Todas tienen la misma
forma: cada candidato escribe el reloj en **su propio parámetro** con un
`MARK` y lo tabula en **su propia tabla** (`TABULATE`, tabla de clase 1) con
un `TABULATE`, de modo que el histograma de la tabla es el instante en que
ese candidato tomó la facilidad:

```
GENERATE (intervalo fijo)  ->  SEIZE Recurso  ->  MARK ParamX
                           ->  TABULATE TablaX ->  ADVANCE (servicio)
                           ->  RELEASE Recurso ->  TERMINATE
```

La tabla se lee en el reporte del original: la clase `100 - 100` con
frecuencia 1 quiere decir que el candidato tomó la facilidad en el instante
100 (la `MEDIA` de la tabla lo confirma: `100,000`). La anchura de la clase
es 1, así que la clase *es* el valor.

| Sonda | Qué hace | Por qué discrimina una regla de otra |
|---|---|---|
| `dosnuevos` | Dos generadores de intervalo fijo 100: las dos transacciones nacen en t=100 y las dos piden la misma facilidad, que está libre. | Si la regla fuera "gana la primera del modelo" o "gana la primera que pidió", el resultado cambia al invertir las cadenas: por eso hay `dosnuevosInvertido`. |
| `dosnuevosInvertido` | Lo mismo, con las dos cadenas al revés en el modelo. | Separa "la que va primero en el modelo" de "la que va último en el modelo". |
| `esperando` | El dueño toma la facilidad en t=10 y la suelta en t=110; el que espera llega en t=50 (la encuentra ocupada) y el que llega nuevo llega en t=110, el mismo instante de la liberación. | Separa "gana el que ya esperaba" de "gana el que llega nuevo". |
| `prioridadEspera` | Lo mismo, con el que llega nuevo de prioridad 10 y el que espera de prioridad 0. | Separa "gana el que ya esperaba" de "gana el de mayor prioridad". |
| `fifo` | Dueño de t=10 a t=110; tres transacciones esperan (t=20, t=30, t=40), todas de prioridad 0, con servicios distintos (100, 10, 10). | El servicio distinto hace que el *orden* de salida se vea en los instantes: si el orden fuera LIFO o por prioridad, los números cambiarían. |
| `prioridadesCola` | Igual, con las tres de distinta prioridad (0 en t=20, 10 en t=30, 5 en t=40). | Separa "la cola es FIFO" de "la cola se sirve por prioridad". |
| `avances` | Dueño de t=10 a t=90; dos transacciones reanudan un avance en t=100 (una lo pidió en t=20, la otra en t=30) y compiten por la facilidad, que ya está libre. | Separa dos lecturas de `dosnuevos`: que los generadores se primesen al revés, o que los eventos de un instante se procesen al revés. Aquí las dos reanudaciones se programaron muy lejos del primado, y aun así la última va primero. |
| `preempta` | Dueño de prioridad 1 (de t=10 a t=1010); en t=110 llegan dos transacciones, una de prioridad 5 y otra de prioridad 10, y las dos preemptan. Cada `PREEMPT` tiene **su propia salida alterna**, así que se sabe cuál de las dos desplazó al dueño. | Separa "desplaza la de mayor prioridad" de "desplaza la que va primero en el modelo", y "desplaza una vez" de "desplaza dos veces". |
| `preemptaInvertida` | Lo mismo, con las dos cadenas al revés en el modelo. | Como `dosnuevosInvertido`: separa la prioridad del orden del modelo. |
| `preemptaTarde` | La de prioridad 5 preempta en t=110 y la de prioridad 10 en t=120. | Control: con instantes distintos el original sí desplaza a la segunda, así que lo de `preempta` no es que no pueda desplazar dos veces, sino que en el mismo instante no llega a hacerlo. |

Las sondas son de un solo paso (una facilidad, una cola de demora, una
preempción): no necesitan declaraciones de almacenes ni de tablas de
entidades, y el original las corre sin quejarse.

## La medida

### Cómo se lee

- **El instante de la toma**: el histograma de la tabla del candidato
  (`100 - 100`, frecuencia 1 → el valor 100). Es un número entero, que es lo
  que cabe en la clase de anchura 1.
- **Quién no pudo entrar**: la fila `ENTRY COUNT` del bloque `PREEMPT` (si
  la transacción fue *denegada la entrada*, el bloque no se cuenta) y la
  columna `DELAY` de la fila de la `FACILITY` (cuántas esperan en la cadena
  de demora).
- **Quién quedó sin facilidad**: la columna `OWNER` de la `FACILITY` y el
  parámetro `REMANENTE` que aparece en la sección `FEC_XACT_NUMBER` (el
  tiempo que le quedaba al desplazado).

### Los números del original

| Sonda | Lectura (el instante en que tomó la facilidad) | Lo que además dice el reporte |
|---|---|---|
| `dosnuevos` | `TablaDos` = **100**, `TablaUno` = 150 | `FACILITY RECURSO`: 2 entradas, `AVE_TIME 50,000` |
| `dosnuevosInvertido` | `TablaUno` = **100**, `TablaDos` = 150 | ídem |
| `esperando` | `TablaEspera` = **110**, `TablaNuevo` = 120 | `FACILITY`: 3 entradas, `AVE_TIME 40,000` |
| `prioridadEspera` | `TablaNuevo` = **110**, `TablaEspera` = 120 | ídem |
| `fifo` | `TablaUno` = **110**, `TablaDos` = **210**, `TablaTres` = **220** | `FACILITY`: 4 entradas, `AVE_TIME 55,000` |
| `prioridadesCola` | `TablaDos` = **110**, `TablaTres` = **120**, `TablaUno` = **130** | `FACILITY`: 4 entradas, `AVE_TIME 55,000` |
| `avances` | `TablaDos` = **100**, `TablaUno` = 110 | `FACILITY`: 3 entradas, `AVE_TIME 33,333` |
| `preempta` | `TablaDiez` = **110**, `TablaCinco` = *(nada)* | `A_PreemptaCinco`: `ENTRY COUNT 0` (nunca entró), `A_PreemptaDiez`: `ENTRY COUNT 1`; `FACILITY ... DELAY 1` (queda uno esperando), `OWNER 1` (el dueño) |
| `preemptaInvertida` | `TablaDiez` = **110**, `TablaCinco` = *(nada)* | ídem |
| `preemptaTarde` | `TablaCinco` = **110**, `TablaDiez` = **120** | `FACILITY`: 3 entradas, `OWNER 2`, `INTER 1` |

### Los números de este motor, sobre las mismas sondas

| Sonda | El original | Este motor | ¿Igual? |
|---|---|---|---|
| `dosnuevos` | Uno 150, Dos 100 | Uno **100**, Dos 150 | **no** |
| `dosnuevosInvertido` | Uno 100, Dos 150 | Uno **150**, Dos 100 | **no** |
| `esperando` | Espera 110, Nuevo 120 | Espera 110, Nuevo 120 | sí |
| `prioridadEspera` | Espera 120, Nuevo 110 | Espera 120, Nuevo 110 | sí |
| `fifo` | 110 / 210 / 220 | 110 / 210 / 220 | sí |
| `prioridadesCola` | 130 / 110 / 120 | 130 / 110 / 120 | sí |
| `avances` | Uno 110, Dos 100 | Uno **100**, Dos 110 | **no** |
| `preempta` | Diez 110, Cinco *(nada)* | Diez 110, Cinco **110** | **no** |
| `preemptaInvertida` | Diez 110, Cinco *(nada)* | Diez 110, Cinco *(nada)* | sí |
| `preemptaTarde` | 110 / 120 | 110 / 120 | sí |

Las tres diferencias vienen de **la misma cosa**: el desempate de los
eventos de un mismo instante y de la misma prioridad. Este motor los
procesa en el orden en que se programaron (FIFO); el original, al revés.

## La regla

### 1. El orden dentro de un instante (la CEC del original)

- **Por prioridad, de mayor a menor, y la prioridad le gana al orden de
  programación.** En `preempta` y en `preemptaInvertida` desplaza al dueño
  la de prioridad 10, aunque en el primer caso sea la segunda del modelo y
  en el segundo la primera: la prioridad manda.
- **Entre los de la misma prioridad, al revés**: el último programado se
  procesa primero.
  - `dosnuevos`: gana la segunda cadena del modelo (la programada después).
  - `dosnuevosInvertido`: gana la segunda cadena del modelo (que ahora es
    la otra). La regla no es "la primera del modelo": es "la última
    programada".
  - `avances`: las dos reanudaciones se programaron en t=20 y en t=30, muy
    lejos del primado de los generadores, y aun así la de t=30 (la última
    programada) va primero. Esto descarta la lectura "los generadores se
    primesen al revés": el desempate está en la cola de eventos, no en el
    primado.
- **La llegada pendiente de un generador lleva la prioridad de su
  transacción.** En `preemptaInvertida` la de prioridad 10 (que es la
  primera del modelo, y por lo tanto la primera programada) desplaza al
  dueño, y la de prioridad 5 (programada después) queda esperando: si el
  orden de un instante fuera sólo "el último programado primero", la de
  prioridad 5 se llevaría la facilidad.

### 2. Quién gana un `SEIZE`

- **El que ya esperaba** le gana al que llega nuevo en el mismo instante de
  la liberación, si tienen la misma prioridad (`esperando`: el que esperaba
  tomó la facilidad en 110, el nuevo en 120).
- **El de mayor prioridad le gana al que ya esperaba** (`prioridadEspera`:
  el nuevo, de prioridad 10, tomó la facilidad en 110; el que esperaba, de
  prioridad 0, en 120).
- **Entre los que llegan nuevos a la vez**, gana el que se procesa primero,
  que es el último programado (`dosnuevos` y `dosnuevosInvertido`).

O sea: la respuesta a la pregunta 1 es *la de mayor prioridad* y, a igualdad
de prioridad, *la que se procesa primero*; y la que se procesa primero, a
igualdad de prioridad, es la última programada, no la primera.

### 3. A quién preempta

- En el mismo instante, **preempta la de mayor prioridad** y la otra **no
  displace a nadie**: en `preempta` y en `preemptaInvertida` hay **un solo
  desplazamiento** (el del dueño, en t=110), el `PREEMPT` de la de
  prioridad 5 tiene `ENTRY COUNT 0` (nunca entró: le negaron la entrada) y
  la `FACILITY` termina con `DELAY 1` (queda esperando).
- En instantes distintos, sí: `preemptaTarde` (la de prioridad 5 preempata
  en t=110 y la de prioridad 10 la desplaza en t=120).

La diferencia entre los dos casos no es la regla del `PREEMPT`, es el orden
dentro del instante: la de prioridad 10 se procesa primero, toma la
facilidad, y la de prioridad 5 llega después y se queda esperando. En la
sonda `preempta` (donde la de prioridad 5 es la primera del modelo) este
motor, que procesa en el orden de programación, llega a la situación
opuesta: la de prioridad 5 desplaza al dueño y la de prioridad 10 desplaza
a la de prioridad 5 (dos desplazamientos, `TablaCinco` = 110).

### 4. El FIFO de la cola

La pregunta 3 dice "el FIFO de la cola", y la respuesta tiene dos partes:

- **La cola de demora de una facilidad no es FIFO puro: se sirve por
  prioridad.** En `prioridadesCola` (prioridades 0, 10 y 5, que llegaron en
  t=20, t=30 y t=40) el orden de salida fue **10 (t=110), 5 (t=120),
  0 (t=130)**: el que llegó primero fue el último en servir.
- **Entre los de la misma prioridad sí es FIFO**: en `fifo` (los tres de
  prioridad 0) el orden fue el de llegada (t=110, t=210, t=220).

### 5. Lo que apareceu y no estaba en la lista

1. **La cadena de demora es por prioridad, no FIFO.** Es lo contrario de lo
   que sugiere la palabra "cola", y es lo que hace que `prioridadEspera` le
   gane al que ya esperaba.
2. **Un desplazado no puede terminar mientras siga siendo propietario de la
   facilidad.** La primera sonda de preempción murió con el error del
   original: `ERROR!!Una transacción propietaria de una facility está
   intentando terminar.` (con `Tiempo de simulación: 161`, `Transacción
   activa: 1`). El manual lo dice: "Una transacción interrumpida, que
   todavía está en condiciones de competir por una facility, no puede
   ingresar en un bloque TERMINATE o ASSEMBLE". Por eso las sondas de
   preempción terminan la cadena del desplazado con un `ADVANCE` que la
   corrida no alcanza.
3. **Cuando el preemptor suelta la facilidad, el dueño interrumpido la
   vuelve a tomar, antes que la cadena de demora.** En `preempta` el tiempo
   de ocupación de `Recurso` es 3590 unidades de 3600 (`AVE_TIME 1795,000`
   con 2 entradas): el dueño la tenía de t=10 a t=110, el preemptor de
   t=110 a t=160, y el dueño ** otra vez** de t=160 hasta el final. No hay
   hueco.
4. **Las estadísticas de cola del original (`QUEUE`/`DEPART`) no miden la
   espera por la facilidad.** En la primera versión de las sondas (con el
   sándwich `QUEUE`/`DEPART` alrededor del `SEIZE`) el `AVE_TIME` de la
   cola dio `0,000` con una transacción que había esperado 60 unidades: la
   espera por la facilidad no pasa por las estadísticas de la cola. Medir
   con colas no sirve; hay que medir con el reloj.
5. **El original no abre un modelo cuya cola se llame `Q1`.** `Q1` es la
   forma corta de "cola número 1", y el original responde `El modelo no
   puede abrirse porque el archivo presenta errores estructurales.` Las
   sondas usan `QUno`, `QDos`, `QTres`.
6. El parámetro que el `PREEMPT` guarda (`REMANENTE`) trae el tiempo que le
   quedaba al desplazado: 900 en `preempta` (el dueño iba de t=10 a t=1010
   y lo desplazaron en t=110) y 40 en `preemptaTarde` (la de prioridad 5 iba
   de t=110 a t=160 y la desplazaron en t=120). Sirve para confirmar *a
   quién* se le quitó la facilidad.

## Lo que un reloj entero necesitaría de esta regla

El reloj entero (S5) multiplica los empates: en el modelo grande son 506
eventos los que comparten un mismo instante (15,9 eventos por unidad
entera de media). Hoy nuestro motor separa esos eventos con una fracción
—un orden arbitrario— y los resultados difieren del original en **cuatro de
las diez sondas**.

Lo que hay que aplicar, con el archivo y la función:

| Dónde | Qué | Qué hay que cambiar |
|---|---|---|
| `crates/genmsi-core/src/sim.rs`, `struct Event` (campo `seq`) y `impl Ord for Event` (línea ~292) | la comparación es `(time, priority, seq)`: tiempo, luego prioridad de mayor a menor, y a igualdad el **número de secuencia** (el orden de programación) | el desempate tiene que ser **al revés**: entre los eventos del mismo instante y de la misma prioridad, gana el último programado |
| `crates/genmsi-core/src/sim.rs`, `Simulation::push_event` (línea ~506) | el número de secuencia; hoy se **niega** para los eventos de la hora en que termina la corrida ("el original procesa esos al revés") | con la regla general, ese caso especial deja de hacer falta (o se invierte la comparación en vez de negar el número) |
| `crates/genmsi-core/src/sim.rs`, `Simulation::install_definitions` (línea ~417) | el primado de los generadores: la primera llegada de cada uno, y **el `tick` se programa con prioridad 0** | el evento de la llegada pendiente tiene que llevar **la prioridad de la transacción** (la del generador), no 0: si no, una llegada pendiente de prioridad 0 compite contra el evento de otra transacción y se cuela por delante |
| `crates/genmsi-core/src/sim.rs`, `Simulation::release_waiters` (línea ~2257) y el brazo `BlockKind::Seize` (línea ~1055) | la cadena de demora de la facilidad: se ordena por `(prioridad de mayor a menor, orden de llegada)` | **ya coincide con el original** (las sondas `fifo` y `prioridadesCola` dan los mismos números en los dos motores) |
| `crates/genmsi-core/src/sim.rs`, el brazo `BlockKind::Preempt` (línea ~1134) | la preempción en modo prioridad: se puede desplazar al dueño si la prioridad del preemptor es mayor | **la regla no cambia**: la diferencia de `preempta` viene del orden dentro del instante, no de la comparación de prioridades |

(Las líneas son las de hoy; la unidad del reloj va a tocar ese archivo, así
que conviene leer las funciones por su nombre.)

### La prueba del camino, en una copia de trabajo

Para no dejar la regla en el papel, se aplicaron los dos cambios en una
**copia del código fuera del repositorio** (`/tmp`, nunca en el árbol de
trabajo) y se corrieron las diez sondas contra los mismos reportes del
original:

| Cambio | Sondas que quedan como el original |
|---|---|
| Ninguno (el motor de hoy) | 6 de 10 |
| Sólo el desempate al revés (`seq` descendente) | 8 de 10: arregla `dosnuevos`, `dosnuevosInvertido`, `avances` y `preempta`, pero **rompe** `prioridadEspera` (el que llega nuevo de prioridad 10 deja de ganar) y `preemptaInvertida` (el de prioridad 5 desplaza al dueño) |
| El desempate al revés **y** la llegada pendiente del generador con la prioridad de la transacción | **10 de 10** |

O sea: los dos cambios son necesarios y juntos, y el segundo (la prioridad
de la llegada pendiente) es el que la unidad del reloj no se habría
esperado: sin él, el motor mete en la cola de eventos una llegada de
prioridad 0 (la del `tick` del generador) que compite con los eventos de
las transacciones y se adelanta a las de mayor prioridad.

La copia se borró al terminar: no queda nada del experimento en el
repositorio, y el motor del árbol de trabajo sigue como estaba.

O sea: **dos cambios** —el desempate de la cola de eventos y la prioridad
de la llegada pendiente de un generador— ponen las diez sondas de acuerdo
con el original (probado en una copia de trabajo, más abajo). Con el reloj
entero, ese desempate pasa a ser el que decide en los 506 empates del modelo
grande, así que la unidad del reloj (S5) lo necesita antes de cambiar la
granularidad.

## Qué NO se midió (y por qué)

- **El orden dentro de un instante entre eventos de distinta prioridad que
  no compiten por el mismo recurso**: la regla general ("prioridad de mayor
  a menor y, a igualdad, al revés") sale de la CEC del original ("la
  transacción con mayor prioridad en la CEC será la transacción activa") y
  de las sondas que sí compiten (`preempta`, `prioridadEspera`); no se midió
  un caso en el que la prioridad separe dos eventos que no compiten por la
  misma facilidad.
- **La cadena de demora de un `storage`**: es otra entidad (tiene su propia
  disciplina de espera) y esta unidad midió la de las `facility`, que es la
  que usan las sondas de `SEIZE` y `PREEMPT`.
- **Las cadenas de usuario (`LINK`/`UNLINK`)**: tienen su propio modo
  (`FIFO`, `LIFO`, por parámetro) y no intervienen en estas tres preguntas.

## Cómo reproducir

```bash
# 1. las sondas (el andamiaje vive fuera del repositorio)
python3 /tmp/simultaneos-probes/make-probes.py

# 2. el original, una corrida por vez y con su propio display
#    (el :99 está compartido con otras sesiones y las corridas se pisan)
for v in dosnuevos dosnuevosInvertido esperando prioridadEspera fifo \
         prioridadesCola avances preempta preemptaInvertida preemptaTarde; do
  pkill -f "[g]enmsi.exe"; sleep 1
  XDISPLAY=:97 bash scripts/wine-oracle/run_oracle.sh \
      /tmp/simultaneos-probes/$v.gmdl /tmp/simultaneos-probes/oracle
done

# 3. la comparación
cargo run --release -p genmsi-core --example simultaneos
```

El ejemplo lee los reportes del original de `/tmp/simultaneos-probes/oracle` y
corre las mismas sondas con este motor; imprime, sonda por sonda, los
instantes que tomó cada candidato en los dos motores, el veredicto y el
sitio del motor donde se aplicaría la regla. Es reproducible: dos corridas
seguidas dan lo mismo (el motor es determinista con la semilla que se le
pasa, `--seed 7` por omisión).

## Apéndice: el script que construye las sondas

```python
#!/usr/bin/env python3
"""Build the probes of the simultaneous-order work unit.

The probes are measurement scaffolding: they live outside the repository and
this script is the recipe, so anybody can rebuild them byte for byte.

The modules come from the benches the repository already generates
(`scripts/wine-oracle/benches`), which the original opens without complaint;
only the chains are written by hand.

Every candidate writes the clock into its own parameter with a `MARK` and
tabulates it into its own table with a `TABULATE`: the histogram of the
table is the moment in which that candidate took the facility.
"""

from __future__ import annotations

import copy
import xml.etree.ElementTree as ET
from pathlib import Path

REPO = Path("/home/lucas/Documentos/Workspace/Genmsi")
BENCHES = REPO / "scripts/wine-oracle/benches"
OUT = Path("/tmp/simultaneos-probes")

# The classes the probes use, and the bench whose module is the template of
# each one.  The template carries the properties of the class with the shape
# the original itself writes (the type of every property, its hint, its
# default), which is what the original needs to open the model without
# adapting it.
TEMPLATE_BENCH = {
    "Generate": "Generate",
    "Queue": "Queue",
    "Depart": "Depart",
    "Seize": "Seize",
    "Release": "Release",
    "Preempt": "Preempt",
    "Return": "Return",
    "Advance": "Advance",
    "Priority": "Priority",
    "Mark": "Mark",
    "Table": "Table",
    "Tabulate": "Tabulate",
    "Terminate": "Terminate",
    "Clock": "Clock",
}


def template_of(class_name: str) -> ET.Element:
    """The module of the bench that is the template of a class."""
    path = BENCHES / f"{TEMPLATE_BENCH[class_name]}.gmdl"
    tree = ET.parse(path)
    for module in tree.getroot().iter("module"):
        if module.get("moduleclass") == class_name:
            return copy.deepcopy(module)
    raise KeyError(class_name)


def set_prop(module: ET.Element, name: str, value: str) -> None:
    """Set the value of a property of a module."""
    for element in module.findall("props/property"):
        if element.get("name") == name:
            element.set("value", value)
            return
    raise KeyError(name)


def prop_value(module: ET.Element, name: str) -> str:
    for element in module.findall("props/property"):
        if element.get("name") == name:
            return element.get("value") or ""
    raise KeyError(name)


class Model:
    """A model of the original: its modules and its links."""

    def __init__(self, title: str) -> None:
        self.title = title
        self.modules: list[ET.Element] = []
        self.links: list[tuple[str, str, str, str]] = []
        self.labels: dict[str, str] = {}

    def add(
        self,
        class_name: str,
        name: str,
        props: dict[str, str] | None = None,
    ) -> ET.Element:
        """Add a block of a class to the model and return it."""
        module = template_of(class_name)
        module.set("moduleid", str(len(self.modules) + 1))
        module.set("name", name)
        module.set("caption", name)
        module.set("x", str(120 + 100 * (len(self.modules) % 4)))
        module.set("y", str(40 + 100 * (len(self.modules) // 4)))
        for prop_name, value in (props or {}).items():
            set_prop(module, prop_name, value)
        if class_name == "Table":
            # The table block is written in the GPSS with the name of the
            # table, not with the name of the module.
            self.labels[name] = prop_value(module, "NombreTabla")
        elif class_name == "Clock":
            self.labels[name] = ""
        else:
            self.labels[name] = f"A_{name}"
        self.modules.append(module)
        return module

    def table(self, name: str, parameter: str) -> ET.Element:
        """The table that collects the readings of a parameter."""
        return self.add(
            "Table",
            name,
            {
                # `P:Parametro` is the way the original writes it: the colon
                # is percent-encoded in the model file.
                "Atributo": f"P%3A{parameter}",
                "NombreTabla": name,
                "PrimerIntervalo": "0",
                "TamFrecuencia": "1",
                "Cantidad": "400",
            },
        )

    def connect(self, source: ET.Element, target: ET.Element) -> None:
        """Connect the main output of a block to the input of another one."""
        outputs = source.findall("outputs/output")
        inputs = target.findall("inputs/input")
        source_output = next(
            output for output in outputs if output.get("name") == "Salida1"
        )
        target_input = inputs[0]
        source_output.set("mylabel", self.labels[target.get("name")])
        target_input.set("mylabel", self.labels[target.get("name")])
        self.links.append(
            (
                source.get("moduleid"),
                source_output.get("ioid"),
                target.get("moduleid"),
                target_input.get("ioid"),
            )
        )

    def connect_alt(self, source: ET.Element, target: ET.Element) -> None:
        """Connect the alternate output of a block (the displaced owner)."""
        outputs = source.findall("outputs/output")
        source_output = next(
            output for output in outputs if output.get("name") == "SalidaAlt"
        )
        target_input = target.findall("inputs/input")[0]
        source_output.set("mylabel", self.labels[target.get("name")])
        target_input.set("mylabel", self.labels[target.get("name")])
        self.links.append(
            (
                source.get("moduleid"),
                source_output.get("ioid"),
                target.get("moduleid"),
                target_input.get("ioid"),
            )
        )

    def write(self, name: str) -> None:
        """Write the model as a `.gmdl` file."""
        OUT.mkdir(parents=True, exist_ok=True)
        root = ET.Element("gmodel", {"versioncode": "1.00.2008.03.23"})
        ET.SubElement(root, "gclasses")
        modules = ET.SubElement(root, "modules")
        main = ET.SubElement(
            modules,
            "module",
            {
                "moduleid": "0",
                "moduleclass": "main",
                "owner": "0",
                "caption": "Modelo",
                "name": "Modelo1",
                "kind": "C",
                "genprefer": "1,1,1,1,1,0,0,0,0,0,0,0,0",
                "exehead": "",
                "nonexehead": "",
                "exefoot": "",
                "nonexefoot": "",
            },
        )
        links = ET.SubElement(main, "links")
        for from_module, from_ioid, to_module, to_oid in self.links:
            ET.SubElement(
                links,
                "link",
                {
                    "frommoduleid": from_module,
                    "fromioid": from_ioid,
                    "tomoduleid": to_module,
                    "toioid": to_oid,
                },
            )
        ET.SubElement(main, "inputs")
        ET.SubElement(main, "outputs")
        for module in self.modules:
            modules.append(module)
        tree = ET.ElementTree(root)
        tree.write(
            OUT / f"{name}.gmdl",
            encoding="latin-1",
            xml_declaration=False,
            short_empty_elements=True,
        )
        print(name, (OUT / f"{name}.gmdl").stat().st_size, "bytes")


def generate(model: Model, name: str, media: str, priority: str = "") -> ET.Element:
    """A generator of one transaction, with a fixed interval."""
    return model.add(
        "Generate",
        name,
        {
            "Media": media,
            "Modificador": "",
            "OffSet": "",
            "Cantidad": "1",
            "Prioridad": priority,
        },
    )


def candidate(
    model: Model,
    media: str,
    service: str,
    priority: str = "",
    prefix: str = "",
    before: str = "",
) -> ET.Element:
    """A candidate: it arrives, takes the facility and says when it took it.

    The chain is `GENERATE -> [ADVANCE] -> SEIZE -> MARK -> TABULATE -> ADVANCE
    -> RELEASE -> TERMINATE`: the `MARK` writes the clock into the parameter
    of the candidate and the `TABULATE` collects it in the table of the
    candidate, so the histogram of the table says the moment in which that
    candidate took the facility.
    """
    parameter = f"Param{prefix}"
    table = f"Tabla{prefix}"
    model.table(table, parameter)
    gen = generate(model, f"Genera{prefix}", media, priority=priority)
    seize = model.add("Seize", f"Toma{prefix}", {"Facility": "Recurso"})
    mark = model.add("Mark", f"Marca{prefix}", {"Parametro": parameter})
    tabulate = model.add("Tabulate", f"Tabula{prefix}", {"NombreTabla": table})
    advance = model.add(
        "Advance", f"Avanza{prefix}", {"Media": service, "Modificador": ""}
    )
    release = model.add("Release", f"Libera{prefix}", {"Facility": "Recurso"})
    terminate = model.add("Terminate", f"Termina{prefix}")
    if before:
        wait = model.add(
            "Advance", f"Espera{prefix}", {"Media": before, "Modificador": ""}
        )
        model.connect(gen, wait)
        model.connect(wait, seize)
    else:
        model.connect(gen, seize)
    model.connect(seize, mark)
    model.connect(mark, tabulate)
    model.connect(tabulate, advance)
    model.connect(advance, release)
    model.connect(release, terminate)
    return gen


def owner(
    model: Model,
    media: str,
    service: str,
    priority: str = "",
    prefix: str = "Dueno",
) -> ET.Element:
    """The transaction that holds the facility and releases it."""
    gen = generate(model, f"Genera{prefix}", media, priority=priority)
    seize = model.add("Seize", f"Toma{prefix}", {"Facility": "Recurso"})
    advance = model.add(
        "Advance", f"Avanza{prefix}", {"Media": service, "Modificador": ""}
    )
    release = model.add("Release", f"Libera{prefix}", {"Facility": "Recurso"})
    terminate = model.add("Terminate", f"Termina{prefix}")
    model.connect(gen, seize)
    model.connect(seize, advance)
    model.connect(advance, release)
    model.connect(release, terminate)
    return gen


def probe_dosnuevos(invertido: bool = False) -> None:
    """Two transactions that arrive in the same instant to a free facility.

    Both generators have a fixed interval of 100, so both transactions are
    born at t=100 and both try to take the same facility at the same
    instant.  The table of each candidate says the moment in which it took
    the facility: the one that took it at t=100 is the one that arrived
    first.

    The `invertido` variant writes the two chains the other way round: it
    says whether the winner follows the order of the model.
    """
    name = "dosnuevosInvertido" if invertido else "dosnuevos"
    model = Model(name)
    if invertido:
        candidate(model, "100", "50", prefix="Dos")
        candidate(model, "100", "50", prefix="Uno")
    else:
        candidate(model, "100", "50", prefix="Uno")
        candidate(model, "100", "50", prefix="Dos")
    model.add("Clock", "Reloj", {"Frecuencia": "3600"})
    model.write(name)


def probe_esperando() -> None:
    """The transaction that was waiting against the one that arrives new.

    The owner takes the facility at t=10 and releases it at t=110.  The
    waiter arrives at t=50 (the facility is busy: it waits).  The newcomer
    arrives at t=110, the very instant the owner releases the facility.
    """
    model = Model("esperando")
    owner(model, "10", "100")
    candidate(model, "50", "10", prefix="Espera")
    candidate(model, "110", "10", prefix="Nuevo")
    model.add("Clock", "Reloj", {"Frecuencia": "3600"})
    model.write("esperando")


def probe_prioridad_espera() -> None:
    """The same as `esperando`, with a newcomer of higher priority.

    The newcomer arrives at t=110 with priority 10; the waiter has the
    default priority 0.  The question is whether the chain of delay of the
    facility is served by priority (the newcomer wins) or in the order in
    which the transactions arrived (the waiter wins).
    """
    model = Model("prioridadEspera")
    owner(model, "10", "100")
    candidate(model, "50", "10", prefix="Espera")
    candidate(model, "110", "10", priority="10", prefix="Nuevo")
    model.add("Clock", "Reloj", {"Frecuencia": "3600"})
    model.write("prioridadEspera")


def probe_fifo() -> None:
    """Three transactions waiting for the same facility: in what order?

    The owner holds the facility from t=10 to t=110.  Three transactions
    arrive at t=20, t=30 and t=40 and wait.  The service time of each one
    is different, so the moment in which each one takes the facility says
    in what order the chain of delay was served.
    """
    model = Model("fifo")
    owner(model, "10", "100")
    candidate(model, "20", "100", prefix="Uno")
    candidate(model, "30", "10", prefix="Dos")
    candidate(model, "40", "10", prefix="Tres")
    model.add("Clock", "Reloj", {"Frecuencia": "3600"})
    model.write("fifo")


def probe_prioridades_cola() -> None:
    """Three transactions waiting, with three different priorities.

    The owner holds the facility from t=10 to t=110.  Three transactions
    arrive at t=20 (priority 0), t=30 (priority 10) and t=40 (priority 5)
    and wait.  The moment in which each one takes the facility says whether
    the chain of delay is served by priority or in the order of arrival.
    """
    model = Model("prioridadesCola")
    owner(model, "10", "100")
    candidate(model, "20", "100", priority="0", prefix="Uno")
    candidate(model, "30", "10", priority="10", prefix="Dos")
    candidate(model, "40", "10", priority="5", prefix="Tres")
    model.add("Clock", "Reloj", {"Frecuencia": "3600"})
    model.write("prioridadesCola")


def probe_preempta(invertido: bool = False, tarde: bool = False) -> None:
    """Two transactions of different priority that preempt at the same instant.

    The owner takes the facility at t=10 with priority 1 and holds it for
    1000 units.  Two transactions try to preempt the facility: one with
    priority 5 and one with priority 10.  Each PREEMPT block has its own
    alternate output, so the table of each chain says which of the two
    displaced the owner, and when.

    Three variants:
      - `preempta`: the two arrive at the same instant (t=110), the one of
        priority 5 first in the model;
      - `preemptaInvertida`: the same, with the one of priority 10 first in
        the model;
      - `preemptaTarde`: the one of priority 10 arrives ten units later
        (t=120), so the order of the instants is not in question.

    The displaced transaction cannot terminate while it still holds the
    facility ("Una transaccion propietaria de una facility esta intentando
    terminar"), so its chain ends in an advance that the run never reaches.
    """
    name = {
        (False, False): "preempta",
        (True, False): "preemptaInvertida",
        (False, True): "preemptaTarde",
    }[(invertido, tarde)]
    model = Model(name)
    owner(model, "10", "1000", priority="1")

    # The chain named `Cinco` always has priority 5 and the one named `Diez`
    # always priority 10; the variants change the order in which they are
    # written in the model, and the instant of the second one.
    chains = [("Cinco", "5", "110"), ("Diez", "10", "110")]
    if invertido:
        chains.reverse()
    if tarde:
        chains = [
            (index, priority, "120" if index == "Diez" else media)
            for index, priority, media in chains
        ]

    for index, priority, media in chains:
        model.table(f"Tabla{index}", f"Param{index}")
        mark = model.add("Mark", f"Marca{index}", {"Parametro": f"Param{index}"})
        tabulate = model.add(
            "Tabulate", f"Tabula{index}", {"NombreTabla": f"Tabla{index}"}
        )
        advance = model.add(
            "Advance", f"Espera{index}", {"Media": "10000", "Modificador": ""}
        )
        model.connect(mark, tabulate)
        model.connect(tabulate, advance)

        gen = generate(model, f"Genera{index}", media, priority=priority)
        preempt = model.add(
            "Preempt",
            f"Preempta{index}",
            {"Facility": "Recurso", "Remanente": f"Remanente{index}"},
        )
        advance = model.add(
            "Advance", f"Avanza{index}", {"Media": "50", "Modificador": ""}
        )
        ret = model.add("Return", f"Devuelve{index}", {"Facility": "Recurso"})
        terminate = model.add("Terminate", f"Termina{index}")
        model.connect(gen, preempt)
        model.connect(preempt, advance)
        model.connect(advance, ret)
        model.connect(ret, terminate)
        model.connect_alt(preempt, mark)

    model.add("Clock", "Reloj", {"Frecuencia": "3600"})
    model.write(name)


def probe_avances() -> None:
    """Two transactions that resume from an advance in the same instant.

    The owner holds the facility from t=10 to t=90.  Two transactions
    arrive at t=20 and t=30 and wait in an advance that ends at t=100 for
    both of them; the facility is free by then, so one of them takes it.
    The two resumptions were scheduled at t=20 and at t=30: the one
    scheduled last is the one that goes first if the events of an instant
    are taken in reverse order, and the second one if they are taken in
    the order in which they were scheduled.

    This probe tells apart the two readings of `dosnuevos`: the generators
    could be primed in reverse order, or the events of an instant could
    be taken in reverse order.
    """
    model = Model("avances")
    owner(model, "10", "80")
    candidate(model, "20", "10", prefix="Uno", before="80")
    candidate(model, "30", "10", prefix="Dos", before="70")
    model.add("Clock", "Reloj", {"Frecuencia": "3600"})
    model.write("avances")


def main() -> None:
    probe_dosnuevos()
    probe_dosnuevos(invertido=True)
    probe_esperando()
    probe_prioridad_espera()
    probe_fifo()
    probe_prioridades_cola()
    probe_avances()
    probe_preempta()
    probe_preempta(invertido=True)
    probe_preempta(tarde=True)


if __name__ == "__main__":
    main()
```

## Qué no tocó esta unidad

- Ningún archivo del motor (`crates/genmsi-core/src/**`), ni los tests del
  núcleo, ni los snapshots, ni el catálogo, ni los contratos publicados.
- Ningún archivo de `fixtures/` ni de `scripts/wine-oracle/out/`.
- No se agregó ninguna dependencia.
- El andamiaje de las sondas vive fuera del repositorio
  (`/tmp/simultaneos-probes`); el script que lo escribe está en el apéndice.
- No se hizo commit.

## La regla aplicada al motor

La unidad que siguió a esta puso la regla en el motor (commit `444d5b2`). Lo
que esta nota dejó abierto, y cómo quedó:

### El caso especial de la hora en que termina la corrida: se borra

La nota decía que la regla general "deja de hacer falta (o se invierte la
comparación en vez de negar el número)". Se eligió la primera rama: **se
borra**.

El caso especial negaba el número de secuencia de los eventos de la hora en
que la corrida termina, para que el tic del reloj acabara la corrida antes de
que se contara la llegada de esa hora. Con la regla general (el último
programado primero) esa llegada ya va antes del tic por sí sola, así que el
caso especial es la regla general restringida a un instante. **Mantenerlo
encima de la regla nueva la rompe**: con el caso Special puesto, el test
`the_run_counts_the_arrival_of_the_hour_in_which_it_ends` falla con **119
entradas de la cola en vez de 120**.

Con el caso borrado disappears también el campo `Simulation::end_time`.

### La llegada pendiente del generador: en los dos sitios

La nota nombraba `Simulation::install_definitions`. La regla se aplicó
también al **reprogramado** de `tick()` (la llegada siguiente del
generador), y esto no es un capricho: el reporte del original lo dice. En
`fixtures/outputs/ejercicio_3__ejercicio_3.report.txt` la llegada pendiente
aparece como `39  1  1839  9  9` — **prioridad 1**, la del generador —, y
nosotros poníamos `0`. Con el cambio sólo en `install_definitions` seguiría
poniendo `0` (la llegada que queda al final de la corrida es la reprogramada
por `tick()`); con los dos sitios, `1`, como el original.

### El numerito de las transacciones que aún no nacieron

`pending_events` ordenaba las llegadas pendientes al revés (`right.cmp(left)`)
para imitar "la hora final al revés". Con la regla general esa inversión era
justo lo contrario del orden de nacimiento, así que el desempate se invierte
(`left.cmp(right)`) y la numeración sigue el orden real de nacimiento.

### El efecto medido

- Las diez sondas coinciden con el original en **las 22 lecturas** (el ejemplo
  `crates/genmsi-core/examples/simultaneos` las imprime una al lado de la
  otra y calcula el número).
- Los 66 bancos del oráculo coinciden en todas sus métricas y su GPSS es
  byte a byte el mismo.
- La paridad con el original de los 25 modelos de `fixtures/models` mejora
  para el `Parcial` (130 de 150 a 134) y para el `TP10` (61 de 64 a 62), y
  baja para el `TP3` (87 de 91 a 84), cuyo orden del listado es una
  diferencia conocida de la fase 6.
