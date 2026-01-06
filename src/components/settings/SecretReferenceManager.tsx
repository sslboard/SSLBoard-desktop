import { useMemo } from "react";
import { KeyRound, RefreshCw } from "lucide-react";
import { Button } from "../ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "../ui/card";
import { cn } from "../../lib/utils";
import { type SecretKind } from "../../lib/secrets";
import { useSecretReferences } from "../../hooks/useSecretReferences";

export function SecretReferenceManager() {
  const {
    secrets,
    loading,
    error,
    refresh,
  } = useSecretReferences();

  const hasSecrets = useMemo(() => secrets.length > 0, [secrets]);

  function formatKind(kind: SecretKind) {
    switch (kind) {
      case "dns_provider_token":
        return "DNS provider token";
      case "dns_provider_access_key":
        return "DNS provider access key";
      case "dns_provider_secret_key":
        return "DNS provider secret key";
      case "acme_account_key":
        return "ACME account key";
      case "managed_private_key":
        return "Managed private key";
      default:
        return kind;
    }
  }

  function formatDate(iso: string) {
    const date = new Date(iso);
    return Number.isNaN(date.getTime())
      ? "—"
      : date.toLocaleString(undefined, {
        month: "short",
        day: "numeric",
        year: "numeric",
        hour: "2-digit",
        minute: "2-digit",
      });
  }

  return (
    <div className="space-y-4">
      {error ? (
        <div className="rounded-lg border border-destructive/50 bg-destructive/10 px-4 py-3 text-sm text-destructive">
          {error}
        </div>
      ) : null}

      <div className="grid gap-6 lg:grid-cols-[1.2fr,1fr]">
        <Card className="shadow-soft">
          <CardHeader className="flex-row items-start justify-between gap-3 space-y-0">
            <div className="flex items-center gap-3">
              <KeyRound className="h-5 w-5 text-primary" />
              <div>
                <CardTitle className="text-sm font-semibold">
                  Secret references
                </CardTitle>
                <p className="text-sm text-muted-foreground">
                  Friendly labels, created dates, and kinds. No secret bytes
                  ever leave Rust.
                </p>
              </div>
            </div>
            <Button
              variant="ghost"
              size="sm"
              onClick={() => void refresh()}
              disabled={loading}
              className="gap-2"
            >
              <RefreshCw className={cn("h-4 w-4", loading && "animate-spin")} />
              Refresh
            </Button>
          </CardHeader>

          <CardContent className="space-y-3">
            {loading ? (
              <div className="rounded-lg border border-dashed border-muted-foreground/30 bg-muted/60 p-4 text-sm text-muted-foreground">
                Loading secrets…
              </div>
            ) : null}
            {!loading && !hasSecrets ? (
              <div className="rounded-lg border border-dashed border-muted-foreground/30 bg-muted/60 p-5 text-sm text-muted-foreground">
                No secret references yet. Secrets are managed automatically when creating DNS providers or issuers.
              </div>
            ) : null}
            {secrets.map((secret) => (
              <div
                key={secret.id}
                className="rounded-lg border bg-background/80 p-4 shadow-sm"
              >
                <div className="flex flex-wrap items-start justify-between gap-3">
                  <div>
                    <div className="flex items-center gap-2 text-sm font-semibold">
                      {secret.label || "Untitled"}
                      <span className="text-xs font-medium text-muted-foreground">
                        {secret.id}
                      </span>
                    </div>
                    <div className="mt-1 text-sm text-muted-foreground">
                      {formatKind(secret.kind)} · Created {formatDate(secret.created_at)}
                    </div>
                  </div>
                </div>
              </div>
            ))}
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
