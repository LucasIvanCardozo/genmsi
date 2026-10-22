import type { Diagnostics } from "../types";

/**
 * What the model compiles to, right now.
 *
 * The diagnostics follow every change: a model that is being edited can be
 * momentarily invalid, and the app says so without stopping the editing. The
 * saving is never blocked either.
 */
export default function DiagnosticsPanel({
  diagnostics,
}: {
  /** What the model in memory compiles to. */
  diagnostics: Diagnostics | null;
}) {
  if (diagnostics === null) {
    return null;
  }

  if (diagnostics.status === "error") {
    return (
      <div className="rounded-lg border border-rose-200 bg-rose-50 px-3 py-2 text-xs text-rose-900">
        <p className="font-semibold">El modelo no compila</p>
        <p className="mt-0.5 font-mono">{diagnostics.message}</p>
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
