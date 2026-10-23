#!/usr/bin/env python3
"""Build one bench model per class of the original editor.

A bench is the smallest model that exercises exactly one class: a `Generate`
that produces transactions, the class under test with its own ports
connected, and a `Terminate` at the end.  A `Clock` closes the run, because
the original only stops when the start counter reaches zero.

Everything the bench needs is read from the original's own class catalogue
(`Genmsi_20230711.backup/gClasses.xml`): the ports of every class come from
the graph section, the properties and their default values from the macro
code.  The output is deterministic: the same XML always gives the same bytes.

Usage:
    python3 scripts/wine-oracle/make-benches.py [outdir]

The default output directory is `scripts/wine-oracle/benches`.
"""

from __future__ import annotations

import re
import sys
import urllib.parse
import xml.etree.ElementTree as ET
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
CLASSES_XML = REPO / "Genmsi_20230711.backup" / "gClasses.xml"
DEFAULT_OUT = REPO / "scripts" / "wine-oracle" / "benches"

MODEL_VERSION = "1.00.2008.03.23"

# ---------------------------------------------------------------------------
# Property types.
#
# The XML declares the name, the hint and the default value of a property but
# not its type; the type is what the original writes in the model file.  The
# table below is the one the original's own models use (the reference models
# in `fixtures/models`): the type a property name carries, and the classes
# where the same name means something else.
# ---------------------------------------------------------------------------

PROP_TYPES: dict[str, tuple[str, str]] = {
    "1_Valor": ("TSNA", "SNA"),
    "2_OP": ("Listed", "op_Relac"),
    "3_Valor": ("TSNA", "SNA"),
    "AltOpcion": ("Listed", "PriorityOption"),
    "AltParam": ("EntityUse", "Parameter"),
    "AltValor": ("TSNA", "SNA"),
    "Atributo": ("TSNA", "SNA"),
    "Cadena": ("EntityUse", "UserChain"),
    "Cantidad": ("TSNA", "SNA"),
    "ClaseSNA": ("Listed", "entitySNAclass"),
    "CompOp": ("Listed", "op_RelacMaxMin"),
    "CompOpcion": ("Listed", "PriorityOption"),
    "CompParam": ("EntityUse", "Parameter"),
    "CompValue": ("TSNA", "SNA"),
    "DeOpción": ("Listed", "PriorityOption"),
    "DeParam": ("EntityUse", "Parameter"),
    "Desvio": ("Expresion", "Expresion"),
    "Distribucion": ("FloatPercentList", ""),
    "EnOpcion": ("Listed", "PriorityOption"),
    "EnParam": ("EntityUse", "Parameter"),
    "Expresion": ("Expresion", "Expresion"),
    "Facility": ("EntityUse", "Facility"),
    "Frecuencia": ("TSNA", "SNA"),
    "Funcion": ("EntityUse", "Function"),
    "Grupo": ("EntityUse", "TransacGroup"),
    "Incremento": ("TSNA", "SNA"),
    "LogicSwich": ("EntityUse", "LogicSwich"),
    "Matriz": ("Table", ""),
    "Maximo": ("Expresion", "Expresion"),
    "Media": ("TSNA", "SNA"),
    "Minimo": ("Expresion", "Expresion"),
    "Modificador": ("TSNA", "SNA"),
    "Modo": ("Listed", "OperationMode"),
    "ModoArrebato": ("Listed", "RemoveMode"),
    "ModoInterrupcion": ("Listed", "PriorityMode"),
    "MultifurcacionCH": ("EntityDef", "Function"),
    "MultifurcacionGO": ("EntityDef", "Function"),
    "NombresDist": ("EntityDef", "Function"),
    "NombresFunc": ("EntityDef", "Function"),
    "NombreDist": ("EntityDef", "Function"),
    "NombreFunc": ("EntityDef", "Function"),
    "NombreTabla": ("EntityDef", "Table"),
    "NombreVar": ("EntityDef", "Variable"),
    "Numero": ("TSNA", "SNA"),
    "OffSet": ("TSNA", "SNA"),
    "Op": ("Listed", "op_Swich"),
    "Operador": ("Listed", "op_AddSub"),
    "Param": ("EntityUse", "Parameter"),
    "Parametro": ("EntityUse", "Parameter"),
    "Peso": ("TSNA", "SNA"),
    "PrimerIntervalo": ("PosInteger", ""),
    "Prioridad": ("TSNA", "SNA"),
    "Prob": ("", "MultiType"),
    "Prob1": ("FloatPercent", ""),
    "Prob2": ("FloatPercent", ""),
    "Prob3": ("FloatPercent", ""),
    "Prob4": ("FloatPercent", ""),
    "Prob5": ("FloatPercent", ""),
    "Queue": ("EntityUse", "Queue"),
    "Remanente": ("EntityUse", "Parameter"),
    "Resultados": ("FloatPercentList", ""),
    "Rotulo": ("EntityUse", "Label"),
    "RotuloMatch": ("EntityUse", "Label"),
    "RotuloPropio": ("EntityDef", "Label"),
    "SaveValue": ("EntityUse", "SaveValue"),
    "Seed": ("EntityUse", "RandomSeed"),
    "Semilla": ("PosInteger", ""),
    "Storage": ("EntityUse", "Storage"),
    "Tabla": ("Table", ""),
    "TamFrecuencia": ("PosInteger", ""),
    "Tamaño": ("PosInteger", ""),
    "Tipo": ("Listed", "FuncTypes"),
    "Unidades": ("TSNA", "SNA"),
    "Valor": ("TSNA", "SNA"),
    "maximo": ("TSNA", "SNA"),
    "minimo": ("TSNA", "SNA"),
    "op": ("Listed", "op_MaxMin"),
    "valorComp": ("TSNA", "SNA"),
}

