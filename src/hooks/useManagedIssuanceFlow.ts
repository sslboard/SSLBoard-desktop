import { useRef, useState, useCallback } from "react";
import {
  completeManagedIssuance,
  keyOptionToParams,
  startManagedIssuance,
  type IssuanceKeyOption,
  type StartIssuanceResponse,
} from "../lib/issuance";
import { normalizeError } from "../lib/errors";
import type { CertificateRecord } from "../lib/certificates";
import { useIssuanceEvents, type IssuanceEventState } from "./useIssuanceEvents";


export function useManagedIssuanceFlow(
  selectedIssuerId: string | null,
  parsedDomains: string[],
  keyOption: IssuanceKeyOption,
  reuseKeyRef: string | null,
  renewingCertId: string | null,
) {
  const [startResult, setStartResult] = useState<StartIssuanceResponse | null>(null);
  const [loadingStart, setLoadingStart] = useState(false);
  const [finalizing, setFinalizing] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [certificate, setCertificate] = useState<CertificateRecord | null>(null);
  const [awaitingManual, setAwaitingManual] = useState(false);
  const [finalizeFailed, setFinalizeFailed] = useState(false);
  const [eventState, setEventState] = useState<IssuanceEventState | null>(null);
  const flowTokenRef = useRef(0);
  
  // Listen to issuance events - only active when this flow is in progress
  const handleEvent = useCallback((state: IssuanceEventState) => {
    setEventState(state);
    if (state.error) {
      setError(state.error);
      if (state.step === "dns-verification" || state.step === "dns-complete") {
        setFinalizeFailed(true);
      }
    }
  }, []);
  
  // Only listen when this flow is active (loading or has a result)
  const isActive = loadingStart || startResult !== null || finalizing;
  useIssuanceEvents(handleEvent, isActive);

  function nextFlowToken() {
    flowTokenRef.current += 1;
    return flowTokenRef.current;
  }

  function isStale(token: number) {
    return flowTokenRef.current !== token;
  }

  async function finalizeIssuance(result: StartIssuanceResponse, token: number) {
    setFinalizing(true);
    setFinalizeFailed(false);
    setError(null);
    try {
      const record = await completeManagedIssuance({
        request_id: result.request_id,
      });
      if (isStale(token)) {
        return;
      }
      setCertificate(record);
    } catch (err) {
      if (isStale(token)) {
        return;
      }
      setFinalizeFailed(true);
      setError(normalizeError(err));
    } finally {
      if (!isStale(token)) {
        setFinalizing(false);
      }
    }
  }

  async function handleStart() {
    const token = nextFlowToken();
    setLoadingStart(true);
    setError(null);
    setCertificate(null);
    setAwaitingManual(false);
    setFinalizeFailed(false);
    try {
      if (!selectedIssuerId) {
        throw new Error("Select an issuer before starting issuance.");
      }
      const keyParams = keyOptionToParams(keyOption);
      const result = await startManagedIssuance({
        domains: parsedDomains,
        issuer_id: selectedIssuerId,
        ...keyParams,
        reuse_key_ref: reuseKeyRef,
        renewing_cert_id: renewingCertId,
      });
      if (isStale(token)) {
        return;
      }
      setStartResult(result);
      const hasManualRecords = result.dns_records.some((rec) => rec.adapter === "manual");
      setAwaitingManual(hasManualRecords);
      if (!hasManualRecords) {
        // For managed records, the backend handles DNS propagation checking
        await finalizeIssuance(result, token);
      }
    } catch (err) {
      if (isStale(token)) {
        return;
      }
      setError(normalizeError(err));
      setStartResult(null);
    } finally {
      if (!isStale(token)) {
        setLoadingStart(false);
      }
    }
  }

  async function continueIssuance() {
    if (!startResult) return;
    const token = nextFlowToken();
    setAwaitingManual(false);
    await finalizeIssuance(startResult, token);
  }

  async function retryFinalization() {
    if (!startResult) return;
    const token = nextFlowToken();
    await finalizeIssuance(startResult, token);
  }

  function reset() {
    nextFlowToken();
    setStartResult(null);
    setError(null);
    setCertificate(null);
    setAwaitingManual(false);
    setFinalizeFailed(false);
    setLoadingStart(false);
    setFinalizing(false);
  }

  const manualRecords = startResult?.dns_records.filter((rec) => rec.adapter === "manual") ?? [];
  const managedRecords = startResult?.dns_records.filter((rec) => rec.adapter !== "manual") ?? [];
  const hasManual = manualRecords.length > 0;
  const hasManaged = managedRecords.length > 0;
  const dnsModeLabel = hasManual && hasManaged
    ? "mixed"
    : hasManual
      ? "manual"
      : startResult?.dns_records[0]?.adapter ?? "manual";

  return {
    startResult,
    loadingStart,
    finalizing,
    error,
    certificate,
    manualRecords,
    managedRecords,
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
  };
}
