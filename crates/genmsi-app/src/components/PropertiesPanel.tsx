import { useEffect, useState } from "react";
import { editorFor, validateProperty } from "../edit/property";
import type { CatalogueDocument, ModelBlock } from "../types";

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

/** The options a declared property offers, taken from the catalogue. */
function optionsOf(
  catalogue: CatalogueDocument | null,
  className: string,
  property: string,
): string[] {
  const spec = catalogue?.classes.find((candidate) => candidate.class === className);
  return spec?.properties.find((candidate) => candidate.name === property)?.options ?? [];
}

/**
 * The properties of the selected block, as editors.
 *
 * The editor of a property comes from the type the model declares, not from
 * the name: `PosInteger` gets a number that only takes whole positive
 * values, `Listed` gets the options of its list, and a name gets a field
 * that refuses the characters the engine refuses.
 */
export default function PropertiesPanel({
  block,
  catalogue,
  onSetProperty,
  onSetCaption,
}: {
  block: ModelBlock | null;
  catalogue: CatalogueDocument | null;
  onSetProperty: (id: number, property: string, value: string) => void;
  onSetCaption: (id: number, caption: string) => void;
}) {
  if (block === null) {
    return (
      <p className="px-1 py-6 text-sm text-slate-500">
        Selecciona un bloque del grafo para ver sus propiedades.
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
        <label className="mt-3 block text-xs text-slate-500" htmlFor="caption">
          Título
        </label>
        <input
          id="caption"
          aria-label="Título del bloque"
          className="mt-1 w-full rounded-md border border-slate-300 bg-white px-2 py-1 text-xs text-slate-800 outline-none focus:border-slate-500 focus:ring-2 focus:ring-slate-200"
          defaultValue={block.caption}
          onBlur={(event) => {
            if (event.target.value !== block.caption) {
              onSetCaption(block.id, event.target.value);
            }
          }}
        />
      </div>

      <div>
        <h3 className="text-xs font-semibold uppercase tracking-wide text-slate-500">
          Propiedades
        </h3>
        {block.properties.length === 0 ? (
          <p className="mt-2 text-xs text-slate-500">Este bloque no declara propiedades.</p>
        ) : (
          <div className="mt-2 flex flex-col gap-2">
            {block.properties.map((property) => (
              <PropertyEditor
                key={property.name}
                blockId={block.id}
                name={property.name}
                value={property.value}
                ty={property.ty}
                subtype={property.subtype}
                options={optionsOf(catalogue, block.class, property.name)}
                onCommit={(value) => onSetProperty(block.id, property.name, value)}
              />
            ))}
          </div>
        )}
      </div>

      <div>
        <h3 className="text-xs font-semibold uppercase tracking-wide text-slate-500">
          Puertos
        </h3>
        {block.inputs.length === 0 && block.outputs.length === 0 ? (
          <p className="mt-2 text-xs text-slate-500">Este bloque no declara puertos.</p>
        ) : (
          <table className="mt-2 w-full table-fixed border-collapse text-xs">
            <thead>
              <tr className="border-b border-slate-200 text-left text-slate-500">
                <th className="w-1/3 py-1 font-medium">Puerto</th>
                <th className="py-1 font-medium">Etiqueta</th>
              </tr>
            </thead>
            <tbody>
              {block.inputs.map((port) => (
                <tr key={`in-${port.ioid}`} className="border-b border-slate-100">
                  <td className="py-1 pr-2 font-mono text-slate-600">entrada {port.name}</td>
                  <td className="py-1 font-mono text-slate-900">{port.label ?? "—"}</td>
                </tr>
              ))}
              {block.outputs.map((port) => (
                <tr key={`out-${port.ioid}`} className="border-b border-slate-100">
                  <td className="py-1 pr-2 font-mono text-slate-600">salida {port.name}</td>
                  <td className="py-1 font-mono text-slate-900">{port.label ?? "—"}</td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>
    </div>
  );
}

/**
 * One property, with the editor its declared type asks for.
 *
 * The value is committed when the field loses the focus or the user presses
 * Enter: one edit is one change, and one change is one undo entry.
 */
function PropertyEditor({
  blockId,
  name,
  value,
  ty,
  subtype,
  options,
  onCommit,
}: {
  blockId: number;
  name: string;
  value: string;
  ty: string;
  subtype: string;
  options: string[];
  onCommit: (value: string) => void;
}) {
  const spec = { name, ty, subtype, options };
  const [draft, setDraft] = useState(value);
  const [problem, setProblem] = useState<string | null>(null);

  useEffect(() => {
    setDraft(value);
  }, [value]);

  const commit = () => {
    const failure = validateProperty(spec, draft);
    setProblem(failure);
    if (failure === null && draft !== value) {
      onCommit(draft);
    }
  };

  const editor = editorFor(spec);
  const label = (
    <label className="block text-xs text-slate-600" htmlFor={`property-${blockId}-${name}`}>
      {name}
      {subtype !== "" && <span className="text-slate-400"> · {ty}/{subtype}</span>}
    </label>
  );

  return (
    <div>
      {label}
      {editor === "select" ? (
        <select
          id={`property-${blockId}-${name}`}
          aria-label={name}
          className="mt-1 w-full rounded-md border border-slate-300 bg-white px-2 py-1 text-xs font-mono text-slate-800 outline-none focus:border-slate-500 focus:ring-2 focus:ring-slate-200"
          value={value}
          onChange={(event) => {
            const next = event.target.value;
            setDraft(next);
            const failure = validateProperty(spec, next);
            setProblem(failure);
            if (failure === null) {
              onCommit(next);
            }
          }}
        >
          <option value="">(sin valor)</option>
          {options.map((option) => (
            <option key={option} value={option}>
              {option}
            </option>
          ))}
          {!options.includes(value) && value !== "" && (
            <option value={value}>{value}</option>
          )}
        </select>
      ) : (
        <input
          id={`property-${blockId}-${name}`}
          aria-label={name}
          type={editor === "integer" || editor === "number" || editor === "percent" ? "number" : "text"}
          min={editor === "integer" ? 1 : undefined}
          value={draft}
          onChange={(event) => setDraft(event.target.value)}
          onBlur={commit}
          onKeyDown={(event) => {
            if (event.key === "Enter") {
              commit();
            }
          }}
          className="mt-1 w-full rounded-md border border-slate-300 bg-white px-2 py-1 text-xs font-mono text-slate-800 outline-none focus:border-slate-500 focus:ring-2 focus:ring-slate-200"
        />
      )}
      {problem !== null && <p className="mt-1 text-[11px] text-rose-600">{problem}</p>}
    </div>
  );
}
