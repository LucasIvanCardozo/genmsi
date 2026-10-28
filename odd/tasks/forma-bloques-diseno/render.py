#!/usr/bin/env python3
"""Integra las especificaciones de las formas de bloque.

Lee los entregables de los grupos (`grupos/*.md`, en el formato del contrato)
y produce dos salidas:

- `--svg <ruta>`: una hoja de contacto con todas las clases, para **mirar**
  el resultado antes de tocar nada.
- `--pen`: el fragmento de JavaScript para el MCP de Pen, que dibuja las
  mismas clases en el lienzo.

Las dos salidas salen de la misma lectura, así que el lienzo y la hoja
difieren exactamente en lo que difieren los lienzos: no hay dos fuentes de
verdad.

Uso:
    python3 render.py --svg /tmp/formas.png
    python3 render.py --pen > /tmp/pen.js
"""

from __future__ import annotations

import argparse
import re
import sys
from dataclasses import dataclass, field
from pathlib import Path

HERE = Path(__file__).resolve().parent
GRUPOS = HERE / "grupos"

# El color de cada familia (contrato v2 §5), en tres niveles.
#
# - `acento`: el color fuerte. Lo llevan la cifra de la cifra viva y el
#   texto del titulo de la familia: es texto, y tiene que se leer.
# - `borde`: el mismo tono, aclarado. Dibuja la silueta del bloque: el color
#   se nota al mirar la forma, pero no grita sobre el lienzo.
# - `relleno`: el mismo tono, muy aclarado. El fondo del bloque, un velo.
#
# Los tres salen del mismo tono: la familia se reconoce por el color, y la
# jerarquia la marca el nivel, no el tono.
FAMILIAS: dict[str, dict[str, str]] = {
    "Fuente": {"acento": "#047857", "borde": "#b3c4bc", "relleno": "#f3f6f5"},
    "Destino": {"acento": "#be123c", "borde": "#deb3b7", "relleno": "#f9f4f4"},
    "Decision": {"acento": "#4338ca", "borde": "#b8b7e4", "relleno": "#f4f4fa"},
    "Decisión": {"acento": "#4338ca", "borde": "#b8b7e4", "relleno": "#f4f4fa"},
    "Cola": {"acento": "#b45309", "borde": "#dabbb3", "relleno": "#f9f4f3"},
    "Recurso": {"acento": "#0f766e", "borde": "#b3c4c1", "relleno": "#f4f6f5"},
    "Deposito": {"acento": "#0369a1", "borde": "#b3c0d2", "relleno": "#f3f5f8"},
    "Depósito": {"acento": "#0369a1", "borde": "#b3c0d2", "relleno": "#f3f5f8"},
    "Tabla": {"acento": "#6d28d9", "borde": "#c1b5eb", "relleno": "#f5f4fc"},
    "Cadena": {"acento": "#c2410c", "borde": "#e0b8b3", "relleno": "#faf4f3"},
    "Grupo": {"acento": "#be185d", "borde": "#deb4bd", "relleno": "#f9f4f5"},
    "Seleccion": {"acento": "#0e7490", "borde": "#b3c3cc", "relleno": "#f3f6f7"},
    "Selección": {"acento": "#0e7490", "borde": "#b3c3cc", "relleno": "#f3f6f7"},
    "Operacion": {"acento": "#475569", "borde": "#b9bcc0", "relleno": "#f4f5f5"},
    "Operación": {"acento": "#475569", "borde": "#b9bcc0", "relleno": "#f4f5f5"},
    "Anotacion": {"acento": "#64748b", "borde": "#bfc3ca", "relleno": "#f5f6f6"},
    "Anotación": {"acento": "#64748b", "borde": "#bfc3ca", "relleno": "#f5f6f6"},
}

# Una clase sin puertos es una declaración: se dibuja apagada.
DECLARACION = {"Clock", "Function", "FunctionDist", "FunctionExpon", "InitiaSV",
                "InitialLS", "MFunction", "MFunctionDist", "Note", "QTable",
                "StorageDef", "Table", "Variable", "VariableN", "VariableA"}

