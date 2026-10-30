/**
 * The geometry of the model's graph.
 *
 * The graph of the model is drawn in two places: as the editor's canvas and
 * as the picture of the replay. Both need the same geometry, so it lives
 * here, in pure functions, and the components only draw it.
 *
 * The module holds three things:
 *
 * - how a block of the model becomes a **node** of the picture, and how a
 *   link becomes an **edge** (`graphOf`);
 * - the **origin** of a scope and the **place of its doors**: a scope is a
 *   canvas of its own, its free spot is measured from its corner, and the
 *   doors of a container are drawn beside the block they join
 *   (`scopeOrigin`, `scopeBlocks`, `doorPlace`, `freeSpotOf`);
 * - which **link** of a scope carries a move of the trace, so the entity
 *   that travels is seen crossing the border of a container (`routesOf`);
 * - where the **ports** of a block sit, so a link leaves the block by the
 *   port it names and not by the middle of the block (`portSpread`);
 * - the **curve** an edge follows, and the point of that curve where a
 *   transaction is at a given moment (`bezierPoints`, `pointOnBezier`).
 *
 * The curve is the one React Flow draws: the same control points, the same
 * path. A ball that travels over it must sit exactly on the line the reader
 * sees, so the arithmetic here is the library's, and `bezierPath` builds the
 * very same string `getBezierPath` returns.
 */
import { getBezierPath, Position, type Edge, type Node } from "@xyflow/react";
import { DOOR_BOX } from "./frames.ts";
import type { ModelBody, ModelBlock, ModelEdge } from "../types";

/**
 * The kind of a block of the picture.
 *
 * `A` is an activity block, `C` a container, and `PortIn` and `PortOut`
 * the two doors of a container: the kinds the model does not have, which
 * the canvas draws as blocks of their own.
 */
export type BlockKind = "A" | "C" | "PortIn" | "PortOut";

/** The size the model gives a block, in the pixels of the model. */
export interface BlockSize {
  /** The width of the block. */
  width: number;
  /** The height of the block. */
  height: number;
}

/** One port of a block, as the node draws it. */
export interface PortData {
  /** Port id, unique inside the block. */
  ioid: number;
  /** Port name (`Entrada1`, `Salida1`, ...). */
  name: string;
}

/** What a block of the graph shows. */
export interface BlockData extends Record<string, unknown> {
  /** The block's class (`Generate`, `Queue`, ...). */
  title: string;
  /** The block's caption, or its label when it has no caption. */
  subtitle: string;
  /** The kind of the block: `A`, `C`, `PortIn` or `PortOut`. */
  kind: BlockKind;
  /** The block's input ports. */
  inputs: PortData[];
  /** The block's output ports. */
  outputs: PortData[];
  /**
   * The size the model gives the block, or `null` when the model gives
   * none.
   *
   * The size is the truth for the container and for a block of a class the
   * catalogue does not know: the canvas draws them at the size the model
   * carries. The doors have no size in the model: they are drawn at the
   * size of their own silhouette.
   */
  size: BlockSize | null;
}

/** The four points of a cubic curve: the two ends and the two controls. */
export interface BezierPoints {
  /** The source. */
  x0: number;
  y0: number;
  /** The first control point. */
  x1: number;
  y1: number;
  /** The second control point. */
  x2: number;
  y2: number;
  /** The target. */
  x3: number;
  y3: number;
}

/** The title a block shows: its class, or the word for a plain container. */
function titleOf(block: ModelBlock): string {
  if (block.kind === "C") {
    return block.class === "" ? "Contenedor" : block.class;
  }
  return block.class === "" ? "Bloque" : block.class;
}

/** The subtitle a block shows: its caption, or its label. */
function subtitleOf(block: ModelBlock): string {
  if (block.caption !== "") {
    return block.caption;
  }
  return block.label ?? "";
}

