import { useApp } from "../state/store";

/**
 * The identity of the work: which model is open, and whether it is saved.
 *
 * The name of the model is the title of the app's window: it says what the
 * reader is working on, and it says it before anything else. The title the
 * model carries travels with it, muted, and the dot of amber says that the
 * work on the disk is not the work on the screen.
 */
export default function ModelIdentity() {
  const currentPath = useApp((state) => state.currentPath);
  const title = useApp((state) => state.model?.title ?? null);
  const dirty = useApp((state) => state.dirty);
  const status = useApp((state) => state.status);

  return (
    <div className="flex min-w-0 items-center gap-2">
      <span
        className="max-w-72 truncate text-[12.5px] font-medium text-slate-800"
        title={currentPath ?? undefined}
      >
        {currentPath === null ? "Sin modelo" : nameOf(currentPath)}
      </span>
      {dirty && (
        <span
          className="size-1.5 shrink-0 rounded-full bg-amber-500"
          title="El modelo tiene cambios sin guardar"
          aria-label="El modelo tiene cambios sin guardar"
        />
      )}
      {title !== null && (
        <span className="truncate text-[11.5px] text-slate-500">{title}</span>
      )}
      {status === "loading" && (
        <span className="text-[11px] text-slate-500">Cargando…</span>
      )}
    </div>
  );
}

/** The name of a path, without its directory. */
function nameOf(path: string): string {
  const parts = path.split(/[\\/]/);
  return parts[parts.length - 1] || path;
}
