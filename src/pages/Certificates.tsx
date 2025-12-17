import {
  AlertCircle,
  ArrowRight,
  Clock,
  Compass,
  Download,
  RefreshCw,
  ShieldCheck,
  Sparkles,
} from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { Link, useNavigate } from "react-router-dom";
import { PageHeader } from "../components/page-header";
import { Button } from "../components/ui/button";
import {
  getCertificate,
  listCertificates,
  seedFakeCertificate,
  type CertificateRecord,
} from "../lib/certificates";

function formatDate(dateString: string) {
  const date = new Date(dateString);
  return date.toLocaleDateString(undefined, {
    month: "short",
    day: "numeric",
    year: "numeric",
  });
}

function daysUntil(dateString: string) {
  const now = Date.now();
  const target = new Date(dateString).getTime();
  return Math.round((target - now) / (1000 * 60 * 60 * 24));
}

function certificateStatus(record: CertificateRecord) {
  const days = daysUntil(record.not_after);
  if (days < 0) {
    return { label: "Expired", tone: "text-red-500 bg-red-50 dark:bg-red-950/40" };
  }
  if (days < 30) {
    return {
      label: `Expiring in ${days}d`,
      tone: "text-amber-500 bg-amber-50 dark:bg-amber-950/40",
    };
  }
  return {
    label: `Healthy · ${days}d left`,
    tone: "text-emerald-600 bg-emerald-50 dark:bg-emerald-950/40",
  };
}

function SubjectPill({ text }: { text: string }) {
  return (
    <span className="rounded-full bg-muted px-3 py-1 text-xs font-semibold text-foreground">
      {text}
    </span>
  );
}

