#!/usr/bin/env python3
"""Build the probe models that measure how the original compiles the
`SELECT` family and the `EXAMINE`.

Every model is built with the machinery of `scripts/wine-oracle/make-benches.py`
(the class catalogue of the original, `Genmsi_20230711.backup/gClasses.xml`),
so the ports, the properties, their hints and their types are exactly the
ones the original itself writes: a model whose module properties do not match
the class definition makes the original open the "adaptation" dialog and the
harness hangs on it.

What is built:

* `select/<group>.gmdl`  — one model per group of entity classes, each class
  exercised by a `Select` module (`ClaseSNA` = the class, `op` = Minimo,
  `Param` = param, `minimo` = 1, `maximo` = 10) chained in one flow:

      Generate -> Select(clase1) -> ... -> Select(claseN) -> Terminate

  plus a `Clock`, which is what ends the run.

* `sels_<op>.gmdl`       — one model per operator of `op_Logic_Stor`
  (SF, SNF, SE, SNE, SV, SNV) in the flow Generate -> SelectS -> Terminate.
  The variant `sels_<op>_libre.gmdl` leaves the second output (`Salida2`,
  "No selec") unconnected, to see what the original writes when it has no
  label to write.

* `examine_probe.gmdl`  — Generate -> Examine -> Terminate, with the failure
  output (`Salida2`) of the `Examine` connected to a second `Terminate`.

Usage: python3 scripts/wine-oracle/probes/select/make-select-probes.py
"""

from __future__ import annotations

import importlib.util
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[3]

# The class catalogue of the original and the machinery that reads it.
_spec = importlib.util.spec_from_file_location(
    "make_benches", REPO / "scripts" / "wine-oracle" / "make-benches.py"
)
assert _spec and _spec.loader
mb = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(mb)

CLASSES = {cls.name: cls for cls in mb.load_classes(mb.CLASSES_XML)}

# The 38 entity classes of the original (`entitySNAclass` in
# `Genmsi_20230711.backup/gmacroconfig.gmcfg`), eight per model.
CLASS_GROUPS: list[list[str]] = [
    ["SR", "FR", "CH", "CC", "CA", "QZ", "GN", "W"],
    ["N", "SC", "FC", "GT", "QC", "CM", "QM", "SM"],
    ["QA", "SA", "MP", "FV", "LR", "LS", "RN", "S"],
    ["F", "Q", "R", "BV", "FN", "V", "SV", "FT"],
    ["QT", "QX", "CT", "ST", "X", "P"],
]

ALL_CLASSES: list[str] = [code for group in CLASS_GROUPS for code in group]

# The six operators of `op_Logic_Stor`: the key of the list and the text the
# original writes on the property.
STOR_OPS: list[tuple[str, str]] = [
    ("SF", "Lleno"),
    ("SNF", "NoLleno"),
    ("SE", "Vacio"),
    ("SNE", "NoVacio"),
    ("SV", "Disponible"),
    ("SNV", "NoDisponible"),
]


def clone(cls, **values: str):
    """A copy of a class with some property values changed."""
    copy = mb.ClassDef(cls.name, cls.width, cls.height)
    copy.inputs = list(cls.inputs)
    copy.outputs = list(cls.outputs)
    for prop in cls.props:
        copy.props.append(
            mb.Prop(
                prop.name,
                prop.hint,
                values.get(prop.name, prop.value),
                prop.ty,
                prop.subtype,
                prop.visible,
                prop.unique,
            )
        )
    return copy


def chain_model(specs, clock: bool = True):
    """A flow of blocks: `specs` is a list of (class, name, values).

    The shape of a block is the one the original writes: the `ioid` of a port
    counts the inputs first, the `mylabel` of an input is the label of the
    block itself and the `mylabel` of an output is the label of the block the
    flow continues in.
    """
    blocks = []
    links = []
    ids: dict[str, int] = {}

    for index, (cls_name, module_name, values) in enumerate(specs):
        cls = clone(CLASSES[cls_name], **values) if values else CLASSES[cls_name]
        moduleid = index + 1
        ids[module_name] = moduleid
        label = "A_" + module_name
        inputs = [(i + 1, p.name, p.caption, label) for i, p in enumerate(cls.inputs)]
        blocks.append(mb.Block(moduleid, cls, module_name, cls_name, 120, 40 + 100 * index, inputs, []))

    # The flow: every block leaves by its `Salida1` into the next one.
    for index in range(len(specs) - 1):
        source = specs[index]
        target = specs[index + 1]
        source_cls = CLASSES[source[0]]
        block = blocks[ids[source[1]] - 1]
        block.outputs = [
            (len(source_cls.inputs) + i + 1, p.name, p.caption, "A_" + target[1])
            for i, p in enumerate(source_cls.outputs)
        ]
        links.append(
            (
                ids[source[1]],
                len(source_cls.inputs) + mb.main_output(source_cls) + 1,
                ids[target[1]],
                1,
            )
        )

    if clock:
        blocks.append(mb.Block(len(blocks) + 1, CLASSES["Clock"], "Clock", "Clock", 420, 40, [], []))
    return mb.render(blocks, links)


