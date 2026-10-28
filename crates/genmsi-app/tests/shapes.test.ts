//! The shape of every block class.

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import {
  BLOCK_SHAPES,
  CLASSES_WITHOUT_FIGURE,
  FAMILY_COLORS,
  SHAPE_CLASSES,
  colorsOf,
  familyColors,
  shapeOf,
  type Zone,
} from "../src/graph/shapes.ts";
import { monospaceWidth } from "../src/graph/catalogue.ts";
import {
  LABEL_SIZE,
  LINE_AIR,
  LINE_HEIGHT,
  ZONE_AIR,
  labelLines,
  type FootprintZone,
} from "../src/graph/node.ts";

/** The classes the catalogue of the engine declares. */
function catalogueClasses(): string[] {
  const url = new URL("../../genmsi-format/src/catalogue.rs", import.meta.url);
  const source = readFileSync(url, "utf8");
  return [...source.matchAll(/class: "([^"]+)"/g)].map((match) => match[1]);
}

/** The footprint the engine gives each class, in its own canvas units. */
function catalogueFootprints(): Map<string, { width: number; height: number }> {
  const url = new URL("../../genmsi-format/src/catalogue.rs", import.meta.url);
  const source = readFileSync(url, "utf8");
  const footprints = new Map<string, { width: number; height: number }>();
  for (const match of source.matchAll(
    /class: "([^"]+)",\s*\n\s*title: "[^"]*",\s*\n\s*width: (\d+),\s*\n\s*height: (\d+)/g,
  )) {
    footprints.set(match[1], { width: Number(match[2]), height: Number(match[3]) });
  }
  return footprints;
}

/** One command of an SVG path, with its arguments. */
interface PathCommand {
  /** The letter of the command (`M`, `L`, `A`, ...). */
  letter: string;
  /** The arguments of the command, in order. */
  args: number[];
}

/** The commands of an SVG path, parsed. */
function parsePath(d: string): PathCommand[] {
  const tokens = d.match(/[A-Za-z]|-?\d*\.?\d+/g) ?? [];
  const commands: PathCommand[] = [];
  let current: PathCommand | undefined;
  for (const token of tokens) {
    if (/^[A-Za-z]$/.test(token)) {
      current = { letter: token, args: [] };
      commands.push(current);
      continue;
    }
    assert.ok(current !== undefined, `a path cannot start with a number: ${d}`);
    current.args.push(Number(token));
  }
  return commands;
}

/**
 * The path reflected on the vertical axis of the footprint.
 *
 * The horizontal coordinates are mirrored about the width of the footprint
 * and the sweep of every arc is inverted, because a mirror turns the arc
 * the other way round.
 */
function reflect(command: PathCommand, width: number): PathCommand {
  const args = command.args.slice();
  switch (command.letter) {
    case "M":
    case "L":
    case "T":
    case "H":
      args[0] = width - args[0];
      break;
    case "A":
      args[4] = 1 - args[4];
      args[5] = width - args[5];
      break;
    case "C":
    case "S":
    case "Q":
      args[0] = width - args[0];
      args[2] = width - args[2];
      args[4] = width - args[4];
      break;
    case "V":
    case "Z":
      break;
    default:
      throw new Error(`comando desconocido: ${command.letter}`);
  }
  return { letter: command.letter, args };
}

/** Whether two zones of the footprint overlap (touching is not overlapping). */
function overlaps(a: Zone, b: Zone): boolean {
  return a.x < b.x + b.width && b.x < a.x + a.width && a.y < b.y + b.height && b.y < a.y + a.height;
}

/**
 * The air a zone keeps from an interior stroke, in pixels.
 *
 * The rule of the design (contract v5 §2.5): the rectangle of a zone does
 * not touch any of the interior strokes of its class.
 */
const STROKE_AIR = 2;

/** A point of a stroke, in the pixels of the footprint. */
type Point = [number, number];

/**
 * The points of an interior stroke, flattened into polylines.
 *
 * The arcs become polylines of 24 steps, the same flattening the design's
 * generator uses, so the two agree on where a stroke is.
 */
