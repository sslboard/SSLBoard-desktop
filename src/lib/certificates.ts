import { invoke } from "@tauri-apps/api/core";

export type CertificateSource = "External" | "Managed";
export type KeyAlgorithm = "rsa" | "ecdsa";
export type KeyCurve = "p256" | "p384";
export type CsrSource = "imported" | "generated";
export type RevocationReason =
  | "keyCompromise"
  | "superseded"
  | "cessationOfOperation"
  | "unspecified";

export const REVOCATION_REASONS: { value: RevocationReason; label: string }[] = [
  { value: "keyCompromise", label: "Key compromise" },
  { value: "superseded", label: "Superseded" },
  { value: "cessationOfOperation", label: "Cessation of operation" },
  { value: "unspecified", label: "Unspecified" },
];

export type CertificateRecord = {
  id: string;
  subjects: string[];
  sans: string[];
  issuer: string;
  serial: string;
  not_before: string;
  not_after: string;
  fingerprint: string;
  source: CertificateSource;
  issuer_id?: string | null;
  domain_roots: string[];
  tags: string[];
  managed_key_ref?: string | null;
  chain_pem?: string | null;
  key_algorithm?: KeyAlgorithm | null;
  key_size?: number | null;
  key_curve?: KeyCurve | null;
  csr_subject?: string | null;
  csr_sans?: string[] | null;
  csr_key_algorithm?: KeyAlgorithm | null;
  csr_key_size?: number | null;
  csr_key_curve?: KeyCurve | null;
  csr_source?: CsrSource | null;
  renewed_from?: string | null;
  revoked_at?: string | null;
  revocation_reason?: RevocationReason | string | null;
};

export type ExportBundle = "cert" | "chain" | "fullchain";

export type ExportCertificateRequest = {
  certificateId: string;
  destinationDir: string;
  folderName: string;
  includePrivateKey: boolean;
  bundle: ExportBundle;
  overwrite: boolean;
};

export type ExportedFile = {
  label: string;
  path: string;
};

export type ExportCertificateResponse =
  | {
      status: "success";
      output_dir: string;
      files: ExportedFile[];
    }
  | {
      status: "overwrite_required";
      output_dir: string;
      existing_files: string[];
    };

export type RevokeCertificateRequest = {
  certificateId: string;
  revocationReason?: RevocationReason | null;
};

export async function listCertificates(): Promise<CertificateRecord[]> {
  return invoke<CertificateRecord[]>("list_certificates");
}

export async function getCertificate(
  id: string,
): Promise<CertificateRecord> {
  return invoke<CertificateRecord>("get_certificate", { id });
}

export async function exportCertificatePem(
  exportReq: ExportCertificateRequest,
): Promise<ExportCertificateResponse> {
  return invoke<ExportCertificateResponse>("export_certificate_pem", {
    exportReq: {
      certificate_id: exportReq.certificateId,
      destination_dir: exportReq.destinationDir,
      folder_name: exportReq.folderName,
      include_private_key: exportReq.includePrivateKey,
      bundle: exportReq.bundle,
      overwrite: exportReq.overwrite,
    },
  });
}

export async function revokeCertificate(
  revokeReq: RevokeCertificateRequest,
): Promise<CertificateRecord> {
  return invoke<CertificateRecord>("revoke_certificate", {
    revokeReq: {
      certificate_id: revokeReq.certificateId,
      revocation_reason: revokeReq.revocationReason ?? null,
    },
  });
}
