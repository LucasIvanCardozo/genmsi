import { useState } from "react";
import { Eraser, LayoutGrid, Redo2, Save, Undo2 } from "lucide-react";

/**
 * The editing actions of the app.
 *
 * Every button is one operation: the undo stack takes one entry per press,
 * and the button is disabled when the operation is not possible.
 */
export default function Toolbar({
  dirty,
  canUndo,
  canRedo,
  hasModel,
  pendingSave,
  onNew,
  onSave,
  onSaveAs,
  onUndo,
  onRedo,
  onRelayout,
  onDelete,
  onConfirmSave,
  onDismissSave,
}: {
  /** Whether the model has changes that were not saved. */
  dirty: boolean;
  /** Whether an operation can be undone. */
  canUndo: boolean;
  /** Whether an operation can be redone. */
  canRedo: boolean;
  /** Whether a model is open. */
  hasModel: boolean;
  /** The path of a save that waits for the user's confirmation. */
  pendingSave: string | null;
  /** Start a new, empty model. */
  onNew: () => void;
  /** Save the model. */
  onSave: () => void;
  /** Save the model under another name. */
  onSaveAs: (path: string) => void;
  /** Undo the last operation. */
  onUndo: () => void;
  /** Redo the last undone operation. */
  onRedo: () => void;
  /** Reorder the graph in layers. */
  onRelayout: () => void;
  /** Delete the selected blocks. */
  onDelete: () => void;
  /** Answer the confirmation of a save with yes. */
  onConfirmSave: () => void;
  /** Answer the confirmation of a save with no. */
  onDismissSave: () => void;
}) {
  const [naming, setNaming] = useState(false);
  const [name, setName] = useState("");

  const button =
    "flex items-center gap-1.5 rounded-md border border-slate-300 bg-white px-2.5 py-1.5 text-xs font-medium text-slate-700 shadow-sm transition hover:bg-slate-50 disabled:cursor-not-allowed disabled:text-slate-300";

  return (
    <div className="flex flex-wrap items-center gap-2 rounded-lg border border-slate-200 bg-white px-3 py-2">
      <button type="button" className={button} onClick={onNew}>
        Nuevo
      </button>
      <button
        type="button"
        className={button}
        onClick={onSave}
        disabled={!hasModel || (dirty && pendingSave !== null)}
        title={dirty ? "El modelo tiene cambios sin guardar" : "El modelo está guardado"}
      >
        <Save className="size-3.5" aria-hidden />
        Guardar{dirty ? "•" : ""}
      </button>
      <button
        type="button"
        className={button}
        onClick={() => {
          setNaming(true);
          setName("");
        }}
        disabled={!hasModel}
      >
        Guardar como…
      </button>

      <span className="mx-1 h-5 w-px bg-slate-200" aria-hidden />

      <button
        type="button"
        className={button}
        onClick={onUndo}
        disabled={!canUndo}
        title="Deshacer (Ctrl+Z)"
      >
        <Undo2 className="size-3.5" aria-hidden />
        Deshacer
      </button>
      <button
        type="button"
        className={button}
        onClick={onRedo}
        disabled={!canRedo}
        title="Rehacer (Ctrl+Shift+Z)"
      >
        <Redo2 className="size-3.5" aria-hidden />
        Rehacer
      </button>
      <button
        type="button"
        className={button}
        onClick={onDelete}
        disabled={!hasModel}
        title="Borrar los bloques seleccionados (Supr)"
      >
        <Eraser className="size-3.5" aria-hidden />
        Borrar
      </button>
      <button
        type="button"
        className={button}
        onClick={onRelayout}
        disabled={!hasModel}
        title="Reordenar el grafo en capas"
      >
        <LayoutGrid className="size-3.5" aria-hidden />
        Reacomodar
      </button>

      {naming && (
        <span className="flex items-center gap-1.5">
          <input
            aria-label="Ruta del modelo"
            className="w-72 rounded-md border border-slate-300 bg-white px-2 py-1 text-xs font-mono text-slate-800 outline-none focus:border-slate-500 focus:ring-2 focus:ring-slate-200"
            value={name}
            onChange={(event) => setName(event.target.value)}
            placeholder="fixtures/models/mi-modelo.gmdl"
          />
          <button
            type="button"
            className={button}
            onClick={() => {
              setNaming(false);
              if (name.trim() !== "") {
                onSaveAs(name.trim());
              }
            }}
          >
            Guardar
          </button>
        </span>
      )}

      {pendingSave !== null && (
        <span className="flex items-center gap-2 rounded-md border border-amber-300 bg-amber-50 px-2.5 py-1.5 text-xs text-amber-900">
          El motor no compila este modelo. ¿Guardarlo de todos modos?
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
        </span>
      )}
    </div>
  );
}
