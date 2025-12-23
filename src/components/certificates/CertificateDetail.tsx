import { AlertCircle, Clock } from "lucide-react";
import { useState } from "react";
import type { CertificateRecord } from "../../lib/certificates";
import { Button } from "../ui/button";
import { CertificateExportModal } from "./CertificateExportModal";
import { DetailItem } from "./DetailItem";
import { SubjectPill } from "./SubjectPill";
import { formatCertificateDate, primarySubject } from "./certificate-utils";

interface CertificateDetailProps {
  selected: CertificateRecord | null;
  loading: boolean;
  error: string | null;
}

export function CertificateDetail({
  selected,
  loading,
  error,
}: CertificateDetailProps) {
  const [isExportOpen, setIsExportOpen] = useState(false);
  return (
    <div className="rounded-xl border bg-card p-4 shadow-soft">
      <div className="flex items-center gap-2 border-b pb-3">
        <Clock className="h-4 w-4 text-primary" />
        <div className="text-sm font-semibold text-muted-foreground">Details</div>
        {selected?.source === "Managed" && (
          <div className="ml-auto">
            <Button size="sm" variant="outline" onClick={() => setIsExportOpen(true)}>
              Export...
            </Button>
          </div>
        )}
      </div>
      {loading ? (
        <div className="py-6 text-sm text-muted-foreground">Loading...</div>
      ) : error ? (
        <div className="flex items-center gap-2 py-4 text-sm text-red-500">
          <AlertCircle className="h-4 w-4" />
          {error}
        </div>
      ) : selected ? (
        <>
          <div className="space-y-4 pt-4">
            <div>
              <div className="text-xs uppercase tracking-wide text-muted-foreground">
                Primary subject
              </div>
              <div className="text-lg font-semibold text-foreground">
                {primarySubject(selected)}
              </div>
            </div>
            <div className="space-y-2 rounded-lg border bg-muted/40 p-3">
              <div className="text-xs font-semibold uppercase tracking-wide text-muted-foreground">
                Subject Alternative Names
              </div>
              <div className="flex flex-wrap gap-2">
                {selected.sans.map((name) => (
                  <SubjectPill key={name} text={name} />
                ))}
              </div>
            </div>
            <div className="grid gap-3 sm:grid-cols-2">
              <DetailItem label="Issuer" value={selected.issuer} />
              <DetailItem label="Serial" value={selected.serial} truncate={true} />
              <DetailItem
                label="Validity"
                value={`${formatCertificateDate(selected.not_before)} – ${formatCertificateDate(selected.not_after)}`}
              />
              <DetailItem
                label="Fingerprint (SHA-256)"
                value={selected.fingerprint}
                truncate={true}
              />
              <DetailItem
                label="Domain roots"
                value={selected.domain_roots.join(", ")}
              />
              <DetailItem label="Source" value={selected.source} />
            </div>
            <div>
              <div className="text-xs font-semibold uppercase tracking-wide text-muted-foreground">
                Tags
              </div>
              <div className="mt-2 flex flex-wrap gap-2">
                {selected.tags.length ? (
                  selected.tags.map((tag) => (
                    <span
                      key={tag}
                      className="rounded-full bg-primary/10 px-3 py-1 text-xs font-semibold text-primary"
                    >
                      {tag}
                    </span>
                  ))
                ) : (
                  <span className="text-xs text-muted-foreground">No tags</span>
                )}
              </div>
            </div>
          </div>
          {selected.source === "Managed" && (
            <CertificateExportModal
              certificate={selected}
              isOpen={isExportOpen}
              onClose={() => setIsExportOpen(false)}
            />
          )}
        </>
      ) : (
        <div className="py-6 text-sm text-muted-foreground">
          Select a certificate to inspect metadata.
        </div>
      )}
    </div>
  );
}
