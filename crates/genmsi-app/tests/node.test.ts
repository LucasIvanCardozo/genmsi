//! The drawing of a block of the canvas.

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import {
  CAPTION_MIN_SIZE,
  CAPTION_SIZE,
  FIGURE_SIZE,
  LABEL_SIZE,
  LINE_AIR,
  LINE_HEIGHT,
  NO_FIGURE,
  SELECTION_RING,
  SUBTITLE_HEIGHT,
  ZONE_AIR,
  captionBaseline,
  captionFits,
  captionLayout,
  captionText,
  defaultPair,
  drawingOf,
  figureBaseline,
  figureCenterX,
  figureLayout,
  figureText,
  figureZoneText,
  labelBaselines,
  labelHeight,
  labelLines,
  nodeSizeOf,
  STROKE_MARGIN,
  portPlacements,
  sideOf,
  textsOf,
  wrapperClass,
  type BlockDrawing,
  type FootprintZone,
  type NodeSize,
  type PortOfBlock,
} from "../src/graph/node.ts";
import { monospaceWidth, textWidth, truncate } from "../src/graph/catalogue.ts";
import {
  FAMILY_COLORS,
  SHAPE_CLASSES,
  shapeOf,
  type BlockShape,
  type PortAnchor,
} from "../src/graph/shapes.ts";

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

/**
 * The drawing of a class of the catalogue, as the canvas asks for it.
 *
 * The 66 classes decide their own shape: the canvas asks for the drawing
 * of a class by its name, and the class is the truth of its silhouette.
 */
function classDrawing(className: string): BlockDrawing {
  return drawingOf({ className });
}

/**
 * The size of the node of a class of the catalogue.
 *
 * The node of a class is the footprint of the class and the room the
 * outline needs, and never the box the model happens to carry.
 */
function classSize(className: string): NodeSize {
  return nodeSizeOf({ className });
}

/** The shape of a class of the catalogue, which always has one. */
function shapeOfClass(className: string): BlockShape {
  const shape = shapeOf(className);
  assert.ok(shape !== undefined, `the class ${className} has a shape`);
  return shape;
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
  // A shape that declares no port keeps the places the ports used to have,
  // spread along the side: a link that loses its handle is a link the
  // reader cannot follow.
  const placed = portPlacements(shapeOfClass("Note"), ports("Entrada1", "Salida1"), "top");
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
  const unknown = classDrawing("ClaseQueNoExiste");
  assert.equal(unknown.fallback, true);
  // It is drawn as the plain plate, and it is drawn the same way every
  // other block is drawn: one silhouette, one wrapper, one ring.
  assert.equal(unknown.shape.family, "Operación");
  assert.ok(unknown.shape.footprint.width > 0);
  // And its figure is the pair of always: how many are inside it and how
  // many have gone through it.
  assert.equal(figureText(undefined, { now: 2, ever: 15 }), "2/15");
  // A class the shapes module knows is drawn with its silhouette.
  const known = classDrawing("Queue");
  assert.equal(known.fallback, false);
  assert.ok(known.shape.footprint.width > 0);
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
    const drawing = classDrawing(className);
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
    const size = classSize(className);
    assert.ok(size !== null, `the class ${className} has a size`);
    // The shape is the truth: the block is drawn at the size of its class,
    // with its own proportion, and never scaled.
    // The box of the block is the footprint of the class plus the room the
    // border needs: the outline is drawn centred on the line of the shape,
    // so without that margin its outer half is cut.
    assert.equal(
      size.width,
      shape.footprint.width + 2 * STROKE_MARGIN,
      `${className}: the width is the footprint and the room of the border`,
    );
    assert.equal(
      size.height,
      shape.footprint.height + 2 * STROKE_MARGIN,
      `${className}: the height is the footprint and the room of the border`,
    );
    // And the drawing of the block is that very silhouette, drawn inside
    // the box with the room of the border around it.
    const drawing = classDrawing(className);
    assert.ok(drawing.shape !== undefined);
    assert.equal(
      drawing.shape.footprint.width + 2 * STROKE_MARGIN,
      size.width,
    );
    assert.equal(
      drawing.shape.footprint.height + 2 * STROKE_MARGIN,
      size.height,
    );
  }
});

