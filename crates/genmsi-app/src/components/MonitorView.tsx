import { useEffect, useMemo, useRef, useState } from "react";
import { ChevronRight, CornerLeftUp, Layers } from "lucide-react";
import { advance, advanceClock, clockWithCrossings, frameAt, stateAt, type ReplayState } from "../monitor/replay";
import { useApp } from "../state/store";
import { FLIGHT_SECONDS, MIN_WALL_FLIGHT, travellersAt, type Move } from "../monitor/balls";
import { graphOf, routeFor, routesOf } from "../graph/graph";
import { countsOf } from "../monitor/counts";
import { figureBlockOf, figuresOf } from "../monitor/figures";
import { useLive, type LiveBall, type LiveCounts } from "../monitor/live";
import ClassList from "./ClassList";
import ModelTree from "./ModelTree";
import StatusLine from "./StatusLine";
import GraphView, { type GraphActions } from "./GraphView";
import MonitorCharts from "./MonitorCharts";
import PropertiesPanel from "./PropertiesPanel";
import ReportView from "./ReportView";

/** The part of the body the window shows. */
type Panel = "reproduccion" | "graficos" | "reporte";

/** The part of the lateral panel the window shows. */
type Aside = "bloque" | "bloques" | "arbol";

/**
 * The window of the simulation: the model, its run and its report.
 *
 * The body is the picture: the graph of the model is the reproduction's
 * picture, and the run's header says what run is on screen. The console of
 * the run —pause, speed, clock and slider— lives in the header of the
 * window, so the body only shows what the run does.
 *
 * The graph is the model's picture and the reproduction's picture at the same
 * time: the blocks the run is filling are marked, the transactions that are
 * on their way travel over the links, and the report of the run grows as the
 * reader watches it.
 *
 * The replay is a **projection** of the trace the engine published: the
 * clock, the content of the queues, the owner of the facilities and the
 * transactions inside every block are rebuilt by walking the trace. Nothing
 * is recomputed: no metric the engine computes is derived here.
 */
