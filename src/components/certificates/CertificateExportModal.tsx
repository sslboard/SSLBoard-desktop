import { useEffect, useMemo, useState } from "react";

import type { CertificateRecord, ExportBundle } from "../../lib/certificates";
import { exportCertificatePem } from "../../lib/certificates";
import { Button } from "../ui/button";
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "../ui/dialog";
import { exportFolderDefault } from "./certificate-utils";
import { useExportDestination } from "../../hooks/useExportDestination";
import { ExportBundleSelector } from "./export/ExportBundleSelector";
import { ExportDestinationPicker } from "./export/ExportDestinationPicker";
import { PrivateKeyExportWarning } from "./export/PrivateKeyExportWarning";
import { ExportResultBanner } from "./export/ExportResultBanner";

type CertificateExportModalProps = {
  certificate: CertificateRecord;
  isOpen: boolean;
  onClose: () => void;
};

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
  const [folderName, setFolderName] = useState(defaultFolder);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [exportError, setExportError] = useState<string | null>(null);
  const [successPath, setSuccessPath] = useState<string | null>(null);

  const { destinationDir, error: destinationError, selectDestination, persistDestination } =
    useExportDestination(isOpen);

  useEffect(() => {
    if (!isOpen) {
      return;
    }
    setBundle("fullchain");
    setIncludeKey(false);
    setConfirmKeyExport(false);
    setFolderName(defaultFolder);
    setExportError(null);
    setSuccessPath(null);
  }, [defaultFolder, isOpen]);

  if (!isOpen) {
    return null;
  }

  async function handleExport(overwrite = false) {
    if (!destinationDir) {
      setExportError("Select an export folder to continue.");
      return;
    }
    if (!folderName.trim()) {
      setExportError("Enter a folder name for the export.");
      return;
    }
    if (includeKey && !confirmKeyExport) {
      setExportError("Confirm the private key warning before exporting.");
      return;
    }
    setIsSubmitting(true);
    setExportError(null);
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
        const existingNames = response.existing_files.map((path) => {
          const parts = path.split(/[\\/]/);
          return parts[parts.length - 1] ?? path;
        });
        const confirmed = window.confirm(
          `One or more files already exist:\n${existingNames.join("\n")}\n\nOverwrite these files?`,
        );
        if (confirmed) {
          await handleExport(true);
        } else {
          setExportError("Export cancelled before overwriting existing files.");
        }
        return;
      }

      if (destinationDir) {
        await persistDestination(destinationDir);
      }
      setSuccessPath(response.output_dir);
    } catch (err) {
      const message =
        err instanceof Error ? err.message : "Failed to export certificate.";
      setExportError(message);
    } finally {
      setIsSubmitting(false);
    }
  }

  return (
    <Dialog
      open={isOpen}
      onOpenChange={(open) => {
        if (!open) {
          onClose();
        }
      }}
    >
      <DialogContent className="max-w-xl">
        <DialogHeader>
          <DialogTitle>Export certificate</DialogTitle>
          <DialogDescription>
            Export PEM files for {certificate.sans[0] ?? certificate.id}
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-5">
          <ExportBundleSelector bundle={bundle} onBundleChange={setBundle} />

          <ExportDestinationPicker
            destinationDir={destinationDir}
            folderName={folderName}
            onSelectDestination={selectDestination}
            onFolderNameChange={setFolderName}
          />

          <PrivateKeyExportWarning
            keyAvailable={keyAvailable}
            includeKey={includeKey}
            confirmKeyExport={confirmKeyExport}
            onIncludeKeyChange={setIncludeKey}
            onConfirmKeyExportChange={setConfirmKeyExport}
          />

          {destinationError && (
            <div className="rounded-lg border border-red-200 bg-red-50 p-3 text-sm text-red-700">
              {destinationError}
            </div>
          )}

          <ExportResultBanner error={exportError} successPath={successPath} />
        </div>

        <DialogFooter className="gap-3 sm:gap-2">
          <DialogClose asChild>
            <Button variant="ghost">Done</Button>
          </DialogClose>
          <Button
            onClick={() => handleExport(false)}
            disabled={isSubmitting || (includeKey && !confirmKeyExport)}
          >
            {isSubmitting ? "Exporting..." : "Export"}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
