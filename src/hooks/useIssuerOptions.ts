import { useEffect, useState } from "react";
import { listIssuers, type IssuerConfig } from "../lib/issuers";
import { normalizeError } from "../lib/errors";

export function useIssuerOptions() {
  const [issuers, setIssuers] = useState<IssuerConfig[]>([]);
  const [issuerLoading, setIssuerLoading] = useState(false);
  const [issuerError, setIssuerError] = useState<string | null>(null);
  const [selectedIssuer, setSelectedIssuer] = useState<IssuerConfig | null>(null);

  useEffect(() => {
    let active = true;
    async function loadData() {
      setIssuerLoading(true);
      setIssuerError(null);
      try {
        const issuerList = await listIssuers();
        if (!active) return;
        setIssuers(issuerList);
        setSelectedIssuer(issuerList[0] ?? null);
      } catch (err) {
        if (active) {
          setIssuerError(normalizeError(err, "Failed to load issuers."));
        }
      } finally {
        if (active) {
          setIssuerLoading(false);
        }
      }
    }
    void loadData();
    return () => {
      active = false;
    };
  }, []);

  useEffect(() => {
    if (!selectedIssuer && issuers.length > 0) {
      setSelectedIssuer(issuers[0] ?? null);
    }
  }, [issuers, selectedIssuer]);

  function selectIssuerById(issuerId: string) {
    const issuer = issuers.find((entry) => entry.issuer_id === issuerId) ?? null;
    setSelectedIssuer(issuer);
  }

  return {
    issuers,
    issuerLoading,
    issuerError,
    selectedIssuer,
    selectIssuerById,
  };
}
