import { Shield, Lock, KeyRound } from "lucide-react";
import { PageHeader } from "../components/page-header";
import { IssuerManager } from "../components/settings/IssuerManager";
import { SecretReferenceManager } from "../components/settings/SecretReferenceManager";
import { Card, CardContent, CardHeader, CardTitle } from "../components/ui/card";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "../components/ui/tabs";

export function SettingsPage() {
  return (
    <div className="space-y-6">
      <PageHeader
        title="Settings"
        description="Configure providers, guardrails, and secret references. Secret values are only sent into Rust once."
      />
      <Tabs defaultValue="issuers" className="space-y-4">
        <TabsList>
          <TabsTrigger value="issuers">Issuers</TabsTrigger>
          <TabsTrigger value="secrets">Secret references</TabsTrigger>
        </TabsList>
        <TabsContent value="issuers">
          <IssuerManager />
        </TabsContent>
        <TabsContent value="secrets">
          <SecretReferenceManager />
        </TabsContent>
      </Tabs>

      <div className="grid gap-4 md:grid-cols-3">
        <Card className="shadow-soft">
          <CardHeader className="flex-row items-center gap-3 space-y-0">
            <Lock className="h-5 w-5 text-primary" />
            <CardTitle className="text-base">Access & policy</CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-sm text-muted-foreground">
              Configure guardrails and audit preferences. Secret values stay inside Rust.
            </p>
          </CardContent>
        </Card>
        <Card className="shadow-soft">
          <CardHeader className="flex-row items-center gap-3 space-y-0">
            <KeyRound className="h-5 w-5 text-primary" />
            <CardTitle className="text-base">Key handling</CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-sm text-muted-foreground">
              Managed keys remain reference-only; UI never renders private material.
            </p>
          </CardContent>
        </Card>
        <Card className="shadow-soft">
          <CardHeader className="flex-row items-center gap-3 space-y-0">
            <Shield className="h-5 w-5 text-primary" />
            <CardTitle className="text-base">Providers</CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-sm text-muted-foreground">
              ACME and DNS providers will attach to secret refs when configured.
            </p>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
