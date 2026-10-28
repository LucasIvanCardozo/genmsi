#!/usr/bin/env python3
"""Build the gallery of the block classes: models that run and that carry,
between them, every class of the catalogue.

The reader wants to look at the 66 block classes one by one **with their
data**: the shape the app draws and the live figure it shows while the run
plays. A block only has a live figure while a run is playing, so every
class needs a model that runs and that contains it.

The models are made by the engine itself, through its own editing session,
so they are valid models by construction: every block carries the
footprint its class declares and every reference is the one the model
declares. The reference models of `fixtures/models` are read-only oracle
material: nothing here writes there.

Two rules shape every model of the gallery:

* **every reference resolves.** The figure of a block is the number of the
  entity it operates on (the content of a queue, the occupancy of a
  facility, the units of a deposit, the samples of a table) or the pair of
  always for the classes that declare no entity. A reference that points at
  nothing shows nothing, so every entity a block names is declared by the
  model itself.
* **every block is walked by the run.** A block that the run never reaches
  has no figure either, so the flow of each model is a spine every
  transaction walks, and the branches that the class offers go back to the
  spine instead of into the void.

Usage:
  crates/genmsi-app/scripts/make-gallery-models.py [OUTPUT_DIR]

The engine binaries must be built:
  cargo build -p genmsi-cli -p genmsi-app-engine
"""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[3]
ENGINE = REPO_ROOT / "target" / "debug" / "genmsi-app-engine"
CLI = REPO_ROOT / "target" / "debug" / "genmsi"
DEFAULT_OUTPUT = REPO_ROOT / "crates" / "genmsi-app" / "fixtures"

# The room every block gets on the canvas. The room is bigger than the
# biggest footprint of the catalogue, so no block touches another one.
COLUMN_WIDTH = 200
ROW_HEIGHT = 130
ORIGIN_X = 40
ORIGIN_Y = 40

# How many blocks fit in a row of the canvas. The gallery is read on the
# screen, so the picture of a model has to fit in a window: the spine of
# the flow walks the rows in a serpentine.
BLOCKS_PER_ROW = 6

# The horizon of the runs of the gallery: the clock of every model ends
# the run there, and the transactions that walk the spine keep coming.
HORIZON = "3600"


# --------------------------------------------------------------------------
# The models of the gallery.
#
# Every block is (key, class, caption, properties). Every link is (from key,
# from port, to key, to port). The port names are the ones of the catalogue;
# the script looks their ids up there. The first blocks of a model are its
# declarations: they do not walk the flow, and they go on the canvas in the
# rows of their own.
# --------------------------------------------------------------------------


