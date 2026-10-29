//! The routes of a scope: the link an entity travels over.

import assert from "node:assert/strict";
import { test } from "node:test";
import { pairKey, routeFor, routesOf } from "../src/graph/graph.ts";
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

/** A link of the model, with the pairs of activity blocks it carries. */
function link(
  id: string,
  container: number,
  from: [number, number],
  to: [number, number],
  hops: Array<[number, number]>,
): ModelBody["edges"][number] {
  return {
    id,
    container,
    from_module: from[0],
    from_ioid: from[1],
    from_port: "Salida1",
    to_module: to[0],
    to_ioid: to[1],
    to_port: "Entrada1",
    label: null,
    hops,
  };
}

/**
 * A model with a container.
 *
 * The block 1 is outside and has two outputs: one enters the container 2,
 * the other goes straight to the block 4. The container has an entry port
 * and an exit port, and it holds the blocks 3 and 5. The links carry the
 * pairs of activity blocks they cross, with the borders already resolved:
 * the link that enters the container carries the pair (1, 3), and so does
 * the internal link that feeds the entry port.
 */
function modelOf(): ModelBody {
  return {
    schema: "genmsi.model/1",
    path: "/tmp/model.gmdl",
    title: "Modelo",
    dirty: false,
    bytes: 0,
    blocks: [
      block(1, "Generate", {
        outputs: [
          { ioid: 1, name: "Salida1", caption: "", label: null },
          { ioid: 2, name: "Salida2", caption: "", label: null },
        ],
      }),
      block(2, "", {
        kind: "C",
        class: "",
        inputs: [{ ioid: 1, name: "Entrada1", caption: "", label: null }],
        outputs: [{ ioid: 2, name: "Salida1", caption: "", label: null }],
      }),
      block(3, "Advance", {
        container: 2,
        inputs: [{ ioid: 1, name: "Entrada1", caption: "", label: null }],
        outputs: [
          { ioid: 2, name: "Salida1", caption: "", label: null },
          { ioid: 3, name: "Salida2", caption: "", label: null },
        ],
      }),
      block(4, "Terminate", {
        inputs: [{ ioid: 1, name: "Entrada1", caption: "", label: null }],
      }),
      block(5, "Queue", {
        container: 2,
        inputs: [{ ioid: 1, name: "Entrada1", caption: "", label: null }],
      }),
    ],
    edges: [
      // The link that enters the container: it carries the pair of the move
      // that crosses the border.
      link("0:1:1->2:1", 0, [1, 1], [2, 1], [[1, 3]]),
      // The other output of the block 1: it goes straight to the block 4.
      link("0:1:2->4:1", 0, [1, 2], [4, 1], [[1, 4]]),
      // The link that leaves the container: it carries the pair (3, 4).
      link("0:2:2->4:1", 0, [2, 2], [4, 1], [[3, 4]]),
      // The internal link that feeds the entry port: the same pair as the
      // link that enters the container.
      link("2:2:1->3:1", 2, [2, 1], [3, 1], [[1, 3]]),
      // The link between the two blocks of the container.
      link("2:3:2->5:1", 2, [3, 2], [5, 1], [[3, 5]]),
      // The internal link that reaches the exit port: the pair (3, 4).
      link("2:3:3->2:2", 2, [3, 3], [2, 2], [[3, 4]]),
    ],
  } as unknown as ModelBody;
}

/** The links of a scope: the ones the scope owns. */
function linksOf(model: ModelBody, scope: number): ModelBody["edges"] {
  return model.edges.filter((edge) => edge.container === scope);
}

test("the pair of a move is keyed by the two blocks it joins", () => {
  assert.equal(pairKey(1, 3), "1->3");
});

test("the move that crosses the border travels over the edge of the border", () => {
  // In the scope above, the link that enters the container is the one that
  // carries the move: the entity is seen crossing the border.
  const model = modelOf();
  const routes = routesOf(linksOf(model, 0), model.edges);
  assert.equal(routeFor(routes, 1, 3), "0:1:1->2:1");
});

test("the same move travels over the edge of the port inside the container", () => {
  // The scope decides: inside the container, the move is carried by the
  // link that feeds the entry port.
  const model = modelOf();
  const routes = routesOf(linksOf(model, 2), model.edges);
  assert.equal(routeFor(routes, 1, 3), "2:2:1->3:1");
});

test("the move that leaves the container travels over the edge of the border", () => {
  const model = modelOf();
  const root = routesOf(linksOf(model, 0), model.edges);
  assert.equal(routeFor(root, 3, 4), "0:2:2->4:1");
  const inside = routesOf(linksOf(model, 2), model.edges);
  assert.equal(routeFor(inside, 3, 4), "2:3:3->2:2");
});

test("a move between two blocks of the container travels over nothing in the scope above", () => {
  // The move happens deeper inside the model: the scope above has no link
  // that carries it, and there is nothing to draw the entity on.
  const model = modelOf();
  const root = routesOf(linksOf(model, 0), model.edges);
  assert.equal(routeFor(root, 3, 5), undefined);
  // Inside the container, the same move travels over its own link.
  const inside = routesOf(linksOf(model, 2), model.edges);
  assert.equal(routeFor(inside, 3, 5), "2:3:2->5:1");
});

test("a move that skipped the block it went through travels over the first hop of the chain", () => {
  // The trace says the entity was counted in the block 1 and went to the
  // block 5, but the flow went through the block 3. No link carries the
  // pair (1, 5): the links of the chain carry it, and the entity travels
  // over the first hop of the chain.
  const model = modelOf();
  const root = routesOf(linksOf(model, 0), model.edges);
  assert.equal(routeFor(root, 1, 5), "0:1:1->2:1");
  const inside = routesOf(linksOf(model, 2), model.edges);
  assert.equal(routeFor(inside, 1, 5), "2:2:1->3:1");
});

test("a move the model carries is not routed by a chain", () => {
  // The pair (1, 4) is the move of the second output of the block 1: a
  // link carries it, so it is a move of the flow and not a chain. The
  // scope that does not carry it has no link for it, and the app does not
  // invent one.
  const model = modelOf();
  const root = routesOf(linksOf(model, 0), model.edges);
  assert.equal(routeFor(root, 1, 4), "0:1:2->4:1");
  const inside = routesOf(linksOf(model, 2), model.edges);
  assert.equal(routeFor(inside, 1, 4), undefined);
});

test("the routes of a scope are the links the scope owns", () => {
  const model = modelOf();
  const scope = linksOf(model, 2);
  const routes = routesOf(scope, model.edges);
  const owned = new Set(scope.map((edge) => edge.id));
  for (const id of routes.byPair.values()) {
    assert.ok(owned.has(id), `${id} belongs to the scope`);
  }
  for (const id of routes.byChain.values()) {
    assert.ok(owned.has(id), `${id} belongs to the scope`);
  }
});

test("a scope with no links carries no move", () => {
  const model = modelOf();
  const routes = routesOf([], model.edges);
  assert.equal(routeFor(routes, 1, 3), undefined);
});
