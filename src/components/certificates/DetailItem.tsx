export function DetailItem({
  label,
  value,
  truncate = false,
}: {
  label: string;
  value: string;
  truncate?: boolean;
}) {
  return (
    <div className="rounded-lg border bg-muted/40 p-3">
      <div className="text-xs uppercase tracking-wide text-muted-foreground">
        {label}
      </div>
      <div
        className={`text-sm font-semibold text-foreground ${truncate ? "truncate" : ""}`}
        title={truncate ? value : undefined}
      >
        {value}
      </div>
    </div>
  );
}
