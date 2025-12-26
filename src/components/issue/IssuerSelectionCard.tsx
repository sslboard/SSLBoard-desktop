import type { IssuerConfig } from "../../lib/issuers";
import { Card, CardContent, CardHeader, CardTitle } from "../ui/card";
import { Label } from "../ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "../ui/select";

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
    <Card className="shadow-soft">
      <CardHeader>
        <CardTitle className="text-sm font-semibold">Issuer selection</CardTitle>
        <p className="text-xs text-muted-foreground">
          Choose the issuer for this issuance run.
        </p>
      </CardHeader>

      {issuerError ? (
        <div className="mx-6 mt-3 rounded-lg border border-destructive/50 bg-destructive/10 px-4 py-3 text-sm text-destructive">
          {issuerError}
        </div>
      ) : null}

      <CardContent className="space-y-2">
        <Label>Issuer</Label>
        <Select
          value={selectedIssuer?.issuer_id ?? undefined}
          onValueChange={onSelectIssuer}
          disabled={issuerLoading}
        >
          <SelectTrigger>
            <SelectValue placeholder="Select issuer" />
          </SelectTrigger>
          <SelectContent>
            {issuers.map((issuer) => (
              <SelectItem key={issuer.issuer_id} value={issuer.issuer_id}>
                {issuer.label}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        <p className="text-xs text-muted-foreground">
          {selectedIssuer?.directory_url ?? "https://acme-staging-v02.api.letsencrypt.org/directory"}
        </p>
        {!issuerReady ? (
          <p className="text-xs text-muted-foreground">
            Configure the issuer&apos;s ACME account in Settings before issuing.
          </p>
        ) : null}
      </CardContent>
    </Card>
  );
}
