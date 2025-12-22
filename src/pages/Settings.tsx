import { Shield, Lock, KeyRound } from "lucide-react";
import { PageHeader } from "../components/page-header";
import { IssuerManager } from "../components/settings/IssuerManager";
import { SecretReferenceManager } from "../components/settings/SecretReferenceManager";

export function SettingsPage() {
  return (
    <div className="space-y-6">
      <PageHeader
        title="Settings"
        description="Configure providers, guardrails, and secret references. Secret values are only sent into Rust once."
      />
      <IssuerManager />
      <SecretReferenceManager />

      <div className="grid gap-4 md:grid-cols-3">
        <div className="rounded-xl border bg-card p-5 shadow-soft">
          <div className="flex items-center gap-3">
            <Lock className="h-5 w-5 text-primary" />
            <div className="font-semibold">Access & policy</div>
          </div>
          <p className="mt-2 text-sm text-muted-foreground">
            Configure guardrails and audit preferences. Secret values stay inside Rust.
          </p>
        </div>
        <div className="rounded-xl border bg-card p-5 shadow-soft">
          <div className="flex items-center gap-3">
            <KeyRound className="h-5 w-5 text-primary" />
            <div className="font-semibold">Key handling</div>
          </div>
          <p className="mt-2 text-sm text-muted-foreground">
            Managed keys remain reference-only; UI never renders private material.
          </p>
        </div>
        <div className="rounded-xl border bg-card p-5 shadow-soft">
          <div className="flex items-center gap-3">
            <Shield className="h-5 w-5 text-primary" />
            <div className="font-semibold">Providers</div>
          </div>
          <p className="mt-2 text-sm text-muted-foreground">
            ACME and DNS providers will attach to secret refs when configured.
          </p>
        </div>
      </div>
    </div>
  );
}
