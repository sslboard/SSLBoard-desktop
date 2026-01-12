import { Card, CardContent, CardHeader, CardTitle } from "../ui/card";
import { Tabs, TabsList, TabsTrigger } from "../ui/tabs";

export type IssuanceMode = "dns" | "csr-import" | "csr-generate";

interface IssuanceModeCardProps {
  mode: IssuanceMode;
  onModeChange: (mode: IssuanceMode) => void;
}

export function IssuanceModeCard({
  mode,
  onModeChange,
}: IssuanceModeCardProps) {
  return (
    <Card className="shadow-soft">
      <CardHeader>
        <div className="text-sm font-semibold text-muted-foreground">
          Issuance mode
        </div>
        <CardTitle className="text-xl font-bold">
          Choose how to issue
        </CardTitle>
      </CardHeader>
      <CardContent>
        <Tabs value={mode} onValueChange={(value) => onModeChange(value as IssuanceMode)}>
          <TabsList className="grid w-full grid-cols-3">
            <TabsTrigger value="dns">DNS names</TabsTrigger>
            <TabsTrigger value="csr-import">CSR import</TabsTrigger>
            <TabsTrigger value="csr-generate">CSR creation</TabsTrigger>
          </TabsList>
        </Tabs>
      </CardContent>
    </Card>
  );
}
