import { Loader2 } from "lucide-react";
import { useMemo } from "react";
import { Button } from "../ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "../ui/card";
import { Label } from "../ui/label";
import { DnsProviderPreviewCard } from "./DnsProviderPreviewCard";
import { useProviderPreview } from "../../hooks/useProviderPreview";
import type { CsrValidationResult } from "../../lib/issuance";

interface CsrIssuanceCardProps {
  issuerLabel: string;
  issuerEnvironment: string;
  issuerReady: boolean;
  loadingStart: boolean;
  hasStartResult: boolean;
  csrPath: string | null;
  csrResult: CsrValidationResult | null;
  csrLoading: boolean;
  csrError: string | null;
  onSelectCsr: () => void;
  onClearCsr: () => void;
  onStart: () => void;
  onReset: () => void;
  disabled?: boolean;
}

function formatKeyInfo(result: CsrValidationResult | null): string {
  if (!result) return "Unknown";
  if (result.metadata.key_algorithm === "rsa") {
    return `RSA ${result.metadata.key_size ?? "?"}`;
  }
  if (result.metadata.key_curve === "p256") {
    return "ECDSA P-256";
  }
  if (result.metadata.key_curve === "p384") {
    return "ECDSA P-384";
  }
  return "ECDSA";
}

export function CsrIssuanceCard({
  issuerLabel,
  issuerEnvironment,
  issuerReady,
  loadingStart,
  hasStartResult,
  csrPath,
  csrResult,
  csrLoading,
  csrError,
  onSelectCsr,
  onClearCsr,
  onStart,
  onReset,
  disabled = false,
}: CsrIssuanceCardProps) {
  // Extract identifiers for DNS provider preview display only
  // We don't block issuance based on this - the backend validates the CSR
  const identifiers = useMemo(() => {
    if (!csrResult) return [];
    // Use identifiers if available
    if (csrResult.identifiers.length > 0) {
      return csrResult.identifiers;
    }
    // Fallback to SANs for preview purposes
    if (csrResult.metadata.sans.length > 0) {
      return csrResult.metadata.sans;
    }
    // If no identifiers or SANs, return empty (preview won't show, but issuance can still proceed)
    return [];
  }, [csrResult]);
  const { providerPreview, providerLoading, providerError } = useProviderPreview(identifiers);

  return (
    <Card className="shadow-soft">
      <CardHeader className="flex-row items-start justify-between gap-4 space-y-0">
        <div>
          <div className="flex items-center gap-2 text-sm font-semibold text-muted-foreground">
            CSR import · DNS-01
          </div>
          <CardTitle className="text-xl font-bold">Issue with a CSR</CardTitle>
          <p className="mt-2 text-sm text-muted-foreground">
            Select a CSR file to derive the issuance identifiers automatically. We&apos;ll
            validate the CSR in the core before starting ACME issuance.
          </p>
        </div>
        <div className="hidden rounded-lg border bg-muted px-3 py-2 text-xs text-muted-foreground sm:block">
          CSR contents never leave the trusted core.
        </div>
      </CardHeader>

      <CardContent className="space-y-4">
        <div className="space-y-2 text-sm">
          <Label className="text-muted-foreground">CSR file</Label>
          <div className="flex flex-wrap items-center gap-2">
            <Button
              variant="outline"
              onClick={onSelectCsr}
              disabled={csrLoading || loadingStart || hasStartResult}
            >
              {csrLoading && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
              {csrPath ? "Choose different CSR" : "Select CSR file"}
            </Button>
            {csrPath && (
              <Button variant="ghost" onClick={onClearCsr} disabled={loadingStart}>
                Clear
              </Button>
            )}
          </div>
          <p className="text-xs text-muted-foreground">
            {csrPath ?? "No CSR selected yet."}
          </p>
          <p className="text-xs text-muted-foreground">
            Issuer: {issuerLabel} ({issuerEnvironment}).
          </p>
        </div>

        {csrError ? (
          <div className="rounded-lg border border-rose-200 bg-rose-50 px-3 py-2 text-xs text-rose-700">
            {csrError}
          </div>
        ) : null}

        {csrResult ? (
          <div className="space-y-3 rounded-lg border bg-muted/40 p-3 text-sm">
            <div>
              <div className="text-xs font-semibold uppercase tracking-wide text-muted-foreground">
                CSR subject
              </div>
              <div className="text-sm text-foreground">{csrResult.metadata.subject}</div>
            </div>
            <div>
              <div className="text-xs font-semibold uppercase tracking-wide text-muted-foreground">
                CSR SANs
              </div>
              <div className="text-xs text-muted-foreground">
                {csrResult.metadata.sans.length
                  ? csrResult.metadata.sans.join(", ")
                  : "No SANs in CSR"}
              </div>
            </div>
            <div className="text-xs text-muted-foreground">
              Key: {formatKeyInfo(csrResult)}
            </div>
            {csrResult.warnings.length ? (
              <div className="rounded-md border border-amber-200 bg-amber-50 px-2 py-2 text-xs text-amber-900">
                {csrResult.warnings.join(" ")}
              </div>
            ) : null}
          </div>
        ) : null}

        {identifiers.length > 0 ? (
          <DnsProviderPreviewCard
            domains={identifiers}
            providerPreview={providerPreview}
            providerLoading={providerLoading}
            providerError={providerError}
          />
        ) : null}

        <div className="flex flex-wrap gap-3">
          <Button
            onClick={() => void onStart()}
            disabled={
              disabled ||
              loadingStart ||
              hasStartResult ||
              !issuerReady ||
              !csrResult ||
              !!csrError
            }
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
