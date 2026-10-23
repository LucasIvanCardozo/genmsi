#!/usr/bin/env python3
"""Tests of the bench generator (`make-benches.py`).

The generator has to ask the original the same question the rest of the
benches ask: the same operands, the same names and the same entities.  These
tests pin the three rules that decide that:

- the label of a block is the one the macro of its class writes (the name of
  the module, with the `A_` in front of it or without it, or a literal);
- a class that uses an entity the original only simulates when the model
  declares it carries that declaration;
- a class whose statement needs a destination the original refuses to
  compile without it carries that destination.

The first tests build their own little catalogue, so they run without the
backup of the original.  The last ones read the real catalogue and are
skipped when the backup is not there.

    python3 -m unittest discover -s scripts/wine-oracle -p "test_*.py"
"""

from __future__ import annotations

import contextlib
import importlib.util
import io
import re
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
MAKER = HERE / "make-benches.py"


def load_maker():
    """Import the generator, whose file name is not a module name."""
    spec = importlib.util.spec_from_file_location("make_benches", MAKER)
    module = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    spec.loader.exec_module(module)
    return module


mb = load_maker()


def catalogue(classes: str) -> str:
    """A catalogue of the original's shape, with the classes given."""
    return (
        '<gmodel versioncode="1.00.2008.03.23"><gclasses>' + classes + "</gclasses></gmodel>"
    )


def module_class(name: str, macro: str, ports: str, width: int = 100, height: int = 40) -> str:
    """One `<moduleclass>` element, with its graph and its macro code."""
    lines = "".join('<line text="%s"/>' % line for line in macro)
    return (
        '<moduleclass className="%s" ClassDescrip="" width="%d" height="%d">'
        % (name, width, height)
        + "<graph>" + ports + "</graph>"
        + "<macrocode>" + lines + "</macrocode>"
        + "</moduleclass>"
    )


# The ports every bench needs.
GENERATE = module_class(
    "Generate",
    ["GENERATE%20[Media:'Frecuencia media'='60']"],
    '<part type="Output" value="38,48" ident="Salida1"/>',
)
TERMINATE = module_class(
    "Terminate",
    ["IN%20%20%20%20%20%20Entrada1:'Ingreso'='A_moduleName'",
     "TERMINATE%20[Unidades:'Cantidad de unidades'='']"],
    '<part type="Input" value="38,0" ident="Entrada1"/>',
)
CLOCK = module_class(
    "Clock",
    ["CLOCK%20[Frecuencia:'frecuencia'='3600']"],
    "",
)


def bench_of(classes: dict, name: str) -> str:
    """The text of the bench of a class."""
    by_name = {cls.name: cls for cls in classes}
    text, _ = mb.build_bench(by_name[name], by_name)
    return text


def labels_of(text: str) -> dict[str, str]:
    """The label of every block of a bench, by the name of its module."""
    labels = {}
    for match in re.finditer(
        r'<module [^>]*moduleclass="([^"]*)"[^>]*name="([^"]*)"[^>]*>(.*?)</module>', text
    ):
        module_class_name, module_name, body = match.groups()
        found = re.search(r'<input [^>]*mylabel="([^"]*)"', body)
        labels[module_name] = found.group(1) if found else ""
    return labels


def outputs_of(text: str, module_name: str) -> dict[str, str]:
    """The label every output of a block points to."""
    match = re.search(
        r'<module [^>]*name="' + re.escape(module_name) + r'"[^>]*>(.*?)</module>', text
    )
    if match is None:
        return {}
    return dict(
        (m.group(1), m.group(2))
        for m in re.finditer(r'<output [^>]*name="([^"]*)"[^>]*mylabel="([^"]*)"', match.group(1))
    )


