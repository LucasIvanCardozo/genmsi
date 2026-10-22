import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
  Background,
  Controls,
  Handle,
  MiniMap,
  Position,
  ReactFlow,
  applyNodeChanges,
  type Connection,
  type Edge,
  type EdgeChange,
  type Node,
  type NodeChange,
  type NodeProps,
  type ReactFlowInstance,
} from "@xyflow/react";
import "@xyflow/react/dist/style.css";
import type { ModelBody, ModelBlock } from "../types";

/** One port of a block, as the node draws it. */
interface PortData {
  /** Port id, unique inside the block. */
  ioid: number;
  /** Port name (`Entrada1`, `Salida1`, ...). */
  name: string;
}

/** What a block of the graph shows. */
interface BlockData extends Record<string, unknown> {
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

/** One block of the model, drawn as a node of the graph. */
function BlockNode({ data, selected }: NodeProps<Node<BlockData>>) {
  const container = data.kind === "C";
  const note = data.title === "Note";
  const spread = (count: number, index: number): string =>
    `${((index + 1) / (count + 1)) * 100}%`;
  return (
    <div
      className={[
        "flex h-full w-full flex-col justify-center rounded-md border px-2 text-left shadow-sm",
        container
          ? "border-slate-300 border-dashed bg-slate-50/80"
          : "border-slate-400 bg-white",
        note ? "opacity-70" : "",
        selected ? "ring-2 ring-indigo-500" : "",
      ].join(" ")}
    >
      {data.inputs.map((port, index) => (
        <Handle
          key={`in-${port.ioid}`}
          type="target"
          position={Position.Top}
          id={String(port.ioid)}
          title={port.name}
          style={{ left: spread(data.inputs.length, index), top: -6 }}
          className="!size-2.5 !border-0 !bg-slate-400"
        />
      ))}
      {/* A container is entered through one of its input ports and left
          through one of its output ports: those links start and end at the
          container itself, so the container needs a source and a target
          handle for them. */}
      {container &&
        data.inputs.map((port, index) => (
          <Handle
            key={`entry-${port.ioid}`}
            type="source"
            position={Position.Top}
            id={`in-${port.ioid}`}
            title={port.name}
            style={{ left: spread(data.inputs.length, index), top: -6 }}
            className="!size-2.5 !border-0 !bg-transparent"
          />
        ))}
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
      {data.outputs.map((port, index) => (
        <Handle
          key={`out-${port.ioid}`}
          type="source"
          position={Position.Bottom}
          id={String(port.ioid)}
          title={port.name}
          style={{ left: spread(data.outputs.length, index), bottom: -6 }}
          className="!size-2.5 !border-0 !bg-slate-400"
        />
      ))}
      {container &&
        data.outputs.map((port, index) => (
          <Handle
            key={`exit-${port.ioid}`}
            type="target"
            position={Position.Bottom}
            id={`out-${port.ioid}`}
            title={port.name}
            style={{ left: spread(data.outputs.length, index), bottom: -6 }}
            className="!size-2.5 !border-0 !bg-transparent"
          />
        ))}
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
function graphOf(model: ModelBody): { nodes: Node<BlockData>[]; edges: Edge[] } {
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

/** What the graph asks the app to do. */
export interface GraphActions {
  /** The blocks the user selected. */
  onSelect: (ids: number[]) => void;
  /** The user moved a block. */
  onMove: (id: number, x: number, y: number) => void;
  /** The user connected two ports. */
  onConnect: (from: number, fromIoid: number, to: number, toIoid: number) => void;
  /** The user removed a block. */
  onRemove: (ids: number[]) => void;
  /** The user removed a link. */
  onDisconnect: (container: number, from: number, fromIoid: number) => void;
  /** The user dropped a new block of a class. */
  onDropBlock: (className: string, x: number, y: number) => void;
}

/**
 * The model's graph, editable.
 *
 * The blocks are placed with the geometry the `.gmdl` carries, so the picture
 * is the one the original editor drew. Every change leaves the graph as a
 * command for the facade: the app never rewrites the file itself.
 */
export default function GraphView({
  model,
  actions,
  fitSignal,
}: {
  model: ModelBody;
  actions: GraphActions;
  /** A counter that asks the graph to frame the model again. */
  fitSignal: number;
}) {
  const [nodes, setNodes] = useState<Node<BlockData>[]>([]);
  const [edges, setEdges] = useState<Edge[]>([]);
  // The instance of the graph, to frame it when a model arrives.
  const instance = useRef<ReactFlowInstance<Node<BlockData>, Edge> | null>(null);
  // The model the view was framed for: a change of the model reframes the
  // graph, an edit of the model does not.
  const framed = useRef<string | null>(null);
  // The blocks the user had selected, so a change of the model does not take
  // the selection away.
  const selected = useRef<Set<string>>(new Set());

  useEffect(() => {
    const graph = graphOf(model);
    setNodes(
      graph.nodes.map((node) => ({ ...node, selected: selected.current.has(node.id) })),
    );
    setEdges(graph.edges);
    // The first view frames the flow of the model: the blocks that simulate.
    // The containers are the frames around them, and framing them too would
    // shrink the picture until nothing could be read.
    const key = `${model.path}|${model.title}`;
    if (framed.current !== key) {
      framed.current = key;
      fitFlow(instance.current, graph.nodes);
    }
  }, [model]);

  // The user asks for the picture to be framed again.
  useEffect(() => {
    if (fitSignal === 0) {
      return;
    }
    fitFlow(instance.current, graphOf(model).nodes);
  }, [fitSignal, model]);

  const onNodesChange = useCallback((changes: NodeChange<Node<BlockData>>[]) => {
    setNodes((current) => applyNodeChanges(changes, current));
  }, []);

  const onEdgesChange = useCallback((changes: EdgeChange[]) => {
    setEdges((current) => changes.reduce((acc, change) => {
      if (change.type === "remove") {
        return acc.filter((edge) => edge.id !== change.id);
      }
      return acc;
    }, current));
  }, []);

  const onConnect = useCallback(
    (connection: Connection) => {
      if (connection.source === null || connection.target === null) {
        return;
      }
      if (connection.sourceHandle === null || connection.targetHandle === null) {
        return;
      }
      actions.onConnect(
        Number(connection.source),
        Number(connection.sourceHandle),
        Number(connection.target),
        Number(connection.targetHandle),
      );
    },
    [actions],
  );

  const onDrop = useCallback(
    (event: React.DragEvent) => {
      event.preventDefault();
      const className = event.dataTransfer.getData("application/genmsi-block");
      if (className === "") {
        return;
      }
      const position = projectPosition(event);
      if (position !== null) {
        actions.onDropBlock(className, position.x, position.y);
      }
    },
    [actions],
  );

  const projected = useMemo(
    () => ({ nodes, edges }),
    [nodes, edges],
  );

  return (
    <div
      className="h-full w-full overflow-hidden rounded-lg border border-slate-200 bg-white"
      onDrop={onDrop}
      onDragOver={(event) => {
        event.preventDefault();
        event.dataTransfer.dropEffect = "copy";
      }}
    >
      <ReactFlow
        nodes={projected.nodes}
        edges={projected.edges}
        nodeTypes={nodeTypes}
        onInit={(flow) => {
          instance.current = flow;
        }}
        onNodesChange={onNodesChange}
        onEdgesChange={onEdgesChange}
        onNodeClick={(_event, node) => actions.onSelect([Number(node.id)])}
        onPaneClick={() => actions.onSelect([])}
        onSelectionChange={({ nodes: picked }) => {
          selected.current = new Set(picked.map((node) => node.id));
          actions.onSelect(picked.map((node) => Number(node.id)));
        }}
        onNodeDragStop={(_event, node) =>
          actions.onMove(Number(node.id), Math.round(node.position.x), Math.round(node.position.y))
        }
        onConnect={onConnect}
        onNodesDelete={(deleted) => actions.onRemove(deleted.map((node) => Number(node.id)))}
        onEdgesDelete={(deleted) => {
          for (const edge of deleted) {
            const data = edge.data as
              | { container: number; from: number; fromIoid: number }
              | undefined;
            if (data !== undefined) {
              actions.onDisconnect(data.container, data.from, data.fromIoid);
            }
          }
        }}
        nodesDraggable
        nodesConnectable
        edgesReconnectable={false}
        deleteKeyCode={["Delete", "Backspace"]}
        selectionOnDrag
        multiSelectionKeyCode={["Control", "Meta"]}
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

/**
 * Frame the flow of the model: the blocks that simulate.
 *
 * The containers are the frames around them; framing them as well would
 * shrink the picture until nothing could be read.
 */
function fitFlow(instance: ReactFlowInstance<Node<BlockData>, Edge> | null, nodes: Node<BlockData>[]): void {
  const flow = nodes.filter((node) => node.data.kind !== "C");
  instance?.fitView({
    nodes: flow.length > 0 ? flow : nodes,
    padding: 0.2,
    minZoom: 0.4,
  });
}

/**
 * The canvas position of a drop.
 *
 * The graph knows where the pointer is in the canvas, so a block dropped on
 * the picture lands there.
 */
function projectPosition(event: React.DragEvent): { x: number; y: number } | null {
  const target = event.currentTarget as HTMLElement;
  const bounds = target.getBoundingClientRect();
  return {
    x: Math.round(event.clientX - bounds.left),
    y: Math.round(event.clientY - bounds.top),
  };
}
