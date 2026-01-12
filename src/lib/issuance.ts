import { invoke } from "@tauri-apps/api/core";
import type { CertificateRecord } from "./certificates";

export type KeyAlgorithm = "rsa" | "ecdsa";
export type KeyCurve = "p256" | "p384";

export type IssuanceKeyOption =
  | "rsa-2048"
  | "rsa-3072"
  | "rsa-4096"
  | "ecdsa-p256"
  | "ecdsa-p384";

export type CsrSource = "imported" | "generated";

export type CsrMetadata = {
  subject: string;
  sans: string[];
  key_algorithm: KeyAlgorithm;
  key_size?: number | null;
  key_curve?: KeyCurve | null;
};

export type CsrValidationResult = {
  metadata: CsrMetadata;
  identifiers: string[];
  warnings: string[];
};

export type StartIssuanceRequest = {
  domains: string[];
  issuer_id: string;
  key_algorithm?: KeyAlgorithm;
  key_size?: number;
  key_curve?: KeyCurve;
};

export type StartIssuanceResponse = {
  request_id: string;
  dns_records: Array<{
    adapter: string;
    record_name: string;
    value: string;
    zone: string;
  }>;
};

export type CompleteIssuanceRequest = {
  request_id: string;
};

export type InspectCsrRequest = {
  csr_path: string;
};

export type GenerateCsrRequest = {
  subject: string;
  sans: string[];
  key_algorithm?: KeyAlgorithm;
  key_size?: number;
  key_curve?: KeyCurve;
  output_path: string;
};

export type GenerateCsrResponse = {
  csr_path: string;
  managed_key_ref: string;
  result: CsrValidationResult;
};

export type StartCsrIssuanceRequest = {
  issuer_id: string;
  csr_path: string;
  csr_source: CsrSource;
  managed_key_ref?: string | null;
};

export type StartCsrIssuanceResponse = {
  request_id: string;
  dns_records: StartIssuanceResponse["dns_records"];
  csr_result: CsrValidationResult;
};

export type CompleteCsrIssuanceRequest = {
  request_id: string;
};

export async function startManagedIssuance(
  req: StartIssuanceRequest,
): Promise<StartIssuanceResponse> {
  return invoke<StartIssuanceResponse>("start_managed_issuance", {
    startReq: req,
  });
}

export async function completeManagedIssuance(
  req: CompleteIssuanceRequest,
): Promise<CertificateRecord> {
  return invoke<CertificateRecord>("complete_managed_issuance", {
    completeReq: req,
  });
}

export async function inspectCsr(
  req: InspectCsrRequest,
): Promise<CsrValidationResult> {
  return invoke<CsrValidationResult>("inspect_csr", {
    inspectReq: req,
  });
}

export async function generateCsr(
  req: GenerateCsrRequest,
): Promise<GenerateCsrResponse> {
  return invoke<GenerateCsrResponse>("generate_csr", {
    generateReq: req,
  });
}

export async function startCsrIssuance(
  req: StartCsrIssuanceRequest,
): Promise<StartCsrIssuanceResponse> {
  return invoke<StartCsrIssuanceResponse>("start_csr_issuance", {
    startReq: req,
  });
}

export async function completeCsrIssuance(
  req: CompleteCsrIssuanceRequest,
): Promise<CertificateRecord> {
  return invoke<CertificateRecord>("complete_csr_issuance", {
    completeReq: req,
  });
}

export function keyOptionToParams(option: IssuanceKeyOption): {
  key_algorithm: KeyAlgorithm;
  key_size?: number;
  key_curve?: KeyCurve;
} {
  switch (option) {
    case "rsa-2048":
      return { key_algorithm: "rsa", key_size: 2048 };
    case "rsa-3072":
      return { key_algorithm: "rsa", key_size: 3072 };
    case "rsa-4096":
      return { key_algorithm: "rsa", key_size: 4096 };
    case "ecdsa-p256":
      return { key_algorithm: "ecdsa", key_curve: "p256" };
    case "ecdsa-p384":
      return { key_algorithm: "ecdsa", key_curve: "p384" };
  }
}
