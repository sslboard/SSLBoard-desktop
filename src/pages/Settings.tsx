import { useEffect, useMemo, useState, type FormEvent } from "react";
import {
  Shield,
  Lock,
  KeyRound,
  RefreshCw,
  Trash2,
  Plus,
  Globe2,
  AlertTriangle,
} from "lucide-react";
import { PageHeader } from "../components/page-header";
import { Button } from "../components/ui/button";
import {
  type CreateSecretRequest,
  type SecretKind,
  type SecretRefRecord,
  type UpdateSecretRequest,
  createSecretRef,
  deleteSecretRef,
  listSecretRefs,
  updateSecretRef,
} from "../lib/secrets";
import {
  ensureAcmeAccount,
  listIssuers,
  selectIssuer,
  type EnsureAcmeAccountRequest,
  type IssuerConfig,
} from "../lib/issuers";
import { cn } from "../lib/utils";

type SecretFormState = CreateSecretRequest;
const DEFAULT_ISSUERS: IssuerConfig[] = [
  {
    issuer_id: "acme_le_staging",
    label: "Let's Encrypt (Staging)",
    directory_url: "https://acme-staging-v02.api.letsencrypt.org/directory",
    environment: "staging",
    contact_email: null,
    account_key_ref: null,
    is_selected: true,
    disabled: false,
  },
  {
    issuer_id: "acme_le_prod",
    label: "Let's Encrypt (Production)",
    directory_url: "https://acme-v02.api.letsencrypt.org/directory",
    environment: "production",
    contact_email: null,
    account_key_ref: null,
    is_selected: false,
    disabled: true,
  },
];

