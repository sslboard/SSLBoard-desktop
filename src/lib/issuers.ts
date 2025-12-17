import { invoke } from "@tauri-apps/api/core";

export type IssuerEnvironment = "staging" | "production";

export type IssuerConfig = {
  issuer_id: string;
  label: string;
  directory_url: string;
  environment: IssuerEnvironment;
  contact_email?: string | null;
  account_key_ref?: string | null;
  is_selected: boolean;
  disabled: boolean;
};

export type EnsureAcmeAccountRequest = {
  issuer_id: string;
  contact_email?: string;
  account_key_ref?: string;
  generate_new_account_key?: boolean;
};

export async function listIssuers(): Promise<IssuerConfig[]> {
  return invoke<IssuerConfig[]>("list_issuers");
}

export async function selectIssuer(issuerId: string): Promise<IssuerConfig> {
  return invoke<IssuerConfig>("select_issuer", { req: { issuer_id: issuerId } });
}

export async function ensureAcmeAccount(
  req: EnsureAcmeAccountRequest,
): Promise<IssuerConfig> {
  return invoke<IssuerConfig>("ensure_acme_account", { req });
}
