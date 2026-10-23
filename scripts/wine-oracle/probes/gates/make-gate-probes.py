#!/usr/bin/env python3
"""Build the probes that measure the six storage operators of the GATE.

The original's `GateS` class has six operators (`op_Logic_Stor` in
`gmacroconfig.gmcfg`): SF (Lleno), SNF (NoLleno), SE (Vacio), SNE (NoVacio),
SV (Disponible) and SNV (NoDisponible).  Each probe asks the original what
every operator does with a storage in one state:

- `vacio`:   the storage is declared with ten units and nobody entered it
             (empty, not full);
- `uno`:     the storage is declared with ten units and one of them is in use
             (not empty, not full);
- `lleno`:   the storage is declared with one unit and it is in use (full,
             not empty);
- `sinAlmacen`: the model does not declare the storage at all.

Every operator has its own flow —a `Generate`, a `GateS` and a `Terminate`— so
the report says, operator by operator, whether the gate opened.

Usage: python3 scripts/wine-oracle/probes/gates/make-gate-probes.py
"""

from pathlib import Path

HERE = Path(__file__).resolve().parent

OPS = [
    ("Lleno", "SF"),
    ("NoLleno", "SNF"),
    ("Vacio", "SE"),
    ("NoVacio", "SNE"),
    ("Disponible", "SV"),
    ("NoDisponible", "SNV"),
]

HEAD = (
    '<gmodel versioncode="1.00.2008.03.23"><gclasses/><modules>'
    '<module moduleid="0" moduleclass="main" owner="0" caption="Modelo"'
    ' name="Modelo1" kind="C"'
    ' genprefer="1,1,1,1,1,0,0,0,0,0,0,0,0" exehead="" nonexehead=""'
    ' exefoot="" nonexefoot=""><links>{links}</links><inputs/><outputs/>'
    "</module>"
)

STORAGE_DEF = (
    '<module moduleid="{id}" moduleclass="StorageDef" owner="0"'
    ' caption="StorageDef" name="StorageDef" kind="A" width="100"'
    ' height="30" x="120" y="40"><props>'
    '<property name="Storage" hint="Storage%20a%20Declarar" value="Almacen"'
    ' type="EntityDef" subType="Storage" visible="1" unique="1"/>'
    '<property name="Tamaño" hint="Capacidad%20del%20Storage" value="{size}"'
    ' type="PosInteger" subType="" visible="1" unique="0"/>'
    "</props></module>"
)

GENERATE = (
    '<module moduleid="{id}" moduleclass="Generate" owner="0"'
    ' caption="Generate" name="{name}" kind="A" width="120" height="63"'
    ' x="120" y="{y}"><outputs>'
    '<output ioid="1" name="Salida1" caption="" mylabel="{label}"/>'
    "</outputs><props>"
    '<property name="Cantidad" hint="Cantidad%20de%20Transacciones%20a%20generar"'
    ' value="{count}" type="TSNA" subType="SNA" visible="1" unique="0"/>'
    '<property name="Media" hint="Frecuencia%20media%20de%20arribos"'
    ' value="{media}" type="TSNA" subType="SNA" visible="1" unique="0"/>'
    '<property name="Modificador" hint="Desvio%20o%20Modificador"'
    ' value="0" type="TSNA" subType="SNA" visible="1" unique="0"/>'
    '<property name="OffSet" hint="Tiempo%20de%20espera" value=""'
    ' type="TSNA" subType="SNA" visible="1" unique="0"/>'
    '<property name="Prioridad" hint="Prioridad%20de%20la%20transaccion%20generada"'
    ' value="" type="TSNA" subType="SNA" visible="1" unique="0"/>'
    "</props></module>"
)

ENTER = (
    '<module moduleid="{id}" moduleclass="Enter" owner="0" caption="Enter"'
    ' name="Enter" kind="A" width="100" height="40" x="120" y="{y}"><inputs>'
    '<input ioid="1" name="Entrada1" caption="Ingreso" mylabel="{label}"/>'
    "</inputs><outputs>"
    '<output ioid="2" name="Salida1" caption="" mylabel="{label}"/>'
    "</outputs><props>"
    '<property name="Cantidad" hint="Cantidad%20de%20unidades%20a%20ingresar"'
    ' value="1" type="TSNA" subType="SNA" visible="1" unique="0"/>'
    '<property name="Storage" hint="Storage%20al%20que%20ingresar" value="Almacen"'
    ' type="EntityUse" subType="Storage" visible="1" unique="0"/>'
    "</props></module>"
)