function strokePolylines(d: string): Point[][] {
  const tokens = d.match(/[A-Za-z]|-?\d*\.?\d+/g) ?? [];
  const polylines: Point[][] = [];
  let current: Point[] = [];
  let cursor: Point = [0, 0];
  let start: Point = [0, 0];
  let letter = "";
  let index = 0;
  while (index < tokens.length) {
    const token = tokens[index] ?? "";
    if (/^[A-Za-z]$/.test(token)) {
      letter = token;
      index += 1;
      if (letter === "Z") {
        if (current.length > 0) {
          current.push(start);
          polylines.push(current);
          current = [];
        }
        cursor = start;
        continue;
      }
    }
    if (letter === "M") {
      cursor = [Number(tokens[index]), Number(tokens[index + 1])];
      index += 2;
      if (current.length > 0) {
        polylines.push(current);
      }
      current = [cursor];
      start = cursor;
      letter = "L";
    } else if (letter === "L") {
      cursor = [Number(tokens[index]), Number(tokens[index + 1])];
      index += 2;
      current.push(cursor);
    } else if (letter === "H") {
      cursor = [Number(tokens[index]), cursor[1]];
      index += 1;
      current.push(cursor);
    } else if (letter === "V") {
      cursor = [cursor[0], Number(tokens[index])];
      index += 1;
      current.push(cursor);
    } else if (letter === "C") {
      const p1: Point = [Number(tokens[index]), Number(tokens[index + 1])];
      const p2: Point = [Number(tokens[index + 2]), Number(tokens[index + 3])];
      const p3: Point = [Number(tokens[index + 4]), Number(tokens[index + 5])];
      index += 6;
      current.push(...flattenCurve(cursor, p1, p2, p3));
      cursor = p3;
    } else if (letter === "A") {
      const args = tokens.slice(index, index + 7).map(Number);
      index += 7;
      const to: Point = [args[5] ?? 0, args[6] ?? 0];
      current.push(...flattenArc(cursor, args, to));
      cursor = to;
    } else {
      throw new Error(`comando desconocido en un trazo: ${letter}`);
    }
  }
  if (current.length > 0) {
    polylines.push(current);
  }
  return polylines;
}

/**
 * The points of a cubic curve, flattened.
 *
 * The curve goes from the point the path is at to the third control point,
 * with the first and the second as its handles.
 */
function flattenCurve(from: Point, p1: Point, p2: Point, p3: Point, steps = 16): Point[] {
  const points: Point[] = [];
  for (let index = 1; index <= steps; index += 1) {
    const t = index / steps;
    const m = 1 - t;
    points.push([
      m * m * m * from[0] + 3 * m * m * t * p1[0] + 3 * m * t * t * p2[0] + t * t * t * p3[0],
      m * m * m * from[1] + 3 * m * m * t * p1[1] + 3 * m * t * t * p2[1] + t * t * t * p3[1],
    ]);
  }
  return points;
}

/**
 * The points of an elliptical arc, flattened.
 *
 * The parametrization is the one of the SVG: the arc goes from the point
 * `from` to the point `to`, and its radii are the ones the path declares.
 */
