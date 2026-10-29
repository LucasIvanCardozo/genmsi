/**
 * How a block of the canvas is drawn.
 *
 * The shapes module holds the truth of every class: its silhouette, the
 * zones where its text and its figure go, and the anchor of each of its
 * ports (S2, S8, S9). The frames module holds the truth of the blocks that
 * are not classes of the catalogue: the container and the doors of a
 * container. This module turns that truth into what the node of the canvas
 * needs:
 *
 * - the **silhouette**, which the class decides for the 66 classes and the
 *   kind decides for the other three: the container, the entry door and
 *   the exit door;
 * - the **size of the node**, which is the footprint of the shape and the
 *   room the outline needs: the shape is the truth, and it is drawn one to
 *   one, never scaled and never cropped;
 * - the **zones**, in the pixels of the shape's own footprint, so the text
 *   lands inside the silhouette;
 * - the **lines of the rótulo**: a name that does not fit its zone is
 *   written on two lines, and no character of it is ever cut;
 * - the **caption**, drawn smaller before it is cut, because it is the
 *   reader's own text;
 * - the **figure**, centred in its zone, and the dash that says the run
 *   published no figure for a block;
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
import { portSpread, type BlockKind } from "./graph.ts";
import { monospaceWidth, textWidth, truncate } from "./catalogue.ts";
import {
  containerShape,
  plainShape,
  portInShape,
  portOutShape,
  STROKE_MARGIN,
  type FrameSize,
} from "./frames.ts";
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

/** The size of the node of a block, in pixels. */
export interface NodeSize {
  /** The width of the node. */
  width: number;
  /** The height of the node. */
  height: number;
}

/**
 * The room the border of the silhouette needs inside the box of the block.
 *
 * The outline is drawn centred on the line of the shape, so the half that
 * falls outside is cut by the box of the drawing. Two pixels on each side
 * leave the whole border visible.
 *
 * The margin lives with the shapes, in the frames module, and it is
 * re-exported here because the node of the canvas draws with it.
 */
export { STROKE_MARGIN };

/** How a block of the canvas is drawn. */
export interface BlockDrawing {
  /** The silhouette of the block. */
  shape: BlockShape;
  /** The colours of the family of the silhouette. */
  colors: FamilyColors;
  /** Whether the block falls back to the plain rounded rectangle. */
  fallback: boolean;
  /** The zone of the label, in the pixels of the footprint of the shape. */
  label: FootprintZone;
  /** The zone of the figure, in the pixels of the footprint of the shape. */
  figure: FootprintZone | null;
  /** Whether the zone of the label has room for the caption. */
  subtitle: boolean;
  /** The box the block is drawn in: the footprint and the room of the outline. */
  size: NodeSize;
}

/**
 * What the canvas knows about a block to draw it.
 *
 * The class decides the silhouette of the 66 classes of the catalogue, and
 * the kind decides the silhouette of the other three blocks of the canvas:
 * the container, the entry door and the exit door. The size is the one the
 * model gives the block, which is the truth for the container and for a
 * block of a class the catalogue does not know.
 */
export interface BlockToDraw {
  /** The class of the block; a container and a door have none. */
  className: string;
  /** The kind of the block; an activity block when the kind is not given. */
  kind?: BlockKind;
  /** The name the block shows as its rótulo; its class when it is not given. */
  title?: string;
  /** The caption the block shows under its rótulo. */
  subtitle?: string;
  /** The size the model gives the block, or `null` when the model gives none. */
  size?: NodeSize | null;
  /** The input ports of the block, in the order of the model. */
  inputs?: readonly PortOfBlock[];
  /** The output ports of the block, in the order of the model. */
  outputs?: readonly PortOfBlock[];
}

/**
 * The size a block is drawn at when the model gives it none.
 *
 * A block the model does not place is not drawn at all, but the drawing
 * module cannot know that: it needs a box to draw in, and this is the
 * box it uses when the model carries none.
 */
export const DEFAULT_SIZE: NodeSize = { width: 100, height: 40 };

/**
 * The name a block shows and the caption under it.
 */
export interface BlockTexts {
  /** The rótulo: the name of the block. */
  label: string;
  /** The caption: what the block says under its name. */
  caption: string;
}

