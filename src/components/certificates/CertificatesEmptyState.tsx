import { Compass, ShieldCheck, Sparkles } from "lucide-react";
import { Button } from "../ui/button";

export function CertificatesEmptyState({
  onIssue,
  onDiscover,
}: {
  onIssue: () => void;
  onDiscover: () => void;
}) {
  return (
    <div className="rounded-xl border bg-gradient-to-br from-primary/5 via-card to-secondary/10 p-8 shadow-soft">
      <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
        <div className="space-y-2">
          <div className="flex items-center gap-2 text-sm font-semibold uppercase tracking-wide text-primary">
            <Sparkles className="h-4 w-4" />
            No certificates yet
          </div>
          <h3 className="text-2xl font-bold text-foreground">
            Inventory is empty — start by importing, discovering, or issuing.
          </h3>
          <p className="max-w-2xl text-sm text-muted-foreground">
            Bring existing certificates into view, or kick off a new issuance.
          </p>
          <div className="flex flex-wrap gap-3">
            <Button onClick={onIssue}>
              <ShieldCheck className="mr-2 h-4 w-4" />
              Issue a certificate
            </Button>
            <Button variant="secondary" onClick={onDiscover}>
              <Compass className="mr-2 h-4 w-4" />
              Discover via CT
            </Button>
          </div>
        </div>
        <div className="hidden rounded-xl border bg-card/80 p-4 text-sm text-muted-foreground shadow-sm sm:block sm:max-w-xs">
          <div className="flex items-center gap-2 text-foreground">
            <ShieldCheck className="h-4 w-4 text-primary" />
            Metadata-only inventory
          </div>
          <p className="mt-2">
            Stored fields: SANs, issuer, serial, validity window, fingerprint,
            source, domain roots, and tags. No private keys, no secrets.
          </p>
        </div>
      </div>
    </div>
  );
}
