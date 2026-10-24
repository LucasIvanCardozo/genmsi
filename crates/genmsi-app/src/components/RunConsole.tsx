import { useState } from "react";
import { Pause, Play, RotateCcw, Square } from "lucide-react";
import { chronometer } from "../format";
import { useLive } from "../monitor/live";
import { SPEEDS, frameAt } from "../monitor/replay";
import { useApp } from "../state/store";
import { isTauri } from "../engine";

/**
 * What the cancel button says about what it can do.
 *
 * The engine cannot be interrupted yet: the app stops waiting for the run
 * and throws its answer away. The button says so, because a button that
 * says "cancel" and does not cancel is a lie.
 */
const CANCEL_EXPLANATION =
  "La app deja de esperar la corrida y descarta su resultado. El motor todavía no puede " +
  "interrumpir una simulación en marcha: sigue trabajando hasta que termina.";

/**
 * The note next to the cancel button.
 *
 * It is short enough to sit in the header and says the whole truth: the
 * engine cannot stop the run yet.
 */
const CANCEL_NOTE = "el motor no puede interrumpirla todavía";

/**
 * The console of the run: the tools that drive a run, in one row.
 *
 * The console lives in the header, so the tools are always there: without a
 * run the play, the speed and the slider are disabled and the clock reads
 * zero. With a run, the reader can simulate the model, run it, play or pause
 * the replay, change its speed and move it with the slider.
 *
 * The clock is read from the live layer with a selector, and nothing else of
 * that layer is subscribed to: the console redraws when the clock ticks, but
 * the window does not.
 */
