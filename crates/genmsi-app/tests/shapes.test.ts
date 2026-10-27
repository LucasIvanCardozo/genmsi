//! The shape of every block class.

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import {
  BLOCK_SHAPES,
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
