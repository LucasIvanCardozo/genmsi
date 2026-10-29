//! The shapes of the blocks that are not classes of the catalogue.

import assert from "node:assert/strict";
import { test } from "node:test";
import {
  CONTAINER_FAMILY,
  DOOR_BOX,
  DOOR_FOOTPRINT,
  PLAIN_FAMILY,
  PORT_IN_FAMILY,
  PORT_OUT_FAMILY,
  STROKE_MARGIN,
  containerShape,
  plainShape,
  portInShape,
  portOutShape,
} from "../src/graph/frames.ts";
import { FAMILY_COLORS } from "../src/graph/shapes.ts";

/** The ports of a container, with the anchor each one sits on. */
function containerPorts(inputs: string[], outputs: string[]) {
  return {
    inputs: inputs.map((name) => ({ name, x: 0.5, y: 0 })),
    outputs: outputs.map((name) => ({ name, x: 0.5, y: 1 })),
  };
}

test("the container is a plate with the box the model gives it", () => {
  const shape = containerShape(
    { width: 149, height: 35 },
    containerPorts(["Entrada1"], ["Salida1"]),
  );
  // Its colour is the one of the family the canvas drew it by hand with.
  assert.equal(shape.family, CONTAINER_FAMILY);
  assert.equal(shape.family, "Decisión");
  assert.equal(FAMILY_COLORS[shape.family].acento, "#4338ca");
  // The plate is drawn at the size it is given, one to one.
  assert.deepEqual(shape.footprint, { width: 149, height: 35 });
  assert.ok(shape.path.startsWith("M "), "the plate is a path of its own");
  assert.deepEqual(shape.details, [], "the plate has no strokes inside");
});

test("the container declares the anchor of each of its ports", () => {
  const shape = containerShape(
    { width: 149, height: 35 },
    containerPorts(["Entrada1", "Entrada2"], ["Salida1"]),
  );
  assert.deepEqual(shape.ports, [
    { name: "Entrada1", x: 0.5, y: 0 },
    { name: "Entrada2", x: 0.5, y: 0 },
    { name: "Salida1", x: 0.5, y: 1 },
  ]);
  // The inputs enter by the top edge and the outputs leave by the bottom
  // one: the flow crosses the box of the container.
  for (const port of shape.ports.filter((one) => one.y === 0)) {
    assert.equal(port.y, 0, "the inputs enter by the top edge");
  }
  for (const port of shape.ports.filter((one) => one.y === 1)) {
    assert.equal(port.y, 1, "the outputs leave by the bottom edge");
  }
});

test("the zones of the container stay inside the plate", () => {
  const shape = containerShape(
    { width: 149, height: 35 },
    containerPorts(["Entrada1"], ["Salida1"]),
  );
  for (const zone of [shape.label, shape.figure]) {
    if (zone === null) {
      continue;
    }
    assert.ok(zone.x >= 0 && zone.x <= shape.footprint.width, "the zone is inside");
    assert.ok(zone.y >= 0 && zone.y <= shape.footprint.height, "the zone is inside");
    assert.ok(
      zone.x + zone.width <= shape.footprint.width + 0.0001,
      "the zone does not leave the plate",
    );
    assert.ok(
      zone.y + zone.height <= shape.footprint.height + 0.0001,
      "the zone does not leave the plate",
    );
  }
  // The name of the container and the figure never sit on the same place.
  assert.ok(shape.figure !== null, "the plate reserves a zone of the figure");
  const label = shape.label;
  const figure = shape.figure;
  assert.ok(figure === null || label.x + label.width <= figure.x + 0.0001);
});

test("the entry door is a source and the exit door is a destination", () => {
  const entry = portInShape("Entrada1");
  const exit = portOutShape("Salida1");
  assert.equal(entry.family, PORT_IN_FAMILY);
  assert.equal(entry.family, "Fuente");
  assert.equal(exit.family, PORT_OUT_FAMILY);
  assert.equal(exit.family, "Destino");
  // The colours are the ones the canvas drew the doors by hand with.
  assert.equal(FAMILY_COLORS[entry.family].acento, "#047857");
  assert.equal(FAMILY_COLORS[exit.family].acento, "#be123c");
  // Both doors are the same plate: the point on the right says that the
  // flow goes on.
  assert.deepEqual(entry.footprint, DOOR_FOOTPRINT);
  assert.deepEqual(exit.footprint, DOOR_FOOTPRINT);
  assert.equal(entry.path, exit.path);
});

test("the door declares the anchor of the port it opens or closes", () => {
  const entry = portInShape("Entrada1");
  const exit = portOutShape("Salida1");
  // The flow comes in through the bottom of the entry door and leaves
  // through the top of the exit door.
  assert.deepEqual(entry.ports, [{ name: "Entrada1", x: 0.5, y: 1 }]);
  assert.deepEqual(exit.ports, [{ name: "Salida1", x: 0.5, y: 0 }]);
});

test("the name of the port fits inside the door", () => {
  const door = portInShape("Entrada1");
  // The zone of the name is inside the plate, and the point of the door
  // does not touch it.
  assert.ok(door.label.x + door.label.width <= door.footprint.width - 24);
  assert.ok(door.label.width > 0);
  // And the figure has a place of its own, beside the name.
  assert.ok(door.figure !== null, "the door reserves a zone of the figure");
  const figure = door.figure;
  assert.ok(figure !== null && door.label.x + door.label.width <= figure.x + 0.0001);
});

test("the box of a door is its footprint and the room of the outline", () => {
  // The box of the node and the viewBox of its silhouette agree: the
  // outline is drawn centred on the line of the shape, so the box leaves
  // the room its outer half needs.
  assert.equal(DOOR_BOX.width, DOOR_FOOTPRINT.width + 2 * STROKE_MARGIN);
  assert.equal(DOOR_BOX.height, DOOR_FOOTPRINT.height + 2 * STROKE_MARGIN);
});

test("the plain plate keeps the size it is given", () => {
  const plate = plainShape({ width: 96, height: 36 });
  assert.equal(plate.family, PLAIN_FAMILY);
  assert.deepEqual(plate.footprint, { width: 96, height: 36 });
  // It has no ports: a class nobody knows has no ports the catalogue
  // declares.
  assert.deepEqual(plate.ports, []);
  // And its name goes on a zone inside the plate.
  assert.ok(plate.label.x + plate.label.width <= plate.footprint.width);
});

test("a plate that is too small for its corners still has a path", () => {
  // A corner wider than the plate is not a corner: the radius is cut to
  // half of the shorter side, and the path is still a path.
  const plate = plainShape({ width: 4, height: 4 });
  assert.ok(plate.path.startsWith("M "));
  assert.ok(plate.path.endsWith("Z"));
  assert.ok(plate.label.width > 0);
  // And its name still has a zone inside the plate.
  assert.ok(plate.label.x + plate.label.width <= plate.footprint.width);
  assert.ok(plate.label.y + plate.label.height <= plate.footprint.height);
});
