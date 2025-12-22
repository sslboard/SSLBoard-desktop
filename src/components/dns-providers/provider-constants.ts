import type {
  DnsProviderErrorCategory,
  DnsProviderType,
} from "../../lib/dns-providers";

export const PROVIDER_LABELS: Record<DnsProviderType, string> = {
  cloudflare: "Cloudflare",
  digitalocean: "DigitalOcean",
  route53: "Route 53",
  manual: "Manual",
};

export const PROVIDER_OPTIONS: { value: DnsProviderType; label: string }[] = [
  { value: "cloudflare", label: "Cloudflare" },
  { value: "digitalocean", label: "DigitalOcean" },
  { value: "route53", label: "Route 53" },
  { value: "manual", label: "Manual" },
];

export const ERROR_CATEGORY_LABELS: Record<DnsProviderErrorCategory, string> = {
  auth_error: "Authentication error",
  not_found: "Not found",
  rate_limited: "Rate limited",
  network_error: "Network error",
  unknown: "Unknown error",
};

export const ERROR_CATEGORY_SUGGESTIONS: Record<DnsProviderErrorCategory, string> = {
  auth_error: "Check the API token or access keys and verify permissions.",
  not_found: "Confirm the zone or domain exists in the provider account.",
  rate_limited: "Wait and retry; reduce request frequency if possible.",
  network_error: "Check network connectivity and DNS resolution.",
  unknown: "Review logs and try again.",
};
