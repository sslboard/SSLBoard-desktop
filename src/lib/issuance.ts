import { invoke } from "@tauri-apps/api/core";
import type { CertificateRecord } from "./certificates";

export type StartIssuanceRequest = {
  domains: string[];
  issuer_id: string;
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
  return invoke<StartIssuanceResponse>("start_managed_issuance", { req });
}

export async function completeManagedIssuance(
  req: CompleteIssuanceRequest,
): Promise<CertificateRecord> {
  return invoke<CertificateRecord>("complete_managed_issuance", { req });
}
