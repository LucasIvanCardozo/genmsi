//! The list of the catalogue: the families, the thumbnail and the text.

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import {
  CELL_GAP,
  CELL_PADDING,
  CELL_TEXT_WIDTH,
  COLUMN_GAP,
  FAMILY_ORDER,
  GROUP_PADDING,
  NAME_SIZE,
  PANEL_WIDTH,
  THUMBNAIL_BOX,
  THUMBNAIL_MARGIN,
  fitName,
  fitTitle,
  groupByFamily,
  monospaceWidth,
  thumbnailOf,
  type CatalogueClassSpec,
} from "../src/graph/catalogue.ts";
import { BLOCK_SHAPES, FAMILY_COLORS } from "../src/graph/shapes.ts";

/** The classes and the titles the catalogue of the engine declares. */
function catalogueClasses(): CatalogueClassSpec[] {
  const url = new URL("../../genmsi-format/src/catalogue.rs", import.meta.url);
  const source = readFileSync(url, "utf8");
  const pairs = [...source.matchAll(/class: "([^"]+)",\n\s*title: "([^"]+)"/g)];
  return pairs.map((match) => ({ class: match[1], title: match[2] }));
}

/** The order of the flow the families of the list follow. */
const FLOW_ORDER = [
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

/** The layout of a cell of the list, in pixels. */
interface CellLayout {
  /** The width of the panel that holds the list. */
  panel: number;
  /** The horizontal padding of the body of the panel (`p-4`). */
  panelPadding: number;
  /** The horizontal padding of a family group. */
  groupPadding: number;
  /** The gap between the two columns of a group. */
  columnGap: number;
  /** The horizontal padding of a cell. */
  cellPadding: number;
  /** The gap between the thumbnail of a cell and its text. */
  cellGap: number;
  /** The width of the thumbnail. */
  thumbnail: number;
}

/** The width of a cell of the list, with the layout given. */
function cellWidthOf(layout: CellLayout): number {
  return (
    (layout.panel -
      2 * layout.panelPadding -
      2 * layout.groupPadding -
      layout.columnGap) /
    2
  );
}

/** The width of the text of a cell of the list, with the layout given. */
function textAreaOf(layout: CellLayout): number {
  return (
    cellWidthOf(layout) -
    2 * layout.cellPadding -
    layout.cellGap -
    layout.thumbnail
  );
}

/** The layout the list had with the thumbnail of 40×20. */
const PREVIOUS_LAYOUT: CellLayout = {
  panel: 400,
  panelPadding: 16,
  groupPadding: 14,
  columnGap: 12,
  cellPadding: 6,
  cellGap: 8,
  thumbnail: 40,
};

/** The layout the list has with the thumbnail of 56×28. */
const CURRENT_LAYOUT: CellLayout = {
  panel: PANEL_WIDTH,
  panelPadding: 16,
  groupPadding: GROUP_PADDING,
  columnGap: COLUMN_GAP,
  cellPadding: CELL_PADDING,
  cellGap: CELL_GAP,
  thumbnail: THUMBNAIL_BOX.width,
};

test("the catalogue declares 66 classes, and the list knows every one", () => {
  const classes = catalogueClasses();
  assert.equal(classes.length, 66);
  const groups = groupByFamily(classes);
  const listed = groups.flatMap((group) => group.entries.map((entry) => entry.className));
  // Every class is in the list exactly once: none is lost and none is twice.
  assert.equal(listed.length, classes.length);
  assert.equal(new Set(listed).size, classes.length);
  for (const classSpec of classes) {
    assert.ok(listed.includes(classSpec.class), `${classSpec.class} is not in the list`);
  }
});

test("the families come in the order of the flow", () => {
  assert.deepEqual([...FAMILY_ORDER], FLOW_ORDER);
  const groups = groupByFamily(catalogueClasses());
  assert.deepEqual(
    groups.map((group) => group.family),
    FLOW_ORDER,
  );
});

test("every class sits in the group of its family", () => {
  const groups = groupByFamily(catalogueClasses());
  for (const group of groups) {
    assert.ok(group.family !== null);
    for (const entry of group.entries) {
      assert.equal(entry.family, group.family);
      assert.equal(BLOCK_SHAPES[entry.className].family, group.family);
      // The entry carries what the list draws: the shape and the colours.
      assert.equal(entry.shape?.family, group.family);
      assert.equal(entry.colors?.acento, FAMILY_COLORS[group.family!].acento);
    }
  }
});

test("the entry of a class keeps the title of the catalogue", () => {
  const groups = groupByFamily([{ class: "Generate", title: "Generar" }]);
  assert.equal(groups[0].entries[0].title, "Generar");
});

test("a class with no shape still shows its name", () => {
  const groups = groupByFamily([{ class: "NoSuchClass", title: "Ninguna" }]);
  assert.equal(groups.length, 1);
  assert.equal(groups[0].family, null);
  assert.equal(groups[0].entries.length, 1);
  assert.equal(groups[0].entries[0].className, "NoSuchClass");
  assert.equal(groups[0].entries[0].shape, null);
  assert.equal(groups[0].entries[0].colors, null);
  // Without a silhouette there is no thumbnail: the name is all there is.
  assert.equal(thumbnailOf("NoSuchClass"), null);
});

test("the thumbnail of a wide class keeps its proportion and fits the box", () => {
  const thumb = thumbnailOf("Generate");
  assert.ok(thumb !== null);
  const footprint = BLOCK_SHAPES.Generate.footprint;
  assert.equal(footprint.width, 120);
  assert.equal(footprint.height, 63);
  // The silhouette fits the box of the cell.
  assert.ok(thumb.width <= THUMBNAIL_BOX.width, `the thumbnail is ${thumb.width} wide`);
  assert.ok(thumb.height <= THUMBNAIL_BOX.height, `the thumbnail is ${thumb.height} tall`);
  // The proportion of the silhouette is the proportion of the block: a wide
  // class is small in height and a square one fills the box.
  assert.ok(
    Math.abs(thumb.width / thumb.height - footprint.width / footprint.height) < 1e-9,
    `${thumb.width}x${thumb.height} is not ${footprint.width}x${footprint.height}`,
  );
  // And it is centred in the box.
  assert.ok(Math.abs(thumb.offsetX - (THUMBNAIL_BOX.width - thumb.width) / 2) < 1e-9);
  assert.ok(Math.abs(thumb.offsetY - (THUMBNAIL_BOX.height - thumb.height) / 2) < 1e-9);
  // The thumbnail draws the very path the canvas draws.
  assert.equal(thumb.path, BLOCK_SHAPES.Generate.path);
  assert.deepEqual(thumb.details, BLOCK_SHAPES.Generate.details);
});

test("the thumbnail of a square class fills the height of the box", () => {
  const thumb = thumbnailOf("Test");
  assert.ok(thumb !== null);
  const footprint = BLOCK_SHAPES.Test.footprint;
  assert.equal(footprint.width, 64);
  assert.equal(footprint.height, 64);
  // The square class fills the box that is left once the border has its
  // room: without that margin the outline is cut flat on the four sides.
  const usable = THUMBNAIL_BOX.height - 2 * THUMBNAIL_MARGIN;
  assert.equal(thumb.height, usable);
  assert.equal(thumb.width, usable);
  assert.equal(thumb.offsetY, THUMBNAIL_MARGIN);
  assert.equal(thumb.offsetX, THUMBNAIL_MARGIN + (THUMBNAIL_BOX.width - 2 * THUMBNAIL_MARGIN - usable) / 2);
});

test("the thumbnail of the list is the box of the helper", () => {
  // The thumbnail grew: 56×28, so the shape is recognised at a glance.
  assert.equal(THUMBNAIL_BOX.width, 56);
  assert.equal(THUMBNAIL_BOX.height, 28);
  const source = readFileSync(
    new URL("../src/components/ClassList.tsx", import.meta.url),
    "utf8",
  );
  // The SVG of the list takes the box of the helper: the two cannot drift.
  assert.ok(source.includes("THUMBNAIL_BOX.width"));
  assert.ok(source.includes("THUMBNAIL_BOX.height"));
  // And it is drawn without fill and without background: only the outline.
  assert.ok(source.includes('fill="none"'));
});

test("the outline of the thumbnail is the light tone of the family", () => {
  const thumb = thumbnailOf("Test");
  assert.ok(thumb !== null);
  assert.equal(thumb.stroke, FAMILY_COLORS["Decisión"].borde);
});

test("the name of every class fits whole in the cell", () => {
  assert.equal(fitName("MultifurcacionP"), "MultifurcacionP");
  for (const classSpec of catalogueClasses()) {
    assert.equal(fitName(classSpec.class), classSpec.class, `${classSpec.class} was cut`);
  }
});

test("the longest title of the catalogue is cut", () => {
  const cut = catalogueClasses()
    .filter((classSpec) => fitTitle(classSpec.title) !== classSpec.title)
    .map((classSpec) => classSpec.class)
    .sort();
  assert.deepEqual(cut, ["MFunctionDist"]);
  // A cut title ends in the ellipsis, never in a broken word.
  for (const classSpec of catalogueClasses()) {
    const title = fitTitle(classSpec.title);
    if (title !== classSpec.title) {
      assert.ok(title.endsWith("…"), `${classSpec.class} was cut without an ellipsis`);
      assert.ok(title.length < classSpec.title.length);
    }
  }
});

test("the entry of the list keeps the hook of the review and the drag", () => {
  const url = new URL("../src/components/ClassList.tsx", import.meta.url);
  const source = readFileSync(url, "utf8");
  // The visual review clicks the entry of a class by this attribute.
  assert.ok(source.includes("data-block-class={entry.className}"));
  // The drag carries the name of the class, and the click creates the block.
  assert.ok(source.includes("application/genmsi-block"));
  assert.ok(source.includes("draggable"));
  assert.ok(source.includes("onAdd(entry.className)"));
  // The list has no search field: the two columns are what makes it browsable.
  assert.equal(source.includes("Buscar"), false);
});

test("the text of the cell is cut to the width of the cell", () => {
  assert.equal(CELL_TEXT_WIDTH, 126);
  // The name is monospaced at 11 px: fifteen characters are 99 px and fit.
  assert.equal(fitName("MultifurcacionP").length, 15);
  // The title is proportional at 9 px, and this one does not fit.
  const title = fitTitle("Definir múltiples funciones de distribución");
  assert.ok(title.endsWith("…"));
  assert.ok(title.length < "Definir múltiples funciones de distribución".length);
});

test("the name of the longest class fits whole and the cut title is still cut", () => {
  // The name of the longest class fits whole, even with the width the
  // browser really gives the text of the cell.
  assert.equal(fitName("MultifurcacionP"), "MultifurcacionP");
  assert.ok(
    monospaceWidth("MultifurcacionP", NAME_SIZE) <= textAreaOf(CURRENT_LAYOUT),
    `the name needs ${monospaceWidth("MultifurcacionP", NAME_SIZE)} px and the cell has ${textAreaOf(CURRENT_LAYOUT)}`,
  );
  // And the title that does not fit is still cut.
  const title = fitTitle("Definir múltiples funciones de distribución");
  assert.ok(title.endsWith("…"));
  assert.notEqual(title, "Definir múltiples funciones de distribución");
});

test("the text of the cell does not lose width when the thumbnail grows", () => {
  // The thumbnail grew 16 px: the cell grew with it, so the text of the
  // cell is not narrower than it was.
  assert.equal(
    cellWidthOf(CURRENT_LAYOUT) - cellWidthOf(PREVIOUS_LAYOUT),
    THUMBNAIL_BOX.width - PREVIOUS_LAYOUT.thumbnail,
  );
  assert.ok(
    textAreaOf(CURRENT_LAYOUT) >= textAreaOf(PREVIOUS_LAYOUT),
    `the text of the cell went from ${textAreaOf(PREVIOUS_LAYOUT)} px to ${textAreaOf(CURRENT_LAYOUT)} px`,
  );
  // The cell itself is the one the design draws: 180 px of the 400 px panel.
  assert.equal(cellWidthOf(CURRENT_LAYOUT), 180);
  assert.equal(cellWidthOf(PREVIOUS_LAYOUT), 164);
});

test("the list draws the paddings the geometry declares", () => {
  const source = readFileSync(
    new URL("../src/components/ClassList.tsx", import.meta.url),
    "utf8",
  );
  // Tailwind draws a unit of 4 px: the classes of the list are the ones
  // the geometry declares.
  assert.ok(source.includes(`px-${GROUP_PADDING / 4}`));
  assert.ok(source.includes(`gap-x-${COLUMN_GAP / 4}`));
  assert.ok(source.includes(`px-${CELL_PADDING / 4}`));
  assert.ok(source.includes(`gap-${CELL_GAP / 4}`));
  // The paddings that the bigger thumbnail demands are gone.
  assert.equal(source.includes("px-3.5"), false);
  assert.equal(source.includes("gap-x-3"), false);
});
