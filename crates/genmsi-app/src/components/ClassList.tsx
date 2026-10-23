import type { CatalogueClass, CatalogueDocument, CataloguePort } from "../types";

/**
 * The classes of block the reader can create.
 *
 * The list lives in the lateral panel, in the tab **Bloques**: it is the
 * catalogue the model is built from, read at a glance. Every class says what
 * it is (its name), what it does (the title of the catalogue) and what it
 * carries (its ports and its properties), so the reader knows what he is
 * about to create before he creates it.
 *
 * A class is created with a click — the block lands in the first free spot
 * of the model — or with a drag: the drag carries the name of the class in
 * the `application/genmsi-block` payload, which is what the picture reads
 * when the block is dropped on it.
 *
 * The list can be long, so it scrolls with the panel that holds it.
 */
export default function ClassList({
  catalogue,
  onAdd,
}: {
  /** The block classes the catalogue offers. */
  catalogue: CatalogueDocument | null;
  /** Add a block of a class. */
  onAdd: (className: string) => void;
}) {
  const classes = catalogue?.classes ?? [];

  if (classes.length === 0) {
    return (
      <p className="px-1 py-6 text-sm text-slate-500">El catálogo todavía no cargó.</p>
    );
  }

  return (
    <ul className="flex flex-col gap-2">
      {classes.map((classSpec) => (
        <li key={classSpec.class}>
          <ClassCard classSpec={classSpec} onAdd={onAdd} />
        </li>
      ))}
    </ul>
  );
}

/**
 * One class of block, with everything the catalogue says about it.
 *
 * The card is a button: clicking it creates the block. It is also
 * draggable, and the drag carries the name of the class, so the block lands
 * where the reader drops it.
 */
function ClassCard({
  classSpec,
  onAdd,
}: {
  /** The class the catalogue declares. */
  classSpec: CatalogueClass;
  /** Add a block of the class. */
  onAdd: (className: string) => void;
}) {
  return (
    <button
      type="button"
      data-block-class={classSpec.class}
      draggable
      onDragStart={(event) => {
        event.dataTransfer.setData("application/genmsi-block", classSpec.class);
        event.dataTransfer.effectAllowed = "copy";
      }}
      onClick={() => onAdd(classSpec.class)}
      className="w-full cursor-grab rounded-md border border-slate-200 bg-white px-3 py-2 text-left transition hover:border-indigo-300 hover:bg-indigo-50/40 active:cursor-grabbing"
    >
      <span className="block font-mono text-xs font-semibold text-slate-800">
        {classSpec.class}
      </span>
      <span className="mt-0.5 block text-[11px] leading-snug text-slate-500">
        {classSpec.title}
      </span>
      <Ports inputs={classSpec.inputs} outputs={classSpec.outputs} />
      {classSpec.properties.length > 0 && (
        <span className="mt-1.5 block">
          <span className="text-[10px] font-semibold uppercase tracking-wide text-slate-400">
            Propiedades
          </span>
          <span className="mt-0.5 flex flex-wrap gap-1">
            {classSpec.properties.map((property) => (
              <span
                key={property.name}
                className="rounded bg-slate-100 px-1.5 py-0.5 font-mono text-[10px] text-slate-600"
              >
                {property.name}
                <span className="text-slate-400">
                  {property.subtype === ""
                    ? ` · ${property.ty}`
                    : ` · ${property.ty}/${property.subtype}`}
                </span>
              </span>
            ))}
          </span>
        </span>
      )}
    </button>
  );
}

/**
 * The ports of a class: the way in and the way out.
 *
 * The names are the ones of the catalogue (`Entrada1`, `Salida1`, ...): they
 * are the names the model uses to connect the block.
 */
function Ports({
  inputs,
  outputs,
}: {
  /** The input ports, in file order. */
  inputs: CataloguePort[];
  /** The output ports, in file order. */
  outputs: CataloguePort[];
}) {
  if (inputs.length === 0 && outputs.length === 0) {
    return (
      <span className="mt-1 block text-[10px] text-slate-400">Sin puertos.</span>
    );
  }
  return (
    <span className="mt-1.5 block">
      <span className="text-[10px] font-semibold uppercase tracking-wide text-slate-400">
        Puertos
      </span>
      <span className="mt-0.5 block space-y-0.5 font-mono text-[10px] text-slate-600">
        <span className="block">
          <span className="text-slate-400">entrada</span>{" "}
          {inputs.length === 0 ? "—" : inputs.map((port) => port.name).join(", ")}
        </span>
        <span className="block">
          <span className="text-slate-400">salida</span>{" "}
          {outputs.length === 0 ? "—" : outputs.map((port) => port.name).join(", ")}
        </span>
      </span>
    </span>
  );
}