# Classes where a property name carries another type than the one above.
CLASS_PROP_TYPES: dict[tuple[str, str], tuple[str, str]] = {
    ("QTable", "Cantidad"): ("PosInteger", ""),
    ("Table", "Cantidad"): ("PosInteger", ""),
    ("QTable", "Queue"): ("EntityDef", "Queue"),
    ("QTable", "NombreTabla"): ("EntityDef", "QTable"),
    ("Tabulate", "NombreTabla"): ("EntityUse", "Table"),
    ("ExamineNumber", "Grupo"): ("EntityUse", "NumericGroup"),
    ("JoinNumber", "Grupo"): ("EntityUse", "NumericGroup"),
    ("RemoveNumber", "Grupo"): ("EntityUse", "NumericGroup"),
    ("InitialLS", "LogicSwich"): ("EntityDef", "LogicSwich"),
    ("InitialLS", "Valor"): ("Listed", "op_Swich"),
    ("InitiaSV", "SaveValue"): ("EntityDef", "SaveValue"),
    ("InitiaSV", "Valor"): ("PosInteger", ""),
    ("Link", "Modo"): ("Listed", "LinkMode"),
    ("SaveValue", "Op"): ("Listed", "op_AddSub"),
    ("RemoveNumber", "Op"): ("Listed", "op_Relac"),
    ("SelectSNA", "op"): ("Listed", "op_Relac"),
    ("SelectS", "op"): ("Listed", "op_Logic_Stor"),
    ("SelectL", "op"): ("Listed", "op_Logic_Swich"),
    ("SelectF", "op"): ("Listed", "op_Logic_Faci"),
    ("GateF", "op"): ("Listed", "op_Logic_Faci"),
    ("GateS", "op"): ("Listed", "op_Logic_Stor"),
    ("GateL", "op"): ("Listed", "op_Logic_Swich"),
    ("GateM", "op"): ("Listed", "op_Logic_Tran"),
    ("FunctionExpon", "Seed"): ("PosInteger", ""),
    ("StorageDef", "Storage"): ("EntityDef", "Storage"),
    ("VariableN", "Media"): ("Expresion", "Expresion"),
    ("VariableN", "Desvio"): ("Expresion", "Expresion"),
    ("MFunction", "NombresFunc"): ("EntityDefList", "Function"),
    ("MFunction", "Matriz"): ("TSNAMatrix", ""),
    ("MFunctionDist", "NombresDist"): ("EntityDefList", "Function"),
    ("MFunctionDist", "Resultados"): ("FloatPercentMatrix", ""),
}

