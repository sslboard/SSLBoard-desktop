import { CheckCircle2, RefreshCw, XCircle } from "lucide-react";
import { Button } from "../ui/button";
import type {
  DnsProviderTokenValidationResult,
  DnsProviderType,
} from "../../lib/dns-providers";
import {
  ERROR_CATEGORY_LABELS,
  ERROR_CATEGORY_SUGGESTIONS,
} from "./provider-constants";
import type { ProviderFormState } from "../../hooks/useDnsProviderManager";

interface ProviderCredentialsFieldsProps {
  providerType: DnsProviderType;
  formMode: "create" | "edit";
  formState: ProviderFormState;
  onFormStateChange: (updates: Partial<ProviderFormState>) => void;
  onTokenInputChange: () => void;
  tokenTestResult: DnsProviderTokenValidationResult | null;
  tokenTestLoading: boolean;
  onTestToken: () => void;
}

export function ProviderCredentialsFields({
  providerType,
  formMode,
  formState,
  onFormStateChange,
  onTokenInputChange,
  tokenTestResult,
  tokenTestLoading,
  onTestToken,
}: ProviderCredentialsFieldsProps) {
  const isRoute53 = providerType === "route53";
  const tokenReady = isRoute53
    ? Boolean(formState.route53_access_key?.trim() && formState.route53_secret_key?.trim())
    : Boolean(formState.api_token?.trim());

  return (
    <div className="space-y-2">
      {isRoute53 ? (
        <div className="space-y-3">
          <div className="space-y-2">
            <label className="text-sm font-medium text-foreground">
              Access key ID
            </label>
            <input
              type="password"
              autoComplete="off"
              className="w-full rounded-lg border bg-background/60 p-3 text-sm shadow-inner outline-none ring-offset-background focus:ring-2 focus:ring-primary/50"
              placeholder={
                formMode === "edit"
                  ? "Enter a new access key (optional)"
                  : "Paste Route 53 access key"
              }
              value={formState.route53_access_key || ""}
              onChange={(e) => {
                onFormStateChange({ route53_access_key: e.target.value });
                onTokenInputChange();
              }}
              required={formMode === "create"}
            />
          </div>
          <div className="space-y-2">
            <label className="text-sm font-medium text-foreground">
              Secret access key
            </label>
            <input
              type="password"
              autoComplete="off"
              className="w-full rounded-lg border bg-background/60 p-3 text-sm shadow-inner outline-none ring-offset-background focus:ring-2 focus:ring-primary/50"
              placeholder={
                formMode === "edit"
                  ? "Enter a new secret key (optional)"
                  : "Paste Route 53 secret key"
              }
              value={formState.route53_secret_key || ""}
              onChange={(e) => {
                onFormStateChange({ route53_secret_key: e.target.value });
                onTokenInputChange();
              }}
              required={formMode === "create"}
            />
          </div>
        </div>
      ) : (
        <div className="space-y-2">
          <label className="text-sm font-medium text-foreground">
            API token
          </label>
          <input
            type="password"
            autoComplete="off"
            className="w-full rounded-lg border bg-background/60 p-3 text-sm shadow-inner outline-none ring-offset-background focus:ring-2 focus:ring-primary/50"
            placeholder={
              formMode === "edit"
                ? "Enter a new token to rotate (optional)"
                : "Paste provider API token"
            }
            value={formState.api_token || ""}
            onChange={(e) => {
              onFormStateChange({ api_token: e.target.value });
              onTokenInputChange();
            }}
            required={formMode === "create"}
          />
        </div>
      )}
      <p className="text-xs text-muted-foreground">
        Tokens are stored in the Rust core and never sent back to the UI.
      </p>
      <div className="flex flex-wrap items-center gap-3">
        <Button
          type="button"
          variant="outline"
          size="sm"
          onClick={onTestToken}
          disabled={tokenTestLoading || !tokenReady}
          className="gap-2"
        >
          {tokenTestLoading ? (
            <RefreshCw className="h-4 w-4 animate-spin" />
          ) : null}
          Test token
        </Button>
        {!tokenReady ? (
          <span className="text-xs text-muted-foreground">
            Enter credentials to test.
          </span>
        ) : null}
      </div>
      {tokenTestResult ? (
        <div
          className={`rounded-md border px-3 py-2 text-xs ${
            tokenTestResult.success
              ? "border-emerald-200 bg-emerald-50 text-emerald-700"
              : "border-rose-200 bg-rose-50 text-rose-700"
          }`}
        >
          <div className="flex items-center gap-2 font-semibold">
            {tokenTestResult.success ? (
              <CheckCircle2 className="h-4 w-4" />
            ) : (
              <XCircle className="h-4 w-4" />
            )}
            {tokenTestResult.success
              ? "Token verified"
              : "Token validation failed"}
          </div>
          {!tokenTestResult.success ? (
            <div className="mt-1 space-y-1 text-xs">
              <div>{tokenTestResult.error || "Validation failed."}</div>
              {tokenTestResult.error_category ? (
                <div className="text-rose-800">
                  {ERROR_CATEGORY_LABELS[tokenTestResult.error_category]} ·{" "}
                  {ERROR_CATEGORY_SUGGESTIONS[tokenTestResult.error_category]}
                </div>
              ) : null}
            </div>
          ) : null}
        </div>
      ) : null}
    </div>
  );
}