def model_flujo() -> dict:
    """The source, the operations and the declarations of the catalogue."""
    return {
        "name": "galeria-flujo",
        "caption": "Galería del flujo: la fuente, las operaciones y las declaraciones",
        "declarations": 12,
        "blocks": [
            # The declarations: the classes that declare an entity instead of
            # walking the flow.
            ("var", "Variable", "Variable V1", {"NombreVar": "V1", "Expresion": "1"}),
            ("vara", "VariableA", "Variable aleatoria V2", {"NombreVar": "V2", "Minimo": "1", "Maximo": "10", "Seed": "1"}),
            ("varn", "VariableN", "Variable normal V3", {"NombreVar": "V3", "NombreFunc": "FNorm", "Media": "0", "Desvio": "1", "Seed": "1"}),
            ("func", "Function", "Función FTab", {"NombreFunc": "FTab", "Atributo": "1", "Tabla": "{1|10;2|20;3|30}", "Tipo": "Discreta"}),
            ("fdist", "FunctionDist", "Función FDist", {"NombreDist": "FDist", "Distribucion": "{25%;40%;35%}", "Semilla": "3"}),
            ("fexp", "FunctionExpon", "Función FExp", {"NombreFunc": "FExp", "Seed": "2"}),
            ("mfunc", "MFunction", "Funciones FM1 y FM2", {"NombresFunc": "{FM1;FM2}", "Matriz": "{{1;2};{3;4}}", "Atributo": "1"}),
            ("mfd", "MFunctionDist", "Distribuciones FD1 y FD2", {"NombresDist": "{FD1;FD2}", "Resultados": "{{50%;50%};{30%;70%}}", "Semilla": "5"}),
            ("initiasv", "InitiaSV", "Valor inicial SV1", {"SaveValue": "SV1", "Valor": "0"}),
            ("initialls", "InitialLS", "Interruptor LS1", {"LogicSwich": "LS1", "Valor": "Set"}),
            ("note", "Note", "Anotación de la galería", {}),
            ("clock", "Clock", "Reloj de la corrida", {"Frecuencia": HORIZON}),
            # The spine: every transaction walks it from end to end.
            ("gen", "Generate", "Genera clientes", {"Cantidad": "120", "Media": "30", "Modificador": "10"}),
            ("asg1", "Assign", "Asigna P1 = 1", {"Parametro": "P1", "Valor": "1", "Operador": "="}),
            ("asg2", "Assign", "Asigna P2 = V1", {"Parametro": "P2", "Valor": "V:V1", "Operador": "="}),
            ("asg3", "Assign", "Asigna P3 = V2", {"Parametro": "P3", "Valor": "V:V2", "Operador": "="}),
            ("asg4", "Assign", "Asigna P4 = V3", {"Parametro": "P4", "Valor": "V:V3", "Operador": "="}),
            ("asg5", "Assign", "Asigna P5 = FTab", {"Parametro": "P5", "Valor": "FN:FTab", "Operador": "="}),
            ("asg6", "Assign", "Asigna P6 = FDist", {"Parametro": "P6", "Valor": "FN:FDist", "Operador": "="}),
            ("asg7", "Assign", "Asigna P7 = FExp", {"Parametro": "P7", "Valor": "FN:FExp", "Operador": "="}),
            ("mark", "Mark", "Marca la hora en P8", {"Parametro": "P8"}),
            ("cnt", "Counter", "Cuenta en SV1", {"SaveValue": "SV1", "Param": "P1", "Incremento": "1"}),
            ("adv", "Advance", "Espera dos unidades", {"Media": "2", "Modificador": "1"}),
            ("prio", "Priority", "Prioridad 1", {"Prioridad": "1"}),
            ("sv", "SaveValue", "Suma uno a SV1", {"SaveValue": "SV1", "Valor": "1", "Op": "+"}),
            ("logic", "Logic", "Enciende LS1", {"LogicSwich": "LS1", "Op": "Set"}),
            ("buf", "Buffer", "Búfer de paso", {}),
            ("adv2", "Advance", "Espera una unidad", {"Media": "1", "Modificador": "0"}),
            ("term", "Terminate", "Termina la transacción", {"Unidades": ""}),
        ],
        "links": [
            ("gen", "Salida1", "asg1", "Entrada1"),
            ("asg1", "Salida1", "asg2", "Entrada1"),
            ("asg2", "Salida1", "asg3", "Entrada1"),
            ("asg3", "Salida1", "asg4", "Entrada1"),
            ("asg4", "Salida1", "asg5", "Entrada1"),
            ("asg5", "Salida1", "asg6", "Entrada1"),
            ("asg6", "Salida1", "asg7", "Entrada1"),
            ("asg7", "Salida1", "mark", "Entrada1"),
            ("mark", "Salida1", "cnt", "Entrada1"),
            ("cnt", "Salida1", "adv", "Entrada1"),
            ("adv", "Salida1", "prio", "Entrada1"),
            ("prio", "Salida1", "sv", "Entrada1"),
            ("sv", "Salida1", "logic", "Entrada1"),
            ("logic", "Salida1", "buf", "Entrada1"),
            ("buf", "Salida1", "adv2", "Entrada1"),
            ("adv2", "Salida1", "term", "Entrada1"),
        ],
    }


