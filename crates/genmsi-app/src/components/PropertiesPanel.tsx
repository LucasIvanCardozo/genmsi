import type { ModelBlock } from "../types";

/** The kind of a block, in the app's words. */
function kindOf(block: ModelBlock): string {
  return block.kind === "C" ? "Contenedor" : "Bloque";
}

/** The position of a block on the canvas. */
function positionOf(block: ModelBlock): string {
  if (block.layout === null) {
    return "sin posición";
  }
  const { x, y, width, height } = block.layout;
  return `x ${x} · y ${y} · ${width}×${height}`;
}

/**
 * The properties of the selected block.
 *
 * The panel shows what the model declares: the values come from the model's
 * own document, decoded, and the app computes nothing.
 */
export default function PropertiesPanel({ block }: { block: ModelBlock | null }) {
  if (block === null) {
    return (
      <p className="px-1 py-6 text-sm text-slate-500">
        Elegí un bloque del grafo para ver sus propiedades.
      </p>
    );
  }

  return (
    <div className="flex flex-col gap-4">
      <div>
        <h2 className="text-sm font-semibold text-slate-800">{block.caption || block.name}</h2>
        <dl className="mt-2 grid grid-cols-2 gap-x-3 gap-y-1 text-xs">
          <dt className="text-slate-500">Clase</dt>
          <dd className="font-mono text-slate-800">{block.class || "—"}</dd>
          <dt className="text-slate-500">Tipo</dt>
          <dd className="text-slate-800">{kindOf(block)}</dd>
          <dt className="text-slate-500">Identificador</dt>
          <dd className="font-mono text-slate-800">{block.id}</dd>
          <dt className="text-slate-500">Etiqueta</dt>
          <dd className="font-mono text-slate-800">{block.label ?? "—"}</dd>
          <dt className="text-slate-500">Posición</dt>
          <dd className="font-mono text-slate-800">{positionOf(block)}</dd>
        </dl>
      </div>

      <div>
        <h3 className="text-xs font-semibold uppercase tracking-wide text-slate-500">
          Propiedades
        </h3>
        {block.properties.length === 0 ? (
          <p className="mt-2 text-xs text-slate-500">Este bloque no declara propiedades.</p>
        ) : (
          <table className="mt-2 w-full table-fixed border-collapse text-xs">
            <thead>
              <tr className="border-b border-slate-200 text-left text-slate-500">
                <th className="w-1/2 py-1 font-medium">Propiedad</th>
                <th className="py-1 font-medium">Valor</th>
              </tr>
            </thead>
            <tbody>
              {block.properties.map((property) => (
                <tr key={property.name} className="border-b border-slate-100 align-top">
                  <td className="py-1 pr-2 text-slate-600">{property.name}</td>
                  <td className="py-1 font-mono break-words text-slate-900">
                    {property.value === "" ? (
                      <span className="text-slate-400">—</span>
                    ) : (
                      property.value
                    )}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>
    </div>
  );
}
