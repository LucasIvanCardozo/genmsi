/**
 * How a block of the canvas is drawn.
 *
 * The shapes module holds the truth of every class: its silhouette, the
 * zones where its text and its figure go, and the anchor of each of its
 * ports (S2, S8, S9). This module turns that truth into what the node of
 * the canvas needs:
 *
 * - the **fit** of the silhouette in the box of the block: the shape keeps
 *   its own proportion and sits in the middle of the box, so a block whose
 *   box is bigger than the class keeps the shape of the class in the middle
 *   instead of stretching it (a circle stays a circle);
 * - the **zones**, in the pixels of the shape's own footprint, so the text
 *   lands inside the silhouette whatever the box of the block is;
 * - the **ports**, at the anchor the original editor gave them, placed in
 *   the shape's own frame;
 * - the **text that fits a zone**, measured with the measure of the design
 *   (`catalogue.ts`), so the canvas cuts the same words at the same place
 *   as the design sheet does.
 *
 * It is pure: it draws nothing and reads nothing, so the tests can ask it
 * about a block without a browser. The node of the canvas only draws what
 * this module says.
 */
import { portSpread } from "./graph.ts";
import { truncate } from "./catalogue.ts";
import {
  familyColors,
  shapeOf,
  type BlockShape,
  type FamilyColors,
  type PortAnchor,
  type Zone,
} from "./shapes.ts";

/** The side of the block a port sits on. */
export type PortSide = "top" | "bottom" | "left" | "right";

/** One port of the model, as the drawing reads it. */
export interface PortOfBlock {
  /** The id of the port inside the block. */
  ioid: number;
  /** The name of the port, as the catalogue declares it. */
  name: string;
}

/** Where a port sits, as a fraction of the footprint of the shape. */
export interface PortPlacement {
  /** The id of the port inside the block. */
  ioid: number;
  /** The name of the port, as the catalogue declares it. */
  name: string;
  /** The horizontal anchor: 0 is the left border, 1 the right one. */
  x: number;
  /** The vertical anchor: 0 is the top border, 1 the bottom one. */
  y: number;
  /** The side of the silhouette the port belongs to. */
  side: PortSide;
}

/** A zone of the block, in the pixels of the shape's own footprint. */
export interface FootprintZone {
  /** The left of the zone, in the pixels of the footprint. */
  left: number;
  /** The top of the zone, in the pixels of the footprint. */
  top: number;
  /** The width of the zone, in the pixels of the footprint. */
  width: number;
  /** The height of the zone, in the pixels of the footprint. */
  height: number;
}

/** The box a block is drawn in, in pixels. */
export interface BlockBox {
  /** The width of the box. */
  width: number;
  /** The height of the box. */
  height: number;
}

/**
 * How the silhouette of a class sits in the box of a block.
 *
 * The shape keeps its own proportion: it is drawn at the size of its
 * footprint, or smaller when the box of the block cannot hold it, and it
 * sits in the middle of the box. A block whose box is bigger than the
 * shape keeps the shape of its class in the middle, untouched.
 */
export interface ShapeFit {
  /** The scale of the footprint inside the box. */
  scale: number;
  /** How far the silhouette is from the left border of the box. */
  offsetX: number;
  /** How far the silhouette is from the top border of the box. */
  offsetY: number;
  /** The width of the silhouette once scaled. */
  width: number;
  /** The height of the silhouette once scaled. */
  height: number;
}

/** How a class of block is drawn. */
export interface BlockDrawing {
  /** The shape of the class, or `undefined` when the class is unknown. */
  shape: BlockShape | undefined;
  /** The colours of the family, or `undefined` when the class is unknown. */
  colors: FamilyColors | undefined;
  /** Whether the block falls back to the plain rounded rectangle. */
  fallback: boolean;
  /** The zone of the label, in the pixels of the footprint of the class. */
  label: FootprintZone | null;
  /** The zone of the figure, in the pixels of the footprint of the class. */
  figure: FootprintZone | null;
  /** Whether the zone of the label has room for the caption. */
  subtitle: boolean;
}

/** The counts a block shows when it has no other figure. */
export interface CountsOfBlock {
  /** The transactions inside the block right now. */
  now: number;
  /** The transactions that have gone through it since the run started. */
  ever: number;
}

