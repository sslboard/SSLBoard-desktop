import { type FormEvent } from "react";
import { Plus, RefreshCw } from "lucide-react";
import { Button } from "../../ui/button";
import type { IssuerEnvironment } from "../../../lib/issuers";
import type { IssuerFormState } from "../../../lib/issuers/validation";

interface IssuerFormProps {
  formState: IssuerFormState;
  formMode: "create" | "edit";
  saving: boolean;
  onFormStateChange: (updates: Partial<IssuerFormState>) => void;
  onEnvironmentChange: (environment: IssuerEnvironment) => void;
  onSubmit: (event: FormEvent<HTMLFormElement>) => void;
}

export function IssuerForm({
  formState,
  formMode,
  saving,
  onFormStateChange,
  onEnvironmentChange,
  onSubmit,
}: IssuerFormProps) {
  return (
    <form className="space-y-3" onSubmit={onSubmit}>
      <div className="space-y-1">
        <label className="text-sm font-medium text-foreground">
          Issuer name
        </label>
        <input
          className="w-full rounded-lg border bg-background/60 p-2.5 text-sm shadow-inner outline-none ring-offset-background focus:ring-2 focus:ring-primary/50"
          value={formState.label}
          onChange={(e) => onFormStateChange({ label: e.target.value })}
          placeholder="Let's Encrypt (Custom)"
          required
        />
      </div>

      <div className="space-y-1">
        <label className="text-sm font-medium text-foreground">
          Environment
        </label>
        <select
          className="w-full rounded-lg border bg-background/60 p-2.5 text-sm shadow-inner outline-none ring-offset-background focus:ring-2 focus:ring-primary/50"
          value={formState.environment}
          onChange={(e) => {
            const env = e.target.value as IssuerEnvironment;
            onEnvironmentChange(env);
          }}
        >
          <option value="staging">Sandbox (staging)</option>
          <option value="production">Production</option>
        </select>
      </div>

      <div className="space-y-1">
        <label className="text-sm font-medium text-foreground">
          Directory URL
        </label>
        <input
          className="w-full rounded-lg border bg-background/60 p-2.5 text-sm shadow-inner outline-none ring-offset-background focus:ring-2 focus:ring-primary/50"
          value={formState.directory_url}
          onChange={(e) => onFormStateChange({ directory_url: e.target.value })}
          required
        />
      </div>

      <div className="space-y-1">
        <label className="text-sm font-medium text-foreground">
          Contact email
        </label>
        <input
          className="w-full rounded-lg border bg-background/60 p-2.5 text-sm shadow-inner outline-none ring-offset-background focus:ring-2 focus:ring-primary/50"
          value={formState.contact_email}
          onChange={(e) => onFormStateChange({ contact_email: e.target.value })}
          placeholder="you@example.com"
          required
        />
      </div>

      <label className="flex items-center gap-2 text-sm font-medium text-foreground">
        <input
          type="checkbox"
          className="h-4 w-4"
          checked={formState.tos_agreed}
          onChange={(e) => onFormStateChange({ tos_agreed: e.target.checked })}
          required
        />
        I accept the ACME Terms of Service
      </label>

      <Button type="submit" className="w-full gap-2" disabled={saving}>
        {saving ? (
          <RefreshCw className="h-4 w-4 animate-spin" />
        ) : (
          <Plus className="h-4 w-4" />
        )}
        {formMode === "edit" ? "Save issuer changes" : "Add issuer"}
      </Button>
    </form>
  );
}

