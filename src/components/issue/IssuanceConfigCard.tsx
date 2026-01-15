import type { IssuerConfig } from "../../lib/issuers";
import { Card, CardContent } from "../ui/card";
import { Label } from "../ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "../ui/select";
import { Tabs, TabsList, TabsTrigger } from "../ui/tabs";
import type { IssuanceMode } from "./IssuanceModeCard";

interface IssuanceConfigCardProps {
  // Issuer selection props
  issuers: IssuerConfig[];
  selectedIssuer: IssuerConfig | null;
  issuerLoading: boolean;
  issuerError: string | null;
  issuerReady: boolean;
  onSelectIssuer: (issuerId: string) => void;
  
  // Issuance mode props
  issuanceMode: IssuanceMode;
  onModeChange: (mode: IssuanceMode) => void;
}

export function IssuanceConfigCard({
  issuers,
  selectedIssuer,
  issuerLoading,
  issuerError,
  issuerReady,
  onSelectIssuer,
  issuanceMode,
  onModeChange,
}: IssuanceConfigCardProps) {
  return (
    <Card className="shadow-soft">
      <CardContent className="p-4">
        <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
          {/* Issuer Selection */}
          <div className="space-y-2">
            <Label className="text-xs">Issuer</Label>
            {issuerError ? (
              <div className="rounded-lg border border-destructive/50 bg-destructive/10 px-3 py-2 text-xs text-destructive">
                {issuerError}
              </div>
            ) : null}
            <Select
              value={selectedIssuer?.issuer_id ?? undefined}
              onValueChange={onSelectIssuer}
              disabled={issuerLoading}
            >
              <SelectTrigger className="h-9">
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
            {selectedIssuer && (
              <p className="text-[11px] text-muted-foreground truncate">
                {selectedIssuer.directory_url}
              </p>
            )}
            {!issuerReady && selectedIssuer && (
              <p className="text-[11px] text-muted-foreground">
                Configure ACME account in Settings
              </p>
            )}
          </div>

          {/* Issuance Mode */}
          <div className="space-y-2">
            <Label className="text-xs">Issuance Mode</Label>
            <Tabs value={issuanceMode} onValueChange={(value) => onModeChange(value as IssuanceMode)}>
              <TabsList className="grid w-full grid-cols-3 h-9">
                <TabsTrigger value="dns" className="text-xs">DNS names</TabsTrigger>
                <TabsTrigger value="csr-import" className="text-xs">CSR import</TabsTrigger>
                <TabsTrigger value="csr-generate" className="text-xs">CSR creation</TabsTrigger>
              </TabsList>
            </Tabs>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
