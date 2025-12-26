import { Loader2 } from "lucide-react";
import { Button } from "../ui/button";
import { InstructionCard } from "./InstructionCard";
import type { StartIssuanceResponse } from "../../lib/issuance";
import type { PropagationResult } from "../../lib/dns";

interface DnsInstructionsPanelProps {
  statusMap: Record<string, PropagationResult | null>;
  hasManual: boolean;
  hasManaged: boolean;
  dnsModeLabel: string;
  manualRecords: StartIssuanceResponse["dns_records"];
  checking: boolean;
  finalizing: boolean;
  awaitingManual: boolean;
  dnsFailed: boolean;
  finalizeFailed: boolean;
  hasCertificate: boolean;
  onContinue: () => void;
  onRetryDns: () => void;
  onRetryFinalize: () => void;
}

export function DnsInstructionsPanel({
  statusMap,
  hasManual,
  hasManaged,
  dnsModeLabel,
  manualRecords,
  checking,
  finalizing,
  awaitingManual,
  dnsFailed,
  finalizeFailed,
  hasCertificate,
  onContinue,
  onRetryDns,
  onRetryFinalize,
}: DnsInstructionsPanelProps) {
  const dnsStatus = awaitingManual
    ? "Waiting on you"
    : dnsFailed
      ? "Needs retry"
      : checking
        ? "Running"
        : hasCertificate
          ? "Complete"
          : "Queued";
  const finalizeStatus = hasCertificate
    ? "Complete"
    : finalizeFailed
      ? "Needs retry"
      : finalizing
        ? "Running"
        : "Queued";
  const showContinue = hasManual && awaitingManual;
  const showRetryDns = dnsFailed;
  const showRetryFinalize = finalizeFailed;

  return (
    <div className="space-y-4 rounded-lg border bg-muted/40 p-4">
      <div className="flex items-center gap-2 text-xs uppercase tracking-wide text-muted-foreground">
        Issuance progress
        <span className="rounded-full bg-primary/10 px-2 py-0.5 text-[11px] font-semibold text-primary">
          {dnsModeLabel}
        </span>
      </div>
      <div className="grid gap-2 text-sm text-muted-foreground">
        <div className="flex items-center justify-between rounded-md border bg-background px-3 py-2">
          <span>Start issuance</span>
          <span className="text-xs font-semibold text-emerald-600">Complete</span>
        </div>
        <div className="flex items-center justify-between rounded-md border bg-background px-3 py-2">
          <span className="flex items-center gap-2">
            DNS verification
            {checking && (
              <span className="inline-flex items-center gap-1 text-xs text-muted-foreground">
                <Loader2 className="h-3 w-3 animate-spin" />
                Testing propagation
              </span>
            )}
          </span>
          <span className="text-xs font-semibold">{dnsStatus}</span>
        </div>
        <div className="flex items-center justify-between rounded-md border bg-background px-3 py-2">
          <span>Finalize issuance</span>
          <span className="text-xs font-semibold">{finalizeStatus}</span>
        </div>
      </div>
      {hasManaged && !hasManual && (
        <div className="rounded-md border bg-background px-3 py-2 text-xs text-muted-foreground">
          Automatic DNS provider is configured. Monitoring propagation and finalizing automatically.
        </div>
      )}
      {hasManaged && hasManual && (
        <div className="rounded-md border bg-background px-3 py-2 text-xs text-muted-foreground">
          Some DNS records are handled automatically. Add the manual TXT records, then continue.
        </div>
      )}
      {hasManual && (
        <div className="grid gap-3 md:grid-cols-2">
          {manualRecords.map((rec) => {
            const recordKey = `${rec.record_name}:${rec.value}`;
            return (
              <InstructionCard
                key={recordKey}
                record={rec}
                status={statusMap[recordKey]}
              />
            );
          })}
        </div>
      )}
      {(showContinue || showRetryDns || showRetryFinalize) && (
        <div className="flex flex-wrap gap-3">
          {showContinue && (
            <Button variant="secondary" onClick={() => void onContinue()} disabled={checking}>
              {checking && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
              Continue issuance
            </Button>
          )}
          {showRetryDns && (
            <Button variant="secondary" onClick={() => void onRetryDns()} disabled={checking}>
              {checking && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
              Retry DNS verification
            </Button>
          )}
          {showRetryFinalize && (
            <Button onClick={() => void onRetryFinalize()} disabled={finalizing}>
              {finalizing && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
              Retry finalization
            </Button>
          )}
        </div>
      )}
    </div>
  );
}
