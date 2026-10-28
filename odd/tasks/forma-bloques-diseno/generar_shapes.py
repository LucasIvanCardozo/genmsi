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
import math
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

# --------------------------------------------------------------------------
# La medida del nombre (contrato v2 §2.3)
#
# El nombre de una clase se escribe entero: en una linea si cabe en su zona
# de texto, y en dos si no cabe. La medida es la misma que mide la app
# (`src/graph/catalogue.ts`): el nombre va en la monoespaciada de 9 px, que
# mide 0,6 em por caracter, y deja 4 px de aire dentro de su zona.
# --------------------------------------------------------------------------

TAMANO_ROTULO = 9.0
EM_MONOESPACADA = 0.6
AIRE_ZONA = 4.0
ALTO_LINEA = 11.0
AIRE_ENTRE_LINEAS = 2.0
# Dos lineas de 9 px, con el aire del diseno entre ellas: 11 + 2 + 11.
ALTO_DOS_LINEAS = 2 * ALTO_LINEA + AIRE_ENTRE_LINEAS

# El aire que una zona deja entre su borde y un trazo interior (contrato
# v5 §2.5): el texto de la zona se dibuja encima del trazo si la zona lo
# toca, y dos pixeles son los que hacen falta para que se lean los dos.
AIRE_TRAZO = 2.0

# Las clases que no pueden tener cifra (contrato v5 §2.7).
#
# Son las declaraciones que el motor nunca recorre: no hay fila del reporte
# para ellas y no hay ningun valor que publicar, asi que una zona de cifra
# reservada seria un hueco vacio que parece un fallo. Las tres variables NO
# estan aqui: el motor publica el valor de una variable cuando la corrida
# se lo asigna, y la app lo muestra.
CLASES_SIN_CIFRA = [
    "Function",
    "FunctionDist",
    "FunctionExpon",
    "MFunction",
    "MFunctionDist",
    "Note",
    "QTable",
    "StorageDef",
    "Table",
]

# Las excepciones de la regla de los trazos interiores, y por que estan.
#
# `Unlink` es el unico caso que la regla no puede cubrir: su capsula mide
# 38 px de ancho y el nombre «Unlink» necesita 36,4 px de zona (32,4 px de
# texto mas 4 px de aire). Los 2 px de aire que la regla pide al eslabon
# (x=38) dejan una zona de 36 px como maximo, y el nombre dejaria de caber
# entero en una linea. Se conserva el nombre entero —el texto acaba a
# 2,6 px del eslabon— y la zona no lo cruza: es el unico caso en que la
# zona llega al eslabon sin tocarlo.
EXCEPCIONES_TRAZO = {
    ("Unlink", "texto"): (
        "el nombre entero («Unlink», 32,4 px) necesita una zona de 36,4 px y la "
        "capsula mide 38 px: con los 2 px de aire del eslabon la zona no puede "
        "ser mas ancha, y el nombre se partiria dentro de la palabra"
    ),
}


def ancho_texto(texto: str) -> float:
    """El ancho de un texto en la monoespaciada del diseno."""
    return len(texto.strip()) * EM_MONOESPACADA * TAMANO_ROTULO


def ancho_nombre(nombre: str) -> float:
    """El ancho que el nombre de la clase necesita."""
    return len(nombre) * EM_MONOESPACADA * TAMANO_ROTULO


def partir_nombre(nombre: str, ancho: float) -> tuple[str, str] | None:
    """Las dos lineas en que se escribe un nombre que no cabe en uno.

    El corte es el de la app (`src/graph/node.ts`): en un espacio, si el
    nombre tiene uno; si no, donde una mayuscula empieza una palabra
    («RemoveNumber» se parte en «Remove» y «Number», no en «Remov» y
    «eNumber»); y si no hay ni una cosa ni la otra, dentro de la palabra, en
    el punto que equilibra mejor las dos lineas.

    Devuelve `None` cuando el nombre no se puede partir en dos lineas que
    quepan.
    """
    mejor: tuple | None = None
    for indice in range(1, len(nombre)):
        primera, segunda = nombre[:indice], nombre[indice:]
        if not primera.strip() or not segunda.strip():
            continue
        candidato = (
            primera,
            segunda,
            # Un corte en un espacio conserva la palabra entera: el mejor es
            # el que cierra la primera linea, y el siguiente el que abre la
            # segunda. Un corte en una mayuscula conserva las palabras de un
            # nombre en camel case, y vale lo mismo que el del espacio.
            2 if primera.endswith(" ") else (
                1 if segunda.startswith(" ") or _mayuscula_empieza_palabra(nombre, indice) else 0
            ),
            ancho_texto(primera) <= ancho and ancho_texto(segunda) <= ancho,
            abs(len(primera) - len(segunda)),
        )
        if mejor is None or _corte_gana(candidato, mejor):
            mejor = candidato
    if mejor is None:
        return None
    return mejor[0], mejor[1]