# Properties the macro code refers to but does not declare: the original
# creates them when the block is drawn.  Their shape is the one the original
# writes (see the reference model that uses `Multifurcacion`), and their
# values are the ones the class needs to run.
EXTRA_PROPS: dict[str, list[tuple[str, str, str, str, str, str]]] = {
    "Multifurcacion": [
        ("MultifurcacionCH", "EntityDef", "Function", "0", "1", "MultifurcacionCH"),
        ("MultifurcacionGO", "EntityDef", "Function", "0", "1", "MultifurcacionGO"),
        ("Prob1", "FloatPercent", "", "1", "0", "20%"),
        ("Prob2", "FloatPercent", "", "1", "0", "20%"),
        ("Prob3", "FloatPercent", "", "1", "0", "20%"),
        ("Prob4", "FloatPercent", "", "1", "0", "20%"),
        ("Prob5", "FloatPercent", "", "1", "0", "20%"),
    ],
    "MultifurcacionP": [
        ("MultifurcacionGO", "EntityDef", "Function", "0", "1", "MultifurcacionGO"),
        ("Parametro", "EntityUse", "Parameter", "1", "0", "Param"),
        ("Prob1", "FloatPercent", "", "1", "0", "20%"),
        ("Prob2", "FloatPercent", "", "1", "0", "20%"),
        ("Prob3", "FloatPercent", "", "1", "0", "20%"),
        ("Prob4", "FloatPercent", "", "1", "0", "20%"),
        ("Prob5", "FloatPercent", "", "1", "0", "20%"),
    ],
}

# Values the bench needs and the XML does not give.  The original refuses to
# simulate a variable with an empty equation, and refuses a match against a
# label that no block defines; the bench uses the label the class itself
# defines, and the smallest equation there is.
PROP_VALUES: dict[tuple[str, str], str] = {
    ("Variable", "Expresion"): "0",
    ("Match", "RotuloMatch"): "AAA",
    # The gate waits for the label the `Match` block in front of it creates.
    ("GateM", "Rotulo"): "AAA",
    ("Transfer", "Prob"): "100%",
}

# Outputs the original needs connected: the `Unlink` block carries in its
# second output the label of the transactions it unchains, and refuses to
# run without it.
EXTRA_OUTPUTS: dict[str, list[str]] = {
    "Unlink": ["SalidaAlt"],
    "Multifurcacion": ["SalidaProb2", "SalidaProb3", "SalidaProb4", "SalidaProb5"],
    "MultifurcacionP": ["SalidaProb2", "SalidaProb3", "SalidaProb4", "SalidaProb5"],
}

# The original refuses to end a transaction that owns a facility, and refuses
# a release of a facility nobody owns.  These classes only run next to the
# block that completes the pair.
COMPANIONS_BEFORE: dict[str, list[str]] = {
    "Release": ["Seize"],
    "Return": ["Seize"],
    # The gate waits for a label: only a `Match` block can create one.
    "GateM": ["Match"],
}

COMPANIONS_AFTER: dict[str, list[str]] = {
    "Seize": ["Release"],
    "Preempt": ["Return"],
}

# Classes that wait for a label: the link that feeds them has to carry the
# label they evaluate, or the original refuses the model.
LABELLED_INPUT: dict[str, str] = {}

# A `Listed` property with no default in the XML cannot run; the original
# picks the first option of the list.
LISTED_FIRST = {
    "op_Relac": "=",
    "op_RelacMaxMin": "=",
    "op_MaxMin": "Minimo",
    "op_Logic_Faci": "Libre",
    "op_Logic_Stor": "NoVacio",
    "op_Logic_Swich": "Set",
    "op_Swich": "Set",
    "op_AddSub": "+",
    "FuncTypes": "Continua",
    "LinkMode": "FIFO",
    "OperationMode": "Algunos",
    "PriorityOption": "Parametro",
    "PriorityMode": "Prioridad",
    "RemoveMode": "NoRemover",
    "entitySNAclass": "Q",
}

