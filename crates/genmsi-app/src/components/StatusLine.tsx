import { useEffect, useRef, useState, type CSSProperties } from "react";
import type { Diagnostics } from "../types";
import { adviceFor } from "../engine/reasons";

/**
 * The status line of the app: the one band that says what is going on.
 *
 * Everything the app has to say —what the model compiles to, that a run
 * failed, that the model was saved, that the engine refuses to save it— is
 * said **here**, at the foot of the window, and nowhere else. There is no
 * band of messages at the top: the top belongs to the header and to the
 * toolbar, and a message that arrives while the reader is looking at the
 * graph belongs where the graph ends.
 *
 * The band that needs the eye says so with a halo: its light pulses three
 * times, because the reader may be looking at the graph and not at the foot
 * of the window.
 */
export default function StatusLine({
  diagnostics,
  failure,
  saved,
  pendingSave,
  onConfirmSave,
  onDismissSave,
}: {
  /** What the model in memory compiles to. */
  diagnostics: Diagnostics | null;
  /** The failure of the last run, if it failed. */
  failure: { message: string; kind: "error" | "notice" } | null;
  /** The confirmation of the last save, while it lasts. */
  saved: string | null;
  /** The path of a save that waits for the reader's confirmation. */
  pendingSave: string | null;
  /** Answer the confirmation of a save with yes. */
  onConfirmSave: () => void;
  /** Answer the confirmation of a save with no. */
  onDismissSave: () => void;
}) {
  // The light: it starts again when a new thing that needs the eye
  // arrives, and when the same thing comes back.
  const [flash, setFlash] = useState(0);
  const last = useRef<string | null>(null);
  const urgent = pendingSave ?? failure?.message ?? null;
  useEffect(() => {
    if (urgent === null) {
      last.current = null;
      return;
    }
    if (urgent !== last.current) {
      last.current = urgent;
      setFlash((count) => count + 1);
    }
  }, [urgent]);

  // The question of the engine comes first: it is the only band that waits
  // for an answer.
  if (pendingSave !== null) {
    return (
      <div
        key={flash}
        role="alert"
        style={{ "--alert-ring": "rgba(217, 119, 6, 0.3)" } as CSSProperties}
        className="alert-flash flex items-center gap-2 rounded-lg border border-amber-300 bg-amber-50 px-3 py-2 text-xs text-amber-900"
      >
        <p className="flex-1">
          El motor no compila este modelo. ¿Guardarlo de todos modos?
        </p>
        <button
          type="button"
          className="rounded-md bg-amber-600 px-2 py-1 font-medium text-white hover:bg-amber-500"
          onClick={onConfirmSave}
        >
          Guardar
        </button>
        <button type="button" className="px-1 text-amber-800" onClick={onDismissSave}>
          Cancelar
        </button>
      </div>
    );
  }

  if (failure !== null) {
    const failed = failure.kind === "error";
    return (
      <div
        key={flash}
        role="alert"
        style={
          {
            "--alert-ring": failed
              ? "rgba(225, 29, 72, 0.3)"
              : "rgba(2, 132, 199, 0.3)",
          } as CSSProperties
        }
        className={`alert-flash rounded-lg border px-3 py-2 text-xs ${
          failed
            ? "border-rose-200 bg-rose-50 text-rose-900"
            : "border-sky-200 bg-sky-50 text-sky-900"
        }`}
      >
        {failed && <p className="font-semibold">La corrida falló</p>}
        <p className={failed ? "mt-0.5" : undefined}>{failure.message}</p>
      </div>
    );
  }

  if (saved !== null) {
    return (
      <div className="rounded-lg border border-emerald-200 bg-emerald-50 px-3 py-1.5 text-xs text-emerald-900">
        {saved}
      </div>
    );
  }

  if (diagnostics === null) {
    return null;
  }

  if (diagnostics.status === "error") {
    // The reason the engine knows is said in the words of the app, with
    // the remedy; a reason the app does not know is the engine's own
    // message, which is never hidden.
    const advice =
      diagnostics.message === null ? null : adviceFor(diagnostics.message);
    return (
      <div className="rounded-lg border border-rose-200 bg-rose-50 px-3 py-2 text-xs text-rose-900">
        <p className="font-semibold">El modelo no compila</p>
        {advice !== null ? (
          <p className="mt-0.5">{advice}</p>
        ) : (
          <p className="mt-0.5 font-mono">{diagnostics.message}</p>
        )}
      </div>
    );
  }

  if (diagnostics.status === "diagnostics") {
    return (
      <div className="rounded-lg border border-amber-200 bg-amber-50 px-3 py-2 text-xs text-amber-900">
        <p className="font-semibold">El modelo compila con observaciones</p>
        <ul className="mt-1 list-disc pl-4">
          {diagnostics.diagnostics.map((diagnostic, index) => (
            <li key={index} className="font-mono">
              {diagnostic}
            </li>
          ))}
        </ul>
      </div>
    );
  }

  return (
    <div className="rounded-lg border border-emerald-200 bg-emerald-50 px-3 py-1.5 text-xs text-emerald-900">
      El modelo compila sin observaciones.
    </div>
  );
}
