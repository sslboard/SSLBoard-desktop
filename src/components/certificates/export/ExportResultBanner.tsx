interface ExportResultBannerProps {
  error: string | null;
  successPath: string | null;
}

export function ExportResultBanner({ error, successPath }: ExportResultBannerProps) {
  if (error) {
    return (
      <div className="rounded-lg border border-red-200 bg-red-50 p-3 text-sm text-red-700">
        {error}
      </div>
    );
  }

  if (successPath) {
    return (
      <div className="rounded-lg border border-emerald-200 bg-emerald-50 p-3 text-sm text-emerald-700">
        Exported to {successPath}
      </div>
    );
  }

  return null;
}

