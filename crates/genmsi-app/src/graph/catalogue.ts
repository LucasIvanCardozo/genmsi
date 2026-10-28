/**
 * The pure helpers of the catalogue list.
 *
 * The tab **Bloques** of the lateral panel is the catalogue the model is
 * built from, read at a glance: every class shows its silhouette, its name
 * and what it does. The arithmetic of that list lives here, in pure
 * functions, so it can be tested without a browser:
 *
 * - the classes of the catalogue **grouped by family**, in the order of the
 *   flow (what is born, what waits, what is taken, what is stored, what
 *   decides, what operates);
 * - the **geometry of the thumbnail**: the silhouette scaled with its own
 *   proportion to fit the box of the cell, centred in it;
 * - the **truncation** of the name and of the title to the width of the
 *   cell.
 *
 * The silhouette is the one of `src/graph/shapes.ts`: the list draws the
 * very same path the canvas draws (S6), or the list lies about the shape.
 */
import {
  familyColors,
  shapeOf,
  type BlockShape,
  type Family,
  type FamilyColors,
} from "./shapes.ts";

/**
 * The order the families appear in the list.
 *
 * It is the order of the flow, the one the design uses: what is born, what
 * ends, what waits, what is taken, what is stored, what is written down,
 * what is chained, what is grouped, what is chosen, what decides, what
 * operates and what only annotates.
 */
export const FAMILY_ORDER: readonly Family[] = [
  "Fuente",
  "Destino",
  "Cola",
  "Recurso",
  "Depósito",
  "Tabla",
  "Cadena",
  "Grupo",
  "Selección",
  "Decisión",
  "Operación",
  "Anotación",
];

/** The part of a class of the catalogue the list needs. */
export interface CatalogueClassSpec {
  /** The name of the class, as the catalogue declares it. */
  class: string;
  /** What the class does, in the words of the catalogue. */
  title: string;
}

/** One class of the list: the name, the title and the shape to draw. */
export interface CatalogueEntry {
  /** The name of the class, as the catalogue declares it. */
  className: string;
  /** What the class does, in the words of the catalogue. */
  title: string;
  /** The family of the class, or `null` when the class has no shape yet. */
  family: Family | null;
  /** The silhouette of the class, or `null` when it has none yet. */
  shape: BlockShape | null;
  /** The three colour levels of the family, or `null` without a family. */
  colors: FamilyColors | null;
}

/** A group of the list: the classes of one family. */
export interface CatalogueGroup {
  /** The family of the group, or `null` for the classes with no shape. */
  family: Family | null;
  /** The three colour levels of the family, or `null` without a family. */
  colors: FamilyColors | null;
  /** The classes of the family, in the order of the catalogue. */
  entries: CatalogueEntry[];
}

/**
 * The classes of the catalogue, grouped by family.
 *
 * The families come in the order of the flow (`FAMILY_ORDER`), and the
 * classes of each family keep the order the catalogue gives them. A class
 * the shapes module does not know yet has no family: it goes last, in a
 * group of its own, and the list still shows its name.
 */
export function groupByFamily(
  classes: readonly CatalogueClassSpec[],
): CatalogueGroup[] {
  const groups: CatalogueGroup[] = FAMILY_ORDER.map((family) => ({
    family,
    colors: familyColors(family),
    entries: [],
  }));
  const orphans: CatalogueGroup = { family: null, colors: null, entries: [] };
  for (const classSpec of classes) {
    const shape = shapeOf(classSpec.class);
    const family = shape?.family ?? null;
    const group =
      family === null ? orphans : groups[FAMILY_ORDER.indexOf(family)];
    group.entries.push({
      className: classSpec.class,
      title: classSpec.title,
      family,
      shape: shape ?? null,
      colors: family === null ? null : familyColors(family),
    });
  }
  // A family with no class does not get a header: the list shows what the
  // catalogue has, not what it could have.
  const listed = groups.filter((group) => group.entries.length > 0);
  if (orphans.entries.length > 0) {
    listed.push(orphans);
  }
  return listed;
}

/**
 * The box the thumbnail of a class is drawn in.
 *
 * The silhouette is 56×28 pixels: big enough to be recognised at a glance,
 * which is the whole point of the thumbnail (S3).
 */
export const THUMBNAIL_BOX = { width: 56, height: 28 } as const;

/**
 * The room the border of the silhouette needs inside the box.
 *
 * The outline is drawn centred on the line of the shape, so half of it
 * falls outside. Without this room, the border is cut flat on the four
 * sides of the thumbnail.
 */
export const THUMBNAIL_MARGIN = 2;

/** What the list draws for a class: its silhouette, scaled into the box. */
export interface Thumbnail {
  /** The `d` of the silhouette, in the coordinates of the footprint. */
  path: string;
  /** The `d` of the interior strokes, drawn without fill. */
  details: string[];
  /** The scale of the footprint inside the box. */
  scale: number;
  /** The width of the silhouette once scaled. */
  width: number;
  /** The height of the silhouette once scaled. */
  height: number;
  /** How far the silhouette is from the left border of the box. */
  offsetX: number;
  /** How far the silhouette is from the top border of the box. */
  offsetY: number;
  /** The tone of the outline: the `borde` of the family. */
  stroke: string;
}

/**
 * The thumbnail of a class: its silhouette, scaled with its own proportion.
 *
 * The scale keeps the proportion of the block (S7), so a wide class is small
 * in height and a square one fills the box, and the silhouette is centred
 * in it. It is the very path the canvas draws, so the list cannot lie about
 * the shape of a class (S6).
 *
 * A class the shapes module does not know has no thumbnail: `null`.
 */