# La cifra de ejemplo de cada clase (contrato v2 §6). Es la cifra que el
# bloque llevara en el lienzo, no un par generico: una cola muestra su
# contenido, un recurso su ocupacion, un deposito sus unidades.
CIFRA: dict[str, str] = {
    "Generate": "1.842",
    "Terminate": "1.842",
    "Queue": "7",
    "Depart": "7",
    "FacilityUse": "82%",
    "FacilityInt": "82%",
    "Seize": "82%",
    "Release": "82%",
    "Preempt": "82%",
    "Return": "82%",
    "StorageUse": "12/40",
    "Enter": "12/40",
    "Leave": "12/40",
    "Tabulate": "36",
    "Link": "3",
    "Unlink": "3",
    "Logic": "on",
    "InitialLS": "on",
    "SaveValue": "12",
    "InitiaSV": "12",
    "Counter": "12",
}
CIFRA_POR_DEFECTO = "2/15"


@dataclass
class Puerto:
    """Un puerto, con su ancla normalizada a la huella."""
    nombre: str
    x: float
    y: float


@dataclass
class Clase:
    """La especificación de una clase."""
    nombre: str
    familia: str
    variante: str = ""
    huella: tuple[int, int] = (0, 0)
    puertos: list[Puerto] = field(default_factory=list)
    path: str = ""
    details: list[str] = field(default_factory=list)
    label: tuple[float, float, float, float] | None = None
    figure: tuple[float, float, float, float] | None = None
    nota: str = ""

    @property
    def color(self) -> str:
        """El color fuerte de la familia: el de la cifra y los titulos."""
        return FAMILIAS.get(self.familia, FAMILIAS["Operacion"])["acento"]

    @property
    def borde(self) -> str:
        """El color con que se dibuja la silueta: el mismo, mas claro."""
        return FAMILIAS.get(self.familia, FAMILIAS["Operacion"])["borde"]

    @property
    def relleno(self) -> str:
        """El fondo del bloque: el mismo, muy aclarado."""
        return FAMILIAS.get(self.familia, FAMILIAS["Operacion"])["relleno"]

    @property
    def declaracion(self) -> bool:
        return not self.puertos


CAMPO = re.compile(r"^-\s+(\w+):\s*(.*)$")


def leer_grupos(ruta: Path) -> list[Clase]:
    """Lee los `## <Clase>` de un entregable de grupo."""
    clases: list[Clase] = []
    actual: Clase | None = None
    for linea in ruta.read_text(encoding="utf-8").splitlines():
        if linea.startswith("## "):
            if actual is not None:
                clases.append(actual)
            nombre = linea[3:].strip()
            if not re.fullmatch(r"[A-Za-z][A-Za-z0-9]*", nombre):
                actual = None
                continue
            actual = Clase(nombre=nombre, familia="")
            continue
        if actual is None:
            continue
        enc = CAMPO.match(linea)
        if not enc:
            continue
        clave, valor = enc.group(1), enc.group(2).strip()
        if clave == "family":
            actual.familia = valor
        elif clave == "variant":
            actual.variante = valor
        elif clave == "footprint":
            ancho, alto = valor.lower().split("x")
            actual.huella = (int(ancho), int(alto))
        elif clave == "ports":
            actual.puertos = _puertos(valor)
        elif clave == "path":
            actual.path = valor
        elif clave == "details":
            actual.details = [] if valor == "none" else [
                _detalle(d) for d in valor.split(";") if d.strip()
            ]
        elif clave == "label":
            actual.label = _caja(valor)
        elif clave == "figure":
            actual.figure = None if valor == "none" else _caja(valor)
        elif clave == "note":
            actual.nota = valor
    if actual is not None:
        clases.append(actual)
    return clases


def _detalle(valor: str) -> str:
    """Un trazo interior.

    Los grupos lo escriben a veces con una descripción delante
    («rombo interior (herencia): M 110,12 L ...»): la descripción no es
    geometría, así que se separa del trazo.
    """
    texto = valor.strip()
    if "://" in texto:
        return texto
    cabeza, separador, cola = texto.rpartition(":")
    if separador and cabeza and not cabeza.strip().startswith(("M", "L", "C", "A", "Z")):
        return cola.strip()
    return texto


def _puertos(valor: str) -> list[Puerto]:
    if valor in ("none", ""):
        return []
    puertos: list[Puerto] = []
    for parte in valor.split(";"):
        parte = parte.strip()
        if not parte or "@" not in parte:
            continue
        nombre, ancla = parte.rsplit("@", 1)
        x, _, y = ancla.partition(",")
        puertos.append(Puerto(nombre.strip(), float(x.strip("% ")), float(y.strip("% "))))
    return puertos