class LabelRule(unittest.TestCase):
    """The label of a block is the one the macro of its class writes."""

    def setUp(self) -> None:
        self.classes = mb.load_classes(self.write_catalogue())

    def write_catalogue(self) -> Path:
        classes = [
            GENERATE,
            TERMINATE,
            CLOCK,
            module_class(
                "Puerta",
                ["IN%20%20Entrada1:'Ingreso'='moduleName'",
                 "GATE%20[op:'Operador'='Libre']"],
                '<part type="Input" value="38,0" ident="Entrada1"/>'
                '<part type="Output" value="38,48" ident="Salida1"/>',
            ),
            module_class(
                "Recurso",
                ["IN%20%20Entrada1:'Ingreso'='A_moduleName'",
                 "SEIZE%20[Facility:'Facility'='Recurso']"],
                '<part type="Input" value="38,0" ident="Entrada1"/>'
                '<part type="Output" value="38,48" ident="Salida1"/>',
            ),
            module_class(
                "Clona",
                ["IN%20%20Entrada1:'AClonar'='Clona'",
                 "CLONE%20[Cantidad:'Cantidad de Copias'='1']"],
                '<part type="Input" value="38,0" ident="Entrada1"/>'
                '<part type="Output" value="38,48" ident="Salida1"/>',
            ),
        ]
        path = Path(tempfile.mkdtemp()) / "gClasses.xml"
        path.write_text(catalogue("".join(classes)), encoding="cp1252")
        return path

    def test_the_macro_without_prefix_gives_the_name_of_the_module(self) -> None:
        labels = labels_of(bench_of(self.classes, "Puerta"))
        self.assertEqual(labels["Puerta"], "Puerta")

    def test_the_macro_with_prefix_gives_the_name_with_the_prefix(self) -> None:
        labels = labels_of(bench_of(self.classes, "Recurso"))
        self.assertEqual(labels["Recurso"], "A_Recurso")

    def test_a_literal_is_written_as_it_is(self) -> None:
        labels = labels_of(bench_of(self.classes, "Clona"))
        self.assertEqual(labels["Clona"], "Clona")

    def test_the_link_points_to_the_label_of_the_block_it_feeds(self) -> None:
        text = bench_of(self.classes, "Puerta")
        labels = labels_of(text)
        self.assertEqual(outputs_of(text, "Generate")["Salida1"], labels["Puerta"])
        self.assertEqual(outputs_of(text, "Puerta")["Salida1"], labels["Terminate"])


class Declarations(unittest.TestCase):
    """A class that uses an entity carries the declaration of it."""

    def setUp(self) -> None:
        classes = [
            GENERATE,
            TERMINATE,
            CLOCK,
            module_class(
                "StorageDef",
                ["STORAGE%20%20[Storage:'Storage a Declarar'='Almacen'],...",
                 "%20%20%20%20[Tama%F1o:'Capacidad del Storage'='10']"],
                "",
            ),
            module_class(
                "StorageUse",
                ["IN%20%20Entrada1:'Ingreso'='modulename'",
                 "ENTER%20%20[Storage:'Storage a utilizar'='Almacen']"],
                '<part type="Input" value="38,0" ident="Entrada1"/>'
                '<part type="Output" value="38,48" ident="Salida1"/>',
            ),
        ]
        path = Path(tempfile.mkdtemp()) / "gClasses.xml"
        path.write_text(catalogue("".join(classes)), encoding="cp1252")
        self.classes = mb.load_classes(path)

    def test_the_bench_declares_the_storage_the_class_uses(self) -> None:
        text = bench_of(self.classes, "StorageUse")
        self.assertIn('moduleclass="StorageDef"', text)
        declaration = re.search(
            r'<module [^>]*moduleclass="StorageDef"[^>]*>(.*?)</module>', text
        )
        self.assertIsNotNone(declaration)
        self.assertIn('name="Storage" hint="Storage%20a%20Declarar" value="Almacen"', declaration.group(1))

    def test_the_label_of_the_class_is_the_one_its_macro_writes(self) -> None:
        # The macro of the storage says `='modulename'`: the original names
        # the block after the module, with no `A_` in front.
        labels = labels_of(bench_of(self.classes, "StorageUse"))
        self.assertEqual(labels["StorageUse"], "StorageUse")