/**
 * The kind of a block of the model: an activity block or a container.
 *
 * The model knows two kinds, and the canvas draws two more: the doors of
 * a container are not blocks of the model.
 */
function kindOf(block: ModelBlock): BlockKind {
  return block.kind === "C" ? "C" : "A";
}

/**
 * A point in the pixels of the model.
 *
 * The model places its blocks in the pixels of its own canvas, and the
 * canvas of a scope draws them from the corner of that scope: a point of
 * the model and a point of the canvas are two ways of saying where a
 * block is.
 */
export interface Point {
  /** The horizontal position, in pixels. */
  x: number;
  /** The vertical position, in pixels. */
  y: number;
}

/**
 * The blocks a scope holds, in the order of the model.
 *
 * A scope is the level the reader is in: the root of the model (0) or a
 * container. The blocks it holds are the ones that name it as their
 * container, and only the ones the model places: a block without a place
 * is not drawn.
 */
export function scopeBlocks(model: ModelBody, scope: number): ModelBlock[] {
  return model.blocks.filter(
    (block) => block.container === scope && block.id !== 0 && block.layout !== null,
  );
}

/**
 * The origin of a scope: the corner of the blocks the scope holds.
 *
 * A scope is a canvas of its own, and its picture starts where the author
 * put its blocks: the rule is the one the composition of the engine uses
 * (`scope_origin` in `crates/genmsi-format/src/layout.rs`).
 *
 * The origin is the frame the free spot of the scope is measured in
 * (`freeSpotOf`): the spot is under the lowest block of the scope, and the
 * position of the model is that spot plus this corner. The drawing of the
 * picture is **not** translated by it: the blocks are drawn where the model
 * places them, so a drag of a block moves that block and nothing else.
 *
 * A scope with no block of its own has no corner: its origin is the
 * origin of the canvas, (0, 0).
 */
export function scopeOrigin(model: ModelBody, scope: number): Point {
  let x = Number.POSITIVE_INFINITY;
  let y = Number.POSITIVE_INFINITY;
  for (const block of scopeBlocks(model, scope)) {
    const layout = block.layout;
    if (layout === null) {
      continue;
    }
    x = Math.min(x, layout.x);
    y = Math.min(y, layout.y);
  }
  if (!Number.isFinite(x) || !Number.isFinite(y)) {
    return { x: 0, y: 0 };
  }
  return { x, y };
}

/**
 * The air between two doors that have no place yet, in pixels.
 *
 * Two doors never share a place: the reader has to be able to tell them
 * apart.
 */
export const DOOR_GAP = 8;

/**
 * The place of a door of a container.
 *
 * The model carries it: the composition of the scope gave the door its
 * place, and the reader may have dragged it since. A door that has no
 * place yet —a new one— goes to the free spot of the scope, under the
 * doors that are already there: two doors never share a place.
 */
export function doorPlace(
  model: ModelBody,
  scope: number,
  kind: "PortIn" | "PortOut",
  ioid: number,
): Point {
  const container = model.blocks.find((block) => block.id === scope);
  const inputs = container?.inputs ?? [];
  const outputs = container?.outputs ?? [];
  const port = (kind === "PortIn" ? inputs : outputs).find(
    (candidate) => candidate.ioid === ioid,
  );
  if (port?.layout != null) {
    return { x: port.layout.x, y: port.layout.y };
  }
  // The door has no place yet: it goes to the free spot of the scope. The
  // spot does not depend on the other doors, so moving one door never moves
  // the others.
  const base = freeSpotOf(model, scope).position;
  // And under the doors that already took that place: two doors never
  // share a place.
  const withoutPlace = [
    ...inputs.map((candidate) => ({ kind: "PortIn" as const, port: candidate })),
    ...outputs.map((candidate) => ({ kind: "PortOut" as const, port: candidate })),
  ].filter((door) => door.port.layout == null);
  const index = withoutPlace.findIndex(
    (door) => door.kind === kind && door.port.ioid === ioid,
  );
  return {
    x: base.x,
    y: base.y + index * (DOOR_BOX.height + DOOR_GAP),
  };
}


