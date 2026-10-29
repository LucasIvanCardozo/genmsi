import {
  memo,
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
  LABEL_SIZE,
  PORT_SIZE,
  STROKE_MARGIN,
  captionBaseline,
  captionFits,
  captionLayout,
  drawingOf,
  figureLayout,
  figureText,
  figureZoneText,
  labelBaselines,
  labelHeight,
  labelLines,
  nodeSizeOf,
  portPlacements,
  textsOf,
  wrapperClass,
  type PortSide,
} from "../graph/node";
import { useBlockCounts, useBlockFigure, useRunHasPublished } from "../monitor/live";
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
 * Every block of the canvas is drawn the same way: the 66 classes of the
 * catalogue, the container and the doors of a container (S3). The shape
 * module holds the zones where the rótulo and the figure go, and the
 * anchor of each of its ports (S2, S8, S9); the shape of a block the
 * catalogue does not know comes from the frames module. The ring of the
 * selection goes on the wrapper that holds the silhouette.
 *
 * The silhouette is the truth: it is drawn at the size of its own shape, one
 * to one, without being scaled and without being cropped. The node of the
 * graph has that very size, so the drawing, the selection and the MiniMap
 * all agree on where the block is.
 */
function BlockNode({ id, data, selected }: NodeProps<Node<BlockData>>) {
  const note = data.kind === "A" && data.title === "Note";
  // How the block is drawn: its silhouette, its zones and its ports. The
  // class decides the shape of the 66 classes of the catalogue, and the
  // kind decides the shape of the container and of the doors.
  const drawing = drawingOf({
    className: data.kind === "A" ? data.title : "",
    kind: data.kind,
    title: data.title,
    subtitle: data.subtitle,
    size: data.size,
    inputs: data.inputs,
    outputs: data.outputs,
  });
  // The name the block shows and the caption under it: the container shows
  // the caption of the model as its name and the word "Contenedor" under
  // it, and the doors show the name of the port.
  const texts = textsOf(data.kind, data.title, data.subtitle);
  // How many transactions the block holds now, and how many have gone
  // through it: the live layer answers for this block alone.
  const counts = useBlockCounts(Number(id));
  // The figure of the block: the number of the entity it operates on, or the
  // pair of always.
  const figure = useBlockFigure(Number(id));
  // The dash of an empty figure only says something while the run plays.
  const running = useRunHasPublished();
  const clip = useId().replace(/[^a-zA-Z0-9_-]/g, "");

  const shape = drawing.shape;
  const colors = drawing.colors;
  // The size of the block: the footprint of its shape, one to one. The
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
  const lines = labelLines(texts.label, labelZone);
  const baselines = labelBaselines(labelZone, lines.length);
  // The caption keeps its own place, the second line of the zone, and it is
  // drawn at the size of the design when it fits: a caption that does not
  // fit is drawn smaller before it is cut, because it is the reader's own
  // text.
  const caption =
    drawing.subtitle &&
    captionFits(labelZone, lines.length) &&
    texts.caption !== ""
      ? captionLayout(texts.caption, labelZone)
      : { text: "", size: CAPTION_SIZE };
  // The figure of the block, centred in the zone the shape declares. A
  // block the run publishes no figure for shows the dash of an empty
  // figure: the zone is reserved, and the hole is not silent. The dash
  // only appears while the run plays: before it, there is no run to say
  // anything about the value of a block.
  const figureZone = drawing.figure;
  const live =
    figureZone === null
      ? null
      : figureLayout(
          figureZone,
          running ? figureZoneText(figureText(figure, counts), figureZone) : "",
        );

  return (
    <div className={wrapperClass(selected, note)}>
      {/* The silhouette of the block and the text of its zones, drawn in the
          footprint of the shape: the path is the one the shape module
          declares, the details are the strokes inside it, and the text goes
          in the zones the shape declares, which are inside the silhouette.
          Nothing is scaled: the viewBox is the box of the node, so a circle
          stays a circle and every figure keeps the width of its stroke. */}
      <svg
        className="pointer-events-none absolute inset-0 h-full w-full"
        viewBox={`0 0 ${drawing.size.width} ${drawing.size.height}`}
        preserveAspectRatio="xMidYMid meet"
        role="img"
        aria-label={data.subtitle !== "" ? `${data.title} ${data.subtitle}` : data.title}
      >
        {/* The whole drawing lives in the footprint of the shape, and the
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
            when the zone has room for it. It is drawn at the size of the
            design, or smaller when that is what it takes to keep it
            whole. */}
        {caption.text !== "" && (
          <text
            x={labelZone.left + 2}
            y={captionBaseline(labelZone)}
            fontSize={caption.size}
            fill={CAPTION_COLOR}
            clipPath={`url(#label-${clip})`}
          >
            {caption.text}
          </text>
        )}
        {/* The live figure of the block, in the middle of the zone the
            shape declares and in the colour of its family. The zone is
            reserved even when the block has no figure, so the rótulo does
            not move when it appears; a block the run publishes no figure
            for shows the dash of an empty figure, in the muted tone of
            the family. */}
        {live !== null && (
          <text
            className="font-mono"
            x={live.x}
            y={live.y}
            textAnchor={live.anchor}
            fontSize={live.size}
            fontWeight={700}
            fill={live.empty ? colors.acento : colors.acento}
            opacity={live.empty ? 0.35 : 1}
            clipPath={`url(#figure-${clip})`}
          >
            {live.text}
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

/**
 * The links the model does not draw: none, by default.
 *
 * The list is a constant of the module and not a fresh array on every
 * render: the effect that draws the graph depends on it, and a list that
 * changes its identity on every render makes that effect run again and
 * again, for ever.
 */
const NO_EXTRA_EDGES: Edge[] = [];

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
export default memo(function GraphView({
  model,
  actions,
  fitSignal,
  extraEdges = NO_EXTRA_EDGES,
  containerId = 0,
  onOpenContainer,
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
  /** The container currently scoped in the canvas (0 = root). */
  containerId?: number;
  /** Navigate into a child container. */
  onOpenContainer?: (id: number) => void;
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
    const graph = graphOf(model, containerId);
    setNodes(
      graph.nodes.map((node) => {
        const drawn = { ...node, selected: selected.current.has(node.id) };
        // The block is drawn at the size of its shape: the shape is the
        // truth, and the box the model carries is not rewritten. A class
        // the shape module does not know, the container and the doors are
        // drawn at the box the model gives them.
        const size = nodeSizeOf({
          className: node.data.kind === "A" ? node.data.title : "",
          kind: node.data.kind,
          title: node.data.title,
          subtitle: node.data.subtitle,
          size: node.data.size,
          inputs: node.data.inputs,
          outputs: node.data.outputs,
        });
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
      `${model.path}|${model.title}|${containerId}`,
      fitSignal,
    );
    frame.current = decision.state;
    if (decision.fit) {
      fitFlow(instance.current, graph.nodes);
    }
  }, [model, extraEdges, fitSignal, containerId]);
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
      let from = Number(connection.source);
      let fromIoid = Number(connection.sourceHandle);
      if (connection.source.startsWith("in-")) {
        from = containerId;
        fromIoid = Number(connection.source.slice(3));
      }

      let to = Number(connection.target);
      let toIoid = Number(connection.targetHandle);
      if (connection.target.startsWith("out-")) {
        to = containerId;
        toIoid = Number(connection.target.slice(4));
      }

      if (
        Number.isNaN(from) ||
        Number.isNaN(fromIoid) ||
        Number.isNaN(to) ||
        Number.isNaN(toIoid)
      ) {
        return;
      }

      actions.onConnect(from, fromIoid, to, toIoid);
    },
    [actions, containerId],
  );

  const onDrop = useCallback(
    (event: React.DragEvent) => {
      event.preventDefault();
      const className = event.dataTransfer.getData("application/genmsi-block");
      if (className === "") {
        return;
      }
      if (instance.current) {
        const position = instance.current.screenToFlowPosition({
          x: event.clientX,
          y: event.clientY,
        });
        actions.onDropBlock(className, Math.round(position.x), Math.round(position.y));
      }
    },
    [actions],
  );

  const projected = useMemo(
    () => ({ nodes, edges }),
    [nodes, edges],
  );

  const onNodeDoubleClick = useCallback(
    (_event: React.MouseEvent, node: Node<BlockData>) => {
      if (node.data.kind === "C") {
        const id = Number(node.id);
        if (id !== containerId) {
          onOpenContainer?.(id);
        }
      }
    },
    [containerId, onOpenContainer],
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
        onNodeDoubleClick={onNodeDoubleClick}
        onNodeClick={(_event, node) => {
          if (!node.id.startsWith("in-") && !node.id.startsWith("out-")) {
            actions.onSelect([Number(node.id)]);
          }
        }}
        onPaneClick={() => actions.onSelect([])}
        onSelectionChange={({ nodes: picked }) => {
          selected.current = new Set(picked.map((node) => node.id));
          const validIds = picked
            .filter((node) => !node.id.startsWith("in-") && !node.id.startsWith("out-"))
            .map((node) => Number(node.id));
          actions.onSelect(validIds);
        }}
        onNodeDragStop={(_event, node) => {
          if (node.id.startsWith("in-") || node.id.startsWith("out-")) {
            return;
          }
          actions.onMove(Number(node.id), Math.round(node.position.x), Math.round(node.position.y));
        }}
        onConnect={onConnect}
        onNodesDelete={(deleted) => {
          const blockIds = deleted
            .filter((node) => !node.id.startsWith("in-") && !node.id.startsWith("out-"))
            .map((node) => Number(node.id));
          if (blockIds.length > 0) {
            actions.onRemove(blockIds);
          }
        }}
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
});

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
