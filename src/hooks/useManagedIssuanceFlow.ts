import { useEffect, useRef, useState } from "react";
import { checkDnsPropagation, type PropagationResult } from "../lib/dns";
import {
  completeManagedIssuance,
  keyOptionToParams,
  startManagedIssuance,
  type IssuanceKeyOption,
  type StartIssuanceResponse,
} from "../lib/issuance";
import { normalizeError } from "../lib/errors";
import type { CertificateRecord } from "../lib/certificates";

type StatusMap = Record<string, PropagationResult | null>;

const DNS_RETRY_WINDOW_MS = 60_000;
const DNS_RETRY_INTERVAL_MS = 8_000;

function getRecordKey(rec: { record_name: string; value: string }): string {
  return `${rec.record_name}:${rec.value}`;
}

function getDnsDomain(recordName: string) {
  return recordName.replace(/^_acme-challenge\./, "");
}

function allRecordsFound(
  statusMap: StatusMap,
  startResult: StartIssuanceResponse,
) {
  return startResult.dns_records.every((rec) => {
    const status = statusMap[getRecordKey(rec)];
    return status?.state === "found";
  });
}

export function useManagedIssuanceFlow(
  selectedIssuerId: string | null,
  parsedDomains: string[],
  keyOption: IssuanceKeyOption,
) {
  const [startResult, setStartResult] = useState<StartIssuanceResponse | null>(null);
  const [statusMap, setStatusMap] = useState<StatusMap>({});
  const [loadingStart, setLoadingStart] = useState(false);
  const [checking, setChecking] = useState(false);
  const [finalizing, setFinalizing] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [certificate, setCertificate] = useState<CertificateRecord | null>(null);
  const [awaitingManual, setAwaitingManual] = useState(false);
  const [dnsFailed, setDnsFailed] = useState(false);
  const [finalizeFailed, setFinalizeFailed] = useState(false);
  const flowTokenRef = useRef(0);

  useEffect(() => {
    if (!startResult) {
      setStatusMap({});
    }
  }, [startResult]);

  function nextFlowToken() {
    flowTokenRef.current += 1;
    return flowTokenRef.current;
  }

  function isStale(token: number) {
    return flowTokenRef.current !== token;
  }

  async function sleep(durationMs: number) {
    return new Promise((resolve) => setTimeout(resolve, durationMs));
  }

  async function checkDnsOnce(result: StartIssuanceResponse, token: number) {
    const updates: StatusMap = {};
    for (const rec of result.dns_records) {
      const domain = getDnsDomain(rec.record_name);
      const status = await checkDnsPropagation(domain, rec.value);
      if (isStale(token)) {
        return updates;
      }
      updates[getRecordKey(rec)] = status;
    }
    setStatusMap(updates);
    return updates;
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

  async function runDnsVerification(result: StartIssuanceResponse, token: number) {
    setChecking(true);
    setDnsFailed(false);
    setError(null);
    const startedAt = Date.now();
    try {
      while (Date.now() - startedAt < DNS_RETRY_WINDOW_MS) {
        if (isStale(token)) {
          return;
        }
        const updates = await checkDnsOnce(result, token);
        if (isStale(token)) {
          return;
        }
        if (allRecordsFound(updates, result)) {
          setChecking(false);
          await finalizeIssuance(result, token);
          return;
        }
        await sleep(DNS_RETRY_INTERVAL_MS);
      }
      if (!isStale(token)) {
        setDnsFailed(true);
        setError("DNS propagation was not detected yet. Try again shortly.");
      }
    } catch (err) {
      if (!isStale(token)) {
        setDnsFailed(true);
        setError(normalizeError(err, "DNS verification failed."));
      }
    } finally {
      if (!isStale(token)) {
        setChecking(false);
      }
    }
  }

  async function handleStart() {
    const token = nextFlowToken();
    setLoadingStart(true);
    setError(null);
    setCertificate(null);
    setAwaitingManual(false);
    setDnsFailed(false);
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
      });
      if (isStale(token)) {
        return;
      }
      setStartResult(result);
      const initialStatus: StatusMap = {};
      result.dns_records.forEach((rec) => {
        initialStatus[getRecordKey(rec)] = null;
      });
      setStatusMap(initialStatus);
      const hasManualRecords = result.dns_records.some((rec) => rec.adapter === "manual");
      setAwaitingManual(hasManualRecords);
      if (!hasManualRecords) {
        await runDnsVerification(result, token);
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
    await runDnsVerification(startResult, token);
  }

  async function retryDnsVerification() {
    if (!startResult) return;
    const token = nextFlowToken();
    await runDnsVerification(startResult, token);
  }

  async function retryFinalization() {
    if (!startResult) return;
    const token = nextFlowToken();
    await finalizeIssuance(startResult, token);
  }

  function reset() {
    nextFlowToken();
    setStartResult(null);
    setStatusMap({});
    setError(null);
    setCertificate(null);
    setAwaitingManual(false);
    setDnsFailed(false);
    setFinalizeFailed(false);
    setLoadingStart(false);
    setChecking(false);
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
    statusMap,
    loadingStart,
    checking,
    finalizing,
    error,
    certificate,
    manualRecords,
    managedRecords,
    hasManual,
    hasManaged,
    dnsModeLabel,
    awaitingManual,
    dnsFailed,
    finalizeFailed,
    handleStart,
    continueIssuance,
    retryDnsVerification,
    retryFinalization,
    reset,
  };
}
