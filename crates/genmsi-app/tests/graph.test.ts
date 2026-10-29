//! The geometry of the model's graph.

import assert from "node:assert/strict";
import { test } from "node:test";
import { Position } from "@xyflow/react";
import {
  bezierPath,
  bezierPoints,
  edgeBetween,
  graphOf,
  libraryPath,
  pointOnBezier,
  portSpread,
} from "../src/graph/graph.ts";
import type { ModelBody } from "../src/types/index.ts";

/** A block of the model, as the engine publishes it. */
function block(
  id: number,
  name: string,
  extra: Partial<ModelBody["blocks"][number]> = {},
): ModelBody["blocks"][number] {
  return {
    id,
    class: name,
    kind: "A",
    name,
    caption: "",
    label: null,
    container: 0,
    layout: { x: 10 * id, y: 20 * id, width: 100, height: 40 },
    properties: [],
    inputs: [],
    outputs: [],
    ...extra,
  };
}

/** A model with two blocks and the link that joins them. */
function modelOf(blocks: ModelBody["blocks"], edges: ModelBody["edges"] = []): ModelBody {
  return {
    schema: "genmsi.model/1",
    path: "/tmp/model.gmdl",
    title: "Modelo",
    dirty: false,
    bytes: 0,
    blocks,
    edges,
  } as unknown as ModelBody;
}

test("a block of the model becomes a node of the picture", () => {
  const model = modelOf([
    block(1, "Queue", {
      caption: "Cola de espera",
      label: "cola",
      inputs: [{ ioid: 1, name: "Entrada1", caption: "", label: null }],
      outputs: [{ ioid: 2, name: "Salida1", caption: "", label: null }],
    }),
  ]);
  const { nodes } = graphOf(model);
  assert.equal(nodes.length, 1);
  const node = nodes[0];
  assert.equal(node.id, "1");
  assert.equal(node.type, "bloque");
  assert.deepEqual(node.position, { x: 10, y: 20 });
  assert.equal(node.data.title, "Queue");
  // The caption is what the block says; the label is only a fallback.
  assert.equal(node.data.subtitle, "Cola de espera");
  assert.deepEqual(node.data.inputs, [{ ioid: 1, name: "Entrada1" }]);
  assert.deepEqual(node.data.outputs, [{ ioid: 2, name: "Salida1" }]);
});

test("a block without a caption shows its label", () => {
  const model = modelOf([block(1, "Queue", { label: "cola" })]);
  const { nodes } = graphOf(model);
  assert.equal(nodes[0].data.subtitle, "cola");
});

test("a container is a node of its own", () => {
  const model = modelOf([block(1, "", { kind: "C", class: "" })]);
  const { nodes } = graphOf(model);
  assert.equal(nodes[0].data.title, "Contenedor");
  assert.equal(nodes[0].data.kind, "C");
  assert.equal(nodes[0].zIndex, 0);
});

test("a container is the frame around what it holds, and it does not move", () => {
  const model = modelOf([
    block(1, "", { kind: "C", class: "" }),
    block(2, "Queue"),
  ]);
  const { nodes } = graphOf(model);
  const container = nodes.find((node) => node.id === "1");
  const activity = nodes.find((node) => node.id === "2");
  // The frame is behind what it holds, and it is not a thing the reader
  // drags: the blocks inside it are.
  assert.equal(container?.zIndex, 0);
  assert.equal(container?.draggable, false);
  assert.equal(activity?.draggable, true);
});

test("a block the model does not place is not a node", () => {
  const model = modelOf([block(1, "Queue", { layout: null })]);
  assert.equal(graphOf(model).nodes.length, 0);
});

test("a link between two placed blocks becomes an edge", () => {
  const model = modelOf(
    [block(1, "Queue"), block(2, "Advance")],
    [
      {
        id: "0:1:1->2:1",
        container: 0,
        from_module: 1,
        from_ioid: 1,
        from_port: "Salida1",
        to_module: 2,
        to_ioid: 1,
        to_port: "Entrada1",
        label: null,
      },
    ],
  );
  const { edges } = graphOf(model);
  assert.equal(edges.length, 1);
  assert.equal(edges[0].id, "0:1:1->2:1");
  assert.equal(edges[0].source, "1");
  assert.equal(edges[0].target, "2");
  // A plain link leaves by the port it names.
  assert.equal(edges[0].sourceHandle, "1");
  assert.equal(edges[0].targetHandle, "1");
});

test("a link to a block the model does not place is not an edge", () => {
  const model = modelOf(
    [block(1, "Queue"), block(2, "Advance", { layout: null })],
    [
      {
        id: "0:1:1->2:1",
        container: 0,
        from_module: 1,
        from_ioid: 1,
        from_port: "Salida1",
        to_module: 2,
        to_ioid: 1,
        to_port: "Entrada1",
        label: null,
      },
    ],
  );
  assert.equal(graphOf(model).edges.length, 0);
});

test("the link of a container leaves by the container's port", () => {
  const model = modelOf(
    [block(1, "Queue", { kind: "C" }), block(2, "Advance")],
    [
      {
        id: "0:1:1->2:1",
        container: 1,
        from_module: 1,
        from_ioid: 7,
        from_port: "Salida1",
        to_module: 2,
        to_ioid: 1,
        to_port: "Entrada1",
        label: null,
      },
    ],
  );
  const { edges } = graphOf(model);
  // A container is entered and left through its own ports.
  assert.equal(edges[0].sourceHandle, "in-7");
  assert.equal(edges[0].targetHandle, "1");
});

