import type { PropagationResult } from "../../lib/dns";
import type { StartIssuanceResponse } from "../../lib/issuance";
import { InstructionField } from "./InstructionField";
import { StatusBadge } from "./StatusBadge";

function formatRecordName(recordName: string) {
  return recordName.startsWith("_acme-challenge.")
    ? recordName.replace("_acme-challenge.", "")
    : recordName;
}

export function InstructionCard({
  record,
  status,
}: {
  record: StartIssuanceResponse["dns_records"][number];
  status: PropagationResult | null;
}) {
  return (
    <div className="space-y-2 rounded-lg border bg-background p-3">
      <div className="flex items-center justify-between gap-2 text-xs font-semibold uppercase text-muted-foreground">
        {formatRecordName(record.record_name)}
        {status ? <StatusBadge state={status.state} /> : null}
      </div>
      <InstructionField label="Record name" value={record.record_name} />
      <InstructionField label="Value" value={record.value} />
      <InstructionField label="Zone" value={record.zone} />
    </div>
  );
}
