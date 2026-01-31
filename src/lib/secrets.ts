import { invoke } from "@tauri-apps/api/core";

export async function lockVault(): Promise<void> {
  return invoke("lock_vault");
}
