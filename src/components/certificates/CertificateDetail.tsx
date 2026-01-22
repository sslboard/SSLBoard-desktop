import { AlertCircle, Clock } from "lucide-react";
import { useState } from "react";
import { useNavigate } from "react-router-dom";
import {
  REVOCATION_REASONS,
  revokeCertificate,
  type CertificateRecord,
  type RevocationReason,
} from "../../lib/certificates";
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
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "../ui/select";
import { CertificateExportModal } from "./CertificateExportModal";
import { DetailItem } from "./DetailItem";
import { SubjectPill } from "./SubjectPill";
import { formatCertificateDate, primarySubject } from "./certificate-utils";
import { buildRenewalPrefill } from "./renewal-utils";

interface CertificateDetailProps {
  selected: CertificateRecord | null;
  loading: boolean;
  error: string | null;
  onRecordUpdated?: (record: CertificateRecord) => void;
  onSelectCertificate?: (id: string) => void;
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

export function CertificateDetail({
  selected,
  loading,
  error,
  onRecordUpdated,
  onSelectCertificate,
}: CertificateDetailProps) {
  const [isExportOpen, setIsExportOpen] = useState(false);
  const [isRevokeOpen, setIsRevokeOpen] = useState(false);
  const [revocationReason, setRevocationReason] =
    useState<RevocationReason>("unspecified");
  const [revocationError, setRevocationError] = useState<string | null>(null);
  const [revoking, setRevoking] = useState(false);
  const navigate = useNavigate();

  const canRevoke = Boolean(
    selected &&
      selected.source === "Managed" &&
      selected.issuer_id &&
      !selected.revoked_at,
  );
  if (selected) {
    console.debug("[certificates] revoke button state", {
      id: selected.id,
      source: selected.source,
      issuer_id: selected.issuer_id,
      revoked_at: selected.revoked_at,
      canRevoke,
    });
  }

  async function handleConfirmRevoke() {
    if (!selected || revoking) return;
    setRevocationError(null);
    setRevoking(true);
    try {
      const updated = await revokeCertificate({
        certificateId: selected.id,
        revocationReason,
      });
      onRecordUpdated?.(updated);
      setIsRevokeOpen(false);
    } catch (err) {
      setRevocationError(
        err instanceof Error ? err.message : "Failed to revoke certificate",
      );
    } finally {
      setRevoking(false);
    }
  }

  const revocationReasonLabel = selected?.revocation_reason
    ? REVOCATION_REASONS.find(
        (entry) => entry.value === selected.revocation_reason,
      )?.label ?? selected.revocation_reason
    : null;

  const canRenew = Boolean(
    selected && (selected.sans.length > 0 || selected.subjects.length > 0),
  );

  return (
    <div className="rounded-xl border bg-card p-4 shadow-soft">
      <div className="flex items-center gap-2 border-b pb-3">
        <Clock className="h-4 w-4 text-primary" />
        <div className="text-sm font-semibold text-muted-foreground">Details</div>
        {selected?.revoked_at ? (
          <span className="rounded-full bg-red-100 px-2 py-1 text-xs font-semibold text-red-700">
            Revoked
          </span>
        ) : null}
        {selected ? (
          <div className="ml-auto flex items-center gap-2">
            {canRenew ? (
              <Button
                size="sm"
                variant="outline"
                onClick={() => {
                  const renewal = buildRenewalPrefill(selected);
                  navigate("/issue", { state: { renewal } });
                }}
              >
                Renew
              </Button>
            ) : null}
            {selected.source === "Managed" ? (
              <>
                {canRevoke ? (
                  <Button
                    size="sm"
                    variant="destructive"
                    className="bg-red-500/80 text-white hover:bg-red-500/90"
                    onClick={() => {
                      setRevocationReason("unspecified");
                      setRevocationError(null);
                      setIsRevokeOpen(true);
                    }}
                  >
                    Revoke
                  </Button>
                ) : null}
                <Button size="sm" variant="outline" onClick={() => setIsExportOpen(true)}>
                  Export...
                </Button>
              </>
            ) : null}
          </div>
        ) : null}
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
              {selected.revoked_at ? (
                <DetailItem
                  label="Revoked"
                  value={formatCertificateDate(selected.revoked_at)}
                />
              ) : null}
              {selected.revocation_reason ? (
                <DetailItem
                  label="Revocation reason"
                  value={revocationReasonLabel ?? selected.revocation_reason}
                />
              ) : null}
              <DetailItem
                label="Fingerprint (SHA-256)"
                value={selected.fingerprint}
                truncate={true}
              />
              <DetailItem
                label="Domain roots"
                value={selected.domain_roots.join(", ")}
              />
              <DetailItem label="Key algorithm" value={formatKeyInfo(selected)} />
              <DetailItem label="Source" value={selected.source} />
            </div>
            {selected.renewed_from ? (
              <div className="rounded-lg border bg-muted/40 p-3 text-sm">
                <div className="text-xs uppercase tracking-wide text-muted-foreground">
                  Renewed from
                </div>
                <div className="mt-2 flex items-center justify-between gap-3">
                  <div className="font-semibold text-foreground">
                    {selected.renewed_from}
                  </div>
                  {onSelectCertificate ? (
                    <Button
                      size="sm"
                      variant="ghost"
                      onClick={() => onSelectCertificate(selected.renewed_from!)}
                    >
                      View previous
                    </Button>
                  ) : null}
                </div>
              </div>
            ) : null}
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
          {selected.source === "Managed" ? (
            <Dialog open={isRevokeOpen} onOpenChange={setIsRevokeOpen}>
              <DialogContent className="max-w-md">
                <DialogHeader>
                  <DialogTitle>Revoke certificate</DialogTitle>
                  <DialogDescription>
                    This action permanently revokes the certificate at the issuer.
                    Make sure you&apos;ve replaced it everywhere before proceeding.
                  </DialogDescription>
                </DialogHeader>

                {revocationError ? (
                  <div className="rounded-lg border border-destructive/50 bg-destructive/10 px-3 py-2 text-xs text-destructive">
                    {revocationError}
                  </div>
                ) : null}

                <div className="space-y-2">
                  <div className="text-xs font-semibold uppercase tracking-wide text-muted-foreground">
                    Revocation reason
                  </div>
                  <Select
                    value={revocationReason}
                    onValueChange={(value) =>
                      setRevocationReason(value as RevocationReason)
                    }
                  >
                    <SelectTrigger>
                      <SelectValue placeholder="Select reason" />
                    </SelectTrigger>
                    <SelectContent>
                      {REVOCATION_REASONS.map((reason) => (
                        <SelectItem key={reason.value} value={reason.value}>
                          {reason.label}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>

                <DialogFooter className="gap-3 sm:gap-2">
                  <DialogClose asChild>
                    <Button type="button" variant="ghost" disabled={revoking}>
                      Cancel
                    </Button>
                  </DialogClose>
                  <Button
                    type="button"
                    variant="destructive"
                    className="bg-red-500/80 text-white hover:bg-red-500/90"
                    onClick={handleConfirmRevoke}
                    disabled={revoking}
                  >
                    {revoking ? "Revoking..." : "Confirm revoke"}
                  </Button>
                </DialogFooter>
              </DialogContent>
            </Dialog>
          ) : null}
        </>
      ) : (
        <div className="py-6 text-sm text-muted-foreground">
          Select a certificate to inspect metadata.
        </div>
      )}
    </div>
  );
}
