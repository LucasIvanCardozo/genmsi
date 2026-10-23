import type { CatalogueClass, CatalogueDocument } from "../types";

/**
 * The classes of block the reader can create.
 *
 * The list lives in the lateral panel, in the tab **Bloques**: it is the
 * catalogue the model is built from, read at a glance. Every class says what
 * it is (its name) and what it does (the title of the catalogue).
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
    <ul className="flex flex-col gap-1">
      {classes.map((classSpec) => (
        <li key={classSpec.class}>
          <ClassCard classSpec={classSpec} onAdd={onAdd} />
        </li>
      ))}
    </ul>
  );
}

/**
 * One class of block: its name and what it does.
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
      className="w-full cursor-grab rounded-md border border-transparent px-2 py-1.5 text-left transition hover:border-indigo-200 hover:bg-indigo-50/50 active:cursor-grabbing"
    >
      <span className="block font-mono text-xs font-semibold text-slate-800">
        {classSpec.class}
      </span>
      {classSpec.title !== "" && (
        <span className="mt-0.5 block text-[11px] leading-snug text-slate-500">
          {classSpec.title}
        </span>
      )}
    </button>
  );
}
