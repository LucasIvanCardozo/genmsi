//! The drawing of a block of the canvas.

import assert from "node:assert/strict";
import { test } from "node:test";
import {
  captionBaseline,
  captionText,
  defaultPair,
  drawingOf,
  figureBaseline,
  figureText,
  figureZoneText,
  fitOf,
  labelBaseline,
  labelText,
  portPlacements,
  sideOf,
  type PortOfBlock,
} from "../src/graph/node.ts";
import { SHAPE_CLASSES, shapeOf } from "../src/graph/shapes.ts";

/** The ports of a block, as the model declares them. */
function ports(...names: string[]): PortOfBlock[] {
  return names.map((name, index) => ({ ioid: index + 1, name }));
}

test("the handle of a port sits at the anchor the shape declares", () => {
  const shape = shapeOf("Test");
  assert.ok(shape !== undefined, "the class Test has a shape");
  const placed = portPlacements(shape, ports("Entrada1", "Salida1", "Salida2"), "top");
  assert.deepEqual(
    placed.map((one) => [one.x, one.y]),
    [
      [0.5, 0],
      [1, 0.5],
      [0, 0.5],
    ],
  );
  // The anchor is a fraction of the footprint of the class.
  for (const one of placed) {
    assert.ok(one.x >= 0 && one.x <= 1, `the anchor ${one.x} is inside the shape`);
    assert.ok(one.y >= 0 && one.y <= 1, `the anchor ${one.y} is inside the shape`);
  }
});

test("a port the model does not declare is not drawn", () => {
  const shape = shapeOf("Test");
  assert.ok(shape !== undefined, "the class Test has a shape");
  // The model only declares the entry and one of the two exits.
  const placed = portPlacements(shape, ports("Entrada1", "Salida1"), "top");
  assert.equal(placed.length, 2);
  assert.deepEqual(
    placed.map((one) => one.name),
    ["Entrada1", "Salida1"],
  );
});

test("the outputs of a multifurcacion spread along its bottom edge", () => {
  const shape = shapeOf("Multifurcacion");
  assert.ok(shape !== undefined, "the class Multifurcacion has a shape");
  const placed = portPlacements(
    shape,
    ports("Salida1", "SalidaProb2", "SalidaProb3", "SalidaProb4", "SalidaProb5"),
    "bottom",
  );
  assert.deepEqual(
    placed.map((one) => one.x),
    [0, 0.25, 0.5, 0.75, 1],
  );
  for (const one of placed) {
    assert.equal(one.y, 1, "the exits leave from the bottom edge");
    assert.equal(one.side, "bottom");
  }
});

test("the side of a port is the side of the silhouette it belongs to", () => {
  assert.equal(sideOf({ name: "Entrada1", x: 0.5, y: 0 }), "top");
  assert.equal(sideOf({ name: "Salida1", x: 0.5, y: 1 }), "bottom");
  assert.equal(sideOf({ name: "Salida1", x: 1, y: 0.5 }), "right");
  assert.equal(sideOf({ name: "Salida2", x: 0, y: 0.5 }), "left");
});

test("a class the shapes module does not know still gets a node", () => {
  const unknown = drawingOf("ClaseQueNoExiste");
  assert.equal(unknown.fallback, true);
  assert.equal(unknown.shape, undefined);
  // And its figure is the pair of always: how many are inside it and how
  // many have gone through it.
  assert.equal(figureText(undefined, { now: 2, ever: 15 }), "2/15");
  // A class the shapes module knows is drawn with its silhouette.
  const known = drawingOf("Queue");
  assert.equal(known.fallback, false);
  assert.ok(known.shape !== undefined);
  assert.ok(known.colors !== undefined);
});

test("the figure of a block comes from the live store, and a block without one shows the pair", () => {
  // The live store keeps the figure of every block by its id.
  const figures = new Map<number, { text: string }>([[1, { text: "7" }]]);
  assert.equal(figureText(figures.get(1), { now: 2, ever: 15 }), "7");
  // A block the store has no figure for shows the pair of always.
  assert.equal(figureText(figures.get(2), { now: 2, ever: 15 }), "2/15");
  // A block the run has not reached shows nothing: the space is reserved
  // anyway, so the label does not move when the figure appears.
  assert.equal(figureText(undefined, { now: 0, ever: 0 }), "");
  assert.equal(defaultPair(undefined), "");
});

test("the zones of the label and of the figure stay inside the block", () => {
  for (const className of SHAPE_CLASSES) {
    const shape = shapeOf(className);
    assert.ok(shape !== undefined, `the class ${className} has a shape`);
    const drawing = drawingOf(className);
    assert.equal(drawing.fallback, false, `the class ${className} is drawn`);
    for (const zone of [drawing.label, drawing.figure]) {
      if (zone === null) {
        continue;
      }
      // The zone is in the pixels of the footprint of the class, so it is
      // inside the silhouette the shape draws.
      assert.ok(zone.left >= 0 && zone.left <= shape.footprint.width, `${className}: inside`);
      assert.ok(zone.top >= 0 && zone.top <= shape.footprint.height, `${className}: inside`);
      assert.ok(zone.width > 0, `${className}: the zone has width`);
      assert.ok(zone.height > 0, `${className}: the zone has height`);
      assert.ok(
        zone.left + zone.width <= shape.footprint.width + 0.0001,
        `${className}: the zone does not leave the shape`,
      );
      assert.ok(
        zone.top + zone.height <= shape.footprint.height + 0.0001,
        `${className}: the zone does not leave the shape`,
      );
    }
    // The text and the figure never sit on the same place.
    if (drawing.figure !== null && drawing.label !== null) {
      const label = drawing.label;
      const figure = drawing.figure;
      const apart =
        label.left + label.width <= figure.left + 0.0001 ||
        figure.left + figure.width <= label.left + 0.0001 ||
        label.top + label.height <= figure.top + 0.0001 ||
        figure.top + figure.height <= label.top + 0.0001;
      assert.ok(apart, `${className}: the label and the figure do not overlap`);
    }
  }
});

