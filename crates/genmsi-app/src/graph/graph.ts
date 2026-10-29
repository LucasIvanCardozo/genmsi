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
 * The nodes and the edges of the model's graph.
 *
 * The blocks are placed with the geometry the `.gmdl` carries, so the picture
 * is the one the original editor drew. When containerId is provided, only
 * the blocks and edges belonging to that container scope are returned.
 */
export function graphOf(
  model: ModelBody,
  containerId?: number | null,
): { nodes: Node<BlockData>[]; edges: Edge[] } {
  const isScoped = containerId !== undefined && containerId !== null;
  const parentContainer =
    isScoped && containerId !== 0
      ? model.blocks.find((b) => b.id === containerId) ?? null
      : null;

  const placed = new Map<number, ModelBlock>();
  const scopedBlocks: ModelBlock[] = [];

  for (const block of model.blocks) {
    if (block.layout === null) continue;
    if (isScoped) {
      if (block.container === containerId && block.id !== 0) {
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
    position: { x: block.layout?.x ?? 0, y: block.layout?.y ?? 0 },
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
    for (let index = 0; index < parentContainer.inputs.length; index += 1) {
      const port = parentContainer.inputs[index];
      nodes.push({
        id: `in-${port.ioid}`,
        type: "bloque",
        position: { x: 120 + index * 180, y: 30 },
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

    for (let index = 0; index < parentContainer.outputs.length; index += 1) {
      const port = parentContainer.outputs[index];
      nodes.push({
        id: `out-${port.ioid}`,
        type: "bloque",
        position: { x: 120 + index * 180, y: 420 },
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
  return { byPair, byChain };
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