def select_group(group: list[str]) -> str:
    """One model: a chain of `Select` modules, one per entity class."""
    specs = [("Generate", "Generate", {})]
    for index, code in enumerate(group, start=1):
        specs.append(
            (
                "Select",
                f"Select{index}",
                {
                    "ClaseSNA": code,
                    "op": "Minimo",
                    "Param": "param",
                    "minimo": "1",
                    "maximo": "10",
                },
            )
        )
    specs.append(("Terminate", "Terminate", {}))
    return chain_model(specs)


def select_s(op_text: str, connect_failure: bool) -> str:
    """One model: Generate -> SelectS -> Terminate.

    The second output of the `SelectS` ("No selec") points to a second
    `Terminate` when `connect_failure` is true, and stays unconnected
    otherwise.
    """
    names = ["Generate", "SelectS", "Terminate"] + (["Terminate1"] if connect_failure else [])
    blocks = []
    links = []
    ids = {}
    for index, module_name in enumerate(names):
        cls = {"Generate": CLASSES["Generate"], "SelectS": clone(CLASSES["SelectS"], op=op_text)}.get(
            module_name, CLASSES["Terminate"]
        )
        moduleid = index + 1
        ids[module_name] = moduleid
        label = "A_" + module_name
        inputs = [(i + 1, p.name, p.caption, label) for i, p in enumerate(cls.inputs)]
        blocks.append(mb.Block(moduleid, cls, module_name, module_name, 120, 40 + 100 * index, inputs, []))

    blocks[ids["Generate"] - 1].outputs = [
        (1, p.name, p.caption, "A_SelectS") for p in CLASSES["Generate"].outputs
    ]
    blocks[ids["SelectS"] - 1].outputs = [
        (2, "Salida1", "", "A_Terminate"),
        (3, "Salida2", "No selec", "A_Terminate1" if connect_failure else ""),
    ]
    links.append((ids["Generate"], 1, ids["SelectS"], 1))
    links.append((ids["SelectS"], 2, ids["Terminate"], 1))
    if connect_failure:
        links.append((ids["SelectS"], 3, ids["Terminate1"], 1))
    blocks.append(mb.Block(len(blocks) + 1, CLASSES["Clock"], "Clock", "Clock", 420, 40, [], []))
    return mb.render(blocks, links)


def select_dest(cls_name: str, values: dict[str, str]) -> str:
    """One model: Generate -> SelectX -> Terminate, Salida2 -> Terminate1.

    The failure output (`Salida2`, "No selec") of the classes of the family
    is connected, which is what makes the original write it in the listing.
    """
    cls = clone(CLASSES[cls_name], **values)
    blocks = []
    links = []
    ids = {}
    names = ["Generate", "Select", "Terminate", "Terminate1"]
    for index, module_name in enumerate(names):
        if module_name == "Select":
            definition = cls
        elif module_name == "Generate":
            definition = CLASSES["Generate"]
        else:
            definition = CLASSES["Terminate"]
        moduleid = index + 1
        ids[module_name] = moduleid
        label = "A_" + module_name
        inputs = [(i + 1, p.name, p.caption, label) for i, p in enumerate(definition.inputs)]
        blocks.append(mb.Block(moduleid, definition, module_name, module_name, 120, 40 + 100 * index, inputs, []))
    blocks[ids["Generate"] - 1].outputs = [
        (1, p.name, p.caption, "A_Select") for p in CLASSES["Generate"].outputs
    ]
    blocks[ids["Select"] - 1].outputs = [
        (2, "Salida1", "", "A_Terminate"),
        (3, "Salida2", "No selec", "A_Terminate1"),
    ]
    links.append((ids["Generate"], 1, ids["Select"], 1))
    links.append((ids["Select"], 2, ids["Terminate"], 1))
    links.append((ids["Select"], 3, ids["Terminate1"], 1))
    blocks.append(mb.Block(len(blocks) + 1, CLASSES["Clock"], "Clock", "Clock", 420, 40, [], []))
    return mb.render(blocks, links)


