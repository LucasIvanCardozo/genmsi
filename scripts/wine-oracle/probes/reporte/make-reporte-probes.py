#!/usr/bin/env python3
"""Build the probes that measure the group and table sections of the report.

Two questions the original's report answers but nobody has measured yet:

1. What does `GROUP_SIZE` of the `XACT_GROUP` and `NUMERIC_GROUP` sections
   count?  The number of joins, the number of live members, the sum of the
   numbers of a numeric group, or the number of distinct values?

2. In the `TABLAS` section, which class of the table holds the values that
   fall beyond the last interval, and what does `CANTIDAD EN RETRY` count?

The probes are plain `.gmdl` models, the same shape the original writes.
Every property (its name, its type, its hint and its default) is read from
the original's own class catalogue (`Genmsi_20230711.backup/gClasses.xml`)
through `make-benches.py`, so the models only carry the values a probe
needs.

Usage:
    python3 scripts/wine-oracle/probes/reporte/make-reporte-probes.py
"""

from __future__ import annotations

import importlib.util
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[3]

# `make-benches.py` reads the original's class catalogue; it is the tool
# this repository already uses to write models the original accepts.
_spec = importlib.util.spec_from_file_location(
    "make_benches", REPO / "scripts" / "wine-oracle" / "make-benches.py"
)
make_benches = importlib.util.module_from_spec(_spec)
sys.modules["make_benches"] = make_benches
_spec.loader.exec_module(make_benches)

CLASSES = make_benches.load_classes(make_benches.CLASSES_XML)
BY_NAME = {cls.name: cls for cls in CLASSES}

MODEL_VERSION = "1.00.2008.03.23"


def attr(value: str) -> str:
    return (
        value.replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace('"', "&quot;")
    )


class Block:
    """One block of a probe model.

    The block owns its ports: the label a port points to belongs to the
    block, not to the shared class definition.
    """

    def __init__(
        self,
        moduleid: int,
        cls_name: str,
        name: str,
        values: dict[str, str],
        x: int = 120,
        y: int = 40,
        caption: str | None = None,
    ) -> None:
        self.moduleid = moduleid
        self.cls = BY_NAME[cls_name]
        self.name = name
        self.values = values
        self.x = x
        self.y = y
        self.caption = caption or cls_name
        self.label = make_benches.label_of(self.cls, self.name)
        self.inputs = [(port.name, port.caption) for port in self.cls.inputs]
        self.outputs = [(port.name, port.caption, "") for port in self.cls.outputs]

    def connect(self, ioid: int, target_label: str) -> None:
        """Point the output `ioid` of this block to `target_label`."""
        index = ioid - len(self.inputs) - 1
        name, caption, _ = self.outputs[index]
        self.outputs[index] = (name, caption, target_label)

    def render(self) -> str:
        out = [
            f'<module moduleid="{self.moduleid}" moduleclass="{self.cls.name}" '
            f'owner="0" caption="{attr(make_benches.encode(self.caption))}" '
            f'name="{attr(self.name)}" kind="A" width="{self.cls.width}" '
            f'height="{self.cls.height}" x="{self.x}" y="{self.y}">'
        ]
        if self.inputs:
            out.append("<inputs>")
            for index, (name, caption) in enumerate(self.inputs):
                out.append(
                    f'<input ioid="{index + 1}" name="{attr(name)}" '
                    f'caption="{attr(make_benches.encode(caption))}" '
                    f'mylabel="{attr(self.label)}"/>'
                )
            out.append("</inputs>")
        if self.outputs:
            out.append("<outputs>")
            base = len(self.inputs)
            for index, (name, caption, target) in enumerate(self.outputs):
                out.append(
                    f'<output ioid="{base + index + 1}" name="{attr(name)}" '
                    f'caption="{attr(make_benches.encode(caption))}" '
                    f'mylabel="{attr(target)}"/>'
                )
            out.append("</outputs>")
        if self.cls.props:
            out.append("<props>")
            for prop in self.cls.props:
                value = self.values.get(prop.name, prop.value)
                out.append(
                    f'<property name="{attr(prop.name)}" '
                    f'hint="{attr(make_benches.encode_hint(prop.hint))}" '
                    f'value="{attr(make_benches.encode(value))}" '
                    f'type="{attr(prop.ty)}" subType="{attr(prop.subtype)}" '
                    f'visible="{prop.visible}" unique="{prop.unique}"/>'
                )
            out.append("</props>")
        out.append("</module>")
        return "".join(out)


def build(blocks: list[Block], links: list[tuple[int, int, int, int]]) -> str:
    """Write the model, giving every output the label of the block it feeds."""
    for from_module, from_ioid, to_module, _ in links:
        source = next(b for b in blocks if b.moduleid == from_module)
        target = next(b for b in blocks if b.moduleid == to_module)
        source.connect(from_ioid, target.label)
    out = [f'<gmodel versioncode="{MODEL_VERSION}"><gclasses/><modules>']
    out.append(
        '<module moduleid="0" moduleclass="main" owner="0" caption="Modelo" '
        'name="Modelo1" kind="C" genprefer="1,1,1,1,1,0,0,0,0,0,0,0,0" '
        'exehead="" nonexehead="" exefoot="" nonexefoot="">'
    )
    if links:
        out.append("<links>")
        for from_module, from_ioid, to_module, to_oid in links:
            out.append(
                f'<link frommoduleid="{from_module}" fromioid="{from_ioid}" '
                f'tomoduleid="{to_module}" toioid="{to_oid}"/>'
            )
        out.append("</links>")
    out.append("<inputs/><outputs/></module>")
    for block in blocks:
        out.append(block.render())
    out.append("</modules></gmodel>")
    return "".join(out) + "\r\n"


