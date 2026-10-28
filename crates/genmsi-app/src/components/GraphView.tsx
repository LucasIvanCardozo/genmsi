import {
  useCallback,
  useEffect,
  useId,
  useMemo,
  useRef,
  useState,
} from "react";
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
import { graphOf, type BlockData } from "../graph/graph";
import { decideFrame, type FrameState } from "../graph/frame";
import {
  CAPTION_SIZE,
  FIGURE_SIZE,
  LABEL_SIZE,
  PORT_SIZE,
  STROKE_MARGIN,
  captionBaseline,
  captionFits,
  captionText,
  drawingOf,
  figureBaseline,
  figureText,
  figureZoneText,
  labelBaselines,
  labelHeight,
  labelLines,
  nodeSizeOf,
  portPlacements,
  type PortSide,
} from "../graph/node";
import { useBlockCounts, useBlockFigure } from "../monitor/live";
import type { ModelBody } from "../types";
import GraphEdge from "./GraphEdge";

/**
 * What a block of the graph shows.
 *
 * The counts of the run are not here: a block asks the live layer for its
 * own, so that a block the run did not reach is not redrawn when the clock
 * ticks.
 */

/** The side of a handle, as React Flow names it. */
const SIDES: Record<PortSide, Position> = {
  top: Position.Top,
  bottom: Position.Bottom,
  left: Position.Left,
  right: Position.Right,
};

/** The colour of the rótulo of a block. */
const LABEL_COLOR = "#1e293b";

/** The colour of the caption of a block, under its rótulo. */
const CAPTION_COLOR = "#64748b";

/** The colour of the rótulo of a note, which is dimmed with the block. */
const NOTE_LABEL_COLOR = "#475569";

/**
 * One block of the model, drawn as a node of the graph.
 *
 * The block is drawn with the silhouette of its class: the shape module
 * holds it, with the zones where the rótulo and the figure go, and with the
 * anchor of each of its ports (S2, S8, S9). A class the shape module does
 * not know keeps the plain rounded rectangle, so the canvas never breaks.
 *
 * The silhouette is the truth: it is drawn at the size of its own class, one
 * to one, without being scaled and without being cropped. The node of the
 * graph has that very size, so the drawing, the selection and the MiniMap
 * all agree on where the block is.
 */
