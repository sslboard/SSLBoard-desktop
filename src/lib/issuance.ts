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