# --- the probes ------------------------------------------------------------

GENERATE_VALUES = {"Cantidad": "", "Media": "100", "Modificador": "0"}
CLOCK_VALUES = {"Frecuencia": "3600"}


def probe_numeric(group: str, numbers: list[str]) -> str:
    """`Generate` -> one `JoinNumber` per number -> `Terminate` + `Clock`."""
    names = ["JoinNumber", "JoinNumber1", "JoinNumber2"]
    blocks = [Block(1, "Generate", "Generate", GENERATE_VALUES)]
    links = []
    for index, number in enumerate(numbers):
        blocks.append(
            Block(2 + index, "JoinNumber", names[index], {"Grupo": group, "Numero": number})
        )
    last_join = 1 + len(numbers)
    blocks.append(Block(last_join + 1, "Terminate", "Terminate", {}))
    blocks.append(Block(last_join + 2, "Clock", "Clock", CLOCK_VALUES, x=420))
    links.append((1, 1, 2, 1))
    for index in range(len(numbers) - 1):
        links.append((2 + index, 2, 3 + index, 1))
    links.append((last_join, 2, last_join + 1, 1))
    return build(blocks, links)


def probe_transaction(group: str) -> str:
    """`Generate` -> `JoinMe` -> `Advance` -> `Terminate` + `Clock`."""
    blocks = [
        Block(1, "Generate", "Generate", GENERATE_VALUES),
        Block(2, "JoinMe", "JoinMe", {"Grupo": group}),
        Block(3, "Advance", "Advance", {"Media": "3000", "Modificador": "0"}),
        Block(4, "Terminate", "Terminate", {}),
        Block(5, "Clock", "Clock", CLOCK_VALUES, x=420),
    ]
    links = [(1, 1, 2, 1), (2, 2, 3, 1), (3, 2, 4, 1)]
    return build(blocks, links)


def probe_tables(
    name1: str = "TBL1",
    name2: str = "TBL2",
    atributo1: str = "P:Atrib",
    atributo2: str = "P:Atrib",
    parametro1: str = "Atrib",
    parametro2: str = "Atrib",
    valor1: str = "5",
    valor2: str = "95",
) -> str:
    """Two tables, two flows that tabulate into them.

    The original refuses a model whose *entity* names are shorter than
    three characters: a table called "T1" makes it answer "El modelo no
    puede abrirse porque el archivo presenta errores estructurales."  The
    tables are therefore called "TBL1" and "TBL2"; `tablas_t1_t2.gmdl`
    keeps the literal "T1"/"T2" model for the record.
    """
    blocks = [
        Block(
            1,
            "Table",
            "Table",
            {
                "NombreTabla": name1,
                "Atributo": atributo1,
                "PrimerIntervalo": "0",
                "TamFrecuencia": "10",
                "Cantidad": "10",
            },
            x=420,
        ),
        Block(
            2,
            "Table",
            "Table1",
            {
                "NombreTabla": name2,
                "Atributo": atributo1,
                "PrimerIntervalo": "0",
                "TamFrecuencia": "10",
                "Cantidad": "10",
            },
            x=420,
            y=140,
        ),
        Block(3, "Generate", "Generate", GENERATE_VALUES),
        Block(
            4,
            "Assign",
            "Assign",
            {"Parametro": parametro1, "Valor": valor1, "Operador": "="},
        ),
        Block(5, "Tabulate", "Tabulate", {"NombreTabla": name1, "Peso": "1"}),
        Block(6, "Terminate", "Terminate", {}),
        Block(7, "Generate", "Generate1", GENERATE_VALUES, y=140),
        Block(
            8,
            "Assign",
            "Assign1",
            {"Parametro": parametro2, "Valor": valor2, "Operador": "="},
            y=140,
        ),
        Block(9, "Tabulate", "Tabulate1", {"NombreTabla": name2, "Peso": "1"}, y=140),
        Block(10, "Terminate", "Terminate1", {}, y=140),
        Block(11, "Clock", "Clock", CLOCK_VALUES, x=700),
    ]
    links = [
        (3, 1, 4, 1),
        (4, 2, 5, 1),
        (5, 2, 6, 1),
        (7, 1, 8, 1),
        (8, 2, 9, 1),
        (9, 2, 10, 1),
    ]
    return build(blocks, links)


def main() -> int:
    HERE.mkdir(parents=True, exist_ok=True)
    probes = {
        "grupo_numerico.gmdl": probe_numeric("GNUM", ["1", "2", "3"]),
        "grupo_numerico_repetido.gmdl": probe_numeric("GNUM", ["2", "2", "2"]),
        "grupo_transaccion.gmdl": probe_transaction("GTRANS"),
        "tablas.gmdl": probe_tables(),
        "tablas_t1_t2.gmdl": probe_tables("T1", "T2"),
        # The table tabulates a parameter nobody assigns: what does the
        # original report (MEDIA, CANTIDAD EN RETRY)?
        "tablas_atributo_inexistente.gmdl": probe_tables(
            "TBL1", "TBL2", atributo1="P:NoExiste", parametro1="Atrib"
        ),
        # The table tabulates "Atrib" but the flow assigns "Param": the
        # Atributo names the parameter the table tabulates.
        "tablas_parametro_distinto.gmdl": probe_tables(
            "TBL1", "TBL2", parametro1="Param"
        ),
    }
    for name, text in probes.items():
        path = HERE / name
        path.write_bytes(text.encode("utf-8"))
        print(f"wrote {path} ({len(text)} bytes)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
