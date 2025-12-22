import { useMemo } from "react";
import {
  AlertTriangle,
  CheckCircle2,
  RefreshCw,
  Trash2,
  XCircle,
} from "lucide-react";
import { Button } from "../ui/button";
import type { DnsProviderRecord, DnsProviderTestResult } from "../../lib/dns-providers";
import { cn } from "../../lib/utils";
import { PROVIDER_LABELS } from "./provider-constants";

function formatDate(iso: string) {
  const date = new Date(iso);
  return Number.isNaN(date.getTime())
    ? "-"
    : date.toLocaleString(undefined, {
        month: "short",
        day: "numeric",
        year: "numeric",
        hour: "2-digit",
        minute: "2-digit",
      });
}

export function DnsProviderList({
  providers,
  loading,
  onRefresh,
  onEdit,
  onDelete,
  confirmDeleteId,
  onConfirmDelete,
  onCancelDelete,
  testResults,
  testLoading,
  onTest,
}: {
  providers: DnsProviderRecord[];
  loading: boolean;
  onRefresh: () => void;
  onEdit: (provider: DnsProviderRecord) => void;
  onDelete: (providerId: string) => void;
  confirmDeleteId: string | null;
  onConfirmDelete: (providerId: string) => void;
  onCancelDelete: () => void;
  testResults: Record<string, DnsProviderTestResult | null>;
  testLoading: Record<string, boolean>;
  onTest: (providerId: string) => void;
}) {
  const overlapSuffixes = useMemo(() => {
    const counts = new Map<string, number>();
    providers.forEach((provider) => {
      provider.domain_suffixes.forEach((suffix) => {
        counts.set(suffix, (counts.get(suffix) ?? 0) + 1);
      });
    });
    return new Set(
      Array.from(counts.entries())
        .filter(([, count]) => count > 1)
        .map(([suffix]) => suffix),
    );
  }, [providers]);

  return (
    <div className="rounded-xl border bg-card p-5 shadow-soft">
      <div className="flex items-center justify-between gap-3">
        <div>
          <div className="text-sm font-semibold">Configured providers</div>
          <p className="text-xs text-muted-foreground">
            Providers are matched by domain suffix; longest suffix wins.
          </p>
        </div>
        <Button
          variant="ghost"
          size="sm"
          onClick={onRefresh}
          disabled={loading}
          className="gap-2"
        >
          <RefreshCw className={cn("h-4 w-4", loading && "animate-spin")} />
          Refresh
        </Button>
      </div>

      <div className="mt-4 space-y-3">
        {loading ? (
          <div className="rounded-lg border border-dashed border-muted-foreground/30 bg-muted/60 p-4 text-sm text-muted-foreground">
            Loading DNS providers...
          </div>
        ) : null}
        {!loading && providers.length === 0 ? (
          <div className="rounded-lg border border-dashed border-muted-foreground/30 bg-muted/60 p-5 text-sm text-muted-foreground">
            No DNS providers configured yet.
          </div>
        ) : null}
        {providers.map((provider) => {
          const overlaps = provider.domain_suffixes.filter((suffix) =>
            overlapSuffixes.has(suffix),
          );
          const testResult = testResults[provider.id];
          const testInFlight = testLoading[provider.id] ?? false;
          return (
            <div
              key={provider.id}
              className="rounded-lg border bg-background/80 p-4 shadow-sm"
            >
              <div className="flex flex-wrap items-start justify-between gap-3">
                <div>
                  <div className="flex flex-wrap items-center gap-2 text-sm font-semibold">
                    {provider.label}
                    <span className="rounded-full bg-primary/10 px-2 py-0.5 text-[11px] font-semibold uppercase text-primary">
                      {PROVIDER_LABELS[provider.provider_type]}
                    </span>
                  </div>
                  <div className="mt-1 text-xs text-muted-foreground">
                    Domains: {provider.domain_suffixes.join(", ") || "-"}
                  </div>
                  <div className="mt-1 text-xs text-muted-foreground">
                    Updated {formatDate(provider.updated_at)}
                  </div>
                  {overlaps.length ? (
                    <div className="mt-2 flex items-center gap-2 text-xs text-amber-700">
                      <AlertTriangle className="h-3.5 w-3.5" />
                      Overlapping suffixes: {overlaps.join(", ")}
                    </div>
                  ) : null}
                  {testResult ? (
                    <div className="mt-2 flex items-center gap-2 text-xs">
                      {testResult.success ? (
                        <CheckCircle2 className="h-3.5 w-3.5 text-emerald-600" />
                      ) : (
                        <XCircle className="h-3.5 w-3.5 text-rose-600" />
                      )}
                      <span
                        className={
                          testResult.success
                            ? "text-emerald-700"
                            : "text-rose-700"
                        }
                      >
                        {testResult.success
                          ? "Connection verified"
                          : testResult.error || "Connection failed"}
                      </span>
                    </div>
                  ) : null}
                </div>
                <div className="flex flex-wrap gap-2">
                  <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    onClick={() => onTest(provider.id)}
                    disabled={testInFlight}
                  >
                    {testInFlight ? (
                      <RefreshCw className="mr-2 h-4 w-4 animate-spin" />
                    ) : null}
                    Test connection
                  </Button>
                  <Button
                    type="button"
                    variant="ghost"
                    size="sm"
                    onClick={() => onEdit(provider)}
                  >
                    Edit
                  </Button>
                  <Button
                    type="button"
                    variant="ghost"
                    size="sm"
                    className="text-destructive hover:bg-destructive/10"
                    onClick={() => onDelete(provider.id)}
                  >
                    <Trash2 className="mr-1 h-4 w-4" />
                    Remove
                  </Button>
                </div>
              </div>
              {confirmDeleteId === provider.id ? (
                <div className="mt-3 flex flex-wrap items-center gap-2 rounded-md border border-destructive/30 bg-destructive/5 px-3 py-2 text-xs text-destructive">
                  <span className="font-semibold">
                    Remove this provider and its stored token?
                  </span>
                  <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    className="border-destructive/40 text-destructive hover:bg-destructive/10"
                    onClick={() => onConfirmDelete(provider.id)}
                  >
                    Confirm remove
                  </Button>
                  <Button
                    type="button"
                    variant="ghost"
                    size="sm"
                    onClick={onCancelDelete}
                  >
                    Cancel
                  </Button>
                </div>
              ) : null}
            </div>
          );
        })}
      </div>
    </div>
  );
}
