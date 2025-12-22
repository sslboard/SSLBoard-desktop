import type { Dispatch, FormEvent, SetStateAction } from "react";
import { Plus, RefreshCw } from "lucide-react";
import { Button } from "../ui/button";
import type { DnsProviderType } from "../../lib/dns-providers";
import { PROVIDER_OPTIONS } from "./provider-constants";
import type { ProviderFormState } from "../../hooks/useDnsProviderManager";

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
  return (
    <div className="rounded-xl border bg-card p-5 shadow-soft">
      <div className="flex items-center justify-between gap-3">
        <div>
          <div className="text-sm font-semibold">
            {formMode === "edit" ? "Edit provider" : "Add provider"}
          </div>
          <p className="text-xs text-muted-foreground">
            Map providers to domain suffixes (comma or space separated).
          </p>
        </div>
        {formMode === "edit" ? (
          <Button variant="ghost" size="sm" onClick={onCancel}>
            Cancel edit
          </Button>
        ) : null}
      </div>

      <form className="mt-4 space-y-4" onSubmit={onSubmit}>
        <div className="space-y-2">
          <label className="text-sm font-medium text-foreground">
            Provider type
          </label>
          <select
            className="w-full rounded-lg border bg-background/60 p-2.5 text-sm shadow-inner outline-none ring-offset-background focus:ring-2 focus:ring-primary/50"
            value={formState.provider_type}
            onChange={(e) =>
              setFormState((prev) => ({
                ...prev,
                provider_type: e.target.value as DnsProviderType,
              }))
            }
            disabled={formMode === "edit"}
          >
            {PROVIDER_OPTIONS.map((option) => (
              <option key={option.value} value={option.value}>
                {option.label}
              </option>
            ))}
          </select>
        </div>

        <div className="space-y-2">
          <label className="text-sm font-medium text-foreground">
            Provider label
          </label>
          <input
            className="w-full rounded-lg border bg-background/60 p-2.5 text-sm shadow-inner outline-none ring-offset-background focus:ring-2 focus:ring-primary/50"
            placeholder="Production DNS"
            value={formState.label}
            onChange={(e) =>
              setFormState((prev) => ({ ...prev, label: e.target.value }))
            }
            required
          />
        </div>

        <div className="space-y-2">
          <label className="text-sm font-medium text-foreground">
            Domain suffixes
          </label>
          <textarea
            className="w-full rounded-lg border bg-background/60 p-2.5 text-sm shadow-inner outline-none ring-offset-background focus:ring-2 focus:ring-primary/50"
            rows={3}
            placeholder="sslboard.com, example.net"
            value={formState.domain_suffixes}
            onChange={(e) =>
              setFormState((prev) => ({
                ...prev,
                domain_suffixes: e.target.value,
              }))
            }
            required
          />
        </div>

        {formState.provider_type !== "manual" ? (
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
              onChange={(e) =>
                setFormState((prev) => ({
                  ...prev,
                  api_token: e.target.value,
                }))
              }
              required={formMode === "create"}
            />
            <p className="text-xs text-muted-foreground">
              Tokens are stored in the Rust core and never sent back to the UI.
            </p>
          </div>
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
    </div>
  );
}
