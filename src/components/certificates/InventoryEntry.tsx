import { ShieldCheck } from "lucide-react";
import type { CertificateRecord } from "../../lib/certificates";

function formatDate(dateString: string) {
  const date = new Date(dateString);
  return date.toLocaleDateString(undefined, {
    month: "short",
    day: "numeric",
    year: "numeric",
  });
}

function daysUntil(dateString: string) {
  const now = Date.now();
  const target = new Date(dateString).getTime();
  return Math.round((target - now) / (1000 * 60 * 60 * 24));
}

function certificateStatus(record: CertificateRecord) {
  const days = daysUntil(record.not_after);
  if (days < 0) {
    return { label: "Expired", tone: "text-red-500 bg-red-50 dark:bg-red-950/40" };
  }
  if (days < 30) {
    return {
      label: `Expiring in ${days}d`,
      tone: "text-amber-500 bg-amber-50 dark:bg-amber-950/40",
    };
  }
  return {
    label: `Healthy · ${days}d left`,
    tone: "text-emerald-600 bg-emerald-50 dark:bg-emerald-950/40",
  };
}

function primarySubject(record: CertificateRecord) {
  return record?.subjects[0] ?? record?.sans[0] ?? record?.domain_roots[0] ?? "—";
}

interface InventoryEntryProps {
  record: CertificateRecord;
  isSelected: boolean;
  onClick: () => void;
}

export function InventoryEntry({ record, isSelected, onClick }: InventoryEntryProps) {
  const status = certificateStatus(record);
  const subject = primarySubject(record);

  return (
    <button
      onClick={onClick}
      className={`flex w-full items-start gap-4 rounded-lg px-3 py-3 text-left transition ${isSelected ? "bg-primary/5 ring-1 ring-primary" : "hover:bg-muted/60"
        }`}
    >
      <div className="flex h-10 w-10 flex-shrink-0 items-center justify-center rounded-lg bg-primary/10 text-primary">
        <ShieldCheck className="h-5 w-5" />
      </div>
      <div className="flex flex-1 flex-col gap-2">
        <div className="flex flex-wrap items-center gap-2">
          <span className="text-sm font-semibold text-foreground">{subject}</span>
          <span
            className={`rounded-full px-2 py-1 text-xs font-semibold ${status.tone}`}
          >
            {status.label}
          </span>
          <span className="rounded-full bg-muted px-2 py-1 text-xs">
            {record.source}
          </span>
        </div>
        <div className="text-xs text-muted-foreground">
          Issuer · {record.issuer}
        </div>
        <div className="text-xs text-muted-foreground">
          Serial {record.serial}
        </div>
        <div className="text-xs text-muted-foreground">
          Valid {formatDate(record.not_before)} – {formatDate(record.not_after)}
        </div>
      </div>
    </button>
  );
}

