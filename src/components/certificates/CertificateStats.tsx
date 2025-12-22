import { ArrowRight } from "lucide-react";

function StatCard({
  title,
  value,
  description,
  tone = "default",
}: {
  title: string;
  value: number;
  description: string;
  tone?: "default" | "warning";
}) {
  const toneClass =
    tone === "warning"
      ? "text-amber-600 bg-amber-50 dark:bg-amber-950/30"
      : "text-primary bg-primary/10";

  return (
    <div className="rounded-xl border bg-card p-4 shadow-soft">
      <div className="flex items-center justify-between">
        <div>
          <div className="text-sm font-semibold text-muted-foreground">
            {title}
          </div>
          <div className="text-2xl font-bold text-foreground">{value}</div>
          <div className="text-xs text-muted-foreground">{description}</div>
        </div>
        <div
          className={`flex h-10 w-10 items-center justify-center rounded-lg text-sm font-semibold ${toneClass}`}
        >
          <ArrowRight className="h-4 w-4" />
        </div>
      </div>
    </div>
  );
}

export function CertificateStats({
  managedCount,
  externalCount,
  expiringSoon,
}: {
  managedCount: number;
  externalCount: number;
  expiringSoon: number;
}) {
  return (
    <div className="grid gap-4 sm:grid-cols-3">
      <StatCard
        title="Managed"
        value={managedCount}
        description="Issued by this app"
      />
      <StatCard
        title="External"
        value={externalCount}
        description="Imported or discovered"
      />
      <StatCard
        title="Expiring soon"
        value={expiringSoon}
        description="< 30 days to expiry"
        tone="warning"
      />
    </div>
  );
}