test("the size of the block does not depend on the box the model carries", () => {
  // A note of an old model keeps the box it had (116×31 for a sheet of
  // 80×80): the block is drawn at the size of its class, and the model is
  // not rewritten.
  const shape = shapeOf("Note");
  assert.ok(shape !== undefined, "the class Note has a shape");
  const size = classSize("Note");
  assert.ok(size !== null);
  assert.equal(size.width, shape.footprint.width + 2 * STROKE_MARGIN);
  assert.equal(size.height, shape.footprint.height + 2 * STROKE_MARGIN);
  // A class that is not square keeps its own proportion: nothing is
  // stretched to the box of the model.
  const wide = classSize("Variable");
  assert.ok(wide !== null);
  assert.ok(wide.width > wide.height, "the class keeps its own proportion");
});

test("the container is a block, and it is drawn at the size the model gives it", () => {
  // The container is not a frame around the blocks it holds: it has a box
  // of its own, the one the model carries (L2, L3).
  const drawing = drawingOf({
    className: "",
    kind: "C",
    title: "Contenedor",
    subtitle: "Modulo 4",
    size: { width: 153, height: 39 },
    inputs: ports("Entrada1"),
    outputs: ports("Salida1"),
  });
  assert.equal(drawing.fallback, false);
  // Its colour is the one of the family it was drawn by hand with.
  assert.equal(drawing.shape.family, "Decisión");
  assert.equal(drawing.colors, FAMILY_COLORS["Decisión"]);
  // The box of the node is the box of the model, and the silhouette is
  // drawn one to one inside it: nothing is scaled and nothing is cropped.
  assert.equal(drawing.size.width, 153);
  assert.equal(drawing.size.height, 39);
  assert.equal(drawing.shape.footprint.width, 153 - 2 * STROKE_MARGIN);
  assert.equal(drawing.shape.footprint.height, 39 - 2 * STROKE_MARGIN);
  // And the ports of the container sit on the anchors its shape declares.
  const inputs = portPlacements(drawing.shape, ports("Entrada1"), "top");
  const outputs = portPlacements(drawing.shape, ports("Salida1"), "bottom");
  assert.deepEqual(inputs.map((one) => [one.x, one.y]), [[0.5, 0]]);
  assert.deepEqual(outputs.map((one) => [one.x, one.y]), [[0.5, 1]]);
  assert.equal(inputs[0].side, "top");
  assert.equal(outputs[0].side, "bottom");
});

test("the doors have a shape and a size of their own", () => {
  // The entry door and the exit door are blocks of the canvas, with the
  // silhouette of the plate the flow crosses.
  const entry = drawingOf({
    className: "",
    kind: "PortIn",
    title: "Entrada1",
    outputs: ports("Entrada1"),
  });
  const exit = drawingOf({
    className: "",
    kind: "PortOut",
    title: "Salida1",
    inputs: ports("Salida1"),
  });
  assert.equal(entry.fallback, false);
  assert.equal(exit.fallback, false);
  assert.equal(entry.shape.family, "Fuente");
  assert.equal(exit.shape.family, "Destino");
  // The door has no size in the model: it is drawn at the size of its own
  // silhouette.
  assert.equal(entry.size.width, entry.shape.footprint.width + 2 * STROKE_MARGIN);
  assert.equal(entry.size.height, entry.shape.footprint.height + 2 * STROKE_MARGIN);
  // The flow comes in through the bottom of the entry door and leaves
  // through the top of the exit door.
  const entryPorts = portPlacements(entry.shape, ports("Entrada1"), "bottom");
  const exitPorts = portPlacements(exit.shape, ports("Salida1"), "top");
  assert.deepEqual(entryPorts.map((one) => [one.x, one.y]), [[0.5, 1]]);
  assert.deepEqual(exitPorts.map((one) => [one.x, one.y]), [[0.5, 0]]);
});

test("a class the catalogue does not know is drawn at the box of the model", () => {
  const unknown = drawingOf({
    className: "ClaseQueNoExiste",
    title: "ClaseQueNoExiste",
    size: { width: 120, height: 60 },
  });
  assert.equal(unknown.fallback, true);
  // The plate is drawn inside the box of the model, with the room the
  // outline needs.
  assert.equal(unknown.size.width, 120);
  assert.equal(unknown.size.height, 60);
  assert.equal(unknown.shape.footprint.width, 120 - 2 * STROKE_MARGIN);
  assert.equal(unknown.shape.footprint.height, 60 - 2 * STROKE_MARGIN);
  // And its name goes on a zone inside the plate.
  assert.ok(unknown.label.width > 0);
  assert.ok(
    unknown.label.left + unknown.label.width <= unknown.shape.footprint.width,
    "the zone of the name is inside the plate",
  );
});