# ---------------------------------------------------------------------------
# The model format
# ---------------------------------------------------------------------------

PROP_RE = re.compile(r"\[([^\[\]:'=]+)(?::'([^']*)')?(?:='([^']*)')?\]")
RN_SEED_RE = re.compile(r"RN:(\d+)")


def encode(text: str) -> str:
    """Percent-encode a value the way the original stores it.

    Only ASCII letters, digits, `_` and `-` stay literal; everything else
    becomes `%XX` of its CP1252 byte.
    """
    out = []
    for byte in text.encode("cp1252", errors="replace"):
        char = chr(byte)
        if char.isascii() and (char.isalnum() or char in "_-"):
            out.append(char)
        else:
            out.append("%%%02X" % byte)
    return "".join(out)


def encode_hint(hint: str) -> str:
    """Encode a hint.

    Some classes store the hint already encoded (the original copies it into
    the model file verbatim), others store it as plain text.
    """
    if "%" in hint:
        return hint
    return encode(hint)


def unquote(text: str) -> str:
    """Undo the percent-encoding of the XML, as CP1252."""
    return urllib.parse.unquote_to_bytes(text).decode("cp1252")


def attr(value: str) -> str:
    """Escape a value for an XML attribute."""
    return (
        value.replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace('"', "&quot;")
    )


class Port:
    """One connection point of a class."""

    def __init__(self, name: str, caption: str) -> None:
        self.name = name
        self.caption = caption


class Prop:
    """One property of a class."""

    def __init__(
        self,
        name: str,
        hint: str,
        value: str,
        ty: str,
        subtype: str,
        visible: str = "1",
        unique: str = "0",
    ) -> None:
        self.name = name
        self.hint = hint
        self.value = value
        self.ty = ty
        self.subtype = subtype
        self.visible = visible
        self.unique = unique


class ClassDef:
    """One class of the original catalogue."""

    def __init__(self, name: str, width: int, height: int) -> None:
        self.name = name
        self.width = width
        self.height = height
        self.inputs: list[Port] = []
        self.outputs: list[Port] = []
        self.props: list[Prop] = []
        self.notes: list[str] = []


def parse_class(element: ET.Element) -> ClassDef:
    """Read one `<moduleclass>` element."""
    name = element.get("className") or ""
    cls = ClassDef(name, int(element.get("width") or 0), int(element.get("height") or 0))

    graph = element.find("graph")
    if graph is not None:
        for part in graph:
            kind = part.get("type")
            ident = part.get("ident") or ""
            if kind == "Input":
                cls.inputs.append(Port(ident, ""))
            elif kind == "Output":
                cls.outputs.append(Port(ident, ""))

    macro = element.find("macrocode")
    lines: list[str] = []
    if macro is not None:
        for line in macro:
            lines.append(unquote(line.get("text") or ""))
    # A line that ends with `...` continues on the next one.
    code = ""
    for line in lines:
        if line.endswith("..."):
            code += line[:-3] + "\n"
        else:
            code += line

    # Port captions: the macro code names every port with its caption.
    for port in cls.inputs + cls.outputs:
        match = re.search(r"(?<![\w])" + re.escape(port.name) + r":'([^']*)'", code)
        if match:
            port.caption = match.group(1)

    # Properties: the macro code declares them with a hint and a default.
    declared: dict[str, Prop] = {}
    for match in PROP_RE.finditer(code):
        prop_name, hint, value = match.group(1), match.group(2), match.group(3)
        if hint is None and value is None:
            # A plain `[name]`: a reference to a property, not a declaration.
            continue
        if prop_name in declared:
            continue
        if value is None:
            value = ""
        elif value.startswith("(") and value.endswith(")"):
            value = value[1:-1]
        declared[prop_name] = Prop(prop_name, hint or "", value, "", "")

    for prop_name, (ty, subtype) in _types_for(name, declared):
        prop = declared[prop_name]
        prop.ty = ty
        prop.subtype = subtype
        if (name, prop_name) in PROP_VALUES:
            prop.value = PROP_VALUES[(name, prop_name)]
        prop.unique = "1" if ty in ("EntityDef", "EntityDefList") else "0"
        if name == "QTable" and prop_name == "Queue":
            # The original writes this one as a definition but does not mark
            # it as unique.
            prop.unique = "0"
        if ty == "Listed" and not prop.value:
            prop.value = LISTED_FIRST.get(subtype, prop.value)

    # A seed the macro code names by number (`RN:2`) is a property of its own.
    for match in RN_SEED_RE.finditer(code):
        seed = match.group(1)
        if seed not in declared:
            declared[seed] = Prop(seed, "", seed, "EntityUse", "RandomSeed", "0", "0")

    for extra in EXTRA_PROPS.get(name, []):
        prop_name, ty, subtype, visible, unique, value = extra
        declared.setdefault(
            prop_name, Prop(prop_name, "", value, ty, subtype, visible, unique)
        )
        declared[prop_name].ty = ty
        declared[prop_name].subtype = subtype
        declared[prop_name].visible = visible
        declared[prop_name].unique = unique
        declared[prop_name].value = value

    # The original writes the properties sorted by name.
    cls.props = [declared[key] for key in sorted(declared)]
    return cls


