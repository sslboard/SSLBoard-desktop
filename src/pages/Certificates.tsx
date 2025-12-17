import { ShieldCheck, UploadCloud } from "lucide-react";
import { Button } from "../components/ui/button";
import { PageHeader } from "../components/page-header";

export function CertificatesPage() {
  return (
    <div className="space-y-6">
      <PageHeader
        title="Certificates"
        description="Inventory of issued certificates, their status, and renewal posture."
        action={
          <Button>
            <UploadCloud className="mr-2 h-4 w-4" />
            Import inventory
          </Button>
        }
      />
      <div className="grid gap-4 sm:grid-cols-2">
        <div className="rounded-xl border bg-card p-5 shadow-soft">
          <div className="flex items-center gap-3">
            <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-primary/10 text-primary">
              <ShieldCheck className="h-5 w-5" />
            </div>
            <div>
              <div className="text-sm font-semibold text-muted-foreground">
                Inventory status
              </div>
              <div className="text-lg font-bold">Placeholder</div>
            </div>
          </div>
          <p className="mt-3 text-sm text-muted-foreground">
            This area will surface certificate lists, issuance dates, and renewal
            readiness once wiring is connected.
          </p>
        </div>
        <div className="rounded-xl border bg-card p-5 shadow-soft">
          <div className="text-sm font-semibold text-muted-foreground">
            Recent activity
          </div>
          <p className="mt-3 text-sm text-muted-foreground">
            Activity feed and certificate detail views will land here.
          </p>
        </div>
      </div>
    </div>
  );
}