/**
 * The free spot of a scope: where a new block goes.
 */
export interface FreeSpot {
  /**
   * The spot in the scope's own space: under the lowest block the scope
   * holds, at the left of its picture.
   */
  spot: Point;
  /**
   * The position of the block in the model: the spot plus the origin of the
   * scope, because the model keeps the absolute coordinates.
   */
  position: Point;
}

/**
 * The free spot of a scope, in the scope's own space and in the model.
 *
 * The spot is computed in the space of the scope —under the lowest block
 * the scope holds— and the position of the model is that spot plus the
 * origin of the scope. A block added inside a container therefore lands
 * inside the picture the reader is looking at, and not far away from it.
 *
 * A scope with no block of its own has its spot at (80, 80): the picture
 * starts there.
 */
export function freeSpotOf(model: ModelBody, scope: number): FreeSpot {
  const origin = scopeOrigin(model, scope);
  const blocks = scopeBlocks(model, scope);
  if (blocks.length === 0) {
    const spot = { x: 80, y: 80 };
    return { spot, position: { x: spot.x + origin.x, y: spot.y + origin.y } };
  }
  let left = Number.POSITIVE_INFINITY;
  let bottom = Number.NEGATIVE_INFINITY;
  for (const block of blocks) {
    const layout = block.layout;
    if (layout === null) {
      continue;
    }
    left = Math.min(left, layout.x - origin.x);
    bottom = Math.max(bottom, layout.y - origin.y + layout.height);
  }
  const spot = { x: left, y: bottom + 40 };
  return {
    spot,
    position: { x: spot.x + origin.x, y: spot.y + origin.y },
  };
}

/**
 * The nodes and the edges of the model's graph.
 *
 * The blocks are placed with the geometry the `.gmdl` carries, so the picture
 * is the one the original editor drew. When containerId is provided, only
 * the blocks and edges belonging to that container scope are returned.
 *
 * A scope is a canvas of its own: the picture shows the blocks of the scope
 * and the doors of the container, and the doors are drawn beside the block
 * they join. The blocks are drawn **where the model places them**: the
 * picture is not translated, so the place of a block on the canvas is the
 * place the model gives it, and a drag of a block moves that block and
 * nothing else.
 */
