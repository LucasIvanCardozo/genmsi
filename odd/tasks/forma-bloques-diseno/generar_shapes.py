#!/usr/bin/env python3
"""Genera el modulo de las formas de los bloques.

Lee el diseno aprobado (los tres entregables de los grupos) y escribe
`crates/genmsi-app/src/graph/shapes.ts`: el modulo del que dibujan tanto el
nodo del lienzo como la miniatura del catalogo (S6).

El diseno es la fuente de la verdad: este script solo lo traduce. No
arregla nada del diseno; cuando el diseno es incoherente lo dice y no
escribe el modulo.

El parseo de los entregables es el de `render.py`, que ya sabe leerlos: no
hay dos lecturas del mismo archivo.

Uso:
    python3 odd/tasks/forma-bloques-diseno/generar_shapes.py
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
RAIZ = HERE.parents[2]
DESTINO = RAIZ / "crates" / "genmsi-app" / "src" / "graph" / "shapes.ts"
CATALOGO = RAIZ / "crates" / "genmsi-format" / "src" / "catalogue.rs"

sys.path.insert(0, str(HERE))
import render  # noqa: E402  (el parseo del diseno es el de render.py)

# El orden de las familias en el modulo: el del flujo (lo que nace, lo que
# espera, lo que se toma, lo que se guarda, lo que decide, lo que opera).
FAMILIAS = [
    "Fuente",
    "Destino",
    "Decisión",
    "Cola",
    "Recurso",
    "Depósito",
    "Tabla",
    "Cadena",
    "Grupo",
    "Selección",
    "Operación",
    "Anotación",
]

# Los entregables, en el orden en que se leen.
GRUPOS = ["A-lineal.md", "B-entidad.md", "C-grupos.md"]


def cargar_clases() -> list[render.Clase]:
    """Las 66 clases del diseno, sin repetir."""
    clases: list[render.Clase] = []
    vistos: set[str] = set()
    for nombre in GRUPOS:
        ruta = HERE / "grupos" / nombre
        if not ruta.exists():
            raise SystemExit(f"falta el entregable {ruta}")
        for clase in render.leer_grupos(ruta):
            if clase.nombre in vistos:
                raise SystemExit(f"la clase {clase.nombre} aparece dos veces")
            vistos.add(clase.nombre)
            clases.append(clase)
    return clases


def clases_del_catalogo() -> set[str]:
    """Las clases que declara el catalogo del motor."""
    texto = CATALOGO.read_text(encoding="utf-8")
    return set(re.findall(r'class: "([^"]+)"', texto))


def verificar(clases: list[render.Clase]) -> list[str]:
    """Las reglas del diseno, comprobadas sobre los entregables.

    El generador no corrige el diseno: si una regla no se cumple, el
    modulo no se escribe y el problema se reporta.
    """
    problemas: list[str] = []
    for clase in clases:
        w, h = clase.huella
        if w <= 0 or h <= 0:
            problemas.append(f"{clase.nombre}: huella invalida {w}x{h}")
            continue
        if not clase.path:
            problemas.append(f"{clase.nombre}: sin path")
        if clase.familia not in FAMILIAS:
            problemas.append(f"{clase.nombre}: familia desconocida {clase.familia!r}")
        if not clase.variante:
            problemas.append(f"{clase.nombre}: sin variante")
        if clase.label is None:
            problemas.append(f"{clase.nombre}: sin zona de texto")
        else:
            lx, ly, lw, lh = clase.label
            if lx < 0 or ly < 0 or lx + lw > w + 0.5 or ly + lh > h + 0.5:
                problemas.append(
                    f"{clase.nombre}: zona de texto fuera ({clase.label} en {w}x{h})"
                )
        if clase.figure is not None:
            fx, fy, fw, fh = clase.figure
            if fx < 0 or fy < 0 or fx + fw > w + 0.5 or fy + fh > h + 0.5:
                problemas.append(
                    f"{clase.nombre}: zona de cifra fuera ({clase.figure} en {w}x{h})"
                )
            if clase.label is not None:
                lx, ly, lw, lh = clase.label
                # dos cajas se solapan solo si se tocan en X **y** en Y
                if lx < fx + fw and fx < lx + lw and ly < fy + fh and fy < ly + lh:
                    problemas.append(f"{clase.nombre}: texto y cifra se solapan")
        for puerto in clase.puertos:
            if not (0 <= puerto.x <= 100) or not (0 <= puerto.y <= 100):
                problemas.append(
                    f"{clase.nombre}: puerto {puerto.nombre} fuera de la huella "
                    f"({puerto.x}%,{puerto.y}%)"
                )
    faltan = sorted(clases_del_catalogo() - {c.nombre for c in clases})
    sobran = sorted({c.nombre for c in clases} - clases_del_catalogo())
    if faltan:
        problemas.append(f"clases del catalogo sin forma: {faltan}")
    if sobran:
        problemas.append(f"formas que no son clases del catalogo: {sobran}")
    for familia in FAMILIAS:
        if not any(c.familia == familia for c in clases):
            problemas.append(f"la familia {familia} no tiene ninguna clase")
        if familia not in render.FAMILIAS:
            problemas.append(f"la familia {familia} no tiene sus tres niveles de color")
    return problemas


def numero(valor: float) -> str:
    """Un numero de TypeScript, sin ceros de mas."""
    if float(valor).is_integer():
        return str(int(valor))
    return repr(float(valor))


def texto(valor: str) -> str:
    """Una cadena de TypeScript, escapada."""
    return json.dumps(valor, ensure_ascii=False)


def zona(caja: tuple[float, float, float, float] | None) -> str:
    """La zona de texto o la de cifra, en una linea."""
    if caja is None:
        return "null"
    x, y, w, h = caja
    return (
        f"{{ x: {numero(x)}, y: {numero(y)}, "
        f"width: {numero(w)}, height: {numero(h)} }}"
    )


def clase_ts(clase: render.Clase) -> str:
    """La forma de una clase, como TypeScript."""
    w, h = clase.huella
    puertos = []
    for puerto in clase.puertos:
        puertos.append(
            f"      {{ name: {texto(puerto.nombre)}, "
            f"x: {numero(puerto.x / 100)}, y: {numero(puerto.y / 100)} }},"
        )
    if clase.details:
        detalles = "    details: [\n" + "".join(
            f"      {texto(d)},\n" for d in clase.details
        ) + "    ],"
    else:
        detalles = "    details: [],"
    if puertos:
        puertos_ts = "    ports: [\n" + "\n".join(puertos) + "\n    ],"
    else:
        puertos_ts = "    ports: [],"
    lineas = [
        f"  {texto(clase.nombre)}: {{",
        f"    family: {texto(clase.familia)},",
        f"    variant: {texto(clase.variante)},",
        f"    footprint: {{ width: {numero(w)}, height: {numero(h)} }},",
        f"    path: {texto(clase.path)},",
        detalles,
        puertos_ts,
        f"    label: {zona(clase.label)},",
        f"    figure: {zona(clase.figure)},",
        f"    note: {texto(clase.nota)},",
        "  },",
    ]
    return "\n".join(lineas)


def modulo(clases: list[render.Clase]) -> str:
    """El modulo completo, como TypeScript."""
    por_familia: dict[str, list[render.Clase]] = {f: [] for f in FAMILIAS}
    for clase in clases:
        por_familia[clase.familia].append(clase)
    ordenadas = []
    for familia in FAMILIAS:
        ordenadas.extend(sorted(por_familia[familia], key=lambda c: c.nombre))

    paleta = "".join(
        f'  {texto(familia)}: {{ acento: {texto(render.FAMILIAS[familia]["acento"])}, '
        f'borde: {texto(render.FAMILIAS[familia]["borde"])}, '
        f'relleno: {texto(render.FAMILIAS[familia]["relleno"])} }},\n'
        for familia in FAMILIAS
    )
    formas = "\n".join(clase_ts(clase) for clase in ordenadas)
    nombres = "".join(f"  {texto(clase.nombre)},\n" for clase in ordenadas)
    union = "".join(f"  | {texto(familia)}\n" for familia in FAMILIAS)

    return f'''/**
 * The visual shape of every block class.
 *
 * A block of the model is not a rounded rectangle any more: every class of
 * the catalogue has its own silhouette, so the reader recognises a block by
 * its shape before reading its name (S2). This module is the single source
 * of that shape (S6): the node of the canvas and the thumbnail of the
 * catalogue draw the same silhouette, or the list of blocks lies.
 *
 * The module holds, for every class:
 *
 * - the **family** it belongs to (the language of shapes) and the name of
 *   its **variant** inside that family (S2, §5.1 of the contract);
 * - the **footprint** the class is drawn at (S7: the silhouette is drawn
 *   inside the footprint the model already carries);
 * - the **path** of the silhouette and the **details** that stroke its
 *   inside;
 * - the **ports**, as fractions of the footprint, at the anchor the
 *   original editor put them (S8);
 * - the **label** zone and the **figure** zone, in footprint pixels, which
 *   never overlap each other (S9).
 *
 * It also holds the three **colour levels** of every family: the strong
 * tone (`acento`) for the live figure and the dot of the port, the same
 * tone lightened (`borde`) for the outline of the silhouette, and the same
 * tone lightened a lot (`relleno`) for the fill of the block.
 *
 * The module is pure data plus small pure helpers: it imports nothing, so
 * it can be unit tested with `node --test`.
 *
 * GENERATED FILE: do not edit it by hand. Its source of truth is
 * `odd/tasks/forma-bloques-diseno/generar_shapes.py`, which reads the
 * approved design in `odd/tasks/forma-bloques-diseno/grupos/*.md`.
 */

/** The family of a block: the language of shapes it belongs to. */
export type Family =
{union};

/**
 * The three colour levels of a family.
 *
 * The three levels are the same tone at three strengths: the family is
 * recognised by the hue, and the level says what the colour is for.
 */
export interface FamilyColors {{
  /** The strong tone: the live figure and the dot of the port. */
  acento: string;
  /** The same tone, lightened: the outline of the silhouette. */
  borde: string;
  /** The same tone, lightened a lot: the fill of the block. */
  relleno: string;
}}

/** Where a port of a block sits, as a fraction of the footprint. */
export interface PortAnchor {{
  /** The name of the port, as the catalogue declares it. */
  name: string;
  /** The horizontal anchor: 0 is the left border, 1 the right one. */
  x: number;
  /** The vertical anchor: 0 is the top border, 1 the bottom one. */
  y: number;
}}

/** A rectangular zone inside the footprint of a block. */
export interface Zone {{
  /** The left of the zone, in footprint pixels. */
  x: number;
  /** The top of the zone, in footprint pixels. */
  y: number;
  /** The width of the zone, in footprint pixels. */
  width: number;
  /** The height of the zone, in footprint pixels. */
  height: number;
}}

/** The shape of one class of block. */
export interface BlockShape {{
  /** The family the class belongs to. */
  family: Family;
  /** The name of the silhouette inside the family. */
  variant: string;
  /** The size the class is drawn at. */
  footprint: {{ width: number; height: number }};
  /** The `d` of the silhouette, in the coordinates of the footprint. */
  path: string;
  /** The `d` of the interior strokes, drawn without fill. */
  details: string[];
  /** Where the ports sit, as fractions of the footprint. */
  ports: PortAnchor[];
  /** The zone of the label and of the subtitle. */
  label: Zone;
  /** The zone of the live figure, or `null` when the class has none. */
  figure: Zone | null;
  /** Why the class has this silhouette, in the words of the design. */
  note: string;
}}

/** The three colour levels of every family. */
export const FAMILY_COLORS: Record<Family, FamilyColors> = {{
{paleta}}};

/** The shape of every class of the catalogue, by class name. */
export const BLOCK_SHAPES: Record<string, BlockShape> = {{
{formas}
}};

/** The classes the module knows, grouped by family. */
export const SHAPE_CLASSES: string[] = [
{nombres}];

/** The shape of a class, or `undefined` when the class has no shape yet. */
export function shapeOf(className: string): BlockShape | undefined {{
  return BLOCK_SHAPES[className];
}}

/** The three colour levels of a family. */
export function familyColors(family: Family): FamilyColors {{
  return FAMILY_COLORS[family];
}}

/** The three colour levels of the family of a class. */
export function colorsOf(className: string): FamilyColors | undefined {{
  const shape = BLOCK_SHAPES[className];
  return shape === undefined ? undefined : FAMILY_COLORS[shape.family];
}}
'''


def main() -> int:
    clases = cargar_clases()
    problemas = verificar(clases)
    if problemas:
        print(f"el diseno tiene {len(problemas)} problemas:", file=sys.stderr)
        for problema in problemas:
            print(f"  {problema}", file=sys.stderr)
        return 1
    DESTINO.write_text(modulo(clases), encoding="utf-8")
    print(f"escrito {DESTINO.relative_to(RAIZ)} ({len(clases)} clases)", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
