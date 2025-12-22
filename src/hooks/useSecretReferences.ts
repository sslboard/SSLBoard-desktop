import { useEffect, useState } from "react";
import {
  createSecretRef,
  deleteSecretRef,
  listSecretRefs,
  updateSecretRef,
  type CreateSecretRequest,
  type SecretRefRecord,
  type UpdateSecretRequest,
} from "../lib/secrets";
import { normalizeError } from "../lib/errors";

export function useSecretReferences() {
  const [secrets, setSecrets] = useState<SecretRefRecord[]>([]);
  const [loading, setLoading] = useState(false);
  const [saving, setSaving] = useState(false);
  const [rotating, setRotating] = useState(false);
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
      setError(normalizeError(err));
    } finally {
      setLoading(false);
    }
  }

  async function createSecret(payload: CreateSecretRequest) {
    setSaving(true);
    setError(null);
    try {
      const created = await createSecretRef(payload);
      setSecrets((prev) => [created, ...prev]);
      return created;
    } catch (err) {
      setError(normalizeError(err));
      return null;
    } finally {
      setSaving(false);
    }
  }

  async function removeSecret(id: string) {
    setError(null);
    try {
      await deleteSecretRef(id);
      setSecrets((prev) => prev.filter((s) => s.id !== id));
      return true;
    } catch (err) {
      setError(normalizeError(err));
      return false;
    }
  }

  async function rotateSecret(payload: UpdateSecretRequest) {
    setRotating(true);
    setError(null);
    try {
      const updated = await updateSecretRef(payload);
      setSecrets((prev) =>
        prev.map((s) => (s.id === updated.id ? updated : s)),
      );
      return updated;
    } catch (err) {
      setError(normalizeError(err));
      return null;
    } finally {
      setRotating(false);
    }
  }

  return {
    secrets,
    loading,
    saving,
    rotating,
    error,
    refresh,
    createSecret,
    removeSecret,
    rotateSecret,
  };
}