/**
 * The text a block shows: its name and the caption under it.
 *
 * The 66 classes show the name of their class and, under it, the caption of
 * the model. The container shows the caption of the model as its name —
 * that is how the reader knows which container it is — and the word
 * "Contenedor" under it, because that is what it is. The doors show the
 * name of the port and, under it, the word that says which side of the
 * container they are.
 */
export function textsOf(kind: BlockKind, title: string, subtitle: string): BlockTexts {
  switch (kind) {
    case "C":
      return { label: subtitle !== "" ? subtitle : title, caption: "Contenedor" };
    case "PortIn":
      return { label: title, caption: "Entrada" };
    case "PortOut":
      return { label: title, caption: "Salida" };
    default:
      return { label: title, caption: subtitle };
  }
}

/**
 * The ring of the selection of a block.
 *
 * The ring goes on the wrapper of the block, the box that holds the
 * silhouette, and not on the silhouette itself: the border of the shape is
 * drawn inside its own box, so a ring on the inner box would only make the
 * border look thicker. On the wrapper it is a clean square around the
 * silhouette, the same square for the 66 classes, for the container and
 * for the doors (S3).
 */
export const SELECTION_RING = "ring-2 ring-indigo-500";

/**
 * The class of the wrapper of a block: the box that holds the silhouette.
 *
 * The wrapper is the outer box of the node of the canvas, and it is the
 * one that carries the ring of the selection: every block of the canvas is
 * selected the same way, because every block is drawn the same way.
 *
 * @param selected whether the block is selected
 * @param note whether the block is a note, which is drawn dimmed
 */