export function graphOf(
  model: ModelBody,
  containerId?: number | null,
): { nodes: Node<BlockData>[]; edges: Edge[] } {
  const isScoped = containerId !== undefined && containerId !== null;
  // The scope of the picture: the root of the model when there is none.
  const scope = containerId ?? 0;
  const parentContainer =
    isScoped && scope !== 0
      ? model.blocks.find((b) => b.id === scope) ?? null
      : null;

  const placed = new Map<number, ModelBlock>();
  const scopedBlocks: ModelBlock[] = [];

  for (const block of model.blocks) {
    if (block.layout === null) continue;
    if (isScoped) {
      if (block.container === scope && block.id !== 0) {
        placed.set(block.id, block);
        scopedBlocks.push(block);
      }
    } else {
      placed.set(block.id, block);
      scopedBlocks.push(block);
    }
  }

  const nodes: Node<BlockData>[] = scopedBlocks.map((block) => ({
    id: String(block.id),
    type: "bloque",
    // The node is drawn where the model places it: the picture is not
    // translated, and the canvas of a scope is the canvas of the model.
    position: {
      x: block.layout?.x ?? 0,
      y: block.layout?.y ?? 0,
    },
    data: {
      title: titleOf(block),
      subtitle: subtitleOf(block),
      kind: kindOf(block),
      inputs: block.inputs.map((port) => ({ ioid: port.ioid, name: port.name })),
      outputs: block.outputs.map((port) => ({ ioid: port.ioid, name: port.name })),
      size:
        block.layout === null
          ? null
          : { width: block.layout.width, height: block.layout.height },
    },
    style: {
      // The container is a block with a box of its own, and that box is the
      // one the model gives it: the model is the truth for a block that has
      // no class to say how big it is (L2, L3).
      width:
        block.kind === "C"
          ? (block.layout?.width ?? 160)
          : Math.max(block.layout?.width ?? 100, 72),
      height:
        block.kind === "C"
          ? (block.layout?.height ?? 56)
          : Math.max(block.layout?.height ?? 40, 32),
    },
    zIndex: block.kind === "C" ? 0 : 2,
    // Blocks and containers at the current level are draggable
    draggable: isScoped ? block.id !== 0 : block.kind !== "C",
  }));

  // When inside a container, project its declared inputs and outputs as boundary port nodes
  if (parentContainer !== null) {
    for (const port of parentContainer.inputs) {
      // The door is drawn where the model puts it: the composition of the
      // scope gave it its place, and the reader may have dragged it.
      const place = doorPlace(model, scope, "PortIn", port.ioid);
      nodes.push({
        id: `in-${port.ioid}`,
        type: "bloque",
        position: {
          x: place.x,
          y: place.y,
        },
        data: {
          title: port.name || `Entrada ${port.ioid}`,
          subtitle: "Entrada de submódulo",
          kind: "PortIn",
          inputs: [],
          outputs: [{ ioid: port.ioid, name: port.name }],
          // The door has no size in the model: it is drawn at the size of
          // its own silhouette.
          size: null,
        },
        style: { width: DOOR_BOX.width, height: DOOR_BOX.height },
        zIndex: 2,
        draggable: true,
      });
    }

    for (const port of parentContainer.outputs) {
      // The exit door is drawn where the model puts it, like the entry one.
      const place = doorPlace(model, scope, "PortOut", port.ioid);
      nodes.push({
        id: `out-${port.ioid}`,
        type: "bloque",
        position: {
          x: place.x,
          y: place.y,
        },
        data: {
          title: port.name || `Salida ${port.ioid}`,
          subtitle: "Salida de submódulo",
          kind: "PortOut",
          inputs: [{ ioid: port.ioid, name: port.name }],
          outputs: [],
          // The door has no size in the model: it is drawn at the size of
          // its own silhouette.
          size: null,
        },
        style: { width: DOOR_BOX.width, height: DOOR_BOX.height },
        zIndex: 2,
        draggable: true,
      });
    }
  }

  const edges: Edge[] = model.edges
    .filter((edge) => {
      if (isScoped && edge.container !== containerId) {
        return false;
      }
      const fromOk =
        isScoped && containerId !== 0 && edge.from_module === containerId
          ? parentContainer?.inputs.some((p) => p.ioid === edge.from_ioid) ?? false
          : placed.has(edge.from_module);
      const toOk =
        isScoped && containerId !== 0 && edge.to_module === containerId
          ? parentContainer?.outputs.some((p) => p.ioid === edge.to_ioid) ?? false
          : placed.has(edge.to_module);
      return fromOk && toOk;
    })
    .map((edge) => {
      const isFromContainer = isScoped && containerId !== 0 && edge.from_module === containerId;
      const isToContainer = isScoped && containerId !== 0 && edge.to_module === containerId;
      const source = isFromContainer ? `in-${edge.from_ioid}` : String(edge.from_module);
      const target = isToContainer ? `out-${edge.to_ioid}` : String(edge.to_module);
      const sourceHandle = isFromContainer
        ? String(edge.from_ioid)
        : edge.from_module === edge.container
          ? `in-${edge.from_ioid}`
          : String(edge.from_ioid);
      const targetHandle = isToContainer
        ? String(edge.to_ioid)
        : edge.to_module === edge.container
          ? `out-${edge.to_ioid}`
          : String(edge.to_ioid);

      return {
        id: edge.id,
        source,
        sourceHandle,
        target,
        targetHandle,
        label: edge.label ?? undefined,
        animated: false,
        style: { stroke: "#94a3b8", strokeWidth: 1.75 },
        labelStyle: { fontSize: 10, fill: "#64748b" },
        labelBgStyle: { fill: "#f8fafc", fillOpacity: 0.9 },
        data: {
          container: edge.container,
          from: edge.from_module,
          fromIoid: edge.from_ioid,
        },
      };
    });

  return { nodes, edges };
}

