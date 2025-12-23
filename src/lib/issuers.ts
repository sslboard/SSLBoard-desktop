import { invoke } from "@tauri-apps/api/core";

export type IssuerEnvironment = "staging" | "production";
export type IssuerType = "acme";

export type IssuerConfig = {
  issuer_id: string;
  label: string;
  directory_url: string;
  environment: IssuerEnvironment;
  issuer_type: IssuerType;
  contact_email?: string | null;
  account_key_ref?: string | null;
  tos_agreed: boolean;
  is_selected: boolean;
};

export type CreateIssuerRequest = {
  label: string;
  issuer_type: IssuerType;
  environment: IssuerEnvironment;
  directory_url: string;
  contact_email?: string;
  tos_agreed: boolean;
};

export type UpdateIssuerRequest = {
  issuer_id: string;
  label: string;
  environment: IssuerEnvironment;
  directory_url: string;
  contact_email?: string;
  tos_agreed: boolean;
};

export type DeleteIssuerRequest = {
  issuer_id: string;
};

export async function listIssuers(): Promise<IssuerConfig[]> {
  return invoke<IssuerConfig[]>("list_issuers");
}

export async function selectIssuer(issuerId: string): Promise<IssuerConfig> {
  return invoke<IssuerConfig>("select_issuer", {
    selectReq: { issuer_id: issuerId },
  });
}

export async function createIssuer(
  req: CreateIssuerRequest,
): Promise<IssuerConfig> {
  return invoke<IssuerConfig>("create_issuer", { createReq: req });
}

export async function updateIssuer(
  req: UpdateIssuerRequest,
): Promise<IssuerConfig> {
  return invoke<IssuerConfig>("update_issuer", { updateReq: req });
}

export async function deleteIssuer(
  req: DeleteIssuerRequest,
): Promise<string> {
  return invoke<string>("delete_issuer", { deleteReq: req });
}
