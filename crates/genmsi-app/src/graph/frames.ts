/**
 * The shapes of the canvas that are not classes of the catalogue.
 *
 * The catalogue of classes owns the silhouette of the 66 classes of the
 * model (`src/graph/shapes.ts` is generated from the approved design and
 * is never touched by hand). The canvas draws three kinds of block that the
 * catalogue does not know, and their shapes live here:
 *
 * - the **container**: a block like any other (L2, L3). It has a box of its
 *   own — the one the model gives it — its caption as its rótulo, and the
 *   word "Contenedor" under it. It is not a frame around the blocks it
 *   holds: the reference models draw it as a block with a box of its own,
 *   and none of its children falls inside that box;
 * - the **doors** of a container, the entry and the exit, which have a
 *   silhouette of their own with the name of the port inside;
 * - the **plain plate**, the shape of a block whose class the catalogue does
 *   not know: the canvas never breaks because of a class it has never heard
 *   of.
 *
 * The colours are the ones of a family of the catalogue: the container is
 * a decision, the entry door is a source and the exit door is a
 * destination — the very colours the canvas drew by hand before this
 * module existed.
 *
 * The module is pure: it draws nothing and reads nothing, so the tests can
 * ask it about a shape without a browser, and the node of the canvas only
 * draws what it says.
 */
import type { BlockShape, Family, PortAnchor, Zone } from "./shapes.ts";

/** The size of a shape, in the pixels of its own footprint. */
export interface FrameSize {
  /** The width of the footprint. */
  width: number;
  /** The height of the footprint. */
  height: number;
}

/**
 * The ports of a structural block, with the anchor of each one.
 *
 * The anchor is a fraction of the footprint of the shape, the same way the
 * catalogue declares the anchors of the classes: the port falls on the very
 * point of the silhouette it belongs to.
 */
export interface FramePorts {
  /** The input ports, at the anchor they sit on. */
  inputs: readonly PortAnchor[];
  /** The output ports, at the anchor they sit on. */
  outputs: readonly PortAnchor[];
}

/**
 * The room the outline of a silhouette needs inside the box of the block.
 *
 * The outline is drawn centred on the line of the shape, so the half that
 * falls outside is cut by the box of the drawing. Two pixels on each side
 * leave the whole border visible.
 */
export const STROKE_MARGIN = 2;

/** The radius of the corners of a plate, in pixels. */
export const PLATE_RADIUS = 8;

/** The air the text of a plate leaves between it and its border, in pixels. */
export const PLATE_AIR = 8;

/** The width of the zone of the figure of a plate, in pixels. */
export const FIGURE_WIDTH = 30;

/**
 * The family of the container.
 *
 * The container was drawn by hand as a decision, and it keeps that colour:
 * the family is what the reader recognises, and the canvas does not change
 * the colours the reader already knows.
 */
export const CONTAINER_FAMILY: Family = "Decisión";

/**
 * The family of the entry door.
 *
 * The entry door is where the flow is born inside the scope of the
 * container: it is a source, and it keeps the colour of one.
 */
export const PORT_IN_FAMILY: Family = "Fuente";

/**
 * The family of the exit door.
 *
 * The exit door is where the flow ends inside the scope of the container:
 * it is a destination, and it keeps the colour of one.
 */
export const PORT_OUT_FAMILY: Family = "Destino";

/**
 * The family of the plain plate.
 *
 * The plate is the shape of a block the catalogue does not know, and the
 * neutral grey of the operations is the only honest colour for a block
 * whose family nobody can say.
 */
export const PLAIN_FAMILY: Family = "Operación";

/**
 * The footprint of a door of a container, in pixels.
 *
 * The door is the plate the flow crosses: it carries the name of the port
 * and the count of what crossed it, and nothing else.
 */
export const DOOR_FOOTPRINT: FrameSize = { width: 136, height: 40 };

/**
 * The box a door is drawn in.
 *
 * It is the footprint of the door and the room the outline needs: the box
 * of the node and the viewBox of its silhouette agree, one to one, with
 * nothing scaled and nothing cropped.
 */