function flattenArc(from: Point, args: number[], to: Point, steps = 24): Point[] {
  let rx = args[0] ?? 0;
  let ry = args[1] ?? 0;
  const rotation = args[2] ?? 0;
  const largeArc = args[3] ?? 0;
  const sweep = args[4] ?? 0;
  if (rx === 0 || ry === 0) {
    return [to];
  }
  const phi = (rotation * Math.PI) / 180;
  const cos = Math.cos(phi);
  const sin = Math.sin(phi);
  const dx = (from[0] - to[0]) / 2;
  const dy = (from[1] - to[1]) / 2;
  const xp = cos * dx + sin * dy;
  const yp = -sin * dx + cos * dy;
  rx = Math.abs(rx);
  ry = Math.abs(ry);
  const lambda = (xp * xp) / (rx * rx) + (yp * yp) / (ry * ry);
  if (lambda > 1) {
    const scale = Math.sqrt(lambda);
    rx *= scale;
    ry *= scale;
  }
  const numerator = rx * rx * ry * ry - rx * rx * yp * yp - ry * ry * xp * xp;
  const denominator = rx * rx * yp * yp + ry * ry * xp * xp;
  let co = denominator === 0 ? 0 : Math.sqrt(Math.max(0, numerator / denominator));
  if (largeArc === sweep) {
    co = -co;
  }
  const cxp = (co * rx * yp) / ry;
  const cyp = (-co * ry * xp) / rx;
  const cx = cos * cxp - sin * cyp + (from[0] + to[0]) / 2;
  const cy = sin * cxp + cos * cyp + (from[1] + to[1]) / 2;
  const angle = (ux: number, uy: number, vx: number, vy: number): number => {
    const length = Math.hypot(ux, uy) * Math.hypot(vx, vy);
    if (length === 0) {
      return 0;
    }
    const a = Math.acos(Math.max(-1, Math.min(1, (ux * vx + uy * vy) / length)));
    return ux * vy - uy * vx < 0 ? -a : a;
  };
  const t1 = angle(1, 0, (xp - cxp) / rx, (yp - cyp) / ry);
  let delta = angle((xp - cxp) / rx, (yp - cyp) / ry, (-xp - cxp) / rx, (-yp - cyp) / ry);
  if (sweep === 0 && delta > 0) {
    delta -= 2 * Math.PI;
  } else if (sweep === 1 && delta < 0) {
    delta += 2 * Math.PI;
  }
  const points: Point[] = [];
  for (let index = 1; index <= steps; index += 1) {
    const t = t1 + (delta * index) / steps;
    points.push([
      cos * rx * Math.cos(t) - sin * ry * Math.sin(t) + cx,
      sin * rx * Math.cos(t) + cos * ry * Math.sin(t) + cy,
    ]);
  }
  return points;
}

/** The distance from a point to the segment `a`-`b`. */
function pointToSegment(point: Point, a: Point, b: Point): number {
  const dx = b[0] - a[0];
  const dy = b[1] - a[1];
  const length = dx * dx + dy * dy;
  if (length === 0) {
    return Math.hypot(point[0] - a[0], point[1] - a[1]);
  }
  const t = Math.max(
    0,
    Math.min(1, ((point[0] - a[0]) * dx + (point[1] - a[1]) * dy) / length),
  );
  return Math.hypot(point[0] - (a[0] + t * dx), point[1] - (a[1] + t * dy));
}

/** The orientation of the three points, for the test of crossing. */
function orientation(p: Point, q: Point, r: Point): number {
  return (q[0] - p[0]) * (r[1] - p[1]) - (q[1] - p[1]) * (r[0] - p[0]);
}

/** Whether the point `q` is on the segment `p`-`r`. */
function onSegment(p: Point, q: Point, r: Point): boolean {
  return (
    Math.min(p[0], r[0]) - 1e-9 <= q[0] &&
    q[0] <= Math.max(p[0], r[0]) + 1e-9 &&
    Math.min(p[1], r[1]) - 1e-9 <= q[1] &&
    q[1] <= Math.max(p[1], r[1]) + 1e-9
  );
}

/** Whether the segments `a`-`b` and `c`-`d` cross. */
function segmentsCross(a: Point, b: Point, c: Point, d: Point): boolean {
  const o1 = orientation(a, b, c);
  const o2 = orientation(a, b, d);
  const o3 = orientation(c, d, a);
  const o4 = orientation(c, d, b);
  if (o1 > 0 !== o2 > 0 && o3 > 0 !== o4 > 0) {
    return true;
  }
  if (o1 === 0 && onSegment(a, c, b)) {
    return true;
  }
  if (o2 === 0 && onSegment(a, d, b)) {
    return true;
  }
  if (o3 === 0 && onSegment(c, a, d)) {
    return true;
  }
  if (o4 === 0 && onSegment(c, b, d)) {
    return true;
  }
  return false;
}

/** The distance between the segments `a`-`b` and `c`-`d`. */
function segmentDistance(a: Point, b: Point, c: Point, d: Point): number {
  if (segmentsCross(a, b, c, d)) {
    return 0;
  }
  return Math.min(
    pointToSegment(a, c, d),
    pointToSegment(b, c, d),
    pointToSegment(c, a, b),
    pointToSegment(d, a, b),
  );
}

