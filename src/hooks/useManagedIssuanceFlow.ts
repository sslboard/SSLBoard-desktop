import { useEffect, useState } from "react";
import { checkDnsPropagation, type PropagationResult } from "../lib/dns";
import {
  completeManagedIssuance,
  startManagedIssuance,
  type StartIssuanceResponse,
} from "../lib/issuance";
import { normalizeError } from "../lib/errors";

type StatusMap = Record<string, PropagationResult | null>;

export function useManagedIssuanceFlow(selectedIssuerId: string | null, parsedDomains: string[]) {
  const [startResult, setStartResult] = useState<StartIssuanceResponse | null>(null);
  const [statusMap, setStatusMap] = useState<StatusMap>({});
  const [loadingStart, setLoadingStart] = useState(false);
  const [checking, setChecking] = useState(false);
  const [finalizing, setFinalizing] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);

  useEffect(() => {
    if (!startResult) {
      setStatusMap({});
    }
  }, [startResult]);

  async function handleStart() {
    setLoadingStart(true);
    setError(null);
    setSuccessMessage(null);
    try {
      if (!selectedIssuerId) {
        throw new Error("Select an issuer before starting issuance.");
      }
      const result = await startManagedIssuance({
        domains: parsedDomains,
        issuer_id: selectedIssuerId,
      });
      setStartResult(result);
      const initialStatus: StatusMap = {};
      result.dns_records.forEach((rec) => {
        initialStatus[rec.record_name] = null;
      });
      setStatusMap(initialStatus);
    } catch (err) {
      setError(normalizeError(err));
      setStartResult(null);
    } finally {
      setLoadingStart(false);
    }
  }

  async function checkAll() {
    if (!startResult) return;
    setChecking(true);
    setError(null);
    try {
      const updates: StatusMap = {};
      for (const rec of startResult.dns_records) {
        const domain = rec.record_name.replace(/^_acme-challenge\./, "");
        const result = await checkDnsPropagation(domain, rec.value);
        updates[rec.record_name] = result;
      }
      setStatusMap((prev) => ({ ...prev, ...updates }));
    } catch (err) {
      setError(normalizeError(err));
    } finally {
      setChecking(false);
    }
  }

  async function finalizeIssuance() {
    if (!startResult) return;
    setFinalizing(true);
    setError(null);
    try {
      const record = await completeManagedIssuance({
        request_id: startResult.request_id,
      });
      setSuccessMessage(
        `Issued ${record.subjects[0]} — expires ${new Date(record.not_after).toLocaleDateString()}`,
      );
    } catch (err) {
      setError(normalizeError(err));
    } finally {
      setFinalizing(false);
    }
  }

  function reset() {
    setStartResult(null);
    setStatusMap({});
    setError(null);
    setSuccessMessage(null);
  }

  const allFound = Boolean(
    startResult &&
    Object.values(statusMap).length === startResult.dns_records.length &&
    Object.values(statusMap).every((s) => s?.state === "found"),
  );

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
    statusMap,
    loadingStart,
    checking,
    finalizing,
    error,
    successMessage,
    allFound,
    manualRecords,
    managedRecords,
    hasManual,
    hasManaged,
    dnsModeLabel,
    handleStart,
    checkAll,
    finalizeIssuance,
    reset,
  };
}

