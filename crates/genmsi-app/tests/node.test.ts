//! The drawing of a block of the canvas.

import assert from "node:assert/strict";
import { test } from "node:test";
import {
  CAPTION_SIZE,
  FIGURE_SIZE,
  LABEL_SIZE,
  LINE_AIR,
  LINE_HEIGHT,
  SUBTITLE_HEIGHT,
  ZONE_AIR,
  captionBaseline,
  captionFits,
  captionText,
  defaultPair,
  drawingOf,
  figureBaseline,
  figureText,
  figureZoneText,
  labelBaselines,
  labelHeight,
  labelLines,
  nodeSizeOf,
  portPlacements,
  sideOf,
  type FootprintZone,
  type PortOfBlock,
} from "../src/graph/node.ts";
import { monospaceWidth, textWidth } from "../src/graph/catalogue.ts";
import { SHAPE_CLASSES, shapeOf, type PortAnchor } from "../src/graph/shapes.ts";

/** The ports of a block, as the model declares them. */
function ports(...names: string[]): PortOfBlock[] {
  return names.map((name, index) => ({ ioid: index + 1, name }));
}

/**
 * A zone of the label, for the rules that do not need a real class.
 *
 * The zones of the classes change with the design, so the rules that do
 * not depend on a class are checked against a zone of a known size.
 */