MASCULAS = "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
MINUSCULAS_Y_DIGITOS = "abcdefghijklmnopqrstuvwxyz0123456789"


def _mayuscula_empieza_palabra(nombre: str, indice: int) -> bool:
    """El corte cae donde una mayuscula empieza una palabra.

    Los nombres de las clases van en camel case, y la mayuscula es donde el
    nombre dice que acaba una palabra.
    """
    letra = nombre[indice] if indice < len(nombre) else ""
    if letra not in MASCULAS:
        return False
    antes = nombre[indice - 1] if indice > 0 else ""
    return antes in MINUSCULAS_Y_DIGITOS


def _corte_gana(candidato: tuple, mejor: tuple) -> bool:
    """Un corte es mejor que otro si sus dos lineas caben, si esta en un
    espacio y si equilibra mejor las dos lineas."""
    if candidato[3] != mejor[3]:
        return candidato[3]
    if candidato[2] != mejor[2]:
        return candidato[2] > mejor[2]
    return candidato[4] < mejor[4]


def lineas_del_nombre(clase: render.Clase) -> list[str]:
    """Las lineas en que se escribe el nombre de la clase.

    Una si cabe en la zona, dos si no: el nombre nunca se corta.
    """
    ancho = clase.label[2] - AIRE_ZONA
    if ancho_nombre(clase.nombre) <= ancho:
        return [clase.nombre]
    partes = partir_nombre(clase.nombre, ancho)
    return list(partes) if partes is not None else []


# --------------------------------------------------------------------------
# La silueta: el `d` del path, en coordenadas de la huella
#
# El generador comprueba que las zonas viven **dentro** de la silueta
# rellena, y no dentro de su huella: el texto se dibuja ahi, y el texto no
# puede salirse de la figura.
# --------------------------------------------------------------------------

def _leer_path(d: str) -> list[list[tuple[float, float]]]:
    """Los contornos de un path, como listas de puntos.

    El diseno usa solo `M`, `L`, `H`, `V`, `A`, `C` y `Z`, y todos en
    coordenadas absolutas: basta con aplanar los arcos y las curvas.
    """
    fichas = re.findall(r"[A-Za-z]|-?\d*\.?\d+", d)
    contornos: list[list[tuple[float, float]]] = []
    contorno: list[tuple[float, float]] = []
    x = y = 0.0
    inicio = (0.0, 0.0)
    i = 0
    comando = ""
    while i < len(fichas):
        ficha = fichas[i]
        if re.fullmatch(r"[A-Za-z]", ficha):
            comando = ficha
            i += 1
            if comando == "Z":
                if contorno:
                    contorno.append(inicio)
                    contornos.append(contorno)
                    contorno = []
                x, y = inicio
                continue
        if comando == "M":
            x, y = float(fichas[i]), float(fichas[i + 1])
            i += 2
            if contorno:
                contornos.append(contorno)
            contorno = [(x, y)]
            inicio = (x, y)
            comando = "L"
        elif comando == "L":
            x, y = float(fichas[i]), float(fichas[i + 1])
            i += 2
            contorno.append((x, y))
        elif comando == "H":
            x = float(fichas[i])
            i += 1
            contorno.append((x, y))
        elif comando == "V":
            y = float(fichas[i])
            i += 1
            contorno.append((x, y))
        elif comando == "C":
            p1 = (float(fichas[i]), float(fichas[i + 1]))
            p2 = (float(fichas[i + 2]), float(fichas[i + 3]))
            p3 = (float(fichas[i + 4]), float(fichas[i + 5]))
            i += 6
            contorno.extend(_curva((x, y), p1, p2, p3))
            x, y = p3
        elif comando == "A":
            rx, ry, giro, grande, suave, ex, ey = (float(fichas[i + k]) for k in range(7))
            i += 7
            contorno.extend(_arco((x, y), rx, ry, giro, int(grande), int(suave), ex, ey))
            x, y = ex, ey
        else:
            raise ValueError(f"comando desconocido en el path: {comando!r} ({d})")
    if contorno:
        contornos.append(contorno)
    return contornos


