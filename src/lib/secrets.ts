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

export type CreateSecretRequest = {
  label: string;
  kind: SecretKind;
  secret_value: string;
};

export type UpdateSecretRequest = {
  id: string;
  secret_value: string;
  label?: string;
};

export async function listSecretRefs(): Promise<SecretRefRecord[]> {
  return invoke<SecretRefRecord[]>("list_secret_refs");
}

export async function createSecretRef(
  req: CreateSecretRequest,
): Promise<SecretRefRecord> {
  return invoke<SecretRefRecord>("create_secret_ref", { createReq: req });
}

export async function updateSecretRef(
  req: UpdateSecretRequest,
): Promise<SecretRefRecord> {
  return invoke<SecretRefRecord>("update_secret_ref", { updateReq: req });
}

export async function deleteSecretRef(id: string): Promise<void> {
  return invoke("delete_secret_ref", { id });
}

export async function unlockVault(): Promise<boolean> {
  return invoke<boolean>("unlock_vault");
}

export async function lockVault(): Promise<boolean> {
  return invoke<boolean>("lock_vault");
}

export async function isVaultUnlocked(): Promise<boolean> {
  return invoke<boolean>("is_vault_unlocked");
}