export function SettingsPage() {
  const [secrets, setSecrets] = useState<SecretRefRecord[]>([]);
  const [loading, setLoading] = useState(false);
  const [formState, setFormState] = useState<SecretFormState>({
    label: "",
    kind: "dns_credential",
    secret_value: "",
  });
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [rotateTarget, setRotateTarget] = useState<string | null>(null);
  const [rotateValue, setRotateValue] = useState("");
  const [rotateLabel, setRotateLabel] = useState("");
  const [rotating, setRotating] = useState(false);
  const [issuers, setIssuers] = useState<IssuerConfig[]>([]);
  const [issuerLoading, setIssuerLoading] = useState(false);
  const [issuerError, setIssuerError] = useState<string | null>(null);
  const [contactEmail, setContactEmail] = useState("");
  const [accountKeyMode, setAccountKeyMode] = useState<"generate" | "existing">("generate");
  const [accountKeyRef, setAccountKeyRef] = useState("");
  const [ensuringAccount, setEnsuringAccount] = useState(false);

  useEffect(() => {
    void refresh();
    void refreshIssuers();
  }, []);

  const hasSecrets = useMemo(() => secrets.length > 0, [secrets]);
  const selectedIssuer = useMemo(
    () => issuers.find((issuer) => issuer.is_selected),
    [issuers],
  );
  const acmeAccountKeys = useMemo(
    () => secrets.filter((s) => s.kind === "acme_account_key"),
    [secrets],
  );
  const sandboxSelected = selectedIssuer?.environment === "staging";

  useEffect(() => {
    if (selectedIssuer?.contact_email) {
      setContactEmail(selectedIssuer.contact_email);
    }
    if (selectedIssuer?.account_key_ref) {
      setAccountKeyRef(selectedIssuer.account_key_ref);
      setAccountKeyMode("existing");
    }
  }, [selectedIssuer]);

  async function refresh() {
    setLoading(true);
    setError(null);
    try {
      const records = await listSecretRefs();
      setSecrets(records);
    } catch (err) {
      setError(normalizeError(err));
    } finally {
      setLoading(false);
    }
  }

  async function refreshIssuers(force = false) {
    if (issuerLoading && !force) return;
    setIssuerLoading(true);
    setIssuerError(null);
    try {
      const configs = await listIssuers();
      setIssuers(configs.length === 0 ? DEFAULT_ISSUERS : configs);
    } catch (err) {
      setIssuerError(normalizeError(err));
    } finally {
      setIssuerLoading(false);
    }
  }

  function resetForm() {
    setFormState({ label: "", kind: "dns_credential", secret_value: "" });
  }

  async function handleCreate(e: FormEvent<HTMLFormElement>) {
    e.preventDefault();
    setSaving(true);
    setError(null);
    try {
      const created = await createSecretRef(formState);
      setSecrets((prev) => [created, ...prev]);
      resetForm();
      setRotateTarget(null);
    } catch (err) {
      setError(normalizeError(err));
    } finally {
      setSaving(false);
    }
  }

  async function handleDelete(id: string) {
    setError(null);
    try {
      await deleteSecretRef(id);
      setSecrets((prev) => prev.filter((s) => s.id !== id));
    } catch (err) {
      setError(normalizeError(err));
    }
  }

  async function handleRotate(e: FormEvent<HTMLFormElement>) {
    e.preventDefault();
    if (!rotateTarget) return;
    setRotating(true);
    setError(null);
    try {
      const payload: UpdateSecretRequest = {
        id: rotateTarget,
        secret_value: rotateValue,
        label: rotateLabel || undefined,
      };
      const updated = await updateSecretRef(payload);
      setSecrets((prev) =>
        prev.map((s) => (s.id === updated.id ? updated : s)),
      );
      setRotateValue("");
      setRotateLabel("");
      setRotateTarget(null);
    } catch (err) {
      setError(normalizeError(err));
    } finally {
      setRotating(false);
    }
  }

  async function handleSelectIssuer(issuerId: string) {
    setIssuerError(null);
    setIssuerLoading(true);
    try {
      const updated = await selectIssuer(issuerId);
      setIssuers((prev) =>
        prev.map((issuer) => ({
          ...issuer,
          is_selected: issuer.issuer_id === updated.issuer_id,
        })),
      );
    } catch (err) {
      setIssuerError(normalizeError(err));
    } finally {
      setIssuerLoading(false);
    }
  }

  async function handleEnsureAccount(e: FormEvent<HTMLFormElement>) {
    e.preventDefault();
    if (!selectedIssuer) {
        setIssuerError("Select an issuer before saving.");
        return;
    }
    setEnsuringAccount(true);
    setIssuerError(null);

    const payload: EnsureAcmeAccountRequest = {
      issuer_id: selectedIssuer.issuer_id,
      contact_email: contactEmail || undefined,
      generate_new_account_key: accountKeyMode === "generate",
    };

    if (accountKeyMode === "existing") {
      payload.account_key_ref = accountKeyRef || undefined;
    }

    try {
      const updated = await ensureAcmeAccount(payload);
      setIssuers((prev) =>
        prev.map((issuer) =>
          issuer.issuer_id === updated.issuer_id ? updated : issuer,
        ),
      );
      if (updated.account_key_ref) {
        setAccountKeyRef(updated.account_key_ref);
      }
      if (updated.contact_email) {
        setContactEmail(updated.contact_email);
      }
      // Fire-and-forget refresh to avoid blocking the button state on IPC latency.
      void refreshIssuers(true);
      void refresh();
    } catch (err) {
      setIssuerError(normalizeError(err));
    } finally {
      setEnsuringAccount(false);
    }
  }

  function formatKind(kind: SecretKind) {
    switch (kind) {
      case "dns_credential":
        return "DNS credential";
      case "acme_account_key":
        return "ACME account key";
      case "managed_private_key":
        return "Managed private key";
      default:
        return kind;
    }
  }

  function formatEnvironment(env?: IssuerConfig["environment"]) {
    switch (env) {
      case "production":
        return "Production";
      case "staging":
      default:
        return "Sandbox";
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
    <div className="space-y-6">
      <PageHeader
        title="Settings"
        description="Configure providers, guardrails, and secret references. Secret values are only sent into Rust once."
      />
      {error ? (
        <div className="rounded-lg border border-destructive/50 bg-destructive/10 px-4 py-3 text-sm text-destructive">
          {error}
        </div>
      ) : null}
      {issuerError ? (
        <div className="rounded-lg border border-destructive/50 bg-destructive/10 px-4 py-3 text-sm text-destructive">
          {issuerError}
        </div>
      ) : null}

      {sandboxSelected ? (
        <div className="flex items-start gap-3 rounded-xl border border-amber-300 bg-amber-50 px-4 py-3 text-sm text-amber-900 shadow-soft">
          <AlertTriangle className="mt-0.5 h-4 w-4" />
          <div>
            <div className="font-semibold">Sandbox issuer active</div>
            <p className="text-[13px] text-amber-900/80">
              Using Let&apos;s Encrypt staging. Safe for end-to-end testing without issuing real certificates.
            </p>
          </div>
        </div>
      ) : null}

      <div className="rounded-xl border bg-card p-5 shadow-soft">
        <div className="flex items-center gap-3">
          <Globe2 className="h-5 w-5 text-primary" />
          <div>
            <div className="font-semibold">Issuer</div>
            <p className="text-sm text-muted-foreground">
              Choose the issuer and ACME account for issuance. Sandbox is the default and safest path.
            </p>
          </div>
        </div>

        <form className="mt-4 space-y-4" onSubmit={handleEnsureAccount}>
          <div className="grid gap-4 md:grid-cols-2">
            <div className="space-y-2">
              <label className="text-sm font-medium text-foreground">
                Issuer endpoint
              </label>
              <select
                className="w-full rounded-lg border bg-background/60 p-2.5 text-sm shadow-inner outline-none ring-offset-background focus:ring-2 focus:ring-primary/50"
                value={selectedIssuer?.issuer_id ?? ""}
                onChange={(e) => void handleSelectIssuer(e.target.value)}
              >
                {issuers.map((issuer) => (
                  <option
                    key={issuer.issuer_id}
                    value={issuer.issuer_id}
                    disabled={issuer.disabled}
                  >
                    {issuer.label}
                    {issuer.disabled ? " (coming soon)" : ""}
                  </option>
                ))}
              </select>
              <p className="text-xs text-muted-foreground">
                {selectedIssuer?.directory_url ?? "https://acme-staging-v02.api.letsencrypt.org/directory"}
              </p>
              <p className="text-xs text-muted-foreground">
                {formatEnvironment(selectedIssuer?.environment)} environment is highlighted above. Production is disabled for now.
              </p>
            </div>

            <div className="space-y-2">
              <label className="text-sm font-medium text-foreground">
                ACME contact email
              </label>
              <input
                className="w-full rounded-lg border bg-background/60 p-2.5 text-sm shadow-inner outline-none ring-offset-background focus:ring-2 focus:ring-primary/50"
                placeholder="you@example.com"
                value={contactEmail}
                onChange={(e) => setContactEmail(e.target.value)}
                required
              />
              <p className="text-xs text-muted-foreground">
                Used for ACME account registration and expiry notices.
              </p>
            </div>
          </div>

            <div className="rounded-lg border bg-background/60 p-4 shadow-inner">
            {/** Account key selection. Dropdown is only interactive when using an existing ref. */}
            <div className="flex items-center justify-between gap-3">
              <div>
                <div className="text-sm font-semibold text-foreground">
                  Account key
                </div>
                <p className="text-xs text-muted-foreground">
                  Stored as a secret reference; UI never sees the key bytes.
                </p>
              </div>
              {selectedIssuer?.account_key_ref ? (
                <span className="rounded-full bg-muted px-3 py-1 text-[11px] font-medium text-foreground/80">
                  {selectedIssuer.account_key_ref}
                </span>
              ) : null}
            </div>

            <div className="mt-3 space-y-2">
              <label className="flex items-center gap-2 text-sm font-medium text-foreground">
                <input
                  type="radio"
                  className="h-4 w-4"
                  checked={accountKeyMode === "generate"}
                  onChange={() => setAccountKeyMode("generate")}
                />
                Generate a new staging account key (recommended)
              </label>
              <label className="flex items-center gap-2 text-sm font-medium text-foreground">
                <input
                  type="radio"
                  className="h-4 w-4"
                checked={accountKeyMode === "existing"}
                onChange={() => setAccountKeyMode("existing")}
              />
              Use an existing secret reference
            </label>
            {/** Disable the dropdown when generating a new key to make the state explicit. */}
            <select
                className={cn(
                  "w-full rounded-lg border bg-background/60 p-2.5 text-sm shadow-inner outline-none ring-offset-background focus:ring-2 focus:ring-primary/50",
                  accountKeyMode !== "existing" && "cursor-not-allowed opacity-50"
                )}
                value={accountKeyRef}
                onChange={(e) => setAccountKeyRef(e.target.value)}
                disabled={accountKeyMode !== "existing"}
              >
                <option value="">
                  {acmeAccountKeys.length === 0
                    ? "No ACME account keys yet"
                    : "Select an ACME account key"}
                </option>
                {acmeAccountKeys.map((secret) => (
                  <option key={secret.id} value={secret.id}>
                    {secret.label || secret.id} ({secret.id})
                  </option>
                ))}
              </select>
              <p className="text-xs text-muted-foreground">
                Create a secret reference below first if you want to import your own key.
              </p>
            </div>
          </div>

          <div className="flex flex-wrap items-center gap-3">
            <Button
              type="submit"
              className="gap-2"
              disabled={ensuringAccount || issuerLoading}
            >
              {ensuringAccount ? (
                <RefreshCw className="h-4 w-4 animate-spin" />
              ) : (
                <RefreshCw className="h-4 w-4" />
              )}
              Save issuer &amp; ensure account
            </Button>
            <p className="text-sm text-muted-foreground">
              Sandbox is safe for testing; production remains locked until explicitly enabled.
            </p>
          </div>
        </form>
      </div>

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
                No secret references yet. Add a DNS credential or ACME account
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
                        <textarea
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
                          Label helps identify DNS configuration; reference ID stays stable.
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
                <option value="dns_credential">DNS credential</option>
                <option value="acme_account_key">ACME account key</option>
                <option value="managed_private_key">Managed private key</option>
              </select>
            </div>

            <div className="space-y-2">
              <label className="text-sm font-medium text-foreground">
                Secret value
              </label>
              <textarea
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

function normalizeError(err: unknown): string {
  if (err instanceof Error) return err.message;
  if (typeof err === "string") return err;
  return "An unexpected error occurred.";
}
