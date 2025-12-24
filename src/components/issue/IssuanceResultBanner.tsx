import { AlertTriangle, CheckCircle2 } from "lucide-react";

interface IssuanceResultBannerProps {
  error: string | null;
  successMessage: string | null;
}

export function IssuanceResultBanner({ error, successMessage }: IssuanceResultBannerProps) {
  if (error) {
    return (
      <div className="flex items-center gap-2 rounded-md bg-destructive/10 px-3 py-2 text-sm text-destructive">
        <AlertTriangle className="h-4 w-4" />
        {error}
      </div>
    );
  }

  if (successMessage) {
    return (
      <div className="flex items-center gap-2 rounded-md bg-emerald-50 px-3 py-2 text-sm text-emerald-700">
        <CheckCircle2 className="h-4 w-4" />
        {successMessage}
      </div>
    );
  }

  return null;
}

