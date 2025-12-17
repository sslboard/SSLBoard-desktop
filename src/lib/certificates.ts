import { invoke } from "@tauri-apps/api/core";

export type CertificateSource = "External" | "Managed";

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
  domain_roots: string[];
  tags: string[];
};

export async function listCertificates(): Promise<CertificateRecord[]> {
  return invoke<CertificateRecord[]>("list_certificates");
}

export async function getCertificate(
  id: string,
): Promise<CertificateRecord> {
  return invoke<CertificateRecord>("get_certificate", { id });
}

export async function seedFakeCertificate(): Promise<void> {
  return invoke("seed_fake_certificate");
}
