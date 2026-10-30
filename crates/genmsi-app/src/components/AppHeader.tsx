import { useRef, useState } from "react";
import mark from "../../src-tauri/icons/128x128.png";
import AboutDialog from "./AboutDialog";
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
 *
 * The name of the app is a button, not a word: it opens the box that says
 * which version is installed, which is the first thing a reader asks when
 * something does not work.
 */
export default function AppHeader() {
  const [about, setAbout] = useState(false);
  const brand = useRef<HTMLButtonElement>(null);

  const close = () => {
    setAbout(false);
    // The focus goes back to the button that opened the box: the reader
    // stays where he was.
    brand.current?.focus();
  };

  return (
    <header className="flex h-11 shrink-0 items-center gap-2.5 border-b border-slate-200 bg-white px-4">
      <img src={mark} alt="" aria-hidden className="size-[22px] rounded-[5px]" />
      <button
        ref={brand}
        type="button"
        className="rounded px-1 py-0.5 text-[13px] font-semibold tracking-tight text-slate-900 transition hover:bg-slate-100 focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-slate-400"
        onClick={() => setAbout(true)}
        aria-haspopup="dialog"
        aria-expanded={about}
      >
        GenMSI
      </button>
      <span className="h-4 w-px bg-slate-200" aria-hidden />
      <ModelIdentity />
      <div className="ml-auto">
        <RunConsole />
      </div>
      {about && <AboutDialog onClose={close} />}
    </header>
  );
}
