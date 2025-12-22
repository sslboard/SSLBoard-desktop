import { useEffect, useState } from "react";
import { AlertTriangle, CheckCircle2, Loader2, ShieldCheck } from "lucide-react";
import { Link } from "react-router-dom";
import { Button } from "../components/ui/button";
import { PageHeader } from "../components/page-header";
import { checkDnsPropagation, type PropagationResult } from "../lib/dns";
import {
  completeManagedIssuance,
  startManagedIssuance,
  type StartIssuanceResponse,
} from "../lib/issuance";
import { normalizeError } from "../lib/errors";
import { InstructionCard } from "../components/issue/InstructionCard";
import { ProviderPreviewRow } from "../components/issue/ProviderPreviewRow";
import { useIssuerOptions } from "../hooks/useIssuerOptions";
import { useProviderPreview } from "../hooks/useProviderPreview";

type StatusMap = Record<string, PropagationResult | null>;

export function IssuePage() {
  const [domainsInput, setDomainsInput] = useState("test.ezs3.net");
  const [startResult, setStartResult] = useState<StartIssuanceResponse | null>(null);
  const [statusMap, setStatusMap] = useState<StatusMap>({});
  const [loadingStart, setLoadingStart] = useState(false);
  const [checking, setChecking] = useState(false);
  const [finalizing, setFinalizing] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);
  const {
    issuers,
    issuerLoading,
    issuerError,
    selectedIssuer,
    selectIssuerById,
  } = useIssuerOptions();

  const parsedDomains = domainsInput
    .split(/[\s,]+/)
    .map((d) => d.trim().toLowerCase())
    .filter(Boolean);

  const { providerPreview, providerLoading, providerError } =
    useProviderPreview(parsedDomains);

  useEffect(() => {
    if (!startResult) {
      setStatusMap({});
    }
  }, [startResult]);

  const issuerLabel = selectedIssuer?.label ?? "No issuer selected";
  const issuerEnvironment = selectedIssuer?.environment ?? "staging";
  const issuerDescription =
    issuerEnvironment === "production" ? "production" : "sandbox";
  const issuerReady = Boolean(
    selectedIssuer &&
      selectedIssuer.contact_email &&
      selectedIssuer.account_key_ref &&
      selectedIssuer.tos_agreed &&
      !selectedIssuer.disabled,
  );

  function handleSelectIssuer(issuerId: string) {
    selectIssuerById(issuerId);
  }

  async function handleStart() {
    setLoadingStart(true);
    setError(null);
    setSuccessMessage(null);
    try {
      if (!selectedIssuer) {
        throw new Error("Select an issuer before starting issuance.");
      }
      const result = await startManagedIssuance({
        domains: parsedDomains,
        issuer_id: selectedIssuer.issuer_id,
      });
      setStartResult(result);
      const initialStatus: StatusMap = {};
      result.dns_records.forEach((rec) => {
        initialStatus[rec.record_name] = null;
      });
      setStatusMap(initialStatus);
    } catch (err) {
      setError(normalizeError(err));
      setStartResult(null);
    } finally {
      setLoadingStart(false);
    }
  }

  async function checkAll() {
    if (!startResult) return;
    setChecking(true);
    setError(null);
    try {
      const updates: StatusMap = {};
      for (const rec of startResult.dns_records) {
        const domain = rec.record_name.replace(/^_acme-challenge\./, "");
        const result = await checkDnsPropagation(domain, rec.value);
        updates[rec.record_name] = result;
      }
      setStatusMap((prev) => ({ ...prev, ...updates }));
    } catch (err) {
      setError(normalizeError(err));
    } finally {
      setChecking(false);
    }
  }

  const allFound =
    startResult &&
    Object.values(statusMap).length === startResult.dns_records.length &&
    Object.values(statusMap).every((s) => s?.state === "found");

  const manualRecords = startResult?.dns_records.filter((rec) => rec.adapter === "manual") ?? [];
  const managedRecords = startResult?.dns_records.filter((rec) => rec.adapter !== "manual") ?? [];
  const hasManual = manualRecords.length > 0;
  const hasManaged = managedRecords.length > 0;
  const dnsModeLabel = hasManual && hasManaged
    ? "mixed"
    : hasManual
      ? "manual"
      : startResult?.dns_records[0]?.adapter ?? "manual";

  async function finalizeIssuance() {
    if (!startResult) return;
    setFinalizing(true);
    setError(null);
    try {
      const record = await completeManagedIssuance({
        request_id: startResult.request_id,
      });
      setSuccessMessage(
        `Issued ${record.subjects[0]} — expires ${new Date(record.not_after).toLocaleDateString()}`,
      );
    } catch (err) {
      setError(normalizeError(err));
    } finally {
      setFinalizing(false);
    }
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title="Issue"
        description={`Issue a ${issuerDescription} certificate via ACME DNS-01 with automatic providers or manual fallback.`}
        action={
          <Button asChild variant="secondary">
            <Link to="/certificates">
              <ShieldCheck className="mr-2 h-4 w-4" />
              View certificates
            </Link>
          </Button>
        }
      />

      {issuerError ? (
        <div className="rounded-lg border border-destructive/50 bg-destructive/10 px-4 py-3 text-sm text-destructive">
          {issuerError}
        </div>
      ) : null}

      {selectedIssuer && issuerEnvironment === "staging" ? (
        <div className="flex items-start gap-3 rounded-xl border border-amber-300 bg-amber-50 px-4 py-3 text-sm text-amber-900 shadow-soft">
          <AlertTriangle className="mt-0.5 h-4 w-4" />
          <div>
            <div className="font-semibold">Sandbox issuer active</div>
            <p className="text-[13px] text-amber-900/80">
              Using Let's Encrypt staging. Safe for end-to-end testing without issuing real certificates.
            </p>
          </div>
        </div>
      ) : null}

      <div className="rounded-xl border bg-card p-6 shadow-soft">
        <div className="flex items-center justify-between gap-3">
          <div>
            <div className="text-sm font-semibold">Issuer selection</div>
            <p className="text-xs text-muted-foreground">
              Choose the issuer for this issuance run.
            </p>
          </div>
        </div>

        <div className="mt-4 space-y-2">
          <label className="text-sm font-medium text-foreground">
            Issuer
          </label>
          <select
            className="w-full rounded-lg border bg-background/60 p-2.5 text-sm shadow-inner outline-none ring-offset-background focus:ring-2 focus:ring-primary/50"
            value={selectedIssuer?.issuer_id ?? ""}
            onChange={(e) => handleSelectIssuer(e.target.value)}
            disabled={issuerLoading}
          >
            {issuers.map((issuer) => (
              <option
                key={issuer.issuer_id}
                value={issuer.issuer_id}
                disabled={issuer.disabled}
              >
                {issuer.label}
                {issuer.disabled ? " (disabled)" : ""}
              </option>
            ))}
          </select>
          <p className="text-xs text-muted-foreground">
            {selectedIssuer?.directory_url ?? "https://acme-staging-v02.api.letsencrypt.org/directory"}
          </p>
          {!issuerReady ? (
            <p className="text-xs text-muted-foreground">
              Configure the issuer&apos;s ACME account in Settings before issuing.
            </p>
          ) : null}
        </div>
      </div>

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
              onChange={(e) => setDomainsInput(e.target.value)}
              rows={3}
              placeholder="test.ezs3.net, test1.ezs3.net"
            />
            <p className="text-xs text-muted-foreground">
              Comma or newline separated. Issuer: {issuerLabel} ({issuerEnvironment}).
            </p>
          </label>

          {parsedDomains.length ? (
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
                {parsedDomains.map((domain) => (
                  <ProviderPreviewRow
                    key={domain}
                    domain={domain}
                    resolution={providerPreview[domain] ?? null}
                  />
                ))}
              </div>
            </div>
          ) : null}

          <div className="flex flex-wrap gap-3">
            <Button
              onClick={() => void handleStart()}
              disabled={loadingStart || !parsedDomains.length || !issuerReady}
            >
              {loadingStart && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
              Start issuance
            </Button>
            <Button variant="outline" onClick={() => setStartResult(null)} disabled={!startResult}>
              Reset
            </Button>
          </div>
          {!issuerReady ? (
            <p className="text-xs text-muted-foreground">
              Select an enabled issuer and configure its ACME details in Settings before starting issuance.
            </p>
          ) : null}

          {error && (
            <div className="flex items-center gap-2 rounded-md bg-destructive/10 px-3 py-2 text-sm text-destructive">
              <AlertTriangle className="h-4 w-4" />
              {error}
            </div>
          )}

          {startResult && (
            <div className="space-y-4 rounded-lg border bg-muted/40 p-4">
              <div className="flex items-center gap-2 text-xs uppercase tracking-wide text-muted-foreground">
                DNS instructions
                <span className="rounded-full bg-primary/10 px-2 py-0.5 text-[11px] font-semibold text-primary">
                  {dnsModeLabel}
                </span>
              </div>
              {hasManaged && !hasManual && (
                <div className="rounded-md border bg-background px-3 py-2 text-xs text-muted-foreground">
                  Automatic DNS provider is configured. TXT records are being created for you.
                </div>
              )}
              {hasManaged && hasManual && (
                <div className="rounded-md border bg-background px-3 py-2 text-xs text-muted-foreground">
                  Some DNS records are handled automatically. Remaining records require manual setup.
                </div>
              )}
              {hasManual && (
                <div className="grid gap-3 md:grid-cols-2">
                  {manualRecords.map((rec) => (
                    <InstructionCard
                      key={rec.record_name}
                      record={rec}
                      status={statusMap[rec.record_name]}
                    />
                  ))}
                </div>
              )}
              <div className="flex flex-wrap gap-3">
                <Button variant="secondary" onClick={() => void checkAll()} disabled={checking}>
                  {checking && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                  {hasManual
                    ? "I&apos;ve added the TXT records — check propagation"
                    : "Check DNS propagation"}
                </Button>
                <Button onClick={() => void finalizeIssuance()} disabled={!allFound || finalizing}>
                  {finalizing && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                  Finalize issuance
                </Button>
              </div>
            </div>
          )}

          {successMessage && (
            <div className="flex items-center gap-2 rounded-md bg-emerald-50 px-3 py-2 text-sm text-emerald-700">
              <CheckCircle2 className="h-4 w-4" />
              {successMessage}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