def _types_for(class_name: str, declared: dict[str, Prop]) -> list[tuple[str, tuple[str, str]]]:
    """Type of every property of a class, class overrides first."""
    typed: list[tuple[str, tuple[str, str]]] = []
    for prop_name in declared:
        override = CLASS_PROP_TYPES.get((class_name, prop_name))
        if override is not None:
            typed.append((prop_name, override))
            continue
        default = PROP_TYPES.get(prop_name)
        if default is None:
            raise SystemExit(
                f"error: unknown property type for {class_name}.{prop_name}; "
                f"add it to PROP_TYPES"
            )
        typed.append((prop_name, default))
    return typed


def load_classes(path: Path) -> list[ClassDef]:
    """Read the original's class catalogue."""
    if not path.is_file():
        raise SystemExit(f"error: {path} not found (the original backup is required)")
    text = path.read_bytes().decode("cp1252")
    root = ET.fromstring(text)
    classes = []
    for element in root.find("gclasses") or []:
        classes.append(parse_class(element))
    return classes


# ---------------------------------------------------------------------------
# The bench
# ---------------------------------------------------------------------------

class Block:
    """One activity block of the bench."""

    def __init__(
        self,
        moduleid: int,
        cls: ClassDef,
        name: str,
        caption: str,
        x: int,
        y: int,
        inputs: list[tuple[int, str, str, str]],
        outputs: list[tuple[int, str, str, str]],
    ) -> None:
        self.moduleid = moduleid
        self.cls = cls
        self.name = name
        self.caption = caption
        self.x = x
        self.y = y
        self.inputs = inputs
        self.outputs = outputs


