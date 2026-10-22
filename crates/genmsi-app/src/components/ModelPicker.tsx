import { useApp } from "../state/store";

/**
 * The model picker: the models the engine offers and the button that opens
 * one of them.
 */
export default function ModelPicker() {
  const { models, currentPath, status, openModel } = useApp();

  return (
    <div className="flex items-center gap-2">
      <label className="text-xs font-medium uppercase tracking-wide text-slate-500">
        Modelo
      </label>
      <select
        aria-label="Modelo"
        className="min-w-64 rounded-md border border-slate-300 bg-white px-3 py-1.5 text-sm text-slate-800 shadow-sm outline-none focus:border-slate-500 focus:ring-2 focus:ring-slate-200"
        value={currentPath ?? ""}
        onChange={(event) => void openModel(event.target.value)}
        disabled={status === "loading"}
      >
        <option value="">Selecciona un modelo…</option>
        {models.map((model) => (
          <option key={model.path} value={model.path}>
            {model.name}
          </option>
        ))}
      </select>
    </div>
  );
}
