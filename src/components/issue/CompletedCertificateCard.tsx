import { CheckCircle2, Copy } from "lucide-react";
import { useMemo, useState } from "react";
import type { CertificateRecord } from "../../lib/certificates";
import { Button } from "../ui/button";
import { CertificateExportModal } from "../certificates/CertificateExportModal";
import { DetailItem } from "../certificates/DetailItem";
import { SubjectPill } from "../certificates/SubjectPill";
import { formatCertificateDate, primarySubject } from "../certificates/certificate-utils";

interface CompletedCertificateCardProps {
  certificate: CertificateRecord;
}

function formatKeyInfo(record: CertificateRecord): string {
  if (!record.key_algorithm) {
    return "Unknown";
  }
  if (record.key_algorithm === "rsa") {
    return `RSA ${record.key_size ?? "?"}`;
  }
  if (record.key_curve === "p256") {
    return "ECDSA P-256";
  }
  if (record.key_curve === "p384") {
    return "ECDSA P-384";
  }
  return "ECDSA";
}

export function CompletedCertificateCard({
  certificate,
}: CompletedCertificateCardProps) {
  const [isExportOpen, setIsExportOpen] = useState(false);
  const [copied, setCopied] = useState(false);
  const canExport = certificate.source === "Managed";
  const summary = useMemo(() => {
    const primary = primarySubject(certificate);
    const sans = certificate.sans.length
      ? certificate.sans.join(", ")
      : "None";
    const expiry = formatCertificateDate(certificate.not_after);
    const keyType = formatKeyInfo(certificate);
    return [
      `CN: ${primary}`,
      `SANs: ${sans}`,
      `Expiry: ${expiry}`,
      `Key type: ${keyType}`,
    ].join("\n");
  }, [certificate]);

  async function handleCopy() {
    try {
      await navigator.clipboard.writeText(summary);
      setCopied(true);
      window.setTimeout(() => setCopied(false), 1500);
    } catch {
      setCopied(false);
    }
  }

  return (
    <div className="rounded-xl border bg-card p-5 shadow-soft">
      <div className="flex flex-wrap items-center gap-3">
        <CheckCircle2 className="h-4 w-4 text-emerald-600" />
        <div className="text-sm font-semibold text-muted-foreground">
          Certificate issued
        </div>
        <div className="ml-auto flex flex-wrap gap-2">
          <Button size="sm" variant="outline" onClick={() => void handleCopy()}>
            <Copy className="mr-2 h-4 w-4" />
            Copy details
          </Button>
          {canExport && (
            <Button size="sm" onClick={() => setIsExportOpen(true)}>
              Export...
            </Button>
          )}
        </div>
      </div>

      <div className="mt-4">
        <div className="text-xs uppercase tracking-wide text-muted-foreground">
          Common name
        </div>
        <div className="text-lg font-semibold text-foreground">
          {primarySubject(certificate)}
        </div>
      </div>

      <div className="mt-4 space-y-2 rounded-lg border bg-muted/40 p-3">
        <div className="text-xs font-semibold uppercase tracking-wide text-muted-foreground">
          Subject Alternative Names
        </div>
        <div className="flex flex-wrap gap-2">
          {certificate.sans.length ? (
            certificate.sans.map((name) => <SubjectPill key={name} text={name} />)
          ) : (
            <span className="text-xs text-muted-foreground">No SANs</span>
          )}
        </div>
      </div>

      <div className="mt-4 grid gap-3 sm:grid-cols-2">
        <DetailItem label="Expiry" value={formatCertificateDate(certificate.not_after)} />
        <DetailItem label="Key type" value={formatKeyInfo(certificate)} />
      </div>

      {copied && (
        <div className="mt-3 text-xs font-semibold text-emerald-600">
          Copied certificate details to clipboard.
        </div>
      )}

      {canExport && (
        <CertificateExportModal
          certificate={certificate}
          isOpen={isExportOpen}
          onClose={() => setIsExportOpen(false)}
        />
      )}
    </div>
  );
}
