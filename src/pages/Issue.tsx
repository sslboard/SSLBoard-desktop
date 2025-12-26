import { useState } from "react";
import { AlertTriangle, ShieldCheck } from "lucide-react";
import { Link } from "react-router-dom";
import { Button } from "../components/ui/button";
import { PageHeader } from "../components/page-header";
import { useIssuerOptions } from "../hooks/useIssuerOptions";
import { useProviderPreview } from "../hooks/useProviderPreview";
import { useManagedIssuanceFlow } from "../hooks/useManagedIssuanceFlow";
import { IssuerSelectionCard } from "../components/issue/IssuerSelectionCard";
import { DomainsInputCard } from "../components/issue/DomainsInputCard";
import { DnsInstructionsPanel } from "../components/issue/DnsInstructionsPanel";
import { IssuanceResultBanner } from "../components/issue/IssuanceResultBanner";
import type { IssuanceKeyOption } from "../lib/issuance";

export function IssuePage() {
  const [domainsInput, setDomainsInput] = useState("test.ezs3.net");
  const [keyOption, setKeyOption] = useState<IssuanceKeyOption>("rsa-2048");
  const {
    issuers,
    issuerLoading,
    issuerError,
    selectedIssuer,
    selectIssuerById,
  } = useIssuerOptions();

  const parsedDomains = domainsInput
    .split(/[\s,]+/)
    .map((d) => d.trim().toLowerCase())
    .filter(Boolean);

  const { providerPreview, providerLoading, providerError } =
    useProviderPreview(parsedDomains);

  const {
    startResult,
    statusMap,
    loadingStart,
    checking,
    finalizing,
    error,
    successMessage,
    allFound,
    manualRecords,
    hasManual,
    hasManaged,
    dnsModeLabel,
    handleStart,
    checkAll,
    finalizeIssuance,
    reset,
  } = useManagedIssuanceFlow(selectedIssuer?.issuer_id ?? null, parsedDomains, keyOption);

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

  function handleSelectIssuer(issuerId: string) {
    selectIssuerById(issuerId);
  }

  function handleReset() {
    setDomainsInput("test.ezs3.net");
    setKeyOption("rsa-2048");
    reset();
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

      {selectedIssuer && issuerEnvironment === "staging" ? (
        <div className="flex items-start gap-3 rounded-xl border border-amber-300 bg-amber-50 px-4 py-3 text-sm text-amber-900 shadow-soft">
          <AlertTriangle className="mt-0.5 h-4 w-4" />
          <div>
            <div className="font-semibold">Sandbox issuer active</div>
            <p className="text-[13px] text-amber-900/80">
              Using Let's Encrypt staging. Safe for end-to-end testing without issuing real certificates.
            </p>
          </div>
        </div>
      ) : null}

      <IssuerSelectionCard
        issuers={issuers}
        selectedIssuer={selectedIssuer}
        issuerLoading={issuerLoading}
        issuerError={issuerError}
        issuerReady={issuerReady}
        onSelectIssuer={handleSelectIssuer}
      />

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
      />

      <IssuanceResultBanner error={error} successMessage={successMessage} />

      {startResult && (
        <DnsInstructionsPanel
          statusMap={statusMap}
          hasManual={hasManual}
          hasManaged={hasManaged}
          dnsModeLabel={dnsModeLabel}
          manualRecords={manualRecords}
          checking={checking}
          allFound={allFound}
          finalizing={finalizing}
          onCheckAll={checkAll}
          onFinalize={finalizeIssuance}
        />
      )}
    </div>
  );
}
