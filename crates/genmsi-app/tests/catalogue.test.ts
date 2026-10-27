//! The list of the catalogue: the families, the thumbnail and the text.

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import {
  CELL_TEXT_WIDTH,
  FAMILY_ORDER,
  fitName,
  fitTitle,
  groupByFamily,
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
  assert.ok(thumb.width <= 40, `the thumbnail is ${thumb.width} wide`);
  assert.ok(thumb.height <= 20, `the thumbnail is ${thumb.height} tall`);
  // The proportion of the silhouette is the proportion of the block: a wide
  // class is small in height and a square one fills the box.
  assert.ok(
    Math.abs(thumb.width / thumb.height - footprint.width / footprint.height) < 1e-9,
    `${thumb.width}x${thumb.height} is not ${footprint.width}x${footprint.height}`,
  );
  // And it is centred in the box.
  assert.ok(Math.abs(thumb.offsetX - (40 - thumb.width) / 2) < 1e-9);
  assert.ok(Math.abs(thumb.offsetY - (20 - thumb.height) / 2) < 1e-9);
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
  assert.equal(thumb.height, 20);
  assert.equal(thumb.width, 20);
  assert.equal(thumb.offsetY, 0);
  assert.equal(thumb.offsetX, 10);
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

test("the three long titles of the catalogue are cut", () => {
  const cut = catalogueClasses()
    .filter((classSpec) => fitTitle(classSpec.title) !== classSpec.title)
    .map((classSpec) => classSpec.class)
    .sort();
  assert.deepEqual(cut, ["FunctionDist", "MFunctionDist", "MultifurcacionP"]);
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
  assert.equal(CELL_TEXT_WIDTH, 120);
  // The name is monospaced at 11 px: fifteen characters are 99 px and fit.
  assert.equal(fitName("MultifurcacionP").length, 15);
  // The title is proportional at 9 px, and this one does not fit.
  const title = fitTitle("Definir múltiples funciones de distribución");
  assert.ok(title.endsWith("…"));
  assert.ok(title.length < "Definir múltiples funciones de distribución".length);
});
