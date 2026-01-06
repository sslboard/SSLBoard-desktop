import { useEffect, useState } from "react";
import {
  listSecretRefs,
  type SecretRefRecord,
} from "../lib/secrets";
import { maybeToastVaultUnlockError, normalizeError } from "../lib/errors";

export function useSecretReferences() {
  const [secrets, setSecrets] = useState<SecretRefRecord[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    void refresh();
  }, []);

  async function refresh() {
    setLoading(true);
    setError(null);
    try {
      const records = await listSecretRefs();
      setSecrets(records);
    } catch (err) {
      const message = normalizeError(err);
      setError(message);
      maybeToastVaultUnlockError(message);
    } finally {
      setLoading(false);
    }
  }

  return {
    secrets,
    loading,
    error,
    refresh,
  };
}
