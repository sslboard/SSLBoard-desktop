import { Shield, Lock, KeyRound } from "lucide-react";
import { PageHeader } from "../components/page-header";

export function SettingsPage() {
  return (
    <div className="space-y-6">
      <PageHeader
        title="Settings"
        description="Configure providers, identity, and guardrails. Secrets are never rendered."
      />
      <div className="grid gap-4 md:grid-cols-3">
        <div className="rounded-xl border bg-card p-5 shadow-soft">
          <div className="flex items-center gap-3">
            <Shield className="h-5 w-5 text-primary" />
            <div className="font-semibold">Providers</div>
          </div>
          <p className="mt-2 text-sm text-muted-foreground">
            Connect ACME CAs, DNS services, and secret stores. UI only surfaces
            metadata; credentials stay outside the shell.
          </p>
        </div>
        <div className="rounded-xl border bg-card p-5 shadow-soft">
          <div className="flex items-center gap-3">
            <Lock className="h-5 w-5 text-primary" />
            <div className="font-semibold">Access & policy</div>
          </div>
          <p className="mt-2 text-sm text-muted-foreground">
            Manage roles, audit preferences, and safe defaults for issuance.
          </p>
        </div>
        <div className="rounded-xl border bg-card p-5 shadow-soft">
          <div className="flex items-center gap-3">
            <KeyRound className="h-5 w-5 text-primary" />
            <div className="font-semibold">Key handling</div>
          </div>
          <p className="mt-2 text-sm text-muted-foreground">
            Placeholder for key custody choices. Private keys are never displayed
            or persisted in this UI.
          </p>
        </div>
      </div>
    </div>
  );
}
