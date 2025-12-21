import { useEffect, useMemo, useState, type FormEvent } from "react";
import {
  AlertTriangle,
  CheckCircle2,
  Plus,
  RefreshCw,
  Trash2,
  XCircle,
} from "lucide-react";
import { PageHeader } from "../../components/page-header";
import { Button } from "../../components/ui/button";
import {
  createDnsProvider,
  deleteDnsProvider,
  listDnsProviders,
  testDnsProvider,
  updateDnsProvider,
  type CreateDnsProviderRequest,
  type DnsProviderRecord,
  type DnsProviderTestResult,
  type DnsProviderType,
} from "../../lib/dns-providers";
import { cn } from "../../lib/utils";

type ProviderFormState = CreateDnsProviderRequest & { provider_id?: string };

const PROVIDER_LABELS: Record<DnsProviderType, string> = {
  cloudflare: "Cloudflare",
  digitalocean: "DigitalOcean",
  route53: "Route 53",
  manual: "Manual",
};

export function DnsProvidersPage() {
  const [providers, setProviders] = useState<DnsProviderRecord[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [formState, setFormState] = useState<ProviderFormState>({
    provider_type: "cloudflare",
    label: "",
    domain_suffixes: "",
    api_token: "",
    config: null,
  });
  const [formMode, setFormMode] = useState<"create" | "edit">("create");
  const [saving, setSaving] = useState(false);
  const [testResults, setTestResults] = useState<Record<string, DnsProviderTestResult | null>>(
    {},
  );
  const [testLoading, setTestLoading] = useState<Record<string, boolean>>({});
  const [confirmDeleteId, setConfirmDeleteId] = useState<string | null>(null);

  useEffect(() => {
    void refreshProviders();
  }, []);

  const overlapSuffixes = useMemo(() => {
    const counts = new Map<string, number>();
    providers.forEach((provider) => {
      provider.domain_suffixes.forEach((suffix) => {
        counts.set(suffix, (counts.get(suffix) ?? 0) + 1);
      });
    });
    return new Set(
      Array.from(counts.entries())
        .filter(([, count]) => count > 1)
        .map(([suffix]) => suffix),
    );
  }, [providers]);

  async function refreshProviders(force = false) {
    if (loading && !force) return;
    setLoading(true);
    setError(null);
    try {
      const records = await listDnsProviders();
      setProviders(records);
    } catch (err) {
      setError(normalizeError(err));
    } finally {
      setLoading(false);
    }
  }

  function resetForm() {
    setFormMode("create");
    setConfirmDeleteId(null);
    setFormState({
      provider_type: "cloudflare",
      label: "",
      domain_suffixes: "",
      api_token: "",
      config: null,
    });
  }

  function startEdit(provider: DnsProviderRecord) {
    setFormMode("edit");
    setConfirmDeleteId(null);
    setFormState({
      provider_id: provider.id,
      provider_type: provider.provider_type,
      label: provider.label,
      domain_suffixes: provider.domain_suffixes.join(", "),
      api_token: "",
      config: provider.config ?? null,
    });
  }

  async function handleSubmit(e: FormEvent<HTMLFormElement>) {
    e.preventDefault();
    if (saving) return;
    setSaving(true);
    setError(null);
    try {
      if (formMode === "create") {
        const created = await createDnsProvider({
          provider_type: formState.provider_type,
          label: formState.label.trim(),
          domain_suffixes: formState.domain_suffixes,
          api_token: formState.api_token,
          config: formState.config ?? null,
        });
        setProviders((prev) => [created, ...prev]);
      } else if (formState.provider_id) {
        const updated = await updateDnsProvider({
          provider_id: formState.provider_id,
          label: formState.label.trim(),
          domain_suffixes: formState.domain_suffixes,
          api_token: formState.api_token || undefined,
          config: formState.config ?? null,
        });
        setProviders((prev) =>
          prev.map((entry) => (entry.id === updated.id ? updated : entry)),
        );
      }
      resetForm();
    } catch (err) {
      setError(normalizeError(err));
    } finally {
      setSaving(false);
    }
  }

  async function handleDelete(providerId: string) {
    setConfirmDeleteId(null);
    setError(null);
    try {
      await deleteDnsProvider(providerId);
      setProviders((prev) => prev.filter((entry) => entry.id !== providerId));
    } catch (err) {
      setError(normalizeError(err));
    }
  }

  async function handleTest(providerId: string) {
    if (testLoading[providerId]) return;
    setTestLoading((prev) => ({ ...prev, [providerId]: true }));
    try {
      const result = await testDnsProvider(providerId);
      setTestResults((prev) => ({ ...prev, [providerId]: result }));
    } catch (err) {
      setTestResults((prev) => ({
        ...prev,
        [providerId]: {
          success: false,
          elapsed_ms: 0,
          error: normalizeError(err),
        },
      }));
    } finally {
      setTestLoading((prev) => ({ ...prev, [providerId]: false }));
    }
  }

  function formatDate(iso: string) {
    const date = new Date(iso);
    return Number.isNaN(date.getTime())
      ? "-"
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
        title="DNS Providers"
        description="Configure automatic DNS providers and the domains they manage."
      />

      {error ? (
        <div className="rounded-lg border border-destructive/50 bg-destructive/10 px-4 py-3 text-sm text-destructive">
          {error}
        </div>
      ) : null}

      <div className="grid gap-6 lg:grid-cols-[1.2fr,1fr]">
        <div className="rounded-xl border bg-card p-5 shadow-soft">
          <div className="flex items-center justify-between gap-3">
            <div>
              <div className="text-sm font-semibold">Configured providers</div>
              <p className="text-xs text-muted-foreground">
                Providers are matched by domain suffix; longest suffix wins.
              </p>
            </div>
            <Button
              variant="ghost"
              size="sm"
              onClick={() => void refreshProviders(true)}
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
                Loading DNS providers...
              </div>
            ) : null}
            {!loading && providers.length === 0 ? (
              <div className="rounded-lg border border-dashed border-muted-foreground/30 bg-muted/60 p-5 text-sm text-muted-foreground">
                No DNS providers configured yet.
              </div>
            ) : null}
            {providers.map((provider) => {
              const overlaps = provider.domain_suffixes.filter((suffix) =>
                overlapSuffixes.has(suffix),
              );
              const testResult = testResults[provider.id];
              const testInFlight = testLoading[provider.id] ?? false;
              return (
                <div
                  key={provider.id}
                  className="rounded-lg border bg-background/80 p-4 shadow-sm"
                >
                  <div className="flex flex-wrap items-start justify-between gap-3">
                    <div>
                      <div className="flex flex-wrap items-center gap-2 text-sm font-semibold">
                        {provider.label}
                        <span className="rounded-full bg-primary/10 px-2 py-0.5 text-[11px] font-semibold uppercase text-primary">
                          {PROVIDER_LABELS[provider.provider_type]}
                        </span>
                      </div>
                      <div className="mt-1 text-xs text-muted-foreground">
                        Domains: {provider.domain_suffixes.join(", ") || "-"}
                      </div>
                      <div className="mt-1 text-xs text-muted-foreground">
                        Updated {formatDate(provider.updated_at)}
                      </div>
                      {overlaps.length ? (
                        <div className="mt-2 flex items-center gap-2 text-xs text-amber-700">
                          <AlertTriangle className="h-3.5 w-3.5" />
                          Overlapping suffixes: {overlaps.join(", ")}
                        </div>
                      ) : null}
                      {testResult ? (
                        <div className="mt-2 flex items-center gap-2 text-xs">
                          {testResult.success ? (
                            <CheckCircle2 className="h-3.5 w-3.5 text-emerald-600" />
                          ) : (
                            <XCircle className="h-3.5 w-3.5 text-rose-600" />
                          )}
                          <span
                            className={
                              testResult.success
                                ? "text-emerald-700"
                                : "text-rose-700"
                            }
                          >
                            {testResult.success
                              ? "Connection verified"
                              : testResult.error || "Connection failed"}
                          </span>
                        </div>
                      ) : null}
                    </div>
                    <div className="flex flex-wrap gap-2">
                      <Button
                        type="button"
                        variant="outline"
                        size="sm"
                        onClick={() => handleTest(provider.id)}
                        disabled={testInFlight}
                      >
                        {testInFlight ? (
                          <RefreshCw className="mr-2 h-4 w-4 animate-spin" />
                        ) : null}
                        Test connection
                      </Button>
                      <Button
                        type="button"
                        variant="ghost"
                        size="sm"
                        onClick={() => startEdit(provider)}
                      >
                        Edit
                      </Button>
                      <Button
                        type="button"
                        variant="ghost"
                        size="sm"
                        className="text-destructive hover:bg-destructive/10"
                        onClick={() => setConfirmDeleteId(provider.id)}
                      >
                        <Trash2 className="mr-1 h-4 w-4" />
                        Remove
                      </Button>
                    </div>
                  </div>
                  {confirmDeleteId === provider.id ? (
                    <div className="mt-3 flex flex-wrap items-center gap-2 rounded-md border border-destructive/30 bg-destructive/5 px-3 py-2 text-xs text-destructive">
                      <span className="font-semibold">
                        Remove this provider and its stored token?
                      </span>
                      <Button
                        type="button"
                        variant="outline"
                        size="sm"
                        className="border-destructive/40 text-destructive hover:bg-destructive/10"
                        onClick={() => void handleDelete(provider.id)}
                      >
                        Confirm remove
                      </Button>
                      <Button
                        type="button"
                        variant="ghost"
                        size="sm"
                        onClick={() => setConfirmDeleteId(null)}
                      >
                        Cancel
                      </Button>
                    </div>
                  ) : null}
                </div>
              );
            })}
          </div>
        </div>

        <div className="rounded-xl border bg-card p-5 shadow-soft">
          <div className="flex items-center justify-between gap-3">
            <div>
              <div className="text-sm font-semibold">
                {formMode === "edit" ? "Edit provider" : "Add provider"}
              </div>
              <p className="text-xs text-muted-foreground">
                Map providers to domain suffixes (comma or space separated).
              </p>
            </div>
            {formMode === "edit" ? (
              <Button variant="ghost" size="sm" onClick={resetForm}>
                Cancel edit
              </Button>
            ) : null}
          </div>

          <form className="mt-4 space-y-4" onSubmit={handleSubmit}>
            <div className="space-y-2">
              <label className="text-sm font-medium text-foreground">
                Provider type
              </label>
              <select
                className="w-full rounded-lg border bg-background/60 p-2.5 text-sm shadow-inner outline-none ring-offset-background focus:ring-2 focus:ring-primary/50"
                value={formState.provider_type}
                onChange={(e) =>
                  setFormState((prev) => ({
                    ...prev,
                    provider_type: e.target.value as DnsProviderType,
                  }))
                }
                disabled={formMode === "edit"}
              >
                <option value="cloudflare">Cloudflare</option>
                <option value="digitalocean">DigitalOcean</option>
                <option value="route53">Route 53</option>
                <option value="manual">Manual</option>
              </select>
            </div>

            <div className="space-y-2">
              <label className="text-sm font-medium text-foreground">
                Provider label
              </label>
              <input
                className="w-full rounded-lg border bg-background/60 p-2.5 text-sm shadow-inner outline-none ring-offset-background focus:ring-2 focus:ring-primary/50"
                placeholder="Production DNS"
                value={formState.label}
                onChange={(e) =>
                  setFormState((prev) => ({ ...prev, label: e.target.value }))
                }
                required
              />
            </div>

            <div className="space-y-2">
              <label className="text-sm font-medium text-foreground">
                Domain suffixes
              </label>
              <textarea
                className="w-full rounded-lg border bg-background/60 p-2.5 text-sm shadow-inner outline-none ring-offset-background focus:ring-2 focus:ring-primary/50"
                rows={3}
                placeholder="sslboard.com, example.net"
                value={formState.domain_suffixes}
                onChange={(e) =>
                  setFormState((prev) => ({
                    ...prev,
                    domain_suffixes: e.target.value,
                  }))
                }
                required
              />
            </div>

            {formState.provider_type !== "manual" ? (
              <div className="space-y-2">
                <label className="text-sm font-medium text-foreground">
                  API token
                </label>
                <input
                  type="password"
                  autoComplete="off"
                  className="w-full rounded-lg border bg-background/60 p-3 text-sm shadow-inner outline-none ring-offset-background focus:ring-2 focus:ring-primary/50"
                  placeholder={
                    formMode === "edit"
                      ? "Enter a new token to rotate (optional)"
                      : "Paste provider API token"
                  }
                  value={formState.api_token || ""}
                  onChange={(e) =>
                    setFormState((prev) => ({
                      ...prev,
                      api_token: e.target.value,
                    }))
                  }
                  required={formMode === "create"}
                />
                <p className="text-xs text-muted-foreground">
                  Tokens are stored in the Rust core and never sent back to the UI.
                </p>
              </div>
            ) : null}

            <Button type="submit" className="w-full gap-2" disabled={saving}>
              {saving ? (
                <RefreshCw className="h-4 w-4 animate-spin" />
              ) : (
                <Plus className="h-4 w-4" />
              )}
              {formMode === "edit" ? "Save provider" : "Add provider"}
            </Button>
          </form>
        </div>
      </div>
    </div>
  );
}

function normalizeError(err: unknown): string {
  if (err instanceof Error) return err.message;
  if (typeof err === "string") return err;
  return "Unexpected error";
}
