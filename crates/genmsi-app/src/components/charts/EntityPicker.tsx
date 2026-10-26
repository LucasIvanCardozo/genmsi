/**
 * The entities the reader chooses to see in a time series.
 *
 * The picker is the same everywhere: the entities of the run, the ones that
 * are on are marked, and clicking one adds it or takes it away.
 */
export default function EntityPicker({
  entities,
  selected,
  onToggle,
}: {
  /** The entities of the run. */
  entities: string[];
  /** The entities that are on. */
  selected: string[];
  /** The click on an entity. */
  onToggle: (name: string) => void;
}) {
  if (entities.length === 0) {
    return null;
  }
  return (
    <div className="mb-2 flex flex-wrap gap-1">
      {entities.map((name) => {
        const active = selected.includes(name);
        return (
          <button
            key={name}
            type="button"
            onClick={() => onToggle(name)}
            className={[
              "rounded-md border px-2 py-1 text-[11px] font-medium transition",
              active
                ? "border-indigo-500 bg-indigo-50 text-indigo-700"
                : "border-slate-300 bg-white text-slate-500 hover:bg-slate-50",
            ].join(" ")}
          >
            {name}
          </button>
        );
      })}
    </div>
  );
}
