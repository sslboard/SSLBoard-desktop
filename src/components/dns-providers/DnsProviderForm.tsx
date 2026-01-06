import type { Dispatch, FormEvent, SetStateAction } from "react";
import { Plus, RefreshCw } from "lucide-react";
import { Button } from "../ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "../ui/card";
import { Input } from "../ui/input";
import { Label } from "../ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "../ui/select";
import { Textarea } from "../ui/textarea";
import type {
  DnsProviderType,
} from "../../lib/dns-providers";
import { PROVIDER_OPTIONS } from "./provider-constants";
import type { ProviderFormState } from "../../hooks/useDnsProviderManager";
import { ProviderCredentialsFields } from "./ProviderCredentialsFields";

export function DnsProviderForm({
  formState,
  setFormState,
  formMode,
  saving,
  onSubmit,
  onCancel,
}: {
  formState: ProviderFormState;
  setFormState: Dispatch<SetStateAction<ProviderFormState>>;
  formMode: "create" | "edit";
  saving: boolean;
  onSubmit: (event: FormEvent<HTMLFormElement>) => void;
  onCancel: () => void;
}) {
  const requiresToken = formState.provider_type !== "manual";

  function handleFormStateChange(updates: Partial<ProviderFormState>) {
    setFormState((prev) => ({ ...prev, ...updates }));
  }

  return (
    <Card className="shadow-soft">
      <CardHeader className="flex-row items-start justify-between gap-3 space-y-0">
        <div>
          <CardTitle className="text-sm font-semibold">
            {formMode === "edit" ? "Edit provider" : "Add provider"}
          </CardTitle>
          <p className="text-xs text-muted-foreground">
            Map providers to domain suffixes (comma or space separated).
          </p>
        </div>
        {formMode === "edit" ? (
          <Button variant="ghost" size="sm" onClick={onCancel}>
            Cancel edit
          </Button>
        ) : null}
      </CardHeader>
      <CardContent>
        <form className="space-y-4" onSubmit={onSubmit}>
        <div className="space-y-2">
          <Label>Provider type</Label>
          <Select
            value={formState.provider_type}
            onValueChange={(value) => {
              handleFormStateChange({
                provider_type: value as DnsProviderType,
              });
            }}
            disabled={formMode === "edit"}
          >
            <SelectTrigger>
              <SelectValue placeholder="Select provider type" />
            </SelectTrigger>
            <SelectContent>
              {PROVIDER_OPTIONS.map((option) => (
                <SelectItem key={option.value} value={option.value}>
                  {option.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>

        <div className="space-y-2">
          <Label htmlFor="provider-label">Provider label</Label>
          <Input
            id="provider-label"
            placeholder="Production DNS"
            value={formState.label}
            onChange={(e) => handleFormStateChange({ label: e.target.value })}
            required
          />
        </div>

        <div className="space-y-2">
          <Label htmlFor="domain-suffixes">Domain suffixes</Label>
          <Textarea
            id="domain-suffixes"
            rows={3}
            placeholder="sslboard.com, example.net"
            value={formState.domain_suffixes}
            onChange={(e) =>
              handleFormStateChange({ domain_suffixes: e.target.value })
            }
            required
          />
        </div>

        {requiresToken ? (
          <ProviderCredentialsFields
            providerType={formState.provider_type}
            formMode={formMode}
            formState={formState}
            onFormStateChange={handleFormStateChange}
          />
        ) : null}

        <Button type="submit" className="w-full gap-2" disabled={saving}>
          {saving ? (
            <RefreshCw className="h-4 w-4 animate-spin" />
          ) : (
            <Plus className="h-4 w-4" />
          )}
          {formMode === "edit" ? "Save provider" : "Add provider"}
        </Button>
        </form>
      </CardContent>
    </Card>
  );
}