test("the edge that joins two blocks is found", () => {
  const model = modelOf(
    [block(1, "Queue"), block(2, "Advance")],
    [
      {
        id: "0:1:1->2:1",
        container: 0,
        from_module: 1,
        from_ioid: 1,
        from_port: "Salida1",
        to_module: 2,
        to_ioid: 1,
        to_port: "Entrada1",
        label: null,
      },
    ],
  );
  const { edges } = graphOf(model);
  const found = edgeBetween(edges, 1, 2);
  assert.equal(found?.id, "0:1:1->2:1");
  assert.equal(edgeBetween(edges, 2, 1), undefined, "the link only goes one way");
  assert.equal(edgeBetween(edges, 1, 3), undefined, "there is no such link");
});

test("the ports of a block are spread along its side", () => {
  assert.equal(portSpread(3, 0), 0.25);
  assert.equal(portSpread(3, 1), 0.5);
  assert.equal(portSpread(3, 2), 0.75);
  // A side without ports has no place for one, and the middle is where it
  // would sit.
  assert.equal(portSpread(0, 0), 0.5);
});

test("the curve of an edge is the one React Flow draws", () => {
  const source = { x: 100, y: 40 };
  const target = { x: 300, y: 160 };
  const points = bezierPoints(source, target, Position.Bottom, Position.Top);
  // The very same string the library builds: the line the reader sees and
  // the line a ball travels over are one line.
  assert.equal(bezierPath(points), libraryPath(source, target, Position.Bottom, Position.Top));
});

test("the curve bends, so its middle is not the middle of the straight line", () => {
  const source = { x: 0, y: 0 };
  const target = { x: 200, y: 200 };
  const points = bezierPoints(source, target, Position.Bottom, Position.Top);
  const middle = pointOnBezier(points, 0.5);
  const straight = { x: 100, y: 100 };
  assert.notEqual(middle.x, straight.x);
  assert.notEqual(middle.y, straight.y);
});

test("a ball at the ends of the curve is at the ends", () => {
  const source = { x: 10, y: 20 };
  const target = { x: 210, y: 120 };
  const points = bezierPoints(source, target, Position.Bottom, Position.Top);
  const start = pointOnBezier(points, 0);
  const end = pointOnBezier(points, 1);
  assert.equal(start.x, source.x);
  assert.equal(start.y, source.y);
  assert.equal(end.x, target.x);
  assert.equal(end.y, target.y);
});

test("a ball out of the curve is held at its end", () => {
  const source = { x: 0, y: 0 };
  const target = { x: 100, y: 100 };
  const points = bezierPoints(source, target, Position.Bottom, Position.Top);
  assert.deepEqual(pointOnBezier(points, -3), { x: 0, y: 0 });
  assert.deepEqual(pointOnBezier(points, 7), { x: 100, y: 100 });
});

test("a ball travels the curve at the same speed all along", () => {
  // The fraction is of the length of the curve, so the stretches of the
  // curve of the same length carry the ball the same distance.
  const source = { x: 0, y: 0 };
  const target = { x: 400, y: 0 };
  const points = bezierPoints(source, target, Position.Bottom, Position.Top);
  const quarter = pointOnBezier(points, 0.25);
  const half = pointOnBezier(points, 0.5);
  const threeQuarters = pointOnBezier(points, 0.75);
  const first = half.x - quarter.x;
  const second = threeQuarters.x - half.x;
  // The curve is not a straight line here, so the stretches are close but
  // not identical; what matters is that the ball does not stop and does not
  // jump.
  assert.ok(Math.abs(first - second) < first * 0.2, `${first} against ${second}`);
  assert.ok(quarter.x < half.x && half.x < threeQuarters.x);
});

test("graphOf scopes nodes and edges by container", () => {
  const model = modelOf(
    [
      // Root level: container 10 and block 1
      block(10, "", {
        kind: "C",
        class: "",
        container: 0,
        inputs: [{ ioid: 1, name: "Entrada1", caption: "", label: null }],
        outputs: [{ ioid: 1, name: "Salida1", caption: "", label: null }],
      }),
      block(1, "Queue", { container: 0 }),
      // Inside container 10: blocks 2 and 3
      block(2, "Generate", { container: 10 }),
      block(3, "Advance", { container: 10 }),
    ],
    [
      // Edge in root: 1 -> 10
      {
        id: "0:1:1->10:1",
        container: 0,
        from_module: 1,
        from_ioid: 1,
        from_port: "Salida1",
        to_module: 10,
        to_ioid: 1,
        to_port: "Entrada1",
        label: null,
      },
      // Edge inside container 10: 2 -> 3
      {
        id: "10:2:1->3:1",
        container: 10,
        from_module: 2,
        from_ioid: 1,
        from_port: "Salida1",
        to_module: 3,
        to_ioid: 1,
        to_port: "Entrada1",
        label: null,
      },
    ],
  );

  // Root view: containerId = 0
  const root = graphOf(model, 0);
  assert.equal(root.nodes.length, 2, "root shows 2 nodes (container 10 and block 1)");
  assert.ok(root.nodes.some((n) => n.id === "10"));
  assert.ok(root.nodes.some((n) => n.id === "1"));
  assert.equal(root.edges.length, 1);
  assert.equal(root.edges[0].id, "0:1:1->10:1");

  // Sub-container view: containerId = 10
  const sub = graphOf(model, 10);
  assert.equal(sub.nodes.length, 4, "sub-container shows blocks 2, 3 and boundary ports in-1, out-1");
  assert.ok(sub.nodes.some((n) => n.id === "2"));
  assert.ok(sub.nodes.some((n) => n.id === "3"));
  assert.ok(sub.nodes.some((n) => n.id === "in-1"));
  assert.ok(sub.nodes.some((n) => n.id === "out-1"));
  assert.equal(sub.edges.length, 1);
  assert.equal(sub.edges[0].id, "10:2:1->3:1");
});
