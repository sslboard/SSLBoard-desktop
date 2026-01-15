import { Card } from "../ui/card";
import { IssuanceProgressIndicator } from "./IssuanceProgressIndicator";
import { DnsInstructionsPanel } from "./DnsInstructionsPanel";
import { CompletedCertificateCard } from "./CompletedCertificateCard";
import type { StartIssuanceResponse, StartCsrIssuanceResponse } from "../../lib/issuance";
import type { CertificateRecord } from "../../lib/certificates";
import type { IssuanceEventState } from "../../hooks/useIssuanceEvents";

type IssuanceResponse = StartIssuanceResponse | StartCsrIssuanceResponse;

interface IssuanceFlowContainerProps {
  // Input phase
  showInput: boolean;
  inputComponent: React.ReactNode;

  // Progress phase
  loadingStart: boolean;
  startResult: IssuanceResponse | null;
  hasManual: boolean;
  hasManaged: boolean;
  dnsModeLabel: string;
  manualRecords: StartIssuanceResponse["dns_records"];
  finalizing: boolean;
  awaitingManual: boolean;
  finalizeFailed: boolean;
  eventState: IssuanceEventState | null;

  // Result phase
  certificate: CertificateRecord | null;
  error: string | null;

  // Actions
  onContinue: () => void;
  onRetryFinalize: () => void;
  onIssueAnother: () => void;
}

export function IssuanceFlowContainer({
  showInput,
  inputComponent,
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
  onContinue,
  onRetryFinalize,
  onIssueAnother,
}: IssuanceFlowContainerProps) {

  // Determine progress steps based on event state
  const getProgressSteps = () => {
    // Start step
    let startStatus: "complete" | "active" | "pending";
    if (certificate) {
      startStatus = "complete";
    } else if (eventState && (eventState.step === "starting" || eventState.step === "challenges-received" || eventState.step === "dns-verification" || eventState.step === "dns-complete" || eventState.step === "finalization" || eventState.step === "complete")) {
      startStatus = "complete";
    } else if (startResult) {
      // Issuance has started - Start step is complete
      startStatus = "complete";
    } else if (loadingStart) {
      startStatus = "active";
    } else {
      startStatus = "pending";
    }

    // If we don't have a startResult yet and not loading, show all pending
    if (!startResult && !loadingStart && !eventState) {
      return [
        { label: "Start", status: "pending" as const },
        { label: "DNS Verification", status: "pending" as const },
        { label: "Finalize", status: "pending" as const },
      ];
    }

    // DNS Verification status - use event state if available
    let dnsStatus: "complete" | "active" | "pending" | "error";
    if (certificate) {
      dnsStatus = "complete";
    } else if (eventState) {
      // Use event state to determine DNS status
      if (eventState.step === "dns-complete" || eventState.step === "finalization" || eventState.step === "complete") {
        dnsStatus = eventState.error ? "error" : "complete";
      } else if (eventState.step === "dns-verification") {
        dnsStatus = "active";
      } else if (eventState.step === "challenges-received") {
        // Challenges received, DNS verification about to start or in progress
        // For managed DNS, verification starts immediately, so show as active
        // For manual DNS, user needs to add records, so also show as active
        dnsStatus = "active";
      } else if (eventState.step === "starting") {
        // Starting issuance, DNS not yet active
        dnsStatus = "pending";
      } else {
        dnsStatus = "pending";
      }
    } else if (finalizing) {
      // Fallback: if finalizing, DNS should be complete
      dnsStatus = "complete";
    } else if (startResult) {
      // Fallback: if we have startResult, DNS verification is in progress
      // Show as active since verification should be happening
      dnsStatus = "active";
    } else {
      dnsStatus = "pending";
    }

    // Finalize status - use event state if available
    let finalizeStatus: "complete" | "active" | "pending" | "error";
    if (certificate) {
      finalizeStatus = "complete";
    } else if (eventState) {
      // Use event state to determine finalize status
      if (eventState.step === "complete") {
        finalizeStatus = eventState.error ? "error" : "complete";
      } else if (eventState.step === "finalization") {
        finalizeStatus = eventState.error ? "error" : "active";
      } else if (eventState.step === "dns-complete") {
        // DNS complete, finalization about to start
        finalizeStatus = "pending";
      } else {
        finalizeStatus = "pending";
      }
    } else if (finalizeFailed) {
      finalizeStatus = "error";
    } else if (finalizing) {
      finalizeStatus = "active";
    } else {
      finalizeStatus = "pending";
    }

    return [
      { label: "Start", status: startStatus },
      { label: "DNS Verification", status: dnsStatus },
      { label: "Finalize", status: finalizeStatus },
    ];
  };

  // Show certificate result
  if (certificate) {
    return (
      <CompletedCertificateCard
        certificate={certificate}
        onIssueAnother={onIssueAnother}
      />
    );
  }

  // Show progress view when loading or when we have a startResult
  if (loadingStart || startResult) {
    return (
      <div className="space-y-6">
        <Card className="p-6 shadow-soft">
          <div className="mb-6">
            <div className="mb-2 text-sm font-semibold text-muted-foreground">
              Issuance in progress
            </div>
            <div className="text-lg font-bold">Certificate issuance</div>
          </div>
          <IssuanceProgressIndicator steps={getProgressSteps()} />
        </Card>

        {error && (
          <div className="rounded-lg border border-rose-200 bg-rose-50 px-4 py-3 text-sm text-rose-700">
            {error}
          </div>
        )}

        <DnsInstructionsPanel
          hasManual={hasManual}
          hasManaged={hasManaged}
          dnsModeLabel={dnsModeLabel}
          manualRecords={manualRecords}
          finalizing={finalizing}
          awaitingManual={awaitingManual}
          finalizeFailed={finalizeFailed}
          hasCertificate={false}
          onContinue={onContinue}
          onRetryFinalize={onRetryFinalize}
        />
      </div>
    );
  }

  // Show input view
  return <>{inputComponent}</>;
}
