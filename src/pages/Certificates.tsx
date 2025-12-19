import {
  AlertCircle,
  ArrowRight,
  Compass,
  Download,
  ShieldCheck,
  Sparkles,
} from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { Link, useNavigate } from "react-router-dom";
import { CertificateDetail } from "../components/certificates/CertificateDetail";
import { Inventory } from "../components/certificates/Inventory";
import { PageHeader } from "../components/page-header";
import { Button } from "../components/ui/button";
import {
  getCertificate,
  listCertificates,
  seedFakeCertificate,
  type CertificateRecord,
} from "../lib/certificates";

function daysUntil(dateString: string) {
  const now = Date.now();
  const target = new Date(dateString).getTime();
  return Math.round((target - now) / (1000 * 60 * 60 * 24));
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

          <div className="grid gap-4 lg:grid-cols-[1fr_1.5fr]">
            <Inventory
              records={records}
              selectedId={selectedId}
              onSelect={setSelectedId}
              loading={loadingList}
              onRefresh={refreshList}
            />
            <CertificateDetail
              selected={selected}
              loading={loadingDetail}
              error={detailError}
            />
          </div>
        </>
      )}
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
