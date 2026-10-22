import { useMemo } from "react";
import {
  Background,
  Controls,
  Handle,
  MiniMap,
  Position,
  ReactFlow,
  type Edge,
  type Node,
  type NodeProps,
} from "@xyflow/react";
import "@xyflow/react/dist/style.css";
import type { ModelBlock, ModelDocument } from "../types";

/** What a block of the graph shows. */
interface BlockData extends Record<string, unknown> {
  /** The block's class (`Generate`, `Queue`, ...). */
  title: string;
  /** The block's caption, or its label when it has no caption. */
  subtitle: string;
  /** `A` for an activity block, `C` for a container. */
  kind: string;
  /** Whether the user selected the block. */
  selected: boolean;
}

/** One block of the model, drawn as a node of the graph. */
function BlockNode({ data }: NodeProps<Node<BlockData>>) {
  const container = data.kind === "C";
  const note = data.title === "Note";
  return (
    <div
      className={[
        "flex h-full w-full flex-col justify-center rounded-md border px-2 text-left shadow-sm",
        container
          ? "border-slate-300 border-dashed bg-slate-50/80"
          : "border-slate-400 bg-white",
        note ? "opacity-70" : "",
        data.selected ? "ring-2 ring-indigo-500" : "",
      ].join(" ")}
    >
      <Handle type="target" position={Position.Top} className="!size-2 !border-0 !bg-slate-400" />
      <span
        className={[
          "truncate text-xs font-semibold tracking-tight",
          container ? "text-slate-500" : "text-slate-800",
          note ? "italic" : "",
        ].join(" ")}
      >
        {data.title}
      </span>
      {data.subtitle !== "" && (
        <span className="truncate text-[10px] text-slate-500">{data.subtitle}</span>
      )}
      <Handle type="source" position={Position.Bottom} className="!size-2 !border-0 !bg-slate-400" />
    </div>
  );
}

const nodeTypes = { bloque: BlockNode };

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

/** The nodes and the edges of the model's graph. */
function graphOf(model: ModelDocument): { nodes: Node<BlockData>[]; edges: Edge[] } {
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
        selected: false,
      },
      style: {
        width: Math.max(block.layout?.width ?? 100, 72),
        height: Math.max(block.layout?.height ?? 40, 32),
      },
      zIndex: block.kind === "C" ? 0 : 2,
      draggable: false,
    }));

  const edges: Edge[] = model.edges
    .filter((edge) => placed.has(edge.from_module) && placed.has(edge.to_module))
    .map((edge) => ({
      id: edge.id,
      source: String(edge.from_module),
      target: String(edge.to_module),
      label: edge.label ?? undefined,
      animated: false,
      style: { stroke: "#94a3b8", strokeWidth: 1.75 },
      labelStyle: { fontSize: 10, fill: "#64748b" },
      labelBgStyle: { fill: "#f8fafc", fillOpacity: 0.9 },
    }));

  return { nodes, edges };
}

/**
 * The model's graph, read-only.
 *
 * The blocks are placed with the geometry the `.gmdl` carries, so the picture
 * is the one the original editor drew; the slice does not lay out anything.
 */
export default function GraphView({
  model,
  selectedBlockId,
  onSelect,
}: {
  model: ModelDocument;
  selectedBlockId: number | null;
  onSelect: (id: number | null) => void;
}) {
  const { nodes, edges } = useMemo(() => {
    const graph = graphOf(model);
    return {
      nodes: graph.nodes.map((node) => ({
        ...node,
        data: { ...node.data, selected: node.id === String(selectedBlockId) },
      })),
      edges: graph.edges,
    };
  }, [model, selectedBlockId]);

  return (
    <div className="h-full w-full overflow-hidden rounded-lg border border-slate-200 bg-white">
      <ReactFlow
        nodes={nodes}
        edges={edges}
        nodeTypes={nodeTypes}
        onInit={(instance) => {
          // The first view frames the model's flow: the annotations (the
          // blocks the original uses as notes) stay reachable, but they do not
          // push the flow into a corner.
          const flowNodes = nodes.filter((node) => node.data.kind === "A");
          instance.fitView({
            nodes: flowNodes.length > 0 ? flowNodes : nodes,
            padding: 0.15,
            minZoom: 0.5,
          });
        }}
        onNodeClick={(_event, node) => onSelect(Number(node.id))}
        onPaneClick={() => onSelect(null)}
        nodesDraggable={false}
        nodesConnectable={false}
        edgesReconnectable={false}
        elementsSelectable
        fitView={false}
        minZoom={0.1}
        maxZoom={2}
        proOptions={{ hideAttribution: true }}
      >
        <Background color="#e2e8f0" gap={16} />
        <Controls showInteractive={false} />
        <MiniMap
          pannable
          zoomable
          className="!bg-white"
          nodeColor="#cbd5e1"
          nodeStrokeColor="#94a3b8"
          maskColor="rgba(241, 245, 249, 0.55)"
        />
      </ReactFlow>
    </div>
  );
}