def _caja(valor: str) -> tuple[float, float, float, float] | None:
    partes = [p.strip() for p in valor.split(",")]
    if len(partes) != 4:
        return None
    return tuple(float(p) for p in partes)  # type: ignore[return-value]


def cargar_todos() -> list[Clase]:
    """Todas las clases de los entregables, sin repetir."""
    clases: list[Clase] = []
    vistos: set[str] = set()
    for ruta in sorted(GRUPOS.glob("*.md")):
        for clase in leer_grupos(ruta):
            if clase.nombre in vistos:
                continue
            vistos.add(clase.nombre)
            clases.append(clase)
    return clases


def svg(clases: list[Clase], ancho: int = 1500) -> str:
    """La hoja de contacto: todas las clases, en su huella real."""
    partes: list[str] = []
    x, y = 40, 130
    alto_fila = 0
    for clase in clases:
        w, h = clase.huella
        if w == 0 or h == 0:
            continue
        if x + w + 120 > ancho:
            x = 40
            y += alto_fila + 76
            alto_fila = 0
        partes.append(_svg_clase(clase, x, y))
        x += w + 120
        alto_fila = max(alto_fila, h)
    alto = y + alto_fila + 60
    encabezado = (
        '<rect width="100%" height="100%" fill="#F8FAFC"/>'
        '<text x="40" y="52" font-family="Inter,sans-serif" font-size="26" '
        'font-weight="700" fill="#1E293B">Las 66 formas de bloque</text>'
        '<text x="40" y="76" font-family="Inter,sans-serif" font-size="13" '
        'fill="#64748B">Cada clase en su huella real, con sus puertos en la ancla '
        'verdadera y sus zonas de rotulo y de cifra.</text>'
    )
    return (
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{ancho}" height="{alto}" '
        f'viewBox="0 0 {ancho} {alto}">{encabezado}{"".join(partes)}</svg>'
    )


def svg_grupo(clases: list[Clase], titulo: str, ancho: int = 1500) -> str:
    """La hoja de un grupo: las clases de ese grupo, en su huella real.

    Es la misma lectura que la hoja de contacto, así que la hoja de un grupo
    y la hoja de todas las clases no pueden discrepar.
    """
    partes: list[str] = []
    x, y = 40, 130
    alto_fila = 0
    for clase in clases:
        w, h = clase.huella
        if w == 0 or h == 0:
            continue
        if x + w + 120 > ancho:
            x = 40
            y += alto_fila + 76
            alto_fila = 0
        partes.append(_svg_clase(clase, x, y))
        x += w + 120
        alto_fila = max(alto_fila, h)
    alto = y + alto_fila + 60
    encabezado = (
        '<rect width="100%" height="100%" fill="#F8FAFC"/>'
        f'<text x="40" y="52" font-family="Inter,sans-serif" font-size="26" '
        f'font-weight="700" fill="#1E293B">{titulo}</text>'
        '<text x="40" y="76" font-family="Inter,sans-serif" font-size="13" '
        'fill="#64748B">Cada clase en su huella real, con sus puertos en la ancla '
        'verdadera y sus zonas de rotulo y de cifra.</text>'
    )
    return (
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{ancho}" height="{alto}" '
        f'viewBox="0 0 {ancho} {alto}">{encabezado}{"".join(partes)}</svg>'
    )


