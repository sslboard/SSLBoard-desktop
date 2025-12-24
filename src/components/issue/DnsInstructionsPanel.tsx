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
  allFound: boolean;
  finalizing: boolean;
  onCheckAll: () => void;
  onFinalize: () => void;
}

export function DnsInstructionsPanel({
  statusMap,
  hasManual,
  hasManaged,
  dnsModeLabel,
  manualRecords,
  checking,
  allFound,
  finalizing,
  onCheckAll,
  onFinalize,
}: DnsInstructionsPanelProps) {
  return (
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
      <div className="flex flex-wrap gap-3">
        <Button variant="secondary" onClick={() => void onCheckAll()} disabled={checking}>
          {checking && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
          {hasManual
            ? "I've added the TXT records — check propagation"
            : "Check DNS propagation"}
        </Button>
        <Button onClick={() => void onFinalize()} disabled={!allFound || finalizing}>
          {finalizing && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
          Finalize issuance
        </Button>
      </div>
    </div>
  );
}