export function wrapperClass(selected: boolean, note = false): string {
  return [
    "relative h-full w-full",
    note ? "opacity-70" : "",
    selected ? SELECTION_RING : "",
  ].join(" ");
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
export const SUBTITLE_HEIGHT = 20;

/** The size of the rótulo, in pixels: the size the design uses. */
export const LABEL_SIZE = 9;

/** The size of the caption under the rótulo, in pixels. */
export const CAPTION_SIZE = 9;

/**
 * The smallest size the caption of a block is drawn at, in pixels.
 *
 * A caption that does not fit its zone at the size of the design is drawn
 * smaller before it is cut: the caption is the reader's own text, and a
 * caption that is cut before it has to be is noise. Below this size the
 * caption is cut with an ellipsis.
 */
export const CAPTION_MIN_SIZE = 7;

/** The size of the live figure, in monospaced, in pixels. */
export const FIGURE_SIZE = 10;

/**
 * The air the text leaves inside its zone, in pixels.
 *
 * It is the air the design sheet leaves between the text and the border of
 * its zone, and it is what keeps the last letter of a cut word off the
 * edge of the silhouette.
 */
export const ZONE_AIR = 4;

/** The height of a line of the text of a block, in pixels. */
export const LINE_HEIGHT = 11;

/**
 * The air between two lines of the same text, in pixels.
 *
 * It is the air the design sheet leaves between the two lines of a name
 * that does not fit in one of them.
 */
export const LINE_AIR = 2;

/**
 * The size of the dot of a port, in pixels.
 *
 * The dot is the strong tone of the family with a white ring around it, and
 * it is the same size for every port of every class (S8).
 */
export const PORT_SIZE = 12;

/**
 * How a block of the canvas is drawn.
 *
 * The class of the block decides its silhouette when the class is one of
 * the 66 of the catalogue. The kind decides it when the class is not: the
 * container, the entry door and the exit door have the shapes of the
 * frames module, and a class the catalogue does not know falls back to the
 * plain plate, so that the canvas never breaks.
 *
 * The box of the node is the footprint of the silhouette and the room the
 * outline needs: the shape is drawn one to one, never scaled and never
 * cropped, and the box of the node and the viewBox of its silhouette
 * agree.
 */
export function drawingOf(block: BlockToDraw): BlockDrawing {
  const kind = block.kind ?? "A";
  const inputs = block.inputs ?? [];
  const outputs = block.outputs ?? [];
  // The class decides the shape of the 66 classes, and only of them.
  const classShape = kind === "A" ? shapeOf(block.className) : undefined;
  const shape = classShape ?? shapeOfKind(kind, block, inputs, outputs);
  return {
    shape,
    colors: familyColors(shape.family),
    fallback: kind === "A" && classShape === undefined,
    label: zoneOf(shape.label),
    figure: shape.figure === null ? null : zoneOf(shape.figure),
    subtitle: shape.label.height >= SUBTITLE_HEIGHT,
    size: {
      width: shape.footprint.width + 2 * STROKE_MARGIN,
      height: shape.footprint.height + 2 * STROKE_MARGIN,
    },
  };
}

/**
 * The silhouette of a block the catalogue of classes does not draw.
 *
 * The container is a block with the box the model gives it, and its ports
 * sit on the edges of its own silhouette: the inputs on the top edge and
 * the outputs on the bottom one, spread along it. The doors have the
 * silhouette of the plate the flow crosses, and the one port they open or
 * close sits on the edge the flow crosses. A class the catalogue does not
 * know is drawn as the plain plate.
 */
function shapeOfKind(
  kind: BlockKind,
  block: BlockToDraw,
  inputs: readonly PortOfBlock[],
  outputs: readonly PortOfBlock[],
): BlockShape {
  switch (kind) {
    case "C":
      return containerShape(footprintOf(block.size), {
        inputs: inputs.map((port, index) => ({
          name: port.name,
          x: portSpread(inputs.length, index),
          y: 0,
        })),
        outputs: outputs.map((port, index) => ({
          name: port.name,
          x: portSpread(outputs.length, index),
          y: 1,
        })),
      });
    case "PortIn":
      return portInShape(doorName(inputs, outputs));
    case "PortOut":
      return portOutShape(doorName(inputs, outputs));
    default:
      return plainShape(footprintOf(block.size));
  }
}

/**
 * The name of the port a door carries: it is the one port the door has.
 */
function doorName(
  inputs: readonly PortOfBlock[],
  outputs: readonly PortOfBlock[],
): string {
  const port = outputs[0] ?? inputs[0];
  return port === undefined ? "" : port.name;
}

/**
 * The footprint of a block the model sizes.
 *
 * The box the model gives the block is the box of the node, and the
 * silhouette is drawn inside it with the room the outline needs: the
 * outline is drawn centred on the line of the shape, so half of it falls
 * outside, and without that room the border is cut flat on the four sides.
 */
function footprintOf(size: NodeSize | null | undefined): FrameSize {
  const box = size ?? DEFAULT_SIZE;
  return {
    width: Math.max(1, Math.round(box.width) - 2 * STROKE_MARGIN),
    height: Math.max(1, Math.round(box.height) - 2 * STROKE_MARGIN),
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
 * The size of the node of a block: the footprint of its shape, one to one.
 *
 * The shape is the truth: a block is drawn at the size of its class, with
 * its own proportion, and never scaled to the box the model happens to
 * carry. A block whose model carries an older, smaller box is drawn at the
 * size of its class: the model is not rewritten, and the reader can lay
 * the canvas out again with the relayout command of the toolbar.
 *
 * The container, the doors and a class the shapes module does not know
 * have no class to say how big they are: they are drawn at the box the
 * model gives them, and their silhouette is the plate that fits in it.
 */
export function nodeSizeOf(block: BlockToDraw): NodeSize {
  return drawingOf(block).size;
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
 * The anchor comes from the shape of the block: a `Test` leaves by the
 * right vertex of its diamond, and a `Multifurcacion` spreads its five
 * exits along its bottom edge (S8). The anchor is a fraction of the
 * footprint of the shape, so the port sits in the shape's own frame, and
 * the block is drawn at the size of that footprint: the port falls on the
 * very point of the silhouette it belongs to.
 *
 * A port the shape does not declare keeps the place the ports used to have,
 * spread along the side: a link that loses its handle is a link the reader
 * cannot follow.
 */
export function portPlacements(
  shape: BlockShape,
  ports: readonly PortOfBlock[],
  fallbackSide: PortSide,
): PortPlacement[] {
  return ports.map((port, index) => {
    const anchor = shape.ports.find((one) => one.name === port.name);
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
 * The lines the rótulo of a block is written on.
 *
 * The name of a class does not always fit in one line of its zone. When it
 * does not fit, it is written on two lines, and no character of it is lost:
 * the name of a class is never cut with an ellipsis.
 *
 * The measure is the one of the design (`catalogue.ts`): the name is
 * monospaced, and the zone is the one the class declares.
 */
export function labelLines(name: string, zone: FootprintZone): string[] {
  const room = zone.width - ZONE_AIR;
  if (monospaceWidth(name, LABEL_SIZE) <= room) {
    return [name];
  }
  return splitInTwo(name, room);
}

/**
 * The name of a class, in two lines that fit the zone.
 *
 * The break is at a space when the name has one and the two halves fit
 * there; otherwise the name is broken in the middle, at the point that
 * balances the two lines best. The space of the break stays with the line
 * it belongs to, so the two lines together are the whole name.
 */
function splitInTwo(name: string, room: number): string[] {
  const width = (part: string): number => monospaceWidth(part.trim(), LABEL_SIZE);
  let best: Break | undefined;
  for (let index = 1; index < name.length; index += 1) {
    const first = name.slice(0, index);
    const second = name.slice(index);
    if (first.trim() === "" || second.trim() === "") {
      continue;
    }
    const candidate: Break = {
      first,
      second,
      // A break at a space keeps the word whole: the space that ends the
      // first line is the best one, and the space that starts the second
      // line is the next best. A break at a capital keeps the words of a
      // name whole too: "RemoveNumber" breaks as "Remove" and "Number",
      // not as "Remov" and "eNumber".
      kind: first.endsWith(" ")
        ? 2
        : second.startsWith(" ")
          ? 1
          : isCapitalBreak(name, index)
            ? 1
            : 0,
      fits: width(first) <= room && width(second) <= room,
      balance: Math.abs(first.length - second.length),
    };
    if (best === undefined || breaksBetter(candidate, best)) {
      best = candidate;
    }
  }
  if (best === undefined) {
    // A name of one character cannot be broken in two.
    return [name];
  }
  return [best.first, best.second];
}

/** One way of breaking a name in two lines. */
interface Break {
  /** The first line of the name. */
  first: string;
  /** The second line of the name. */
  second: string;
  /** How good the place of the break is: 2 is a space at the end of the
   * first line, 1 is a space at the start of the second one or a capital
   * that starts a word, and 0 is a break inside a word. */
  kind: number;
  /** Whether both lines fit the zone. */
  fits: boolean;
  /** How far the two lines are from being the same length. */
  balance: number;
}

/**
 * Whether a break falls where a capital starts a word.
 *
 * The names of the classes are written in camel case, and the capital is
 * where the name says a word ends.
 */
function isCapitalBreak(name: string, index: number): boolean {
  const letter = name[index];
  if (letter === undefined || !/[A-Z]/.test(letter)) {
    return false;
  }
  const before = name[index - 1];
  return before !== undefined && /[a-z0-9]/.test(before);
}

/**
 * Whether one break of a name is better than another.
 *
 * A break whose two lines fit the zone wins over one whose lines do not; a
 * break at a space wins over one at a capital, and one at a capital wins
 * over one inside a word; and, between two breaks of the same kind, the one
 * that balances the two lines wins.
 */
function breaksBetter(candidate: Break, best: Break): boolean {
  if (candidate.fits !== best.fits) {
    return candidate.fits;
  }
  if (candidate.kind !== best.kind) {
    return candidate.kind > best.kind;
  }
  return candidate.balance < best.balance;
}

/** How the caption of a block is drawn. */
export interface CaptionLayout {
  /** The text to draw in the zone of the rótulo. */
  text: string;
  /** The size to draw it at, in pixels. */
  size: number;
}

/**
 * How the caption of a block is drawn: its text and its size.
 *
 * The caption shares the zone of the rótulo: it goes in the second line of
 * it, and the room it has is the width of the zone minus the air the
 * design leaves.
 *
 * The caption is drawn at the size of the design when it fits. When it does
 * not fit, it is drawn smaller — down to the smallest size the design
 * allows — and only a caption that does not fit even there is cut with an
 * ellipsis. The caption is the reader's own text: it is the last thing to
 * be cut.
 */
export function captionLayout(caption: string, zone: FootprintZone): CaptionLayout {
  const room = zone.width - ZONE_AIR;
  // The width of the caption per pixel of size: the measure of the design
  // is proportional, so the size that fits is the room divided by it.
  const perPixel = textWidth(caption, 1);
  if (caption === "" || room <= 0 || perPixel <= 0) {
    return { text: caption, size: CAPTION_SIZE };
  }
  const size = room / perPixel;
  if (size >= CAPTION_SIZE) {
    return { text: caption, size: CAPTION_SIZE };
  }
  if (size >= CAPTION_MIN_SIZE) {
    // The whole caption fits at a smaller size: it is drawn smaller, and
    // not one character of it is lost.
    return { text: caption, size: Math.floor(size * 10) / 10 };
  }
  // It does not fit even at the smallest size: then, and only then, it is
  // cut with the ellipsis that says the text goes on.
  return {
    text: truncate(caption, room, CAPTION_MIN_SIZE),
    size: CAPTION_MIN_SIZE,
  };
}

/**
 * The caption of a block, cut to the width of its zone.
 *
 * The text of the caption, at whatever size it has to be drawn: the size
 * is `captionLayout`.
 */
export function captionText(caption: string, zone: FootprintZone): string {
  return captionLayout(caption, zone).text;
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
 * The dash that says the run published no figure for a block.
 *
 * A class that reserves the zone of the figure and a run that publishes no
 * number for it would leave a silent hole, and a hole reads like a fault.
 * The dash says "there is no value here": it is not a number, and it does
 * not pretend to be one.
 */
export const NO_FIGURE = "—";

/** How the figure of a block is drawn. */
export interface FigureLayout {
  /** The text to draw in the zone of the figure. */
  text: string;
  /** The x of the text: the middle of the zone of the figure. */
  x: number;
  /** The baseline of the text. */
  y: number;
  /** The anchor of the text: the figure is centred in its zone. */
  anchor: "middle";
  /** The size of the text, in pixels. */
  size: number;
  /** Whether the zone shows the dash of a block with no figure. */
  empty: boolean;
}

/**
 * How the figure of a block is drawn: its text, its place and its size.
 *
 * The figure is centred in the zone the shape declares: the round blocks
 * read better with it (the `Terminate` is a circle whose number sat off to
 * the right of its middle), and the wide blocks keep their own box for the
 * figure, so the number stays in the middle of it.
 *
 * A block the run publishes no figure for shows the dash of an empty
 * figure: the zone is reserved, and the hole it leaves is filled with a
 * sign that says there is no value here.
 */
export function figureLayout(zone: FootprintZone, text: string): FigureLayout {
  const empty = text === "";
  return {
    text: empty ? NO_FIGURE : text,
    x: figureCenterX(zone),
    y: figureBaseline(zone),
    anchor: "middle",
    size: FIGURE_SIZE,
    empty,
  };
}

/**
 * The middle of the zone of the figure, in pixels.
 *
 * The figure is written in the middle of its zone: that is where the
 * reader looks for it.
 */
export function figureCenterX(zone: FootprintZone): number {
  return zone.left + zone.width / 2;
}

/**
 * The lines the rótulo is written on, as the baselines of its text.
 *
 * The rótulo goes on the first line of the zone of the label, at the top of
 * it: that leaves the second line for the caption, and it keeps the text
 * away from the edge of the silhouette. A name that needs two lines takes
 * the second line too, with the air the design leaves between two lines.
 */
export function labelBaselines(zone: FootprintZone, lines: number): number[] {
  const first = zone.top + Math.min(LABEL_SIZE, zone.height - 3);
  const baselines: number[] = [];
  for (let index = 0; index < lines; index += 1) {
    baselines.push(first + index * (LINE_HEIGHT + LINE_AIR));
  }
  return baselines;
}

/**
 * The height the lines of the rótulo take, in pixels.
 *
 * A line of text is `LINE_HEIGHT` high and two lines have `LINE_AIR` of
 * air between them: two lines take 24 px, which is what a zone needs to
 * carry a name that does not fit in one of them.
 */
export function labelHeight(lines: number): number {
  return lines * LINE_HEIGHT + Math.max(0, lines - 1) * LINE_AIR;
}

/**
 * Whether the caption has its place under the lines of the rótulo.
 *
 * The caption goes on the second line of the zone of the label. A name that
 * needed two lines takes that line as well, and then the caption has no
 * place: the name of the class matters more than its caption.
 */
export function captionFits(zone: FootprintZone, lines: number): boolean {
  const baselines = labelBaselines(zone, lines);
  return captionBaseline(zone) > baselines[baselines.length - 1];
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