def model_decisiones() -> dict:
    """The decisions of the catalogue: the gates, the tests and the picks."""
    return {
        "name": "galeria-decisiones",
        "caption": "Galería de las decisiones: los tests, las puertas y las selecciones",
        "declarations": 2,
        "blocks": [
            ("clock", "Clock", "Reloj de la corrida", {"Frecuencia": HORIZON}),
            # The deposit the gate watches: the gate of a deposit needs the
            # deposit to be declared, or its reference points at nothing.
            ("storagedef", "StorageDef", "Depósito S1", {"Storage": "S1", "Tamaño": "10"}),
            ("gen", "Generate", "Genera clientes", {"Cantidad": "120", "Media": "30", "Modificador": "10"}),
            ("asg", "Assign", "Asigna P1 = 1", {"Parametro": "P1", "Valor": "1", "Operador": "="}),
            ("asg2", "Assign", "Asigna P2 = 2", {"Parametro": "P2", "Valor": "2", "Operador": "="}),
            ("asg3", "Assign", "Asigna P3 = 1", {"Parametro": "P3", "Valor": "1", "Operador": "="}),
            ("test", "Test", "Prueba P1 = 1", {"1_Valor": "P:P1", "2_OP": "=", "3_Valor": "1"}),
            ("xfer", "Transfer", "Transfiere siempre", {"Prob": "1000"}),
            ("loop", "Loop", "Repite P2 veces", {"Parametro": "P2"}),
            ("loopasg", "Assign", "Resta uno a P2", {"Parametro": "P2", "Valor": "P:P2-1", "Operador": "="}),
            ("split", "Split", "Duplica la transacción", {"Cantidad": "2"}),
            ("multi", "Multifurcacion", "Reparte por probabilidad", {"Prob1": "100", "Prob2": "0", "Prob3": "0", "Prob4": "0", "Prob5": "0"}),
            ("multip", "MultifurcacionP", "Reparte por P3", {"Parametro": "P3", "Prob1": "100", "Prob2": "0", "Prob3": "0", "Prob4": "0", "Prob5": "0"}),
            ("gatef", "GateF", "Espera el recurso F1", {"Facility": "F1", "op": "Libre"}),
            ("gatel", "GateL", "Espera el interruptor LS2", {"LogicSwich": "LS2", "op": "Reset"}),
            ("gatem", "GateM", "Espera el rótulo V", {"Rotulo": "V", "op": "NotMaching"}),
            ("gates", "GateS", "Espera el depósito S1", {"Storage": "S1", "op": "Vacio"}),
            ("select", "Select", "Elige la cola más corta", {"ClaseSNA": "Q", "Param": "P4", "minimo": "1", "maximo": "1", "op": "Minimo"}),
            ("selectf", "SelectF", "Elige un recurso libre", {"Param": "P5", "minimo": "1", "maximo": "1", "op": "Libre"}),
            ("selectl", "SelectL", "Elige un interruptor", {"Param": "P6", "minimo": "1", "maximo": "1", "op": "Set"}),
            ("selects", "SelectS", "Elige un depósito", {"Param": "P7", "minimo": "1", "maximo": "1", "op": "Vacio"}),
            ("selectsna", "SelectSNA", "Elige por valor", {"ClaseSNA": "Q", "Param": "P8", "minimo": "1", "maximo": "1", "op": "=", "valorComp": "0"}),
            ("term", "Terminate", "Termina la transacción", {"Unidades": ""}),
            ("term2", "Terminate", "Termina las copias", {"Unidades": ""}),
        ],
        "links": [
            ("gen", "Salida1", "asg", "Entrada1"),
            ("asg", "Salida1", "asg2", "Entrada1"),
            ("asg2", "Salida1", "asg3", "Entrada1"),
            ("asg3", "Salida1", "test", "Entrada1"),
            # The test and the transfer have two outputs: the failure one
            # goes back to the spine, so the flow never stops.
            ("test", "Salida1", "xfer", "Entrada1"),
            ("test", "Salida2", "xfer", "Entrada1"),
            ("xfer", "Salida1", "loop", "Entrada1"),
            ("xfer", "Salida2", "loop", "Entrada1"),
            ("loop", "Salida1", "split", "Entrada1"),
            ("loop", "Salida2", "loopasg", "Entrada1"),
            ("loopasg", "Salida1", "loop", "Entrada1"),
            ("split", "Salida1", "multi", "Entrada"),
            ("split", "Salida2", "term2", "Entrada1"),
            ("multi", "Salida1", "multip", "Entrada"),
            ("multip", "Salida1", "gatef", "Entrada1"),
            ("gatef", "Salida1", "gatel", "Entrada1"),
            ("gatef", "Salida2", "gatel", "Entrada1"),
            ("gatel", "Salida1", "gatem", "Entrada1"),
            ("gatel", "Salida2", "gatem", "Entrada1"),
            ("gatem", "Salida1", "gates", "Entrada1"),
            ("gatem", "Salida2", "gates", "Entrada1"),
            ("gates", "Salida1", "select", "Entrada1"),
            ("gates", "Salida2", "select", "Entrada1"),
            ("select", "Salida1", "selectf", "Entrada1"),
            ("selectf", "Salida1", "selectl", "Entrada1"),
            ("selectf", "Salida2", "selectl", "Entrada1"),
            ("selectl", "Salida1", "selects", "Entrada1"),
            ("selectl", "Salida2", "selects", "Entrada1"),
            ("selects", "Salida1", "selectsna", "Entrada1"),
            ("selects", "Salida2", "selectsna", "Entrada1"),
            ("selectsna", "Salida1", "term", "Entrada1"),
            ("selectsna", "Salida2", "term", "Entrada1"),
        ],
    }