export function thumbnailOf(className: string): Thumbnail | null {
  const shape = shapeOf(className);
  if (shape === undefined) {
    return null;
  }
  const { width, height } = shape.footprint;
  // The border of the silhouette is drawn centred on its own line, so the
  // half that falls outside the shape needs room: without it, the outline
  // is cut flat on the four sides of the box.
  const usableWidth = THUMBNAIL_BOX.width - 2 * THUMBNAIL_MARGIN;
  const usableHeight = THUMBNAIL_BOX.height - 2 * THUMBNAIL_MARGIN;
  const scale = Math.min(usableWidth / width, usableHeight / height);
  const scaledWidth = width * scale;
  const scaledHeight = height * scale;
  return {
    path: shape.path,
    details: shape.details,
    scale,
    width: scaledWidth,
    height: scaledHeight,
    offsetX: THUMBNAIL_MARGIN + (usableWidth - scaledWidth) / 2,
    offsetY: THUMBNAIL_MARGIN + (usableHeight - scaledHeight) / 2,
    stroke: familyColors(shape.family).borde,
  };
}

/**
 * The width of the panel that holds the list.
 *
 * The panel is 400 px wide (`src/components/MonitorView.tsx`); the list
 * takes all of it.
 */
export const PANEL_WIDTH = 400;

/**
 * The horizontal padding of a family group (`px-0`).
 *
 * The panel already gives the list its air: the group adds none, and the
 * room it leaves goes to the text of the cells.
 */
export const GROUP_PADDING = 0;

/** The gap between the two columns of a group (`gap-x-2`). */
export const COLUMN_GAP = 8;

/** The horizontal padding of a cell of the list (`px-1`). */
export const CELL_PADDING = 4;

/** The gap between the thumbnail of a cell and its text (`gap-1.5`). */
export const CELL_GAP = 6;

/**
 * The width of the text of a cell of the list.
 *
 * It is the width the design measures: the panel of 400 px, in two columns,
 * leaves 126 px of text beside the 56 px of the thumbnail.
 *
 * The measurement does not count the padding of the body of the panel
 * (16 px per side): the list does not depend on it, and the panel's owner
 * may trim it. With it, the text of the cell is 110 px wide — wider than
 * the 104 px it had when the thumbnail was 40 px wide, so the bigger
 * thumbnail costs the text nothing.
 */
export const CELL_TEXT_WIDTH =
  (PANEL_WIDTH - 2 * GROUP_PADDING - COLUMN_GAP) / 2 -
  2 * CELL_PADDING -
  CELL_GAP -
  THUMBNAIL_BOX.width;

/** The size of the name of the class, in pixels. */
export const NAME_SIZE = 11;

/** The size of the title of the class, in pixels. */
export const TITLE_SIZE = 9;

/** The width of a character of the monospaced text, in em. */
export const MONOSPACE_EM = 0.6;

/**
 * The width of the characters of the proportional text, in em.
 *
 * It is the measure the design uses to cut the titles, so the list of the
 * app and the drawing of the design cut the same words at the same place.
 */
const CHAR_EM: Record<string, number> = {
  " ": 0.28,
  ".": 0.29,
  ",": 0.29,
  ":": 0.29,
  ";": 0.29,
  "'": 0.22,
  i: 0.28,
  j: 0.28,
  l: 0.28,
  t: 0.36,
  f: 0.34,
  r: 0.38,
  I: 0.3,
  "0": 0.55,
  "1": 0.55,
  "2": 0.55,
  "3": 0.55,
  "4": 0.55,
  "5": 0.55,
  "6": 0.55,
  "7": 0.55,
  "8": 0.55,
  "9": 0.55,
  m: 0.88,
  w: 0.74,
  M: 0.88,
  W: 0.94,
  "/": 0.4,
  "%": 0.86,
  "-": 0.36,
};

/** The width of an ordinary letter, in em. */
const DEFAULT_EM = 0.55;

/** The width of a capital letter, in em. */
const UPPER_EM = 0.68;

/**
 * The width of a text in the proportional typography of the interface.
 *
 * It is an estimate of the real width, good enough to know whether the text
 * fits in the cell: the browser cuts the rest.
 */
export function textWidth(text: string, size: number): number {
  let total = 0;
  for (const character of text) {
    const known = CHAR_EM[character];
    if (known !== undefined) {
      total += known;
    } else if (character !== character.toLowerCase()) {
      total += UPPER_EM;
    } else {
      total += DEFAULT_EM;
    }
  }
  return total * size;
}

/**
 * The width of a text in the monospaced typography of the interface.
 *
 * Every character of the monospaced text is the same: six tenths of its
 * size.
 */
export function monospaceWidth(text: string, size: number): number {
  return text.length * MONOSPACE_EM * size;
}

/**
 * The text that fits in a width, cut with an ellipsis when it does not.
 *
 * The cut never breaks the words: it stops where the text stops fitting.
 */
export function truncate(
  text: string,
  maxWidth: number,
  size: number,
  monospace = false,
): string {
  const width = (part: string): number =>
    monospace ? monospaceWidth(part, size) : textWidth(part, size);
  if (maxWidth <= 0 || width(text) <= maxWidth) {
    return text;
  }
  const ellipsis = "…";
  const available = maxWidth - width(ellipsis);
  let cut = "";
  for (const character of text) {
    if (width(cut + character) > available) {
      break;
    }
    cut += character;
  }
  return cut.trimEnd() + ellipsis;
}

/** The name of a class, cut to the width of the cell. */
export function fitName(className: string): string {
  return truncate(className, CELL_TEXT_WIDTH, NAME_SIZE, true);
}

/** The title of a class, cut to the width of the cell. */
export function fitTitle(title: string): string {
  return truncate(title, CELL_TEXT_WIDTH, TITLE_SIZE);
}