def _svg_clase(clase: Clase, x: int, y: int) -> str:
    w, h = clase.huella
    color = clase.color
    borde = clase.borde
    if clase.declaracion:
        # una declaracion se apaga del todo: mismo tono, mas claro todavia
        relleno, borde, guiones = clase.relleno, clase.borde, ' stroke-dasharray="4 3"'
    else:
        relleno, guiones = clase.relleno, ""
    partes = [
        f'<g transform="translate({x},{y})">',
        f'<text x="0" y="-12" font-family="Inter,sans-serif" font-size="12" '
        f'font-weight="700" fill="{color}">{clase.nombre}</text>',
        f'<text x="0" y="2" font-family="Inter,sans-serif" font-size="9" '
        f'fill="#94A3B8">{w}x{h}</text>',
        f'<path d="{clase.path}" fill="{relleno}" stroke="{borde}" '
        f'stroke-width="2" stroke-linejoin="round"{guiones}/>',
    ]
    for detalle in clase.details:
        partes.append(
            f'<path d="{detalle}" fill="none" stroke="{borde}" stroke-width="1.8" '
            f'stroke-linecap="round"/>'
        )
    if clase.label:
        lx, ly, lw, lh = clase.label
        partes.append(
            f'<rect x="{lx}" y="{ly}" width="{lw}" height="{lh}" fill="none" '
            f'stroke="#6366F1" stroke-width="1" stroke-dasharray="3 2"/>'
        )
        partes.append(
            f'<text x="{lx + 2}" y="{ly + lh - 3}" font-family="Inter,sans-serif" '
            f'font-size="9" font-weight="600" fill="#1E293B">'
            f'{recortar(clase.nombre, lw - 4, 9)}</text>'
        )
    if clase.figure:
        fx, fy, fw, fh = clase.figure
        partes.append(
            f'<rect x="{fx}" y="{fy}" width="{fw}" height="{fh}" fill="none" '
            f'stroke="#0D9488" stroke-width="1" stroke-dasharray="3 2"/>'
        )
        partes.append(
            f'<text x="{fx + fw - 2}" y="{fy + fh - 2}" text-anchor="end" '
            f'font-family="monospace" font-size="9" font-weight="700" '
            f'fill="{color}">{CIFRA.get(clase.nombre, CIFRA_POR_DEFECTO)}</text>'
        )
    for puerto in clase.puertos:
        px, py = puerto.x / 100 * w, puerto.y / 100 * h
        partes.append(
            f'<circle cx="{px}" cy="{py}" r="4" fill="{color}" stroke="#FFFFFF" '
            f'stroke-width="2"/>'
        )
    partes.append("</g>")
    return "".join(partes)


def pen(clases: list[Clase]) -> str:
    """El fragmento de JavaScript que dibuja las clases en el lienzo de Pen.

    El fragmento construye la pantalla completa: una rejilla de fichas, cada
    una con su clase en su huella real, su puerto en el ancla verdadera y sus
    dos zonas. Se pega tal cual en el MCP de Pen.
    """
    import json

    datos = [
        {
            "n": c.nombre,
            "f": c.familia,
            "w": c.huella[0],
            "h": c.huella[1],
            "c": c.color,
            "bd": c.borde,
            "rl": c.relleno,
            "dd": c.declaracion,
            "p": c.path,
            "t": c.details,
            "l": list(c.label) if c.label else None,
            "g": list(c.figure) if c.figure else None,
            "fig": CIFRA.get(c.nombre, CIFRA_POR_DEFECTO),
            "txt": recortar(c.nombre, (c.label[2] if c.label else 0) - 4, 9),
            "pu": [[p.nombre, p.x, p.y] for p in c.puertos],
        }
        for c in clases
    ]
    return (
        "const CLASES = " + json.dumps(datos, ensure_ascii=False) + ";\n"
        + "gridId = Insert(document, {type:'frame', name:'Shapes Screen', x:0, y:0, "
        + "width:1440, height:1200, layout:'none', fill:'$bg-app', clip:true, placeholder:true});\n"
        + "Insert(gridId, {type:'text', name:'Title', content:'Las 66 formas de bloque', "
        + "fontFamily:'Inter', fontSize:24, fontWeight:'700', fill:'$text-primary', x:32, y:28});\n"
        + "Insert(gridId, {type:'text', name:'Subtitle', content:'Cada clase en su huella real, "
        + "con el puerto en su ancla verdadera y las tres zonas.', fontFamily:'Inter', fontSize:12, "
        + "fill:'$text-secondary', x:32, y:58});\n"
        + "let colX = 32, rowY = 96, rowH = 100;\n"
        + "for (const c of CLASES) {\n"
        + "  const w = c.w + 40, h = c.h + 34;\n"
        + "  if (colX + w > 1400) { colX = 32; rowY += rowH; rowH = 0; }\n"
        + "  if (h > rowH) rowH = h;\n"
        + "  const box = Insert(gridId, {type:'frame', name:'Box ' + c.n, layout:'none', x:colX, "
        + "y:rowY, width:w, height:h, fill:'$surface', stroke:'$border', strokeWidth:1, cornerRadius:8});\n"
        + "  Insert(box, {type:'text', name:'Nm ' + c.n, content:c.n, fontFamily:'Inter', "
        + "fontSize:9, fontWeight:'600', fill:'$text-secondary', x:10, y:6});\n"
        + "  const node = Insert(box, {type:'frame', name:'Blk ' + c.n, layout:'none', x:16, y:20, "
        + "width:c.w, height:c.h, fill:'$surface'});\n"
        + "  Insert(node, {type:'path', name:'Sil', geometry:c.p, viewBox:[0,0,c.w,c.h], x:0, y:0, "
        + "width:c.w, height:c.h, fill: c.dd ? c.rl : c.rl, "
        + "stroke: c.bd, strokeWidth:2, strokeLinejoin:'round'});\n"
        + "  for (const t of c.t) Insert(node, {type:'path', name:'Det', geometry:t, "
        + "viewBox:[0,0,c.w,c.h], x:0, y:0, width:c.w, height:c.h, fill:c.rl, "
        + "stroke:c.bd, strokeWidth:1.8, strokeLinecap:'round'});\n"
        + "  for (const pu of c.pu) Insert(node, {type:'ellipse', name:'Port ' + pu[0], "
        + "x: pu[1]/100*c.w - 4, y: pu[2]/100*c.h - 4, width:8, height:8, fill:c.c, "
        + "stroke:'$surface', strokeWidth:2});\n"
        + "  if (c.l) Insert(node, {type:'text', name:'Label ' + c.n, content:c.txt, "
        + "fontFamily:'Inter', fontSize:9, fontWeight:'600', fill:'$text-primary', "
        + "textGrowth:'fixed-width', width:c.l[2], x:c.l[0], y:c.l[1]});\n"
        + "  if (c.g) Insert(node, {type:'text', name:'Fig ' + c.n, content:c.fig, "
        + "fontFamily:'JetBrains Mono', fontSize:10, fontWeight:'700', fill:c.c, "
        + "textGrowth:'fixed-width', width:c.g[2], x:c.g[0], y:c.g[1], textAlign:'right'});\n"
        + "  colX += w + 24;\n"
        + "}\n"
        + "Update(gridId, {placeholder:false});\n"
        + "TakeScreenshot([gridId]);\n"
    )


