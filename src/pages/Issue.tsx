import { Wand2, ArrowRight } from "lucide-react";
import { Button } from "../components/ui/button";
import { PageHeader } from "../components/page-header";

export function IssuePage() {
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
      </div>
    </div>
  );
}
