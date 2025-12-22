import { cn } from "../../lib/utils";
import type { PropagationResult } from "../../lib/dns";

export function StatusBadge({ state }: { state: PropagationResult["state"] }) {
  const styles = {
    found: "bg-emerald-100 text-emerald-700 border-emerald-200",
    pending: "bg-amber-50 text-amber-700 border-amber-200",
    wrong_content: "bg-orange-100 text-orange-700 border-orange-200",
    nx_domain: "bg-rose-100 text-rose-700 border-rose-200",
    error: "bg-rose-100 text-rose-700 border-rose-200",
  } as const;

  const label = {
    found: "Found",
    pending: "Waiting",
    wrong_content: "Mismatch",
    nx_domain: "Not found",
    error: "Error",
  }[state];

  return (
    <span
      className={cn(
        "inline-flex items-center gap-2 rounded-full border px-2 py-1 text-[11px] font-semibold",
        styles[state],
      )}
    >
      {label}
    </span>
  );
}
