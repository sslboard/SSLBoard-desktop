import type { CertificateRecord } from "../../lib/certificates";
import type { IssuanceKeyOption } from "../../lib/issuance";
import { primarySubject } from "./certificate-utils";

export type RenewalPrefill = {
  domains: string[];
  issuer_id?: string | null;
  issuer_hint?: string | null;
  key_ref?: string | null;
  renewing_cert_id: string;
  key_option?: IssuanceKeyOption | null;
  label?: string | null;
};

function keyOptionFromRecord(record: CertificateRecord): IssuanceKeyOption | null {
  if (record.key_algorithm === "rsa") {
    switch (record.key_size) {
      case 3072:
        return "rsa-3072";
      case 4096:
        return "rsa-4096";
      default:
        return "rsa-2048";
    }
  }
  if (record.key_algorithm === "ecdsa") {
    if (record.key_curve === "p384") {
      return "ecdsa-p384";
    }
    return "ecdsa-p256";
  }
  return null;
}

export function buildRenewalPrefill(record: CertificateRecord): RenewalPrefill {
  const domains =
    record.sans.length > 0 ? record.sans : record.subjects.length > 0 ? record.subjects : [];
  return {
    domains,
    issuer_id: record.issuer_id ?? null,
    issuer_hint: record.issuer ?? null,
    key_ref: record.managed_key_ref ?? null,
    renewing_cert_id: record.id,
    key_option: keyOptionFromRecord(record),
    label: primarySubject(record),
  };
}
