import { useEffect, useState } from "react";
import { AlertTriangle, CheckCircle2, Copy, Loader2, ShieldCheck } from "lucide-react";
import { Link } from "react-router-dom";
import { Button } from "../components/ui/button";
import { PageHeader } from "../components/page-header";
import { checkDnsPropagation, type PropagationResult } from "../lib/dns";
import {
  completeManagedIssuance,
  startManagedIssuance,
  type StartIssuanceResponse,
} from "../lib/issuance";
import { cn } from "../lib/utils";

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

  useEffect(() => {
    if (!startResult) {
      setStatusMap({});
    }
  }, [startResult]);

  const parsedDomains = domainsInput
    .split(/[\s,]+/)
    .map((d) => d.trim().toLowerCase())
    .filter(Boolean);

  async function handleStart() {
    setLoadingStart(true);
    setError(null);
    setSuccessMessage(null);
    try {
      const result = await startManagedIssuance({ domains: parsedDomains });
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
        description="Issue a sandbox certificate via ACME DNS-01 (manual DNS adapter)."
        action={
          <Button asChild variant="secondary">
            <Link to="/certificates">
              <ShieldCheck className="mr-2 h-4 w-4" />
              View certificates
            </Link>
          </Button>
        }
      />

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
              Comma or newline separated. ACME staging issuer is used by default.
            </p>
          </label>

          <div className="flex flex-wrap gap-3">
            <Button onClick={() => void handleStart()} disabled={loadingStart || !parsedDomains.length}>
              {loadingStart && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
              Start issuance
            </Button>
            <Button variant="outline" onClick={() => setStartResult(null)} disabled={!startResult}>
              Reset
            </Button>
          </div>

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
                  manual
                </span>
              </div>
              <div className="grid gap-3 md:grid-cols-2">
                {startResult.dns_records.map((rec) => (
                  <InstructionCard key={rec.record_name} record={rec} status={statusMap[rec.record_name]} />
                ))}
              </div>
              <div className="flex flex-wrap gap-3">
                <Button variant="secondary" onClick={() => void checkAll()} disabled={checking}>
                  {checking && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                  I&apos;ve added the TXT records — check propagation
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

function InstructionCard({
  record,
  status,
}: {
  record: StartIssuanceResponse["dns_records"][number];
  status: PropagationResult | null;
}) {
  return (
    <div className="space-y-2 rounded-lg border bg-background p-3">
      <div className="flex items-center justify-between gap-2 text-xs font-semibold uppercase text-muted-foreground">
        {_acme(record.record_name)}
        {status ? <StatusBadge state={status.state} /> : null}
      </div>
      <InstructionField label="Record name" value={record.record_name} />
      <InstructionField label="Value" value={record.value} />
      <InstructionField label="Zone" value={record.zone} />
    </div>
  );
}

function InstructionField({ label, value }: { label: string; value: string }) {
  return (
    <div className="space-y-1">
      <div className="text-[11px] font-semibold uppercase tracking-wide text-muted-foreground">
        {label}
      </div>
      <div className="flex items-center justify-between gap-2 rounded-md border bg-muted/60 px-3 py-2 font-mono text-xs">
        <span className="truncate">{value}</span>
        <Button
          variant="ghost"
          size="sm"
          className="h-7 px-2 text-xs"
          onClick={() => void navigator.clipboard.writeText(value)}
        >
          <Copy className="mr-1 h-3.5 w-3.5" />
          Copy
        </Button>
      </div>
    </div>
  );
}

function StatusBadge({ state }: { state: PropagationResult["state"] }) {
  const styles = {
    found: "bg-emerald-100 text-emerald-700 border-emerald-200",
    pending: "bg-amber-50 text-amber-700 border-amber-200",
    wrong_content: "bg-orange-100 text-orange-700 border-orange-200",
    nx_domain: "bg-rose-100 text-rose-700 border-rose-200",
    error: "bg-rose-100 text-rose-700 border-rose-200",
  } as const;

  const label = {
    found: "Found",
    pending: "Waiting",
    wrong_content: "Mismatch",
    nx_domain: "Not found",
    error: "Error",
  }[state];

  return (
    <span
      className={cn(
        "inline-flex items-center gap-2 rounded-full border px-2 py-1 text-[11px] font-semibold",
        styles[state],
      )}
    >
      {label}
    </span>
  );
}

function _acme(recordName: string) {
  return recordName.startsWith("_acme-challenge.")
    ? recordName.replace("_acme-challenge.", "")
    : recordName;
}

function normalizeError(err: unknown): string {
  if (err instanceof Error) return err.message;
  if (typeof err === "string") return err;
  return "Unexpected error";
}