def select_s_storage(
    ops: list[str], capacity: int, filler: int, storage: str = "Almacen"
) -> str:
    """One model: a declared storage in one state and the `SelectS` of each
    operator, with the failure output connected.

    The storage is declared (`StorageDef`) because the original does not run
    a `SelectS` that asks for a storage nobody declared.  `filler` is the
    number of units a flow puts into it and keeps there: 0 leaves it empty,
    1 leaves one unit in it.  `storage` is the name of the storage: the
    `SELECT` walks the entities whose number falls in the range of the
    statement, so a storage named `1` is the one the range `1,10` covers.
    """
    blocks = []
    links = []
    next_id = 1
    y = 40

    blocks.append(
        mb.Block(next_id, clone(CLASSES["StorageDef"], Storage=storage, Tamaño=str(capacity)), "StorageDef", "StorageDef", 120, y, [], [])
    )
    next_id += 1

    if filler > 0:
        # A flow that enters one unit and never leaves it: the storage holds
        # one unit from the first arrival on.
        blocks.append(
            mb.Block(next_id, clone(CLASSES["Generate"], Cantidad="1", Media="10"), "Generate", "Generate", 120, y, [], [(1, "Salida1", "", "A_Enter")])
        )
        links.append((next_id, 1, next_id + 1, 1))
        next_id += 1
        blocks.append(
            mb.Block(next_id, clone(CLASSES["Enter"], Cantidad="1", Storage=storage), "Enter", "Enter", 120, y, [(1, "Entrada1", "Ingreso", "A_Enter")], [])
        )
        next_id += 1

    for index, op in enumerate(ops):
        name = f"SelectS{index}"
        blocks.append(
            mb.Block(next_id, clone(CLASSES["Generate"], Media="60"), f"Generate{index}", "Generate", 120, y, [], [(1, "Salida1", "", f"A_{name}")])
        )
        links.append((next_id, 1, next_id + 1, 1))
        next_id += 1
        select_id = next_id
        blocks.append(
            mb.Block(next_id, clone(CLASSES["SelectS"], op=op), name, "SelectS", 120, y, [(1, "Entrada1", "Ingreso", f"A_{name}")], [])
        )
        next_id += 1
        # The two outputs of the `SelectS`: the one it selects by and the one
        # it fails by ("No selec").  Each one gets a terminator of its own.
        terms = []
        for port in (2, 3):
            number = index * 2 + port - 1
            term_name = "Terminate" if number == 1 else f"Terminate{number}"
            blocks.append(
                mb.Block(next_id, CLASSES["Terminate"], term_name, "Terminate", 120, y, [(1, "Entrada1", "Ingreso", f"A_{term_name}")], [])
            )
            links.append((select_id, port, next_id, 1))
            terms.append(f"A_{term_name}")
            next_id += 1
        blocks[select_id - 1].outputs = [
            (2, "Salida1", "", terms[0]),
            (3, "Salida2", "No selec", terms[1]),
        ]

    blocks.append(mb.Block(next_id, CLASSES["Clock"], "Clock", "Clock", 420, 40, [], []))
    return mb.render(blocks, links)


