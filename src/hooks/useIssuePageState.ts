import { useState } from "react";
import type { IssuanceMode } from "../components/issue/IssuanceModeCard";
import type { IssuanceKeyOption, CsrValidationResult, CsrSource } from "../lib/issuance";

export function useIssuePageState() {
  const [issuanceMode, setIssuanceMode] = useState<IssuanceMode>("dns");
  const [domainsInput, setDomainsInput] = useState("test.ezs3.net");
  const [keyOption, setKeyOption] = useState<IssuanceKeyOption>("rsa-2048");
  const [csrPath, setCsrPath] = useState<string | null>(null);
  const [csrResult, setCsrResult] = useState<CsrValidationResult | null>(null);
  const [csrLoading, setCsrLoading] = useState(false);
  const [csrError, setCsrError] = useState<string | null>(null);
  const [csrSource, setCsrSource] = useState<CsrSource>("imported");
  const [csrManagedKeyRef, setCsrManagedKeyRef] = useState<string | null>(null);
  const [reuseKeyRef, setReuseKeyRef] = useState<string | null>(null);
  const [reuseKeyEnabled, setReuseKeyEnabled] = useState(false);

  function resetDomainsState() {
    setDomainsInput("test.ezs3.net");
    setKeyOption("rsa-2048");
    setReuseKeyRef(null);
    setReuseKeyEnabled(false);
  }

  function resetCsrState() {
    setCsrPath(null);
    setCsrResult(null);
    setCsrError(null);
    setCsrManagedKeyRef(null);
    setCsrSource("imported");
  }

  return {
    issuanceMode,
    setIssuanceMode,
    domainsInput,
    setDomainsInput,
    keyOption,
    setKeyOption,
    csrPath,
    setCsrPath,
    csrResult,
    setCsrResult,
    csrLoading,
    setCsrLoading,
    csrError,
    setCsrError,
    csrSource,
    setCsrSource,
    csrManagedKeyRef,
    setCsrManagedKeyRef,
    reuseKeyRef,
    setReuseKeyRef,
    reuseKeyEnabled,
    setReuseKeyEnabled,
    resetDomainsState,
    resetCsrState,
  };
}