export default function RunConsole() {
  const monitor = useApp((state) => state.monitor);
  const playing = useApp((state) => state.playing);
  const speed = useApp((state) => state.speed);
  const status = useApp((state) => state.status);
  // A model is open when it is in memory: while it is loading there is
  // nothing to run, and the buttons wait.
  const hasModel = useApp((state) => state.model !== null);
  const loading = status === "loading";
  const simulate = useApp((state) => state.simulate);
  const runMonitor = useApp((state) => state.runMonitor);
  const cancelRun = useApp((state) => state.cancelRun);
  const cancelling = useApp((state) => state.cancelling);
  const setPlayhead = useApp((state) => state.setPlayhead);
  const setPlaying = useApp((state) => state.setPlaying);
  const setSpeed = useApp((state) => state.setSpeed);
  // The clock, and only the clock: the live layer is read through a selector
  // so the window is not redrawn sixty times a second.
  const clock = useLive((state) => state.clock);
  const setClock = useLive((state) => state.setClock);

  // Which button is running: the two actions share the status of the app,
  // and the reader has to see which one is busy.
  const [busy, setBusy] = useState<"simular" | "ejecutar" | null>(null);
  const running = status === "running";
  // Only the desktop shell can stop a run: the bridge of the development
  // server cannot throw the answer of a run away.
  const canCancel = isTauri();

  // The run: its times are the ends of the slider and the span that says
  // how long the playback lasts.
  const span = monitor === null ? 0 : monitor.endTime - monitor.startTime;
  // Without a run the clock reads zero: there is nothing to show.
  const shown = monitor === null ? 0 : clock;

  const onSimulate = async () => {
    setBusy("simular");
    try {
      await simulate();
    } finally {
      setBusy(null);
    }
  };

  const onRun = async () => {
    setBusy("ejecutar");
    try {
      await runMonitor();
    } finally {
      setBusy(null);
    }
  };

  /**
   * The reader moves the run: the clock goes to the instant that was picked
   * and the replay goes to the record at or before it.
   *
   * The replay is paused first: the reader took the wheel, so the animation
   * stops and does not carry the clock on from where it was.
   */
  const seek = (time: number) => {
    if (monitor === null) {
      return;
    }
    setPlaying(false);
    setClock(time);
    setPlayhead(frameAt(monitor, time));
  };

  /** The run goes back to its beginning. */
  const backToStart = () => {
    if (monitor === null) {
      return;
    }
    setPlaying(false);
    setClock(monitor.startTime);
    setPlayhead(0);
  };

  return (
    <div className="flex items-center gap-3">
      <button
        type="button"
        onClick={() => void onSimulate()}
        disabled={!hasModel || running || loading || busy !== null}
        className="flex items-center gap-1.5 rounded-md border border-slate-300 bg-white px-2.5 py-1.5 text-xs font-medium text-slate-700 shadow-sm transition hover:bg-slate-50 disabled:cursor-not-allowed disabled:text-slate-400"
      >
        <Play className="size-3.5" aria-hidden />
        {busy === "simular" ? "Simulando…" : "Simular"}
      </button>

      <button
        type="button"
        onClick={() => void onRun()}
        disabled={!hasModel || running || loading || busy !== null}
        className="flex items-center gap-1.5 rounded-md bg-indigo-600 px-3 py-1.5 text-xs font-medium text-white shadow-sm transition hover:bg-indigo-500 disabled:cursor-not-allowed disabled:bg-slate-300"
      >
        <Play className="size-3.5" aria-hidden />
        {busy === "ejecutar" ? "Ejecutando…" : "Ejecutar"}
      </button>

      {/* The cancel button is there while the engine works: the run does not
          hold the window hostage any more, and the reader can say that he
          does not want the answer. */}
      {running && (
        <span className="flex items-center gap-1.5">
          <button
            type="button"
            onClick={() => void cancelRun()}
            disabled={!canCancel || cancelling}
            title={CANCEL_EXPLANATION}
            className="flex items-center gap-1.5 rounded-md border border-slate-300 bg-white px-2.5 py-1.5 text-xs font-medium text-slate-700 shadow-sm transition hover:bg-slate-50 disabled:cursor-not-allowed disabled:text-slate-400"
          >
            <Square className="size-3" aria-hidden />
            {cancelling ? "Cancelando…" : "Cancelar"}
          </button>
          <span className="text-[11px] text-slate-500" title={CANCEL_EXPLANATION}>
            {CANCEL_NOTE}
          </span>
        </span>
      )}

      <button
        type="button"
        onClick={() => setPlaying(!playing)}
        disabled={monitor === null}
        aria-label={playing ? "Pausar la reproducción" : "Reproducir la traza"}
        title={playing ? "Pausar la reproducción" : "Reproducir la traza"}
        className="flex size-8 items-center justify-center rounded-md border border-slate-300 bg-white text-slate-700 shadow-sm transition hover:bg-slate-50 disabled:cursor-not-allowed disabled:text-slate-400"
      >
        {playing ? (
          <Pause className="size-3.5" aria-hidden />
        ) : (
          <Play className="size-3.5" aria-hidden />
        )}
      </button>

      <button
        type="button"
        onClick={backToStart}
        disabled={monitor === null}
        aria-label="Volver al principio"
        title="Volver al principio"
        className="flex size-8 items-center justify-center rounded-md border border-slate-300 bg-white text-slate-700 shadow-sm transition hover:bg-slate-50 disabled:cursor-not-allowed disabled:text-slate-400"
      >
        <RotateCcw className="size-3.5" aria-hidden />
      </button>

      <label className="flex items-center gap-1.5 text-xs text-slate-500">
        <span className="sr-only">Velocidad</span>
        <select
          aria-label="Velocidad"
          value={speed}
          onChange={(event) => setSpeed(Number(event.target.value))}
          disabled={monitor === null}
          className="w-16 rounded-md border border-slate-300 bg-white px-1.5 py-1 text-xs font-mono text-slate-800 outline-none focus:border-slate-500 disabled:cursor-not-allowed disabled:text-slate-400"
        >
          {SPEEDS.map((option) => (
            <option key={option} value={option}>
              {option}×
            </option>
          ))}
        </select>
      </label>

      <span
        className="font-mono text-xs text-slate-800"
        title="El tiempo de la corrida en el que está la reproducción"
      >
        {chronometer(shown)}
      </span>

      <input
        type="range"
        aria-label="Posición de la reproducción"
        min={monitor === null ? 0 : monitor.startTime}
        max={monitor === null ? 0 : monitor.endTime}
        step={span > 0 ? span / 1000 : 1}
        value={shown}
        onChange={(event) => seek(Number(event.target.value))}
        disabled={monitor === null}
        className="w-32 accent-indigo-600 disabled:cursor-not-allowed"
      />
    </div>
  );
}
