import { useEffect, useState, type FormEvent } from "react";
import {
  createDnsProvider,
  deleteDnsProvider,
  listDnsProviders,
  testDnsProvider,
  updateDnsProvider,
  type CreateDnsProviderRequest,
  type DnsProviderRecord,
  type DnsProviderTestResult,
} from "../lib/dns-providers";
import { maybeToastVaultUnlockError, normalizeError } from "../lib/errors";

export type ProviderFormState = CreateDnsProviderRequest & { provider_id?: string };

export function useDnsProviderManager() {
  const [providers, setProviders] = useState<DnsProviderRecord[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [formState, setFormState] = useState<ProviderFormState>({
    provider_type: "cloudflare",
    label: "",
    domain_suffixes: "",
    api_token: "",
    route53_access_key: "",
    route53_secret_key: "",
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

  async function refreshProviders(force = false) {
    if (loading && !force) return;
    setLoading(true);
    setError(null);
    try {
      const records = await listDnsProviders();
      setProviders(records);
    } catch (err) {
      const message = normalizeError(err);
      setError(message);
      maybeToastVaultUnlockError(message);
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
      route53_access_key: "",
      route53_secret_key: "",
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
      route53_access_key: "",
      route53_secret_key: "",
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
          route53_access_key: formState.route53_access_key,
          route53_secret_key: formState.route53_secret_key,
          config: formState.config ?? null,
        });
        setProviders((prev) => [created, ...prev]);
      } else if (formState.provider_id) {
        const updated = await updateDnsProvider({
          provider_id: formState.provider_id,
          label: formState.label.trim(),
          domain_suffixes: formState.domain_suffixes,
          api_token: formState.api_token || undefined,
          route53_access_key: formState.route53_access_key || undefined,
          route53_secret_key: formState.route53_secret_key || undefined,
          config: formState.config ?? null,
        });
        setProviders((prev) =>
          prev.map((entry) => (entry.id === updated.id ? updated : entry)),
        );
      }
      resetForm();
    } catch (err) {
      const message = normalizeError(err);
      setError(message);
      maybeToastVaultUnlockError(message);
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
      const message = normalizeError(err);
      setError(message);
      maybeToastVaultUnlockError(message);
    }
  }

  async function handleTest(providerId: string) {
    if (testLoading[providerId]) return;
    setTestLoading((prev) => ({ ...prev, [providerId]: true }));
    try {
      const result = await testDnsProvider(providerId);
      setTestResults((prev) => ({ ...prev, [providerId]: result }));
    } catch (err) {
      const message = normalizeError(err);
      maybeToastVaultUnlockError(message);
      setTestResults((prev) => ({
        ...prev,
        [providerId]: {
          success: false,
          elapsed_ms: 0,
          error: message,
        },
      }));
    } finally {
      setTestLoading((prev) => ({ ...prev, [providerId]: false }));
    }
  }

  return {
    providers,
    loading,
    error,
    refreshProviders,
    formState,
    setFormState,
    formMode,
    resetForm,
    startEdit,
    saving,
    handleSubmit,
    confirmDeleteId,
    setConfirmDeleteId,
    handleDelete,
    testResults,
    testLoading,
    handleTest,
  };
}