def model_recursos() -> dict:
    """The resources of the catalogue: the queues and the facilities."""
    return {
        "name": "galeria-recursos",
        "caption": "Galería de los recursos: las colas y las máquinas",
        "declarations": 1,
        "blocks": [
            ("clock", "Clock", "Reloj de la corrida", {"Frecuencia": HORIZON}),
            ("gen", "Generate", "Genera clientes", {"Cantidad": "180", "Media": "20", "Modificador": "5"}),
            ("queue", "Queue", "Entra en la cola Q2", {"Queue": "Q2", "Cantidad": "1"}),
            ("adv", "Advance", "Espera treinta unidades", {"Media": "30", "Modificador": "5"}),
            ("depart", "Depart", "Sale de la cola Q2", {"Queue": "Q2", "Cantidad": "1"}),
            ("facuse", "FacilityUse", "Usa la máquina F1", {"Facility": "F1", "Media": "60", "Modificador": "10", "Queue": "Q1"}),
            ("seize", "Seize", "Toma la máquina F3", {"Facility": "F3"}),
            ("release", "Release", "Devuelve la máquina F3", {"Facility": "F3"}),
            ("preempt", "Preempt", "Arrebata la máquina F2", {"Facility": "F2", "ModoArrebato": "Remover", "ModoInterrupcion": "Incondicional", "Remanente": "P9"}),
            ("ret", "Return", "Devuelve la máquina F2", {"Facility": "F2"}),
            ("facint", "FacilityInt", "Interrumpe la máquina F4", {"Facility": "F4", "Media": "10", "Modificador": "0", "Queue": "Q3", "ModoArrebato": "Remover", "ModoInterrupcion": "Incondicional"}),
            ("term", "Terminate", "Termina la transacción", {"Unidades": ""}),
        ],
        "links": [
            ("gen", "Salida1", "queue", "Entrada1"),
            ("queue", "Salida1", "adv", "Entrada1"),
            ("adv", "Salida1", "depart", "Entrada1"),
            ("depart", "Salida1", "facuse", "Entrada1"),
            ("facuse", "Salida1", "seize", "Entrada1"),
            ("seize", "Salida1", "release", "Entrada1"),
            ("release", "Salida1", "preempt", "Entrada1"),
            ("preempt", "Salida1", "ret", "Entrada1"),
            ("preempt", "SalidaAlt", "term", "Entrada1"),
            ("ret", "Salida1", "facint", "Entrada1"),
            ("facint", "Salida1", "term", "Entrada1"),
            ("facint", "SalidaAlt", "term", "Entrada1"),
        ],
    }


