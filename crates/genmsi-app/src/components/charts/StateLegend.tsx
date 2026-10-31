/**
 * How to read the figures of the section.
 *
 * The legend is the one the panel generates: the four states of a figure,
 * each with the color it is painted in and one sentence of what it means.
 * No card writes a legend of its own — a legend that is repeated is a
 * legend that ends up saying two different things.
 *
 * The words are the ones of the chosen artboard
 * (`design/graficos-verdad/03-panel-lateral-estado.html`), textual.
 */
import { FIGURE_STATES, FIGURE_STATE_ORDER } from "./states.ts";

export default function StateLegend() {
  return (
    <div>
      <h3 className="text-[10px] font-bold uppercase tracking-wider text-slate-500">
        Cómo leer estas cifras
      </h3>
      <ul className="mt-2 flex flex-col gap-2">
        {FIGURE_STATE_ORDER.map((id) => {
          const state = FIGURE_STATES[id];
          return (
            <li key={id} className="flex items-start gap-2">
              <span
                className={`mt-0.5 h-2.5 w-2.5 shrink-0 rounded-sm ${state.markClass}`}
                aria-hidden="true"
              />
              <p className="text-[11px] leading-snug text-slate-500">
                <span className="font-semibold text-slate-800">{state.title}</span>
                {" — "}
                {state.explanation}
              </p>
            </li>
          );
        })}
      </ul>
      <p className="mt-3 text-[11px] leading-snug text-slate-500">
        El eje del tiempo de todos los gráficos va de 0 al final de la corrida
        y la línea vertical marca el reloj. Ninguna cifra derivada se muestra
        si el reloj y las cifras no son del mismo instante.
      </p>
    </div>
  );
}
