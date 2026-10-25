import { useEffect, useMemo, useRef, useState } from "react";
import { number } from "../format";
import { advance, advanceClock, frameAt, stateAt, type ReplayState } from "../monitor/replay";
import { useApp } from "../state/store";
import { FLIGHT_SECONDS, travellersAt, type Move } from "../monitor/balls";
import { graphOf, routeOf, visibleLinks } from "../graph/graph";
import { countsOf } from "../monitor/counts";
import { useLive, type LiveBall, type LiveCounts } from "../monitor/live";
import ClassList from "./ClassList";
import DiagnosticsPanel from "./DiagnosticsPanel";
import GraphView, { type GraphActions } from "./GraphView";
import MonitorCharts from "./MonitorCharts";
import PropertiesPanel from "./PropertiesPanel";
import ReportView from "./ReportView";

/** The part of the body the window shows. */
type Panel = "reproduccion" | "graficos" | "reporte";

/** The part of the lateral panel the window shows. */
type Aside = "bloque" | "estado" | "bloques";

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
    catalogue,
    diagnostics,
    addBlock,
    setPlayhead,
    setPlaying,
    apply,
  } = useApp();
  const [panel, setPanel] = useState<Panel>("reproduccion");
  const [aside, setAside] = useState<Aside>("estado");
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

  // The links of the model: a ball travels over the one that joins the two
  // blocks it goes from and to.
  const edges = useMemo(() => (model === null ? [] : graphOf(model).edges), [model]);

  // The links the model does not draw but a ball travels over: the direct
  // way between two blocks that the model joins through a container.
  const links = useMemo(
    () => visibleLinks(transitions?.moves ?? [], edges),
    [transitions, edges],
  );

  // What the animation needs to publish the picture, kept up to date without
  // restarting the animation.
  const live = useRef({ counts, edges, moves: [] as Move[], times: [] as number[] });
  live.current = {
    counts,
    edges,
    moves: (transitions?.moves ?? []) as Move[],
    times: transitions?.times ?? [],
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
    )) {
      const id = routeOf(current.edges, traveller.from, traveller.to);
      const list = balls.get(id);
      const ball: LiveBall = { xact: traveller.xact, progress: traveller.progress };
      if (list === undefined) {
        balls.set(id, [ball]);
      } else {
        list.push(ball);
      }
    }
    useLive.getState().publish({ clock, counts: current.counts, balls });
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
      const elapsed = (now - last) / 1000;
      last = now;
      const current = useApp.getState().playhead;
      // The reader moved the playhead (the slider of the header): the clock
      // is already where the reader put it, so it is not re-seeded from the
      // record. The playhead the monitor set itself is remembered, so the
      // clock is not re-seeded on every frame it moves.
      if (clockRef.current === null || current !== setRef.current) {
        clockRef.current = useLive.getState().clock;
        setRef.current = current;
        publish(clockRef.current);
      }
      const clock = advanceClock(clockRef.current, elapsed, rate);
      // The clock stops at the end of the run: a run that is over does not
      // keep counting, and the picture is the picture of its last instant.
      const reached = clock >= monitor.endTime;
      clockRef.current = reached ? monitor.endTime : clock;
      // The clock the reader sees follows the animation, so it ticks instead
      // of jumping from one record to the next.
      publish(clockRef.current);
      if (reached) {
        setPlayhead(monitor.frames.length - 1);
        setPlaying(false);
        return;
      }
      const frame = frameAt(monitor, clock);
      if (frame !== current) {
        setRef.current = frame;
        setPlayhead(frame);
      }
      handle = requestAnimationFrame(step);
    };
    handle = requestAnimationFrame(step);
    return () => cancelAnimationFrame(handle);
  }, [playing, monitor, speed, setPlayhead, setPlaying]);

  // A new run starts its clock again, and going back to the start does too.
  useEffect(() => {
    clockRef.current = null;
    setRef.current = -1;
    useLive.getState().reset();
  }, [monitor]);

  // The picture follows the state of the replay.
  //
  // The slider of the header moves the run while the reproduction is
  // paused, and the picture has to move with it: this effect publishes the
  // picture again every time the state of the replay changes.
  //
  // The clock is the one the live layer holds — the animation publishes it
  // on every frame — and it is read without subscribing, so the window is
  // not redrawn by this effect. The animation publishes anyway, and the
  // live layer keeps the identity of what did not change: a second
  // publication with the same numbers is harmless.
  useEffect(() => {
    publish(useLive.getState().clock);
  }, [state]);

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
          <div className="min-w-0 flex-1 rounded-lg border border-slate-200 bg-white p-2">
            <GraphView
              model={model}
              actions={actions}
              fitSignal={fitSignal}
              extraEdges={links}
            />
          </div>
          <aside className="flex w-80 shrink-0 flex-col overflow-hidden rounded-lg border border-slate-200 bg-white">
            <div className="flex gap-1 border-b border-slate-200 px-2 pt-2">
              {(
                [
                  { id: "bloque", label: "Bloque" },
                  { id: "estado", label: "Estado" },
                  { id: "bloques", label: "Bloques" },
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
                />
              ) : aside === "estado" ? (
                <StatePanel state={state} />
              ) : (
                <ClassList
                  catalogue={catalogue}
                  onAdd={(className) => void addBlock(className)}
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
          <MonitorCharts report={report} monitor={monitor} figures={state?.figures ?? null} />
        ))}
      {panel === "reporte" &&
        (report === null ? (
          <p className="flex flex-1 items-center justify-center rounded-lg border border-dashed border-slate-300 bg-white/60 text-sm text-slate-500">
            Ejecuta la simulación para ver su reporte.
          </p>
        ) : (
          <ReportView report={report} state={state} figures={state?.figures ?? null} />
        ))}
      {/* The diagnostics live at the foot of the window: a discreet line
          under the picture, not a band that takes the height of the graph. */}
      <DiagnosticsPanel diagnostics={diagnostics} />
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
function StatePanel({ state }: { state: ReplayState | null }) {
  if (state === null) {
    return <p className="text-xs text-slate-500">Sin estado.</p>;
  }
  const queues = Object.entries(state.queues);
  const facilities = Object.entries(state.facilities);
  const storages = Object.entries(state.storages);
  return (
    <div className="flex flex-col gap-4 text-xs">
      <section>
        <h3 className="mb-1.5 text-[11px] font-semibold uppercase tracking-wide text-slate-500">
          Colas
        </h3>
        {queues.length === 0 ? (
          <p className="text-slate-400">Ninguna.</p>
        ) : (
          <ul className="flex flex-col gap-1">
            {queues.map(([name, content]) => (
              <li key={name} className="flex items-baseline justify-between gap-2">
                <span className="truncate text-slate-700">{name}</span>
                <span className="font-mono text-slate-900">{number(content)}</span>
              </li>
            ))}
          </ul>
        )}
      </section>
      <section>
        <h3 className="mb-1.5 text-[11px] font-semibold uppercase tracking-wide text-slate-500">
          Facilities
        </h3>
        {facilities.length === 0 ? (
          <p className="text-slate-400">Ninguna.</p>
        ) : (
          <ul className="flex flex-col gap-1">
            {facilities.map(([name, owner]) => (
              <li key={name} className="flex items-baseline justify-between gap-2">
                <span className="truncate text-slate-700">{name}</span>
                <span className="font-mono text-slate-900">
                  {owner === null ? "libre" : `tx ${owner}`}
                </span>
              </li>
            ))}
          </ul>
        )}
      </section>
      {storages.length > 0 && (
        <section>
          <h3 className="mb-1.5 text-[11px] font-semibold uppercase tracking-wide text-slate-500">
            Almacenes
          </h3>
          <ul className="flex flex-col gap-1">
            {storages.map(([name, units]) => (
              <li key={name} className="flex items-baseline justify-between gap-2">
                <span className="truncate text-slate-700">{name}</span>
                <span className="font-mono text-slate-900">{number(units)}</span>
              </li>
            ))}
          </ul>
        </section>
      )}
    </div>
  );
}
