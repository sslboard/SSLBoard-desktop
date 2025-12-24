import { Loader2 } from "lucide-react";
import { Button } from "../ui/button";
import { DnsProviderPreviewCard } from "./DnsProviderPreviewCard";
import type { DnsProviderResolution } from "../../lib/dns-providers";

interface DomainsInputCardProps {
  domainsInput: string;
  parsedDomains: string[];
  issuerLabel: string;
  issuerEnvironment: string;
  issuerReady: boolean;
  loadingStart: boolean;
  hasStartResult: boolean;
  providerPreview: Record<string, DnsProviderResolution | null>;
  providerLoading: boolean;
  providerError: string | null;
  onDomainsChange: (value: string) => void;
  onStart: () => void;
  onReset: () => void;
}

export function DomainsInputCard({
  domainsInput,
  parsedDomains,
  issuerLabel,
  issuerEnvironment,
  issuerReady,
  loadingStart,
  hasStartResult,
  providerPreview,
  providerLoading,
  providerError,
  onDomainsChange,
  onStart,
  onReset,
}: DomainsInputCardProps) {
  return (
    <div className="rounded-xl border bg-card p-6 shadow-soft">
      <div className="flex items-start justify-between gap-4">
        <div>
          <div className="flex items-center gap-2 text-sm font-semibold text-muted-foreground">
            Managed key · DNS-01
          </div>
          <h2 className="text-xl font-bold text-foreground">Issue a certificate</h2>
          <p className="mt-2 text-sm text-muted-foreground">
            Enter the domains/SANs, start issuance to get TXT instructions, confirm propagation,
            then finalize. Private keys stay in the OS keychain.
          </p>
        </div>
        <div className="hidden rounded-lg border bg-muted px-3 py-2 text-xs text-muted-foreground sm:block">
          Your private key stays on your machine, encrypted at rest.
        </div>
      </div>

      <div className="mt-6 space-y-4">
        <label className="space-y-2 text-sm">
          <span className="text-muted-foreground">Domains / SANs</span>
          <textarea
            className="w-full rounded-md border bg-background px-3 py-2 text-foreground shadow-sm outline-none focus:border-primary"
            value={domainsInput}
            onChange={(e) => onDomainsChange(e.target.value)}
            rows={3}
            placeholder="test.ezs3.net, test1.ezs3.net"
          />
          <p className="text-xs text-muted-foreground">
            Comma or newline separated. Issuer: {issuerLabel} ({issuerEnvironment}).
          </p>
        </label>

        {parsedDomains.length > 0 ? (
          <DnsProviderPreviewCard
            domains={parsedDomains}
            providerPreview={providerPreview}
            providerLoading={providerLoading}
            providerError={providerError}
          />
        ) : null}

        <div className="flex flex-wrap gap-3">
          <Button
            onClick={() => void onStart()}
            disabled={loadingStart || !parsedDomains.length || !issuerReady}
          >
            {loadingStart && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
            Start issuance
          </Button>
          <Button variant="outline" onClick={onReset} disabled={!hasStartResult}>
            Reset
          </Button>
        </div>
        {!issuerReady ? (
          <p className="text-xs text-muted-foreground">
            Configure the issuer&apos;s ACME details in Settings before starting issuance.
          </p>
        ) : null}
      </div>
    </div>
  );
}

