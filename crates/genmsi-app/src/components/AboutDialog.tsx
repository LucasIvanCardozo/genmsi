import { useEffect, useRef } from "react";
import { createPortal } from "react-dom";
import { useApp } from "../state/store";
import { groupLabel, textButton } from "./chrome";

/**
 * The formats every release publishes.
 *
 * They are the contract of the release: the same formats, with the same
 * version, in every system.
 */
const FORMATS = [
  ".deb (x86_64 y aarch64)",
  ".rpm (x86_64 y aarch64)",
  ".pkg.tar.zst (x86_64)",
  ".dmg (universal)",
  "-setup.exe y .msi (x64)",
  ".flatpak",
  ".tar.gz (portátil)",
  "SHA256SUMS",
];

/**
 * The box that says what this app is.
 *
 * It answers the question a reader has when something goes wrong: which
 * version is installed. It is opened from the header and it is closed with
 * `Escape` or with the button, so the keyboard alone can open it and close
 * it.
 */
export default function AboutDialog({ onClose }: { onClose: () => void }) {
  const version = useApp((state) => state.appVersion);
  const panel = useRef<HTMLDivElement>(null);

  // The box takes the focus when it opens: the reader can close it without
  // reaching for the mouse.
  useEffect(() => {
    panel.current?.focus();
  }, []);

  // `Escape` closes the box: it is the key every dialog closes with.
  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key !== "Escape") {
        return;
      }
      event.stopPropagation();
      onClose();
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [onClose]);

  return createPortal(
    <div
      className="fixed inset-0 z-50 flex items-start justify-center bg-slate-900/20 pt-24"
      onClick={onClose}
    >
      <div
        ref={panel}
        role="dialog"
        aria-modal="true"
        aria-label="Acerca de GenMSI"
        tabIndex={-1}
        className="w-[420px] rounded-lg border border-slate-200 bg-white p-4 shadow-xl outline-none"
        onClick={(event) => event.stopPropagation()}
      >
        <h2 className="text-[13px] font-semibold tracking-tight text-slate-900">
          Acerca de GenMSI
        </h2>
        <p className="mt-1 text-[12px] text-slate-600">
          GenMSI es un simulador de eventos discretos al estilo GPSS: se
          escribe un modelo con bloques, se corre y se ve la corrida reproducida,
          con sus gráficos y su reporte.
        </p>

        <p className="mt-3 text-[12px] text-slate-700">
          <span className={groupLabel}>Versión</span>
          <span className="ml-2 font-mono text-slate-900">
            {version ?? "—"}
          </span>
        </p>

        <p className="mt-3">
          <span className={groupLabel}>Formatos publicados</span>
        </p>
        <ul className="mt-1 grid grid-cols-2 gap-x-3 gap-y-0.5 text-[11.5px] text-slate-600">
          {FORMATS.map((format) => (
            <li key={format} className="truncate">
              {format}
            </li>
          ))}
        </ul>

        <p className="mt-3">
          <span className={groupLabel}>Licencia</span>
        </p>
        <p className="mt-1 text-[11.5px] text-slate-600">
          MIT. GenMSI es software libre: se puede usar, copiar y modificar,
          y la licencia viaja con el programa. Autor: Lucas Cardozo.
        </p>

        <div className="mt-4 flex justify-end">
          <button type="button" className={textButton} onClick={onClose}>
            Cerrar
          </button>
        </div>
      </div>
    </div>,
    document.body,
  );
}