def model_depositos() -> dict:
    """The deposits, the tables and the chains of the catalogue."""
    return {
        "name": "galeria-depositos",
        "caption": "Galería de los depósitos, las tablas y las cadenas",
        "declarations": 4,
        "blocks": [
            ("storagedef", "StorageDef", "Depósito ALMACEN", {"Storage": "ALMACEN", "Tamaño": "10"}),
            ("table", "Table", "Tabla T1", {"NombreTabla": "T1", "Atributo": "P:P1", "Cantidad": "5", "PrimerIntervalo": "0", "TamFrecuencia": "10"}),
            ("qtable", "QTable", "Tabla de cola QT1", {"NombreTabla": "QT1", "Queue": "Q2", "Cantidad": "5", "PrimerIntervalo": "0", "TamFrecuencia": "10"}),
            ("clock", "Clock", "Reloj de la corrida", {"Frecuencia": HORIZON}),
            ("gen", "Generate", "Genera clientes", {"Cantidad": "120", "Media": "30", "Modificador": "10"}),
            ("asg", "Assign", "Asigna P1 = 5", {"Parametro": "P1", "Valor": "5", "Operador": "="}),
            ("enter", "Enter", "Toma dos unidades", {"Storage": "ALMACEN", "Cantidad": "2"}),
            ("adv", "Advance", "Espera cinco unidades", {"Media": "5", "Modificador": "0"}),
            ("leave", "Leave", "Devuelve dos unidades", {"Storage": "ALMACEN", "Cantidad": "2"}),
            ("tab", "Tabulate", "Tabula P1 en T1", {"NombreTabla": "T1", "Peso": "1"}),
            ("sus", "StorageUse", "Usa el depósito ALMACEN", {"Storage": "ALMACEN", "Media": "10", "Modificador": "0", "Cantidad": "1", "Queue": "Q2"}),
            ("term", "Terminate", "Termina la transacción", {"Unidades": ""}),
            ("gen2", "Generate", "Genera eslabones", {"Cantidad": "120", "Media": "10", "Modificador": "0"}),
            ("link", "Link", "Engancha en la cadena CADENA", {"Cadena": "CADENA", "Modo": "FIFO"}),
            ("term2", "Terminate", "Termina el eslabón suelto", {"Unidades": ""}),
            ("gen3", "Generate", "Genera soltadores", {"Cantidad": "120", "Media": "30", "Modificador": "0"}),
            ("unlink", "Unlink", "Suelta la cadena CADENA", {"Cadena": "CADENA", "Cantidad": "1", "Modo": "Algunos"}),
            ("term3", "Terminate", "Termina el soltador", {"Unidades": ""}),
            ("term4", "Terminate", "Termina los eslabones sueltos", {"Unidades": ""}),
        ],
        "links": [
            ("gen", "Salida1", "asg", "Entrada1"),
            ("asg", "Salida1", "enter", "Entrada1"),
            ("enter", "Salida1", "adv", "Entrada1"),
            ("adv", "Salida1", "leave", "Entrada1"),
            ("leave", "Salida1", "tab", "Entrada1"),
            ("tab", "Salida1", "sus", "Entrada1"),
            ("sus", "Salida1", "term", "Entrada1"),
            ("gen2", "Salida1", "link", "Entrada1"),
            ("link", "Salida1", "term2", "Entrada1"),
            ("gen3", "Salida1", "unlink", "Entrada1"),
            ("unlink", "Salida1", "term3", "Entrada1"),
            ("unlink", "SalidaAlt", "term4", "Entrada1"),
            ("unlink", "Salida2", "term3", "Entrada1"),
        ],
    }


