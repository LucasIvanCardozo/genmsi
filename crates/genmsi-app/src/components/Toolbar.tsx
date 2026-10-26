import {
  Copy,
  Eraser,
  FilePlus2,
  LayoutGrid,
  Redo2,
  Save,
  Scan,
  Undo2,
} from "lucide-react";
import ModelBar from "./ModelBar";
import { useApp } from "../state/store";
import {
  groupLabel,
  groupRule,
  iconButton,
  numberField,
  textButton,
} from "./chrome";

/**
 * The commands of the app, in one dense row.
 *
 * The row is split by what the commands **do**, and every group says what
 * it is: *archivo* opens and saves the model, *edición* changes it, *vista*
 * looks at it, and *corrida* holds the three numbers the next run takes.
 * A flat strip of fifteen buttons explained nothing; these four names say
 * it in four words.
 *
 * The commands that have an icon and no doubt about it keep only the icon:
 * the toolbar is a keyboard's echo, and the space it takes is the space the
 * graph does not get. The seed, the horizon and the `START` of the run live
 * here too: they are the three numbers the next run takes, and the seed is
 * a 64-bit one, so its field is wide enough to read it whole. The horizon
 * comes with the 3600 of the original: a run without a horizon lasts until
 * the engine runs out of patience. The `START` comes with the one of the
 * original: it is the counter of terminations of the run, and the original
 * has it in the panel of its simulator, next to the clock of the run.
 */