def ancho_figura(texto: str, tamano: float = 10.0) -> float:
    """El ancho de la cifra: la monoespaciada del app (0,6 em por caracter)."""
    return len(texto) * 0.6 * tamano


def ancho_minimo_figura(texto: str) -> float:
    """El ancho que la zona de la cifra necesita para guardar su texto.

    No es un minimo fijo: depende de lo que la clase muestra. El par de
    siempre («2/15», cinco caracteres) pide 30 px; el tamano de una cadena
    («3», un caracter) pide mucho menos, y pedirle 30 seria forzar una
    silueta mas grande de la que necesita.
    """
    return ancho_figura(texto) + 2


# La medida del nombre de la clase: monoespaciada, 0,6 em por caracter
# (contrato v4 §2.3), a 9 px como el rotulo.
EM_NOMBRE = 0.6
TAMANO_NOMBRE = 9
# El aire que la zona de texto deja alrededor del texto (contrato v4 §2.3).
AIRE = 4
# Dos lineas de 9 px con 2 px de aire entre ellas.
ALTO_DOS_LINEAS = 24


def ancho_nombre(nombre: str) -> float:
    """El ancho del nombre de una clase, con la medida del diseno."""
    return len(nombre) * EM_NOMBRE * TAMANO_NOMBRE


def el_nombre_cabe(nombre: str, zona: tuple[float, float, float, float]) -> bool:
    """Si el nombre cabe en la zona, en una linea o en dos.

    El nombre entero, nunca cortado: o cabe en una linea, o se parte en
    dos que caben las dos, y la zona es alta para las dos.
    """
    x, y, ancho, alto = zona
    util = ancho - AIRE
    if ancho_nombre(nombre) <= util:
        return True
    if alto < ALTO_DOS_LINEAS:
        return False
    for corte in range(1, len(nombre)):
        if ancho_nombre(nombre[:corte]) <= util and ancho_nombre(nombre[corte:]) <= util:
            return True
    return False


