import { useState } from "react";
import { ShieldCheck } from "lucide-react";
import { Link } from "react-router-dom";
import { open } from "@tauri-apps/plugin-dialog";
import { Button } from "../components/ui/button";
import { PageHeader } from "../components/page-header";
import { useIssuerOptions } from "../hooks/useIssuerOptions";
import { useProviderPreview } from "../hooks/useProviderPreview";
import { useManagedIssuanceFlow } from "../hooks/useManagedIssuanceFlow";
import { useCsrIssuanceFlow } from "../hooks/useCsrIssuanceFlow";
import { IssuanceConfigCard } from "../components/issue/IssuanceConfigCard";
import { DomainsInputCard } from "../components/issue/DomainsInputCard";
import { CsrIssuanceCard } from "../components/issue/CsrIssuanceCard";
import { CsrGenerationCard } from "../components/issue/CsrGenerationCard";
import type { IssuanceMode } from "../components/issue/IssuanceModeCard";
import { IssuanceResultBanner } from "../components/issue/IssuanceResultBanner";
import { IssuanceFlowContainer } from "../components/issue/IssuanceFlowContainer";
import {
  inspectCsr,
  type GenerateCsrResponse,
  type CsrSource,
  type CsrValidationResult,
  type IssuanceKeyOption,
} from "../lib/issuance";
import { normalizeError } from "../lib/errors";

export function IssuePage() {
  const [issuanceMode, setIssuanceMode] = useState<IssuanceMode>("dns");
  const [domainsInput, setDomainsInput] = useState("test.ezs3.net");
  const [keyOption, setKeyOption] = useState<IssuanceKeyOption>("rsa-2048");
  const [csrPath, setCsrPath] = useState<string | null>(null);
  const [csrResult, setCsrResult] = useState<CsrValidationResult | null>(null);
  const [csrLoading, setCsrLoading] = useState(false);
  const [csrError, setCsrError] = useState<string | null>(null);
  const [csrSource, setCsrSource] = useState<CsrSource>("imported");
  const [csrManagedKeyRef, setCsrManagedKeyRef] = useState<string | null>(null);
  const {
    issuers,
    issuerLoading,
    issuerError,
    selectedIssuer,
    selectIssuerById,
  } = useIssuerOptions();

  const normalizedInput = domainsInput.normalize("NFC");
  const parsedDomains = normalizedInput
    .split(/[\s,]+/)
    .map((d) => d.normalize("NFC").trim().toLowerCase())
    .filter(Boolean);

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
  } = useManagedIssuanceFlow(selectedIssuer?.issuer_id ?? null, parsedDomains, keyOption);

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
    csrPath,
    csrSource,
    csrManagedKeyRef,
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

  function handleSelectIssuer(issuerId: string) {
    selectIssuerById(issuerId);
  }

  function handleReset() {
    setDomainsInput("test.ezs3.net");
    setKeyOption("rsa-2048");
    reset();
  }

  function handleIssueAnother() {
    setDomainsInput("test.ezs3.net");
    setKeyOption("rsa-2048");
    reset();
  }

  function handleCsrIssueAnother() {
    setCsrPath(null);
    setCsrResult(null);
    setCsrError(null);
    setCsrManagedKeyRef(null);
    setCsrSource("imported");
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
    setCsrLoading(true);
    setCsrError(null);
    setCsrPath(selection);
    setCsrSource("imported");
    setCsrManagedKeyRef(null);
    try {
      const result = await inspectCsr({ csr_path: selection });
      setCsrResult(result);
    } catch (err) {
      setCsrResult(null);
      setCsrError(normalizeError(err));
    } finally {
      setCsrLoading(false);
    }
  }

  function handleClearCsr() {
    setCsrPath(null);
    setCsrResult(null);
    setCsrError(null);
    setCsrManagedKeyRef(null);
    setCsrSource("imported");
    resetCsrFlow();
  }

  function handleGeneratedCsr(result: GenerateCsrResponse) {
    setCsrPath(result.csr_path);
    setCsrManagedKeyRef(result.managed_key_ref);
    setCsrResult(result.result);
    setCsrSource("generated");
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
        issuanceMode={issuanceMode}
        onModeChange={setIssuanceMode}
      />

      {issuanceMode === "dns" ? (
        <IssuanceFlowContainer
          inputComponent={
            <>
              <DomainsInputCard
                domainsInput={domainsInput}
                parsedDomains={parsedDomains}
                issuerLabel={issuerLabel}
                issuerEnvironment={issuerEnvironment}
                issuerReady={issuerReady}
                loadingStart={loadingStart}
                hasStartResult={Boolean(startResult)}
                providerPreview={providerPreview}
                providerLoading={providerLoading}
                providerError={providerError}
                keyOption={keyOption}
                onDomainsChange={setDomainsInput}
                onKeyOptionChange={setKeyOption}
                onStart={handleStart}
                onReset={handleReset}
                disabled={managedFlowDisabled}
              />
              {error && (
                <IssuanceResultBanner error={error} successMessage={null} />
              )}
            </>
          }
          loadingStart={loadingStart}
          startResult={startResult}
          hasManual={hasManual}
          hasManaged={hasManaged}
          dnsModeLabel={dnsModeLabel}
          manualRecords={manualRecords}
          finalizing={finalizing}
          awaitingManual={awaitingManual}
          finalizeFailed={finalizeFailed}
          certificate={certificate}
          error={error}
          eventState={eventState}
          onContinue={continueIssuance}
          onRetryFinalize={retryFinalization}
          onIssueAnother={handleIssueAnother}
        />
      ) : null}

      {issuanceMode === "csr-import" ? (
        <IssuanceFlowContainer
          inputComponent={
            <>
              <CsrIssuanceCard
                issuerLabel={issuerLabel}
                issuerEnvironment={issuerEnvironment}
                issuerReady={issuerReady}
                loadingStart={csrLoadingStart}
                hasStartResult={Boolean(csrStartResult)}
                csrPath={csrPath}
                csrResult={csrResult}
                csrLoading={csrLoading}
                csrError={csrError}
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
          }
          loadingStart={csrLoadingStart}
          startResult={csrStartResult}
          hasManual={csrHasManual}
          hasManaged={csrHasManaged}
          dnsModeLabel={csrDnsModeLabel}
          manualRecords={csrManualRecords}
          finalizing={csrFinalizing}
          awaitingManual={csrAwaitingManual}
          finalizeFailed={csrFinalizeFailed}
          certificate={csrCertificate}
          error={csrFlowError}
          eventState={csrEventState}
          onContinue={continueCsrIssuance}
          onRetryFinalize={retryCsrFinalization}
          onIssueAnother={handleCsrIssueAnother}
        />
      ) : null}

      {issuanceMode === "csr-generate" ? (
        <CsrGenerationCard onGenerated={handleGeneratedCsr} />
      ) : null}
    </div>
  );
}
