import { type FormEvent } from "react";
import { Plus, RefreshCw } from "lucide-react";
import { Button } from "../../ui/button";
import { Checkbox } from "../../ui/checkbox";
import { Input } from "../../ui/input";
import { Label } from "../../ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "../../ui/select";
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
        <Label htmlFor="issuer-name">Issuer name</Label>
        <Input
          id="issuer-name"
          value={formState.label}
          onChange={(e) => onFormStateChange({ label: e.target.value })}
          placeholder="Let's Encrypt (Custom)"
          required
        />
      </div>

      <div className="space-y-1">
        <Label>Environment</Label>
        <Select
          value={formState.environment}
          onValueChange={(value) =>
            onEnvironmentChange(value as IssuerEnvironment)
          }
        >
          <SelectTrigger>
            <SelectValue placeholder="Select environment" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="staging">Sandbox (staging)</SelectItem>
            <SelectItem value="production">Production</SelectItem>
          </SelectContent>
        </Select>
      </div>

      <div className="space-y-1">
        <Label htmlFor="issuer-directory-url">Directory URL</Label>
        <Input
          id="issuer-directory-url"
          value={formState.directory_url}
          onChange={(e) => onFormStateChange({ directory_url: e.target.value })}
          required
        />
      </div>

      <div className="space-y-1">
        <Label htmlFor="issuer-contact-email">Contact email</Label>
        <Input
          id="issuer-contact-email"
          value={formState.contact_email}
          onChange={(e) => onFormStateChange({ contact_email: e.target.value })}
          placeholder="you@example.com"
          required
        />
      </div>

      <div className="flex items-center gap-2">
        <Checkbox
          id="issuer-tos-agreed"
          checked={formState.tos_agreed}
          onCheckedChange={(checked) =>
            onFormStateChange({ tos_agreed: checked === true })
          }
          required
        />
        <Label htmlFor="issuer-tos-agreed" className="text-sm font-medium">
        I accept the ACME Terms of Service
        </Label>
      </div>

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
