/**
 * The deterministic re-layout of the graph.
 *
 * The re-layout is explicit: the user presses `Reacomodar` and the graph is
 * reordered in layers. It is never run on open, so the picture the author of
 * the file left stays until the user asks for a new one.
 *
 * The frontend computes the positions and sends them as ONE command, so the
 * whole re-layout is one undo entry. The algorithm is pure and deterministic:
 * the same model always produces the same positions.
 *
 * This module has no imports on purpose: it is the pure part of the editor,
 * and the tests run it without a bundler.
 */

/** A block of the graph, as the re-layout needs it. */
export interface LayoutBlock {
  /** `moduleid`, unique in the model. */
  id: number;
  /** `moduleclass`; `main` for the model root. */
  class: string;
  /** `A` for an activity block, `C` for a container. */
  kind: string;
  /** Id of the container that owns the block. */
  container: number;
  /** Canvas geometry, absent when the model does not place the block. */
  layout: { x: number; y: number; width: number; height: number } | null;
}

/** A link between two blocks. */
export interface LayoutEdge {
  /** Id of the container that owns the link. */
  container: number;
  /** Source module id. */
  from_module: number;
  /** Target module id. */
  to_module: number;
}

/** The geometry of a block after the re-layout. */
export interface LayoutPosition {
  /** `moduleid` of the block. */
  id: number;
  /** Horizontal position. */
  x: number;
  /** Vertical position. */
  y: number;
  /** Block width. */
  width: number;
  /** Block height. */
  height: number;
}

/** Horizontal gap between two layers. */
const GAP_X = 60;
/** Vertical gap between two blocks of the same layer. */
const GAP_Y = 30;
/** Margin between a container and the blocks it holds. */
const PAD = 24;

/**
 * The geometry every block of the model has after the re-layout.
 *
 * The blocks of every container are placed in layers: a block with no incoming
 * link is in the first layer, and a block that follows another one is in the
 * layer after it. Inside a layer the blocks keep their horizontal order, so
 * the re-layout reorders the flow and not the author's reading order.
 *
 * The containers grow to hold their blocks, and the model root (which has no
 * geometry of its own) is left alone.
 */
export function relayout(
  blocks: LayoutBlock[],
  edges: LayoutEdge[],
  rootId: number,
): LayoutPosition[] {
  const positions = new Map<number, LayoutPosition>();
  const byId = new Map(blocks.map((block) => [block.id, block]));

  const childrenOf = (container: number): LayoutBlock[] =>
    blocks
      .filter(
        (block) =>
          block.container === container && block.id !== container && block.layout !== null,
      )
      .sort((a, b) => a.id - b.id);

  const place = (containerId: number, originX: number, originY: number): void => {
    const children = childrenOf(containerId);
    if (children.length === 0) {
      return;
    }
    const layers = layersOf(children, edges.filter((edge) => edge.container === containerId));
    let x = originX;
    for (const layer of layers) {
      // The order inside a layer is the current horizontal order, and the id
      // breaks a tie: the same model always lays out the same way.
      const ordered = [...layer].sort(
        (a, b) => (a.layout?.x ?? 0) - (b.layout?.x ?? 0) || a.id - b.id,
      );
      let y = originY;
      let width = 0;
      for (const child of ordered) {
        const size = child.layout ?? { x: 0, y: 0, width: 120, height: 48 };
        positions.set(child.id, {
          id: child.id,
          x,
          y,
          width: size.width,
          height: size.height,
        });
        width = Math.max(width, size.width);
        y += size.height + GAP_Y;
      }
      x += width + GAP_X;
    }

    // The blocks of a container that is itself a container.
    for (const child of children) {
      if (child.kind !== "C") {
        continue;
      }
      const placed = positions.get(child.id);
      if (placed !== undefined) {
        place(child.id, placed.x + PAD, placed.y + PAD);
      }
    }

    // The container holds its blocks: it grows when they need more room.
    const container = byId.get(containerId);
    if (container?.layout !== null && container?.layout !== undefined) {
      const box = bounds(
        children.map((child) => positions.get(child.id)).filter((p): p is LayoutPosition => p !== undefined),
      );
      positions.set(containerId, {
        id: containerId,
        x: box.x - PAD,
        y: box.y - PAD,
        width: box.width + 2 * PAD,
        height: box.height + 2 * PAD,
      });
    }
  };

  const rootChildren = childrenOf(rootId);
  const current = rootChildren
    .map((child) => child.layout)
    .filter((layout): layout is NonNullable<LayoutBlock["layout"]> => layout !== null);
  const originX = current.length > 0 ? Math.min(...current.map((layout) => layout.x)) : 0;
  const originY = current.length > 0 ? Math.min(...current.map((layout) => layout.y)) : 0;
  place(rootId, originX, originY);

  return blocks
    .filter((block) => positions.has(block.id))
    .map((block) => positions.get(block.id) as LayoutPosition)
    .sort((a, b) => a.id - b.id);
}

/**
 * The layers of the blocks of one container.
 *
 * A block with no incoming link between the blocks of the container is in the
 * first layer; a block that follows another one is one layer deeper. A cycle
 * (the editor allows one) is broken at the block the walk meets again, so the
 * result is still deterministic.
 */
function layersOf(children: LayoutBlock[], edges: LayoutEdge[]): LayoutBlock[][] {
  const ids = new Set(children.map((child) => child.id));
  const incoming = new Map<number, number[]>();
  for (const edge of edges) {
    if (!ids.has(edge.from_module) || !ids.has(edge.to_module)) {
      continue;
    }
    if (edge.from_module === edge.to_module) {
      continue;
    }
    const sources = incoming.get(edge.to_module) ?? [];
    sources.push(edge.from_module);
    incoming.set(edge.to_module, sources);
  }

  const layerOf = new Map<number, number>();
  const visiting = new Set<number>();
  const depth = (id: number): number => {
    const known = layerOf.get(id);
    if (known !== undefined) {
      return known;
    }
    if (visiting.has(id)) {
      return 0;
    }
    visiting.add(id);
    const sources = incoming.get(id) ?? [];
    const value = sources.length === 0 ? 0 : 1 + Math.max(...sources.map(depth));
    visiting.delete(id);
    layerOf.set(id, value);
    return value;
  };
  for (const child of children) {
    depth(child.id);
  }

  const deepest = Math.max(...children.map((child) => layerOf.get(child.id) ?? 0));
  const layers: LayoutBlock[][] = Array.from({ length: deepest + 1 }, () => []);
  for (const child of children) {
    layers[layerOf.get(child.id) ?? 0].push(child);
  }
  return layers;
}

/** The rectangle that holds a list of blocks. */
function bounds(list: LayoutPosition[]): {
  x: number;
  y: number;
  width: number;
  height: number;
} {
  if (list.length === 0) {
    return { x: 0, y: 0, width: 0, height: 0 };
  }
  const x = Math.min(...list.map((item) => item.x));
  const y = Math.min(...list.map((item) => item.y));
  const right = Math.max(...list.map((item) => item.x + item.width));
  const bottom = Math.max(...list.map((item) => item.y + item.height));
  return { x, y, width: right - x, height: bottom - y };
}
