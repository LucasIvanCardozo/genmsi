import type { Diagnostics } from "../types";

/**
 * What the model compiles to.
 *
 * The numbers come from the engine: the app shows what the engine says, it
 * does not count anything itself.
 */
export default function ModelSummary({ diagnostics }: { diagnostics: Diagnostics | null }) {
  if (diagnostics === null || diagnostics.blocks === null || diagnostics.definitions === null) {
    return null;
  }
  const definitions = diagnostics.definitions;
  return (
    <div className="flex flex-wrap items-center gap-x-5 gap-y-1 rounded-lg border border-slate-200 bg-white px-4 py-2 text-xs text-slate-600">
      <span>
        <span className="font-mono text-slate-900">{diagnostics.blocks}</span> bloques ejecutables
      </span>
      <span>
        <span className="font-mono text-slate-900">{definitions.functions}</span> funciones
      </span>
      <span>
        <span className="font-mono text-slate-900">
          {definitions.variables + definitions.random_variables}
        </span>{" "}
        variables
      </span>
      <span>
        <span className="font-mono text-slate-900">{definitions.storages}</span> almacenes
      </span>
      <span>
        <span className="font-mono text-slate-900">{definitions.tables}</span> tablas
      </span>
      <span>
        <span className="font-mono text-slate-900">{definitions.qtables}</span> qtablas
      </span>
      {diagnostics.status === "diagnostics" && (
        <span className="text-amber-700">
          {diagnostics.diagnostics.length}{" "}
          {diagnostics.diagnostics.length === 1 ? "aviso" : "avisos"}
        </span>
      )}
    </div>
  );
}
