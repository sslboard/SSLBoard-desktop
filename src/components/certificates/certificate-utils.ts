import type { CertificateRecord } from "../../lib/certificates";

export function formatCertificateDate(dateString: string) {
  const date = new Date(dateString);
  return date.toLocaleDateString(undefined, {
    month: "short",
    day: "numeric",
    year: "numeric",
  });
}

export function primarySubject(record: CertificateRecord | null) {
  return record?.subjects[0] ?? record?.sans[0] ?? record?.domain_roots[0] ?? "—";
}

export function daysUntil(dateString: string) {
  const now = Date.now();
  const target = new Date(dateString).getTime();
  return Math.round((target - now) / (1000 * 60 * 60 * 24));
}

export function certificateStatus(record: CertificateRecord) {
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
