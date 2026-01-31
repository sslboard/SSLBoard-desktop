import { PageHeader } from "../components/page-header";
import { IssuerManager } from "../components/settings/IssuerManager";

export function SettingsPage() {
  return (
    <div className="space-y-6">
      <PageHeader
        title="Issuers"
        description="Configure ACME providers for certificate issuance."
      />
      <IssuerManager />
    </div>
  );
}