/** The figure a block shows, as the live layer publishes it. */
export interface FigureOfBlock {
  /** The text to draw in the zone of the figure. */
  text: string;
}

/**
 * The height a zone of a label needs to carry the rótulo and the caption.
 *
 * Below it, the block carries the rótulo alone: the caption would not fit
 * inside the silhouette.
 */
const SUBTITLE_HEIGHT = 20;

/** The size of the rótulo, in pixels: the size the design uses. */
export const LABEL_SIZE = 9;

/** The size of the caption under the rótulo, in pixels. */
export const CAPTION_SIZE = 9;

/** The size of the live figure, in monospaced, in pixels. */
export const FIGURE_SIZE = 10;

/**
 * The air the text leaves inside its zone, in pixels.
 *
 * It is the air the design sheet leaves between the text and the border of
 * its zone, and it is what keeps the last letter of a cut word off the
 * edge of the silhouette.
 */
const ZONE_AIR = 4;

/** The height of a line of the text of a block, in pixels. */
const LINE_HEIGHT = 11;

/**
 * The size of the dot of a port, in pixels.
 *
 * The dot is the strong tone of the family with a white ring around it, and
 * it is the same size for every port of every class (S8).
 */
export const PORT_SIZE = 12;

/**
 * How a class of block is drawn.
 *
 * A class the shapes module does not know falls back to the plain rounded
 * rectangle, so that the canvas never breaks. A container falls back too:
 * it is the frame around the blocks, not a step of the flow.
 */
export function drawingOf(className: string, container = false): BlockDrawing {
  const shape = container ? undefined : shapeOf(className);
  if (shape === undefined) {
    return {
      shape: undefined,
      colors: undefined,
      fallback: true,
      label: null,
      figure: null,
      subtitle: false,
    };
  }
  return {
    shape,
    colors: familyColors(shape.family),
    fallback: false,
    label: zoneOf(shape.label),
    figure: shape.figure === null ? null : zoneOf(shape.figure),
    subtitle: shape.label.height >= SUBTITLE_HEIGHT,
  };
}

/**
 * A zone of the shape, in the pixels of the footprint of its class.
 *
 * The zones are declared in the pixels of the footprint of the class, and
 * they are the zones of the *shape*: the text lands inside the silhouette
 * whatever the box of the block is, because the silhouette is drawn in the
 * very frame these pixels describe.
 */
export function zoneOf(zone: Zone): FootprintZone {
  return {
    left: zone.x,
    top: zone.y,
    width: zone.width,
    height: zone.height,
  };
}

/**
 * How the silhouette of a class sits in the box of a block.
 *
 * The silhouette keeps its own proportion: it is drawn at the size of its
 * footprint, and it only shrinks when the box of the block cannot hold it
 * (a note of 80×80 in a box of 116×32 is drawn as a 32×32 sheet in the
 * middle). A block whose box is bigger than the shape keeps the shape of
 * its class in the middle, without stretching it.
 */
export function fitOf(shape: BlockShape, box: BlockBox): ShapeFit {
  const { width, height } = shape.footprint;
  // A box that is not measured yet draws the shape at its own size.
  const boxWidth = box.width > 0 ? box.width : width;
  const boxHeight = box.height > 0 ? box.height : height;
  const scale = Math.min(1, boxWidth / width, boxHeight / height);
  const drawnWidth = width * scale;
  const drawnHeight = height * scale;
  return {
    scale,
    offsetX: (boxWidth - drawnWidth) / 2,
    offsetY: (boxHeight - drawnHeight) / 2,
    width: drawnWidth,
    height: drawnHeight,
  };
}

/**
 * The side of the silhouette a port belongs to.
 *
 * A port on a border of the footprint belongs to that border; a port in the
 * middle of a side belongs to the side it is closest to.
 */
export function sideOf(anchor: PortAnchor): PortSide {
  if (anchor.y <= 0) {
    return "top";
  }
  if (anchor.y >= 1) {
    return "bottom";
  }
  if (anchor.x <= 0) {
    return "left";
  }
  if (anchor.x >= 1) {
    return "right";
  }
  const sides: [PortSide, number][] = [
    ["top", anchor.y],
    ["bottom", 1 - anchor.y],
    ["left", anchor.x],
    ["right", 1 - anchor.x],
  ];
  let closest = sides[0];
  for (const side of sides) {
    if (side[1] < closest[1]) {
      closest = side;
    }
  }
  return closest[0];
}

