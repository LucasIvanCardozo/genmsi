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
import type { ModelBody, ModelBlock } from "../types";

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
  /** `A` for an activity block, `C` for a container. */
  kind: string;
  /** The block's input ports. */
  inputs: PortData[];
  /** The block's output ports. */
  outputs: PortData[];
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
 * The nodes and the edges of the model's graph.
 *
 * The blocks are placed with the geometry the `.gmdl` carries, so the picture
 * is the one the original editor drew. The blocks the model does not place
 * are not nodes, and neither are the links that reach them.
 */
export function graphOf(model: ModelBody): { nodes: Node<BlockData>[]; edges: Edge[] } {
  const placed = new Map<number, ModelBlock>();
  for (const block of model.blocks) {
    if (block.layout !== null) {
      placed.set(block.id, block);
    }
  }

  const nodes: Node<BlockData>[] = model.blocks
    .filter((block) => block.layout !== null)
    .map((block) => ({
      id: String(block.id),
      type: "bloque",
      position: { x: block.layout?.x ?? 0, y: block.layout?.y ?? 0 },
      data: {
        title: titleOf(block),
        subtitle: subtitleOf(block),
        kind: block.kind,
        inputs: block.inputs.map((port) => ({ ioid: port.ioid, name: port.name })),
        outputs: block.outputs.map((port) => ({ ioid: port.ioid, name: port.name })),
      },
      style: {
        width: Math.max(block.layout?.width ?? 100, 72),
        height: Math.max(block.layout?.height ?? 40, 32),
      },
      zIndex: block.kind === "C" ? 0 : 2,
    }));

  const edges: Edge[] = model.edges
    .filter((edge) => placed.has(edge.from_module) && placed.has(edge.to_module))
    .map((edge) => ({
      id: edge.id,
      source: String(edge.from_module),
      sourceHandle:
        edge.from_module === edge.container ? `in-${edge.from_ioid}` : String(edge.from_ioid),
      target: String(edge.to_module),
      targetHandle:
        edge.to_module === edge.container ? `out-${edge.to_ioid}` : String(edge.to_ioid),
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
    }));

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

/**
 * The point of a curve at a fraction of its length.
 *
 * The fraction is of the *length* of the curve and not of its parameter, so
 * the thing that travels over it goes at the same speed all along, which is
 * what a reader expects to see.
 */
export function pointOnBezier(points: BezierPoints, t: number): { x: number; y: number } {
  const fraction = t < 0 ? 0 : t > 1 ? 1 : t;
  if (fraction === 0) {
    return { x: points.x0, y: points.y0 };
  }
  if (fraction === 1) {
    return { x: points.x3, y: points.y3 };
  }
  // The curve is measured in a few points, and the length of each stretch is
  // added up: the fraction of the length is then the point wanted, found
  // between the two measured points that hold it.
  const measured: { x: number; y: number; length: number }[] = [];
  let length = 0;
  let previous = pointAtParameter(points, 0);
  for (let index = 1; index <= SAMPLES; index += 1) {
    const current = pointAtParameter(points, index / SAMPLES);
    length += Math.hypot(current.x - previous.x, current.y - previous.y);
    measured.push({ x: current.x, y: current.y, length });
    previous = current;
  }
  const wanted = length * fraction;
  let before = { x: points.x0, y: points.y0, length: 0 };
  for (const point of measured) {
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
  return { x: points.x3, y: points.y3 };
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