export default function MonitorView({
  actions,
  fitSignal,
}: {
  /** What the graph asks the app to do. */
  actions: GraphActions;
  /** A counter that asks the graph to frame the model again. */
  fitSignal: number;
}) {
  const {
    model,
    report,
    trace,
    monitor,
    transitions,
    playhead,
    playing,
    speed,
    selectedBlockIds,
    activeContainerId,
    setActiveContainerId,
    focusBlockId,
    focusToken,
    focusBlock,
    select,
    catalogue,
    diagnostics,
    error,
    notice,
    savedMessage,
    pendingSave,
    confirmSave,
    dismissSave,
    addBlock,
    setPlayhead,
    setPlaying,
    apply,
  } = useApp();
  const [panel, setPanel] = useState<Panel>("reproduccion");
  const [aside, setAside] = useState<Aside>("bloque");
  // A block the reader picks brings its properties forward: the lateral
  // panel follows the selection.
  useEffect(() => {
    if (selectedBlockIds.length > 0) {
      setAside("bloque");
    }
  }, [selectedBlockIds]);
  // The state of the replay at the playhead. It is carried from one frame to
  // the next while the replay runs forward, and rebuilt when it goes back.
  const carried = useRef<{ run: unknown; frame: number; state: ReplayState } | null>(null);

  // The state at the playhead.
  const state = useMemo(() => {
    if (monitor === null) {
      return null;
    }
    const cached = carried.current;
    if (cached !== null && cached.run === monitor && cached.frame <= playhead) {
      const next = advance(cached.state, monitor, cached.frame, playhead);
      carried.current = { run: monitor, frame: playhead, state: next };
      return next;
    }
    const next = stateAt(monitor, playhead);
    carried.current = { run: monitor, frame: playhead, state: next };
    return next;
  }, [monitor, playhead]);

  // The counts of every block of the model: how many transactions are inside
  // it now, and how many have gone through it. The program blocks of a block
  // (a facility use expands into several) are added up.
  // The figures of every block: the one of the row that counts it, never the
  // sum of the rows of the block (see `src/monitor/counts.ts`).
  const counts = useMemo(() => {
    if (report === null || state === null) {
      return new Map<number, LiveCounts>();
    }
    return countsOf(report.blocks, state);
  }, [report, state]);

  // The links of the model: the canvas draws the ones of the scope the
  // reader is in, and an entity travels over the one that carries its move.
  const edges = useMemo(
    () => (model === null ? [] : graphOf(model, activeContainerId).edges),
    [model, activeContainerId],
  );

  // The links of the scope, with the pairs of activity blocks they carry:
  // the engine publishes them with the borders of the containers already
  // resolved, so the app does not have to resolve them on its own.
  const scopeLinks = useMemo(() => {
    if (model === null) {
      return [];
    }
    const drawn = new Set(edges.map((edge) => edge.id));
    return model.edges.filter((link) => drawn.has(link.id));
  }, [model, edges]);

  // The routes of the scope: the link that carries each move of the trace.
  // The same move is carried by a different link in each scope, so the
  // routes are the ones of the scope the reader is in.
  const routes = useMemo(
    () => routesOf(scopeLinks, model?.edges ?? []),
    [scopeLinks, model],
  );

  // The blocks of the model, as the figures read them: the class of the
  // block and the properties it declares.
  const figureBlocks = useMemo(
    () => (model === null ? [] : model.blocks.map((block) => figureBlockOf(block))),
    [model],
  );

  // Breadcrumb path from root (id: 0) to activeContainerId.
  const breadcrumbs = useMemo(() => {
    if (model === null) return [];
    const rootLabel = model.title.trim() || model.path?.split("/").pop() || "Modelo";
    if (activeContainerId === 0) {
      return [{ id: 0, label: rootLabel }];
    }
    const chain: { id: number; label: string }[] = [];
    let currentId: number | null = activeContainerId;
    const visited = new Set<number>();
    while (currentId !== null && currentId !== 0 && !visited.has(currentId)) {
      visited.add(currentId);
      const searchId: number = currentId;
      const found = model.blocks.find((b) => b.id === searchId);
      if (found === undefined) break;
      const label = found.caption.trim() || found.name.trim() || `Contenedor ${found.id}`;
      chain.unshift({ id: found.id, label });
      currentId = found.container;
    }
    chain.unshift({ id: 0, label: rootLabel });
    return chain;
  }, [model, activeContainerId]);

  const parentContainerId = useMemo(() => {
    if (model === null || activeContainerId === 0) return 0;
    const current = model.blocks.find((b) => b.id === activeContainerId);
    return current?.container ?? 0;
  }, [model, activeContainerId]);

  // What the animation needs to publish the picture, kept up to date without
  // restarting the animation.
  const live = useRef({
    counts,
    edges,
    routes,
    moves: [] as Move[],
    times: [] as number[],
    figures: state?.figures ?? null,
    report,
    blocks: figureBlocks,
  });
  live.current = {
    counts,
    edges,
    routes,
    moves: (transitions?.moves ?? []) as Move[],
    times: transitions?.times ?? [],
    figures: state?.figures ?? null,
    report,
    blocks: figureBlocks,
  };

  // The clock of the replay: the time the run has reached, kept up to date
  // by the animation. It used to be the time of the last record, which made
  // it jump from one event to the next instead of ticking.
  //
  // The clock of the replay accumulates: it is the time the run has reached,
  // and it only moves by what the wall clock spent. Anchoring it at the time
  // of the current frame instead would make the replay stall whenever a
  // paint advances less than the gap between two frames, and only the fastest
  // speeds would move at all.
  //
  // It lives in a ref so that changing the speed or pausing does not throw
  // the time away: the effect restarts and the clock carries on from where
  // the run had reached. A new run is the one thing that moves it, and it
  // does so explicitly.
  const clockRef = useRef<number | null>(null);
  // The playhead the monitor set itself, to tell the reader's moves from its
  // own.
  const setRef = useRef(-1);
  const frameRef = useRef(playhead);
  const stateRef = useRef<ReplayState | null>(state);
  frameRef.current = playhead;
  stateRef.current = state;

  // The picture of this instant: the clock, the counts of the blocks and the
  // balls that are on their way.
  const publish = (clock: number) => {
    const current = live.current;
    const balls = new Map<string, LiveBall[]>();
    for (const traveller of travellersAt(
      current.moves,
      current.times,
      clock,
      FLIGHT_SECONDS,
      // The floor of the flight is in the clock of the reader: at a hundred
      // times the speed, a ball that crossed a block with no waiting time
      // still flies long enough to be seen.
      speed * MIN_WALL_FLIGHT,
    )) {
      const id = routeFor(current.routes, traveller.from, traveller.to);
      if (id === undefined) {
        // The scope has no link that carries the move: it happens deeper
        // inside the model, and there is nothing to draw the entity on.
        continue;
      }
      const list = balls.get(id);
      const ball: LiveBall = { xact: traveller.xact, progress: traveller.progress };
      if (list === undefined) {
        balls.set(id, [ball]);
      } else {
        list.push(ball);
      }
    }
    // The figure every block shows: the figure of the entity it operates
    // on, and the pair of always for the block that operates on none. It is
    // read with the clock of this instant, because the occupancy of a
    // resource is the time it was held over the time the run has reached.
    const blockFigures = figuresOf(current.blocks, {
      figures: current.figures,
      report: current.report,
      clock,
      counts: current.counts,
    });
    useLive.getState().publish({
      clock,
      counts: current.counts,
      balls,
      figures: current.figures ?? undefined,
      blockFigures,
    });
  };

  useEffect(() => {
    if (!playing || monitor === null) {
      return;
    }
    let handle = 0;
    let last = performance.now();
    // The speed is a multiple of the real time: one simulated second per
    // second of wall clock at 1x.
    const rate = speed;
    const step = (now: number) => {
      if (!useApp.getState().playing) {
        return;
      }
      const elapsed = (now - last) / 1000;
      last = now;
      const current = frameRef.current;
      // The reader moved the playhead (the slider of the header): the clock
      // is already where the reader put it, so it is not re-seeded from the
      // record. The playhead the monitor set itself is remembered, so the
      // clock is not re-seeded on every frame it moves.
      if (clockRef.current === null || current !== setRef.current) {
        clockRef.current = useLive.getState().clock;
        setRef.current = current;
      }
      // At speeds up to 2x, the replay stops at crossings so instantaneous
      // transfers are visible. At higher speeds, the reader is fast-forwarding,
      // so the clock advances at the requested speed without throttling.
      const clock =
        rate <= 2
          ? clockWithCrossings(
              clockRef.current,
              elapsed,
              rate,
              live.current.times,
            )
          : advanceClock(clockRef.current, elapsed, rate);
      // The clock stops at the end of the run: a run that is over does not
      // keep counting, and the picture is the picture of its last instant.
      const reached = clock >= monitor.endTime;
      clockRef.current = reached ? monitor.endTime : clock;

      const frame = frameAt(monitor, clockRef.current);
      if (frame !== frameRef.current && report !== null) {
        const prev = frameRef.current;
        frameRef.current = frame;
        setRef.current = frame;
        const prevState = stateRef.current;
        const nextState =
          prevState !== null && prev <= frame
            ? advance(prevState, monitor, prev, frame)
            : stateAt(monitor, frame);
        stateRef.current = nextState;
        live.current.counts = countsOf(report.blocks, nextState);
        live.current.figures = nextState.figures;
      }

      // The clock the reader sees follows the animation, so it ticks instead
      // of jumping from one record to the next.
      publish(clockRef.current);

      if (reached) {
        setPlayhead(monitor.frames.length - 1);
        setPlaying(false);
        return;
      }
      handle = requestAnimationFrame(step);
    };
    handle = requestAnimationFrame(step);
    return () => {
      cancelAnimationFrame(handle);
      if (frameRef.current !== useApp.getState().playhead) {
        setPlayhead(frameRef.current);
      }
    };
  }, [playing, monitor, speed, report, setPlayhead, setPlaying]);

  // A new run starts its clock again, and going back to the start does too.
  useEffect(() => {
    clockRef.current = monitor === null ? null : monitor.startTime;
    setRef.current = -1;
    useLive.getState().reset();
    if (monitor !== null) {
      useLive.getState().setClock(monitor.startTime);
    }
  }, [monitor]);

  // The picture follows the state of the replay.
  //
  // The slider of the header moves the run while the reproduction is
  // paused, and the picture has to move with it: this effect publishes the
  // picture again every time the state of the replay changes.
  //
  // When playing, the animation step already publishes on each frame with fresh
  // figures, so publishing here would duplicate the work.
  useEffect(() => {
    if (!playing) {
      publish(useLive.getState().clock);
    }
  }, [state, playing]);

  if (model === null) {
    return (
      <div className="flex flex-1 items-center justify-center rounded-lg border border-dashed border-slate-300 bg-white/60 text-sm text-slate-500">
        Abre un modelo para ver su reproducción.
      </div>
    );
  }

  const truncated = trace?.truncated ?? monitor?.truncated ?? false;
  const selectedBlock =
    model.blocks.find((block) => block.id === selectedBlockIds[0]) ?? null;

  return (
    <div className="flex h-full flex-col gap-3 overflow-hidden">
      {truncated && (
        <p className="rounded-lg border border-amber-200 bg-amber-50 px-3 py-1.5 text-xs text-amber-900">
          La traza se cortó en el límite de {monitor?.limit} registros: la
          reproducción muestra sólo esa parte de la ejecución.
        </p>
      )}

      <div className="flex gap-1 border-b border-slate-200">
        {(
          [
            { id: "reproduccion", label: "Reproducción" },
            { id: "graficos", label: "Gráficos" },
            { id: "reporte", label: "Reporte" },
          ] as const
        ).map((candidate) => (
          <button
            key={candidate.id}
            type="button"
            data-monitor-panel={candidate.id}
            onClick={() => setPanel(candidate.id)}
            className={[
              "-mb-px rounded-t-md border-b-2 px-4 py-1.5 text-sm font-medium transition",
              panel === candidate.id
                ? "border-indigo-500 text-indigo-700"
                : "border-transparent text-slate-500 hover:text-slate-800",
            ].join(" ")}
          >
            {candidate.label}
          </button>
        ))}
      </div>

      {panel === "reproduccion" && (
        <div className="flex min-h-0 flex-1 gap-3">
          <div className="flex min-w-0 flex-1 flex-col rounded-lg border border-slate-200 bg-white p-2 overflow-hidden">
            {/* Breadcrumbs navigation bar */}
            <div className="mb-2 flex shrink-0 items-center justify-between border-b border-slate-100 pb-1.5 text-xs">
              <nav aria-label="Ruta de módulos" className="flex items-center gap-1 overflow-x-auto text-slate-500">
                <Layers className="size-3.5 text-indigo-600 shrink-0 mr-0.5" />
                {breadcrumbs.map((crumb, idx) => (
                  <div key={crumb.id} className="flex items-center gap-1 shrink-0">
                    {idx > 0 && <ChevronRight className="size-3 text-slate-300" />}
                    <button
                      type="button"
                      onClick={() => setActiveContainerId(crumb.id)}
                      className={[
                        "rounded px-1.5 py-0.5 transition font-medium text-xs",
                        crumb.id === activeContainerId
                          ? "bg-indigo-50 font-semibold text-indigo-900"
                          : "text-slate-600 hover:bg-slate-100 hover:text-slate-800",
                      ].join(" ")}
                      title={`Ir a ${crumb.label}`}
                    >
                      {crumb.label}
                    </button>
                  </div>
                ))}
              </nav>
              {activeContainerId !== 0 && (
                <button
                  type="button"
                  onClick={() => setActiveContainerId(parentContainerId)}
                  className="flex items-center gap-1 rounded border border-slate-200 bg-slate-50 px-2 py-0.5 text-[11px] font-medium text-slate-600 hover:bg-slate-100 transition shadow-2xs"
                  title="Subir al contenedor padre"
                >
                  <CornerLeftUp className="size-3" />
                  Subir nivel
                </button>
              )}
            </div>

            <div className="min-h-0 flex-1">
              <GraphView
                model={model}
                actions={actions}
                fitSignal={fitSignal}
                containerId={activeContainerId}
                onOpenContainer={setActiveContainerId}
                focusBlockId={focusBlockId}
                focusToken={focusToken}
              />
            </div>
          </div>
          <aside className="flex w-[400px] shrink-0 flex-col overflow-hidden rounded-lg border border-slate-200 bg-white">
            <div className="flex gap-1 border-b border-slate-200 px-2 pt-2">
              {(
                [
                  { id: "bloque", label: "Bloque" },
                  { id: "bloques", label: "Bloques" },
                  { id: "arbol", label: "Árbol" },
                ] as const
              ).map((candidate) => (
                <button
                  key={candidate.id}
                  type="button"
                  data-monitor-aside={candidate.id}
                  onClick={() => setAside(candidate.id)}
                  className={[
                    "-mb-px rounded-t-md border-b-2 px-3 py-1.5 text-xs font-medium transition",
                    aside === candidate.id
                      ? "border-indigo-500 text-indigo-700"
                      : "border-transparent text-slate-500 hover:text-slate-800",
                  ].join(" ")}
                >
                  {candidate.label}
                </button>
              ))}
            </div>
            <div className="flex-1 overflow-auto p-4">
              {aside === "bloque" ? (
                <PropertiesPanel
                  block={selectedBlock}
                  catalogue={catalogue}
                  onSetProperty={(id, property, value) =>
                    void apply({ op: "setProperty", id, property, value })
                  }
                  onSetCaption={(id, caption) => void apply({ op: "setCaption", id, caption })}
                  onOpenContainer={setActiveContainerId}
                />
              ) : aside === "bloques" ? (
                <ClassList
                  catalogue={catalogue}
                  onAdd={(className) => void addBlock(className)}
                />
              ) : (
                <ModelTree
                  model={model}
                  activeContainerId={activeContainerId}
                  selectedBlockIds={selectedBlockIds}
                  onSelectContainer={(id) => setActiveContainerId(id)}
                  onSelectBlock={(id) => {
                    // Navigate to the block's container if it is inside one
                    const targetBlock = model.blocks.find((b) => b.id === id);
                    if (targetBlock && targetBlock.container !== activeContainerId) {
                      setActiveContainerId(targetBlock.container);
                    }
                    select([id]);
                    // The canvas follows the block the reader picked: the
                    // picture moves to it and is not framed again.
                    focusBlock(id);
                    setAside("bloque");
                  }}
                  onAddContainer={() => void addBlock("Container")}
                />
              )}
            </div>
          </aside>
        </div>
      )}
      {panel === "graficos" &&
        (monitor === null || report === null ? (
          <p className="flex flex-1 items-center justify-center rounded-lg border border-dashed border-slate-300 bg-white/60 text-sm text-slate-500">
            Ejecuta la simulación para ver sus gráficos.
          </p>
        ) : (
          <MonitorCharts report={report} monitor={monitor} state={state} model={model} />
        ))}
      {panel === "reporte" &&
        (report === null ? (
          <p className="flex flex-1 items-center justify-center rounded-lg border border-dashed border-slate-300 bg-white/60 text-sm text-slate-500">
            Ejecuta la simulación para ver su reporte.
          </p>
        ) : (
          <ReportView report={report} state={state} figures={state?.figures ?? null} />
        ))}
      {/* The status line lives at the foot of the window: a discreet line
          under the picture, not a band that takes the height of the graph.
          It is the only band of the app: the diagnostics, the failure of
          the last run, the confirmation of a save and the question of the
          engine are all said here. */}
      <StatusLine
        diagnostics={diagnostics}
        failure={
          error !== null
            ? { message: error, kind: "error" }
            : notice !== null
              ? { message: notice, kind: "notice" }
              : null
        }
        saved={savedMessage}
        pendingSave={pendingSave}
        onConfirmSave={() => void confirmSave()}
        onDismissSave={dismissSave}
        onClearFailure={() => {
          if (error !== null) useApp.getState().clearError();
          if (notice !== null) useApp.getState().clearNotice();
        }}
      />
    </div>
  );
}

/**
 * The state of the run at the instant the replay is at.
 *
 * The queues, the facilities and the storages are the state the trace
 * rebuilds; the numbers the report publishes (the content mean, the
 * utilization) are **not** here: the engine computes them and the document
 * does not carry them.
 */
