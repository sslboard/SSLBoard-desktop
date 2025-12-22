import { PageHeader } from "../../components/page-header";
import { DnsProviderForm } from "../../components/dns-providers/DnsProviderForm";
import { DnsProviderList } from "../../components/dns-providers/DnsProviderList";
import { useDnsProviderManager } from "../../hooks/useDnsProviderManager";

export function DnsProvidersPage() {
  const {
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
    tokenTestResult,
    tokenTestLoading,
    handleTokenTest,
    clearTokenTestResult,
  } = useDnsProviderManager();

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
        <DnsProviderList
          providers={providers}
          loading={loading}
          onRefresh={() => void refreshProviders(true)}
          onEdit={startEdit}
          onDelete={(providerId) => setConfirmDeleteId(providerId)}
          confirmDeleteId={confirmDeleteId}
          onConfirmDelete={(providerId) => void handleDelete(providerId)}
          onCancelDelete={() => setConfirmDeleteId(null)}
          testResults={testResults}
          testLoading={testLoading}
          onTest={(providerId) => void handleTest(providerId)}
        />
        <DnsProviderForm
          formState={formState}
          setFormState={setFormState}
          formMode={formMode}
          saving={saving}
          onSubmit={handleSubmit}
          onCancel={resetForm}
          tokenTestResult={tokenTestResult}
          tokenTestLoading={tokenTestLoading}
          onTestToken={() => void handleTokenTest()}
          onTokenInputChange={clearTokenTestResult}
        />
      </div>
    </div>
  );
}