def verificar(clases: list[Clase]) -> list[str]:
    """Las reglas que el contrato v5 impone, comprobadas sobre las entregas.

    El verificador es la red de seguridad del padre: los grupos se revisan a
    sí mismos, pero la comprobación final la hace el integrador, sobre las
    tres entregas juntas, que es donde pueden aparecer las colisiones
    entre grupos.
    """
    problemas: list[str] = []
    for c in clases:
        w, h = c.huella
        if not c.path:
            problemas.append(f"{c.nombre}: sin path")
            continue
        if not c.variante:
            problemas.append(f"{c.nombre}: sin variante")
        if c.label:
            lx, ly, lw, lh = c.label
            if lx < 0 or ly < 0 or lx + lw > w + 0.5 or ly + lh > h + 0.5:
                problemas.append(f"{c.nombre}: zona de texto fuera ({c.label} en {w}x{h})")
        if c.label:
            if not el_nombre_cabe(c.nombre, c.label):
                problemas.append(
                    f"{c.nombre}: el nombre no cabe en la zona de texto "
                    f"({c.label}); necesita {ancho_nombre(c.nombre):.1f} px y la zona "
                    f"tiene {c.label[2]:.1f} px"
                )
        if c.figure:
            fx, fy, fw, fh = c.figure
            texto = CIFRA.get(c.nombre, CIFRA_POR_DEFECTO)
            if fw < ancho_minimo_figura(texto):
                problemas.append(
                    f"{c.nombre}: zona de cifra demasiado estrecha ({c.figure}) "
                    f"para «{texto}»"
                )
            if fx < 0 or fy < 0 or fx + fw > w + 0.5 or fy + fh > h + 0.5:
                problemas.append(f"{c.nombre}: zona de cifra fuera ({c.figure} en {w}x{h})")
            if c.label:
                lx, ly, lw, lh = c.label
                # dos cajas se solapan sólo si se tocan en X **y** en Y
                if lx < fx + fw and fx < lx + lw and ly < fy + fh and fy < ly + lh:
                    problemas.append(f"{c.nombre}: texto y cifra se solapan")
    return problemas


