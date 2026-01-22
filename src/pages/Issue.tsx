import { useMemo } from "react";
import { ShieldCheck } from "lucide-react";
import { Link } from "react-router-dom";
import { open } from "@tauri-apps/plugin-dialog";
import { Button } from "../components/ui/button";
import { PageHeader } from "../components/page-header";
import { useIssuerOptions } from "../hooks/useIssuerOptions";
import { useProviderPreview } from "../hooks/useProviderPreview";
import { useManagedIssuanceFlow } from "../hooks/useManagedIssuanceFlow";
import { useCsrIssuanceFlow } from "../hooks/useCsrIssuanceFlow";
import { useIssuePageState } from "../hooks/useIssuePageState";
import { IssuanceConfigCard } from "../components/issue/IssuanceConfigCard";
import { DomainsInputCard } from "../components/issue/DomainsInputCard";
import { CsrIssuanceCard } from "../components/issue/CsrIssuanceCard";
import { CsrGenerationCard } from "../components/issue/CsrGenerationCard";
import { IssuanceResultBanner } from "../components/issue/IssuanceResultBanner";
import { IssuanceFlowContainer } from "../components/issue/IssuanceFlowContainer";
import {
  inspectCsr,
  type GenerateCsrResponse,
} from "../lib/issuance";
import { normalizeError } from "../lib/errors";

