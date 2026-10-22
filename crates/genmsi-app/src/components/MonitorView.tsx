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
import MonitorCanvas from "./MonitorCanvas";
import MonitorCharts from "./MonitorCharts";



/** The part of the monitor the body shows. */
type Panel = "reproduccion" | "graficos";

/**
 * The monitor: the replay of the last run and the charts of the run.
 *
 * The monitor is a tab of its own (the phase's decision): the graph stays
 * clean and the replay has its own space, with the clock and the metrics.
 *
 * The replay is a **projection** of the trace the engine published: the
 * clock, the content of the queues, the owner of the facilities and the
 * transactions inside every block are rebuilt by walking the trace. Nothing
 * is recomputed: no metric the engine computes is derived here.
 */
export default function MonitorView() {
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
    runMonitor,
    setPlayhead,
    setPlaying,
    setSpeed,
  } = useApp();
  const [panel, setPanel] = useState<Panel>("reproduccion");
  // The state of the replay at the playhead. It is carried from one frame to
  // the next while the replay runs forward, and rebuilt when it goes back.
  const carried = useRef<{ run: unknown; frame: number; state: ReplayState } | null>(null);

  // The clock of the replay: the time the run has reached, kept up to date by
  // the animation. It used to be the time of the last record, which made it
  // jump from one event to the next instead of ticking.
  const [clock, setClock] = useState(0);

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

  // The animation: the clock moves at a constant rate and the playhead
  // follows it. The index is read, never walked again.
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
        setClock(clockRef.current);
      }
      const clock = advanceClock(clockRef.current, elapsed, rate);
      clockRef.current = clock;
      // The clock the reader sees follows the animation, so it ticks instead
      // of jumping from one record to the next.
      setClock(clock);
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
    setClock(0);
  }, [monitor]);
  const backToStart = () => {
    clockRef.current = null;
    setRef.current = -1;
    setClock(monitor?.startTime ?? 0);
    setPlayhead(0);
    setPlaying(false);
  };

  // The transactions inside every block of the model: the program blocks of a
  // block (a facility use expands into several) are added up.
  const activity = useMemo(() => {
    const byNode = new Map<number, number>();
    if (report === null || state === null) {
      return byNode;
    }
    for (const block of report.blocks) {
      const count = state.blocks[block.line - 1] ?? 0;
      byNode.set(block.node, (byNode.get(block.node) ?? 0) + count);
    }
    return byNode;
  }, [report, state]);

  // The entries every block has had since the run started, the same way: the
  // program blocks of a block are added up. This is the total the report
  // publishes, rebuilt from the trace.
  const totals = useMemo(() => {
    const byNode = new Map<number, number>();
    if (report === null || state === null) {
      return byNode;
    }
    for (const block of report.blocks) {
      const count = state.totals[block.line - 1] ?? 0;
      byNode.set(block.node, (byNode.get(block.node) ?? 0) + count);
    }
    return byNode;
  }, [report, state]);

  // The class of every block of the model, so the canvas can tell a queue
  // from a facility and show what the block is about.
  const classes = useMemo(() => {
    const byNode = new Map<number, string>();
    if (report === null) {
      return byNode;
    }
    for (const block of report.blocks) {
      byNode.set(block.node, block.class);
    }
    return byNode;
  }, [report]);

  if (model === null) {
    return (
      <div className="flex flex-1 items-center justify-center rounded-lg border border-dashed border-slate-300 bg-white/60 text-sm text-slate-500">
        Selecciona un modelo para ver su reproducción.
      </div>
    );
  }

  if (monitor === null) {
    return (
      <div className="flex flex-1 flex-col items-center justify-center gap-3 rounded-lg border border-dashed border-slate-300 bg-white/60 text-sm text-slate-500">
        <p>
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
      </div>
    );
  }

  const truncated = trace?.truncated ?? monitor.truncated;

  return (
    <div className="flex h-full flex-col gap-3 overflow-hidden">
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
          Registros{" "}
          <span className="font-mono text-slate-800">{monitor.records}</span>
        </span>
        {/* One clock: the time of the run. The wall clock the reader spends
            on it is told apart by the scale, which says how much of the run
            a second of screen is. */}
        <span className="text-slate-500">
          Reloj{" "}
          <span className="font-mono text-slate-800">{chronometer(clock)}</span>
        </span>
      </div>

      {truncated && (
        <p className="rounded-lg border border-amber-200 bg-amber-50 px-3 py-1.5 text-xs text-amber-900">
          La traza se cortó en el límite de {monitor.limit} registros: la
          reproducción muestra sólo esa parte de la ejecución.
        </p>
      )}

      <div className="flex flex-wrap items-center gap-3 rounded-lg border border-slate-200 bg-white px-4 py-2">
        <button
          type="button"
          onClick={() => setPlaying(!playing)}
          aria-label={playing ? "Pausar la reproducción" : "Reproducir la traza"}
          className="flex items-center gap-1.5 rounded-md bg-indigo-600 px-3 py-1.5 text-xs font-medium text-white shadow-sm transition hover:bg-indigo-500"
        >
          {playing ? <Pause className="size-3.5" aria-hidden /> : <Play className="size-3.5" aria-hidden />}
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
        <input
          type="range"
          aria-label="Posición de la reproducción"
          min={monitor.startTime}
          max={monitor.endTime}
          step={(monitor.endTime - monitor.startTime) / 1000}
          // The slider is a clock: its ends are the times of the run and its
          // value is the time of the replay.
          value={clock}
          onChange={(event) => {
            setPlaying(false);
            // The slider is a clock: the reader picks a time and the replay
            // goes to the last record at or before it.
            const time = Number(event.target.value);
            clockRef.current = time;
            setRef.current = frameAt(monitor, time);
            setClock(time);
            setPlayhead(frameAt(monitor, time));
          }}
          className="min-w-40 flex-1"
        />
        <span className="font-mono text-[11px] text-slate-500">
          {chronometer(clock)} / {chronometer(monitor.endTime)}
        </span>
      </div>

      <div className="flex gap-1 border-b border-slate-200">
        {(
          [
            { id: "reproduccion", label: "Reproducción" },
            { id: "graficos", label: "Gráficos" },
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

      {panel === "reproduccion" ? (
        <div className="flex min-h-0 flex-1 gap-3">
          <div className="min-w-0 flex-1 rounded-lg border border-slate-200 bg-white p-2">
            <MonitorCanvas
              model={model}
              activity={activity}
              totals={totals}
              classes={classes}
              moves={transitions?.moves ?? []}
              moveTimes={transitions?.times ?? []}
              clock={clock}
              speed={speed}
            />
          </div>
          <aside className="w-80 shrink-0 overflow-auto rounded-lg border border-slate-200 bg-white p-4">
            <StatePanel state={state} />
          </aside>
        </div>
      ) : (
        <MonitorCharts report={report} monitor={monitor} />
      )}
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