test("the silhouette keeps its own proportion in the box of the block", () => {
  const shape = shapeOf("Terminate");
  assert.ok(shape !== undefined, "the class Terminate has a shape");
  // The model of an old file keeps the geometry it had: 84×64 for a
  // terminate. The circle of the class is 64×64, so it is drawn as a
  // circle in the middle of the box, and not as an ellipse.
  const fit = fitOf(shape, { width: 84, height: 64 });
  assert.equal(fit.scale, 1, "the circle keeps its size");
  assert.equal(fit.width, 64);
  assert.equal(fit.height, 64);
  assert.equal(fit.offsetX, 10, "the circle sits in the middle of the box");
  assert.equal(fit.offsetY, 0);
});

test("a box bigger than the shape keeps the shape in the middle", () => {
  const shape = shapeOf("Advance");
  assert.ok(shape !== undefined, "the class Advance has a shape");
  const fit = fitOf(shape, { width: 200, height: 100 });
  assert.equal(fit.scale, 1, "the shape is not stretched");
  assert.equal(fit.width, shape.footprint.width);
  assert.equal(fit.height, shape.footprint.height);
  assert.equal(fit.offsetX, (200 - shape.footprint.width) / 2);
  assert.equal(fit.offsetY, (100 - shape.footprint.height) / 2);
});

test("a box smaller than the shape shrinks it with its proportion", () => {
  const shape = shapeOf("Note");
  assert.ok(shape !== undefined, "the class Note has a shape");
  // A note is 80×80 and the box of the block is 116×32: the sheet shrinks
  // to the height of the box and keeps its square shape.
  const fit = fitOf(shape, { width: 116, height: 32 });
  assert.equal(fit.scale, 32 / 80);
  assert.equal(fit.width, 32);
  assert.equal(fit.height, 32);
  assert.equal(fit.offsetX, (116 - 32) / 2);
  assert.equal(fit.offsetY, 0);
});

test("a name that fits its zone is not cut", () => {
  const drawing = drawingOf("Generate");
  assert.ok(drawing.label !== null);
  // The lobe of the generate is 63 px wide and the name fits in its zone.
  assert.equal(labelText("Generate", drawing.label), "Generate");
  // And the advance fits in the zone of its plate.
  const advance = drawingOf("Advance");
  assert.ok(advance.label !== null);
  assert.equal(labelText("Advance", advance.label), "Advance");
  // The queue fits too, and the depart is cut with an ellipsis.
  const queue = drawingOf("Queue");
  assert.ok(queue.label !== null);
  assert.equal(labelText("Queue", queue.label), "Queue");
  const depart = drawingOf("Depart");
  assert.ok(depart.label !== null);
  assert.equal(labelText("Depart", depart.label), "Depa…");
});

test("a name that does not fit its zone is cut with an ellipsis", () => {
  const drawing = drawingOf("Counter");
  assert.ok(drawing.label !== null);
  // The counter does not fit in its zone: the design sheet cuts it the
  // same way.
  const cut = labelText("Counter", drawing.label);
  assert.equal(cut, "Count…");
  assert.ok(cut.length < "Counter".length, "the name is shorter");
});

test("the caption is cut with the same measure as the rótulo", () => {
  const drawing = drawingOf("Generate");
  assert.ok(drawing.label !== null);
  assert.equal(captionText("Clientes", drawing.label), "Clientes");
  assert.equal(captionText("Un nombre larguisimo de bloque", drawing.label), "Un nomb…");
});

test("the live figure is cut to the width of its zone", () => {
  const drawing = drawingOf("Terminate");
  assert.ok(drawing.figure !== null);
  // The zone of the figure holds the pair of always.
  assert.equal(figureZoneText("2/15", drawing.figure), "2/15");
  assert.equal(figureZoneText("", drawing.figure), "");
});

test("the rótulo is on the first line of its zone and the caption on the second", () => {
  const drawing = drawingOf("Generate");
  assert.ok(drawing.label !== null);
  const zone = drawing.label;
  // The rótulo is written at the top of the zone, and the caption below
  // it: the caption does not sit on the edge of the silhouette.
  assert.ok(labelBaseline(zone) < zone.top + zone.height / 2);
  assert.ok(captionBaseline(zone) > labelBaseline(zone));
  assert.ok(captionBaseline(zone) <= zone.top + zone.height);
  // The figure is written at the right border of its zone.
  assert.ok(drawing.figure !== null);
  assert.equal(figureBaseline(drawing.figure), drawing.figure.top + drawing.figure.height - 2);
});

test("the subtitle only fits when the zone of the label has room for it", () => {
  // A block of 32 px of height carries the rótulo alone.
  assert.equal(drawingOf("Queue").subtitle, false);
  assert.equal(drawingOf("Test").subtitle, false);
  // A taller block carries the rótulo and the caption.
  assert.equal(drawingOf("Generate").subtitle, true);
  assert.equal(drawingOf("Select").subtitle, true);
});

test("a container keeps the plain block of always", () => {
  const container = drawingOf("Contenedor", true);
  assert.equal(container.fallback, true);
  assert.equal(container.shape, undefined);
});
