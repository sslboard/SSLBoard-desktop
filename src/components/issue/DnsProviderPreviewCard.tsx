import { AlertTriangle, Loader2 } from "lucide-react";
import type { DnsProviderResolution } from "../../lib/dns-providers";
import { ProviderPreviewRow } from "./ProviderPreviewRow";

interface DnsProviderPreviewCardProps {
  domains: string[];
  providerPreview: Record<string, DnsProviderResolution | null>;
  providerLoading: boolean;
  providerError: string | null;
}

export function DnsProviderPreviewCard({
  domains,
  providerPreview,
  providerLoading,
  providerError,
}: DnsProviderPreviewCardProps) {
  return (
    <div className="rounded-lg border bg-background/70 p-3 shadow-sm">
      <div className="flex items-center justify-between gap-2 text-xs font-semibold uppercase tracking-wide text-muted-foreground">
        DNS provider preview
        {providerLoading ? (
          <span className="inline-flex items-center gap-1 text-[11px]">
            <Loader2 className="h-3.5 w-3.5 animate-spin" />
            Resolving
          </span>
        ) : null}
      </div>
      {providerError ? (
        <div className="mt-2 flex items-center gap-2 rounded-md bg-destructive/10 px-2 py-1 text-xs text-destructive">
          <AlertTriangle className="h-3.5 w-3.5" />
          {providerError}
        </div>
      ) : null}
      <div className="mt-2 space-y-2">
        {domains.map((domain) => (
          <ProviderPreviewRow
            key={domain}
            domain={domain}
            resolution={providerPreview[domain] ?? null}
          />
        ))}
      </div>
    </div>
  );
}