test("the container shows its caption as its name, and the doors their own", () => {
  // The container shows the caption of the model as its name, and the word
  // that says what it is under it.
  const container = textsOf("C", "Contenedor", "Modulo 4");
  assert.equal(container.label, "Modulo 4");
  assert.equal(container.caption, "Contenedor");
  // A container with no caption shows the word that names it.
  const bare = textsOf("C", "Contenedor", "");
  assert.equal(bare.label, "Contenedor");
  // The doors show the name of the port and the side they are on.
  assert.deepEqual(textsOf("PortIn", "Entrada1", ""), {
    label: "Entrada1",
    caption: "Entrada",
  });
  assert.deepEqual(textsOf("PortOut", "Salida1", ""), {
    label: "Salida1",
    caption: "Salida",
  });
  // And a block of the catalogue shows its class and the caption of the
  // model, as it always did.
  assert.deepEqual(textsOf("A", "Queue", "Cola de espera"), {
    label: "Queue",
    caption: "Cola de espera",
  });
});

test("the ring of the selection is on the wrapper of the block", () => {
  // The ring goes on the wrapper, the box that holds the silhouette, and
  // not on the silhouette itself: a ring on the inner box would only make
  // the border of the shape look thicker (S3).
  assert.ok(wrapperClass(true).includes(SELECTION_RING));
  assert.equal(wrapperClass(false).includes(SELECTION_RING), false);
  // The wrapper is the same for every kind of block: the 66 classes, the
  // container and the doors are selected the same way.
  assert.equal(wrapperClass(true, false), wrapperClass(true, false));
  assert.ok(wrapperClass(true, true).includes("opacity-70"));
  // And the canvas writes the ring in one place only: the wrapper of the
  // node, which every block of the canvas shares.
  const source = readFileSync(
    new URL("../src/components/GraphView.tsx", import.meta.url),
    "utf8",
  );
  assert.equal(
    source.includes("ring-2 ring-indigo-500"),
    false,
    "the ring is not written by hand in the canvas",
  );
  assert.ok(source.includes("wrapperClass(selected, note)"));
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
    const drawing = classDrawing(className);
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
  const drawing = classDrawing("Generate");
  assert.ok(drawing.label !== null);
  const label = drawing.label;
  // A caption that fits the zone is never cut.
  assert.equal(captionText("Clientes", zone(80)), "Clientes");
  // And a caption that does not fit is cut with an ellipsis.
  const layout = captionLayout("Un nombre larguisimo de bloque", label);
  assert.ok(layout.text.length < "Un nombre larguisimo de bloque".length, "the caption is shorter");
  assert.ok(layout.text.endsWith("…"), "the cut is marked with an ellipsis");
  assert.ok(
    textWidth(layout.text, layout.size) <= label.width - ZONE_AIR,
    "the caption fits the width of its zone",
  );
});

test("a caption that fits its zone is whole, at the size of the design", () => {
  const drawing = classDrawing("Generate");
  assert.ok(drawing.label !== null);
  const label = drawing.label;
  const caption = "Clientes";
  // The caption of the example fits the zone of the label of the class.
  assert.ok(
    textWidth(caption, CAPTION_SIZE) <= label.width - ZONE_AIR,
    "the caption fits the zone at the size of the design",
  );
  const layout = captionLayout(caption, label);
  assert.equal(layout.text, caption, "the caption is whole");
  assert.equal(layout.size, CAPTION_SIZE, "the caption keeps the size of the design");
  assert.ok(!layout.text.includes("…"), "the caption is not cut");
});

test("a caption that does not fit is drawn smaller, and keeps more of its text than before", () => {
  const drawing = classDrawing("Generate");
  assert.ok(drawing.label !== null);
  const label = drawing.label;
  const caption = "ProbClientes";
  // The caption does not fit at the size of the design, and it does fit
  // at the smallest size the design allows.
  assert.ok(
    textWidth(caption, CAPTION_SIZE) > label.width - ZONE_AIR,
    "the caption does not fit at the size of the design",
  );
  assert.ok(
    textWidth(caption, CAPTION_MIN_SIZE) <= label.width - ZONE_AIR,
    "the caption fits at the smallest size",
  );
  const layout = captionLayout(caption, label);
  assert.ok(layout.size < CAPTION_SIZE, "the caption is drawn smaller");
  assert.ok(layout.size >= CAPTION_MIN_SIZE, "the caption does not go below the minimum size");
  assert.equal(layout.text, caption, "the whole caption is kept");
  // And it keeps more of its text than the rule that cut it at the size of
  // the design.
  const before = truncate(caption, label.width - ZONE_AIR, CAPTION_SIZE);
  assert.ok(before.endsWith("…"), "before, the caption was cut with an ellipsis");
  assert.ok(before.length < layout.text.length, "more of the caption is kept");
});