export const DOOR_BOX: FrameSize = {
  width: DOOR_FOOTPRINT.width + 2 * STROKE_MARGIN,
  height: DOOR_FOOTPRINT.height + 2 * STROKE_MARGIN,
};

/**
 * The shape of the container, at the size the model gives it.
 *
 * The container is a block: its box is the one the model carries, and the
 * silhouette is drawn inside it with the room the outline needs. Its ports
 * sit on the anchors the caller declares — the inputs on the top edge and
 * the outputs on the bottom edge, spread along it — so the flow enters the
 * container and leaves it by the very points of its own silhouette.
 *
 * @param size the footprint of the silhouette, in pixels
 * @param ports the ports of the container, with their anchors
 */
export function containerShape(size: FrameSize, ports: FramePorts): BlockShape {
  return plateShape(
    CONTAINER_FAMILY,
    size,
    [...ports.inputs, ...ports.outputs],
    "the container is a block and not a frame around the blocks it holds: it has a box of its own, the one the model gives it, and the children of the container live in the scope of the container, not inside this box. The name of the container is its caption and the word «Contenedor» goes under it.",
    // The container shows its figures —the entities inside it and the ones
    // that went through it—, so the zone of the figure is always reserved,
    // even when the plate is too small for it on its own: the figures of a
    // container are the whole point of showing them.
    true,
  );
}

/**
 * The shape of the entry door: the door the flow comes in through.
 *
 * The door carries the name of its port, and the port it opens sits on the
 * bottom edge: the flow comes from above, from outside the scope, and
 * leaves the door downwards.
 *
 * @param name the name of the port the door opens
 */
export function portInShape(name: string): BlockShape {
  return doorShape(
    PORT_IN_FAMILY,
    { name, x: 0.5, y: 1 },
    "the door the flow comes in through: it carries the name of the port and the count of what crossed it. The port it opens sits on the bottom edge, because the flow arrives from above, from outside the scope of the container.",
  );
}

/**
 * The shape of the exit door: the door the flow leaves through.
 *
 * The door carries the name of its port, and the port it closes sits on
 * the top edge: the flow comes from below, from inside the scope, and
 * leaves the door upwards.
 *
 * @param name the name of the port the door closes
 */
export function portOutShape(name: string): BlockShape {
  return doorShape(
    PORT_OUT_FAMILY,
    { name, x: 0.5, y: 0 },
    "the door the flow leaves through: it carries the name of the port and the count of what crossed it. The port it closes sits on the top edge, because the flow arrives from below, from inside the scope of the container.",
  );
}

/**
 * The plain plate: the shape of a block whose class nobody knows.
 *
 * The plate keeps the proportion of the box the model gives the block, so
 * the canvas shows the block instead of breaking on it.
 *
 * @param size the footprint of the silhouette, in pixels
 */
export function plainShape(size: FrameSize): BlockShape {
  return plateShape(
    PLAIN_FAMILY,
    size,
    [],
    "the plain plate of a block whose class the catalogue does not know: the canvas never breaks because of a class it has never heard of, and the block keeps the box the model gives it.",
    false,
  );
}

/**
 * A plate: a rounded rectangle with the zones of its name and its figure.
 *
 * The plate is the shape of the blocks the catalogue does not draw: the
 * container, the plain plate of an unknown class and the doors. Its name
 * goes on the upper left, the figure on its right, and both zones stay
 * inside the silhouette, so the text never falls outside the shape.
 *
 * @param family the family of the plate, and with it its colours
 * @param size the footprint of the silhouette, in pixels
 * @param ports the ports of the plate, with their anchors
 * @param note why the plate looks the way it looks
 * @param reserveFigure whether the zone of the figure is always reserved
 */
function plateShape(
  family: Family,
  size: FrameSize,
  ports: PortAnchor[],
  note: string,
  reserveFigure: boolean,
): BlockShape {
  const width = Math.max(1, Math.round(size.width));
  const height = Math.max(1, Math.round(size.height));
  const zones = plateZones(width, height, reserveFigure);
  return {
    family,
    variant: "rounded plate",
    footprint: { width, height },
    path: roundedPath(width, height),
    details: [],
    ports,
    label: zones.label,
    figure: zones.figure,
    note,
  };
}