def panel(clases: list[Clase]) -> str:
    """La pantalla de la pestaña Bloques: la lista con la miniatura de cada clase.

    La miniatura es la misma silueta del bloque, escalada con su proporcion:
    lo que se ve en la lista es lo que cae en el lienzo.
    """
    import json

    orden = ["Fuente", "Destino", "Cola", "Recurso", "Depósito", "Tabla",
             "Cadena", "Grupo", "Selección", "Decisión", "Operación", "Anotación"]
    datos = []
    for familia in orden:
        for c in clases:
            if c.familia != familia:
                continue
            escala = min(40 / c.huella[0], 20 / c.huella[1])
            w = round(c.huella[0] * escala, 1)
            h = round(c.huella[1] * escala, 1)
            datos.append({
                "n": c.nombre, "fam": familia, "w": w, "h": h,
                "ox": round((40 - w) / 2, 1), "oy": round((20 - h) / 2, 1),
                "vw": c.huella[0], "vh": c.huella[1], "c": c.color,
                "bd": c.borde, "rl": c.relleno,
                # el texto recortado al ancho de la celda: el nombre y el
                # titulo tienen que caber en la columna, no desbordarla
                "txt": recortar(c.nombre, ANCHO_TEXTO_CELDA, 11),
                "ttit": recortar(TITULOS.get(c.nombre, ""), ANCHO_TEXTO_CELDA, 9),
                "dd": c.declaracion, "p": c.path, "t": c.details,
            })
    return (
        "const LISTA = " + json.dumps(datos, ensure_ascii=False) + ";\n"
        + "panelId = Insert(document, {type:'frame', name:'Bloques Tab Screen', x:0, y:0, "
        + "width:1440, height:1000, layout:'vertical', gap:18, padding:36, fill:'$bg-app', "
        + "clip:true, placeholder:true});\n"
        + "const phId = Insert(panelId, {type:'frame', name:'Header', layout:'vertical', gap:8, "
        + "width:'fill_container'});\n"
        + "Insert(phId, {type:'text', name:'Title', content:'Pestana Bloques: la forma antes del "
        + "nombre', fontFamily:'Inter', fontSize:26, fontWeight:'700', fill:'$text-primary'});\n"
        + "Insert(phId, {type:'text', name:'Subtitle', content:'Cada clase del catalogo lleva la "
        + "miniatura de su silueta, agrupada por familia. Se reconoce la clase por la forma sin "
        + "leer el texto.', fontFamily:'Inter', fontSize:13, fill:'$text-secondary'});\n"
        + "const bodyId = Insert(panelId, {type:'frame', name:'Body', layout:'horizontal', gap:24, "
        + "width:'fill_container', height:'fill_container'});\n"
        + "const asideId = Insert(bodyId, {type:'frame', name:'Aside', layout:'vertical', gap:0, "
        + "width:400, height:880, fill:'$surface', stroke:'$border', strokeWidth:1, cornerRadius:10, "
        + "clip:true});\n"
        + "const tabsId = Insert(asideId, {type:'frame', name:'Tabs', layout:'horizontal', gap:4, "
        + "padding:10, width:'fill_container', fill:'$bg-app'});\n"
        + "const tb1 = Insert(tabsId, {type:'frame', name:'Tab Bloque', layout:'horizontal', "
        + "padding:[7,14], fill:'$surface', cornerRadius:6, stroke:'$border', strokeWidth:1});\n"
        + "Insert(tb1, {type:'text', name:'L', content:'Bloque', fontFamily:'Inter', fontSize:12, "
        + "fontWeight:'600', fill:'$text-secondary'});\n"
        + "const tb2 = Insert(tabsId, {type:'frame', name:'Tab Bloques', layout:'horizontal', "
        + "padding:[7,14], fill:'$accent-soft', cornerRadius:6});\n"
        + "Insert(tb2, {type:'text', name:'L', content:'Bloques', fontFamily:'Inter', fontSize:12, "
        + "fontWeight:'600', fill:'$accent-strong'});\n"
        + "const listId = Insert(asideId, {type:'frame', name:'List', layout:'vertical', gap:0, "
        + "width:'fill_container', padding:[6,0], fill:'$surface'});\n"
        + "let famActual = '';\n"
        + "let par = null;\n"
        + "let enPar = 0;\n"
        + "const fila = (c) => {\n"
        + "  const row = Insert(par, {type:'frame', name:'Row ' + c.n, layout:'horizontal', gap:8, "
        + "alignItems:'center', padding:[5,6], width:'fill_container'});\n"
        + "  const th = Insert(row, {type:'frame', name:'Thumb ' + c.n, layout:'none', width:40, "
        + "height:20});\n"
        + "  Insert(th, {type:'path', name:'S', geometry:c.p, viewBox:[0,0,c.vw,c.vh], x:c.ox, "
        + "y:c.oy, width:c.w, height:c.h, stroke: c.bd, "
        + "strokeWidth:1.6, strokeLinejoin:'round'});\n"
        + "  const tx = Insert(row, {type:'frame', name:'Tx ' + c.n, layout:'vertical', gap:0, "
        + "width:'fill_container'});\n"
        + "  Insert(tx, {type:'text', name:'N ' + c.n, content:c.txt, fontFamily:'JetBrains Mono', "
        + "fontSize:11, fontWeight:'600', fill:'$text-primary', textGrowth:'fixed-width', "
        + "width:'fill_container'});\n"
        + "  if (c.ttit) Insert(tx, {type:'text', name:'T ' + c.n, content:c.ttit, fontFamily:'Inter', "
        + "fontSize:9, fill:'$text-secondary', textGrowth:'fixed-width', width:'fill_container'});\n"
        + "};\n"
        + "for (const c of LISTA) {\n"
        + "  if (c.fam !== famActual) {\n"
        + "    famActual = c.fam;\n"
        + "    par = null;\n"
        + "    enPar = 0;\n"
        + "    const gh = Insert(listId, {type:'frame', name:'Group ' + famActual, layout:'horizontal', "
        + "gap:8, alignItems:'center', padding:[12,14,6,14], width:'fill_container'});\n"
        + "    Insert(gh, {type:'text', name:'N', content:famActual, fontFamily:'Inter', fontSize:11, "
        + "fontWeight:'700', fill:c.c});\n"
        + "  }\n"
        + "  if (enPar % 2 === 0) par = Insert(listId, {type:'frame', name:'Par', layout:'horizontal', "
        + "gap:12, width:'fill_container'});\n"
        + "  fila(c);\n"
        + "  enPar += 1;\n"
        + "}\n"
        + "Update(panelId, {placeholder:false});\n"
    )


# El ancho de un caracter de Inter, en proporcion del tamano de letra.
# El generador recorta el rotulo con esta medida, para que la hoja y el
# lienzo digan lo mismo que dira la app (que recorta con CSS).
ANCHO_CHARS = {
    " ": 0.28, ".": 0.29, ",": 0.29, ":": 0.29, ";": 0.29, "'": 0.22,
    "i": 0.28, "j": 0.28, "l": 0.28, "t": 0.36, "f": 0.34, "r": 0.38,
    "I": 0.30, "1": 0.55, "m": 0.88, "w": 0.74, "M": 0.88, "W": 0.94,
    "0": 0.55, "2": 0.55, "3": 0.55, "4": 0.55, "5": 0.55, "6": 0.55,
    "7": 0.55, "8": 0.55, "9": 0.55, "/": 0.40, "%": 0.86, "-": 0.36,
}
ANCHO_POR_DEFECTO = 0.55
ANCHO_MAYUSCULA = 0.68


