import { invoke } from "@tauri-apps/api/core";

export type SecretKind =
  | "dns_provider_token"
  | "dns_provider_access_key"
  | "dns_provider_secret_key"
  | "acme_account_key"
  | "managed_private_key";

export type SecretRefRecord = {
  id: string;
  kind: SecretKind;
  label: string;
  created_at: string;
};

export async function listSecretRefs(): Promise<SecretRefRecord[]> {
  return invoke<SecretRefRecord[]>("list_secret_refs");
}

export async function lockVault(): Promise<void> {
  return invoke("lock_vault");
}
