import { useEffect, useRef, useState, type CSSProperties } from "react";
import type { Diagnostics } from "../types";
import { adviceFor } from "../engine/reasons";
import type { ProposalBand } from "../state/proposal";

/**
 * The status line of the app: the one band that says what is going on.
 *
 * Everything the app has to say —what the model compiles to, that a run
 * failed, that the model was saved, that the engine refuses to save it,
 * that the agent proposed something— is said **here**, at the foot of the
 * window, and nowhere else. There is no band of messages at the top: the
 * top belongs to the header and to the toolbar, and a message that
 * arrives while the reader is looking at the graph belongs where the graph
 * ends.
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
  proposal,
  onAcceptProposal,
  onDiscardProposal,
  onAdoptProposal,
  onClearFailure,
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
  /** The proposal of the agent, and what the reader can do about it. */
  proposal: ProposalBand;
  /** Accept the proposal: the model in memory goes to the file. */
  onAcceptProposal: () => void;
  /** Discard the proposal: the model that was there comes back. */
  onDiscardProposal: (note: string) => void;
  /** Put the proposal back on the canvas, after the reader undid it. */
  onAdoptProposal: () => void;
  /** Clear the failure or notice banner. */
  onClearFailure?: () => void;
}) {
  // The light: it starts again when a new thing that needs the eye
  // arrives, and when the same thing comes back.
  const [flash, setFlash] = useState(0);
  const last = useRef<string | null>(null);
  const proposalText = proposal.kind === "none" ? null : proposal.text;
  const urgent = pendingSave ?? failure?.message ?? proposalText ?? null;
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

  // The note the reader writes when they discard the proposal: it travels
  // with the rejection, and the agent reads it. It starts empty with every
  // proposal, because the note of one proposal is not the note of another.
  const [note, setNote] = useState("");
  const proposalId = proposal.kind === "none" ? null : proposal.id;
  useEffect(() => {
    setNote("");
  }, [proposalId]);

  // The question of the engine comes first: it is the band that blocks
  // the save, and the app waits for an answer.
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
        <div className="flex items-start justify-between gap-2">
          <div>
            {failed && <p className="font-semibold">La corrida falló</p>}
            <p className={failed ? "mt-0.5" : undefined}>{failure.message}</p>
          </div>
          {onClearFailure && (
            <button
              type="button"
              onClick={onClearFailure}
              className={`rounded px-1 text-xs font-semibold hover:opacity-75 ${
                failed ? "text-rose-700" : "text-sky-700"
              }`}
              title="Cerrar aviso"
              aria-label="Cerrar aviso"
            >
              ×
            </button>
          )}
        </div>
      </div>
    );
  }

  // The proposal of the agent: it waits for the person too, and it says
  // what the agent did before it asks anything.
  if (proposal.kind !== "none") {
    const onCanvas = proposal.kind === "onCanvas";
    const waiting = proposal.kind === "waiting";
    return (
      <div
        key={flash}
        role="alert"
        style={{ "--alert-ring": "rgba(217, 119, 6, 0.3)" } as CSSProperties}
        className="alert-flash flex flex-wrap items-center gap-2 rounded-lg border border-amber-300 bg-amber-50 px-3 py-2 text-xs text-amber-900"
      >
        <p className="flex-1">{proposal.text}</p>
        {onCanvas && (
          <>
            <input
              type="text"
              value={note}
              onChange={(event) => setNote(event.target.value)}
              placeholder="¿Por qué lo descartás? (opcional)"
              aria-label="Por qué se descarta la propuesta"
              className="w-48 rounded-md border border-amber-300 bg-white px-2 py-1 text-xs text-amber-900 placeholder:text-amber-600"
            />
            <button
              type="button"
              className="rounded-md bg-amber-600 px-2 py-1 font-medium text-white hover:bg-amber-500"
              onClick={onAcceptProposal}
            >
              Aceptar
            </button>
            <button
              type="button"
              className="px-1 text-amber-800"
              onClick={() => onDiscardProposal(note)}
            >
              Descartar
            </button>
          </>
        )}
        {!onCanvas && !waiting && (
          <button
            type="button"
            className="rounded-md bg-amber-600 px-2 py-1 font-medium text-white hover:bg-amber-500"
            onClick={onAdoptProposal}
          >
            Volver a adoptarla
          </button>
        )}
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