/**
 * Where the ports of a block sit.
 *
 * The anchor comes from the shape of the class: a `Test` leaves by the right
 * vertex of its diamond, and a `Multifurcacion` spreads its five exits
 * along its bottom edge (S8). The anchor is a fraction of the footprint of
 * the class, so the port sits in the shape's own frame and follows it when
 * the silhouette keeps its proportion in the box of the block.
 *
 * A port the shape does not know keeps the place the ports used to have,
 * spread along the side: a link that loses its handle is a link the reader
 * cannot follow.
 */
export function portPlacements(
  shape: BlockShape | undefined,
  ports: readonly PortOfBlock[],
  fallbackSide: PortSide,
): PortPlacement[] {
  return ports.map((port, index) => {
    const anchor =
      shape === undefined ? undefined : shape.ports.find((one) => one.name === port.name);
    if (anchor === undefined) {
      return {
        ioid: port.ioid,
        name: port.name,
        x: portSpread(ports.length, index),
        y: fallbackSide === "top" ? 0 : 1,
        side: fallbackSide,
      };
    }
    return {
      ioid: port.ioid,
      name: port.name,
      x: anchor.x,
      y: anchor.y,
      side: sideOf(anchor),
    };
  });
}

/**
 * The pair of always: how many transactions are inside the block, and how
 * many have gone through it.
 *
 * A block the run has not reached shows nothing: the space of the figure is
 * reserved anyway, so the rótulo does not move when the figure appears.
 */
export function defaultPair(counts: CountsOfBlock | undefined): string {
  if (counts === undefined || counts.ever <= 0) {
    return "";
  }
  return `${counts.now}/${counts.ever}`;
}

/**
 * The text the zone of the figure shows.
 *
 * The figure of the block comes from the live layer: the number of the
 * entity the block operates on, or the pair of always. A block the live
 * layer has no figure for shows the pair of always.
 */
export function figureText(
  figure: FigureOfBlock | undefined,
  counts: CountsOfBlock | undefined,
): string {
  if (figure !== undefined) {
    return figure.text;
  }
  return defaultPair(counts);
}

/**
 * The rótulo of a block, cut to the width of its zone.
 *
 * The measure is the one of the design (`catalogue.ts`), and the cut is the
 * one of the design sheet: the name is cut with an ellipsis when it does
 * not fit in the zone, and a name that fits is never cut.
 */
export function labelText(name: string, zone: FootprintZone): string {
  return truncate(name, zone.width - ZONE_AIR, LABEL_SIZE);
}

/**
 * The caption of a block, cut to the width of its zone.
 *
 * The caption shares the zone of the rótulo: it goes in the second line of
 * it, and it is cut with the same measure and the same air.
 */
export function captionText(caption: string, zone: FootprintZone): string {
  return truncate(caption, zone.width - ZONE_AIR, CAPTION_SIZE);
}

/**
 * The live figure of a block, cut to the width of its zone.
 *
 * The figure is monospaced, and its zone already holds the air the design
 * leaves around it, so the cut is the width of the zone itself.
 */
export function figureZoneText(text: string, zone: FootprintZone): string {
  return truncate(text, zone.width, FIGURE_SIZE, true);
}

/**
 * The line the rótulo is written on, as the baseline of its text.
 *
 * The rótulo goes on the first line of the zone of the label, at the top of
 * it: that leaves the second line for the caption, and it keeps the text
 * away from the edge of the silhouette.
 */
export function labelBaseline(zone: FootprintZone): number {
  return zone.top + Math.min(LABEL_SIZE, zone.height - 3);
}

/**
 * The line the caption is written on, as the baseline of its text.
 *
 * The caption goes on the second line of the zone of the label, and never
 * below the zone: a zone that is too short for two lines carries the
 * rótulo alone.
 */
export function captionBaseline(zone: FootprintZone): number {
  return zone.top + Math.min(LABEL_SIZE + LINE_HEIGHT, zone.height - 2);
}

/**
 * The line the figure is written on, as the baseline of its text.
 *
 * The figure is written at the top of its zone, at its right border, the
 * way the design sheet writes it.
 */
export function figureBaseline(zone: FootprintZone): number {
  return zone.top + zone.height - 2;
}
