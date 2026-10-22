import { useState } from "react";
import { ChevronDown, Plus } from "lucide-react";
import type { CatalogueDocument } from "../types";

/**
 * The palette of the block classes the editor offers.
 *
 * A class can be dragged onto the picture (the block lands where it is
 * dropped) or clicked (the block lands in the first free spot). The palette
 * shows the class and what it does, in the words of the interface.
 */
export default function BlockPalette({
  catalogue,
  onAdd,
}: {
  /** The block classes the editor offers. */
  catalogue: CatalogueDocument | null;
  /** Add a block of a class. */
  onAdd: (className: string) => void;
}) {
  const [open, setOpen] = useState(false);
  const classes = catalogue?.classes ?? [];

  return (
    <div className="relative">
      <button
        type="button"
        aria-label="Añadir bloque"
        aria-expanded={open}
        onClick={() => setOpen((current) => !current)}
        className="flex items-center gap-1.5 rounded-md border border-slate-300 bg-white px-2.5 py-1.5 text-xs font-medium text-slate-700 shadow-sm transition hover:bg-slate-50"
      >
        <Plus className="size-3.5" aria-hidden />
        Añadir bloque
        <ChevronDown className="size-3" aria-hidden />
      </button>

      {open && (
        <div className="absolute left-0 z-20 mt-1 max-h-72 w-72 overflow-auto rounded-md border border-slate-200 bg-white py-1 shadow-lg">
          {classes.length === 0 && (
            <p className="px-3 py-2 text-xs text-slate-500">El catálogo todavía no cargó.</p>
          )}
          {classes.map((classSpec) => (
            <button
              key={classSpec.class}
              type="button"
              draggable
              onDragStart={(event) => {
                event.dataTransfer.setData("application/genmsi-block", classSpec.class);
                event.dataTransfer.effectAllowed = "copy";
              }}
              onClick={() => {
                onAdd(classSpec.class);
                setOpen(false);
              }}
              className="flex w-full flex-col items-start px-3 py-1.5 text-left hover:bg-slate-50"
            >
              <span className="font-mono text-xs font-semibold text-slate-800">
                {classSpec.class}
              </span>
              <span className="text-[11px] text-slate-500">{classSpec.title}</span>
            </button>
          ))}
        </div>
      )}
    </div>
  );
}