def _curva(p0, p1, p2, p3, pasos: int = 16) -> list[tuple[float, float]]:
    """Una curva cubica, aplanada en segmentos."""
    puntos = []
    for i in range(1, pasos + 1):
        t = i / pasos
        m = 1 - t
        puntos.append((
            m**3 * p0[0] + 3 * m * m * t * p1[0] + 3 * m * t * t * p2[0] + t**3 * p3[0],
            m**3 * p0[1] + 3 * m * m * t * p1[1] + 3 * m * t * t * p2[1] + t**3 * p3[1],
        ))
    return puntos


def _arco(p0, rx, ry, giro, grande, suave, ex, ey, pasos: int = 24) -> list[tuple[float, float]]:
    """Un arco eliptico, aplanado en segmentos (SVG: de punto a punto)."""
    if rx == 0 or ry == 0:
        return [(ex, ey)]
    ang = math.radians(giro)
    cos, sen = math.cos(ang), math.sin(ang)
    dx, dy = (p0[0] - ex) / 2.0, (p0[1] - ey) / 2.0
    xp = cos * dx + sen * dy
    yp = -sen * dx + cos * dy
    rx, ry = abs(rx), abs(ry)
    lameta = xp * xp / (rx * rx) + yp * yp / (ry * ry)
    if lameta > 1:
        escala = math.sqrt(lameta)
        rx, ry = rx * escala, ry * escala
    numerador = rx * rx * ry * ry - rx * rx * yp * yp - ry * ry * xp * xp
    denominador = rx * rx * yp * yp + ry * ry * xp * xp
    co = math.sqrt(max(0.0, numerador / denominador)) if denominador else 0.0
    if grande == suave:
        co = -co
    cxp = co * rx * yp / ry
    cyp = -co * ry * xp / rx
    cx = cos * cxp - sen * cyp + (p0[0] + ex) / 2.0
    cy = sen * cxp + cos * cyp + (p0[1] + ey) / 2.0

    def angulo(ux, uy, vx, vy) -> float:
        largo = math.hypot(ux, uy) * math.hypot(vx, vy)
        if largo == 0:
            return 0.0
        a = math.acos(max(-1.0, min(1.0, (ux * vx + uy * vy) / largo)))
        return -a if ux * vy - uy * vx < 0 else a

    t1 = angulo(1, 0, (xp - cxp) / rx, (yp - cyp) / ry)
    delta = angulo((xp - cxp) / rx, (yp - cyp) / ry, (-xp - cxp) / rx, (-yp - cyp) / ry)
    if suave == 0 and delta > 0:
        delta -= 2 * math.pi
    elif suave == 1 and delta < 0:
        delta += 2 * math.pi
    puntos = []
    for i in range(1, pasos + 1):
        t = t1 + delta * i / pasos
        puntos.append((
            cos * rx * math.cos(t) - sen * ry * math.sin(t) + cx,
            sen * rx * math.cos(t) + cos * ry * math.sin(t) + cy,
        ))
    return puntos


def _vueltas(contornos, x, y) -> int:
    """Las vueltas del punto alrededor de la silueta."""
    vueltas = 0
    for contorno in contornos:
        for k in range(len(contorno) - 1):
            x0, y0 = contorno[k]
            x1, y1 = contorno[k + 1]
            if y0 <= y:
                if y1 > y and (x1 - x0) * (y - y0) - (x - x0) * (y1 - y0) > 0:
                    vueltas += 1
            elif y1 <= y and (x1 - x0) * (y - y0) - (x - x0) * (y1 - y0) < 0:
                vueltas -= 1
    return vueltas


def _en_borde(contornos, x, y, eps: float = 0.05) -> bool:
    """El punto esta en el borde de la silueta."""
    for contorno in contornos:
        for k in range(len(contorno) - 1):
            x0, y0 = contorno[k]
            x1, y1 = contorno[k + 1]
            cruz = (x1 - x0) * (y - y0) - (y1 - y0) * (x - x0)
            if abs(cruz) > eps:
                continue
            if (min(x0, x1) - eps <= x <= max(x0, x1) + eps
                    and min(y0, y1) - eps <= y <= max(y0, y1) + eps):
                return True
    return False