def select_s_parameter(op_text: str, capacity: int, filler: int, storage: str = "1") -> str:
    """One model that says whether the selection found an entity.

    The `SelectS` writes the number of the entity it chose in a parameter,
    and the manual says what it writes when it finds none: "Si la entidad
    no es encontrada, 0 es colocado en el parámetro de la transacción
    activa".  A `Test` after the selection compares that parameter with one,
    so the report says how many transactions found an entity and how many
    did not.
    """
    blocks = []
    links = []
    next_id = 1
    y = 40

    blocks.append(
        mb.Block(next_id, clone(CLASSES["StorageDef"], Storage=storage, Tamaño=str(capacity)), "StorageDef", "StorageDef", 120, y, [], [])
    )
    next_id += 1

    if filler > 0:
        blocks.append(
            mb.Block(next_id, clone(CLASSES["Generate"], Cantidad="1", Media="10"), "Generate", "Generate", 120, y, [], [(1, "Salida1", "", "A_Enter")])
        )
        links.append((next_id, 1, next_id + 1, 1))
        next_id += 1
        blocks.append(
            mb.Block(next_id, clone(CLASSES["Enter"], Cantidad="1", Storage=storage), "Enter", "Enter", 120, y, [(1, "Entrada1", "Ingreso", "A_Enter")], [])
        )
        next_id += 1

    blocks.append(
        mb.Block(next_id, clone(CLASSES["Generate"], Media="60"), "Generate", "Generate", 120, y, [], [(1, "Salida1", "", "A_SelectS")])
    )
    links.append((next_id, 1, next_id + 1, 1))
    next_id += 1
    blocks.append(
        mb.Block(next_id, clone(CLASSES["SelectS"], op=op_text, Param="ParamSel"), "SelectS", "SelectS", 120, y, [(1, "Entrada1", "Ingreso", "A_SelectS")], [(2, "Salida1", "", "A_Test"), (3, "Salida2", "No selec", "")])
    )
    links.append((next_id, 2, next_id + 1, 1))
    next_id += 1
    blocks.append(
        mb.Block(next_id, clone(CLASSES["Test"], **{"1_Valor": "P:ParamSel", "2_OP": "=", "3_Valor": "1"}), "Test", "Test", 120, y, [(1, "Entrada1", "Entrada1", "A_Test")], [])
    )
    test_id = next_id
    next_id += 1
    for port, term_name in ((2, "Terminate"), (3, "Terminate1")):
        blocks.append(
            mb.Block(next_id, CLASSES["Terminate"], term_name, "Terminate", 120, y, [(1, "Entrada1", "Ingreso", f"A_{term_name}")], [])
        )
        links.append((test_id, port, next_id, 1))
        next_id += 1
    blocks[test_id - 1].outputs = [
        (2, "Salida1", "Verdadero", "A_Terminate"),
        (3, "Salida2", "Falso", "A_Terminate1"),
    ]
    blocks.append(mb.Block(next_id, CLASSES["Clock"], "Clock", "Clock", 420, 40, [], []))
    return mb.render(blocks, links)


def examine_probe(connect_failure: bool = True) -> str:
    """One model: Generate -> Examine -> Terminate, Salida2 -> Terminate1.

    With `connect_failure` false the failure output of the `Examine` stays
    unconnected, which is what makes the original refuse the model.
    """
    specs = [
        ("Generate", "Generate", {}),
        ("Examine", "Examine", {}),
        ("Terminate", "Terminate", {}),
        ("Terminate", "Terminate1", {}),
    ]
    blocks = []
    links = []
    ids = {}
    for index, (cls_name, module_name, values) in enumerate(specs):
        cls = CLASSES[cls_name]
        moduleid = index + 1
        ids[module_name] = moduleid
        label = "A_" + module_name
        inputs = [(i + 1, p.name, p.caption, label) for i, p in enumerate(cls.inputs)]
        blocks.append(mb.Block(moduleid, cls, module_name, cls_name, 120, 40 + 100 * index, inputs, []))
    # Generate -> Examine, Examine -> Terminate, Examine -> Terminate1.
    links.append((ids["Generate"], 1, ids["Examine"], 1))
    links.append((ids["Examine"], 2, ids["Terminate"], 1))
    blocks[ids["Generate"] - 1].outputs = [
        (1, p.name, p.caption, "A_Examine") for p in CLASSES["Generate"].outputs
    ]
    blocks[ids["Examine"] - 1].outputs = [
        (2, "Salida1", "En [Grupo]", "A_Terminate"),
        (3, "Salida2", "No en [Grupo]", "A_Terminate1" if connect_failure else ""),
    ]
    if connect_failure:
        links.append((ids["Examine"], 3, ids["Terminate1"], 1))
    blocks.append(mb.Block(len(blocks) + 1, CLASSES["Clock"], "Clock", "Clock", 420, 40, [], []))
    return mb.render(blocks, links)