/** Whether the segment `a`-`b` cuts the interior of the zone. */
function segmentCutsZone(a: Point, b: Point, zone: Zone): boolean {
  const dx = b[0] - a[0];
  const dy = b[1] - a[1];
  let t0 = 0;
  let t1 = 1;
  const bounds: Array<[number, number]> = [
    [-dx, a[0] - zone.x],
    [dx, zone.x + zone.width - a[0]],
    [-dy, a[1] - zone.y],
    [dy, zone.y + zone.height - a[1]],
  ];
  for (const [p, q] of bounds) {
    if (p === 0) {
      if (q < 0) {
        return false;
      }
      continue;
    }
    const t = q / p;
    if (p < 0) {
      if (t > t1) {
        return false;
      }
      t0 = Math.max(t0, t);
    } else {
      if (t < t0) {
        return false;
      }
      t1 = Math.min(t1, t);
    }
  }
  return t1 > t0 + 1e-9;
}

/** The distance from the segment `a`-`b` to the zone. */
function segmentToZone(a: Point, b: Point, zone: Zone): number {
  if (segmentCutsZone(a, b, zone)) {
    return 0;
  }
  const corners: Point[] = [
    [zone.x, zone.y],
    [zone.x + zone.width, zone.y],
    [zone.x + zone.width, zone.y + zone.height],
    [zone.x, zone.y + zone.height],
  ];
  let closest = Number.POSITIVE_INFINITY;
  for (let index = 0; index < corners.length; index += 1) {
    const c = corners[index] as Point;
    const d = corners[(index + 1) % corners.length] as Point;
    closest = Math.min(closest, segmentDistance(a, b, c, d));
  }
  return closest;
}

/** The three colour levels the design settled on, one per family. */
const APPROVED_COLORS: Record<string, { acento: string; borde: string; relleno: string }> = {
  Fuente: { acento: "#047857", borde: "#b3c4bc", relleno: "#f3f6f5" },
  Destino: { acento: "#be123c", borde: "#deb3b7", relleno: "#f9f4f4" },
  Decisión: { acento: "#4338ca", borde: "#b8b7e4", relleno: "#f4f4fa" },
  Cola: { acento: "#b45309", borde: "#dabbb3", relleno: "#f9f4f3" },
  Recurso: { acento: "#0f766e", borde: "#b3c4c1", relleno: "#f4f6f5" },
  Depósito: { acento: "#0369a1", borde: "#b3c0d2", relleno: "#f3f5f8" },
  Tabla: { acento: "#6d28d9", borde: "#c1b5eb", relleno: "#f5f4fc" },
  Cadena: { acento: "#c2410c", borde: "#e0b8b3", relleno: "#faf4f3" },
  Grupo: { acento: "#be185d", borde: "#deb4bd", relleno: "#f9f4f5" },
  Selección: { acento: "#0e7490", borde: "#b3c3cc", relleno: "#f3f6f7" },
  Operación: { acento: "#475569", borde: "#b9bcc0", relleno: "#f4f5f5" },
  Anotación: { acento: "#64748b", borde: "#bfc3ca", relleno: "#f5f6f6" },
};

test("the module knows every class of the catalogue, and nothing else", () => {
  const catalogue = new Set(catalogueClasses());
  assert.equal(catalogue.size, 66, "the catalogue declares 66 classes");
  const known = new Set(Object.keys(BLOCK_SHAPES));
  assert.equal(known.size, 66, "the module knows 66 classes");
  for (const name of catalogue) {
    assert.ok(known.has(name), `the module does not know ${name}`);
  }
  for (const name of known) {
    assert.ok(catalogue.has(name), `${name} is not a class of the catalogue`);
  }
});

test("the list of classes and the table of shapes are the same", () => {
  assert.equal(SHAPE_CLASSES.length, 66);
  assert.equal(new Set(SHAPE_CLASSES).size, 66);
  assert.deepEqual(new Set(SHAPE_CLASSES), new Set(Object.keys(BLOCK_SHAPES)));
  for (const name of SHAPE_CLASSES) {
    assert.ok(shapeOf(name) !== undefined);
    assert.equal(shapeOf(name)?.family, BLOCK_SHAPES[name].family);
  }
  assert.equal(shapeOf("NoSuchClass"), undefined);
});

