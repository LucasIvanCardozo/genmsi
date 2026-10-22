import assert from "node:assert/strict";
import { test } from "node:test";
import { relayout, type LayoutBlock, type LayoutEdge } from "../src/edit/layout.ts";

/** A block of the graph, with the geometry the model carries. */
function block(
  id: number,
  container: number,
  x: number,
  y: number,
  kind = "A",
  cls = "Generate",
): LayoutBlock {
  return {
    id,
    class: cls,
    kind,
    container,
    layout: { x, y, width: 120, height: 48 },
  };
}

/** A link between two blocks of a container. */
function edge(container: number, from: number, to: number): LayoutEdge {
  return { container, from_module: from, to_module: to };
}

test("a chain of blocks is placed in layers", () => {
  const blocks = [block(1, 0, 300, 40), block(2, 0, 100, 40), block(3, 0, 50, 300)];
  const edges = [edge(0, 1, 2), edge(0, 2, 3)];
  const positions = relayout(blocks, edges, 0);
  const byId = new Map(positions.map((position) => [position.id, position]));

  // The first block of the chain is the leftmost one.
  assert.equal(byId.get(1)?.x, 50);
  assert.equal(byId.get(2)?.x, 50 + 120 + 60);
  assert.equal(byId.get(3)?.x, 50 + 2 * (120 + 60));
  // Every block of a layer shares the row.
  assert.equal(byId.get(1)?.y, byId.get(2)?.y);
});

test("the same model always lays out the same way", () => {
  const blocks = [block(1, 0, 300, 40), block(2, 0, 100, 40), block(3, 0, 50, 300)];
  const edges = [edge(0, 1, 2), edge(0, 2, 3)];
  const first = relayout(blocks, edges, 0);
  const second = relayout(blocks, edges, 0);
  assert.deepEqual(first, second);

  // The order of the arrays is not part of the model: the result does not move.
  const shuffled = relayout([...blocks].reverse(), [...edges].reverse(), 0);
  assert.deepEqual(shuffled, first);
});

test("a container grows to hold its blocks", () => {
  const blocks = [
    block(0, 0, 0, 0, "C", "main"),
    block(1, 0, 200, 100, "C"),
    block(2, 1, 220, 120),
    block(3, 1, 220, 200),
  ];
  const edges = [edge(1, 2, 3)];
  const positions = relayout(blocks, edges, 0);
  const container = positions.find((position) => position.id === 1);
  const children = positions.filter((position) => position.id === 2 || position.id === 3);

  assert.ok(container !== undefined, "the container has a position");
  assert.ok(children.length === 2);
  for (const child of children) {
    assert.ok(
      child.x >= container.x && child.y >= container.y,
      `the block ${child.id} is outside the container`,
    );
    assert.ok(
      child.x + child.width <= container.x + container.width &&
        child.y + child.height <= container.y + container.height,
      `the block ${child.id} does not fit in the container`,
    );
  }
});

test("a cycle does not stop the layout", () => {
  const blocks = [block(1, 0, 0, 0), block(2, 0, 200, 0)];
  const edges = [edge(0, 1, 2), edge(0, 2, 1)];
  const positions = relayout(blocks, edges, 0);
  assert.equal(positions.length, 2);
});

test("a block with no geometry is left alone", () => {
  const blocks: LayoutBlock[] = [
    { id: 0, class: "main", kind: "C", container: 0, layout: null },
    block(1, 0, 10, 20),
  ];
  const positions = relayout(blocks, [], 0);
  assert.equal(positions.length, 1);
  assert.equal(positions[0].id, 1);
});
