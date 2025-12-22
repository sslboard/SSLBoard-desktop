import type { DnsProviderType } from "../../lib/dns-providers";

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
