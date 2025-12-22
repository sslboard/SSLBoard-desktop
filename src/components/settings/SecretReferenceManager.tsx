import { useMemo, useState, type FormEvent } from "react";
import { KeyRound, Plus, RefreshCw, Shield, Trash2 } from "lucide-react";
import { Button } from "../ui/button";
import { cn } from "../../lib/utils";
import {
  type CreateSecretRequest,
  type SecretKind,
  type UpdateSecretRequest,
} from "../../lib/secrets";
import { useSecretReferences } from "../../hooks/useSecretReferences";

export function SecretReferenceManager() {
  const {
    secrets,
    loading,
    saving,
    rotating,
    error,
    refresh,
    createSecret,
    removeSecret,
    rotateSecret,
  } = useSecretReferences();
  const [formState, setFormState] = useState<CreateSecretRequest>({
    label: "",
    kind: "acme_account_key",
    secret_value: "",
  });
  const [rotateTarget, setRotateTarget] = useState<string | null>(null);
  const [rotateValue, setRotateValue] = useState("");
  const [rotateLabel, setRotateLabel] = useState("");

  const hasSecrets = useMemo(() => secrets.length > 0, [secrets]);

  function resetForm() {
    setFormState({ label: "", kind: "acme_account_key", secret_value: "" });
  }

  async function handleCreate(e: FormEvent<HTMLFormElement>) {
    e.preventDefault();
    const created = await createSecret(formState);
    if (created) {
      resetForm();
      setRotateTarget(null);
    }
    setFormState((prev) => ({ ...prev, secret_value: "" }));
  }

  async function handleDelete(id: string) {
    await removeSecret(id);
  }

  async function handleRotate(e: FormEvent<HTMLFormElement>) {
    e.preventDefault();
    if (!rotateTarget) return;
    const payload: UpdateSecretRequest = {
      id: rotateTarget,
      secret_value: rotateValue,
      label: rotateLabel || undefined,
    };
    const updated = await rotateSecret(payload);
    if (updated) {
      setRotateValue("");
      setRotateLabel("");
      setRotateTarget(null);
    }
    setRotateValue("");
  }

  function formatKind(kind: SecretKind) {
    switch (kind) {
      case "dns_provider_token":
        return "DNS provider token";
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
        <div className="rounded-xl border bg-card p-5 shadow-soft">
          <div className="flex items-center justify-between gap-3">
            <div className="flex items-center gap-3">
              <KeyRound className="h-5 w-5 text-primary" />
              <div>
                <div className="font-semibold">Secret references</div>
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
          </div>

          <div className="mt-4 space-y-3">
            {loading ? (
              <div className="rounded-lg border border-dashed border-muted-foreground/30 bg-muted/60 p-4 text-sm text-muted-foreground">
                Loading secrets…
              </div>
            ) : null}
            {!loading && !hasSecrets ? (
              <div className="rounded-lg border border-dashed border-muted-foreground/30 bg-muted/60 p-5 text-sm text-muted-foreground">
                No secret references yet. Add an ACME account or managed key
                to begin.
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
                  <div className="flex gap-2">
                    <Button
                      variant="outline"
                      size="sm"
                      className="gap-1"
                      onClick={() => {
                        setRotateTarget(secret.id);
                        setRotateValue("");
                        setRotateLabel(secret.label);
                      }}
                    >
                      <RefreshCw className="h-4 w-4" />
                      Replace
                    </Button>
                    <Button
                      variant="ghost"
                      size="sm"
                      className="gap-1 text-destructive hover:bg-destructive/10"
                      onClick={() => void handleDelete(secret.id)}
                    >
                      <Trash2 className="h-4 w-4" />
                      Remove
                    </Button>
                  </div>
                </div>

                {rotateTarget === secret.id ? (
                  <form className="mt-3 space-y-3" onSubmit={handleRotate}>
                    <div className="grid gap-3 sm:grid-cols-[2fr,1fr]">
                      <div>
                        <label className="text-sm font-medium text-foreground">
                          New secret value
                        </label>
                        <input
                          type="password"
                          autoComplete="off"
                          required
                          className="mt-2 w-full rounded-lg border bg-background/60 p-3 text-sm shadow-inner outline-none ring-offset-background focus:ring-2 focus:ring-primary/50"
                          placeholder="Paste token or key material (kept in Rust only)"
                          value={rotateValue}
                          onChange={(e) => setRotateValue(e.target.value)}
                        />
                      </div>
                      <div>
                        <label className="text-sm font-medium text-foreground">
                          Label (optional)
                        </label>
                        <input
                          className="mt-2 w-full rounded-lg border bg-background/60 p-2.5 text-sm shadow-inner outline-none ring-offset-background focus:ring-2 focus:ring-primary/50"
                          value={rotateLabel}
                          onChange={(e) => setRotateLabel(e.target.value)}
                        />
                        <p className="mt-2 text-xs text-muted-foreground">
                          Label helps identify the secret reference; the ID stays stable.
                        </p>
                      </div>
                    </div>
                    <div className="flex gap-2">
                      <Button
                        type="submit"
                        size="sm"
                        disabled={rotating}
                        className="gap-2"
                      >
                        {rotating ? (
                          <RefreshCw className="h-4 w-4 animate-spin" />
                        ) : (
                          <RefreshCw className="h-4 w-4" />
                        )}
                        Save replacement
                      </Button>
                      <Button
                        type="button"
                        variant="ghost"
                        size="sm"
                        onClick={() => setRotateTarget(null)}
                      >
                        Cancel
                      </Button>
                    </div>
                  </form>
                ) : null}
              </div>
            ))}
          </div>
        </div>

        <div className="rounded-xl border bg-card p-5 shadow-soft">
          <div className="flex items-center gap-3">
            <Shield className="h-5 w-5 text-primary" />
            <div>
              <div className="font-semibold">Add secret reference</div>
              <p className="text-sm text-muted-foreground">
                UI sends the value into Rust once. Only metadata is stored for listing.
              </p>
            </div>
          </div>

          <form className="mt-4 space-y-4" onSubmit={handleCreate}>
            <div className="space-y-2">
              <label className="text-sm font-medium text-foreground">
                Label
              </label>
              <input
                className="w-full rounded-lg border bg-background/60 p-2.5 text-sm shadow-inner outline-none ring-offset-background focus:ring-2 focus:ring-primary/50"
                placeholder="e.g., Cloudflare prod DNS"
                value={formState.label}
                onChange={(e) =>
                  setFormState((prev) => ({ ...prev, label: e.target.value }))
                }
                required
              />
            </div>

            <div className="space-y-2">
              <label className="text-sm font-medium text-foreground">
                Secret type
              </label>
              <select
                className="w-full rounded-lg border bg-background/60 p-2.5 text-sm shadow-inner outline-none ring-offset-background focus:ring-2 focus:ring-primary/50"
                value={formState.kind}
                onChange={(e) =>
                  setFormState((prev) => ({
                    ...prev,
                    kind: e.target.value as SecretKind,
                  }))
                }
              >
                <option value="acme_account_key">ACME account key</option>
                <option value="managed_private_key">Managed private key</option>
              </select>
            </div>

            <div className="space-y-2">
              <label className="text-sm font-medium text-foreground">
                Secret value
              </label>
              <input
                type="password"
                autoComplete="off"
                className="w-full rounded-lg border bg-background/60 p-3 text-sm shadow-inner outline-none ring-offset-background focus:ring-2 focus:ring-primary/50"
                placeholder="Paste token or key material. It is sent into Rust only once."
                value={formState.secret_value}
                onChange={(e) =>
                  setFormState((prev) => ({
                    ...prev,
                    secret_value: e.target.value,
                  }))
                }
                required
              />
              <p className="text-xs text-muted-foreground">
                Value is never returned to the UI. A prefixed reference ID will be created.
              </p>
            </div>

            <Button
              type="submit"
              className="w-full gap-2"
              disabled={saving || !formState.secret_value}
            >
              {saving ? (
                <RefreshCw className="h-4 w-4 animate-spin" />
              ) : (
                <Plus className="h-4 w-4" />
              )}
              Add secret reference
            </Button>
          </form>
        </div>
      </div>
    </div>
  );
}
