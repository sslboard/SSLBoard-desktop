import { invoke } from "@tauri-apps/api/core";
import type { PropagationResult } from "./dns";

export type DnsProviderType =
  | "cloudflare"
  | "digitalocean"
  | "route53"
  | "manual";

export type DnsProviderRecord = {
  id: string;
  provider_type: DnsProviderType;
  label: string;
  domain_suffixes: string[];
  config?: Record<string, unknown> | null;
  created_at: string;
  updated_at: string;
};

export type DnsProviderResolution = {
  provider?: DnsProviderRecord | null;
  matched_suffix?: string | null;
  ambiguous: DnsProviderRecord[];
};

export type CreateDnsProviderRequest = {
  provider_type: DnsProviderType;
  label: string;
  domain_suffixes: string;
  api_token?: string;
  config?: Record<string, unknown> | null;
};

export type UpdateDnsProviderRequest = {
  provider_id: string;
  label: string;
  domain_suffixes: string;
  api_token?: string;
  config?: Record<string, unknown> | null;
};

export type DnsProviderTestResult = {
  success: boolean;
  record_name?: string | null;
  value?: string | null;
  propagation?: PropagationResult | null;
  error?: string | null;
  error_stage?: string | null;
  elapsed_ms: number;
  create_ms?: number | null;
  propagation_ms?: number | null;
  cleanup_ms?: number | null;
};

export async function listDnsProviders(): Promise<DnsProviderRecord[]> {
  return invoke("dns_provider_list");
}

export async function createDnsProvider(
  req: CreateDnsProviderRequest,
): Promise<DnsProviderRecord> {
  return invoke("dns_provider_create", { req });
}

export async function updateDnsProvider(
  req: UpdateDnsProviderRequest,
): Promise<DnsProviderRecord> {
  return invoke("dns_provider_update", { req });
}

export async function deleteDnsProvider(providerId: string): Promise<string> {
  return invoke("dns_provider_delete", { req: { provider_id: providerId } });
}

export async function testDnsProvider(
  providerId: string,
): Promise<DnsProviderTestResult> {
  return invoke("dns_provider_test", { req: { provider_id: providerId } });
}

export async function resolveDnsProvider(
  hostname: string,
): Promise<DnsProviderResolution> {
  return invoke("dns_resolve_provider", { req: { hostname } });
}
