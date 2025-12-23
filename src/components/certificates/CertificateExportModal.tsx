import { open } from "@tauri-apps/plugin-dialog";
import { useEffect, useMemo, useState } from "react";

import type { CertificateRecord, ExportBundle } from "../../lib/certificates";
import { exportCertificatePem } from "../../lib/certificates";
import { Button } from "../ui/button";
import { exportFolderDefault } from "./certificate-utils";

type CertificateExportModalProps = {
  certificate: CertificateRecord;
  isOpen: boolean;
  onClose: () => void;
};

const bundleOptions: { value: ExportBundle; label: string; hint: string }[] = [
  { value: "cert", label: "Certificate", hint: "Leaf certificate only" },
  { value: "chain", label: "Chain", hint: "Issuer chain only" },
  { value: "fullchain", label: "Full chain", hint: "Leaf + issuer chain" },
];

export function CertificateExportModal({
  certificate,
  isOpen,
  onClose,
}: CertificateExportModalProps) {
  const keyAvailable = Boolean(certificate.managed_key_ref);
  const defaultFolder = useMemo(
    () => exportFolderDefault(certificate),
    [certificate],
  );

  const [bundle, setBundle] = useState<ExportBundle>("fullchain");
  const [includeKey, setIncludeKey] = useState(false);
  const [confirmKeyExport, setConfirmKeyExport] = useState(false);
  const [destinationDir, setDestinationDir] = useState<string | null>(null);
  const [folderName, setFolderName] = useState(defaultFolder);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [successPath, setSuccessPath] = useState<string | null>(null);

  useEffect(() => {
    if (!isOpen) {
      return;
    }
    setBundle("fullchain");
    setIncludeKey(false);
    setConfirmKeyExport(false);
    setDestinationDir(null);
    setFolderName(defaultFolder);
    setError(null);
    setSuccessPath(null);
  }, [defaultFolder, isOpen]);

  if (!isOpen) {
    return null;
  }

  async function handleSelectDestination() {
    setError(null);
    const selection = await open({ directory: true, multiple: false });
    if (typeof selection === "string") {
      setDestinationDir(selection);
    } else if (Array.isArray(selection) && selection[0]) {
      setDestinationDir(selection[0]);
    }
  }

  async function handleExport(overwrite = false) {
    if (!destinationDir) {
      setError("Select an export folder to continue.");
      return;
    }
    if (!folderName.trim()) {
      setError("Enter a folder name for the export.");
      return;
    }
    if (includeKey && !confirmKeyExport) {
      setError("Confirm the private key warning before exporting.");
      return;
    }
    setIsSubmitting(true);
    setError(null);
    try {
      const response = await exportCertificatePem({
        certificateId: certificate.id,
        destinationDir,
        folderName: folderName.trim(),
        includePrivateKey: includeKey,
        bundle,
        overwrite,
      });

      if (response.status === "overwrite_required") {
        const confirmed = window.confirm(
          `One or more files already exist:\n${response.existing_files.join("\n")}\n\nOverwrite these files?`,
        );
        if (confirmed) {
          await handleExport(true);
        } else {
          setError("Export cancelled before overwriting existing files.");
        }
        return;
      }

      setSuccessPath(response.output_dir);
    } catch (err) {
      const message =
        err instanceof Error ? err.message : "Failed to export certificate.";
      setError(message);
    } finally {
      setIsSubmitting(false);
    }
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 p-4">
      <div className="w-full max-w-xl rounded-xl border bg-card p-6 shadow-xl">
        <div className="flex items-start justify-between gap-4">
          <div>
            <div className="text-lg font-semibold text-foreground">
              Export certificate
            </div>
            <div className="text-sm text-muted-foreground">
              Export PEM files for {certificate.sans[0] ?? certificate.id}
            </div>
          </div>
          <Button variant="ghost" size="sm" onClick={onClose}>
            Close
          </Button>
        </div>

        <div className="mt-6 space-y-5">
          <div>
            <div className="text-xs font-semibold uppercase tracking-wide text-muted-foreground">
              Bundle selection
            </div>
            <div className="mt-2 grid gap-2 sm:grid-cols-3">
              {bundleOptions.map((option) => (
                <button
                  key={option.value}
                  type="button"
                  onClick={() => setBundle(option.value)}
                  className={`rounded-lg border px-3 py-2 text-left text-sm ${
                    bundle === option.value
                      ? "border-primary bg-primary/10 text-primary"
                      : "border-border bg-background text-foreground hover:bg-muted/50"
                  }`}
                >
                  <div className="font-semibold">{option.label}</div>
                  <div className="text-xs text-muted-foreground">
                    {option.hint}
                  </div>
                </button>
              ))}
            </div>
          </div>

          <div className="rounded-lg border bg-muted/40 p-4">
            <div className="text-xs font-semibold uppercase tracking-wide text-muted-foreground">
              Destination
            </div>
            <div className="mt-3 flex flex-col gap-3 sm:flex-row sm:items-center">
              <Button variant="outline" onClick={handleSelectDestination}>
                Choose folder
              </Button>
              <div className="text-sm text-muted-foreground">
                {destinationDir ?? "No folder selected"}
              </div>
            </div>
            <div className="mt-3">
              <label className="text-xs font-semibold uppercase tracking-wide text-muted-foreground">
                Export subfolder name
              </label>
              <input
                value={folderName}
                onChange={(event) => setFolderName(event.target.value)}
                className="mt-2 w-full rounded-md border border-border bg-background px-3 py-2 text-sm text-foreground"
              />
            </div>
          </div>

          <div className="rounded-lg border border-amber-200/60 bg-amber-50/70 p-4 text-sm text-amber-900">
            <div className="font-semibold">Private key export</div>
            <div className="mt-2 text-xs text-amber-800/80">
              Exporting private keys can expose sensitive material. Only proceed
              if you trust the destination.
            </div>
            <div className="mt-3 flex items-start gap-2">
              <input
                id="include-key"
                type="checkbox"
                className="mt-1 h-4 w-4"
                disabled={!keyAvailable}
                checked={includeKey}
                onChange={(event) => {
                  setIncludeKey(event.target.checked);
                  if (!event.target.checked) {
                    setConfirmKeyExport(false);
                  }
                }}
              />
              <label htmlFor="include-key" className="text-sm font-semibold">
                Include private key (`privkey.pem`)
              </label>
            </div>
            {!keyAvailable && (
              <div className="mt-2 text-xs text-amber-700">
                Private key export is unavailable for certificates without a
                managed key reference.
              </div>
            )}
            {includeKey && (
              <div className="mt-3 flex items-start gap-2 rounded-md bg-amber-100/80 p-2">
                <input
                  id="confirm-key"
                  type="checkbox"
                  className="mt-1 h-4 w-4"
                  checked={confirmKeyExport}
                  onChange={(event) => setConfirmKeyExport(event.target.checked)}
                />
                <label htmlFor="confirm-key" className="text-xs text-amber-900">
                  I understand exporting private keys is sensitive and I accept
                  the risk.
                </label>
              </div>
            )}
          </div>

          {error && (
            <div className="rounded-lg border border-red-200 bg-red-50 p-3 text-sm text-red-700">
              {error}
            </div>
          )}
          {successPath && (
            <div className="rounded-lg border border-emerald-200 bg-emerald-50 p-3 text-sm text-emerald-700">
              Exported to {successPath}
            </div>
          )}
        </div>

        <div className="mt-6 flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-end">
          <Button variant="ghost" onClick={onClose}>
            Done
          </Button>
          <Button
            onClick={() => handleExport(false)}
            disabled={isSubmitting || (includeKey && !confirmKeyExport)}
          >
            {isSubmitting ? "Exporting..." : "Export"}
          </Button>
        </div>
      </div>
    </div>
  );
}