export default function Toolbar({
  dirty,
  canUndo,
  canRedo,
  hasModel,
  onNew,
  onSave,
  onSaveAs,
  onUndo,
  onRedo,
  onRelayout,
  onFit,
  onDelete,
}: {
  /** Whether the model has changes that were not saved. */
  dirty: boolean;
  /** Whether an operation can be undone. */
  canUndo: boolean;
  /** Whether an operation can be redone. */
  canRedo: boolean;
  /** Whether a model is open. */
  hasModel: boolean;
  /** Start a new, empty model. */
  onNew: () => void;
  /** Save the model. */
  onSave: () => void;
  /** Save the model under another name. */
  onSaveAs: () => void;
  /** Undo the last operation. */
  onUndo: () => void;
  /** Redo the last undone operation. */
  onRedo: () => void;
  /** Reorder the graph in layers. */
  onRelayout: () => void;
  /** Frame the graph again. */
  onFit: () => void;
  /** Delete the selected blocks. */
  onDelete: () => void;
}) {
  // The two numbers of the next run: the seed and the horizon. They are read
  // from the store with selectors, so a change of the run does not redraw
  // the toolbar.
  const seed = useApp((state) => state.seed);
  const seedIsAuto = useApp((state) => state.seedIsAuto);
  const until = useApp((state) => state.until);
  const start = useApp((state) => state.start);
  const setSeed = useApp((state) => state.setSeed);
  const setSeedAuto = useApp((state) => state.setSeedAuto);
  const setUntil = useApp((state) => state.setUntil);
  const setStart = useApp((state) => state.setStart);

  return (
    <div className="flex h-[38px] shrink-0 items-center gap-3 border-b border-slate-200 bg-white px-4">
      {/* The file commands. */}
      <div className="flex items-center gap-1.5">
        <span className={groupLabel}>Archivo</span>
        <button
          type="button"
          className={iconButton}
          onClick={onNew}
          title="Un modelo nuevo"
          aria-label="Un modelo nuevo"
        >
          <FilePlus2 className="size-[14px]" aria-hidden />
        </button>
        <ModelBar />
        <button
          type="button"
          className={iconButton}
          onClick={onSave}
          disabled={!hasModel}
          title={dirty ? "El modelo tiene cambios sin guardar" : "El modelo está guardado"}
          aria-label="Guardar el modelo"
        >
          <Save className="size-[14px]" aria-hidden />
        </button>
        <button
          type="button"
          className={textButton}
          onClick={onSaveAs}
          disabled={!hasModel}
          title="Guardar el modelo con el diálogo del sistema"
        >
          <Copy className="size-[13px]" aria-hidden />
          Guardar como…
        </button>
      </div>

      <span className={groupRule} aria-hidden />

      {/* The commands that change the model. */}
      <div className="flex items-center gap-1.5">
        <span className={groupLabel}>Edición</span>
        <button
          type="button"
          className={iconButton}
          onClick={onUndo}
          disabled={!canUndo}
          title="Deshacer (Ctrl+Z)"
          aria-label="Deshacer"
        >
          <Undo2 className="size-[14px]" aria-hidden />
        </button>
        <button
          type="button"
          className={iconButton}
          onClick={onRedo}
          disabled={!canRedo}
          title="Rehacer (Ctrl+Shift+Z)"
          aria-label="Rehacer"
        >
          <Redo2 className="size-[14px]" aria-hidden />
        </button>
        <button
          type="button"
          className={iconButton}
          onClick={onDelete}
          disabled={!hasModel}
          title="Borrar los bloques seleccionados (Supr)"
          aria-label="Borrar los bloques seleccionados"
        >
          <Eraser className="size-[14px]" aria-hidden />
        </button>
      </div>

      <span className={groupRule} aria-hidden />

      {/* The commands that look at the model. */}
      <div className="flex items-center gap-1.5">
        <span className={groupLabel}>Vista</span>
        <button
          type="button"
          className={iconButton}
          onClick={onFit}
          disabled={!hasModel}
          title="Encuadrar el grafo"
          aria-label="Encuadrar el grafo"
        >
          <Scan className="size-[14px]" aria-hidden />
        </button>
        <button
          type="button"
          className={iconButton}
          onClick={onRelayout}
          disabled={!hasModel}
          title="Reordenar el grafo en capas"
          aria-label="Reordenar el grafo en capas"
        >
          <LayoutGrid className="size-[14px]" aria-hidden />
        </button>
      </div>

      {/* The three numbers of the next run, at the other end of the row. */}
      <div className="ml-auto flex items-center gap-2.5">
        <label className="flex items-center gap-1.5">
          <span className={groupLabel}>Semilla</span>
          <input
            aria-label="Semilla"
            type="number"
            min={1}
            value={seed}
            onChange={(event) => setSeed(Number(event.target.value) || 1)}
            className={`${numberField} w-24`}
          />
          {/* The lock of the seed: checked, the run repeats itself; unchecked,
              every run draws its own seed, and the field shows which one it
              was. Typing a seed checks it, because a seed the reader chose
              is a seed the reader fixed. */}
          <span
            className="flex items-center gap-1"
            title="Fijar la semilla: la corrida se repite en vez de sortear una nueva"
          >
            <input
              type="checkbox"
              aria-label="Fijar la semilla"
              checked={!seedIsAuto}
              onChange={(event) => setSeedAuto(!event.target.checked)}
              className="size-3.5 accent-indigo-600"
            />
            <span className={groupLabel}>Fijar</span>
          </span>
        </label>
        <label className="flex items-center gap-1.5">
          <span className={groupLabel}>Hasta</span>
          <input
            aria-label="Hasta"
            type="number"
            min={0}
            placeholder="—"
            title="El horizonte de la corrida, en tiempo simulado. Por defecto 3600, como el reloj del original; vacío es una corrida sin tope."
            value={until ?? ""}
            onChange={(event) =>
              setUntil(event.target.value === "" ? null : Number(event.target.value))
            }
            className={`${numberField} w-[52px]`}
          />
        </label>
        <label className="flex items-center gap-1.5">
          <span className={groupLabel}>START</span>
          <input
            aria-label="START"
            type="number"
            min={1}
            step={1}
            value={start}
            title="El START de la corrida: cuántas unidades de terminación la terminan. Por defecto 1, como el panel del simulador del original. El original no admite un Start de cero («El Start debe ser mayor a cero») y el campo tampoco."
            onChange={(event) => setStart(Number(event.target.value) || 1)}
            className={`${numberField} w-11`}
          />
        </label>
      </div>
    </div>
  );
}
