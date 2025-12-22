import { AlertTriangle } from "lucide-react";
import type { DnsProviderResolution } from "../../lib/dns-providers";

export function ProviderPreviewRow({
  domain,
  resolution,
}: {
  domain: string;
  resolution: DnsProviderResolution | null;
}) {
  if (!resolution) {
    return (
      <div className="flex items-center justify-between rounded-md border bg-muted/40 px-3 py-2 text-xs text-muted-foreground">
        <span>{domain}</span>
        <span>Unable to resolve</span>
      </div>
    );
  }

  const provider = resolution.provider;
  const ambiguous = resolution.ambiguous.length > 1;
  const label = provider?.label ?? "Manual DNS required";
  const type = provider?.provider_type ?? "manual";
  const suffix = resolution.matched_suffix;

  return (
    <div className="rounded-md border bg-muted/30 px-3 py-2 text-xs">
      <div className="flex flex-wrap items-center justify-between gap-2">
        <span className="font-medium text-foreground">{domain}</span>
        <span className="rounded-full bg-primary/10 px-2 py-0.5 text-[11px] font-semibold uppercase text-primary">
          {type}
        </span>
      </div>
      <div className="mt-1 text-muted-foreground">
        {label}
        {suffix ? ` · matches ${suffix}` : ""}
      </div>
      {ambiguous ? (
        <div className="mt-1 flex items-center gap-2 text-amber-700">
          <AlertTriangle className="h-3.5 w-3.5" />
          Ambiguous: {resolution.ambiguous.map((entry) => entry.label).join(", ")}
        </div>
      ) : null}
    </div>
  );
}