def _corta_caja(a, b, caja) -> bool:
    """El segmento `a`-`b` corta el interior de la caja."""
    x, y, ancho, alto = caja
    x0, y0 = a
    x1, y1 = b
    dx, dy = x1 - x0, y1 - y0
    t0, t1 = 0.0, 1.0
    for p, q in ((-dx, x0 - x), (dx, x + ancho - x0), (-dy, y0 - y), (dy, y + alto - y0)):
        if p == 0:
            if q < 0:
                return False
            continue
        t = q / p
        if p < 0:
            if t > t1:
                return False
            t0 = max(t0, t)
        else:
            if t < t0:
                return False
            t1 = min(t1, t)
    return t1 > t0 + 1e-9


def _zona_dentro(contornos, caja) -> tuple[bool, str]:
    """La caja esta dentro de la silueta rellena?"""
    x, y, ancho, alto = caja
    esquinas = [(x, y), (x + ancho, y), (x + ancho, y + alto), (x, y + alto)]
    for px, py in esquinas:
        if _vueltas(contornos, px, py) == 0 and not _en_borde(contornos, px, py):
            return False, f"la esquina ({px:g},{py:g}) se sale de la silueta"
    for contorno in contornos:
        for k in range(len(contorno) - 1):
            if _corta_caja(contorno[k], contorno[k + 1], caja):
                return False, "la silueta corta la zona"
    return True, ""


def problema_trazo(clase: render.Clase) -> str | None:
    """Los trazos interiores tienen que vivir dentro de la silueta.

    Un trazo que asoma por fuera del borde parece un trozo suelto: es la
    marca del parametro de la multifunlcaci6n por par6metro, que se salia
    media pixela por encima del peine.
    """
    if not clase.path or not clase.details:
        return None
    contornos = _leer_path(clase.path)
    for detalle in clase.details:
        for contorno in _leer_path(detalle):
            for x, y in contorno:
                if _vueltas(contornos, x, y) == 0 and not _en_borde(contornos, x, y):
                    return (
                        f"{clase.nombre}: el trazo interior se sale de la silueta "
                        f"en ({x:g},{y:g})"
                    )
    return None


def _distancia_punto_segmento(punto, a, b) -> float:
    """La distancia del punto al segmento `a`-`b`."""
    ax, ay = a
    bx, by = b
    px, py = punto
    dx, dy = bx - ax, by - ay
    largo = dx * dx + dy * dy
    if largo == 0:
        return math.hypot(px - ax, py - ay)
    t = max(0.0, min(1.0, ((px - ax) * dx + (py - ay) * dy) / largo))
    return math.hypot(px - (ax + t * dx), py - (ay + t * dy))


def _orientacion(p, q, r) -> float:
    return (q[0] - p[0]) * (r[1] - p[1]) - (q[1] - p[1]) * (r[0] - p[0])


def _en_segmento(p, q, r) -> bool:
    """El punto `q` esta en el segmento `p`-`r`."""
    return (
        min(p[0], r[0]) - 1e-9 <= q[0] <= max(p[0], r[0]) + 1e-9
        and min(p[1], r[1]) - 1e-9 <= q[1] <= max(p[1], r[1]) + 1e-9
    )


def _corta_segmento(a, b, c, d) -> bool:
    """Los segmentos `a`-`b` y `c`-`d` se cortan."""
    o1, o2 = _orientacion(a, b, c), _orientacion(a, b, d)
    o3, o4 = _orientacion(c, d, a), _orientacion(c, d, b)
    if ((o1 > 0) != (o2 > 0)) and ((o3 > 0) != (o4 > 0)):
        return True
    if o1 == 0 and _en_segmento(a, c, b):
        return True
    if o2 == 0 and _en_segmento(a, d, b):
        return True
    if o3 == 0 and _en_segmento(c, a, d):
        return True
    if o4 == 0 and _en_segmento(c, b, d):
        return True
    return False


def _distancia_segmentos(a, b, c, d) -> float:
    """La distancia entre los segmentos `a`-`b` y `c`-`d`."""
    if _corta_segmento(a, b, c, d):
        return 0.0
    return min(
        _distancia_punto_segmento(a, c, d),
        _distancia_punto_segmento(b, c, d),
        _distancia_punto_segmento(c, a, b),
        _distancia_punto_segmento(d, a, b),
    )


