import { useEffect, useState } from "react";
import {
  resolveDnsProvider,
  type DnsProviderResolution,
} from "../lib/dns-providers";
import { normalizeError } from "../lib/errors";

type PreviewMap = Record<string, DnsProviderResolution | null>;

export function useProviderPreview(domains: string[]) {
  const [providerPreview, setProviderPreview] = useState<PreviewMap>({});
  const [providerLoading, setProviderLoading] = useState(false);
  const [providerError, setProviderError] = useState<string | null>(null);

  useEffect(() => {
    let active = true;
    if (!domains.length) {
      setProviderPreview({});
      setProviderError(null);
      setProviderLoading(false);
      return undefined;
    }
    setProviderLoading(true);
    setProviderError(null);
    const timer = window.setTimeout(() => {
      Promise.all(
        domains.map(async (domain) => {
          try {
            const resolution = await resolveDnsProvider(domain);
            return { domain, resolution };
          } catch (err) {
            return { domain, error: normalizeError(err) };
          }
        }),
      )
        .then((results) => {
          if (!active) return;
          const next: PreviewMap = {};
          let errorMessage: string | null = null;
          results.forEach((item) => {
            if ("error" in item) {
              next[item.domain] = null;
              errorMessage = item.error;
            } else {
              next[item.domain] = item.resolution;
            }
          });
          setProviderPreview(next);
          setProviderError(errorMessage);
        })
        .catch((err) => {
          if (active) {
            setProviderError(normalizeError(err));
          }
        })
        .finally(() => {
          if (active) {
            setProviderLoading(false);
          }
        });
    }, 300);

    return () => {
      active = false;
      window.clearTimeout(timer);
    };
  }, [domains.join("|")]);

  return {
    providerPreview,
    providerLoading,
    providerError,
  };
}