def build_bench(cls: ClassDef, classes: dict[str, ClassDef]) -> tuple[str, list[str]]:
    """Return the text of the bench of a class and the notes about it.

    The bench is a chain of blocks: a `Generate`, the blocks the class needs
    around it, the class under test, and a `Terminate`.  Every link of the
    chain is named after the block it feeds (`A_<block>`), and that name is
    what the original writes as the name of the GPSS block.
    """
    notes: list[str] = []
    blocks: list[Block] = []
    links: list[tuple[int, int, int, int]] = []
    extra_outputs = EXTRA_OUTPUTS.get(cls.name, [])

    # The flow of the class leaves by its `Salida1`; the other outputs are
    # alternates the original declares next to it.
    main = next(
        (index for index, port in enumerate(cls.outputs) if port.name == "Salida1"), 0
    )

    caption_notes = []
    if not cls.inputs and cls.outputs:
        caption_notes.append("sin puerto de entrada")
    if not cls.inputs and not cls.outputs:
        caption_notes.append("clase sin puertos: no se puede conectar")
    unconnected = [
        port.name
        for index, port in enumerate(cls.outputs)
        if index != main and port.name not in extra_outputs
    ]
    if unconnected:
        caption_notes.append("salidas sin conectar: " + ", ".join(unconnected))
    if COMPANIONS_BEFORE.get(cls.name) or COMPANIONS_AFTER.get(cls.name):
        companions = COMPANIONS_BEFORE.get(cls.name, []) + COMPANIONS_AFTER.get(cls.name, [])
        caption_notes.append("con bloque auxiliar: " + ", ".join(companions))
    caption = cls.name
    if caption_notes:
        caption = cls.name + " (" + "; ".join(caption_notes) + ")"

    # The chain of the bench.
    chain: list[tuple[str, ClassDef, str]] = []
    if cls.inputs and cls.name != "Generate":
        chain.append(("Generate", classes["Generate"], "Generate"))
    for companion in COMPANIONS_BEFORE.get(cls.name, []):
        chain.append((companion, classes[companion], companion))
    chain.append((cls.name, cls, caption))
    for companion in COMPANIONS_AFTER.get(cls.name, []):
        chain.append((companion, classes[companion], companion))
    if cls.outputs and cls.name != "Terminate":
        chain.append(("Terminate", classes["Terminate"], "Terminate"))

    # The blocks of the chain, with their ports and their labels.
    moduleids: dict[str, int] = {}
    for index, (name, definition, block_caption) in enumerate(chain):
        moduleids[name] = index + 1
        label_in = "A_" + name
        if definition is cls and cls.name in LABELLED_INPUT:
            # The class waits for a label: the link that feeds it carries it.
            label_in = LABELLED_INPUT[cls.name]
        label_out = "A_" + chain[index + 1][0] if index + 1 < len(chain) else ""
        inputs = []
        for port_index, port in enumerate(definition.inputs):
            inputs.append((port_index + 1, port.name, port.caption, label_in))
        outputs = []
        for port_index, port in enumerate(definition.outputs):
            if port_index == main and definition is cls:
                label = label_out
            elif port.name in extra_outputs and definition is cls:
                label = label_out + "_" + port.name
            else:
                label = label_out if port_index == main else ""
            outputs.append((len(definition.inputs) + port_index + 1, port.name, port.caption, label))
        blocks.append(
            Block(
                moduleids[name],
                definition,
                name,
                block_caption,
                120,
                40 + 100 * index,
                inputs,
                outputs,
            )
        )

    # The links of the chain.
    for index in range(len(chain) - 1):
        source, target = chain[index], chain[index + 1]
        source_main = next(
            (i for i, port in enumerate(source[1].outputs) if port.name == "Salida1"), 0
        )
        links.append(
            (
                moduleids[source[0]],
                len(source[1].inputs) + source_main + 1,
                moduleids[target[0]],
                1,
            )
        )

    # An output the original needs connected gets a terminator of its own.
    for index, port in enumerate(cls.outputs):
        if port.name not in extra_outputs:
            continue
        terminator = classes["Terminate"]
        label = "A_Terminate_" + port.name
        blocks.append(
            Block(
                len(blocks) + 1,
                terminator,
                "Terminate",
                "Terminate " + port.name,
                120,
                40 + 100 * len(blocks),
                [(1, item.name, item.caption, label) for item in terminator.inputs],
                [],
            )
        )
        links.append((moduleids[cls.name], len(cls.inputs) + index + 1, len(blocks), 1))

    # A `Clock` closes the run: the original stops when the start counter
    # reaches zero, and only the clock (or a terminate with an operand) does.
    if cls.name != "Clock":
        clock = classes["Clock"]
        blocks.append(Block(len(blocks) + 1, clock, "Clock", "Clock", 420, 40, [], []))

    text = render(blocks, links)
    if not cls.inputs and not cls.outputs:
        notes.append("la clase no declara puertos: queda en el modelo sin conectar")
    if unconnected:
        notes.append("salidas sin conectar: " + ", ".join(unconnected))
    for companion in COMPANIONS_BEFORE.get(cls.name, []):
        notes.append(f"el banco antepone un bloque {companion}: hace falta para que la classe corra")
    for companion in COMPANIONS_AFTER.get(cls.name, []):
        notes.append(f"el banco añade un bloque {companion}: hace falta para que la classe corra")
    return text, notes


