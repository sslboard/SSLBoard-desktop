export function normalizeError(err: unknown, fallback = "Unexpected error") {
  if (err instanceof Error) return err.message;
  if (typeof err === "string") return err;
  return fallback;
}