export function IssuePage() {
  const pageState = useIssuePageState();
  const {
    issuers,
    issuerLoading,
    issuerError,
    selectedIssuer,
    selectIssuerById,
  } = useIssuerOptions();

  const parsedDomains = useMemo(() => {
    const normalizedInput = pageState.domainsInput.normalize("NFC");
    return normalizedInput
      .split(/[\s,]+/)
      .map((d: string) => d.normalize("NFC").trim().toLowerCase())
      .filter(Boolean);
  }, [pageState.domainsInput]);

  const { providerPreview, providerLoading, providerError } =
    useProviderPreview(parsedDomains);

  const {
    startResult,
    loadingStart,
    finalizing,
    error,
    certificate,
    manualRecords,
    hasManual,
    hasManaged,
    dnsModeLabel,
    awaitingManual,
    finalizeFailed,
    eventState,
    handleStart,
    continueIssuance,
    retryFinalization,
    reset,
  } = useManagedIssuanceFlow(selectedIssuer?.issuer_id ?? null, parsedDomains, pageState.keyOption);

  const {
    startResult: csrStartResult,
    loadingStart: csrLoadingStart,
    finalizing: csrFinalizing,
    error: csrFlowError,
    certificate: csrCertificate,
    manualRecords: csrManualRecords,
    hasManual: csrHasManual,
    hasManaged: csrHasManaged,
    dnsModeLabel: csrDnsModeLabel,
    awaitingManual: csrAwaitingManual,
    finalizeFailed: csrFinalizeFailed,
    eventState: csrEventState,
    handleStart: handleCsrStart,
    continueIssuance: continueCsrIssuance,
    retryFinalization: retryCsrFinalization,
    reset: resetCsrFlow,
  } = useCsrIssuanceFlow(
    selectedIssuer?.issuer_id ?? null,
    pageState.csrPath,
    pageState.csrSource,
    pageState.csrManagedKeyRef,
  );

  const issuerLabel = selectedIssuer?.label ?? "No issuer selected";
  const issuerEnvironment = selectedIssuer?.environment ?? "staging";
  const issuerDescription =
    issuerEnvironment === "production" ? "production" : "sandbox";
  const issuerReady = Boolean(
    selectedIssuer &&
    selectedIssuer.contact_email &&
    selectedIssuer.account_key_ref &&
    selectedIssuer.tos_agreed,
  );

  // Determine if each flow is active (only one can be active at a time)
  const managedFlowActive = loadingStart || startResult !== null || finalizing;
  const csrFlowActive = csrLoadingStart || csrStartResult !== null || csrFinalizing;

  // Disable one flow when the other is active
  const managedFlowDisabled = csrFlowActive;
  const csrFlowDisabled = managedFlowActive;

  const dnsFlowProps = {
    inputComponent: (
      <>
        <DomainsInputCard
          domainsInput={pageState.domainsInput}
          parsedDomains={parsedDomains}
          issuerLabel={issuerLabel}
          issuerEnvironment={issuerEnvironment}
          issuerReady={issuerReady}
          loadingStart={loadingStart}
          hasStartResult={Boolean(startResult)}
          providerPreview={providerPreview}
          providerLoading={providerLoading}
          providerError={providerError}
          keyOption={pageState.keyOption}
          onDomainsChange={pageState.setDomainsInput}
          onKeyOptionChange={pageState.setKeyOption}
          onStart={handleStart}
          onReset={handleReset}
          disabled={managedFlowDisabled}
        />
        {error && (
          <IssuanceResultBanner error={error} successMessage={null} />
        )}
      </>
    ),
    loadingStart,
    startResult,
    hasManual,
    hasManaged,
    dnsModeLabel,
    manualRecords,
    finalizing,
    awaitingManual,
    finalizeFailed,
    certificate,
    error,
    eventState,
    onContinue: continueIssuance,
    onRetryFinalize: retryFinalization,
    onIssueAnother: handleIssueAnother,
  };

  const csrFlowProps = {
    inputComponent: (
      <>
        <CsrIssuanceCard
          issuerLabel={issuerLabel}
          issuerEnvironment={issuerEnvironment}
          issuerReady={issuerReady}
          loadingStart={csrLoadingStart}
          hasStartResult={Boolean(csrStartResult)}
          csrPath={pageState.csrPath}
          csrResult={pageState.csrResult}
          csrLoading={pageState.csrLoading}
          csrError={pageState.csrError}
          onSelectCsr={handleSelectCsr}
          onClearCsr={handleClearCsr}
          onStart={handleCsrStart}
          onReset={resetCsrFlow}
          disabled={csrFlowDisabled}
        />
        {csrFlowError && (
          <IssuanceResultBanner error={csrFlowError} successMessage={null} />
        )}
      </>
    ),
    loadingStart: csrLoadingStart,
    startResult: csrStartResult,
    hasManual: csrHasManual,
    hasManaged: csrHasManaged,
    dnsModeLabel: csrDnsModeLabel,
    manualRecords: csrManualRecords,
    finalizing: csrFinalizing,
    awaitingManual: csrAwaitingManual,
    finalizeFailed: csrFinalizeFailed,
    certificate: csrCertificate,
    error: csrFlowError,
    eventState: csrEventState,
    onContinue: continueCsrIssuance,
    onRetryFinalize: retryCsrFinalization,
    onIssueAnother: handleCsrIssueAnother,
  };

  function handleSelectIssuer(issuerId: string) {
    selectIssuerById(issuerId);
  }

  function handleReset() {
    pageState.resetDomainsState();
    reset();
  }

  function handleIssueAnother() {
    pageState.resetDomainsState();
    reset();
  }

  function handleCsrIssueAnother() {
    pageState.resetCsrState();
    resetCsrFlow();
  }

  async function handleSelectCsr() {
    const selection = await open({
      multiple: false,
      filters: [{ name: "CSR", extensions: ["csr", "pem"] }],
    });
    if (typeof selection !== "string") {
      return;
    }
    pageState.setCsrLoading(true);
    pageState.setCsrError(null);
    pageState.setCsrPath(selection);
    pageState.setCsrSource("imported");
    pageState.setCsrManagedKeyRef(null);
    try {
      const result = await inspectCsr({ csr_path: selection });
      pageState.setCsrResult(result);
    } catch (err) {
      pageState.setCsrResult(null);
      pageState.setCsrError(normalizeError(err));
    } finally {
      pageState.setCsrLoading(false);
    }
  }

  function handleClearCsr() {
    pageState.resetCsrState();
    resetCsrFlow();
  }

  function handleGeneratedCsr(result: GenerateCsrResponse) {
    pageState.setCsrPath(result.csr_path);
    pageState.setCsrManagedKeyRef(result.managed_key_ref);
    pageState.setCsrResult(result.result);
    pageState.setCsrSource("generated");
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title="Issue"
        description={`Issue a ${issuerDescription} certificate via ACME DNS-01 with automatic providers or manual fallback.`}
        action={
          <Button asChild variant="secondary">
            <Link to="/certificates">
              <ShieldCheck className="mr-2 h-4 w-4" />
              View certificates
            </Link>
          </Button>
        }
      />

      <IssuanceConfigCard
        issuers={issuers}
        selectedIssuer={selectedIssuer}
        issuerLoading={issuerLoading}
        issuerError={issuerError}
        issuerReady={issuerReady}
        onSelectIssuer={handleSelectIssuer}
        issuanceMode={pageState.issuanceMode}
        onModeChange={pageState.setIssuanceMode}
      />

      {pageState.issuanceMode === "dns" && <IssuanceFlowContainer {...dnsFlowProps} />}

      {pageState.issuanceMode === "csr-import" && <IssuanceFlowContainer {...csrFlowProps} />}

      {pageState.issuanceMode === "csr-generate" ? (
        <CsrGenerationCard onGenerated={handleGeneratedCsr} />
      ) : null}
    </div>
  );
}