function BlockNode({ id, data, selected }: NodeProps<Node<BlockData>>) {
  const container = data.kind === "C";
  const note = data.title === "Note";
  const spread = (count: number, index: number): string =>
    `${((index + 1) / (count + 1)) * 100}%`;
  // How the block is drawn: its silhouette, its zones and its ports.
  const drawing = drawingOf(data.title, container);
  // How many transactions the block holds now, and how many have gone
  // through it: the live layer answers for this block alone.
  const counts = useBlockCounts(Number(id));
  // The figure of the block: the number of the entity it operates on, or the
  // pair of always.
  const figure = useBlockFigure(Number(id));
  const clip = useId().replace(/[^a-zA-Z0-9_-]/g, "");

  if (
    drawing.fallback ||
    drawing.shape === undefined ||
    drawing.colors === undefined ||
    drawing.label === null
  ) {
    // The plain block of always: the rounded rectangle the canvas drew
    // before the redesign, with its ports spread along the edges.
    return (
      <div
        className={[
          "relative flex h-full w-full flex-col justify-center rounded-md border px-2 text-left shadow-sm",
          container
            ? "border-slate-300 border-dashed bg-transparent"
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
            className="!size-3 !rounded-full !border-2 !border-white !bg-slate-400"
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
              className="!size-3 !rounded-full !border-2 !border-transparent !bg-transparent"
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
            className="!size-3 !rounded-full !border-2 !border-white !bg-slate-400"
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
              className="!size-3 !rounded-full !border-2 !border-transparent !bg-transparent"
            />
          ))}
        {/* The two numbers of the block: how many transactions are inside it
            now, and how many have gone through it since the run started. */}
        {counts !== undefined && counts.ever > 0 && (
          <span className="pointer-events-none absolute right-1.5 top-1 font-mono text-[10px] leading-none text-indigo-600">
            {counts.now}
            <span className="text-slate-400">/{counts.ever}</span>
          </span>
        )}
      </div>
    );
  }

  const shape = drawing.shape;
  const colors = drawing.colors;
  // The size of the block: the footprint of its class, one to one. The
  // silhouette is never scaled and never cropped.
  const width = shape.footprint.width;
  const height = shape.footprint.height;
  // The ports of the block, at the anchor the shape declares.
  const inputs = portPlacements(shape, data.inputs, "top");
  const outputs = portPlacements(shape, data.outputs, "bottom");
  // The zone of the label, in the pixels of the footprint of the class.
  const labelZone = drawing.label;
  // The rótulo of the block: the name of the class, in one line when it
  // fits its zone and in two lines when it does not. No character of it is
  // ever cut.
  const lines = labelLines(data.title, labelZone);
  const baselines = labelBaselines(labelZone, lines.length);
  // The caption keeps its own place, the second line of the zone, and it is
  // cut to the zone as it always was.
  const caption =
    drawing.subtitle &&
    captionFits(labelZone, lines.length) &&
    data.subtitle !== ""
      ? captionText(data.subtitle, labelZone)
      : "";
  const text =
    drawing.figure === null ? "" : figureZoneText(figureText(figure, counts), drawing.figure);

  return (
    <div
      className={[
        "relative h-full w-full",
        note ? "opacity-70" : "",
        selected ? "ring-2 ring-indigo-500" : "",
      ].join(" ")}
    >
      {/* The silhouette of the class and the text of its zones, drawn in the
          footprint of the class: the path is the one the shape module
          declares, the details are the strokes inside it, and the text goes
          in the zones the shape declares, which are inside the silhouette.
          Nothing is scaled: the viewBox is the footprint of the class, so a
          circle stays a circle and every figure keeps the width of its
          stroke. */}
      <svg
        className="pointer-events-none absolute inset-0 h-full w-full"
        viewBox={`0 0 ${width + 2 * STROKE_MARGIN} ${height + 2 * STROKE_MARGIN}`}
        preserveAspectRatio="xMidYMid meet"
        role="img"
        aria-label={data.subtitle !== "" ? `${data.title} ${data.subtitle}` : data.title}
      >
        {/* The whole drawing lives in the footprint of the class, and the
            group carries it the room the border needs: the outline is
            drawn centred on the line of the shape, so without that margin
            its outer half is cut by the box. */}
        <g transform={`translate(${STROKE_MARGIN} ${STROKE_MARGIN})`}>
        <defs>
          <clipPath id={`label-${clip}`}>
            <rect
              x={labelZone.left}
              y={labelZone.top}
              width={labelZone.width}
              height={Math.max(labelZone.height, labelHeight(lines.length))}
            />
          </clipPath>
          {drawing.figure !== null && (
            <clipPath id={`figure-${clip}`}>
              <rect
                x={drawing.figure.left}
                y={drawing.figure.top}
                width={drawing.figure.width}
                height={drawing.figure.height}
              />
            </clipPath>
          )}
        </defs>
        <path
          d={shape.path}
          fill={colors.relleno}
          stroke={colors.borde}
          strokeWidth={2}
          strokeLinejoin="round"
        />
        {shape.details.map((detail, index) => (
          <path
            key={index}
            d={detail}
            fill="none"
            stroke={colors.borde}
            strokeWidth={1.8}
            strokeLinecap="round"
          />
        ))}
        {/* The rótulo, on the lines of the zone of the label. The name is
            monospaced, as the design declares it and as the measure of the
            zone assumes. */}
        {lines.map((line, index) => (
          <text
            key={index}
            className="font-mono"
            x={labelZone.left + 2}
            y={baselines[index]}
            fontSize={LABEL_SIZE}
            fontWeight={600}
            fill={note ? NOTE_LABEL_COLOR : LABEL_COLOR}
            clipPath={`url(#label-${clip})`}
            style={note ? { fontStyle: "italic" } : undefined}
          >
            {line}
          </text>
        ))}
        {/* The caption, on the second line of the same zone, and only
            when the zone has room for it. */}
        {caption !== "" && (
          <text
            x={labelZone.left + 2}
            y={captionBaseline(labelZone)}
            fontSize={CAPTION_SIZE}
            fill={CAPTION_COLOR}
            clipPath={`url(#label-${clip})`}
          >
            {caption}
          </text>
        )}
        {/* The live figure of the block, in the zone the shape declares
            and in the colour of its family. The zone is reserved even
            when the block has no figure, so the rótulo does not move
            when it appears. */}
        {drawing.figure !== null && text !== "" && (
          <text
            className="font-mono"
            x={drawing.figure.left + drawing.figure.width - 2}
            y={figureBaseline(drawing.figure)}
            textAnchor="end"
            fontSize={FIGURE_SIZE}
            fontWeight={700}
            fill={colors.acento}
            clipPath={`url(#figure-${clip})`}
          >
            {text}
          </text>
        )}
        </g>
      </svg>
      {/* The ports, each one at the anchor the shape declares: the flow
          leaves the block by the very point the original editor put it. */}
      {inputs.map((port) => (
        <Handle
          key={`in-${port.ioid}`}
          type="target"
          position={SIDES[port.side]}
          id={String(port.ioid)}
          title={port.name}
          style={{
            left: `${STROKE_MARGIN + port.x * width}px`,
            top: `${STROKE_MARGIN + port.y * height}px`,
            right: "auto",
            bottom: "auto",
            transform: "translate(-50%, -50%)",
            width: PORT_SIZE,
            height: PORT_SIZE,
            borderRadius: "50%",
            background: colors.acento,
            border: "2px solid #ffffff",
          }}
        />
      ))}
      {outputs.map((port) => (
        <Handle
          key={`out-${port.ioid}`}
          type="source"
          position={SIDES[port.side]}
          id={String(port.ioid)}
          title={port.name}
          style={{
            left: `${STROKE_MARGIN + port.x * width}px`,
            top: `${STROKE_MARGIN + port.y * height}px`,
            right: "auto",
            bottom: "auto",
            transform: "translate(-50%, -50%)",
            width: PORT_SIZE,
            height: PORT_SIZE,
            borderRadius: "50%",
            background: colors.acento,
            border: "2px solid #ffffff",
          }}
        />
      ))}
    </div>
  );
}