/**
 * The zones of a plate: the name on the left and the figure on its right.
 *
 * The name needs two lines of text — the name of the block and the caption
 * under it — so the zone of the name is as tall as they are. The figure
 * only has a place when there is room for it beside the name: a zone that
 * does not fit is not reserved, and the hole it would leave is better than
 * a figure written over the name. The container asks for the zone anyway:
 * it shows its figures, and the hole of a figure that does not fit is
 * better than a container that cannot say how many entities it holds.
 */
function plateZones(
  width: number,
  height: number,
  reserveFigure = false,
): { label: Zone; figure: Zone | null } {
  // Two lines of text: the name of the block and the caption under it.
  const labelHeight = Math.max(1, Math.min(24, height - PLATE_AIR));
  // The room the figure takes beside the name, and the name that has to
  // stay whole for the figure to have a place at all.
  const room = FIGURE_WIDTH + 6;
  const hasFigure =
    reserveFigure || (width - PLATE_AIR * 2 - room >= 40 && height >= 24);
  const wanted = hasFigure ? width - PLATE_AIR * 2 - room : width - PLATE_AIR * 2;
  // The zone of the name stays inside the plate, whatever the plate is: a
  // plate too small for the air of the design keeps the air it has.
  const left = Math.min(PLATE_AIR, Math.max(0, width - 1));
  const top = Math.min(PLATE_AIR - 2, Math.max(0, height - 1));
  const labelWidth = Math.max(1, Math.min(wanted, width - left));
  const labelHeightInside = Math.max(1, Math.min(labelHeight, height - top));
  // The zone of the figure is inside the plate too: a plate too small for
  // the whole figure keeps the part of it that fits.
  const figureLeft = Math.max(0, width - PLATE_AIR - FIGURE_WIDTH);
  return {
    label: {
      x: left,
      y: top,
      width: labelWidth,
      height: labelHeightInside,
    },
    figure: hasFigure
      ? {
          x: figureLeft,
          y: top,
          width: Math.max(1, Math.min(FIGURE_WIDTH, width - figureLeft)),
          height: 14,
        }
      : null,
  };
}

/**
 * A door: the plate the flow crosses.
 *
 * The door is the plate with a point on its right: the shape says that the
 * flow goes on, and the family says whether it comes in or goes out. Its
 * name goes on the left and the count of what crossed it on the right, and
 * neither of them touches the point.
 */
function doorShape(
  family: Family,
  anchor: PortAnchor,
  note: string,
): BlockShape {
  const width = DOOR_FOOTPRINT.width;
  const height = DOOR_FOOTPRINT.height;
  // The width of the point on the right, where the flow goes on.
  const tip = 24;
  return {
    family,
    variant: "door",
    footprint: { width, height },
    path: `M 0 0 H ${width - tip} L ${width} ${height / 2} L ${width - tip} ${height} H 0 Z`,
    details: [],
    ports: [anchor],
    label: { x: PLATE_AIR, y: PLATE_AIR - 2, width: 70, height: 24 },
    figure: { x: 80, y: 10, width: FIGURE_WIDTH, height: 14 },
    note,
  };
}

/**
 * The path of a rounded rectangle.
 *
 * The corners are rounded with the radius of the plate, or with half of
 * the shorter side when the plate is too small for that radius: a corner
 * wider than the plate is not a corner.
 */
function roundedPath(width: number, height: number): string {
  const radius = Math.min(PLATE_RADIUS, Math.floor(Math.min(width, height) / 2));
  return (
    `M ${radius} 0 H ${width - radius} A ${radius} ${radius} 0 0 1 ${width} ${radius} ` +
    `V ${height - radius} A ${radius} ${radius} 0 0 1 ${width - radius} ${height} ` +
    `H ${radius} A ${radius} ${radius} 0 0 1 0 ${height - radius} ` +
    `V ${radius} A ${radius} ${radius} 0 0 1 ${radius} 0 Z`
  );
}