def model_grupos() -> dict:
    """The groups of transactions: the joins, the removes and the matches."""
    return {
        "name": "galeria-grupos",
        "caption": "Galería de los grupos: las uniones, los retiros y las parejas",
        "declarations": 1,
        "blocks": [
            ("clock", "Clock", "Reloj de la corrida", {"Frecuencia": HORIZON}),
            ("gen", "Generate", "Genera clientes", {"Cantidad": "120", "Media": "30", "Modificador": "10"}),
            ("asg", "Assign", "Asigna P1 = 1", {"Parametro": "P1", "Valor": "1", "Operador": "="}),
            ("joinme", "JoinMe", "Entra en el grupo G1", {"Grupo": "G1"}),
            ("joinnum", "JoinNumber", "Entra en el grupo N1 con el 5", {"Grupo": "N1", "Numero": "5"}),
            ("examine", "Examine", "Pregunta si está en G1", {"Grupo": "G1"}),
            ("examenum", "ExamineNumber", "Pregunta si el 5 está en N1", {"Grupo": "N1", "Numero": "5"}),
            ("scan", "Scan", "Registra el rango de G1", {"Grupo": "G1", "CompOp": "=", "CompOpcion": "Parametro", "CompParam": "P1", "CompValue": "1", "DeOpción": "Parametro", "DeParam": "P2", "EnOpcion": "Parametro", "EnParam": "P3"}),
            ("alter", "Alter", "Altera el grupo G1", {"Grupo": "G1", "AltOpcion": "Parametro", "AltParam": "P1", "AltValor": "7", "Cantidad": "1", "CompOp": "=", "CompOpcion": "Parametro", "CompParam": "P1", "CompValue": "1", "Modo": "Algunos"}),
            ("removeme", "RemoveMe", "Sale del grupo G1", {"Grupo": "G1"}),
            ("removetrans", "RemoveTrans", "Saca una transacción de G1", {"Grupo": "G1", "Cantidad": "1", "CompOp": "=", "CompOpcion": "Parametro", "CompParam": "P1", "CompValue": "7", "Modo": "Algunos"}),
            ("removenum", "RemoveNumber", "Saca el 5 de N1", {"Grupo": "N1", "Numero": "5", "Op": "="}),
            ("assemble", "Assemble", "Reúne una transacción", {"Cantidad": "1"}),
            ("gather", "Gather", "Reúne y suelta", {"Cantidad": "1"}),
            ("matcha", "Match", "Pareja con el rótulo MB", {"RotuloPropio": "MA", "RotuloMatch": "MB"}),
            ("term", "Terminate", "Termina la transacción", {"Unidades": ""}),
            ("gen2", "Generate", "Genera parejas", {"Cantidad": "120", "Media": "36", "Modificador": "10"}),
            ("matchb", "Match", "Pareja con el rótulo MA", {"RotuloPropio": "MB", "RotuloMatch": "MA"}),
            ("term2", "Terminate", "Termina la pareja", {"Unidades": ""}),
        ],
        "links": [
            ("gen", "Salida1", "asg", "Entrada1"),
            ("asg", "Salida1", "joinme", "Entrada1"),
            ("joinme", "Salida1", "joinnum", "Entrada1"),
            ("joinnum", "Salida1", "examine", "Entrada1"),
            ("examine", "Salida1", "examenum", "Entrada1"),
            ("examine", "Salida2", "examenum", "Entrada1"),
            ("examenum", "Salida1", "scan", "Entrada1"),
            ("examenum", "Salida2", "scan", "Entrada1"),
            ("scan", "Salida1", "alter", "Entrada1"),
            ("scan", "Salida2", "alter", "Entrada1"),
            ("alter", "Salida1", "removeme", "Entrada1"),
            ("alter", "Salida2", "removeme", "Entrada1"),
            ("removeme", "Salida1", "removetrans", "Entrada1"),
            ("removetrans", "Salida1", "removenum", "Entrada1"),
            ("removetrans", "Salida2", "removenum", "Entrada1"),
            ("removenum", "Salida1", "assemble", "Entrada1"),
            ("removenum", "Salida2", "assemble", "Entrada1"),
            ("assemble", "Salida1", "gather", "Entrada1"),
            ("gather", "Salida1", "matcha", "Entrada1"),
            ("matcha", "Salida1", "term", "Entrada1"),
            ("gen2", "Salida1", "matchb", "Entrada1"),
            ("matchb", "Salida1", "term2", "Entrada1"),
        ],
    }


MODELS = [
    model_flujo(),
    model_decisiones(),
    model_recursos(),
    model_depositos(),
    model_grupos(),
]


# --------------------------------------------------------------------------
# The engine session: the commands that build the model.
# --------------------------------------------------------------------------