const nodeTypes = { bloque: BlockNode };

/** The links are drawn by the component that knows the balls that travel. */
const edgeTypes = { bloque: GraphEdge };

/** The nodes and the edges of the model's graph. */
function graphOfUnused(): void {
  void graphOf;
}
void graphOfUnused;

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
  extraEdges = [],
}: {
  model: ModelBody;
  actions: GraphActions;
  /** A counter that asks the graph to frame the model again. */
  fitSignal: number;
  /**
   * The links the model does not draw but a ball travels over: the direct
   * way between two blocks that the model joins through a container.
   */
  extraEdges?: Edge[];
}) {
  const [nodes, setNodes] = useState<Node<BlockData>[]>([]);
  const [edges, setEdges] = useState<Edge[]>([]);
  // The instance of the graph, to frame it when a model arrives.
  const instance = useRef<ReactFlowInstance<Node<BlockData>, Edge> | null>(null);
  // The framing of the picture: a model that arrives is framed whole, and
  // the picture is framed again when the reader asks for it. A change of the
  // model —the reader moves a block— does not move the picture.
  const frame = useRef<FrameState>({ framed: null, fitted: 0 });
  // The blocks the user had selected, so a change of the model does not take
  // the selection away.
  const selected = useRef<Set<string>>(new Set());

  useEffect(() => {
    const graph = graphOf(model);
    setNodes(
      graph.nodes.map((node) => {
        const drawn = { ...node, selected: selected.current.has(node.id) };
        // The block is drawn at the size of its class: the shape is the
        // truth, and the box the model carries is not rewritten. A class
        // the shape module does not know, and every container, keep the
        // box of the model.
        const size = nodeSizeOf(node.data.title, node.data.kind === "C");
        if (size === null) {
          return drawn;
        }
        return { ...drawn, style: { ...drawn.style, width: size.width, height: size.height } };
      }),
    );
    // The links are drawn by the component that carries the balls.
    setEdges([
      ...graph.edges.map((edge) => ({ ...edge, type: "bloque" })),
      ...extraEdges,
    ]);
    // The first view frames the flow of the model: the blocks that simulate.
    // The containers are the frames around them, and framing them too would
    // shrink the picture until nothing could be read.
    const decision = decideFrame(
      frame.current,
      `${model.path}|${model.title}`,
      fitSignal,
    );
    frame.current = decision.state;
    if (decision.fit) {
      fitFlow(instance.current, graph.nodes);
    }
  }, [model, extraEdges, fitSignal]);
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
        edgeTypes={edgeTypes}
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
    // A model that is composed can be wide: the picture is framed whole,
    // even if the reader has to zoom out to see it.
    minZoom: 0.15,
    // And a fit never zooms past a readable level: the picture of a small
    // model does not become a block the size of the window.
    maxZoom: 1.25,
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