def ancho_texto(texto: str, tamano: float) -> float:
    """El ancho de un texto en la tipografia de la interfaz."""
    total = 0.0
    for ch in texto:
        if ch in ANCHO_CHARS:
            total += ANCHO_CHARS[ch]
        elif ch.isupper():
            total += ANCHO_MAYUSCULA
        else:
            total += ANCHO_POR_DEFECTO
    return total * tamano


def recortar(texto: str, ancho: float, tamano: float) -> str:
    """El texto que cabe en un ancho dado, con puntos suspensivos si no cabe.

    Es lo que hace la app con `truncate`: el nombre largo se recorta con
    puntos, nunca desborda la zona ni pisa a la cifra.
    """
    if ancho <= 0 or ancho_texto(texto, tamano) <= ancho:
        return texto
    puntos = "…"
    disponible = ancho - ancho_texto(puntos, tamano)
    corte = ""
    for ch in texto:
        if ancho_texto(corte + ch, tamano) > disponible:
            break
        corte += ch
    return corte.rstrip() + puntos


def _clases_del_grupo(nombre_grupo: str) -> set[str]:
    """Las clases de un grupo, leidas de su propio entregable."""
    ruta = GRUPOS / f"{nombre_grupo}.md"
    if not ruta.exists():
        return set()
    return {c.nombre for c in leer_grupos(ruta)}


def titulos_del_catalogo() -> dict[str, str]:
    """El titulo de cada clase, como lo declara el catalogo del motor.

    El titulo es lo que dice la clase en palabras («Usar recurso»), y es lo
    que la lista muestra debajo del nombre: el nombre identifica la clase en
    el modelo, el titulo explica que hace.
    """
    raiz = Path(__file__).resolve().parents[3]
    ruta = raiz / "crates" / "genmsi-format" / "src" / "catalogue.rs"
    if not ruta.exists():
        return {}
    texto = ruta.read_text(encoding="utf-8")
    pares = re.findall(r'class: "([^"]+)",\n\s*title: "([^"]+)"', texto)
    return dict(pares)


TITULOS = titulos_del_catalogo()

# El ancho del texto de una celda de la lista: el panel mide 400 px, con
# 14 de padding a cada lado y 12 de hueco entre columnas; de ahi le queda
# media celda, menos el padding de la fila, la miniatura y su hueco.
ANCHO_TEXTO_CELDA = 120


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--svg", metavar="RUTA", help="escribe la hoja de contacto")
    parser.add_argument("--grupo", metavar="RUTA",
                        help="escribe la hoja de un solo grupo (A-lineal, B-entidad, C-grupos)")
    parser.add_argument("--pen", action="store_true", help="emite el fragmento para Pen")
    parser.add_argument("--panel", action="store_true", help="emite la pantalla de la pestana Bloques")
    args = parser.parse_args()

    clases = cargar_todos()
    incompletas = [c.nombre for c in clases if not c.path or not c.huella[0]]
    problemas = verificar(clases)
    if args.pen:
        sys.stdout.write(pen(clases))
    if args.panel:
        sys.stdout.write(panel(clases))
    if args.svg:
        Path(args.svg).write_text(svg(clases), encoding="utf-8")
    if args.grupo:
        destino = Path(args.grupo)
        grupo = [c for c in clases if c.nombre in _clases_del_grupo(destino.stem)]
        titulo = f"Grupo {destino.stem} — {len(grupo)} clases"
        destino.write_text(svg_grupo(grupo, titulo), encoding="utf-8")
        print(f"hoja de {destino.stem}: {len(grupo)} clases", file=sys.stderr)
    print(f"clases leidas: {len(clases)}", file=sys.stderr)
    if incompletas:
        print(f"sin path o huella: {incompletas}", file=sys.stderr)
    if problemas:
        print(f"problemas: {len(problemas)}", file=sys.stderr)
        for p in problemas:
            print(f"  {p}", file=sys.stderr)
        return 1
    print("reglas del contrato v5: OK", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
