import { AlertCircle, Compass, ShieldCheck } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { Link, useNavigate } from "react-router-dom";
import { CertificateDetail } from "../components/certificates/CertificateDetail";
import { CertificatesEmptyState } from "../components/certificates/CertificatesEmptyState";
import { CertificateStats } from "../components/certificates/CertificateStats";
import { Inventory } from "../components/certificates/Inventory";
import { PageHeader } from "../components/page-header";
import { Button } from "../components/ui/button";
import { daysUntil } from "../components/certificates/certificate-utils";
import {
  getCertificate,
  listCertificates,
  type CertificateRecord,
} from "../lib/certificates";

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
        <CertificatesEmptyState
          onIssue={() => navigate("/issue")}
          onDiscover={() => navigate("/discover")}
        />
      ) : (
        <>
          <CertificateStats
            managedCount={managedCount}
            externalCount={externalCount}
            expiringSoon={expiringSoon}
          />

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
