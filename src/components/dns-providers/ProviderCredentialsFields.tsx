import { Input } from "../ui/input";
import { Label } from "../ui/label";
import type {
  DnsProviderType,
} from "../../lib/dns-providers";
import type { ProviderFormState } from "../../hooks/useDnsProviderManager";

interface ProviderCredentialsFieldsProps {
  providerType: DnsProviderType;
  formMode: "create" | "edit";
  formState: ProviderFormState;
  onFormStateChange: (updates: Partial<ProviderFormState>) => void;
}

export function ProviderCredentialsFields({
  providerType,
  formMode,
  formState,
  onFormStateChange,
}: ProviderCredentialsFieldsProps) {
  const isRoute53 = providerType === "route53";

  return (
    <div className="space-y-2">
      {isRoute53 ? (
        <div className="space-y-3">
          <div className="space-y-2">
            <Label htmlFor="route53-access-key">Access key ID</Label>
            <Input
              id="route53-access-key"
              type="password"
              autoComplete="off"
              placeholder={
                formMode === "edit"
                  ? "Enter a new access key (optional)"
                  : "Paste Route 53 access key"
              }
              value={formState.route53_access_key || ""}
              onChange={(e) => {
                onFormStateChange({ route53_access_key: e.target.value });
              }}
              required={formMode === "create"}
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="route53-secret-key">Secret access key</Label>
            <Input
              id="route53-secret-key"
              type="password"
              autoComplete="off"
              placeholder={
                formMode === "edit"
                  ? "Enter a new secret key (optional)"
                  : "Paste Route 53 secret key"
              }
              value={formState.route53_secret_key || ""}
              onChange={(e) => {
                onFormStateChange({ route53_secret_key: e.target.value });
              }}
              required={formMode === "create"}
            />
          </div>
        </div>
      ) : (
        <div className="space-y-2">
          <Label htmlFor="provider-api-token">API token</Label>
          <Input
            id="provider-api-token"
            type="password"
            autoComplete="off"
            placeholder={
              formMode === "edit"
                ? "Enter a new token to rotate (optional)"
                : "Paste provider API token"
            }
            value={formState.api_token || ""}
            onChange={(e) => {
              onFormStateChange({ api_token: e.target.value });
            }}
            required={formMode === "create"}
          />
        </div>
      )}
      <p className="text-xs text-muted-foreground">
        Tokens are stored in the Rust core and never sent back to the UI. Test the connection after saving the provider.
      </p>
    </div>
  );
}