/**
 * The edge that joins two blocks, if there is one.
 *
 * A transaction that moves from one block to another travels over the link
 * that joins them. When several links join the same pair of blocks, the first
 * one is the one it travels over: the reader sees one curve and the ball
 * stays on it.
 */
export function edgeBetween(edges: Edge[], from: number, to: number): Edge | undefined {
  const source = String(from);
  const target = String(to);
  return edges.find((edge) => edge.source === source && edge.target === target);
}

/**
 * The key of the pair of activity blocks a link carries.
 *
 * The pair is the move the trace records: the block the flow comes from and
 * the block it goes to.
 */
export function pairKey(from: number, to: number): string {
  return `${from}->${to}`;
}

/**
 * The key of a step of the flow: the port of a block the flow is at.
 *
 * A step is where the flow is: at the port of a block, or at the port of a
 * container (which is where the flow crosses its border).
 */
function stepKey(module: number, ioid: number): string {
  return `${module}:${ioid}`;
}

/**
 * The chain of a move: the links the flow travels, in order.
 *
 * The chain is the flow of the engine seen from the links: every link of
 * the model joins two steps, and the links a move travels are the ones that
 * join them in a row.
 *
 * The app does not resolve the borders of the containers: the links **are**
 * the borders, written by the editor. The app only walks them, and it walks
 * them by the rule of the engine: of the links that start at a step, the
 * first one is the one the flow travels.
 */
export interface ChainIndex {
  /** The first link that starts at each step. */
  readonly first: Map<string, ModelEdge>;
  /** The steps of each block that have a link leaving them. */
  readonly starts: Map<number, string[]>;
  /** The chains of the moves, by the pair of blocks they join. */
  readonly chains: Map<string, ModelEdge[] | undefined>;
}

/**
 * The index of the chains of the model.
 *
 * @param model The links of the whole model.
 */
export function chainsOf(model: ModelEdge[]): ChainIndex {
  const first = new Map<string, ModelEdge>();
  const starts = new Map<number, string[]>();
  for (const link of model) {
    const key = stepKey(link.from_module, link.from_ioid);
    // The first link that starts at a step is the one the flow travels: the
    // engine resolves the borders the same way, and the two agree.
    if (!first.has(key)) {
      first.set(key, link);
    }
    const steps = starts.get(link.from_module);
    if (steps === undefined) {
      starts.set(link.from_module, [key]);
    } else if (!steps.includes(key)) {
      steps.push(key);
    }
  }
  return { first, starts, chains: new Map() };
}

/**
 * The chain of a move: the links the flow travels, in order.
 *
 * `undefined` when the flow of the move is not a chain of links: the entity
 * left a block and reached another one without a link in between (the entry
 * door of a container with no link inside it, for instance).
 */
export function chainFor(
  index: ChainIndex,
  from: number,
  to: number,
): ModelEdge[] | undefined {
  const key = pairKey(from, to);
  if (index.chains.has(key)) {
    return index.chains.get(key);
  }
  const chain = walkChain(index, from, to);
  index.chains.set(key, chain);
  return chain;
}

/**
 * The walk of the chain: from a step of the block the entity left to a step
 * of the block it reached.
 */
function walkChain(
  index: ChainIndex,
  from: number,
  to: number,
): ModelEdge[] | undefined {
  for (const start of index.starts.get(from) ?? []) {
    const path = walkFrom(index, start, to);
    if (path !== undefined) {
      return path;
    }
  }
  return undefined;
}