GATE = (
    '<module moduleid="{id}" moduleclass="GateS" owner="0" caption="GateS"'
    ' name="{name}" kind="A" width="100" height="40" x="120" y="{y}"><inputs>'
    '<input ioid="1" name="Entrada1" caption="Ingreso" mylabel="{label}"/>'
    "</inputs><outputs>"
    '<output ioid="2" name="Salida1" caption="" mylabel="{label}"/>'
    "</outputs><props>"
    '<property name="Storage" hint="Storage%20a%20evaluar." value="Almacen"'
    ' type="EntityUse" subType="Storage" visible="1" unique="0"/>'
    '<property name="op" hint="Operador%20logico%20del%20storage." value="{op}"'
    ' type="Listed" subType="op_Logic_Stor" visible="1" unique="0"/>'
    "</props></module>"
)

TERMINATE = (
    '<module moduleid="{id}" moduleclass="Terminate" owner="0"'
    ' caption="Terminate" name="Terminate" kind="A" width="84" height="64"'
    ' x="120" y="{y}"><inputs>'
    '<input ioid="1" name="Entrada1" caption="Ingreso" mylabel="{label}"/>'
    "</inputs><props>"
    '<property name="Unidades" hint="Cantidad%20de%20unidades%20a%20decrementar"'
    ' value="" type="TSNA" subType="SNA" visible="1" unique="0"/>'
    "</props></module>"
)

CLOCK = (
    '<module moduleid="{id}" moduleclass="Clock" owner="0" caption="Clock"'
    ' name="Clock" kind="A" width="50" height="50" x="420" y="40"><props>'
    '<property name="Frecuencia" hint="frecuencia%2C%20en%20unidad%20de%20tiempo"'
    ' value="3600" type="TSNA" subType="SNA" visible="1" unique="0"/>'
    "</props></module>"
)


def probe(capacity: int | None, filler: int) -> str:
    """The model of one state of the storage.

    `capacity` is the size of the declared storage (`None` leaves the model
    without a `StorageDef`) and `filler` is the number of units a flow puts
    into it.
    """
    links = []
    modules = []
    next_id = 1
    y = 40

    if capacity is not None:
        modules.append(STORAGE_DEF.format(id=next_id, size=capacity))
        next_id += 1

    if filler > 0:
        # A flow that enters one unit and never leaves it: the storage holds
        # one unit from the first arrival on.
        modules.append(
            GENERATE.format(
                id=next_id, name="Generate", label="A_Enter", media="10", y=y, count="1"
            )
        )
        links.append(
            f'<link frommoduleid="{next_id}" fromioid="1" tomoduleid="{next_id + 1}" toioid="1"/>'
        )
        next_id += 1
        modules.append(ENTER.format(id=next_id, label="A_Enter", y=y))
        next_id += 1

    for op, _letter in OPS:
        label = f"A_Gate{op}"
        modules.append(
            GENERATE.format(
                id=next_id, name=f"Generate{op}", label=label, media="60", y=y, count=""
            )
        )
        links.append(
            f'<link frommoduleid="{next_id}" fromioid="1" tomoduleid="{next_id + 1}" toioid="1"/>'
        )
        next_id += 1
        modules.append(GATE.format(id=next_id, name=f"Gate{op}", label=label, op=op, y=y))
        links.append(
            f'<link frommoduleid="{next_id}" fromioid="2" tomoduleid="{next_id + 1}" toioid="1"/>'
        )
        next_id += 1
        modules.append(TERMINATE.format(id=next_id, label=label, y=y))
        next_id += 1

    modules.append(CLOCK.format(id=next_id))

    return HEAD.format(links="".join(links)) + "".join(modules) + "</modules></gmodel>"


def main() -> None:
    for name, capacity, filler in (
        ("gateS_vacio", 10, 0),
        ("gateS_uno", 10, 1),
        ("gateS_lleno", 1, 1),
        ("gateS_sinAlmacen", None, 0),
    ):
        path = HERE / f"{name}.gmdl"
        path.write_text(probe(capacity, filler), encoding="utf-8")
        print(f"wrote {path}")


if __name__ == "__main__":
    main()