def _distancia_segmento_caja(a, b, caja) -> float:
    """La distancia del segmento `a`-`b` a la caja.

    Cero cuando el segmento la cruza o la toca, y la distancia al borde de
    la caja cuando vive fuera de ella.
    """
    if _corta_caja(a, b, caja):
        return 0.0
    x, y, ancho, alto = caja
    esquinas = [(x, y), (x + ancho, y), (x + ancho, y + alto), (x, y + alto)]
    mejor = float("inf")
    for indice in range(4):
        c = esquinas[indice]
        d = esquinas[(indice + 1) % 4]
        mejor = min(mejor, _distancia_segmentos(a, b, c, d))
    return mejor


def problema_zona_toca_trazo(clase: render.Clase) -> list[str]:
    """Ninguna zona puede cruzar un trazo interior de la clase.

    El texto de la zona se dibuja encima del trazo, y un trazo debajo del
    texto es un texto ilegible: el subtítulo de una tabla encima de la linea
    de la tabla, o el rótulo de una multifurcación encima del raíl de su
    parámetro. La zona deja `AIRE_TRAZO` de aire entre su borde y el trazo
    (contrato v5 §2.5).
    """
    if not clase.details:
        return []
    problemas: list[str] = []
    zonas: list[tuple[str, tuple[float, float, float, float]]] = []
    if clase.label is not None:
        zonas.append(("texto", clase.label))
    if clase.figure is not None:
        zonas.append(("cifra", clase.figure))
    for nombre_zona, caja in zonas:
        for detalle in clase.details:
            if (clase.nombre, nombre_zona) in EXCEPCIONES_TRAZO:
                continue
            minima = None
            for contorno in _leer_path(detalle):
                for indice in range(len(contorno) - 1):
                    a, b = contorno[indice], contorno[indice + 1]
                    distancia = _distancia_segmento_caja(a, b, caja)
                    if minima is None or distancia < minima:
                        minima = distancia
            if minima is not None and minima < AIRE_TRAZO - 1e-9:
                problemas.append(
                    f"{clase.nombre}: la zona de {nombre_zona} "
                    f"({caja[0]:g},{caja[1]:g},{caja[2]:g},{caja[3]:g}) toca el "
                    f"trazo interior «{detalle}»: deja {minima:.1f} px de "
                    f"aire y hacen falta {AIRE_TRAZO:g} px"
                )
    return problemas


def problema_cifra_imposible(clase: render.Clase) -> str | None:
    """Una clase que no puede tener cifra no declara zona de cifra.

    El motor nunca recorre una declaracion: no hay fila del reporte para
    ella y no hay ningun valor que publicar. Una zona de cifra reservada
    seria un hueco vacio que parece un fallo (contrato v5 §2.7).
    """
    if clase.nombre not in CLASES_SIN_CIFRA:
        return None
    if clase.figure is None:
        return None
    return (
        f"{clase.nombre}: declara zona de cifra "
        f"({clase.figure[0]:g},{clase.figure[1]:g},{clase.figure[2]:g},"
        f"{clase.figure[3]:g}) y es una clase que no puede tener cifra: el motor "
        f"nunca la recorre y no publica ningun valor para ella, asi que la zona "
        f"quedaria siempre vacia"
    )


def problema_corte(clase: render.Clase) -> str | None:
    """El nombre se parte donde empieza una palabra.

    Los nombres de las clases van en camel case, y la mayuscula es donde el
    nombre dice que acaba una palabra: cuando el nombre no cabe en una
    linea, las dos lineas tienen que caer en una mayuscula, para que las
    palabras enteras se lean («Remove» y «Number», no «Remov» y «eNumber»).

    Los nombres que son una sola palabra («Multifurcacion») no tienen
    mayuscula: se parten donde mejor queden, y no hay nada que exigir. La
    mayuscula que se queda sola al final del nombre («MultifurcacionP») es un
    sufijo, no el principio de una palabra, y tampoco exige nada.
    """
    ancho = clase.label[2] - AIRE_ZONA
    if ancho_nombre(clase.nombre) <= ancho:
        return None
    cortes = [
        indice
        for indice in range(1, len(clase.nombre))
        if _mayuscula_empieza_palabra(clase.nombre, indice)
        # Una mayuscula que se queda sola al final del nombre es un sufijo
        # («MultifurcacionP»), no el principio de una palabra.
        and len(clase.nombre) - indice >= 2
    ]
    if not cortes:
        return None
    for indice in cortes:
        if (ancho_texto(clase.nombre[:indice]) <= ancho
                and ancho_texto(clase.nombre[indice:]) <= ancho):
            return None
    mejor = min(
        cortes,
        key=lambda indice: max(ancho_texto(clase.nombre[:indice]), ancho_texto(clase.nombre[indice:])),
    )
    return (
        f"{clase.nombre}: el nombre se partiria dentro de una palabra: la zona de "
        f"texto tiene {clase.label[2]:g} px (util {ancho:g} px) y el corte en "
        f"«{clase.nombre[:mejor]}»/«{clase.nombre[mejor:]}» necesita "
        f"{max(ancho_texto(clase.nombre[:mejor]), ancho_texto(clase.nombre[mejor:])):g} px"
    )


