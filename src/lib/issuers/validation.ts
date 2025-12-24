import type { IssuerEnvironment } from "../issuers";

export type IssuerFormState = {
  issuer_id?: string;
  label: string;
  environment: IssuerEnvironment;
  directory_url: string;
  contact_email: string;
  tos_agreed: boolean;
};

export function validateIssuerForm(form: IssuerFormState): string | null {
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

