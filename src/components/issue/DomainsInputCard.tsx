import { Loader2 } from "lucide-react";
import { Button } from "../ui/button";
import { DnsProviderPreviewCard } from "./DnsProviderPreviewCard";
import type { DnsProviderResolution } from "../../lib/dns-providers";
import type { IssuanceKeyOption } from "../../lib/issuance";

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
  keyOption: IssuanceKeyOption;
  onDomainsChange: (value: string) => void;
  onKeyOptionChange: (value: IssuanceKeyOption) => void;
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
  keyOption,
  onDomainsChange,
  onKeyOptionChange,
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
            Enter the domains/SANs, then start issuance. We will verify DNS and finalize
            automatically, pausing only if manual TXT records are required. Private keys stay in
            the OS keychain.
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

        <label className="space-y-2 text-sm">
          <span className="text-muted-foreground">Key algorithm</span>
          <select
            className="w-full rounded-md border bg-background px-3 py-2 text-sm text-foreground shadow-sm outline-none focus:border-primary"
            value={keyOption}
            onChange={(e) =>
              onKeyOptionChange(
                e.target.value as IssuanceKeyOption,
              )
            }
            disabled={loadingStart || hasStartResult}
          >
            <option value="rsa-2048">RSA 2048</option>
            <option value="rsa-3072">RSA 3072</option>
            <option value="rsa-4096">RSA 4096</option>
            <option value="ecdsa-p256">ECDSA P-256</option>
            <option value="ecdsa-p384">ECDSA P-384</option>
          </select>
          <p className="text-xs text-muted-foreground">
            Choose a key type and size/curve for this issuance run.
          </p>
        </label>

        <div className="flex flex-wrap gap-3">
          <Button
            onClick={() => void onStart()}
            disabled={loadingStart || hasStartResult || !parsedDomains.length || !issuerReady}
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