def catalogue() -> dict[str, dict]:
    """The classes of the engine, with their ports and their properties."""
    out = subprocess.run(
        [str(ENGINE), "catalogue"], capture_output=True, text=True, check=True
    ).stdout
    return {entry["class"]: entry for entry in json.loads(out)["classes"]}


def port_id(classes: dict[str, dict], class_name: str, port: str) -> int:
    """The id of a port of a class, as the catalogue declares it."""
    entry = classes[class_name]
    for group in ("inputs", "outputs"):
        for candidate in entry[group]:
            if candidate["name"] == port:
                return int(candidate["ioid"])
    raise SystemExit(f"the class {class_name} has no port named {port}")


def layout(model: dict) -> list[tuple[int, int]]:
    """Where every block of the model goes on the canvas.

    The declarations go first, in rows of their own: they do not walk the
    flow. The spine walks the rows in a serpentine, so the flow of the model
    reads from the left to the right and then from the right to the left.
    """
    declarations = model["declarations"]
    places: list[tuple[int, int]] = []
    for index in range(declarations):
        places.append((index % BLOCKS_PER_ROW, index // BLOCKS_PER_ROW))
    first_row = (declarations + BLOCKS_PER_ROW - 1) // BLOCKS_PER_ROW
    spine = len(model["blocks"]) - declarations
    for index in range(spine):
        row = index // BLOCKS_PER_ROW
        column = index % BLOCKS_PER_ROW
        if row % 2 == 1:
            column = BLOCKS_PER_ROW - 1 - column
        places.append((column, first_row + row))
    return places


def requests_for(model: dict, classes: dict[str, dict]) -> list[dict]:
    """The requests of the editing session that build the model."""
    ids: dict[str, int] = {}
    requests: list[dict] = [{"op": "new"}]
    places = layout(model)
    for index, (key, class_name, caption, props) in enumerate(model["blocks"]):
        ids[key] = index + 1
        column, row = places[index]
        requests.append(
            {
                "op": "apply",
                "command": {
                    "op": "addBlock",
                    "class": class_name,
                    "container": 0,
                    "x": ORIGIN_X + column * COLUMN_WIDTH,
                    "y": ORIGIN_Y + row * ROW_HEIGHT,
                },
            }
        )
        requests.append(
            {"op": "apply", "command": {"op": "setCaption", "id": index + 1, "caption": caption}}
        )
        for name, value in props.items():
            requests.append(
                {
                    "op": "apply",
                    "command": {
                        "op": "setProperty",
                        "id": index + 1,
                        "property": name,
                        "value": value,
                    },
                }
            )
    for from_key, from_port, to_key, to_port in model["links"]:
        requests.append(
            {
                "op": "apply",
                "command": {
                    "op": "connect",
                    "from": ids[from_key],
                    "fromIoid": port_id(classes, model["class_of"][from_key], from_port),
                    "to": ids[to_key],
                    "toIoid": port_id(classes, model["class_of"][to_key], to_port),
                },
            }
        )
    return requests


def build(model: dict, classes: dict[str, dict], output: Path) -> None:
    """Build the model with the editing session of the engine and save it."""
    requests = requests_for(model, classes)
    requests.append({"op": "save", "path": str(output)})
    payload = "".join(json.dumps(request) + "\n" for request in requests)
    out = subprocess.run(
        [str(ENGINE), "session"], input=payload, capture_output=True, text=True
    )
    if out.returncode != 0:
        raise SystemExit(f"the engine failed:\n{out.stderr}")
    answers = [json.loads(line) for line in out.stdout.splitlines() if line.strip()]
    errors = [answer for answer in answers if answer.get("ok") is False]
    if errors:
        raise SystemExit(f"the engine refused:\n{json.dumps(errors[:3], indent=2)}")
    # The state of the last answer is the state of the model that was saved:
    # the answers before it are the states the model had on the way.
    states = [answer["state"] for answer in answers if answer.get("state")]
    if not states:
        raise SystemExit("the engine answered nothing")
    final = states[-1]["diagnostics"]
    if final.get("status") == "error":
        raise SystemExit(
            f"the model does not compile:\n{json.dumps(final, indent=2)}"
        )
    for message in final.get("diagnostics") or []:
        print(f"    aviso: {message}")


def simulate(path: Path) -> str:
    """Run the model and return the report of the run."""
    out = subprocess.run(
        [str(CLI), "simulate", str(path)], capture_output=True, text=True
    )
    if out.returncode != 0:
        raise SystemExit(f"the model does not run:\n{out.stderr}")
    return out.stdout


# The property each kind of entity is named by, and the section of the
# report that publishes it. The classes that are not here operate on no
# entity: they show the pair of always.
ENTITY_PROPERTY = {
    "queue": ("Queue", "queues"),
    "facility": ("Facility", "facilities"),
    "storage": ("Storage", "storages"),
    "table": ("NombreTabla", "tables"),
    "chain": ("Cadena", "chains"),
    "logic": ("LogicSwich", "logic"),
    "value": ("SaveValue", "savevalues"),
}

# The kind of entity each class of the catalogue operates on.
ENTITY_KIND = {
    "Queue": "queue", "Depart": "queue",
    "Seize": "facility", "Release": "facility", "Return": "facility",
    "Preempt": "facility", "FacilityUse": "facility", "FacilityInt": "facility",
    "Enter": "storage", "Leave": "storage", "StorageUse": "storage",
    "Tabulate": "table", "Link": "chain", "Unlink": "chain",
    "Logic": "logic", "InitialLS": "logic",
    "SaveValue": "value", "InitiaSV": "value", "Counter": "value",
}


def check(model: dict, path: Path) -> None:
    """Check the two promises of the gallery: the run walks every block and
    every reference of a block resolves.

    The check reads the report of the run: a block with a row in the report
    has been walked by the run (its row counts the entries), and the entity
    a block names has to be one the report publishes.
    """
    out = subprocess.run(
        [str(CLI), "simulate", "--json", str(path)], capture_output=True, text=True
    )
    if out.returncode != 0:
        raise SystemExit(f"the model does not run:\n{out.stderr}")
    report = json.loads(out.stdout)
    rows: dict[int, list[dict]] = {}
    for row in report["blocks"]:
        rows.setdefault(row["node"], []).append(row)
    entities = report["entities"]
    for key, class_name, caption, props in model["blocks"]:
        walked = rows.get(model["ids"][key], [])
        if walked and not any(row["entries"] > 0 for row in walked):
            raise SystemExit(
                f"{path.name}: the run never walked the {class_name} block"
            )
        kind = ENTITY_KIND.get(class_name)
        if kind is None:
            continue
        property, section = ENTITY_PROPERTY[kind]
        name = props.get(property, "").strip()
        if not name:
            raise SystemExit(
                f"{path.name}: the {class_name} block names no {property}"
            )
        if name not in (entities.get(section) or {}):
            raise SystemExit(
                f"{path.name}: the {class_name} block names the {kind} "
                f"'{name}', and the run does not publish it"
            )


def main() -> int:
    output_dir = Path(sys.argv[1]).resolve() if len(sys.argv) > 1 else DEFAULT_OUTPUT
    if not ENGINE.exists() or not CLI.exists():
        raise SystemExit(
            f"{ENGINE} is missing; run cargo build -p genmsi-cli -p genmsi-app-engine"
        )
    classes = catalogue()
    output_dir.mkdir(parents=True, exist_ok=True)
    covered: dict[str, str] = {}
    for model in MODELS:
        model["class_of"] = {key: class_name for key, class_name, *_ in model["blocks"]}
        model["ids"] = {key: index + 1 for index, (key, *_) in enumerate(model["blocks"])}
        path = output_dir / f"{model['name']}.gmdl"
        build(model, classes, path)
        simulate(path)
        check(model, path)
        for key, class_name, *_ in model["blocks"]:
            covered.setdefault(class_name, model["name"])
        print(f"==> {path.name}: {len(model['blocks'])} bloques, corre y todos caminan")
    missing = sorted(set(classes) - set(covered))
    if missing:
        raise SystemExit(f"the gallery does not cover: {', '.join(missing)}")
    print(f"==> la galeria cubre las {len(covered)} clases del catalogo")
    return 0


if __name__ == "__main__":
    sys.exit(main())
