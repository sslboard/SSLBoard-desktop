import { toast } from "sonner";

export function normalizeError(err: unknown, fallback = "Unexpected error") {
  if (err instanceof Error) return err.message;
  if (typeof err === "string") return err;
  return fallback;
}

const VAULT_ERROR_HINTS = ["vault", "keyring", "biometric", "credential"];

export function maybeToastVaultUnlockError(message: string) {
  const lower = message.toLowerCase();
  if (!VAULT_ERROR_HINTS.some((hint) => lower.includes(hint))) return;
  toast.error("Vault unlock failed", {
    description: message,
  });
}
