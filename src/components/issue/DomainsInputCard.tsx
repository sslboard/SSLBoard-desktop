import { Info, Loader2 } from "lucide-react";
import { Button } from "../ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "../ui/card";
import { Checkbox } from "../ui/checkbox";
import { Label } from "../ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "../ui/select";
import { Textarea } from "../ui/textarea";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "../ui/tooltip";
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
  reuseKeyAvailable?: boolean;
  reuseKeyEnabled?: boolean;
  onDomainsChange: (value: string) => void;
  onKeyOptionChange: (value: IssuanceKeyOption) => void;
  onReuseKeyToggle?: (value: boolean) => void;
  onStart: () => void;
  onReset: () => void;
  disabled?: boolean;
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
  reuseKeyAvailable = false,
  reuseKeyEnabled = false,
  onDomainsChange,
  onKeyOptionChange,
  onReuseKeyToggle,
  onStart,
  onReset,
  disabled = false,
}: DomainsInputCardProps) {
  return (
    <Card className="shadow-soft">
      <CardHeader className="flex-row items-start justify-between gap-4 space-y-0">
        <div>
          <div className="flex items-center gap-2 text-sm font-semibold text-muted-foreground">
            Managed key · DNS-01
          </div>
          <CardTitle className="text-xl font-bold">Issue a certificate</CardTitle>
          <p className="mt-2 text-sm text-muted-foreground">
            Enter the domains/SANs, then start issuance. We will verify DNS and finalize
            automatically, pausing only if manual TXT records are required. Private keys stay in
            the OS keychain.
          </p>
        </div>
        <div className="hidden rounded-lg border bg-muted px-3 py-2 text-xs text-muted-foreground sm:block">
          Your private key stays on your machine, encrypted at rest.
        </div>
      </CardHeader>

      <CardContent className="space-y-4">
        <div className="space-y-2 text-sm">
          <Label className="text-muted-foreground" htmlFor="domains-input">
            Domains / SANs
          </Label>
          <Textarea
            id="domains-input"
            value={domainsInput}
            onChange={(e) => onDomainsChange(e.target.value.normalize("NFC"))}
            rows={3}
            placeholder="test.ezs3.net, test1.ezs3.net"
          />
          <p className="text-xs text-muted-foreground">
            Comma or newline separated. Issuer: {issuerLabel} ({issuerEnvironment}).
          </p>
        </div>

        {parsedDomains.length > 0 ? (
          <DnsProviderPreviewCard
            domains={parsedDomains}
            providerPreview={providerPreview}
            providerLoading={providerLoading}
            providerError={providerError}
          />
        ) : null}

        <div className="space-y-2 text-sm">
          <Label className="text-muted-foreground">Key algorithm</Label>
          <Select
            value={keyOption}
            onValueChange={(value) =>
              onKeyOptionChange(value as IssuanceKeyOption)
            }
            disabled={loadingStart || hasStartResult || reuseKeyEnabled}
          >
            <SelectTrigger>
              <SelectValue placeholder="Select key algorithm" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="rsa-2048">RSA 2048</SelectItem>
              <SelectItem value="rsa-3072">RSA 3072</SelectItem>
              <SelectItem value="rsa-4096">RSA 4096</SelectItem>
              <SelectItem value="ecdsa-p256">ECDSA P-256</SelectItem>
              <SelectItem value="ecdsa-p384">ECDSA P-384</SelectItem>
            </SelectContent>
          </Select>
          <p className="text-xs text-muted-foreground">
            Choose a key type and size/curve for this issuance run.
          </p>
        </div>

        {reuseKeyAvailable ? (
          <div className="space-y-2 text-sm">
            <div className="flex items-center justify-between gap-3">
              <div className="flex items-center gap-2">
                <Checkbox
                  id="reuse-key"
                  checked={reuseKeyEnabled}
                  onCheckedChange={(value) =>
                    onReuseKeyToggle?.(Boolean(value))
                  }
                  disabled={loadingStart || hasStartResult}
                />
                <Label htmlFor="reuse-key">Reuse existing private key</Label>
                <TooltipProvider>
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <span className="inline-flex h-5 w-5 items-center justify-center rounded-full border text-muted-foreground">
                        <Info className="h-3 w-3" />
                      </span>
                    </TooltipTrigger>
                    <TooltipContent>
                      Reusing the key preserves pinning/allowlists but skips key rotation.
                    </TooltipContent>
                  </Tooltip>
                </TooltipProvider>
              </div>
              <span className="text-xs text-muted-foreground">Managed key found</span>
            </div>
            <p className="text-xs text-muted-foreground">
              Turn this off to generate a fresh key pair for stronger rotation hygiene.
            </p>
          </div>
        ) : null}

        <div className="flex flex-wrap gap-3">
          <Button
            onClick={() => void onStart()}
            disabled={disabled || loadingStart || hasStartResult || !parsedDomains.length || !issuerReady}
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
      </CardContent>
    </Card>
  );
}
