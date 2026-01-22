import { ShieldCheck } from "lucide-react";
import { useNavigate } from "react-router-dom";
import type { CertificateRecord } from "../../lib/certificates";
import { Button } from "../ui/button";
import {
  certificateStatus,
  daysUntil,
  formatCertificateDate,
  primarySubject,
} from "./certificate-utils";
import { buildRenewalPrefill } from "./renewal-utils";

interface InventoryEntryProps {
  record: CertificateRecord;
  isSelected: boolean;
  onClick: () => void;
}

export function InventoryEntry({ record, isSelected, onClick }: InventoryEntryProps) {
  const status = certificateStatus(record);
  const subject = primarySubject(record);
  const navigate = useNavigate();
  const showQuickRenew =
    (record.sans.length > 0 || record.subjects.length > 0) &&
    daysUntil(record.not_after) < 30;

  return (
    <div
      className={`flex w-full items-start gap-3 rounded-lg px-3 py-3 transition ${
        isSelected ? "bg-primary/5 ring-1 ring-primary" : "hover:bg-muted/60"
      }`}
    >
      <Button
        variant="ghost"
        onClick={onClick}
        className="flex h-auto flex-1 items-start justify-start gap-4 rounded-lg p-0 text-left font-normal"
      >
        <div className="flex h-10 w-10 flex-shrink-0 items-center justify-center rounded-lg bg-primary/10 text-primary">
          <ShieldCheck className="h-5 w-5" />
        </div>
        <div className="flex flex-1 flex-col gap-2">
          <div className="flex flex-wrap items-center gap-2">
            <span className="text-sm font-semibold text-foreground">{subject}</span>
            <span
              className={`rounded-full px-2 py-1 text-xs font-semibold ${status.tone}`}
            >
              {status.label}
            </span>
            <span className="rounded-full bg-muted px-2 py-1 text-xs">
              {record.source}
            </span>
          </div>
          <div className="text-xs text-muted-foreground">
            Issuer · {record.issuer}
          </div>
          <div className="text-xs text-muted-foreground">
            Serial {record.serial}
          </div>
          <div className="text-xs text-muted-foreground">
            Valid {formatCertificateDate(record.not_before)} – {formatCertificateDate(record.not_after)}
          </div>
        </div>
      </Button>
      {showQuickRenew ? (
        <Button
          size="sm"
          variant="outline"
          onClick={(event) => {
            event.stopPropagation();
            const renewal = buildRenewalPrefill(record);
            navigate("/issue", { state: { renewal } });
          }}
        >
          Renew
        </Button>
      ) : null}
    </div>
  );
}