def select_alone(code: str) -> str:
    """One model with the `Select` of a class out of the flow.

    Some classes make the original crash (an access violation in `genmsi.exe`)
    as soon as a transaction reaches the `Select`; the listing is written
    before the simulation, so the same line is measured with the `Select`
    present in the model but not connected to the flow.
    """
    cls = clone(CLASSES["Select"], ClaseSNA=code, op="Minimo", Param="param", minimo="1", maximo="10")
    blocks = [
        mb.Block(1, CLASSES["Generate"], "Generate", "Generate", 120, 40, [], [(1, "Salida1", "", "A_Terminate")]),
        mb.Block(2, CLASSES["Terminate"], "Terminate", "Terminate", 120, 140, [(1, "Entrada1", "Ingreso", "A_Terminate")], []),
        mb.Block(3, CLASSES["Clock"], "Clock", "Clock", 420, 40, [], []),
        mb.Block(4, cls, f"Select{code}", "Select", 120, 240, [(1, "Entrada1", "Ingreso", f"A_Select{code}")], [(2, "Salida1", "", "")]),
    ]
    links = [(1, 1, 2, 1)]
    return mb.render(blocks, links)


def main() -> None:
    out = HERE / "models"
    out.mkdir(parents=True, exist_ok=True)

    for index, group in enumerate(CLASS_GROUPS, start=1):
        path = out / f"select_g{index}.gmdl"
        path.write_bytes(select_group(group).encode("utf-8"))
        print(f"wrote {path}  ({', '.join(group)})")

    # One model per class: the groups are the fast path, but a class that
    # makes the original crash takes the whole group down with it, so every
    # class also gets a model of its own.
    for code in ALL_CLASSES:
        path = out / f"select_{code}.gmdl"
        path.write_bytes(select_group([code]).encode("utf-8"))
        print(f"wrote {path}  ({code})")
        path = out / f"select_{code}_alone.gmdl"
        path.write_bytes(select_alone(code).encode("utf-8"))
        print(f"wrote {path}  ({code}, Select out of the flow)")

    for key, text in STOR_OPS:
        path = out / f"sels_{key}.gmdl"
        path.write_bytes(select_s(text, connect_failure=True).encode("utf-8"))
        print(f"wrote {path}  (op {key} = {text}, Salida2 connected)")
        path = out / f"sels_{key}_libre.gmdl"
        path.write_bytes(select_s(text, connect_failure=False).encode("utf-8"))
        print(f"wrote {path}  (op {key} = {text}, Salida2 unconnected)")

    path = out / "examine_probe.gmdl"
    path.write_bytes(examine_probe().encode("utf-8"))
    print(f"wrote {path}")

    path = out / "examine_sin_destino.gmdl"
    path.write_bytes(examine_probe(connect_failure=False).encode("utf-8"))
    print(f"wrote {path}  (the failure output of the Examine is unconnected)")

    # The failure output of the other classes of the family, connected: what
    # the original writes in the listing when it has a destination to write.
    for cls_name, values in (
        ("SelectF", {"op": "Libre", "Param": "param", "minimo": "1", "maximo": "10"}),
        ("SelectL", {"op": "Set", "Param": "param", "minimo": "1", "maximo": "10"}),
        ("SelectSNA", {"op": "=", "Param": "param", "minimo": "1", "maximo": "10", "valorComp": "*1", "ClaseSNA": "Q"}),
    ):
        path = out / f"select_{cls_name.lower()}_dest.gmdl"
        path.write_bytes(select_dest(cls_name, values).encode("utf-8"))
        print(f"wrote {path}  ({cls_name}, Salida2 connected)")

    # The `Disponible` and the `NoDisponible` of a storage: the storage is
    # declared and in one of three states (empty, one unit in it, full).
    for state, capacity, filler in (
        ("vacio", 10, 0),
        ("uno", 10, 1),
        ("lleno", 1, 1),
    ):
        path = out / f"sels_sv_{state}.gmdl"
        path.write_bytes(
            select_s_storage(["Disponible", "NoDisponible"], capacity, filler).encode("utf-8")
        )
        print(f"wrote {path}  (storage {state})")

    # The same, with the storage named `1`: the `SELECT` walks the entities
    # whose number falls in the range of the statement (`1,10`), so a storage
    # named `Almacen` is never in it.
    for key, text in (("sv", "Disponible"), ("snv", "NoDisponible")):
        for state, capacity, filler in (
            ("vacio", 10, 0),
            ("uno", 10, 1),
            ("lleno", 1, 1),
        ):
            path = out / f"sels_{key}_num_{state}.gmdl"
            path.write_bytes(
                select_s_storage([text], capacity, filler, storage="1").encode("utf-8")
            )
            print(f"wrote {path}  (op {text}, storage 1 {state})")


if __name__ == "__main__":
    main()
