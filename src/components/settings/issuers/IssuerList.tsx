import { Button } from "../../ui/button";
import type { IssuerConfig } from "../../../lib/issuers";
import { formatEnvironment, formatIssuerType } from "../../../lib/issuers/format";

interface IssuerListProps {
  issuers: IssuerConfig[];
  issuerLoading: boolean;
  onEdit: (issuer: IssuerConfig) => void;
  onDelete: (issuer: IssuerConfig) => void;
}

export function IssuerList({
  issuers,
  issuerLoading,
  onEdit,
  onDelete,
}: IssuerListProps) {
  return (
    <div className="space-y-3">
      {issuers.map((issuer) => (
        <div
          key={issuer.issuer_id}
          className="rounded-lg border bg-background/80 p-4 shadow-sm"
        >
          <div className="flex flex-wrap items-start justify-between gap-3">
            <div>
              <div className="flex flex-wrap items-center gap-2 text-sm font-semibold">
                {issuer.label}
              </div>
              <div className="mt-1 text-xs text-muted-foreground">
                {formatIssuerType(issuer.issuer_type)} · {formatEnvironment(issuer.environment)} ·{" "}
                {issuer.tos_agreed ? "ToS accepted" : "ToS pending"}
              </div>
              <div className="mt-1 text-xs text-muted-foreground">
                {issuer.directory_url}
              </div>
            </div>
            <div className="flex flex-wrap gap-2">
              <Button
                type="button"
                variant="outline"
                size="sm"
                onClick={() => onEdit(issuer)}
                disabled={issuerLoading}
              >
                Edit
              </Button>
              <Button
                type="button"
                variant="ghost"
                size="sm"
                className="text-destructive hover:bg-destructive/10"
                onClick={() => void onDelete(issuer)}
                disabled={issuerLoading}
              >
                Remove
              </Button>
            </div>
          </div>
        </div>
      ))}
    </div>
  );
}

