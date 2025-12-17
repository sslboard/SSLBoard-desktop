import { useEffect, useRef, useState } from "react";
import {
  Wand2,
  ArrowRight,
  Copy,
  Loader2,
  CheckCircle2,
  AlertTriangle,
  Clock,
} from "lucide-react";
import { Button } from "../components/ui/button";
import { PageHeader } from "../components/page-header";
import {
  checkDnsPropagation,
  prepareDnsChallenge,
  type PreparedDnsChallenge,
  type PropagationResult,
} from "../lib/dns";
import { cn } from "../lib/utils";

export function IssuePage() {
  const [domain, setDomain] = useState("example.com");
  const [txtValue, setTxtValue] = useState("token-value-from-acme");
  const [prepared, setPrepared] = useState<PreparedDnsChallenge | null>(null);
  const [preparing, setPreparing] = useState(false);
  const [checking, setChecking] = useState(false);
  const [status, setStatus] = useState<PropagationResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [elapsed, setElapsed] = useState(0);
  const timerRef = useRef<number | null>(null);
  const startTimeRef = useRef<number | null>(null);

  useEffect(() => {
    return () => {
      if (timerRef.current) {
        window.clearTimeout(timerRef.current);
      }
    };
  }, []);

  async function handlePrepare() {
    setPreparing(true);
    setError(null);
    setStatus(null);
    try {
      const result = await prepareDnsChallenge({
        domain: domain.trim(),
        txt_value: txtValue.trim(),
      });
      setPrepared(result);
    } catch (err) {
      setError(normalizeError(err));
      setPrepared(null);
    } finally {
      setPreparing(false);
    }
  }

  async function pollOnce(): Promise<PropagationResult> {
    return checkDnsPropagation(domain.trim(), txtValue.trim());
  }

  async function startPolling() {
    if (!prepared) {
      setError("Generate DNS instructions first.");
      return;
    }
    if (checking) return;

    setChecking(true);
    setError(null);
    setStatus(null);
    setElapsed(0);
    startTimeRef.current = Date.now();

    const tick = async () => {
      try {
        const result = await pollOnce();
        setStatus(result);

        const now = Date.now();
        if (startTimeRef.current) {
          setElapsed(Math.floor((now - startTimeRef.current) / 1000));
        }

        if (result.state === "found") {
          setChecking(false);
          return;
        }

        const timedOut =
          startTimeRef.current !== null && now - startTimeRef.current > 90_000;
        if (timedOut) {
          setChecking(false);
          setStatus((prev) => ({
            state: prev?.state === "found" ? "found" : "error",
            reason:
              prev?.state === "found"
                ? prev.reason
                : "Timed out after 90s without seeing the expected TXT record.",
            observed_values: prev?.observed_values ?? [],
          }));
          return;
        }

        timerRef.current = window.setTimeout(tick, 2000);
      } catch (err) {
        setChecking(false);
        setError(normalizeError(err));
      }
    };

    await tick();
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title="Issue"
        description="Start an issuance request, pick validation, and provision certificates."
        action={
          <Button variant="secondary">
            <Wand2 className="mr-2 h-4 w-4" />
            New issuance
          </Button>
        }
      />
      <div className="rounded-xl border bg-card p-6 shadow-soft">
        <div className="flex items-start justify-between gap-4">
          <div>
            <div className="flex items-center gap-2 text-sm font-semibold text-muted-foreground">
              Wizard preview
            </div>
            <h2 className="text-xl font-bold text-foreground">ACME workflow</h2>
            <p className="mt-2 text-sm text-muted-foreground">
              A guided wizard will collect domain details, validation approach,
              and output certificate artifacts.
            </p>
          </div>
          <div className="hidden rounded-lg border bg-muted px-3 py-2 text-xs text-muted-foreground sm:block">
            Unprivileged UI: no keys or secrets stored here.
          </div>
        </div>
        <div className="mt-4 flex items-center gap-3 text-sm text-muted-foreground">
          <span className="flex h-8 w-8 items-center justify-center rounded-full bg-primary/10 text-primary">
            <Wand2 className="h-4 w-4" />
          </span>
          <span>Issuance steps will appear here soon.</span>
          <ArrowRight className="h-4 w-4 text-muted-foreground" />
        </div>
        <div className="mt-6 rounded-xl border bg-card p-4 shadow-sm">
          <div className="flex items-start justify-between gap-4">
            <div>
              <div className="flex items-center gap-2 text-sm font-semibold text-muted-foreground">
                DNS-01 manual flow
              </div>
              <h2 className="text-xl font-bold text-foreground">
                Provide the TXT record, then poll for propagation
              </h2>
              <p className="mt-2 text-sm text-muted-foreground">
                Enter the domain and TXT value from your ACME order. We&apos;ll
                show the exact record name/value and poll every 2s until it
                shows up (90s budget).
              </p>
            </div>
          </div>

          <div className="mt-4 grid gap-4 md:grid-cols-2">
            <label className="space-y-2 text-sm">
              <span className="text-muted-foreground">Domain</span>
              <input
                className="w-full rounded-md border bg-background px-3 py-2 text-foreground shadow-sm outline-none focus:border-primary"
                value={domain}
                onChange={(e) => setDomain(e.target.value)}
                placeholder="example.com"
              />
            </label>
            <label className="space-y-2 text-sm">
              <span className="text-muted-foreground">TXT value</span>
              <input
                className="w-full rounded-md border bg-background px-3 py-2 text-foreground shadow-sm outline-none focus:border-primary"
                value={txtValue}
                onChange={(e) => setTxtValue(e.target.value)}
                placeholder="value from ACME challenge"
              />
            </label>
          </div>

          <div className="mt-4 flex flex-wrap gap-3">
            <Button onClick={() => void handlePrepare()} disabled={preparing}>
              {preparing && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
              Generate instructions
            </Button>
            <Button
              variant="secondary"
              onClick={() => void startPolling()}
              disabled={!prepared || checking}
            >
              {checking && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
              I&apos;ve added it — check propagation
            </Button>
          </div>

          {error && (
            <div className="mt-3 flex items-center gap-2 rounded-md bg-destructive/10 px-3 py-2 text-sm text-destructive">
              <AlertTriangle className="h-4 w-4" />
              {error}
            </div>
          )}

          {prepared && (
            <div className="mt-4 rounded-lg border bg-muted/40 p-4 text-sm">
              <div className="flex items-center gap-2 text-xs uppercase tracking-wide text-muted-foreground">
                Manual instructions
                <span className="rounded-full bg-primary/10 px-2 py-0.5 text-[11px] font-semibold text-primary">
                  {prepared.record.adapter}
                </span>
              </div>
              <div className="mt-2 grid gap-3 md:grid-cols-3">
                <InstructionField label="Record name" value={prepared.record.record_name} />
                <InstructionField label="Value" value={prepared.record.value} />
                <InstructionField label="Zone" value={prepared.record.zone} />
              </div>
            </div>
          )}

          {status && (
            <div className="mt-4 rounded-lg border bg-background p-4 text-sm">
              <div className="flex flex-wrap items-center gap-2">
                <StatusBadge state={status.state} />
                <span className="text-muted-foreground">
                  Elapsed: {elapsed}s (polling every 2s, 90s max)
                </span>
              </div>
              {status.reason && (
                <p className="mt-2 text-muted-foreground">{status.reason}</p>
              )}
              {status.observed_values.length > 0 && (
                <div className="mt-3">
                  <div className="text-xs font-semibold uppercase text-muted-foreground">
                    Observed TXT values
                  </div>
                  <ul className="mt-2 space-y-1">
                    {status.observed_values.map((val) => (
                      <li
                        key={val}
                        className="rounded-md border bg-muted/60 px-2 py-1 font-mono text-xs"
                      >
                        {val}
                      </li>
                    ))}
                  </ul>
                </div>
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

function InstructionField({ label, value }: { label: string; value: string }) {
  return (
    <div className="space-y-1">
      <div className="text-xs font-semibold uppercase text-muted-foreground">
        {label}
      </div>
      <div className="flex items-center justify-between gap-2 rounded-md border bg-background px-3 py-2 font-mono text-xs">
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
    found: "Propagation found",
    pending: "Waiting for DNS",
    wrong_content: "TXT value mismatch",
    nx_domain: "Record not found (NXDOMAIN)",
    error: "Propagation error",
  }[state];

  const icon = {
    found: CheckCircle2,
    pending: Clock,
    wrong_content: AlertTriangle,
    nx_domain: AlertTriangle,
    error: AlertTriangle,
  }[state];

  const Icon = icon;

  return (
    <span
      className={cn(
        "inline-flex items-center gap-2 rounded-full border px-3 py-1 text-xs font-semibold",
        styles[state],
      )}
    >
      <Icon className="h-4 w-4" />
      {label}
    </span>
  );
}

function normalizeError(err: unknown): string {
  if (err instanceof Error) return err.message;
  if (typeof err === "string") return err;
  return "Unexpected error";
}
