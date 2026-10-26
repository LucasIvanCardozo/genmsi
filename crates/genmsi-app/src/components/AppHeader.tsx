import mark from "../../src-tauri/icons/128x128.png";
import ModelIdentity from "./ModelIdentity";
import RunConsole from "./RunConsole";

/**
 * The header of the app: who it is, what it has open, and the console of the
 * run.
 *
 * The order of the header is the order of the questions the reader asks:
 * **what am I editing** comes first —the mark, the name, the model— and
 * **what is the run doing** comes last, at the right, where the hand
 * already is. The model is the subject of the work, so its name leads; the
 * console is the instrument, so it ends the row.
 */
export default function AppHeader() {
  return (
    <header className="flex h-11 shrink-0 items-center gap-2.5 border-b border-slate-200 bg-white px-4">
      <img src={mark} alt="" aria-hidden className="size-[22px] rounded-[5px]" />
      <span className="text-[13px] font-semibold tracking-tight text-slate-900">
        GenMSI
      </span>
      <span className="h-4 w-px bg-slate-200" aria-hidden />
      <ModelIdentity />
      <div className="ml-auto">
        <RunConsole />
      </div>
    </header>
  );
}