/**
 * The walk from one step, following the links the flow travels.
 */
function walkFrom(
  index: ChainIndex,
  start: string,
  to: number,
): ModelEdge[] | undefined {
  const seen = new Set<string>();
  const stack: Array<{ step: string; path: ModelEdge[] }> = [
    { step: start, path: [] },
  ];
  while (stack.length > 0) {
    const { step, path } = stack.pop() ?? { step: start, path: [] };
    if (seen.has(step)) {
      continue;
    }
    seen.add(step);
    const link = index.first.get(step);
    if (link === undefined) {
      continue;
    }
    const next = path.concat(link);
    if (link.to_module === to) {
      return next;
    }
    stack.push({ step: stepKey(link.to_module, link.to_ioid), path: next });
  }
  return undefined;
}

/**
 * The routes of a scope: the link that carries each move of the trace.
 *
 * The truth is the model's: every link publishes the pairs of activity
 * blocks it carries (`ModelEdge.hops`), with the borders of the containers
 * already resolved by the engine. The routes are that truth seen from one
 * scope: the link of the scope that carries the pair of a move.
 *
 * A move the trace records in the block the transaction was counted in
 * skips the block it really went through, and no link carries its pair.
 * For those, the route is the link that carries the **first hop of the
 * chain** the flow crossed: the chain is already published as the hops of
 * the links, so the app does not resolve the flow of the engine on its own.
 *
 * The chain is only for the pairs that **no link carries**: a pair that
 * some link carries is a move of the flow, and the scope that does not
 * carry it simply has no link for it.
 */
export interface ScopeRoutes {
  /** The link of the scope that carries a pair, by the key of the pair. */
  byPair: Map<string, string>;
  /**
   * The link of the scope that carries the first hop of the chain of a pair
   * that no link carries, by the key of the pair.
   */
  byChain: Map<string, string>;
  /** The chains of the moves of the model. */
  chain: ChainIndex;
  /** The links the scope owns, by their id. */
  owned: Set<string>;
}

/**
 * The routes of a scope.
 *
 * The scope's links are the ones it owns, and they are the ones the canvas
 * draws in it: the link that enters a container belongs to the scope
 * above, and the link that feeds the entry port of the container belongs
 * to the container.
 *
 * @param scope The links the scope draws.
 * @param model The links of the whole model: the chain of a move is read
 *   from them, because the flow of a move crosses the borders.
 */
export function routesOf(scope: ModelEdge[], model: ModelEdge[]): ScopeRoutes {
  const byPair = new Map<string, string>();
  for (const link of scope) {
    for (const [from, to] of link.hops) {
      const key = pairKey(from, to);
      if (!byPair.has(key)) {
        byPair.set(key, link.id);
      }
    }
  }
  // The block every pair reaches, in every scope: the chain of a move is
  // the pair of its first hop and the pair that follows it, and the flow of
  // a move crosses the borders, so the second hop may live in another
  // scope.
  const reaches = new Map<number, number[]>();
  // The pairs the model carries: a pair that some link carries is a move
  // of the flow of the engine, and the chain is not for it.
  const carried = new Set<string>();
  for (const link of model) {
    for (const [from, to] of link.hops) {
      carried.add(pairKey(from, to));
      const list = reaches.get(from);
      if (list === undefined) {
        reaches.set(from, [to]);
      } else if (!list.includes(to)) {
        list.push(to);
      }
    }
  }
  // The chain of a move: the first hop is the one the scope carries, and
  // the second one is the pair that follows it.
  const byChain = new Map<string, string>();
  for (const link of scope) {
    for (const [from, to] of link.hops) {
      for (const next of reaches.get(to) ?? []) {
        const key = pairKey(from, next);
        if (!byPair.has(key) && !carried.has(key) && !byChain.has(key)) {
          byChain.set(key, link.id);
        }
      }
    }
  }
  return { byPair, byChain, chain: chainsOf(model), owned: ownedOf(scope) };
}

