import { RefreshCw, ShieldCheck } from "lucide-react";
import { Link } from "react-router-dom";
import { Button } from "../ui/button";
import type { CertificateRecord } from "../../lib/certificates";
import { InventoryEntry } from "./InventoryEntry";

interface InventoryProps {
  records: CertificateRecord[];
  selectedId: string | null;
  onSelect: (id: string) => void;
  loading: boolean;
  onRefresh: () => void;
}

export function Inventory({
  records,
  selectedId,
  onSelect,
  loading,
  onRefresh,
}: InventoryProps) {
  if (loading) {
    return (
      <div className="rounded-xl border bg-card p-6 text-sm text-muted-foreground shadow-soft">
        Loading inventory...
      </div>
    );
  }

  return (
    <div className="rounded-xl border bg-card p-4 shadow-soft">
      <div className="flex items-center justify-between gap-3 border-b pb-3">
        <div>
          <div className="text-sm font-semibold text-muted-foreground">Inventory</div>
          <div className="text-lg font-bold text-foreground">
            {records.length} certificate{records.length === 1 ? "" : "s"}
          </div>
        </div>
        <div className="flex items-center gap-2">
          <Button variant="ghost" size="sm" onClick={onRefresh}>
            <RefreshCw className="mr-2 h-4 w-4" />
            Refresh
          </Button>
          <Button asChild size="sm">
            <Link to="/issue">
              <ShieldCheck className="mr-2 h-4 w-4" />
              Issue new
            </Link>
          </Button>
        </div>
      </div>
      <div className="mt-3 divide-y">
        {records.length === 0 ? (
          <div className="rounded-lg border border-dashed bg-muted/40 p-4 text-sm text-muted-foreground">
            No certificates yet.
          </div>
        ) : (
          records.map((record) => (
            <InventoryEntry
              key={record.id}
              record={record}
              isSelected={selectedId === record.id}
              onClick={() => onSelect(record.id)}
            />
          ))
        )}
      </div>
    </div>
  );
}

