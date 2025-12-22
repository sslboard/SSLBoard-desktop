import { useEffect, useState, type FormEvent } from "react";
import { Plus, RefreshCw } from "lucide-react";
import { Button } from "../ui/button";
import {
  createIssuer,
  deleteIssuer,
  listIssuers,
  updateIssuer,
  type IssuerConfig,
  type IssuerEnvironment,
} from "../../lib/issuers";
import { normalizeError } from "../../lib/errors";

type IssuerFormState = {
  issuer_id?: string;
  label: string;
  environment: IssuerEnvironment;
  directory_url: string;
  contact_email: string;
  tos_agreed: boolean;
};

const ACME_DIRECTORY_URLS: Record<IssuerEnvironment, string> = {
  staging: "https://acme-staging-v02.api.letsencrypt.org/directory",
  production: "https://acme-v02.api.letsencrypt.org/directory",
};

export function IssuerManager() {
  const [issuers, setIssuers] = useState<IssuerConfig[]>([]);
  const [issuerLoading, setIssuerLoading] = useState(false);
  const [issuerError, setIssuerError] = useState<string | null>(null);
  const [issuerFormMode, setIssuerFormMode] = useState<"create" | "edit">(
    "create",
  );
  const [issuerForm, setIssuerForm] = useState<IssuerFormState>({
    label: "",
    environment: "staging",
    directory_url: ACME_DIRECTORY_URLS.staging,
    contact_email: "",
    tos_agreed: false,
  });
  const [issuerFormSaving, setIssuerFormSaving] = useState(false);
  const [issuerFormError, setIssuerFormError] = useState<string | null>(null);

  useEffect(() => {
    void refreshIssuers();
  }, []);

  async function refreshIssuers(force = false) {
    if (issuerLoading && !force) return;
    setIssuerLoading(true);
    setIssuerError(null);
    try {
      const configs = await listIssuers();
      setIssuers(configs);
    } catch (err) {
      setIssuerError(normalizeError(err));
    } finally {
      setIssuerLoading(false);
    }
  }

  function resetIssuerForm() {
    setIssuerFormMode("create");
    setIssuerForm({
      label: "",
      environment: "staging",
      directory_url: ACME_DIRECTORY_URLS.staging,
      contact_email: "",
      tos_agreed: false,
    });
    setIssuerFormError(null);
  }

  function updateIssuerEnvironment(environment: IssuerEnvironment) {
    setIssuerForm((prev) => {
      const wasDefault =
        prev.directory_url === ACME_DIRECTORY_URLS[prev.environment];
      return {
        ...prev,
        environment,
        directory_url: wasDefault ? ACME_DIRECTORY_URLS[environment] : prev.directory_url,
      };
    });
  }

  function validateIssuerForm(form: IssuerFormState) {
    if (!form.label.trim()) {
      return "Issuer name is required.";
    }
    if (!form.directory_url.trim()) {
      return "Directory URL is required for ACME issuers.";
    }
    if (!form.contact_email.trim()) {
      return "Contact email is required for ACME issuers.";
    }
    if (!form.tos_agreed) {
      return "You must accept the ACME Terms of Service.";
    }
    return null;
  }

  async function handleIssuerSubmit(e: FormEvent<HTMLFormElement>) {
    e.preventDefault();
    if (issuerFormSaving) return;
    setIssuerFormError(null);
    const validation = validateIssuerForm(issuerForm);
    if (validation) {
      setIssuerFormError(validation);
      return;
    }

    setIssuerFormSaving(true);
    try {
      if (issuerFormMode === "create") {
        const created = await createIssuer({
          label: issuerForm.label.trim(),
          issuer_type: "acme",
          environment: issuerForm.environment,
          directory_url: issuerForm.directory_url.trim(),
          contact_email: issuerForm.contact_email.trim(),
          tos_agreed: issuerForm.tos_agreed,
        });
        setIssuers((prev) => [...prev, created]);
      } else if (issuerForm.issuer_id) {
        const updated = await updateIssuer({
          issuer_id: issuerForm.issuer_id,
          label: issuerForm.label.trim(),
          environment: issuerForm.environment,
          directory_url: issuerForm.directory_url.trim(),
          contact_email: issuerForm.contact_email.trim(),
          tos_agreed: issuerForm.tos_agreed,
        });
        setIssuers((prev) =>
          prev.map((issuer) =>
            issuer.issuer_id === updated.issuer_id ? updated : issuer,
          ),
        );
      }
      resetIssuerForm();
      void refreshIssuers(true);
    } catch (err) {
      setIssuerFormError(normalizeError(err));
    } finally {
      setIssuerFormSaving(false);
    }
  }

  function handleEditIssuer(issuer: IssuerConfig) {
    setIssuerFormMode("edit");
    setIssuerForm({
      issuer_id: issuer.issuer_id,
      label: issuer.label,
      environment: issuer.environment,
      directory_url: issuer.directory_url,
      contact_email: issuer.contact_email ?? "",
      tos_agreed: issuer.tos_agreed,
    });
    setIssuerFormError(null);
  }

  async function handleDeleteIssuer(issuer: IssuerConfig) {
    if (issuerLoading) return;
    setIssuerError(null);
    setIssuerLoading(true);
    try {
      const deletedId = await deleteIssuer({ issuer_id: issuer.issuer_id });
      setIssuers((prev) => prev.filter((entry) => entry.issuer_id !== deletedId));
    } catch (err) {
      setIssuerError(normalizeError(err));
    } finally {
      setIssuerLoading(false);
    }
  }

  function formatEnvironment(env?: IssuerConfig["environment"]) {
    switch (env) {
      case "production":
        return "Production";
      case "staging":
      default:
        return "Sandbox";
    }
  }

  function formatIssuerType(type?: IssuerConfig["issuer_type"]) {
    switch (type) {
      case "acme":
      default:
        return "ACME";
    }
  }

  return (
    <div className="rounded-xl border bg-card p-5 shadow-soft">
      <div className="flex items-center justify-between gap-3">
        <div>
          <div className="text-sm font-semibold">Issuer management</div>
          <p className="text-xs text-muted-foreground">
            Add or edit issuer entries. ACME issuers require contact email and ToS acceptance.
          </p>
        </div>
        {issuerFormMode === "edit" ? (
          <Button
            type="button"
            variant="ghost"
            size="sm"
            onClick={resetIssuerForm}
          >
            Cancel edit
          </Button>
        ) : null}
      </div>

      {issuerError ? (
        <div className="mt-3 rounded-lg border border-destructive/50 bg-destructive/10 px-3 py-2 text-xs text-destructive">
          {issuerError}
        </div>
      ) : null}

      {issuerFormError ? (
        <div className="mt-3 rounded-lg border border-destructive/50 bg-destructive/10 px-3 py-2 text-xs text-destructive">
          {issuerFormError}
        </div>
      ) : null}

      <div className="mt-4 grid gap-4 lg:grid-cols-[1.2fr,1fr]">
        <div className="space-y-3">
          {issuers.map((issuer) => (
            <div
              key={issuer.issuer_id}
              className="rounded-lg border bg-background/80 p-4 shadow-sm"
            >
              <div className="flex flex-wrap items-start justify-between gap-3">
                <div>
                  <div className="flex flex-wrap items-center gap-2 text-sm font-semibold">
                    {issuer.label}
                  </div>
                  <div className="mt-1 text-xs text-muted-foreground">
                    {formatIssuerType(issuer.issuer_type)} · {formatEnvironment(issuer.environment)} ·{" "}
                    {issuer.tos_agreed ? "ToS accepted" : "ToS pending"}
                  </div>
                  <div className="mt-1 text-xs text-muted-foreground">
                    {issuer.directory_url}
                  </div>
                </div>
                <div className="flex flex-wrap gap-2">
                  <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    onClick={() => handleEditIssuer(issuer)}
                    disabled={issuerLoading}
                  >
                    Edit
                  </Button>
                  <Button
                    type="button"
                    variant="ghost"
                    size="sm"
                    className="text-destructive hover:bg-destructive/10"
                    onClick={() => void handleDeleteIssuer(issuer)}
                    disabled={issuerLoading}
                  >
                    Remove
                  </Button>
                </div>
              </div>
            </div>
          ))}
        </div>

        <form className="space-y-3" onSubmit={handleIssuerSubmit}>
          <div className="space-y-1">
            <label className="text-sm font-medium text-foreground">
              Issuer name
            </label>
            <input
              className="w-full rounded-lg border bg-background/60 p-2.5 text-sm shadow-inner outline-none ring-offset-background focus:ring-2 focus:ring-primary/50"
              value={issuerForm.label}
              onChange={(e) =>
                setIssuerForm((prev) => ({ ...prev, label: e.target.value }))
              }
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
              value={issuerForm.environment}
              onChange={(e) =>
                updateIssuerEnvironment(e.target.value as IssuerEnvironment)
              }
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
              value={issuerForm.directory_url}
              onChange={(e) =>
                setIssuerForm((prev) => ({
                  ...prev,
                  directory_url: e.target.value,
                }))
              }
              required
            />
          </div>

          <div className="space-y-1">
            <label className="text-sm font-medium text-foreground">
              Contact email
            </label>
            <input
              className="w-full rounded-lg border bg-background/60 p-2.5 text-sm shadow-inner outline-none ring-offset-background focus:ring-2 focus:ring-primary/50"
              value={issuerForm.contact_email}
              onChange={(e) =>
                setIssuerForm((prev) => ({
                  ...prev,
                  contact_email: e.target.value,
                }))
              }
              placeholder="you@example.com"
              required
            />
          </div>

          <label className="flex items-center gap-2 text-sm font-medium text-foreground">
            <input
              type="checkbox"
              className="h-4 w-4"
              checked={issuerForm.tos_agreed}
              onChange={(e) =>
                setIssuerForm((prev) => ({
                  ...prev,
                  tos_agreed: e.target.checked,
                }))
              }
              required
            />
            I accept the ACME Terms of Service
          </label>

          <Button type="submit" className="w-full gap-2" disabled={issuerFormSaving}>
            {issuerFormSaving ? (
              <RefreshCw className="h-4 w-4 animate-spin" />
            ) : (
              <Plus className="h-4 w-4" />
            )}
            {issuerFormMode === "edit" ? "Save issuer changes" : "Add issuer"}
          </Button>
        </form>
      </div>
    </div>
  );
}