/**
 * The links a scope owns, by their id.
 */
function ownedOf(scope: ModelEdge[]): Set<string> {
  return new Set(scope.map((link) => link.id));
}

/**
 * The links of the scope that the entity of a move travels over, in order.
 *
 * A move that crosses the border of a container travels over **several**
 * links: the one that enters the container, and the one that leaves it. The
 * links of the scope are the ones of the picture the reader is looking at,
 * and the entity crosses them one after the other.
 *
 * A move the scope has no link for travels over nothing: it happens deeper
 * inside the model, and there is nothing to draw the entity on.
 */
export function legsOf(
  routes: ScopeRoutes,
  from: number,
  to: number,
): string[] {
  const chain = chainFor(routes.chain, from, to);
  if (chain !== undefined) {
    const legs = chain.filter((link) => routes.owned.has(link.id));
    if (legs.length > 0) {
      return legs.map((link) => link.id);
    }
  }
  // The move is not a chain of links: the link that carries its pair is the
  // one the entity travels over.
  const id = routeFor(routes, from, to);
  return id === undefined ? [] : [id];
}

/**
 * The link a move travels over in a scope.
 *
 * The link that carries the pair of the move, or the link that carries the
 * first hop of the chain of a move that no link carries. `undefined` when
 * the scope has no link for the move: the move happens deeper inside the
 * model, and there is nothing to draw the entity on.
 */
export function routeFor(
  routes: ScopeRoutes,
  from: number,
  to: number,
): string | undefined {
  const key = pairKey(from, to);
  return routes.byPair.get(key) ?? routes.byChain.get(key);
}

/**
 * Where a port sits along the side of its block, as a fraction of its width.
 *
 * The ports of a block are spread along its side, the first one a quarter in,
 * the last one three quarters in. An empty side has no port, and the middle
 * is where a port would sit.
 */
export function portSpread(count: number, index: number): number {
  if (count <= 0) {
    return 0.5;
  }
  return (index + 1) / (count + 1);
}

/**
 * The offset of a control point from its end of the curve.
 *
 * This is the arithmetic React Flow uses (`calculateControlOffset` in
 * `@xyflow/system`): a distance that grows with the gap between the two
 * ends, and a fixed share of the curvature when the ends are on top of each
 * other.
 */
function controlOffset(distance: number, curvature: number): number {
  if (distance >= 0) {
    return 0.5 * distance;
  }
  return curvature * 25 * Math.sqrt(-distance);
}

/**
 * The control point of one end of the curve.
 *
 * The end that leaves downwards bends downwards, the one that arrives from
 * above bends upwards, and the ones that leave sideways bend sideways.
 */
function controlWithCurvature(
  position: Position,
  x1: number,
  y1: number,
  x2: number,
  y2: number,
  curvature: number,
): [number, number] {
  switch (position) {
    case Position.Left:
      return [x1 - controlOffset(x1 - x2, curvature), y1];
    case Position.Right:
      return [x1 + controlOffset(x2 - x1, curvature), y1];
    case Position.Top:
      return [x1, y1 - controlOffset(y1 - y2, curvature)];
    case Position.Bottom:
    default:
      return [x1, y1 + controlOffset(y2 - y1, curvature)];
  }
}

/**
 * The curve an edge follows.
 *
 * The same curve React Flow draws between two handles: the source and the
 * target, and the two control points that bend the line towards each other.
 */
export function bezierPoints(
  source: { x: number; y: number },
  target: { x: number; y: number },
  sourcePosition: Position = Position.Bottom,
  targetPosition: Position = Position.Top,
  curvature = 0.25,
): BezierPoints {
  const [x1, y1] = controlWithCurvature(
    sourcePosition,
    source.x,
    source.y,
    target.x,
    target.y,
    curvature,
  );
  const [x2, y2] = controlWithCurvature(
    targetPosition,
    target.x,
    target.y,
    source.x,
    source.y,
    curvature,
  );
  return { x0: source.x, y0: source.y, x1, y1, x2, y2, x3: target.x, y3: target.y };
}

