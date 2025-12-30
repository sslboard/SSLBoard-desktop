import { useState } from "react";
import {
  validateDnsProviderToken,
  type DnsProviderTokenValidationResult,
  type ValidateDnsProviderTokenRequest,
} from "../lib/dns-providers";
import { maybeToastVaultUnlockError, normalizeError } from "../lib/errors";
import type { ProviderFormState } from "./useDnsProviderManager";

export function useDnsProviderTokenTest() {
  const [tokenTestResult, setTokenTestResult] =
    useState<DnsProviderTokenValidationResult | null>(null);
  const [tokenTestLoading, setTokenTestLoading] = useState(false);

  async function testToken(formState: ProviderFormState) {
    if (tokenTestLoading) return;
    setTokenTestLoading(true);
    try {
      const payload: ValidateDnsProviderTokenRequest = {
        provider_type: formState.provider_type,
        api_token: formState.api_token,
        route53_access_key: formState.route53_access_key,
        route53_secret_key: formState.route53_secret_key,
      };
      const result = await validateDnsProviderToken(payload);
      setTokenTestResult(result);
    } catch (err) {
      const message = normalizeError(err);
      maybeToastVaultUnlockError(message);
      setTokenTestResult({
        success: false,
        error: message,
      });
    } finally {
      setTokenTestLoading(false);
    }
  }

  function clearTokenTestResult() {
    if (tokenTestResult) {
      setTokenTestResult(null);
    }
  }

  return {
    tokenTestResult,
    tokenTestLoading,
    testToken,
    clearTokenTestResult,
  };
}