test("a caption that does not fit even at the smallest size is cut with an ellipsis", () => {
  const drawing = classDrawing("Alter");
  assert.ok(drawing.label !== null);
  const label = drawing.label;
  const caption = "ProbClientes";
  // The caption does not fit at the size of the design, nor at the
  // smallest one: then, and only then, it is cut.
  assert.ok(
    textWidth(caption, CAPTION_MIN_SIZE) > label.width - ZONE_AIR,
    "the caption does not fit even at the smallest size",
  );
  const layout = captionLayout(caption, label);
  assert.equal(layout.size, CAPTION_MIN_SIZE, "the caption is cut at the smallest size");
  assert.ok(layout.text.endsWith("…"), "the cut is marked with an ellipsis");
  assert.ok(
    textWidth(layout.text, layout.size) <= label.width - ZONE_AIR,
    "the cut caption fits the width of its zone",
  );
  // Even cut, it keeps more of the text than the rule that cut it at the
  // size of the design.
  const before = truncate(caption, label.width - ZONE_AIR, CAPTION_SIZE);
  assert.ok(before.endsWith("…"), "before, the caption was cut with an ellipsis");
  assert.ok(before.length < layout.text.length, "more of the caption is kept");
});

test("the figure is centred in its zone", () => {
  const drawing = classDrawing("Terminate");
  assert.ok(drawing.figure !== null);
  const zoneOfFigure = drawing.figure;
  const layout = figureLayout(zoneOfFigure, "3");
  // The number sits in the middle of its zone: the round blocks read
  // better with it.
  assert.equal(layout.x, zoneOfFigure.left + zoneOfFigure.width / 2, "the figure is centred");
  assert.equal(layout.anchor, "middle", "the figure is anchored in its middle");
  assert.equal(layout.text, "3", "the figure is the number the run published");
  assert.equal(layout.empty, false, "the zone holds a figure");
  assert.equal(layout.y, figureBaseline(zoneOfFigure), "the figure keeps its line");
  assert.equal(layout.size, FIGURE_SIZE, "the figure keeps the size of the design");
  // And every class with a zone of the figure centres its figure there.
  for (const className of SHAPE_CLASSES) {
    const drawn = classDrawing(className);
    if (drawn.figure === null) {
      continue;
    }
    assert.equal(
      figureCenterX(drawn.figure),
      drawn.figure.left + drawn.figure.width / 2,
      `${className}: the figure is centred in its zone`,
    );
  }
});

test("a reserved zone with no figure shows the dash, and not nothing", () => {
  const drawing = classDrawing("Variable");
  assert.ok(drawing.figure !== null, "the class reserves a zone of the figure");
  const zoneOfFigure = drawing.figure;
  // The run publishes no figure for the block: the zone shows the dash,
  // which says there is no value here.
  const empty = figureLayout(zoneOfFigure, "");
  assert.equal(empty.text, NO_FIGURE, "the empty zone shows the dash");
  assert.equal(empty.empty, true, "the zone holds no figure");
  assert.equal(empty.x, zoneOfFigure.left + zoneOfFigure.width / 2, "the dash is in the middle of the zone");
  // And a figure the run does publish is written as it is.
  const shown = figureLayout(zoneOfFigure, "7");
  assert.equal(shown.text, "7", "the figure is the number the run published");
  assert.equal(shown.empty, false, "the zone holds a figure");
  // A class that reserves no zone of the figure has no dash to draw.
  const without = classDrawing("Note");
  assert.equal(without.figure, null, "the class Note reserves no zone of the figure");
});

test("the live figure is cut to the width of its zone", () => {
  const drawing = classDrawing("Terminate");
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
  const drawing = classDrawing("Generate");
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
  const drawing = classDrawing("Generate");
  assert.ok(drawing.label !== null);
  // With one line of the rótulo, the caption has its place under it.
  assert.equal(captionFits(drawing.label, 1), true);
  // With two lines, the name of the class takes that line too: the caption
  // has no place, and the block carries the rótulo alone.
  assert.equal(captionFits(drawing.label, 2), false);
});

test("the subtitle only fits when the zone of the label has room for it", () => {
  for (const className of SHAPE_CLASSES) {
    const drawing = classDrawing(className);
    assert.ok(drawing.label !== null, `${className}: the class has a zone of the label`);
    // The caption needs a line of its own under the rótulo.
    assert.equal(
      drawing.subtitle,
      drawing.label.height >= SUBTITLE_HEIGHT,
      `${className}: the caption fits the zone of the label`,
    );
  }
});
