import { useMemo } from "react";
import type { ModelBody } from "../types";

/** The space the picture leaves around the blocks, in model units. */
const PADDING = 40;

/**
 * The blocks of the model, drawn as the monitor's canvas.
 *
 * The canvas is the model's own geometry (the `.gmdl` carries it), read-only
 * and simpler than the graph: the monitor shows *what the run is doing*, not
 * an editor. A block with transactions inside is highlighted and shows how
 * many.
 */
export default function MonitorCanvas({
  model,
  activity,
  totals,
  classes,
}: {
  model: ModelBody;
  /** The transactions inside every block of the model, by block id. */
  activity: Map<number, number>;
  /** The entries every block has had since the run started, by block id. */
  totals: Map<number, number>;
  /** The class of every block of the model, by block id. */
  classes: Map<number, string>;
}) {
  const picture = useMemo(() => {
    const placed = model.blocks.filter((block) => block.layout !== null);
    if (placed.length === 0) {
      return null;
    }
    // The picture frames the flow (the blocks that simulate), like the graph
    // does: the containers are the frames around them.
    const flow = placed.filter((block) => block.kind !== "C");
    const framed = flow.length > 0 ? flow : placed;
    const left = Math.min(...framed.map((block) => block.layout?.x ?? 0));
    const top = Math.min(...framed.map((block) => block.layout?.y ?? 0));
    const right = Math.max(
      ...framed.map((block) => (block.layout?.x ?? 0) + (block.layout?.width ?? 0)),
    );
    const bottom = Math.max(
      ...framed.map((block) => (block.layout?.y ?? 0) + (block.layout?.height ?? 0)),
    );
    const width = right - left + PADDING * 2;
    const height = bottom - top + PADDING * 2;
    const byId = new Map(placed.map((block) => [block.id, block]));
    // The links, drawn from the middle of a block to the middle of the next.
    const links = model.edges
      .filter((edge) => byId.has(edge.from_module) && byId.has(edge.to_module))
      .map((edge) => {
        const from = byId.get(edge.from_module);
        const to = byId.get(edge.to_module);
        if (from?.layout === null || to?.layout === null || from === undefined || to === undefined) {
          return null;
        }
        return {
          id: edge.id,
          x1: (from.layout?.x ?? 0) - left + PADDING + (from.layout?.width ?? 0) / 2,
          y1: (from.layout?.y ?? 0) - top + PADDING + (from.layout?.height ?? 0) / 2,
          x2: (to.layout?.x ?? 0) - left + PADDING + (to.layout?.width ?? 0) / 2,
          y2: (to.layout?.y ?? 0) - top + PADDING + (to.layout?.height ?? 0) / 2,
        };
      })
      .filter((link) => link !== null);
    // The containers are drawn first: they are the frames around the flow.
    const drawn = [...placed].sort((a, b) => {
      const containerA = a.kind === "C" ? 0 : 1;
      const containerB = b.kind === "C" ? 0 : 1;
      return containerA - containerB;
    });
    return { left, top, width, height, placed: drawn, links };
  }, [model]);

  if (picture === null) {
    return (
      <div className="flex h-full items-center justify-center rounded-lg border border-dashed border-slate-300 bg-white/60 text-sm text-slate-500">
        Este modelo no coloca sus bloques en el lienzo.
      </div>
    );
  }

  return (
    <div className="flex h-full flex-col gap-1">
      <svg
      viewBox={`0 0 ${picture.width} ${picture.height}`}
      className="h-full w-full"
      role="img"
      aria-label="Los bloques del modelo y su actividad en la reproducción"
    >

      <g stroke="#cbd5e1" strokeWidth={2}>
        {picture.links.map((link) => (
          <line key={link.id} x1={link.x1} y1={link.y1} x2={link.x2} y2={link.y2} />
        ))}
      </g>
      {picture.placed.map((block) => {
        const layout = block.layout;
        if (layout === null) {
          return null;
        }
        const count = activity.get(block.id) ?? 0;
        const total = totals.get(block.id) ?? 0;
        const x = (layout.x ?? 0) - picture.left + PADDING;
        const y = (layout.y ?? 0) - picture.top + PADDING;
        const container = block.kind === "C";
        const title = block.caption !== "" ? block.caption : block.class;
        const className = classes.get(block.id) ?? block.class;
        const width = layout.width ?? 100;
        const height = layout.height ?? 40;
        // The two numbers of a block: how many transactions are inside it now
        // and how many have gone through it since the run started.
        const now = count;
        const ever = total;
        const busy = now > 0;
        return (
          <g key={block.id}>
            <rect
              x={x}
              y={y}
              width={width}
              height={height}
              rx={6}
              fill={busy ? "#e0e7ff" : container ? "#f8fafc" : "#ffffff"}
              stroke={busy ? "#6366f1" : container ? "#cbd5e1" : "#94a3b8"}
              strokeWidth={busy ? 2.5 : 1.5}
              strokeDasharray={container ? "6 4" : undefined}
            />
            <text
              x={x + 8}
              y={y + 18}
              fontSize={13}
              fill={busy ? "#3730a3" : "#334155"}
              fontWeight={busy ? 600 : 400}
            >
              {title.length > 22 ? `${title.slice(0, 21)}…` : title}
            </text>
            {/* The class of the block, so a queue reads as a queue. */}
            <text x={x + 8} y={y + 32} fontSize={9} fill="#94a3b8">
              {className}
            </text>
            {ever > 0 && (
              /* The count of the moment, and the total of the block under
                 it: "3 / 145" reads as "three now, a hundred and forty-five in
                 all", which is what the two numbers are. */
              <>
                <text
                  x={x + width - 8}
                  y={y + 17}
                  fontSize={13}
                  fill={busy ? "#4338ca" : "#64748b"}
                  textAnchor="end"
                  fontWeight={600}
                >
                  {now}
                </text>
                <text
                  x={x + width - 8}
                  y={y + 31}
                  fontSize={10}
                  fill="#94a3b8"
                  textAnchor="end"
                >
                  / {ever}
                </text>
              </>
            )}
          </g>
        );
      })}
      </svg>
      <p className="px-1 text-[11px] text-slate-400">
        Cada bloque muestra las transacciones que tiene ahora y, debajo, el total que
        lleva desde el comienzo de la corrida.
      </p>
    </div>
  );
}
