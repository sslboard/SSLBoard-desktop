import { useEffect, useMemo, useState, type FormEvent } from "react";
import {
  Shield,
  Lock,
  KeyRound,
  RefreshCw,
  Trash2,
  Plus,
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
  createIssuer,
  listIssuers,
  deleteIssuer,
  setIssuerDisabled,
  updateIssuer,
  type IssuerConfig,
  type IssuerEnvironment,
} from "../lib/issuers";
import { cn } from "../lib/utils";

type SecretFormState = CreateSecretRequest;
type IssuerFormState = {
  issuer_id?: string;
  label: string;
  environment: IssuerEnvironment;
  directory_url: string;
  contact_email: string;
  tos_agreed: boolean;
};

const ACME_DIRECTORY_URLS: Record<IssuerEnvironment, string> = {
  staging: "https://acme-staging-v02.api.letsencrypt.org/directory",
  production: "https://acme-v02.api.letsencrypt.org/directory",
};

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
  const [issuerFormMode, setIssuerFormMode] = useState<"create" | "edit">("create");
  const [issuerForm, setIssuerForm] = useState<IssuerFormState>({
    label: "",
    environment: "staging",
    directory_url: ACME_DIRECTORY_URLS.staging,
    contact_email: "",
    tos_agreed: false,
  });
  const [issuerFormSaving, setIssuerFormSaving] = useState(false);
  const [issuerFormError, setIssuerFormError] = useState<string | null>(null);

  useEffect(() => {
    void refresh();
    void refreshIssuers();
  }, []);

  const hasSecrets = useMemo(() => secrets.length > 0, [secrets]);

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
      setIssuers(configs);
    } catch (err) {
      setIssuerError(normalizeError(err));
    } finally {
      setIssuerLoading(false);
    }
  }

  function resetForm() {
    setFormState({ label: "", kind: "dns_credential", secret_value: "" });
  }

  function resetIssuerForm() {
    setIssuerFormMode("create");
    setIssuerForm({
      label: "",
      environment: "staging",
      directory_url: ACME_DIRECTORY_URLS.staging,
      contact_email: "",
      tos_agreed: false,
    });
    setIssuerFormError(null);
  }

  function updateIssuerEnvironment(environment: IssuerEnvironment) {
    setIssuerForm((prev) => {
      const wasDefault = prev.directory_url === ACME_DIRECTORY_URLS[prev.environment];
      return {
        ...prev,
        environment,
        directory_url: wasDefault ? ACME_DIRECTORY_URLS[environment] : prev.directory_url,
      };
    });
  }

  function validateIssuerForm(form: IssuerFormState) {
    if (!form.label.trim()) {
      return "Issuer name is required.";
    }
    if (!form.directory_url.trim()) {
      return "Directory URL is required for ACME issuers.";
    }
    if (!form.contact_email.trim()) {
      return "Contact email is required for ACME issuers.";
    }
    if (!form.tos_agreed) {
      return "You must accept the ACME Terms of Service.";
    }
    return null;
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
      setFormState((prev) => ({ ...prev, secret_value: "" }));
    }
  }

  async function handleIssuerSubmit(e: FormEvent<HTMLFormElement>) {
    e.preventDefault();
    if (issuerFormSaving) return;
    setIssuerFormError(null);
    const validation = validateIssuerForm(issuerForm);
    if (validation) {
      setIssuerFormError(validation);
      return;
    }

    setIssuerFormSaving(true);
    try {
      if (issuerFormMode === "create") {
        const created = await createIssuer({
          label: issuerForm.label.trim(),
          issuer_type: "acme",
          environment: issuerForm.environment,
          directory_url: issuerForm.directory_url.trim(),
          contact_email: issuerForm.contact_email.trim(),
          tos_agreed: issuerForm.tos_agreed,
        });
        setIssuers((prev) => [...prev, created]);
      } else if (issuerForm.issuer_id) {
        const updated = await updateIssuer({
          issuer_id: issuerForm.issuer_id,
          label: issuerForm.label.trim(),
          environment: issuerForm.environment,
          directory_url: issuerForm.directory_url.trim(),
          contact_email: issuerForm.contact_email.trim(),
          tos_agreed: issuerForm.tos_agreed,
        });
        setIssuers((prev) =>
          prev.map((issuer) =>
            issuer.issuer_id === updated.issuer_id ? updated : issuer,
          ),
        );
      }
      resetIssuerForm();
      void refreshIssuers(true);
    } catch (err) {
      setIssuerFormError(normalizeError(err));
    } finally {
      setIssuerFormSaving(false);
    }
  }

  function handleEditIssuer(issuer: IssuerConfig) {
    setIssuerFormMode("edit");
    setIssuerForm({
      issuer_id: issuer.issuer_id,
      label: issuer.label,
      environment: issuer.environment,
      directory_url: issuer.directory_url,
      contact_email: issuer.contact_email ?? "",
      tos_agreed: issuer.tos_agreed,
    });
    setIssuerFormError(null);
  }

  async function handleToggleIssuerDisabled(issuer: IssuerConfig) {
    setIssuerError(null);
    setIssuerLoading(true);
    try {
      const updated = await setIssuerDisabled({
        issuer_id: issuer.issuer_id,
        disabled: !issuer.disabled,
      });
      setIssuers((prev) =>
        prev.map((entry) =>
          entry.issuer_id === updated.issuer_id ? updated : entry,
        ),
      );
    } catch (err) {
      setIssuerError(normalizeError(err));
    } finally {
      setIssuerLoading(false);
    }
  }

  async function handleDeleteIssuer(issuer: IssuerConfig) {
    if (issuerLoading) return;
    setIssuerError(null);
    setIssuerLoading(true);
    try {
      const deletedId = await deleteIssuer({ issuer_id: issuer.issuer_id });
      setIssuers((prev) =>
        prev.filter((entry) => entry.issuer_id !== deletedId),
      );
    } catch (err) {
      setIssuerError(normalizeError(err));
    } finally {
      setIssuerLoading(false);
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
      setRotateValue("");
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

  function formatIssuerType(type?: IssuerConfig["issuer_type"]) {
    switch (type) {
      case "acme":
      default:
        return "ACME";
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

      <div className="rounded-xl border bg-card p-5 shadow-soft">
        <div className="flex items-center justify-between gap-3">
          <div>
            <div className="text-sm font-semibold">Issuer management</div>
            <p className="text-xs text-muted-foreground">
              Add, edit, or disable issuer entries. ACME issuers require contact email and ToS acceptance.
            </p>
          </div>
          {issuerFormMode === "edit" ? (
            <Button
              type="button"
              variant="ghost"
              size="sm"
              onClick={resetIssuerForm}
            >
              Cancel edit
            </Button>
          ) : null}
        </div>

        {issuerFormError ? (
          <div className="mt-3 rounded-lg border border-destructive/50 bg-destructive/10 px-3 py-2 text-xs text-destructive">
            {issuerFormError}
          </div>
        ) : null}

        <div className="mt-4 grid gap-4 lg:grid-cols-[1.2fr,1fr]">
          <div className="space-y-3">
            {issuers.map((issuer) => (
              <div
                key={issuer.issuer_id}
                className="rounded-lg border bg-background/80 p-4 shadow-sm"
              >
                <div className="flex flex-wrap items-start justify-between gap-3">
                    <div>
                      <div className="flex flex-wrap items-center gap-2 text-sm font-semibold">
                        {issuer.label}
                        {issuer.disabled ? (
                          <span className="rounded-full bg-muted px-2 py-0.5 text-[11px] text-muted-foreground">
                            Disabled
                          </span>
                      ) : null}
                    </div>
                    <div className="mt-1 text-xs text-muted-foreground">
                      {formatIssuerType(issuer.issuer_type)} · {formatEnvironment(issuer.environment)} ·{" "}
                      {issuer.tos_agreed ? "ToS accepted" : "ToS pending"}
                    </div>
                    <div className="mt-1 text-xs text-muted-foreground">
                      {issuer.directory_url}
                    </div>
                  </div>
                  <div className="flex flex-wrap gap-2">
                    <Button
                      type="button"
                      variant="outline"
                      size="sm"
                      onClick={() => handleEditIssuer(issuer)}
                      disabled={issuerLoading}
                    >
                      Edit
                    </Button>
                    <Button
                      type="button"
                      variant="ghost"
                      size="sm"
                      className={issuer.disabled ? "" : "text-destructive hover:bg-destructive/10"}
                      onClick={() => void handleToggleIssuerDisabled(issuer)}
                      disabled={issuerLoading}
                    >
                      {issuer.disabled ? "Enable" : "Disable"}
                    </Button>
                    <Button
                      type="button"
                      variant="ghost"
                      size="sm"
                      className="text-destructive hover:bg-destructive/10"
                      onClick={() => void handleDeleteIssuer(issuer)}
                      disabled={issuerLoading}
                    >
                      Remove
                    </Button>
                  </div>
                </div>
              </div>
            ))}
          </div>

          <form className="space-y-3" onSubmit={handleIssuerSubmit}>
            <div className="space-y-1">
              <label className="text-sm font-medium text-foreground">
                Issuer name
              </label>
              <input
                className="w-full rounded-lg border bg-background/60 p-2.5 text-sm shadow-inner outline-none ring-offset-background focus:ring-2 focus:ring-primary/50"
                value={issuerForm.label}
                onChange={(e) =>
                  setIssuerForm((prev) => ({ ...prev, label: e.target.value }))
                }
                placeholder="Let's Encrypt (Custom)"
                required
              />
            </div>

            <div className="space-y-1">
              <label className="text-sm font-medium text-foreground">
                Environment
              </label>
              <select
                className="w-full rounded-lg border bg-background/60 p-2.5 text-sm shadow-inner outline-none ring-offset-background focus:ring-2 focus:ring-primary/50"
                value={issuerForm.environment}
                onChange={(e) =>
                  updateIssuerEnvironment(e.target.value as IssuerEnvironment)
                }
              >
                <option value="staging">Sandbox (staging)</option>
                <option value="production">Production</option>
              </select>
            </div>

            <div className="space-y-1">
              <label className="text-sm font-medium text-foreground">
                Directory URL
              </label>
              <input
                className="w-full rounded-lg border bg-background/60 p-2.5 text-sm shadow-inner outline-none ring-offset-background focus:ring-2 focus:ring-primary/50"
                value={issuerForm.directory_url}
                onChange={(e) =>
                  setIssuerForm((prev) => ({
                    ...prev,
                    directory_url: e.target.value,
                  }))
                }
                required
              />
            </div>

            <div className="space-y-1">
              <label className="text-sm font-medium text-foreground">
                Contact email
              </label>
              <input
                className="w-full rounded-lg border bg-background/60 p-2.5 text-sm shadow-inner outline-none ring-offset-background focus:ring-2 focus:ring-primary/50"
                value={issuerForm.contact_email}
                onChange={(e) =>
                  setIssuerForm((prev) => ({
                    ...prev,
                    contact_email: e.target.value,
                  }))
                }
                placeholder="you@example.com"
                required
              />
            </div>

            <label className="flex items-center gap-2 text-sm font-medium text-foreground">
              <input
                type="checkbox"
                className="h-4 w-4"
                checked={issuerForm.tos_agreed}
                onChange={(e) =>
                  setIssuerForm((prev) => ({
                    ...prev,
                    tos_agreed: e.target.checked,
                  }))
                }
                required
              />
              I accept the ACME Terms of Service
            </label>

            <Button
              type="submit"
              className="w-full gap-2"
              disabled={issuerFormSaving}
            >
              {issuerFormSaving ? (
                <RefreshCw className="h-4 w-4 animate-spin" />
              ) : (
                <Plus className="h-4 w-4" />
              )}
              {issuerFormMode === "edit" ? "Save issuer changes" : "Add issuer"}
            </Button>
          </form>
        </div>
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