def problema_rotulo(clase: render.Clase) -> str | None:
    """El nombre de la clase tiene que caber en su zona de texto.

    En una linea, o en dos si la zona tiene sitio para las dos. El nombre
    nunca se corta: el generador no escribe el modulo si una clase no
    cumple la regla.
    """
    ancho_zona = clase.label[2]
    ancho = ancho_zona - AIRE_ZONA
    necesita = ancho_nombre(clase.nombre)
    if necesita <= ancho:
        return None
    if clase.label[3] < ALTO_DOS_LINEAS:
        return (
            f"{clase.nombre}: el nombre necesita {necesita:g} px y la zona de texto "
            f"tiene {ancho_zona:g} px (util {ancho:g} px), y la zona es demasiado baja "
            f"para las dos lineas ({clase.label[3]:g} px de {ALTO_DOS_LINEAS:g} px)"
        )
    lineas = partir_nombre(clase.nombre, ancho)
    if lineas is None:
        return (
            f"{clase.nombre}: el nombre necesita {necesita:g} px y la zona de texto "
            f"tiene {ancho_zona:g} px (util {ancho:g} px): no cabe en dos lineas"
        )
    for linea in lineas:
        if ancho_texto(linea) > ancho:
            return (
                f"{clase.nombre}: el nombre necesita {necesita:g} px y la zona de "
                f"texto tiene {ancho_zona:g} px (util {ancho:g} px): la linea "
                f"«{linea}» necesita {ancho_texto(linea):g} px"
            )
    return None



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
            # el nombre de la clase tiene que caber en su zona, en una linea
            # o en dos (contrato v2 §2.3)
            problema = problema_rotulo(clase)
            if problema is not None:
                problemas.append(problema)
            # y las dos lineas tienen que caer donde empieza una palabra
            problema = problema_corte(clase)
            if problema is not None:
                problemas.append(problema)
            # y la zona tiene que vivir dentro de la silueta rellena
            contornos = _leer_path(clase.path)
            dentro, porque = _zona_dentro(contornos, clase.label)
            if not dentro:
                problemas.append(
                    f"{clase.nombre}: la zona de texto se sale de la silueta "
                    f"({clase.label} en {w}x{h}): {porque}"
                )
        # y ninguna zona puede cruzar un trazo interior (contrato v5 §2.5)
        for problema in problema_zona_toca_trazo(clase):
            problemas.append(problema)
        if clase.figure is not None:
            fx, fy, fw, fh = clase.figure
            if fx < 0 or fy < 0 or fx + fw > w + 0.5 or fy + fh > h + 0.5:
                problemas.append(
                    f"{clase.nombre}: zona de cifra fuera ({clase.figure} en {w}x{h})"
                )
            contornos = _leer_path(clase.path)
            dentro, porque = _zona_dentro(contornos, clase.figure)
            if not dentro:
                problemas.append(
                    f"{clase.nombre}: la zona de cifra se sale de la silueta "
                    f"({clase.figure} en {w}x{h}): {porque}"
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
        # los trazos interiores tampoco salen de la silueta
        problema = problema_trazo(clase)
        if problema is not None:
            problemas.append(problema)
        # y una clase que no puede tener cifra no declara zona de cifra
        problema = problema_cifra_imposible(clase)
        if problema is not None:
            problemas.append(problema)
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
    sin_cifra = "".join(f"  {texto(clase)},\n" for clase in CLASES_SIN_CIFRA)
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

/**
 * The classes that can never have a figure.
 *
 * They are the declarations the engine never walks into: there is no row of
 * the report for them and no value to publish, so a reserved zone of the
 * figure would be an empty hole that looks like a fault (contract v5 §2.7).
 * The three variables are not among them: the engine publishes the value of
 * a variable when the run assigns it.
 *
 * The list lives here so that the app reads it from the module instead of
 * repeating it: the design's generator is the one that writes it.
 */
export const CLASSES_WITHOUT_FIGURE: string[] = [
{sin_cifra}];

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
