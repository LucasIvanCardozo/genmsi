import type { LintDocument } from "../types";

/**
 * What the model compiles to.
 *
 * The numbers come from the engine's `genmsi.lint/1` document; the app shows
 * them, it does not count anything itself.
 */
export default function ModelSummary({ lint }: { lint: LintDocument | null }) {
  if (lint === null) {
    return null;
  }
  const definitions = lint.definitions;
  return (
    <div className="flex flex-wrap items-center gap-x-5 gap-y-1 rounded-lg border border-slate-200 bg-white px-4 py-2 text-xs text-slate-600">
      <span>
        <span className="font-mono text-slate-900">{lint.blocks}</span> bloques ejecutables
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
      {lint.status === "diagnostics" && (
        <span className="text-amber-700">
          {lint.diagnostics.length} {lint.diagnostics.length === 1 ? "aviso" : "avisos"}
        </span>
      )}
    </div>
  );
}
