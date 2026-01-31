import { ExternalLink, Radar } from "lucide-react";
import { Button } from "../components/ui/button";
import { PageHeader } from "../components/page-header";

export function DiscoverPage() {
  return (
    <div className="space-y-6">
      <PageHeader
        title="Discover"
        description="Find certificates across infrastructure and consolidate inventory."
      />

      <div className="rounded-xl border bg-card p-8 text-center">
        <div className="mx-auto mb-6 flex h-16 w-16 items-center justify-center rounded-full bg-primary/10 text-primary">
          <Radar className="h-8 w-8" />
        </div>
        <h2 className="mb-3 text-2xl font-bold text-foreground">
          Certificate Discovery
        </h2>
        <p className="mx-auto mb-8 max-w-md text-base text-muted-foreground">
          Network scanning and Certificate Transparency discovery are available
          in our cloud edition. Keep your certificate inventory fresh across
          your entire infrastructure.
        </p>
        <Button asChild size="lg">
          <a href="https://sslboard.com" target="_blank" rel="noopener noreferrer">
            Try SSLBoard Cloud
            <ExternalLink className="ml-2 h-5 w-5" />
          </a>
        </Button>
      </div>
    </div>
  );
}