test("every class has a silhouette, a footprint, a family and a variant", () => {
  for (const [name, shape] of Object.entries(BLOCK_SHAPES)) {
    assert.ok(shape.path.length > 0, `${name} has no path`);
    assert.ok(shape.footprint.width > 0, `${name} has no width`);
    assert.ok(shape.footprint.height > 0, `${name} has no height`);
    assert.ok(shape.family.length > 0, `${name} has no family`);
    assert.ok(shape.variant.length > 0, `${name} has no variant`);
    assert.ok(Object.hasOwn(FAMILY_COLORS, shape.family), `${name} has an unknown family`);
    assert.ok(shape.details.every((detail) => detail.length > 0), `${name} has an empty detail`);
  }
});

test("the ports of a class sit inside its footprint", () => {
  for (const [name, shape] of Object.entries(BLOCK_SHAPES)) {
    for (const port of shape.ports) {
      assert.ok(port.name.length > 0, `${name} has a port without name`);
      assert.ok(port.x >= 0 && port.x <= 1, `${name}.${port.name} is outside: x=${port.x}`);
      assert.ok(port.y >= 0 && port.y <= 1, `${name}.${port.name} is outside: y=${port.y}`);
    }
  }
});

test("the classes of the flow have their ports", () => {
  // A declaration has no ports: it does not take part in the flow.
  const declarations = [
    "Clock",
    "Function",
    "FunctionDist",
    "FunctionExpon",
    "InitiaSV",
    "InitialLS",
    "MFunction",
    "MFunctionDist",
    "Note",
    "QTable",
    "StorageDef",
    "Table",
    "Variable",
    "VariableA",
    "VariableN",
  ];
  for (const name of Object.keys(BLOCK_SHAPES)) {
    const ports = BLOCK_SHAPES[name].ports;
    if (declarations.includes(name)) {
      assert.equal(ports.length, 0, `${name} is a declaration and has no ports`);
    } else {
      assert.ok(ports.length > 0, `${name} takes part in the flow and has no ports`);
    }
  }
});

test("the zones of text and of figure sit inside the footprint", () => {
  for (const [name, shape] of Object.entries(BLOCK_SHAPES)) {
    const { width, height } = shape.footprint;
    const label = shape.label;
    assert.ok(label.x >= 0 && label.y >= 0, `${name}: the text zone starts outside`);
    assert.ok(
      label.x + label.width <= width + 0.5,
      `${name}: the text zone ends outside (${label.x + label.width} of ${width})`,
    );
    assert.ok(
      label.y + label.height <= height + 0.5,
      `${name}: the text zone ends outside (${label.y + label.height} of ${height})`,
    );
    const figure = shape.figure;
    if (figure !== null) {
      assert.ok(figure.x >= 0 && figure.y >= 0, `${name}: the figure zone starts outside`);
      assert.ok(
        figure.x + figure.width <= width + 0.5,
        `${name}: the figure zone ends outside (${figure.x + figure.width} of ${width})`,
      );
      assert.ok(
        figure.y + figure.height <= height + 0.5,
        `${name}: the figure zone ends outside (${figure.y + figure.height} of ${height})`,
      );
    }
  }
});

test("the text and the figure of a class never overlap", () => {
  for (const [name, shape] of Object.entries(BLOCK_SHAPES)) {
    const figure = shape.figure;
    if (figure === null) {
      continue;
    }
    assert.ok(!overlaps(shape.label, figure), `${name}: the text and the figure overlap`);
  }
});

