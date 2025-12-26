import { useEffect, useState, type FormEvent } from "react";
import { Button } from "../ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "../ui/card";
import {
  createIssuer,
  deleteIssuer,
  listIssuers,
  updateIssuer,
  type IssuerConfig,
  type IssuerEnvironment,
} from "../../lib/issuers";
import { normalizeError } from "../../lib/errors";
import { validateIssuerForm, type IssuerFormState } from "../../lib/issuers/validation";
import { IssuerList } from "./issuers/IssuerList";
import { IssuerForm } from "./issuers/IssuerForm";

const ACME_DIRECTORY_URLS: Record<IssuerEnvironment, string> = {
  staging: "https://acme-staging-v02.api.letsencrypt.org/directory",
  production: "https://acme-v02.api.letsencrypt.org/directory",
};

export function IssuerManager() {
  const [issuers, setIssuers] = useState<IssuerConfig[]>([]);
  const [issuerLoading, setIssuerLoading] = useState(false);
  const [issuerError, setIssuerError] = useState<string | null>(null);
  const [issuerFormMode, setIssuerFormMode] = useState<"create" | "edit">("create");
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

  function handleFormStateChange(updates: Partial<IssuerFormState>) {
    setIssuerForm((prev) => ({ ...prev, ...updates }));
  }

  return (
    <Card className="shadow-soft">
      <CardHeader className="flex-row items-start justify-between gap-3 space-y-0">
        <div>
          <CardTitle className="text-sm font-semibold">Issuer management</CardTitle>
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
      </CardHeader>

      {issuerError ? (
        <div className="mx-6 mt-3 rounded-lg border border-destructive/50 bg-destructive/10 px-3 py-2 text-xs text-destructive">
          {issuerError}
        </div>
      ) : null}

      {issuerFormError ? (
        <div className="mx-6 mt-3 rounded-lg border border-destructive/50 bg-destructive/10 px-3 py-2 text-xs text-destructive">
          {issuerFormError}
        </div>
      ) : null}

      <CardContent>
        <div className="grid gap-4 lg:grid-cols-[1.2fr,1fr]">
          <IssuerList
            issuers={issuers}
            issuerLoading={issuerLoading}
            onEdit={handleEditIssuer}
            onDelete={handleDeleteIssuer}
          />

          <IssuerForm
            formState={issuerForm}
            formMode={issuerFormMode}
            saving={issuerFormSaving}
            onFormStateChange={handleFormStateChange}
            onEnvironmentChange={updateIssuerEnvironment}
            onSubmit={handleIssuerSubmit}
          />
        </div>
      </CardContent>
    </Card>
  );
}
