import { Radar, RefreshCw } from "lucide-react";
import { Button } from "../components/ui/button";
import { PageHeader } from "../components/page-header";

export function DiscoverPage() {
  return (
    <div className="space-y-6">
      <PageHeader
        title="Discover"
        description="Find certificates across infrastructure and consolidate inventory."
        action={
          <Button variant="outline">
            <RefreshCw className="mr-2 h-4 w-4" />
            Start scan
          </Button>
        }
      />
      <div className="rounded-xl border bg-card p-6 shadow-soft">
        <div className="flex items-start gap-3">
          <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-primary/10 text-primary">
            <Radar className="h-5 w-5" />
          </div>
          <div className="space-y-1">
            <h2 className="text-lg font-bold text-foreground">
              Discovery placeholder
            </h2>
            <p className="text-sm text-muted-foreground">
              Configure network ranges, providers, and schedulers here to keep
              certificate inventory fresh.
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}