test("the name of every class fits its zone, in one line or in two", () => {
  // The name of a class is never cut: it is written whole, in one line when
  // it fits its zone, and in two lines when it does not (contract v2 §2.3).
  // The measure is the one of the design: the name is monospaced, and the
  // zone leaves 4 px of air inside it.
  for (const [name, shape] of Object.entries(BLOCK_SHAPES)) {
    const zone: FootprintZone = {
      left: shape.label.x,
      top: shape.label.y,
      width: shape.label.width,
      height: shape.label.height,
    };
    const room = zone.width - ZONE_AIR;
    const lines = labelLines(name, zone);
    assert.ok(lines.length === 1 || lines.length === 2, `${name}: one line or two`);
    assert.equal(lines.join(""), name, `${name}: the whole name is written`);
    assert.ok(!lines.join("").includes("…"), `${name}: the name is never cut`);
    if (lines.length === 1) {
      assert.ok(
        monospaceWidth(name, LABEL_SIZE) <= room,
        `${name}: the name fits in one line (${monospaceWidth(name, LABEL_SIZE)} of ${room})`,
      );
      continue;
    }
    // Two lines: the zone has to hold them, and both have to fit it.
    assert.ok(
      zone.height >= 2 * LINE_HEIGHT + LINE_AIR,
      `${name}: the zone holds two lines (${zone.height} of ${2 * LINE_HEIGHT + LINE_AIR})`,
    );
    for (const line of lines) {
      assert.ok(
        monospaceWidth(line.trim(), LABEL_SIZE) <= room,
        `${name}: the line "${line}" fits the zone (${monospaceWidth(line.trim(), LABEL_SIZE)} of ${room})`,
      );
    }
  }
});

test("the two combs write their whole name in one line", () => {
  // The reader looked at the real pictures and asked for it: the name of
  // the two multifurcaciones has to fit in one line of the crown of the
  // comb, so the comb grew until the name fits whole.
  for (const name of ["Multifurcacion", "MultifurcacionP"]) {
    const shape = shapeOf(name);
    assert.ok(shape, `the module knows ${name}`);
    const zone: FootprintZone = {
      left: shape.label.x,
      top: shape.label.y,
      width: shape.label.width,
      height: shape.label.height,
    };
    const lines = labelLines(name, zone);
    assert.equal(lines.length, 1, `${name}: the name is written in one line`);
    assert.equal(lines[0], name, `${name}: the whole name is written`);
    assert.ok(
      monospaceWidth(name, LABEL_SIZE) <= zone.width - ZONE_AIR,
      `${name}: the name fits the zone (${monospaceWidth(name, LABEL_SIZE)} of ${zone.width - ZONE_AIR})`,
    );
  }
});

test("no zone of text or of figure sits on an interior stroke", () => {
  // The text of a zone is drawn on top of the zone, and a stroke under the
  // text is a text nobody can read: the subtitle of a table over the line
  // of the table, the name of a multifurcacion over the rail of its
  // parameter. Every zone keeps 2 px of air from every interior stroke
  // (contract v5 §2.5).
  //
  // `Unlink` is the one exception the design declares: its capsule is 38 px
  // wide and the whole name needs a zone of 36.4 px, so the 2 px of air
  // from the seam (x=38) leaves a zone of 36 px at most and the name would
  // be broken inside the word. The name stays whole and the zone does not
  // cross the seam.
  const exceptions = new Set(["Unlink"]);
  for (const [name, shape] of Object.entries(BLOCK_SHAPES)) {
    if (exceptions.has(name)) {
      continue;
    }
    const zones: Array<{ kind: string; zone: Zone }> = [{ kind: "text", zone: shape.label }];
    if (shape.figure !== null) {
      zones.push({ kind: "figure", zone: shape.figure });
    }
    for (const { kind, zone } of zones) {
      for (const detail of shape.details) {
        for (const polyline of strokePolylines(detail)) {
          for (let index = 0; index + 1 < polyline.length; index += 1) {
            const distance = segmentToZone(polyline[index] as Point, polyline[index + 1] as Point, zone);
            assert.ok(
              distance >= STROKE_AIR - 1e-9,
              `${name}: the ${kind} zone is ${distance.toFixed(1)} px from the stroke ` +
                `«${detail}» and it needs ${STROKE_AIR} px of air`,
            );
          }
        }
      }
    }
  }
});