export function CertificatesPage() {
  const [records, setRecords] = useState<CertificateRecord[]>([]);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [selected, setSelected] = useState<CertificateRecord | null>(null);
  const [loadingList, setLoadingList] = useState(true);
  const [loadingDetail, setLoadingDetail] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [detailError, setDetailError] = useState<string | null>(null);
  const navigate = useNavigate();

  const managedCount = useMemo(
    () => records.filter((r) => r.source === "Managed").length,
    [records],
  );
  const externalCount = useMemo(
    () => records.filter((r) => r.source === "External").length,
    [records],
  );
  const expiringSoon = useMemo(
    () => records.filter((r) => daysUntil(r.not_after) < 30).length,
    [records],
  );

  useEffect(() => {
    refreshList();
  }, []);

  useEffect(() => {
    if (!selectedId) {
      setSelected(null);
      setDetailError(null);
      return;
    }

    const currentId = selectedId;
    let cancelled = false;
    async function loadDetail() {
      setLoadingDetail(true);
      setDetailError(null);
      try {
        const detail = await getCertificate(currentId);
        if (!cancelled) {
          setSelected(detail);
        }
      } catch (err) {
        if (!cancelled) {
          const message =
            err instanceof Error ? err.message : "Failed to load certificate";
          setDetailError(message);
        }
      } finally {
        if (!cancelled) setLoadingDetail(false);
      }
    }

    loadDetail();
    return () => {
      cancelled = true;
    };
  }, [selectedId]);

  async function refreshList() {
    setLoadingList(true);
    setError(null);
    try {
      const result = await listCertificates();
      setRecords(result);
      setSelectedId((prev) => {
        if (!result.length) return null;
        if (prev && result.some((r) => r.id === prev)) return prev;
        return result[0].id;
      });
    } catch (err) {
      const message =
        err instanceof Error ? err.message : "Failed to load certificates";
      setError(message);
    } finally {
      setLoadingList(false);
    }
  }

  async function handleSeed() {
    setLoadingList(true);
    try {
      await seedFakeCertificate();
      await refreshList();
    } catch (err) {
      const message =
        err instanceof Error ? err.message : "Unable to insert sample data";
      setError(message);
      setLoadingList(false);
    }
  }

  const primarySubject = (record: CertificateRecord | null) =>
    record?.subjects[0] ?? record?.sans[0] ?? record?.domain_roots[0] ?? "—";

  const renderEmptyState = () => (
    <div className="rounded-xl border bg-gradient-to-br from-primary/5 via-card to-secondary/10 p-8 shadow-soft">
      <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
        <div className="space-y-2">
          <div className="flex items-center gap-2 text-sm font-semibold uppercase tracking-wide text-primary">
            <Sparkles className="h-4 w-4" />
            No certificates yet
          </div>
          <h3 className="text-2xl font-bold text-foreground">
            Inventory is empty — start by importing, discovering, or issuing.
          </h3>
          <p className="max-w-2xl text-sm text-muted-foreground">
            Bring existing certificates into view, or kick off a new issuance.
            The UI stays unprivileged: metadata only, no secrets.
          </p>
          <div className="flex flex-wrap gap-3">
            <Button onClick={() => navigate("/issue")}>
              <ShieldCheck className="mr-2 h-4 w-4" />
              Issue a certificate
            </Button>
            <Button
              variant="secondary"
              onClick={() => navigate("/discover")}
            >
              <Compass className="mr-2 h-4 w-4" />
              Discover via CT
            </Button>
            <Button variant="outline" onClick={handleSeed}>
              <Download className="mr-2 h-4 w-4" />
              Add demo record
            </Button>
          </div>
        </div>
        <div className="hidden rounded-xl border bg-card/80 p-4 text-sm text-muted-foreground shadow-sm sm:block sm:max-w-xs">
          <div className="flex items-center gap-2 text-foreground">
            <ShieldCheck className="h-4 w-4 text-primary" />
            Metadata-only inventory
          </div>
          <p className="mt-2">
            Stored fields: SANs, issuer, serial, validity window, fingerprint,
            source, domain roots, and tags. No private keys, no secrets.
          </p>
        </div>
      </div>
    </div>
  );

  const renderList = () => (
    <div className="rounded-xl border bg-card p-4 shadow-soft">
      <div className="flex items-center justify-between gap-3 border-b pb-3">
        <div>
          <div className="text-sm font-semibold text-muted-foreground">
            Inventory
          </div>
          <div className="text-lg font-bold text-foreground">
            {records.length} certificate{records.length === 1 ? "" : "s"}
          </div>
        </div>
        <div className="flex items-center gap-2">
          <Button variant="ghost" size="sm" onClick={refreshList}>
            <RefreshCw className="mr-2 h-4 w-4" />
            Refresh
          </Button>
          <Button asChild size="sm">
            <Link to="/issue">
              <ShieldCheck className="mr-2 h-4 w-4" />
              Issue new
            </Link>
          </Button>
        </div>
      </div>
      <div className="mt-3 divide-y">
        {records.map((record) => {
          const status = certificateStatus(record);
          const isSelected = selectedId === record.id;
          return (
            <button
              key={record.id}
              onClick={() => setSelectedId(record.id)}
              className={`flex w-full items-start gap-4 rounded-lg px-3 py-3 text-left transition ${
                isSelected ? "bg-primary/5 ring-1 ring-primary" : "hover:bg-muted/60"
              }`}
            >
              <div className="flex h-10 w-10 flex-shrink-0 items-center justify-center rounded-lg bg-primary/10 text-primary">
                <ShieldCheck className="h-5 w-5" />
              </div>
              <div className="flex flex-1 flex-col gap-2">
                <div className="flex flex-wrap items-center gap-2">
                  <span className="text-sm font-semibold text-foreground">
                    {primarySubject(record)}
                  </span>
                  <span
                    className={`rounded-full px-2 py-1 text-xs font-semibold ${status.tone}`}
                  >
                    {status.label}
                  </span>
                  <span className="rounded-full bg-muted px-2 py-1 text-xs">
                    {record.source}
                  </span>
                </div>
                <div className="text-xs text-muted-foreground">
                  Issuer · {record.issuer} — Serial {record.serial}
                </div>
                <div className="text-xs text-muted-foreground">
                  Valid {formatDate(record.not_before)} –{" "}
                  {formatDate(record.not_after)}
                </div>
              </div>
            </button>
          );
        })}
      </div>
      {records.length === 0 ? (
        <div className="rounded-lg border border-dashed bg-muted/40 p-4 text-sm text-muted-foreground">
          No certificates yet.
        </div>
      ) : null}
    </div>
  );

  const renderDetails = () => (
    <div className="rounded-xl border bg-card p-4 shadow-soft">
      <div className="flex items-center gap-2 border-b pb-3">
        <Clock className="h-4 w-4 text-primary" />
        <div className="text-sm font-semibold text-muted-foreground">
          Details
        </div>
      </div>
      {loadingDetail ? (
        <div className="py-6 text-sm text-muted-foreground">Loading...</div>
      ) : detailError ? (
        <div className="flex items-center gap-2 py-4 text-sm text-red-500">
          <AlertCircle className="h-4 w-4" />
          {detailError}
        </div>
      ) : selected ? (
        <div className="space-y-4 pt-4">
          <div>
            <div className="text-xs uppercase tracking-wide text-muted-foreground">
              Primary subject
            </div>
            <div className="text-lg font-semibold text-foreground">
              {primarySubject(selected)}
            </div>
          </div>
          <div className="space-y-2 rounded-lg border bg-muted/40 p-3">
            <div className="text-xs font-semibold uppercase tracking-wide text-muted-foreground">
              Subject Alternative Names
            </div>
            <div className="flex flex-wrap gap-2">
              {selected.sans.map((name) => (
                <SubjectPill key={name} text={name} />
              ))}
            </div>
          </div>
          <div className="grid gap-3 sm:grid-cols-2">
            <DetailItem label="Issuer" value={selected.issuer} />
            <DetailItem label="Serial" value={selected.serial} />
            <DetailItem
              label="Validity"
              value={`${formatDate(selected.not_before)} – ${formatDate(selected.not_after)}`}
            />
            <DetailItem
              label="Fingerprint (SHA-256)"
              value={selected.fingerprint}
            />
            <DetailItem
              label="Domain roots"
              value={selected.domain_roots.join(", ")}
            />
            <DetailItem label="Source" value={selected.source} />
          </div>
          <div>
            <div className="text-xs font-semibold uppercase tracking-wide text-muted-foreground">
              Tags
            </div>
            <div className="mt-2 flex flex-wrap gap-2">
              {selected.tags.length ? (
                selected.tags.map((tag) => (
                  <span
                    key={tag}
                    className="rounded-full bg-primary/10 px-3 py-1 text-xs font-semibold text-primary"
                  >
                    {tag}
                  </span>
                ))
              ) : (
                <span className="text-xs text-muted-foreground">No tags</span>
              )}
            </div>
          </div>
        </div>
      ) : (
        <div className="py-6 text-sm text-muted-foreground">
          Select a certificate to inspect metadata.
        </div>
      )}
    </div>
  );

  return (
    <div className="space-y-6">
      <PageHeader
        title="Certificates"
        description="Metadata-first inventory for issued and discovered certificates."
        action={
          <div className="flex gap-2">
            <Button asChild variant="outline">
              <Link to="/discover">
                <Compass className="mr-2 h-4 w-4" />
                Discover
              </Link>
            </Button>
            <Button asChild>
              <Link to="/issue">
                <ShieldCheck className="mr-2 h-4 w-4" />
                Issue
              </Link>
            </Button>
          </div>
        }
      />

      {error ? (
        <div className="flex items-center gap-2 rounded-lg border border-red-200 bg-red-50 p-3 text-sm text-red-600 dark:border-red-900 dark:bg-red-950/30">
          <AlertCircle className="h-4 w-4" />
          {error}
        </div>
      ) : null}

      {records.length === 0 && !loadingList ? (
        renderEmptyState()
      ) : (
        <>
          <div className="grid gap-4 sm:grid-cols-3">
            <StatCard
              title="Managed"
              value={managedCount}
              description="Issued by this app"
            />
            <StatCard
              title="External"
              value={externalCount}
              description="Imported or discovered"
            />
            <StatCard
              title="Expiring soon"
              value={expiringSoon}
              description="< 30 days to expiry"
              tone="warning"
            />
          </div>

          <div className="grid gap-4 lg:grid-cols-[1.2fr_1fr]">
            {loadingList ? (
              <div className="rounded-xl border bg-card p-6 text-sm text-muted-foreground shadow-soft">
                Loading inventory...
              </div>
            ) : (
              renderList()
            )}
            {renderDetails()}
          </div>
        </>
      )}
    </div>
  );
}

function DetailItem({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-lg border bg-muted/40 p-3">
      <div className="text-xs uppercase tracking-wide text-muted-foreground">
        {label}
      </div>
      <div className="text-sm font-semibold text-foreground">{value}</div>
    </div>
  );
}

function StatCard({
  title,
  value,
  description,
  tone = "default",
}: {
  title: string;
  value: number;
  description: string;
  tone?: "default" | "warning";
}) {
  const toneClass =
    tone === "warning"
      ? "text-amber-600 bg-amber-50 dark:bg-amber-950/30"
      : "text-primary bg-primary/10";

  return (
    <div className="rounded-xl border bg-card p-4 shadow-soft">
      <div className="flex items-center justify-between">
        <div>
          <div className="text-sm font-semibold text-muted-foreground">
            {title}
          </div>
          <div className="text-2xl font-bold text-foreground">{value}</div>
          <div className="text-xs text-muted-foreground">{description}</div>
        </div>
        <div
          className={`flex h-10 w-10 items-center justify-center rounded-lg text-sm font-semibold ${toneClass}`}
        >
          <ArrowRight className="h-4 w-4" />
        </div>
      </div>
    </div>
  );
}
