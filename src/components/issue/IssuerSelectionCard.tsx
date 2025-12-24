import type { IssuerConfig } from "../../lib/issuers";

interface IssuerSelectionCardProps {
  issuers: IssuerConfig[];
  selectedIssuer: IssuerConfig | null;
  issuerLoading: boolean;
  issuerError: string | null;
  issuerReady: boolean;
  onSelectIssuer: (issuerId: string) => void;
}

export function IssuerSelectionCard({
  issuers,
  selectedIssuer,
  issuerLoading,
  issuerError,
  issuerReady,
  onSelectIssuer,
}: IssuerSelectionCardProps) {
  return (
    <div className="rounded-xl border bg-card p-6 shadow-soft">
      <div className="flex items-center justify-between gap-3">
        <div>
          <div className="text-sm font-semibold">Issuer selection</div>
          <p className="text-xs text-muted-foreground">
            Choose the issuer for this issuance run.
          </p>
        </div>
      </div>

      {issuerError ? (
        <div className="mt-3 rounded-lg border border-destructive/50 bg-destructive/10 px-4 py-3 text-sm text-destructive">
          {issuerError}
        </div>
      ) : null}

      <div className="mt-4 space-y-2">
        <label className="text-sm font-medium text-foreground">
          Issuer
        </label>
        <select
          className="w-full rounded-lg border bg-background/60 p-2.5 text-sm shadow-inner outline-none ring-offset-background focus:ring-2 focus:ring-primary/50"
          value={selectedIssuer?.issuer_id ?? ""}
          onChange={(e) => onSelectIssuer(e.target.value)}
          disabled={issuerLoading}
        >
          {issuers.map((issuer) => (
            <option key={issuer.issuer_id} value={issuer.issuer_id}>
              {issuer.label}
            </option>
          ))}
        </select>
        <p className="text-xs text-muted-foreground">
          {selectedIssuer?.directory_url ?? "https://acme-staging-v02.api.letsencrypt.org/directory"}
        </p>
        {!issuerReady ? (
          <p className="text-xs text-muted-foreground">
            Configure the issuer&apos;s ACME account in Settings before issuing.
          </p>
        ) : null}
      </div>
    </div>
  );
}

