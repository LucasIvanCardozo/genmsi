import { useEffect, useMemo, useRef, useState } from "react";
import { Pause, Play, RotateCcw } from "lucide-react";
import { chronometer, number } from "../format";
import {
  SPEEDS,
  advance,
  advanceClock,
  frameAt,
  stateAt,
  type ReplayState,
} from "../monitor/replay";
import { useApp } from "../state/store";
import { FLIGHT_SECONDS, travellersAt, type Move } from "../monitor/balls";
import { graphOf, routeOf, visibleLinks } from "../graph/graph";
import { useLive, type LiveBall, type LiveCounts } from "../monitor/live";
import DiagnosticsPanel from "./DiagnosticsPanel";
import GraphView, { type GraphActions } from "./GraphView";
import ModelSummary from "./ModelSummary";
import MonitorCharts from "./MonitorCharts";
import PropertiesPanel from "./PropertiesPanel";
import ReportView from "./ReportView";
import type { ReplayIndex } from "../monitor/replay";

/** The part of the body the window shows. */
type Panel = "reproduccion" | "graficos" | "reporte";

/** The part of the lateral panel the window shows. */
type Aside = "bloque" | "estado";

/**
 * The window of the simulation: the model, its run and its report.
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
    status,
    selectedBlockIds,
    catalogue,
    diagnostics,
    runMonitor,
    setPlayhead,
    setPlaying,
    setSpeed,
    apply,
  } = useApp();
  const [panel, setPanel] = useState<Panel>("reproduccion");
  const [aside, setAside] = useState<Aside>("estado");
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
  const counts = useMemo(() => {
    const byNode = new Map<number, LiveCounts>();
    if (report === null || state === null) {
      return byNode;
    }
    for (const block of report.blocks) {
      const before = byNode.get(block.node);
      byNode.set(block.node, {
        now: (before?.now ?? 0) + (state.blocks[block.line - 1] ?? 0),
        ever: (before?.ever ?? 0) + (state.totals[block.line - 1] ?? 0),
      });
    }
    return byNode;
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
  // the run had reached. Going back to the start is the one thing that moves
  // it, and it does so explicitly.
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
      // The reader moved the playhead (the slider, or "Al inicio"): the clock
      // follows it instead of running on from where it was. The playhead the
      // monitor set itself is remembered, so the clock is not re-seeded on
      // every frame it moves.
      if (clockRef.current === null || current !== setRef.current) {
        clockRef.current = monitor.frames[current]?.time ?? monitor.startTime;
        setRef.current = current;
        publish(clockRef.current);
      }
      const clock = advanceClock(clockRef.current, elapsed, rate);
      clockRef.current = clock;
      // The clock the reader sees follows the animation, so it ticks instead
      // of jumping from one record to the next.
      publish(clock);
      if (clock >= monitor.endTime) {
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
  const backToStart = () => {
    clockRef.current = null;
    setRef.current = -1;
    setPlayhead(0);
    setPlaying(false);
    if (monitor !== null) {
      publish(monitor.startTime);
    }
  };

  // The reader moves the clock: the replay goes to the last record at or
  // before that instant.
  const seek = (time: number) => {
    if (monitor === null) {
      return;
    }
    setPlaying(false);
    clockRef.current = time;
    setRef.current = frameAt(monitor, time);
    setPlayhead(frameAt(monitor, time));
    publish(time);
  };

  if (model === null) {
    return (
      <div className="flex flex-1 items-center justify-center rounded-lg border border-dashed border-slate-300 bg-white/60 text-sm text-slate-500">
        Selecciona un modelo para ver su reproducción.
      </div>
    );
  }

  const truncated = trace?.truncated ?? monitor?.truncated ?? false;
  const selectedBlock =
    model.blocks.find((block) => block.id === selectedBlockIds[0]) ?? null;

  return (
    <div className="flex h-full flex-col gap-3 overflow-hidden">
      <ModelSummary diagnostics={diagnostics} />
      <DiagnosticsPanel diagnostics={diagnostics} />
      {monitor !== null && (
        <div className="flex flex-wrap items-baseline gap-x-6 gap-y-1 rounded-lg border border-slate-200 bg-white px-4 py-3 text-xs">
          <span className="text-sm font-semibold text-slate-800">{report?.title ?? ""}</span>
          <span className="text-slate-500">
            Semilla <span className="font-mono text-slate-800">{report?.seed ?? 1}</span>
          </span>
          <span className="text-slate-500">
            Tiempos{" "}
            <span className="font-mono text-slate-800">
              {number(monitor.startTime)} → {number(monitor.endTime)}
            </span>
          </span>
          <span className="text-slate-500">
            Registros <span className="font-mono text-slate-800">{monitor.records}</span>
          </span>
          {/* One clock: the time of the run. The wall clock the reader spends
              on it is told apart by the scale, which says how much of the run
              a second of screen is. */}
          <span className="text-slate-500">
            Reloj <Clock />
          </span>
        </div>
      )}

      {truncated && (
        <p className="rounded-lg border border-amber-200 bg-amber-50 px-3 py-1.5 text-xs text-amber-900">
          La traza se cortó en el límite de {monitor?.limit} registros: la
          reproducción muestra sólo esa parte de la ejecución.
        </p>
      )}

      <div className="flex flex-wrap items-center gap-3 rounded-lg border border-slate-200 bg-white px-4 py-2">
        {monitor === null ? (
          <>
            <p className="text-xs text-slate-500">
              La reproducción necesita la traza de una ejecución: el motor la
              publica junto al reporte.
            </p>
            <button
              type="button"
              onClick={() => void runMonitor()}
              disabled={status === "running"}
              className="flex items-center gap-1.5 rounded-md bg-indigo-600 px-3 py-1.5 text-xs font-medium text-white shadow-sm transition hover:bg-indigo-500 disabled:cursor-not-allowed disabled:bg-slate-300"
            >
              <Play className="size-3.5" aria-hidden />
              {status === "running" ? "Ejecutando…" : "Ejecutar y reproducir"}
            </button>
          </>
        ) : (
          <>
            <button
              type="button"
              onClick={() => setPlaying(!playing)}
              aria-label={playing ? "Pausar la reproducción" : "Reproducir la traza"}
              className="flex items-center gap-1.5 rounded-md bg-indigo-600 px-3 py-1.5 text-xs font-medium text-white shadow-sm transition hover:bg-indigo-500"
            >
              {playing ? (
                <Pause className="size-3.5" aria-hidden />
              ) : (
                <Play className="size-3.5" aria-hidden />
              )}
              {playing ? "Pausar" : "Reproducir"}
            </button>
            <button
              type="button"
              onClick={backToStart}
              aria-label="Volver al principio"
              className="flex items-center gap-1.5 rounded-md border border-slate-300 bg-white px-2.5 py-1.5 text-xs font-medium text-slate-700 transition hover:bg-slate-50"
            >
              <RotateCcw className="size-3.5" aria-hidden />
              Al inicio
            </button>
            <label className="flex items-center gap-1.5 text-xs text-slate-500">
              Velocidad
              <select
                aria-label="Velocidad"
                value={speed}
                onChange={(event) => setSpeed(Number(event.target.value))}
                className="rounded-md border border-slate-300 bg-white px-2 py-1 text-xs font-mono text-slate-800 outline-none focus:border-slate-500"
              >
                {SPEEDS.map((option) => (
                  <option key={option} value={option}>
                    {option}×
                  </option>
                ))}
              </select>
              {/* How long the playback lasts at this speed: the one thing the
                  speed does not say on its own. */}
              <span className="font-mono text-[11px] text-slate-400">
                dura {chronometer((monitor.endTime - monitor.startTime) / speed)}
              </span>
            </label>
            <Slider monitor={monitor} onSeek={seek} />
            <span className="font-mono text-[11px] text-slate-500">
              <Clock /> / {chronometer(monitor.endTime)}
            </span>
          </>
        )}
      </div>

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
              ) : (
                <StatePanel state={state} />
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
          <MonitorCharts report={report} monitor={monitor} />
        ))}
      {panel === "reporte" &&
        (report === null ? (
          <p className="flex flex-1 items-center justify-center rounded-lg border border-dashed border-slate-300 bg-white/60 text-sm text-slate-500">
            Ejecuta la simulación para ver su reporte.
          </p>
        ) : (
          <ReportView report={report} state={state} figures={state?.figures ?? null} />
        ))}
    </div>
  );
}

/** The clock of the replay, as the reader sees it. */
function Clock() {
  const clock = useLive((state) => state.clock);
  return <span className="font-mono text-slate-800">{chronometer(clock)}</span>;
}

/**
 * The slider of the clock.
 *
 * It is a clock: its ends are the times of the run and its value is the time
 * of the replay. It reads the clock from the live layer, so it follows the
 * animation without the window redrawing itself.
 */
function Slider({
  monitor,
  onSeek,
}: {
  monitor: ReplayIndex;
  onSeek: (time: number) => void;
}) {
  const clock = useLive((state) => state.clock);
  return (
    <input
      type="range"
      aria-label="Posición de la reproducción"
      min={monitor.startTime}
      max={monitor.endTime}
      step={(monitor.endTime - monitor.startTime) / 1000}
      value={clock}
      onChange={(event) => onSeek(Number(event.target.value))}
      className="min-w-40 flex-1"
    />
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
