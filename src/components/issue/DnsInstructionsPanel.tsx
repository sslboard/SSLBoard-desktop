import { Loader2 } from "lucide-react";
import { Button } from "../ui/button";
import { InstructionCard } from "./InstructionCard";
import type { StartIssuanceResponse } from "../../lib/issuance";

interface DnsInstructionsPanelProps {
  hasManual: boolean;
  hasManaged: boolean;
  dnsModeLabel: string;
  manualRecords: StartIssuanceResponse["dns_records"];
  finalizing: boolean;
  awaitingManual: boolean;
  finalizeFailed: boolean;
  hasCertificate: boolean;
  onContinue: () => void;
  onRetryFinalize: () => void;
}

export function DnsInstructionsPanel({
  hasManual,
  hasManaged,
  dnsModeLabel,
  manualRecords,
  finalizing,
  awaitingManual,
  finalizeFailed,
  hasCertificate,
  onContinue,
  onRetryFinalize,
}: DnsInstructionsPanelProps) {
  const dnsStatus = awaitingManual
    ? "Waiting on you"
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
          <span>DNS verification</span>
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
              />
            );
          })}
        </div>
      )}
      {(showContinue || showRetryFinalize) && (
        <div className="flex flex-wrap gap-3">
          {showContinue && (
            <Button variant="secondary" onClick={() => void onContinue()} disabled={finalizing}>
              {finalizing && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
              Continue issuance
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
