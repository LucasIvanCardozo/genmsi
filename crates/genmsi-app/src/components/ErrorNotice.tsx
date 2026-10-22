import { AlertTriangle } from "lucide-react";

/**
 * The failure the engine reported.
 *
 * A model the engine refuses is an error the user reads, not an empty model.
 */
export default function ErrorNotice({ message }: { message: string }) {
  return (
    <div
      role="alert"
      className="flex items-start gap-3 rounded-lg border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-900"
    >
      <AlertTriangle className="mt-0.5 size-4 shrink-0 text-red-500" aria-hidden />
      <p className="font-mono text-[13px] leading-relaxed break-words">{message}</p>
    </div>
  );
}