def render(blocks: list[Block], links: list[tuple[int, int, int, int]]) -> str:
    """Write the bench as a `.gmdl` file, in the original's own shape."""
    out = [f'<gmodel versioncode="{MODEL_VERSION}"><gclasses/><modules>']
    out.append(
        '<module moduleid="0" moduleclass="main" owner="0" caption="Modelo" '
        'name="Modelo1" kind="C" genprefer="1,1,1,1,1,0,0,0,0,0,0,0,0" '
        'exehead="" nonexehead="" exefoot="" nonexefoot="">'
    )
    if links:
        out.append("<links>")
        for from_module, from_ioid, to_module, to_ioid in links:
            out.append(
                f'<link frommoduleid="{from_module}" fromioid="{from_ioid}" '
                f'tomoduleid="{to_module}" toioid="{to_ioid}"/>'
            )
        out.append("</links>")
    out.append("<inputs/><outputs/></module>")

    for block in blocks:
        cls = block.cls
        out.append(
            f'<module moduleid="{block.moduleid}" moduleclass="{attr(cls.name)}" '
            f'owner="0" caption="{attr(encode(block.caption))}" '
            f'name="{attr(block.name)}" kind="A" width="{cls.width}" '
            f'height="{cls.height}" x="{block.x}" y="{block.y}">'
        )
        if block.inputs:
            out.append("<inputs>")
            for ioid, name, caption, label in block.inputs:
                out.append(
                    f'<input ioid="{ioid}" name="{attr(name)}" '
                    f'caption="{attr(encode(caption))}" mylabel="{attr(label)}"/>'
                )
            out.append("</inputs>")
        if block.outputs:
            out.append("<outputs>")
            for ioid, name, caption, label in block.outputs:
                out.append(
                    f'<output ioid="{ioid}" name="{attr(name)}" '
                    f'caption="{attr(encode(caption))}" mylabel="{attr(label)}"/>'
                )
            out.append("</outputs>")
        if cls.props:
            out.append("<props>")
            for prop in cls.props:
                out.append(
                    f'<property name="{attr(prop.name)}" hint="{attr(encode_hint(prop.hint))}" '
                    f'value="{attr(encode(prop.value))}" type="{attr(prop.ty)}" '
                    f'subType="{attr(prop.subtype)}" visible="{prop.visible}" '
                    f'unique="{prop.unique}"/>'
                )
            out.append("</props>")
        out.append("</module>")

    out.append("</modules></gmodel>")
    return "".join(out) + "\r\n"


def safe_name(class_name: str) -> str:
    """A file name for a class."""
    return re.sub(r"[^A-Za-z0-9._-]", "_", class_name)


def main(argv: list[str]) -> int:
    outdir = Path(argv[1]) if len(argv) > 1 else DEFAULT_OUT
    if not outdir.is_absolute():
        outdir = (Path.cwd() / outdir).resolve()
    outdir.mkdir(parents=True, exist_ok=True)

    classes = load_classes(CLASSES_XML)
    by_name = {cls.name: cls for cls in classes}

    written = 0
    without_ports = []
    for cls in sorted(classes, key=lambda item: item.name):
        text, notes = build_bench(cls, by_name)
        path = outdir / f"{safe_name(cls.name)}.gmdl"
        path.write_bytes(text.encode("utf-8"))
        written += 1
        if not cls.inputs and not cls.outputs:
            without_ports.append(cls.name)
        for note in notes:
            print(f"{cls.name}: {note}")

    print(f"wrote {written} benches to {outdir}")
    print(f"classes without ports: {', '.join(without_ports) if without_ports else 'none'}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