test("the classes that can never have a figure declare no zone for it", () => {
  // The reader's criterion: show the figure when the engine publishes it,
  // never invent one, and do not leave a reserved hole that looks like a
  // fault. The classes the engine never walks into have no value to show,
  // so they declare no zone of the figure at all (contract v5 §2.7).
  //
  // The list is the one the module publishes: the design's generator writes
  // it, so the test does not repeat it.
  assert.equal(CLASSES_WITHOUT_FIGURE.length, 9, "the list holds the nine classes");
  assert.equal(new Set(CLASSES_WITHOUT_FIGURE).size, 9, "the list has no repeats");
  for (const name of CLASSES_WITHOUT_FIGURE) {
    const shape = shapeOf(name);
    assert.ok(shape, `the module knows ${name}`);
    assert.equal(shape.figure, null, `${name} declares no zone of the figure`);
  }
  for (const name of SHAPE_CLASSES) {
    if (CLASSES_WITHOUT_FIGURE.includes(name)) {
      continue;
    }
    const shape = shapeOf(name);
    assert.notEqual(shape?.figure, null, `${name} declares a zone of the figure`);
  }
  // The three variables are not on the list: the engine publishes the value
  // of a variable when the run assigns it.
  for (const name of ["Variable", "VariableA", "VariableN"]) {
    assert.ok(
      !CLASSES_WITHOUT_FIGURE.includes(name),
      `${name} keeps its zone of the figure`,
    );
  }
});

test("the three colour levels of a family are the approved ones", () => {
  assert.deepEqual(FAMILY_COLORS, APPROVED_COLORS);
  assert.equal(familyColors("Cola").acento, "#b45309");
  assert.equal(colorsOf("Queue")?.borde, "#dabbb3");
  assert.equal(colorsOf("NoSuchClass"), undefined);
});

test("the terminate is the clean circle of the design", () => {
  const shape = BLOCK_SHAPES.Terminate;
  assert.deepEqual(shape.footprint, { width: 64, height: 64 });
  const path = shape.path.replace(/\s+/g, " ");
  assert.ok(path.startsWith("M32 0 A32 32"), `the circle does not start at the top: ${path}`);
  // The two arcs of the path are the halves of one circle, not of an ellipse.
  const arcs = parsePath(shape.path).filter((command) => command.letter === "A");
  assert.equal(arcs.length, 2);
  for (const arc of arcs) {
    assert.equal(arc.args[0], 32, "the radius is not 32");
    assert.equal(arc.args[1], 32, "the shape is an ellipse, not a circle");
  }
});

test("the unlink is the link reflected", () => {
  const link = BLOCK_SHAPES.Link;
  const unlink = BLOCK_SHAPES.Unlink;
  assert.deepEqual(link.footprint, unlink.footprint);
  const width = link.footprint.width;
  const mirrored = parsePath(unlink.path).map((command) => reflect(command, width));
  const original = parsePath(link.path);
  assert.equal(mirrored.length, original.length, "the two paths are not the same commands");
  for (let index = 0; index < original.length; index += 1) {
    const expected = original[index];
    const actual = mirrored[index];
    assert.equal(actual.letter, expected.letter, `the command ${index} is not the same`);
    assert.equal(actual.args.length, expected.args.length, `the command ${index} has other arguments`);
    for (let argument = 0; argument < expected.args.length; argument += 1) {
      assert.ok(
        Math.abs(actual.args[argument] - expected.args[argument]) <= 1e-6,
        `the argument ${argument} of the command ${index} is not the mirror: ` +
          `${actual.args[argument]} against ${expected.args[argument]}`,
      );
    }
  }
});

test("the module is pure data: it imports nothing", () => {
  const url = new URL("../src/graph/shapes.ts", import.meta.url);
  const source = readFileSync(url, "utf8");
  assert.equal(source.includes("import "), false, "the module must not import anything");
  assert.ok(source.includes("GENERATED FILE"), "the module says that it is generated");
});

test("the engine gives every class the footprint its shape draws", () => {
  const footprints = catalogueFootprints();
  assert.equal(footprints.size, SHAPE_CLASSES.length, "every class has a footprint");
  for (const className of SHAPE_CLASSES) {
    const shape = shapeOf(className);
    assert.ok(shape, `the module knows ${className}`);
    const footprint = footprints.get(className);
    assert.ok(footprint, `the engine declares ${className}`);
    assert.deepEqual(
      footprint,
      { width: shape.footprint.width, height: shape.footprint.height },
      `${className} is born with the size its shape draws`,
    );
  }
});