class FailureOutput(unittest.TestCase):
    """A class whose statement needs a destination carries it."""

    def setUp(self) -> None:
        classes = [
            GENERATE,
            TERMINATE,
            CLOCK,
            module_class(
                "Examine",
                ["IN%20%20Entrada1:'Entrada1'='A_ModuleName'",
                 "EXAMINEME%20[Grupo:'Grupo de transacciones a examinar'='UnGrupo'],...",
                 "%20%20%20%20Salida2:'No en [Grupo]'",
                 "OUT%20%20%20%20%20Salida1:'En [Grupo]'"],
                '<part type="Input" value="38,0" ident="Entrada1"/>'
                '<part type="Output" value="38,48" ident="Salida1"/>'
                '<part type="Output" value="0,24" ident="Salida2"/>',
            ),
        ]
        path = Path(tempfile.mkdtemp()) / "gClasses.xml"
        path.write_text(catalogue("".join(classes)), encoding="cp1252")
        self.classes = mb.load_classes(path)

    def test_the_failure_output_points_to_a_terminator(self) -> None:
        text = bench_of(self.classes, "Examine")
        labels = labels_of(text)
        self.assertEqual(outputs_of(text, "Examine")["Salida2"], labels["Terminate1"])

    def test_the_terminator_is_named_the_way_the_original_names_it(self) -> None:
        # The original names the blocks that end a run `Terminate`,
        # `Terminate1`, ... , and their labels `A_Terminate`, `A_Terminate1`.
        labels = labels_of(bench_of(self.classes, "Examine"))
        self.assertEqual(labels["Terminate"], "A_Terminate")
        self.assertEqual(labels["Terminate1"], "A_Terminate1")


@unittest.skipUnless(mb.CLASSES_XML.is_file(), "the backup of the original is not here")
class TheRealCatalogue(unittest.TestCase):
    """The labels of the benches of the original's own catalogue."""

    @classmethod
    def setUpClass(cls) -> None:
        cls.classes = mb.load_classes(mb.CLASSES_XML)

    def bench(self, name: str) -> str:
        return bench_of(self.classes, name)

    def test_the_four_gates_are_named_after_the_module(self) -> None:
        for name in ("GateF", "GateL", "GateS", "GateM"):
            self.assertEqual(labels_of(self.bench(name))[name], name)

    def test_the_split_and_the_transfer_name_their_own_statements(self) -> None:
        self.assertEqual(labels_of(self.bench("Split"))["Split"], "Clona")
        self.assertEqual(labels_of(self.bench("Transfer"))["Transfer"], "Entrada1")

    def test_the_storage_use_is_named_after_the_module(self) -> None:
        self.assertEqual(labels_of(self.bench("StorageUse"))["StorageUse"], "StorageUse")

    def test_the_others_keep_the_prefix(self) -> None:
        self.assertEqual(labels_of(self.bench("FacilityUse"))["FacilityUse"], "A_FacilityUse")

    def test_every_bench_is_written(self) -> None:
        with tempfile.TemporaryDirectory() as outdir, contextlib.redirect_stdout(io.StringIO()):
            self.assertEqual(mb.main(["make-benches.py", outdir]), 0)
            written = sorted(path.name for path in Path(outdir).glob("*.gmdl"))
        self.assertEqual(len(written), 66)

    def test_the_same_catalogue_gives_the_same_bytes(self) -> None:
        with tempfile.TemporaryDirectory() as outdir, contextlib.redirect_stdout(io.StringIO()):
            mb.main(["make-benches.py", outdir])
            first = {
                path.name: path.read_bytes() for path in sorted(Path(outdir).glob("*.gmdl"))
            }
            mb.main(["make-benches.py", outdir])
            second = {
                path.name: path.read_bytes() for path in sorted(Path(outdir).glob("*.gmdl"))
            }
        self.assertEqual(first, second)


if __name__ == "__main__":
    unittest.main()