function zone(width: number, height = 24): FootprintZone {
  return { left: 0, top: 0, width, height };
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

test("the ports keep their anchors in the footprint of the class", () => {
  const shape = shapeOf("Test");
  assert.ok(shape !== undefined, "the class Test has a shape");
  const placed = portPlacements(shape, ports("Entrada1", "Salida1", "Salida2"), "top");
  // Every port sits at the very anchor the shape declares, and the anchor
  // is a fraction of the footprint: the block is drawn at the size of its
  // class, so the port falls on the point of the silhouette it belongs to.
  for (const port of placed) {
    const anchor: PortAnchor | undefined = shape.ports.find(
      (one) => one.name === port.name,
    );
    assert.ok(anchor !== undefined, `the shape knows the port ${port.name}`);
    assert.equal(port.x, anchor.x, `${port.name}: the horizontal anchor`);
    assert.equal(port.y, anchor.y, `${port.name}: the vertical anchor`);
  }
});

test("a port the shape does not know keeps the spread of the ports", () => {
  // A class without a shape keeps the places the ports used to have,
  // spread along the side: a link that loses its handle is a link the
  // reader cannot follow.
  const placed = portPlacements(undefined, ports("Entrada1", "Salida1"), "top");
  assert.deepEqual(
    placed.map((one) => one.x),
    [1 / 3, 2 / 3],
  );
  assert.deepEqual(
    placed.map((one) => one.side),
    ["top", "top"],
  );
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
    for (const zoneOfBlock of [drawing.label, drawing.figure]) {
      if (zoneOfBlock === null) {
        continue;
      }
      // The zone is in the pixels of the footprint of the class, so it is
      // inside the silhouette the shape draws.
      assert.ok(zoneOfBlock.left >= 0 && zoneOfBlock.left <= shape.footprint.width, `${className}: inside`);
      assert.ok(zoneOfBlock.top >= 0 && zoneOfBlock.top <= shape.footprint.height, `${className}: inside`);
      assert.ok(zoneOfBlock.width > 0, `${className}: the zone has width`);
      assert.ok(zoneOfBlock.height > 0, `${className}: the zone has height`);
      assert.ok(
        zoneOfBlock.left + zoneOfBlock.width <= shape.footprint.width + 0.0001,
        `${className}: the zone does not leave the shape`,
      );
      assert.ok(
        zoneOfBlock.top + zoneOfBlock.height <= shape.footprint.height + 0.0001,
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

test("the node of a block is the footprint of its class, one to one", () => {
  for (const className of SHAPE_CLASSES) {
    const shape = shapeOf(className);
    assert.ok(shape !== undefined, `the class ${className} has a shape`);
    const size = nodeSizeOf(className);
    assert.ok(size !== null, `the class ${className} has a size`);
    // The shape is the truth: the block is drawn at the size of its class,
    // with its own proportion, and never scaled.
    assert.equal(size.width, shape.footprint.width, `${className}: the width is the footprint`);
    assert.equal(size.height, shape.footprint.height, `${className}: the height is the footprint`);
    // And the drawing of the block is that very size.
    const drawing = drawingOf(className);
    assert.ok(drawing.shape !== undefined);
    assert.equal(drawing.shape.footprint.width, size.width);
    assert.equal(drawing.shape.footprint.height, size.height);
  }
});

test("the size of the block does not depend on the box the model carries", () => {
  // A note of an old model keeps the box it had (116×31 for a sheet of
  // 80×80): the block is drawn at the size of its class, and the model is
  // not rewritten.
  const shape = shapeOf("Note");
  assert.ok(shape !== undefined, "the class Note has a shape");
  const size = nodeSizeOf("Note");
  assert.ok(size !== null);
  assert.equal(size.width, shape.footprint.width);
  assert.equal(size.height, shape.footprint.height);
  // A class that is not square keeps its own proportion: nothing is
  // stretched to the box of the model.
  const wide = nodeSizeOf("Variable");
  assert.ok(wide !== null);
  assert.ok(wide.width > wide.height, "the class keeps its own proportion");
});

test("a container and an unknown class keep the plain rectangle of the model", () => {
  const container = drawingOf("Contenedor", true);
  assert.equal(container.fallback, true);
  assert.equal(container.shape, undefined);
  assert.equal(container.label, null);
  assert.equal(container.figure, null);
  assert.equal(container.subtitle, false);
  // They keep the box of the model: the canvas draws no silhouette for them.
  assert.equal(nodeSizeOf("Contenedor", true), null);
  const unknown = drawingOf("ClaseQueNoExiste");
  assert.equal(unknown.fallback, true);
  assert.equal(unknown.shape, undefined);
  assert.equal(unknown.label, null);
  assert.equal(unknown.figure, null);
  assert.equal(unknown.subtitle, false);
  assert.equal(nodeSizeOf("ClaseQueNoExiste"), null);
});

test("a name that fits its zone goes on one line", () => {
  // The name of the class fits in the zone of the label.
  const lines = labelLines("Generate", zone(80));
  assert.deepEqual(lines, ["Generate"]);
  assert.equal(lines.length, 1, "the name goes on one line");
  // And the measure is the one of the design: the name is monospaced.
  const room = 80 - ZONE_AIR;
  assert.ok(monospaceWidth("Generate", LABEL_SIZE) <= room);
});

test("a name that does not fit its zone goes on two lines and loses no character", () => {
  // The zone holds six characters of the monospaced rótulo: the name of a
  // class does not fit in one of them.
  const narrow = zone(44);
  const lines = labelLines("Multifurcacion", narrow);
  assert.equal(lines.length, 2, "the name goes on two lines");
  assert.equal(lines.join(""), "Multifurcacion", "no character of the name is lost");
  assert.ok(!lines.join("").includes("…"), "the name is never cut with an ellipsis");
  for (const line of lines) {
    assert.ok(
      monospaceWidth(line.trim(), LABEL_SIZE) <= narrow.width - ZONE_AIR,
      `the line "${line}" fits the zone`,
    );
  }
});

test("the name of a class is never cut, and it goes on two lines when it does not fit", () => {
  for (const className of SHAPE_CLASSES) {
    const drawing = drawingOf(className);
    assert.ok(drawing.label !== null, `${className}: the class has a zone of the label`);
    const label = drawing.label;
    const lines = labelLines(className, label);
    // One line or two, never more.
    assert.ok(lines.length === 1 || lines.length === 2, `${className}: one or two lines`);
    // No character of the name is lost, and it is never cut.
    assert.equal(lines.join(""), className, `${className}: the whole name is drawn`);
    assert.ok(!lines.join("").includes("…"), `${className}: the name is not cut`);
    const room = label.width - ZONE_AIR;
    const fits = monospaceWidth(className, LABEL_SIZE) <= room;
    if (fits) {
      assert.deepEqual(lines, [className], `${className}: the name goes on one line`);
    } else {
      assert.equal(lines.length, 2, `${className}: the name goes on two lines`);
      // When the name can be split in two lines that fit, it is.
      const perLine = Math.floor(room / monospaceWidth("x", LABEL_SIZE));
      if (className.length <= perLine * 2) {
        for (const line of lines) {
          assert.ok(
            monospaceWidth(line.trim(), LABEL_SIZE) <= room,
            `${className}: the line "${line}" fits the zone`,
          );
        }
      }
    }
  }
});

test("a name with a space is broken at the space", () => {
  const narrow = zone(60);
  const lines = labelLines("Function Dist", narrow);
  assert.equal(lines.length, 2, "the name goes on two lines");
  assert.equal(lines.join(""), "Function Dist", "no character of the name is lost");
  assert.ok(lines[0].endsWith(" "), "the break is at the space of the name");
  assert.equal(lines[1].trim(), "Dist");
  for (const line of lines) {
    assert.ok(
      monospaceWidth(line.trim(), LABEL_SIZE) <= narrow.width - ZONE_AIR,
      `the line "${line}" fits the zone`,
    );
  }
});

test("a name without a space is broken in the middle", () => {
  const narrow = zone(44);
  const lines = labelLines("Multifurcacion", narrow);
  assert.equal(lines.length, 2, "the name goes on two lines");
  // The word cannot be kept whole: it is broken at the point that
  // balances the two lines best.
  const balance = Math.abs(lines[0].length - lines[1].length);
  assert.ok(balance <= 1, `the two lines are balanced (${balance})`);
  assert.equal(lines.join(""), "Multifurcacion", "no character of the name is lost");
});

test("the lines of the rótulo are one line and an air apart", () => {
  const label = zone(44);
  const baselines = labelBaselines(label, 2);
  assert.equal(baselines.length, 2);
  assert.equal(baselines[1] - baselines[0], LINE_HEIGHT + LINE_AIR);
  // And the two lines take the height the design declares: a line of
  // text and the air of the design between them.
  assert.equal(labelHeight(1), LINE_HEIGHT);
  assert.equal(labelHeight(2), 2 * LINE_HEIGHT + LINE_AIR);
});

test("the caption is cut to the width of its zone", () => {
  const drawing = drawingOf("Generate");
  assert.ok(drawing.label !== null);
  const label = drawing.label;
  // A caption that fits the zone is never cut.
  assert.equal(captionText("Clientes", zone(80)), "Clientes");
  // And a caption that does not fit is cut with an ellipsis.
  const cut = captionText("Un nombre larguisimo de bloque", label);
  assert.ok(cut.length < "Un nombre larguisimo de bloque".length, "the caption is shorter");
  assert.ok(cut.endsWith("…"), "the cut is marked with an ellipsis");
  assert.ok(
    textWidth(cut, CAPTION_SIZE) <= label.width - ZONE_AIR,
    "the caption fits the width of its zone",
  );
});

test("the live figure is cut to the width of its zone", () => {
  const drawing = drawingOf("Terminate");
  assert.ok(drawing.figure !== null);
  // The zone of the figure holds the pair of always.
  const text = figureZoneText("2/15", drawing.figure);
  assert.ok(
    monospaceWidth(text, FIGURE_SIZE) <= drawing.figure.width,
    "the figure fits the width of its zone",
  );
  assert.equal(figureZoneText("", drawing.figure), "");
});

test("the rótulo is on the first line of its zone and the caption on the second", () => {
  const drawing = drawingOf("Generate");
  assert.ok(drawing.label !== null);
  const label = drawing.label;
  // The rótulo is written at the top of the zone, and the caption below
  // it: the caption does not sit on the edge of the silhouette.
  const baselines = labelBaselines(label, 1);
  assert.equal(baselines.length, 1);
  assert.ok(baselines[0] < label.top + label.height / 2);
  assert.ok(captionBaseline(label) > baselines[0]);
  assert.ok(captionBaseline(label) <= label.top + label.height);
  // The figure is written at the right border of its zone.
  assert.ok(drawing.figure !== null);
  assert.equal(figureBaseline(drawing.figure), drawing.figure.top + drawing.figure.height - 2);
});

test("a name that takes two lines leaves no place for the caption", () => {
  const drawing = drawingOf("Generate");
  assert.ok(drawing.label !== null);
  // With one line of the rótulo, the caption has its place under it.
  assert.equal(captionFits(drawing.label, 1), true);
  // With two lines, the name of the class takes that line too: the caption
  // has no place, and the block carries the rótulo alone.
  assert.equal(captionFits(drawing.label, 2), false);
});

test("the subtitle only fits when the zone of the label has room for it", () => {
  for (const className of SHAPE_CLASSES) {
    const drawing = drawingOf(className);
    assert.ok(drawing.label !== null, `${className}: the class has a zone of the label`);
    // The caption needs a line of its own under the rótulo.
    assert.equal(
      drawing.subtitle,
      drawing.label.height >= SUBTITLE_HEIGHT,
      `${className}: the caption fits the zone of the label`,
    );
  }
});