/**
 * The path of a curve, in the format React Flow draws it.
 *
 * The string is the one `getBezierPath` returns, so the line the reader sees
 * and the line the ball travels over are the same line.
 */
export function bezierPath(points: BezierPoints): string {
  return `M${points.x0},${points.y0} C${points.x1},${points.y1} ${points.x2},${points.y2} ${points.x3},${points.y3}`;
}

/** The point of a cubic curve at its parameter `t`, from 0 to 1. */
function pointAtParameter(points: BezierPoints, t: number): { x: number; y: number } {
  const u = 1 - t;
  const a = u * u * u;
  const b = 3 * u * u * t;
  const c = 3 * u * t * t;
  const d = t * t * t;
  return {
    x: a * points.x0 + b * points.x1 + c * points.x2 + d * points.x3,
    y: a * points.y0 + b * points.y1 + c * points.y2 + d * points.y3,
  };
}

/** How many points of the curve are measured to know its length. */
const SAMPLES = 64;

/** A sampled cubic bezier curve with precalculated arc-length table. */
export interface SampledBezier {
  points: BezierPoints;
  totalLength: number;
  samples: { x: number; y: number; length: number }[];
}

/** Precalculate arc-length samples for a bezier curve. */
export function sampleBezier(points: BezierPoints): SampledBezier {
  const samples: { x: number; y: number; length: number }[] = [];
  let length = 0;
  let previous = pointAtParameter(points, 0);
  for (let index = 1; index <= SAMPLES; index += 1) {
    const current = pointAtParameter(points, index / SAMPLES);
    length += Math.hypot(current.x - previous.x, current.y - previous.y);
    samples.push({ x: current.x, y: current.y, length });
    previous = current;
  }
  return { points, totalLength: length, samples };
}

/**
 * The point of a pre-sampled curve at a fraction of its length.
 */
export function pointOnSampledBezier(curve: SampledBezier, t: number): { x: number; y: number } {
  const fraction = t < 0 ? 0 : t > 1 ? 1 : t;
  if (fraction === 0) {
    return { x: curve.points.x0, y: curve.points.y0 };
  }
  if (fraction === 1) {
    return { x: curve.points.x3, y: curve.points.y3 };
  }
  const wanted = curve.totalLength * fraction;
  let before = { x: curve.points.x0, y: curve.points.y0, length: 0 };
  for (const point of curve.samples) {
    if (point.length >= wanted) {
      const stretch = point.length - before.length;
      const part = stretch <= 0 ? 0 : (wanted - before.length) / stretch;
      return {
        x: before.x + (point.x - before.x) * part,
        y: before.y + (point.y - before.y) * part,
      };
    }
    before = point;
  }
  return { x: curve.points.x3, y: curve.points.y3 };
}

/**
 * The point of a curve at a fraction of its length.
 *
 * The fraction is of the *length* of the curve and not of its parameter, so
 * the thing that travels over it goes at the same speed all along, which is
 * what a reader expects to see.
 */
export function pointOnBezier(points: BezierPoints, t: number): { x: number; y: number } {
  return pointOnSampledBezier(sampleBezier(points), t);
}

/**
 * The curve React Flow draws between two handles, as the library draws it.
 *
 * The module uses its own control points, and this is the proof that they
 * are the library's: the two paths are the same string.
 */
export function libraryPath(
  source: { x: number; y: number },
  target: { x: number; y: number },
  sourcePosition: Position = Position.Bottom,
  targetPosition: Position = Position.Top,
): string {
  return getBezierPath({
    sourceX: source.x,
    sourceY: source.y,
    sourcePosition,
    targetX: target.x,
    targetY: target.y,
    targetPosition,
  })[0];
}
